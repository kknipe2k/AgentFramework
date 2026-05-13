//! Pure-function walker over a parsed `Framework`. Spec §4b Layer 1.
//!
//! Walks every inline `Agent` in `framework.agents[]` and checks each
//! reference against the framework's declared primitive sets:
//!
//! - `agent.allowed_tools[]` ⊆ `framework.tools[].name`     → `tool_missing`
//! - `agent.allowed_skills[]` ⊆ `framework.skills[].name`   → `skill_missing`
//! - `agent.spawns[]` ⊆ `framework.agents[].id`             → `agent_missing`
//!
//! `mcp_missing` is NOT emitted at Layer 1 in v0.1: the v0.1 `framework.v1.json`
//! schema has no MCP-server declaration field (MCP server lifecycle lands at
//! M06 per `docs/MVP-v0.1.md`). The variant is reserved for Layer 2
//! (`request_capability`) emission this milestone and Layer 1 emission once
//! M06 adds the framework declaration field.
//!
//! The walker is pure — it does NOT emit events; it returns a `Vec<Gap>` the
//! caller routes through an emitter. Multiple gaps in one framework all
//! surface (no short-circuit), so the renderer can paint the full gap set
//! at once.

use runtime_core::event::{AgentEvent, GapSeverityRef, GapSourceRef};
use runtime_core::generated::framework::{Framework, FrameworkAgentsItem};

/// Discriminator over the four gap kinds. Mirrors the four `*_missing`
/// event variants in `schemas/event.v1.json`; `to_event` below produces
/// the wire-form event from this discriminator + the gap payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapKind {
    /// `allowed_tools[]` reference unresolved.
    Tool,
    /// `allowed_skills[]` reference unresolved.
    Skill,
    /// MCP server reference unresolved. Layer 2 (`request_capability`) only
    /// in v0.1; Layer 1 emission lands with M06's MCP framework declaration.
    Mcp,
    /// `spawns[]` reference unresolved.
    Agent,
}

/// One unresolved-reference finding produced by [`walk`]. Carries enough
/// context for the emitter to construct the appropriate `*_missing`
/// [`AgentEvent`] variant with no further lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    /// Which primitive is missing.
    pub kind: GapKind,
    /// Agent that referenced the missing primitive. For Layer 1 this is
    /// the agent declaring the unresolved `allowed_*` or `spawns[]` entry.
    pub agent_id: String,
    /// Name (or id, for `Agent` kind) of the missing primitive.
    pub missing_name: String,
    /// Severity per spec §4b severity matrix.
    pub severity: GapSeverityRef,
    /// Plain-English next-step text the renderer surfaces. Composed by
    /// the walker so the emitter has a non-empty default; HITL prompt
    /// copy may override.
    pub suggested_action: String,
}

impl Gap {
    /// Render this gap to the canonical [`AgentEvent`] union for emission.
    /// `source` distinguishes loader-driven (Layer 1) from
    /// `request_capability`-driven (Layer 2) gaps.
    #[must_use]
    pub fn to_event(&self, source: GapSourceRef) -> AgentEvent {
        match self.kind {
            GapKind::Tool => AgentEvent::ToolMissing {
                agent_id: self.agent_id.clone(),
                tool_name: self.missing_name.clone(),
                severity: self.severity,
                suggested_action: self.suggested_action.clone(),
                requested_via: source,
            },
            GapKind::Skill => AgentEvent::SkillMissing {
                agent_id: self.agent_id.clone(),
                skill_name: self.missing_name.clone(),
                severity: self.severity,
                suggested_action: self.suggested_action.clone(),
                requested_via: source,
            },
            GapKind::Mcp => AgentEvent::McpMissing {
                agent_id: self.agent_id.clone(),
                server_name: self.missing_name.clone(),
                severity: self.severity,
                suggested_action: self.suggested_action.clone(),
                requested_via: source,
            },
            GapKind::Agent => AgentEvent::AgentMissing {
                agent_id: self.agent_id.clone(),
                missing_agent_id: self.missing_name.clone(),
                severity: self.severity,
                suggested_action: self.suggested_action.clone(),
                requested_via: source,
            },
        }
    }
}

/// Walk a parsed framework and collect every Layer-1 gap. Returns an empty
/// vec when every reference resolves.
#[must_use]
pub fn walk(framework: &Framework) -> Vec<Gap> {
    let declared_tools: Vec<&str> = framework.tools.iter().map(|t| t.name.as_str()).collect();
    let declared_skills: Vec<&str> = framework.skills.iter().map(|s| s.name.as_str()).collect();
    let declared_agents: Vec<&str> = framework.agents.iter().map(agents_item_id).collect();

    let mut gaps = Vec::new();

    for item in &framework.agents {
        let FrameworkAgentsItem::Agent(agent) = item else {
            // `Object { id, path }` declarations live in `framework.agents`
            // but their references (allowed_*, spawns[]) are walked only
            // when the agent.md is loaded — that's M07 registry-import
            // territory. Skip at v0.1.
            continue;
        };
        let agent_id: &str = &agent.id;

        for tool in &agent.allowed_tools {
            if !declared_tools.contains(&tool.as_str()) {
                gaps.push(Gap {
                    kind: GapKind::Tool,
                    agent_id: agent_id.to_string(),
                    missing_name: tool.clone(),
                    severity: GapSeverityRef::Critical,
                    suggested_action: format!(
                        "Install tool '{tool}' and click Resume; agent '{agent_id}' cannot start until it resolves.",
                    ),
                });
            }
        }

        for skill in &agent.allowed_skills {
            if !declared_skills.contains(&skill.as_str()) {
                gaps.push(Gap {
                    kind: GapKind::Skill,
                    agent_id: agent_id.to_string(),
                    missing_name: skill.clone(),
                    severity: GapSeverityRef::Advisory,
                    suggested_action: format!(
                        "Skill '{skill}' not installed; agent '{agent_id}' continues without it — install async or dismiss.",
                    ),
                });
            }
        }

        for spawn in &agent.spawns {
            if !declared_agents.contains(&spawn.as_str()) {
                gaps.push(Gap {
                    kind: GapKind::Agent,
                    agent_id: agent_id.to_string(),
                    missing_name: spawn.clone(),
                    severity: GapSeverityRef::Critical,
                    suggested_action: format!(
                        "Sub-agent '{spawn}' not declared in framework; fix framework JSON before reloading.",
                    ),
                });
            }
        }
    }

    gaps
}

/// Extract the agent id from a `FrameworkAgentsItem` regardless of whether
/// it's an inline `Agent` or an `Object { id, path }` declaration.
fn agents_item_id(item: &FrameworkAgentsItem) -> &str {
    match item {
        FrameworkAgentsItem::Object { id, .. } => id.as_str(),
        FrameworkAgentsItem::Agent(a) => &a.id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal-valid Framework from a JSON value. Tests use
    /// `serde_json::from_value` to dodge the typify-generated builders
    /// and private newtype fields. Mirrors `examples/aria/framework.json`
    /// shape — every required field per `schemas/framework.v1.json`.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "json! macro consumes the value into the outer JSON map; clippy can't see through the macro"
    )]
    fn fw_from_agents(tools: &[&str], skills: &[&str], agents: serde_json::Value) -> Framework {
        let tool_items: Vec<serde_json::Value> = tools
            .iter()
            .map(|n| serde_json::json!({ "name": n, "source": "builtin" }))
            .collect();
        let skill_items: Vec<serde_json::Value> = skills
            .iter()
            .map(|n| serde_json::json!({ "name": n, "source": "local" }))
            .collect();

        // Required: session_root_agent must reference one of `agents[]`.
        let root = agents
            .as_array()
            .and_then(|a| a.first())
            .and_then(|first| first.get("id").or_else(|| first.get("id")))
            .and_then(|v| v.as_str())
            .unwrap_or("root")
            .to_string();

        serde_json::from_value(serde_json::json!({
            "name": "test",
            "version": "1.0.0",
            "description": "test framework",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "agents": agents,
            "tools": tool_items,
            "skills": skill_items,
            "session_root_agent": root,
        }))
        .expect("test framework round-trips")
    }

    fn inline_agent(
        id: &str,
        allowed_tools: &[&str],
        allowed_skills: &[&str],
        spawns: &[&str],
    ) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "capabilities": {
                "file_access": { "read": [], "write": [] },
                "network": [],
                "shell": false,
                "skills_loaded": [],
                "spawn_agents": [],
                "tools_called": []
            },
            "allowed_tools": allowed_tools,
            "allowed_skills": allowed_skills,
            "spawns": spawns
        })
    }

    #[test]
    fn walks_valid_framework_emits_zero_gaps() {
        let fw = fw_from_agents(
            &["Read", "Write"],
            &["planning"],
            serde_json::json!([inline_agent(
                "worker",
                &["Read", "Write"],
                &["planning"],
                &[]
            )]),
        );
        assert_eq!(walk(&fw), vec![]);
    }

    #[test]
    fn unresolved_tool_reference_emits_tool_missing() {
        let fw = fw_from_agents(
            &["Read"],
            &[],
            serde_json::json!([inline_agent("worker", &["Read", "MissingTool"], &[], &[])]),
        );
        let gaps = walk(&fw);
        assert_eq!(gaps.len(), 1, "exactly one gap for one unresolved tool");
        assert_eq!(gaps[0].kind, GapKind::Tool);
        assert_eq!(gaps[0].agent_id, "worker");
        assert_eq!(gaps[0].missing_name, "MissingTool");
        assert_eq!(gaps[0].severity, GapSeverityRef::Critical);
        assert!(
            !gaps[0].suggested_action.is_empty(),
            "suggested_action must be non-empty (minLength: 1)",
        );
    }

    #[test]
    fn unresolved_skill_reference_emits_skill_missing() {
        let fw = fw_from_agents(
            &[],
            &[],
            serde_json::json!([inline_agent("worker", &[], &["MissingSkill"], &[])]),
        );
        let gaps = walk(&fw);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].kind, GapKind::Skill);
        assert_eq!(gaps[0].missing_name, "MissingSkill");
        assert_eq!(
            gaps[0].severity,
            GapSeverityRef::Advisory,
            "skill_missing is advisory per spec §4b severity matrix",
        );
    }

    #[test]
    fn unresolved_subagent_reference_emits_agent_missing() {
        let fw = fw_from_agents(
            &[],
            &[],
            serde_json::json!([inline_agent(
                "orchestrator",
                &[],
                &[],
                &["nonexistent-child"]
            )]),
        );
        let gaps = walk(&fw);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].kind, GapKind::Agent);
        assert_eq!(gaps[0].agent_id, "orchestrator");
        assert_eq!(gaps[0].missing_name, "nonexistent-child");
        assert_eq!(
            gaps[0].severity,
            GapSeverityRef::Critical,
            "agent_missing blocks load per spec §4b severity matrix",
        );
    }

    #[test]
    fn multiple_gaps_in_one_walk_all_surface() {
        // Walker MUST NOT short-circuit on first gap — the renderer
        // surfaces all gaps at once so the user fixes them in one round.
        let fw = fw_from_agents(
            &[],
            &[],
            serde_json::json!([inline_agent(
                "worker",
                &["MissingTool1", "MissingTool2"],
                &["MissingSkill"],
                &["missing-agent"]
            )]),
        );
        let gaps = walk(&fw);
        assert_eq!(gaps.len(), 4, "all four unresolved refs surface");

        let kinds: Vec<GapKind> = gaps.iter().map(|g| g.kind).collect();
        assert!(kinds.contains(&GapKind::Tool));
        assert!(kinds.contains(&GapKind::Skill));
        assert!(kinds.contains(&GapKind::Agent));
    }

    #[test]
    fn severity_critical_for_tool_and_agent_advisory_for_skill() {
        // Spec §4b severity matrix: tool / agent gaps block; skill gaps
        // continue with warning. The walker assigns severity per kind.
        let fw = fw_from_agents(
            &[],
            &[],
            serde_json::json!([inline_agent(
                "worker",
                &["MissingTool"],
                &["MissingSkill"],
                &["missing-agent"]
            )]),
        );
        let gaps = walk(&fw);

        for g in &gaps {
            match g.kind {
                GapKind::Tool | GapKind::Agent => {
                    assert_eq!(g.severity, GapSeverityRef::Critical, "{:?}", g.kind);
                }
                GapKind::Skill => {
                    assert_eq!(g.severity, GapSeverityRef::Advisory);
                }
                GapKind::Mcp => unreachable!("walker does not emit Mcp at Layer 1 in v0.1"),
            }
        }
    }

    #[test]
    fn declared_subagent_via_object_form_still_resolves_spawns() {
        // `FrameworkAgentsItem::Object { id, path }` is the registry form
        // (agent.md on disk); the walker must consider its `id` declared
        // so a parent's `spawns: ["report-writer"]` doesn't false-positive.
        let fw = fw_from_agents(
            &[],
            &[],
            serde_json::json!([
                { "id": "report-writer", "path": "agents/report-writer.md" },
                inline_agent("orchestrator", &[], &[], &["report-writer"])
            ]),
        );
        assert_eq!(walk(&fw), vec![], "object-form agent counts as declared");
    }

    #[test]
    fn gap_to_event_round_trip_carries_source_discriminator() {
        // Layer 1 (loader) vs Layer 2 (request_capability) gaps share the
        // event shape; the discriminator is `requested_via`.
        let gap = Gap {
            kind: GapKind::Tool,
            agent_id: "worker".into(),
            missing_name: "fetch_prs".into(),
            severity: GapSeverityRef::Critical,
            suggested_action: "Install tool 'fetch_prs' and click Resume.".into(),
        };

        let loader_event = gap.to_event(GapSourceRef::Loader);
        let request_event = gap.to_event(GapSourceRef::RequestCapability);

        match loader_event {
            AgentEvent::ToolMissing {
                requested_via,
                tool_name,
                ..
            } => {
                assert_eq!(requested_via, GapSourceRef::Loader);
                assert_eq!(tool_name, "fetch_prs");
            }
            _ => panic!("expected ToolMissing"),
        }
        match request_event {
            AgentEvent::ToolMissing { requested_via, .. } => {
                assert_eq!(requested_via, GapSourceRef::RequestCapability);
            }
            _ => panic!("expected ToolMissing"),
        }
    }

    #[test]
    fn gap_to_event_covers_all_four_kinds() {
        let kinds = [
            (GapKind::Tool, GapSeverityRef::Critical),
            (GapKind::Skill, GapSeverityRef::Advisory),
            (GapKind::Mcp, GapSeverityRef::Important),
            (GapKind::Agent, GapSeverityRef::Critical),
        ];
        for (kind, severity) in kinds {
            let gap = Gap {
                kind,
                agent_id: "worker".into(),
                missing_name: "x".into(),
                severity,
                suggested_action: "fix it".into(),
            };
            let event = gap.to_event(GapSourceRef::RequestCapability);
            match (kind, &event) {
                (GapKind::Tool, AgentEvent::ToolMissing { .. })
                | (GapKind::Skill, AgentEvent::SkillMissing { .. })
                | (GapKind::Mcp, AgentEvent::McpMissing { .. })
                | (GapKind::Agent, AgentEvent::AgentMissing { .. }) => {}
                _ => panic!("kind {kind:?} did not map to matching event"),
            }
        }
    }
}
