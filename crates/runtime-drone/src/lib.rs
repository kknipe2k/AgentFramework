//! Drone library — exposes orchestration for binary + tests.
//!
//! Implements spec §1 (The Drone): heartbeat, snapshot, IPC server, signal
//! handling. The `run` function ties them together; see `main.rs` for the
//! binary entry point.

pub mod command_handler;
pub mod db;
pub mod heartbeat;
pub mod ipc;
/// Plan + Task projection — drone-internal continuous projector.
///
/// Consumes plan/task signals and UPSERTs `plans` + `tasks` rows.
/// Parallel to [`vdr`] in architecture (M03.E archetype). M04 Stage B.
pub mod plan_projector;
pub mod shutdown;
pub mod snapshot;
/// `token_usage` projection — drone-internal continuous projector.
///
/// Consumes `token_usage` signals (the multi-turn agent-with-tools
/// loop's per-turn usage report) and INSERTs `token_usage` rows.
/// Parallel to [`vdr`] + [`plan_projector`] (M07.D2, ADR-0011 d;
/// closes the M06.5 `token_usage = 0` finding).
pub mod token_usage;
pub mod vdr;

use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};

/// Heartbeat interval per `agent-runtime-spec.md` §1 (Heartbeat).
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Top-level drone error.
#[derive(Debug, Error)]
pub enum DroneError {
    /// Database error.
    #[error(transparent)]
    Db(#[from] db::DbError),

    /// IPC server error.
    #[error(transparent)]
    Ipc(#[from] ipc::IpcError),

    /// Snapshot writer error.
    #[error(transparent)]
    Snapshot(#[from] snapshot::SnapshotError),

    /// Shutdown handler error.
    #[error(transparent)]
    Shutdown(#[from] shutdown::ShutdownError),

    /// Heartbeat task error.
    #[error(transparent)]
    Heartbeat(#[from] heartbeat::HeartbeatError),
}

/// Run the drone main loop until shutdown or fatal error.
///
/// Spawns the heartbeat, IPC server, and command-handler tasks, then awaits
/// a shutdown signal. On shutdown, the IPC server and heartbeat are
/// aborted; the shutdown handler writes a final snapshot before returning.
///
/// # Errors
///
/// Returns `DroneError` if database initialization or the shutdown handler
/// fail.
pub async fn run(
    session_id: String,
    db_path: PathBuf,
    ipc_socket: PathBuf,
) -> Result<(), DroneError> {
    let conn = bootstrap(&session_id, &db_path)?;
    run_inner(conn, session_id, ipc_socket, shutdown_signal_future()).await
}

/// Test-friendly variant of `run`.
///
/// Takes an already-bootstrapped connection and an injectable shutdown
/// future. Used by both production (with the OS signal future) and unit
/// tests (with a deterministic future).
///
/// # Errors
///
/// Returns `DroneError` if the shutdown handler fails.
pub async fn run_inner<F>(
    conn: Arc<Mutex<rusqlite::Connection>>,
    session_id: String,
    ipc_socket: PathBuf,
    shutdown_source: F,
) -> Result<(), DroneError>
where
    F: Future<Output = &'static str>,
{
    let (event_tx, _event_rx) = broadcast::channel(64);
    let (cmd_tx, cmd_rx) = mpsc::channel(64);
    let (ipc_shutdown_tx, ipc_shutdown_rx) = oneshot::channel::<&'static str>();

    let hb_handle = tokio::spawn(heartbeat::run(
        session_id.clone(),
        conn.clone(),
        event_tx.clone(),
    ));
    let ipc_handle = tokio::spawn(ipc::serve(ipc_socket, cmd_tx, event_tx.clone()));
    let ch_handle = tokio::spawn(command_handler::run(
        session_id.clone(),
        conn.clone(),
        cmd_rx,
        event_tx.clone(),
        Some(ipc_shutdown_tx),
    ));

    let combined_shutdown = async move {
        tokio::select! {
            reason = shutdown_source => reason,
            ipc = ipc_shutdown_rx => ipc.unwrap_or("ipc_graceful"),
        }
    };

    shutdown::wait_and_handle_with(combined_shutdown, conn, session_id, event_tx).await?;

    hb_handle.abort();
    ipc_handle.abort();
    ch_handle.abort();
    Ok(())
}

/// Open the database at `db_path`, ensure a row exists in `sessions` for
/// `session_id`, and return the wrapped connection.
///
/// Extracted from `run` so the bootstrap path is unit-testable without
/// actually spawning the heartbeat / IPC / command-handler tasks.
///
/// # Errors
///
/// Returns `DroneError::Db` if the database cannot be opened or the
/// session row cannot be inserted.
pub fn bootstrap(
    session_id: &str,
    db_path: &std::path::Path,
) -> Result<Arc<Mutex<rusqlite::Connection>>, DroneError> {
    let conn = db::init(db_path)?;
    seed_session(&conn, session_id)?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn seed_session(conn: &rusqlite::Connection, session_id: &str) -> Result<(), db::DbError> {
    conn.execute(
        "INSERT OR IGNORE INTO sessions (id, status) VALUES (?1, 'active')",
        rusqlite::params![session_id],
    )?;
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
    use tempfile::TempDir;
    use tokio::time::{timeout, Duration};

    #[test]
    fn bootstrap_creates_database_and_seeds_session() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("d.sqlite");

        let conn = bootstrap("session-x", &path).expect("bootstrap");

        let count: i64 = conn
            .blocking_lock()
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE id = 'session-x'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(count, 1, "bootstrap must seed the session row");
    }

    #[test]
    fn bootstrap_is_idempotent_for_same_session() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("d.sqlite");

        let _conn1 = bootstrap("s1", &path).expect("first");
        let conn2 = bootstrap("s1", &path).expect("second");

        let count: i64 = conn2
            .blocking_lock()
            .query_row("SELECT COUNT(*) FROM sessions WHERE id = 's1'", [], |r| {
                r.get(0)
            })
            .expect("count");
        assert_eq!(count, 1, "duplicate bootstrap must not create extra rows");
    }

    #[test]
    fn drone_error_wraps_db_error() {
        let err = db::DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows);
        let top: DroneError = err.into();
        assert!(matches!(top, DroneError::Db(_)));
    }

    fn temp_socket_path() -> PathBuf {
        #[cfg(unix)]
        {
            let dir = TempDir::new().expect("tempdir");
            let p = dir.path().join("d.sock");
            std::mem::forget(dir);
            p
        }
        #[cfg(windows)]
        {
            let suffix = uuid::Uuid::new_v4();
            PathBuf::from(format!(r"\\.\pipe\drone-run-test-{suffix}"))
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_inner_drives_full_orchestration() {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("d.sqlite");
        let socket = temp_socket_path();

        let conn = bootstrap("s1", &db_path).expect("bootstrap");
        let signal = async { "shutdown" };

        let result = timeout(
            Duration::from_secs(3),
            run_inner(conn.clone(), "s1".to_string(), socket, signal),
        )
        .await
        .expect("run_inner did not return");
        result.expect("run_inner returned an error");

        let count: i64 = conn
            .lock()
            .await
            .query_row(
                "SELECT COUNT(*) FROM snapshots WHERE event_type = 'shutdown'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(count, 1, "shutdown must produce one emergency snapshot");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_inner_exits_on_ipc_graceful_shutdown() {
        use runtime_core::DroneCommand;
        use std::time::Instant;

        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("d.sqlite");
        let socket = temp_socket_path();

        let conn = bootstrap("s2", &db_path).expect("bootstrap");
        // Pending OS signal — never fires; only IPC shutdown should drive exit.
        let never_signal = std::future::pending::<&'static str>();

        let conn_clone = conn.clone();
        let socket_clone = socket.clone();
        let join = tokio::spawn(async move {
            run_inner(conn_clone, "s2".to_string(), socket_clone, never_signal).await
        });

        // Give the IPC server a moment to bind.
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Drive shutdown via the IPC channel by talking through the pipe.
        let cmd = DroneCommand::GracefulShutdown { timeout_ms: 50 };
        send_command_over_pipe(&socket, &cmd).await;

        let started = Instant::now();
        let outcome = timeout(Duration::from_secs(3), join)
            .await
            .expect("run_inner did not exit on IPC GracefulShutdown")
            .expect("join failed");
        outcome.expect("run_inner returned an error");
        assert!(started.elapsed() < Duration::from_secs(3));

        let count: i64 = conn
            .lock()
            .await
            .query_row(
                "SELECT COUNT(*) FROM snapshots WHERE event_type = 'ipc_graceful'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(
            count, 1,
            "IPC graceful shutdown must produce exactly one emergency snapshot"
        );
    }

    async fn send_command_over_pipe(socket: &std::path::Path, cmd: &runtime_core::DroneCommand) {
        use tokio::io::AsyncWriteExt;
        let line = format!("{}\n", serde_json::to_string(cmd).expect("encode"));

        #[cfg(unix)]
        {
            let deadline = std::time::Instant::now() + Duration::from_secs(2);
            loop {
                match tokio::net::UnixStream::connect(socket).await {
                    Ok(mut s) => {
                        s.write_all(line.as_bytes()).await.expect("write");
                        s.flush().await.expect("flush");
                        return;
                    }
                    Err(_) if std::time::Instant::now() < deadline => {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                    Err(e) => panic!("connect: {e}"),
                }
            }
        }
        #[cfg(windows)]
        {
            use tokio::net::windows::named_pipe::ClientOptions;
            let deadline = std::time::Instant::now() + Duration::from_secs(2);
            loop {
                match ClientOptions::new().open(socket) {
                    Ok(mut s) => {
                        s.write_all(line.as_bytes()).await.expect("write");
                        s.flush().await.expect("flush");
                        return;
                    }
                    Err(_) if std::time::Instant::now() < deadline => {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                    Err(e) => panic!("client connect: {e}"),
                }
            }
        }
    }
}
