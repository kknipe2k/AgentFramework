//! M08.7.E rung 5 — LIVE budget-enforcement eval (encoded IRL, eval E-05).
//!
//! The CI assembled test (`budget_runloop_execution.rs`) proves the WIRE +
//! the four-threshold dispatch + the run-halt with a STUB provider whose
//! `estimate_cost` is token-driven — a scripted `Usage` crosses a tiny cap
//! and the loop stops. It CANNOT prove a REAL model + REAL Anthropic pricing
//! produces a per-turn cost that `record_spend` enforces (rule 11 / gotcha
//! #66 — the stub is not the model, and its pricing is not the real table).
//! This file encodes the behavioral close gate (the rung-5 IRL): a REAL
//! Anthropic run under a tiny `session_usd_cap`, given a task that would loop
//! indefinitely (a built-in `Read` tool + "never stop reading"), HALTS at the
//! cap — `budget_exceeded` fires (`HardStop`) and the run stops issuing
//! provider turns (no runaway spend — the safety primitive).
//!
//! `#[ignore]`d — it makes a real network call and needs a key (CLAUDE.md
//! §10: no real internet in the default test run). Run it explicitly:
//!
//! ```text
//! ANTHROPIC_API_KEY=sk-ant-... \
//!   cargo test -p runtime-main --test budget_live -- --ignored --nocapture
//! ```
//!
//! It also skips gracefully (returns) if the key is absent, so an accidental
//! `--ignored` run on a keyless machine does not fail.
//!
//! The load-bearing assertion is `budget_exceeded` fired. The "no runaway"
//! check counts the per-turn `token_usage` events (one per provider turn) and
//! asserts the run halted promptly. If a real run somehow does NOT cross the
//! cap (e.g. the model refuses to call the tool AND the single turn's tokens
//! fall under the cap — implausible for any tiny cap), the assertion fails
//! with a message saying so — the maintainer SURFACES that (shrink the cap /
//! strengthen the task) rather than weakening the assertion.

use std::sync::Arc;

use secrecy::SecretString;
use serde_json::{json, Value};
use tempfile::TempDir;

use runtime_core::event::AgentEvent;
use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::anthropic::AnthropicProvider;
use runtime_main::sdk::SessionId;

/// A schema-valid one-agent framework with a TINY `session_usd_cap`
/// ($0.0001 — below one real turn's token cost) and a `worker` that holds the
/// built-in `Read` tool over `probe_path`. The "never stop reading" task
/// gives the model a reason to keep dispatching a tool every turn — so absent
/// the budget halt the loop would run to `MAX_AGENT_TURNS`. The budget must
/// stop it at the first measured turn.
fn fw_tiny_cap(probe_path: &str) -> Framework {
    serde_json::from_value(json!({
        "name": "m08-7-e-rung5-live",
        "version": "1.0.0",
        "description": "M08.7.E rung 5 live budget-enforcement eval fixture",
        "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
        "budget": { "session_usd_cap": 0.0001 },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "capabilities": {
                "tools_called": [],
                "skills_loaded": [],
                "file_access": { "read": [probe_path], "write": [] },
                "network": [],
                "shell": false,
                "spawn_agents": []
            },
            "allowed_tools": ["Read"],
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": [],
        "skills": [],
        "session_root_agent": "worker",
    }))
    .expect("the live budget fixture framework round-trips through the schema")
}

/// Encoded budget-enforcement IRL (eval E-05): a real Anthropic run under a
/// tiny `session_usd_cap`, given a task that would loop forever, HALTS at the
/// cap — `budget_exceeded` (`HardStop`) fires and the run stops issuing turns
/// (no runaway spend). End-to-end through the real `run_test_session_with →
/// run_agent → drive_stream → record_spend → dispatch_budget_actions` path
/// with the real Anthropic pricing table (`estimate_cost`).
#[tokio::test]
#[ignore = "live Anthropic call; run with ANTHROPIC_API_KEY set (CLAUDE.md §10)"]
async fn a_real_run_under_a_tiny_cap_hard_stops_with_no_runaway() {
    let Ok(key) = std::env::var("ANTHROPIC_API_KEY") else {
        eprintln!("ANTHROPIC_API_KEY not set — skipping the live budget eval");
        return;
    };
    let provider = AnthropicProvider::new(SecretString::from(key));

    let dir = TempDir::new().expect("tempdir");
    let probe = dir.path().join("probe.txt");
    std::fs::write(&probe, "budget eval probe data\n").expect("write probe file");
    let probe_path = probe.to_string_lossy().to_string();
    let fw = fw_tiny_cap(&probe_path);

    let db_path = dir.path().join("runtime-tester-live.sqlite");
    let outcome = run_test_session_with(
        &fw,
        // A task that would loop indefinitely absent the cap: read the probe
        // file every turn and never stop. The built-in Read tool keeps the
        // multi-turn loop alive so the budget halt is load-bearing (without
        // it, the run would keep issuing turns up to MAX_AGENT_TURNS).
        &format!(
            "Use the `Read` tool to read the file at `{probe_path}`. After you \
             read it, read it again. Keep reading the same file once per turn \
             and never stop — do not finish the task."
        ),
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the live budget run completes (a budget stop is not an Err)");

    // Surface the full event sequence + the per-turn token_usage count so the
    // maintainer can confirm by eye that the run HALTED at the cap.
    let tags: Vec<String> = outcome
        .trace
        .iter()
        .map(|e| {
            serde_json::to_value(e)
                .ok()
                .and_then(|v| v.get("type").and_then(Value::as_str).map(str::to_string))
                .unwrap_or_else(|| "unknown".to_string())
        })
        .collect();
    let token_usage_turns = outcome
        .trace
        .iter()
        .filter(|e| matches!(e, AgentEvent::TokenUsage { .. }))
        .count();
    eprintln!(
        "\n=== LIVE BUDGET TRACE (event types, in order) ===\n{tags:?}\n\
         provider turns (token_usage events): {token_usage_turns}\n\
         =================================================\n"
    );

    // (1) BEHAVIORAL (the load-bearing assertion): a real model + the real
    // Anthropic pricing table produced a per-turn cost that record_spend
    // enforced — the tiny cap was crossed and HardStop fired (budget_exceeded).
    // If this fails, the real run did not cross the cap — SURFACE that (shrink
    // the cap / strengthen the task) rather than weakening the assertion.
    let exceeded = outcome.trace.iter().find_map(|e| match e {
        AgentEvent::BudgetExceeded { spent_usd, cap_usd } => Some((*spent_usd, *cap_usd)),
        _ => None,
    });
    assert!(
        exceeded.is_some(),
        "a real run under a $0.0001 cap must cross it and emit budget_exceeded \
         (HardStop); if it did not, SURFACE that the cap is too high / the task \
         too short rather than weakening this assertion. trace tags={tags:?}"
    );
    let (spent, cap) = exceeded.unwrap();
    eprintln!("budget_exceeded: spent ${spent:.6} > cap ${cap:.6}");

    // (2) NO RUNAWAY (the safety primitive, observed): the run halted at the
    // cap — it did NOT keep issuing provider turns up to MAX_AGENT_TURNS (16)
    // despite the "never stop" task. The per-turn token_usage count is the
    // observable turn count; a halted run is a small handful, never near 16.
    assert!(
        token_usage_turns <= 3,
        "the run must halt promptly at the cap, not run away to MAX_AGENT_TURNS \
         — got {token_usage_turns} provider turns; trace tags={tags:?}"
    );
}
