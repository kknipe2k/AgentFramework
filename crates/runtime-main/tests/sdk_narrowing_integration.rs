//! M06 Stage A — production integration tests for the L2a wire-up.
//!
//! Canonical "what the SDK actually does" surface for the L2a
//! `narrow(parent, proposed)` primitive (M05.B) wired into the
//! production sub-agent spawn path at
//! `crates/runtime-main/src/sdk/agent_sdk.rs::spawn_framework_subagents`.
//! Replaces the synthetic narrowing scenario in M05.B's
//! `capability_enforcer_smoke.rs` (which exercises the per-call
//! narrowing logic but no longer claims to test the SDK wire-up).
//!
//! Closes ADR-0009 Finding #2: M06.V Wire pass trace #3 (`narrow` before
//! `AgentSpawned`) is satisfiable from this integration test after
//! Stage A lands.
//!
//! Test seam: each test constructs an `AgentSdk::with_capability_wiring`
//! over a synthetic multi-agent `Framework` (inline JSON) and asserts
//! on the `AgentSpawned` events the renderer would see for each declared
//! sub-agent. The `narrowed_from` field on `AgentSpawned` is the
//! load-bearing wire-trace endpoint: present means the spawn went
//! through `narrow()`; absent means it did not.
//!
//! Per gotcha #66: every test asserts BOTH directions
//! (`AgentSpawned` MUST appear for valid narrowings; `AgentSpawned` MUST
//! NOT appear for widening attempts; `CapabilityViolation` MUST appear
//! when the wire blocks a spawn).

use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, BoxStream};
use runtime_core::event::AgentEvent;
use runtime_core::generated::framework::Framework;
use runtime_main::capability::CapabilityEnforcer;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::framework_loader::FrameworkRef;
use runtime_main::hitl::{HitlChoice, HitlSeam};
use runtime_main::providers::{
    AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
    ProviderSupport,
};
use runtime_main::sdk::{AgentSdk, CapabilityWiring, SessionId};
use runtime_main::tier::Tier;
use serde_json::json;
use tokio::sync::mpsc;

// ── Test fixtures ─────────────────────────────────────────────────────

struct ScriptedProvider {
    script: std::sync::Mutex<Option<Vec<ProviderEvent>>>,
}

impl ScriptedProvider {
    const fn new(events: Vec<ProviderEvent>) -> Self {
        Self {
            script: std::sync::Mutex::new(Some(events)),
        }
    }
}

#[async_trait::async_trait]
impl LLMProvider for ScriptedProvider {
    fn name(&self) -> &'static str {
        "scripted-stub"
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
        _config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        let events = self
            .script
            .lock()
            .expect("no poisoning")
            .take()
            .unwrap_or_default();
        Ok(Box::pin(stream::iter(events)))
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

/// Build a parent-child agent framework with the given `parent`
/// capability surface and a `child` whose proposed capabilities are
/// supplied. The child is wired as a declared inline agent of the
/// framework so the SDK's spawn-walk picks it up.
#[allow(clippy::too_many_arguments)]
fn fw_parent_child(
    parent_id: &str,
    parent_tools: &[&str],
    parent_read: &[&str],
    parent_write: &[&str],
    child_id: &str,
    child_tools: &[&str],
    child_read: &[&str],
    child_write: &[&str],
    framework_tools: &[(&str, &str)],
) -> Framework {
    let tool_items: Vec<serde_json::Value> = framework_tools
        .iter()
        .map(|(n, src)| json!({ "name": n, "source": src }))
        .collect();
    serde_json::from_value(json!({
        "name": "test-fw",
        "version": "1.0.0",
        "description": "L2a wire-up integration test framework",
        "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
        "agents": [
            {
                "id": parent_id,
                "role": "parent",
                "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
                "capabilities": {
                    "tools_called": parent_tools,
                    "skills_loaded": [],
                    "file_access": { "read": parent_read, "write": parent_write },
                    "network": [],
                    "shell": false,
                    "spawn_agents": [child_id]
                },
                "allowed_tools": parent_tools,
                "allowed_skills": [],
                "spawns": [child_id]
            },
            {
                "id": child_id,
                "role": "child",
                "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
                "capabilities": {
                    "tools_called": child_tools,
                    "skills_loaded": [],
                    "file_access": { "read": child_read, "write": child_write },
                    "network": [],
                    "shell": false,
                    "spawn_agents": []
                },
                "allowed_tools": child_tools,
                "allowed_skills": [],
                "spawns": []
            }
        ],
        "tools": tool_items,
        "skills": [],
        "session_root_agent": parent_id,
    }))
    .expect("test framework round-trips")
}

/// Sync — the `tokio::spawn` block below does its own awaits inside
/// an async block; the outer fn does not need to be async.
fn build_sdk(
    framework: Framework,
    auto_resolve_hitl: bool,
) -> (AgentSdk<ScriptedProvider>, mpsc::Receiver<AgentEvent>) {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    let enforcer = Arc::new(enforcer);
    let framework: FrameworkRef = Arc::new(framework);
    let hitl_seam = Arc::new(HitlSeam::new());

    // Streaming script is empty + MessageStop only — these tests
    // assert on the spawn-walk events, not on dispatch behavior.
    let provider = Arc::new(ScriptedProvider::new(vec![ProviderEvent::MessageStop {
        stop_reason: "end_turn".into(),
        total_tokens: None,
    }]));
    let drone = Arc::new(DroneClient::noop());
    let (tx, rx) = mpsc::channel::<AgentEvent>(64);
    let session_id = SessionId::new();
    let wiring = CapabilityWiring::new(
        Arc::clone(&enforcer),
        Arc::clone(&framework),
        Arc::clone(&hitl_seam),
    );
    let sdk = AgentSdk::with_capability_wiring(provider, tx, drone, session_id.clone(), wiring);

    if auto_resolve_hitl {
        let seam = Arc::clone(&hitl_seam);
        let prompt_id = format!("capability_violation:{}", session_id.as_string());
        tokio::spawn(async move {
            for _ in 0..600 {
                if seam.pending_len().await > 0 {
                    let _ = seam.resolve(&prompt_id, HitlChoice::new("abort")).await;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });
    }

    (sdk, rx)
}

fn default_config() -> AgentConfig {
    AgentConfig {
        model: "claude-sonnet-4-6".into(),
        messages: vec![],
        max_tokens: 100,
        temperature: None,
        system_prompt: None,
        tools: vec![],
    }
}

async fn drain_events(rx: &mut mpsc::Receiver<AgentEvent>) -> Vec<AgentEvent> {
    let mut events = Vec::new();
    while let Some(e) = rx.recv().await {
        events.push(e);
    }
    events
}

fn agent_spawned_for(events: &[AgentEvent], id: &str) -> Option<Vec<String>> {
    events.iter().find_map(|e| {
        if let AgentEvent::AgentSpawned {
            agent_id,
            narrowed_from,
            ..
        } = e
        {
            if agent_id == id {
                Some(narrowed_from.clone())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn capability_violation_for(events: &[AgentEvent], id: &str) -> bool {
    events.iter().any(|e| {
        matches!(
            e,
            AgentEvent::CapabilityViolation { agent_id, .. } if agent_id == id
        )
    })
}

// ── Tests ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn spawn_with_narrowed_grants_succeeds_and_emits_agent_spawned_with_narrowed_from() {
    // Wire-up trace #3 happy path: parent grants {Exec Read} +
    // child proposes {Exec Read} → narrow returns Ok →
    // AgentSpawned emitted with narrowed_from populated.
    let fw = fw_parent_child(
        "parent",
        &["Read"],
        &[],
        &[],
        "child",
        &["Read"],
        &[],
        &[],
        &[("Read", "builtin")],
    );

    let (sdk, mut rx) = build_sdk(fw, false);
    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    let narrowed = agent_spawned_for(&events, "child");
    assert!(narrowed.is_some(), "child must spawn; events: {events:?}");
    let narrowed = narrowed.expect("checked Some above");
    assert!(
        !narrowed.is_empty(),
        "narrowed_from must be populated for wired spawns; got empty for child"
    );
    // Sanity-check the description format: each entry is a
    // `kind:resource:scope:class` lowercased string.
    assert!(
        narrowed
            .iter()
            .any(|d| d.contains("exec") && d.contains("read")),
        "narrowed_from descriptions must reflect proposed grants; got: {narrowed:?}"
    );
    assert!(
        !capability_violation_for(&events, "child"),
        "successful narrowing MUST NOT emit CapabilityViolation for child"
    );
}

#[tokio::test]
async fn spawn_with_widening_attempt_emits_capability_violation_and_blocks_spawn() {
    // Wire-up trace #3 unhappy path: parent grants {Exec Read} +
    // child proposes {Write filesystem src/**} → narrow returns Err →
    // CapabilityViolation emitted, AgentSpawned NOT emitted for child.
    let fw = fw_parent_child(
        "parent",
        &["Read"],
        &[],
        &[],
        "child",
        &[],
        &[],
        &["src/**"], // child claims write access parent doesn't have
        &[("Read", "builtin")],
    );

    let (sdk, mut rx) = build_sdk(fw, true);
    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    assert!(
        capability_violation_for(&events, "child"),
        "widening must emit CapabilityViolation for child; events: {events:?}"
    );
    assert!(
        agent_spawned_for(&events, "child").is_none(),
        "widening MUST NOT emit AgentSpawned for child; events: {events:?}"
    );
}

#[tokio::test]
async fn spawn_with_empty_child_grants_succeeds_with_empty_narrowed_set() {
    // Edge case: child declares no capabilities (empty Capabilities
    // block). Narrowing returns Ok(empty) trivially. AgentSpawned
    // emits with empty narrowed_from — present-but-empty signals
    // "wired through narrow but proposed nothing."
    let fw = fw_parent_child(
        "parent",
        &["Read"],
        &[],
        &[],
        "child",
        &[],
        &[],
        &[],
        &[("Read", "builtin")],
    );

    let (sdk, mut rx) = build_sdk(fw, false);
    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    let narrowed = agent_spawned_for(&events, "child");
    assert!(narrowed.is_some(), "child must spawn even with no grants");
    let narrowed = narrowed.expect("checked Some above");
    assert!(
        narrowed.is_empty(),
        "empty proposed must produce empty narrowed_from; got: {narrowed:?}"
    );
    assert!(
        !capability_violation_for(&events, "child"),
        "empty proposed MUST NOT emit CapabilityViolation"
    );
}

#[tokio::test]
async fn spawn_walk_twice_in_sequence_both_succeed_and_emit_separately() {
    // Gotcha #69 multi-call invariant: two SDK runs against the same
    // framework state must each independently produce the spawn walk.
    // First-run mutation of internal state must not affect second run.
    let fw = fw_parent_child(
        "parent",
        &["Read"],
        &[],
        &[],
        "child",
        &["Read"],
        &[],
        &[],
        &[("Read", "builtin")],
    );

    // First run.
    let (sdk1, mut rx1) = build_sdk(fw.clone(), false);
    let task1 = tokio::spawn(async move { sdk1.run_agent(default_config()).await });
    let events1 = drain_events(&mut rx1).await;
    let () = task1.await.expect("join sdk1").expect("run_agent ok");

    // Second run with a fresh SDK + fresh framework (Arc-wrapped fresh
    // refs simulate a re-load).
    let (sdk2, mut rx2) = build_sdk(fw, false);
    let task2 = tokio::spawn(async move { sdk2.run_agent(default_config()).await });
    let events2 = drain_events(&mut rx2).await;
    let () = task2.await.expect("join sdk2").expect("run_agent ok");

    let narrowed1 = agent_spawned_for(&events1, "child");
    let narrowed2 = agent_spawned_for(&events2, "child");
    assert!(narrowed1.is_some(), "first run must spawn child");
    assert!(narrowed2.is_some(), "second run must spawn child");
    assert_eq!(
        narrowed1.expect("first"),
        narrowed2.expect("second"),
        "narrowed_from must be deterministic across runs"
    );
}
