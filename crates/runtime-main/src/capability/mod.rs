//! Capability enforcement — spec §8.security L1 + L2a (M05 Stage B).
//!
//! Two-layer in-process check that gates every tool dispatch + sub-agent
//! spawn against the calling agent's declared capability grants.
//!
//! - `enforcer::CapabilityEnforcer` — L1 check. Owns per-agent grants;
//!   `check(agent, requested)` returns `Ok(())` when at least one grant
//!   subsumes the request; default-deny otherwise.
//! - `narrowing::narrow` — L2a evaluator. `narrow(parent, proposed)`
//!   returns the (possibly clamped) child grants when every proposed
//!   capability is subsumed by some parent grant; errors otherwise.
//! - `declaration::subsumes` — pure-function predicate. Asymmetric:
//!   `parent.subsumes(child)` means the parent's grant covers everything
//!   the child requests. The asymmetry is load-bearing — flipping the
//!   direction would let children widen parent grants.
//!
//! L3 sandbox (out-of-process validation for generated artifacts), L4
//! tier gates, and L5 provenance/audit are subsequent stages. Stage B's
//! enforcer exposes no IO surface: callers feed it parsed
//! `CapabilityDeclaration`s and observe `Ok`/`Err`. Event emission is the
//! SDK's responsibility — Stage B integrates via the same in-process
//! emitter pattern as the framework_loader (ADR-0007).
//!
//! Default-deny semantics are non-negotiable (gotcha trap #1 from the
//! M05.B stage prompt): an agent with no declarations gets `Err`, not
//! `Ok`. Tests pin this explicitly via the `enforcer::DenyReason`
//! discriminator.
//!
//! Safety primitive: ≥95% coverage gate per CLAUDE.md §5.

/// Capability declaration helpers — predicates over the typify-generated
/// [`CapabilityDeclaration`] type.
///
/// Free functions rather than inherent methods because typify-generated
/// types live in `runtime-core::generated::capability` and cannot be
/// extended in this crate.
///
/// [`CapabilityDeclaration`]: runtime_core::generated::capability::CapabilityDeclaration
pub mod declaration;
/// L1 enforcer — default-deny `check` against per-agent grants.
pub mod enforcer;
/// Error types raised by the enforcer + narrowing evaluator.
pub mod error;
/// L2a narrowing — parent → child capability subset enforcement.
pub mod narrowing;

pub use declaration::{scope_contains, subsumes};
pub use enforcer::{CapabilityEnforcer, DenyReason};
pub use error::{CapabilityError, NarrowingError};
pub use narrowing::narrow;
