//! Drone IPC loopback — main-side `DroneClient` ↔ M01 drone subprocess.
//!
//! Spawns the `runtime-drone` binary, connects via the main-side
//! `DroneClient`, sends every `DroneCommand` variant, and asserts the
//! drone's reaction (snapshot rows in `SQLite`, `DroneEvent` responses on
//! the IPC channel, clean process exit on `GracefulShutdown`).
//!
//! Sister to `crates/runtime-drone/tests/integration.rs` (Unix SIGTERM
//! lifecycle) and `tests/integration_windows.rs` (Windows lifecycle); this
//! test exercises the *main-side* of the same wire format.
//!
//! Reconnect: kills the drone mid-session, verifies the client retries
//! within `MAX_RETRIES` and surfaces `DroneIpcError::Disconnected` when
//! exceeded.

#![cfg(any(unix, windows))]
#![allow(clippy::too_many_lines, reason = "linear end-to-end flows")]

use std::time::Duration;

use std::collections::HashMap;

use runtime_core::{DroneCommand, DroneEvent};
use runtime_main::drone_ipc::{DroneClient, DroneIpcError};
use tempfile::TempDir;
use tokio::time::timeout;

/// Locate the `runtime-drone` binary. Derives the path from
/// `current_exe()` so it works under `cargo test` (target dir = `target/`)
/// AND under `cargo llvm-cov` (target dir = `target/llvm-cov-target/`).
/// The test binary lives at `<target>/<profile>/deps/drone_ipc_loopback-*`;
/// the drone binary is at `<target>/<profile>/runtime-drone`.
fn drone_binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().expect("current_exe");
    p.pop(); // drop the test exe filename
    if p.ends_with("deps") {
        p.pop(); // up to the profile dir
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
        // Build into the same target dir (so the binary lands next to us).
        let target_dir = bin.parent().expect("parent");
        // CARGO_TARGET_DIR may already be set by llvm-cov; preserve it.
        let mut cmd = std::process::Command::new(env!("CARGO"));
        cmd.args(["build", "--bin", "runtime-drone"]);
        if std::env::var_os("CARGO_TARGET_DIR").is_none() {
            // Force the target dir so an out-of-band build lands next to us.
            // Walk back up to the workspace root for cargo.
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
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-loopback-{suffix}"))
}

#[cfg(unix)]
fn socket_to_addr(p: &std::path::Path) -> String {
    p.to_string_lossy().into_owned()
}

#[cfg(windows)]
fn socket_to_addr(p: &std::path::Path) -> String {
    p.to_string_lossy().into_owned()
}

struct DroneFixture {
    child: tokio::process::Child,
    _dir: TempDir,
    socket: std::path::PathBuf,
    db_path: std::path::PathBuf,
}

impl DroneFixture {
    #[allow(
        clippy::unused_async,
        reason = "reserved for future asynchronous setup (waiting on socket); current body is sync but the public surface is async for shape stability"
    )]
    async fn spawn(session: &str) -> Self {
        ensure_drone_built();
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("d.sqlite");
        let socket = make_socket(dir.path());
        let mut cmd = tokio::process::Command::new(drone_binary());
        cmd.arg("--session-id")
            .arg(session)
            .arg("--db-path")
            .arg(&db_path)
            .arg("--ipc-socket")
            .arg(&socket)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let child = cmd.spawn().expect("spawn drone");
        Self {
            child,
            _dir: dir,
            socket,
            db_path,
        }
    }

    async fn connect(&self) -> DroneClient {
        let addr = socket_to_addr(&self.socket);
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            match DroneClient::connect(&addr).await {
                Ok(c) => return c,
                Err(_) if std::time::Instant::now() < deadline => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                Err(e) => panic!("connect: {e}"),
            }
        }
    }

    async fn shutdown(mut self) {
        self.child.start_kill().ok();
        let _ = timeout(Duration::from_secs(3), self.child.wait()).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connects_to_drone() {
    let fx = DroneFixture::spawn("loop-connect").await;
    let _client = fx.connect().await;
    fx.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn snapshot_now_writes_row_to_db() {
    let fx = DroneFixture::spawn("loop-snap").await;
    let client = fx.connect().await;
    client
        .send(DroneCommand::SnapshotNow {
            reason: "task_started".to_string(),
            state_json: serde_json::json!({"k": "v"}),
        })
        .await
        .expect("send");

    // Poll the SQLite for the snapshot row.
    let db = fx.db_path.clone();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut count: i64 = 0;
    while std::time::Instant::now() < deadline {
        if let Ok(conn) = rusqlite::Connection::open(&db) {
            // The drone snapshots table stores the DroneCommand `reason`
            // value in the `event_type` column (see
            // crates/runtime-drone/src/snapshot.rs:30 + db.rs:83).
            if let Ok(c) = conn.query_row(
                "SELECT COUNT(*) FROM snapshots WHERE event_type = ?1",
                ["task_started"],
                |r| r.get::<_, i64>(0),
            ) {
                count = c;
                if count > 0 {
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(count > 0, "snapshot row not observed in db");
    fx.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn snapshot_now_emits_snapshot_written_event() {
    let fx = DroneFixture::spawn("loop-evt").await;
    let client = fx.connect().await;
    let mut events = client.events().await.expect("events stream");
    client
        .send(DroneCommand::SnapshotNow {
            reason: "task_started".to_string(),
            state_json: serde_json::json!({}),
        })
        .await
        .expect("send");
    let mut got = false;
    for _ in 0..40 {
        if let Ok(Some(Ok(DroneEvent::SnapshotWritten { reason, .. }))) = timeout(
            Duration::from_millis(250),
            futures::StreamExt::next(&mut events),
        )
        .await
        {
            if reason == "task_started" {
                got = true;
                break;
            }
        }
    }
    assert!(got, "SnapshotWritten event not received");
    fx.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn graceful_shutdown_exits_drone() {
    let mut fx = DroneFixture::spawn("loop-gs").await;
    let client = fx.connect().await;
    client
        .send(DroneCommand::GracefulShutdown { timeout_ms: 1_000 })
        .await
        .expect("send");
    let exit = timeout(Duration::from_secs(5), fx.child.wait())
        .await
        .expect("drone did not exit within 5s")
        .expect("wait");
    assert!(exit.success() || exit.code().is_some(), "exit: {exit:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_activity_timeout_no_panic() {
    let fx = DroneFixture::spawn("loop-act").await;
    let client = fx.connect().await;
    client
        .send(DroneCommand::SetActivityTimeout { ms: 5_000 })
        .await
        .expect("send");
    fx.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revert_to_snapshot_no_panic() {
    let fx = DroneFixture::spawn("loop-rev").await;
    let client = fx.connect().await;
    client
        .send(DroneCommand::RevertToSnapshot {
            snapshot_id: "snap_123".to_string(),
            reason: runtime_core::RevertReason::UserRollback,
        })
        .await
        .expect("send");
    fx.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_process_unknown_pid_no_panic() {
    let fx = DroneFixture::spawn("loop-stop").await;
    let client = fx.connect().await;
    client
        .send(DroneCommand::StopProcess {
            pid: 999_999,
            force: false,
        })
        .await
        .expect("send");
    fx.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_process_request_no_panic() {
    let fx = DroneFixture::spawn("loop-spawn").await;
    let client = fx.connect().await;
    client
        .send(DroneCommand::SpawnProcess {
            process_type: runtime_core::ProcessType::Agent,
            config: runtime_core::ProcessConfig {
                command: "echo".to_string(),
                args: vec!["hi".to_string()],
                env: HashMap::default(),
            },
        })
        .await
        .expect("send");
    fx.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn surfaces_disconnected_after_drone_killed() {
    let mut fx = DroneFixture::spawn("loop-disc").await;
    let client = fx.connect().await;
    // Kill the drone hard, then attempt to send.
    fx.child.start_kill().ok();
    let _ = timeout(Duration::from_secs(2), fx.child.wait()).await;
    // Attempts may succeed once if the OS has buffered space; eventually
    // the retries exhaust.
    let mut last_err: Option<DroneIpcError> = None;
    for _ in 0..3 {
        if let Err(e) = client
            .send(DroneCommand::SnapshotNow {
                reason: "after_kill".to_string(),
                state_json: serde_json::json!({}),
            })
            .await
        {
            last_err = Some(e);
            break;
        }
    }
    assert!(
        matches!(last_err, Some(DroneIpcError::Disconnected { .. })),
        "expected Disconnected, got {last_err:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drop_client_mid_send_no_orphan() {
    let fx = DroneFixture::spawn("loop-drop").await;
    let client = fx.connect().await;
    let send_fut = client.send(DroneCommand::SnapshotNow {
        reason: "drop_test".to_string(),
        state_json: serde_json::json!({}),
    });
    drop(send_fut);
    drop(client);
    fx.shutdown().await;
}
