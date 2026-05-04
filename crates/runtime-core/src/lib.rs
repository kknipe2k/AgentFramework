//! Runtime core: shared types for the agent runtime.
//!
//! Types in `generated/` are emitted by `cargo xtask regenerate-types` from
//! `schemas/*.v1.json` — do not hand-edit them. Types in `event.rs`, `drone.rs`,
//! and `error.rs` are hand-curated; they are the contract every later milestone
//! evolves.

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
pub use event::{AgentEvent, ToolSource};
pub use generated::*;
