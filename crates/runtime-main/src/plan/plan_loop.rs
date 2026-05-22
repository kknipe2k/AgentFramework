//! Plan-driver loop — spec §3a.
//!
//! Walks the `PlanStateMachine` from `PendingApproval` through
//! `Complete`, routing the approval gate through the in-process
//! `HitlSeam` (ADR-0007) and streaming the plan-lifecycle `AgentEvent`s
//! out through a `tokio::sync::mpsc::UnboundedSender`. Task *execution*
//! is [`crate::sdk::AgentSdk::run_agent`] (M07.D2); this module is the
//! M04-🟡 driver *shell* that advances the FSM — it does not itself run
//! tasks.

use std::time::Duration;

use runtime_core::event::{AgentEvent, ApprovedBy};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

use crate::hitl::{HitlError, HitlSeam};
use crate::plan::state_machine::{
    PlanEvent, PlanState, PlanStateMachine, PlanStatus, TransitionError,
};

/// The HITL choice token that approves a plan.
///
/// Any other token (the renderer sends `"reject"`) aborts the plan —
/// fail-closed: an unrecognised answer never advances a plan past its
/// approval gate.
const APPROVE_TOKEN: &str = "approve";

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
    let started = std::time::Instant::now();
    // A disconnected renderer must not abort the plan FSM — drop the
    // SendError. `events.send` takes `&self`, so this stays an `Fn`.
    let emit = |event: AgentEvent| {
        let _ = events.send(event);
    };

    // 1. Approval gate — only a plan created approval-required is
    //    PendingApproval; await the in-process HITL seam.
    if plan.status == PlanStatus::PendingApproval {
        emit(AgentEvent::PlanApprovalRequested {
            plan_id: plan.plan_id.clone(),
        });
        let choice = hitl.await_response(&plan.plan_id, approval_timeout).await?;
        if choice.token == APPROVE_TOKEN {
            PlanStateMachine::transition(plan, PlanEvent::Approved)?;
            emit(AgentEvent::PlanApproved {
                plan_id: plan.plan_id.clone(),
                approved_by: ApprovedBy::User,
            });
        } else {
            PlanStateMachine::transition(plan, PlanEvent::Aborted)?;
            emit(AgentEvent::PlanAborted {
                plan_id: plan.plan_id.clone(),
                reason: format!("rejected at the approval gate (chose '{}')", choice.token),
            });
            return Ok(());
        }
    } else if plan.status == PlanStatus::Approved {
        // approval_required = false — PlanState::new starts the plan
        // Approved; the SDK auto-approves without a HITL round-trip.
        emit(AgentEvent::PlanApproved {
            plan_id: plan.plan_id.clone(),
            approved_by: ApprovedBy::Auto,
        });
    }
    // Any other entry state (terminal / mid-execution) falls through to
    // the transition below, which surfaces the FSM rejection as a
    // PlanLoopError::Transition driver-bug error.

    // 2. Approved → InProgress.
    PlanStateMachine::transition(plan, PlanEvent::Started)?;

    // 3. Task execution is AgentSdk::run_agent (M07.D2) — this shell
    //    drives only the FSM; there is no task loop here.

    // 4. InProgress → Complete.
    PlanStateMachine::transition(plan, PlanEvent::Complete)?;
    emit(AgentEvent::PlanComplete {
        plan_id: plan.plan_id.clone(),
        duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    });
    Ok(())
}
