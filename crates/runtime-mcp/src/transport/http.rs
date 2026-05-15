//! Streamable-HTTP transport (spec §5; MCP specification 2025-11-25).
//!
//! Wraps `rmcp::transport::StreamableHttpClientTransport` (rmcp 1.7.0
//! with the `transport-streamable-http-client-reqwest` + `reqwest`
//! features). The HTTP transport speaks JSON-RPC 2.0 over a
//! POST-then-stream protocol per MCP specification 2025-11-25;
//! `rmcp::ServiceExt::serve` (called via `ClientInfo::serve`) performs
//! the `initialize` handshake and returns a live
//! `rmcp::service::RunningService<RoleClient, ClientInfo>` we wrap in
//! [`HttpConnection`].
//!
//! Coverage holdout: [`HttpTransport::connect`]'s happy path requires a
//! real MCP-protocol-compliant HTTP server. Wiremock can stand in for
//! HTTP connect failures (404 / 500 / connection refused) but cannot
//! easily satisfy rmcp's full `initialize` JSON-RPC handshake without
//! reimplementing the protocol in the mock. Parallel to
//! `runtime-main::providers::anthropic` per CLAUDE.md §5.

use async_trait::async_trait;
use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation};
use rmcp::service::RunningService;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::ServiceExt;
use serde_json::Value;

use super::{Connection, McpTool, Transport};
use crate::error::McpError;

/// Streamable-HTTP transport for a remote MCP server.
#[derive(Debug, Clone)]
pub struct HttpTransport {
    url: String,
}

impl HttpTransport {
    /// New transport pointing at `url`. Connection is lazy — actual
    /// HTTP traffic happens only inside [`Transport::connect`].
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }

    /// Returns the configured server URL. Useful for diagnostics and
    /// for matching against a registry entry (Stage C).
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Build the rmcp `ClientInfo` we hand to `ServiceExt::serve`.
    /// Pure logic; exposed for tests via `pub(crate)`.
    pub(crate) fn build_client_info() -> ClientInfo {
        ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("agent-runtime", env!("CARGO_PKG_VERSION")),
        )
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn connect(&self) -> Result<Box<dyn Connection>, McpError> {
        let transport = StreamableHttpClientTransport::from_uri(self.url.clone());
        let client_info = Self::build_client_info();
        let service = client_info
            .serve(transport)
            .await
            .map_err(McpError::connect_failed)?;
        Ok(Box::new(HttpConnection { service }))
    }
}

/// Live MCP connection over streamable HTTP.
pub struct HttpConnection {
    service: RunningService<rmcp::RoleClient, ClientInfo>,
}

#[async_trait]
impl Connection for HttpConnection {
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let result = self
            .service
            .list_all_tools()
            .await
            .map_err(McpError::transport)?;
        Ok(result.into_iter().map(rmcp_tool_to_mcp_tool).collect())
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        let mut params = CallToolRequestParams::new(name.to_string());
        if let Some(args) = value_to_object(arguments) {
            params = params.with_arguments(args);
        }
        let result = self
            .service
            .call_tool(params)
            .await
            .map_err(McpError::transport)?;
        serde_json::to_value(result).map_err(McpError::protocol)
    }

    async fn ping(&self) -> Result<(), McpError> {
        // See StdioConnection::ping notes — rmcp 1.7.0 lacks a
        // client-side ping; we use `list_tools` as a liveness probe.
        self.service
            .list_tools(Option::default())
            .await
            .map_err(McpError::transport)?;
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), McpError> {
        // See StdioConnection::shutdown notes — Connection trait takes
        // &self; RunningService::cancel takes self; downstream Stage
        // C lifecycle holds the connection by value and tears down
        // directly.
        Ok(())
    }
}

fn rmcp_tool_to_mcp_tool(tool: rmcp::model::Tool) -> McpTool {
    let input_schema = serde_json::to_value(&*tool.input_schema).unwrap_or(Value::Null);
    McpTool {
        name: tool.name.to_string(),
        description: tool.description.map(|d| d.to_string()),
        input_schema,
    }
}

fn value_to_object(arguments: Value) -> Option<serde_json::Map<String, Value>> {
    match arguments {
        Value::Object(m) => Some(m),
        Value::Null => None,
        other => {
            let mut m = serde_json::Map::new();
            m.insert("value".to_string(), other);
            Some(m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn new_stores_url() {
        let t = HttpTransport::new("http://localhost:8000/mcp");
        assert_eq!(t.url(), "http://localhost:8000/mcp");
    }

    #[test]
    fn build_client_info_advertises_agent_runtime_name() {
        let info = HttpTransport::build_client_info();
        assert_eq!(info.client_info.name, "agent-runtime");
        // Version pulled from CARGO_PKG_VERSION at build time.
        assert!(!info.client_info.version.is_empty());
    }

    #[tokio::test]
    async fn connect_returns_connect_failed_for_404_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let url = format!("{}/mcp", server.uri());
        let t = HttpTransport::new(url);
        match t.connect().await {
            Ok(_) => panic!("expected connect to fail on 404"),
            Err(err) => assert!(
                err.is_connect_failure(),
                "expected ConnectFailed, got {err:?}"
            ),
        }
    }

    #[tokio::test]
    async fn connect_returns_connect_failed_for_500_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        let url = format!("{}/mcp", server.uri());
        let t = HttpTransport::new(url);
        match t.connect().await {
            Ok(_) => panic!("expected connect to fail on 500"),
            Err(err) => assert!(
                err.is_connect_failure(),
                "expected ConnectFailed, got {err:?}"
            ),
        }
    }

    #[tokio::test]
    async fn connect_returns_connect_failed_for_unreachable_url() {
        // Use a port that's almost certainly closed; rmcp's reqwest
        // client refuses the connection → ConnectFailed.
        let t = HttpTransport::new("http://127.0.0.1:1/mcp");
        match t.connect().await {
            Ok(_) => panic!("expected connect to fail for unreachable url"),
            Err(err) => assert!(
                err.is_connect_failure(),
                "expected ConnectFailed, got {err:?}"
            ),
        }
    }

    #[test]
    fn value_to_object_passes_through_object() {
        let v = serde_json::json!({"a": 1});
        let out = value_to_object(v).unwrap();
        assert_eq!(out.get("a").unwrap(), &serde_json::json!(1));
    }

    #[test]
    fn value_to_object_maps_null_to_none() {
        assert!(value_to_object(Value::Null).is_none());
    }

    #[test]
    fn value_to_object_wraps_non_object_under_value_key() {
        let v = serde_json::json!("hi");
        let out = value_to_object(v).unwrap();
        assert_eq!(out.get("value").unwrap(), &serde_json::json!("hi"));
    }
}
