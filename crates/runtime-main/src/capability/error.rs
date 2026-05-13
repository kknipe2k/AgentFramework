//! Error types for the §8.security L1 enforcer + L2a narrowing evaluator.

use runtime_core::generated::capability::CapabilityDeclaration;
use thiserror::Error;

use crate::capability::enforcer::DenyReason;

/// Errors raised by [`CapabilityEnforcer::check`].
///
/// [`CapabilityEnforcer::check`]: crate::capability::CapabilityEnforcer::check
#[derive(Debug, Clone, Error)]
pub enum CapabilityError {
    /// The agent's declared grants do not cover the requested capability.
    /// Inspect `reason` to distinguish "no declarations at all"
    /// (default-deny) from "declarations present but none satisfy this
    /// request" — the renderer surfaces different copy per reason.
    #[error("capability denied for agent {agent_id}: {reason:?}")]
    Denied {
        /// The agent whose dispatch was rejected.
        agent_id: String,
        /// Why the dispatch was denied.
        reason: DenyReason,
    },
}

/// Errors raised by [`narrow`].
///
/// Carries the offending proposed declaration so the caller can surface
/// the specific mismatch (and so the L2a `capability_violation` event
/// can name what the child tried to claim that the parent doesn't have).
///
/// [`narrow`]: crate::capability::narrow
#[derive(Debug, Clone, Error)]
pub enum NarrowingError {
    /// One of the proposed child capabilities is not subsumed by any
    /// parent grant. The child is attempting to widen the parent's
    /// scope — denied.
    #[error("proposed capability is not held by parent: {proposed:?}")]
    CapabilityNotHeldByParent {
        /// The proposed declaration that the parent's grants do not
        /// subsume.
        proposed: Box<CapabilityDeclaration>,
    },
}
