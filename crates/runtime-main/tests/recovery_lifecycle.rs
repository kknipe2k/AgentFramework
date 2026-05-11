//! Recovery lifecycle integration test — M04 Stage F (spec §1b).
//!
//! Drives a session through: write some signals → take a snapshot →
//! recover via the new `RecoverSession` IPC command → assert the
//! `ResumePlan` carries the expected state. Then resolves an uncertain
//! tool invocation via `respond_uncertainty_with` and verifies the
//! resolution signal lands in the `signals` table.
//!
//! Per spec §1b + gotcha #15: resume rebuilds HISTORY, not execution.
//! Tools in the snapshot are NOT re-invoked. This test verifies that
//! the resume flow surfaces uncertainty without dispatching any new
//! `tool_invoked` signals.

#![cfg(any(unix, windows))]

use std::time::Duration;

use runtime_main::drone_ipc::DroneClient;
use runtime_main::recovery::{
    request_resume_with, respond_uncertainty_with, ToolCallUncertaintyAction,
};
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
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-recovery-life-{suffix}"))
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
#[allow(
    clippy::too_many_lines,
    reason = "end-to-end recovery flow: seed 3 signals + snapshot + recover + resolve + assert all in one cohesive test"
)]
async fn recovery_lifecycle_round_trip_via_drone_ipc() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("d.sqlite");
    let socket = make_socket(dir.path());
    let session_id = "recovery-lifecycle-1";

    let mut child = spawn_drone(session_id, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    let plan_id = Uuid::new_v4().to_string();
    let task_id = Uuid::new_v4().to_string();
    let invoke_id = Uuid::new_v4().to_string();

    // Seed plan + task + a stranded tool_invoked (no matching result).
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
                "title": "Recovery lifecycle",
                "task_count": 1,
                "approval_required": false,
            }),
        )
        .await
        .expect("plan_created");

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
                "agent_id": "orch",
            }),
        )
        .await
        .expect("task_started");

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
                "input": {"path": "/tmp/x"},
            }),
        )
        .await
        .expect("tool_invoked");

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

    // Take a snapshot.
    client
        .send(runtime_core::DroneCommand::SnapshotNow {
            reason: "test_recovery".to_string(),
            state_json: json!({"checkpoint": 1}),
        })
        .await
        .expect("snapshot");
    poll_until(
        &db_path,
        |c| {
            let n: i64 = c
                .query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))
                .unwrap_or(0);
            n >= 1
        },
        "snapshot row written",
    )
    .await;

    // Call recover_session via the new IPC variant. Uses the main-side
    // recovery::resume::request_resume_with seam wrapping the client.
    let plan = request_resume_with(session_id.to_string(), |id| {
        let c = &client;
        async move { c.recover_session(id).await }
    })
    .await
    .expect("resume plan");

    assert!(plan.has_state, "session has prior snapshot + signals");
    assert!(plan.snapshot_id.is_some(), "snapshot_id populated");
    assert_eq!(plan.plans.len(), 1, "one plan recovered");
    assert_eq!(plan.tasks.len(), 1, "one task recovered");
    assert_eq!(
        plan.tasks[0].get("status").and_then(|v| v.as_str()),
        Some("pending"),
        "running task downgraded to pending per spec §1b"
    );
    assert_eq!(
        plan.uncertain_tool_invocations,
        vec![invoke_id.clone()],
        "stranded tool_invoked surfaced as uncertain"
    );

    // Resolve the uncertainty with `skip`. The decision signal lands in
    // the signals table via the same drone IPC `WriteSignal` path used by
    // the SDK.
    let pre_signal_count: i64 = Connection::open(&db_path)
        .expect("open db")
        .query_row("SELECT COUNT(*) FROM signals", [], |r| r.get(0))
        .expect("count pre");

    let resolution = respond_uncertainty_with(
        session_id.to_string(),
        invoke_id.clone(),
        "skip".to_string(),
        Some("orch".to_string()),
        |args| {
            let c = &client;
            async move {
                c.write_signal(
                    args.signal_id,
                    args.session_id,
                    args.kind,
                    args.event,
                    args.context_type,
                    args.payload,
                )
                .await
            }
        },
    )
    .await
    .expect("respond_uncertainty");
    assert_eq!(resolution.action, ToolCallUncertaintyAction::Skip);

    // Wait for the new decision signal to land.
    poll_until(
        &db_path,
        move |c| {
            let n: i64 = c
                .query_row("SELECT COUNT(*) FROM signals", [], |r| r.get(0))
                .unwrap_or(0);
            n > pre_signal_count
        },
        "uncertainty resolution signal written",
    )
    .await;

    // Critical invariant per gotcha #15: recovery does NOT re-invoke
    // the tool. Verify by counting `tool_invoked` signals — should still
    // be exactly 1 (the original); no new tool_invoked emitted by the
    // resume flow.
    let conn = Connection::open(&db_path).expect("open db");
    let invoked_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM signals WHERE event = 'tool_invoked'",
            [],
            |r| r.get(0),
        )
        .expect("count tool_invoked");
    assert_eq!(
        invoked_count, 1,
        "spec §1b + gotcha #15: resume must NOT re-invoke tools"
    );

    // The resolution signal is recorded with event = 'tool_call_uncertainty_resolved'.
    let resolved_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM signals WHERE event = 'tool_call_uncertainty_resolved'",
            [],
            |r| r.get(0),
        )
        .expect("count resolutions");
    assert_eq!(resolved_count, 1, "exactly one resolution signal recorded");

    drop(client);
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn recovery_lifecycle_empty_session_returns_has_state_false() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("d.sqlite");
    let socket = make_socket(dir.path());
    let session_id = "recovery-lifecycle-empty";

    let mut child = spawn_drone(session_id, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    // Don't write anything; call recover.
    let plan = request_resume_with("never-existed".to_string(), |id| {
        let c = &client;
        async move { c.recover_session(id).await }
    })
    .await
    .expect("resume plan");

    assert!(!plan.has_state, "empty session has nothing to resume");
    assert!(plan.snapshot_id.is_none());
    assert!(plan.plans.is_empty());
    assert!(plan.tasks.is_empty());
    assert!(plan.uncertain_tool_invocations.is_empty());

    drop(client);
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}
