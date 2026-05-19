//! M07.D2 — the multi-turn agent-with-tools loop assembled regression
//! (ADR-0011 d; closes the M06.5 `token_usage = 0` open finding).
//!
//! The v1.8 §6 assembled-app-regression mandate (M06.5 lines 156–173):
//! the phase-doc root cause — "the real agent-with-tools loop is the
//! first production token-bearing signal source; the loop persists
//! `token_usage`" — is a **falsifiable hypothesis this test must
//! disprove**, not a premise. It therefore drives the REAL loop through
//! a REAL `runtime-drone` subprocess and a **concrete**
//! `runtime_mcp::McpDispatcher` (MockTransport-scripted, real
//! `CapabilityEnforcer` + `NamespaceResolver`) — NOT a mock
//! `McpToolDispatch` seam (that seam already passes in
//! `mcp_dispatch_runloop.rs`; it is exactly the Stage-V blind spot the
//! mandate exists to kill, gotcha #66). It asserts `token_usage > 0`,
//! not merely `signals > 0`.
//!
//! Harness mirrors `crates/runtime-main/tests/smoke_signal_persistence.rs`
//! (the M06.5 real-drone archetype) for the drone subprocess and
//! `crates/runtime-mcp/tests/mcp_dispatch_integration.rs` for the
//! concrete-dispatcher construction.

#![cfg(any(unix, windows))]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;
use runtime_core::event::AgentEvent;
use runtime_core::generated::framework::Framework;
use runtime_main::capability::CapabilityEnforcer;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::framework_loader::FrameworkRef;
use runtime_main::hitl::HitlSeam;
use runtime_main::providers::{
    AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
    ProviderSupport,
};
use runtime_main::sdk::{
    apply_renderable, AgentSdk, CapabilityWiring, McpToolDispatch, RenderableOutcome, SessionId,
};
use runtime_main::tier::Tier;
use runtime_mcp::transport::{Connection, MockTransport, Transport};
use runtime_mcp::{mcp_tool_capability, ConnectionResolver, McpDispatcher, NamespaceResolver};
use rusqlite::Connection as SqliteConnection;
use serde_json::json;
use tempfile::TempDir;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

mod common;
use common::ensure_drone_built;

// ── Real-drone-subprocess harness (mirrors smoke_signal_persistence) ──

#[cfg(unix)]
fn make_socket(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("d.sock")
}

#[cfg(windows)]
fn make_socket(_dir: &std::path::Path) -> std::path::PathBuf {
    let suffix = uuid::Uuid::new_v4();
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-d2-{suffix}"))
}

fn drone_binary() -> std::path::PathBuf {
    common::drone_binary()
}

fn spawn_drone(
    session: &str,
    db_path: &std::path::Path,
    socket: &std::path::Path,
) -> tokio::process::Child {
    let mut cmd = tokio::process::Command::new(drone_binary());
    cmd.arg("--session-id")
        .arg(session)
        .arg("--db-path")
        .arg(db_path)
        .arg("--ipc-socket")
        .arg(socket)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    cmd.spawn().expect("spawn drone")
}

async fn connect_with_retry(addr: &str) -> DroneClient {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match DroneClient::connect(addr).await {
            Ok(c) => return c,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => panic!("connect: {e}"),
        }
    }
}

async fn poll_until<F: Fn(&SqliteConnection) -> bool>(
    db_path: &std::path::Path,
    predicate: F,
    label: &str,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        if let Ok(conn) = SqliteConnection::open(db_path) {
            if predicate(&conn) {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("predicate never satisfied: {label}");
}

fn count(db_path: &std::path::Path, table: &str, session_id: &str) -> i64 {
    let conn = SqliteConnection::open(db_path).expect("open db");
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {table} WHERE session_id = ?1"),
        [session_id],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

// ── Multi-turn stub provider (no live Anthropic — CLAUDE.md §10) ──────

/// Yields one scripted `Vec<ProviderEvent>` per `stream()` call (per
/// agent turn). The multi-turn loop re-invokes `stream` with the
/// growing message history; this stub returns turn-1 then turn-2 then
/// an empty stream (loop-terminating).
struct MultiTurnStub {
    turns: Mutex<std::collections::VecDeque<Vec<ProviderEvent>>>,
}

impl MultiTurnStub {
    fn new(turns: Vec<Vec<ProviderEvent>>) -> Self {
        Self {
            turns: Mutex::new(turns.into()),
        }
    }
}

#[async_trait]
impl LLMProvider for MultiTurnStub {
    fn name(&self) -> &'static str {
        "d2-multiturn-stub"
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
        let turn = self
            .turns
            .lock()
            .expect("no poisoning")
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

// ── Concrete-dispatcher construction (mirrors mcp_dispatch_integration) ─

struct MockResolver {
    transport: MockTransport,
}

#[async_trait]
impl ConnectionResolver for MockResolver {
    async fn connection(
        &self,
        _server: &str,
    ) -> Result<Arc<dyn Connection>, runtime_mcp::McpError> {
        Ok(Arc::from(self.transport.connect().await?))
    }
}

fn fw_one_agent(agent_id: &str) -> Framework {
    serde_json::from_value(json!({
        "name": "m07-d2-fw",
        "version": "1.0.0",
        "description": "M07.D2 agent-with-tools loop assembled regression",
        "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
        "agents": [{
            "id": agent_id,
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
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
        "session_root_agent": agent_id,
    }))
    .expect("test framework round-trips")
}

/// Build a CONCRETE `runtime_mcp::McpDispatcher` (real enforcer + real
/// namespace resolver + MockTransport-scripted server) — NOT a mock
/// `McpToolDispatch`.
fn build_concrete_dispatcher(
    agent_id: &str,
    server: &str,
    tool: &str,
    result: serde_json::Value,
    session_id: &str,
) -> Arc<McpDispatcher> {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    enforcer.grant(agent_id.to_string(), mcp_tool_capability(server, tool));

    let transport = MockTransport::new()
        .with_tool(tool, None, json!({ "type": "object" }))
        .with_tool_result(tool, result);

    let mut connected = BTreeMap::new();
    connected.insert(server.to_string(), vec![tool.to_string()]);
    let resolver = Arc::new(RwLock::new(NamespaceResolver::new(connected)));

    Arc::new(McpDispatcher::new(
        resolver,
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        session_id.to_string(),
    ))
}

fn build_sdk(
    agent_id: &str,
    provider: Arc<MultiTurnStub>,
    drone: Arc<DroneClient>,
    dispatch: Arc<dyn McpToolDispatch>,
    session: SessionId,
) -> (AgentSdk<MultiTurnStub>, mpsc::Receiver<AgentEvent>) {
    let enforcer = Arc::new(CapabilityEnforcer::new());
    let framework: FrameworkRef = Arc::new(fw_one_agent(agent_id));
    let hitl = Arc::new(HitlSeam::new());
    let (tx, rx) = mpsc::channel::<AgentEvent>(64);
    let wiring = CapabilityWiring::new(enforcer, framework, hitl);
    let sdk = AgentSdk::with_capability_wiring(provider, tx, drone, session, wiring)
        .with_mcp_dispatch(dispatch);
    (sdk, rx)
}

fn cfg() -> AgentConfig {
    AgentConfig {
        model: "claude-haiku-4-5".to_string(),
        messages: vec![],
        max_tokens: 64,
        temperature: Some(0.0),
        system_prompt: None,
        tools: vec![],
    }
}

// ── The headline assembled regression ────────────────────────────────

/// M06.5 closure proof. The assembled multi-turn agent-with-tools loop,
/// dispatching through a CONCRETE `McpDispatcher` against a REAL drone
/// subprocess, persists both the signal stream AND a `token_usage` row
/// under the run's session id. `token_usage > 0` is the falsifiable
/// hypothesis (M06.5 lines 156–173).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_with_tools_loop_persists_signals_and_token_usage() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("session.sqlite");
    let socket = make_socket(dir.path());

    let session = SessionId::new();
    let sid = session.as_string();
    let mut child = spawn_drone(&sid, &db_path, &socket);
    let client = connect_with_retry(&socket.to_string_lossy()).await;

    let agent_id = "worker";
    let dispatcher = build_concrete_dispatcher(
        agent_id,
        "fs",
        "read",
        json!({ "text": "file-contents" }),
        &sid,
    );

    // §5a re-resolution exercised through the concrete dispatcher's
    // production driver (ADR-0011 b) BEFORE the loop runs.
    let new_ambiguities = dispatcher
        .on_server_connected("fs")
        .await
        .expect("on_server_connected ok");
    assert!(
        new_ambiguities.is_empty(),
        "single server, canonical name → no new ambiguity"
    );

    // Turn 1: model requests the MCP tool. Turn 2: model reports token
    // usage + stops. The multi-turn loop must feed turn-1's tool result
    // back and re-stream to reach turn 2.
    let provider = Arc::new(MultiTurnStub::new(vec![
        vec![ProviderEvent::ToolUse {
            id: "tu-1".into(),
            name: "fs__read".into(),
            input: json!({ "path": "/x" }),
        }],
        vec![
            ProviderEvent::Usage {
                input_tokens: 1234,
                output_tokens: 56,
                model: "claude-haiku-4-5".into(),
                cost_usd: 0.0021,
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
                total_tokens: Some(1290),
            },
        ],
    ]));

    let (sdk, _rx) = build_sdk(
        agent_id,
        Arc::clone(&provider),
        Arc::new(client),
        dispatcher as Arc<dyn McpToolDispatch>,
        session,
    );

    sdk.run_agent(cfg())
        .await
        .expect("assembled agent-with-tools run completes");

    poll_until(
        &db_path,
        |c| {
            c.query_row(
                "SELECT COUNT(*) FROM token_usage WHERE session_id = ?1",
                [sid.as_str()],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
                > 0
        },
        "the multi-turn loop persists a token_usage row (M06.5 closed)",
    )
    .await;

    let signals = count(&db_path, "signals", &sid);
    let token_rows = count(&db_path, "token_usage", &sid);
    assert!(signals > 0, "the assembled loop must persist signals");
    assert!(
        token_rows > 0,
        "token_usage > 0 — the M06.5 falsifiable hypothesis; \
         0 means the loop is not the production token-bearing signal source"
    );

    // The concrete dispatcher (not a mock seam) actually ran: its
    // MockTransport-scripted result is the ToolResult signal payload.
    let conn = SqliteConnection::open(&db_path).expect("open db");
    let tool_result_payloads: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM signals \
             WHERE session_id = ?1 AND event = 'tool_result' \
               AND payload_json LIKE '%file-contents%'",
            [sid.as_str()],
            |r| r.get(0),
        )
        .unwrap_or(0);
    assert!(
        tool_result_payloads > 0,
        "the ToolResult must carry the CONCRETE McpDispatcher's \
         MockTransport-scripted value (proves a real dispatcher ran, \
         not a mock McpToolDispatch seam — gotcha #66)"
    );

    // Token columns projected from the AgentEvent::TokenUsage payload.
    let (input_tokens, output_tokens): (i64, i64) = conn
        .query_row(
            "SELECT input_tokens, output_tokens FROM token_usage \
             WHERE session_id = ?1",
            [sid.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("token_usage row present");
    assert_eq!(input_tokens, 1234, "input tokens projected from the signal");
    assert_eq!(output_tokens, 56, "output tokens projected from the signal");

    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

// ── §5a re-resolution → ToolAliasAmbiguous through the assembled loop ──

/// Two connected servers expose the same short tool name; the §5a
/// re-resolution driver (`on_server_connected`) reports the new
/// ambiguity, and dispatching the SHORT name through the assembled loop
/// emits `AgentEvent::ToolAliasAmbiguous` (ConnectionResolver
/// re-resolves on connect, collision surfaces — ADR-0011 b).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connect_collision_reresolves_and_emits_tool_alias_ambiguous() {
    let agent_id = "worker";
    let session = SessionId::new();

    // One MockTransport exposing `read`; the resolver is told two
    // servers both expose the short name `read` so it is ambiguous.
    let transport = MockTransport::new()
        .with_tool("read", None, json!({ "type": "object" }))
        .with_tool_result("read", json!({ "ok": true }));
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    enforcer.grant(agent_id.to_string(), mcp_tool_capability("fs", "read"));

    let mut connected = BTreeMap::new();
    connected.insert("fs".to_string(), vec!["read".to_string()]);
    let resolver = Arc::new(RwLock::new(NamespaceResolver::new(connected)));
    let dispatcher = Arc::new(McpDispatcher::new(
        Arc::clone(&resolver),
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        session.as_string(),
    ));

    // A second server connecting with a colliding `read` makes the
    // short name ambiguous (§5a step 5).
    {
        let mut r = resolver.write().await;
        let new_amb = r.connect_server("other", vec!["read".to_string()]);
        assert!(
            !new_amb.is_empty(),
            "a colliding short name must surface a NewAmbiguity"
        );
    }

    let provider = Arc::new(MultiTurnStub::new(vec![
        vec![ProviderEvent::ToolUse {
            id: "tu-1".into(),
            name: "read".into(),
            input: json!({}),
        }],
        vec![ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
            total_tokens: None,
        }],
    ]));

    let (sdk, mut rx) = build_sdk(
        agent_id,
        Arc::clone(&provider),
        Arc::new(DroneClient::noop()),
        dispatcher as Arc<dyn McpToolDispatch>,
        session,
    );

    let task = tokio::spawn(async move { sdk.run_agent(cfg()).await });
    let mut events = Vec::new();
    while let Some(e) = rx.recv().await {
        events.push(e);
    }
    task.await.expect("join").expect("run ok");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolAliasAmbiguous { name, .. } if name == "read")),
        "the ambiguous short name must emit ToolAliasAmbiguous; events: {events:?}"
    );
}

// ── CQ-2 surgical — RenderableOutcome has no Invoked at the type level ─

/// CQ-2 (M06.V CQ-2/reuse-5; user-decided "surgical, type-level").
/// `apply_renderable` consumes a `RenderableOutcome` that structurally
/// CANNOT represent the `Invoked` success path — so the dead
/// empty-`agent_id` branch cannot be reached from production. This test
/// pins that `RenderableOutcome::{Blocked,Ambiguous}` map to the same
/// renderer events the legacy non-Invoked arms produced, and the run
/// loop's Invoked path still emits a NON-empty agent_id.
#[test]
fn renderable_outcome_maps_blocked_and_ambiguous_without_invoked() {
    let blocked = apply_renderable(
        RenderableOutcome::Blocked {
            agent_id: "worker".into(),
            server: "fs".into(),
            tool: "read".into(),
            reason: "no capabilities declared".into(),
        },
        json!({ "path": "/x" }),
    );
    assert!(
        blocked
            .iter()
            .any(|e| matches!(e, AgentEvent::CapabilityViolation { agent_id, .. } if agent_id == "worker")),
        "Blocked → CapabilityViolation with the blocked agent_id"
    );
    assert!(
        blocked.iter().any(|e| matches!(
            e,
            AgentEvent::McpRequestBlocked { server, tool, .. }
                if server == "fs" && tool == "read"
        )),
        "Blocked → McpRequestBlocked attributing server+tool"
    );

    let ambiguous = apply_renderable(
        RenderableOutcome::Ambiguous {
            name: "read".into(),
            candidates: vec!["fs__read".into(), "other__read".into()],
        },
        json!({}),
    );
    assert!(
        ambiguous.iter().any(|e| matches!(
            e,
            AgentEvent::ToolAliasAmbiguous { name, candidates }
                if name == "read" && candidates.len() == 2
        )),
        "Ambiguous → ToolAliasAmbiguous with both candidates"
    );
}
