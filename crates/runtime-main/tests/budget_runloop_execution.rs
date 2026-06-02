//! M08.7.E rung 5 — budget enforcement at the run loop assembled
//! regression + the four-threshold dispatch contract.
//!
//! The cluster-gate close contract (`docs/cluster-pattern.md` §1/§4): the
//! assembled tests drive the REAL `run_test_session_with` →
//! `AgentSdk::run_agent` → `drive_stream` multi-turn loop. The ONLY stub
//! is the provider (no live Anthropic — CLAUDE.md §10); the session
//! `BudgetEnforcer` construction (from the framework's `budget` block,
//! already threaded into `CapabilityWiring`), the per-turn `record_spend`
//! wire, and the `ThresholdAction` dispatch (events + the model swap + the
//! session stop) are all real.
//!
//! Grounded-claims (CLAUDE.md §4 rule 11 / gotcha #66): a `budget_exceeded`
//! event firing licenses ONLY "the event fired" — NOT "the run halted." So
//! the close test asserts the run-HALT side effect — no further provider
//! turn is issued after the cap is crossed, counted via the `seen` Arc —
//! not the event alone. The BEHAVIORAL close (a real Anthropic model run
//! halting at a tiny cap, no runaway spend) is the IRL gate (maintainer-run),
//! because the scripted stub is NOT the model.
//!
//! Scope (M08.7.E): the four `ThresholdAction`s are dispatched + asserted;
//! the load-bearing safety primitive is `HardStop` (the run halts). The
//! `Suspend` action records `budget_suspended` (suspend-and-record) — its
//! HITL **resume** half is NOT a permanent dead-end: it is the same
//! resolve-and-resume pattern as the rung-4 gap (suspend → human approves
//! the budget → resume), folded into ADR-0029 generalized to budget. Rung
//! 5 records the suspend; the resume joins that rung. The OS desktop
//! notifier for `Warn` is scoped out to TD-038 (the in-app `BudgetWarn`
//! toast covers v0.1).

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::{json, Value};
use tempfile::TempDir;

use runtime_core::event::AgentEvent;
use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{
    AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
    ProviderSupport,
};
use runtime_main::sdk::{McpDispatchError, McpDispatchOutcome, McpToolDispatch, SessionId};

// ── helpers ───────────────────────────────────────────────────────────

/// A schema-valid one-agent framework carrying a `budget` block with the
/// given `session_usd_cap`. `model_id` is the framework + agent model id
/// (the run's starting model, observed via the stub's captured configs and
/// fed to the downshift ladder). Default 50/75/90/100 percent thresholds
/// (no `actions` override).
fn fw_with_budget(model_id: &str, session_usd_cap: f64) -> Framework {
    serde_json::from_value(json!({
        "name": "m08-7-e-rung5",
        "version": "1.0.0",
        "description": "M08.7.E rung 5 budget-at-the-run-loop fixture",
        "model": { "provider": "anthropic", "id": model_id },
        "budget": { "session_usd_cap": session_usd_cap },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": model_id },
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
    .expect("the rung-5 fixture framework round-trips through the schema")
}

/// One `ProviderEvent::Usage` carrying the turn's token usage. `model` /
/// `cost_usd` are left empty/zero — the wire recomputes USD from the
/// `(input, output)` token breakdown via the provider's `estimate_cost`
/// (it does NOT consume the carried `cost_usd`), exactly as the production
/// Anthropic wrapper does (`anthropic.rs` rewrites these fields).
const fn usage(input: u64, output: u64) -> ProviderEvent {
    ProviderEvent::Usage {
        input_tokens: input,
        output_tokens: output,
        model: String::new(),
        cost_usd: 0.0,
    }
}

/// One MCP `echo` `ToolUse` — dispatched by [`EchoDispatch`] to an
/// `Invoked` outcome, so the turn joins `feedback.dispatched` and WOULD
/// drive a next turn absent a budget halt (the gotcha-#66 discriminator:
/// the run must halt because of the budget, not because the model stopped).
fn echo_tooluse(id: &str) -> ProviderEvent {
    ProviderEvent::ToolUse {
        id: id.to_string(),
        name: "echo".to_string(),
        input: json!({ "msg": "hi" }),
    }
}

/// Count the budget events of one kind in a trace.
fn count_budget_warns(trace: &[AgentEvent]) -> usize {
    trace
        .iter()
        .filter(|e| matches!(e, AgentEvent::BudgetWarn { .. }))
        .count()
}

// ── provider stub (no live Anthropic) ─────────────────────────────────

/// Yields one scripted `Vec<ProviderEvent>` per `stream()` call (empty —
/// loop-terminating — once exhausted) and CAPTURES every config it is
/// handed, so a test can observe how many provider turns the loop issued
/// (the run-halt signal) AND which model each turn used (the downshift
/// signal). `estimate_cost` is token-driven: `(input + output) × $0.001`,
/// so a turn's scripted `Usage` tokens determine its USD cost.
struct BudgetScriptStub {
    turns: Mutex<VecDeque<Vec<ProviderEvent>>>,
    seen: Arc<Mutex<Vec<AgentConfig>>>,
}

impl BudgetScriptStub {
    fn new(turns: Vec<Vec<ProviderEvent>>, seen: Arc<Mutex<Vec<AgentConfig>>>) -> Self {
        Self {
            turns: Mutex::new(turns.into()),
            seen,
        }
    }
}

#[async_trait]
impl LLMProvider for BudgetScriptStub {
    fn name(&self) -> &'static str {
        "m08-7-e-budget-script-stub"
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
    fn estimate_cost(&self, b: &CostBreakdown, _m: &str) -> f64 {
        // Token-driven so scripted Usage tokens drive the turn's USD cost:
        // $0.001 per token ⇒ 1000 tokens = $1.00, 400 tokens = $0.40.
        #[allow(clippy::cast_precision_loss)]
        let tokens = (b.input_tokens + b.output_tokens) as f64;
        tokens * 0.001
    }
}

/// Reused rung-4 MCP dispatch: resolves `echo` to `Invoked` (so the turn
/// joins `feedback.dispatched` and would drive a next turn), every other
/// name falls through (`None`).
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
            None
        }
    }
}

/// Drive a budget run through the real `run_test_session_with` loop with
/// the `EchoDispatch` MCP seam, returning the outcome trace + the captured
/// per-turn configs.
async fn run_budget_session(
    fw: &Framework,
    turns: Vec<Vec<ProviderEvent>>,
) -> (Vec<AgentEvent>, Vec<AgentConfig>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = BudgetScriptStub::new(turns, Arc::clone(&seen));
    let dir = TempDir::new().expect("tempdir");
    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        fw,
        "do the work",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        Some(Arc::new(EchoDispatch) as Arc<dyn McpToolDispatch>),
        SessionId::new(),
    )
    .await
    .expect("the assembled rung-5 run completes — a budget stop is not an Err");
    let configs = seen.lock().expect("seen lock").clone();
    (outcome.trace, configs)
}

// ── assembled regressions (the E.4.6 BDD close contract) ──────────────

/// E.4.1 — the BDD close contract + the mutation-gate target. A tiny
/// `session_usd_cap` ($0.0001) is exceeded by the first turn ($0.10); the
/// run HALTS: `budget_exceeded` is in the trace AND the provider stream is
/// NOT invoked again (exactly one turn). Turn 0 also dispatches an `echo`
/// (which would, absent the hard-stop, feed back and drive turn 1) and a
/// turn 1 IS scripted — so a missing/severed hard-stop break shows up as a
/// 2nd captured config. The counted run-halt, not the event alone (#66).
#[tokio::test]
async fn tiny_cap_hard_stops_the_run_and_issues_no_further_turn() {
    let fw = fw_with_budget("claude-haiku-4-5", 0.0001);
    let (trace, configs) = run_budget_session(
        &fw,
        vec![
            // Turn 0: $0.10 ≫ the $0.0001 cap ⇒ HardStop.
            vec![echo_tooluse("tu-0"), usage(100, 0)],
            // Turn 1: scripted but must NEVER run — the hard-stop halts the loop.
            vec![echo_tooluse("tu-1"), usage(100, 0)],
        ],
    )
    .await;

    // (1) the run reported a budget-stop reason: budget_exceeded is in the
    // TestOutcome trace, carrying the spend + cap.
    assert!(
        trace
            .iter()
            .any(|e| matches!(e, AgentEvent::BudgetExceeded { .. })),
        "a budget_exceeded event must be in the trace once the cap is crossed; trace={trace:?}"
    );

    // (2) LOAD-BEARING (rule 11 / gotcha #66): the run HALTED — exactly one
    // provider turn, despite turn 0 having dispatched an echo that would
    // otherwise feed back and drive turn 1. No runaway spend.
    assert_eq!(
        configs.len(),
        1,
        "the run must stop at the cap — no further provider turn; got {} turn(s)",
        configs.len()
    );
}

/// E.4.2 — `record_spend` is called PER TURN and the enforcer accumulates
/// across turns. Two turns of $0.40 each under a $1.00 cap: only the
/// cumulative $0.80 (80%) crosses the 50% warn. A wire that did not record
/// per turn (or did not accumulate) would never reach 50% on a single
/// $0.40 turn — so the presence of `budget_warn` proves per-turn
/// accumulation.
#[tokio::test]
async fn record_spend_accumulates_across_turns() {
    let fw = fw_with_budget("claude-haiku-4-5", 1.00);
    let (trace, configs) = run_budget_session(
        &fw,
        vec![
            vec![echo_tooluse("tu-0"), usage(400, 0)], // $0.40 — 40%, below warn
            vec![echo_tooluse("tu-1"), usage(400, 0)], // cumulative $0.80 — 80%, crosses warn
            vec![usage(10, 0)],                        // model stops
        ],
    )
    .await;

    assert!(
        count_budget_warns(&trace) >= 1,
        "two $0.40 turns must accumulate to 80% and cross the 50% warn — \
         a single turn or a non-accumulating wire never reaches it; trace={trace:?}"
    );
    assert_eq!(
        configs.len(),
        3,
        "the run continues through both spend turns + the stop turn; got {} turn(s)",
        configs.len()
    );
}

/// E.4.3 — a `Downshift` action swaps the model the NEXT turn actually uses
/// (gotcha: assert the swapped model on the captured config, not just that
/// the hook returned a new model). Opus start, $1.00 cap, turn 0 spends
/// $0.80 (crosses the 75% downshift); the ladder swaps opus→sonnet; turn 1
/// must run under `claude-sonnet-4-6`.
#[tokio::test]
async fn downshift_swaps_the_model_for_subsequent_turns() {
    let fw = fw_with_budget("claude-opus-4-7", 1.00);
    let (trace, configs) = run_budget_session(
        &fw,
        vec![
            vec![echo_tooluse("tu-0"), usage(800, 0)], // $0.80 — crosses downshift (75%)
            vec![usage(50, 0)],                        // runs under the swapped model, then stops
        ],
    )
    .await;

    // The downshift event reports the opus→sonnet swap.
    assert!(
        trace.iter().any(|e| matches!(
            e,
            AgentEvent::BudgetDownshift { from_model, to_model, .. }
                if from_model.contains("opus") && to_model.contains("sonnet")
        )),
        "a budget_downshift opus→sonnet must be emitted; trace={trace:?}"
    );

    // LOAD-BEARING: the NEXT turn actually used the cheaper model.
    assert!(
        configs.len() >= 2,
        "turn 1 must run so the swapped model is observable; got {} turn(s)",
        configs.len()
    );
    assert!(
        configs[1].model.contains("sonnet"),
        "after the downshift, the next turn must use the cheaper model; got {:?}",
        configs[1].model
    );
}

/// E.4.4 — a `Warn` action emits `budget_warn` and does NOT halt the run.
/// $1.00 cap, turn 0 spends $0.50 (exactly the 50% warn, below the 75%
/// downshift); turn 1 runs (warn did not stop the loop); no
/// `budget_exceeded`.
#[tokio::test]
async fn warn_emits_budget_warn_and_does_not_stop() {
    let fw = fw_with_budget("claude-haiku-4-5", 1.00);
    let (trace, configs) = run_budget_session(
        &fw,
        vec![
            vec![echo_tooluse("tu-0"), usage(500, 0)], // $0.50 — exactly the warn
            vec![usage(10, 0)],                        // runs (warn didn't halt), then stops
        ],
    )
    .await;

    assert!(
        count_budget_warns(&trace) >= 1,
        "crossing the 50% threshold must emit budget_warn; trace={trace:?}"
    );
    assert!(
        !trace
            .iter()
            .any(|e| matches!(e, AgentEvent::BudgetExceeded { .. })),
        "a warn must NOT emit budget_exceeded; trace={trace:?}"
    );
    assert_eq!(
        configs.len(),
        2,
        "the warn must not halt the run — turn 1 still runs; got {} turn(s)",
        configs.len()
    );
}

/// E.4.5 — idempotence at the wire level: a threshold that has fired does
/// not re-fire on subsequent turns. Turn 0 crosses the 50% warn ($0.60);
/// turns 1+2 add small spend that stays in the warn band (65%, 70%) without
/// crossing a new threshold. `budget_warn` must appear EXACTLY ONCE across
/// the whole run.
#[tokio::test]
async fn threshold_is_idempotent_across_turns() {
    let fw = fw_with_budget("claude-haiku-4-5", 1.00);
    let (trace, configs) = run_budget_session(
        &fw,
        vec![
            vec![echo_tooluse("tu-0"), usage(600, 0)], // $0.60 — crosses warn (50%)
            vec![echo_tooluse("tu-1"), usage(50, 0)],  // $0.65 — still in warn band
            vec![usage(50, 0)],                        // $0.70 — still in warn band, then stops
        ],
    )
    .await;

    assert_eq!(
        count_budget_warns(&trace),
        1,
        "budget_warn must fire exactly once — not re-fire each turn (idempotence); trace={trace:?}"
    );
    assert_eq!(
        configs.len(),
        3,
        "the run continues through all three turns (no halt below 90%); got {} turn(s)",
        configs.len()
    );
}

// ── additive coverage (v1.8 follow-up — net-new, separate from the
//    red→impl pair) — the isolated Suspend threshold ──────────────────────

/// Additive (maintainer-requested before the mutation gate) — the `Suspend`
/// (HITL) threshold in ISOLATION halts the run. The E.4.1 tiny-cap close
/// fires all four actions at once, so its halt could be attributed to
/// `HardStop`; this pins that `Suspend` ALONE stops the loop. A $0.95 turn
/// under a $1.00 cap crosses warn (50%) + downshift (75%) + suspend (90%)
/// but NOT hard-stop (100%): `budget_suspended` is emitted, the run halts
/// (exactly one turn — the suspend break wins over the co-dispatched echo
/// that would otherwise drive turn 1), and NO `budget_exceeded` fires. A
/// mutant that drops the `Suspend` arm's `budget_suspended` break (so the
/// echo feeds back and a 2nd turn runs) fails here.
#[tokio::test]
async fn suspend_threshold_emits_budget_suspended_and_halts_the_run() {
    let fw = fw_with_budget("claude-haiku-4-5", 1.00);
    let (trace, configs) = run_budget_session(
        &fw,
        vec![
            // Turn 0: $0.95 — crosses suspend (90%) but not hard-stop (100%).
            vec![echo_tooluse("tu-0"), usage(950, 0)],
            // Turn 1: scripted but must NEVER run — the suspend halts the loop.
            vec![echo_tooluse("tu-1"), usage(100, 0)],
        ],
    )
    .await;

    assert!(
        trace
            .iter()
            .any(|e| matches!(e, AgentEvent::BudgetSuspended { .. })),
        "crossing the 90% suspend threshold must emit budget_suspended; trace={trace:?}"
    );
    // It is a SUSPEND, not a hard-stop: no budget_exceeded fired.
    assert!(
        !trace
            .iter()
            .any(|e| matches!(e, AgentEvent::BudgetExceeded { .. })),
        "a suspend (90%) must NOT emit budget_exceeded (100%); trace={trace:?}"
    );
    // LOAD-BEARING: the suspend ALONE halted the run — exactly one provider
    // turn, despite turn 0 having dispatched an echo that would feed back.
    assert_eq!(
        configs.len(),
        1,
        "the suspend must halt the run — no further provider turn; got {} turn(s)",
        configs.len()
    );
}

/// Additive (mutation-gate kill) — OUTPUT tokens contribute to the per-turn
/// spend. The other tests drive cost via input tokens (`usage(N, 0)`), so a
/// mutant corrupting the `turn_output` accumulation (`+= → *=`, which on a
/// zero output is a no-op) survives. A turn of 0 input + 600 OUTPUT tokens
/// ($0.60) must cross the 50% warn on its own: with the output half dropped
/// (`turn_output` stuck at 0), cost is $0.00 → no warn → this test fails,
/// killing the mutant and pinning that output tokens count toward the cap.
#[tokio::test]
async fn output_tokens_contribute_to_budget_spend() {
    let fw = fw_with_budget("claude-haiku-4-5", 1.00);
    let (trace, _configs) = run_budget_session(
        &fw,
        vec![
            vec![usage(0, 600)], // 0 input, 600 output ⇒ $0.60 ⇒ crosses warn (50%)
        ],
    )
    .await;

    assert!(
        count_budget_warns(&trace) >= 1,
        "output tokens alone must drive spend across the 50% warn; trace={trace:?}"
    );
}
