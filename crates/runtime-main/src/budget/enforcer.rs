//! Budget enforcer — spec §2a threshold-crossing dispatch.
//!
//! Holds three scope caps (session / framework / global) and four
//! percent thresholds; each `record_spend` call returns zero or more
//! [`ThresholdAction`]s describing what just crossed. The caller (SDK
//! turn loop) translates actions into events + side effects.
//!
//! Idempotence: once a threshold has fired for a (cap, percent) pair,
//! re-recording spend at the same percentage does not re-fire. The
//! tracked-fired set resets when the tightest cap changes (e.g. the
//! framework starts a new scope) or when the enforcer is `reset()`.
//!
//! Tightest-cap-wins per spec §2a: if `session_cap=$5`, `framework_cap=$3`,
//! `global_cap=$10`, the framework cap wins for percent math. The other
//! scopes still contribute to overall accounting; only the percent
//! computation picks the tightest.

use runtime_core::generated::budget::BudgetScope;

use crate::providers::CostBreakdown;

/// One scope's cap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BudgetScopeCap {
    /// Which scope this cap belongs to.
    pub scope: BudgetScope,
    /// USD cap. `None` means "no cap configured for this scope".
    pub cap_usd: Option<f64>,
}

impl BudgetScopeCap {
    /// Build a session-scope cap.
    #[must_use]
    pub const fn session(cap_usd: f64) -> Self {
        Self {
            scope: BudgetScope::Session,
            cap_usd: Some(cap_usd),
        }
    }

    /// Build a framework-scope cap.
    #[must_use]
    pub const fn framework(cap_usd: f64) -> Self {
        Self {
            scope: BudgetScope::Framework,
            cap_usd: Some(cap_usd),
        }
    }

    /// Build a global-scope cap.
    #[must_use]
    pub const fn global(cap_usd: f64) -> Self {
        Self {
            scope: BudgetScope::Global,
            cap_usd: Some(cap_usd),
        }
    }
}

/// The four threshold actions described by spec §2a. Returned by
/// [`BudgetEnforcer::record_spend`] in firing order (lowest percent first)
/// so callers can dispatch them deterministically.
#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdAction {
    /// Emit `budget_warn` + UI toast.
    Warn {
        /// Tightest-cap scope.
        scope: BudgetScope,
        /// Spend so far against the tightest cap.
        spent_usd: f64,
        /// The tightest cap value.
        cap_usd: f64,
        /// Percentage crossed (the configured `warn_at_percent`).
        percent: u32,
    },
    /// Invoke downshift hook + emit `budget_downshift`.
    Downshift {
        /// Tightest-cap scope.
        scope: BudgetScope,
        /// Spend so far against the tightest cap.
        spent_usd: f64,
        /// The tightest cap value.
        cap_usd: f64,
        /// Percentage crossed (the configured `downshift_at_percent`).
        percent: u32,
    },
    /// Trigger `on_budget_threshold` HITL flow + emit `budget_suspended`.
    Suspend {
        /// Tightest-cap scope.
        scope: BudgetScope,
        /// Spend so far against the tightest cap.
        spent_usd: f64,
        /// The tightest cap value.
        cap_usd: f64,
        /// Percentage crossed (the configured `hitl_at_percent`).
        percent: u32,
    },
    /// Dispatch drone `StopProcess` + emit `budget_exceeded`.
    HardStop {
        /// Tightest-cap scope.
        scope: BudgetScope,
        /// Spend so far against the tightest cap.
        spent_usd: f64,
        /// The tightest cap value.
        cap_usd: f64,
        /// Percentage crossed (the configured `hard_stop_at_percent`).
        percent: u32,
    },
}

/// Which threshold a fired action corresponds to. Used internally for
/// idempotence tracking; exposed for tests that want to assert on the
/// fired set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BudgetThreshold {
    /// `warn_at_percent` (default 50).
    Warn,
    /// `downshift_at_percent` (default 75).
    Downshift,
    /// `hitl_at_percent` (default 90).
    Suspend,
    /// `hard_stop_at_percent` (default 100).
    HardStop,
}

/// Stage F enforcer holding the four percent thresholds + three scope caps.
#[derive(Debug, Clone)]
pub struct BudgetEnforcer {
    /// Session / framework / global caps; sparse — only configured scopes
    /// appear here.
    caps: Vec<BudgetScopeCap>,
    /// Threshold percentages.
    warn_at: u32,
    downshift_at: u32,
    suspend_at: u32,
    hard_stop_at: u32,
    /// Idempotence: per (`cap_usd`, threshold) we record whether we've
    /// already fired so re-recording at the same percent doesn't
    /// duplicate. Keyed by cap as integer-microcents to dodge f64 hashing.
    fired: std::collections::HashSet<(i64, BudgetThreshold)>,
    /// Current spent so far against the SESSION accumulator.
    ///
    /// Framework + global spend tracking is the caller's job (per spec §2a
    /// Stage F scope — those would draw from longer-lived persistence).
    /// `record_spend` adds to this; the caller can also pass per-scope
    /// running totals via [`record_spend_with_scopes`].
    spent_session_usd: f64,
}

impl BudgetEnforcer {
    /// New enforcer with the given caps + threshold percentages.
    /// `caps` may include zero, one, two, or three scope entries.
    /// Threshold percentages default to 50/75/90/100 if `None` per spec §2a.
    #[must_use]
    pub fn new(
        caps: Vec<BudgetScopeCap>,
        warn_at: Option<u32>,
        downshift_at: Option<u32>,
        suspend_at: Option<u32>,
        hard_stop_at: Option<u32>,
    ) -> Self {
        Self {
            caps,
            warn_at: warn_at.unwrap_or(50),
            downshift_at: downshift_at.unwrap_or(75),
            suspend_at: suspend_at.unwrap_or(90),
            hard_stop_at: hard_stop_at.unwrap_or(100),
            fired: std::collections::HashSet::new(),
            spent_session_usd: 0.0,
        }
    }

    /// Compute incremental USD cost from a [`CostBreakdown`] at a known
    /// per-million input/output pricing. Pure helper extracted so tests
    /// can drive the enforcer deterministically without going through
    /// `LLMProvider::estimate_cost`.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "u64 token counts up to 2^53 are exact in f64; spend math uses USD and is dollar-precision anyway"
    )]
    pub fn cost_from(
        breakdown: &CostBreakdown,
        input_per_million: f64,
        output_per_million: f64,
    ) -> f64 {
        // Mirror `CostBreakdown`'s cache-aware accounting:
        //   plain input + 1.25× 5m-cache-writes + 2.0× 1h-cache-writes
        //   + 0.1× cache-reads + output
        let cb = breakdown;
        let input_units = (cb.cache_reads as f64).mul_add(
            0.10,
            (cb.cache_1h_writes as f64).mul_add(
                2.0,
                (cb.cache_5m_writes as f64).mul_add(1.25, cb.input_tokens as f64),
            ),
        );
        let input_cost = input_units * input_per_million / 1_000_000.0;
        let output_cost = (cb.output_tokens as f64) * output_per_million / 1_000_000.0;
        input_cost + output_cost
    }

    /// Record additional spend and return any threshold actions that
    /// fired as a result. `incremental_usd` is the cost of *this* call;
    /// the enforcer accumulates internally.
    ///
    /// Per spec §2a: the tightest applicable cap wins (lowest non-`None`
    /// cap across configured scopes). Thresholds fire in order
    /// (warn → downshift → suspend → hard-stop) so callers dispatch them
    /// deterministically.
    pub fn record_spend(&mut self, incremental_usd: f64) -> Vec<ThresholdAction> {
        self.record_spend_with_scopes(incremental_usd, None, None)
    }

    /// Same as [`Self::record_spend`] but with caller-supplied running
    /// totals for the framework + global scopes. Session spend is tracked
    /// internally. Returns the set of fired actions.
    ///
    /// The tightest-cap-wins evaluation works per scope: each configured
    /// scope's `(cap, spend)` pair is scored, the lowest `cap_usd` wins,
    /// and `percent = round(spend / tightest_cap * 100)` drives the
    /// threshold crossings.
    pub fn record_spend_with_scopes(
        &mut self,
        incremental_usd: f64,
        framework_spend_usd: Option<f64>,
        global_spend_usd: Option<f64>,
    ) -> Vec<ThresholdAction> {
        self.spent_session_usd += incremental_usd;
        let Some((tightest, spend)) = self.tightest_scope(framework_spend_usd, global_spend_usd)
        else {
            return Vec::new();
        };
        // Percent crossing in [0, 100]; .floor() ensures we round down so a
        // 49.9% spend doesn't trigger the 50% warn. Cap is non-zero here
        // (tightest_scope filters out cap_usd <= 0).
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "spend/cap*100 is bounded [0, ~200] in normal use; truncation/sign-loss are not real concerns"
        )]
        let percent = (spend / tightest.cap_usd.unwrap_or(0.0) * 100.0).floor() as u32;
        let cap_key = cap_microcents(tightest.cap_usd.unwrap_or(0.0));
        let mut out = Vec::new();
        for (threshold, at, build) in &[
            (
                BudgetThreshold::Warn,
                self.warn_at,
                build_action as fn(BudgetThreshold, BudgetScope, f64, f64, u32) -> ThresholdAction,
            ),
            (
                BudgetThreshold::Downshift,
                self.downshift_at,
                build_action as fn(BudgetThreshold, BudgetScope, f64, f64, u32) -> ThresholdAction,
            ),
            (
                BudgetThreshold::Suspend,
                self.suspend_at,
                build_action as fn(BudgetThreshold, BudgetScope, f64, f64, u32) -> ThresholdAction,
            ),
            (
                BudgetThreshold::HardStop,
                self.hard_stop_at,
                build_action as fn(BudgetThreshold, BudgetScope, f64, f64, u32) -> ThresholdAction,
            ),
        ] {
            if percent >= *at && self.fired.insert((cap_key, *threshold)) {
                out.push(build(
                    *threshold,
                    tightest.scope,
                    spend,
                    tightest.cap_usd.unwrap_or(0.0),
                    *at,
                ));
            }
        }
        out
    }

    /// Identify the tightest cap across configured scopes plus the
    /// matching spend total. Returns `None` if no caps are configured.
    fn tightest_scope(
        &self,
        framework_spend_usd: Option<f64>,
        global_spend_usd: Option<f64>,
    ) -> Option<(BudgetScopeCap, f64)> {
        let mut best: Option<(BudgetScopeCap, f64)> = None;
        for cap in &self.caps {
            let Some(cap_usd) = cap.cap_usd else { continue };
            if cap_usd <= 0.0 {
                continue;
            }
            let spend = match cap.scope {
                BudgetScope::Session => self.spent_session_usd,
                BudgetScope::Framework => framework_spend_usd.unwrap_or(self.spent_session_usd),
                BudgetScope::Global => global_spend_usd.unwrap_or(self.spent_session_usd),
            };
            let is_tighter = best
                .as_ref()
                .map_or(true, |(b, _)| cap_usd < b.cap_usd.unwrap_or(f64::MAX));
            if is_tighter {
                best = Some((*cap, spend));
            }
        }
        best
    }

    /// Reset fired-threshold tracking. Used on session restart or
    /// model-tier change (after a downshift the enforcer continues firing
    /// against the same caps; the higher thresholds may fire again as
    /// spend accumulates).
    pub fn reset_fired(&mut self) {
        self.fired.clear();
    }

    /// Total session spend recorded so far (USD).
    #[must_use]
    pub const fn spent_session_usd(&self) -> f64 {
        self.spent_session_usd
    }

    /// How many thresholds have fired so far. Used by tests.
    #[must_use]
    pub fn fired_count(&self) -> usize {
        self.fired.len()
    }
}

const fn build_action(
    threshold: BudgetThreshold,
    scope: BudgetScope,
    spent_usd: f64,
    cap_usd: f64,
    percent: u32,
) -> ThresholdAction {
    match threshold {
        BudgetThreshold::Warn => ThresholdAction::Warn {
            scope,
            spent_usd,
            cap_usd,
            percent,
        },
        BudgetThreshold::Downshift => ThresholdAction::Downshift {
            scope,
            spent_usd,
            cap_usd,
            percent,
        },
        BudgetThreshold::Suspend => ThresholdAction::Suspend {
            scope,
            spent_usd,
            cap_usd,
            percent,
        },
        BudgetThreshold::HardStop => ThresholdAction::HardStop {
            scope,
            spent_usd,
            cap_usd,
            percent,
        },
    }
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "cap is a USD amount; the i64 range covers $90 quadrillion at 5-decimal precision"
)]
fn cap_microcents(cap_usd: f64) -> i64 {
    // Stable integer key for hashing; preserves cap identity across
    // multiple record_spend calls without f64 hash quirks.
    (cap_usd * 100_000.0).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_enforcer(cap_usd: f64) -> BudgetEnforcer {
        BudgetEnforcer::new(
            vec![BudgetScopeCap::session(cap_usd)],
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn no_caps_returns_no_actions() {
        let mut e = BudgetEnforcer::new(Vec::new(), None, None, None, None);
        assert!(e.record_spend(10.0).is_empty());
    }

    #[test]
    fn below_warn_threshold_returns_empty() {
        let mut e = session_enforcer(1.00);
        let actions = e.record_spend(0.10); // 10% — below warn at 50%
        assert!(actions.is_empty());
    }

    #[test]
    fn at_warn_threshold_emits_warn_only() {
        let mut e = session_enforcer(1.00);
        let actions = e.record_spend(0.50); // 50% — exactly warn
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            ThresholdAction::Warn { scope, percent, .. } => {
                assert_eq!(*scope, BudgetScope::Session);
                assert_eq!(*percent, 50);
            }
            other => panic!("expected Warn, got {other:?}"),
        }
    }

    #[test]
    fn crossing_two_thresholds_in_one_record_fires_both_in_order() {
        let mut e = session_enforcer(1.00);
        // Jump from 0% to 80% in one call — should fire Warn (50%) +
        // Downshift (75%) in order, not Suspend (90%) or HardStop (100%).
        let actions = e.record_spend(0.80);
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], ThresholdAction::Warn { .. }));
        assert!(matches!(actions[1], ThresholdAction::Downshift { .. }));
    }

    #[test]
    fn crossing_all_four_thresholds_in_one_record_fires_in_order() {
        let mut e = session_enforcer(1.00);
        let actions = e.record_spend(1.00); // 100% — all four
        assert_eq!(actions.len(), 4);
        assert!(matches!(actions[0], ThresholdAction::Warn { .. }));
        assert!(matches!(actions[1], ThresholdAction::Downshift { .. }));
        assert!(matches!(actions[2], ThresholdAction::Suspend { .. }));
        assert!(matches!(actions[3], ThresholdAction::HardStop { .. }));
    }

    #[test]
    fn idempotent_does_not_re_fire_same_threshold() {
        let mut e = session_enforcer(1.00);
        let _ = e.record_spend(0.51); // fires Warn
        let actions = e.record_spend(0.05); // 56% — no new threshold
        assert!(actions.is_empty());
    }

    #[test]
    fn idempotent_until_next_threshold() {
        let mut e = session_enforcer(1.00);
        let _ = e.record_spend(0.51); // Warn fires
        let actions = e.record_spend(0.25); // 76% — Downshift fires; Warn does not re-fire
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], ThresholdAction::Downshift { .. }));
    }

    #[test]
    fn tightest_cap_wins_session_v_framework() {
        // session=$5, framework=$3 — framework wins.
        let mut e = BudgetEnforcer::new(
            vec![
                BudgetScopeCap::session(5.00),
                BudgetScopeCap::framework(3.00),
            ],
            None,
            None,
            None,
            None,
        );
        // Spend $1.50 — that's 50% of $3 (framework) but 30% of $5 (session).
        // Framework wins → Warn fires.
        let actions = e.record_spend_with_scopes(1.50, Some(1.50), None);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            ThresholdAction::Warn { scope, cap_usd, .. } => {
                assert_eq!(*scope, BudgetScope::Framework);
                assert!((cap_usd - 3.00).abs() < f64::EPSILON);
            }
            other => panic!("expected Warn from framework scope, got {other:?}"),
        }
    }

    #[test]
    fn tightest_cap_wins_with_global() {
        // session=$5, global=$10, framework=$3 — framework still wins.
        let mut e = BudgetEnforcer::new(
            vec![
                BudgetScopeCap::session(5.00),
                BudgetScopeCap::framework(3.00),
                BudgetScopeCap::global(10.00),
            ],
            None,
            None,
            None,
            None,
        );
        let actions = e.record_spend_with_scopes(1.50, Some(1.50), Some(1.50));
        assert!(matches!(
            actions[0],
            ThresholdAction::Warn {
                scope: BudgetScope::Framework,
                ..
            }
        ));
    }

    #[test]
    fn zero_cap_is_ignored() {
        // A zero-or-negative cap is treated as "not configured" so the
        // enforcer doesn't divide by zero.
        let mut e = BudgetEnforcer::new(
            vec![
                BudgetScopeCap {
                    scope: BudgetScope::Session,
                    cap_usd: Some(0.0),
                },
                BudgetScopeCap::framework(5.00),
            ],
            None,
            None,
            None,
            None,
        );
        let actions = e.record_spend_with_scopes(2.50, Some(2.50), None); // 50% of $5
        match &actions[0] {
            ThresholdAction::Warn { scope, .. } => assert_eq!(*scope, BudgetScope::Framework),
            other => panic!("expected Warn from framework, got {other:?}"),
        }
    }

    #[test]
    fn custom_thresholds_respected() {
        let mut e = BudgetEnforcer::new(
            vec![BudgetScopeCap::session(1.00)],
            Some(25), // warn at 25%
            Some(50), // downshift at 50%
            Some(75), // suspend at 75%
            Some(99), // hard stop at 99%
        );
        let actions = e.record_spend(0.30); // 30% — fires Warn (25%) only
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            ThresholdAction::Warn { percent, .. } => assert_eq!(*percent, 25),
            other => panic!("expected Warn at 25%, got {other:?}"),
        }
    }

    #[test]
    fn reset_fired_allows_re_fire() {
        let mut e = session_enforcer(1.00);
        let _ = e.record_spend(0.51); // fires Warn
        e.reset_fired();
        let actions = e.record_spend(0.01); // back at 52%, Warn re-fires
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], ThresholdAction::Warn { .. }));
    }

    #[test]
    fn cost_from_simple_breakdown() {
        let b = CostBreakdown::simple(1_000_000, 0);
        // $3/M input → $3 for 1M input tokens.
        assert!((BudgetEnforcer::cost_from(&b, 3.00, 15.00) - 3.00).abs() < 1e-9);
    }

    #[test]
    fn cost_from_cache_writes_are_priced_higher() {
        // 1M tokens of 5m-cache writes at 1.25× input → 1.25× $3 = $3.75
        let b = CostBreakdown {
            input_tokens: 0,
            output_tokens: 0,
            cache_5m_writes: 1_000_000,
            cache_1h_writes: 0,
            cache_reads: 0,
        };
        assert!((BudgetEnforcer::cost_from(&b, 3.00, 15.00) - 3.75).abs() < 1e-9);
    }

    #[test]
    fn cost_from_cache_reads_are_priced_lower() {
        // 1M cache-reads at 0.1× input → $0.30
        let b = CostBreakdown {
            input_tokens: 0,
            output_tokens: 0,
            cache_5m_writes: 0,
            cache_1h_writes: 0,
            cache_reads: 1_000_000,
        };
        assert!((BudgetEnforcer::cost_from(&b, 3.00, 15.00) - 0.30).abs() < 1e-9);
    }

    #[test]
    fn cost_from_output_priced_separately() {
        let b = CostBreakdown::simple(0, 1_000_000);
        // $15/M output → $15.
        assert!((BudgetEnforcer::cost_from(&b, 3.00, 15.00) - 15.00).abs() < 1e-9);
    }

    #[test]
    fn spent_session_usd_accumulates() {
        let mut e = session_enforcer(10.00);
        let _ = e.record_spend(0.10);
        let _ = e.record_spend(0.20);
        assert!((e.spent_session_usd() - 0.30).abs() < 1e-9);
    }

    #[test]
    fn fired_count_grows_with_thresholds() {
        let mut e = session_enforcer(1.00);
        assert_eq!(e.fired_count(), 0);
        let _ = e.record_spend(0.51);
        assert_eq!(e.fired_count(), 1);
        let _ = e.record_spend(0.26); // 77% — Downshift
        assert_eq!(e.fired_count(), 2);
    }

    #[test]
    fn framework_only_cap_uses_framework_spend() {
        // Only framework cap is configured. Caller supplies framework_spend
        // explicitly; session accumulator is informational.
        let mut e = BudgetEnforcer::new(
            vec![BudgetScopeCap::framework(2.00)],
            None,
            None,
            None,
            None,
        );
        let actions = e.record_spend_with_scopes(0.0, Some(1.50), None); // 75% of $2
        assert_eq!(actions.len(), 2);
        assert!(matches!(
            actions[0],
            ThresholdAction::Warn {
                scope: BudgetScope::Framework,
                ..
            }
        ));
        assert!(matches!(
            actions[1],
            ThresholdAction::Downshift {
                scope: BudgetScope::Framework,
                ..
            }
        ));
    }

    #[test]
    fn global_only_cap_uses_global_spend() {
        let mut e =
            BudgetEnforcer::new(vec![BudgetScopeCap::global(10.00)], None, None, None, None);
        let actions = e.record_spend_with_scopes(0.0, None, Some(9.00)); // 90% of $10
                                                                         // Warn + Downshift + Suspend (90%) fire; HardStop does not (100%).
        assert_eq!(actions.len(), 3);
        assert!(matches!(
            actions.last().unwrap(),
            ThresholdAction::Suspend {
                scope: BudgetScope::Global,
                ..
            }
        ));
    }

    #[test]
    fn over_cap_clamps_to_caller_input_not_panic() {
        // Spending past the cap should fire HardStop, not panic.
        let mut e = session_enforcer(1.00);
        let actions = e.record_spend(2.00);
        assert!(actions
            .iter()
            .any(|a| matches!(a, ThresholdAction::HardStop { .. })));
    }
}
