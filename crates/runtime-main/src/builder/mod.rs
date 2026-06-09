//! Builder backend — spec Phase 9 "Visual Canvas and Tester" (M08 Stage B).
//!
//! The single backend the Builder Canvas (Stages D1/D2), the Inspector
//! (E), and the Tester (F1) share. There is exactly **one** validator,
//! **one** capability summary, **one** save/load path — spec §9 forbids
//! duplicating validation logic between TS and Rust.
//!
//! Four surfaces, all reuse-heavy by mandate:
//!
//! - `validate::validate_framework` composes schema-shape validation
//!   (serde into the typify-generated `Framework` type — the project's
//!   schema-as-source-of-truth mechanism, CLAUDE.md §14), reference
//!   validation (`framework_loader::walk`), and the capability summary
//!   into one `FrameworkValidationReport`.
//! - `summary::framework_capability_summary` aggregates
//!   `framework_loader::capability_map` into whole-framework totals and
//!   carries, per Agent→Agent spawn edge, the narrowing triple computed
//!   by the reused `capability::narrowing::narrow`.
//! - `persist::save_framework` / `persist::load_framework` —
//!   path-agnostic `&Path` persistence (CLAUDE.md §9 archetype — the
//!   Tauri shell resolves the directory).
//! - `validate::list_installed` — the first production `skills.lock`
//!   reader (closes M07-IRL #6 + the read half of M07.V 🟡 #2).
//!
//! The module is pure / seam / `tempfile`-tested — every filesystem call
//! is `&Path`-parameterised and tempfile-reachable, so it adds no new
//! coverage exclusion (the `skills_lock` ≥95 precedent).

/// Error surface for the Builder backend.
pub mod error;
/// Framework save/load — path-agnostic `&Path` persistence.
pub mod persist;
/// Whole-framework capability summary + per-spawn-edge narrowing.
pub mod summary;
/// The Builder's Tester — isolated, throwaway test session (Stage F1).
pub mod tester;
/// Continuous framework validation + the `skills.lock` reader.
pub mod validate;

pub use error::BuilderError;
pub use persist::{load_framework, save_framework, Companion, LoadedFramework};
pub use summary::{framework_capability_summary, FrameworkCapabilitySummary, SpawnEdgeNarrowing};
pub use tester::{
    fold_outcome, load_verified_artifact, run_test_session_with, run_test_session_with_skills,
    run_test_session_with_tier, run_test_session_with_tools, CapabilityFailure, TestOutcome,
    TesterError, TokenSpend,
};
pub use validate::{
    list_installed, validate_framework, FrameworkValidationReport, InstalledArtifact, NodeError,
};
