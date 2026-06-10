//! Renderer-supplied path containment (M09.5.A / TD-051, external
//! review C2).
//!
//! The OS file dialog is a UX affordance, not an enforcement boundary —
//! a direct `invoke("save_framework", {dir: "../../anywhere"})` reaches
//! `fs` unconfined on the pre-fix tree. `confine` is the enforcement:
//! it canonicalizes a renderer-supplied path and accepts it only when
//! the result is strictly contained under one of the allow-listed roots
//! (the dialog-registered directories ∪ the app's local data dir).
//!
//! Save targets often do not exist yet (`save_framework` creates the
//! directory), so `confine` canonicalizes the **nearest existing
//! ancestor** and re-joins the non-existing remainder — rejecting any
//! `..` component in that remainder, which cannot be resolved away by
//! the filesystem and would otherwise let a target escape its root.
//!
//! Symlink policy: resolve-then-check. `std::fs::canonicalize` follows
//! symlinks, so a link inside a root that points outside it resolves to
//! the outside target and is therefore refused — the same escape the
//! review proved against the built-in tools (TD-052), closed here at
//! the shell perimeter.
//!
//! Pure + std-only (path-agnostic persistence archetype, CLAUDE.md §9):
//! no Tauri dependency, `&Path` + `&[PathBuf]` in, `tempfile`-tested.

use std::path::{Component, Path, PathBuf};

/// Why a renderer-supplied path was refused.
#[derive(Debug, thiserror::Error)]
pub enum ConfineError {
    /// The path resolved outside every allow-listed root (or no root was
    /// registered). Carries a human-readable explanation for the typed
    /// `CmdError::PathNotPermitted` the shell surfaces.
    #[error("path is not under any permitted root: {0}")]
    NotPermitted(String),
}

/// Canonicalize `path` and accept it only if it resolves strictly under
/// one of `roots`.
///
/// Returns the canonical, contained path on success. The shell uses the
/// returned path for the subsequent IO so the check and the IO operate
/// on the **same** resolved path (no check-vs-use divergence at the
/// string level; the residual OS-level race is not held across
/// check→use in v0.1, matching the built-in-tool resolver).
///
/// # Errors
///
/// [`ConfineError::NotPermitted`] when `path` resolves outside every
/// root, when `roots` is empty, or when a non-existing save remainder
/// contains a `..` component.
pub fn confine(path: &Path, roots: &[PathBuf]) -> Result<PathBuf, ConfineError> {
    let resolved = resolve(path)?;
    for root in roots {
        // Canonicalize each root so the comparison is canonical-vs-
        // canonical (Windows `\\?\` verbatim prefixes and separators
        // never poison a raw prefix match). A root that does not
        // canonicalize (deleted out from under us) simply cannot
        // contain anything and is skipped.
        if let Ok(canonical_root) = std::fs::canonicalize(root) {
            if resolved.starts_with(&canonical_root) {
                return Ok(resolved);
            }
        }
    }
    Err(ConfineError::NotPermitted(resolved.display().to_string()))
}

/// Resolve `path` to a canonical absolute path.
///
/// If `path` exists, canonicalize it directly. Otherwise (the save
/// case), walk up to the nearest existing ancestor, canonicalize that,
/// and re-join the non-existing remainder — rejecting any `..` in the
/// remainder, which the filesystem cannot normalize away.
fn resolve(path: &Path) -> Result<PathBuf, ConfineError> {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return Ok(canonical);
    }

    // Find the nearest existing ancestor.
    let mut existing = path;
    let mut remainder: Vec<Component> = Vec::new();
    loop {
        match existing.parent() {
            Some(parent) => {
                // Collect the trailing component(s) as the remainder, in
                // root→leaf order.
                if let Some(last) = existing.file_name() {
                    remainder.push(Component::Normal(last));
                } else {
                    // A trailing `..` or `.` shows up here with no
                    // file_name — capture it so the `..` rejection below
                    // sees it.
                    if let Some(comp) = existing.components().next_back() {
                        remainder.push(comp);
                    }
                }
                existing = parent;
                if existing.exists() {
                    break;
                }
            }
            None => {
                // Reached the filesystem root without finding an existing
                // ancestor — nothing legitimate resolves here.
                return Err(ConfineError::NotPermitted(path.display().to_string()));
            }
        }
    }

    let canonical_ancestor = std::fs::canonicalize(existing)
        .map_err(|e| ConfineError::NotPermitted(format!("{}: {e}", path.display())))?;

    // Re-join the remainder (it was pushed leaf→root above; reverse to
    // root→leaf), rejecting any `..` that the missing-path walk could
    // not resolve.
    let mut out = canonical_ancestor;
    for comp in remainder.into_iter().rev() {
        match comp {
            Component::Normal(name) => out.push(name),
            Component::ParentDir => {
                return Err(ConfineError::NotPermitted(format!(
                    "{}: contains an unresolved parent (`..`) segment",
                    path.display()
                )));
            }
            // `.` is a no-op; a CurDir / Prefix / RootDir in the
            // remainder is not meaningful for a relative leaf — skip.
            Component::CurDir | Component::Prefix(_) | Component::RootDir => {}
        }
    }
    Ok(out)
}
