//! Plan-driver loop — spec §3a.
//!
//! Walks a [`PlanStateMachine`] from `PendingApproval` through
//! `Complete`, routing the approval gate through the in-process
//! [`HitlSeam`] (ADR-0007) and streaming the plan-lifecycle
//! [`AgentEvent`]s out through an [`mpsc::UnboundedSender`]. Task
//! *execution* is [`crate::sdk::AgentSdk::run_agent`] (M07.D2); this
//! module is the M04-🟡 driver *shell* that advances the FSM — it does
//! not itself run tasks.
//!
//! [`PlanStateMachine`]: crate::plan::state_machine::PlanStateMachine
//! [`mpsc::UnboundedSender`]: tokio::sync::mpsc::UnboundedSender

use std::time::Duration;

use runtime_core::event::AgentEvent;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

use crate::hitl::{HitlError, HitlSeam};
use crate::plan::state_machine::{PlanState, TransitionError};

/// Failure modes raised by [`drive_plan`].
#[derive(Debug, Error)]
pub enum PlanLoopError {
    /// The FSM rejected a transition the driver attempted — a driver
    /// bug (the driver must only issue legal `(status, event)` pairs for
    /// the state it observed).
    #[error(transparent)]
    Transition(#[from] TransitionError),
    /// The HITL approval gate failed to resolve — the seam was
    /// cancelled or the wait elapsed before the user answered.
    #[error(transparent)]
    Hitl(#[from] HitlError),
}

/// Drive `plan` through approval + execution to a terminal state.
///
/// Plan-lifecycle events are streamed out through `events` as they
/// occur — `plan_approval_requested` reaches the renderer *before* the
/// approval gate blocks, so the user can be shown the prompt. A send to
/// a dropped receiver is ignored: a disconnected renderer does not
/// abort the plan FSM.
///
/// 1. If the plan is `PendingApproval`, emit `plan_approval_requested`
///    and await the HITL approval gate on `hitl` (up to
///    `approval_timeout`); a `Reject` aborts the plan (FSM `Aborted`,
///    `plan_aborted` emitted) and returns, an `Approve` advances it
///    (FSM `Approved`, `plan_approved { approved_by: user }` emitted).
/// 2. A plan created `approval_required: false` starts `Approved`; the
///    driver emits `plan_approved { approved_by: auto }` and skips the
///    gate.
/// 3. Advance `Approved → InProgress` (FSM `Started`). Task execution is
///    [`crate::sdk::AgentSdk::run_agent`] — this shell drives only the
///    FSM, it runs no tasks.
/// 4. Advance `InProgress → Complete` (FSM `Complete`) and emit
///    `plan_complete`.
///
/// The phase-doc pseudocode named an `AgentEvent::PlanStarted`; the
/// schema-generated `AgentEvent` has no such variant — the plan
/// lifecycle on the wire is `plan_approval_requested → plan_approved →
/// plan_complete` (a plan reaching `InProgress` is observed through the
/// first `task_started`, out of this shell's scope). The driver emits
/// only events that exist.
///
/// # Errors
///
/// - [`PlanLoopError::Hitl`] if the approval gate is cancelled or times
///   out before the user answers.
/// - [`PlanLoopError::Transition`] if the FSM rejects a driver-issued
///   transition (indicates a driver bug — e.g. driving a plan already
///   in a terminal state — not a user-facing condition).
pub async fn drive_plan(
    plan: &mut PlanState,
    hitl: &HitlSeam,
    approval_timeout: Duration,
    events: UnboundedSender<AgentEvent>,
) -> Result<(), PlanLoopError> {
    let _ = (plan, hitl, approval_timeout, events);
    todo!("M08.A green phase — drive_plan driver shell")
}
