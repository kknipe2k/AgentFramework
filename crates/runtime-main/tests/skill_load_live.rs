//! M08.7.C rung 3 — LIVE behavior-change eval (encoded IRL).
//!
//! The CI assembled test (`skill_load_execution.rs`) proves
//! *injection-into-context* with a stub provider — the skill body is
//! present in the turn-2 `AgentConfig`. It CANNOT prove behavior-change:
//! the stub is not the model (rule 11 / gotcha #66). This file encodes the
//! behavioral close gate: a REAL Anthropic model that loads a "reply in
//! ALL CAPS" skill via `LoadSkill` replies in all caps.
//!
//! `#[ignore]`d — it makes a real network call and needs a key (CLAUDE.md
//! §10: no real internet in the default test run). Run it explicitly:
//!
//! ```text
//! ANTHROPIC_API_KEY=sk-ant-... \
//!   cargo test -p runtime-main --test skill_load_live -- --ignored --nocapture
//! ```
//!
//! It also skips gracefully (returns) if the key is absent, so an
//! accidental `--ignored` run on a keyless machine does not fail.

use std::collections::BTreeMap;
use std::sync::Arc;

use secrecy::SecretString;
use serde_json::{json, Value};
use tempfile::TempDir;

use runtime_core::event::AgentEvent;
use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with_skills;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::anthropic::AnthropicProvider;
use runtime_main::sdk::SessionId;

/// A schema-valid one-agent framework whose `worker` agent may load the
/// given skills (`allowed_skills`) — mirrors the assembled-test fixture.
fn fw_with_skills(allowed_skills: &[&str]) -> Framework {
    let skills: Vec<Value> = allowed_skills
        .iter()
        .map(|s| json!({ "name": s }))
        .collect();
    serde_json::from_value(json!({
        "name": "m08-7-c-rung3-live",
        "version": "1.0.0",
        "description": "M08.7.C rung 3 live skill-load eval fixture",
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
    .expect("the live fixture framework round-trips through the schema")
}

fn skills_map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

/// The full concatenated `StreamText` the run emitted (the model's reply).
fn final_text(trace: &[AgentEvent]) -> String {
    trace
        .iter()
        .filter_map(|e| match e {
            AgentEvent::StreamText { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Encoded behavior-change eval (the rung-3 IRL): a real Anthropic model
/// that loads the "reply in ALL CAPS" skill via `LoadSkill` replies in all
/// caps — proving the skill content reached the model and changed its
/// behavior, end-to-end through the real `run_test_session_with_skills →
/// run_agent → drive_stream → load_skill` path.
#[tokio::test]
#[ignore = "live Anthropic call; run with ANTHROPIC_API_KEY set (CLAUDE.md §10)"]
async fn loaded_skill_makes_a_real_model_reply_in_all_caps() {
    let Ok(key) = std::env::var("ANTHROPIC_API_KEY") else {
        eprintln!("ANTHROPIC_API_KEY not set — skipping the live skill-load behavior eval");
        return;
    };
    let provider = AnthropicProvider::new(SecretString::from(key));
    let fw = fw_with_skills(&["shout"]);
    let body = "CRITICAL OUTPUT RULE: from now on, reply ENTIRELY IN UPPERCASE. \
                Use only capital letters — never any lowercase letter.";

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester-live.sqlite");
    let outcome = run_test_session_with_skills(
        &fw,
        "First use the LoadSkill tool to load the skill named 'shout'. \
         Then greet the user in one short sentence.",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
        skills_map(&[("shout", body)]),
    )
    .await
    .expect("the live skill-load run completes");

    // The skill loaded (the event fired) — necessary but NOT sufficient.
    assert!(
        outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::SkillLoaded { skill_name, .. } if skill_name == "shout"
        )),
        "the model must call LoadSkill('shout'); trace={:?}",
        outcome.trace
    );

    // BEHAVIOR-CHANGE (the load-bearing assertion): the model's reply is
    // all-caps — the loaded skill changed observable behavior. Lenient on
    // non-letters (spaces/punctuation/digits); every ALPHABETIC char must
    // be uppercase, and there must be some alphabetic reply text.
    let reply = final_text(&outcome.trace);
    let letters: String = reply.chars().filter(|c| c.is_alphabetic()).collect();
    assert!(
        !letters.is_empty(),
        "the model produced no alphabetic reply text; reply={reply:?}"
    );
    assert!(
        letters.chars().all(char::is_uppercase),
        "the loaded shout skill must make the reply ALL CAPS; reply={reply:?}"
    );
}
