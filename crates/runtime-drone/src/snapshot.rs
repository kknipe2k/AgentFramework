//! Append-only snapshot writer with SHA-256 `state_hash`.
//!
//! Per `agent-runtime-spec.md` §1 (Session Snapshots) snapshots are
//! immutable: a write is always an `INSERT`, never an `UPDATE`. The
//! `state_hash` is `sha256(state_json_canonical)` where the canonical form
//! is `serde_json::to_string` of the value.

use rusqlite::Connection;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

/// Errors raised by `write`.
#[derive(Debug, Error)]
pub enum SnapshotError {
    /// Underlying rusqlite error.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    /// JSON serialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Append a snapshot row for `session_id` with the given `reason` and
/// `state`. Returns the new snapshot UUID.
///
/// `event_type` in the table holds the human-readable `reason` string (e.g.
/// `"manual"`, `"sigterm"`, `"task_completed"`). The wire-level event
/// taxonomy in `runtime-core::event` carries the structured event names.
///
/// # Errors
///
/// Returns `SnapshotError::Json` if `state` cannot be serialized, or
/// `SnapshotError::Sqlite` if the row cannot be inserted.
pub fn write(
    conn: &Connection,
    session_id: &str,
    reason: &str,
    state: &Value,
) -> Result<String, SnapshotError> {
    let id = Uuid::new_v4().to_string();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX));
    let state_json = serde_json::to_string(state)?;
    let state_hash = sha256_hex(&state_json);

    conn.execute(
        "INSERT INTO snapshots (id, session_id, timestamp, event_type, state_json, state_hash) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, session_id, timestamp, reason, state_json, state_hash],
    )?;
    Ok(id)
}

fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    fn open() -> (TempDir, rusqlite::Connection) {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("d.sqlite");
        let conn = db::init(&path).expect("init");
        conn.execute(
            "INSERT INTO sessions (id, status) VALUES ('s1', 'active')",
            [],
        )
        .expect("seed session");
        (dir, conn)
    }

    #[test]
    fn snapshot_writes_correct_row() {
        let (_dir, conn) = open();
        let state = serde_json::json!({"k": "v"});
        let id = write(&conn, "s1", "test", &state).expect("write");

        let (got_session, got_reason, got_state, got_hash): (String, String, String, String) = conn
            .query_row(
                "SELECT session_id, event_type, state_json, state_hash FROM snapshots WHERE id = ?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .expect("row");

        assert_eq!(got_session, "s1");
        assert_eq!(got_reason, "test");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&got_state).unwrap(),
            state
        );
        assert_eq!(got_hash.len(), 64, "sha256 hex is 64 chars");
    }

    #[test]
    fn snapshot_state_hash_is_sha256() {
        let (_dir, conn) = open();
        let state = serde_json::json!({"k": "v"});
        let id = write(&conn, "s1", "h", &state).expect("write");
        let got_hash: String = conn
            .query_row(
                "SELECT state_hash FROM snapshots WHERE id = ?1",
                [&id],
                |r| r.get(0),
            )
            .expect("row");
        let canonical = serde_json::to_string(&state).unwrap();
        let expected = sha256_hex(&canonical);
        assert_eq!(got_hash, expected);
    }

    #[test]
    fn snapshot_appends_does_not_update() {
        let (_dir, conn) = open();
        let state = serde_json::json!({"k": "v"});
        let _id1 = write(&conn, "s1", "a", &state).expect("write 1");
        let _id2 = write(&conn, "s1", "a", &state).expect("write 2");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 2, "two writes must produce two rows (append-only)");
    }
}
