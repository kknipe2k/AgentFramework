//! Continuous framework validation + the `skills.lock` reader (M08
//! Stage B) — the single validator the Canvas, Inspector, and Tester
//! share.
//!
//! [`validate_framework`] composes the typify-generated `Framework`
//! shape check + [`crate::framework_loader::walk`] +
//! [`framework_capability_summary`]. There is **no** second validator in
//! TS (spec §9) and **no** Rust JSON-Schema library — the generated
//! `Framework` type *is* the schema-as-source-of-truth check
//! (CLAUDE.md §14).
//!
//! [`list_installed`] is the first production `skills.lock` reader — it
//! flattens the lock's `installed` map for the Palette / Import panel
//! (closes M07-IRL #6 + the read half of M07.V 🟡 #2).
//!
//! [`framework_capability_summary`]: crate::builder::summary::framework_capability_summary

use std::path::Path;

use serde_json::Value;

use runtime_core::generated::skills_lock::{ArtifactKind, Source};

use crate::builder::error::BuilderError;
use crate::builder::summary::FrameworkCapabilitySummary;

/// One validation problem, keyed to the offending node / JSON-path so
/// the renderer (D2 red badges, E Validate result) can attribute it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NodeError {
    /// JSON-path or node id the error attaches to. For a reference /
    /// narrowing problem this is the agent id (e.g. `worker`); for a
    /// schema-shape failure it is `(root)` — the whole document failed
    /// to match the `Framework` shape (v0.1 has no Rust JSON-Schema
    /// validator, so the serde error has no structured JSON-path; the
    /// `message` names the offending field).
    pub node_path: String,
    /// Human-readable problem description.
    pub message: String,
}

/// Structured validation report. `ok` iff both error lists are empty.
///
/// Crosses the Tauri IPC boundary as one report — the renderer reads
/// one report, not a second command. Carries no `PartialEq`: the
/// embedded [`FrameworkCapabilitySummary`] holds
/// `runtime_core` `CapabilityDeclaration`s, which derive no `PartialEq`
/// (their `CapabilityScope` `oneOf` wrapper derives only `Clone` +
/// `Debug`). Idempotency tests compare via `serde_json::to_value`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FrameworkValidationReport {
    /// Schema-shape problems — failed deserialization into `Framework`,
    /// pattern-constraint violations.
    pub schema_errors: Vec<NodeError>,
    /// Capability / reference problems — unresolved `tool` / `skill` /
    /// `agent` references, failed Agent→Agent capability narrowing.
    pub capability_errors: Vec<NodeError>,
    /// `schema_errors.is_empty() && capability_errors.is_empty()`.
    pub ok: bool,
    /// The whole-framework capability summary — carries the per-Agent→
    /// Agent-edge narrowing triple. `None` when schema validation fails
    /// (no parsed `Framework` to summarize). Rides on the report so the
    /// Inspector (E) and the canvas (D2) render one capability picture
    /// from one backend computation.
    pub capability_summary: Option<FrameworkCapabilitySummary>,
}

/// Validate an in-progress framework document.
///
/// Pure: bytes in, report out. No filesystem, no network. The seam the
/// `validate_framework` Tauri command wraps. The document may be
/// incomplete or invalid — that is the point; continuous validation
/// runs as the user edits the canvas.
///
/// A schema-shape failure (the document does not deserialize into the
/// typify-generated `Framework`) short-circuits: reference + capability
/// checks need a parsed `Framework`, and `capability_summary` is `None`
/// because there is nothing to summarize.
#[must_use]
pub fn validate_framework(_doc: &Value) -> FrameworkValidationReport {
    todo!("M08.B green phase")
}

/// One installed artifact, flattened from a `skills.lock` entry for the
/// Palette / Import panel (Stage C consumes via `list_installed_artifacts`).
///
/// Carries no `PartialEq`: [`Source`] is a typify `oneOf` that derives
/// only `Clone` + `Debug`. Tests compare via `serde_json::to_value` or
/// per-field.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstalledArtifact {
    /// The `name@version` lock key.
    pub key: String,
    /// `skill` / `tool` / `agent` / `mcp_server`.
    pub kind: ArtifactKind,
    /// Where the artifact was imported from (URL or local file).
    pub source: Source,
    /// RFC-3339 install timestamp (from the lock entry's `installed_at`).
    pub installed_at: String,
}

/// Read every installed artifact from the `skills.lock` at `lock_path`.
///
/// The result is sorted by `key` so the Palette ordering is stable (the
/// lock's `installed` map is a `HashMap` — unordered in memory).
///
/// An **absent** lock returns `Ok(vec![])` — a framework with nothing
/// installed is valid, not an error (the M07-IRL #6 fix: the Import
/// panel calls this on startup and gets an empty list rather than a
/// failure). A **present-but-corrupt** lock returns
/// [`BuilderError::Lock`].
///
/// # Errors
///
/// [`BuilderError::Lock`] when the lock file exists but is corrupt /
/// not schema-valid. Every non-`NotFound` lock failure propagates — an
/// over-broad catch that treated a corrupt lock as empty would be a
/// silent-failure bug.
pub fn list_installed(_lock_path: &Path) -> Result<Vec<InstalledArtifact>, BuilderError> {
    todo!("M08.B green phase")
}
