//! ADR-0011 (a) — `impl ConnectionResolver for McpClient` (M07.D1).
//!
//! M06 shipped the [`ConnectionResolver`] trait (`McpDispatcher`
//! consumes it to obtain a live [`Connection`] for a resolved server)
//! but no production impl — only a test mock — so a concrete
//! `McpDispatcher` was not constructible in `src-tauri` (ADR-0011
//! Context #1). [`McpClient`] is the natural home: it already owns the
//! registry + the live-connection cache. The impl rebuilds the
//! transport from the persisted registry record and delegates to
//! [`McpClient::get_connection`] (which caches), mapping the
//! lifecycle-layer [`LifecycleError`] onto the stable [`McpError`] the
//! dispatch path speaks.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;

use crate::client::error::LifecycleError;
use crate::client::registry::McpServerRecord;
use crate::client::McpClient;
use crate::dispatch::ConnectionResolver;
use crate::error::McpError;
use crate::transport::{Connection, HttpTransport, StdioTransport, Transport};

/// Collapse a lifecycle-layer error onto the stable dispatch-facing
/// [`McpError`]. A missing registry row is a connect-time failure class
/// (Stage C lifecycle's retry-vs-surface policy keys off
/// [`McpError::is_connect_failure`]); the name is preserved for the
/// audit log + renderer.
fn lifecycle_to_mcp(server: &str, e: LifecycleError) -> McpError {
    match e {
        LifecycleError::Mcp(m) => m,
        LifecycleError::NotFound(name) => {
            McpError::connect_failed(format!("MCP server not found: {name}"))
        }
        LifecycleError::Registry(re) => {
            McpError::transport(format!("registry error resolving '{server}': {re}"))
        }
        LifecycleError::AlreadyExists(name) => {
            McpError::connect_failed(format!("MCP server registry inconsistent for '{name}'"))
        }
        LifecycleError::Auth(a) => {
            McpError::connect_failed(format!("auth resolving '{server}': {a}"))
        }
        LifecycleError::Json(j) => {
            McpError::protocol(format!("registry row decode for '{server}': {j}"))
        }
    }
}

/// Rebuild the transport from a persisted registry record. Mirrors
/// [`McpClient::transport_from_config`] but reads the row shape (the
/// `connection()` path resolves by name → record, not by a live
/// `McpServerConfig`).
fn record_to_transport(record: &McpServerRecord) -> Result<Arc<dyn Transport>, McpError> {
    match record.transport.as_str() {
        "stdio" => {
            let command = record.command.as_deref().ok_or_else(|| {
                McpError::connect_failed(format!(
                    "stdio server '{}' has no command in the registry",
                    record.name
                ))
            })?;
            let args: Vec<String> = record
                .args_json
                .as_deref()
                .map(|s| serde_json::from_str(s).unwrap_or_default())
                .unwrap_or_default();
            let env: BTreeMap<String, String> = record
                .env_json
                .as_deref()
                .map(|s| serde_json::from_str(s).unwrap_or_default())
                .unwrap_or_default();
            let mut t = StdioTransport::new(command).with_args(args);
            if !env.is_empty() {
                t = t.with_env(env);
            }
            if let Some(cwd) = &record.cwd {
                t = t.with_cwd(PathBuf::from(cwd));
            }
            Ok(Arc::new(t))
        }
        "http" => {
            let url = record.url.as_deref().ok_or_else(|| {
                McpError::connect_failed(format!(
                    "http server '{}' has no url in the registry",
                    record.name
                ))
            })?;
            Ok(Arc::new(HttpTransport::new(url)))
        }
        other => Err(McpError::connect_failed(format!(
            "server '{}' has unknown transport '{other}'",
            record.name
        ))),
    }
}

#[async_trait]
impl ConnectionResolver for McpClient {
    async fn connection(&self, server: &str) -> Result<Arc<dyn Connection>, McpError> {
        let record = self
            .registry
            .get(server)
            .map_err(|e| lifecycle_to_mcp(server, e))?;
        let transport = record_to_transport(&record)?;
        self.get_connection(server, transport)
            .await
            .map_err(|e| lifecycle_to_mcp(server, e))
    }
}
