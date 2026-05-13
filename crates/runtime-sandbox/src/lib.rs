//! `runtime-sandbox` — §8.security L3 sandbox subprocess.
//!
//! Cross-platform plumbing in M05 Stage C1: binary entry point + framed-
//! JSON IPC + pure-function validator. Stage C2 layers OS-level isolation
//! on top of the `run` entry point:
//!
//! - **Linux:** seccomp BPF allowlist (`seccomp` module) + landlock
//!   filesystem fence (`landlock` module). Both install via
//!   `restrict_self`-class calls that affect the calling process for
//!   the remainder of its lifetime.
//! - **Windows:** Job Objects (`job_objects` module) for process-tree
//!   containment (`KILL_ON_JOB_CLOSE` + `BREAKAWAY_OK`).
//!
//! Bare backticks rather than intra-doc links because the named
//! modules are cfg-gated per platform (gotcha #55).
//!
//! Isolation installs ONCE at sandbox subprocess startup, BEFORE
//! `ipc::serve` binds the socket. The seccomp allowlist accommodates
//! the syscalls `ipc::serve` needs for bind / accept / read / write
//! (per `seccomp::ALLOWED_SYSCALLS` — bare backticks per gotcha #55
//! because `seccomp` is cfg-gated to Linux); landlock's filesystem
//! fence allows read+write under the socket's parent directory only.
//!
//! Subprocess lifetime is the app session (single subprocess spawned by
//! the Tauri main process at startup; shut down at app exit). Per spec
//! §8.security L3 the validator is pure — no IO, no platform branches.
//!
//! ## Wire protocol
//!
//! `LinesCodec`-framed JSON over Unix domain socket / Windows named pipe.
//! Request: [`SandboxRequest`]. Response: [`SandboxResponse`]. Strict
//! request-response — every `validate_artifact` request produces exactly
//! one response. `shutdown` produces no response; the subprocess exits.

pub mod error;
pub mod ipc;
pub mod protocol;
pub mod validator;

#[cfg(windows)]
pub mod job_objects;
#[cfg(target_os = "linux")]
pub mod landlock;
#[cfg(target_os = "linux")]
pub mod seccomp;

pub use error::{IpcError, SandboxError};
pub use protocol::{AlertLevel, SandboxRequest, SandboxResponse};
pub use validator::{validate, Artifact, DetectedSyscall, ValidationResult};

use std::path::PathBuf;

/// Run the sandbox subprocess until shutdown.
///
/// Installs OS isolation, binds the IPC socket / pipe, accepts a
/// connection from main, and loops handling requests until a
/// [`SandboxRequest::Shutdown`] arrives or the connection drops.
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if OS isolation cannot be
/// installed (seccomp / landlock / Job Objects); [`SandboxError::Ipc`]
/// if the socket cannot be bound or the IPC server task surfaces a
/// fatal accept error.
pub async fn run(session_id: String, ipc_socket: PathBuf) -> Result<(), SandboxError> {
    run_inner(session_id, ipc_socket, shutdown_signal_future()).await
}

/// Test-friendly variant. Takes an injectable shutdown future so unit
/// tests can drive the subprocess loop without installing OS handlers.
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if OS isolation install fails;
/// [`SandboxError::Ipc`] if the underlying IPC server fails.
pub async fn run_inner<F>(
    session_id: String,
    ipc_socket: PathBuf,
    shutdown_source: F,
) -> Result<(), SandboxError>
where
    F: std::future::Future<Output = &'static str>,
{
    tracing::info!(
        session_id = %session_id,
        path = %ipc_socket.display(),
        "sandbox starting"
    );
    install_isolation(&ipc_socket)?;
    let serve_fut = ipc::serve(ipc_socket);
    tokio::select! {
        result = serve_fut => result.map_err(SandboxError::from),
        reason = shutdown_source => {
            tracing::info!(reason, "sandbox shutdown signal received");
            Ok(())
        }
    }
}

/// Install the per-platform OS isolation primitives:
///
/// - Linux: landlock first (filesystem fence — allows R+W on the
///   socket's parent dir, denies the rest) THEN seccomp (syscall
///   allowlist). Order matters because landlock's `restrict_self` must
///   run with file-open syscalls still permitted, and seccomp's
///   allowlist accommodates the post-install syscalls landlock + the
///   IPC server need.
/// - Windows: Job Object with `KILL_ON_JOB_CLOSE` + `BREAKAWAY_OK`.
///
/// The function is `#[allow(unused_variables)]`-tolerant on platforms
/// that lack a particular fence; on macOS (not a v0.1 target) the
/// function is a no-op pending a sandbox-exec wrapper.
fn install_isolation(ipc_socket: &std::path::Path) -> Result<(), SandboxError> {
    #[cfg(target_os = "linux")]
    {
        let socket_parent = ipc_socket
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("/tmp"));
        if !socket_parent.exists() {
            std::fs::create_dir_all(socket_parent).map_err(|e| {
                SandboxError::Isolation(format!(
                    "landlock pre-create {}: {e}",
                    socket_parent.display()
                ))
            })?;
        }
        landlock::install(&[socket_parent])?;
        seccomp::install()?;
    }
    #[cfg(windows)]
    {
        let _ = ipc_socket; // socket-path scoping is N/A on Windows JO
        job_objects::install_restrictions()?;
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = ipc_socket;
        tracing::warn!("OS isolation not implemented on this platform; sandbox runs unfenced");
    }
    Ok(())
}

#[cfg(unix)]
async fn shutdown_signal_future() -> &'static str {
    use tokio::signal::unix::{signal, SignalKind};
    let mut term = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    let mut int = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    tokio::select! {
        _ = term.recv() => "sigterm",
        _ = int.recv()  => "sigint",
    }
}

#[cfg(windows)]
async fn shutdown_signal_future() -> &'static str {
    use tokio::signal::windows::{ctrl_break, ctrl_c};
    let mut br = ctrl_break().expect("install ctrl_break handler");
    let mut ci = ctrl_c().expect("install ctrl_c handler");
    tokio::select! {
        _ = br.recv() => "ctrl_break",
        _ = ci.recv() => "ctrl_c",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn temp_socket_path() -> PathBuf {
        #[cfg(unix)]
        {
            let dir = tempfile::TempDir::new().expect("tempdir");
            let p = dir.path().join("sb.sock");
            std::mem::forget(dir);
            p
        }
        #[cfg(windows)]
        {
            let suffix = uuid::Uuid::new_v4();
            PathBuf::from(format!(r"\\.\pipe\sandbox-run-test-{suffix}"))
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_inner_exits_on_shutdown_signal() {
        // Inject an immediately-ready shutdown future; run_inner should
        // pick that branch over the bind-and-serve branch and return Ok.
        //
        // On Linux, `install_isolation` would install seccomp+landlock
        // on the cargo test runner — disastrous for subsequent tests.
        // We skip this test on Linux; the integration test in
        // `tests/integration.rs` covers the install path via a real
        // subprocess.
        #[cfg(target_os = "linux")]
        {
            eprintln!("skipped on linux: install_isolation would poison test runner");
            return;
        }
        #[cfg(not(target_os = "linux"))]
        {
            let socket = temp_socket_path();
            let signal = async { "shutdown" };
            let result = tokio::time::timeout(
                Duration::from_secs(3),
                run_inner("sid-test".to_string(), socket, signal),
            )
            .await
            .expect("run_inner did not return");
            result.expect("run_inner returned an error");
        }
    }

    #[test]
    fn sandbox_error_wraps_ipc_io() {
        let io = std::io::Error::new(std::io::ErrorKind::AddrInUse, "x");
        let ipc = IpcError::Io(io);
        let top: SandboxError = ipc.into();
        assert!(matches!(top, SandboxError::Ipc(_)));
    }

    #[test]
    fn isolation_variant_carries_message() {
        let e = SandboxError::Isolation("test reason".to_string());
        assert!(format!("{e}").contains("test reason"));
    }
}
