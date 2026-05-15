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

use rusqlite::Connection;

use crate::client::error::LifecycleError;

/// `SQLite` registry handle. Mutex-guarded around the rusqlite Connection
/// because rusqlite's Connection is `Send` but not `Sync`.
#[derive(Debug)]
pub struct Registry {
    #[allow(dead_code, reason = "M06.C green phase will use this field")]
    conn: Mutex<Connection>,
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
    /// Server connection status — `configured` | `connected` | `errored`
    /// | `disabled` | `failed`. Driven by lifecycle health-pings.
    pub status: String,
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
    pub fn open(_path: &Path) -> Result<Self, LifecycleError> {
        todo!("M06.C green phase")
    }

    /// Insert one server config into the registry.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::AlreadyExists`] when a server with the same
    ///   name already exists.
    /// - [`LifecycleError::Registry`] for other `SQLite` failures.
    pub fn insert(&self, _record: &McpServerRecord) -> Result<(), LifecycleError> {
        todo!("M06.C green phase")
    }

    /// Fetch one server by name.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] when no row matches the name.
    /// - [`LifecycleError::Registry`] for other `SQLite` failures.
    pub fn get(&self, _name: &str) -> Result<McpServerRecord, LifecycleError> {
        todo!("M06.C green phase")
    }

    /// List all registered servers.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] for `SQLite` failures.
    pub fn list(&self) -> Result<Vec<McpServerRecord>, LifecycleError> {
        todo!("M06.C green phase")
    }

    /// Remove the server row matching `name`. Idempotent — removing a
    /// missing name returns `Ok(())`.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] for `SQLite` failures.
    pub fn remove(&self, _name: &str) -> Result<(), LifecycleError> {
        todo!("M06.C green phase")
    }

    /// Update `last_connected_at` to the supplied unix-ms timestamp.
    /// Reused as the "last known alive" signal — Stage C lifecycle's
    /// health-ping loop calls this on every successful ping.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] for `SQLite` failures.
    pub fn update_last_alive(&self, _name: &str, _ts_unix_ms: i64) -> Result<(), LifecycleError> {
        todo!("M06.C green phase")
    }
}
