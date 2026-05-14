//! Error types for the §8.security L4 tier evaluator (M05 Stage D).

use runtime_core::generated::capability::CapabilityKind;
use thiserror::Error;

use crate::tier::evaluator::Tier;

/// Errors raised by [`TierEvaluator::allows`].
///
/// Distinct from [`CapabilityError::Denied`] — the L1 enforcer's
/// "no matching grant" rejection — because the renderer routes
/// `tier_violation` events differently from `capability_violation`.
///
/// [`TierEvaluator::allows`]: crate::tier::TierEvaluator::allows
/// [`CapabilityError::Denied`]: crate::capability::CapabilityError::Denied
#[derive(Debug, Clone, Error)]
pub enum TierError {
    /// The requested capability is not permitted for this tier's
    /// allowlist. Carries the tier and the requested kind so the
    /// `tier_violation` event can surface both in the renderer.
    #[error("capability kind {capability_kind:?} forbidden in tier {tier:?}")]
    ForbiddenInTier {
        /// The tier that rejected the request.
        tier: Tier,
        /// The coarse capability kind the request declared.
        capability_kind: CapabilityKind,
    },
}

/// Errors raised by tier persistence ([`load_tier`] / [`save_tier`]).
///
/// [`load_tier`]: crate::tier::load_tier
/// [`save_tier`]: crate::tier::save_tier
#[derive(Debug, Error)]
pub enum TierPersistenceError {
    /// Filesystem I/O error (read, write, or directory creation).
    #[error("tier persistence I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialize / deserialize error on `tier.json`.
    #[error("tier persistence JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
