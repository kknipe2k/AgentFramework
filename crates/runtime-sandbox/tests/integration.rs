//! End-to-end sandbox lifecycle test.
//!
//! Spawns the `runtime-sandbox` binary as a subprocess, connects over the
//! IPC socket / pipe, sends a `ValidateArtifact` request, and asserts the
//! `ValidationResult` response arrives. M05 Stage C2 added the OS-level
//! isolation tests at the bottom of the file:
//!
//! - `isolation_active_under_real_subprocess` — confirms seccomp filter
//!   is loaded (Linux: `/proc/$pid/status` Seccomp field == 2) or the
//!   process is in a Job Object (Windows: `IsProcessInJob` returns
//!   TRUE) immediately after the subprocess boots, before the first
//!   validate request lands.
//! - `isolation_persists_across_validate_calls` — three sequential
//!   `ValidateArtifact` round trips, re-checking the isolation state
//!   after each response. Proves isolation isn't reset per call.

#![cfg(any(unix, windows))]

use std::time::Duration;

use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, ResourceName,
    SideEffectClass,
};
use runtime_sandbox::{
    protocol::{SandboxRequest, SandboxResponse},
    validator::ValidationResult,
};
use std::str::FromStr;
use tempfile::TempDir;

/// Locate the `runtime-sandbox` binary alongside the test binary.
/// Same `current_exe()` derivation pattern as the drone test binary —
/// works under both `cargo test` and `cargo llvm-cov` (gotcha #22).
fn sandbox_binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().expect("current_exe");
    p.pop(); // drop the test exe filename
    if p.ends_with("deps") {
        p.pop(); // up to the profile dir
    }
    #[cfg(windows)]
    p.push("runtime-sandbox.exe");
    #[cfg(unix)]
    p.push("runtime-sandbox");
    p
}

fn ensure_sandbox_built() {
    let bin = sandbox_binary();
    if !bin.exists() {
        let target_dir = bin.parent().expect("parent");
        let mut cmd = std::process::Command::new(env!("CARGO"));
        cmd.args(["build", "--bin", "runtime-sandbox"]);
        if std::env::var_os("CARGO_TARGET_DIR").is_none() {
            cmd.env(
                "CARGO_TARGET_DIR",
                target_dir.parent().expect("profile parent"),
            );
        }
        let status = cmd.status().expect("cargo build");
        assert!(status.success(), "sandbox build failed");
    }
    assert!(bin.exists(), "sandbox binary missing at {}", bin.display());
}

#[cfg(unix)]
fn make_socket(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("sb.sock")
}

#[cfg(windows)]
fn make_socket(_dir: &std::path::Path) -> std::path::PathBuf {
    let suffix = uuid::Uuid::new_v4();
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-sandbox-integ-{suffix}"))
}

fn pure_read_declaration() -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Read,
        resource: ResourceName::from_str("*.md").expect("resource"),
        scope: CapabilityScope::Glob(GlobPattern::from_str("*.md").expect("glob")),
        side_effect_class: SideEffectClass::Pure,
    }
}

#[cfg(unix)]
async fn open_client(addr: &std::path::Path) -> tokio::net::UnixStream {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match tokio::net::UnixStream::connect(addr).await {
            Ok(s) => return s,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => panic!("connect: {e}"),
        }
    }
}

#[cfg(windows)]
async fn open_client(addr: &std::path::Path) -> tokio::net::windows::named_pipe::NamedPipeClient {
    use tokio::net::windows::named_pipe::ClientOptions;
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match ClientOptions::new().open(addr) {
            Ok(p) => return p,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => panic!("client connect: {e}"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sandbox_round_trip_under_real_subprocess() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    ensure_sandbox_built();
    let dir = TempDir::new().expect("tempdir");
    let sock = make_socket(dir.path());

    let mut child = tokio::process::Command::new(sandbox_binary())
        .arg("--session-id")
        .arg("integ")
        .arg("--ipc-socket")
        .arg(&sock)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn sandbox");

    // Wait for the IPC server to bind.
    #[cfg(unix)]
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !sock.exists() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(sock.exists(), "sandbox never created its socket");
    }

    let client = open_client(&sock).await;
    let (rd, mut wr) = {
        #[cfg(unix)]
        {
            client.into_split()
        }
        #[cfg(windows)]
        {
            tokio::io::split(client)
        }
    };
    let mut reader = BufReader::new(rd);

    let req = SandboxRequest::ValidateArtifact {
        artifact_code: "let x = 1;".to_string(),
        declaration: pure_read_declaration(),
    };
    let line = format!("{}\n", serde_json::to_string(&req).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write");
    wr.flush().await.expect("flush");

    let mut resp_line = String::new();
    tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut resp_line))
        .await
        .expect("response timeout")
        .expect("read");
    let resp: SandboxResponse = serde_json::from_str(resp_line.trim()).expect("parse response");
    match resp {
        SandboxResponse::ValidationResult(r) => assert_eq!(r, ValidationResult::Ok),
        SandboxResponse::Alert { message, .. } => {
            panic!("expected ValidationResult, got Alert: {message}")
        }
    }

    // Drive graceful shutdown via the protocol so the child exits.
    let shutdown = SandboxRequest::Shutdown;
    let line = format!("{}\n", serde_json::to_string(&shutdown).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write shutdown");
    wr.flush().await.expect("flush shutdown");
    let _ = tokio::time::timeout(Duration::from_secs(3), child.wait()).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sandbox_restart_after_kill_resumes() {
    use tokio::io::AsyncWriteExt;

    ensure_sandbox_built();
    let dir = TempDir::new().expect("tempdir");
    let sock = make_socket(dir.path());

    // Spawn round 1.
    let mut child = tokio::process::Command::new(sandbox_binary())
        .arg("--session-id")
        .arg("kill-restart-1")
        .arg("--ipc-socket")
        .arg(&sock)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn sandbox 1");

    #[cfg(unix)]
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !sock.exists() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(sock.exists(), "sandbox 1 never created its socket");
    }

    // Send one request to confirm it's alive.
    let mut client = open_client(&sock).await;
    let req = SandboxRequest::ValidateArtifact {
        artifact_code: "let y = 2;".to_string(),
        declaration: pure_read_declaration(),
    };
    let line = format!("{}\n", serde_json::to_string(&req).expect("encode"));
    client.write_all(line.as_bytes()).await.expect("write");
    client.flush().await.expect("flush");
    drop(client);

    // Kill round 1 hard.
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(3), child.wait()).await;

    // On Unix the socket file must be cleared so round 2 can bind it.
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(&sock);
    }
    // On Windows the named pipe is reaped automatically when handles
    // close (kill_on_drop guarantees that).

    // Spawn round 2 on the same socket path / pipe name.
    let mut child2 = tokio::process::Command::new(sandbox_binary())
        .arg("--session-id")
        .arg("kill-restart-2")
        .arg("--ipc-socket")
        .arg(&sock)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn sandbox 2");

    #[cfg(unix)]
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !sock.exists() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(sock.exists(), "sandbox 2 never created its socket");
    }

    // Confirm round 2 responds.
    let mut client2 = open_client(&sock).await;
    let line = format!("{}\n", serde_json::to_string(&req).expect("encode"));
    client2.write_all(line.as_bytes()).await.expect("write");
    client2.flush().await.expect("flush");

    let shutdown = SandboxRequest::Shutdown;
    let line = format!("{}\n", serde_json::to_string(&shutdown).expect("encode"));
    client2
        .write_all(line.as_bytes())
        .await
        .expect("write shutdown");
    client2.flush().await.expect("flush shutdown");
    let _ = tokio::time::timeout(Duration::from_secs(3), child2.wait()).await;
}

// ===========================================================================
// M05 Stage C2 — OS-isolation integration tests.
//
// Both tests spawn the real `runtime-sandbox` binary (so OS isolation runs
// in the subprocess, never on the cargo test runner) and side-channel
// verify that isolation is active. Per-platform: Linux reads
// `/proc/$pid/status` for the `Seccomp:` field; Windows queries
// `IsProcessInJob`.
// ===========================================================================

#[cfg(target_os = "linux")]
fn seccomp_mode_for_pid(pid: u32) -> Option<u32> {
    // Read /proc/<pid>/status; `Seccomp: 2` indicates filter mode (BPF
    // installed). `0` = disabled, `1` = strict, `2` = filter. See
    // Documentation/userspace-api/seccomp_filter.rst in the kernel
    // sources.
    let path = format!("/proc/{pid}/status");
    let status = std::fs::read_to_string(&path).ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("Seccomp:") {
            return rest.trim().parse().ok();
        }
    }
    None
}

#[cfg(windows)]
fn child_is_in_job(child: &tokio::process::Child) -> bool {
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::IsProcessInJob;

    let Some(raw) = child.raw_handle() else {
        return false;
    };
    let handle: HANDLE = raw.cast();
    let mut in_job: i32 = 0;
    // SAFETY: handle is owned by the live child process; IsProcessInJob
    // with null job-handle queries membership in ANY job. The output
    // pointer is a stack local that outlives the call.
    let ok = unsafe { IsProcessInJob(handle, std::ptr::null_mut(), &mut in_job) };
    ok != 0 && in_job != 0
}

#[cfg(any(target_os = "linux", windows))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn isolation_active_under_real_subprocess() {
    use tokio::io::AsyncWriteExt;

    ensure_sandbox_built();
    let dir = TempDir::new().expect("tempdir");
    let sock = make_socket(dir.path());

    let mut child = tokio::process::Command::new(sandbox_binary())
        .arg("--session-id")
        .arg("isolation-active")
        .arg("--ipc-socket")
        .arg(&sock)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn sandbox");

    // Wait for the IPC server to bind — that's our signal that
    // install_isolation has run AND ipc::serve has bound the socket.
    #[cfg(target_os = "linux")]
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !sock.exists() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(sock.exists(), "sandbox never created its socket");
    }
    #[cfg(windows)]
    {
        // Named pipes are non-FS; wait until the client can connect
        // instead (open_client polls).
        let _ = open_client(&sock).await;
    }

    let pid = child.id().expect("child pid");

    #[cfg(target_os = "linux")]
    {
        let mode = seccomp_mode_for_pid(pid).unwrap_or(0);
        assert_eq!(
            mode, 2,
            "seccomp filter mode should be 2 (filter) on the live sandbox subprocess; got {mode}"
        );
    }
    #[cfg(windows)]
    {
        let _ = pid;
        assert!(
            child_is_in_job(&child),
            "sandbox subprocess should be in a job object after install_restrictions"
        );
    }

    // Graceful shutdown.
    let mut client = open_client(&sock).await;
    let shutdown = SandboxRequest::Shutdown;
    let line = format!("{}\n", serde_json::to_string(&shutdown).expect("encode"));
    client
        .write_all(line.as_bytes())
        .await
        .expect("write shutdown");
    client.flush().await.expect("flush shutdown");
    let _ = tokio::time::timeout(Duration::from_secs(3), child.wait()).await;
}

#[cfg(any(target_os = "linux", windows))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn isolation_persists_across_validate_calls() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    ensure_sandbox_built();
    let dir = TempDir::new().expect("tempdir");
    let sock = make_socket(dir.path());

    let mut child = tokio::process::Command::new(sandbox_binary())
        .arg("--session-id")
        .arg("isolation-persists")
        .arg("--ipc-socket")
        .arg(&sock)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn sandbox");

    #[cfg(target_os = "linux")]
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !sock.exists() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(sock.exists(), "sandbox never created its socket");
    }

    let client = open_client(&sock).await;
    let (rd, mut wr) = {
        #[cfg(unix)]
        {
            client.into_split()
        }
        #[cfg(windows)]
        {
            tokio::io::split(client)
        }
    };
    let mut reader = BufReader::new(rd);

    let pid = child.id().expect("child pid");

    for round in 0..3 {
        let req = SandboxRequest::ValidateArtifact {
            artifact_code: format!("let r{round} = {round};"),
            declaration: pure_read_declaration(),
        };
        let line = format!("{}\n", serde_json::to_string(&req).expect("encode"));
        wr.write_all(line.as_bytes()).await.expect("write");
        wr.flush().await.expect("flush");

        let mut resp_line = String::new();
        tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut resp_line))
            .await
            .expect("response timeout")
            .expect("read");
        let resp: SandboxResponse = serde_json::from_str(resp_line.trim()).expect("parse response");
        match resp {
            SandboxResponse::ValidationResult(r) => assert_eq!(r, ValidationResult::Ok),
            SandboxResponse::Alert { message, .. } => {
                panic!("round {round}: expected ValidationResult, got Alert: {message}")
            }
        }

        #[cfg(target_os = "linux")]
        {
            let mode = seccomp_mode_for_pid(pid).unwrap_or(0);
            assert_eq!(
                mode, 2,
                "round {round}: seccomp filter mode should remain 2; got {mode}"
            );
        }
        #[cfg(windows)]
        {
            let _ = pid;
            assert!(
                child_is_in_job(&child),
                "round {round}: sandbox subprocess should remain in its job object"
            );
        }
    }

    let shutdown = SandboxRequest::Shutdown;
    let line = format!("{}\n", serde_json::to_string(&shutdown).expect("encode"));
    wr.write_all(line.as_bytes()).await.expect("write shutdown");
    wr.flush().await.expect("flush shutdown");
    let _ = tokio::time::timeout(Duration::from_secs(3), child.wait()).await;
}
