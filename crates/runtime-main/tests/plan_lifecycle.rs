//! End-to-end Plan & Task lifecycle test — M04 Stage B.
//!
//! Drives a full 3-task plan flow against a real drone subprocess via
//! the `WriteSignal` IPC variant. Closes M03 🟡 (vdr + plan projector
//! wired at signal-write call-site).

#![cfg(any(unix, windows))]
#![allow(
    clippy::too_many_lines,
    reason = "linear end-to-end plan flows; splitting hides the drive sequence"
)]
#![allow(
    clippy::doc_markdown,
    reason = "free-text identifiers in module-level test docs"
)]

use std::time::Duration;

use runtime_main::drone_ipc::DroneClient;
use rusqlite::Connection;
use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;

fn drone_binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().expect("current_exe");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    #[cfg(windows)]
    p.push("runtime-drone.exe");
    #[cfg(unix)]
    p.push("runtime-drone");
    p
}

fn ensure_drone_built() {
    let bin = drone_binary();
    if !bin.exists() {
        let target_dir = bin.parent().expect("parent");
        let mut cmd = std::process::Command::new(env!("CARGO"));
        cmd.args(["build", "--bin", "runtime-drone"]);
        if std::env::var_os("CARGO_TARGET_DIR").is_none() {
            cmd.env(
                "CARGO_TARGET_DIR",
                target_dir.parent().expect("profile parent"),
            );
        }
        let status = cmd.status().expect("cargo build");
        assert!(status.success(), "drone build failed");
    }
    assert!(bin.exists(), "drone binary missing at {}", bin.display());
}

#[cfg(unix)]
fn make_socket(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("d.sock")
}

#[cfg(windows)]
fn make_socket(_dir: &std::path::Path) -> std::path::PathBuf {
    let suffix = uuid::Uuid::new_v4();
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-plan-lifecycle-{suffix}"))
}

fn socket_to_addr(p: &std::path::Path) -> String {
    p.to_string_lossy().into_owned()
}

fn spawn_drone(
    session: &str,
    db_path: &std::path::Path,
    socket: &std::path::Path,
) -> tokio::process::Child {
    let mut cmd = tokio::process::Command::new(drone_binary());
    cmd.arg("--session-id")
        .arg(session)
        .arg("--db-path")
        .arg(db_path)
        .arg("--ipc-socket")
        .arg(socket)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    cmd.spawn().expect("spawn drone")
}

async fn connect_with_retry(addr: &str) -> DroneClient {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match DroneClient::connect(addr).await {
            Ok(c) => return c,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => panic!("connect: {e}"),
        }
    }
}

/// Polls SQLite for the expected row state, retrying with a tight deadline.
/// The drone-side projector is asynchronous w.r.t. the IPC ack.
async fn poll_until<F: Fn(&Connection) -> bool>(
    db_path: &std::path::Path,
    predicate: F,
    label: &str,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Ok(conn) = Connection::open(db_path) {
            if predicate(&conn) {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("predicate never satisfied: {label}");
}

fn plan_status(conn: &Connection, plan_id: &str) -> Option<String> {
    conn.query_row("SELECT status FROM plans WHERE id = ?1", [plan_id], |r| {
        r.get::<_, String>(0)
    })
    .ok()
}

fn task_status(conn: &Connection, task_id: &str) -> Option<String> {
    conn.query_row("SELECT status FROM tasks WHERE id = ?1", [task_id], |r| {
        r.get::<_, String>(0)
    })
    .ok()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plan_lifecycle_three_task_happy_path() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("d.sqlite");
    let socket = make_socket(dir.path());
    let session_id = "plan-lifecycle-1";

    let mut child = spawn_drone(session_id, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    let plan_id = Uuid::new_v4().to_string();
    let task_ids: Vec<String> = (0..3).map(|_| Uuid::new_v4().to_string()).collect();

    // 1. plan_created — approval_required = true
    client
        .write_signal(
            Uuid::new_v4().to_string(),
            session_id.to_string(),
            "agent".to_string(),
            "plan_created".to_string(),
            "plan_create".to_string(),
            json!({
                "type": "plan_created",
                "plan_id": plan_id,
                "title": "Migrate auth flow",
                "task_count": 3,
                "approval_required": true,
            }),
        )
        .await
        .expect("write plan_created");

    let pid = plan_id.clone();
    poll_until(
        &db_path,
        move |c| plan_status(c, &pid).as_deref() == Some("pending_approval"),
        "plans.status='pending_approval'",
    )
    .await;

    // 2. plan_approved
    client
        .write_signal(
            Uuid::new_v4().to_string(),
            session_id.to_string(),
            "agent".to_string(),
            "plan_approved".to_string(),
            "plan_create".to_string(),
            json!({"type": "plan_approved", "plan_id": plan_id, "approved_by": "user"}),
        )
        .await
        .expect("write plan_approved");

    let pid = plan_id.clone();
    poll_until(
        &db_path,
        move |c| plan_status(c, &pid).as_deref() == Some("approved"),
        "plans.status='approved'",
    )
    .await;

    // 3. task_started × 3 + task_completed × 3
    for (i, tid) in task_ids.iter().enumerate() {
        client
            .write_signal(
                Uuid::new_v4().to_string(),
                session_id.to_string(),
                "agent".to_string(),
                "task_started".to_string(),
                "agent_loop".to_string(),
                json!({
                    "type": "task_started",
                    "plan_id": plan_id,
                    "task_id": tid,
                    "agent_id": "orchestrator",
                }),
            )
            .await
            .expect("write task_started");

        let task_id_clone = tid.clone();
        poll_until(
            &db_path,
            move |c| task_status(c, &task_id_clone).as_deref() == Some("running"),
            &format!("tasks[{i}].status='running'"),
        )
        .await;

        client
            .write_signal(
                Uuid::new_v4().to_string(),
                session_id.to_string(),
                "agent".to_string(),
                "task_completed".to_string(),
                "agent_loop".to_string(),
                json!({
                    "type": "task_completed",
                    "plan_id": plan_id,
                    "task_id": tid,
                    "duration_ms": 100 * (i as u64 + 1),
                }),
            )
            .await
            .expect("write task_completed");

        let task_id_clone = tid.clone();
        poll_until(
            &db_path,
            move |c| task_status(c, &task_id_clone).as_deref() == Some("done"),
            &format!("tasks[{i}].status='done'"),
        )
        .await;
    }

    // 4. plan_complete
    client
        .write_signal(
            Uuid::new_v4().to_string(),
            session_id.to_string(),
            "agent".to_string(),
            "plan_complete".to_string(),
            "plan_create".to_string(),
            json!({
                "type": "plan_complete",
                "plan_id": plan_id,
                "duration_ms": 500,
            }),
        )
        .await
        .expect("write plan_complete");

    let pid = plan_id.clone();
    poll_until(
        &db_path,
        move |c| plan_status(c, &pid).as_deref() == Some("complete"),
        "plans.status='complete'",
    )
    .await;

    // Final assertion: completed_at populated.
    let conn = Connection::open(&db_path).expect("reopen db");
    let completed_at: Option<i64> = conn
        .query_row(
            "SELECT completed_at FROM plans WHERE id = ?1",
            [&plan_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(completed_at.is_some());
    let task_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE plan_id = ?1 AND status = 'done'",
            [&plan_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(task_count, 3, "all three tasks must land status='done'");
    drop(client);
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plan_lifecycle_failure_escalation_variant() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("d.sqlite");
    let socket = make_socket(dir.path());
    let session_id = "plan-lifecycle-2";

    let mut child = spawn_drone(session_id, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    let plan_id = Uuid::new_v4().to_string();
    let task_id = Uuid::new_v4().to_string();

    // Auto-approved plan with one task.
    client
        .write_signal(
            Uuid::new_v4().to_string(),
            session_id.to_string(),
            "agent".to_string(),
            "plan_created".to_string(),
            "plan_create".to_string(),
            json!({
                "type": "plan_created",
                "plan_id": plan_id,
                "title": "Auto plan",
                "task_count": 1,
                "approval_required": false,
            }),
        )
        .await
        .expect("write plan_created");

    // 3 attempts: each starts then fails (max_failures=3 so 3rd failure escalates).
    for failure_count in 1..=3_u32 {
        client
            .write_signal(
                Uuid::new_v4().to_string(),
                session_id.to_string(),
                "agent".to_string(),
                "task_started".to_string(),
                "agent_loop".to_string(),
                json!({
                    "type": "task_started",
                    "plan_id": plan_id,
                    "task_id": task_id,
                    "agent_id": "orchestrator",
                }),
            )
            .await
            .expect("write task_started");

        client
            .write_signal(
                Uuid::new_v4().to_string(),
                session_id.to_string(),
                "agent".to_string(),
                "task_failed".to_string(),
                "agent_loop".to_string(),
                json!({
                    "type": "task_failed",
                    "plan_id": plan_id,
                    "task_id": task_id,
                    "error": format!("attempt {failure_count} failed"),
                    "failure_count": failure_count,
                }),
            )
            .await
            .expect("write task_failed");
    }

    // Emit task_escalated (the plan_loop / FSM emits this when
    // failure_count >= max_failures; here we drive it directly).
    client
        .write_signal(
            Uuid::new_v4().to_string(),
            session_id.to_string(),
            "agent".to_string(),
            "task_escalated".to_string(),
            "agent_loop".to_string(),
            json!({
                "type": "task_escalated",
                "plan_id": plan_id,
                "task_id": task_id,
                "failure_count": 3,
                "max_failures": 3,
            }),
        )
        .await
        .expect("write task_escalated");

    let tid_clone = task_id.clone();
    poll_until(
        &db_path,
        move |c| task_status(c, &tid_clone).as_deref() == Some("escalated"),
        "tasks.status='escalated'",
    )
    .await;

    let conn = Connection::open(&db_path).expect("reopen");
    let (fc, mx): (i64, i64) = conn
        .query_row(
            "SELECT failure_count, max_failures FROM tasks WHERE id = ?1",
            [&task_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(fc, 3);
    assert_eq!(mx, 3);

    drop(client);
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}
