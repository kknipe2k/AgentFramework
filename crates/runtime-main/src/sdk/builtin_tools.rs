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

use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use serde_json::{json, Value};

use crate::capability::{subsumes, CapabilityEnforcer, CapabilityError, DenyReason};
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
            let lexical = normalize_lexical(path);
            let decl = file_decl(CapabilityKind::Read, &lexical, SideEffectClass::Pure)?;
            enforcer
                .check(agent_id, &decl)
                .map_err(BuiltinExecError::Capability)?;
            // Resolution AFTER the lexical check passed — a missing or
            // unresolvable out-of-scope path is denied above and leaks no
            // existence information here.
            let canonical = std::fs::canonicalize(Path::new(&lexical))
                .map_err(|e| BuiltinExecError::Op(format!("read '{path}': {e}")))?;
            confine_to_grant_anchor(enforcer, agent_id, &decl, &canonical)?;
            let content = std::fs::read_to_string(&canonical)
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
            let lexical = normalize_lexical(path);
            let decl = file_decl(
                CapabilityKind::Write,
                &lexical,
                SideEffectClass::FilesystemMutate,
            )?;
            enforcer
                .check(agent_id, &decl)
                .map_err(BuiltinExecError::Capability)?;
            let canonical = canonicalize_for_write(Path::new(&lexical))
                .map_err(|e| BuiltinExecError::Op(format!("write '{path}': {e}")))?;
            confine_to_grant_anchor(enforcer, agent_id, &decl, &canonical)?;
            std::fs::write(&canonical, content)
                .map_err(|e| BuiltinExecError::Op(format!("write '{path}': {e}")))?;
            Ok(json!({ "ok": true, "bytes_written": content.len() }))
        }
        other => Err(BuiltinExecError::Op(format!(
            "not an in-process built-in: {other}"
        ))),
    }
}

// ── TD-052 path resolution (review C3) ────────────────────────────────────
//
// SYMLINK POLICY — resolve-then-check. The model-supplied path is reduced
// to a lexically normalized form ONCE; that form feeds the L1 scope check
// (grant space unchanged — authored globs keep their exact semantics for
// in-scope paths). After the check passes, the path is resolved to its
// canonical form (symlinks/junctions followed, `..` and short names
// collapsed by the OS) and the IO runs on that canonical path ONLY IF it
// is confined under the canonicalized LITERAL ANCHOR of a matching grant
// (canonical-to-canonical comparison — both sides through
// `fs::canonicalize`, so the Windows `\\?\` verbatim prefix, short-name
// expansion, and macOS `/var → /private/var` never poison the match). A
// symlink inside the grant pointing outside it is therefore DENIED; a
// link resolving inside the grant is allowed.
//
// Residual, stated honestly (CLAUDE.md §4 rule 11): containment confines
// resolved targets to the grant's LITERAL ANCHOR — equal to the full
// scope for literal-prefix grants (`{dir}/**`, the v0.1 norm), COARSER
// for metachar-bearing grants (under `{tmp}/*/out/**` a symlink can
// still move laterally anywhere under `{tmp}/`), and VACUOUS for a bare
// `**` grant by that grant's own semantics. The claim is "resolved
// targets cannot escape the grant's literal anchor", never "symlinks
// cannot escape the glob".
//
// Ordering invariant: layer 2 (containment) runs ONLY after layer 1 (the
// enforcer's L4→L1 check) passes — denied-by-glob paths get the identical
// denial surface as before and no existence information leaks. TOCTOU:
// v0.1 holds no handle across check→use; an OS-level race between
// resolution and IO remains and is accepted (documented, not solved).

/// Lexically normalize a model-supplied path: `.` dropped, `..` resolved
/// against preceding components (absolute paths cannot ascend past the
/// root; a relative path's irreducible leading `..` is KEPT so the glob
/// check denies it naturally), separators unified to `/`. Relative paths
/// stay relative — the grant-match base is unchanged (TD-035 stays open).
fn normalize_lexical(raw: &str) -> String {
    let mut prefix: Option<String> = None;
    let mut rooted = false;
    let mut parts: Vec<String> = Vec::new();
    for comp in Path::new(raw).components() {
        match comp {
            Component::Prefix(p) => {
                prefix = Some(p.as_os_str().to_string_lossy().replace('\\', "/"));
            }
            Component::RootDir => rooted = true,
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.last().is_some_and(|c| c != "..") {
                    parts.pop();
                } else if !rooted {
                    parts.push("..".to_string());
                }
                // rooted with nothing to pop: `/..` is `/` — drop.
            }
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
        }
    }
    let mut out = prefix.unwrap_or_default();
    if rooted {
        out.push('/');
    }
    out.push_str(&parts.join("/"));
    if out.is_empty() {
        ".".to_string()
    } else {
        out
    }
}

/// Canonicalize a Write target that may not exist yet: resolve the
/// nearest existing ancestor via `fs::canonicalize` and re-join the
/// non-existing remainder. A skipped component that exists for
/// `symlink_metadata` but fails `canonicalize` is a dangling link — its
/// target cannot be verified, so the Write is refused (resolve-then-check
/// cannot vouch for it). Residual `..` in the remainder is rejected
/// defensively (the lexical normalizer leaves none in absolute paths).
fn canonicalize_for_write(lexical: &Path) -> std::io::Result<PathBuf> {
    let mut existing = lexical.to_path_buf();
    let mut remainder: Vec<std::ffi::OsString> = Vec::new();
    loop {
        match std::fs::canonicalize(&existing) {
            Ok(canon) => {
                let mut out = canon;
                for part in remainder.iter().rev() {
                    if part == ".." {
                        return Err(std::io::Error::other(
                            "residual '..' after lexical normalization",
                        ));
                    }
                    out.push(part);
                }
                return Ok(out);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if std::fs::symlink_metadata(&existing).is_ok() {
                    return Err(std::io::Error::other(format!(
                        "'{}' is a dangling link — unresolvable target refused",
                        existing.display()
                    )));
                }
                let Some(name) = existing.file_name() else {
                    return Err(e);
                };
                remainder.push(name.to_os_string());
                let parent = existing.parent().unwrap_or_else(|| Path::new("."));
                existing = if parent.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    parent.to_path_buf()
                };
            }
            Err(e) => return Err(e),
        }
    }
}

/// Layer 2 of the TD-052 gate (see the policy comment above): the
/// canonical resolved target must sit under the canonicalized literal
/// anchor of at least one grant that subsumes the (lexical) request.
/// Runs ONLY after `enforcer.check` passed.
fn confine_to_grant_anchor(
    enforcer: &CapabilityEnforcer,
    agent_id: &str,
    requested: &CapabilityDeclaration,
    canonical_target: &Path,
) -> Result<(), BuiltinExecError> {
    for grant in enforcer
        .grants_for(agent_id)
        .iter()
        .filter(|g| subsumes(g, requested))
    {
        match grant_literal_anchor(&grant.scope) {
            // A pattern with no literal prefix (`**`) confines nothing by
            // its own semantics — vacuous containment (see the residual
            // note above).
            None => return Ok(()),
            // The ancestor-walk (not bare `canonicalize`) so a grant
            // whose anchor directory does not exist YET still anchors —
            // the IO then fails parent-missing as an `Op` error exactly
            // as before this gate existed.
            Some(anchor) => {
                if let Ok(canon_anchor) = canonicalize_for_write(Path::new(&anchor)) {
                    if canonical_target.starts_with(&canon_anchor) {
                        return Ok(());
                    }
                }
            }
        }
    }
    tracing::warn!(
        agent_id,
        requested = ?requested.scope,
        resolved = %canonical_target.display(),
        "TD-052 containment denial: the resolved target escapes every \
         matching grant's literal anchor (symlink/junction or traversal)"
    );
    Err(BuiltinExecError::Capability(CapabilityError::Denied {
        agent_id: agent_id.to_string(),
        reason: DenyReason::NoMatchingGrant,
    }))
}

/// The literal directory anchor of a grant scope: for a glob, the prefix
/// before the first metachar (`*?[{`), trimmed to the last whole
/// component; for a `Path` prefix grant, the entire string. `None` means
/// the pattern has no literal prefix (it starts with a metachar — e.g. a
/// bare `**`). Windows-authored backslash separators are unified first.
fn grant_literal_anchor(scope: &CapabilityScope) -> Option<String> {
    let pattern = match scope {
        CapabilityScope::Glob(g) => {
            let p = if cfg!(windows) {
                g.replace('\\', "/")
            } else {
                g.to_string()
            };
            let cut = p.find(['*', '?', '[', '{']).unwrap_or(p.len());
            let literal = &p[..cut];
            // Cut AT the last separator (the partial trailing component is
            // never part of the anchor); trailing separators trimmed.
            literal[..literal.rfind('/').unwrap_or(0)]
                .trim_end_matches('/')
                .to_string()
        }
        CapabilityScope::Path(p) => p.trim_end_matches('/').to_string(),
        CapabilityScope::Domain(_) => return None,
    };
    if pattern.is_empty() {
        None
    } else {
        Some(pattern)
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
