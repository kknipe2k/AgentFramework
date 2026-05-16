//! M06 Stage F — the SDK run-loop MCP-dispatch interception seam
//! (ADR-0010 + ADR-0011).
//!
//! Canonical "what the run loop actually does when an
//! `Arc<dyn McpToolDispatch>` is injected" surface. Stage D shipped the
//! `McpToolDispatch` trait + the `apply_mcp_dispatch` outcome→event
//! mapping + the concrete dispatcher (tested in `runtime-mcp`); this
//! pins the runtime-main side ADR-0011 scopes Stage F to: the
//! `AgentSdk` run loop, at the Stage A `ProviderEvent::ToolUse` site,
//! calls the injected `dispatch_if_mcp` FIRST and routes its result.
//!
//! Per ADR-0011 trace #11a: verified against a **mock**
//! `Arc<dyn McpToolDispatch>` (the concrete dispatcher's
//! resolve/check/invoke/audit behavior is the `runtime-mcp` integration
//! test's job; the concrete construction + live agent-loop exercise are
//! the ADR-0011 M07 carry-forward — trace #11b, deliberately NOT here).
//!
//! gotcha #68 (the load-bearing assertion): the run loop holds
//! `agent_id`; `McpDispatchOutcome::Invoked` does not. The success-path
//! `ToolInvoked`/`ToolResult` MUST carry a non-empty `agent_id` that
//! EQUALS the run-loop agent — never the empty string
//! `apply_mcp_dispatch`'s Invoked branch produces.
//!
//! gotcha #66: every test asserts the OBSERVABLE renderer-facing event
//! contract (agent_id-correct events on the `mpsc::Receiver`), not that
//! an internal function returned `Ok`. gotcha #69: a
//! `*_twice_in_sequence_*` multi-call invariant test.

use std::collections::{BTreeMap, VecDeque};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use runtime_core::event::{AgentEvent, ToolSource};
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
use runtime_main::sdk::{
    AgentSdk, CapabilityWiring, McpDispatchError, McpDispatchOutcome, McpToolDispatch, SessionId,
};
use runtime_main::tier::Tier;
use serde_json::{json, Value};
use tokio::sync::mpsc;

// ── Fixtures ──────────────────────────────────────────────────────────

/// In-process provider that yields a scripted `ProviderEvent` sequence.
struct ScriptedProvider {
    script: Mutex<Option<Vec<ProviderEvent>>>,
}

impl ScriptedProvider {
    const fn new(events: Vec<ProviderEvent>) -> Self {
        Self {
            script: Mutex::new(Some(events)),
        }
    }
}

#[async_trait]
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

type DispatchResult = Option<Result<McpDispatchOutcome, McpDispatchError>>;

/// Mock `McpToolDispatch`. Pops one scripted result per call and records
/// the `(agent_id, tool_name)` it was invoked with so tests can assert
/// the run loop passed its own agent id through (gotcha #68 root cause).
struct ScriptedMcpDispatch {
    results: Mutex<VecDeque<DispatchResult>>,
    seen: Mutex<Vec<(String, String)>>,
}

impl ScriptedMcpDispatch {
    fn new(results: Vec<DispatchResult>) -> Self {
        Self {
            results: Mutex::new(results.into()),
            seen: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl McpToolDispatch for ScriptedMcpDispatch {
    async fn dispatch_if_mcp(
        &self,
        agent_id: &str,
        tool_name: &str,
        _args: Value,
        _aliases: &BTreeMap<String, String>,
    ) -> DispatchResult {
        self.seen
            .lock()
            .expect("no poisoning")
            .push((agent_id.to_string(), tool_name.to_string()));
        self.results
            .lock()
            .expect("no poisoning")
            .pop_front()
            .expect("scripted dispatch result available for each call")
    }
}

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
        "description": "M06.F run-loop MCP dispatch test framework",
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

fn exec_grant_for_builtin_tool(name: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Exec,
        resource: ResourceName::from_str(name).expect("non-empty tool name"),
        scope: CapabilityScope::Glob(GlobPattern::from_str("*").expect("non-empty glob")),
        side_effect_class: SideEffectClass::Pure,
    }
}

/// Build an SDK with capability wiring (so the run-loop `agent_id` is the
/// framework's `session_root_agent`, the HITL seam exists for the
/// blocked path, and the non-MCP fall-through exercises the real Stage A
/// L1 pipeline) PLUS an injected mock `McpToolDispatch`.
fn build_sdk_with_mcp(
    framework: Framework,
    grants: Vec<(String, CapabilityDeclaration)>,
    tier: Tier,
    script: Vec<ProviderEvent>,
    dispatch: Arc<ScriptedMcpDispatch>,
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
    let sdk = AgentSdk::with_capability_wiring(provider, tx, drone, session_id.clone(), wiring)
        .with_mcp_dispatch(dispatch as Arc<dyn McpToolDispatch>);

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

// ── Tests ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn mcp_tool_use_routes_through_injected_dispatch_and_emits_agent_id_correct_events() {
    // gotcha #68 load-bearing: the run loop holds agent_id (==
    // framework.session_root_agent == "worker"); McpDispatchOutcome::
    // Invoked does not. The emitted ToolInvoked + ToolResult MUST carry
    // agent_id == "worker", non-empty — NOT the empty string
    // apply_mcp_dispatch's Invoked branch produces.
    let agent_id = "worker";
    let fw = fw_with_one_agent(agent_id, &[], &[]);
    let dispatch = Arc::new(ScriptedMcpDispatch::new(vec![Some(Ok(
        McpDispatchOutcome::Invoked {
            server: "pdf-mcp".to_string(),
            tool: "extract_text".to_string(),
            value: json!({"text": "extracted"}),
        },
    ))]));

    let (sdk, mut rx) = build_sdk_with_mcp(
        fw,
        vec![],
        Tier::Promoted,
        vec![
            ProviderEvent::ToolUse {
                id: "t1".into(),
                name: "pdf-mcp__extract_text".into(),
                input: json!({"path": "doc.pdf"}),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: None,
            },
        ],
        Arc::clone(&dispatch),
        false,
    );

    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    // The mock was actually consulted with the run-loop's agent id.
    let seen = dispatch.seen.lock().expect("no poisoning").clone();
    assert_eq!(
        seen,
        vec![(agent_id.to_string(), "pdf-mcp__extract_text".to_string())],
        "run loop must call dispatch_if_mcp with its own agent_id + the tool name"
    );

    let invoked = events
        .iter()
        .find_map(|e| match e {
            AgentEvent::ToolInvoked {
                agent_id: aid,
                tool_name,
                source,
                server,
                input,
            } => Some((aid, tool_name, source, server, input)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected a ToolInvoked; events: {events:?}"));
    assert!(
        !invoked.0.is_empty(),
        "agent_id MUST NOT be empty (gotcha #68)"
    );
    assert_eq!(
        invoked.0, agent_id,
        "agent_id MUST equal the run-loop agent, not the empty apply_mcp_dispatch value"
    );
    assert_eq!(invoked.1, "extract_text");
    assert_eq!(*invoked.2, ToolSource::Mcp);
    assert_eq!(invoked.3.as_deref(), Some("pdf-mcp"));
    assert_eq!(
        *invoked.4,
        json!({"path": "doc.pdf"}),
        "original tool args ride into ToolInvoked"
    );

    let result = events
        .iter()
        .find_map(|e| match e {
            AgentEvent::ToolResult {
                agent_id: aid,
                output,
                ..
            } => Some((aid, output)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected a ToolResult; events: {events:?}"));
    assert_eq!(
        result.0, agent_id,
        "ToolResult agent_id MUST also equal the run-loop agent (gotcha #68)"
    );
    assert_eq!(*result.1, json!({"text": "extracted"}));

    // The MCP path MUST NOT also run the non-MCP Stage A translation
    // for the same ToolUse (no double-emit).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolInvoked { source, .. } if *source == ToolSource::Builtin)),
        "MCP dispatch must not also emit the Stage A Builtin ToolInvoked; events: {events:?}"
    );
}

#[tokio::test]
async fn non_mcp_tool_use_falls_through_to_stage_a_l1_path_unchanged() {
    // dispatch_if_mcp returns None → pure fall-through: the event takes
    // the EXISTING Stage A L1 path unchanged (CapabilityGrant +
    // Builtin ToolInvoked), exactly as sdk_capability_integration.rs
    // pins it. No MCP-shaped events; no regression.
    let agent_id = "worker";
    let fw = fw_with_one_agent(agent_id, &["Read"], &[("Read", "builtin")]);
    let dispatch = Arc::new(ScriptedMcpDispatch::new(vec![None]));

    let (sdk, mut rx) = build_sdk_with_mcp(
        fw,
        vec![(agent_id.to_string(), exec_grant_for_builtin_tool("Read"))],
        Tier::Promoted,
        vec![
            ProviderEvent::ToolUse {
                id: "t1".into(),
                name: "Read".into(),
                input: json!({"path": "src/lib.rs"}),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: None,
            },
        ],
        Arc::clone(&dispatch),
        false,
    );

    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::CapabilityGrant { granted_to, .. } if granted_to == agent_id)),
        "non-MCP fall-through must still emit the Stage A CapabilityGrant; events: {events:?}"
    );
    let invoked_source = events.iter().find_map(|e| match e {
        AgentEvent::ToolInvoked { source, .. } => Some(source.clone()),
        _ => None,
    });
    assert_eq!(
        invoked_source,
        Some(ToolSource::Builtin),
        "fall-through ToolInvoked MUST keep the Stage A Builtin source, not Mcp; events: {events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolResult { .. })),
        "the Stage A non-MCP path does not synthesize a ToolResult here; events: {events:?}"
    );
}

#[tokio::test]
async fn blocked_mcp_tool_use_awaits_hitl_and_emits_dispatch_error_event() {
    // Two MCP ToolUses: (1) Blocked → CapabilityViolation +
    // McpRequestBlocked + the existing on_capability_violation HITL
    // await (ADR-0007, no new seam); (2) Some(Err(Transport)) →
    // mcp_dispatch_error_event (a ToolError carrying the cause).
    let agent_id = "worker";
    let fw = fw_with_one_agent(agent_id, &[], &[]);
    let dispatch = Arc::new(ScriptedMcpDispatch::new(vec![
        Some(Ok(McpDispatchOutcome::Blocked {
            agent_id: agent_id.to_string(),
            server: "pdf-mcp".to_string(),
            tool: "extract_text".to_string(),
            reason: "no capabilities declared".to_string(),
        })),
        Some(Err(McpDispatchError::Transport(
            "connection refused".to_string(),
        ))),
    ]));

    let (sdk, mut rx) = build_sdk_with_mcp(
        fw,
        vec![],
        Tier::Promoted,
        vec![
            ProviderEvent::ToolUse {
                id: "t1".into(),
                name: "pdf-mcp__extract_text".into(),
                input: json!({}),
            },
            ProviderEvent::ToolUse {
                id: "t2".into(),
                name: "pdf-mcp__extract_text".into(),
                input: json!({}),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: None,
            },
        ],
        Arc::clone(&dispatch),
        true,
    );

    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    let viol = events.iter().find_map(|e| match e {
        AgentEvent::CapabilityViolation { agent_id: aid, .. } => Some(aid),
        _ => None,
    });
    assert_eq!(
        viol,
        Some(&agent_id.to_string()),
        "Blocked must emit CapabilityViolation with the blocked agent_id; events: {events:?}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            AgentEvent::McpRequestBlocked { server, tool, .. }
                if server == "pdf-mcp" && tool == "extract_text"
        )),
        "Blocked must emit McpRequestBlocked attributing server+tool; events: {events:?}"
    );
    // Err path → a ToolError carrying the transport cause.
    assert!(
        events.iter().any(|e| matches!(
            e,
            AgentEvent::ToolError { agent_id: aid, error, .. }
                if aid == agent_id && error.contains("connection refused")
        )),
        "Some(Err) must emit a ToolError dispatch-error event; events: {events:?}"
    );
    // The blocked dispatch MUST NOT emit a success ToolInvoked.
    assert!(
        !events.iter().any(
            |e| matches!(e, AgentEvent::ToolInvoked { source, .. } if *source == ToolSource::Mcp)
        ),
        "a blocked MCP dispatch MUST NOT emit an Mcp ToolInvoked; events: {events:?}"
    );
}

#[tokio::test]
async fn mcp_tool_use_twice_in_sequence_both_emit_correct_events() {
    // gotcha #69 multi-call invariant: two sequential injected-dispatch
    // Invoked outcomes must EACH emit their own agent_id-correct
    // ToolInvoked + ToolResult; the first call's state must not bleed
    // into the second (distinct inputs + distinct values ride
    // independently).
    let agent_id = "worker";
    let fw = fw_with_one_agent(agent_id, &[], &[]);
    let dispatch = Arc::new(ScriptedMcpDispatch::new(vec![
        Some(Ok(McpDispatchOutcome::Invoked {
            server: "pdf-mcp".to_string(),
            tool: "extract_text".to_string(),
            value: json!({"call": 1}),
        })),
        Some(Ok(McpDispatchOutcome::Invoked {
            server: "pdf-mcp".to_string(),
            tool: "extract_text".to_string(),
            value: json!({"call": 2}),
        })),
    ]));

    let (sdk, mut rx) = build_sdk_with_mcp(
        fw,
        vec![],
        Tier::Promoted,
        vec![
            ProviderEvent::ToolUse {
                id: "t1".into(),
                name: "pdf-mcp__extract_text".into(),
                input: json!({"call": 1}),
            },
            ProviderEvent::ToolUse {
                id: "t2".into(),
                name: "pdf-mcp__extract_text".into(),
                input: json!({"call": 2}),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: None,
            },
        ],
        Arc::clone(&dispatch),
        false,
    );

    let task = tokio::spawn(async move { sdk.run_agent(default_config()).await });
    let events = drain_events(&mut rx).await;
    let () = task.await.expect("join sdk").expect("run_agent ok");

    let invoked: Vec<&AgentEvent> = events
        .iter()
        .filter(|e| matches!(e, AgentEvent::ToolInvoked { .. }))
        .collect();
    let results: Vec<&Value> = events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::ToolResult { output, .. } => Some(output),
            _ => None,
        })
        .collect();
    assert_eq!(invoked.len(), 2, "two MCP dispatches → two ToolInvoked");
    assert_eq!(results.len(), 2, "two MCP dispatches → two ToolResult");
    for e in &invoked {
        if let AgentEvent::ToolInvoked {
            agent_id: aid,
            source,
            ..
        } = e
        {
            assert_eq!(aid, agent_id, "each ToolInvoked agent_id correct (#68)");
            assert_eq!(*source, ToolSource::Mcp);
        }
    }
    assert_eq!(*results[0], json!({"call": 1}));
    assert_eq!(
        *results[1],
        json!({"call": 2}),
        "second call's value must be independent of the first"
    );
}
