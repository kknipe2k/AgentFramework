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
    dir: &Path,
    fw: &Framework,
    companions: &[Companion],
) -> Result<(), BuilderError> {
    if dir.exists() && !dir.is_dir() {
        return Err(BuilderError::NotADirectory(dir.display().to_string()));
    }
    std::fs::create_dir_all(dir)?;
    // Pretty-printed + a single trailing newline gives a deterministic
    // serialization — a save→load→save cycle is byte-stable.
    let json = serde_json::to_string_pretty(fw)?;
    std::fs::write(dir.join("framework.json"), format!("{json}\n"))?;
    for companion in companions {
        std::fs::write(dir.join(&companion.file_name), &companion.body)?;
    }
    Ok(())
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
pub fn load_framework(dir: &Path) -> Result<LoadedFramework, BuilderError> {
    let raw = std::fs::read_to_string(dir.join("framework.json"))?;
    // Parse into the same typify-generated `Framework` type the rest of
    // the runtime uses — no second loader (spec §9). Gap-tolerant: a
    // partially-built framework reloads fine; the Builder surfaces gaps
    // via `validate_framework`, it does not refuse the load.
    let framework: Framework = serde_json::from_str(&raw)?;
    let companions = read_companions(dir)?;
    Ok(LoadedFramework {
        framework,
        companions,
    })
}

/// Scan `dir` for companion markdown files and read each one.
///
/// Companions are the `*.skill.md` / `*.tool.md` / `*.agent.md` files
/// alongside `framework.json`; the result is sorted by file name for a
/// deterministic (byte-stable) round-trip.
fn read_companions(dir: &Path) -> Result<Vec<Companion>, BuilderError> {
    let mut companions: Vec<Companion> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if is_companion_md(file_name) {
            companions.push(Companion {
                file_name: file_name.to_string(),
                body: std::fs::read_to_string(&path)?,
            });
        }
    }
    companions.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(companions)
}

/// Whether `name` is a companion markdown file the canvas round-trips.
fn is_companion_md(name: &str) -> bool {
    name.ends_with(".skill.md") || name.ends_with(".tool.md") || name.ends_with(".agent.md")
}
