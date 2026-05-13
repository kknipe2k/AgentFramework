//! Tier persistence — spec §8.security L4 + §14 first-run UX (M05 Stage D).
//!
//! Stores the user's current tier in `<dir>/tier.json` where `<dir>`
//! is supplied by the caller. The Tauri layer resolves the path via
//! `AppHandle::path().app_local_data_dir()` (Windows: `%APPDATA%\<id>\`;
//! Linux: `$XDG_DATA_HOME/<id>/` or `~/.local/share/<id>/`); tests pass
//! a `tempfile::TempDir` path. Keeping the module path-agnostic avoids a
//! new workspace dep and keeps the unit-test surface clean.
//!
//! First-run default is [`Tier::Novice`] — `tier.json` absent →
//! `Tier::default()`. Default-safe matches §8.security spirit.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::tier::error::TierPersistenceError;
use crate::tier::evaluator::Tier;

/// On-disk shape of `tier.json`. The `since_unix_ms` field records when
/// the tier was set; surfaced in the audit log + the renderer's Settings
/// panel ("Promoted since ...") in M10.
#[derive(Debug, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
struct StoredTier {
    tier: Tier,
    since_unix_ms: u64,
}

/// Read the persisted tier from `<dir>/tier.json`.
///
/// Returns [`Tier::Novice`] (the [`Tier::default`]) when the file is
/// absent — first-run default per §14 + phase-doc gotcha "Default-safe
/// matches §8.security spirit".
///
/// # Errors
///
/// - [`TierPersistenceError::Io`] if the file exists but cannot be
///   read.
/// - [`TierPersistenceError::Json`] if the file exists but contains
///   malformed JSON.
pub fn load_tier(dir: &Path) -> Result<Tier, TierPersistenceError> {
    let path = dir.join("tier.json");
    if !path.exists() {
        return Ok(Tier::default());
    }
    let raw = fs::read_to_string(&path)?;
    let stored: StoredTier = serde_json::from_str(&raw)?;
    Ok(stored.tier)
}

/// Write the current tier to `<dir>/tier.json`. Creates `<dir>` if it
/// doesn't exist (mirrors the `resolve_db_path` pattern in
/// `src-tauri/src/main.rs`).
///
/// `since_unix_ms` is captured from the system clock at write time; a
/// clock that fails to read post-1970 falls back to 0 (rare; would
/// indicate a system clock pre-epoch — surface-level renderer copy
/// degrades gracefully via the "first run" framing).
///
/// # Errors
///
/// - [`TierPersistenceError::Io`] on directory creation or file write
///   failure.
/// - [`TierPersistenceError::Json`] on serialization failure (rare;
///   would indicate the type derivation diverged).
pub fn save_tier(dir: &Path, tier: Tier) -> Result<(), TierPersistenceError> {
    fs::create_dir_all(dir)?;
    let stored = StoredTier {
        tier,
        since_unix_ms: now_unix_ms(),
    };
    let json = serde_json::to_string_pretty(&stored)?;
    fs::write(dir.join("tier.json"), json)?;
    Ok(())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_returns_novice_on_first_run() {
        let dir = tempdir().unwrap();
        let tier = load_tier(dir.path()).unwrap();
        assert_eq!(tier, Tier::Novice);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempdir().unwrap();
        save_tier(dir.path(), Tier::Promoted).unwrap();
        let loaded = load_tier(dir.path()).unwrap();
        assert_eq!(loaded, Tier::Promoted);
    }

    #[test]
    fn save_and_load_twice_in_sequence() {
        // Gotcha #69: multi-call invariant. Two sequential save/load
        // cycles must both succeed with their respective values. Catches
        // any per-call mutation bug in the path-derivation or
        // serialization layer.
        let dir = tempdir().unwrap();
        save_tier(dir.path(), Tier::Promoted).unwrap();
        assert_eq!(load_tier(dir.path()).unwrap(), Tier::Promoted);
        save_tier(dir.path(), Tier::Novice).unwrap();
        assert_eq!(load_tier(dir.path()).unwrap(), Tier::Novice);
        save_tier(dir.path(), Tier::Promoted).unwrap();
        assert_eq!(load_tier(dir.path()).unwrap(), Tier::Promoted);
    }

    #[test]
    fn save_creates_parent_dir_when_missing() {
        let parent = tempdir().unwrap();
        let nested = parent.path().join("agent-runtime").join("nested");
        assert!(!nested.exists());
        save_tier(&nested, Tier::Promoted).unwrap();
        assert!(nested.join("tier.json").exists());
        assert_eq!(load_tier(&nested).unwrap(), Tier::Promoted);
    }

    #[test]
    fn load_returns_io_error_on_unreadable_file() {
        // Simulate a malformed file — the load path must surface a
        // structured error rather than panic.
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("tier.json"), "not json at all").unwrap();
        let err = load_tier(dir.path()).expect_err("malformed JSON must err");
        assert!(matches!(err, TierPersistenceError::Json(_)));
    }

    #[test]
    fn saved_file_contains_since_unix_ms_field() {
        // The renderer's Settings panel surfaces "Promoted since ..." —
        // the field must be present in the on-disk shape.
        let dir = tempdir().unwrap();
        save_tier(dir.path(), Tier::Promoted).unwrap();
        let raw = fs::read_to_string(dir.path().join("tier.json")).unwrap();
        assert!(raw.contains("since_unix_ms"));
        assert!(raw.contains("\"promoted\""));
    }

    #[test]
    fn now_unix_ms_returns_post_epoch() {
        // Sanity check on the helper — should be > 1.7e12 in 2026.
        let now = now_unix_ms();
        assert!(
            now > 1_700_000_000_000,
            "clock returned pre-2024 value: {now}"
        );
    }
}
