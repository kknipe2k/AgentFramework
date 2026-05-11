//! Long-lived `events()` subscription survival across drone restart
//! (M04 Stage A2).
//!
//! Per `agent-runtime-spec.md` §1d ⚠️ note (M03 → M04 carry-forward): does
//! the renderer's long-lived `agent_event` subscription survive a mid-
//! session main↔drone reconnect? The answer determines a v0.1 behavior
//! lock + spec edit.
//!
//! # The test outcome (locked here for v0.1)
//!
//! The current `Connection::take_event_stream` design (single-consumer,
//! consumes the reader half) means the `events()` subscription is bound
//! to the *original* connection's reader. When the drone subprocess is
//! killed, the reader EOFs and the stream terminates. `send_with_reconnect`
//! re-opens the underlying socket and installs a fresh reader, but the
//! original subscriber's stream stays bound to the now-closed reader and
//! does NOT see events from the post-reconnect drone.
//!
//! **v0.1 behavior:** subscribers must resubscribe on reconnect. The
//! renderer's `agent_event` channel is fed by `commands.rs::forward_events`
//! / `replay_session`, which open a fresh subscription per task —
//! survival-across-reconnect is not required at the application layer.
//!
//! Spec §1d updated at this commit to reflect the lock; see also
//! `docs/build-prompts/M04-plan-verify-hitl-budget.md` Stage A2.
//!
//! Sister test for cross-validation: `drone_ipc_loopback.rs::
//! surfaces_disconnected_after_drone_killed` — verifies the *send* path
//! exhausts retries on a dead drone. This file verifies the *events*
//! path terminates.

#![cfg(any(unix, windows))]

use std::time::Duration;

use futures::StreamExt;
use runtime_core::{DroneCommand, DroneEvent};
use runtime_main::drone_ipc::DroneClient;
use tempfile::TempDir;
use tokio::time::timeout;

/// Locate the `runtime-drone` binary alongside the test binary. Same
/// archetype as `drone_ipc_loopback::drone_binary` and
/// `crates/runtime-drone/tests/integration.rs::drone_binary`. Per
/// `docs/gotchas.md` #22 — `current_exe()` works under both `cargo test`
/// (target/debug/deps/) and `cargo llvm-cov` (target/llvm-cov-target/).
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
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-reconnect-{suffix}"))
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn long_lived_events_subscription_terminates_when_drone_killed() {
    // The events() subscription is single-consumer and bound to the
    // original connection's reader half. When the underlying socket
    // closes (drone killed), the stream yields None and terminates.
    //
    // This is the foundational invariant for the v0.1 "resubscribe on
    // reconnect" behavior — without termination on close, a renderer
    // would hang forever waiting for events that will never arrive.
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("d.sqlite");
    let socket = make_socket(dir.path());
    let mut child = spawn_drone("reconnect-1", &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;
    let mut events = client.events().await.expect("events stream");

    // Drive at least one event through the subscription so we know it's
    // alive before we kill the drone.
    client
        .send(DroneCommand::SnapshotNow {
            reason: "pre_kill".to_string(),
            state_json: serde_json::json!({}),
        })
        .await
        .expect("send pre_kill");
    let mut saw_pre_kill = false;
    for _ in 0..40 {
        if let Ok(Some(Ok(DroneEvent::SnapshotWritten { reason, .. }))) =
            timeout(Duration::from_millis(250), events.next()).await
        {
            if reason == "pre_kill" {
                saw_pre_kill = true;
                break;
            }
        }
    }
    assert!(saw_pre_kill, "subscription must receive events pre-kill");

    // Now kill the drone hard. The subscription's underlying reader
    // EOFs; the stream yields None on its next poll.
    child.start_kill().ok();
    let _ = timeout(Duration::from_secs(2), child.wait()).await;

    // The stream terminates within the timeout window — it does not
    // hang waiting for a reconnect that the events() subscription
    // wouldn't observe anyway.
    // Split the timeout from the stream result so the inner match doesn't
    // need a wildcard arm for the elapsed branch (which would trip
    // `clippy::match_wild_err_arm`).
    let stream_result = timeout(Duration::from_secs(2), events.next())
        .await
        .expect("stream did not terminate within 2s of drone kill");
    match stream_result {
        // Clean EOF (None) is the Unix-typical termination shape;
        // a Codec/IO error before EOF (Some(Err(_))) is also a valid
        // termination signal — some platforms surface partial reads
        // as errors. Either is observable termination, which is what
        // the v0.1 behavior lock requires.
        None | Some(Err(_)) => {}
        Some(Ok(event)) => {
            panic!("expected stream termination, got post-kill event: {event:?}");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn old_events_subscription_does_not_observe_post_reconnect_drone() {
    // The v0.1 behavior lock: a long-lived events() subscription bound
    // to the original connection does NOT receive events from a
    // post-reconnect drone. Renderers must resubscribe on reconnect.
    //
    // The test simulates: subscribe → drone dies → start fresh drone →
    // continue using the SAME client (which reconnects on next send)
    // → assert the OLD events stream did not pick up the new drone's
    // emissions.
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("d.sqlite");
    let socket = make_socket(dir.path());
    let mut drone1 = spawn_drone("reconnect-2", &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;
    let mut old_events = client.events().await.expect("events stream");

    // Kill the first drone.
    drone1.start_kill().ok();
    let _ = timeout(Duration::from_secs(2), drone1.wait()).await;
    // Drain any buffered events the reader emits before EOF. Track
    // whether we've observed termination (None or Err); after a stream
    // yields Ready(None), polling again would panic via futures-util's
    // `unfold` (Unfold must not be polled after it returned None).
    let mut stream_terminated = false;
    while !stream_terminated {
        match timeout(Duration::from_millis(150), old_events.next()).await {
            Ok(None | Some(Err(_))) => stream_terminated = true,
            Ok(Some(Ok(_))) => {} // pre-kill buffered event; discard
            Err(_) => break,      // pending — stream not yet drained
        }
    }

    // On Unix the socket file persists; remove it so the new drone can
    // bind fresh. On Windows named pipes are reaped automatically.
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(&socket);
    }

    // Spawn a fresh drone bound to the SAME socket address.
    let mut drone2 = spawn_drone("reconnect-2-restart", &db_path, &socket);

    // Wait until the new drone has bound the socket. We don't assert the
    // client successfully reconnects here — `send_with_reconnect`
    // exercises that path under `drone_ipc_loopback::surfaces_disconnected_after_drone_killed`.
    // What we DO assert: the OLD events subscription, bound to the
    // original connection's reader, does not yield events from the new
    // drone within a reasonable window.
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        // On Unix, "bound" is observable via the socket file appearing.
        // On Windows the named pipe is opaque; just sleep the budget.
        #[cfg(unix)]
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Sanity: the new drone is alive — confirm by sending a command via
    // a FRESH client (the exec path that the renderer would use after
    // resubscribing).
    let fresh_client = connect_with_retry(&socket_to_addr(&socket)).await;
    fresh_client
        .send(DroneCommand::SnapshotNow {
            reason: "post_restart".to_string(),
            state_json: serde_json::json!({}),
        })
        .await
        .expect("send to fresh drone");
    drop(fresh_client);

    // If the drain phase already saw termination, that IS the v0.1
    // behavior lock — the OLD subscription will never observe the
    // post-reconnect drone, period. Only poll again when the drain
    // phase ended in pending (timeout): then we want to confirm that
    // nothing arrives within a longer window.
    if !stream_terminated {
        let observed_post_restart = match timeout(Duration::from_millis(750), old_events.next())
            .await
        {
            Ok(Some(Ok(DroneEvent::SnapshotWritten { reason, .. }))) => reason == "post_restart",
            _ => false,
        };
        assert!(
            !observed_post_restart,
            "v0.1 behavior lock: old events() subscription must not observe post-reconnect drone events"
        );
    }

    drone2.start_kill().ok();
    let _ = timeout(Duration::from_secs(2), drone2.wait()).await;
}
