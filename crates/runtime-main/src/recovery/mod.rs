//! Recovery primitive — spec §1b (M04 Stage F).
//!
//! Two halves:
//! - `resume` — coordinate session resume via drone IPC. Loads the
//!   latest snapshot + projected plan/task rows + the set of uncertain
//!   tool invocations, then surfaces a `ResumePlan` the SDK uses to
//!   rebuild message history WITHOUT re-invoking tools (gotcha #15).
//! - `uncertainty` — handle the user's choice for each uncertain tool
//!   invocation. Emits `tool_call_uncertainty_resolved` decision signals
//!   so the VDR projection records the audit trail.
//!
//! MCP reconnect is a no-op seam at v0.1 (no MCP servers configured per
//! §0d STANDARD-mode scope; M5/M6 wire the real path).
//! Capability state restoration is a placeholder at v0.1 (capability
//! enforcement is M5).
//!
//! Safety primitive: ≥95% coverage per CLAUDE.md §5.

/// Resume coordinator.
pub mod resume;
/// Tool-call uncertainty handler.
pub mod uncertainty;

pub use resume::{request_resume_with, ResumeError, ResumePlan};
pub use uncertainty::{
    respond_uncertainty_with, ToolCallUncertaintyAction, UncertaintyError, UncertaintyResolution,
};
