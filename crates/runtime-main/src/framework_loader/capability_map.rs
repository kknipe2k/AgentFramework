//! Capability translation helpers — M06 Stage A wire-up support.
//!
//! Two getters the SDK consumes at production call sites (M06.A wire-up
//! of the M05.B L1 + L2a primitives):
//!
//! - [`capabilities_for_tool`] — given a tool name declared in
//!   `framework.tools[]`, returns the per-action capability declarations
//!   the L1 enforcer's `check(agent_id, &needed)` consumes. v0.1's
//!   framework-tool declarations name only `{name, source}`; per-action
//!   capability needs derive from the source class (`builtin` → `Pure`
//!   `Exec`; `generated` / `external` / `registry` → `Irreversible`
//!   `Exec`). When framework.json grows tool-declaration capability
//!   metadata in M07+, this function widens to consume it.
//!
//! - [`parent_grants_for_agent`] — given a declared agent id, returns
//!   the grant set the L2a `narrow(parent, proposed)` evaluator
//!   consumes as `parent`. Translates the agent's coarse `Capabilities`
//!   block (`tools_called` / `skills_loaded` / `file_access` /
//!   `network` / `shell` / `spawn_agents`) into the per-action
//!   `CapabilityDeclaration` set the enforcer + narrowing primitives
//!   ground on. Returns `None` when no agent with that id is declared
//!   in the framework.
//!
//! Both helpers are free functions over the typify-generated
//! `runtime_core::generated::framework::Framework` (typify-generated
//! types cannot be extended outside `runtime-core`; the established
//! pattern is free functions in the consuming crate — same shape as
//! `crate::capability::declaration::subsumes`). Per ADR-0009 closure +
//! M06 phase doc A.3.3.

use std::str::FromStr;
use std::sync::Arc;

use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, DomainPattern, GlobPattern,
    ResourceName, SideEffectClass,
};
use runtime_core::generated::framework::{
    Agent, Capabilities, Framework, FrameworkAgentsItem, FrameworkToolsItem,
    FrameworkToolsItemSource,
};
use thiserror::Error;

/// Failure modes raised by [`capabilities_for_tool`].
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CapabilityLookupError {
    /// The tool name was not found in `framework.tools[]`. The L1 wire
    /// emits `tool_missing` (Layer 1 gap source) so the renderer's
    /// `GapPanel` surfaces the unresolved reference.
    #[error("tool '{name}' not declared in framework.tools[]")]
    ToolNotFound {
        /// Tool name the SDK looked up.
        name: String,
    },
}

/// Glob pattern reused for any-resource scoping (`*`). The
/// L1 + L2a primitives compare `kind + resource + scope +
/// side_effect_class` — the resource is the tool's own name; the scope
/// is `Glob(*)` because tool dispatches don't carry per-resource
/// constraints in v0.1 (file/network constraints attach to the agent's
/// Capabilities block instead).
const ANY_GLOB: &str = "*";

/// Per-tool capability set the L1 enforcer's `check` consumes when an
/// agent dispatches `tool_name`.
///
/// Returns `Ok(vec![CapabilityDeclaration {kind: Exec, resource:
/// tool_name, scope: Glob(*), side_effect_class: Pure | Irreversible}])`
/// for any tool declared in `framework.tools[]`. Returns
/// [`CapabilityLookupError::ToolNotFound`] when the lookup misses (the
/// L1 wire fans this through the gap-detection seam as `tool_missing`).
///
/// Per CLAUDE.md §6 schema-as-source-of-truth + the M05.B L1 contract:
/// the per-action declaration MUST mirror the agent's grant declaration
/// exactly so `subsumes(grant, requested)` matches. The grant
/// translator [`parent_grants_for_agent`] uses the same `(Exec, name,
/// Glob(*), Pure)` shape for every entry in
/// `Capabilities::tools_called`.
///
/// # Errors
///
/// - [`CapabilityLookupError::ToolNotFound`] when `tool_name` does not
///   match any `framework.tools[].name`.
pub fn capabilities_for_tool(
    framework: &Framework,
    tool_name: &str,
) -> Result<Vec<CapabilityDeclaration>, CapabilityLookupError> {
    let tool = framework
        .tools
        .iter()
        .find(|t| matches_tool_name(t, tool_name))
        .ok_or_else(|| CapabilityLookupError::ToolNotFound {
            name: tool_name.to_string(),
        })?;
    Ok(vec![tool_to_declaration(tool_name, tool.source)])
}

fn matches_tool_name(tool: &FrameworkToolsItem, name: &str) -> bool {
    tool.name == name
}

fn tool_to_declaration(name: &str, source: FrameworkToolsItemSource) -> CapabilityDeclaration {
    let side_effect_class = match source {
        // Built-in tools are statically reviewed; M01-M05 ship the
        // current set. Default to `Pure` — file / network side effects
        // attach to the agent's Capabilities block, not the tool's
        // exec declaration.
        FrameworkToolsItemSource::Builtin => SideEffectClass::Pure,
        // Generator + external + registry tools are not statically
        // reviewable; default to the most conservative classification
        // (`Irreversible`) until per-tool metadata lands at M07+ tool
        // import.
        FrameworkToolsItemSource::Generated
        | FrameworkToolsItemSource::External
        | FrameworkToolsItemSource::Registry => SideEffectClass::Irreversible,
    };
    CapabilityDeclaration {
        kind: CapabilityKind::Exec,
        resource: ResourceName::from_str(name).unwrap_or_else(|_| {
            // Tool names in framework.tools[] are non-empty by schema;
            // a runtime conversion failure means the framework parsed
            // a malformed entry. Fall back to a placeholder so the
            // enforcer surfaces a `NoMatchingGrant` rather than
            // panicking — gotcha #66 (contract failure must surface as
            // a deniable event, not a crash).
            ResourceName::from_str("invalid_tool_name").expect("constant non-empty")
        }),
        scope: CapabilityScope::Glob(
            GlobPattern::from_str(ANY_GLOB).expect("constant non-empty glob"),
        ),
        side_effect_class,
    }
}

/// Per-agent grant set the L2a `narrow` evaluator consumes as `parent`.
///
/// Returns `Some(declarations)` when `agent_id` is found in
/// `framework.agents[]` (inline `Agent` form); returns `None` for
/// unknown ids OR for `FrameworkAgentsItem::Object { id, path }` items
/// (the registry-import form whose capability declaration lives in the
/// referenced agent.md file — M07 scope).
///
/// Translation rules from `Capabilities` → `Vec<CapabilityDeclaration>`:
///
/// - `tools_called[name]` →
///   `(Exec, name, Glob(*), Pure)` — matches
///   [`capabilities_for_tool`]'s declaration shape.
/// - `skills_loaded[name]` →
///   `(Exec, name, Glob(*), Pure)` — skills are loadable like tools at
///   the L1 dispatch surface; future tier-aware code may differentiate.
/// - `file_access.read[glob]` →
///   `(Read, "filesystem", Glob(glob), Pure)`.
/// - `file_access.write[glob]` →
///   `(Write, "filesystem", Glob(glob), FilesystemMutate)`.
/// - `network[host]` →
///   `(Network, "host", Domain(host), NetworkEgress)`.
/// - `shell == true` →
///   `(ProcessSpawn, "shell", Glob(*), ProcessSpawn)`.
/// - `spawn_agents[id]` →
///   `(ProcessSpawn, id, Glob(*), ProcessSpawn)`.
///
/// The `resource` field disambiguates per-action targets — the L2a
/// `narrow` evaluator's `subsumes(grant, requested)` predicate compares
/// `kind + resource + scope + side_effect_class` exactly. The
/// translator pins these consistently so a child's proposed grant
/// produced by re-running the same translator on the child's
/// `Capabilities` block matches its parent's grant set when the child
/// is a strict subset.
#[must_use]
pub fn parent_grants_for_agent(
    framework: &Framework,
    agent_id: &str,
) -> Option<Vec<CapabilityDeclaration>> {
    let agent = framework.agents.iter().find_map(|item| match item {
        FrameworkAgentsItem::Agent(a) if a.id.as_str() == agent_id => Some(a),
        _ => None,
    })?;
    Some(capabilities_to_declarations(&agent.capabilities))
}

/// Translate a coarse `Capabilities` block into the per-action
/// declaration set the L1 + L2a primitives consume.
///
/// Public so M06.A integration tests + future M06.D MCP dispatch tests
/// can construct fixtures without re-deriving the translation shape.
#[must_use]
pub fn capabilities_to_declarations(caps: &Capabilities) -> Vec<CapabilityDeclaration> {
    let mut out: Vec<CapabilityDeclaration> = Vec::new();

    for tool in &caps.tools_called {
        out.push(CapabilityDeclaration {
            kind: CapabilityKind::Exec,
            resource: resource_or_placeholder(tool),
            scope: CapabilityScope::Glob(any_glob()),
            side_effect_class: SideEffectClass::Pure,
        });
    }

    for skill in &caps.skills_loaded {
        out.push(CapabilityDeclaration {
            kind: CapabilityKind::Exec,
            resource: resource_or_placeholder(skill),
            scope: CapabilityScope::Glob(any_glob()),
            side_effect_class: SideEffectClass::Pure,
        });
    }

    for read_glob in caps.file_access.read.iter() {
        if let Ok(g) = GlobPattern::from_str(read_glob) {
            out.push(CapabilityDeclaration {
                kind: CapabilityKind::Read,
                resource: resource_or_placeholder("filesystem"),
                scope: CapabilityScope::Glob(g),
                side_effect_class: SideEffectClass::Pure,
            });
        }
    }

    for write_glob in caps.file_access.write.iter() {
        if let Ok(g) = GlobPattern::from_str(write_glob) {
            out.push(CapabilityDeclaration {
                kind: CapabilityKind::Write,
                resource: resource_or_placeholder("filesystem"),
                scope: CapabilityScope::Glob(g),
                side_effect_class: SideEffectClass::FilesystemMutate,
            });
        }
    }

    for host in &caps.network {
        if let Ok(d) = DomainPattern::from_str(host) {
            out.push(CapabilityDeclaration {
                kind: CapabilityKind::Network,
                resource: resource_or_placeholder(host),
                scope: CapabilityScope::Domain(d),
                side_effect_class: SideEffectClass::NetworkEgress,
            });
        }
    }

    if caps.shell {
        out.push(CapabilityDeclaration {
            kind: CapabilityKind::ProcessSpawn,
            resource: resource_or_placeholder("shell"),
            scope: CapabilityScope::Glob(any_glob()),
            side_effect_class: SideEffectClass::ProcessSpawn,
        });
    }

    for child_id in &caps.spawn_agents {
        out.push(CapabilityDeclaration {
            kind: CapabilityKind::ProcessSpawn,
            resource: resource_or_placeholder(child_id),
            scope: CapabilityScope::Glob(any_glob()),
            side_effect_class: SideEffectClass::ProcessSpawn,
        });
    }

    out
}

/// Render a [`CapabilityDeclaration`] to a short string for the wire.
///
/// Suitable for the `AgentSpawned.narrowed_from` field (M06.A schema
/// addition). Format is `kind:resource:scope:side_effect_class` with
/// the scope variant inlined as `glob:<pattern>` / `domain:<host>` /
/// `path:<prefix>`.
#[must_use]
pub fn declaration_to_narrowed_from_str(decl: &CapabilityDeclaration) -> String {
    let scope_str = match &decl.scope {
        CapabilityScope::Glob(g) => format!("glob:{}", **g),
        CapabilityScope::Domain(d) => format!("domain:{}", **d),
        CapabilityScope::Path(p) => format!("path:{}", **p),
    };
    format!(
        "{kind:?}:{resource}:{scope_str}:{class:?}",
        kind = decl.kind,
        resource = *decl.resource,
        class = decl.side_effect_class,
    )
    .to_lowercase()
}

/// Resolve a framework agent id to its display name (`role`).
///
/// M08.5 🔴-2 — the Builder's Tester emits the session root agent as
/// `AgentSpawned { agent_id, agent_name }`. The id derives from
/// `framework.session_root_agent`; the name must derive from the matching
/// inline agent's `role`, mirroring how `spawn_framework_subagents`
/// already names every sub-agent (`crates/runtime-main/src/sdk/agent_sdk.rs:480`).
///
/// Returns:
///
/// - the inline agent's `role` when `agent_id` matches a
///   `FrameworkAgentsItem::Agent`;
/// - the literal `agent_id` for the `{ id, path }` path-ref form (no
///   inline `role` available; resolving the external `.md` is M08.6's
///   loader work, ADR-0022);
/// - the literal `agent_id` when no `framework.agents[]` entry matches
///   (the same id-as-name fallback as the path-ref case).
///
/// Pure; safe to call on every session start.
#[must_use]
pub fn root_agent_role(_framework: &Framework, _agent_id: &str) -> String {
    // M08.5.C.fix red phase — `unimplemented!()` panics deliberately so
    // every unit test that calls this resolver fails for the right
    // reason (gotcha #66). The green-phase impl removes the underscores
    // and walks `framework.agents[]`.
    unimplemented!(
        "M08.5.C.fix red phase — green phase derives root agent_name from framework.agents[]"
    )
}

/// All declared inline agents in spawn order.
///
/// Used by the M06.A L2a wire-up at the SDK to walk children of
/// `parent_id` and emit `AgentSpawned` per child after running
/// `narrow`. Returns references into `framework.agents[]` so the
/// caller can read each agent's `Capabilities` block to construct
/// proposed grants.
#[must_use]
pub fn inline_agents(framework: &Framework) -> Vec<&Agent> {
    framework
        .agents
        .iter()
        .filter_map(|item| match item {
            FrameworkAgentsItem::Agent(a) => Some(a),
            FrameworkAgentsItem::Object { .. } => None,
        })
        .collect()
}

fn resource_or_placeholder(value: &str) -> ResourceName {
    ResourceName::from_str(value).unwrap_or_else(|_| {
        ResourceName::from_str("invalid_resource_name").expect("constant non-empty")
    })
}

fn any_glob() -> GlobPattern {
    GlobPattern::from_str(ANY_GLOB).expect("constant non-empty glob")
}

/// Type alias used by the M06.A SDK wire-up — `Arc<Framework>` is the
/// shape `AgentSdk::with_capability_wiring` consumes so the wire-up
/// does not clone the full framework per emission.
pub type FrameworkRef = Arc<Framework>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[allow(
        clippy::too_many_arguments,
        reason = "test fixture parameters mirror Capabilities + framework.tools fields one-to-one; bundling into a struct adds boilerplate without clarifying call sites"
    )]
    fn fw_with_agent(
        agent_id: &str,
        tools_called: &[&str],
        skills_loaded: &[&str],
        read: &[&str],
        write: &[&str],
        network: &[&str],
        shell: bool,
        spawn_agents: &[&str],
        framework_tools: &[(&str, &str)],
    ) -> Framework {
        let tool_items: Vec<serde_json::Value> = framework_tools
            .iter()
            .map(|(n, src)| json!({ "name": n, "source": src }))
            .collect();
        serde_json::from_value(json!({
            "name": "test",
            "version": "1.0.0",
            "description": "test framework",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "agents": [{
                "id": agent_id,
                "role": "worker",
                "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
                "capabilities": {
                    "tools_called": tools_called,
                    "skills_loaded": skills_loaded,
                    "file_access": { "read": read, "write": write },
                    "network": network,
                    "shell": shell,
                    "spawn_agents": spawn_agents,
                },
                "allowed_tools": tools_called,
                "allowed_skills": skills_loaded,
                "spawns": spawn_agents,
            }],
            "tools": tool_items,
            "skills": [],
            "session_root_agent": agent_id,
        }))
        .expect("test framework round-trips")
    }

    #[test]
    fn capabilities_for_tool_returns_exec_pure_for_builtin() {
        let fw = fw_with_agent(
            "worker",
            &["Read"],
            &[],
            &[],
            &[],
            &[],
            false,
            &[],
            &[("Read", "builtin")],
        );
        let needed = capabilities_for_tool(&fw, "Read").expect("declared tool resolves");
        assert_eq!(needed.len(), 1);
        assert_eq!(needed[0].kind, CapabilityKind::Exec);
        assert_eq!(*needed[0].resource, "Read");
        assert_eq!(needed[0].side_effect_class, SideEffectClass::Pure);
    }

    #[test]
    fn capabilities_for_tool_returns_exec_irreversible_for_external() {
        let fw = fw_with_agent(
            "worker",
            &["external_tool"],
            &[],
            &[],
            &[],
            &[],
            false,
            &[],
            &[("external_tool", "external")],
        );
        let needed = capabilities_for_tool(&fw, "external_tool").expect("declared tool resolves");
        assert_eq!(needed[0].side_effect_class, SideEffectClass::Irreversible);
    }

    #[test]
    fn capabilities_for_tool_returns_err_for_unknown() {
        let fw = fw_with_agent(
            "worker",
            &[],
            &[],
            &[],
            &[],
            &[],
            false,
            &[],
            &[("Read", "builtin")],
        );
        let err = capabilities_for_tool(&fw, "Mystery").expect_err("unknown tool errs");
        match err {
            CapabilityLookupError::ToolNotFound { name } => {
                assert_eq!(name, "Mystery");
            }
        }
    }

    #[test]
    fn parent_grants_for_agent_returns_none_for_unknown() {
        let fw = fw_with_agent("worker", &[], &[], &[], &[], &[], false, &[], &[]);
        assert!(parent_grants_for_agent(&fw, "ghost").is_none());
    }

    #[test]
    fn parent_grants_for_agent_translates_full_capability_surface() {
        let fw = fw_with_agent(
            "worker",
            &["Read"],
            &["planning"],
            &["src/**"],
            &["target/**"],
            &[".example.com"],
            true,
            &["worker"],
            &[("Read", "builtin")],
        );
        let grants = parent_grants_for_agent(&fw, "worker").expect("agent declared");
        // 1 tool + 1 skill + 1 read + 1 write + 1 network + 1 shell + 1 spawn = 7
        assert_eq!(grants.len(), 7);
        // Spot-check kinds.
        let kinds: Vec<CapabilityKind> = grants.iter().map(|g| g.kind).collect();
        assert!(kinds.contains(&CapabilityKind::Read));
        assert!(kinds.contains(&CapabilityKind::Write));
        assert!(kinds.contains(&CapabilityKind::Network));
        assert!(kinds.contains(&CapabilityKind::Exec));
        assert!(kinds.contains(&CapabilityKind::ProcessSpawn));
    }

    #[test]
    fn capabilities_to_declarations_produces_consistent_shape_for_round_trip() {
        // Translating the SAME Capabilities block twice must produce
        // the same Vec — the L2a `narrow(parent, proposed)` evaluator's
        // `subsumes(grant, requested)` predicate depends on identical
        // (kind, resource, scope, class) tuples for parent ⊇ child to
        // hold when both sides translate from the same source.
        let fw = fw_with_agent(
            "worker",
            &["Read"],
            &[],
            &["src/**"],
            &[],
            &[],
            false,
            &[],
            &[("Read", "builtin")],
        );
        let g1 = parent_grants_for_agent(&fw, "worker").unwrap();
        let g2 = parent_grants_for_agent(&fw, "worker").unwrap();
        assert_eq!(g1.len(), g2.len());
        for (a, b) in g1.iter().zip(g2.iter()) {
            assert_eq!(a.kind, b.kind);
            assert_eq!(*a.resource, *b.resource);
            assert_eq!(a.side_effect_class, b.side_effect_class);
        }
    }

    #[test]
    fn declaration_to_narrowed_from_str_round_trips_useful_substring() {
        let fw = fw_with_agent(
            "worker",
            &["Read"],
            &[],
            &["src/**"],
            &[],
            &[],
            false,
            &[],
            &[("Read", "builtin")],
        );
        let grants = parent_grants_for_agent(&fw, "worker").unwrap();
        let descriptions: Vec<String> = grants
            .iter()
            .map(declaration_to_narrowed_from_str)
            .collect();
        // Each description carries kind, resource, scope-variant.
        assert!(descriptions.iter().any(|d| d.contains("read")));
        assert!(descriptions.iter().any(|d| d.contains("src/**")));
    }

    #[test]
    fn inline_agents_skips_object_form() {
        let fw: Framework = serde_json::from_value(json!({
            "name": "test",
            "version": "1.0.0",
            "description": "test framework",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "agents": [
                { "id": "external-ref", "path": "agents/external.md" },
                {
                    "id": "inline",
                    "role": "worker",
                    "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
                    "capabilities": {
                        "tools_called": [], "skills_loaded": [],
                        "file_access": { "read": [], "write": [] },
                        "network": [], "shell": false, "spawn_agents": []
                    },
                    "allowed_tools": [], "allowed_skills": [], "spawns": []
                }
            ],
            "tools": [],
            "skills": [],
            "session_root_agent": "inline",
        }))
        .unwrap();
        let agents = inline_agents(&fw);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id.as_str(), "inline");
    }

    // ── root_agent_role (M08.5.C.fix 🔴-2) ───────────────────────────

    /// Build a framework with one inline `Agent` carrying a distinct
    /// `id` + `role`. The Tester uses this shape: the candidate
    /// framework's root agent is declared inline, and its `role` is the
    /// display name `session_prelude` must emit.
    fn fw_one_inline_agent(id: &str, role: &str) -> Framework {
        serde_json::from_value(json!({
            "name": "m08-5-c-fix-fixture",
            "version": "1.0.0",
            "description": "M08.5.C.fix root_agent_role fixture",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "agents": [{
                "id": id,
                "role": role,
                "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
                "capabilities": {
                    "tools_called": [], "skills_loaded": [],
                    "file_access": { "read": [], "write": [] },
                    "network": [], "shell": false, "spawn_agents": []
                },
                "allowed_tools": [], "allowed_skills": [], "spawns": []
            }],
            "tools": [],
            "skills": [],
            "session_root_agent": id,
        }))
        .expect("inline-agent fixture round-trips")
    }

    #[test]
    fn root_agent_role_returns_role_for_inline_agent() {
        // The candidate framework's inline root agent declares both an
        // `id` and a distinct `role`. The resolver MUST return the
        // `role` — that is the contract the smoke-vs-candidate root
        // labelling enforces (M08 🔴-2). A resolver that returned
        // `id` here would still produce a non-"smoke" label and SEEM
        // correct, but would not match the sub-agent naming pattern
        // (`spawn_framework_subagents` uses `role`, not `id`).
        let fw = fw_one_inline_agent("alpha", "lead");
        assert_eq!(
            root_agent_role(&fw, "alpha"),
            "lead",
            "inline agent: resolver returns the `role`, not the `id`"
        );
    }

    #[test]
    fn root_agent_role_falls_back_to_id_for_path_ref_agent() {
        // The `{ id, path }` form has no inline `role` (resolving the
        // referenced .md is M08.6's loader work, ADR-0022). The
        // resolver MUST fall back to the `id` — the common production
        // case for the archetype frameworks (examples/aria,
        // examples/ralph) whose agents are all path-refs.
        let fw: Framework = serde_json::from_value(json!({
            "name": "m08-5-c-fix-path-ref-fixture",
            "version": "1.0.0",
            "description": "M08.5.C.fix path-ref fallback fixture",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "agents": [
                { "id": "alpha", "path": "agents/alpha.md" }
            ],
            "tools": [],
            "skills": [],
            "session_root_agent": "alpha",
        }))
        .expect("path-ref fixture round-trips");
        assert_eq!(
            root_agent_role(&fw, "alpha"),
            "alpha",
            "path-ref form has no inline `role`; resolver falls back to the id"
        );
    }

    #[test]
    fn root_agent_role_falls_back_to_id_when_not_found() {
        // A `session_root_agent` not declared in `framework.agents[]`
        // is technically a schema-invalid framework, but the resolver
        // must not panic — falling back to the id is the same shape
        // as the path-ref case and produces a stable display name.
        let fw = fw_one_inline_agent("alpha", "lead");
        assert_eq!(
            root_agent_role(&fw, "ghost"),
            "ghost",
            "unknown id falls back to the literal id (never panics; never returns empty)"
        );
    }

    #[test]
    fn root_agent_role_picks_the_matching_agent_when_multiple_inline_agents_declared() {
        // A framework declares two inline agents; the resolver must
        // pick the one whose `id` matches, not the first listed.
        // Guards against an off-by-one walker bug.
        let fw: Framework = serde_json::from_value(json!({
            "name": "m08-5-c-fix-multi-fixture",
            "version": "1.0.0",
            "description": "M08.5.C.fix multi-agent fixture",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "agents": [
                {
                    "id": "alpha", "role": "alpha-lead",
                    "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
                    "capabilities": {
                        "tools_called": [], "skills_loaded": [],
                        "file_access": { "read": [], "write": [] },
                        "network": [], "shell": false, "spawn_agents": []
                    },
                    "allowed_tools": [], "allowed_skills": [], "spawns": []
                },
                {
                    "id": "beta", "role": "beta-worker",
                    "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
                    "capabilities": {
                        "tools_called": [], "skills_loaded": [],
                        "file_access": { "read": [], "write": [] },
                        "network": [], "shell": false, "spawn_agents": []
                    },
                    "allowed_tools": [], "allowed_skills": [], "spawns": []
                }
            ],
            "tools": [],
            "skills": [],
            "session_root_agent": "beta",
        }))
        .expect("multi-agent fixture round-trips");
        assert_eq!(
            root_agent_role(&fw, "beta"),
            "beta-worker",
            "resolver matches by id, not by position"
        );
    }

    #[test]
    fn capabilities_for_tool_twice_in_sequence_both_succeed() {
        // Gotcha #69: stateful primitives (the framework reference is
        // arc-clone-shared across the SDK's run loop) need multi-call
        // invariant tests. Two sequential lookups against the same
        // framework must each return the same grant set.
        let fw = fw_with_agent(
            "worker",
            &["Read"],
            &[],
            &[],
            &[],
            &[],
            false,
            &[],
            &[("Read", "builtin")],
        );
        let first = capabilities_for_tool(&fw, "Read").expect("first");
        let second = capabilities_for_tool(&fw, "Read").expect("second");
        assert_eq!(first.len(), second.len());
        assert_eq!(first[0].kind, second[0].kind);
    }
}
