//! M08.5.5 Stage C.fix — startup cleanup of the pre-ADR-0012 stray
//! `mcp.sqlite` file. ADR-0012 (M06.5 Stage A.fix) prevented future
//! creation; this module handles the cleanup of legacy files left on
//! users' machines from pre-fix testing.
//!
//! Path-agnostic per CLAUDE.md §9: accepts `dir: &Path`; the Tauri
//! shell resolves `app_local_data_dir()`. Tested with `tempfile`-
//! backed paths.
//!
//! Forensic recoverability: the original bytes are preserved by
//! renaming the file (NOT deleting it). The renamed file
//! (`.stray-mcp.sqlite.bak`) is fully recoverable via
//! `sqlite3 .stray-mcp.sqlite.bak ".dump"`. If `.stray-mcp.sqlite.bak`
//! already exists from a prior cleanup pass, the rename target gets a
//! millisecond-timestamp suffix to preserve both backups (idempotent
//! across repeated launches).
//!
//! Scope deviation from the M08.5.5 phase doc C.3.1: this module does
//! NOT integrate with the existing `AuditWriter`. Reason: the
//! schema-generated `AuditEntryKind` enum has no `StrayDbCleaned`
//! variant; adding one is a schema change + ADR (out of stage scope).
//! The `.bak` file IS the forensic record (full byte preservation);
//! `tracing::info!` provides operator-visible cleanup notification.
//! See the maintainer's design decision on the C.fix red-phase
//! approval surface.

use std::path::{Path, PathBuf};

/// Outcome of one startup cleanup pass.
#[derive(Debug, PartialEq, Eq)]
pub enum CleanupOutcome {
    /// No legacy `mcp.sqlite` file present; no action taken.
    NotPresent,
    /// Legacy file detected + renamed; original bytes preserved at
    /// `renamed_to`.
    Cleaned {
        /// Path the stray file used to live at (always `dir/mcp.sqlite`).
        original: PathBuf,
        /// Path the file was renamed to (`.stray-mcp.sqlite.bak` or
        /// `.stray-mcp.sqlite.bak.<unix-ms>` if the canonical target
        /// was occupied).
        renamed_to: PathBuf,
    },
}

/// Errors raised by [`cleanup_stray_db`].
#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    /// I/O error during the rename (target dir gone, permission denied,
    /// target volume is a different filesystem on Windows).
    #[error("rename of stray mcp.sqlite failed: {0}")]
    Rename(#[from] std::io::Error),
}

/// Detect + rename the pre-ADR-0012 stray `mcp.sqlite` in `dir`.
///
/// If `dir/mcp.sqlite` exists, rename it to `dir/.stray-mcp.sqlite.bak`
/// (preserving all original bytes for forensic recovery). If the
/// `.stray-mcp.sqlite.bak` target already exists from a prior cleanup
/// pass, the new target gets a millisecond-timestamp suffix
/// (`.stray-mcp.sqlite.bak.<unix-ms>`).
///
/// Returns [`CleanupOutcome::NotPresent`] when there's no stray file;
/// returns [`CleanupOutcome::Cleaned`] with both paths when the rename
/// completes.
///
/// Idempotent: running twice in a row produces `NotPresent` the second
/// time (the first run renames the stray; the second sees no
/// `mcp.sqlite` and exits).
///
/// # Errors
///
/// Returns [`CleanupError::Rename`] when the `std::fs::rename` call
/// fails (target directory missing, permission denied, target volume
/// is a different filesystem on Windows). The Tauri shell logs at
/// WARN and continues startup — a stray file is a recoverable state.
pub fn cleanup_stray_db(dir: &Path) -> Result<CleanupOutcome, CleanupError> {
    // STUB (red phase): probes the stray path but always returns
    // NotPresent so the behavioral tests (2, 3, 4) fail right-reason
    // when the stray IS present (assertions on Cleaned-outcome shape
    // + .bak existence). Replaced by the impl commit per the strict-
    // TDD v1.8 invariant. The `dir.join` is intentional — keeps the
    // function non-const so the impl doesn't have to remove a
    // `const` qualifier added by clippy::missing_const_for_fn.
    let _stray_path = dir.join("mcp.sqlite");
    Ok(CleanupOutcome::NotPresent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn cleanup_returns_not_present_when_stray_absent() {
        let dir = tempdir().expect("tempdir");
        let outcome = cleanup_stray_db(dir.path()).expect("cleanup");
        assert_eq!(outcome, CleanupOutcome::NotPresent);
        assert!(
            !dir.path().join(".stray-mcp.sqlite.bak").exists(),
            "no .bak file should be created when no stray was present"
        );
    }

    #[test]
    fn cleanup_renames_stray_and_preserves_bytes_when_present() {
        let dir = tempdir().expect("tempdir");
        let stray = dir.path().join("mcp.sqlite");
        // The cleanup never opens the SQLite content — it just renames.
        // Use a recognizable marker for the byte-preservation assertion.
        fs::write(&stray, b"legacy-mcp-sqlite-bytes\x00\x01\x02").expect("seed stray");

        let outcome = cleanup_stray_db(dir.path()).expect("cleanup");

        let expected_target = dir.path().join(".stray-mcp.sqlite.bak");
        assert_eq!(
            outcome,
            CleanupOutcome::Cleaned {
                original: stray.clone(),
                renamed_to: expected_target.clone(),
            },
        );
        assert!(
            !stray.exists(),
            "original mcp.sqlite must be gone after rename"
        );
        assert!(expected_target.exists(), "rename target must exist");
        let preserved = fs::read(&expected_target).expect("read bak");
        assert_eq!(
            preserved, b"legacy-mcp-sqlite-bytes\x00\x01\x02",
            "rename must preserve original bytes verbatim"
        );
    }

    #[test]
    fn cleanup_picks_timestamp_suffix_when_bak_already_exists() {
        let dir = tempdir().expect("tempdir");
        let stray = dir.path().join("mcp.sqlite");
        let existing_bak = dir.path().join(".stray-mcp.sqlite.bak");
        fs::write(&stray, b"new-stray-bytes").expect("seed stray");
        fs::write(&existing_bak, b"prior-bak-bytes").expect("seed prior bak");

        let outcome = cleanup_stray_db(dir.path()).expect("cleanup");

        match outcome {
            CleanupOutcome::Cleaned {
                original,
                renamed_to,
            } => {
                assert_eq!(original, stray);
                assert_ne!(
                    renamed_to, existing_bak,
                    "second cleanup must NOT clobber the existing .bak"
                );
                assert!(renamed_to.exists(), "timestamped target must exist");
                let name = renamed_to
                    .file_name()
                    .expect("filename")
                    .to_string_lossy()
                    .to_string();
                assert!(
                    name.starts_with(".stray-mcp.sqlite.bak."),
                    "timestamped suffix should extend the canonical .bak name: got {name}"
                );
                let suffix = name.trim_start_matches(".stray-mcp.sqlite.bak.");
                assert!(
                    suffix.chars().all(|c| c.is_ascii_digit()),
                    "suffix must be the unix-ms timestamp digits: got {suffix}"
                );
            }
            CleanupOutcome::NotPresent => {
                panic!("expected Cleaned outcome but got NotPresent — cleanup didn't fire");
            }
        }
        // The pre-existing .bak must be preserved untouched.
        let prior = fs::read(&existing_bak).expect("read prior bak");
        assert_eq!(
            prior, b"prior-bak-bytes",
            "existing .bak must be preserved untouched"
        );
    }
}
