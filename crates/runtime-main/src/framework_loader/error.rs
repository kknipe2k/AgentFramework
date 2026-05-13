//! Errors raised by [`crate::framework_loader::load_and_validate`].
//!
//! `Io` wraps disk read failures; `Json` wraps `serde_json` parse failures;
//! `GapsFound` is the structural "framework parsed but references don't
//! resolve" outcome — caller knows the loader emitted one gap event per
//! unresolved reference via the supplied emitter and may choose how to
//! handle the suspension (HITL on_gap trigger per spec §6a).

use thiserror::Error;

/// Failure modes the framework loader surfaces.
#[derive(Debug, Error)]
pub enum FrameworkLoadError {
    /// Disk read failed.
    #[error("framework JSON read failed: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parse failed.
    #[error("framework JSON parse failed: {0}")]
    Json(#[from] serde_json::Error),
    /// Framework parsed but `count` references did not resolve. Gap
    /// events were emitted via the supplied emitter before this error
    /// returned; the caller routes the suspension flow (per spec §4b).
    #[error("framework has {count} unresolved reference(s); see gap events")]
    GapsFound {
        /// Number of gaps emitted.
        count: usize,
    },
}
