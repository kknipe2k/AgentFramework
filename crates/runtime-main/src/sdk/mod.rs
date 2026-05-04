//! Agent SDK — wraps any [`LLMProvider`](crate::providers::LLMProvider) to
//! drive an agent loop and emit typed `runtime_core::AgentEvent`s. Spec §2.
//!
//! Three submodules:
//! - `agent_sdk` — `AgentSdk<P>` struct + `run_agent` entry point.
//! - `event_pipeline` — pure `ProviderEvent` → `AgentEvent` translator with
//!   consecutive-`TextDelta` bundling.
//! - `decision_extractor` — heuristic first-line `Decision:`/`Rationale:`
//!   extractor (M02 ships the simplest version; M04 verify+rails replaces it
//!   with a structured emitter).

mod agent_sdk;
mod decision_extractor;
mod event_pipeline;

pub use agent_sdk::{AgentSdk, SdkError, SessionId};
pub use decision_extractor::{extract_decision, DecisionRecord};
pub use event_pipeline::EventPipeline;
