//! M08.7.B rung 2 — capability enforcement gates a LIVE built-in tool.
//!
//! The cluster-gate close contract (`docs/cluster-pattern.md` §1/§4): these
//! assembled tests drive the REAL `run_test_session_with_tier` →
//! `AgentSdk::run_agent` multi-turn loop against a real `tempfile`
//! workspace. The ONLY stub is the provider (no live Anthropic — CLAUDE.md
//! §10); the executor, the capability enforcer (grants loaded from
//! `file_access` via `grant_framework_capabilities`), the filesystem write,
//! and the multi-turn feedback are all real.
//!
//! Rung 1 (`builtin_tool_execution.rs`) proved the *read* scope-denial and
//! the *Novice tier* write-denial through the loop. Rung 1's blocked-Write
//! test denies at the **L4 tier gate** (Novice forbids all writes) BEFORE
//! the `file_access` scope is consulted (`enforcer.rs` checks tier first) —
//! so the **scope gate on Write** stayed unproven through the assembled
//! loop. Rung 2 runs the session at the **Promoted** tier (L4 pass-through)
//! so a Write REACHES the L1 scope check and is denied THERE — the
//! `CapabilityViolation { capability_kind: Write }` scope-gate denial,
//! distinct from rung 1's `TierViolation`.
//!
//! Grounded-claims (CLAUDE.md §4 rule 11 / gotcha #66): the load-bearing
//! assertion is the **observable side effect — the file on disk** (absent on
//! a denial, present with its content on an allow), never the
//! `CapabilityViolation` event alone.
//!
//! Tier-wire scope (M08.7.B ground-at-red, Finding B): NO production path
//! wires the user's tier into the run-loop enforcer — `run_test_session_with`
//! always runs at Novice. `run_test_session_with_tier` is the **test-path**
//! seam that lets the assembled loop express Promoted so the scope gate is
//! provable. Production tier-wiring is a separate gap (TD — painted, not
//! wired) routed to the live-session rung; rung 2 does NOT wire it
//! (Hard Rule 8 / ADR-0019 scope lock).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::json;
use tempfile::TempDir;

use runtime_core::event::{AgentEvent, CapabilityKindRef};
use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with_tier;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{
    AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
    ProviderSupport,
};
use runtime_main::sdk::SessionId;
use runtime_main::tier::Tier;

// ── helpers (a tests/*.rs binary cannot import builtin_tool_execution.rs's) ─

/// Forward-slash a path so the same string is a valid `std::fs` argument
/// (Windows accepts `/`) and a stable `globset` match target.
fn fwd(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// A schema-valid one-agent framework whose `worker` agent declares the
/// given `file_access` globs + `allowed_tools`; `session_root_agent` is
/// `worker`, so the run's dispatch agent id is `worker`.
fn one_agent_fw(read: &[&str], write: &[&str], allowed_tools: &[&str]) -> Framework {
    serde_json::from_value(json!({
        "name": "m08-7-b-rung2",
        "version": "1.0.0",
        "description": "M08.7.B rung 2 capability-on-live-tool fixture",
        "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "capabilities": {
                "tools_called": [],
                "skills_loaded": [],
                "file_access": { "read": read, "write": write },
                "network": [],
                "shell": false,
                "spawn_agents": []
            },
            "allowed_tools": allowed_tools,
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": [],
        "skills": [],
        "session_root_agent": "worker",
    }))
    .expect("the rung-2 fixture framework round-trips through the schema")
}

/// The first `CapabilityViolation` in the trace, as `(kind, action)`. Does
/// NOT match `TierViolation` — so a `Some` result discriminates a SCOPE
/// denial from rung 1's tier denial.
fn first_violation(trace: &[AgentEvent]) -> Option<(CapabilityKindRef, String)> {
    trace.iter().find_map(|e| match e {
        AgentEvent::CapabilityViolation {
            capability_kind,
            requested_action,
            ..
        } => Some((*capability_kind, requested_action.clone())),
        _ => None,
    })
}

/// A provider stub that emits one scripted `Write` `ToolUse` on turn 1 and
/// stops on every later turn — the only stub in the assembled path.
struct WriteToolStub {
    path: String,
    content: String,
    turn: Mutex<usize>,
}

impl WriteToolStub {
    fn new(path: String, content: &str) -> Self {
        Self {
            path,
            content: content.to_string(),
            turn: Mutex::new(0),
        }
    }
}

#[async_trait]
impl LLMProvider for WriteToolStub {
    fn name(&self) -> &'static str {
        "m08-7-b-write-stub"
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
        let n = {
            let mut t = self.turn.lock().expect("turn lock");
            let n = *t;
            *t += 1;
            n
        };
        if n == 0 {
            return Ok(Box::pin(futures::stream::iter(vec![
                ProviderEvent::ToolUse {
                    id: "tu-1".to_string(),
                    name: "Write".to_string(),
                    input: json!({ "path": self.path, "content": self.content }),
                },
            ])));
        }
        // Any turn after the first dispatches no tool — the multi-turn loop
        // terminates (agent_sdk.rs run_agent: a turn that dispatches nothing
        // ends the loop).
        Ok(Box::pin(futures::stream::iter(vec![
            ProviderEvent::TextDelta {
                text: "ok".to_string(),
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

// ── B.4.1 — Promoted out-of-scope Write denies on SCOPE (not tier) ────────

/// Scenario: A write outside `file_access` scope does not run.
///
/// At the Promoted tier the L4 gate is a pass-through, so a Write to a path
/// outside `file_access.write` reaches the L1 scope check and is denied
/// THERE — `CapabilityViolation { capability_kind: Write }`, NOT rung 1's
/// `TierViolation`. The grounded assertion is that NO file appears on disk
/// (the executor never ran), with the tmp dir present so a non-denied write
/// WOULD have succeeded — the absence is the denial, not an IO failure.
#[tokio::test]
async fn promoted_write_outside_scope_denies_on_scope_and_creates_no_file() {
    let dir = TempDir::new().expect("tempdir");
    let target = dir.path().join("secret.txt");
    let path_arg = format!("{}/secret.txt", fwd(dir.path()));

    // Write grant covers a DIFFERENT subtree — the request path is outside it.
    let fw = one_agent_fw(&[], &["allowed/**"], &["Write"]);
    let provider = WriteToolStub::new(path_arg, "should-not-be-written");

    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with_tier(
        &fw,
        "write the secret file",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
        Tier::Promoted,
    )
    .await
    .expect("the assembled run completes (a denial is a failed test, not Err)");

    let (kind, _action) = first_violation(&outcome.trace)
        .expect("an out-of-scope Promoted write must emit CapabilityViolation");
    assert_eq!(
        kind,
        CapabilityKindRef::Write,
        "the denial is the L1 SCOPE gate on Write, reached because Promoted passes L4"
    );
    assert!(
        !outcome
            .trace
            .iter()
            .any(|e| matches!(e, AgentEvent::TierViolation { .. })),
        "the Promoted write must reach the SCOPE gate — NOT be tier-denied (rung 1's path); trace={:?}",
        outcome.trace
    );
    assert!(
        !target.exists(),
        "the scope-denied write must create no file on disk (the executor never ran)"
    );
    assert!(
        !outcome.passed,
        "a capability violation fails the test outcome"
    );
}

// ── B.4.2 — Promoted in-scope Write runs, through the assembled loop ──────

/// Scenario: A write inside scope does run.
///
/// Promotes the unit-tested happy path
/// (`builtin_tool_execution.rs::execute_write_inside_scope_promoted_writes_file`)
/// to an assembled-loop proof: the executor wrote the file through the REAL
/// `run_test_session_with_tier → drive_stream → dispatch_builtin →
/// std::fs::write` path. The grounded assertion is the file's content read
/// back off disk.
#[tokio::test]
async fn promoted_write_inside_scope_runs_and_writes_the_file() {
    let dir = TempDir::new().expect("tempdir");
    let target = dir.path().join("out.txt");
    let path_arg = format!("{}/out.txt", fwd(dir.path()));
    let dir_glob = format!("{}/**", fwd(dir.path()));

    let fw = one_agent_fw(&[], &[&dir_glob], &["Write"]);
    let provider = WriteToolStub::new(path_arg, "rung-2-ok");

    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with_tier(
        &fw,
        "write out.txt",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
        Tier::Promoted,
    )
    .await
    .expect("the assembled run completes");

    assert_eq!(
        std::fs::read_to_string(&target).expect("the in-scope Promoted write produced the file"),
        "rung-2-ok",
        "the write produced its content on disk through the assembled loop"
    );
    assert!(
        outcome
            .trace
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolResult { .. })),
        "the successful write feeds a ToolResult back; trace={:?}",
        outcome.trace
    );
    assert!(
        outcome.passed,
        "an in-scope write is no violation — the test outcome passes"
    );
}

// ── B.4.3 — the violation surfaces as a CapabilityFailure, unattended ─────

/// Scenario (HITL triage outcome b — the Tester is HITL-less by design,
/// ADR-0019 + `hitl/seam.rs::test_defaults`): a capability violation does
/// NOT raise a live HITL prompt on the Tester path — it FOLDS into
/// `TestOutcome.capability_failures`, and the auto-default seam keeps the
/// run unattended. This asserts the violation surfaces as a `write`
/// failure AND the run completes within a bound (proving `test_defaults`
/// auto-resolved the capability-violation HITL await rather than blocking).
/// Live-session HITL surfacing on a capability violation is tested where a
/// live `HitlSeam` is used (`hitl/seam.rs` unit tests); it is not a Tester
/// concern.
#[tokio::test]
async fn promoted_scope_violation_folds_into_capability_failures_unattended() {
    let dir = TempDir::new().expect("tempdir");
    let path_arg = format!("{}/secret.txt", fwd(dir.path()));

    let fw = one_agent_fw(&[], &["allowed/**"], &["Write"]);
    let provider = WriteToolStub::new(path_arg, "x");

    let db_path = dir.path().join("runtime-tester.sqlite");
    let run = run_test_session_with_tier(
        &fw,
        "write the secret file",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
        Tier::Promoted,
    );
    // A bound proves the run never blocked on a HITL prompt — the
    // test-defaults seam auto-resolved the capability-violation await.
    let outcome = tokio::time::timeout(Duration::from_secs(10), run)
        .await
        .expect("the Tester must complete unattended — a HITL block would time out here")
        .expect("the assembled run completes");

    assert!(
        outcome
            .capability_failures
            .iter()
            .any(|f| f.needed == "write"),
        "the scope violation surfaces as a `write` CapabilityFailure on the outcome; got {:?}",
        outcome.capability_failures
    );
    assert!(
        !outcome.passed,
        "a capability failure forces passed = false"
    );
}
