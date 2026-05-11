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

pub use state_machine::{
    PlanEvent, PlanState, PlanStateMachine, PlanStatus, TaskEvent, TaskState, TaskStateMachine,
    TaskStatus, TransitionError,
};
