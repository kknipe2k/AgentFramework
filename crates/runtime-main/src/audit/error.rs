//! Error type for the §8.security L5 audit writer (M05 Stage E).
//!
//! Distinct from the other primitives' error enums because audit failures
//! are observability-class — best-effort — and surface to the call site
//! only so the caller can `tracing::error!` and continue. Per phase doc
//! E.3.4 + spec §13.5 dev-logging discipline: audit availability is not
//! a dispatch gate.

use thiserror::Error;

/// Errors raised by [`crate::audit::AuditWriter::open`] +
/// [`crate::audit::AuditWriter::log`].
///
/// Callers MUST NOT propagate these into dispatch — log via
/// `tracing::error!` and continue. The two layers (open vs. log) are
/// split because open failure is fatal-on-this-installation (no audit
/// trail will ever land) while log failure is per-entry (typically
/// transient disk pressure / permission flip).
#[derive(Debug, Error)]
pub enum AuditError {
    /// Filesystem I/O error (open, write, flush, or directory create).
    #[error("audit I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialize error on an `AuditEntry`.
    ///
    /// Rare — the entry shape derives Serialize and the writer never
    /// sees a non-serializable variant — but surface as a structured
    /// error rather than panic so the call site can log and continue.
    #[error("audit JSON serialize failed: {0}")]
    Json(#[from] serde_json::Error),
}
