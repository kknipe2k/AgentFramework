//! `runtime-sandbox` — §8.security L3 sandbox subprocess.
//!
//! Cross-platform plumbing in M05 Stage C1: binary entry point + framed-
//! JSON IPC + pure-function validator. Stage C2 layers OS-level isolation
//! (seccomp / landlock on Linux; Job Objects on Windows) on top of the
//! `run` entry point.
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

pub use error::{IpcError, SandboxError};
pub use protocol::{AlertLevel, SandboxRequest, SandboxResponse};
pub use validator::{validate, Artifact, DetectedSyscall, ValidationResult};

use std::path::PathBuf;

/// Run the sandbox subprocess: bind the IPC socket / pipe, accept a
/// connection from main, and loop handling requests until a
/// [`SandboxRequest::Shutdown`] arrives or the connection drops.
///
/// # Errors
///
/// Returns [`SandboxError::Ipc`] if the socket cannot be bound or the
/// IPC server task surfaces a fatal accept error.
pub async fn run(session_id: String, ipc_socket: PathBuf) -> Result<(), SandboxError> {
    run_inner(session_id, ipc_socket, shutdown_signal_future()).await
}

/// Test-friendly variant. Takes an injectable shutdown future so unit
/// tests can drive the subprocess loop without installing OS handlers.
///
/// # Errors
///
/// Returns [`SandboxError::Ipc`] if the underlying IPC server fails.
pub async fn run_inner<F>(
    session_id: String,
    ipc_socket: PathBuf,
    shutdown_source: F,
) -> Result<(), SandboxError>
where
    F: std::future::Future<Output = &'static str>,
{
    tracing::info!(session_id = %session_id, path = %ipc_socket.display(), "sandbox ipc starting");
    let serve_fut = ipc::serve(ipc_socket);
    tokio::select! {
        result = serve_fut => result.map_err(SandboxError::from),
        reason = shutdown_source => {
            tracing::info!(reason, "sandbox shutdown signal received");
            Ok(())
        }
    }
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

    #[test]
    fn sandbox_error_wraps_ipc_io() {
        let io = std::io::Error::new(std::io::ErrorKind::AddrInUse, "x");
        let ipc = IpcError::Io(io);
        let top: SandboxError = ipc.into();
        assert!(matches!(top, SandboxError::Ipc(_)));
    }
}
