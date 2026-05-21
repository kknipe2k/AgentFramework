//! Framework save/load (M08 Stage B).
//!
//! Path-agnostic persistence per CLAUDE.md §9 (the `audit::file_path` /
//! `skills_lock` archetype): the functions take `dir: &Path`; the Tauri
//! shell resolves the directory (from the save dialog / file picker,
//! wired in Stage C) and passes it in. No workspace dependency on
//! `dirs`. Tested with `tempfile`-backed paths.
//!
//! [`save_framework`] writes `framework.json` (pretty-printed, stable
//! field order, trailing newline) plus one companion `.md` per inline
//! artifact; [`load_framework`] parses `framework.json` into the same
//! typify-generated `Framework` type the rest of the runtime uses — no
//! second loader (spec §9). A save→load→save cycle is byte-stable
//! (MVP §M8 criterion 8).

use std::path::Path;

use runtime_core::generated::framework::Framework;

use crate::builder::error::BuilderError;

/// One inline-defined artifact's companion markdown file.
///
/// Derives `serde` both ways: it crosses the Tauri IPC boundary as a
/// `save_framework` argument and a `load_framework` return value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Companion {
    /// File name relative to the framework directory — e.g.
    /// `summarize.skill.md`.
    pub file_name: String,
    /// Full markdown body (frontmatter + content), written verbatim.
    pub body: String,
}

/// A framework reloaded from disk — the canvas reconstructs from this.
///
/// Derives `serde::Serialize` (it crosses the Tauri IPC boundary as the
/// `load_framework` return value). The round-trip is byte-stable so the
/// canvas reconstructs identical to save state (the ADR-0020
/// canvas-projection contract, anticipated here, filed at Stage C).
#[derive(Debug, Clone, serde::Serialize)]
pub struct LoadedFramework {
    /// The parsed `framework.json`.
    pub framework: Framework,
    /// The companion markdown files found alongside it, sorted by file
    /// name for a deterministic (byte-stable) round-trip.
    pub companions: Vec<Companion>,
}

/// Write `framework.json` + a companion `.md` for every inline artifact.
///
/// Writes `dir/framework.json` (pretty-printed, stable field order, one
/// trailing newline) plus one file per `companions` entry. `dir` is
/// created if absent.
///
/// # Errors
///
/// - [`BuilderError::NotADirectory`] if `dir` exists and is a file.
/// - [`BuilderError::Json`] if the framework cannot serialize.
/// - [`BuilderError::Io`] on any write failure.
pub fn save_framework(
    _dir: &Path,
    _fw: &Framework,
    _companions: &[Companion],
) -> Result<(), BuilderError> {
    todo!("M08.B green phase")
}

/// Read `framework.json` + its companion `.md` files back from `dir`.
///
/// Parses `framework.json` into the same `Framework` type
/// `framework_loader` uses — gap-tolerantly (a partially-built
/// framework with unresolved references reloads fine; the Builder
/// surfaces gaps as red badges, it does not refuse the load). Companion
/// files are the `*.skill.md` / `*.tool.md` / `*.agent.md` files
/// alongside `framework.json`.
///
/// # Errors
///
/// - [`BuilderError::Io`] if `dir/framework.json` is missing/unreadable.
/// - [`BuilderError::Json`] if `framework.json` does not parse.
pub fn load_framework(_dir: &Path) -> Result<LoadedFramework, BuilderError> {
    todo!("M08.B green phase")
}
