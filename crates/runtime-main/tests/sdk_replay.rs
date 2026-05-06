//! Replay tests — Stage E (M03).
//!
//! Exercises `runtime_main::sdk::replay::replay_signals_to_events`. Pure-
//! function translator from drone-stored signal-log JSON shape to
//! `runtime_core::event::AgentEvent`. Inverse of the M02.D `EventPipeline`.

use runtime_core::event::AgentEvent;
use runtime_main::sdk::replay::replay_signals_to_events;
use serde_json::json;

fn signal(sig_type: &str, payload: &serde_json::Value) -> serde_json::Value {
    json!({
        "type": sig_type,
        "payload_json": payload,
    })
}

#[test]
fn each_signal_type_translates_to_expected_agent_event() {
    let signals = vec![
        signal(
            "session",
            &json!({"event": "start", "session_id": "s1", "framework": "aria", "model": "haiku"}),
        ),
        signal(
            "agent",
            &json!({"event": "spawned", "agent_id": "a1", "agent_name": "smoke", "session_id": "s1"}),
        ),
        signal(
            "tool",
            &json!({"agent_id": "a1", "tool_name": "search", "input": {"q": "hi"}}),
        ),
        signal(
            "skill",
            &json!({"agent_id": "a1", "skill_name": "skim", "mode": "lite"}),
        ),
        signal(
            "decision",
            &json!({
                "agent_id": "a1",
                "decision": "pick haiku",
                "rationale": "cost",
                "tool_used": "estimate_cost"
            }),
        ),
        signal(
            "agent",
            &json!({"event": "complete", "agent_id": "a1", "result": "hi"}),
        ),
    ];

    let events = replay_signals_to_events(&signals);
    let kinds: Vec<&str> = events
        .iter()
        .map(|e| match e {
            AgentEvent::SessionStart { .. } => "session_start",
            AgentEvent::AgentSpawned { .. } => "agent_spawned",
            AgentEvent::ToolInvoked { .. } => "tool_invoked",
            AgentEvent::SkillLoaded { .. } => "skill_loaded",
            AgentEvent::DecisionRecord { .. } => "decision_record",
            AgentEvent::AgentComplete { .. } => "agent_complete",
            _ => "other",
        })
        .collect();

    assert_eq!(
        kinds,
        vec![
            "session_start",
            "agent_spawned",
            "tool_invoked",
            "skill_loaded",
            "decision_record",
            "agent_complete"
        ]
    );
}

#[test]
fn ordering_is_preserved_across_translation() {
    let signals = vec![
        signal(
            "agent",
            &json!({"event": "spawned", "agent_id": "a1", "agent_name": "n", "session_id": "s1"}),
        ),
        signal(
            "agent",
            &json!({"event": "spawned", "agent_id": "a2", "agent_name": "n", "session_id": "s1"}),
        ),
        signal(
            "agent",
            &json!({"event": "spawned", "agent_id": "a3", "agent_name": "n", "session_id": "s1"}),
        ),
    ];

    let events = replay_signals_to_events(&signals);
    let ids: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::AgentSpawned { agent_id, .. } => Some(agent_id.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(ids, vec!["a1", "a2", "a3"]);
}

#[test]
fn missing_required_fields_are_filtered_not_panicked() {
    // Each entry deliberately omits a required discriminator or field.
    let signals = vec![
        json!({}),                                     // no type at all
        signal("tool", &json!({})),                    // tool with no agent_id/tool_name
        signal("decision", &json!({"agent_id": "a"})), // decision missing rationale
        signal("agent", &json!({"event": "spawned"})), // agent_spawned missing agent_id
        signal("agent", &json!({"event": "weird"})),   // unknown agent event
        signal("nope", &json!({})),                    // unknown signal type
    ];

    // Must not panic.
    let events = replay_signals_to_events(&signals);
    // None of these produce events because they miss required fields.
    assert!(
        events.is_empty(),
        "malformed signals must be filtered, got {events:?}"
    );
}

#[test]
fn large_signal_log_translates_without_panic_or_oom() {
    // Per docs/gotchas.md #28 — bounded fixture (Vec, not stream::repeat).
    // 100 signals exercises iteration path + heap allocations without
    // approaching memory pressure (each signal is small JSON).
    let mut signals = Vec::with_capacity(100);
    for i in 0..100 {
        signals.push(signal(
            "agent",
            &json!({
                "event": "spawned",
                "agent_id": format!("a{i}"),
                "agent_name": "n",
                "session_id": "s1",
            }),
        ));
    }

    let events = replay_signals_to_events(&signals);
    assert_eq!(events.len(), 100, "all 100 signals must translate");
    // Spot-check first + last to ensure ordering preserved at scale.
    if let AgentEvent::AgentSpawned { agent_id, .. } = &events[0] {
        assert_eq!(agent_id, "a0");
    } else {
        panic!("first event must be AgentSpawned");
    }
    if let AgentEvent::AgentSpawned { agent_id, .. } = &events[99] {
        assert_eq!(agent_id, "a99");
    } else {
        panic!("last event must be AgentSpawned");
    }
}
