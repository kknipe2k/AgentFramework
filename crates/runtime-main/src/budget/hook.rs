//! Downshift hook seam — spec §2a Downshift policy.
//!
//! v0.1 ships [`DefaultLadder`] which mirrors ARIA's tiers:
//!
//! ```text
//! opus    → sonnet  (any time downshift fires)
//! sonnet  → haiku   (only if remaining < 10% AND avg-task-cost > remaining/3)
//! haiku   → haiku   (no further downshift; once at haiku, HITL/hard-stop only)
//! ```
//!
//! Frameworks can override the ladder by setting `framework.budget.downshift_hook`
//! in framework JSON — the runtime resolves the named tool from the framework's
//! tool registry and dispatches it. v0.1 reads the field but the tool-dispatch
//! wiring is deferred to M5/M9 generators; until then [`DefaultLadder`] is the
//! only implementation, with the hook field carried in the policy struct so
//! later milestones can replace it without changing call sites.

/// Trait every downshift hook implements. Takes the current model and the
/// remaining budget snapshot; returns the model to switch to (or `None` if
/// no downshift is possible at this point).
pub trait DownshiftHook: Send + Sync {
    /// Pick the next model on the ladder, or return `None` to indicate
    /// no further downshift is available (the runtime escalates to HITL
    /// at that point per spec §2a).
    fn next_model(&self, current_model: &str, remaining: RemainingBudget) -> Option<String>;
}

/// Snapshot of remaining budget passed to the hook. v0.1 carries the bare
/// minimum the default ladder needs; v1.0 may extend with token-rate and
/// average-task-cost as those flow through the spend tracker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RemainingBudget {
    /// Spend so far on the tightest scope, USD.
    pub spent_usd: f64,
    /// Cap on the tightest scope, USD.
    pub cap_usd: f64,
    /// Average per-task cost so far, USD. `None` if no tasks have
    /// completed yet — the ladder treats this as "no data" and falls back
    /// to the conservative branch.
    pub avg_task_cost_usd: Option<f64>,
}

impl RemainingBudget {
    /// USD remaining under the tightest cap. Saturates at zero for spend
    /// over cap (hard-stop has already fired in that case but the value
    /// is well-defined for the hook).
    #[must_use]
    pub fn remaining_usd(&self) -> f64 {
        (self.cap_usd - self.spent_usd).max(0.0)
    }

    /// `spent_usd / cap_usd` clamped to `[0.0, 1.0]`. Caller side uses
    /// this to drive the spec's "remaining < 10%" branch in
    /// [`DefaultLadder::next_model`].
    #[must_use]
    pub fn remaining_fraction(&self) -> f64 {
        if self.cap_usd <= 0.0 {
            return 0.0;
        }
        ((self.cap_usd - self.spent_usd) / self.cap_usd).clamp(0.0, 1.0)
    }
}

/// Hardcoded `opus → sonnet → haiku` ladder per spec §2a Downshift policy.
///
/// Matching is by model-id prefix so the ladder works against the actual
/// API ids (`claude-opus-4-7`, `claude-sonnet-4-6`, `claude-haiku-4-5`)
/// without baking the dated revisions in. CLAUDE.md §3 keeps the latest
/// IDs current; v1.0 should source these from `LLMProvider::list_models`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultLadder;

impl DefaultLadder {
    /// Construct a fresh ladder. No state.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

const OPUS_TARGET: &str = "claude-sonnet-4-6";
const SONNET_TARGET: &str = "claude-haiku-4-5";

impl DownshiftHook for DefaultLadder {
    fn next_model(&self, current_model: &str, remaining: RemainingBudget) -> Option<String> {
        let tier = classify_tier(current_model);
        match tier {
            ModelTier::Opus => Some(OPUS_TARGET.to_string()),
            ModelTier::Sonnet => {
                // Spec §2a: sonnet → haiku only when remaining < 10% AND
                // avg-task-cost > remaining/3 (avoid stranding tasks).
                if remaining.remaining_fraction() >= 0.10 {
                    return None;
                }
                let avg = remaining.avg_task_cost_usd?;
                if avg > remaining.remaining_usd() / 3.0 {
                    Some(SONNET_TARGET.to_string())
                } else {
                    None
                }
            }
            ModelTier::Haiku | ModelTier::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelTier {
    Opus,
    Sonnet,
    Haiku,
    Unknown,
}

fn classify_tier(model: &str) -> ModelTier {
    let lower = model.to_ascii_lowercase();
    if lower.contains("opus") {
        ModelTier::Opus
    } else if lower.contains("sonnet") {
        ModelTier::Sonnet
    } else if lower.contains("haiku") {
        ModelTier::Haiku
    } else {
        ModelTier::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remaining(spent: f64, cap: f64, avg: Option<f64>) -> RemainingBudget {
        RemainingBudget {
            spent_usd: spent,
            cap_usd: cap,
            avg_task_cost_usd: avg,
        }
    }

    #[test]
    fn opus_always_downshifts_to_sonnet() {
        let l = DefaultLadder::new();
        assert_eq!(
            l.next_model("claude-opus-4-7", remaining(2.5, 5.0, None))
                .as_deref(),
            Some(OPUS_TARGET),
            "opus should downshift to sonnet at any spend level"
        );
    }

    #[test]
    fn opus_unknown_revision_still_downshifts() {
        let l = DefaultLadder::new();
        assert_eq!(
            l.next_model("claude-opus-future", remaining(0.0, 1.0, None))
                .as_deref(),
            Some(OPUS_TARGET)
        );
    }

    #[test]
    fn sonnet_holds_when_remaining_above_ten_percent() {
        let l = DefaultLadder::new();
        // 50% remaining — sonnet stays at sonnet (no further downshift).
        assert!(l
            .next_model("claude-sonnet-4-6", remaining(2.5, 5.0, Some(0.5)))
            .is_none());
    }

    #[test]
    fn sonnet_holds_below_ten_percent_when_avg_task_cost_low() {
        let l = DefaultLadder::new();
        // 5% remaining, but avg task cost = $0.05 << remaining/3 = $0.083.
        assert!(l
            .next_model("claude-sonnet-4-6", remaining(4.75, 5.0, Some(0.05)))
            .is_none());
    }

    #[test]
    fn sonnet_downshifts_when_remaining_low_and_avg_task_high() {
        let l = DefaultLadder::new();
        // 5% remaining ($0.25), avg task = $0.50 > $0.25/3 = $0.083.
        assert_eq!(
            l.next_model("claude-sonnet-4-6", remaining(4.75, 5.0, Some(0.50)))
                .as_deref(),
            Some(SONNET_TARGET)
        );
    }

    #[test]
    fn sonnet_no_avg_treats_as_no_downshift() {
        let l = DefaultLadder::new();
        // Low remaining, no avg signal — defer to HITL/hard-stop.
        assert!(l
            .next_model("claude-sonnet-4-6", remaining(4.95, 5.0, None))
            .is_none());
    }

    #[test]
    fn haiku_does_not_downshift_further() {
        let l = DefaultLadder::new();
        assert!(l
            .next_model("claude-haiku-4-5", remaining(4.95, 5.0, Some(0.5)))
            .is_none());
    }

    #[test]
    fn unknown_model_does_not_downshift() {
        let l = DefaultLadder::new();
        // Unrecognized model id — no downshift; runtime escalates to HITL.
        assert!(l.next_model("gpt-4", remaining(0.0, 5.0, None)).is_none());
    }

    #[test]
    fn remaining_fraction_clamps_above_cap() {
        let r = remaining(10.0, 5.0, None);
        assert!((r.remaining_fraction() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_fraction_clamps_negative_spend() {
        // Defensive: a negative spent value (shouldn't happen) clamps cleanly.
        let r = remaining(-1.0, 5.0, None);
        assert!((r.remaining_fraction() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_fraction_zero_cap_returns_zero() {
        let r = remaining(0.0, 0.0, None);
        assert!((r.remaining_fraction() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_usd_saturates_at_zero() {
        let r = remaining(7.0, 5.0, None);
        assert!((r.remaining_usd() - 0.0).abs() < f64::EPSILON);
    }
}
