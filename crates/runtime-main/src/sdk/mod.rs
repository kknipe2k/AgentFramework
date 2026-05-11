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
mod event_pipeline;
pub mod replay;

/// Approval-gate seam — spec §3a (M04 Stage B).
pub mod approval;
/// Delimited-block emitter parser — spec §2 + §3a (M04 Stage B).
pub mod structured_emitter;

pub use agent_sdk::{AgentSdk, SdkError, SessionId};
pub use approval::{ApprovalDecision, ApprovalError, ApprovalSeam};
pub use event_pipeline::EventPipeline;
pub use replay::replay_signals_to_events;
pub use structured_emitter::{parse_structured, EmitterError, EmitterOutput};
