//! Concrete MCP tool dispatcher — M06.D, ADR-0010.
//!
//! Implements `runtime_main::sdk::McpToolDispatch` (the SDK run-loop
//! seam defined in `runtime-main` so this crate's existing
//! `runtime-mcp → runtime-main` dependency stays acyclic). The
//! dispatcher routes a tool call:
//!
//! 1. [`NamespaceResolver::resolve`] (§5a) — `None`/`NotFound` ⇒ not an
//!    MCP tool, return `None` so the SDK falls through to the Stage A
//!    non-MCP L1 path; `Ambiguous` ⇒ `McpDispatchOutcome::Ambiguous`;
//!    `UnknownAlias` ⇒ `McpDispatchError::Config`.
//! 2. [`CapabilityEnforcer::check`] (Stage A's L1+L4 primitive) — on
//!    `Err` write the `mcp_request_blocked` audit line (gotcha #66
//!    correlation) + return `McpDispatchOutcome::Blocked`.
//! 3. [`Connection::call_tool`] via the injected [`ConnectionResolver`]
//!    seam — on `Ok` return `McpDispatchOutcome::Invoked`.
//!
//! [`ConnectionResolver`] decouples dispatch from the awkward
//! `McpClient::get_connection(name, transport)` signature (the `*_with`
//! testable-seam archetype): `McpClient` impls it for production;
//! tests inject a mock backed by `MockTransport`.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use runtime_core::generated::capability::CapabilityDeclaration;
use runtime_main::audit::AuditWriter;
use runtime_main::capability::CapabilityEnforcer;
use runtime_main::sdk::{McpDispatchError, McpDispatchOutcome, McpToolDispatch};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::error::McpError;
use crate::namespace::NamespaceResolver;
use crate::transport::Connection;

/// Seam that yields a live [`Connection`] for a resolved server name.
/// `McpClient` implements it for production; tests inject a mock.
#[async_trait]
pub trait ConnectionResolver: Send + Sync {
    /// Get (or establish) the connection for `server`.
    ///
    /// # Errors
    ///
    /// - [`McpError`] when the server is unknown or the (re)connect
    ///   fails.
    async fn connection(&self, server: &str) -> Result<Arc<dyn Connection>, McpError>;
}

/// Build the [`CapabilityDeclaration`] an MCP tool call needs.
///
/// v0.1 maps an MCP tool invocation to `kind = exec` (the
/// capability.v1.json "tool invocations" category) with the canonical
/// `<server>__<tool>` as the resource + a glob scope equal to the
/// canonical name (exact-match per the M05.B `scope_contains`
/// glob==glob equality rule). The framework grants the matching
/// declaration; the enforcer's `subsumes` accepts it.
#[must_use]
pub fn mcp_tool_capability(server: &str, tool: &str) -> CapabilityDeclaration {
    // Red-phase stub (M06.D strict TDD): green phase implements.
    let _ = (server, tool);
    unimplemented!("M06.D green phase: mcp_tool_capability")
}

/// Concrete MCP dispatcher (ADR-0010 impl side).
pub struct McpDispatcher {
    resolver: Arc<RwLock<NamespaceResolver>>,
    enforcer: Arc<CapabilityEnforcer>,
    connections: Arc<dyn ConnectionResolver>,
    audit: Option<Arc<AuditWriter>>,
    session_id: String,
}

impl McpDispatcher {
    /// Construct a dispatcher. `audit` is `None` in tests that don't
    /// assert the audit line; `Some` in production (the same
    /// `Arc<AuditWriter>` shared with the enforcer + `McpClient`).
    #[must_use]
    pub fn new(
        resolver: Arc<RwLock<NamespaceResolver>>,
        enforcer: Arc<CapabilityEnforcer>,
        connections: Arc<dyn ConnectionResolver>,
        audit: Option<Arc<AuditWriter>>,
        session_id: impl Into<String>,
    ) -> Self {
        // Red-phase stub (M06.D strict TDD): green phase implements.
        let _ = (resolver, enforcer, connections, audit, session_id.into());
        unimplemented!("M06.D green phase: McpDispatcher::new")
    }
}

#[async_trait]
impl McpToolDispatch for McpDispatcher {
    async fn dispatch_if_mcp(
        &self,
        agent_id: &str,
        tool_name: &str,
        args: Value,
        aliases: &BTreeMap<String, String>,
    ) -> Option<Result<McpDispatchOutcome, McpDispatchError>> {
        // Red-phase stub (M06.D strict TDD): green phase implements.
        let _ = (
            agent_id,
            tool_name,
            args,
            aliases,
            &self.resolver,
            &self.enforcer,
            &self.connections,
            &self.audit,
            &self.session_id,
        );
        unimplemented!("M06.D green phase: McpDispatcher::dispatch_if_mcp")
    }
}
