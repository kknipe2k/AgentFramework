//! Transport abstraction for MCP clients (spec §5).
//!
//! A two-trait pair separates *connection setup* from *connection use*:
//!
//! - [`Transport`] is a factory — calling [`Transport::connect`] performs
//!   the handshake (subprocess spawn + JSON-RPC `initialize`, or HTTP
//!   request + `initialize`) and returns a live [`Connection`].
//! - [`Connection`] is the live handle — [`Connection::list_tools`] +
//!   [`Connection::call_tool`] + [`Connection::ping`] drive the
//!   request-response surface; [`Connection::shutdown`] tears down
//!   cleanly.
//!
//! Both traits are [`Send`] + [`Sync`] so Stage C's lifecycle manager
//! can hold them in an `Arc` and share across the lifecycle loop +
//! the SDK dispatch path. Boxing (`Box<dyn Connection>`) keeps the
//! factory return type uniform across stdio + http + mock.
//!
//! [`McpTool`] is the runtime's internal tool descriptor — the format
//! consumers see after the rmcp-side `Tool` type is decoded. Stage D
//! consumes [`Vec<McpTool>`] when building the §5a namespace.
//!
//! ## Production transports
//!
//! [`StdioTransport`] wraps `rmcp::transport::TokioChildProcess` for local
//! subprocess MCP servers. [`HttpTransport`] wraps
//! `rmcp::transport::StreamableHttpClientTransport` for remote streamable-
//! HTTP MCP servers per MCP specification 2025-11-25.
//!
//! ## Test transport
//!
//! `MockTransport` is gated behind the `test-helpers` cargo feature; it
//! returns scripted tool lists + call results without any rmcp
//! involvement. Downstream consumers (Stage C lifecycle + Stage D
//! dispatch) link the mock via `runtime-mcp = { path = "...",
//! features = ["test-helpers"] }` and drive deterministic test
//! scenarios.

mod http;
mod stdio;

#[cfg(any(test, feature = "test-helpers"))]
mod mock;

pub use http::HttpTransport;
pub use stdio::StdioTransport;

#[cfg(any(test, feature = "test-helpers"))]
pub use mock::MockTransport;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::McpError;

/// Description of one tool an MCP server exposes.
///
/// Decoded from the rmcp wire-format `Tool` type by each transport's
/// [`Connection::list_tools`] impl. Stage D builds the §5a tool namespace
/// from `Vec<McpTool>` returned per server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpTool {
    /// Tool name as the server exposes it. May collide across servers;
    /// §5a namespace resolution (Stage D) disambiguates.
    pub name: String,

    /// Human-readable description. Optional — some servers omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema describing the tool's input parameters. Opaque to
    /// runtime-mcp; consumed by Stage D when validating capability
    /// declarations against tool calls.
    pub input_schema: Value,
}

/// Live MCP server connection.
///
/// Methods take `&self` so a [`Connection`] can be shared across tasks
/// via `Arc<dyn Connection>`. The underlying transport implementations
/// handle internal mutability (rmcp's `RunningService` is `Sync`).
#[async_trait]
pub trait Connection: Send + Sync {
    /// List the tools the server exposes.
    ///
    /// # Errors
    ///
    /// - [`McpError::Transport`] if the underlying transport errors.
    /// - [`McpError::Protocol`] if the response is malformed.
    /// - [`McpError::Timeout`] if the server doesn't respond in time.
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError>;

    /// Invoke a tool by name with JSON-encoded arguments.
    ///
    /// Returns the tool's structured result as a JSON value.
    ///
    /// # Errors
    ///
    /// - [`McpError::ToolNotFound`] if the server doesn't expose `name`.
    /// - [`McpError::Transport`] / [`McpError::Protocol`] /
    ///   [`McpError::Timeout`] per the underlying transport.
    /// - [`McpError::Cancelled`] if the caller cancels via the runtime's
    ///   cancellation token (Stage C / D).
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError>;

    /// Health-check ping. Round-trips a JSON-RPC `ping` request.
    ///
    /// # Errors
    ///
    /// - [`McpError::Transport`] if the connection is broken.
    /// - [`McpError::Timeout`] if the server doesn't respond in time.
    async fn ping(&self) -> Result<(), McpError>;

    /// Tear down the connection cleanly. After this returns, calling any
    /// other method on the same handle returns [`McpError::Transport`].
    ///
    /// # Errors
    ///
    /// - [`McpError::Transport`] if the underlying shutdown fails.
    async fn shutdown(&self) -> Result<(), McpError>;
}

/// Factory that produces live [`Connection`] handles.
///
/// Each [`Transport::connect`] call performs a full handshake — subprocess
/// spawn + JSON-RPC `initialize`, or HTTP request + `initialize`. Stage C
/// lifecycle holds one [`Transport`] per registered MCP server and calls
/// `connect` on add + on reconnect after a transient transport error.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the MCP server and return a live handle.
    ///
    /// # Errors
    ///
    /// - [`McpError::ConnectFailed`] if the handshake fails for any
    ///   reason — subprocess spawn failure, HTTP non-2xx, TLS error,
    ///   `initialize` JSON-RPC error, version mismatch.
    async fn connect(&self) -> Result<Box<dyn Connection>, McpError>;
}

/// Convert an rmcp protocol tool descriptor into the runtime's
/// internal [`McpTool`]. Shared by every [`Transport`] impl — the
/// rmcp `Tool` shape is the same regardless of stdio vs HTTP, so this
/// lives once at the transport-module root rather than per transport.
fn rmcp_tool_to_mcp_tool(tool: rmcp::model::Tool) -> McpTool {
    let input_schema = serde_json::to_value(&*tool.input_schema).unwrap_or(Value::Null);
    McpTool {
        name: tool.name.to_string(),
        description: tool.description.map(|d| d.to_string()),
        input_schema,
    }
}

/// Normalize a tool-call arguments [`Value`] into the `Map` rmcp's
/// `call_tool` requires. Shared by every [`Transport`] impl.
fn value_to_object(arguments: Value) -> Option<serde_json::Map<String, Value>> {
    match arguments {
        Value::Object(m) => Some(m),
        Value::Null => None,
        other => {
            // Wrap non-object payloads in {"value": other}; rmcp
            // requires Map for arguments. Stage D's dispatch layer
            // should be passing Map values; this is a defensive
            // shim with a single allocation.
            let mut m = serde_json::Map::new();
            m.insert("value".to_string(), other);
            Some(m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn value_to_object_passes_through_object() {
        let v = json!({"path": "/tmp"});
        let out = value_to_object(v).unwrap();
        assert_eq!(out.get("path").unwrap(), &json!("/tmp"));
    }

    #[test]
    fn value_to_object_maps_null_to_none() {
        assert!(value_to_object(Value::Null).is_none());
    }

    #[test]
    fn value_to_object_wraps_non_object_under_value_key() {
        let v = json!(42);
        let out = value_to_object(v).unwrap();
        assert_eq!(out.get("value").unwrap(), &json!(42));
    }

    #[test]
    fn mcptool_serde_round_trip_preserves_all_fields() {
        let original = McpTool {
            name: "read_file".to_string(),
            description: Some("Read a file from disk".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }),
        };
        let serialized = serde_json::to_string(&original).expect("serialize");
        let decoded: McpTool = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn mcptool_serde_omits_description_when_none() {
        let tool = McpTool {
            name: "ping".to_string(),
            description: None,
            input_schema: json!({}),
        };
        let serialized = serde_json::to_string(&tool).expect("serialize");
        assert!(!serialized.contains("description"));
    }

    #[test]
    fn mcptool_deserializes_when_description_missing() {
        let wire = r#"{"name":"ping","input_schema":{}}"#;
        let tool: McpTool = serde_json::from_str(wire).expect("deserialize");
        assert_eq!(tool.name, "ping");
        assert!(tool.description.is_none());
    }

    #[test]
    fn connection_trait_is_object_safe() {
        // Compile-time check: Box<dyn Connection> is valid.
        fn _assert_object_safe(_c: Box<dyn Connection>) {}
    }

    #[test]
    fn transport_trait_is_object_safe() {
        fn _assert_object_safe(_t: Box<dyn Transport>) {}
    }

    #[test]
    fn connection_is_send_plus_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn Connection>();
    }

    #[test]
    fn transport_is_send_plus_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn Transport>();
    }
}
