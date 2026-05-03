//! Signal handler — emergency-snapshot logic for the shutdown path.
//!
//! Per `agent-runtime-spec.md` §1 (Graceful Shutdown) the drone catches
//! termination signals and writes one last "emergency" snapshot before
//! exiting. The snapshot is best-effort: failure to write is logged but
//! does not propagate as an error — the drone exits 0 either way so the OS
//! does not retry the signal.
//!
//! The OS signal source itself lives in `lib::shutdown_signal_future`;
//! this module is the snapshot/event side of the shutdown flow.

use crate::snapshot;
use runtime_core::DroneEvent;
use rusqlite::Connection;
use std::future::Future;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info};

/// Errors raised by the shutdown handler. Currently unreachable because
/// `wait_and_handle_with` swallows snapshot errors; reserved for future
/// orchestration paths that need fatal-error escalation.
#[derive(Debug, Error)]
pub enum ShutdownError {
    /// Snapshot writer error (kept for completeness; not currently emitted).
    #[error(transparent)]
    Snapshot(#[from] snapshot::SnapshotError),
}

/// Await `signal_source`, then run the emergency-snapshot path.
///
/// Tests pass a deterministic ready-future; `lib::run` passes the
/// platform-specific OS signal future from `lib::shutdown_signal_future`.
///
/// # Errors
///
/// Currently infallible — snapshot failures are logged and swallowed so
/// the drone always exits 0. Reserved for future orchestration paths.
pub async fn wait_and_handle_with<F>(
    signal_source: F,
    conn: Arc<Mutex<Connection>>,
    session_id: String,
    event_tx: broadcast::Sender<DroneEvent>,
) -> Result<(), ShutdownError>
where
    F: Future<Output = &'static str>,
{
    let signal_label = signal_source.await;
    info!(signal = signal_label, "drone received termination signal");
    handle_emergency(&conn, &session_id, signal_label, &event_tx).await;
    Ok(())
}

/// Write an emergency snapshot for `session_id` with the given `reason`
/// label and broadcast a `SnapshotWritten` event. Snapshot failures are
/// logged and swallowed; the function never returns an error.
pub async fn handle_emergency(
    conn: &Arc<Mutex<Connection>>,
    session_id: &str,
    reason: &str,
    event_tx: &broadcast::Sender<DroneEvent>,
) {
    let snapshot_state = serde_json::json!({
        "session_id": session_id,
        "reason": reason,
    });

    let snapshot_id = {
        let guard = conn.lock().await;
        match snapshot::write(&guard, session_id, reason, &snapshot_state) {
            Ok(id) => Some(id),
            Err(e) => {
                error!(error = %e, "emergency snapshot failed");
                None
            }
        }
    };

    if let Some(id) = snapshot_id {
        let _ = event_tx.send(DroneEvent::SnapshotWritten {
            snapshot_id: id,
            session_id: session_id.to_string(),
            reason: reason.to_string(),
            timestamp: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use runtime_core::DroneEvent;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::{broadcast, Mutex};
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
    async fn handle_emergency_writes_snapshot_and_emits_event() {
        let (_dir, conn) = open();
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(8);

        handle_emergency(&conn, "s1", "sigterm", &event_tx).await;

        let count: i64 = conn
            .lock()
            .await
            .query_row(
                "SELECT COUNT(*) FROM snapshots WHERE event_type = 'sigterm'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(count, 1, "emergency snapshot must land in DB");

        let event = timeout(Duration::from_millis(500), event_rx.recv())
            .await
            .expect("event timeout")
            .expect("event channel closed");
        match event {
            DroneEvent::SnapshotWritten {
                session_id, reason, ..
            } => {
                assert_eq!(session_id, "s1");
                assert_eq!(reason, "sigterm");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn handle_emergency_swallows_db_failure() {
        let (dir, conn) = open();
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(8);

        // Drop the snapshots table to force snapshot::write to fail.
        conn.lock()
            .await
            .execute_batch("DROP TABLE snapshots")
            .expect("drop");

        handle_emergency(&conn, "s1", "sigint", &event_tx).await;

        let result = timeout(Duration::from_millis(100), event_rx.recv()).await;
        assert!(
            result.is_err() || matches!(result, Ok(Err(_))),
            "no event should be emitted when snapshot fails"
        );
        drop(dir);
    }

    #[tokio::test]
    async fn wait_and_handle_with_drives_orchestration() {
        let (_dir, conn) = open();
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(8);

        let signal = async { "test_signal" };
        wait_and_handle_with(signal, conn.clone(), "s1".to_string(), event_tx)
            .await
            .expect("infallible");

        let event = timeout(Duration::from_millis(500), event_rx.recv())
            .await
            .expect("event timeout")
            .expect("event channel closed");
        match event {
            DroneEvent::SnapshotWritten { reason, .. } => assert_eq!(reason, "test_signal"),
            other => panic!("unexpected event: {other:?}"),
        }
    }
}
