//! In-process mock transport for unit testing.
//!
//! Gated behind `#[cfg(any(test, feature = "test-helpers"))]` so production
//! builds don't link it. Downstream crates that need deterministic
//! MCP-server fakes for their own tests pull `runtime-mcp` with the
//! `test-helpers` feature.
//!
//! The mock scripts behavior at the [`Transport`] / [`Connection`] trait
//! level — not at the raw byte-frame level (which is what
//! `tokio::io::duplex` covers for sandbox IPC at M05.C1). The MCP
//! protocol's seam is the JSON-RPC method surface; faking bytes
//! underneath would force every consumer test to reproduce the rmcp
//! wire format. Scripting at the trait level keeps tests focused on
//! consumer behavior (Stage C lifecycle, Stage D dispatch).
//!
//! Gotchas #72 (`tokio::io::duplex` EOF propagation) + #77 (duplex
//! buffer-vs-payload sizing) were considered when authoring this stage
//! — both apply to raw byte-frame IPC, not to MCP-trait-level mocks.
//! No duplex usage here.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;

use super::{Connection, McpTool, Transport};
use crate::error::McpError;

/// Builder + factory for [`MockConnection`].
///
/// Configure the scripted tool list, per-tool call results, per-tool call
/// errors, and ping behavior via builder methods. Calling [`MockTransport::connect`]
/// returns a fresh [`MockConnection`] that owns a clone of the scripted
/// state — multiple connections from one transport behave identically.
///
/// # Example
///
/// ```ignore
/// use runtime_mcp::transport::{MockTransport, Transport};
/// use serde_json::json;
///
/// let transport = MockTransport::new()
///     .with_tool("read_file", Some("Read a file"), json!({"type":"object"}))
///     .with_tool_result("read_file", json!({"contents": "hello"}));
///
/// let conn = transport.connect().await.unwrap();
/// let tools = conn.list_tools().await.unwrap();
/// assert_eq!(tools.len(), 1);
/// ```
#[derive(Debug, Default, Clone)]
pub struct MockTransport {
    state: ScriptedState,
}

#[derive(Debug, Default, Clone)]
struct ScriptedState {
    tools: Vec<McpTool>,
    results: BTreeMap<String, Value>,
    errors: BTreeMap<String, McpError>,
    ping_error: Option<McpError>,
    shutdown_error: Option<McpError>,
}

impl MockTransport {
    /// New mock transport with empty scripted state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tool to the scripted `list_tools` response.
    #[must_use]
    pub fn with_tool(
        mut self,
        name: impl Into<String>,
        description: Option<String>,
        input_schema: Value,
    ) -> Self {
        self.state.tools.push(McpTool {
            name: name.into(),
            description,
            input_schema,
        });
        self
    }

    /// Script a successful `call_tool(name, ..)` to return `result`.
    #[must_use]
    pub fn with_tool_result(mut self, name: impl Into<String>, result: Value) -> Self {
        self.state.results.insert(name.into(), result);
        self
    }

    /// Script a failing `call_tool(name, ..)` to return `error`.
    /// Errors take precedence over results for the same name.
    #[must_use]
    pub fn with_tool_error(mut self, name: impl Into<String>, error: McpError) -> Self {
        self.state.errors.insert(name.into(), error);
        self
    }

    /// Script `ping()` to return the supplied error instead of `Ok(())`.
    #[must_use]
    pub fn with_ping_error(mut self, error: McpError) -> Self {
        self.state.ping_error = Some(error);
        self
    }

    /// Script `shutdown()` to return the supplied error instead of `Ok(())`.
    #[must_use]
    pub fn with_shutdown_error(mut self, error: McpError) -> Self {
        self.state.shutdown_error = Some(error);
        self
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&self) -> Result<Box<dyn Connection>, McpError> {
        Ok(Box::new(MockConnection {
            state: Arc::new(Mutex::new(self.state.clone())),
        }))
    }
}

/// Live mock connection.
///
/// Methods consult the scripted state to produce responses. State is
/// wrapped in `Arc<Mutex<_>>` so the connection is `Send + Sync` (the
/// `Connection` trait contract).
#[derive(Debug)]
pub struct MockConnection {
    state: Arc<Mutex<ScriptedState>>,
}

#[async_trait]
impl Connection for MockConnection {
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        Ok(self.state.lock().await.tools.clone())
    }

    async fn call_tool(&self, name: &str, _arguments: Value) -> Result<Value, McpError> {
        let (scripted_err, scripted_result) = {
            let state = self.state.lock().await;
            (
                state.errors.get(name).cloned(),
                state.results.get(name).cloned(),
            )
        };
        if let Some(err) = scripted_err {
            return Err(err);
        }
        scripted_result.ok_or_else(|| McpError::ToolNotFound(name.to_string()))
    }

    async fn ping(&self) -> Result<(), McpError> {
        if let Some(err) = &self.state.lock().await.ping_error {
            return Err(err.clone());
        }
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), McpError> {
        if let Some(err) = &self.state.lock().await.shutdown_error {
            return Err(err.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn connect_returns_a_connection() {
        let transport = MockTransport::new();
        let _conn: Box<dyn Connection> = transport.connect().await.expect("connect");
    }

    #[tokio::test]
    async fn list_tools_returns_scripted_list_when_configured() {
        let transport = MockTransport::new()
            .with_tool(
                "read_file",
                Some("Read a file".into()),
                json!({"type": "object"}),
            )
            .with_tool("write_file", None, json!({"type": "object"}));
        let conn = transport.connect().await.unwrap();
        let tools = conn.list_tools().await.unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "read_file");
        assert_eq!(tools[0].description.as_deref(), Some("Read a file"));
        assert_eq!(tools[1].name, "write_file");
        assert!(tools[1].description.is_none());
    }

    #[tokio::test]
    async fn list_tools_returns_empty_when_unconfigured() {
        let conn = MockTransport::new().connect().await.unwrap();
        let tools = conn.list_tools().await.unwrap();
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn call_tool_returns_scripted_result() {
        let conn = MockTransport::new()
            .with_tool_result("echo", json!({"ok": true, "msg": "hi"}))
            .connect()
            .await
            .unwrap();
        let result = conn.call_tool("echo", json!({})).await.unwrap();
        assert_eq!(result, json!({"ok": true, "msg": "hi"}));
    }

    #[tokio::test]
    async fn call_tool_returns_scripted_error_when_configured() {
        let conn = MockTransport::new()
            .with_tool_error("bad", McpError::transport("simulated peer drop"))
            .connect()
            .await
            .unwrap();
        let err = conn.call_tool("bad", json!({})).await.unwrap_err();
        assert!(matches!(err, McpError::Transport(_)));
    }

    #[tokio::test]
    async fn call_tool_error_takes_precedence_over_result() {
        let conn = MockTransport::new()
            .with_tool_result("dual", json!({}))
            .with_tool_error("dual", McpError::Cancelled)
            .connect()
            .await
            .unwrap();
        let err = conn.call_tool("dual", json!({})).await.unwrap_err();
        assert!(matches!(err, McpError::Cancelled));
    }

    #[tokio::test]
    async fn call_tool_returns_tool_not_found_for_unscripted_name() {
        let conn = MockTransport::new().connect().await.unwrap();
        let err = conn.call_tool("missing", json!({})).await.unwrap_err();
        match err {
            McpError::ToolNotFound(name) => assert_eq!(name, "missing"),
            other => panic!("expected ToolNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn ping_succeeds_by_default() {
        let conn = MockTransport::new().connect().await.unwrap();
        conn.ping().await.expect("ping ok");
    }

    #[tokio::test]
    async fn ping_returns_scripted_error_when_configured() {
        let conn = MockTransport::new()
            .with_ping_error(McpError::Timeout { timeout_ms: 1000 })
            .connect()
            .await
            .unwrap();
        let err = conn.ping().await.unwrap_err();
        assert!(matches!(err, McpError::Timeout { timeout_ms: 1000 }));
    }

    #[tokio::test]
    async fn shutdown_succeeds_by_default() {
        let conn = MockTransport::new().connect().await.unwrap();
        conn.shutdown().await.expect("shutdown ok");
    }

    #[tokio::test]
    async fn shutdown_returns_scripted_error_when_configured() {
        let conn = MockTransport::new()
            .with_shutdown_error(McpError::transport("subprocess hung"))
            .connect()
            .await
            .unwrap();
        let err = conn.shutdown().await.unwrap_err();
        assert!(matches!(err, McpError::Transport(_)));
    }

    // gotcha #69 — multi-call invariants on every public method.

    #[tokio::test]
    async fn list_tools_twice_in_sequence_both_succeed() {
        let conn = MockTransport::new()
            .with_tool("t", None, json!({}))
            .connect()
            .await
            .unwrap();
        let first = conn.list_tools().await.unwrap();
        let second = conn.list_tools().await.unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn call_tool_twice_in_sequence_both_succeed() {
        let conn = MockTransport::new()
            .with_tool_result("t", json!({"n": 1}))
            .connect()
            .await
            .unwrap();
        let first = conn.call_tool("t", json!({})).await.unwrap();
        let second = conn.call_tool("t", json!({})).await.unwrap();
        assert_eq!(first, json!({"n": 1}));
        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn ping_twice_in_sequence_both_succeed() {
        let conn = MockTransport::new().connect().await.unwrap();
        conn.ping().await.unwrap();
        conn.ping().await.unwrap();
    }

    #[tokio::test]
    async fn connect_twice_yields_two_independent_connections() {
        let transport = MockTransport::new().with_tool("t", None, serde_json::json!({}));
        let c1 = transport.connect().await.unwrap();
        let c2 = transport.connect().await.unwrap();
        assert_eq!(c1.list_tools().await.unwrap().len(), 1);
        assert_eq!(c2.list_tools().await.unwrap().len(), 1);
    }

    #[test]
    fn mock_transport_is_send_plus_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockTransport>();
        assert_send_sync::<MockConnection>();
    }
}
