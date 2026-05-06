//! `SQLite` setup — WAL pragmas, schema initialization.
//!
//! Mirrors `agent-runtime-spec.md` §1c (Multi-Session & `SQLite`
//! Concurrency) and §11 (Persistence Layer DDL). Pragmas are issued in the
//! order the spec requires: `journal_mode=WAL`, `synchronous=NORMAL`,
//! `busy_timeout=5000`, `foreign_keys=ON`.
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

/// Open or create the drone's `SQLite` database at `path`, configure
/// pragmas, and create the schema if missing.
///
/// The four pragmas are set in the exact order required by spec §1c. The
/// schema (`sessions`, `snapshots`, `signals`, `heartbeats`, `vdr`,
/// `token_usage`, `skills`, `mcp_servers`) is created with `IF NOT EXISTS`
/// so this function is idempotent.
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

/// Run the schema-creation step against an already-open `Connection`.
///
/// Intended for callers that want to pre-seed an existing database (e.g.
/// integration tests that need to write fixture rows BEFORE the drone
/// subprocess opens the same path). Idempotent over the schema —
/// `CREATE TABLE IF NOT EXISTS` lets a subsequent `init` call coexist.
///
/// # Errors
///
/// Returns `DbError::Sqlite` if any `CREATE TABLE` / `ALTER TABLE`
/// fails.
pub fn init_in_existing(conn: &Connection) -> Result<(), DbError> {
    init_schema(conn)
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
          contributing_signal_id TEXT,
          FOREIGN KEY (session_id) REFERENCES sessions(id),
          FOREIGN KEY (snapshot_id) REFERENCES snapshots(id)
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_vdr_contributing_signal
          ON vdr(contributing_signal_id);

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

    init_mcp_servers(conn)?;
    Ok(())
}

// 8th table — `mcp_servers`. Per spec §11:2435-2444 + MCP best practice
// (Claude Code, Claude Desktop, VS Code MCP client schemas). Fields cover:
// identity, transport-specific config (stdio: command/args/env; remote:
// url/headers), authentication (with keychain refs — NEVER literal
// secrets), connection lifecycle, timeouts, scope tracking, and
// capability caching. Mutual-exclusion CHECK enforces stdio-vs-remote
// invariant at the SQL level. Schema is stable; M06 wires the MCP client.
fn init_mcp_servers(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        r"
        CREATE TABLE IF NOT EXISTS mcp_servers (
            id                          INTEGER PRIMARY KEY AUTOINCREMENT,

            name                        TEXT NOT NULL UNIQUE,
            transport                   TEXT NOT NULL
                                        CHECK (transport IN ('stdio', 'http', 'sse', 'streamable_http')),

            command                     TEXT,
            args_json                   TEXT,
            env_json                    TEXT,

            url                         TEXT,
            headers_json                TEXT,

            auth_kind                   TEXT
                                        CHECK (auth_kind IN ('none', 'bearer', 'oauth', 'custom') OR auth_kind IS NULL),
            auth_token_ref              TEXT,
            oauth_state_json            TEXT,

            status                      TEXT NOT NULL DEFAULT 'configured'
                                        CHECK (status IN ('configured', 'connected', 'errored', 'disabled', 'failed')),
            last_error                  TEXT,
            last_connected_at           INTEGER,
            retry_count                 INTEGER NOT NULL DEFAULT 0,

            startup_timeout_ms          INTEGER NOT NULL DEFAULT 10000,
            tool_timeout_ms             INTEGER NOT NULL DEFAULT 60000,

            enabled                     BOOLEAN NOT NULL DEFAULT 1,
            scope                       TEXT NOT NULL DEFAULT 'user'
                                        CHECK (scope IN ('user', 'project', 'plugin', 'local')),
            plugin_id                   TEXT,

            discovered_tool_count       INTEGER,
            last_capabilities_refresh   INTEGER,

            added_at                    INTEGER NOT NULL,
            updated_at                  INTEGER NOT NULL,

            CHECK (
                (transport = 'stdio' AND command IS NOT NULL AND url IS NULL)
                OR
                (transport IN ('http', 'sse', 'streamable_http') AND url IS NOT NULL AND command IS NULL)
            )
        );
        CREATE INDEX IF NOT EXISTS idx_mcp_servers_status  ON mcp_servers(status);
        CREATE INDEX IF NOT EXISTS idx_mcp_servers_enabled ON mcp_servers(enabled);
        CREATE INDEX IF NOT EXISTS idx_mcp_servers_scope   ON mcp_servers(scope);
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
