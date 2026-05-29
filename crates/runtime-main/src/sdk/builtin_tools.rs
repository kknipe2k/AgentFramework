//! In-process, capability-scoped built-in tool executor (M08.7 rung 1).
//!
//! The runtime built-in *file* tools — `Read` (and its close sibling
//! `Write`) — run **in-process** under the agent's capability scope. The
//! capability scope IS the boundary (Hard Rule 8): every op builds a
//! per-op `CapabilityDeclaration` and runs it through
//! [`CapabilityEnforcer::check`] BEFORE touching the filesystem; an
//! out-of-scope op is denied, never executed, and never spawns a process.
//!
//! Scope lock (M08.7 rung 1): `Read`/`Write` only. `Bash` and any
//! OS-spawning op are a SEPARATE ADR-class rung — the sandbox protocol has
//! no command-execution variant (`runtime-sandbox/src/protocol.rs`), and
//! adding one is an IPC-protocol change requiring an ADR + CODEOWNERS
//! review. This module does NOT touch the sandbox.
//!
//! The executor is path-string-parameterised and capability-checked, so it
//! is `tempfile`-testable without OS wiring (CLAUDE.md §9 path-agnostic
//! archetype). The SDK run loop ([`crate::sdk::AgentSdk`]) calls
//! [`execute_builtin`] between the MCP-dispatch branch and the emit-only
//! pipeline path, and feeds the result back through the SAME multi-turn
//! feedback contract MCP uses, so built-in and MCP tools converge on one
//! path.

use serde_json::Value;

use crate::capability::{CapabilityEnforcer, CapabilityError};
use crate::providers::ToolDef;

/// Built-in `Read` tool name (Anthropic tool-use naming convention).
pub const READ_TOOL: &str = "Read";
/// Built-in `Write` tool name.
pub const WRITE_TOOL: &str = "Write";

/// Is `name` an in-process built-in this executor runs? v0.1 rung 1:
/// `Read` + `Write` only.
#[must_use]
pub const fn is_builtin_tool(_name: &str) -> bool {
    // RED skeleton — filled in the impl commit.
    false
}

/// Failure executing an in-process built-in.
#[derive(Debug)]
pub enum BuiltinExecError {
    /// The capability check denied the op (L4 tier or L1 grant). The op
    /// did NOT run; the run loop maps this to `CapabilityViolation` /
    /// `TierViolation` (rung 2 verifies the blocked-side behavior).
    Capability(CapabilityError),
    /// The op ran but failed after the check passed — malformed input or
    /// an IO error. Fed back to the model as an error `tool_result` so the
    /// multi-turn loop survives.
    Op(String),
}

/// Execute one in-process built-in (`Read`/`Write`) under capability scope.
///
/// Builds the `Path`-scoped per-op capability declaration, runs
/// `enforcer.check(agent_id, &decl)`, and only on `Ok` touches the
/// filesystem.
///
/// # Errors
///
/// - [`BuiltinExecError::Capability`] when the check denies the op (no
///   filesystem access happens).
/// - [`BuiltinExecError::Op`] for malformed input / IO failure.
pub fn execute_builtin(
    _enforcer: &CapabilityEnforcer,
    _agent_id: &str,
    _tool_name: &str,
    _input: &Value,
) -> Result<Value, BuiltinExecError> {
    // RED skeleton — filled in the impl commit.
    unimplemented!("M08.7.A executor — impl commit")
}

/// Advertise the in-process built-ins among `allowed_tools` as `ToolDef`s.
///
/// The model needs each tool advertised to emit the matching `ToolUse`.
/// Non-built-in names (MCP / framework tools) are skipped — those
/// advertise through their own path.
#[must_use]
pub const fn builtin_tool_defs(_allowed_tools: &[String]) -> Vec<ToolDef> {
    // RED skeleton — filled in the impl commit.
    Vec::new()
}
