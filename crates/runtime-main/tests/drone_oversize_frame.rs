//! TD-053 adversarial assembled tests — oversize / at-cap frames against
//! the REAL drone subprocess (M09.5.C).
//!
//! The review's C4 finding: every IPC decode site used `LinesCodec::new()`
//! (max length `usize::MAX`), so a peer writing bytes with no newline
//! buffered unbounded memory and the `MaxLineLengthExceeded` arms were
//! dead code. The hostile case IS the acceptance here: a `CAP + 1` byte
//! unterminated write must fail the connection (the length signal) while
//! the drone stays alive and serves the next connection; a frame at
//! exactly the cap must round-trip (the cap clips nothing legitimate).
//!
//! Reuses the TD-005 pre-staged drone fixture (`tests/common/mod.rs`) and
//! the `drone_ipc_loopback` raw-socket pattern.

#![cfg(any(unix, windows))]

use std::time::Duration;

use runtime_core::{DroneCommand, DroneEvent};
use runtime_main::drone_ipc::DroneClient;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

mod common;
use common::{drone_binary, ensure_drone_built};

/// Mirrors `runtime_core::MAX_IPC_FRAME_BYTES` as a literal on purpose:
/// the test pins the agreed 4 MiB boundary VALUE (delimiter-exclusive,
/// per the tokio-util 0.7.18 `LinesCodec` decode semantics), so a silent
/// change to the production constant fails here.
const CAP: usize = 4 * 1024 * 1024;

#[cfg(unix)]
fn make_socket(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("d.sock")
}

#[cfg(windows)]
fn make_socket(_dir: &std::path::Path) -> std::path::PathBuf {
    let suffix = uuid::Uuid::new_v4();
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-oversize-{suffix}"))
}

struct DroneFixture {
    child: tokio::process::Child,
    _dir: TempDir,
    socket: std::path::PathBuf,
}

impl DroneFixture {
    #[allow(
        clippy::unused_async,
        reason = "mirrors the loopback fixture: reserved for future asynchronous setup (waiting on socket); current body is sync but the public surface is async for shape stability"
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
            .stderr(std::process::Stdio::piped())
            // On assert-failure the fixture's shutdown() is never
            // reached; without this the orphaned drone inherits the
            // test runner's stdout pipe handle on Windows and hangs
            // the `cargo test | ...` pipeline (observed at this
            // stage's red run).
            .kill_on_drop(true);
        let child = cmd.spawn().expect("spawn drone");
        Self {
            child,
            _dir: dir,
            socket,
        }
    }

    async fn connect(&self) -> DroneClient {
        let addr = self.socket.to_string_lossy().into_owned();
        // 30s window per the loopback fixture's rationale: under heavy CI
        // load the drone's pipe creation can be CPU-starved well past 5s.
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
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

#[cfg(unix)]
async fn open_raw_client(path: &std::path::Path) -> tokio::net::UnixStream {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        match tokio::net::UnixStream::connect(path).await {
            Ok(s) => return s,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => panic!("raw connect: {e}"),
        }
    }
}

#[cfg(windows)]
async fn open_raw_client(
    path: &std::path::Path,
) -> tokio::net::windows::named_pipe::NamedPipeClient {
    use tokio::net::windows::named_pipe::ClientOptions;
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        match ClientOptions::new().open(path) {
            Ok(p) => return p,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => panic!("raw client connect: {e}"),
        }
    }
}

/// Build a `SnapshotNow` whose serialized line is EXACTLY `CAP` content
/// bytes (the `\n` the codec appends is on top — the limit is
/// delimiter-exclusive). The pad is plain `x`s so JSON escaping adds
/// nothing.
fn at_cap_snapshot_cmd() -> DroneCommand {
    let base = serde_json::to_string(&DroneCommand::SnapshotNow {
        reason: "at-cap".to_string(),
        state_json: serde_json::json!({"pad": ""}),
    })
    .expect("serialize base")
    .len();
    let pad = "x".repeat(CAP - base);
    let cmd = DroneCommand::SnapshotNow {
        reason: "at-cap".to_string(),
        state_json: serde_json::json!({ "pad": pad }),
    };
    let line = serde_json::to_string(&cmd).expect("serialize");
    assert_eq!(
        line.len(),
        CAP,
        "fixture bug: at-cap line must be exactly CAP bytes"
    );
    cmd
}

/// THE adversarial acceptance (C.4 scenario 1): a `CAP + 1` byte write
/// with NO newline must fail the connection — the drone closes it — and
/// the drone must remain alive, serving a subsequent well-formed
/// connection.
///
/// RED (pre-impl): the uncapped codec buffers the blob forever; the
/// connection stays open past the deadline (heartbeats keep flowing) and
/// the assertion fails — the review's finding reproduced.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversize_unterminated_frame_is_rejected_and_drone_serves_next_connection() {
    let fx = DroneFixture::spawn("oversize-reject").await;
    let mut raw = open_raw_client(&fx.socket).await;

    let blob = vec![b'x'; CAP + 1];
    // Post-impl the drone may drop the connection while we are still
    // writing; a write/flush error is the same rejection signal as a
    // read-side EOF.
    let write_result = match raw.write_all(&blob).await {
        Ok(()) => raw.flush().await,
        Err(e) => Err(e),
    };

    let terminated = if write_result.is_err() {
        true
    } else {
        // The connection still carries the drone's event broadcast
        // (heartbeats); drain data until EOF / error / deadline. Only a
        // server-side close counts as rejection.
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut buf = [0u8; 4096];
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break false;
            }
            match timeout(remaining, raw.read(&mut buf)).await {
                Ok(Ok(0) | Err(_)) => break true,
                Ok(Ok(_)) => {}
                Err(_) => break false,
            }
        }
    };
    assert!(
        terminated,
        "drone accepted a {} byte unterminated frame (connection still open \
         after deadline) — the unbounded LinesCodec buffered it (TD-053)",
        CAP + 1
    );

    // Liveness: a NEW well-formed connection must round-trip.
    let client = fx.connect().await;
    let mut events = client.events().await.expect("events stream");
    client
        .send(DroneCommand::SnapshotNow {
            reason: "post-oversize-liveness".to_string(),
            state_json: serde_json::json!({}),
        })
        .await
        .expect("send on new connection after oversize rejection");
    let mut got = false;
    for _ in 0..40 {
        if let Ok(Some(Ok(DroneEvent::SnapshotWritten { reason, .. }))) = timeout(
            Duration::from_millis(250),
            futures::StreamExt::next(&mut events),
        )
        .await
        {
            if reason == "post-oversize-liveness" {
                got = true;
                break;
            }
        }
    }
    assert!(
        got,
        "drone did not serve the next connection after the oversize frame"
    );
    fx.shutdown().await;
}

/// PIN — green at red by design (rider 3): a frame at EXACTLY the cap
/// round-trips through the real drone. Passes today (no cap) and must
/// keep passing post-impl — it pins that the 4 MiB cap clips nothing
/// legitimate and that the limit is delimiter-exclusive (rider 2).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_cap_frame_round_trips_through_the_real_drone() {
    let fx = DroneFixture::spawn("at-cap").await;
    let client = fx.connect().await;
    let mut events = client.events().await.expect("events stream");
    client
        .send(at_cap_snapshot_cmd())
        .await
        .expect("send at-cap frame");
    let mut got = false;
    for _ in 0..80 {
        if let Ok(Some(Ok(DroneEvent::SnapshotWritten { reason, .. }))) = timeout(
            Duration::from_millis(250),
            futures::StreamExt::next(&mut events),
        )
        .await
        {
            if reason == "at-cap" {
                got = true;
                break;
            }
        }
    }
    assert!(
        got,
        "a frame at exactly the cap must round-trip (SnapshotWritten not observed)"
    );
    fx.shutdown().await;
}
