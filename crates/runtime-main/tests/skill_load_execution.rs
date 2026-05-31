//! M08.7.C rung 3 — skill load (`LoadSkill`) assembled regression + unit
//! contract.
//!
//! The cluster-gate close contract (`docs/cluster-pattern.md` §1/§4): the
//! assembled tests drive the REAL `run_test_session_with_skills` →
//! `AgentSdk::run_agent` multi-turn loop. The ONLY stub is the provider
//! (no live Anthropic — CLAUDE.md §10); the `LoadSkill` handler, the
//! `drive_stream` branch, the `SkillLoaded` emission, and the multi-turn
//! message-history feedback are all real.
//!
//! Grounded-claims (CLAUDE.md §4 rule 11 / gotcha #66): a `SkillLoaded`
//! event firing licenses ONLY "the event fired" — NOT "the skill changed
//! behavior." The CI close gate here is STRUCTURAL: the skill body is
//! present in the turn-2 `AgentConfig` the loop re-sent (observed on the
//! real config, the rung-1 `latest_tool_result_text` pattern), proving
//! injection-into-context. The BEHAVIORAL assertion — a real model reads
//! the "reply in ALL CAPS" skill and replies in all caps — is the IRL
//! gate (real Anthropic, maintainer-watched), because the scripted stub
//! is NOT the model and cannot prove behavior change.
//!
//! Injection model: ADR-0027 (spec §0b) — the skill body rides back as
//! the `LoadSkill` tool's `tool_result`, which persists in
//! `config.messages` across all subsequent turns (`run_agent` re-sends
//! the accumulated history every turn). Composition is additive.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::{json, Value};
use tempfile::TempDir;

use runtime_core::event::AgentEvent;
use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with_skills;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{
    AgentConfig, ContentBlock, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError,
    ProviderEvent, ProviderSupport, ToolResultContent,
};
use runtime_main::sdk::load_skill::{load_skill, LoadSkillError, LoadedSkill};
use runtime_main::sdk::SessionId;

// ── helpers ───────────────────────────────────────────────────────────

/// A schema-valid one-agent framework whose `worker` agent declares the
/// given `allowed_skills` (and a matching `skills[]` reference entry per
/// name). `session_root_agent` is `worker`, so the run's dispatch agent
/// id is `worker`. `allowed_tools` is empty on purpose — `LoadSkill`
/// advertisement is driven by `allowed_skills` (spec §0b auto-inject),
/// not by `allowed_tools`.
fn fw_with_skills(allowed_skills: &[&str]) -> Framework {
    let skills: Vec<Value> = allowed_skills
        .iter()
        .map(|s| json!({ "name": s }))
        .collect();
    serde_json::from_value(json!({
        "name": "m08-7-c-rung3",
        "version": "1.0.0",
        "description": "M08.7.C rung 3 skill-load fixture",
        "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "capabilities": {
                "tools_called": [],
                "skills_loaded": [],
                "file_access": { "read": [], "write": [] },
                "network": [],
                "shell": false,
                "spawn_agents": []
            },
            "allowed_tools": [],
            "allowed_skills": allowed_skills,
            "spawns": []
        }],
        "tools": [],
        "skills": skills,
        "session_root_agent": "worker",
    }))
    .expect("the rung-3 fixture framework round-trips through the schema")
}

/// Build a resolved-skills map (skill name → already-resolved body) — the
/// shape rung 3 threads into the run path (the body was resolved once at
/// load per ADR-0022; this is a lookup map, NOT a re-resolution).
fn skills_map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

/// The latest `tool_result` text anywhere in a config's message history.
fn latest_tool_result_text(config: &AgentConfig) -> Option<String> {
    config.messages.iter().rev().find_map(|m| {
        m.content.iter().rev().find_map(|b| match b {
            ContentBlock::ToolResult {
                content: ToolResultContent::Text(t),
                ..
            } => Some(t.clone()),
            _ => None,
        })
    })
}

/// Every `tool_result` text in a config's message history, in order — for
/// the additive-composition assertion (multiple loaded skills all
/// present).
fn all_tool_result_texts(config: &AgentConfig) -> Vec<String> {
    config
        .messages
        .iter()
        .flat_map(|m| m.content.iter())
        .filter_map(|b| match b {
            ContentBlock::ToolResult {
                content: ToolResultContent::Text(t),
                ..
            } => Some(t.clone()),
            _ => None,
        })
        .collect()
}

// ── provider stub (no live Anthropic) ─────────────────────────────────

/// Emits one scripted `LoadSkill` `ToolUse` per leading turn (so a test
/// can drive one or several loads), then on the final turn ECHOES the
/// latest `tool_result` text it received back as stream text — a faithful
/// stand-in for a model that follows the skill it was handed. CAPTURES
/// every config it is given so a test can observe what the loop fed back.
struct SkillScriptStub {
    tool_calls: Vec<(String, Value)>,
    seen: Arc<Mutex<Vec<AgentConfig>>>,
    turn: Mutex<usize>,
}

impl SkillScriptStub {
    const fn new(tool_calls: Vec<(String, Value)>, seen: Arc<Mutex<Vec<AgentConfig>>>) -> Self {
        Self {
            tool_calls,
            seen,
            turn: Mutex::new(0),
        }
    }
}

#[async_trait]
impl LLMProvider for SkillScriptStub {
    fn name(&self) -> &'static str {
        "m08-7-c-skill-script-stub"
    }
    fn supports(&self) -> ProviderSupport {
        ProviderSupport {
            tool_use: true,
            streaming: true,
            thinking: false,
        }
    }
    async fn stream(
        &self,
        config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        self.seen.lock().expect("seen lock").push(config.clone());
        let n = {
            let mut t = self.turn.lock().expect("turn lock");
            let n = *t;
            *t += 1;
            n
        };
        if let Some((name, input)) = self.tool_calls.get(n) {
            return Ok(Box::pin(futures::stream::iter(vec![
                ProviderEvent::ToolUse {
                    id: format!("tu-{n}"),
                    name: name.clone(),
                    input: input.clone(),
                },
            ])));
        }
        let echoed = latest_tool_result_text(&config).unwrap_or_default();
        Ok(Box::pin(futures::stream::iter(vec![
            ProviderEvent::TextDelta {
                text: format!("following the skill: {echoed}"),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
                total_tokens: None,
            },
        ])))
    }
    async fn count_tokens(&self, _m: &[Message]) -> Result<u64, ProviderError> {
        Ok(0)
    }
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        Ok(Vec::new())
    }
    fn estimate_cost(&self, _b: &CostBreakdown, _m: &str) -> f64 {
        0.0
    }
}

// ── assembled regressions (the C.4.6 BDD close contract) ──────────────

/// C.4.1 — the close contract. Loading a skill injects its body into the
/// agent's context for the NEXT turn.
///
/// Given a skill "shout" whose body says "ALWAYS REPLY IN ALL CAPS" and a
/// one-agent framework with shout in `allowed_skills`, when the agent
/// calls `LoadSkill("shout")` then takes a normal turn, THEN a
/// `SkillLoaded(shout)` event is emitted AND the skill body is present in
/// the turn-2 `AgentConfig` the loop re-sent (the structural CI gate;
/// the all-caps *behavior* is the IRL gate). Also proves `LoadSkill` is
/// advertised in `config.tools` (driven by `allowed_skills`).
#[tokio::test]
async fn loading_a_skill_injects_its_body_into_the_next_turn_context() {
    let body = "ALWAYS REPLY IN ALL CAPS — NEVER USE LOWERCASE.";
    let fw = fw_with_skills(&["shout"]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = SkillScriptStub::new(
        vec![(
            "LoadSkill".to_string(),
            json!({ "skill_name": "shout", "reason": "the task needs shouting" }),
        )],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with_skills(
        &fw,
        "say hello, loading the shout skill first",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
        skills_map(&[("shout", body)]),
    )
    .await
    .expect("the assembled rung-3 run completes");

    // (a) a SkillLoaded(shout) event was emitted (the event fired).
    assert!(
        outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::SkillLoaded { skill_name, .. } if skill_name == "shout"
        )),
        "a SkillLoaded(shout) event must be emitted; trace={:?}",
        outcome.trace
    );

    let configs: Vec<AgentConfig> = seen.lock().expect("seen lock").clone();

    // LoadSkill is advertised in config.tools (driven by allowed_skills).
    assert!(
        configs[0].tools.iter().any(|t| t.name == "LoadSkill"),
        "LoadSkill must be advertised in the agent's tools; got {:?}",
        configs[0].tools.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // (b) LOAD-BEARING (rule 11 / gotcha #66): the skill body is present
    // in the turn-2 AgentConfig the loop re-sent — observed on the REAL
    // config the multi-turn loop built, not from the emitted event. The
    // behavioral all-caps assertion is the IRL gate (the stub is not the
    // model).
    assert!(
        configs.len() >= 2,
        "the loop must re-stream a 2nd turn after the skill loaded; got {} turn(s)",
        configs.len()
    );
    let injected = latest_tool_result_text(&configs[1]).unwrap_or_default();
    assert!(
        injected.contains("ALL CAPS"),
        "turn 2 must carry the skill body as a tool_result; got {injected:?}"
    );
}

/// C.4.4 (mutation-gate target) — the `SkillLoaded` event payload is keyed
/// correctly: `agent_id` is the session-root agent and `skill_name` is the
/// loaded skill (not swapped, not hardcoded).
#[tokio::test]
async fn skill_loaded_event_carries_agent_id_and_skill_name() {
    let fw = fw_with_skills(&["shout"]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = SkillScriptStub::new(
        vec![(
            "LoadSkill".to_string(),
            json!({ "skill_name": "shout", "reason": "x" }),
        )],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with_skills(
        &fw,
        "load shout",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
        skills_map(&[("shout", "BE LOUD.")]),
    )
    .await
    .expect("the run completes");

    let loaded = outcome.trace.iter().find_map(|e| match e {
        AgentEvent::SkillLoaded {
            agent_id,
            skill_name,
            ..
        } => Some((agent_id.clone(), skill_name.clone())),
        _ => None,
    });
    assert_eq!(
        loaded,
        Some(("worker".to_string(), "shout".to_string())),
        "SkillLoaded must carry agent_id='worker' + skill_name='shout'"
    );
}

/// C.4.5 — composition: two `LoadSkill` calls are additive; both skills'
/// bodies are present in the final turn's context (ADR-0027 additive
/// injection).
#[tokio::test]
async fn two_load_skills_compose_additively_in_context() {
    let fw = fw_with_skills(&["shout", "brief"]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = SkillScriptStub::new(
        vec![
            (
                "LoadSkill".to_string(),
                json!({ "skill_name": "shout", "reason": "loud" }),
            ),
            (
                "LoadSkill".to_string(),
                json!({ "skill_name": "brief", "reason": "short" }),
            ),
        ],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let _outcome = run_test_session_with_skills(
        &fw,
        "load two skills",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
        skills_map(&[
            ("shout", "SKILL-A: ALL CAPS."),
            ("brief", "SKILL-B: BE TERSE."),
        ]),
    )
    .await
    .expect("the run completes");

    let configs: Vec<AgentConfig> = seen.lock().expect("seen lock").clone();
    let last = configs.last().expect("at least one turn captured");
    let bodies = all_tool_result_texts(last);
    let joined = bodies.join(" | ");
    assert!(
        joined.contains("SKILL-A") && joined.contains("SKILL-B"),
        "both loaded skills' bodies must persist additively in context; got {joined:?}"
    );
}

// ── unit contract (the pure handler) ──────────────────────────────────

/// C.4.2 — `load_skill` returns the resolved body for a skill that is in
/// `allowed_skills` and present in the resolved map.
#[test]
fn load_skill_returns_body_for_allowed_and_resolved_skill() {
    let allowed = vec!["shout".to_string()];
    let resolved = skills_map(&[("shout", "ALWAYS SHOUT.")]);
    let loaded: LoadedSkill =
        load_skill("shout", &allowed, &resolved).expect("an allowed+resolved skill loads");
    assert_eq!(loaded.name, "shout");
    assert_eq!(loaded.body, "ALWAYS SHOUT.");
}

/// C.4.3 — `load_skill` denies a skill NOT in `allowed_skills` (the
/// capability gate — mirrors rung 1's scope denial). The denied skill's
/// name is carried for the surface.
#[test]
fn load_skill_denies_skill_not_in_allowed_skills() {
    let allowed = vec!["shout".to_string()];
    let resolved = skills_map(&[("shout", "x"), ("evil", "y")]);
    let err = load_skill("evil", &allowed, &resolved)
        .expect_err("a skill not in allowed_skills must be denied even if a body exists");
    assert!(
        matches!(err, LoadSkillError::NotAllowed(ref s) if s == "evil"),
        "expected NotAllowed(\"evil\"), got {err:?}"
    );
}

/// Defensive (kills the mutant that ignores the resolved-skills lookup):
/// a skill that IS allowed but has no resolved body is an error, not an
/// empty-body success.
#[test]
fn load_skill_allowed_but_unresolved_is_an_error() {
    let allowed = vec!["shout".to_string()];
    let resolved = BTreeMap::new();
    let err = load_skill("shout", &allowed, &resolved)
        .expect_err("an allowed skill with no resolved body must error, not return an empty body");
    assert!(
        matches!(err, LoadSkillError::NotResolved(ref s) if s == "shout"),
        "expected NotResolved(\"shout\"), got {err:?}"
    );
}
