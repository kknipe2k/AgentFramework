//! Replay tests — Stage E (M03); real-shape rewrite at M08.8.B.fix2 (TD-044).
//!
//! Exercises `runtime_main::sdk::replay::replay_signals_to_events`. Pure-
//! function translator from drone-stored signal-log JSON shape to
//! `runtime_core::event::AgentEvent`. Inverse of the M02.D `EventPipeline`.
//!
//! These tests were rewritten when TD-044 found the original fixtures
//! FABRICATED a signal shape that never existed on disk (`payload_json`
//! keyed on a fictional `event` field). The real `payload_json` IS a
//! serialized `AgentEvent`, so the fixtures here build each signal by
//! `serde_json::to_value`-ing a real `AgentEvent` — the §5 behavior-not-
//! tautology fix. The drone's `ReadSignals` row also carries top-level
//! `type` (the signal kind) + `event` columns alongside `payload_json`
//! (see `vdr::signals_for_session`); the translator reads only
//! `payload_json`, but the fixtures include the siblings to mirror the
//! real on-disk row.

use runtime_core::event::{AgentEvent, ToolSource};
use runtime_main::sdk::replay::replay_signals_to_events;
use serde_json::json;

/// Build a drone signal-log row: top-level `type` (kind) + `event`
/// columns plus `payload_json` = the serialized `AgentEvent` (the real
/// shape `vdr::signals_for_session` returns).
fn signal(kind: &str, event: &str, evt: &AgentEvent) -> serde_json::Value {
    json!({
        "type": kind,
        "event": event,
        "payload_json": serde_json::to_value(evt).expect("serialize AgentEvent"),
    })
}

#[test]
fn each_signal_type_translates_to_expected_agent_event() {
    let originals = [
        (
            "session",
            "session_start",
            AgentEvent::SessionStart {
                session_id: "s1".into(),
                framework: "aria".into(),
                model: "haiku".into(),
            },
        ),
        (
            "agent",
            "agent_spawned",
            AgentEvent::AgentSpawned {
                agent_id: "a1".into(),
                agent_name: "smoke".into(),
                parent_id: None,
                session_id: "s1".into(),
                narrowed_from: Vec::new(),
            },
        ),
        (
            "tool",
            "tool_invoked",
            AgentEvent::ToolInvoked {
                agent_id: "a1".into(),
                tool_name: "search".into(),
                source: ToolSource::Builtin,
                server: None,
                input: json!({"q": "hi"}),
            },
        ),
        (
            "skill",
            "skill_loaded",
            AgentEvent::SkillLoaded {
                agent_id: "a1".into(),
                skill_name: "skim".into(),
                mode: Some("lite".into()),
            },
        ),
        (
            "decision",
            "decision_record",
            AgentEvent::DecisionRecord {
                agent_id: "a1".into(),
                decision: "pick haiku".into(),
                rationale: "cost".into(),
                tool_used: Some("estimate_cost".into()),
            },
        ),
        (
            "agent",
            "agent_complete",
            AgentEvent::AgentComplete {
                agent_id: "a1".into(),
                result: "hi".into(),
                tokens_total: None,
            },
        ),
    ];
    let signals: Vec<serde_json::Value> = originals
        .iter()
        .map(|(kind, event, evt)| signal(kind, event, evt))
        .collect();

    let events = replay_signals_to_events(&signals);
    let expected: Vec<AgentEvent> = originals.iter().map(|(_, _, e)| e.clone()).collect();
    assert_eq!(
        events, expected,
        "each real signal row must replay back to its exact AgentEvent"
    );
}

#[test]
fn ordering_is_preserved_across_translation() {
    let signals: Vec<serde_json::Value> = ["a1", "a2", "a3"]
        .iter()
        .map(|id| {
            signal(
                "agent",
                "agent_spawned",
                &AgentEvent::AgentSpawned {
                    agent_id: (*id).into(),
                    agent_name: "n".into(),
                    parent_id: None,
                    session_id: "s1".into(),
                    narrowed_from: Vec::new(),
                },
            )
        })
        .collect();

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
fn malformed_payloads_are_filtered_not_panicked() {
    // None of these `payload_json` values is a serialized `AgentEvent`, so
    // each must drop out (deserialize → Err → filtered) rather than panic.
    let signals = vec![
        json!({}),                                            // no payload_json at all
        json!({ "payload_json": null }),                      // null payload
        json!({ "payload_json": {"agent_id": "a"} }),         // no `type` tag
        json!({ "payload_json": {"type": "not_an_event"} }),  // unknown variant tag
        json!({ "payload_json": {"type": "agent_spawned"} }), // missing required fields
        json!({ "payload_json": {"type": "token_usage"} }),   // projector-only signal
    ];

    let events = replay_signals_to_events(&signals);
    assert!(
        events.is_empty(),
        "malformed / non-event signals must be filtered, got {events:?}"
    );
}

#[test]
fn large_signal_log_translates_without_panic_or_oom() {
    // Per docs/gotchas.md #28 — bounded fixture (Vec, not stream::repeat).
    // 100 signals exercises iteration path + heap allocations without
    // approaching memory pressure (each signal is small JSON).
    let signals: Vec<serde_json::Value> = (0..100)
        .map(|i| {
            signal(
                "agent",
                "agent_spawned",
                &AgentEvent::AgentSpawned {
                    agent_id: format!("a{i}"),
                    agent_name: "n".into(),
                    parent_id: None,
                    session_id: "s1".into(),
                    narrowed_from: Vec::new(),
                },
            )
        })
        .collect();

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
