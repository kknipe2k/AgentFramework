//! `SQLite` setup — WAL pragmas, versioned migration runner.
//!
//! Mirrors `agent-runtime-spec.md` §1c, §11, and §3a + §10.
//!
//! Pragmas are issued in the order the spec requires:
//! `journal_mode=WAL`, `synchronous=NORMAL`, `busy_timeout=5000`,
//! `foreign_keys=ON`.
//!
//! ## Migration runner architecture (M04 Stage B)
//!
//! Migrations live in `crates/runtime-drone/migrations/NNN_<name>.sql`,
//! embedded at build time via `include_str!` (single-binary deployment;
//! no runtime filesystem dependency). [`run_migrations`] tracks applied
//! versions in the `_migrations` table; each migration runs at most once
//! per database. Adding a new migration:
//!
//! 1. Create `migrations/NNN_<name>.sql` with the `CREATE TABLE IF NOT
//!    EXISTS` content. Choose the next free `NNN`.
//! 2. Add an entry to the `MIGRATIONS` slice with the same `NNN` + name
//!    + the `include_str!`'d content.
//! 3. Add a unit test asserting the migration applies + re-applies cleanly.
//!
//! ## Secrets handling for `mcp_servers`
//!
//! The `mcp_servers` table stores **references to OS keychain entries**,
//! never literal secrets:
//!
//! - `auth_token_ref` — keychain entry name (e.g., `agent-runtime/mcp/
//!   github/token`).
//! - `env_json` map values — either non-secret literals (e.g.,
//!   `RUST_LOG=info`) or keychain refs prefixed with `keychain://`.
//! - `oauth_state_json` — `{access_token_ref, refresh_token_ref,
//!   expires_at, scopes[]}`; all token fields are refs.
//!
//! The Rust insert/update path enforces this at runtime in M06 when the
//! MCP client lands. M02 only ships the schema.

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

/// One migration: a numeric version (lexical / monotonic), a short name
/// (recorded for forensics), and the SQL body. Embedded at build time.
struct Migration {
    version: u32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 0,
        name: "initial",
        sql: include_str!("../migrations/000_initial.sql"),
    },
    Migration {
        version: 1,
        name: "plans_tasks",
        sql: include_str!("../migrations/001_plans_tasks.sql"),
    },
];

/// Open or create the drone's `SQLite` database at `path`, configure
/// pragmas, and apply pending migrations.
///
/// The four pragmas are set in the exact order required by spec §1c.
/// Migrations are applied via [`run_migrations`] — idempotent across
/// process restarts.
///
/// # Errors
///
/// Returns `DbError::Sqlite` if the database cannot be opened, the pragmas
/// cannot be set, or any migration fails.
pub fn init(path: &Path) -> Result<Connection, DbError> {
    let conn = Connection::open(path)?;
    set_pragmas(&conn)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Run all pending migrations against an already-open `Connection`.
///
/// Intended for callers that pre-seed an existing database (e.g.
/// integration tests that need to write fixture rows BEFORE the drone
/// subprocess opens the same path). Idempotent: each migration applies
/// at most once per database, tracked via the `_migrations` table.
///
/// # Errors
///
/// Returns `DbError::Sqlite` if any migration fails.
pub fn init_in_existing(conn: &Connection) -> Result<(), DbError> {
    run_migrations(conn)
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

/// Apply pending migrations in version order; track applied versions in
/// the `_migrations` table.
///
/// Each migration runs in its own transaction so a malformed migration
/// rolls back cleanly without leaving partial schema state. The
/// `_migrations` row is `INSERT`ed inside the same transaction; rollback
/// also rolls back the version-tracking write.
///
/// # Errors
///
/// Returns `DbError::Sqlite` for any migration that fails (transaction
/// rolled back; subsequent migrations not attempted).
pub fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (\
            version INTEGER PRIMARY KEY,\
            name TEXT NOT NULL,\
            applied_at INTEGER NOT NULL\
        )",
    )?;

    let applied: std::collections::HashSet<u32> = {
        let mut stmt = conn.prepare("SELECT version FROM _migrations")?;
        let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
        let mut set = std::collections::HashSet::new();
        for row in rows {
            set.insert(u32::try_from(row?).unwrap_or(u32::MAX));
        }
        set
    };

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));

    for m in MIGRATIONS {
        if applied.contains(&m.version) {
            continue;
        }
        // Wrap each migration + its tracking insert in a transaction so a
        // failure rolls back partial schema state. rusqlite's
        // `execute_batch` does not auto-transaction; we BEGIN/COMMIT
        // explicitly.
        conn.execute_batch("BEGIN")?;
        let body_result = conn.execute_batch(m.sql);
        if let Err(e) = body_result {
            // Best-effort rollback. If rollback itself fails (e.g. the
            // SQL aborted the transaction implicitly), ignore the
            // secondary error and surface the original failure.
            let _ = conn.execute_batch("ROLLBACK");
            return Err(DbError::Sqlite(e));
        }
        let track = conn.execute(
            "INSERT INTO _migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![i64::from(m.version), m.name, now_ms],
        );
        if let Err(e) = track {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(DbError::Sqlite(e));
        }
        conn.execute_batch("COMMIT")?;
    }
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
            "mcp_servers",
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

    #[test]
    fn init_schema_creates_mcp_servers_table() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mcp_servers'",
                [],
                |r| r.get(0),
            )
            .expect("query mcp_servers presence");
        assert_eq!(count, 1, "mcp_servers table must exist after init");

        let columns: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('mcp_servers') ORDER BY cid")
            .expect("prepare")
            .query_map([], |r| r.get::<_, String>(0))
            .expect("query")
            .map(Result::unwrap)
            .collect();
        for expected in [
            "id",
            "name",
            "transport",
            "command",
            "args_json",
            "env_json",
            "url",
            "headers_json",
            "auth_kind",
            "auth_token_ref",
            "oauth_state_json",
            "status",
            "last_error",
            "last_connected_at",
            "retry_count",
            "startup_timeout_ms",
            "tool_timeout_ms",
            "enabled",
            "scope",
            "plugin_id",
            "discovered_tool_count",
            "last_capabilities_refresh",
            "added_at",
            "updated_at",
        ] {
            assert!(
                columns.iter().any(|c| c == expected),
                "missing mcp_servers column {expected}; got {columns:?}"
            );
        }

        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='mcp_servers'")
            .expect("prepare")
            .query_map([], |r| r.get::<_, String>(0))
            .expect("query")
            .map(Result::unwrap)
            .collect();
        for expected in [
            "idx_mcp_servers_status",
            "idx_mcp_servers_enabled",
            "idx_mcp_servers_scope",
        ] {
            assert!(
                indexes.iter().any(|i| i == expected),
                "missing index {expected}; got {indexes:?}"
            );
        }
    }

    fn insert_stdio(conn: &Connection, name: &str) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO mcp_servers (name, transport, command, added_at, updated_at) \
             VALUES (?1, 'stdio', ?2, 0, 0)",
            rusqlite::params![name, "/bin/echo"],
        )?;
        Ok(())
    }

    #[test]
    fn mcp_servers_stdio_invariant_enforced() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        let result = conn.execute(
            "INSERT INTO mcp_servers (name, transport, added_at, updated_at) \
             VALUES ('bad-stdio', 'stdio', 0, 0)",
            [],
        );
        assert!(
            result.is_err(),
            "stdio row WITHOUT command must violate CHECK constraint"
        );

        let result_with_url = conn.execute(
            "INSERT INTO mcp_servers (name, transport, command, url, added_at, updated_at) \
             VALUES ('mixed-stdio', 'stdio', '/bin/x', 'https://x', 0, 0)",
            [],
        );
        assert!(
            result_with_url.is_err(),
            "stdio row WITH url must violate CHECK constraint"
        );
    }

    #[test]
    fn mcp_servers_remote_invariant_enforced() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        let no_url = conn.execute(
            "INSERT INTO mcp_servers (name, transport, added_at, updated_at) \
             VALUES ('bad-remote', 'http', 0, 0)",
            [],
        );
        assert!(
            no_url.is_err(),
            "http row WITHOUT url must violate CHECK constraint"
        );

        let with_command = conn.execute(
            "INSERT INTO mcp_servers (name, transport, command, url, added_at, updated_at) \
             VALUES ('mixed-remote', 'sse', '/bin/x', 'https://x', 0, 0)",
            [],
        );
        assert!(
            with_command.is_err(),
            "sse row WITH command must violate CHECK constraint"
        );
    }

    #[test]
    fn mcp_servers_status_transitions() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        insert_stdio(&conn, "srv-status").expect("insert");

        conn.execute(
            "UPDATE mcp_servers SET status='connected' WHERE name='srv-status'",
            [],
        )
        .expect("transition to connected");
        conn.execute(
            "UPDATE mcp_servers SET status='errored', last_error='boom' WHERE name='srv-status'",
            [],
        )
        .expect("transition to errored");

        let bad = conn.execute(
            "UPDATE mcp_servers SET status='gone' WHERE name='srv-status'",
            [],
        );
        assert!(bad.is_err(), "invalid status string must be rejected");
    }

    #[test]
    fn mcp_servers_unique_name_enforced() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        insert_stdio(&conn, "dup").expect("first");
        let second = insert_stdio(&conn, "dup");
        assert!(
            second.is_err(),
            "second insert with same name must violate UNIQUE"
        );
    }

    #[test]
    fn mcp_servers_default_values_applied() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        insert_stdio(&conn, "defaults").expect("insert");

        let (status, enabled, scope, retry, startup, tool): (String, i64, String, i64, i64, i64) =
            conn.query_row(
                "SELECT status, enabled, scope, retry_count, startup_timeout_ms, tool_timeout_ms \
                 FROM mcp_servers WHERE name='defaults'",
                [],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                    ))
                },
            )
            .expect("query defaults");
        assert_eq!(status, "configured");
        assert_eq!(enabled, 1);
        assert_eq!(scope, "user");
        assert_eq!(retry, 0);
        assert_eq!(startup, 10000);
        assert_eq!(tool, 60000);
    }

    #[test]
    fn mcp_servers_invalid_auth_kind_rejected() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        let bad = conn.execute(
            "INSERT INTO mcp_servers (name, transport, command, auth_kind, added_at, updated_at) \
             VALUES ('badauth', 'stdio', '/bin/x', 'ssh-key', 0, 0)",
            [],
        );
        assert!(bad.is_err(), "auth_kind='ssh-key' must be rejected");
    }

    #[test]
    fn mcp_servers_invalid_scope_rejected() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        let bad = conn.execute(
            "INSERT INTO mcp_servers (name, transport, command, scope, added_at, updated_at) \
             VALUES ('badscope', 'stdio', '/bin/x', 'enterprise', 0, 0)",
            [],
        );
        assert!(bad.is_err(), "scope='enterprise' must be rejected");
    }

    #[test]
    fn mcp_servers_invalid_transport_rejected() {
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        let bad = conn.execute(
            "INSERT INTO mcp_servers (name, transport, command, added_at, updated_at) \
             VALUES ('badtrans', 'websocket', '/bin/x', 0, 0)",
            [],
        );
        assert!(bad.is_err(), "transport='websocket' must be rejected");
    }

    #[test]
    fn heartbeat_status_roundtrip_via_db() {
        use runtime_core::HeartbeatStatus;
        let (_dir, path) = temp_db_path();
        let conn = init(&path).expect("init");
        conn.execute(
            "INSERT INTO sessions (id, status) VALUES ('s1', 'active')",
            [],
        )
        .expect("seed session");

        let written = HeartbeatStatus::Degraded;
        let serialized = written.to_string();
        conn.execute(
            "INSERT INTO heartbeats (id, session_id, timestamp, status) VALUES ('hb1', 's1', 0, ?1)",
            rusqlite::params![serialized],
        )
        .expect("insert heartbeat");

        let row: String = conn
            .query_row("SELECT status FROM heartbeats WHERE id='hb1'", [], |r| {
                r.get(0)
            })
            .expect("query");
        let read: HeartbeatStatus = row.parse().expect("parse HeartbeatStatus");
        assert_eq!(read, written);
    }
}
