//! Assembled smoke→drone signal-persistence regression — M06.5 Stage
//! B.fix (closes `docs/M06-irl-findings.md` 🔴-2).
//!
//! IRL ground truth: after a smoke run the live `session.sqlite` had
//! `signals = 0` while `heartbeats`/`snapshots` populated the same DB.
//! Two necessary conditions (the phase doc diagnosed only the first):
//! (1) missing emission — `emit` (`agent_sdk.rs`) only did
//! `event_tx.send`, never `write_signal`; (2) session-id mismatch —
//! `signals.session_id` is a FK into `sessions(id)` and the drone
//! seeds one `sessions` row = its `--session-id`, so the SDK's
//! independent `SessionId::new()` was silently FK-rejected. The B.fix
//! shares the drone's seeded id with the SDK; these tests model that
//! corrected composition (drone spawned with the SDK's session id).
//!
//! The DISCRIMINATOR vs the existing-green
//! `crates/runtime-main/tests/recovery_lifecycle.rs` (the Stage-V blind
//! spot): that test proves the plumbing by calling `client.write_signal()`
//! MANUALLY. These tests drive the ASSEMBLED SDK run loop
//! (`AgentSdk::run_agent` — the exact path `run_smoke_session_with`
//! wraps with zero added logic, `src-tauri/src/commands.rs:165-172`;
//! a `runtime-main` integration test cannot depend on the `src-tauri`
//! binary crate without a dependency cycle, so the faithful in-crate
//! equivalent is `AgentSdk::run_agent` itself) against a real drone
//! subprocess and assert the rows land via the SMOKE PATH.
//!
//! Harness mirrors `recovery_lifecycle.rs`
//! (`ensure_drone_built`/`spawn_drone`/`poll_until`) which already runs
//! cross-platform in CI.

#![cfg(any(unix, windows))]

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;
use runtime_core::event::AgentEvent;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{
    AgentConfig, ContentBlock, CostBreakdown, LLMProvider, Message, MessageRole, ModelInfo,
    ProviderError, ProviderEvent, ProviderSupport,
};
use runtime_main::sdk::{AgentSdk, SessionId};
use rusqlite::Connection;
use tempfile::TempDir;
use tokio::sync::{mpsc, Notify};

// ── Real-drone-subprocess harness (mirrors recovery_lifecycle.rs) ──────

fn drone_binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().expect("current_exe");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    #[cfg(windows)]
    p.push("runtime-drone.exe");
    #[cfg(unix)]
    p.push("runtime-drone");
    p
}

fn ensure_drone_built() {
    let bin = drone_binary();
    if !bin.exists() {
        let target_dir = bin.parent().expect("parent");
        let mut cmd = std::process::Command::new(env!("CARGO"));
        cmd.args(["build", "--bin", "runtime-drone"]);
        if std::env::var_os("CARGO_TARGET_DIR").is_none() {
            cmd.env(
                "CARGO_TARGET_DIR",
                target_dir.parent().expect("profile parent"),
            );
        }
        let status = cmd.status().expect("cargo build");
        assert!(status.success(), "drone build failed");
    }
    assert!(bin.exists(), "drone binary missing at {}", bin.display());
}

#[cfg(unix)]
fn make_socket(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("d.sock")
}

#[cfg(windows)]
fn make_socket(_dir: &std::path::Path) -> std::path::PathBuf {
    let suffix = uuid::Uuid::new_v4();
    std::path::PathBuf::from(format!(r"\\.\pipe\runtime-drone-smoke-sig-{suffix}"))
}

fn socket_to_addr(p: &std::path::Path) -> String {
    p.to_string_lossy().into_owned()
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

async fn poll_until<F: Fn(&Connection) -> bool>(
    db_path: &std::path::Path,
    predicate: F,
    label: &str,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        if let Ok(conn) = Connection::open(db_path) {
            if predicate(&conn) {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("predicate never satisfied: {label}");
}

fn signals_count(db_path: &std::path::Path) -> i64 {
    let conn = Connection::open(db_path).expect("open db");
    conn.query_row("SELECT COUNT(*) FROM signals", [], |r| r.get(0))
        .unwrap_or(0)
}

fn signals_count_for_session(db_path: &std::path::Path, session_id: &str) -> i64 {
    let conn = Connection::open(db_path).expect("open db");
    conn.query_row(
        "SELECT COUNT(*) FROM signals WHERE session_id = ?1",
        [session_id],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

fn smoke_config() -> AgentConfig {
    // Equivalent to src-tauri's smoke_config() (commands.rs:917) — a
    // runtime-main integration test cannot import the src-tauri binary
    // crate, so the assembled config is reconstructed here.
    AgentConfig {
        model: "claude-haiku-4-5".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text {
                text: "Say only the word: hello".to_string(),
            }],
        }],
        max_tokens: 16,
        temperature: Some(0.0),
        system_prompt: None,
        tools: vec![],
    }
}

// ── Stub providers (no live Anthropic — CLAUDE.md §10) ─────────────────

/// Deterministic stub: `TextDelta` + `MessageStop`. Drives the smoke
/// path's three signal-bearing events: `AgentSpawned` → `StreamText`
/// (buffered "hello" flushed at `MessageStop`) → `AgentComplete`.
struct StubProvider;

#[async_trait]
impl LLMProvider for StubProvider {
    #[allow(
        clippy::unnecessary_literal_bound,
        reason = "trait method returns &str by signature; literal &'static str must reborrow"
    )]
    fn name(&self) -> &str {
        "smoke-sig-stub"
    }
    fn supports(&self) -> ProviderSupport {
        ProviderSupport {
            tool_use: false,
            streaming: true,
            thinking: false,
        }
    }
    async fn stream(
        &self,
        _config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        Ok(Box::pin(futures::stream::iter(vec![
            ProviderEvent::TextDelta {
                text: "hello".to_string(),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
                total_tokens: None,
            },
        ])))
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

/// Gated stub: yields a single `MessageStop` only AFTER `gate` is
/// notified. Lets the test interleave a drone kill between the
/// pre-loop `emit(AgentSpawned)` (drone alive → signal persists) and
/// the loop's `emit(AgentComplete)` (drone dead → `write_signal`
/// errors, must be tolerated and the run must still complete).
struct GatedStub {
    gate: Arc<Notify>,
}

#[async_trait]
impl LLMProvider for GatedStub {
    #[allow(
        clippy::unnecessary_literal_bound,
        reason = "trait method returns &str by signature; literal &'static str must reborrow"
    )]
    fn name(&self) -> &str {
        "smoke-sig-gated-stub"
    }
    fn supports(&self) -> ProviderSupport {
        ProviderSupport {
            tool_use: false,
            streaming: true,
            thinking: false,
        }
    }
    async fn stream(
        &self,
        _config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        let gate = Arc::clone(&self.gate);
        Ok(Box::pin(futures::stream::once(async move {
            gate.notified().await;
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
                total_tokens: None,
            }
        })))
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

// ── Tests ──────────────────────────────────────────────────────────────

/// 🔴-2 closure proof. The assembled SDK run loop persists the agent
/// signal stream to the live drone DB under the run's session id.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_session_persists_signals_to_live_drone_db() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("session.sqlite");
    let socket = make_socket(dir.path());

    // Model the corrected composition the B.fix production change
    // establishes: the drone is seeded with the SAME id the SDK
    // writes signals under (DroneLifecycle::sdk_session_id →
    // run_smoke_session). A divergent id is exactly the real-app
    // bug — signals.session_id is a FK into sessions(id) and the
    // drone seeds one sessions row = its --session-id, so a
    // mismatched id is silently FK-rejected (IRL 🔴-2, condition 2).
    let session = SessionId::new();
    let sid = session.as_string();
    let mut child = spawn_drone(&sid, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    // Hold `rx` for the whole run — a dropped receiver makes
    // `emit`'s event_tx.send fail with EventChannelClosed and aborts
    // the run before any signal could persist.
    let (tx, _rx) = mpsc::channel::<AgentEvent>(16);
    let sdk = AgentSdk::new(Arc::new(StubProvider), tx, Arc::new(client), session);

    sdk.run_agent(smoke_config())
        .await
        .expect("assembled smoke run completes");

    poll_until(
        &db_path,
        |c| {
            c.query_row("SELECT COUNT(*) FROM signals", [], |r| r.get::<_, i64>(0))
                .unwrap_or(0)
                > 0
        },
        "smoke path persists ≥1 signal to the live drone DB",
    )
    .await;

    let total = signals_count(&db_path);
    let for_session = signals_count_for_session(&db_path, &sid);
    assert!(total > 0, "smoke run must persist signals (IRL 🔴-2)");
    assert_eq!(
        for_session, total,
        "every persisted signal must carry the run's session id ({sid}); \
         a mismatched-session signal is as broken as none for recovery/replay"
    );

    drop(sdk);
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

/// Pins the wiring is COMPLETE, not partial: the number of persisted
/// signal rows equals the number of signal-bearing `AgentEvent`s the
/// assembled run emits (guards an "only `AgentSpawned` persisted"
/// regression).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_session_signal_count_matches_emitted_event_count() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("session.sqlite");
    let socket = make_socket(dir.path());

    // Corrected shared-id composition (see test 1).
    let session = SessionId::new();
    let sid = session.as_string();
    let mut child = spawn_drone(&sid, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    let (tx, mut rx) = mpsc::channel::<AgentEvent>(16);
    let sdk = AgentSdk::new(Arc::new(StubProvider), tx, Arc::new(client), session);

    sdk.run_agent(smoke_config())
        .await
        .expect("assembled smoke run completes");
    drop(sdk);

    let mut emitted = 0usize;
    while let Some(_e) = rx.recv().await {
        emitted += 1;
    }
    assert!(
        emitted >= 3,
        "smoke path emits ≥3 AgentEvents (AgentSpawned, StreamText, AgentComplete); got {emitted}"
    );
    let emitted = i64::try_from(emitted).expect("event count fits i64");

    poll_until(
        &db_path,
        |c| {
            c.query_row(
                "SELECT COUNT(*) FROM signals WHERE session_id = ?1",
                [sid.as_str()],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
                >= emitted
        },
        "every emitted AgentEvent persisted as a signal",
    )
    .await;

    assert_eq!(
        signals_count_for_session(&db_path, &sid),
        emitted,
        "persisted signal count must equal the emitted signal-bearing \
         AgentEvent count (wiring complete, not partial)"
    );

    drop(rx);
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

/// A transient `write_signal` IPC failure mid-run must NOT abort the
/// agent run, and the renderer/in-mem-bus sink must stay intact (the
/// persist is ADDITIVE). The drone is killed AFTER the pre-loop
/// `emit(AgentSpawned)` persists (drone alive) but BEFORE the gated
/// `MessageStop` drives `emit(AgentComplete)` (drone dead →
/// `write_signal` errors). The run must still return `Ok`, all
/// events must still reach `event_tx`, and the pre-kill signal must
/// be on disk.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transient_signal_write_failure_does_not_abort_run() {
    ensure_drone_built();
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("session.sqlite");
    let socket = make_socket(dir.path());

    // Corrected shared-id composition (see test 1) so the pre-kill
    // AgentSpawned signal is FK-accepted; the kill then exercises
    // the transient-failure tolerance on the post-kill emit.
    let session = SessionId::new();
    let sid = session.as_string();
    let mut child = spawn_drone(&sid, &db_path, &socket);
    let client = connect_with_retry(&socket_to_addr(&socket)).await;

    let gate = Arc::new(Notify::new());
    let (tx, mut rx) = mpsc::channel::<AgentEvent>(16);
    let sdk = AgentSdk::new(
        Arc::new(GatedStub {
            gate: Arc::clone(&gate),
        }),
        tx,
        Arc::new(client),
        session,
    );

    let run = tokio::spawn(async move { sdk.run_agent(smoke_config()).await });

    // Wait for the pre-loop AgentSpawned signal + the one-shot
    // SnapshotNow to land while the drone is alive.
    poll_until(
        &db_path,
        |c| {
            let sig: i64 = c
                .query_row("SELECT COUNT(*) FROM signals", [], |r| r.get(0))
                .unwrap_or(0);
            let snap: i64 = c
                .query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))
                .unwrap_or(0);
            sig >= 1 && snap >= 1
        },
        "pre-kill AgentSpawned signal + SnapshotNow landed",
    )
    .await;

    let pre_kill = signals_count_for_session(&db_path, &sid);
    assert_eq!(pre_kill, 1, "exactly the AgentSpawned signal pre-kill");

    // Kill the drone, then release the gated MessageStop so the loop's
    // emit(AgentComplete) hits a dead drone.
    child.start_kill().ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
    gate.notify_one();

    let result = tokio::time::timeout(Duration::from_secs(20), run)
        .await
        .expect("run task did not hang")
        .expect("join");
    assert!(
        result.is_ok(),
        "a transient write_signal IPC failure must NOT abort the agent run; got {result:?}"
    );

    let mut events = Vec::new();
    while let Some(e) = rx.recv().await {
        events.push(e);
    }
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::AgentSpawned { .. })),
        "renderer sink intact: AgentSpawned delivered despite drone death"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::AgentComplete { .. })),
        "renderer sink intact: AgentComplete delivered despite the failed signal write"
    );
}
