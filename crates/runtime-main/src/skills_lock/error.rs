//! Error type for the `skills.lock` integrity primitive (M07 Stage B,
//! spec §2181-2216; ADR-0014).
//!
//! Distinct from the audit primitive's best-effort error: a hash
//! mismatch is a HARD block (integrity > availability — a tampered or
//! drifted artifact must not silently run), not observability. The load
//! path maps `LockError::HashMismatch` onto the schema-faithful
//! `AgentEvent::ArtifactHashMismatch` and refuses to load the artifact.

use thiserror::Error;

/// Errors raised by `crate::skills_lock::read`,
/// `crate::skills_lock::write_entry`, and
/// `crate::skills_lock::verify`.
#[derive(Debug, Error)]
pub enum LockError {
    /// The artifact's recomputed SRI content hash does not match the
    /// hash locked in `skills.lock`. The load path BLOCKS the artifact
    /// and surfaces a Reinstall / Remove prompt (spec §2214). The three
    /// fields map 1:1 onto `AgentEvent::ArtifactHashMismatch`.
    #[error("content hash mismatch for {artifact_ref}: locked {expected}, computed {actual}")]
    HashMismatch {
        /// `name@version` of the drifted artifact.
        artifact_ref: String,
        /// The SRI hash recorded in `skills.lock` at install time.
        expected: String,
        /// The SRI hash recomputed over the bytes on disk now.
        actual: String,
    },
    /// `verify` was asked about an artifact that is not in the lock.
    /// Treated as a block (not a silent pass) so an artifact that was
    /// never installed-and-locked cannot load by virtue of an absent
    /// entry.
    #[error("no skills.lock entry for {0}")]
    NotFound(String),
    /// Filesystem I/O error reading or writing the lock file (missing
    /// lock file on read, missing parent directory on write, permission
    /// denied, disk pressure).
    #[error("skills.lock I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// The lock file is not valid JSON or does not match the
    /// schema-derived `SkillsLock` shape (e.g. a non-SRI `content_hash`).
    #[error("skills.lock parse failed: {0}")]
    Parse(#[from] serde_json::Error),
}
