//! In-process, capability-scoped built-in tool executor (M08.7 rung 1).
//!
//! The runtime built-in *file* tools — `Read` (and its close sibling
//! `Write`) — run **in-process** under the agent's capability scope. The
//! capability scope IS the boundary (Hard Rule 8): every op builds a
//! per-op `CapabilityDeclaration` and runs it through
//! [`crate::capability::CapabilityEnforcer::check`] BEFORE touching the
//! filesystem; an out-of-scope op is denied, never executed, and never
//! spawns a process.
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
//! [`crate::sdk::builtin_tools::execute_builtin`] between the MCP-dispatch
//! branch and the emit-only
//! pipeline path, and feeds the result back through the SAME multi-turn
//! feedback contract MCP uses, so built-in and MCP tools converge on one
//! path.

use std::str::FromStr;

use serde_json::{json, Value};

use crate::capability::{CapabilityEnforcer, CapabilityError};
use crate::providers::ToolDef;

use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, PathPattern, ResourceName,
    SideEffectClass,
};

/// Built-in `Read` tool name (Anthropic tool-use naming convention).
pub const READ_TOOL: &str = "Read";
/// Built-in `Write` tool name.
pub const WRITE_TOOL: &str = "Write";

/// The `resource` every in-process file op declares. Matches the grant
/// shape [`crate::framework_loader::capabilities_to_declarations`] derives
/// from an agent's `file_access` block, so `subsumes(grant, request)`
/// matches on `kind + resource + scope + side_effect_class`.
const FS_RESOURCE: &str = "filesystem";

/// Is `name` an in-process built-in this executor runs? v0.1 rung 1:
/// `Read` + `Write` only.
#[must_use]
pub fn is_builtin_tool(name: &str) -> bool {
    matches!(name, READ_TOOL | WRITE_TOOL)
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
    enforcer: &CapabilityEnforcer,
    agent_id: &str,
    tool_name: &str,
    input: &Value,
) -> Result<Value, BuiltinExecError> {
    let path = input
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| BuiltinExecError::Op("built-in tool input missing 'path' string".into()))?;
    match tool_name {
        READ_TOOL => {
            enforcer
                .check(
                    agent_id,
                    &file_decl(CapabilityKind::Read, path, SideEffectClass::Pure)?,
                )
                .map_err(BuiltinExecError::Capability)?;
            let content = std::fs::read_to_string(path)
                .map_err(|e| BuiltinExecError::Op(format!("read '{path}': {e}")))?;
            Ok(json!({ "content": content }))
        }
        WRITE_TOOL => {
            let content = input
                .get("content")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    BuiltinExecError::Op("Write input missing 'content' string".into())
                })?;
            enforcer
                .check(
                    agent_id,
                    &file_decl(
                        CapabilityKind::Write,
                        path,
                        SideEffectClass::FilesystemMutate,
                    )?,
                )
                .map_err(BuiltinExecError::Capability)?;
            std::fs::write(path, content)
                .map_err(|e| BuiltinExecError::Op(format!("write '{path}': {e}")))?;
            Ok(json!({ "ok": true, "bytes_written": content.len() }))
        }
        other => Err(BuiltinExecError::Op(format!(
            "not an in-process built-in: {other}"
        ))),
    }
}

/// Build the `Path`-scoped request declaration for a file op on `path`,
/// mirroring the `file_access` grant shape so `subsumes` matches.
fn file_decl(
    kind: CapabilityKind,
    path: &str,
    side_effect_class: SideEffectClass,
) -> Result<CapabilityDeclaration, BuiltinExecError> {
    Ok(CapabilityDeclaration {
        kind,
        resource: ResourceName::from_str(FS_RESOURCE).expect("constant non-empty resource"),
        scope: CapabilityScope::Path(
            PathPattern::from_str(path)
                .map_err(|_| BuiltinExecError::Op(format!("invalid path: '{path}'")))?,
        ),
        side_effect_class,
    })
}

/// Advertise the in-process built-ins among `allowed_tools` as `ToolDef`s.
///
/// The model needs each tool advertised to emit the matching `ToolUse`.
/// Non-built-in names (MCP / framework tools) are skipped — those
/// advertise through their own path.
#[must_use]
pub fn builtin_tool_defs(allowed_tools: &[String]) -> Vec<ToolDef> {
    allowed_tools
        .iter()
        .filter(|n| is_builtin_tool(n))
        .map(|n| builtin_tool_def(n))
        .collect()
}

/// The `ToolDef` for a single in-process built-in. `name` is an in-process
/// built-in (callers gate on [`is_builtin_tool`]); `Write` carries the
/// `content` field, everything else is the `Read` shape.
fn builtin_tool_def(name: &str) -> ToolDef {
    if name == WRITE_TOOL {
        ToolDef {
            name: WRITE_TOOL.to_string(),
            description: "Write a UTF-8 text file at `path` with `content`, within the agent's \
                          file_access.write capability scope."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Filesystem path to write." },
                    "content": { "type": "string", "description": "UTF-8 text to write." }
                },
                "required": ["path", "content"]
            }),
        }
    } else {
        ToolDef {
            name: READ_TOOL.to_string(),
            description: "Read a UTF-8 text file at `path`, within the agent's file_access.read \
                          capability scope."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Filesystem path to read." }
                },
                "required": ["path"]
            }),
        }
    }
}
