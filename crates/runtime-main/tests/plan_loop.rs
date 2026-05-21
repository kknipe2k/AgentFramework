//! `plan_loop::drive_plan` driver-shell behavior tests — M08 Stage A,
//! the M04-🟡 `plan_loop` carry-forward.
//!
//! Exercises the FSM-walk paths the driver shell discharges:
//! approval-required → approve → `Complete`; approval-required → reject
//! → `Aborted`; no-approval → `Complete`; and the two error paths (HITL
//! gate timeout, illegal FSM transition). The driver routes the
//! approval gate through the in-process [`HitlSeam`] (ADR-0007) and
//! streams plan-lifecycle events out through an `mpsc::UnboundedSender`
//! (the codebase emission pattern). Tests resolve the seam concurrently
//! via `tokio::join!` — the driver borrows `&mut PlanState`, so it
//! cannot be `spawn`ed.

use std::time::Duration;

use runtime_core::event::AgentEvent;
use runtime_main::hitl::{HitlChoice, HitlSeam};
use runtime_main::plan::{drive_plan, PlanLoopError, PlanState, PlanStatus};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

/// Generous HITL wait — long enough that the concurrent resolver always
/// wins; the timeout path is exercised separately with a short wait.
const GENEROUS: Duration = Duration::from_secs(5);

/// Spin until `drive_plan` has registered its HITL await on `seam`, then
/// resolve it with `token`. Panics if the await never registers.
async fn resolve_when_pending(seam: &HitlSeam, prompt_id: &str, token: &str) {
    for _ in 0..500 {
        if seam.pending_len().await == 1 {
            seam.resolve(prompt_id, HitlChoice::new(token))
                .await
                .expect("resolve the registered HITL await");
            return;
        }
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    panic!("drive_plan never registered a HITL await on the seam");
}

/// Drain every buffered event from `rx` — the driver consumed + dropped
/// its sender by the time it returns, so the receiver is disconnected.
fn drain(rx: &mut UnboundedReceiver<AgentEvent>) -> Vec<AgentEvent> {
    let mut out = Vec::new();
    while let Ok(event) = rx.try_recv() {
        out.push(event);
    }
    out
}

#[tokio::test]
async fn drive_plan_approval_required_runs_to_complete_on_approve() {
    let seam = HitlSeam::new();
    let mut plan = PlanState::new("p1", true);
    assert_eq!(plan.status, PlanStatus::PendingApproval);
    let (tx, _rx) = unbounded_channel::<AgentEvent>();

    let (res, ()) = tokio::join!(
        drive_plan(&mut plan, &seam, GENEROUS, tx),
        resolve_when_pending(&seam, "p1", "approve"),
    );

    res.expect("drive_plan returns Ok on an approved plan");
    assert_eq!(
        plan.status,
        PlanStatus::Complete,
        "an approved plan walks the FSM through to Complete"
    );
}

#[tokio::test]
async fn drive_plan_approval_required_aborts_plan_on_reject() {
    let seam = HitlSeam::new();
    let mut plan = PlanState::new("p1", true);
    let (tx, _rx) = unbounded_channel::<AgentEvent>();

    let (res, ()) = tokio::join!(
        drive_plan(&mut plan, &seam, GENEROUS, tx),
        resolve_when_pending(&seam, "p1", "reject"),
    );

    res.expect("drive_plan returns Ok after a clean rejection");
    assert_eq!(
        plan.status,
        PlanStatus::Aborted,
        "a rejected plan lands in Aborted, not Complete"
    );
}

#[tokio::test]
async fn drive_plan_no_approval_required_runs_through_to_complete() {
    let seam = HitlSeam::new();
    // approval_required = false → PlanState::new starts the plan Approved.
    let mut plan = PlanState::new("p1", false);
    assert_eq!(plan.status, PlanStatus::Approved);
    let (tx, mut rx) = unbounded_channel::<AgentEvent>();

    drive_plan(&mut plan, &seam, GENEROUS, tx)
        .await
        .expect("a no-approval plan needs no HITL round-trip");

    assert_eq!(plan.status, PlanStatus::Complete);
    assert_eq!(
        seam.pending_len().await,
        0,
        "the no-approval path never touches the HITL seam"
    );
    let events = drain(&mut rx);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::PlanApproved { approved_by, .. }
                if matches!(approved_by, runtime_core::event::ApprovedBy::Auto))),
        "a no-approval plan emits plan_approved with approved_by = auto"
    );
}

#[tokio::test]
async fn drive_plan_emits_plan_lifecycle_events_in_order_on_approve() {
    let seam = HitlSeam::new();
    let mut plan = PlanState::new("p1", true);
    let (tx, mut rx) = unbounded_channel::<AgentEvent>();

    let (res, ()) = tokio::join!(
        drive_plan(&mut plan, &seam, GENEROUS, tx),
        resolve_when_pending(&seam, "p1", "approve"),
    );
    res.expect("drive_plan ok");

    let events = drain(&mut rx);
    let kinds: Vec<&str> = events
        .iter()
        .map(|e| match e {
            AgentEvent::PlanApprovalRequested { .. } => "plan_approval_requested",
            AgentEvent::PlanApproved { .. } => "plan_approved",
            AgentEvent::PlanComplete { .. } => "plan_complete",
            AgentEvent::PlanAborted { .. } => "plan_aborted",
            _ => "other",
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["plan_approval_requested", "plan_approved", "plan_complete"],
        "the driver emits the plan lifecycle in approve → start → complete order"
    );
}

#[tokio::test]
async fn drive_plan_reject_emits_plan_aborted_and_no_plan_complete() {
    let seam = HitlSeam::new();
    let mut plan = PlanState::new("p1", true);
    let (tx, mut rx) = unbounded_channel::<AgentEvent>();

    let (res, ()) = tokio::join!(
        drive_plan(&mut plan, &seam, GENEROUS, tx),
        resolve_when_pending(&seam, "p1", "reject"),
    );
    res.expect("drive_plan ok");

    let events = drain(&mut rx);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::PlanAborted { plan_id, .. } if plan_id == "p1")),
        "a rejected plan emits plan_aborted"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, AgentEvent::PlanComplete { .. })),
        "a rejected plan never reaches plan_complete"
    );
}

#[tokio::test]
async fn drive_plan_illegal_transition_surfaces_plan_loop_error() {
    // Driver-bug path: a plan already in a terminal state cannot be
    // driven. The shell skips the approval gate (not PendingApproval,
    // not Approved) and the first FSM transition it attempts is
    // rejected.
    let seam = HitlSeam::new();
    let mut plan = PlanState {
        plan_id: "p1".to_string(),
        status: PlanStatus::Complete,
    };
    let (tx, _rx) = unbounded_channel::<AgentEvent>();

    let err = drive_plan(&mut plan, &seam, GENEROUS, tx)
        .await
        .expect_err("driving a Complete plan is an illegal transition");
    assert!(
        matches!(err, PlanLoopError::Transition(_)),
        "an illegal FSM transition surfaces as PlanLoopError::Transition, got {err:?}"
    );
}

#[tokio::test]
async fn drive_plan_hitl_timeout_surfaces_plan_loop_error() {
    // The approval gate is never resolved; the short wait elapses and
    // the HITL error propagates as PlanLoopError::Hitl.
    let seam = HitlSeam::new();
    let mut plan = PlanState::new("p1", true);
    let (tx, _rx) = unbounded_channel::<AgentEvent>();

    let err = drive_plan(&mut plan, &seam, Duration::from_millis(20), tx)
        .await
        .expect_err("an unanswered approval gate times out");
    assert!(
        matches!(err, PlanLoopError::Hitl(_)),
        "a HITL gate failure surfaces as PlanLoopError::Hitl, got {err:?}"
    );
}
