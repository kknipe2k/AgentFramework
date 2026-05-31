//! In-process `LoadSkill` handler (M08.7 rung 3; ADR-0027).
//!
//! Skills are context-loaded instruction sets, loaded via the
//! runtime-injected `LoadSkill` tool (spec §0b). This module is the pure,
//! capability-gated handler the SDK run loop
//! ([`crate::sdk::AgentSdk`]) calls when a `LoadSkill` `ToolUse` arrives:
//! it checks the requested skill is in the agent's `allowed_skills` (the
//! capability gate — analogous to the executor's `file_access` check) and
//! returns the **already-resolved** skill body (resolved once at load per
//! ADR-0022 — this module does NOT re-read skill files).
//!
//! Injection model (ADR-0027): the run loop emits `SkillLoaded` and feeds
//! the body back as the `LoadSkill` tool's `tool_result`, which persists
//! in the agent's message history across subsequent turns (the rung-1
//! built-in feedback contract). The grounded acceptance is behavioral —
//! a loaded skill changes the agent's observable replies — not the
//! `SkillLoaded` event alone (CLAUDE.md §4 rule 11 / gotcha #66).

use std::collections::BTreeMap;

use serde_json::json;

use crate::providers::ToolDef;

/// The runtime-injected `LoadSkill` tool name (spec §0b — auto-advertised
/// to an agent that has skills to load).
pub const LOAD_SKILL_TOOL: &str = "LoadSkill";

/// A resolved skill ready to inject into the agent's context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSkill {
    /// The skill's name (matches the `LoadSkill` `skill_name` argument).
    pub name: String,
    /// The resolved skill markdown body (frontmatter + content), injected
    /// verbatim per ADR-0027 (no `mode_variants` filtering in v0.1 —
    /// STANDARD mode is hardcoded, §0d).
    pub body: String,
}

/// Failure loading a skill.
#[derive(Debug)]
pub enum LoadSkillError {
    /// The requested skill is not in the agent's `allowed_skills` (the
    /// capability gate). The skill is NOT loaded. Carries the skill name.
    NotAllowed(String),
    /// The skill IS allowed but no resolved body was threaded into the
    /// run path (a malformed framework / missing companion). The skill is
    /// NOT loaded. Carries the skill name.
    NotResolved(String),
}

/// Load a skill body for injection into the agent's context.
///
/// Checks `skill_name` is in `allowed_skills` (the capability gate), then
/// looks up the **already-resolved** body from `resolved_skills` (ADR-0022
/// resolves skill companions to bodies once, at load; this is a lookup,
/// not a re-resolution).
///
/// # Errors
///
/// - [`LoadSkillError::NotAllowed`] when `skill_name` is not in
///   `allowed_skills` — even if a resolved body exists, an ungranted
///   skill never loads.
/// - [`LoadSkillError::NotResolved`] when the skill is allowed but no
///   resolved body is present.
pub fn load_skill(
    skill_name: &str,
    allowed_skills: &[String],
    resolved_skills: &BTreeMap<String, String>,
) -> Result<LoadedSkill, LoadSkillError> {
    if !allowed_skills.iter().any(|s| s == skill_name) {
        return Err(LoadSkillError::NotAllowed(skill_name.to_string()));
    }
    let body = resolved_skills
        .get(skill_name)
        .ok_or_else(|| LoadSkillError::NotResolved(skill_name.to_string()))?;
    Ok(LoadedSkill {
        name: skill_name.to_string(),
        body: body.clone(),
    })
}

/// The `ToolDef` advertised to the model for `LoadSkill` (spec §0b input
/// shape `{skill_name, reason}`).
#[must_use]
pub fn load_skill_tool_def() -> ToolDef {
    ToolDef {
        name: LOAD_SKILL_TOOL.to_string(),
        description: "Load instructional context for a named skill before performing related work."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "skill_name": {
                    "type": "string",
                    "description": "Name of the skill to load (from the agent's allowed skills)."
                },
                "reason": {
                    "type": "string",
                    "description": "Why you're loading this skill now."
                }
            },
            "required": ["skill_name", "reason"]
        }),
    }
}
