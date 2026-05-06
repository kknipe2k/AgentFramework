//! Round-trip serialization tests for runtime-core types.

use runtime_core::{AgentEvent, DroneCommand, DroneEvent};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn framework_v1_round_trip() {
    let path = workspace_root().join("examples/aria/framework.json");
    let text = std::fs::read_to_string(&path).expect("read examples/aria/framework.json");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("parse aria framework");
    let into_typed: runtime_core::generated::framework::Framework =
        serde_json::from_value(parsed).expect("deserialize aria framework into typed Framework");
    // Re-serialize and deserialize again to verify no data loss through the typed layer.
    // Direct JSON comparison with the original isn't viable because typify emits default
    // values for optional fields absent in the original. Instead, verify the round-trip
    // through the typed layer is stable (serialize → deserialize → serialize produces
    // identical JSON).
    let first: serde_json::Value = serde_json::to_value(&into_typed).expect("serialize first");
    let round_tripped: runtime_core::generated::framework::Framework =
        serde_json::from_value(first.clone()).expect("deserialize round-tripped JSON");
    let second: serde_json::Value = serde_json::to_value(&round_tripped).expect("serialize second");
    assert_eq!(
        first, second,
        "typed round-trip should produce stable JSON output"
    );
}

#[test]
fn framework_ralph_round_trip() {
    let path = workspace_root().join("examples/ralph/framework.json");
    let text = std::fs::read_to_string(&path).expect("read examples/ralph/framework.json");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("parse ralph framework");
    let into_typed: runtime_core::generated::framework::Framework =
        serde_json::from_value(parsed).expect("deserialize ralph framework");
    let first: serde_json::Value = serde_json::to_value(&into_typed).expect("serialize first");
    let round_tripped: runtime_core::generated::framework::Framework =
        serde_json::from_value(first.clone()).expect("deserialize round-tripped JSON");
    let second: serde_json::Value = serde_json::to_value(&round_tripped).expect("serialize second");
    assert_eq!(first, second);
}

#[test]
fn drone_event_serde_tags_correct() {
    let event = DroneEvent::Heartbeat {
        status: runtime_core::HeartbeatStatus::Ok,
        timestamp: 1_234_567_890,
    };
    let json = serde_json::to_value(&event).expect("serialize");
    assert_eq!(
        json["type"], "heartbeat",
        "tag must be snake_case 'heartbeat'"
    );
    assert_eq!(json["status"], "ok");
    assert_eq!(json["timestamp"], 1_234_567_890_i64);
}

#[test]
fn drone_command_serde_tags_correct() {
    let cmd = DroneCommand::SnapshotNow {
        reason: "task_boundary".into(),
        state_json: serde_json::json!({}),
    };
    let json = serde_json::to_value(&cmd).expect("serialize");
    assert_eq!(json["type"], "snapshot_now", "tag must be 'snapshot_now'");
}

#[test]
fn agent_event_session_start_round_trip() {
    let event = AgentEvent::SessionStart {
        session_id: "s1".into(),
        framework: "examples/aria".into(),
        model: "claude-sonnet-4-6".into(),
    };
    let json: serde_json::Value = serde_json::to_value(&event).unwrap();
    let back: AgentEvent = serde_json::from_value(json).unwrap();
    assert_eq!(event, back);
}

#[test]
fn agent_event_capability_violation_round_trip() {
    let event = AgentEvent::CapabilityViolation {
        agent_id: "a1".into(),
        declared: "tools_called: [Read]".into(),
        attempted: "Bash".into(),
    };
    let json: serde_json::Value = serde_json::to_value(&event).unwrap();
    let back: AgentEvent = serde_json::from_value(json).unwrap();
    assert_eq!(event, back);
}

#[test]
fn skill_planning_frontmatter_round_trip() {
    let path = workspace_root().join("examples/aria/skills/planning.md");
    let text = std::fs::read_to_string(&path).expect("read planning.md");

    // Extract frontmatter (between leading --- and second ---).
    // Normalize line endings for Windows compatibility.
    let text = text.replace("\r\n", "\n");
    let parts: Vec<&str> = text.splitn(3, "---\n").collect();
    assert!(parts.len() >= 3, "expected frontmatter delimited by ---");
    let frontmatter_yaml = parts[1];

    let parsed: serde_yaml::Value =
        serde_yaml::from_str(frontmatter_yaml).expect("parse frontmatter");
    let typed: runtime_core::generated::skill::Skill =
        serde_yaml::from_value(parsed).expect("deserialize into Skill");
    // Verify round-trip stability: serialize → deserialize → serialize produces identical output.
    // Direct comparison with original YAML isn't viable because typify emits defaults.
    let first: serde_json::Value = serde_json::to_value(&typed).expect("serialize first");
    let round_tripped: runtime_core::generated::skill::Skill =
        serde_json::from_value(first.clone()).expect("deserialize round-tripped");
    let second: serde_json::Value = serde_json::to_value(&round_tripped).expect("serialize second");
    assert_eq!(first, second);
}

#[test]
fn drone_event_variant_count_matches_spec() {
    // If this test fails, you removed a variant — that's a breaking change.
    // Update this count if you added a variant in a later milestone (additive
    // changes are fine; keep this assertion in sync with spec §1d).
    // M03 Stage E added QueryResult + SignalLog (read-only IPC commands), bumping the count to 9.
    let variants_in_drone_event = 9;
    let _check = match (DroneEvent::Heartbeat {
        status: runtime_core::HeartbeatStatus::Ok,
        timestamp: 0,
    }) {
        DroneEvent::Heartbeat { .. } => 1,
        DroneEvent::SnapshotWritten { .. } => 2,
        DroneEvent::ActivityStateChange { .. } => 3,
        DroneEvent::ProcessSpawned { .. } => 4,
        DroneEvent::ProcessStopped { .. } => 5,
        DroneEvent::RecoveryAvailable { .. } => 6,
        DroneEvent::Alert { .. } => 7,
        DroneEvent::QueryResult { .. } => 8,
        DroneEvent::SignalLog { .. } => 9,
    };
    let _ = variants_in_drone_event;
    // Test passes if it compiles — the match exhaustiveness check enforces variant count.
}
