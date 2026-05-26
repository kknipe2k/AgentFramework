//! Error surface for the Builder backend (M08 Stage B).
//!
//! [`validate_framework`] itself does **not** return `Result` —
//! validation *failures are the report*, not errors. `BuilderError`
//! covers only [`persist`] (filesystem) and [`validate::list_installed`]
//! (lock-corruption) operational failures. Mirrors the M07
//! import-pipeline error layering.
//!
//! [`validate_framework`]: crate::builder::validate::validate_framework
//! [`persist`]: crate::builder::persist
//! [`validate::list_installed`]: crate::builder::validate::list_installed

use thiserror::Error;

use crate::framework_loader::FrameworkLoadError;
use crate::skills_lock::LockError;

/// Failure modes raised by the `builder` module.
#[derive(Debug, Error)]
pub enum BuilderError {
    /// Filesystem error writing/reading a framework directory.
    #[error("builder I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// `framework.json` serialization/deserialization failed.
    #[error("framework JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// The framework loader rejected a load (gaps / parse). Carried for
    /// the Builder error surface the Stage F1 Tester artifact-load path
    /// composes onto; `builder` itself parses with gap-tolerance.
    #[error(transparent)]
    Load(#[from] FrameworkLoadError),
    /// The `skills.lock` reader rejected the lock (corrupt / parse). An
    /// *absent* lock is not an error — [`list_installed`] maps that to
    /// an empty list; only a present-but-corrupt lock surfaces here.
    ///
    /// [`list_installed`]: crate::builder::validate::list_installed
    #[error(transparent)]
    Lock(#[from] LockError),
    /// [`save_framework`] was given a path that exists and is not a
    /// directory.
    ///
    /// [`save_framework`]: crate::builder::persist::save_framework
    #[error("save target is not a directory: {0}")]
    NotADirectory(String),
    /// A `framework.json` `{id,path}` agents[] reference (or a
    /// path-referenced `tools[]` / `skills[]` entry) could not be
    /// resolved — the target `.md` is missing, unreadable, or its YAML
    /// frontmatter does not parse. Per ADR-0022 a broken reference is an
    /// error (distinct from a partially-built inline framework's
    /// unfilled-field gap, which `validate_framework` reports as a red
    /// badge); the Builder surfaces this so the user can fix the
    /// reference.
    #[error("could not resolve framework reference {reference}: {cause}")]
    ReferenceResolution {
        /// The relative `{id,path}` (or tools[]/skills[] `path`) value
        /// from `framework.json` that could not be resolved.
        reference: String,
        /// Human-readable cause (missing file, IO error, parse error).
        cause: String,
    },
}
