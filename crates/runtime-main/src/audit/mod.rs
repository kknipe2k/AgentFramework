//! Audit log — spec §8.security L5 (M05 Stage E).
//!
//! Appends one newline-delimited JSON record to `skills.audit.jsonl`
//! per security-relevant event: framework load, gap detection, gap
//! resolution, capability grant, capability denial, tier transition.
//!
//! v0.1 ships:
//! - File-based JSONL writer (no SQLite; flat file in app data dir)
//! - One line per event; structured JSON; UTC unix-ms timestamps
//! - No rotation, no hash-chain, no per-event provenance block — those
//!   land post-v0.1 per the §0d release scope matrix
//!
//! Best-effort observability: write failures `tracing::error!` and
//! continue. Audit availability is NOT a dispatch gate per phase doc
//! E.3.4 + spec §13.5 dev-logging discipline.
//!
//! Wiring (M05 Stage E): the Stage A framework_loader, Stage B
//! capability enforcer, and Stage D tier evaluator each hold an
//! `Option<Arc<AuditWriter>>`. Optional rather than required so the
//! existing default-constructible surfaces ([`Default`] on
//! `CapabilityEnforcer`, plain construction of `framework_loader`) stay
//! usable in unit tests that don't care about audit. Production wires
//! `Some(writer)` from the Tauri shell at app startup.
//!
//! Safety primitive: ≥95% per-module coverage gate per CLAUDE.md §5.

/// Per-kind constructors for `AuditEntry`.
///
/// Pin the `details` shape at the call site so renderers + maintainers
/// grep by `kind` and find a consistent payload.
pub mod entry;
/// `AuditError` enum — best-effort observability surface.
pub mod error;
/// Audit log path resolution helpers.
///
/// File-name constant + `audit_path(dir)` join. The Tauri layer owns
/// the directory half via `AppHandle::path().app_local_data_dir()`.
pub mod file_path;
/// `AuditWriter` — mutex-guarded append-only JSONL writer.
pub mod writer;

pub use entry::{
    capability_denied, capability_granted, framework_loaded, gap_detected, gap_resolved,
    mcp_auth_granted, mcp_installed, mcp_uninstalled, tier_transition,
};
pub use error::AuditError;
pub use file_path::{audit_path, AUDIT_FILE_NAME};
pub use writer::AuditWriter;
