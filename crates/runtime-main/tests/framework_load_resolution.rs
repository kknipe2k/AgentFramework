//! M08.6 Stage B — `load_framework` reference resolution (ADR-0022).
//!
//! Discriminator (gotcha #66): every assembled regression loads the
//! REAL `examples/aria/` + `examples/ralph/` archetypes, not an inline
//! fixture. The M08 Builder shipped green because the archetype was
//! never tested; this file closes that.

use std::path::PathBuf;

use runtime_core::generated::framework::FrameworkAgentsItem;
use runtime_core::generated::skill::Skill;
use runtime_core::generated::tool::Tool;
use runtime_main::builder::persist::split_frontmatter;
use runtime_main::builder::{load_framework, BuilderError};

/// `crates/runtime-main/` → workspace root.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has a parent (crates/)")
        .parent()
        .expect("crates/ has a parent (workspace root)")
        .to_path_buf()
}

/// Normalize CRLF → LF so frontmatter splits work on Windows-checked-out
/// archetypes. The splitter operates on LF-only `&str` (caller-normalize
/// contract; mirrors the `runtime-core/tests/round_trip.rs:139`
/// precedent).
fn lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

// ── The agents[] oneOf flip — `Object {id,path}` → `Agent(_)` ───────

#[test]
fn load_framework_resolves_every_aria_agent_to_inline() {
    let aria = workspace_root().join("examples/aria");
    let loaded = load_framework(&aria).expect("examples/aria/ loads");
    assert_eq!(
        loaded.framework.agents.len(),
        8,
        "examples/aria/framework.json declares 8 agents",
    );
    for (i, item) in loaded.framework.agents.iter().enumerate() {
        assert!(
            matches!(item, FrameworkAgentsItem::Agent(_)),
            "agents[{i}] is still {item:?} — load_framework did not resolve the {{id,path}} reference",
        );
    }
}

#[test]
fn load_framework_resolves_aria_orchestrator_role_and_allowed_tools() {
    let aria = workspace_root().join("examples/aria");
    let loaded = load_framework(&aria).expect("examples/aria/ loads");
    let orchestrator = loaded
        .framework
        .agents
        .iter()
        .find_map(|item| match item {
            FrameworkAgentsItem::Agent(a) if a.id.as_str() == "orchestrator" => Some(a),
            _ => None,
        })
        .expect("orchestrator agent resolved to inline form");
    assert!(
        orchestrator.role.starts_with("Session root agent"),
        "orchestrator.role from agents/orchestrator.md frontmatter: {}",
        orchestrator.role.as_str(),
    );
    assert!(
        orchestrator.allowed_tools.iter().any(|t| t == "LoadSkill"),
        "orchestrator.allowed_tools contains LoadSkill from frontmatter: {:?}",
        orchestrator.allowed_tools,
    );
}

// ── Tools + skills: surface the referenced .md bodies as companions ─
//
// `FrameworkToolsItem` and `FrameworkSkillsItem` are flat structs
// (`crates/runtime-core/src/generated/framework.rs:2786,3029`) — NOT a
// `oneOf`, unlike `FrameworkAgentsItem` (line 1375). Only agents have
// an inline-vs-reference variant; tools/skills are always references
// in the framework struct. Resolution for tools/skills reads the
// referenced `.md` files and surfaces them as `companions`, so the
// canvas projection + Stage C re-split see the body content.

#[test]
fn load_framework_surfaces_aria_tool_companions() {
    let aria = workspace_root().join("examples/aria");
    let loaded = load_framework(&aria).expect("examples/aria/ loads");
    let git_checkpoint = loaded
        .companions
        .iter()
        .find(|c| c.file_name == "tools/git_checkpoint.md")
        .expect("tools/git_checkpoint.md surfaced as a companion");
    let body = lf(&git_checkpoint.body);
    let (frontmatter, _body) =
        split_frontmatter(&body).expect("git_checkpoint.md has YAML frontmatter");
    let _: Tool = serde_yaml::from_str(frontmatter)
        .expect("git_checkpoint.md frontmatter parses into runtime-core Tool");
}

#[test]
fn load_framework_surfaces_aria_skill_companions() {
    let aria = workspace_root().join("examples/aria");
    let loaded = load_framework(&aria).expect("examples/aria/ loads");
    let planning = loaded
        .companions
        .iter()
        .find(|c| c.file_name == "skills/planning.md")
        .expect("skills/planning.md surfaced as a companion");
    let body = lf(&planning.body);
    let (frontmatter, _body) = split_frontmatter(&body).expect("planning.md has YAML frontmatter");
    let skill: Skill = serde_yaml::from_str(frontmatter)
        .expect("planning.md frontmatter parses into runtime-core Skill");
    assert_eq!(skill.name.as_str(), "planning");
}

#[test]
fn load_framework_surfaces_all_aria_agent_companions() {
    // The agent .md bodies are surfaced too (they carry the system-
    // prompt body per agent.v1.json; Stage B captures, M09 applies).
    let aria = workspace_root().join("examples/aria");
    let loaded = load_framework(&aria).expect("examples/aria/ loads");
    for name in [
        "agents/orchestrator.md",
        "agents/router.md",
        "agents/planner.md",
        "agents/analyzer.md",
        "agents/implementer.md",
        "agents/verify-app.md",
        "agents/simplifier.md",
        "agents/report-writer.md",
    ] {
        assert!(
            loaded.companions.iter().any(|c| c.file_name == name),
            "{name} surfaced as a companion (all 8 agent bodies captured)",
        );
    }
}

// ── Cross-framework `../` references (Ralph) ────────────────────────

#[test]
fn load_framework_resolves_ralph_cross_framework_refs() {
    let ralph = workspace_root().join("examples/ralph");
    let loaded = load_framework(&ralph).expect("examples/ralph/ loads");
    assert_eq!(loaded.framework.agents.len(), 1, "ralph has one agent");
    assert!(
        matches!(&loaded.framework.agents[0], FrameworkAgentsItem::Agent(_)),
        "ralph-agent resolved to inline form",
    );
    let aria_verify = loaded
        .companions
        .iter()
        .find(|c| c.file_name == "../aria/tools/aria_verify.md")
        .expect("cross-framework ../aria/tools/aria_verify.md surfaces as a companion");
    assert!(
        !aria_verify.body.is_empty(),
        "the cross-framework body was actually read",
    );
}

// ── Broken reference + gap-tolerance ────────────────────────────────

#[test]
fn load_framework_broken_agent_reference_is_a_builder_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let broken = serde_json::json!({
        "$schema": "https://schemas.aria-runtime.dev/framework/v1.json",
        "name": "broken-ref",
        "version": "1.0.0",
        "description": "A framework whose agents[] references a missing .md — load_framework must surface this as a BuilderError, not silently drop the agent and not panic.",
        "author": "test",
        "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
        "tools": [],
        "skills": [],
        "agents": [
            { "id": "ghost", "path": "agents/ghost.md" }
        ],
        "session_root_agent": "ghost"
    });
    std::fs::write(
        dir.path().join("framework.json"),
        serde_json::to_string_pretty(&broken).unwrap(),
    )
    .unwrap();
    let err = load_framework(dir.path())
        .expect_err("missing referenced .md must error, not silently drop");
    assert!(
        matches!(err, BuilderError::ReferenceResolution { .. }),
        "expected BuilderError::ReferenceResolution; got {err:?}",
    );
}

#[test]
fn load_framework_inline_only_framework_still_loads() {
    // The existing gap-tolerant posture is preserved for an inline-only
    // framework (no {id,path} agent refs, no path-referenced tools or
    // skills): load_framework returns it unchanged without touching the
    // filesystem beyond framework.json itself. Regression guard against
    // B's resolver breaking the M08-Builder inline write path.
    let dir = tempfile::tempdir().expect("temp dir");
    let inline = serde_json::json!({
        "$schema": "https://schemas.aria-runtime.dev/framework/v1.json",
        "name": "inline-only",
        "version": "1.0.0",
        "description": "Inline-only framework: agents[] is one inline Agent; no path references anywhere. Stage B's resolver must leave it untouched.",
        "author": "test",
        "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
        "tools": [],
        "skills": [],
        "agents": [{
            "id": "solo",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "allowed_tools": [],
            "allowed_skills": [],
            "spawns": [],
            "capabilities": {
                "tools_called": [],
                "skills_loaded": [],
                "file_access": { "read": [], "write": [] },
                "network": [],
                "shell": false,
                "spawn_agents": []
            }
        }],
        "session_root_agent": "solo"
    });
    std::fs::write(
        dir.path().join("framework.json"),
        serde_json::to_string_pretty(&inline).unwrap(),
    )
    .unwrap();
    let loaded = load_framework(dir.path()).expect("inline-only framework loads");
    assert_eq!(loaded.framework.agents.len(), 1);
    assert!(
        matches!(&loaded.framework.agents[0], FrameworkAgentsItem::Agent(_)),
        "the inline agent stays inline (already in the Agent variant)",
    );
}

// ── The frontmatter splitter ────────────────────────────────────────

#[test]
fn split_frontmatter_extracts_yaml_block_between_dashes() {
    let text = "---\nname: x\nversion: 1\n---\n# body\nmore body\n";
    let (frontmatter, body) = split_frontmatter(text).expect("well-formed --- … --- block splits");
    assert_eq!(frontmatter, "name: x\nversion: 1\n");
    assert_eq!(body, "# body\nmore body\n");
}

#[test]
fn split_frontmatter_returns_none_when_no_dashes() {
    assert!(
        split_frontmatter("# just a body, no dashes").is_none(),
        "input with no leading --- yields None",
    );
}

#[test]
fn split_frontmatter_returns_none_when_no_closing_dashes() {
    assert!(
        split_frontmatter("---\nname: x\nno close\n").is_none(),
        "input with no closing --- yields None",
    );
}
