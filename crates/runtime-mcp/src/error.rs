//! Error mapping for the runtime-mcp surface.
//!
//! [`McpError`] is the stable error type runtime-mcp consumers see. It
//! collapses rmcp's internal error variants (which evolve across rmcp
//! versions) into six stable categories aligned with the §5 protocol
//! contract:
//!
//! - [`ConnectFailed`](McpError::ConnectFailed) — initial handshake failed
//!   (subprocess spawn error, HTTP non-2xx, TLS error, JSON-RPC
//!   `initialize` rejection).
//! - [`Transport`](McpError::Transport) — connection succeeded but the
//!   underlying byte stream errored mid-session (peer crash, network
//!   blip, TLS reset).
//! - [`Protocol`](McpError::Protocol) — wire-format violation (malformed
//!   JSON-RPC, unexpected message ordering, schema mismatch).
//! - [`Timeout`](McpError::Timeout) — operation exceeded the configured
//!   deadline (per-call, applied by Stage C lifecycle).
//! - [`ToolNotFound`](McpError::ToolNotFound) — server replied that the
//!   requested tool name is unknown.
//! - [`Cancelled`](McpError::Cancelled) — operation aborted by the caller
//!   (Stage C HITL deny path, Stage D capability violation).
//!
//! Per CLAUDE.md §9: errors carry root cause. Transport+Protocol+
//! `ConnectFailed` variants embed an owned [`String`] message so the
//! original rmcp error display is preserved for the audit log + renderer.

use thiserror::Error;

/// Stable error surface for runtime-mcp.
///
/// `Clone` is derived so test-helpers can script per-call error
/// responses without juggling `Box<dyn Fn() -> McpError>` factories
/// (see `transport::MockTransport`, gated behind the `test-helpers`
/// cargo feature).
#[derive(Debug, Clone, Error)]
pub enum McpError {
    /// Initial handshake to an MCP server failed.
    #[error("MCP connect failed: {0}")]
    ConnectFailed(String),

    /// Transport-level error mid-session (peer drop, network reset).
    #[error("MCP transport error: {0}")]
    Transport(String),

    /// Wire-format protocol violation.
    #[error("MCP protocol error: {0}")]
    Protocol(String),

    /// Operation exceeded the configured deadline.
    #[error("MCP operation timed out after {timeout_ms} ms")]
    Timeout {
        /// Timeout window in milliseconds.
        timeout_ms: u64,
    },

    /// Server reported the requested tool name is unknown.
    #[error("MCP tool not found: {0}")]
    ToolNotFound(String),

    /// Operation aborted by the caller.
    #[error("MCP operation cancelled")]
    Cancelled,
}

impl McpError {
    /// Build a [`McpError::ConnectFailed`] from any displayable cause.
    #[must_use]
    pub fn connect_failed(cause: impl std::fmt::Display) -> Self {
        Self::ConnectFailed(cause.to_string())
    }

    /// Build a [`McpError::Transport`] from any displayable cause.
    #[must_use]
    pub fn transport(cause: impl std::fmt::Display) -> Self {
        Self::Transport(cause.to_string())
    }

    /// Build a [`McpError::Protocol`] from any displayable cause.
    #[must_use]
    pub fn protocol(cause: impl std::fmt::Display) -> Self {
        Self::Protocol(cause.to_string())
    }

    /// Returns true when the variant is one of the connect-time failure
    /// classes (vs a mid-session error). Used by Stage C lifecycle to
    /// decide retry vs surface-to-user.
    #[must_use]
    pub const fn is_connect_failure(&self) -> bool {
        matches!(self, Self::ConnectFailed(_))
    }

    /// Returns true when the variant signals the operation may succeed
    /// on retry (transient transport / timeout). Stage C reads this when
    /// applying the health-ping retry policy.
    #[must_use]
    pub const fn is_transient(&self) -> bool {
        matches!(self, Self::Transport(_) | Self::Timeout { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_failed_display_includes_cause() {
        let err = McpError::connect_failed("subprocess not found");
        assert_eq!(err.to_string(), "MCP connect failed: subprocess not found");
    }

    #[test]
    fn transport_display_includes_cause() {
        let err = McpError::transport("peer dropped");
        assert_eq!(err.to_string(), "MCP transport error: peer dropped");
    }

    #[test]
    fn protocol_display_includes_cause() {
        let err = McpError::protocol("unexpected message ordering");
        assert_eq!(
            err.to_string(),
            "MCP protocol error: unexpected message ordering"
        );
    }

    #[test]
    fn timeout_display_includes_window() {
        let err = McpError::Timeout { timeout_ms: 5000 };
        assert_eq!(err.to_string(), "MCP operation timed out after 5000 ms");
    }

    #[test]
    fn tool_not_found_display_includes_name() {
        let err = McpError::ToolNotFound("read_file".to_string());
        assert_eq!(err.to_string(), "MCP tool not found: read_file");
    }

    #[test]
    fn cancelled_display_is_stable() {
        assert_eq!(McpError::Cancelled.to_string(), "MCP operation cancelled");
    }

    #[test]
    fn is_connect_failure_only_for_connect_failed_variant() {
        assert!(McpError::connect_failed("x").is_connect_failure());
        assert!(!McpError::transport("x").is_connect_failure());
        assert!(!McpError::protocol("x").is_connect_failure());
        assert!(!McpError::Timeout { timeout_ms: 1 }.is_connect_failure());
        assert!(!McpError::ToolNotFound("x".into()).is_connect_failure());
        assert!(!McpError::Cancelled.is_connect_failure());
    }

    #[test]
    fn is_transient_covers_transport_and_timeout_only() {
        assert!(!McpError::connect_failed("x").is_transient());
        assert!(McpError::transport("x").is_transient());
        assert!(!McpError::protocol("x").is_transient());
        assert!(McpError::Timeout { timeout_ms: 1 }.is_transient());
        assert!(!McpError::ToolNotFound("x".into()).is_transient());
        assert!(!McpError::Cancelled.is_transient());
    }

    #[test]
    fn error_implements_std_error_trait() {
        fn assert_error<E: std::error::Error>(_e: &E) {}
        assert_error(&McpError::Cancelled);
    }

    #[test]
    fn error_is_send_plus_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<McpError>();
    }
}
