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
}
