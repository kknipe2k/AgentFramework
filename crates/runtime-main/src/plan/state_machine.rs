//! Plan + Task state machines — spec §3a (Plan & Task lifecycle).
//!
//! Pure logic. Each FSM exposes a single `transition(state, event)` that
//! either advances the state in-place or returns [`TransitionError`].
//! Safety primitive: ≥95% coverage gate per CLAUDE.md §5.
//!
//! # Plan FSM
//!
//! - `PendingApproval` → `Approved` | `Aborted`
//! - `Approved` → `InProgress`
//! - `InProgress` → `Complete` | `Aborted` | `AwaitingReplan`
//! - `AwaitingReplan` → `InProgress` | `Aborted`
//!
//! Terminal: `Complete`, `Aborted`.
//!
//! # Task FSM
//!
//! - `Pending` → `Running` | `Skipped`
//! - `Running` → `Done` | `Failed` | `Blocked`
//! - `Failed` → `Pending` (retry, when `failure_count < max_failures`)
//!            | `Escalated` (when `failure_count >= max_failures`)
//! - `Blocked` → `Pending`
//!
//! Terminal: `Done`, `Skipped`, `Escalated`.

use thiserror::Error;

/// Plan FSM state per spec §3a. Mirrors `runtime_core::plan::PlanStatus`
/// (the schema-derived enum) but kept independent so the FSM can evolve
/// without churning the wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStatus {
    /// Plan suspended awaiting human approval.
    PendingApproval,
    /// Plan approved, not yet started.
    Approved,
    /// Plan executing (one or more tasks have started).
    InProgress,
    /// Plan suspended awaiting a revision (HITL or auto-replan).
    AwaitingReplan,
    /// Plan finished — every task reached terminal-non-failed.
    Complete,
    /// Plan cancelled.
    Aborted,
}

impl PlanStatus {
    /// Terminal states do not allow further transitions.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Complete | Self::Aborted)
    }
}

/// Events the [`PlanStateMachine`] reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanEvent {
    /// Approval received (HITL or auto).
    Approved,
    /// Execution begun (first task started).
    Started,
    /// All tasks reached terminal-non-failed.
    Complete,
    /// User or escalation cancelled the plan.
    Aborted,
    /// Plan halted to await a revision.
    AwaitingReplan,
    /// Revised plan ready; resume execution.
    Revised,
}

/// Owned Plan FSM state. Cheap to clone (pure-data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanState {
    /// Plan UUID.
    pub plan_id: String,
    /// Current FSM status.
    pub status: PlanStatus,
}

impl PlanState {
    /// New plan in `PendingApproval` (or `InProgress` when
    /// `approval_required = false`).
    #[must_use]
    pub fn new(plan_id: impl Into<String>, approval_required: bool) -> Self {
        Self {
            plan_id: plan_id.into(),
            status: if approval_required {
                PlanStatus::PendingApproval
            } else {
                PlanStatus::Approved
            },
        }
    }
}

/// Pure-logic Plan FSM. Holds no state of its own; mutates the
/// passed-in [`PlanState`].
pub struct PlanStateMachine;

impl PlanStateMachine {
    /// Apply `event` to `state` per spec §3a. Returns `Ok(())` and
    /// mutates `state.status`; returns [`TransitionError`] for any
    /// illegal pair.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::IllegalPlan`] if the (status, event)
    /// pair is not in the legal transition matrix.
    pub fn transition(state: &mut PlanState, event: PlanEvent) -> Result<(), TransitionError> {
        let next = match (state.status, event) {
            (PlanStatus::PendingApproval, PlanEvent::Approved) => PlanStatus::Approved,
            (PlanStatus::Approved, PlanEvent::Started)
            | (PlanStatus::AwaitingReplan, PlanEvent::Revised) => PlanStatus::InProgress,
            (
                PlanStatus::PendingApproval
                | PlanStatus::Approved
                | PlanStatus::InProgress
                | PlanStatus::AwaitingReplan,
                PlanEvent::Aborted,
            ) => PlanStatus::Aborted,
            (PlanStatus::InProgress, PlanEvent::Complete) => PlanStatus::Complete,
            (PlanStatus::InProgress, PlanEvent::AwaitingReplan) => PlanStatus::AwaitingReplan,
            (from, e) => {
                return Err(TransitionError::IllegalPlan { from, event: e });
            }
        };
        state.status = next;
        Ok(())
    }
}

/// Task FSM state per spec §3a.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task created, not yet started.
    Pending,
    /// Task currently executing.
    Running,
    /// Task completed successfully.
    Done,
    /// Task failed; eligible for retry while `failure_count < max_failures`.
    Failed,
    /// Task halted on a missing capability (gap detection); waits for
    /// `gap_resolved` to return to `Pending`.
    Blocked,
    /// Task skipped (HITL chose to skip after escalation, or
    /// preconditions unmet).
    Skipped,
    /// Task hit `failure_count >= max_failures`; awaiting HITL.
    Escalated,
}

impl TaskStatus {
    /// Terminal states do not allow further transitions.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Skipped | Self::Escalated)
    }
}

/// Events the [`TaskStateMachine`] reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskEvent {
    /// Execution begun.
    Started,
    /// Successful completion.
    Completed,
    /// Failure observed (non-terminal until retry budget exhausted).
    Failed,
    /// Capability gap detected (transitions to `Blocked`).
    Blocked,
    /// Gap resolved; task reset to `Pending` for retry.
    Resolved,
    /// User/HITL chose to skip.
    Skipped,
}

/// Owned Task FSM state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskState {
    /// Task UUID.
    pub task_id: String,
    /// Owning plan UUID.
    pub plan_id: String,
    /// Current FSM status.
    pub status: TaskStatus,
    /// Number of failed attempts; reset on `Resolved`.
    pub failure_count: u32,
    /// Failure budget. When `failure_count >= max_failures`, a `Failed`
    /// event escalates instead of returning to `Pending`.
    pub max_failures: u32,
}

impl TaskState {
    /// New task in `Pending` with `failure_count = 0`. Default
    /// `max_failures = 3` per spec §3a.
    #[must_use]
    pub fn new(task_id: impl Into<String>, plan_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            plan_id: plan_id.into(),
            status: TaskStatus::Pending,
            failure_count: 0,
            max_failures: 3,
        }
    }

    /// Same as [`Self::new`] but with an explicit failure budget. Used
    /// when the framework JSON overrides per-task or session-wide.
    #[must_use]
    pub fn with_max_failures(
        task_id: impl Into<String>,
        plan_id: impl Into<String>,
        max_failures: u32,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            plan_id: plan_id.into(),
            status: TaskStatus::Pending,
            failure_count: 0,
            max_failures: max_failures.max(1),
        }
    }
}

/// Pure-logic Task FSM.
pub struct TaskStateMachine;

impl TaskStateMachine {
    /// Apply `event` to `state` per spec §3a. Mutates `state.status` and
    /// `state.failure_count` for `Failed` events.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError::IllegalTask`] if the (status, event)
    /// pair is not in the legal transition matrix.
    pub fn transition(state: &mut TaskState, event: TaskEvent) -> Result<(), TransitionError> {
        let next = match (state.status, event) {
            (TaskStatus::Pending, TaskEvent::Started) => TaskStatus::Running,
            (TaskStatus::Pending, TaskEvent::Skipped) => TaskStatus::Skipped,
            (TaskStatus::Running, TaskEvent::Completed) => TaskStatus::Done,
            (TaskStatus::Running, TaskEvent::Failed) => {
                state.failure_count = state.failure_count.saturating_add(1);
                if state.failure_count >= state.max_failures {
                    TaskStatus::Escalated
                } else {
                    TaskStatus::Failed
                }
            }
            (TaskStatus::Running, TaskEvent::Blocked) => TaskStatus::Blocked,
            (TaskStatus::Failed, TaskEvent::Started) => {
                // Retry attempt — back to Running.
                TaskStatus::Running
            }
            (TaskStatus::Blocked, TaskEvent::Resolved) => TaskStatus::Pending,
            (from, e) => {
                return Err(TransitionError::IllegalTask { from, event: e });
            }
        };
        state.status = next;
        Ok(())
    }
}

/// Errors raised by the Plan / Task state machines.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TransitionError {
    /// Illegal plan transition (the (status, event) pair is not in the
    /// legal matrix per spec §3a).
    #[error("illegal plan transition from {from:?} on event {event:?}")]
    IllegalPlan {
        /// Plan status at the time of the attempted transition.
        from: PlanStatus,
        /// The event that was rejected.
        event: PlanEvent,
    },
    /// Illegal task transition.
    #[error("illegal task transition from {from:?} on event {event:?}")]
    IllegalTask {
        /// Task status at the time of the attempted transition.
        from: TaskStatus,
        /// The event that was rejected.
        event: TaskEvent,
    },
}

#[cfg(test)]
#[allow(
    clippy::match_wildcard_for_single_variants,
    reason = "test panics on unexpected variant; `other` keeps test bodies short"
)]
mod tests {
    use super::*;

    // ── PlanState constructor ─────────────────────────────────────

    #[test]
    fn plan_state_new_with_approval_required_starts_pending() {
        let p = PlanState::new("p1", true);
        assert_eq!(p.status, PlanStatus::PendingApproval);
        assert_eq!(p.plan_id, "p1");
    }

    #[test]
    fn plan_state_new_without_approval_required_starts_approved() {
        let p = PlanState::new("p1", false);
        assert_eq!(p.status, PlanStatus::Approved);
    }

    #[test]
    fn plan_status_terminal_predicates() {
        assert!(PlanStatus::Complete.is_terminal());
        assert!(PlanStatus::Aborted.is_terminal());
        assert!(!PlanStatus::PendingApproval.is_terminal());
        assert!(!PlanStatus::Approved.is_terminal());
        assert!(!PlanStatus::InProgress.is_terminal());
        assert!(!PlanStatus::AwaitingReplan.is_terminal());
    }

    // ── Plan FSM legal transitions ────────────────────────────────

    #[test]
    fn plan_pending_approval_to_approved() {
        let mut p = PlanState::new("p1", true);
        PlanStateMachine::transition(&mut p, PlanEvent::Approved).unwrap();
        assert_eq!(p.status, PlanStatus::Approved);
    }

    #[test]
    fn plan_pending_approval_to_aborted() {
        let mut p = PlanState::new("p1", true);
        PlanStateMachine::transition(&mut p, PlanEvent::Aborted).unwrap();
        assert_eq!(p.status, PlanStatus::Aborted);
    }

    #[test]
    fn plan_approved_to_in_progress_via_started() {
        let mut p = PlanState::new("p1", false);
        PlanStateMachine::transition(&mut p, PlanEvent::Started).unwrap();
        assert_eq!(p.status, PlanStatus::InProgress);
    }

    #[test]
    fn plan_approved_to_aborted() {
        let mut p = PlanState::new("p1", false);
        PlanStateMachine::transition(&mut p, PlanEvent::Aborted).unwrap();
        assert_eq!(p.status, PlanStatus::Aborted);
    }

    #[test]
    fn plan_in_progress_to_complete() {
        let mut p = PlanState::new("p1", false);
        PlanStateMachine::transition(&mut p, PlanEvent::Started).unwrap();
        PlanStateMachine::transition(&mut p, PlanEvent::Complete).unwrap();
        assert_eq!(p.status, PlanStatus::Complete);
    }

    #[test]
    fn plan_in_progress_to_awaiting_replan_then_back() {
        let mut p = PlanState::new("p1", false);
        PlanStateMachine::transition(&mut p, PlanEvent::Started).unwrap();
        PlanStateMachine::transition(&mut p, PlanEvent::AwaitingReplan).unwrap();
        assert_eq!(p.status, PlanStatus::AwaitingReplan);
        PlanStateMachine::transition(&mut p, PlanEvent::Revised).unwrap();
        assert_eq!(p.status, PlanStatus::InProgress);
    }

    #[test]
    fn plan_awaiting_replan_can_abort() {
        let mut p = PlanState::new("p1", false);
        PlanStateMachine::transition(&mut p, PlanEvent::Started).unwrap();
        PlanStateMachine::transition(&mut p, PlanEvent::AwaitingReplan).unwrap();
        PlanStateMachine::transition(&mut p, PlanEvent::Aborted).unwrap();
        assert_eq!(p.status, PlanStatus::Aborted);
    }

    #[test]
    fn plan_in_progress_to_aborted() {
        let mut p = PlanState::new("p1", false);
        PlanStateMachine::transition(&mut p, PlanEvent::Started).unwrap();
        PlanStateMachine::transition(&mut p, PlanEvent::Aborted).unwrap();
        assert_eq!(p.status, PlanStatus::Aborted);
    }

    // ── Plan FSM illegal transitions ──────────────────────────────

    #[test]
    fn plan_complete_rejects_further_transitions() {
        let mut p = PlanState::new("p1", false);
        PlanStateMachine::transition(&mut p, PlanEvent::Started).unwrap();
        PlanStateMachine::transition(&mut p, PlanEvent::Complete).unwrap();
        let err = PlanStateMachine::transition(&mut p, PlanEvent::Approved).unwrap_err();
        assert!(matches!(err, TransitionError::IllegalPlan { .. }));
    }

    #[test]
    fn plan_aborted_rejects_further_transitions() {
        let mut p = PlanState::new("p1", true);
        PlanStateMachine::transition(&mut p, PlanEvent::Aborted).unwrap();
        let err = PlanStateMachine::transition(&mut p, PlanEvent::Approved).unwrap_err();
        assert!(matches!(err, TransitionError::IllegalPlan { .. }));
    }

    #[test]
    fn plan_pending_approval_rejects_started() {
        // Cannot start before Approved.
        let mut p = PlanState::new("p1", true);
        let err = PlanStateMachine::transition(&mut p, PlanEvent::Started).unwrap_err();
        match err {
            TransitionError::IllegalPlan { from, event } => {
                assert_eq!(from, PlanStatus::PendingApproval);
                assert_eq!(event, PlanEvent::Started);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn plan_in_progress_rejects_approved_event() {
        let mut p = PlanState::new("p1", false);
        PlanStateMachine::transition(&mut p, PlanEvent::Started).unwrap();
        let err = PlanStateMachine::transition(&mut p, PlanEvent::Approved).unwrap_err();
        assert!(matches!(err, TransitionError::IllegalPlan { .. }));
    }

    #[test]
    fn plan_approved_rejects_complete() {
        let mut p = PlanState::new("p1", false);
        let err = PlanStateMachine::transition(&mut p, PlanEvent::Complete).unwrap_err();
        assert!(matches!(err, TransitionError::IllegalPlan { .. }));
    }

    // ── TaskState constructors ────────────────────────────────────

    #[test]
    fn task_state_new_default_max_failures_is_three() {
        let t = TaskState::new("t1", "p1");
        assert_eq!(t.failure_count, 0);
        assert_eq!(t.max_failures, 3);
        assert_eq!(t.status, TaskStatus::Pending);
    }

    #[test]
    fn task_state_with_max_failures_clamps_to_one() {
        let t = TaskState::with_max_failures("t1", "p1", 0);
        assert_eq!(t.max_failures, 1, "max_failures must clamp to >= 1");
    }

    #[test]
    fn task_status_terminal_predicates() {
        assert!(TaskStatus::Done.is_terminal());
        assert!(TaskStatus::Skipped.is_terminal());
        assert!(TaskStatus::Escalated.is_terminal());
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
        assert!(!TaskStatus::Failed.is_terminal());
        assert!(!TaskStatus::Blocked.is_terminal());
    }

    // ── Task FSM legal transitions ────────────────────────────────

    #[test]
    fn task_pending_to_running() {
        let mut t = TaskState::new("t1", "p1");
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        assert_eq!(t.status, TaskStatus::Running);
    }

    #[test]
    fn task_pending_to_skipped() {
        let mut t = TaskState::new("t1", "p1");
        TaskStateMachine::transition(&mut t, TaskEvent::Skipped).unwrap();
        assert_eq!(t.status, TaskStatus::Skipped);
    }

    #[test]
    fn task_running_to_done() {
        let mut t = TaskState::new("t1", "p1");
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        TaskStateMachine::transition(&mut t, TaskEvent::Completed).unwrap();
        assert_eq!(t.status, TaskStatus::Done);
    }

    #[test]
    fn task_running_to_blocked_then_resolved_back_to_pending() {
        let mut t = TaskState::new("t1", "p1");
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        TaskStateMachine::transition(&mut t, TaskEvent::Blocked).unwrap();
        assert_eq!(t.status, TaskStatus::Blocked);
        TaskStateMachine::transition(&mut t, TaskEvent::Resolved).unwrap();
        assert_eq!(t.status, TaskStatus::Pending);
    }

    // ── Task FSM failure-escalation boundary ──────────────────────

    #[test]
    fn task_failed_below_threshold_lands_in_failed() {
        let mut t = TaskState::with_max_failures("t1", "p1", 3);
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        TaskStateMachine::transition(&mut t, TaskEvent::Failed).unwrap();
        assert_eq!(t.status, TaskStatus::Failed);
        assert_eq!(t.failure_count, 1);
    }

    #[test]
    fn task_failed_at_threshold_escalates() {
        let mut t = TaskState::with_max_failures("t1", "p1", 2);
        // First attempt: Started → Failed (count 1, below threshold).
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        TaskStateMachine::transition(&mut t, TaskEvent::Failed).unwrap();
        assert_eq!(t.status, TaskStatus::Failed);
        assert_eq!(t.failure_count, 1);
        // Retry: Started → Failed (count 2, at threshold → Escalated).
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        TaskStateMachine::transition(&mut t, TaskEvent::Failed).unwrap();
        assert_eq!(t.status, TaskStatus::Escalated);
        assert_eq!(t.failure_count, 2);
    }

    #[test]
    fn task_failed_can_retry_via_started() {
        let mut t = TaskState::with_max_failures("t1", "p1", 5);
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        TaskStateMachine::transition(&mut t, TaskEvent::Failed).unwrap();
        assert_eq!(t.status, TaskStatus::Failed);
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        assert_eq!(t.status, TaskStatus::Running);
        assert_eq!(t.failure_count, 1, "retry preserves failure_count");
    }

    // ── Task FSM illegal transitions ──────────────────────────────

    #[test]
    fn task_done_rejects_further_transitions() {
        let mut t = TaskState::new("t1", "p1");
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        TaskStateMachine::transition(&mut t, TaskEvent::Completed).unwrap();
        let err = TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTask { .. }));
    }

    #[test]
    fn task_escalated_rejects_further_transitions() {
        let mut t = TaskState::with_max_failures("t1", "p1", 1);
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        TaskStateMachine::transition(&mut t, TaskEvent::Failed).unwrap();
        assert_eq!(t.status, TaskStatus::Escalated);
        let err = TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTask { .. }));
    }

    #[test]
    fn task_pending_rejects_completed() {
        let mut t = TaskState::new("t1", "p1");
        let err = TaskStateMachine::transition(&mut t, TaskEvent::Completed).unwrap_err();
        match err {
            TransitionError::IllegalTask { from, event } => {
                assert_eq!(from, TaskStatus::Pending);
                assert_eq!(event, TaskEvent::Completed);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn task_running_rejects_skipped() {
        let mut t = TaskState::new("t1", "p1");
        TaskStateMachine::transition(&mut t, TaskEvent::Started).unwrap();
        let err = TaskStateMachine::transition(&mut t, TaskEvent::Skipped).unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTask { .. }));
    }

    #[test]
    fn transition_error_displays_meaningfully() {
        let plan_err = TransitionError::IllegalPlan {
            from: PlanStatus::Complete,
            event: PlanEvent::Approved,
        };
        let msg = plan_err.to_string();
        assert!(msg.contains("Complete"));
        assert!(msg.contains("Approved"));

        let task_err = TransitionError::IllegalTask {
            from: TaskStatus::Done,
            event: TaskEvent::Started,
        };
        let msg = task_err.to_string();
        assert!(msg.contains("Done"));
        assert!(msg.contains("Started"));
    }
}
