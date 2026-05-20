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
pub mod auth_keyring;
pub mod connection_resolver;
pub mod error;
pub mod lifecycle;
pub mod registry;

pub use auth::{InMemorySecretStore, SecretStore, MCP_KEYRING_SERVICE};
pub use auth_keyring::KeyringSecretStore;
pub use error::LifecycleError;
pub use registry::{McpServerRecord, Registry};

/// CQ-6 — the server connection status is the schema-generated
/// `McpServerStatus` enum (`schemas/mcp.v1.json#/$defs/McpServerStatus`,
/// shipped M06.B), re-exported as `ServerStatus`. Hand-writing it would
/// violate Hard Rule 5; the registry round-trips it via the generated
/// `Display`/`FromStr` at the `SQLite` TEXT boundary.
pub use runtime_core::generated::mcp::McpServerStatus as ServerStatus;

use std::collections::BTreeMap;
use std::sync::Arc;

use runtime_core::generated::mcp::{McpServerConfig, McpTransport};
use runtime_main::audit::{self, AuditWriter};
use tokio::sync::RwLock;

use crate::transport::{Connection, McpTool};

/// MCP client lifecycle manager.
///
/// One per process (Tauri shell constructs at app startup). Holds the
/// `SQLite` registry, the secret store, the audit writer, and a cache of
/// live connections keyed by server name.
pub struct McpClient {
    registry: Arc<Registry>,
    secret_store: Arc<dyn SecretStore>,
    audit: Option<Arc<AuditWriter>>,
    session_id: String,
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
    /// Current connection status (CQ-6 — the generated schema enum).
    pub status: ServerStatus,
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
    pub async fn add_server(
        &self,
        config: McpServerConfig,
        auth: Option<String>,
        transport: Arc<dyn crate::transport::Transport>,
    ) -> Result<(), LifecycleError> {
        // Test connection FIRST (handshake + disconnect) so a broken
        // server never lands in the registry / secret store / audit log.
        // Per the contract docstring + test
        // `add_server_failing_test_connection_does_not_persist_anything`.
        let probe = transport.connect().await?;
        // Drop the probe; the cached connection is established lazily on
        // the first `get_connection` call.
        drop(probe);

        // Registry persistence — derive the row shape from the schema.
        let name = config.name.to_string();
        let kind = Self::transport_kind(&config.transport);
        let record = config_to_record(&config);
        self.registry.insert(&record)?;

        // Secret persistence (if supplied). Failure here is observable —
        // the secret was supposed to land alongside the install. Return
        // the error so the caller surfaces; registry row stays in place
        // (a follow-up `remove_server` cleans up). v0.1 keeps the
        // failure-recovery shallow.
        let has_auth = if let (Some(secret), Some(ref_)) = (&auth, &config.auth_secret_ref) {
            self.secret_store.store_secret(ref_, secret).await?;
            true
        } else {
            false
        };

        // Audit emissions per gotcha #66 correlation. mcp_installed
        // ALWAYS fires on success; mcp_auth_granted fires on success
        // ONLY when an auth secret was stored.
        if let Some(writer) = &self.audit {
            // Best-effort observability per spec §13.5 — failures log via
            // tracing and don't propagate into dispatch.
            if let Err(e) = writer
                .log(&audit::mcp_installed(
                    &self.session_id,
                    &name,
                    kind,
                    has_auth,
                ))
                .await
            {
                tracing::error!(error = %e, name = %name, "audit mcp_installed failed");
            }
            if has_auth {
                if let Err(e) = writer
                    .log(&audit::mcp_auth_granted(&self.session_id, &name))
                    .await
                {
                    tracing::error!(error = %e, name = %name, "audit mcp_auth_granted failed");
                }
            }
        }
        Ok(())
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
    pub async fn remove_server(&self, name: &str) -> Result<(), LifecycleError> {
        // Resolve the row first so we can drop the auth secret + know
        // the row exists before we remove. NotFound is the user-
        // actionable case — return it rather than silently succeeding.
        let record = self.registry.get(name)?;

        // Disconnect + drop cached connection.
        {
            let mut cache = self.connections.write().await;
            if let Some(conn) = cache.remove(name) {
                if let Err(e) = conn.shutdown().await {
                    tracing::warn!(error = %e, name = %name, "MCP shutdown returned error; continuing remove");
                }
            }
        }

        self.registry.remove(name)?;

        if let Some(ref_) = &record.auth_secret_ref {
            self.secret_store.remove_secret(ref_).await?;
        }

        if let Some(writer) = &self.audit {
            if let Err(e) = writer
                .log(&audit::mcp_uninstalled(&self.session_id, name))
                .await
            {
                tracing::error!(error = %e, name = %name, "audit mcp_uninstalled failed");
            }
        }
        Ok(())
    }

    /// Test a server connection without persisting. Connect + `list_tools`
    /// + disconnect. For the Settings panel's "Test" button.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Mcp`] for connect / `list_tools` failures.
    pub async fn test_connection(
        &self,
        transport: Arc<dyn crate::transport::Transport>,
    ) -> Result<Vec<McpTool>, LifecycleError> {
        let conn = transport.connect().await?;
        let tools = conn.list_tools().await?;
        // Best-effort shutdown; the caller's contract is "no persistence
        // side-effects," so a failure here is logged but doesn't change
        // the success of the probe.
        if let Err(e) = conn.shutdown().await {
            tracing::warn!(error = %e, "test_connection shutdown returned error");
        }
        Ok(tools)
    }

    /// List registered servers + their current state.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::Registry`] for `SQLite` failures.
    #[expect(
        clippy::unused_async,
        reason = "list_servers stays async-shaped for symmetry with the other lifecycle methods + future async-driven sources (e.g., remote registry)"
    )]
    pub async fn list_servers(&self) -> Result<Vec<McpServerSummary>, LifecycleError> {
        let rows = self.registry.list()?;
        Ok(rows
            .into_iter()
            .map(|r| McpServerSummary {
                name: r.name,
                transport: r.transport,
                has_auth: r.auth_secret_ref.is_some(),
                status: r.status,
            })
            .collect())
    }

    /// Return the cached connection for `name`, or connect + cache + return.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::NotFound`] when no server matches the name.
    /// - [`LifecycleError::Mcp`] when the (re)connect fails.
    pub async fn get_connection(
        &self,
        name: &str,
        transport: Arc<dyn crate::transport::Transport>,
    ) -> Result<Arc<dyn Connection>, LifecycleError> {
        // Read-side cache check first to avoid the write-lock cost on
        // the common path (cached hit).
        {
            let cache = self.connections.read().await;
            if let Some(conn) = cache.get(name) {
                return Ok(Arc::clone(conn));
            }
        }
        // Confirm the server is registered — NotFound here distinguishes
        // "you didn't add this server" from "the server is offline."
        self.registry.get(name)?;
        // Connect outside the lock so concurrent connect attempts to
        // distinct servers don't serialize. Then upgrade-lock to insert.
        let new_conn: Arc<dyn Connection> = Arc::from(transport.connect().await?);
        let conn = {
            let mut cache = self.connections.write().await;
            // Race: another task may have inserted between our read-
            // release and write-acquire. Honor that one (return the
            // existing Arc) so both callers see Arc::ptr_eq.
            cache.entry(name.to_string()).or_insert(new_conn).clone()
        };
        Ok(conn)
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
    pub async fn run_health_pass<F>(&self, emit_missing: F)
    where
        F: Fn(&str),
    {
        // Snapshot the cache so we can ping without holding the lock
        // (callers may want concurrent get_connection during the pass).
        let snapshot: Vec<(String, Arc<dyn Connection>)> = {
            let cache = self.connections.read().await;
            cache
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect()
        };
        // EFF-4 — accumulate every server's outcome and persist the
        // whole pass in ONE batched registry transaction (was K
        // sequential `update_last_alive` calls). CQ-6 — the persisted
        // value is the `ServerStatus` enum: a successful ping is
        // `Connected`, a failed one `Error`. Atomic across the pass so
        // the multi-server path (an M07 registry may hold >1 server)
        // never observes a half-written set.
        let ts = i64::try_from(audit::entry::now_unix_ms()).unwrap_or(i64::MAX);
        let mut updates: Vec<(String, ServerStatus, i64)> = Vec::with_capacity(snapshot.len());
        let mut to_drop = Vec::new();
        for (name, conn) in &snapshot {
            match conn.ping().await {
                Ok(()) => updates.push((name.clone(), ServerStatus::Connected, ts)),
                Err(e) => {
                    tracing::warn!(error = %e, name = %name, "MCP health-ping failed; emitting mcp_missing");
                    emit_missing(name);
                    updates.push((name.clone(), ServerStatus::Error, ts));
                    to_drop.push(name.clone());
                }
            }
        }
        if !updates.is_empty() {
            // Best-effort persistence per spec §13.5 — a registry write
            // failure logs + continues; it does not gate the pass.
            if let Err(e) = self.registry.update_health_batch(&updates) {
                tracing::warn!(error = %e, "registry health-batch update failed");
            }
        }
        if !to_drop.is_empty() {
            let mut cache = self.connections.write().await;
            for name in to_drop {
                cache.remove(&name);
            }
        }
    }

    /// Helper for transport construction from an `McpServerConfig`.
    /// Stage E's renderer-driven flow constructs an
    /// `Arc<dyn Transport>` from the user input; this helper covers the
    /// "rebuild from persisted config" path used by health-ping reconnect.
    #[must_use]
    pub fn transport_from_config(config: &McpServerConfig) -> Arc<dyn crate::transport::Transport> {
        match &config.transport {
            McpTransport::Stdio {
                command,
                args,
                env,
                cwd,
            } => {
                let mut t = crate::transport::StdioTransport::new(command.to_string())
                    .with_args(args.clone());
                if !env.is_empty() {
                    t = t.with_env(env.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                }
                if let Some(c) = cwd {
                    t = t.with_cwd(std::path::PathBuf::from(c));
                }
                Arc::new(t)
            }
            McpTransport::Http { url } => {
                Arc::new(crate::transport::HttpTransport::new(url.clone()))
            }
        }
    }

    /// Internal: derive the transport-kind discriminant string from an
    /// `McpTransport` for audit / event emission.
    #[must_use]
    pub const fn transport_kind(transport: &McpTransport) -> &'static str {
        match transport {
            McpTransport::Stdio { .. } => "stdio",
            McpTransport::Http { .. } => "http",
        }
    }
}

fn config_to_record(config: &McpServerConfig) -> McpServerRecord {
    let (command, args_json, env_json, cwd, url) = match &config.transport {
        McpTransport::Stdio {
            command,
            args,
            env,
            cwd,
        } => (
            Some(command.to_string()),
            Some(serde_json::to_string(args).unwrap_or_else(|_| "[]".to_string())),
            Some(serde_json::to_string(env).unwrap_or_else(|_| "{}".to_string())),
            cwd.clone(),
            None,
        ),
        McpTransport::Http { url } => (None, None, None, None, Some(url.clone())),
    };
    McpServerRecord {
        name: config.name.to_string(),
        transport: McpClient::transport_kind(&config.transport).to_string(),
        command,
        args_json,
        env_json,
        cwd,
        url,
        auth_secret_ref: config.auth_secret_ref.clone(),
        // CQ-6 — a freshly-added server is `disconnected` until the
        // first health pass / connect (schema transition
        // `disconnected → health_pending → connected on add`).
        status: ServerStatus::Disconnected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::mcp::{McpServerName, McpTransportCommand};
    use std::collections::HashMap;
    use std::str::FromStr;

    fn stdio_cfg(name: &str, cwd: Option<&str>) -> McpServerConfig {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        McpServerConfig {
            name: McpServerName::from_str(name).unwrap(),
            transport: McpTransport::Stdio {
                command: McpTransportCommand::from_str("/usr/bin/echo").unwrap(),
                args: vec!["hello".into()],
                env,
                cwd: cwd.map(str::to_string),
            },
            auth_secret_ref: None,
        }
    }

    fn http_cfg(name: &str) -> McpServerConfig {
        McpServerConfig {
            name: McpServerName::from_str(name).unwrap(),
            transport: McpTransport::Http {
                url: "https://example.com".to_string(),
            },
            auth_secret_ref: Some(format!("mcp.{name}")),
        }
    }

    #[test]
    fn transport_kind_stdio_returns_stdio_string() {
        let cfg = stdio_cfg("a", None);
        assert_eq!(McpClient::transport_kind(&cfg.transport), "stdio");
    }

    #[test]
    fn transport_kind_http_returns_http_string() {
        let cfg = http_cfg("a");
        assert_eq!(McpClient::transport_kind(&cfg.transport), "http");
    }

    #[test]
    fn transport_from_config_stdio_constructs_stdio_transport() {
        let cfg = stdio_cfg("a", Some("/tmp"));
        let _t: Arc<dyn crate::transport::Transport> = McpClient::transport_from_config(&cfg);
    }

    #[test]
    fn transport_from_config_stdio_without_cwd_or_env_constructs_minimal_transport() {
        let cfg = McpServerConfig {
            name: McpServerName::from_str("min").unwrap(),
            transport: McpTransport::Stdio {
                command: McpTransportCommand::from_str("/bin/x").unwrap(),
                args: vec![],
                env: HashMap::new(),
                cwd: None,
            },
            auth_secret_ref: None,
        };
        let _t: Arc<dyn crate::transport::Transport> = McpClient::transport_from_config(&cfg);
    }

    #[test]
    fn transport_from_config_http_constructs_http_transport() {
        let cfg = http_cfg("a");
        let _t: Arc<dyn crate::transport::Transport> = McpClient::transport_from_config(&cfg);
    }

    #[test]
    fn config_to_record_stdio_populates_command_args_env_cwd_clears_url() {
        let cfg = stdio_cfg("a", Some("/tmp"));
        let r = config_to_record(&cfg);
        assert_eq!(r.transport, "stdio");
        assert_eq!(r.command.as_deref(), Some("/usr/bin/echo"));
        assert!(r.args_json.as_deref().unwrap().contains("hello"));
        assert!(r.env_json.as_deref().unwrap().contains("FOO"));
        assert_eq!(r.cwd.as_deref(), Some("/tmp"));
        assert!(r.url.is_none());
    }

    #[test]
    fn config_to_record_http_populates_url_clears_command_args_env_cwd() {
        let cfg = http_cfg("a");
        let r = config_to_record(&cfg);
        assert_eq!(r.transport, "http");
        assert_eq!(r.url.as_deref(), Some("https://example.com"));
        assert!(r.command.is_none());
        assert!(r.args_json.is_none());
        assert!(r.env_json.is_none());
        assert!(r.cwd.is_none());
    }
}
