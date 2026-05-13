//! Runtime core: shared types for the agent runtime.
//!
//! Types in `generated/` are emitted by `cargo xtask regenerate-types` from
//! `schemas/*.v1.json` — do not hand-edit them. Types in `event.rs`, `drone.rs`,
//! and `error.rs` are hand-curated; they are the contract every later milestone
//! evolves.

/// Inherent + trait impls for the typify-generated [`CmdError`].
/// Adds helper constructors, `Display`, and `std::error::Error` so the
/// generated tuple-variant enum has parity with the M02 hand-rolled
/// struct-variant enum it replaces. M04 Stage A2.
mod cmd_error_ext;
/// Drone IPC types — events and commands for main↔drone communication.
pub mod drone;
/// Error types for the runtime.
pub mod error;
/// Canonical event union emitted by the runtime.
pub mod event;
/// Types generated from JSON schemas via typify.
pub mod generated;
/// Signal Schema v2 — forensic event log types (spec §2b).
pub mod signal;

pub use drone::{
    ActivityState, AlertLevel, DroneCommand, DroneEvent, HeartbeatStatus, ProcessConfig,
    ProcessType, RevertReason, StopReason,
};
pub use error::RuntimeError;
pub use event::{AgentEvent, GapSeverityRef, GapSourceRef, ToolSource};
// Re-export only schema-derived modules whose names don't collide with the
// hand-curated top-level modules above. `generated::event` and
// `generated::error` (M04 Stage A1 codegen extensions) are reachable via
// `runtime_core::generated::{event, error}`; lifting them here would
// shadow `runtime_core::event` / `runtime_core::error`. The wire-format
// `CmdError` + `ErrorMessage` are lifted by name (no collision with the
// hand-curated `RuntimeError` in `error::`).
pub use generated::error::{CmdError, ErrorMessage};
pub use generated::{agent, budget, common, framework, plan, skill, task, tool};
