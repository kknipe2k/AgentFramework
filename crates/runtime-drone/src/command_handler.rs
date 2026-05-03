//! `DroneCommand` dispatch loop.
//!
//! Receives commands from the IPC server (via `cmd_rx`) and acts on them
//! against the `SQLite` database. Emits `DroneEvent` results on the
//! broadcast channel. Six command variants per `agent-runtime-spec.md` §1d.

use crate::snapshot;
use runtime_core::{AlertLevel, DroneCommand, DroneEvent, ProcessConfig, ProcessType};
use rusqlite::Connection;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio::time::{timeout, Duration};
use tracing::warn;

/// Errors raised by the command handler.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Snapshot writer error.
    #[error(transparent)]
    Snapshot(#[from] snapshot::SnapshotError),

    /// Underlying rusqlite error.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// Run the command-dispatch loop until `cmd_rx` closes or
/// `GracefulShutdown` is received.
///
/// On `GracefulShutdown`, the handler fires `shutdown_tx` (if provided) to
/// signal the drone's top-level shutdown source, then returns. Callers that
/// don't need IPC-driven shutdown (e.g., unit tests for individual command
/// behaviors) may pass `None`.
///
/// # Errors
///
/// Returns `CommandError::Snapshot` if a `SnapshotNow` write fails, or
/// `CommandError::Sqlite` for any other database error.
pub async fn run(
    session_id: String,
    conn: Arc<Mutex<Connection>>,
    mut cmd_rx: mpsc::Receiver<DroneCommand>,
    event_tx: broadcast::Sender<DroneEvent>,
    shutdown_tx: Option<oneshot::Sender<&'static str>>,
) -> Result<(), CommandError> {
    let mut shutdown_tx = shutdown_tx;
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            DroneCommand::SnapshotNow { reason, state_json } => {
                handle_snapshot_now(&conn, &session_id, &reason, &state_json, &event_tx).await?;
            }
            DroneCommand::GracefulShutdown { timeout_ms } => {
                let _ = timeout(Duration::from_millis(timeout_ms), async {}).await;
                if let Some(tx) = shutdown_tx.take() {
                    let _ = tx.send("ipc_graceful");
                }
                return Ok(());
            }
            DroneCommand::RevertToSnapshot {
                snapshot_id,
                reason: _,
            } => {
                handle_revert(&conn, &snapshot_id, &event_tx).await;
            }
            DroneCommand::SpawnProcess {
                process_type,
                config,
            } => {
                emit_unsupported(&event_tx, "spawn_process", &process_type, &config);
            }
            DroneCommand::StopProcess { pid, force } => {
                let _ = event_tx.send(DroneEvent::Alert {
                    level: AlertLevel::Warn,
                    message: format!("stop_process not yet implemented (pid={pid}, force={force})"),
                });
            }
            DroneCommand::SetActivityTimeout { ms } => {
                let _ = event_tx.send(DroneEvent::Alert {
                    level: AlertLevel::Warn,
                    message: format!("set_activity_timeout not yet implemented (ms={ms})"),
                });
            }
        }
    }
    Ok(())
}

async fn handle_snapshot_now(
    conn: &Arc<Mutex<Connection>>,
    session_id: &str,
    reason: &str,
    state: &serde_json::Value,
    event_tx: &broadcast::Sender<DroneEvent>,
) -> Result<(), CommandError> {
    let id = {
        let guard = conn.lock().await;
        snapshot::write(&guard, session_id, reason, state)?
    };
    let _ = event_tx.send(DroneEvent::SnapshotWritten {
        snapshot_id: id,
        session_id: session_id.to_string(),
        reason: reason.to_string(),
        timestamp: 0,
    });
    Ok(())
}

async fn handle_revert(
    conn: &Arc<Mutex<Connection>>,
    snapshot_id: &str,
    event_tx: &broadcast::Sender<DroneEvent>,
) {
    let lookup = {
        let guard = conn.lock().await;
        guard
            .query_row(
                "SELECT session_id FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
    };

    if let Some(session_id) = lookup {
        let _ = event_tx.send(DroneEvent::SnapshotWritten {
            snapshot_id: snapshot_id.to_string(),
            session_id,
            reason: "revert".to_string(),
            timestamp: 0,
        });
    } else {
        warn!(snapshot_id, "revert requested for unknown snapshot");
        let _ = event_tx.send(DroneEvent::Alert {
            level: AlertLevel::Critical,
            message: format!("unknown snapshot id: {snapshot_id}"),
        });
    }
}

fn emit_unsupported(
    event_tx: &broadcast::Sender<DroneEvent>,
    op: &str,
    process_type: &ProcessType,
    config: &ProcessConfig,
) {
    let _ = event_tx.send(DroneEvent::Alert {
        level: AlertLevel::Warn,
        message: format!(
            "{op} not yet implemented (type={process_type:?}, command={})",
            config.command
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, snapshot};
    use runtime_core::{
        AlertLevel, DroneCommand, DroneEvent, ProcessConfig, ProcessType, RevertReason,
    };
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
    use tokio::time::{timeout, Duration};

    fn open() -> (TempDir, Arc<Mutex<rusqlite::Connection>>) {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("d.sqlite");
        let conn = db::init(&path).expect("init");
        conn.execute(
            "INSERT INTO sessions (id, status) VALUES ('s1', 'active')",
            [],
        )
        .expect("seed");
        (dir, Arc::new(Mutex::new(conn)))
    }

    #[tokio::test]
    async fn graceful_shutdown_flushes_within_timeout() {
        let (_dir, conn) = open();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, _event_rx) = broadcast::channel::<DroneEvent>(8);

        let conn_clone = conn.clone();
        let task = tokio::spawn(run("s1".to_string(), conn_clone, cmd_rx, event_tx, None));

        cmd_tx
            .send(DroneCommand::SnapshotNow {
                reason: "pre".into(),
                state_json: serde_json::json!({"x": 1}),
            })
            .await
            .expect("send pre");
        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 100 })
            .await
            .expect("send shutdown");

        let result = timeout(Duration::from_secs(2), task).await;
        assert!(result.is_ok(), "command handler must exit ≤2s on shutdown");

        let count: i64 = conn
            .lock()
            .await
            .query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 1, "pending snapshot should have been written");
    }

    #[tokio::test]
    async fn revert_to_snapshot_returns_blob() {
        let (_dir, conn) = open();
        let known = serde_json::json!({"resume_at": 7});
        let snap_id = {
            let guard = conn.lock().await;
            snapshot::write(&guard, "s1", "seed", &known).expect("seed snapshot")
        };

        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(16);

        let conn_clone = conn.clone();
        let task = tokio::spawn(run("s1".to_string(), conn_clone, cmd_rx, event_tx, None));

        cmd_tx
            .send(DroneCommand::RevertToSnapshot {
                snapshot_id: snap_id.clone(),
                reason: RevertReason::UserRollback,
            })
            .await
            .expect("send revert");

        let mut found = false;
        for _ in 0..10 {
            if let Ok(Ok(DroneEvent::SnapshotWritten { snapshot_id, .. })) =
                timeout(Duration::from_millis(500), event_rx.recv()).await
            {
                if snapshot_id == snap_id {
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "revert should emit a SnapshotWritten referencing the requested snapshot"
        );

        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 100 })
            .await
            .expect("shutdown");
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn unknown_snapshot_id_emits_alert() {
        let (_dir, conn) = open();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(16);

        let task = tokio::spawn(run("s1".to_string(), conn, cmd_rx, event_tx, None));

        cmd_tx
            .send(DroneCommand::RevertToSnapshot {
                snapshot_id: "does-not-exist".to_string(),
                reason: RevertReason::UserRollback,
            })
            .await
            .expect("send revert");

        let mut got_alert = false;
        for _ in 0..10 {
            if let Ok(Ok(DroneEvent::Alert {
                level: AlertLevel::Critical,
                ..
            })) = timeout(Duration::from_millis(500), event_rx.recv()).await
            {
                got_alert = true;
                break;
            }
        }
        assert!(got_alert, "unknown snapshot id must emit Critical Alert");

        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 100 })
            .await
            .expect("shutdown");
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn unsupported_commands_emit_warn_alerts() {
        let (_dir, conn) = open();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(16);

        let task = tokio::spawn(run("s1".to_string(), conn, cmd_rx, event_tx, None));

        cmd_tx
            .send(DroneCommand::SpawnProcess {
                process_type: ProcessType::Agent,
                config: ProcessConfig {
                    command: "echo".to_string(),
                    args: vec![],
                    env: std::collections::HashMap::new(),
                },
            })
            .await
            .expect("send spawn");
        cmd_tx
            .send(DroneCommand::StopProcess {
                pid: 42,
                force: true,
            })
            .await
            .expect("send stop");
        cmd_tx
            .send(DroneCommand::SetActivityTimeout { ms: 1000 })
            .await
            .expect("send set");

        let mut warn_count = 0;
        for _ in 0..10 {
            if let Ok(Ok(DroneEvent::Alert {
                level: AlertLevel::Warn,
                ..
            })) = timeout(Duration::from_millis(500), event_rx.recv()).await
            {
                warn_count += 1;
                if warn_count >= 3 {
                    break;
                }
            }
        }
        assert_eq!(
            warn_count, 3,
            "each unsupported command should emit a Warn Alert"
        );

        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 100 })
            .await
            .expect("shutdown");
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn graceful_shutdown_fires_shutdown_signal() {
        let (_dir, conn) = open();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, _event_rx) = broadcast::channel::<DroneEvent>(8);
        let (sd_tx, sd_rx) = oneshot::channel::<&'static str>();

        let task = tokio::spawn(run("s1".to_string(), conn, cmd_rx, event_tx, Some(sd_tx)));

        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 50 })
            .await
            .expect("send shutdown");

        let signal = timeout(Duration::from_secs(2), sd_rx)
            .await
            .expect("shutdown signal timed out")
            .expect("shutdown channel closed");
        assert_eq!(signal, "ipc_graceful");

        let _ = timeout(Duration::from_secs(1), task).await;
    }
}
