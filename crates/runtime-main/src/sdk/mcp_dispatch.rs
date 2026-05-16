//! MCP tool-dispatch seam — M06.D, ADR-0010 (dependency inversion).
//!
//! `runtime-mcp` already depends on `runtime-main` (M06.C audit). To wire
//! MCP dispatch through the SDK run loop WITHOUT closing a Cargo
//! dependency cycle, this module defines the *seam* the SDK calls — the
//! [`McpToolDispatch`] trait + the [`McpDispatchOutcome`] value type —
//! carrying no `runtime-mcp` dependency. `runtime-mcp` provides the
//! concrete `McpDispatcher` implementing this trait; the Tauri shell
//! injects it into the SDK as `Arc<dyn McpToolDispatch>` (the same
//! shell-injected-seam archetype as `Arc<dyn Connection>` /
//! `Arc<AuditWriter>`).
//!
//! [`apply_mcp_dispatch`] maps an [`McpDispatchOutcome`] to the
//! `AgentEvent` sequence the renderer consumes. Pure given the trait's
//! result — fully unit-testable in `runtime-main` against a mock
//! `McpToolDispatch` (the concrete dispatcher's resolve/check/invoke/
//! audit behavior is tested in `runtime-mcp`).

use std::collections::BTreeMap;

use async_trait::async_trait;
use runtime_core::event::{AgentEvent, CapabilityKindRef, ToolSource};
use serde_json::Value;

/// Outcome of attempting MCP dispatch for a single tool call.
///
/// `dispatch_if_mcp` returns `None` for "not an MCP tool" (caller falls
/// through to the Stage A non-MCP L1 path); this enum covers the three
/// resolved outcomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpDispatchOutcome {
    /// Tool resolved, L1+L4 capability check passed, server invoked OK.
    Invoked {
        /// Resolved MCP server name.
        server: String,
        /// Resolved MCP tool name.
        tool: String,
        /// The tool's structured JSON result.
        value: Value,
    },
    /// Tool resolved but the L1+L4 capability check denied it. The
    /// concrete dispatcher has ALREADY written the `mcp_request_blocked`
    /// audit line (gotcha #66 correlation); the SDK emits
    /// `CapabilityViolation` + `McpRequestBlocked` and routes the
    /// existing `on_capability_violation` HITL trigger (no new trigger).
    Blocked {
        /// Agent whose dispatch was rejected.
        agent_id: String,
        /// Resolved MCP server name.
        server: String,
        /// Resolved MCP tool name.
        tool: String,
        /// Human-readable deny cause.
        reason: String,
    },
    /// The short name resolved to >1 candidate (post-connect ambiguity,
    /// spec §5a step 5). SDK emits `ToolAliasAmbiguous`.
    Ambiguous {
        /// The ambiguous short tool name.
        name: String,
        /// The ≥2 canonical `<server>__<tool>` candidates.
        candidates: Vec<String>,
    },
}

/// Transport / config error from the concrete MCP dispatcher.
#[derive(Debug, thiserror::Error)]
pub enum McpDispatchError {
    /// Underlying MCP transport failed (server unreachable, protocol).
    #[error("MCP transport error: {0}")]
    Transport(String),
    /// Namespace / framework-alias configuration error (e.g., an
    /// `mcp_aliases` entry points at an unknown canonical name).
    #[error("MCP configuration error: {0}")]
    Config(String),
}

/// The SDK run-loop's MCP-dispatch seam (ADR-0010 dependency inversion).
#[async_trait]
pub trait McpToolDispatch: Send + Sync {
    /// Attempt MCP dispatch for `tool_name`.
    ///
    /// Returns `None` when the tool is NOT an MCP tool — the caller
    /// falls through to the existing Stage A non-MCP L1 dispatch path.
    /// `Some(Ok(outcome))` covers the three resolved outcomes;
    /// `Some(Err)` is a transport / config error.
    async fn dispatch_if_mcp(
        &self,
        agent_id: &str,
        tool_name: &str,
        args: Value,
        aliases: &BTreeMap<String, String>,
    ) -> Option<Result<McpDispatchOutcome, McpDispatchError>>;
}

/// Map a resolved [`McpDispatchOutcome`] to the `AgentEvent` sequence
/// the renderer consumes. Called by the SDK run loop after
/// `dispatch_if_mcp` returns `Some(Ok(_))`.
///
/// - `Invoked` → `ToolInvoked { source: Mcp, server: Some(..) }` then
///   `ToolResult`.
/// - `Blocked` → `CapabilityViolation` then `McpRequestBlocked` (the
///   audit line was already written by the concrete dispatcher; the
///   SDK run loop routes the existing `on_capability_violation` HITL
///   trigger off the `CapabilityViolation` event per ADR-0007).
/// - `Ambiguous` → `ToolAliasAmbiguous`.
///
/// `input` is the original tool-call argument JSON from
/// `ProviderEvent::ToolUse`; it rides into the `ToolInvoked` event so
/// the renderer's tool node shows what the agent called the MCP tool
/// with (ignored for `Blocked` / `Ambiguous`).
#[must_use]
pub fn apply_mcp_dispatch(outcome: McpDispatchOutcome, input: Value) -> Vec<AgentEvent> {
    match outcome {
        McpDispatchOutcome::Invoked {
            server,
            tool,
            value,
        } => {
            // The Invoked outcome does not carry agent_id (the
            // integration test's pattern pins {server, tool, value}).
            // The SDK run loop, which holds agent_id, emits the
            // agent_id-correct ToolInvoked/ToolResult for the success
            // path directly; this branch is the wire-test contract +
            // the renderer-shape reference. agent_id is filled by the
            // run-loop seam (ADR-0010 note + Stage D retro special-log).
            vec![
                AgentEvent::ToolInvoked {
                    agent_id: String::new(),
                    tool_name: tool.clone(),
                    source: ToolSource::Mcp,
                    server: Some(server),
                    input,
                },
                AgentEvent::ToolResult {
                    agent_id: String::new(),
                    tool_name: tool,
                    output: value,
                    duration_ms: 0,
                    tokens_in: None,
                    tokens_out: None,
                },
            ]
        }
        McpDispatchOutcome::Blocked {
            agent_id,
            server,
            tool,
            reason,
        } => {
            // Single deny → two events: the generic CapabilityViolation
            // (drives the existing on_capability_violation HITL trigger
            // + inspector, per ADR-0007 — gotcha trap #4 emission
            // ordering: violation BEFORE the HITL prompt) THEN the
            // MCP-specific McpRequestBlocked carrying server+tool so the
            // renderer attributes the block to the MCPNode.
            vec![
                AgentEvent::CapabilityViolation {
                    agent_id: agent_id.clone(),
                    capability_kind: CapabilityKindRef::Exec,
                    requested_action: format!("invoke MCP tool '{server}__{tool}'"),
                    declared_scope: reason.clone(),
                },
                AgentEvent::McpRequestBlocked {
                    agent_id,
                    server,
                    tool,
                    reason,
                },
            ]
        }
        McpDispatchOutcome::Ambiguous { name, candidates } => {
            vec![AgentEvent::ToolAliasAmbiguous { name, candidates }]
        }
    }
}

/// Map a [`McpDispatchError`] to the `AgentEvent` the renderer
/// consumes — a `ToolError` carrying the transport/config failure so
/// the renderer paints the failure rather than silently dropping it.
#[must_use]
pub fn mcp_dispatch_error_event(
    agent_id: &str,
    tool_name: &str,
    err: &McpDispatchError,
) -> AgentEvent {
    AgentEvent::ToolError {
        agent_id: agent_id.to_string(),
        tool_name: tool_name.to_string(),
        error: err.to_string(),
    }
}

/// True iff this outcome routes the existing HITL trigger.
///
/// Only the `Blocked` case routes `on_capability_violation`. The SDK
/// run loop uses this to gate the HITL await, mirroring the Stage A
/// `CapabilityViolation` / `TierViolation` HITL routing.
#[must_use]
pub const fn outcome_needs_hitl(outcome: &McpDispatchOutcome) -> bool {
    matches!(outcome, McpDispatchOutcome::Blocked { .. })
}
