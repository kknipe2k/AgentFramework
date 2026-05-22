//! M08.F1 — the Tester isolated-session assembled regression.
//!
//! The v1.8 §6 assembled-app-regression mandate: the phase-doc root
//! cause — "the Tester is the production tool-driving session; running a
//! tool-bearing framework through it persists signals + a non-zero
//! `token_usage` under the test session id, isolated from any user DB" —
//! is a **falsifiable hypothesis this test must disprove**, not a
//! premise. It therefore drives the REAL `run_test_session_with` through
//! a REAL `runtime-drone` subprocess and a **concrete**
//! `runtime_mcp::McpDispatcher` (MockTransport-scripted, real
//! `CapabilityEnforcer` + `NamespaceResolver`) — NOT a mock
//! `McpToolDispatch` seam (gotcha #66). It asserts `token_usage > 0`,
//! not merely `signals > 0`.
//!
//! Harness mirrors `crates/runtime-main/tests/agent_with_tools_loop.rs`
//! (the M07.D2 real-drone + concrete-dispatcher archetype).

#![cfg(any(unix, windows))]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;
use runtime_core::generated::framework::Framework;
use runtime_main::builder::run_test_session_with;
use runtime_main::capability::CapabilityEnforcer;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{
    AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
    ProviderSupport,
};
use runtime_main::sdk::{McpToolDispatch, SessionId};
use runtime_main::tier::Tier;
use runtime_mcp::transport::{Connection, MockTransport, Transport};
use runtime_mcp::{mcp_tool_capability, ConnectionResolver, McpDispatcher, NamespaceResolver};
use rusqlite::Connection as SqliteConnection;
use serde_json::json;
use tempfile::TempDir;
use tokio::sync::RwLock;

mod common;
use common::ensure_drone_built;

// ── Real-drone-subprocess harness (mirrors agent_with_tools_loop) ─────

#[cfg(unix)]
fn make_socket(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("d.sock")
}

#[cfg(windows)]
fn make_socket(_dir: &std::path::Path) -> std::path::PathBuf {
    let suffix = uuid::Uuid::new_v4();
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-f1-{suffix}"))
}

fn spawn_drone(
    session: &str,
    db_path: &std::path::Path,
    socket: &std::path::Path,
) -> tokio::process::Child {
    let mut cmd = tokio::process::Command::new(common::drone_binary());
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
        "f1-multiturn-stub"
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

// ── Concrete-dispatcher construction (mirrors agent_with_tools_loop) ──

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
        "name": "m08-f1-tester-assembled",
        "version": "1.0.0",
        "description": "M08.F1 Tester isolated-session assembled regression",
        "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
        "agents": [{
            "id": agent_id,
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
        "session_root_agent": agent_id,
    }))
    .expect("test framework round-trips")
}

/// A concrete `McpDispatcher` whose `MockTransport`-scripted `fs` server
/// exposes `read`, granted to `agent_id` — so a `ToolUse` for `fs__read`
/// is INVOKED through the real dispatcher, not blocked.
fn build_concrete_dispatcher(agent_id: &str, session_id: &str) -> Arc<McpDispatcher> {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    enforcer.grant(agent_id.to_string(), mcp_tool_capability("fs", "read"));

    let transport = MockTransport::new()
        .with_tool("read", None, json!({ "type": "object" }))
        .with_tool_result("read", json!({ "text": "file-contents" }));

    let mut connected = BTreeMap::new();
    connected.insert("fs".to_string(), vec!["read".to_string()]);
    let resolver = Arc::new(RwLock::new(NamespaceResolver::new(connected)));

    Arc::new(McpDispatcher::new(
        resolver,
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        session_id.to_string(),
    ))
}

/// A provider that requests `fs__read` on turn 1, then reports token
/// usage and stops on turn 2 — the agent-with-tools production driver
/// (M07.V 🟡 #5) the Tester now exercises.
fn tool_using_provider() -> MultiTurnStub {
    MultiTurnStub::new(vec![
        vec![ProviderEvent::ToolUse {
            id: "tu-1".to_string(),
            name: "fs__read".to_string(),
            input: json!({ "path": "/x" }),
        }],
        vec![
            ProviderEvent::Usage {
                input_tokens: 1234,
                output_tokens: 56,
                model: "claude-haiku-4-5".to_string(),
                cost_usd: 0.0021,
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
                total_tokens: Some(1290),
            },
        ],
    ])
}

// ── The assembled regressions ────────────────────────────────────────

/// M07.V 🟡 #5 discharge. The Tester runs a tool-bearing framework
/// through `run_test_session_with` → `AgentSdk::run_agent`, dispatching a
/// real `ProviderEvent::ToolUse` through a CONCRETE `McpDispatcher` in a
/// production code path — and the run persists signals under the test
/// session id.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tester_runs_a_tool_bearing_framework_through_the_real_loop_and_persists_signals() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let socket = make_socket(dir.path());

    let session = SessionId::new();
    let sid = session.as_string();
    let mut child = spawn_drone(&sid, &db_path, &socket);
    let client = connect_with_retry(&socket.to_string_lossy()).await;

    let dispatcher = build_concrete_dispatcher("worker", &sid);

    let outcome = run_test_session_with(
        &fw_one_agent("worker"),
        "read a file and summarize it",
        &db_path,
        tool_using_provider(),
        Arc::new(client),
        Some(dispatcher as Arc<dyn McpToolDispatch>),
        session,
    )
    .await
    .expect("the assembled Tester run completes");

    poll_until(
        &db_path,
        |c| {
            c.query_row(
                "SELECT COUNT(*) FROM signals WHERE session_id = ?1",
                [sid.as_str()],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
                > 0
        },
        "the assembled Tester run persists signals under the test session id",
    )
    .await;

    assert!(
        count(&db_path, "signals", &sid) > 0,
        "the tool-bearing run must persist signals"
    );
    assert!(
        !outcome.trace.is_empty(),
        "the outcome carries the full AgentEvent trace for F2 to render"
    );

    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

/// `token_usage > 0` — the falsifiable hypothesis (gotcha #66). The
/// CONCRETE dispatcher actually ran the tool, and the run's
/// `AgentEvent::TokenUsage` both persists a `token_usage` DB row AND
/// folds into `TestOutcome::token_spend`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tester_run_persists_non_zero_token_usage_under_the_test_session_id() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let socket = make_socket(dir.path());

    let session = SessionId::new();
    let sid = session.as_string();
    let mut child = spawn_drone(&sid, &db_path, &socket);
    let client = connect_with_retry(&socket.to_string_lossy()).await;

    let dispatcher = build_concrete_dispatcher("worker", &sid);
    let outcome = run_test_session_with(
        &fw_one_agent("worker"),
        "read a file",
        &db_path,
        tool_using_provider(),
        Arc::new(client),
        Some(dispatcher as Arc<dyn McpToolDispatch>),
        session,
    )
    .await
    .expect("the assembled Tester run completes");

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
        "the assembled Tester run persists a token_usage row",
    )
    .await;

    assert!(
        count(&db_path, "token_usage", &sid) > 0,
        "token_usage > 0 — the Tester IS the production token-bearing driver"
    );
    assert!(
        outcome.token_spend.total > 0,
        "the TestOutcome folds the run's token usage; got {:?}",
        outcome.token_spend
    );

    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

/// ADR-0019 isolation. The throwaway test DB is a DISTINCT file from a
/// user session DB; the Tester run writes only to the throwaway path and
/// leaves the user DB byte-untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tester_throwaway_db_is_isolated_from_a_user_session_db() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let throwaway_db = dir.path().join("runtime-tester.sqlite");
    let user_db = dir.path().join("user-session.sqlite");
    let socket = make_socket(dir.path());

    // A pre-existing user session DB carrying a marker row.
    {
        let conn = SqliteConnection::open(&user_db).expect("open user db");
        conn.execute("CREATE TABLE marker (x INTEGER)", []).unwrap();
        conn.execute("INSERT INTO marker (x) VALUES (42)", [])
            .unwrap();
    }

    let session = SessionId::new();
    let sid = session.as_string();
    let mut child = spawn_drone(&sid, &throwaway_db, &socket);
    let client = connect_with_retry(&socket.to_string_lossy()).await;

    let dispatcher = build_concrete_dispatcher("worker", &sid);
    run_test_session_with(
        &fw_one_agent("worker"),
        "read a file",
        &throwaway_db,
        tool_using_provider(),
        Arc::new(client),
        Some(dispatcher as Arc<dyn McpToolDispatch>),
        session,
    )
    .await
    .expect("the assembled Tester run completes");

    assert_ne!(
        throwaway_db, user_db,
        "the test session DB is a distinct file from any user session DB"
    );
    // The user DB still carries ONLY its marker — the Tester never wrote
    // a `signals` table or any row into it.
    let conn = SqliteConnection::open(&user_db).expect("reopen user db");
    let marker: i64 = conn
        .query_row("SELECT x FROM marker", [], |r| r.get(0))
        .expect("the user db marker row is intact");
    assert_eq!(marker, 42, "the user DB is byte-untouched by the test run");
    let has_signals_table: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='signals'",
            [],
            |r| r.get::<_, i64>(0).map(|n| n > 0),
        )
        .unwrap_or(false);
    assert!(
        !has_signals_table,
        "the Tester wrote no signals into the user DB"
    );

    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

/// ADR-0019 teardown. After the test-session drone is reaped, the
/// throwaway DB file is no longer locked and is deleted — proving the
/// teardown order (reap drone, then remove the file) the Tauri-shell
/// `test_framework` command performs is sound on every platform
/// (Windows holds a file lock until the owning process exits).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tester_session_teardown_removes_the_throwaway_db() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let socket = make_socket(dir.path());

    let session = SessionId::new();
    let sid = session.as_string();
    let mut child = spawn_drone(&sid, &db_path, &socket);
    let client = connect_with_retry(&socket.to_string_lossy()).await;

    let dispatcher = build_concrete_dispatcher("worker", &sid);
    run_test_session_with(
        &fw_one_agent("worker"),
        "read a file",
        &db_path,
        tool_using_provider(),
        Arc::new(client),
        Some(dispatcher as Arc<dyn McpToolDispatch>),
        session,
    )
    .await
    .expect("the assembled Tester run completes");

    assert!(db_path.exists(), "the throwaway DB exists during the run");

    // Teardown: reap the drone subprocess FIRST so it releases the DB
    // lock, then delete the throwaway file.
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
    std::fs::remove_file(&db_path).expect("the throwaway DB deletes after the drone is reaped");
    assert!(
        !db_path.exists(),
        "the throwaway DB is gone after teardown — nothing persists to disk"
    );
}
