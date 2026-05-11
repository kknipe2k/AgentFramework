//! HITL primitive — spec §6a (M04 Stage E).
//!
//! 9 trigger types + 3 UI variants + notifier plugin interface. Wires
//! Stage B's failure-escalation flow (`task_escalated` → `on_failure_threshold`)
//! through the HITL seam to the renderer's Panel / Modal / Toast surfaces.
//!
//! Submodules:
//! - `seam` — `HitlSeam` (oneshot channel) the SDK awaits on while a
//!   HITL prompt is outstanding. Mirrors Stage B's `ApprovalSeam`.
//! - `policy` — 9-trigger policy evaluator. Pure logic: `evaluate(policy,
//!   trigger, context)` returns the resolved `HitlTriggerPolicy` (or `None`
//!   when disabled). v0.1 STANDARD-mode-hardcoded (CLAUDE.md §3).
//! - `notifiers` — 3 built-in notifiers (terminal_bell / desktop / sound)
//!   + the trait `HitlNotifier`. Notifier failures are NON-FATAL (spec §6a):
//!   the seam still resolves on user response or timeout regardless.
//!
//! Safety primitive: ≥95% coverage gate per CLAUDE.md §5 (capability-
//! enforcer-adjacent). `notifiers/desktop.rs` real-Tauri-call path is
//! excluded with documented rationale (M02 / A2 / M04.D OS-call holdout
//! precedent).

/// Notifier plugin interface + 3 built-in notifiers (`terminal_bell`,
/// `desktop`, `sound`).
pub mod notifiers;
/// 9-trigger policy evaluator. Pure logic.
pub mod policy;
/// `HitlSeam` — channel-backed gate the SDK awaits on while a HITL prompt
/// is outstanding.
pub mod seam;

pub use notifiers::{
    HitlNotifier, HitlNotifyEvent, NotifierError, NotifierOutcome, NotifierRegistry,
};
pub use policy::{HitlContext, HitlPolicyEvaluator, ResolvedTrigger};
pub use seam::{HitlChoice, HitlError, HitlPrompt, HitlSeam};
