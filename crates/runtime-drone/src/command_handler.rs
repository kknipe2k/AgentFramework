//! `DroneCommand` dispatch loop.
//!
//! Receives commands from the IPC server (via `cmd_rx`) and acts on them
//! against the `SQLite` database. Emits `DroneEvent` results on the
//! broadcast channel. Six command variants per `agent-runtime-spec.md` §1d.

use crate::{plan_projector, snapshot, vdr};
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

    /// VDR projection / read-only query error.
    #[error(transparent)]
    Vdr(#[from] vdr::VdrError),

    /// Plan projector error (M04 Stage B).
    #[error(transparent)]
    PlanProjector(#[from] plan_projector::PlanProjectorError),
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
                reason,
            } => {
                handle_revert(&conn, &snapshot_id, &reason, &event_tx).await;
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
            DroneCommand::QuerySessionDb { sql } => {
                handle_query_session_db(&conn, &sql, &event_tx).await;
            }
            DroneCommand::ReadSignals { session_id } => {
                handle_read_signals(&conn, &session_id, &event_tx).await;
            }
            DroneCommand::WriteSignal {
                signal_id,
                session_id,
                kind,
                event,
                context_type,
                payload,
            } => {
                handle_write_signal(
                    &conn,
                    &signal_id,
                    &session_id,
                    &kind,
                    &event,
                    &context_type,
                    &payload,
                    &event_tx,
                )
                .await;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_write_signal(
    conn: &Arc<Mutex<Connection>>,
    signal_id: &str,
    session_id: &str,
    kind: &str,
    event_name: &str,
    context_type: &str,
    payload: &serde_json::Value,
    event_tx: &broadcast::Sender<DroneEvent>,
) {
    let payload_str = serde_json::to_string(payload).unwrap_or_else(|_| "null".to_string());
    let guard = conn.lock().await;
    let result: Result<(usize, usize), CommandError> = (|| {
        guard.execute(
            "INSERT OR IGNORE INTO signals (\
                id, session_id, type, event, timestamp, payload_json, context_type\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                signal_id,
                session_id,
                kind,
                event_name,
                now_ms_string(),
                payload_str,
                context_type,
            ],
        )?;
        // Two projectors: vdr (decision/verify) + plan_projector (plan/task).
        // Both are idempotent; either may no-op for non-matching kinds /
        // event types.
        let vdr_n = match vdr::project_signal(&guard, signal_id) {
            Ok(n) => n,
            Err(vdr::VdrError::SignalNotFound(_)) => 0,
            Err(e) => return Err(CommandError::from(e)),
        };
        let plan_n = match plan_projector::project_signal(&guard, signal_id) {
            Ok(n) => n,
            Err(plan_projector::PlanProjectorError::SignalNotFound(_)) => 0,
            Err(e) => return Err(CommandError::from(e)),
        };
        Ok((vdr_n, plan_n))
    })();
    drop(guard);
    match result {
        Ok(_) => {
            // ACK by emitting an Alert at Warn level with the signal_id —
            // existing event channel; M04 Stage B keeps the IPC surface
            // minimal. Future stages may add a typed `SignalWritten`
            // event variant to DroneEvent.
            let _ = event_tx.send(DroneEvent::Alert {
                level: AlertLevel::Warn,
                message: format!("write_signal ok: {signal_id}"),
            });
        }
        Err(e) => {
            let _ = event_tx.send(DroneEvent::Alert {
                level: AlertLevel::Critical,
                message: format!("write_signal failed for {signal_id}: {e}"),
            });
        }
    }
}

fn now_ms_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or_else(|_| "0".to_string(), |d| d.as_millis().to_string())
}

async fn handle_query_session_db(
    conn: &Arc<Mutex<Connection>>,
    sql: &str,
    event_tx: &broadcast::Sender<DroneEvent>,
) {
    if !vdr::is_select_only(sql) {
        let _ = event_tx.send(DroneEvent::Alert {
            level: AlertLevel::Critical,
            message: format!(
                "query_session_db rejected: only SELECT statements permitted; got: {}",
                truncate_for_log(sql, 80)
            ),
        });
        return;
    }
    let result = {
        let guard = conn.lock().await;
        vdr::execute_select(&guard, sql)
    };
    match result {
        Ok(rows) => {
            let _ = event_tx.send(DroneEvent::QueryResult { rows });
        }
        Err(e) => {
            let _ = event_tx.send(DroneEvent::Alert {
                level: AlertLevel::Critical,
                message: format!("query_session_db failed: {e}"),
            });
        }
    }
}

async fn handle_read_signals(
    conn: &Arc<Mutex<Connection>>,
    session_id: &str,
    event_tx: &broadcast::Sender<DroneEvent>,
) {
    let result = {
        let guard = conn.lock().await;
        vdr::signals_for_session(&guard, session_id)
    };
    match result {
        Ok(signals) => {
            let _ = event_tx.send(DroneEvent::SignalLog { signals });
        }
        Err(e) => {
            let _ = event_tx.send(DroneEvent::Alert {
                level: AlertLevel::Critical,
                message: format!("read_signals failed: {e}"),
            });
        }
    }
}

fn truncate_for_log(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        // Find a UTF-8 boundary at-or-below max so str slicing is safe.
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
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
    reason: &runtime_core::RevertReason,
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
        // Spec §4a: HookRollback variant carries hook_id so the SDK-side
        // `task_failed` emit (not the drone's job) can render
        // "rolled_back_after_hook_<hook_id>". The drone confirms revert
        // by re-emitting SnapshotWritten with a reason string that
        // includes the hook id when available.
        let reason_str = match reason {
            runtime_core::RevertReason::HookRollback { hook_id } => {
                format!("revert:hook_rollback:{hook_id}")
            }
            runtime_core::RevertReason::UserRollback => "revert:user_rollback".to_string(),
            runtime_core::RevertReason::GapRecovery => "revert:gap_recovery".to_string(),
        };
        let _ = event_tx.send(DroneEvent::SnapshotWritten {
            snapshot_id: snapshot_id.to_string(),
            session_id,
            reason: reason_str,
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
    async fn query_session_db_returns_rows_via_event() {
        let (_dir, conn) = open();
        // Seed two signals.
        {
            let g = conn.lock().await;
            g.execute(
                "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
                 VALUES ('sig1', 's1', 'tool', 'invoked', '0', '{}', 'agent_loop')",
                [],
            )
            .expect("seed sig1");
            g.execute(
                "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
                 VALUES ('sig2', 's1', 'decision', 'decision', '1', '{}', 'agent_loop')",
                [],
            )
            .expect("seed sig2");
        }
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(16);
        let task = tokio::spawn(run("s1".to_string(), conn, cmd_rx, event_tx, None));
        cmd_tx
            .send(DroneCommand::QuerySessionDb {
                sql: "SELECT id FROM signals ORDER BY id".to_string(),
            })
            .await
            .expect("send query");

        let mut got = None;
        for _ in 0..10 {
            if let Ok(Ok(DroneEvent::QueryResult { rows })) =
                timeout(Duration::from_millis(500), event_rx.recv()).await
            {
                got = Some(rows);
                break;
            }
        }
        let rows = got.expect("expected QueryResult event");
        assert_eq!(rows.len(), 2);
        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 50 })
            .await
            .expect("shutdown");
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn query_session_db_rejects_non_select_with_alert() {
        let (_dir, conn) = open();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(16);
        let task = tokio::spawn(run("s1".to_string(), conn, cmd_rx, event_tx, None));
        cmd_tx
            .send(DroneCommand::QuerySessionDb {
                sql: "DROP TABLE signals".to_string(),
            })
            .await
            .expect("send bad query");

        let mut got_alert = false;
        for _ in 0..10 {
            if let Ok(Ok(DroneEvent::Alert {
                level: AlertLevel::Critical,
                message,
            })) = timeout(Duration::from_millis(500), event_rx.recv()).await
            {
                if message.contains("only SELECT") {
                    got_alert = true;
                    break;
                }
            }
        }
        assert!(got_alert, "non-SELECT must emit a Critical alert");
        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 50 })
            .await
            .expect("shutdown");
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn read_signals_returns_signal_log_event() {
        let (_dir, conn) = open();
        {
            let g = conn.lock().await;
            for (id, ts) in [("a", "1"), ("b", "2")] {
                g.execute(
                    "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
                     VALUES (?1, 's1', 'agent', 'spawned', ?2, '{}', 'agent_loop')",
                    rusqlite::params![id, ts],
                )
                .expect("seed signal");
            }
        }
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(16);
        let task = tokio::spawn(run("s1".to_string(), conn, cmd_rx, event_tx, None));
        cmd_tx
            .send(DroneCommand::ReadSignals {
                session_id: "s1".to_string(),
            })
            .await
            .expect("send read");
        let mut got = None;
        for _ in 0..10 {
            if let Ok(Ok(DroneEvent::SignalLog { signals })) =
                timeout(Duration::from_millis(500), event_rx.recv()).await
            {
                got = Some(signals);
                break;
            }
        }
        let signals = got.expect("expected SignalLog event");
        assert_eq!(signals.len(), 2);
        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 50 })
            .await
            .expect("shutdown");
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[test]
    fn truncate_for_log_handles_short_strings() {
        assert_eq!(truncate_for_log("abc", 80), "abc");
    }

    #[test]
    fn truncate_for_log_respects_utf8_boundaries() {
        // 3-byte UTF-8 chars; truncating to 4 must back off to 3 (the
        // boundary before the second multibyte char).
        let s = "été café"; // contains multi-byte é
        let truncated = truncate_for_log(s, 4);
        // Just must be a valid &str slice — assertion is implicit; this
        // exercises the boundary-walk loop.
        assert!(truncated.is_char_boundary(truncated.len()));
        assert!(truncated.len() <= 4);
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

    // ── M04 Stage B: WriteSignal arm ─────────────────────────────────

    async fn drain_until<F>(rx: &mut broadcast::Receiver<DroneEvent>, predicate: F) -> bool
    where
        F: Fn(&DroneEvent) -> bool,
    {
        for _ in 0..20 {
            if let Ok(Ok(e)) = timeout(Duration::from_millis(250), rx.recv()).await {
                if predicate(&e) {
                    return true;
                }
            }
        }
        false
    }

    #[tokio::test]
    async fn write_signal_persists_to_signals_table() {
        let (_dir, conn) = open();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(16);
        let conn_clone = conn.clone();
        let task = tokio::spawn(run("s1".to_string(), conn_clone, cmd_rx, event_tx, None));

        cmd_tx
            .send(DroneCommand::WriteSignal {
                signal_id: "sig-write-1".to_string(),
                session_id: "s1".to_string(),
                kind: "agent".to_string(),
                event: "plan_created".to_string(),
                context_type: "plan_create".to_string(),
                payload: serde_json::json!({
                    "type": "plan_created",
                    "plan_id": "p1",
                    "title": "T",
                    "task_count": 0,
                    "approval_required": false,
                }),
            })
            .await
            .expect("send write_signal");

        let acked = drain_until(&mut event_rx, |e| {
            matches!(e, DroneEvent::Alert { message, .. } if message.contains("write_signal ok"))
        })
        .await;
        assert!(acked, "write_signal must emit ack alert");

        let row_count: i64 = conn
            .lock()
            .await
            .query_row(
                "SELECT COUNT(*) FROM signals WHERE id = 'sig-write-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(row_count, 1);

        // Plan_projector should have UPSERTed the plans row.
        let plan_status: String = conn
            .lock()
            .await
            .query_row("SELECT status FROM plans WHERE id = 'p1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(plan_status, "approved");

        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 50 })
            .await
            .expect("shutdown");
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn write_signal_idempotent_on_duplicate_id() {
        let (_dir, conn) = open();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, _event_rx) = broadcast::channel::<DroneEvent>(16);
        let conn_clone = conn.clone();
        let task = tokio::spawn(run("s1".to_string(), conn_clone, cmd_rx, event_tx, None));

        let cmd = DroneCommand::WriteSignal {
            signal_id: "sig-dup".to_string(),
            session_id: "s1".to_string(),
            kind: "tool".to_string(),
            event: "tool_invoked".to_string(),
            context_type: "tool_invoke".to_string(),
            payload: serde_json::json!({
                "type": "tool_invoked",
                "agent_id": "a1",
                "tool_name": "Read"
            }),
        };
        cmd_tx.send(cmd.clone()).await.unwrap();
        cmd_tx.send(cmd).await.unwrap();

        // Wait briefly for both writes to land.
        tokio::time::sleep(Duration::from_millis(150)).await;

        let count: i64 = conn
            .lock()
            .await
            .query_row(
                "SELECT COUNT(*) FROM signals WHERE id = 'sig-dup'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "INSERT OR IGNORE → exactly one row");

        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 50 })
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(1), task).await;
    }

    #[tokio::test]
    async fn write_signal_with_decision_payload_projects_to_vdr() {
        let (_dir, conn) = open();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, _event_rx) = broadcast::channel::<DroneEvent>(16);
        let conn_clone = conn.clone();
        let task = tokio::spawn(run("s1".to_string(), conn_clone, cmd_rx, event_tx, None));

        cmd_tx
            .send(DroneCommand::WriteSignal {
                signal_id: "sig-dec".to_string(),
                session_id: "s1".to_string(),
                kind: "decision".to_string(),
                event: "decision".to_string(),
                context_type: "agent_loop".to_string(),
                payload: serde_json::json!({
                    "agent_id": "a1",
                    "decision": "pick haiku",
                    "rationale": "cost",
                }),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(150)).await;

        let vdr_count: i64 = conn
            .lock()
            .await
            .query_row(
                "SELECT COUNT(*) FROM vdr WHERE contributing_signal_id = 'sig-dec'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(vdr_count, 1, "decision signal must project to vdr");

        cmd_tx
            .send(DroneCommand::GracefulShutdown { timeout_ms: 50 })
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(1), task).await;
    }
}
