//! Plan & Task primitive — spec §3a.
//!
//! Holds the pure-logic state machines for the plan/task lifecycle.
//!
//! Exposed surface (re-exported below): `PlanStateMachine` /
//! `TaskStateMachine` exhaustive transition validators (safety primitive,
//! ≥95% coverage); `PlanState` / `TaskState` owned state structs the
//! FSMs mutate; `TransitionError` typed transition errors.
//!
//! M04 Stage B authored the FSM; Stage C wires the renderer surface.

/// Plan + Task state machines (spec §3a). Pure logic, no I/O.
pub mod state_machine;

/// Plan-driver loop (spec §3a) — the M04-🟡 driver shell that walks the
/// FSM through approval + execution. M08 Stage A.
pub mod plan_loop;

pub use plan_loop::{drive_plan, PlanLoopError};
pub use state_machine::{
    PlanEvent, PlanState, PlanStateMachine, PlanStatus, TaskEvent, TaskState, TaskStateMachine,
    TaskStatus, TransitionError,
};
