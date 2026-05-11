//! HITL policy evaluator — spec §6a (9 trigger types).
//!
//! Pure logic: given a `HitlPolicy` (loaded from framework JSON) + a
//! [`HitlContext`] (the trigger condition that just fired), returns a
//! [`ResolvedTrigger`] when the trigger is enabled, or `None` when the
//! framework configured it disabled.
//!
//! Exhaustive matching over the 9-trigger enum is enforced by the compiler;
//! a future trigger added to [`runtime_core::generated::hitl::HitlTrigger`]
//! breaks this module's compile until handled.
//!
//! v0.1 STANDARD-mode-hardcoded (CLAUDE.md §3). Mode-keyed HITL overrides
//! in framework JSON are loaded by the framework loader and reach this
//! module already mode-resolved — the evaluator itself is mode-agnostic.
//!
//! Safety primitive: ≥95% coverage gate per CLAUDE.md §5.

use runtime_core::event::{HitlTriggerRef, HitlUiVariantRef};
use runtime_core::generated::hitl::{HitlPolicy, HitlTriggerPolicy, HitlUiVariant};

/// Default UI variant per trigger type, per spec §6a table. Used when the
/// framework JSON does not set a per-trigger `ui` override.
#[must_use]
pub const fn default_ui_for(trigger: HitlTriggerRef) -> HitlUiVariantRef {
    match trigger {
        // Panel: substantial decisions that take over the surface.
        HitlTriggerRef::OnGap
        | HitlTriggerRef::OnFailureThreshold
        | HitlTriggerRef::OnPlanApproval
        | HitlTriggerRef::PerEpic => HitlUiVariantRef::Panel,
        // Modal: quick yes/no, blocks adjacent.
        HitlTriggerRef::OnRiskyTool
        | HitlTriggerRef::OnDontTouchEdit
        | HitlTriggerRef::OnCapabilityViolation
        | HitlTriggerRef::OnBudgetThreshold
        | HitlTriggerRef::PerTask => HitlUiVariantRef::Modal,
    }
}

/// Trigger-specific context the evaluator needs to decide whether the
/// trigger fires. Carries everything from the originating event so the
/// caller does not need a second event lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitlContext {
    /// `on_gap` — a `tool_missing` or `skill_missing` event fired.
    Gap {
        /// Agent that observed the gap.
        agent_id: String,
        /// What's missing (tool or skill name).
        missing: String,
    },
    /// `on_risky_tool` — agent attempted a tool in the allowlist.
    RiskyTool {
        /// Agent attempting the tool.
        agent_id: String,
        /// Tool name (without args) — matched against `tools` allowlist.
        tool_name: String,
    },
    /// `on_dont_touch_edit` — agent attempted to edit a `dont_touch` path.
    DontTouchEdit {
        /// Agent attempting the edit.
        agent_id: String,
        /// Path that triggered the rail.
        path: String,
    },
    /// `on_failure_threshold` — `task_escalated` fired.
    FailureThreshold {
        /// Task that escalated.
        task_id: String,
        /// Plan owning the task.
        plan_id: String,
        /// Failure count that triggered escalation.
        failure_count: u32,
    },
    /// `on_capability_violation` — M5 source.
    CapabilityViolation {
        /// Agent that violated.
        agent_id: String,
        /// Attempted operation.
        attempted: String,
    },
    /// `on_budget_threshold` — Stage F source.
    BudgetThreshold {
        /// Percent of cap spent.
        percent: u32,
    },
    /// `on_plan_approval` — Stage C already wired via `ApprovalSeam`.
    PlanApproval {
        /// Plan awaiting approval.
        plan_id: String,
    },
    /// `per_task` — HITL gate before each task.
    PerTask {
        /// Task about to run.
        task_id: String,
    },
    /// `per_epic` — HITL gate at plan boundary.
    PerEpic {
        /// Plan boundary.
        plan_id: String,
    },
}

impl HitlContext {
    /// Map this context to its trigger discriminator.
    #[must_use]
    pub const fn trigger(&self) -> HitlTriggerRef {
        match self {
            Self::Gap { .. } => HitlTriggerRef::OnGap,
            Self::RiskyTool { .. } => HitlTriggerRef::OnRiskyTool,
            Self::DontTouchEdit { .. } => HitlTriggerRef::OnDontTouchEdit,
            Self::FailureThreshold { .. } => HitlTriggerRef::OnFailureThreshold,
            Self::CapabilityViolation { .. } => HitlTriggerRef::OnCapabilityViolation,
            Self::BudgetThreshold { .. } => HitlTriggerRef::OnBudgetThreshold,
            Self::PlanApproval { .. } => HitlTriggerRef::OnPlanApproval,
            Self::PerTask { .. } => HitlTriggerRef::PerTask,
            Self::PerEpic { .. } => HitlTriggerRef::PerEpic,
        }
    }

    /// Originating `agent_id`, when the trigger is agent-scoped. Plan-scoped
    /// triggers (`per_task`, `per_epic`, `on_plan_approval`,
    /// `on_failure_threshold`, `on_budget_threshold`) return `None`.
    #[must_use]
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            Self::Gap { agent_id, .. }
            | Self::RiskyTool { agent_id, .. }
            | Self::DontTouchEdit { agent_id, .. }
            | Self::CapabilityViolation { agent_id, .. } => Some(agent_id.as_str()),
            Self::FailureThreshold { .. }
            | Self::BudgetThreshold { .. }
            | Self::PlanApproval { .. }
            | Self::PerTask { .. }
            | Self::PerEpic { .. } => None,
        }
    }
}

/// Resolved trigger ready to drive the HITL seam. Returned by
/// [`HitlPolicyEvaluator::evaluate`] when the configured policy fires for
/// the given context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTrigger {
    /// Which trigger fired.
    pub trigger: HitlTriggerRef,
    /// UI variant to mount (per-trigger override OR the spec §6a default).
    pub ui_variant: HitlUiVariantRef,
    /// Wall-clock seconds the seam should wait before timeout (from
    /// `HitlPolicy::timeout_seconds`).
    pub timeout_seconds: u64,
    /// Default action when timeout fires (from
    /// `HitlPolicy::default_action_on_timeout`).
    pub default_action: String,
}

/// 9-trigger policy evaluator. Pure logic; cheap to construct.
pub struct HitlPolicyEvaluator;

impl HitlPolicyEvaluator {
    /// Evaluate `policy` for `context`. Returns `Some(ResolvedTrigger)`
    /// when the per-trigger policy is enabled (and any trigger-specific
    /// pre-conditions met, e.g. `on_risky_tool` requires the tool to be
    /// in the allowlist); `None` otherwise.
    ///
    /// The trigger-specific pre-conditions:
    /// - `on_risky_tool` — fires only when `tool_name` is in
    ///   `policy.tools`. Empty tools array means "every tool is risky".
    /// - `on_failure_threshold` — fires only when the context's
    ///   `failure_count >= policy.threshold` (default 3).
    /// - `on_budget_threshold` — fires only when the context's
    ///   `percent >= policy.percent` (default 90).
    ///
    /// All other triggers fire whenever `enabled = true`.
    #[must_use]
    pub fn evaluate(policy: &HitlPolicy, context: &HitlContext) -> Option<ResolvedTrigger> {
        let trigger = context.trigger();
        let per_trigger = lookup_policy(policy, trigger)?;
        if !per_trigger.enabled {
            return None;
        }
        if !precondition_satisfied(context, per_trigger) {
            return None;
        }
        let ui_variant = per_trigger
            .ui
            .map_or_else(|| default_ui_for(trigger), map_ui_variant);
        let timeout_seconds = policy.timeout_seconds.get();
        Some(ResolvedTrigger {
            trigger,
            ui_variant,
            timeout_seconds,
            default_action: policy.default_action_on_timeout.clone(),
        })
    }
}

const fn lookup_policy(policy: &HitlPolicy, trigger: HitlTriggerRef) -> Option<&HitlTriggerPolicy> {
    match trigger {
        HitlTriggerRef::OnGap => policy.on_gap.as_ref(),
        HitlTriggerRef::OnRiskyTool => policy.on_risky_tool.as_ref(),
        HitlTriggerRef::OnDontTouchEdit => policy.on_dont_touch_edit.as_ref(),
        HitlTriggerRef::OnFailureThreshold => policy.on_failure_threshold.as_ref(),
        HitlTriggerRef::OnCapabilityViolation => policy.on_capability_violation.as_ref(),
        HitlTriggerRef::OnBudgetThreshold => policy.on_budget_threshold.as_ref(),
        HitlTriggerRef::OnPlanApproval => policy.on_plan_approval.as_ref(),
        HitlTriggerRef::PerTask => policy.per_task.as_ref(),
        HitlTriggerRef::PerEpic => policy.per_epic.as_ref(),
    }
}

fn precondition_satisfied(context: &HitlContext, per_trigger: &HitlTriggerPolicy) -> bool {
    match context {
        HitlContext::RiskyTool { tool_name, .. } => {
            // Empty tools list = no allowlist filter = every attempt fires.
            per_trigger.tools.is_empty()
                || per_trigger
                    .tools
                    .iter()
                    .any(|pat| matches_tool(pat.as_str(), tool_name.as_str()))
        }
        HitlContext::FailureThreshold { failure_count, .. } => per_trigger
            .threshold
            .map_or(true, |t| u64::from(*failure_count) >= t.get()),
        HitlContext::BudgetThreshold { percent } => per_trigger
            .percent
            .map_or(true, |p| i64::from(*percent) >= p),
        // All other contexts have no trigger-specific precondition.
        _ => true,
    }
}

/// Risky-tool allowlist pattern matcher. v0.1 supports exact match + a
/// trailing-wildcard form (`Bash:*`). Spec §6a example: `Bash:rm`,
/// `Bash:git push`, `WebFetch:*`.
fn matches_tool(pattern: &str, candidate: &str) -> bool {
    pattern.strip_suffix('*').map_or_else(
        || pattern == candidate,
        |prefix| candidate.starts_with(prefix),
    )
}

const fn map_ui_variant(v: HitlUiVariant) -> HitlUiVariantRef {
    match v {
        HitlUiVariant::Panel => HitlUiVariantRef::Panel,
        HitlUiVariant::Modal => HitlUiVariantRef::Modal,
        HitlUiVariant::Toast => HitlUiVariantRef::Toast,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::hitl::{
        HitlNotifier, HitlNotifierType, HitlPolicy, HitlTriggerPolicy, HitlUiVariant,
    };
    use std::num::NonZeroU64;

    fn enabled_policy(trigger_setter: impl FnOnce(&mut HitlPolicy)) -> HitlPolicy {
        let mut p = HitlPolicy {
            default_action_on_timeout: "abort".into(),
            notifiers: Vec::new(),
            on_budget_threshold: None,
            on_capability_violation: None,
            on_dont_touch_edit: None,
            on_failure_threshold: None,
            on_gap: None,
            on_plan_approval: None,
            on_risky_tool: None,
            per_epic: None,
            per_task: None,
            timeout_seconds: NonZeroU64::new(3600).unwrap(),
        };
        trigger_setter(&mut p);
        p
    }

    fn enabled_trigger() -> HitlTriggerPolicy {
        HitlTriggerPolicy {
            enabled: true,
            percent: None,
            threshold: None,
            tools: Vec::new(),
            ui: None,
        }
    }

    // ── default_ui_for: 9 exhaustive cases ───────────────────────────

    #[test]
    fn default_ui_panel_for_gap_failure_planapproval_perepic() {
        assert_eq!(
            default_ui_for(HitlTriggerRef::OnGap),
            HitlUiVariantRef::Panel
        );
        assert_eq!(
            default_ui_for(HitlTriggerRef::OnFailureThreshold),
            HitlUiVariantRef::Panel
        );
        assert_eq!(
            default_ui_for(HitlTriggerRef::OnPlanApproval),
            HitlUiVariantRef::Panel
        );
        assert_eq!(
            default_ui_for(HitlTriggerRef::PerEpic),
            HitlUiVariantRef::Panel
        );
    }

    #[test]
    fn default_ui_modal_for_risky_donttouch_capability_budget_pertask() {
        assert_eq!(
            default_ui_for(HitlTriggerRef::OnRiskyTool),
            HitlUiVariantRef::Modal
        );
        assert_eq!(
            default_ui_for(HitlTriggerRef::OnDontTouchEdit),
            HitlUiVariantRef::Modal
        );
        assert_eq!(
            default_ui_for(HitlTriggerRef::OnCapabilityViolation),
            HitlUiVariantRef::Modal
        );
        assert_eq!(
            default_ui_for(HitlTriggerRef::OnBudgetThreshold),
            HitlUiVariantRef::Modal
        );
        assert_eq!(
            default_ui_for(HitlTriggerRef::PerTask),
            HitlUiVariantRef::Modal
        );
    }

    // ── evaluate: 9 exhaustive happy-path cases ─────────────────────

    #[test]
    fn evaluate_on_gap_when_enabled() {
        let policy = enabled_policy(|p| p.on_gap = Some(enabled_trigger()));
        let ctx = HitlContext::Gap {
            agent_id: "a1".into(),
            missing: "WebSearch".into(),
        };
        let r = HitlPolicyEvaluator::evaluate(&policy, &ctx).unwrap();
        assert_eq!(r.trigger, HitlTriggerRef::OnGap);
        assert_eq!(r.ui_variant, HitlUiVariantRef::Panel);
        assert_eq!(r.timeout_seconds, 3600);
        assert_eq!(r.default_action, "abort");
    }

    #[test]
    fn evaluate_on_risky_tool_with_matching_pattern() {
        let policy = enabled_policy(|p| {
            p.on_risky_tool = Some(HitlTriggerPolicy {
                enabled: true,
                tools: vec!["Bash:rm".into(), "WebFetch:*".into()],
                ..enabled_trigger()
            });
        });
        let ctx = HitlContext::RiskyTool {
            agent_id: "a1".into(),
            tool_name: "WebFetch:example.com".into(),
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_some());
    }

    #[test]
    fn evaluate_on_risky_tool_no_match_returns_none() {
        let policy = enabled_policy(|p| {
            p.on_risky_tool = Some(HitlTriggerPolicy {
                enabled: true,
                tools: vec!["Bash:rm".into()],
                ..enabled_trigger()
            });
        });
        let ctx = HitlContext::RiskyTool {
            agent_id: "a1".into(),
            tool_name: "Bash:ls".into(),
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_none());
    }

    #[test]
    fn evaluate_on_risky_tool_empty_allowlist_fires_for_any_tool() {
        // Empty tools = no filter = every tool attempt fires the trigger.
        let policy = enabled_policy(|p| p.on_risky_tool = Some(enabled_trigger()));
        let ctx = HitlContext::RiskyTool {
            agent_id: "a1".into(),
            tool_name: "Anything".into(),
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_some());
    }

    #[test]
    fn evaluate_on_dont_touch_edit_when_enabled() {
        let policy = enabled_policy(|p| p.on_dont_touch_edit = Some(enabled_trigger()));
        let ctx = HitlContext::DontTouchEdit {
            agent_id: "a1".into(),
            path: "LICENSE".into(),
        };
        assert_eq!(
            HitlPolicyEvaluator::evaluate(&policy, &ctx)
                .unwrap()
                .trigger,
            HitlTriggerRef::OnDontTouchEdit
        );
    }

    #[test]
    fn evaluate_on_failure_threshold_at_or_above_default() {
        let policy = enabled_policy(|p| p.on_failure_threshold = Some(enabled_trigger()));
        // No threshold set = always fires.
        let ctx = HitlContext::FailureThreshold {
            task_id: "t1".into(),
            plan_id: "p1".into(),
            failure_count: 1,
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_some());
    }

    #[test]
    fn evaluate_on_failure_threshold_below_threshold_returns_none() {
        let policy = enabled_policy(|p| {
            p.on_failure_threshold = Some(HitlTriggerPolicy {
                enabled: true,
                threshold: NonZeroU64::new(5),
                ..enabled_trigger()
            });
        });
        let ctx = HitlContext::FailureThreshold {
            task_id: "t1".into(),
            plan_id: "p1".into(),
            failure_count: 3,
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_none());
    }

    #[test]
    fn evaluate_on_failure_threshold_at_explicit_threshold_fires() {
        let policy = enabled_policy(|p| {
            p.on_failure_threshold = Some(HitlTriggerPolicy {
                enabled: true,
                threshold: NonZeroU64::new(3),
                ..enabled_trigger()
            });
        });
        let ctx = HitlContext::FailureThreshold {
            task_id: "t1".into(),
            plan_id: "p1".into(),
            failure_count: 3,
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_some());
    }

    #[test]
    fn evaluate_on_capability_violation_when_enabled() {
        let policy = enabled_policy(|p| p.on_capability_violation = Some(enabled_trigger()));
        let ctx = HitlContext::CapabilityViolation {
            agent_id: "a1".into(),
            attempted: "Bash".into(),
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_some());
    }

    #[test]
    fn evaluate_on_budget_threshold_at_or_above_percent() {
        let policy = enabled_policy(|p| {
            p.on_budget_threshold = Some(HitlTriggerPolicy {
                enabled: true,
                percent: Some(90),
                ..enabled_trigger()
            });
        });
        let ctx_below = HitlContext::BudgetThreshold { percent: 80 };
        let ctx_at = HitlContext::BudgetThreshold { percent: 90 };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx_below).is_none());
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx_at).is_some());
    }

    #[test]
    fn evaluate_on_plan_approval_when_enabled() {
        let policy = enabled_policy(|p| p.on_plan_approval = Some(enabled_trigger()));
        let ctx = HitlContext::PlanApproval {
            plan_id: "p1".into(),
        };
        assert_eq!(
            HitlPolicyEvaluator::evaluate(&policy, &ctx)
                .unwrap()
                .trigger,
            HitlTriggerRef::OnPlanApproval
        );
    }

    #[test]
    fn evaluate_per_task_when_enabled() {
        let policy = enabled_policy(|p| p.per_task = Some(enabled_trigger()));
        let ctx = HitlContext::PerTask {
            task_id: "t1".into(),
        };
        assert_eq!(
            HitlPolicyEvaluator::evaluate(&policy, &ctx)
                .unwrap()
                .trigger,
            HitlTriggerRef::PerTask
        );
    }

    #[test]
    fn evaluate_per_epic_when_enabled() {
        let policy = enabled_policy(|p| p.per_epic = Some(enabled_trigger()));
        let ctx = HitlContext::PerEpic {
            plan_id: "p1".into(),
        };
        assert_eq!(
            HitlPolicyEvaluator::evaluate(&policy, &ctx)
                .unwrap()
                .trigger,
            HitlTriggerRef::PerEpic
        );
    }

    // ── evaluate: disabled / missing → None ─────────────────────────

    #[test]
    fn evaluate_missing_policy_returns_none() {
        let policy = enabled_policy(|_| {});
        let ctx = HitlContext::Gap {
            agent_id: "a1".into(),
            missing: "x".into(),
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_none());
    }

    #[test]
    fn evaluate_disabled_policy_returns_none() {
        let policy = enabled_policy(|p| {
            p.on_gap = Some(HitlTriggerPolicy {
                enabled: false,
                ..enabled_trigger()
            });
        });
        let ctx = HitlContext::Gap {
            agent_id: "a1".into(),
            missing: "x".into(),
        };
        assert!(HitlPolicyEvaluator::evaluate(&policy, &ctx).is_none());
    }

    // ── UI override ─────────────────────────────────────────────────

    #[test]
    fn evaluate_uses_per_trigger_ui_override_when_present() {
        let policy = enabled_policy(|p| {
            // on_gap defaults to Panel; override to Modal.
            p.on_gap = Some(HitlTriggerPolicy {
                enabled: true,
                ui: Some(HitlUiVariant::Modal),
                ..enabled_trigger()
            });
        });
        let ctx = HitlContext::Gap {
            agent_id: "a1".into(),
            missing: "x".into(),
        };
        let r = HitlPolicyEvaluator::evaluate(&policy, &ctx).unwrap();
        assert_eq!(r.ui_variant, HitlUiVariantRef::Modal);
    }

    #[test]
    fn evaluate_falls_back_to_default_ui_when_unset() {
        let policy = enabled_policy(|p| p.on_failure_threshold = Some(enabled_trigger()));
        let ctx = HitlContext::FailureThreshold {
            task_id: "t1".into(),
            plan_id: "p1".into(),
            failure_count: 3,
        };
        let r = HitlPolicyEvaluator::evaluate(&policy, &ctx).unwrap();
        assert_eq!(r.ui_variant, HitlUiVariantRef::Panel);
    }

    #[test]
    fn evaluate_passes_through_toast_ui_override() {
        let policy = enabled_policy(|p| {
            p.per_task = Some(HitlTriggerPolicy {
                enabled: true,
                ui: Some(HitlUiVariant::Toast),
                ..enabled_trigger()
            });
        });
        let ctx = HitlContext::PerTask {
            task_id: "t1".into(),
        };
        let r = HitlPolicyEvaluator::evaluate(&policy, &ctx).unwrap();
        assert_eq!(r.ui_variant, HitlUiVariantRef::Toast);
    }

    // ── HitlContext::trigger() / agent_id() exhaustive coverage ─────

    #[test]
    fn context_trigger_maps_each_variant() {
        let cases: Vec<(HitlContext, HitlTriggerRef)> = vec![
            (
                HitlContext::Gap {
                    agent_id: "a".into(),
                    missing: "x".into(),
                },
                HitlTriggerRef::OnGap,
            ),
            (
                HitlContext::RiskyTool {
                    agent_id: "a".into(),
                    tool_name: "x".into(),
                },
                HitlTriggerRef::OnRiskyTool,
            ),
            (
                HitlContext::DontTouchEdit {
                    agent_id: "a".into(),
                    path: "x".into(),
                },
                HitlTriggerRef::OnDontTouchEdit,
            ),
            (
                HitlContext::FailureThreshold {
                    task_id: "t".into(),
                    plan_id: "p".into(),
                    failure_count: 1,
                },
                HitlTriggerRef::OnFailureThreshold,
            ),
            (
                HitlContext::CapabilityViolation {
                    agent_id: "a".into(),
                    attempted: "x".into(),
                },
                HitlTriggerRef::OnCapabilityViolation,
            ),
            (
                HitlContext::BudgetThreshold { percent: 50 },
                HitlTriggerRef::OnBudgetThreshold,
            ),
            (
                HitlContext::PlanApproval {
                    plan_id: "p".into(),
                },
                HitlTriggerRef::OnPlanApproval,
            ),
            (
                HitlContext::PerTask {
                    task_id: "t".into(),
                },
                HitlTriggerRef::PerTask,
            ),
            (
                HitlContext::PerEpic {
                    plan_id: "p".into(),
                },
                HitlTriggerRef::PerEpic,
            ),
        ];
        for (ctx, expected) in cases {
            assert_eq!(ctx.trigger(), expected, "{ctx:?}");
        }
    }

    #[test]
    fn context_agent_id_returns_some_for_agent_scoped_triggers() {
        assert_eq!(
            HitlContext::Gap {
                agent_id: "a1".into(),
                missing: "x".into()
            }
            .agent_id(),
            Some("a1")
        );
        assert_eq!(
            HitlContext::RiskyTool {
                agent_id: "a2".into(),
                tool_name: "x".into()
            }
            .agent_id(),
            Some("a2")
        );
        assert_eq!(
            HitlContext::DontTouchEdit {
                agent_id: "a3".into(),
                path: "x".into()
            }
            .agent_id(),
            Some("a3")
        );
        assert_eq!(
            HitlContext::CapabilityViolation {
                agent_id: "a4".into(),
                attempted: "x".into()
            }
            .agent_id(),
            Some("a4")
        );
    }

    #[test]
    fn context_agent_id_returns_none_for_plan_scoped_triggers() {
        assert!(HitlContext::FailureThreshold {
            task_id: "t".into(),
            plan_id: "p".into(),
            failure_count: 3
        }
        .agent_id()
        .is_none());
        assert!(HitlContext::BudgetThreshold { percent: 50 }
            .agent_id()
            .is_none());
        assert!(HitlContext::PlanApproval {
            plan_id: "p".into()
        }
        .agent_id()
        .is_none());
        assert!(HitlContext::PerTask {
            task_id: "t".into()
        }
        .agent_id()
        .is_none());
        assert!(HitlContext::PerEpic {
            plan_id: "p".into()
        }
        .agent_id()
        .is_none());
    }

    // ── matches_tool: pattern matcher direct ─────────────────────────

    #[test]
    fn matches_tool_exact_pattern() {
        assert!(matches_tool("Bash:rm", "Bash:rm"));
        assert!(!matches_tool("Bash:rm", "Bash:ls"));
    }

    #[test]
    fn matches_tool_trailing_wildcard() {
        assert!(matches_tool("WebFetch:*", "WebFetch:anything"));
        assert!(matches_tool("WebFetch:*", "WebFetch:"));
        assert!(!matches_tool("WebFetch:*", "WebSearch:anything"));
    }

    #[test]
    fn matches_tool_bare_wildcard_matches_all() {
        assert!(matches_tool("*", "anything"));
        assert!(matches_tool("*", ""));
    }

    // ── Sanity: notifiers field can hold built-in entries ───────────

    #[test]
    fn policy_can_carry_built_in_notifiers() {
        let policy = enabled_policy(|p| {
            p.notifiers = vec![
                HitlNotifier {
                    type_: HitlNotifierType::TerminalBell,
                    enabled: true,
                    name: None,
                    config: ::serde_json::Map::new(),
                },
                HitlNotifier {
                    type_: HitlNotifierType::Desktop,
                    enabled: true,
                    name: None,
                    config: ::serde_json::Map::new(),
                },
                HitlNotifier {
                    type_: HitlNotifierType::Sound,
                    enabled: false,
                    name: None,
                    config: ::serde_json::Map::new(),
                },
            ];
        });
        assert_eq!(policy.notifiers.len(), 3);
        assert!(!policy.notifiers[2].enabled);
    }
}
