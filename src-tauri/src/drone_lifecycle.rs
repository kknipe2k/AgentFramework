//! Drone subprocess lifecycle (M04 Stage A2).
//!
//! Owns spawning the `runtime-drone` binary at Tauri startup, exposing
//! the [`Arc<DroneClient>`] as Tauri-managed state, and shutting the
//! subprocess down gracefully on app exit.
//!
//! # Test seam (CLAUDE.md §5 `*_with` archetype)
//!
//! [`DroneLifecycle::spawn_with`] takes injectable spawn + connect
//! closures so unit tests exercise the lifecycle invariants without
//! needing a real `runtime-drone` binary at test time. The production
//! [`DroneLifecycle::spawn`] composes [`DroneLifecycle::spawn_with`] with the real
//! [`tokio::process::Command`] and [`DroneClient::connect`]. The
//! production wrapper falls under the `tauri-shell` 50% patch gate in
//! `codecov.yml`; the seam is unit-tested at workspace ≥80%.
//!
//! # IPC addressing (cross-platform)
//!
//! - Unix: filesystem path `<temp>/runtime-drone-<session_id>.sock`
//! - Windows: named pipe `\\.\pipe\runtime-drone-<session_id>`
//!
//! Per `agent-runtime-spec.md` §1d.
//!
//! # Cleanup discipline
//!
//! The `tokio::process::Command::kill_on_drop(true)` flag is the
//! load-bearing failsafe: if the Tauri app crashes before
//! [`DroneLifecycle::shutdown`] runs, the OS still SIGKILLs the drone
//! when the [`tokio::process::Child`] handle is dropped. Without that
//! flag, a host crash leaves a zombie drone holding the `SQLite` WAL
//! lock — gotcha #29-class silent-failure mode in production.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use runtime_core::{CmdError, DroneCommand};
use runtime_main::drone_ipc::DroneClient;
use uuid::Uuid;

/// Number of `DroneClient::connect` attempts before surfacing
/// [`CmdError::Drone`] at the Tauri setup hook.
const CONNECT_MAX_RETRIES: u32 = 5;
/// Base backoff between connect attempts. Doubles each retry; cumulative
/// wait at `CONNECT_MAX_RETRIES` is `200 + 400 + 800 + 1600 = 3000ms`
/// (no sleep after the final attempt). Matches the `runtime-main` IPC
/// reconnect policy (200ms exp backoff, 5 attempts).
const CONNECT_BASE_BACKOFF: Duration = Duration::from_millis(200);
/// Time the graceful-shutdown handshake gets before the lifecycle
/// falls back to `Child::start_kill`. The drone's own
/// `GracefulShutdown` handler treats `timeout_ms` as a soft budget, so
/// this outer cap is twice that value.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);
/// Inner timeout the lifecycle ships with the `GracefulShutdown` command.
const GRACEFUL_TIMEOUT_MS: u64 = 1500;

/// Owns the spawned `runtime-drone` subprocess for an app session.
///
/// Held as Tauri-managed state via `app.manage(Mutex::new(Some(lc)))`.
/// On `RunEvent::ExitRequested`, the handler `take()`s the value and
/// calls [`Self::shutdown`] to flush + reap before propagating exit.
pub struct DroneLifecycle {
    child: tokio::process::Child,
    /// The IPC client connected to this lifecycle's drone. Cloned out
    /// at setup time and registered as Tauri-managed state for command
    /// access.
    pub client: Arc<DroneClient>,
    /// The IPC address (filesystem socket on Unix; named pipe on
    /// Windows). Used by [`Self::shutdown`] to delete the socket file
    /// post-exit on Unix.
    addr: String,
    session_id: String,
}

impl DroneLifecycle {
    /// Production wrapper. Locates the `runtime-drone` binary alongside
    /// the Tauri app's executable, spawns with `kill_on_drop(true)`,
    /// connects the IPC client with exponential-backoff retry, and
    /// returns the lifecycle.
    ///
    /// # Errors
    ///
    /// Returns [`CmdError::Drone`] if the binary cannot be located, the
    /// subprocess fails to spawn, or all connect attempts fail within
    /// the cumulative backoff window.
    pub async fn spawn(db_path: PathBuf) -> Result<Self, CmdError> {
        let session_id = Uuid::new_v4().to_string();
        let addr = compute_ipc_addr(&session_id);
        let bin = locate_drone_binary()?;
        Self::spawn_with(
            session_id,
            db_path,
            addr,
            move |args| async move {
                tokio::process::Command::new(&bin)
                    .args(&args)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .kill_on_drop(true)
                    .spawn()
                    .map_err(|e| CmdError::drone(format!("spawn drone subprocess: {e}")))
            },
            |addr| async move { connect_with_retry(&addr).await },
        )
        .await
    }

    /// Test seam. Constructs the lifecycle from caller-supplied spawn +
    /// connect closures — unit tests inject closures that synthesize a
    /// `Child` (e.g., wrapping `cargo` itself or an empty no-op binary)
    /// and a `DroneClient::noop()`, verifying the lifecycle's args
    /// composition + ordering without exec'ing `runtime-drone`.
    ///
    /// The closures take and return owned types so the seam can run
    /// in any task without lifetime gymnastics.
    ///
    /// # Errors
    ///
    /// Surfaces whatever `spawn_fn` or `connect_fn` returns.
    pub async fn spawn_with<S, SFut, C, CFut>(
        session_id: String,
        db_path: PathBuf,
        addr: String,
        spawn_fn: S,
        connect_fn: C,
    ) -> Result<Self, CmdError>
    where
        S: FnOnce(Vec<String>) -> SFut,
        SFut: std::future::Future<Output = Result<tokio::process::Child, CmdError>>,
        C: FnOnce(String) -> CFut,
        CFut: std::future::Future<Output = Result<DroneClient, CmdError>>,
    {
        tracing::info!(
            session_id = %session_id,
            addr = %addr,
            db_path = %db_path.display(),
            "drone lifecycle spawn_with starting"
        );
        let args = vec![
            "--session-id".to_string(),
            session_id.clone(),
            "--db-path".to_string(),
            db_path.to_string_lossy().into_owned(),
            "--ipc-socket".to_string(),
            addr.clone(),
        ];
        let child = spawn_fn(args).await?;
        let pid = child.id();
        let client = connect_fn(addr.clone()).await?;
        tracing::info!(pid = ?pid, "drone subprocess spawned and connected");
        Ok(Self {
            child,
            client: Arc::new(client),
            addr,
            session_id,
        })
    }

    /// Send a [`DroneCommand::GracefulShutdown`], await the child to
    /// exit within [`SHUTDOWN_TIMEOUT`], and fall back to
    /// `Child::start_kill` on timeout. Always cleans up the IPC socket
    /// file (Unix) before returning.
    ///
    /// # Errors
    ///
    /// Never errors — graceful failure paths are logged via
    /// `tracing::warn!` and the function still cleans up. The `Result`
    /// return is for forward-compat with explicit error surfaces.
    pub async fn shutdown(mut self) -> Result<(), CmdError> {
        tracing::info!(session_id = %self.session_id, "drone shutdown initiated");
        // Best-effort graceful: the drone may already be dead, or the
        // IPC pipe may already be torn down. Either way we proceed to
        // the wait + kill fallback.
        if let Err(e) = self
            .client
            .send(DroneCommand::GracefulShutdown {
                timeout_ms: GRACEFUL_TIMEOUT_MS,
            })
            .await
        {
            tracing::warn!(error = %e, "graceful shutdown command send failed; proceeding to wait/kill");
        }
        match tokio::time::timeout(SHUTDOWN_TIMEOUT, self.child.wait()).await {
            Ok(Ok(status)) => {
                tracing::info!(status = ?status, "drone exited cleanly");
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "drone wait failed; child may have been reaped externally");
            }
            Err(_) => {
                tracing::warn!("drone graceful shutdown timed out; sending SIGKILL");
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

/// Locate the `runtime-drone` binary alongside the Tauri app's executable.
///
/// Archetype follows `crates/runtime-drone/tests/integration.rs::drone_binary`
/// and `crates/runtime-main/tests/drone_ipc_loopback.rs::drone_binary`.
/// Using `current_exe()` works under both `cargo run` (target/debug/) and
/// `cargo llvm-cov` (`target/llvm-cov-target/`...) per gotcha #22.
fn locate_drone_binary() -> Result<PathBuf, CmdError> {
    let mut p =
        std::env::current_exe().map_err(|e| CmdError::drone(format!("current_exe failed: {e}")))?;
    p.pop(); // drop the current exe filename
    if p.ends_with("deps") {
        p.pop(); // up to the profile dir (cargo test layout)
    }
    #[cfg(windows)]
    p.push("runtime-drone.exe");
    #[cfg(unix)]
    p.push("runtime-drone");
    Ok(p)
}

/// Compose the IPC address for a session. Cross-platform divergence is
/// captured here so spawn/connect callers stay portable.
#[must_use]
pub fn compute_ipc_addr(session_id: &str) -> String {
    #[cfg(windows)]
    {
        format!(r"\\.\pipe\runtime-drone-{session_id}")
    }
    #[cfg(unix)]
    {
        std::env::temp_dir()
            .join(format!("runtime-drone-{session_id}.sock"))
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

async fn connect_with_retry(addr: &str) -> Result<DroneClient, CmdError> {
    let mut backoff = CONNECT_BASE_BACKOFF;
    let mut last_err: Option<String> = None;
    for attempt in 0..CONNECT_MAX_RETRIES {
        match DroneClient::connect(addr).await {
            Ok(c) => {
                if attempt > 0 {
                    tracing::info!(attempt, "drone connected after retry");
                }
                return Ok(c);
            }
            Err(e) => {
                let msg = e.to_string();
                tracing::warn!(attempt, error = %msg, "drone connect attempt failed");
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
        "drone connect failed after {} attempts: {}",
        CONNECT_MAX_RETRIES,
        last_err.unwrap_or_else(|| "no error captured".to_string())
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Spawn a tokio Child wrapping a no-op subprocess so the test seam
    /// can populate a real [`tokio::process::Child`] without needing the
    /// `runtime-drone` binary on disk.
    ///
    /// Cross-platform pick: on Unix invoke `true`; on Windows invoke
    /// `cmd /C exit 0`. Both exit immediately with code 0 — the
    /// lifecycle's spawn step doesn't itself await the child, so the
    /// quick exit doesn't affect spawn-time assertions.
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
    async fn spawn_with_returns_lifecycle_holding_arc_drone_client() {
        let captured_args = std::sync::Arc::new(std::sync::Mutex::new(None));
        let captured_clone = std::sync::Arc::clone(&captured_args);
        let lc = DroneLifecycle::spawn_with(
            "sid-test".to_string(),
            PathBuf::from("/tmp/test.sqlite"),
            "/tmp/test.sock".to_string(),
            move |args| async move {
                *captured_clone.lock().unwrap() = Some(args);
                Ok(spawn_noop_child())
            },
            |_addr| async move { Ok(DroneClient::noop()) },
        )
        .await
        .expect("spawn_with");

        // The Arc<DroneClient> must be cloneable for Tauri managed-state
        // registration; verify ref-count semantics.
        let cloned = Arc::clone(&lc.client);
        assert!(Arc::strong_count(&lc.client) >= 2);
        drop(cloned);

        // Args composition is the wire contract with the runtime-drone
        // CLI; assert exact ordering + values.
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
                "--db-path".to_string(),
                PathBuf::from("/tmp/test.sqlite")
                    .to_string_lossy()
                    .into_owned(),
                "--ipc-socket".to_string(),
                "/tmp/test.sock".to_string(),
            ]
        );

        // shutdown should not error on a noop client + already-exited
        // subprocess (the noop child returns immediately).
        let _ = lc.shutdown().await;
    }

    #[tokio::test]
    async fn spawn_with_propagates_spawn_failure() {
        let result = DroneLifecycle::spawn_with(
            "sid-fail".to_string(),
            PathBuf::from("/tmp/fail.sqlite"),
            "/tmp/fail.sock".to_string(),
            |_args| async move { Err(CmdError::drone("spawn failed")) },
            |_addr| async move { Ok(DroneClient::noop()) },
        )
        .await;
        // DroneLifecycle holds tokio::process::Child which is !Debug, so
        // .expect_err / unwrap_err can't print the Ok side. Use let-else.
        let Err(err) = result else {
            panic!("expected spawn error, got Ok")
        };
        assert!(matches!(err, CmdError::Drone(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn spawn_with_propagates_connect_failure() {
        let result = DroneLifecycle::spawn_with(
            "sid-noconn".to_string(),
            PathBuf::from("/tmp/x.sqlite"),
            "/tmp/x.sock".to_string(),
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
        // The lifecycle's shutdown path should be no-throw when:
        //   (a) the IPC client is a noop (graceful command short-circuits to Ok)
        //   (b) the child process has already exited (wait returns immediately)
        // This is the common production path on graceful app exit when
        // the drone has already completed its work.
        let lc = DroneLifecycle::spawn_with(
            "sid-shutdown".to_string(),
            PathBuf::from("/tmp/sd.sqlite"),
            "/tmp/sd.sock".to_string(),
            |_args| async move { Ok(spawn_noop_child()) },
            |_addr| async move { Ok(DroneClient::noop()) },
        )
        .await
        .expect("spawn");

        // Give the noop child a moment to exit so wait() returns the
        // exit status path rather than the kill-fallback path. Using
        // sleep here rather than `wait` because we want shutdown to
        // observe the already-exited state.
        tokio::time::sleep(Duration::from_millis(50)).await;

        lc.shutdown().await.expect("shutdown");
    }

    #[test]
    fn compute_ipc_addr_uses_session_id_in_path() {
        // Cross-platform sanity: the address must include the session_id
        // and must follow the platform's IPC addressing convention.
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
        // The IPC socket may have already been cleaned up by the drone
        // itself on graceful exit; cleanup_socket must not log a warn
        // for that case (which is the ENOENT branch).
        let path = std::env::temp_dir().join(format!(
            "runtime-drone-cleanup-test-{}.sock",
            Uuid::new_v4()
        ));
        // The file does not exist; this should be a no-op (no panic).
        cleanup_socket(&path.to_string_lossy());
    }

    #[test]
    fn cmd_error_drone_message_carries_through() {
        // Sanity: the spawn path's error wrapping uses CmdError::drone()
        // helpers, which preserve the underlying message. Tracing logs
        // and test assertions on `.message()` rely on this.
        let e = CmdError::drone("spawn drone subprocess: not found");
        assert!(matches!(e, CmdError::Drone(_)));
        assert_eq!(e.message(), Some("spawn drone subprocess: not found"));
    }
}
