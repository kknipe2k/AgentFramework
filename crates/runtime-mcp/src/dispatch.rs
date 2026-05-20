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
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, ResourceName,
    SideEffectClass,
};
use runtime_main::audit::{self, AuditWriter};
use runtime_main::capability::CapabilityEnforcer;
use runtime_main::sdk::{McpDispatchError, McpDispatchOutcome, McpToolDispatch};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::namespace::NamespaceError;

use crate::error::McpError;
use crate::namespace::{NamespaceResolver, NewAmbiguity};
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
///
/// # Panics
///
/// Panics if `server` and `tool` are both empty (the canonical
/// `<server>__<tool>` would violate the schema `minLength 1` on
/// `ResourceName` / `GlobPattern`). Callers pass a `ResolvedTool` whose
/// segments came from a connected server's non-empty name + tool list,
/// so this is unreachable in the dispatch path.
#[must_use]
pub fn mcp_tool_capability(server: &str, tool: &str) -> CapabilityDeclaration {
    let canonical = format!("{server}__{tool}");
    CapabilityDeclaration {
        kind: CapabilityKind::Exec,
        // Both resource + glob are the canonical name. The M05.B
        // `scope_contains` glob==glob rule is equality, and `subsumes`
        // requires resource equality — a framework grant built from
        // this same fn for the same (server, tool) matches exactly.
        resource: ResourceName::from_str(&canonical)
            .expect("canonical <server>__<tool> is non-empty"),
        scope: CapabilityScope::Glob(
            GlobPattern::from_str(&canonical).expect("canonical glob is non-empty"),
        ),
        side_effect_class: SideEffectClass::Irreversible,
    }
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
        Self {
            resolver,
            enforcer,
            connections,
            audit,
            session_id: session_id.into(),
        }
    }

    /// §5a step 5 — re-resolution on server connect (ADR-0011 (b);
    /// M06.V 🟡 #1's "no production driver").
    ///
    /// Authored against `McpDispatcher`, NOT `McpClient`: the
    /// [`NamespaceResolver`] lives here per ADR-0010, so the production
    /// driver belongs here too. `McpClient` only *impls*
    /// [`ConnectionResolver`] (ADR-0011 (a)); the resolver re-evaluation
    /// is the dispatcher's responsibility. This is the M06.V Dec-6
    /// `<wire_trace_vs_adr_reconcile>` #6 reconciliation made concrete.
    ///
    /// Snapshots the connected server's tool set (through the same
    /// injected [`ConnectionResolver`] dispatch uses) into the resolver
    /// and returns the short names that BECAME ambiguous as a result —
    /// the caller (D2's agent-with-tools loop) emits a
    /// `tool_alias_ambiguous` event per [`NewAmbiguity`].
    ///
    /// # Errors
    ///
    /// - [`McpError`] when the server connection or its `list_tools`
    ///   handshake fails.
    pub async fn on_server_connected(&self, server: &str) -> Result<Vec<NewAmbiguity>, McpError> {
        let connection = self.connections.connection(server).await?;
        let tools = connection.list_tools().await?;
        let names: Vec<String> = tools.into_iter().map(|t| t.name).collect();
        Ok(self.resolver.write().await.connect_server(server, names))
    }

    /// §5a step 5 — re-resolution on server disconnect (ADR-0011 (b)).
    ///
    /// Drops the server from the resolver snapshot so subsequent
    /// `resolve` calls reflect the smaller connected set. A disconnect
    /// can only REMOVE ambiguity, never introduce it, so there is no
    /// new-ambiguity delta to surface (mirrors
    /// [`NamespaceResolver::disconnect_server`]).
    pub async fn on_server_disconnected(&self, server: &str) {
        self.resolver.write().await.disconnect_server(server);
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
        // §5a resolution. NotFound ⇒ not an MCP tool: return None so
        // the SDK falls through to the Stage A non-MCP L1 path.
        let resolved = {
            let resolver = self.resolver.read().await;
            match resolver.resolve(tool_name, aliases) {
                Ok(r) => r,
                Err(NamespaceError::NotFound(_)) => return None,
                Err(NamespaceError::Ambiguous { name, candidates }) => {
                    return Some(Ok(McpDispatchOutcome::Ambiguous { name, candidates }));
                }
                Err(NamespaceError::UnknownAlias(alias, canonical)) => {
                    return Some(Err(McpDispatchError::Config(format!(
                        "mcp_aliases entry '{alias}' points at unknown canonical '{canonical}'"
                    ))));
                }
            }
        };

        // L1+L4 capability check (Stage A's primitive). On deny: write
        // the mcp_request_blocked audit line (gotcha #66 correlation —
        // best-effort per §13.5: a log failure traces + continues, it
        // is not a dispatch gate) and return Blocked.
        let needed = mcp_tool_capability(&resolved.server, &resolved.tool);
        if let Err(e) = self.enforcer.check(agent_id, &needed) {
            let reason = e.to_string();
            if let Some(writer) = &self.audit {
                let entry = audit::mcp_request_blocked(
                    &self.session_id,
                    agent_id,
                    &resolved.server,
                    &resolved.tool,
                    &reason,
                );
                if let Err(log_err) = writer.log(&entry).await {
                    tracing::error!(
                        error = %log_err,
                        server = %resolved.server,
                        tool = %resolved.tool,
                        "audit mcp_request_blocked failed"
                    );
                }
            }
            return Some(Ok(McpDispatchOutcome::Blocked {
                agent_id: agent_id.to_string(),
                server: resolved.server,
                tool: resolved.tool,
                reason,
            }));
        }

        // Dispatch to the MCP server via the injected ConnectionResolver.
        let connection = match self.connections.connection(&resolved.server).await {
            Ok(c) => c,
            Err(e) => return Some(Err(McpDispatchError::Transport(e.to_string()))),
        };
        match connection.call_tool(&resolved.tool, args).await {
            Ok(value) => Some(Ok(McpDispatchOutcome::Invoked {
                server: resolved.server,
                tool: resolved.tool,
                value,
            })),
            Err(e) => Some(Err(McpDispatchError::Transport(e.to_string()))),
        }
    }
}
