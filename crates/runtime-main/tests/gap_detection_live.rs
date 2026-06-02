//! M08.7.D rung 4 — LIVE gap-detection eval (encoded IRL, eval E-04).
//!
//! The CI assembled test (`gap_detection_execution.rs`) proves the WIRE +
//! the SUSPEND mechanics with a stub provider — a scripted
//! `request_capability` `ToolUse` routes to `handle_request_capability` and
//! the loop suspends. It CANNOT prove a REAL model triggers it: the stub is
//! not the model (rule 11 / gotcha #66). This file encodes the behavioral
//! close gate (the rung-4 IRL): a REAL Anthropic model that lacks a
//! capability calls `request_capability` for it, and the session suspends
//! cleanly (a `ToolMissing` gap with `requested_via=request_capability`, and
//! the gap is left UNRESOLVED — no `tool_result` fed back, no resume — the
//! v0.1 suspend-and-record outcome; resolve-and-resume is the scheduled
//! gap-resume rung, ADR-0029).
//!
//! `#[ignore]`d — it makes a real network call and needs a key (CLAUDE.md
//! §10: no real internet in the default test run). Run it explicitly:
//!
//! ```text
//! ANTHROPIC_API_KEY=sk-ant-... \
//!   cargo test -p runtime-main --test gap_detection_live -- --ignored --nocapture
//! ```
//!
//! It also skips gracefully (returns) if the key is absent, so an
//! accidental `--ignored` run on a keyless machine does not fail.
//!
//! If a real model will not RELIABLY call `request_capability` for the
//! missing tool (it improvises / refuses instead), the assertion below
//! fails with a message saying so — the maintainer SURFACES that (the task
//! prompt needs strengthening) rather than weakening the assertion. The
//! suspend behavior is not in question; the model's tool-call reliability is.

use std::sync::Arc;

use secrecy::SecretString;
use serde_json::{json, Value};
use tempfile::TempDir;

use runtime_core::event::{AgentEvent, GapSourceRef};
use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::anthropic::AnthropicProvider;
use runtime_main::sdk::SessionId;

/// A schema-valid one-agent framework whose `worker` agent declares NO
/// tools — so it genuinely lacks a "deploy" capability. `request_capability`
/// is the runtime-auto-injected meta-tool (spec §4b), so the agent can
/// always signal the gap.
fn fw_no_deploy() -> Framework {
    serde_json::from_value(json!({
        "name": "m08-7-d-rung4-live",
        "version": "1.0.0",
        "description": "M08.7.D rung 4 live gap-detection eval fixture",
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
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": [],
        "skills": [],
        "session_root_agent": "worker",
    }))
    .expect("the live fixture framework round-trips through the schema")
}

/// Encoded gap-detection IRL (eval E-04): a real Anthropic model that lacks
/// a "deploy" tool calls `request_capability` for it, and the session
/// suspends cleanly — the `ToolMissing` gap fires with
/// `requested_via=request_capability`, the meta-tool is NOT denied as an
/// undeclared tool (the wire fired), and the gap is left unresolved (no
/// resume — suspend-and-record, the v0.1 outcome; resolve-and-resume is the
/// scheduled gap-resume rung, ADR-0029). End-to-end through the real
/// `run_test_session_with → run_agent → drive_stream → handle_request_capability`
/// path.
#[tokio::test]
#[ignore = "live Anthropic call; run with ANTHROPIC_API_KEY set (CLAUDE.md §10)"]
async fn a_real_model_lacking_a_capability_raises_a_gap_and_suspends() {
    let Ok(key) = std::env::var("ANTHROPIC_API_KEY") else {
        eprintln!("ANTHROPIC_API_KEY not set — skipping the live gap-detection eval");
        return;
    };
    let provider = AnthropicProvider::new(SecretString::from(key));
    let fw = fw_no_deploy();

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester-live.sqlite");
    let outcome = run_test_session_with(
        &fw,
        // Direct instruction so the model reliably calls the meta-tool
        // (mirrors skill_load_live's explicit \"First use the LoadSkill\").
        "You must deploy the current build, but you have NO deployment tool \
         available. Do not improvise and do not explain — use the \
         `request_capability` tool to request the missing capability: set \
         `capability_kind` to \"tool\", `capability_name` to \"deploy\", and \
         give a one-line reason.",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the live gap-detection run completes (a suspend is not an Err)");

    // Surface the full event sequence so the maintainer can confirm the run
    // HALTED at the gap (no further turn) by eye.
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
    eprintln!("\n=== LIVE GAP TRACE (event types, in order) ===\n{tags:?}\n==============================================\n");

    // (1) BEHAVIORAL (the load-bearing assertion): the real model called
    // request_capability for the missing tool, and it routed to the gap
    // handler — a ToolMissing gap with requested_via=request_capability.
    // (Lenient on the exact name; the requested_via source is the proof the
    // meta-tool fired.) If this fails, the model did not call
    // request_capability — SURFACE that (strengthen the task prompt), do not
    // weaken the assertion.
    let gap = outcome.trace.iter().find_map(|e| match e {
        AgentEvent::ToolMissing {
            tool_name,
            requested_via: GapSourceRef::RequestCapability,
            ..
        } => Some(tool_name.clone()),
        _ => None,
    });
    assert!(
        gap.is_some(),
        "the real model must call request_capability for the missing tool \
         (a ToolMissing gap with requested_via=request_capability); if it \
         improvised/refused, SURFACE that the task prompt needs strengthening \
         rather than weakening this assertion. trace tags={tags:?}"
    );
    eprintln!("gap raised for capability: {:?}", gap.unwrap());

    // (2) CLEAN SUSPEND: the meta-tool was NOT denied as an undeclared tool
    // (the wire fired, not the painted pipeline.next_event CapabilityViolation).
    assert!(
        !outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::CapabilityViolation { requested_action, .. }
                if requested_action.contains("request_capability")
        )),
        "request_capability must route to the gap handler, never be denied as \
         an undeclared tool; trace tags={tags:?}"
    );

    // (3) SUSPEND-AND-RECORD (the v0.1 outcome): the gap is left UNRESOLVED —
    // no tool_result was fed back for the request_capability call (no resume).
    // resolve-and-resume is the scheduled gap-resume rung (ADR-0029), NOT
    // exercised here.
    assert!(
        !outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::ToolResult { tool_name, .. } if tool_name == "request_capability"
        )),
        "the gap must be left unresolved (no tool_result fed back) — \
         suspend-and-record, not resolve-and-resume; trace tags={tags:?}"
    );
}
