//! `SQLite` setup — WAL pragmas, schema initialization.
//!
//! Mirrors `agent-runtime-spec.md` §1c (Multi-Session & `SQLite`
//! Concurrency) and §11 (Persistence Layer DDL). Pragmas are issued in the
//! order the spec requires: `journal_mode=WAL`, `synchronous=NORMAL`,
//! `busy_timeout=5000`, `foreign_keys=ON`.

use rusqlite::Connection;
use std::path::Path;
use thiserror::Error;

/// Errors raised by `init`.
#[derive(Debug, Error)]
pub enum DbError {
    /// Underlying `rusqlite` error.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// Open or create the drone's `SQLite` database at `path`, configure
/// pragmas, and create the schema if missing.
///
/// The four pragmas are set in the exact order required by spec §1c. The
/// schema (`sessions`, `snapshots`, `signals`, `heartbeats`, `vdr`,
/// `token_usage`, `skills`) is created with `IF NOT EXISTS` so this
/// function is idempotent.
///
/// # Errors
///
/// Returns `DbError::Sqlite` if the database cannot be opened, the pragmas
/// cannot be set, or the schema cannot be created.
pub fn init(path: &Path) -> Result<Connection, DbError> {
    let conn = Connection::open(path)?;
    set_pragmas(&conn)?;
    init_schema(&conn)?;
    Ok(conn)
}

fn set_pragmas(conn: &Connection) -> Result<(), DbError> {
    // journal_mode is a query pragma — must be read via query_row.
    let _: String = conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get(0))?;
    conn.execute_batch(
        "PRAGMA synchronous = NORMAL;\n\
         PRAGMA busy_timeout = 5000;\n\
         PRAGMA foreign_keys = ON;",
    )?;
    Ok(())
}

fn init_schema(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        r"
        CREATE TABLE IF NOT EXISTS sessions (
          id TEXT PRIMARY KEY,
          framework_name TEXT,
          framework_version TEXT,
          model TEXT,
          started_at INTEGER,
          last_active INTEGER,
          status TEXT,
          mode TEXT
        );

        CREATE TABLE IF NOT EXISTS snapshots (
          id TEXT PRIMARY KEY,
          session_id TEXT,
          timestamp INTEGER,
          event_type TEXT,
          state_json TEXT,
          state_hash TEXT,
          FOREIGN KEY (session_id) REFERENCES sessions(id)
        );

        CREATE TABLE IF NOT EXISTS signals (
          id TEXT PRIMARY KEY,
          session_id TEXT,
          type TEXT,
          event TEXT,
          timestamp TEXT,
          duration_ms INTEGER,
          payload_json TEXT,
          pre_signal_id TEXT,
          parent_signal_id TEXT,
          retry_of TEXT,
          context_type TEXT,
          FOREIGN KEY (session_id) REFERENCES sessions(id)
        );
        CREATE INDEX IF NOT EXISTS idx_signals_session_time ON signals(session_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_signals_type ON signals(type);
        CREATE INDEX IF NOT EXISTS idx_signals_correlation ON signals(pre_signal_id, parent_signal_id, retry_of);

        CREATE TABLE IF NOT EXISTS heartbeats (
          id TEXT PRIMARY KEY,
          session_id TEXT,
          timestamp INTEGER,
          status TEXT,
          FOREIGN KEY (session_id) REFERENCES sessions(id)
        );
        CREATE INDEX IF NOT EXISTS idx_heartbeats_session_time ON heartbeats(session_id, timestamp);

        CREATE TABLE IF NOT EXISTS vdr (
          id TEXT PRIMARY KEY,
          session_id TEXT,
          agent_id TEXT,
          timestamp INTEGER,
          decision TEXT,
          rationale TEXT,
          tool_invoked TEXT,
          tool_input_json TEXT,
          tool_output_json TEXT,
          token_cost_usd REAL,
          outcome TEXT,
          snapshot_id TEXT,
          signal_ids TEXT,
          context_type TEXT,
          FOREIGN KEY (session_id) REFERENCES sessions(id),
          FOREIGN KEY (snapshot_id) REFERENCES snapshots(id)
        );

        CREATE TABLE IF NOT EXISTS token_usage (
          id TEXT PRIMARY KEY,
          session_id TEXT,
          agent_id TEXT,
          timestamp INTEGER,
          model TEXT,
          input_tokens INTEGER,
          output_tokens INTEGER,
          cost_usd REAL
        );

        CREATE TABLE IF NOT EXISTS skills (
          id TEXT PRIMARY KEY,
          name TEXT,
          version TEXT,
          source_url TEXT,
          installed_at INTEGER,
          validated INTEGER,
          skill_md TEXT
        );
        ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_db_path() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("drone.sqlite");
        (dir, path)
    }

    #[test]
    fn pragmas_set_in_correct_order() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");

        let journal: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .expect("journal_mode");
        assert_eq!(journal.to_lowercase(), "wal");

        let sync: i64 = conn
            .query_row("PRAGMA synchronous", [], |r| r.get(0))
            .expect("synchronous");
        assert_eq!(sync, 1, "synchronous=NORMAL is 1");

        let busy: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |r| r.get(0))
            .expect("busy_timeout");
        assert_eq!(busy, 5000);

        let fks: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .expect("foreign_keys");
        assert_eq!(fks, 1);
    }

    #[test]
    fn schema_creates_all_tables() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .expect("prepare");
        let names: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .expect("query")
            .map(Result::unwrap)
            .collect();

        for expected in [
            "heartbeats",
            "sessions",
            "signals",
            "skills",
            "snapshots",
            "token_usage",
            "vdr",
        ] {
            assert!(
                names.iter().any(|n| n == expected),
                "missing table {expected}; got {names:?}"
            );
        }
    }

    #[test]
    fn init_idempotent() {
        let (_dir, path) = temp_db_path();
        let _conn1 = init(&path).expect("first init");
        let _conn2 = init(&path).expect("second init must not error");
    }
}
