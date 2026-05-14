//! Audit log path resolution — spec §8.security L5 (M05 Stage E).
//!
//! Centralizes the file-name constant + the path-join helper. The Tauri
//! shell resolves the data-directory side via
//! `AppHandle::path().app_local_data_dir()`; this module owns the
//! file-name half so callers stay consistent (no hand-typed
//! `"skills.audit.jsonl"` literals scattered through the codebase).

use std::path::{Path, PathBuf};

/// File name for the audit log. Stored under the app's local data
/// directory; the platform-appropriate parent path is resolved by the
/// Tauri shell layer at app startup.
pub const AUDIT_FILE_NAME: &str = "skills.audit.jsonl";

/// Compose the audit log path from a data-directory base. Per the
/// M05.D `tier::persistence` archetype, the module stays path-agnostic
/// — callers pass the directory, this module owns the file-name.
#[must_use]
pub fn audit_path(dir: &Path) -> PathBuf {
    dir.join(AUDIT_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_file_name_is_skills_audit_jsonl() {
        // Lock the file-name constant — the spec §8.security L5 wire
        // contract names `skills.audit.jsonl` literally, and renamers
        // would break human grep workflows + any external consumer that
        // streams the file by name.
        assert_eq!(AUDIT_FILE_NAME, "skills.audit.jsonl");
    }

    #[test]
    fn audit_path_joins_file_name_to_dir() {
        let dir = Path::new("/var/lib/agent-runtime");
        let path = audit_path(dir);
        assert!(path.ends_with("skills.audit.jsonl"));
        assert_eq!(path.parent().unwrap(), dir);
    }

    #[test]
    fn audit_path_round_trips_through_tempdir() {
        // Cross-platform sanity: tempfile's TempDir path joins the
        // file-name correctly on Windows + Unix + macOS.
        let dir = tempfile::tempdir().unwrap();
        let path = audit_path(dir.path());
        assert_eq!(path.file_name().unwrap(), AUDIT_FILE_NAME);
        assert_eq!(path.parent().unwrap(), dir.path());
    }
}
