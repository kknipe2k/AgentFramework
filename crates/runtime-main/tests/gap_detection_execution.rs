//! M08.7.D rung 4 — gap detection (`request_capability`) assembled
//! regression + the suspend contract.
//!
//! The cluster-gate close contract (`docs/cluster-pattern.md` §1/§4): the
//! assembled tests drive the REAL `run_test_session_with` →
//! `AgentSdk::run_agent` multi-turn loop. The ONLY stub is the provider
//! (no live Anthropic — CLAUDE.md §10); the `drive_stream` interception,
//! the reused `handle_request_capability` handler, the `*Missing` gap
//! emission, and the suspend wire are all real.
//!
//! Grounded-claims (CLAUDE.md §4 rule 11 / gotcha #66): a `ToolMissing`
//! event firing licenses ONLY "the gap event fired" — NOT "the session
//! suspended cleanly." So these tests assert the SUSPEND behavior (exactly
//! one provider turn — no re-stream; the run completes Ok; the gap was NOT
//! treated as an ordinary tool), not the event emission alone. The
//! BEHAVIORAL close — a real Anthropic model calling `request_capability`
//! for a missing tool → a clean suspend — is the IRL gate (maintainer-run),
//! because the scripted stub is NOT the model.
//!
//! Scope (M08.7.D): the v0.1 outcome is Pending → suspend → recoverable
//! (suspend-and-record, ADR-0019), NOT the full grant/install/decline
//! resolution UI (resolve-and-resume is NOT exercised — M08.6.7/M09).

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::{json, Value};
use tempfile::TempDir;

use runtime_core::event::{AgentEvent, GapSeverityRef, GapSourceRef};
use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{
    AgentConfig, ContentBlock, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError,
    ProviderEvent, ProviderSupport, ToolResultContent,
};
use runtime_main::sdk::{McpDispatchError, McpDispatchOutcome, McpToolDispatch, SessionId};

// ── helpers ───────────────────────────────────────────────────────────

/// A schema-valid one-agent framework. `session_root_agent` is `worker`,
/// so the run's dispatch agent id is `worker`. The agent declares no
/// `allowed_tools` — `request_capability` is the runtime-auto-injected
/// meta-tool (spec §4b "auto-injects into every agent's tool list"), not a
/// declared tool, and the wire intercepts it by name regardless of
/// `allowed_tools`.
fn fw_one_agent() -> Framework {
    serde_json::from_value(json!({
        "name": "m08-7-d-rung4",
        "version": "1.0.0",
        "description": "M08.7.D rung 4 gap-detection fixture",
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
    .expect("the rung-4 fixture framework round-trips through the schema")
}

/// One `request_capability` `ToolUse` carrying the spec §4b input shape
/// `{capability_name, capability_kind, reason}`.
fn request_capability_tooluse(id: &str, kind: &str, name: &str, reason: &str) -> ProviderEvent {
    ProviderEvent::ToolUse {
        id: id.to_string(),
        name: "request_capability".to_string(),
        input: json!({
            "capability_kind": kind,
            "capability_name": name,
            "reason": reason,
        }),
    }
}

/// The `(agent_id, name, severity, source)` of the first `*Missing` gap of
/// any kind in a trace — so an assertion can pin both the kind AND the
/// payload.
fn first_gap(
    trace: &[AgentEvent],
) -> Option<(&'static str, String, String, GapSeverityRef, GapSourceRef)> {
    trace.iter().find_map(|e| match e {
        AgentEvent::ToolMissing {
            agent_id,
            tool_name,
            severity,
            requested_via,
            ..
        } => Some((
            "tool",
            agent_id.clone(),
            tool_name.clone(),
            *severity,
            *requested_via,
        )),
        AgentEvent::SkillMissing {
            agent_id,
            skill_name,
            severity,
            requested_via,
            ..
        } => Some((
            "skill",
            agent_id.clone(),
            skill_name.clone(),
            *severity,
            *requested_via,
        )),
        _ => None,
    })
}

// ── provider stub (no live Anthropic) ─────────────────────────────────

/// Yields one scripted `Vec<ProviderEvent>` per `stream()` call (empty —
/// loop-terminating — once exhausted) and CAPTURES every config it is
/// handed, so a test can observe how many provider turns the loop issued
/// (the suspend signal: a suspended session issues exactly one turn).
struct GapScriptStub {
    turns: Mutex<VecDeque<Vec<ProviderEvent>>>,
    seen: Arc<Mutex<Vec<AgentConfig>>>,
}

impl GapScriptStub {
    fn new(turns: Vec<Vec<ProviderEvent>>, seen: Arc<Mutex<Vec<AgentConfig>>>) -> Self {
        Self {
            turns: Mutex::new(turns.into()),
            seen,
        }
    }
}

#[async_trait]
impl LLMProvider for GapScriptStub {
    fn name(&self) -> &'static str {
        "m08-7-d-gap-script-stub"
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
        self.seen.lock().expect("seen lock").push(config);
        let turn = self
            .turns
            .lock()
            .expect("turns lock")
            .pop_front()
            .unwrap_or_default();
        Ok(Box::pin(futures::stream::iter(turn)))
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

/// An `McpToolDispatch` that resolves `echo` to an `Invoked` outcome (so it
/// joins `feedback.dispatched` and WOULD drive a 2nd turn) and lets every
/// other tool name fall through (`None`) — used to prove the gap suspend
/// short-circuits the feed-back even when the same turn dispatched a tool.
struct EchoDispatch;

#[async_trait]
impl McpToolDispatch for EchoDispatch {
    async fn dispatch_if_mcp(
        &self,
        _agent_id: &str,
        tool_name: &str,
        _args: Value,
        _aliases: &BTreeMap<String, String>,
    ) -> Option<Result<McpDispatchOutcome, McpDispatchError>> {
        if tool_name == "echo" {
            Some(Ok(McpDispatchOutcome::Invoked {
                server: "echo-srv".to_string(),
                tool: "echo".to_string(),
                value: json!({ "echoed": true }),
            }))
        } else {
            // Not an MCP tool — fall through to the request_capability branch.
            None
        }
    }
}

// ── assembled regressions (the D.4.5 BDD close contract) ──────────────

/// D.4.1 — the BDD close contract. A `request_capability` for an unheld
/// tool raises a `ToolMissing` gap (`requested_via=request_capability`) AND
/// suspends the session: the loop issues no further provider turn, the run
/// completes cleanly (no crash/Err), and the meta-tool was NOT treated as
/// an ordinary tool (no `ToolInvoked` for it — it routed to the gap
/// handler, not `pipeline.next_event`).
#[tokio::test]
async fn request_capability_for_unheld_tool_raises_gap_and_suspends() {
    let fw = fw_one_agent();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = GapScriptStub::new(
        vec![vec![request_capability_tooluse(
            "tu-1",
            "tool",
            "deploy",
            "I need to ship the build but have no deploy tool",
        )]],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "ship the build",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled rung-4 run completes — a suspend is not an Err");

    // (1) the gap fired: ToolMissing(deploy), requested_via=request_capability,
    // severity=Requested, agent_id=worker (gotcha #68 — the requesting agent).
    assert_eq!(
        first_gap(&outcome.trace),
        Some((
            "tool",
            "worker".to_string(),
            "deploy".to_string(),
            GapSeverityRef::Requested,
            GapSourceRef::RequestCapability
        )),
        "a ToolMissing(deploy) gap with requested_via=request_capability must be emitted; trace={:?}",
        outcome.trace
    );

    let configs: Vec<AgentConfig> = seen.lock().expect("seen lock").clone();

    // request_capability is auto-advertised in the agent's tools (spec §4b
    // — the runtime injects it into every agent's tool list).
    assert!(
        configs[0]
            .tools
            .iter()
            .any(|t| t.name == "request_capability"),
        "request_capability must be auto-advertised in the agent's tools; got {:?}",
        configs[0].tools.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // (2) LOAD-BEARING (rule 11 / gotcha #66): the session SUSPENDED — the
    // loop issued exactly one provider turn (no re-stream past the gap).
    assert_eq!(
        configs.len(),
        1,
        "the session must suspend at the gap — no further provider turn; got {} turn(s)",
        configs.len()
    );

    // (3) it routed to the gap handler, NOT pipeline.next_event: there is
    // no `ToolInvoked` for request_capability (the painted behavior treated
    // it as an unknown tool and emitted ToolInvoked).
    assert!(
        !outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::ToolInvoked { tool_name, .. } if tool_name == "request_capability"
        )),
        "request_capability must route to the gap handler, never be invoked as a tool; trace={:?}",
        outcome.trace
    );
}

/// D.4.3 (the load-bearing suspend / mutation-gate target) — the gap
/// suspend short-circuits the multi-turn feed-back EVEN when the same turn
/// dispatched a real tool. Turn 0 dispatches an MCP `echo` (which joins
/// `feedback.dispatched` and would, on its own, drive a 2nd turn) AND calls
/// `request_capability`. The session must still suspend: exactly one
/// provider turn. Without the suspend break, the echo result would feed
/// back and the loop would re-stream a 2nd turn — this test is what makes
/// deleting the break fail (cluster-pattern §5 blocking mutation gate).
#[tokio::test]
async fn request_capability_suspends_even_when_the_turn_also_dispatched_a_tool() {
    let fw = fw_one_agent();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = GapScriptStub::new(
        vec![vec![
            ProviderEvent::ToolUse {
                id: "tu-echo".to_string(),
                name: "echo".to_string(),
                input: json!({ "msg": "hi" }),
            },
            request_capability_tooluse("tu-gap", "tool", "deploy", "still need deploy"),
        ]],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "echo then request a capability",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        Some(Arc::new(EchoDispatch) as Arc<dyn McpToolDispatch>),
        SessionId::new(),
    )
    .await
    .expect("the assembled run completes");

    // The gap fired despite the co-dispatched echo.
    assert!(
        outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::ToolMissing { tool_name, requested_via, .. }
                if tool_name == "deploy" && *requested_via == GapSourceRef::RequestCapability
        )),
        "the ToolMissing gap must fire even when the turn also dispatched a tool; trace={:?}",
        outcome.trace
    );

    // The session SUSPENDED: exactly one provider turn, despite the echo
    // having joined feedback.dispatched. The suspend break wins over the
    // feed-back — the session does not silently continue past the gap.
    let configs: Vec<AgentConfig> = seen.lock().expect("seen lock").clone();
    assert_eq!(
        configs.len(),
        1,
        "the gap suspend must short-circuit the feed-back — no 2nd turn; got {} turn(s)",
        configs.len()
    );
}

/// D.4.4 (kind-routing mutant) — a `skill`-kind `request_capability` routes
/// to a `SkillMissing` gap (not `ToolMissing`). With D.4.1's tool-kind case,
/// this pins the `capability_kind` parse + the handler's `match kind` arms:
/// a mutant that maps `skill`→`Tool` (or `tool`→`Skill`) fails one of the
/// two tests.
#[tokio::test]
async fn skill_kind_request_capability_emits_skill_missing() {
    let fw = fw_one_agent();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = GapScriptStub::new(
        vec![vec![request_capability_tooluse(
            "tu-1",
            "skill",
            "rag",
            "I need retrieval context",
        )]],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "retrieve context",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled run completes");

    assert_eq!(
        first_gap(&outcome.trace),
        Some((
            "skill",
            "worker".to_string(),
            "rag".to_string(),
            GapSeverityRef::Requested,
            GapSourceRef::RequestCapability
        )),
        "a skill-kind request_capability must emit SkillMissing(rag), not ToolMissing; trace={:?}",
        outcome.trace
    );
}

// ── additive coverage (v1.8 follow-up — net-new tests, separate from the
//    red→impl pair) — the remaining capability_kind arms + the malformed
//    (no-suspend) path ──────────────────────────────────────────────────

/// Additive — the `mcp` kind arm of `parse_capability_kind` + the handler's
/// `match kind`: routes to `McpMissing`. Without this, deleting the `"mcp"`
/// arm (→ falling to the `tool` default) survives, since no other test
/// sends `mcp`.
#[tokio::test]
async fn mcp_kind_request_capability_emits_mcp_missing() {
    let fw = fw_one_agent();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = GapScriptStub::new(
        vec![vec![request_capability_tooluse(
            "tu-1",
            "mcp",
            "pdf-mcp",
            "I need to parse a PDF",
        )]],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "parse a pdf",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled run completes");

    assert!(
        outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::McpMissing { server_name, requested_via, .. }
                if server_name == "pdf-mcp" && *requested_via == GapSourceRef::RequestCapability
        )),
        "an mcp-kind request_capability must emit McpMissing(pdf-mcp); trace={:?}",
        outcome.trace
    );
}

/// Additive — the `agent` kind arm: routes to `AgentMissing`. Pins the
/// fourth `match kind` arm (the mutation-gate completion for the kind set).
#[tokio::test]
async fn agent_kind_request_capability_emits_agent_missing() {
    let fw = fw_one_agent();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = GapScriptStub::new(
        vec![vec![request_capability_tooluse(
            "tu-1",
            "agent",
            "report-writer",
            "I need a sub-agent to draft the report",
        )]],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "draft a report",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled run completes");

    assert!(
        outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::AgentMissing { missing_agent_id, requested_via, .. }
                if missing_agent_id == "report-writer"
                    && *requested_via == GapSourceRef::RequestCapability
        )),
        "an agent-kind request_capability must emit AgentMissing(report-writer); trace={:?}",
        outcome.trace
    );
}

/// Additive — the malformed path: a `request_capability` with an EMPTY
/// `capability_name` is refused by the handler (it cannot construct a gap),
/// so the wire feeds an error `tool_result` back and the session does NOT
/// suspend — the model can recover and the loop continues to a 2nd turn.
/// Pins the `Continue` disposition (a mutant that suspends on the Err arm,
/// or drops the error feed-back, fails here).
#[tokio::test]
async fn malformed_request_capability_feeds_error_and_does_not_suspend() {
    let fw = fw_one_agent();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = GapScriptStub::new(
        vec![
            // Turn 0: a malformed request_capability (empty capability_name).
            vec![request_capability_tooluse(
                "tu-1",
                "tool",
                "",
                "missing the name",
            )],
            // Turn 1: the model, having received the error, stops cleanly.
            vec![ProviderEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
                total_tokens: None,
            }],
        ],
        Arc::clone(&seen),
    );

    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "request a capability badly",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled run completes");

    // No gap was emitted — an unconstructable request is not a gap.
    assert!(
        !outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::ToolMissing { .. }
                | AgentEvent::SkillMissing { .. }
                | AgentEvent::McpMissing { .. }
                | AgentEvent::AgentMissing { .. }
        )),
        "a malformed request_capability must NOT emit a gap; trace={:?}",
        outcome.trace
    );

    // The session did NOT suspend — the loop continued to a 2nd turn (the
    // error tool_result was fed back so the model can recover).
    let configs: Vec<AgentConfig> = seen.lock().expect("seen lock").clone();
    assert_eq!(
        configs.len(),
        2,
        "a malformed request must NOT suspend — the loop continues; got {} turn(s)",
        configs.len()
    );

    // Turn 2 carries an error tool_result for the request_capability call.
    let fed_back = configs[1]
        .messages
        .iter()
        .flat_map(|m| m.content.iter())
        .find_map(|b| match b {
            ContentBlock::ToolResult {
                content: ToolResultContent::Text(t),
                ..
            } => Some(t.clone()),
            _ => None,
        })
        .unwrap_or_default();
    assert!(
        fed_back.contains("error"),
        "the malformed request must feed an error tool_result back; got {fed_back:?}"
    );
}
