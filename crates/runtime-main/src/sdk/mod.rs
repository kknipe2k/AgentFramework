//! Agent SDK — wraps any [`LLMProvider`](crate::providers::LLMProvider) to
//! drive an agent loop and emit typed `runtime_core::AgentEvent`s. Spec §2.
//!
//! Submodules:
//! - `agent_sdk` — `AgentSdk<P>` struct + `run_agent` entry point.
//! - `event_pipeline` — pure `ProviderEvent` → `AgentEvent` translator with
//!   consecutive-`TextDelta` bundling.
//! - `structured_emitter` — delimited-block parser for `<<DECISION>>`+
//!   `<<PLAN>>` markers (M04 Stage B; replaces the M02 `decision_extractor`
//!   heuristic — closes M02 🟡 false-positive carry-forward).
//! - `approval` — `ApprovalSeam` (oneshot channel) the SDK awaits on for
//!   plan-approval HITL gates (M04 Stage B; Stage E wires the UI).
//! - `replay` — replay a saved signal log to the renderer event channel.

mod agent_sdk;
/// In-process, capability-scoped built-in tool executor — M08.7 rung 1.
///
/// Runs the runtime file built-ins (`Read`/`Write`) in-process behind
/// [`crate::capability::CapabilityEnforcer::check`] and feeds the result
/// back through the multi-turn loop's MCP-shared feedback contract.
pub mod builtin_tools;
mod event_pipeline;
/// MCP tool-dispatch seam — M06.D, ADR-0010 (dependency inversion).
///
/// Defines the [`mcp_dispatch::McpToolDispatch`] trait + the
/// [`mcp_dispatch::McpDispatchOutcome`] value type the SDK run loop
/// calls; the concrete dispatcher lives in `runtime-mcp` and is
/// shell-injected as `Arc<dyn McpToolDispatch>`.
pub mod mcp_dispatch;
pub mod replay;

/// Approval-gate seam — spec §3a (M04 Stage B).
pub mod approval;
/// `request_capability` meta-tool — spec §4b Layer 2 (M05 Stage A).
///
/// Inline-dispatched (not LLM-routed) meta-tool an agent invokes when it
/// realizes mid-task that it needs a tool / skill / MCP server / sub-agent
/// it doesn't have. Emits the appropriate `*_missing` event via the
/// `framework_loader::Emitter` and returns `Pending`; HITL `on_gap`
/// trigger drives the user-facing resolution flow.
pub mod request_capability;
/// Delimited-block emitter parser — spec §2 + §3a (M04 Stage B).
pub mod structured_emitter;

pub use agent_sdk::{AgentSdk, CapabilityWiring, SdkError, SessionId};
pub use approval::{ApprovalDecision, ApprovalError, ApprovalSeam};
pub use event_pipeline::{EnforcementContext, EventPipeline};
pub use mcp_dispatch::{
    apply_mcp_dispatch, apply_renderable, mcp_dispatch_error_event, outcome_needs_hitl,
    renderable_needs_hitl, McpDispatchError, McpDispatchOutcome, McpToolDispatch,
    RenderableOutcome,
};
pub use replay::replay_signals_to_events;
pub use request_capability::{
    handle_request_capability, CapabilityKind, RequestCapabilityError, RequestCapabilityInvocation,
    RequestCapabilityResult,
};
pub use structured_emitter::{parse_structured, EmitterError, EmitterOutput};
