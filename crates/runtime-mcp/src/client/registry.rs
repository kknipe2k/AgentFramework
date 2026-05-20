//! SQLite-backed `mcp_servers` registry (M06 Stage C).
//!
//! Owns the read/write surface against the `mcp_servers` table that M02
//! scaffolded in `crates/runtime-drone/migrations/000_initial.sql` +
//! M06.C extended via `migrations/002_mcp_servers.sql` (RENAME
//! `auth_token_ref` → `auth_secret_ref` + ADD cwd). The schema is
//! source-of-truth per CLAUDE.md §14: every column name on the wire
//! matches `mcp.v1.json::McpServerConfig`.
//!
//! Path-agnostic per CLAUDE.md §9 + docs/style.md archetype: the Tauri
//! shell resolves `AppHandle::path().app_local_data_dir()` and passes
//! the resolved path to [`Registry::open`]. Tests use `tempfile::tempdir()`.
//!
//! ## Migration strategy
//!
//! The drone-owned migration runner (`runtime_drone::db::run_migrations`)
//! is the authoritative migration applier in production (the drone owns
//! the `SQLite` database file and applies migrations at process startup).
//! The runtime-mcp Registry wires through a thin reuse: `Registry::open`
//! calls into `runtime_drone::db::init` to get the same WAL pragmas +
//! migration runner semantics. Same DB file used by both processes.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::client::error::LifecycleError;
use crate::client::ServerStatus;

/// `SQLite` registry handle. Mutex-guarded around the rusqlite Connection
/// because rusqlite's Connection is `Send` but not `Sync`.
#[derive(Debug)]
pub struct Registry {
    conn: Mutex<Connection>,
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

fn map_unique_violation(name: &str, e: rusqlite::Error) -> LifecycleError {
    if let rusqlite::Error::SqliteFailure(ref err, _) = e {
        if err.code == rusqlite::ErrorCode::ConstraintViolation {
            return LifecycleError::AlreadyExists(name.to_string());
        }
    }
    LifecycleError::Registry(e)
}

/// Snapshot of one registered MCP server, as the Registry exposes it.
///
/// Wire shape closely mirrors `mcp.v1.json::McpServerConfig` but is
/// `serde::Serialize` for IPC + Tauri command return types. Stage E's
/// renderer round-trips this shape into the `MCPNode` display.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpServerRecord {
    /// Server name (DNS-label per mcp.v1.json `McpServerName`).
    pub name: String,
    /// Transport discriminant (`stdio` or `http`).
    pub transport: String,
    /// Stdio command, when transport=`stdio`.
    pub command: Option<String>,
    /// Stdio args (JSON array as string), when transport=`stdio`.
    pub args_json: Option<String>,
    /// Stdio env (JSON object as string), when transport=`stdio`.
    pub env_json: Option<String>,
    /// Stdio working directory, when transport=`stdio`. Added M06.C 002.
    pub cwd: Option<String>,
    /// HTTP url, when transport=`http`.
    pub url: Option<String>,
    /// Per-server keychain-key reference, when an auth secret is stored.
    /// Renamed from `auth_token_ref` at M06.C migration 002.
    pub auth_secret_ref: Option<String>,
    /// Server connection status (CQ-6 — the schema-generated
    /// [`ServerStatus`] enum: `connected` | `disconnected` |
    /// `health_pending` | `error`). Driven by lifecycle health-pings;
    /// stored in the TEXT column via the generated `Display`/`FromStr`.
    pub status: ServerStatus,
}

impl Registry {
    /// Open or create the `SQLite` registry at `path`.
    ///
    /// Wires through `runtime_drone::db::init` for the WAL pragmas +
    /// migration runner. Idempotent — calling twice on the same path
    /// reopens (does not re-run migrations; the `_migrations` table
    /// tracks applied versions).
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] when the `SQLite` open / pragma /
    ///   migration step fails.
    pub fn open(path: &Path) -> Result<Self, LifecycleError> {
        let conn = runtime_drone::db::init(path).map_err(|e| match e {
            runtime_drone::db::DbError::Sqlite(s) => LifecycleError::Registry(s),
        })?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Insert one server config into the registry.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::AlreadyExists`] when a server with the same
    ///   name already exists.
    /// - [`LifecycleError::Registry`] for other `SQLite` failures.
    ///
    /// # Panics
    ///
    /// Panics if the inner connection mutex is poisoned (a prior holder
    /// panicked while the lock was held). This is unrecoverable and
    /// indicates a bug elsewhere in the runtime.
    pub fn insert(&self, record: &McpServerRecord) -> Result<(), LifecycleError> {
        let now = now_unix_ms();
        let conn = self.conn.lock().expect("registry conn mutex poisoned");
        let result = conn
            .execute(
                "INSERT INTO mcp_servers (\
                name, transport, command, args_json, env_json, cwd, url, \
                auth_secret_ref, status, added_at, updated_at\
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                params![
                    record.name,
                    record.transport,
                    record.command,
                    record.args_json,
                    record.env_json,
                    record.cwd,
                    record.url,
                    record.auth_secret_ref,
                    record.status.to_string(),
                    now,
                ],
            )
            .map_err(|e| map_unique_violation(&record.name, e));
        drop(conn);
        result?;
        Ok(())
    }

    /// Fetch one server by name.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] when no row matches the name.
    /// - [`LifecycleError::Registry`] for other `SQLite` failures.
    ///
    /// # Panics
    ///
    /// Panics if the inner connection mutex is poisoned.
    pub fn get(&self, name: &str) -> Result<McpServerRecord, LifecycleError> {
        let conn = self.conn.lock().expect("registry conn mutex poisoned");
        let result = conn
            .query_row(
                "SELECT name, transport, command, args_json, env_json, cwd, url, \
                    auth_secret_ref, status \
             FROM mcp_servers WHERE name = ?1",
                params![name],
                row_to_record,
            )
            .optional();
        drop(conn);
        result?.ok_or_else(|| LifecycleError::NotFound(name.to_string()))
    }

    /// List all registered servers.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] for `SQLite` failures.
    ///
    /// # Panics
    ///
    /// Panics if the inner connection mutex is poisoned.
    pub fn list(&self) -> Result<Vec<McpServerRecord>, LifecycleError> {
        let conn = self.conn.lock().expect("registry conn mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT name, transport, command, args_json, env_json, cwd, url, \
                    auth_secret_ref, status \
             FROM mcp_servers ORDER BY name",
        )?;
        let rows = stmt.query_map([], row_to_record)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        drop(stmt);
        drop(conn);
        Ok(out)
    }

    /// Remove the server row matching `name`. Idempotent — removing a
    /// missing name returns `Ok(())`.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] for `SQLite` failures.
    ///
    /// # Panics
    ///
    /// Panics if the inner connection mutex is poisoned.
    pub fn remove(&self, name: &str) -> Result<(), LifecycleError> {
        let conn = self.conn.lock().expect("registry conn mutex poisoned");
        let result = conn.execute("DELETE FROM mcp_servers WHERE name = ?1", params![name]);
        drop(conn);
        result?;
        Ok(())
    }

    /// Update `last_connected_at` to the supplied unix-ms timestamp.
    /// Reused as the "last known alive" signal — Stage C lifecycle's
    /// health-ping loop calls this on every successful ping.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] for `SQLite` failures.
    ///
    /// # Panics
    ///
    /// Panics if the inner connection mutex is poisoned.
    pub fn update_last_alive(&self, name: &str, ts_unix_ms: i64) -> Result<(), LifecycleError> {
        let conn = self.conn.lock().expect("registry conn mutex poisoned");
        let result = conn.execute(
            "UPDATE mcp_servers SET last_connected_at = ?1, updated_at = ?1 WHERE name = ?2",
            params![ts_unix_ms, name],
        );
        drop(conn);
        result?;
        Ok(())
    }

    /// EFF-4 — persist a whole health pass in ONE transaction.
    ///
    /// `run_health_pass` previously issued K sequential `UPDATE`s (one
    /// `update_last_alive` per server, no status write). This applies
    /// every server's `(name, status, ts)` in a single transaction so
    /// the multi-server set is updated atomically with one fsync and a
    /// reader never observes a half-written pass. Each tuple writes the
    /// CQ-6 [`ServerStatus`] (serialized via the generated `Display`) +
    /// `last_connected_at`/`updated_at`. A name with no matching row is
    /// a silent no-op (the server was removed mid-pass).
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] when the transaction or any
    ///   statement fails (the whole pass rolls back).
    ///
    /// # Panics
    ///
    /// Panics if the inner connection mutex is poisoned.
    pub fn update_health_batch(
        &self,
        updates: &[(String, ServerStatus, i64)],
    ) -> Result<(), LifecycleError> {
        let mut conn = self.conn.lock().expect("registry conn mutex poisoned");
        let tx = conn.transaction()?;
        {
            let mut up = tx.prepare(
                "UPDATE mcp_servers SET status = ?1, last_connected_at = ?2, \
                 updated_at = ?2 WHERE name = ?3",
            )?;
            for (name, status, ts) in updates {
                up.execute(params![status.to_string(), ts, name])?;
            }
        }
        tx.commit()?;
        drop(conn);
        Ok(())
    }
}

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<McpServerRecord> {
    // CQ-6 — the status TEXT column round-trips through the generated
    // `FromStr`. A value outside the schema enum is registry corruption,
    // surfaced as a conversion failure (→ `LifecycleError::Registry`)
    // rather than silently coerced.
    let status_str: String = row.get(8)?;
    let status = status_str.parse::<ServerStatus>().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            8,
            rusqlite::types::Type::Text,
            format!("invalid McpServerStatus {status_str:?}: {e}").into(),
        )
    })?;
    Ok(McpServerRecord {
        name: row.get(0)?,
        transport: row.get(1)?,
        command: row.get(2)?,
        args_json: row.get(3)?,
        env_json: row.get(4)?,
        cwd: row.get(5)?,
        url: row.get(6)?,
        auth_secret_ref: row.get(7)?,
        status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_unique_violation_passes_through_non_constraint_errors() {
        // Non-constraint SQLite errors (e.g., InvalidColumnIndex) must
        // surface as Registry, not AlreadyExists, so the caller can tell
        // the difference between "schema mismatch" and "duplicate name."
        let raw = rusqlite::Error::InvalidColumnIndex(99);
        let mapped = map_unique_violation("name-doesnt-matter", raw);
        assert!(matches!(mapped, LifecycleError::Registry(_)));
    }

    #[test]
    fn now_unix_ms_returns_post_epoch() {
        let t = now_unix_ms();
        assert!(t > 1_700_000_000_000, "got {t}");
    }
}
