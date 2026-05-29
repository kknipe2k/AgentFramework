//! The Builder's Tester backend — M08 Stage F1 (spec Phase 9; ADR-0019).
//!
//! `run_test_session_with` runs a candidate framework (loaded from the
//! canvas, never saved to disk) in an **isolated** test session: its own
//! throwaway `SQLite` path, a test-defaults [`HitlSeam`](crate::hitl::HitlSeam)
//! so the run never blocks on user input, and §8.security L2 capability
//! violations collected onto [`TestOutcome`] as **test failures** rather than raised
//! as live HITL/gap prompts. The session reuses the smoke-session
//! construction (`AgentSdk::with_capability_wiring` →
//! optional `with_mcp_dispatch` → `run_agent`); it does NOT rebuild a
//! session engine.
//!
//! `run_test_session_with` is the `*_with` seam (CLAUDE.md §5): provider,
//! drone, and MCP dispatch are injected so unit tests exercise it with
//! stubs. The OS-touching production wrapper — the throwaway-DB path
//! resolution, the drone spawn, the teardown — lives in the Tauri shell
//! (`src-tauri/src/commands.rs::test_framework`), mirroring how
//! `run_smoke_session` wraps `run_smoke_session_with`.
//!
//! `load_verified_artifact` is the first production load-path caller of
//! [`crate::skills_lock::verify`] (M07.V 🟡 #2): when the test session
//! byte-loads an imported skill/tool for execution, it recomputes the SRI
//! hash and HARD-BLOCKS on drift (integrity > availability; ADR-0014).

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use runtime_core::event::{AgentEvent, CapabilityKindRef};
use runtime_core::generated::framework::Framework;
use tokio::sync::mpsc;

use crate::capability::CapabilityEnforcer;
use crate::drone_ipc::DroneClient;
use crate::framework_loader::{grant_framework_capabilities, inline_agents};
use crate::hitl::HitlSeam;
use crate::providers::{AgentConfig, ContentBlock, LLMProvider, Message, MessageRole};
use crate::sdk::builtin_tools::builtin_tool_defs;
use crate::sdk::{AgentSdk, CapabilityWiring, McpToolDispatch, SessionId};
use crate::skills_lock::LockError;

/// The result of one Tester run. Crosses the Tauri wire to F2 (the modal
/// renders every field).
///
/// `passed == false` covers BOTH a session that hit a capability
/// violation / an integrity block AND a clean run the user judged
/// wrong — a failed test is never a [`TesterError`] (those are
/// infrastructure-only).
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestOutcome {
    /// Whether the test session completed without a capability failure
    /// or an integrity block. Any `capability_failures` entry or any
    /// `AgentEvent::ArtifactHashMismatch` in `trace` forces `false`.
    pub passed: bool,
    /// §8.security L2 capability violations observed during the run.
    /// Non-empty ⇒ `passed == false`. F2 surfaces these as test
    /// failures; they are NEVER raised as a live HITL/gap prompt.
    pub capability_failures: Vec<CapabilityFailure>,
    /// Token spend for the run (in / out / total).
    pub token_spend: TokenSpend,
    /// Wall-clock duration of the test session.
    pub timing: Duration,
    /// The VDR (Verification & Decision Record) the test session
    /// produced, or `Value::Null` when the run emitted none.
    pub vdr: serde_json::Value,
    /// The full ordered `AgentEvent` trace, so F2 can render the smaller
    /// graph pane + the pass/fail trace from one payload.
    pub trace: Vec<AgentEvent>,
}

/// One §8.security L2 capability violation observed in the test session.
/// Collected onto [`TestOutcome::capability_failures`], not raised as HITL.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CapabilityFailure {
    /// The runtime agent id that attempted the denied action.
    pub agent_id: String,
    /// The capability that was missing/denied (human-readable).
    pub needed: String,
    /// The enforcer's reason string.
    pub reason: String,
}

/// Token in / out / total for a test run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct TokenSpend {
    /// Input tokens summed across the run's `token_usage` signals.
    pub input: u64,
    /// Output tokens summed across the run's `token_usage` signals.
    pub output: u64,
    /// `input + output`.
    pub total: u64,
}

/// Infrastructure failure of the Tester itself — NOT a failed test.
///
/// A capability violation / a hash mismatch is `Ok(TestOutcome { passed:
/// false, .. })`; `TesterError` is reserved for "the test could not be
/// run at all" — the drone failed to spawn, the throwaway temp file
/// could not be created, the provider stream open failed.
#[derive(Debug, thiserror::Error)]
pub enum TesterError {
    /// The throwaway test database could not be created / resolved.
    #[error("test database setup failed: {0}")]
    DbSetup(#[from] std::io::Error),
    /// The drone subprocess for the test session failed to spawn or
    /// connect.
    #[error("test-session drone failed: {0}")]
    Drone(String),
    /// The agent-with-tools loop surfaced an infrastructure error
    /// (provider stream open failed, event channel closed). A
    /// *capability* failure is a failed test, not this.
    #[error("test session run failed: {0}")]
    Run(String),
}

/// Fold an ordered `AgentEvent` trace into a [`TestOutcome`].
///
/// Pure: every `AgentEvent::CapabilityViolation` becomes a
/// [`CapabilityFailure`]; every `AgentEvent::TokenUsage` accumulates into
/// [`TokenSpend`]; the presence of any `AgentEvent::ArtifactHashMismatch`
/// (or any capability failure) forces `passed = false`.
#[must_use]
pub fn fold_outcome(trace: Vec<AgentEvent>, timing: Duration) -> TestOutcome {
    let mut capability_failures = Vec::new();
    let mut token_spend = TokenSpend::default();
    let mut integrity_blocked = false;
    for event in &trace {
        match event {
            AgentEvent::CapabilityViolation {
                agent_id,
                capability_kind,
                declared_scope,
                requested_action,
            } => capability_failures.push(CapabilityFailure {
                agent_id: agent_id.clone(),
                needed: capability_kind_label(*capability_kind).to_string(),
                reason: format!(
                    "requested `{requested_action}` — declared scope `{declared_scope}`"
                ),
            }),
            AgentEvent::TokenUsage { input, output, .. } => {
                token_spend.input += *input;
                token_spend.output += *output;
            }
            AgentEvent::ArtifactHashMismatch { .. } => integrity_blocked = true,
            _ => {}
        }
    }
    token_spend.total = token_spend.input + token_spend.output;
    TestOutcome {
        passed: capability_failures.is_empty() && !integrity_blocked,
        capability_failures,
        token_spend,
        timing,
        vdr: serde_json::Value::Null,
        trace,
    }
}

/// Byte-load an imported artifact for execution in the test session,
/// verifying its integrity against `skills.lock` first.
///
/// [`crate::skills_lock::verify`] recomputes the SRI content hash and
/// HARD-BLOCKS on drift (integrity > availability; ADR-0014; spec
/// §2214). A `HashMismatch` is mapped to `AgentEvent::ArtifactHashMismatch`
/// via `emit` and the load is REFUSED (the `Err` propagates) — the test
/// fails with a clear hash-mismatch reason. This is the first production
/// load-path caller of `verify` (M07.V 🟡 #2 — discharged here).
///
/// # Errors
///
/// - [`LockError::HashMismatch`] when the bytes drifted from the lock —
///   `emit` has already fired `ArtifactHashMismatch`.
/// - [`LockError::NotFound`] / [`LockError::Io`] / [`LockError::Parse`]
///   propagated from `verify` (an un-locked or unreadable artifact must
///   not load by virtue of an absent / corrupt record).
pub fn load_verified_artifact(
    lock_path: &Path,
    artifact_ref: &str,
    bytes: &[u8],
    emit: &impl Fn(AgentEvent),
) -> Result<Vec<u8>, LockError> {
    match crate::skills_lock::verify(lock_path, artifact_ref, bytes) {
        Ok(()) => Ok(bytes.to_vec()),
        Err(LockError::HashMismatch {
            artifact_ref,
            expected,
            actual,
        }) => {
            emit(AgentEvent::ArtifactHashMismatch {
                artifact_ref: artifact_ref.clone(),
                expected: expected.clone(),
                actual: actual.clone(),
            });
            Err(LockError::HashMismatch {
                artifact_ref,
                expected,
                actual,
            })
        }
        Err(other) => Err(other),
    }
}

/// The canonical `snake_case` label for a capability kind — surfaced as
/// [`CapabilityFailure::needed`]. `CapabilityKindRef` is a hand-rolled
/// enum (no `Display`), so the mapping is explicit; a new kind forces an
/// update here.
const fn capability_kind_label(kind: CapabilityKindRef) -> &'static str {
    match kind {
        CapabilityKindRef::Read => "read",
        CapabilityKindRef::Write => "write",
        CapabilityKindRef::Exec => "exec",
        CapabilityKindRef::Network => "network",
        CapabilityKindRef::ProcessSpawn => "process_spawn",
    }
}

/// The candidate framework's imported artifacts that carry a filesystem
/// path — `(name, path)` for every skill / tool the integrity pre-flight
/// should byte-load.
fn imported_artifact_refs(framework: &Framework) -> Vec<(String, String)> {
    let skills = framework
        .skills
        .iter()
        .filter_map(|s| s.path.as_ref().map(|p| (s.name.clone(), p.clone())));
    let tools = framework
        .tools
        .iter()
        .filter_map(|t| t.path.as_ref().map(|p| (t.name.clone(), p.clone())));
    skills.chain(tools).collect()
}

/// Verify the candidate framework's imported artifacts against the
/// `skills.lock` co-located with the throwaway test DB.
///
/// Returns one `AgentEvent::ArtifactHashMismatch` per drifted artifact;
/// an empty vec when the framework imports nothing or no lock is present
/// (an un-saved canvas framework legitimately has no lock).
fn verify_framework_artifacts(framework: &Framework, db_path: &Path) -> Vec<AgentEvent> {
    let root = db_path.parent().unwrap_or_else(|| Path::new("."));
    let lock_path = root.join("skills.lock");
    if !lock_path.exists() {
        return Vec::new();
    }
    let events = std::cell::RefCell::new(Vec::new());
    for (name, rel) in imported_artifact_refs(framework) {
        if let Ok(bytes) = std::fs::read(root.join(&rel)) {
            let _ = load_verified_artifact(&lock_path, &name, &bytes, &|e| {
                events.borrow_mut().push(e);
            });
        }
    }
    events.into_inner()
}

/// Build the `AgentConfig` for the test run from the candidate framework
/// and the user's task.
///
/// MCP / framework tools reach the run through the injected MCP dispatcher
/// (the smoke / D2 archetype). The **in-process built-ins** (`Read`/
/// `Write`, M08.7.A) are advertised in `config.tools` so a real model
/// emits the matching `ToolUse` — the union of every inline agent's
/// `allowed_tools` filtered to the in-process built-in set, deduplicated
/// (the Anthropic API rejects duplicate tool names).
fn test_agent_config(framework: &Framework, task: &str) -> AgentConfig {
    let mut seen = std::collections::BTreeSet::new();
    let allowed: Vec<String> = inline_agents(framework)
        .iter()
        .flat_map(|a| a.allowed_tools.iter())
        .filter(|t| seen.insert((*t).clone()))
        .cloned()
        .collect();
    AgentConfig {
        model: framework.model.id.as_str().to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text {
                text: task.to_string(),
            }],
        }],
        max_tokens: 4096,
        temperature: Some(0.0),
        system_prompt: None,
        tools: builtin_tool_defs(&allowed),
    }
}

/// Test-seam: run an isolated test session against caller-supplied
/// drone / provider / dispatch collaborators over a caller-supplied
/// throwaway `db_path`.
///
/// `db_path` MUST be a throwaway temp-file path — NEVER the user session
/// DB (ADR-0019; the Tauri-shell `test_framework` command guarantees
/// this). Unit tests pass a `tempfile`-backed path. It also roots the
/// resolution of any relative imported-artifact path the integrity
/// pre-flight walks.
///
/// Reuses the smoke-session construction
/// (`AgentSdk::with_capability_wiring` → optional `with_mcp_dispatch` →
/// `run_agent`); it does not rebuild a session engine. The `HitlSeam`
/// woven into the session is the test-defaults variant so the run never
/// blocks on user input (spec Phase 9).
///
/// # Errors
///
/// [`TesterError`] for infrastructure failure only. A capability
/// violation / a hash mismatch produces `Ok(TestOutcome { passed:
/// false, .. })`, NOT an `Err`.
pub async fn run_test_session_with<P: LLMProvider + 'static>(
    framework: &Framework,
    task: &str,
    db_path: &Path,
    provider: P,
    drone: Arc<DroneClient>,
    mcp_dispatch: Option<Arc<dyn McpToolDispatch>>,
    session_id: SessionId,
) -> Result<TestOutcome, TesterError> {
    let started = Instant::now();

    // Artifact-integrity pre-flight (M07.V 🟡 #2): verify every imported
    // artifact the candidate framework byte-references against its
    // `skills.lock`. A drift refuses the run (integrity > availability —
    // ADR-0014), with the `ArtifactHashMismatch` events leading the trace.
    let preflight = verify_framework_artifacts(framework, db_path);
    if preflight
        .iter()
        .any(|e| matches!(e, AgentEvent::ArtifactHashMismatch { .. }))
    {
        return Ok(fold_outcome(preflight, started.elapsed()));
    }

    // Reuse the smoke-session construction (`with_capability_wiring` →
    // optional `with_mcp_dispatch` → `run_agent`). The `HitlSeam` is the
    // test-defaults variant so the run never blocks on user input.
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(1024);
    // Load the candidate framework's declared capabilities into the L1
    // enforcer (M08.7.A §1.3-B): without this the enforcer is empty
    // (default-deny `NoDeclarations`) and no built-in tool can execute.
    let mut enforcer = CapabilityEnforcer::new();
    grant_framework_capabilities(&mut enforcer, framework);
    let wiring = CapabilityWiring::new(
        Arc::new(enforcer),
        Arc::new(framework.clone()),
        Arc::new(HitlSeam::test_defaults()),
    );
    let mut sdk =
        AgentSdk::with_capability_wiring(Arc::new(provider), event_tx, drone, session_id, wiring);
    if let Some(dispatch) = mcp_dispatch {
        sdk = sdk.with_mcp_dispatch(dispatch);
    }
    let config = test_agent_config(framework, task);
    let run = tokio::spawn(async move { sdk.run_agent(config).await });

    let mut trace = preflight;
    while let Some(event) = event_rx.recv().await {
        trace.push(event);
    }
    run.await
        .map_err(|e| TesterError::Run(format!("test session task panicked: {e}")))?
        .map_err(|e| TesterError::Run(e.to_string()))?;

    Ok(fold_outcome(trace, started.elapsed()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::VecDeque;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use futures::stream::BoxStream;
    use runtime_core::generated::skills_lock::LockEntry;
    use serde_json::json;

    use crate::providers::{
        AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
        ProviderSupport,
    };
    use crate::sdk::{McpDispatchError, McpDispatchOutcome};

    // ── Fixtures ─────────────────────────────────────────────────────

    /// A schema-valid single-agent framework — the candidate the Tester
    /// runs. `session_root_agent` matches the inline agent's id.
    fn fw_one_agent() -> Framework {
        serde_json::from_value(json!({
            "name": "m08-f1-tester-fixture",
            "version": "1.0.0",
            "description": "M08.F1 Tester backend fixture framework",
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
        .expect("the fixture framework round-trips through the schema")
    }

    /// Multi-turn stub provider: yields one scripted `Vec<ProviderEvent>`
    /// per `stream()` call, empty (loop-terminating) once exhausted.
    struct TurnStub {
        turns: Mutex<VecDeque<Vec<ProviderEvent>>>,
    }

    impl TurnStub {
        fn new(turns: Vec<Vec<ProviderEvent>>) -> Self {
            Self {
                turns: Mutex::new(turns.into()),
            }
        }
        /// A provider that says one word and stops — the clean-run case.
        fn clean() -> Self {
            Self::new(vec![vec![
                ProviderEvent::TextDelta {
                    text: "done".to_string(),
                },
                ProviderEvent::MessageStop {
                    stop_reason: "end_turn".to_string(),
                    total_tokens: None,
                },
            ]])
        }
    }

    #[async_trait]
    impl LLMProvider for TurnStub {
        fn name(&self) -> &'static str {
            "m08-f1-turn-stub"
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

    /// Mock `McpToolDispatch` that denies every dispatch — exercises the
    /// capability-violation → `CapabilityFailure` fold without a real
    /// `McpDispatcher`.
    struct BlockingDispatch;

    #[async_trait]
    impl McpToolDispatch for BlockingDispatch {
        async fn dispatch_if_mcp(
            &self,
            agent_id: &str,
            tool_name: &str,
            _args: serde_json::Value,
            _aliases: &std::collections::BTreeMap<String, String>,
        ) -> Option<Result<McpDispatchOutcome, McpDispatchError>> {
            Some(Ok(McpDispatchOutcome::Blocked {
                agent_id: agent_id.to_string(),
                server: "fs".to_string(),
                tool: tool_name.to_string(),
                reason: "no capabilities declared for this tool".to_string(),
            }))
        }
    }

    /// Build a `LockEntry` for `bytes`, mirroring the
    /// `skills_lock_integration.rs` schema-shaped fixture.
    fn lock_entry_for(bytes: &[u8]) -> LockEntry {
        serde_json::from_value(json!({
            "kind": "skill",
            "source": { "type": "file", "path": "./skill.md" },
            "content_hash": crate::skills_lock::content_hash(bytes),
            "installed_at": "2026-05-18T14:23:00Z",
            "tier_at_install": "promoted",
            "validation_report_id": "vr-m08-f1"
        }))
        .expect("schema-shaped LockEntry deserializes")
    }

    /// Deserialize an `AgentEvent` from its schema-faithful wire shape —
    /// sidesteps the generated newtype constructors.
    fn event(value: serde_json::Value) -> AgentEvent {
        serde_json::from_value(value).expect("schema-faithful AgentEvent deserializes")
    }

    // ── fold_outcome ────────────────────────────────────────────────

    #[test]
    fn fold_outcome_clean_trace_yields_passed_true() {
        let trace = vec![event(json!({
            "type": "agent_spawned",
            "agent_id": "worker",
            "agent_name": "worker",
            "session_id": "s-1"
        }))];
        let outcome = fold_outcome(trace, Duration::from_millis(5));
        assert!(
            outcome.passed,
            "a trace with no violation and no hash mismatch passes"
        );
        assert!(outcome.capability_failures.is_empty());
    }

    #[test]
    fn fold_outcome_capability_violation_yields_a_failure_and_passed_false() {
        let trace = vec![event(json!({
            "type": "capability_violation",
            "agent_id": "worker",
            "capability_kind": "read",
            "declared_scope": "none",
            "requested_action": "read /etc/passwd"
        }))];
        let outcome = fold_outcome(trace, Duration::from_millis(5));
        assert!(!outcome.passed, "a capability violation fails the test");
        assert_eq!(
            outcome.capability_failures.len(),
            1,
            "the violation folds into exactly one CapabilityFailure"
        );
        assert_eq!(outcome.capability_failures[0].agent_id, "worker");
    }

    #[test]
    fn fold_outcome_artifact_hash_mismatch_yields_passed_false() {
        let trace = vec![event(json!({
            "type": "artifact_hash_mismatch",
            "artifact_ref": "skill@1.0.0",
            "expected": "sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
            "actual": "sha256-deadBEEF000000000000000000000000000000000000="
        }))];
        let outcome = fold_outcome(trace, Duration::from_millis(5));
        assert!(
            !outcome.passed,
            "an integrity block forces passed = false even with no capability failure"
        );
    }

    #[test]
    fn fold_outcome_sums_token_usage_into_token_spend() {
        let trace = vec![
            event(json!({
                "type": "token_usage",
                "cost_usd": 0.01, "input": 100, "output": 40, "model": "claude-haiku-4-5"
            })),
            event(json!({
                "type": "token_usage",
                "cost_usd": 0.02, "input": 50, "output": 10, "model": "claude-haiku-4-5"
            })),
        ];
        let outcome = fold_outcome(trace, Duration::from_millis(5));
        assert_eq!(outcome.token_spend.input, 150, "input tokens summed");
        assert_eq!(outcome.token_spend.output, 50, "output tokens summed");
        assert_eq!(
            outcome.token_spend.total, 200,
            "total = input + output across every token_usage event"
        );
    }

    #[test]
    fn fold_outcome_preserves_the_full_event_trace() {
        let trace = vec![
            event(json!({
                "type": "agent_spawned",
                "agent_id": "worker", "agent_name": "worker", "session_id": "s-1"
            })),
            event(json!({ "type": "stream_text", "agent_id": "worker", "text": "hi" })),
        ];
        let outcome = fold_outcome(trace.clone(), Duration::from_millis(5));
        assert_eq!(
            outcome.trace.len(),
            trace.len(),
            "the outcome carries the full ordered trace for F2 to render"
        );
    }

    // ── run_test_session_with ───────────────────────────────────────

    #[tokio::test]
    async fn run_test_session_with_returns_test_outcome_for_a_clean_run() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("runtime-tester.sqlite");
        let outcome = run_test_session_with(
            &fw_one_agent(),
            "summarize the input",
            &db_path,
            TurnStub::clean(),
            Arc::new(DroneClient::noop()),
            None,
            SessionId::new(),
        )
        .await
        .expect("a clean run returns Ok(TestOutcome), not Err");
        assert!(outcome.passed, "a clean tool-free run passes");
        assert!(outcome.capability_failures.is_empty());
    }

    #[tokio::test]
    async fn run_test_session_with_twice_in_sequence_with_distinct_db_paths_both_succeed() {
        // gotcha #69 — the seam must be re-entrant across runs.
        let dir = tempfile::tempdir().expect("tempdir");
        for run in 0..2 {
            let db_path = dir.path().join(format!("runtime-tester-{run}.sqlite"));
            let outcome = run_test_session_with(
                &fw_one_agent(),
                "task",
                &db_path,
                TurnStub::clean(),
                Arc::new(DroneClient::noop()),
                None,
                SessionId::new(),
            )
            .await
            .unwrap_or_else(|e| panic!("run {run} should succeed: {e}"));
            assert!(outcome.passed, "run {run} passes");
        }
    }

    #[tokio::test]
    async fn run_test_session_with_does_not_block_on_a_test_defaults_hitl_seam() {
        // The test session is woven with HitlSeam::test_defaults() — the
        // run must complete unattended (no renderer ever resolves a
        // prompt). A timeout would mean the seam blocked on user input.
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("runtime-tester.sqlite");
        let framework = fw_one_agent();
        let run = run_test_session_with(
            &framework,
            "task",
            &db_path,
            TurnStub::clean(),
            Arc::new(DroneClient::noop()),
            None,
            SessionId::new(),
        );
        let outcome = tokio::time::timeout(Duration::from_secs(10), run)
            .await
            .expect("the test session must not block on user input")
            .expect("the run completes");
        assert!(outcome.passed);
    }

    #[tokio::test]
    async fn run_test_session_with_blocked_mcp_dispatch_is_a_failed_test_not_an_error() {
        // A §8.security L2 capability violation is a FAILED TEST
        // (Ok(TestOutcome { passed: false })), never Err(TesterError).
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("runtime-tester.sqlite");
        let provider = TurnStub::new(vec![
            vec![ProviderEvent::ToolUse {
                id: "tu-1".to_string(),
                name: "fs__read".to_string(),
                input: json!({ "path": "/etc/passwd" }),
            }],
            vec![ProviderEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
                total_tokens: None,
            }],
        ]);
        let outcome = run_test_session_with(
            &fw_one_agent(),
            "read a file",
            &db_path,
            provider,
            Arc::new(DroneClient::noop()),
            Some(Arc::new(BlockingDispatch) as Arc<dyn McpToolDispatch>),
            SessionId::new(),
        )
        .await
        .expect("a capability violation is Ok(TestOutcome), not Err(TesterError)");
        assert!(
            !outcome.capability_failures.is_empty(),
            "the blocked dispatch folds into a CapabilityFailure"
        );
        assert!(
            !outcome.passed,
            "a capability failure forces passed = false"
        );
    }

    // ── load_verified_artifact (M07.V 🟡 #2 discharge) ───────────────

    #[test]
    fn load_verified_artifact_passes_for_matching_bytes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let lock_path = dir.path().join("skills.lock");
        let bytes = b"the real artifact bytes";
        crate::skills_lock::write_entry(&lock_path, "skill@1.0.0", lock_entry_for(bytes))
            .expect("write lock");

        let emitted = std::cell::RefCell::new(Vec::<AgentEvent>::new());
        let loaded = load_verified_artifact(&lock_path, "skill@1.0.0", bytes, &|e| {
            emitted.borrow_mut().push(e);
        })
        .expect("matching bytes verify Ok");
        assert_eq!(
            loaded, bytes,
            "the verified bytes are returned for execution"
        );
        assert!(
            emitted.borrow().is_empty(),
            "a clean verify emits no ArtifactHashMismatch"
        );
    }

    #[test]
    fn load_verified_artifact_tampered_bytes_emit_artifact_hash_mismatch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let lock_path = dir.path().join("skills.lock");
        let good = b"the real artifact bytes";
        crate::skills_lock::write_entry(&lock_path, "skill@1.0.0", lock_entry_for(good))
            .expect("write lock");

        let emitted = std::cell::RefCell::new(Vec::<AgentEvent>::new());
        let _ = load_verified_artifact(&lock_path, "skill@1.0.0", b"TAMPERED bytes", &|e| {
            emitted.borrow_mut().push(e);
        });
        assert!(
            emitted
                .borrow()
                .iter()
                .any(|e| matches!(e, AgentEvent::ArtifactHashMismatch { .. })),
            "drift emits the schema-faithful ArtifactHashMismatch event"
        );
    }

    #[test]
    fn load_verified_artifact_hash_mismatch_refuses_the_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let lock_path = dir.path().join("skills.lock");
        let good = b"the real artifact bytes";
        crate::skills_lock::write_entry(&lock_path, "skill@1.0.0", lock_entry_for(good))
            .expect("write lock");

        let err = load_verified_artifact(&lock_path, "skill@1.0.0", b"TAMPERED bytes", &|_| {})
            .expect_err("drifted bytes must be refused, never run");
        assert!(matches!(err, LockError::HashMismatch { .. }), "got {err:?}");
    }

    #[test]
    fn load_verified_artifact_unknown_artifact_propagates_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let lock_path = dir.path().join("skills.lock");
        crate::skills_lock::write_entry(&lock_path, "known@1.0.0", lock_entry_for(b"x"))
            .expect("write lock");

        let err = load_verified_artifact(&lock_path, "ghost@9.9.9", b"x", &|_| {})
            .expect_err("an un-locked artifact must not load by virtue of an absent record");
        assert!(
            matches!(err, LockError::NotFound(ref r) if r == "ghost@9.9.9"),
            "got {err:?}"
        );
    }

    // ── M07.V 🟡 #2 — the integrity pre-flight through the seam ──────

    #[tokio::test]
    async fn run_test_session_with_a_tampered_imported_artifact_refuses_the_run() {
        // A candidate framework that byte-references an imported artifact
        // whose on-disk bytes drifted from the co-located skills.lock
        // fails the test — `run_test_session_with`'s integrity pre-flight
        // refuses the run rather than executing the tampered artifact
        // (integrity > availability; ADR-0014).
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("runtime-tester.sqlite");
        // The imported skill on disk carries TAMPERED bytes; the
        // co-located skills.lock records the hash of the GOOD bytes.
        std::fs::write(
            dir.path().join("imported-skill.md"),
            b"TAMPERED skill bytes",
        )
        .expect("write artifact");
        crate::skills_lock::write_entry(
            &dir.path().join("skills.lock"),
            "imported-skill",
            lock_entry_for(b"the real skill bytes"),
        )
        .expect("write lock");
        let framework: Framework = serde_json::from_value(json!({
            "name": "m08-f1-tampered-fixture",
            "version": "1.0.0",
            "description": "M08.F1 tampered imported-artifact fixture",
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
            "skills": [
                { "name": "imported-skill", "path": "imported-skill.md", "source": "external" }
            ],
            "session_root_agent": "worker",
        }))
        .expect("the tampered-artifact fixture framework round-trips");

        let outcome = run_test_session_with(
            &framework,
            "summarize the input",
            &db_path,
            TurnStub::clean(),
            Arc::new(DroneClient::noop()),
            None,
            SessionId::new(),
        )
        .await
        .expect("a tampered artifact is a failed test, not Err(TesterError)");
        assert!(
            !outcome.passed,
            "a hash mismatch refuses the run — the candidate fails the test"
        );
        assert!(
            outcome
                .trace
                .iter()
                .any(|e| matches!(e, AgentEvent::ArtifactHashMismatch { .. })),
            "the refused run carries the schema-faithful ArtifactHashMismatch"
        );
    }
}
