//! M06 Stage C — `runtime-mcp` client lifecycle (server install + auth +
//! connection management).
//!
//! Wraps the Stage B [`crate::transport`] primitive with server lifecycle
//! management:
//!
//! - [`McpClient`] — public surface. `add_server` / `remove_server` /
//!   `test_connection` / `list_servers` / `get_connection`. Holds the
//!   `mcp_servers` registry, secret store, audit writer, and live
//!   connection cache.
//! - [`Registry`] — SQLite-backed persistence; path-agnostic.
//! - [`SecretStore`] — per-server auth secret abstraction.
//! - [`LifecycleError`] — error enum aggregating Mcp / Registry / Auth
//!   variants.
//! - `lifecycle::spawn_health_pinger` — connection health-ping loop.
//!
//! Per ADR-0007: HITL seams + audit live in the main process; the drone
//! is audit + projection, not orchestrator. `McpClient` holds the audit
//! writer + emits via the existing [`runtime_main::audit`] surface.
//!
//! Per CLAUDE.md §14 schema-as-source-of-truth: registry SQL column
//! names match `mcp.v1.json::McpServerConfig` exactly. Migration 002
//! aligned the existing M02-scaffolded `mcp_servers` table to this name.

pub mod auth;
pub mod error;
pub mod lifecycle;
pub mod registry;

pub use auth::{InMemorySecretStore, KeyringSecretStore, SecretStore, MCP_KEYRING_SERVICE};
pub use error::LifecycleError;
pub use registry::{McpServerRecord, Registry};

use std::collections::BTreeMap;
use std::sync::Arc;

use runtime_core::generated::mcp::{McpServerConfig, McpTransport};
use runtime_main::audit::AuditWriter;
use tokio::sync::RwLock;

use crate::transport::{Connection, McpTool};

/// MCP client lifecycle manager.
///
/// One per process (Tauri shell constructs at app startup). Holds the
/// `SQLite` registry, the secret store, the audit writer, and a cache of
/// live connections keyed by server name.
pub struct McpClient {
    #[allow(dead_code, reason = "M06.C green phase will use this field")]
    registry: Arc<Registry>,
    #[allow(dead_code, reason = "M06.C green phase will use this field")]
    secret_store: Arc<dyn SecretStore>,
    #[allow(dead_code, reason = "M06.C green phase will use this field")]
    audit: Option<Arc<AuditWriter>>,
    #[allow(dead_code, reason = "M06.C green phase will use this field")]
    session_id: String,
    #[allow(dead_code, reason = "M06.C green phase will use this field")]
    connections: RwLock<BTreeMap<String, Arc<dyn Connection>>>,
}

/// Summary of one registered server, returned by [`McpClient::list_servers`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpServerSummary {
    /// Server name.
    pub name: String,
    /// Transport discriminant (`stdio` or `http`).
    pub transport: String,
    /// True iff a per-server auth secret is registered.
    pub has_auth: bool,
    /// Current connection status.
    pub status: String,
}

impl McpClient {
    /// Construct a `McpClient` WITHOUT audit wiring. For tests + the v0.1
    /// pre-audit code path (which doesn't exist — every call site wires
    /// audit). Prefer [`McpClient::new_with_audit`] in production.
    #[must_use]
    pub fn new(
        registry: Arc<Registry>,
        secret_store: Arc<dyn SecretStore>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            registry,
            secret_store,
            audit: None,
            session_id: session_id.into(),
            connections: RwLock::new(BTreeMap::new()),
        }
    }

    /// Construct a `McpClient` with the M05.E audit writer wired. The
    /// writer is `Arc<AuditWriter>` so the same writer can be shared
    /// across the capability enforcer, framework loader, tier evaluator,
    /// and `McpClient` — every audit line lands in the same JSONL file.
    #[must_use]
    pub fn new_with_audit(
        registry: Arc<Registry>,
        secret_store: Arc<dyn SecretStore>,
        audit: Arc<AuditWriter>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            registry,
            secret_store,
            audit: Some(audit),
            session_id: session_id.into(),
            connections: RwLock::new(BTreeMap::new()),
        }
    }

    /// Add a new MCP server. Persists to registry, optionally stores the
    /// auth secret, runs a one-shot test connection (handshake +
    /// disconnect), and emits the audit lines.
    ///
    /// Audit emission contract per gotcha #66:
    /// - On success WITHOUT auth: emits exactly one `mcp_installed` line.
    /// - On success WITH auth: emits TWO lines in order — `mcp_installed`
    ///   then `mcp_auth_granted`.
    /// - On failure (`test_connection` fails): emits ZERO lines and does
    ///   NOT persist to registry / secret store.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::AlreadyExists`] when a server of the same name
    ///   already exists.
    /// - [`LifecycleError::Mcp`] when the test connection fails.
    /// - [`LifecycleError::Auth`] / [`LifecycleError::Registry`] for
    ///   persistence failures.
    #[expect(
        clippy::unused_async,
        reason = "M06.C green phase awaits registry / transport / audit"
    )]
    pub async fn add_server(
        &self,
        _config: McpServerConfig,
        _auth: Option<String>,
        _transport: Arc<dyn crate::transport::Transport>,
    ) -> Result<(), LifecycleError> {
        todo!("M06.C green phase")
    }

    /// Remove a registered MCP server. Disconnects (if connected),
    /// removes from registry, drops the auth secret (if any), emits
    /// `mcp_uninstalled` audit line.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] when no server matches the name.
    /// - [`LifecycleError::Registry`] / [`LifecycleError::Auth`] for
    ///   underlying failures.
    #[expect(
        clippy::unused_async,
        reason = "M06.C green phase awaits registry / transport / audit"
    )]
    pub async fn remove_server(&self, _name: &str) -> Result<(), LifecycleError> {
        todo!("M06.C green phase")
    }

    /// Test a server connection without persisting. Connect + `list_tools`
    /// + disconnect. For the Settings panel's "Test" button.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Mcp`] for connect / `list_tools` failures.
    #[expect(
        clippy::unused_async,
        reason = "M06.C green phase awaits transport.connect"
    )]
    pub async fn test_connection(
        &self,
        _transport: Arc<dyn crate::transport::Transport>,
    ) -> Result<Vec<McpTool>, LifecycleError> {
        todo!("M06.C green phase")
    }

    /// List registered servers + their current state.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] for `SQLite` failures.
    #[expect(
        clippy::unused_async,
        reason = "M06.C green phase awaits registry list"
    )]
    pub async fn list_servers(&self) -> Result<Vec<McpServerSummary>, LifecycleError> {
        todo!("M06.C green phase")
    }

    /// Return the cached connection for `name`, or connect + cache + return.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] when no server matches the name.
    /// - [`LifecycleError::Mcp`] when the (re)connect fails.
    #[expect(
        clippy::unused_async,
        reason = "M06.C green phase awaits cache lookup / connect"
    )]
    pub async fn get_connection(
        &self,
        _name: &str,
        _transport: Arc<dyn crate::transport::Transport>,
    ) -> Result<Arc<dyn Connection>, LifecycleError> {
        todo!("M06.C green phase")
    }

    /// Internal hook: run a single health-check pass across all cached
    /// connections. Failed pings emit the existing `mcp_missing` event
    /// variant via the supplied event sink + drop the cached connection
    /// so the next `get_connection` call reconnects.
    ///
    /// Called by `lifecycle::spawn_health_pinger`'s loop body.
    ///
    /// # Errors
    ///
    /// Best-effort: per-server failures are logged via `tracing::warn!`
    /// and routed via the event sink; this method itself never returns
    /// an error.
    #[expect(
        clippy::unused_async,
        reason = "M06.C green phase awaits per-connection ping"
    )]
    pub async fn run_health_pass<F>(&self, _emit_missing: F)
    where
        F: Fn(&str),
    {
        todo!("M06.C green phase")
    }

    /// Helper for transport construction from a registry record.
    /// Stage E's renderer-driven flow constructs an
    /// `Arc<dyn Transport>` from the user input; this helper covers the
    /// "rebuild from persisted record" path used by health-ping reconnect.
    #[allow(
        dead_code,
        reason = "M06.C green phase wires this from McpServerConfig"
    )]
    #[must_use]
    pub fn transport_from_config(
        _config: &McpServerConfig,
    ) -> Arc<dyn crate::transport::Transport> {
        todo!("M06.C green phase")
    }

    /// Internal: derive the transport-kind discriminant string from an
    /// `McpTransport` for audit / event emission.
    #[allow(dead_code, reason = "M06.C green phase uses this in audit emission")]
    #[must_use]
    pub const fn transport_kind(transport: &McpTransport) -> &'static str {
        match transport {
            McpTransport::Stdio { .. } => "stdio",
            McpTransport::Http { .. } => "http",
        }
    }
}
