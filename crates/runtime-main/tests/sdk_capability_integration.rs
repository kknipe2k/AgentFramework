//! M06 Stage A — production integration tests for the L1 wire-up.
//!
//! Canonical "what the SDK actually does" surface for the L1 enforcer
//! (M05.B `enforcer.check`) wired into the production dispatch path at
//! `crates/runtime-main/src/sdk/event_pipeline.rs`. Replaces the M05.B
//! `capability_enforcer_smoke.rs` stand-in (which still exercises the
//! per-method enforcer logic but no longer claims to test the SDK
//! wire-up — see its updated header comment).
//!
//! Closes ADR-0009 Finding #1: M06.V Wire pass trace #2 (`enforcer.check`
//! before tool dispatch) is satisfiable from this integration test
//! after Stage A lands.
//!
//! Test seam: each test constructs an `AgentSdk::with_capability_wiring`
//! (real production constructor) over a synthetic `Framework` (inline
//! JSON; no fixture file dependency), feeds a scripted
//! `Stream<Item = ProviderEvent>` through the test seam
//! `run_agent_with_provider_stream`, and asserts on the
//! `mpsc::Receiver<AgentEvent>` events that the renderer would see.
//!
//! Per gotcha #66 contract-fidelity discipline: every test asserts BOTH
//! the events that MUST appear AND the events that MUST NOT appear
//! (e.g., `ToolInvoked` absent on a denied dispatch). The `tests-pass-
//! but-contract-fails` failure mode requires both directions of
//! assertion.
//!
//! Per gotcha #69 multi-call invariant: includes a
//! `*_twice_in_sequence_*` test — two sequential `ToolUse` events
//! through the same SDK both flow through the wire independently.

use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, BoxStream};
use runtime_core::event::{AgentEvent, CapabilityKindRef};
use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, ResourceName,
    SideEffectClass,
};
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
use std::str::FromStr;
use tokio::sync::mpsc;

// ── Test fixtures ─────────────────────────────────────────────────────

/// In-process provider that yields a scripted `ProviderEvent` sequence.
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

/// Build a single-agent framework JSON with the named tools declared.
/// The agent's capabilities `tools_called` mirrors `framework_tools` so
/// the agent's translated grant set covers the listed tools.
fn fw_with_one_agent(
    agent_id: &str,
    tools_called: &[&str],
    framework_tools: &[(&str, &str)],
) -> Framework {
    let tool_items: Vec<serde_json::Value> = framework_tools
        .iter()
        .map(|(n, src)| json!({ "name": n, "source": src }))
        .collect();
    serde_json::from_value(json!({
        "name": "test-fw",
        "version": "1.0.0",
        "description": "L1 wire-up integration test framework",
        "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
        "agents": [{
            "id": agent_id,
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "capabilities": {
                "tools_called": tools_called,
                "skills_loaded": [],
                "file_access": { "read": [], "write": [] },
                "network": [],
                "shell": false,
                "spawn_agents": []
            },
            "allowed_tools": tools_called,
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": tool_items,
        "skills": [],
        "session_root_agent": agent_id,
    }))
    .expect("test framework round-trips")
}

/// Build the per-tool Exec capability declaration matching the
/// `framework_loader`'s translator output for a builtin tool. Tests
/// pre-grant the enforcer with this shape so the L1 check finds a
/// matching grant.
fn exec_grant_for_builtin_tool(name: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Exec,
        resource: ResourceName::from_str(name).expect("non-empty tool name"),
        scope: CapabilityScope::Glob(GlobPattern::from_str("*").expect("non-empty glob")),
        side_effect_class: SideEffectClass::Pure,
    }
}

/// Build the SDK + receiver pair the tests drive. The runtime
/// `agent_id` matches `framework.session_root_agent` when wiring is
/// present, so tests grant the enforcer using the same id.
///
/// Sync — the `tokio::spawn` block below does its own awaits inside an
/// async block; the outer fn does not need to be async.
fn build_sdk(
    framework: Framework,
    grants: Vec<(String, CapabilityDeclaration)>,
    tier: Tier,
    script: Vec<ProviderEvent>,
    auto_resolve_hitl: bool,
) -> (AgentSdk<ScriptedProvider>, mpsc::Receiver<AgentEvent>) {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(tier);
    for (agent_id, decl) in grants {
        enforcer.grant(agent_id, decl);
    }
    let enforcer = Arc::new(enforcer);
    let framework: FrameworkRef = Arc::new(framework);
    let hitl_seam = Arc::new(HitlSeam::new());

    let provider = Arc::new(ScriptedProvider::new(script));
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
        // Background resolver: any pending HITL prompts get auto-
        // resolved with `abort` so the SDK loop continues. The
        // production prompt_id format is
        // `capability_violation:<session_uuid>` per agent_sdk.rs.
        let seam = Arc::clone(&hitl_seam);
        let prompt_id = format!("capability_violation:{}", session_id.as_string());
        tokio::spawn(async move {
            for _ in 0..600 {
                if seam.pending_len().await > 0 {
                    let _ = seam.resolve(&prompt_id, HitlChoice::new("abort")).await;
                    // Brief gap before next pending check — the SDK may
                    // not register a new await immediately after this
                    // resolve.
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });
    }

    (sdk, rx)
}

// ── Tests ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_dispatch_with_valid_grant_emits_capability_grant_and_tool_invoked() {
    // Wire-up trace #2 happy path: a framework tool dispatched + agent
    // has matching Exec grant → CapabilityGrant + ToolInvoked emit, in
    // that order. Uses "Echo" (a non-executor tool) rather than the
    // built-in "Read"/"Write" — those are owned by the M08.7.A in-process
    // executor (sdk/builtin_tools.rs) and no longer take this painted
    // pipeline path. This test pins the generic pipeline L1 contract.
    let agent_id = "worker";
    let fw = fw_with_one_agent(agent_id, &["Echo"], &[("Echo", "builtin")]);

    let (sdk, mut rx) = build_sdk(
        fw,
        vec![(agent_id.to_string(), exec_grant_for_builtin_tool("Echo"))],
        Tier::Promoted,
        vec![
            ProviderEvent::ToolUse {
                id: "tool_1".into(),
                name: "Echo".into(),
                input: json!({"path": "src/lib.rs"}),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: None,
            },
        ],
        false,
    );

    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    assert!(
        has_capability_grant_for(&events, agent_id),
        "valid grant must emit CapabilityGrant; events: {events:?}"
    );
    assert!(
        has_tool_invoked(&events),
        "approved dispatch must emit ToolInvoked; events: {events:?}"
    );
    // Ordering invariant: CapabilityGrant precedes ToolInvoked so the
    // renderer can paint the grant arc before the tool node lights.
    assert!(
        capability_grant_index(&events) < tool_invoked_index(&events).unwrap_or(usize::MAX),
        "CapabilityGrant must emit before ToolInvoked; events: {events:?}"
    );
    assert!(
        !has_capability_violation(&events),
        "approved dispatch MUST NOT emit CapabilityViolation; events: {events:?}"
    );
}

#[tokio::test]
async fn tool_dispatch_missing_grant_emits_capability_violation_and_no_tool_invoked() {
    // Wire-up trace #2 unhappy path: a framework tool dispatched + agent
    // has NO Exec grant on it → CapabilityViolation emit, ToolInvoked
    // absent. Asserts BOTH directions per gotcha #66. Uses "Echo" (a
    // non-executor tool) so it exercises the pipeline L1 denial, not the
    // M08.7.A built-in executor's file_access denial.
    let agent_id = "worker";
    let fw = fw_with_one_agent(agent_id, &["Echo"], &[("Echo", "builtin")]);

    let (sdk, mut rx) = build_sdk(
        fw,
        // No grants — default-deny path.
        vec![],
        Tier::Promoted,
        vec![
            ProviderEvent::ToolUse {
                id: "tool_1".into(),
                name: "Echo".into(),
                input: json!({"path": "src/lib.rs"}),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: None,
            },
        ],
        true,
    );

    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    assert!(
        has_capability_violation(&events),
        "missing grant must emit CapabilityViolation; events: {events:?}"
    );
    assert!(
        !has_tool_invoked(&events),
        "denied dispatch MUST NOT emit ToolInvoked; events: {events:?}"
    );
    assert!(
        !has_capability_grant_for(&events, agent_id),
        "denied dispatch MUST NOT emit CapabilityGrant; events: {events:?}"
    );
    let kind = capability_violation_kind(&events).expect("violation kind present");
    assert_eq!(kind, CapabilityKindRef::Exec);
}

#[tokio::test]
async fn tool_dispatch_with_unknown_tool_emits_capability_violation() {
    // capabilities_for_tool returns ToolNotFound when the dispatched
    // tool is not declared in framework.tools[]. The wire-up surfaces
    // this as CapabilityViolation per gotcha #66 (contract failure
    // must surface as a deniable event, not a silent skip).
    let agent_id = "worker";
    let fw = fw_with_one_agent(agent_id, &["Read"], &[("Read", "builtin")]);

    let (sdk, mut rx) = build_sdk(
        fw,
        vec![(agent_id.to_string(), exec_grant_for_builtin_tool("Read"))],
        Tier::Promoted,
        vec![
            ProviderEvent::ToolUse {
                id: "tool_1".into(),
                name: "Mystery".into(),
                input: json!({}),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: None,
            },
        ],
        true,
    );

    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    assert!(
        has_capability_violation(&events),
        "unknown-tool dispatch must emit CapabilityViolation; events: {events:?}"
    );
    assert!(
        !has_tool_invoked(&events),
        "unknown-tool dispatch MUST NOT emit ToolInvoked; events: {events:?}"
    );
}

#[tokio::test]
async fn tool_dispatch_twice_in_sequence_both_succeed_and_emit_separately() {
    // Gotcha #69 multi-call invariant: two sequential approved
    // dispatches against the same enforcer state must each emit
    // their own CapabilityGrant + ToolInvoked. First-call mutation
    // must not affect the second.
    let agent_id = "worker";
    let fw = fw_with_one_agent(agent_id, &["Echo"], &[("Echo", "builtin")]);

    let (sdk, mut rx) = build_sdk(
        fw,
        vec![(agent_id.to_string(), exec_grant_for_builtin_tool("Echo"))],
        Tier::Promoted,
        vec![
            ProviderEvent::ToolUse {
                id: "tool_1".into(),
                name: "Echo".into(),
                input: json!({"call": 1}),
            },
            ProviderEvent::ToolUse {
                id: "tool_2".into(),
                name: "Echo".into(),
                input: json!({"call": 2}),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: None,
            },
        ],
        false,
    );

    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    let grant_count = events
        .iter()
        .filter(|e| matches!(e, AgentEvent::CapabilityGrant { .. }))
        .count();
    let invoked_count = events
        .iter()
        .filter(|e| matches!(e, AgentEvent::ToolInvoked { .. }))
        .count();
    assert_eq!(
        grant_count, 2,
        "two dispatches must each emit a grant; events: {events:?}"
    );
    assert_eq!(
        invoked_count, 2,
        "two dispatches must each emit a ToolInvoked; events: {events:?}"
    );
    assert!(
        !has_capability_violation(&events),
        "approved sequence MUST NOT emit CapabilityViolation; events: {events:?}"
    );
}

// ── Helpers ──────────────────────────────────────────────────────────

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

fn has_capability_grant_for(events: &[AgentEvent], agent_id: &str) -> bool {
    events.iter().any(|e| {
        matches!(
            e,
            AgentEvent::CapabilityGrant { granted_to, .. } if granted_to == agent_id
        )
    })
}

fn has_capability_violation(events: &[AgentEvent]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, AgentEvent::CapabilityViolation { .. }))
}

fn has_tool_invoked(events: &[AgentEvent]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, AgentEvent::ToolInvoked { .. }))
}

fn capability_violation_kind(events: &[AgentEvent]) -> Option<CapabilityKindRef> {
    events.iter().find_map(|e| {
        if let AgentEvent::CapabilityViolation {
            capability_kind, ..
        } = e
        {
            Some(*capability_kind)
        } else {
            None
        }
    })
}

fn capability_grant_index(events: &[AgentEvent]) -> usize {
    events
        .iter()
        .position(|e| matches!(e, AgentEvent::CapabilityGrant { .. }))
        .unwrap_or(usize::MAX)
}

fn tool_invoked_index(events: &[AgentEvent]) -> Option<usize> {
    events
        .iter()
        .position(|e| matches!(e, AgentEvent::ToolInvoked { .. }))
}
