//! Heartbeat task — 5s tokio interval.
//!
//! Per `agent-runtime-spec.md` §1 (Heartbeat) the drone pings every 5
//! seconds, writes a row to the `heartbeats` table, and emits a
//! `DroneEvent::Heartbeat` on the broadcast channel for the IPC server to
//! relay back to main.

use crate::HEARTBEAT_INTERVAL;
use runtime_core::{DroneEvent, HeartbeatStatus};
use rusqlite::Connection;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};
use tokio::time::interval;
use uuid::Uuid;

/// Errors raised by the heartbeat task.
#[derive(Debug, Error)]
pub enum HeartbeatError {
    /// Underlying rusqlite error.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// Run the heartbeat task until cancelled.
///
/// On each tick, writes a row to the `heartbeats` table and emits
/// `DroneEvent::Heartbeat` on `event_tx`. Broadcast send errors are
/// non-fatal — they only mean nobody is currently subscribed.
///
/// # Errors
///
/// Returns `HeartbeatError::Sqlite` if a heartbeat row cannot be inserted.
pub async fn run(
    session_id: String,
    conn: Arc<Mutex<Connection>>,
    event_tx: broadcast::Sender<DroneEvent>,
) -> Result<(), HeartbeatError> {
    let mut ticker = interval(HEARTBEAT_INTERVAL);
    let status = HeartbeatStatus::Ok;
    loop {
        ticker.tick().await;
        let timestamp = current_timestamp();
        let id = Uuid::new_v4().to_string();
        {
            let guard = conn.lock().await;
            guard.execute(
                "INSERT INTO heartbeats (id, session_id, timestamp, status) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![id, session_id, timestamp, status.to_string()],
            )?;
        }
        let _ = event_tx.send(DroneEvent::Heartbeat { status, timestamp });
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use runtime_core::DroneEvent;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::{broadcast, Mutex};
    use tokio::time::Duration;

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

    #[tokio::test(start_paused = true)]
    async fn heartbeat_fires_at_5_second_interval() {
        let (_dir, conn) = open();
        let (tx, mut rx) = broadcast::channel(16);

        let task = tokio::spawn(run("s1".to_string(), conn, tx));

        // First tick is fired immediately by tokio::time::interval; consume it.
        let first = rx.recv().await.expect("first tick");
        assert!(matches!(first, DroneEvent::Heartbeat { .. }));

        tokio::time::advance(Duration::from_secs(5)).await;
        let second = rx.recv().await.expect("second tick at 5s");
        assert!(matches!(second, DroneEvent::Heartbeat { .. }));

        tokio::time::advance(Duration::from_secs(5)).await;
        let third = rx.recv().await.expect("third tick at 10s");
        assert!(matches!(third, DroneEvent::Heartbeat { .. }));

        task.abort();
    }

    #[tokio::test(start_paused = true)]
    async fn heartbeat_writes_row_to_db() {
        let (_dir, conn) = open();
        let (tx, mut rx) = broadcast::channel(16);

        let conn_clone = conn.clone();
        let task = tokio::spawn(run("s1".to_string(), conn_clone, tx));

        // Drain at least one tick so the writer has run.
        let _first = rx.recv().await.expect("first tick");

        let count: i64 = conn
            .lock()
            .await
            .query_row(
                "SELECT COUNT(*) FROM heartbeats WHERE session_id = 's1'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert!(count >= 1, "heartbeats table should have at least one row");
        task.abort();
    }

    #[tokio::test(start_paused = true)]
    async fn heartbeat_writes_typed_status_to_db() {
        use runtime_core::HeartbeatStatus;
        let (_dir, conn) = open();
        let (tx, mut rx) = broadcast::channel(16);

        let conn_clone = conn.clone();
        let task = tokio::spawn(run("s1".to_string(), conn_clone, tx));

        // Drain one tick so the writer has run.
        let _ = rx.recv().await.expect("first tick");

        let stored: String = conn
            .lock()
            .await
            .query_row(
                "SELECT status FROM heartbeats WHERE session_id = 's1' LIMIT 1",
                [],
                |r| r.get(0),
            )
            .expect("query stored status");
        let parsed: HeartbeatStatus = stored.parse().expect("parse stored status");
        assert_eq!(parsed, HeartbeatStatus::Ok);

        task.abort();
    }
}
