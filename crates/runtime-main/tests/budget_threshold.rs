//! Budget threshold integration test — M04 Stage F (spec §2a).
//!
//! Drives the [`BudgetEnforcer`] against a deterministic stream of
//! incremental spend amounts and asserts the four threshold actions
//! fire in 50 → 75 → 90 → 100 order. Per spec §2a:
//!
//! - 50% → `Warn`
//! - 75% → `Downshift` (also invokes the downshift hook)
//! - 90% → `Suspend` (the renderer routes this through the
//!   `on_budget_threshold` HITL trigger; that wiring is the SDK
//!   turn-loop integration site)
//! - 100% → `HardStop` (renderer surfaces the terminal banner; the
//!   runtime dispatches drone `StopProcess` — that wire-up is the SDK
//!   integration site, exercised under the unit tests for
//!   `BudgetEnforcer`)
//!
//! Stage F's enforcer is the standalone primitive.
//!
//! The SDK turn-loop integration site is out-of-scope for this test
//! (covered in unit tests + the Tauri command surface tests); this
//! integration test pins the enforcer's contract against the
//! four-threshold order invariant + the downshift hook invocation.

use runtime_main::budget::{
    BudgetEnforcer, BudgetScopeCap, DefaultLadder, DownshiftHook, RemainingBudget, ThresholdAction,
};

#[test]
fn enforcer_fires_all_four_actions_in_order_under_session_cap() {
    // Session cap $1.00 with default 50/75/90/100 thresholds.
    let mut enforcer =
        BudgetEnforcer::new(vec![BudgetScopeCap::session(1.00)], None, None, None, None);

    // Incremental spends: small steps, then jumps to land at each percent.
    // - 0.10 → 10% (below warn)
    // - 0.40 → 50% (warn)
    // - 0.25 → 75% (downshift)
    // - 0.15 → 90% (suspend)
    // - 0.10 → 100% (hard stop)
    let mut fired: Vec<ThresholdAction> = Vec::new();
    for delta in [0.10, 0.40, 0.25, 0.15, 0.10] {
        fired.extend(enforcer.record_spend(delta));
    }

    assert_eq!(
        fired.len(),
        4,
        "exactly four threshold actions, got {fired:?}"
    );
    assert!(
        matches!(fired[0], ThresholdAction::Warn { percent: 50, .. }),
        "first action is Warn @50%, got {:?}",
        fired[0]
    );
    assert!(
        matches!(fired[1], ThresholdAction::Downshift { percent: 75, .. }),
        "second action is Downshift @75%, got {:?}",
        fired[1]
    );
    assert!(
        matches!(fired[2], ThresholdAction::Suspend { percent: 90, .. }),
        "third action is Suspend @90%, got {:?}",
        fired[2]
    );
    assert!(
        matches!(fired[3], ThresholdAction::HardStop { percent: 100, .. }),
        "fourth action is HardStop @100%, got {:?}",
        fired[3]
    );

    // Idempotent — further spend past 100% does not re-fire.
    let extra = enforcer.record_spend(0.05);
    assert!(
        extra.is_empty(),
        "post-hard-stop spend must not re-fire thresholds, got {extra:?}"
    );
}

#[test]
fn enforcer_downshift_hook_invokes_at_75_percent() {
    // Same setup; when Downshift fires the renderer/SDK calls the hook
    // to pick the next model. This test exercises the chained invocation.
    let mut enforcer =
        BudgetEnforcer::new(vec![BudgetScopeCap::session(1.00)], None, None, None, None);
    let hook = DefaultLadder::new();
    let mut downshift_picks: Vec<String> = Vec::new();
    let mut current_model = String::from("claude-opus-4-7");

    for delta in [0.30, 0.20, 0.30] {
        for action in enforcer.record_spend(delta) {
            if let ThresholdAction::Downshift {
                spent_usd, cap_usd, ..
            } = action
            {
                let next = hook.next_model(
                    &current_model,
                    RemainingBudget {
                        spent_usd,
                        cap_usd,
                        avg_task_cost_usd: None,
                    },
                );
                if let Some(picked) = next {
                    downshift_picks.push(picked.clone());
                    current_model = picked;
                }
            }
        }
    }

    assert_eq!(
        downshift_picks.len(),
        1,
        "downshift hook invoked exactly once at 75% threshold"
    );
    assert!(
        downshift_picks[0].contains("sonnet"),
        "opus downshifts to sonnet, got {}",
        downshift_picks[0]
    );
}

#[test]
fn enforcer_emits_correct_scope_when_framework_cap_is_tightest() {
    // session=$5, framework=$2; framework wins.
    let mut enforcer = BudgetEnforcer::new(
        vec![
            BudgetScopeCap::session(5.00),
            BudgetScopeCap::framework(2.00),
        ],
        None,
        None,
        None,
        None,
    );
    let mut framework_spend = 0.0;
    let mut all_fired = Vec::new();
    for delta in [1.00, 0.50, 0.30, 0.20] {
        framework_spend += delta;
        all_fired.extend(enforcer.record_spend_with_scopes(delta, Some(framework_spend), None));
    }
    for action in &all_fired {
        let scope_ok = match action {
            ThresholdAction::Warn { scope, .. }
            | ThresholdAction::Downshift { scope, .. }
            | ThresholdAction::Suspend { scope, .. }
            | ThresholdAction::HardStop { scope, .. } => *scope,
        };
        assert_eq!(
            scope_ok,
            runtime_core::generated::budget::BudgetScope::Framework,
            "tightest cap wins → framework scope, got {action:?}"
        );
    }
}
