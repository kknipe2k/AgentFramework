//! M08.5.5 Stage C.fix — assembled regression for the stray-DB cleanup
//! module. Exercises the public API end-to-end with a `TempDir`-backed
//! path; mirrors the composition the Tauri shell uses in `setup()`.
//!
//! This test fails right-reason on pre-fix (the red-commit stub returns
//! `NotPresent` unconditionally, so the assertion that the stray was
//! renamed + the canonical session.sqlite was untouched panics on the
//! `match` arm). Passes on the impl-commit module.

use runtime_main::stray_db_cleanup::{cleanup_stray_db, CleanupOutcome};
use std::fs;
use tempfile::tempdir;

#[test]
fn cleanup_runs_at_setup_before_canonical_db_opens() {
    let dir = tempdir().expect("tempdir");
    let stray = dir.path().join("mcp.sqlite");
    let session = dir.path().join("session.sqlite");
    fs::write(&stray, b"legacy-pre-adr-0012-bytes").expect("seed stray");
    fs::write(&session, b"canonical-session-bytes").expect("seed session");

    let outcome = cleanup_stray_db(dir.path()).expect("cleanup");

    match outcome {
        CleanupOutcome::Cleaned {
            original,
            renamed_to,
        } => {
            assert_eq!(original, stray);
            assert_eq!(renamed_to, dir.path().join(".stray-mcp.sqlite.bak"));
            assert!(!stray.exists(), "stray mcp.sqlite must be renamed away");
            assert!(renamed_to.exists(), ".stray-mcp.sqlite.bak must exist");
        }
        CleanupOutcome::NotPresent => {
            panic!("expected Cleaned outcome but got NotPresent — cleanup didn't fire");
        }
    }

    // The canonical session.sqlite must be untouched — the cleanup acts
    // only on the stray mcp.sqlite. This pins the ordering invariant:
    // even if a session.sqlite pre-exists in the same directory, the
    // cleanup must not modify it.
    assert!(
        session.exists(),
        "session.sqlite must be untouched by stray cleanup"
    );
    let canonical_bytes = fs::read(&session).expect("read session");
    assert_eq!(
        canonical_bytes, b"canonical-session-bytes",
        "session.sqlite bytes must be unchanged"
    );
}
