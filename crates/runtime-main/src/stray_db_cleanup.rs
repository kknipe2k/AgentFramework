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
use std::time::{SystemTime, UNIX_EPOCH};

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
    let stray = dir.join("mcp.sqlite");
    if !stray.exists() {
        return Ok(CleanupOutcome::NotPresent);
    }
    let target = pick_rename_target(dir);
    std::fs::rename(&stray, &target)?;
    tracing::info!(
        original = %stray.display(),
        renamed_to = %target.display(),
        "M08.5.5: pre-ADR-0012 stray mcp.sqlite renamed; bytes preserved for forensic recovery"
    );
    Ok(CleanupOutcome::Cleaned {
        original: stray,
        renamed_to: target,
    })
}

/// Pick the rename target inside `dir`. Defaults to
/// `.stray-mcp.sqlite.bak`; if that already exists (a prior cleanup
/// pass already produced one), falls back to
/// `.stray-mcp.sqlite.bak.<unix-ms>` so the rename never clobbers an
/// existing backup. Pre-1970 system clocks fall back to a `0` suffix —
/// same posture as `tier::persistence::now_unix_ms`.
fn pick_rename_target(dir: &Path) -> PathBuf {
    let canonical = dir.join(".stray-mcp.sqlite.bak");
    if !canonical.exists() {
        return canonical;
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
    dir.join(format!(".stray-mcp.sqlite.bak.{ts}"))
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
