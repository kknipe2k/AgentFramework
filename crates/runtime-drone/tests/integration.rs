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
