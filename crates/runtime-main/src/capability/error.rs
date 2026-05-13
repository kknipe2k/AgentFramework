//! Error types for the §8.security L1 enforcer + L2a narrowing evaluator
//! + L4 tier gate (M05 Stage D).

use runtime_core::generated::capability::{CapabilityDeclaration, CapabilityKind};
use thiserror::Error;

use crate::capability::enforcer::DenyReason;
use crate::tier::Tier;

/// Errors raised by [`CapabilityEnforcer::check`].
///
/// Three failure shapes:
/// - [`Self::TierForbidden`] — L4 rejected the request before L1 ran.
///   Renderer routes `tier_violation` event.
/// - [`Self::Denied`] — L1 rejected: either no declarations
///   ([`DenyReason::NoDeclarations`]) or declarations present but
///   nothing subsumes the request ([`DenyReason::NoMatchingGrant`]).
///   Renderer routes `capability_violation` event.
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
    /// The L4 tier gate rejected the request — the user's current tier
    /// does not permit this capability kind at all, regardless of
    /// per-agent grants. Carries the tier and the requested kind so the
    /// renderer's `tier_violation` event surfaces both.
    #[error("capability {capability_kind:?} forbidden in tier {tier:?} for agent {agent_id}")]
    TierForbidden {
        /// The agent whose dispatch was rejected.
        agent_id: String,
        /// The tier that rejected the request.
        tier: Tier,
        /// The coarse kind that the tier's allowlist excludes.
        capability_kind: CapabilityKind,
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
