//! Sandbox subprocess lifecycle (M05 Stage C1).
//!
//! Owns spawning the `runtime-sandbox` binary at Tauri startup, exposing
//! the [`Arc<SandboxClient>`] as Tauri-managed state, and shutting the
//! subprocess down gracefully on app exit. Mirrors the drone lifecycle
//! shape (M04 Stage A2) exactly — same `spawn_with` test seam, same
//! `compute_ipc_addr` cross-platform routine, same `kill_on_drop`
//! failsafe.
//!
//! # Test seam (CLAUDE.md §5 `*_with` archetype)
//!
//! [`SandboxLifecycle::spawn_with`] takes injectable spawn + connect
//! closures so unit tests exercise the lifecycle invariants without
//! needing a real `runtime-sandbox` binary on disk.
//!
//! # IPC addressing (cross-platform)
//!
//! - Unix: filesystem path `<temp>/runtime-sandbox-<session_id>.sock`
//! - Windows: named pipe `\\.\pipe\runtime-sandbox-<session_id>`
//!
//! Per `agent-runtime-spec.md` §1d.
//!
//! # Cleanup discipline
//!
//! `tokio::process::Command::kill_on_drop(true)` is the failsafe — if the
//! Tauri app crashes before [`SandboxLifecycle::shutdown`] runs, the OS
//! still SIGKILLs the sandbox when the [`tokio::process::Child`] handle
//! is dropped.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use runtime_core::CmdError;
use runtime_main::sandbox_ipc::SandboxClient;
use uuid::Uuid;

/// Number of `SandboxClient::connect` attempts before surfacing
/// [`CmdError::Drone`] at the Tauri setup hook. (Reuses the `Drone`
/// variant for parity with drone errors; v0.1 keeps `CmdError` minimal —
/// a dedicated `Sandbox` variant lands when M09 wires the production
/// caller and surfaces become user-facing.)
const CONNECT_MAX_RETRIES: u32 = 5;
/// Base backoff between connect attempts. Cumulative wait at
/// `CONNECT_MAX_RETRIES` is `200 + 400 + 800 + 1600 = 3000ms`.
const CONNECT_BASE_BACKOFF: Duration = Duration::from_millis(200);
/// Outer cap on graceful shutdown before falling back to `start_kill`.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

/// Owns the spawned `runtime-sandbox` subprocess for an app session.
///
/// Held as Tauri-managed state via `app.manage(Mutex::new(Some(lc)))`.
/// On `RunEvent::ExitRequested`, the handler `take()`s the value and
/// calls [`Self::shutdown`] before propagating exit.
pub struct SandboxLifecycle {
    child: tokio::process::Child,
    /// The IPC client connected to this lifecycle's sandbox. Cloned out
    /// at setup time and registered as Tauri-managed state for command
    /// access.
    pub client: Arc<SandboxClient>,
    /// The IPC address. Used by [`Self::shutdown`] to delete the socket
    /// file post-exit on Unix.
    addr: String,
    session_id: String,
}

impl SandboxLifecycle {
    /// Production wrapper. Locates the `runtime-sandbox` binary
    /// alongside the Tauri app's executable, spawns with
    /// `kill_on_drop(true)`, connects the IPC client with
    /// exponential-backoff retry, and returns the lifecycle.
    ///
    /// # Errors
    ///
    /// Returns [`CmdError::Drone`] if the binary cannot be located, the
    /// subprocess fails to spawn, or all connect attempts fail.
    pub async fn spawn() -> Result<Self, CmdError> {
        let session_id = Uuid::new_v4().to_string();
        let addr = compute_ipc_addr(&session_id);
        let bin = locate_sandbox_binary()?;
        Self::spawn_with(
            session_id,
            addr,
            move |args| async move {
                tokio::process::Command::new(&bin)
                    .args(&args)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .kill_on_drop(true)
                    .spawn()
                    .map_err(|e| CmdError::drone(format!("spawn sandbox subprocess: {e}")))
            },
            |addr| async move { connect_with_retry(&addr).await },
        )
        .await
    }

    /// Test seam. Constructs the lifecycle from caller-supplied spawn +
    /// connect closures.
    ///
    /// # Errors
    ///
    /// Surfaces whatever `spawn_fn` or `connect_fn` returns.
    pub async fn spawn_with<S, SFut, C, CFut>(
        session_id: String,
        addr: String,
        spawn_fn: S,
        connect_fn: C,
    ) -> Result<Self, CmdError>
    where
        S: FnOnce(Vec<String>) -> SFut,
        SFut: std::future::Future<Output = Result<tokio::process::Child, CmdError>>,
        C: FnOnce(String) -> CFut,
        CFut: std::future::Future<Output = Result<SandboxClient, CmdError>>,
    {
        tracing::info!(
            session_id = %session_id,
            addr = %addr,
            "sandbox lifecycle spawn_with starting"
        );
        let args = vec![
            "--session-id".to_string(),
            session_id.clone(),
            "--ipc-socket".to_string(),
            addr.clone(),
        ];
        let child = spawn_fn(args).await?;
        let pid = child.id();
        let client = connect_fn(addr.clone()).await?;
        tracing::info!(pid = ?pid, "sandbox subprocess spawned and connected");
        Ok(Self {
            child,
            client: Arc::new(client),
            addr,
            session_id,
        })
    }

    /// Send a [`runtime_main::sandbox_ipc::SandboxClient::shutdown`] and
    /// await the child to exit within [`SHUTDOWN_TIMEOUT`]; fall back
    /// to `Child::start_kill` on timeout. Always cleans up the IPC
    /// socket file (Unix) before returning.
    ///
    /// # Errors
    ///
    /// Never errors — graceful failure paths are logged via
    /// `tracing::warn!` and the function still cleans up.
    pub async fn shutdown(mut self) -> Result<(), CmdError> {
        tracing::info!(session_id = %self.session_id, "sandbox shutdown initiated");
        if let Err(e) = self.client.shutdown().await {
            tracing::warn!(error = %e, "graceful shutdown command send failed; proceeding to wait/kill");
        }
        match tokio::time::timeout(SHUTDOWN_TIMEOUT, self.child.wait()).await {
            Ok(Ok(status)) => {
                tracing::info!(status = ?status, "sandbox exited cleanly");
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "sandbox wait failed; child may have been reaped externally");
            }
            Err(_) => {
                tracing::warn!("sandbox graceful shutdown timed out; sending SIGKILL");
                if let Err(e) = self.child.start_kill() {
                    tracing::warn!(error = %e, "start_kill failed");
                }
                if let Err(e) = self.child.wait().await {
                    tracing::warn!(error = %e, "post-kill wait failed");
                }
            }
        }
        cleanup_socket(&self.addr);
        Ok(())
    }
}

/// Locate the `runtime-sandbox` binary alongside the Tauri app's
/// executable. Same `current_exe()` derivation as
/// `drone_lifecycle::locate_drone_binary` per gotcha #22.
fn locate_sandbox_binary() -> Result<PathBuf, CmdError> {
    let mut p =
        std::env::current_exe().map_err(|e| CmdError::drone(format!("current_exe failed: {e}")))?;
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    #[cfg(windows)]
    p.push("runtime-sandbox.exe");
    #[cfg(unix)]
    p.push("runtime-sandbox");
    Ok(p)
}

/// Compose the IPC address for a session.
#[must_use]
pub fn compute_ipc_addr(session_id: &str) -> String {
    #[cfg(windows)]
    {
        format!(r"\\.\pipe\runtime-sandbox-{session_id}")
    }
    #[cfg(unix)]
    {
        std::env::temp_dir()
            .join(format!("runtime-sandbox-{session_id}.sock"))
            .to_string_lossy()
            .into_owned()
    }
}

#[cfg(unix)]
fn cleanup_socket(addr: &str) {
    if let Err(e) = std::fs::remove_file(addr) {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(addr, error = %e, "cleanup_socket: remove_file failed");
        }
    }
}

#[cfg(windows)]
const fn cleanup_socket(_addr: &str) {
    // Windows named pipes are not filesystem entries; the OS reaps them
    // when the last handle closes (which `kill_on_drop` guarantees).
}

async fn connect_with_retry(addr: &str) -> Result<SandboxClient, CmdError> {
    let mut backoff = CONNECT_BASE_BACKOFF;
    let mut last_err: Option<String> = None;
    for attempt in 0..CONNECT_MAX_RETRIES {
        match SandboxClient::connect(addr).await {
            Ok(c) => {
                if attempt > 0 {
                    tracing::info!(attempt, "sandbox connected after retry");
                }
                return Ok(c);
            }
            Err(e) => {
                let msg = e.to_string();
                tracing::warn!(attempt, error = %msg, "sandbox connect attempt failed");
                last_err = Some(msg);
                if attempt + 1 == CONNECT_MAX_RETRIES {
                    break;
                }
                tokio::time::sleep(backoff).await;
                backoff *= 2;
            }
        }
    }
    Err(CmdError::drone(format!(
        "sandbox connect failed after {} attempts: {}",
        CONNECT_MAX_RETRIES,
        last_err.unwrap_or_else(|| "no error captured".to_string())
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_noop_child() -> tokio::process::Child {
        #[cfg(windows)]
        let mut cmd = tokio::process::Command::new("cmd");
        #[cfg(windows)]
        cmd.args(["/C", "exit 0"]);

        #[cfg(unix)]
        let mut cmd = tokio::process::Command::new("true");

        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("spawn noop child")
    }

    #[tokio::test]
    async fn spawn_with_returns_lifecycle_holding_arc_sandbox_client() {
        let captured_args = std::sync::Arc::new(std::sync::Mutex::new(None));
        let captured_clone = std::sync::Arc::clone(&captured_args);
        let lc = SandboxLifecycle::spawn_with(
            "sid-test".to_string(),
            "/tmp/sb-test.sock".to_string(),
            move |args| async move {
                *captured_clone.lock().unwrap() = Some(args);
                Ok(spawn_noop_child())
            },
            |_addr| async move { Ok(SandboxClient::noop()) },
        )
        .await
        .expect("spawn_with");

        let cloned = Arc::clone(&lc.client);
        assert!(Arc::strong_count(&lc.client) >= 2);
        drop(cloned);

        let args = captured_args
            .lock()
            .unwrap()
            .clone()
            .expect("spawn_fn was called");
        assert_eq!(
            args,
            vec![
                "--session-id".to_string(),
                "sid-test".to_string(),
                "--ipc-socket".to_string(),
                "/tmp/sb-test.sock".to_string(),
            ]
        );

        let _ = lc.shutdown().await;
    }

    #[tokio::test]
    async fn spawn_with_propagates_spawn_failure() {
        let result = SandboxLifecycle::spawn_with(
            "sid-fail".to_string(),
            "/tmp/sb-fail.sock".to_string(),
            |_args| async move { Err(CmdError::drone("spawn failed")) },
            |_addr| async move { Ok(SandboxClient::noop()) },
        )
        .await;
        let Err(err) = result else {
            panic!("expected spawn error, got Ok")
        };
        assert!(matches!(err, CmdError::Drone(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn spawn_with_propagates_connect_failure() {
        let result = SandboxLifecycle::spawn_with(
            "sid-noconn".to_string(),
            "/tmp/sb-x.sock".to_string(),
            |_args| async move { Ok(spawn_noop_child()) },
            |_addr| async move { Err(CmdError::drone("connect refused")) },
        )
        .await;
        let Err(err) = result else {
            panic!("expected connect error, got Ok")
        };
        assert!(matches!(err, CmdError::Drone(_)), "got {err:?}");
        assert!(err.message().is_some_and(|m| m.contains("connect")));
    }

    #[tokio::test]
    async fn shutdown_completes_for_noop_client_and_already_exited_child() {
        let lc = SandboxLifecycle::spawn_with(
            "sid-shutdown".to_string(),
            "/tmp/sb-sd.sock".to_string(),
            |_args| async move { Ok(spawn_noop_child()) },
            |_addr| async move { Ok(SandboxClient::noop()) },
        )
        .await
        .expect("spawn");

        tokio::time::sleep(Duration::from_millis(50)).await;
        lc.shutdown().await.expect("shutdown");
    }

    #[test]
    fn compute_ipc_addr_uses_session_id_in_path() {
        let addr = compute_ipc_addr("abc-123");
        assert!(addr.contains("abc-123"), "got {addr}");
        #[cfg(windows)]
        assert!(addr.starts_with(r"\\.\pipe\"), "got {addr}");
        #[cfg(unix)]
        assert!(
            std::path::Path::new(&addr)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sock")),
            "got {addr}"
        );
    }

    #[test]
    fn compute_ipc_addr_is_unique_per_session_id() {
        let a = compute_ipc_addr("s1");
        let b = compute_ipc_addr("s2");
        assert_ne!(a, b);
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_socket_silently_ignores_missing_file() {
        let path = std::env::temp_dir().join(format!(
            "runtime-sandbox-cleanup-test-{}.sock",
            Uuid::new_v4()
        ));
        cleanup_socket(&path.to_string_lossy());
    }
}
