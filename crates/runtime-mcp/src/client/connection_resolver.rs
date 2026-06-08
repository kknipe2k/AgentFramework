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
use crate::transport::{Connection, HttpTransport, McpTool, StdioTransport, Transport};

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

impl McpClient {
    /// Enumerate a *registered* server's tools by name (M09.C — the
    /// Palette's "attach an installed server's tool" source). Resolves the
    /// name through the same record→transport→connection path the
    /// dispatcher uses ([`ConnectionResolver::connection`]) and lists the
    /// server's tools. Read-only — no registry / secret mutation, no new
    /// transport.
    ///
    /// # Errors
    ///
    /// - [`McpError::ConnectFailed`] when `name` is not a registered server
    ///   (registry `NotFound`) or the connect handshake fails.
    /// - [`McpError`] when the `list_tools` call fails.
    pub async fn list_server_tools(&self, name: &str) -> Result<Vec<McpTool>, McpError> {
        let conn = self.connection(name).await?;
        conn.list_tools().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ServerStatus;

    fn rec(transport: &str) -> McpServerRecord {
        McpServerRecord {
            name: "srv".to_string(),
            transport: transport.to_string(),
            command: None,
            args_json: None,
            env_json: None,
            cwd: None,
            url: None,
            auth_secret_ref: None,
            status: ServerStatus::Disconnected,
        }
    }

    #[test]
    fn record_to_transport_stdio_full_builds_a_transport() {
        let mut r = rec("stdio");
        r.command = Some("/usr/bin/echo".to_string());
        r.args_json = Some(r#"["hello","world"]"#.to_string());
        r.env_json = Some(r#"{"FOO":"bar"}"#.to_string());
        r.cwd = Some("/tmp".to_string());
        assert!(
            record_to_transport(&r).is_ok(),
            "full stdio row → transport"
        );
    }

    #[test]
    fn record_to_transport_stdio_minimal_builds_a_transport() {
        let mut r = rec("stdio");
        r.command = Some("/bin/true".to_string());
        // No args_json / env_json / cwd — the unwrap_or_default arms.
        assert!(record_to_transport(&r).is_ok());
    }

    #[test]
    fn record_to_transport_stdio_without_command_errs_connect_failed() {
        let r = rec("stdio"); // command is None
                              // `Arc<dyn Transport>` is not Debug, so don't `{:?}` the Ok arm.
        match record_to_transport(&r) {
            Err(McpError::ConnectFailed(m)) => assert!(m.contains("no command")),
            Err(other) => panic!("expected ConnectFailed(no command), got {other:?}"),
            Ok(_) => panic!("expected ConnectFailed(no command), got Ok"),
        }
    }

    #[test]
    fn record_to_transport_http_builds_a_transport() {
        let mut r = rec("http");
        r.url = Some("https://example.com/mcp".to_string());
        assert!(record_to_transport(&r).is_ok());
    }

    #[test]
    fn record_to_transport_http_without_url_errs_connect_failed() {
        let r = rec("http"); // url is None
        match record_to_transport(&r) {
            Err(McpError::ConnectFailed(m)) => assert!(m.contains("no url")),
            Err(other) => panic!("expected ConnectFailed(no url), got {other:?}"),
            Ok(_) => panic!("expected ConnectFailed(no url), got Ok"),
        }
    }

    #[test]
    fn record_to_transport_unknown_transport_errs_connect_failed() {
        let r = rec("sse");
        match record_to_transport(&r) {
            Err(McpError::ConnectFailed(m)) => assert!(m.contains("unknown transport")),
            Err(other) => panic!("expected ConnectFailed(unknown transport), got {other:?}"),
            Ok(_) => panic!("expected ConnectFailed(unknown transport), got Ok"),
        }
    }

    #[test]
    fn lifecycle_to_mcp_passes_through_inner_mcp_error() {
        let mapped = lifecycle_to_mcp("srv", LifecycleError::Mcp(McpError::Cancelled));
        assert!(matches!(mapped, McpError::Cancelled));
    }

    #[test]
    fn lifecycle_to_mcp_not_found_is_connect_failed_naming_the_server() {
        let mapped = lifecycle_to_mcp("srv", LifecycleError::NotFound("ghost".to_string()));
        match mapped {
            McpError::ConnectFailed(m) => {
                assert!(m.contains("not found") && m.contains("ghost"), "got {m}");
            }
            other => panic!("expected ConnectFailed, got {other:?}"),
        }
    }

    #[test]
    fn lifecycle_to_mcp_registry_error_is_transport() {
        let mapped = lifecycle_to_mcp(
            "srv",
            LifecycleError::Registry(rusqlite::Error::InvalidColumnIndex(9)),
        );
        assert!(matches!(mapped, McpError::Transport(_)));
    }

    #[test]
    fn lifecycle_to_mcp_already_exists_and_auth_are_connect_failed() {
        assert!(matches!(
            lifecycle_to_mcp("srv", LifecycleError::AlreadyExists("dup".to_string())),
            McpError::ConnectFailed(_)
        ));
        assert!(matches!(
            lifecycle_to_mcp("srv", LifecycleError::auth("kc offline")),
            McpError::ConnectFailed(_)
        ));
    }

    #[test]
    fn lifecycle_to_mcp_json_error_is_protocol() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let mapped = lifecycle_to_mcp("srv", LifecycleError::Json(json_err));
        assert!(matches!(mapped, McpError::Protocol(_)));
    }

    #[tokio::test]
    async fn list_server_tools_enumerates_a_registered_servers_tools() {
        // M09.C — `list_server_tools` resolves a *registered* server by name
        // through the same `connection()` path dispatch uses (registry.get →
        // record_to_transport → cached connection) and returns its tools.
        // Seed the connection cache with a MockTransport-backed connection so
        // `get_connection` short-circuits to it before any real connect — the
        // unit observes the list_tools enumeration, not a subprocess spawn.
        use crate::client::{InMemorySecretStore, Registry, SecretStore};
        use crate::transport::{Connection, MockTransport, Transport};

        let dir = tempfile::TempDir::new().expect("tempdir");
        let registry =
            Arc::new(Registry::open(&dir.path().join("mcp.sqlite")).expect("open registry"));
        let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
        let client = McpClient::new(registry, secret_store, "sess-lst");

        // Register a stdio row so `connection()`'s registry.get +
        // record_to_transport both succeed for the name.
        let mut record = rec("stdio");
        record.name = "fs".to_string();
        record.command = Some("/bin/true".to_string());
        client.registry.insert(&record).expect("insert record");

        // Seed the cache with a scripted mock so the resolve returns it.
        let transport = MockTransport::new().with_tool(
            "read_file",
            Some("Read a file".to_string()),
            serde_json::json!({ "type": "object" }),
        );
        let conn: Arc<dyn Connection> = Arc::from(transport.connect().await.expect("mock connect"));
        client
            .connections
            .write()
            .await
            .insert("fs".to_string(), conn);

        let tools = client
            .list_server_tools("fs")
            .await
            .expect("list_server_tools enumerates the registered server's tools");
        assert_eq!(tools.len(), 1, "the registered server's single tool enumerates");
        assert_eq!(tools[0].name, "read_file");
    }

    #[tokio::test]
    async fn list_server_tools_for_unregistered_server_errs() {
        // Read-only enumeration of an unknown name surfaces the registry
        // NotFound (mapped to ConnectFailed) rather than an empty list.
        use crate::client::{InMemorySecretStore, Registry, SecretStore};
        let dir = tempfile::TempDir::new().expect("tempdir");
        let registry =
            Arc::new(Registry::open(&dir.path().join("mcp.sqlite")).expect("open registry"));
        let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
        let client = McpClient::new(registry, secret_store, "sess-lst");
        let result = client.list_server_tools("ghost").await;
        assert!(
            matches!(result, Err(McpError::ConnectFailed(_))),
            "an unregistered name maps registry NotFound → ConnectFailed, got {result:?}"
        );
    }
}
