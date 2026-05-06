//! End-to-end drone lifecycle test.
//!
//! Spawns the `runtime-drone` binary as a subprocess, waits for the IPC
//! socket to appear, signals SIGTERM (Unix only — Windows v0.1 skips per
//! `agent-runtime-spec.md` §0d), and asserts a `sigterm` snapshot row
//! lands in `SQLite` before the process exits.

#![cfg(unix)]

use std::time::Duration;
use tempfile::TempDir;

/// Locate the `runtime-drone` binary alongside the test binary.
///
/// Per `docs/gotchas.md` #22: `cargo test` puts the test binary under
/// `target/debug/deps/` while `cargo llvm-cov --workspace` uses a distinct
/// target dir (`target/llvm-cov-target/...`). Hard-coding `target/debug/`
/// breaks under coverage runs. Deriving from `std::env::current_exe()`
/// works for both. Archetype: `crates/runtime-main/tests/drone_ipc_loopback.rs::drone_binary`.
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drone_lifecycle_end_to_end() {
    let bin = drone_binary();
    if !bin.exists() {
        // CI builds the binary in the same job; locally `cargo test` builds
        // tests but not other binaries unless `cargo build` was invoked.
        let status = std::process::Command::new(env!("CARGO"))
            .args(["build", "--bin", "runtime-drone"])
            .status()
            .expect("cargo build");
        assert!(status.success(), "build failed");
    }
    assert!(bin.exists(), "drone binary missing at {}", bin.display());

    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("d.sqlite");
    let sock = dir.path().join("d.sock");

    let mut child = tokio::process::Command::new(&bin)
        .arg("--session-id")
        .arg("smoke")
        .arg("--db-path")
        .arg(&db)
        .arg("--ipc-socket")
        .arg(&sock)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn drone");

    // Wait up to 5s for the socket to appear (drone has set up its server).
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !sock.exists() && std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(sock.exists(), "drone never created its socket");

    // Send SIGTERM via nix.
    let raw_pid = i32::try_from(child.id().expect("pid")).expect("pid fits in i32");
    let pid = nix::unistd::Pid::from_raw(raw_pid);
    nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM).expect("sigterm");

    let exit = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("drone did not exit within 5s after SIGTERM")
        .expect("wait");
    assert!(
        exit.success() || exit.code().is_none(),
        "drone exit: {exit:?}"
    );

    // Open the database and confirm an emergency snapshot row exists.
    let conn = rusqlite::Connection::open(&db).expect("open db");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM snapshots WHERE event_type IN ('sigterm', 'sigint', 'emergency')",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert!(
        count >= 1,
        "expected ≥1 emergency snapshot row, got {count}"
    );
}

// ── Stage E (M03) IPC roundtrip tests for QuerySessionDb + ReadSignals.
// ── Each spawns the drone, seeds the database via raw rusqlite, then
// ── connects over the socket and exchanges one JSON-line command + one
// ── response event. Same subprocess pattern as drone_lifecycle_end_to_end
// ── above; differs in that we drive shutdown via DroneCommand::GracefulShutdown
// ── over the IPC channel rather than SIGTERM.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_session_db_roundtrip_returns_rows() {
    use runtime_core::DroneCommand;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let bin = drone_binary();
    if !bin.exists() {
        let status = std::process::Command::new(env!("CARGO"))
            .args(["build", "--bin", "runtime-drone"])
            .status()
            .expect("cargo build");
        assert!(status.success(), "build failed");
    }

    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("d.sqlite");
    let sock = dir.path().join("d.sock");

    // Seed two signals into the database BEFORE the drone starts so the
    // SELECT returns deterministic rows. The drone's startup will see the
    // existing schema (idempotent) and the seed rows.
    {
        let conn = rusqlite::Connection::open(&db).expect("seed open");
        runtime_drone::db::init_in_existing(&conn).expect("init schema");
        conn.execute(
            "INSERT INTO sessions (id, status) VALUES ('s1', 'active')",
            [],
        )
        .expect("seed session");
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES ('sig1', 's1', 'tool', 'invoked', '0', '{}', 'agent_loop')",
            [],
        )
        .expect("seed sig1");
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES ('sig2', 's1', 'decision', 'decision', '0', '{}', 'agent_loop')",
            [],
        )
        .expect("seed sig2");
    }

    let mut child = tokio::process::Command::new(&bin)
        .arg("--session-id")
        .arg("s1")
        .arg("--db-path")
        .arg(&db)
        .arg("--ipc-socket")
        .arg(&sock)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn drone");

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !sock.exists() && std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(sock.exists(), "drone never created its socket");

    let stream = UnixStream::connect(&sock).await.expect("connect");
    let (rd, mut wr) = stream.into_split();
    let mut reader = BufReader::new(rd);

    let cmd = DroneCommand::QuerySessionDb {
        sql: "SELECT id, type FROM signals ORDER BY id".to_string(),
    };
    let line = format!("{}\n", serde_json::to_string(&cmd).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write");
    wr.flush().await.expect("flush");

    // The drone replies with one or more events; filter for QueryResult.
    let mut got = None;
    let read_deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < read_deadline {
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_millis(500), reader.read_line(&mut line))
            .await
            .ok()
            .and_then(|r| r.ok())
            .unwrap_or(0);
        if n == 0 {
            continue;
        }
        if let Ok(evt) = serde_json::from_str::<runtime_core::DroneEvent>(line.trim()) {
            if let runtime_core::DroneEvent::QueryResult { rows } = evt {
                got = Some(rows);
                break;
            }
        }
    }
    let rows = got.expect("expected QueryResult event");
    assert_eq!(rows.len(), 2, "two seeded signals → two rows");

    // Drive graceful shutdown via IPC.
    let shutdown = DroneCommand::GracefulShutdown { timeout_ms: 50 };
    let line = format!("{}\n", serde_json::to_string(&shutdown).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write shutdown");
    wr.flush().await.expect("flush shutdown");
    let _ = tokio::time::timeout(Duration::from_secs(3), child.wait()).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn read_signals_roundtrip_preserves_ordering() {
    use runtime_core::DroneCommand;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let bin = drone_binary();
    if !bin.exists() {
        let status = std::process::Command::new(env!("CARGO"))
            .args(["build", "--bin", "runtime-drone"])
            .status()
            .expect("cargo build");
        assert!(status.success(), "build failed");
    }

    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("d.sqlite");
    let sock = dir.path().join("d.sock");

    {
        let conn = rusqlite::Connection::open(&db).expect("seed open");
        runtime_drone::db::init_in_existing(&conn).expect("init schema");
        conn.execute(
            "INSERT INTO sessions (id, status) VALUES ('s1', 'active')",
            [],
        )
        .expect("seed session");
        for (id, ts) in [("a", "1"), ("b", "2"), ("c", "3")] {
            conn.execute(
                "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
                 VALUES (?1, 's1', 'agent', 'spawned', ?2, '{}', 'agent_loop')",
                rusqlite::params![id, ts],
            )
            .expect("seed sig");
        }
    }

    let mut child = tokio::process::Command::new(&bin)
        .arg("--session-id")
        .arg("s1")
        .arg("--db-path")
        .arg(&db)
        .arg("--ipc-socket")
        .arg(&sock)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn drone");

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !sock.exists() && std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(sock.exists());

    let stream = UnixStream::connect(&sock).await.expect("connect");
    let (rd, mut wr) = stream.into_split();
    let mut reader = BufReader::new(rd);

    let cmd = DroneCommand::ReadSignals {
        session_id: "s1".to_string(),
    };
    let line = format!("{}\n", serde_json::to_string(&cmd).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write");
    wr.flush().await.expect("flush");

    let mut got = None;
    let read_deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < read_deadline {
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_millis(500), reader.read_line(&mut line))
            .await
            .ok()
            .and_then(|r| r.ok())
            .unwrap_or(0);
        if n == 0 {
            continue;
        }
        if let Ok(evt) = serde_json::from_str::<runtime_core::DroneEvent>(line.trim()) {
            if let runtime_core::DroneEvent::SignalLog { signals } = evt {
                got = Some(signals);
                break;
            }
        }
    }
    let signals = got.expect("expected SignalLog event");
    assert_eq!(signals.len(), 3);
    let ids: Vec<&str> = signals
        .iter()
        .filter_map(|s| s.get("id").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(ids, vec!["a", "b", "c"], "ordering by timestamp preserved");

    let shutdown = DroneCommand::GracefulShutdown { timeout_ms: 50 };
    let line = format!("{}\n", serde_json::to_string(&shutdown).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write shutdown");
    wr.flush().await.expect("flush shutdown");
    let _ = tokio::time::timeout(Duration::from_secs(3), child.wait()).await;
}
