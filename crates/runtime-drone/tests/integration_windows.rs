//! Windows end-to-end drone lifecycle test.
//!
//! Spawns the `runtime-drone` binary as a subprocess on Windows, connects
//! to its named-pipe IPC, sends `DroneCommand::SnapshotNow` and verifies
//! a row appears in `SQLite`, then sends `DroneCommand::GracefulShutdown`
//! and verifies the process exits cleanly within the timeout. Sister to
//! `integration.rs` (Unix SIGTERM lifecycle); together they exercise the
//! drone's two cross-platform shutdown paths.

#![cfg(windows)]
// Single-flow integration test — the >100-line body is by design (full
// end-to-end lifecycle). Splitting it into helpers would obscure the
// linear narrative reviewers need to follow; the equivalent Unix test in
// `tests/integration.rs` is the same shape.
#![allow(clippy::too_many_lines)]

use std::time::Duration;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::time::timeout;

use runtime_core::{DroneCommand, DroneEvent, HeartbeatStatus};

fn drone_binary() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("..");
    p.push("target");
    p.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    p.push("runtime-drone.exe");
    p
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drone_lifecycle_end_to_end_windows() {
    let bin = drone_binary();
    if !bin.exists() {
        let status = std::process::Command::new(env!("CARGO"))
            .args(["build", "--bin", "runtime-drone"])
            .status()
            .expect("cargo build");
        assert!(status.success(), "build failed");
    }
    assert!(bin.exists(), "drone binary missing at {}", bin.display());

    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("d.sqlite");
    let suffix = uuid::Uuid::new_v4();
    let pipe_name = format!(r"\\.\pipe\runtime-drone-int-{suffix}");

    let mut child = tokio::process::Command::new(&bin)
        .arg("--session-id")
        .arg("smoke-win")
        .arg("--db-path")
        .arg(&db)
        .arg("--ipc-socket")
        .arg(&pipe_name)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn drone");

    // Wait up to 5s for the named pipe to become connectable.
    let client = {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            assert!(
                std::time::Instant::now() <= deadline,
                "drone never opened pipe {pipe_name}"
            );
            match ClientOptions::new().open(&pipe_name) {
                Ok(c) => break c,
                Err(_) => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
    };

    let (rd, mut wr) = tokio::io::split(client);
    let mut reader = BufReader::new(rd);

    // Send SnapshotNow over IPC.
    let snap = DroneCommand::SnapshotNow {
        reason: "manual-win".to_string(),
        state_json: serde_json::json!({"hello": "windows"}),
    };
    let line = format!("{}\n", serde_json::to_string(&snap).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write snap");
    wr.flush().await.expect("flush snap");

    // Read events until SnapshotWritten arrives (skipping the first heartbeat).
    let mut saw_snapshot = false;
    let snap_deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < snap_deadline {
        let mut buf = String::new();
        match timeout(Duration::from_secs(2), reader.read_line(&mut buf)).await {
            Ok(Ok(0)) => break,
            Ok(Ok(_)) => {
                let trimmed = buf.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(event) = serde_json::from_str::<DroneEvent>(trimmed) {
                    match event {
                        DroneEvent::SnapshotWritten { .. } => {
                            saw_snapshot = true;
                            break;
                        }
                        DroneEvent::Heartbeat { status, .. } => {
                            // Confirm the typed enum surfaces over IPC.
                            assert!(matches!(
                                status,
                                HeartbeatStatus::Ok
                                    | HeartbeatStatus::Degraded
                                    | HeartbeatStatus::Stalled
                            ));
                        }
                        _ => {}
                    }
                }
            }
            _ => break,
        }
    }
    assert!(
        saw_snapshot,
        "expected DroneEvent::SnapshotWritten over IPC after SnapshotNow"
    );

    // Verify the snapshot row landed in SQLite.
    let conn = rusqlite::Connection::open(&db).expect("open db");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM snapshots WHERE event_type = 'manual-win'",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert!(
        count >= 1,
        "expected ≥1 snapshot row with reason='manual-win', got {count}"
    );

    // Send GracefulShutdown — the drone process must exit within the timeout.
    let shutdown = DroneCommand::GracefulShutdown { timeout_ms: 1000 };
    let line = format!("{}\n", serde_json::to_string(&shutdown).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write shutdown");
    wr.flush().await.expect("flush shutdown");

    // Drop the pipe halves so the drone's pipe loop unblocks if it's reading.
    drop(wr);
    drop(reader);

    let exit = timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("drone did not exit within 5s after GracefulShutdown")
        .expect("wait");
    assert!(
        exit.success() || exit.code().is_none(),
        "drone exit: {exit:?}"
    );

    // Confirm an `ipc_graceful` snapshot was written by the shutdown handler.
    let final_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM snapshots WHERE event_type = 'ipc_graceful'",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert!(
        final_count >= 1,
        "expected ≥1 ipc_graceful snapshot row, got {final_count}"
    );
}
