//! Recovery semantics test — M04 Stage B + spec §1b.
//!
//! Drives a plan to mid-execution, kills the drone subprocess, restarts
//! it against the same DB, and verifies:
//!
//! 1. The projected `plans` + `tasks` rows survive (drone projection is
//!    durable — written inside the same transaction as the signal).
//! 2. Currently-running tasks (`status = 'running'`) are recovered via
//!    `snapshot::recover_session_state` as `pending` per spec §1b.
//! 3. `tool_call_uncertain` flag is set when `tool_invoked` has no
//!    matching `tool_result`.

#![cfg(any(unix, windows))]

use std::time::Duration;

use runtime_drone::snapshot;
use runtime_main::drone_ipc::DroneClient;
use rusqlite::Connection;
use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;

mod common;
use common::{drone_binary, ensure_drone_built};

#[cfg(unix)]
fn make_socket(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("d.sock")
}

#[cfg(windows)]
fn make_socket(_dir: &std::path::Path) -> std::path::PathBuf {
    let suffix = uuid::Uuid::new_v4();
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-plan-recovery-{suffix}"))
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn currently_running_task_is_recovered_as_pending() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("d.sqlite");
    let socket = make_socket(dir.path());
    let session_id = "plan-recovery-1";

    let mut child = spawn_drone(session_id, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    let plan_id = Uuid::new_v4().to_string();
    let task_id = Uuid::new_v4().to_string();

    // Drive plan + task to mid-execution: plan_created (auto-approved) +
    // task_started, but NOT task_completed.
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
                "title": "Recovery test",
                "task_count": 1,
                "approval_required": false,
            }),
        )
        .await
        .expect("write plan_created");

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

    // Verify the task landed status='running' before kill.
    let tid_clone = task_id.clone();
    poll_until(
        &db_path,
        move |c| {
            c.query_row(
                "SELECT status FROM tasks WHERE id = ?1",
                [&tid_clone],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .as_deref()
                == Some("running")
        },
        "tasks.status='running' pre-kill",
    )
    .await;

    // Kill the drone hard. drop the client first so the IPC socket
    // closes cleanly and the drone's reaper doesn't race.
    drop(client);
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;

    // Recovery: open the DB directly (NOT via a fresh drone subprocess —
    // that would re-init and we want to test recover_session_state on
    // the persisted state). The drone uses WAL so the file is consistent
    // even though the process died.
    let conn = Connection::open(&db_path).expect("reopen db");
    let recovered =
        snapshot::recover_session_state(&conn, session_id).expect("recover_session_state");

    // Plan is preserved.
    assert_eq!(recovered.plans.len(), 1);
    assert_eq!(
        recovered.plans[0].get("status").and_then(|v| v.as_str()),
        Some("approved")
    );

    // Task — formerly 'running' — is recovered as 'pending' per spec §1b.
    assert_eq!(recovered.tasks.len(), 1);
    assert_eq!(
        recovered.tasks[0].get("status").and_then(|v| v.as_str()),
        Some("pending"),
        "spec §1b: currently-running task must be recovered as pending"
    );
    assert_eq!(
        recovered.tasks[0].get("id").and_then(|v| v.as_str()),
        Some(task_id.as_str())
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tool_invoked_without_matching_result_marks_uncertain() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("d.sqlite");
    let socket = make_socket(dir.path());
    let session_id = "plan-recovery-2";

    let mut child = spawn_drone(session_id, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    let invoke_id = Uuid::new_v4().to_string();

    // Emit a tool_invoked WITHOUT a matching tool_result (simulating
    // crash mid-tool-call).
    client
        .write_signal(
            invoke_id.clone(),
            session_id.to_string(),
            "tool".to_string(),
            "tool_invoked".to_string(),
            "tool_invoke".to_string(),
            json!({
                "type": "tool_invoked",
                "agent_id": "orch",
                "tool_name": "Read",
                "source": "builtin",
                "input": {"path": "/tmp/x"},
            }),
        )
        .await
        .expect("write tool_invoked");

    // And a different tool that DID complete (control: must not appear
    // in uncertain set).
    let invoke_id_done = Uuid::new_v4().to_string();
    client
        .write_signal(
            invoke_id_done,
            session_id.to_string(),
            "tool".to_string(),
            "tool_invoked".to_string(),
            "tool_invoke".to_string(),
            json!({
                "type": "tool_invoked",
                "agent_id": "orch",
                "tool_name": "Bash",
                "source": "builtin",
                "input": {"command": "echo ok"},
            }),
        )
        .await
        .expect("write tool_invoked control");
    client
        .write_signal(
            Uuid::new_v4().to_string(),
            session_id.to_string(),
            "tool".to_string(),
            "tool_result".to_string(),
            "tool_invoke".to_string(),
            json!({
                "type": "tool_result",
                "agent_id": "orch",
                "tool_name": "Bash",
                "output": {"stdout": "ok"},
                "duration_ms": 10,
            }),
        )
        .await
        .expect("write tool_result");

    // Wait for the writes to land.
    poll_until(
        &db_path,
        |c| {
            let n: i64 = c
                .query_row("SELECT COUNT(*) FROM signals", [], |r| r.get(0))
                .unwrap_or(0);
            n >= 3
        },
        "3 signals written",
    )
    .await;

    drop(client);
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;

    let conn = Connection::open(&db_path).expect("reopen");
    let recovered =
        snapshot::recover_session_state(&conn, session_id).expect("recover_session_state");

    // Read tool_invoked has no matching tool_result → uncertain.
    assert_eq!(
        recovered.uncertain_tool_invocations,
        vec![invoke_id.clone()],
        "Read invocation without matching tool_result must be uncertain"
    );
}
