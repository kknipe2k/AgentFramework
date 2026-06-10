//! M09.5.A (TD-051 / review C2) — unit contract for the pure path-
//! containment helper `runtime_main::path_confine::confine`.
//!
//! The helper is the engine half of the shell perimeter fix: it
//! canonicalizes a renderer-supplied path (for save targets that do not
//! exist yet: canonicalize the nearest existing ancestor and re-join the
//! remainder, rejecting residual `..` components) and accepts the result
//! only when it is strictly contained under one of the allow-listed
//! roots. The shell (`save_framework` / `load_framework` /
//! `import_artifact(file)`) confines against
//! `{dialog-registered dirs} ∪ {app_local_data_dir}` and refuses with a
//! typed `path_not_permitted` error otherwise.
//!
//! Path-agnostic persistence archetype (docs/style.md / CLAUDE.md §9):
//! the helper takes `&Path` + `&[PathBuf]`, no Tauri dependency,
//! `tempfile`-tested here. The adversarial cases ARE the acceptance
//! (phase doc A.4): escapes must be refused; harmless normalization must
//! keep working.

use std::fs;
use std::path::PathBuf;

use runtime_main::path_confine::{confine, ConfineError};
use tempfile::tempdir;

#[test]
fn confine_allows_existing_path_inside_root() {
    let root = tempdir().expect("tempdir");
    let inside = root.path().join("frameworks").join("my-fw");
    fs::create_dir_all(&inside).expect("create inside dir");

    let roots = vec![root.path().to_path_buf()];
    let confined = confine(&inside, &roots).expect("in-root path must be allowed");

    let canonical_root = fs::canonicalize(root.path()).expect("canonicalize root");
    assert!(
        confined.starts_with(&canonical_root),
        "confined path {} must stay under canonical root {}",
        confined.display(),
        canonical_root.display()
    );
}

#[test]
fn confine_allows_nonexistent_target_dir_under_root() {
    // The save case: `save_framework` targets a directory that does not
    // exist yet (it create_dir_all's it). The nearest EXISTING ancestor
    // canonicalizes; the non-existing remainder re-joins.
    let root = tempdir().expect("tempdir");
    let target = root.path().join("not-yet").join("deeper");

    let roots = vec![root.path().to_path_buf()];
    let confined = confine(&target, &roots).expect("nonexistent target under root must be allowed");

    let canonical_root = fs::canonicalize(root.path()).expect("canonicalize root");
    assert!(
        confined.starts_with(&canonical_root),
        "confined save target {} must stay under canonical root {}",
        confined.display(),
        canonical_root.display()
    );
    assert!(
        confined.ends_with(PathBuf::from("not-yet").join("deeper")),
        "remainder must be re-joined onto the canonical ancestor, got {}",
        confined.display()
    );
}

#[test]
fn confine_rejects_dotdot_escape() {
    // The review's literal case: invoke("save_framework", {dir:"../../x"})
    // shaped as an absolute path whose `..` components resolve OUTSIDE
    // the root.
    let root = tempdir().expect("tempdir");
    let escape = root
        .path()
        .join("inner")
        .join("..")
        .join("..")
        .join("escaped");

    let roots = vec![root.path().to_path_buf()];
    let err = confine(&escape, &roots).expect_err(".. traversal escaping the root must be refused");
    assert!(
        matches!(err, ConfineError::NotPermitted(_)),
        "escape must surface as NotPermitted, got {err:?}"
    );
}

#[test]
fn confine_rejects_path_outside_all_roots() {
    let root = tempdir().expect("tempdir");
    let elsewhere = tempdir().expect("second tempdir");
    let outside = elsewhere.path().join("file.json");

    let roots = vec![root.path().to_path_buf()];
    let err = confine(&outside, &roots).expect_err("path outside every root must be refused");
    assert!(
        matches!(err, ConfineError::NotPermitted(_)),
        "outside-roots must surface as NotPermitted, got {err:?}"
    );
}

#[test]
fn confine_rejects_everything_when_roots_empty() {
    let somewhere = tempdir().expect("tempdir");
    let err =
        confine(somewhere.path(), &[]).expect_err("no registered roots means nothing is permitted");
    assert!(
        matches!(err, ConfineError::NotPermitted(_)),
        "empty roots must surface as NotPermitted, got {err:?}"
    );
}

#[test]
fn confine_allows_dotdot_that_resolves_inside() {
    // The fix narrows to ESCAPES, not to harmless normalization: a `..`
    // that still resolves under the root is allowed (phase doc A.4 /
    // the B.4 "harmless normalization" principle applied to the shell).
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("sub")).expect("create sub");
    let normalized_inside = root.path().join("sub").join("..").join("in-scope");

    let roots = vec![root.path().to_path_buf()];
    let confined =
        confine(&normalized_inside, &roots).expect("in-root .. normalization must be allowed");

    let canonical_root = fs::canonicalize(root.path()).expect("canonicalize root");
    assert_eq!(
        confined,
        canonical_root.join("in-scope"),
        "sub/../in-scope must normalize to <root>/in-scope"
    );
}

#[test]
fn confine_allows_under_any_of_multiple_roots() {
    // The shell confines against {registered dirs} ∪ {app_local_data_dir}
    // — containment under ANY root suffices.
    let first = tempdir().expect("tempdir");
    let second = tempdir().expect("second tempdir");
    let under_second = second.path().join("fw");
    fs::create_dir_all(&under_second).expect("create under second");

    let roots = vec![first.path().to_path_buf(), second.path().to_path_buf()];
    confine(&under_second, &roots).expect("path under the second root must be allowed");
}

#[test]
fn confine_rejects_residual_dotdot_in_nonexistent_remainder() {
    // Save-case hardening: when the target does not exist, the
    // non-existing remainder is re-joined WITHOUT filesystem resolution
    // — so a `..` hiding in that remainder cannot be canonicalized away
    // and must be rejected outright.
    let root = tempdir().expect("tempdir");
    let sneaky = root
        .path()
        .join("ghost-dir")
        .join("..")
        .join("..")
        .join("escaped");

    let roots = vec![root.path().to_path_buf()];
    let err = confine(&sneaky, &roots)
        .expect_err("residual .. in a nonexistent remainder must be refused");
    assert!(
        matches!(err, ConfineError::NotPermitted(_)),
        "residual-.. remainder must surface as NotPermitted, got {err:?}"
    );
}

#[test]
fn confine_skips_a_root_that_does_not_canonicalize() {
    // A registered root that no longer exists cannot contain anything —
    // the canonicalize(root) Err arm is skipped, and a real root after it
    // still decides containment. Exercises the skip branch.
    let missing = tempdir().expect("tempdir");
    let missing_root = missing.path().to_path_buf();
    drop(missing); // delete the dir so canonicalize(root) fails

    let real = tempdir().expect("real tempdir");
    let under_real = real.path().join("fw");
    fs::create_dir_all(&under_real).expect("create under real");

    // Order matters: the dead root is tried first and skipped, then the
    // real root matches.
    let roots = vec![missing_root.clone(), real.path().to_path_buf()];
    confine(&under_real, &roots).expect("a live root after a dead one must still permit");

    // And a path under neither (only the dead root would have covered it)
    // is refused.
    let orphan = missing_root.join("fw");
    let err = confine(&orphan, &roots).expect_err("a path only the dead root covered is refused");
    assert!(matches!(err, ConfineError::NotPermitted(_)));
}

#[test]
fn confine_allows_nonexistent_remainder_with_curdir_segment() {
    // A `.` segment in the nonexistent remainder is a no-op (the CurDir
    // match arm) and must not change containment.
    let root = tempdir().expect("tempdir");
    let with_curdir = root.path().join("ghost").join(".").join("leaf");

    let roots = vec![root.path().to_path_buf()];
    let confined =
        confine(&with_curdir, &roots).expect("a `.` segment under the root must be allowed");
    let canonical_root = fs::canonicalize(root.path()).expect("canonicalize root");
    assert!(confined.starts_with(&canonical_root));
    assert!(confined.ends_with(PathBuf::from("ghost").join("leaf")));
}

#[cfg(unix)]
#[test]
fn confine_rejects_symlink_escape_from_root() {
    // The symlink variant the review proved against the builtin tools
    // (TD-052), applied at the shell perimeter: a link INSIDE a root
    // pointing OUTSIDE it must be refused after resolution.
    use std::os::unix::fs::symlink;

    let root = tempdir().expect("tempdir");
    let outside = tempdir().expect("outside tempdir");
    let link = root.path().join("link");
    symlink(outside.path(), &link).expect("create symlink");

    let roots = vec![root.path().to_path_buf()];
    let err = confine(&link.join("secret.txt"), &roots)
        .expect_err("symlink escaping the root must be refused");
    assert!(
        matches!(err, ConfineError::NotPermitted(_)),
        "symlink escape must surface as NotPermitted, got {err:?}"
    );
}

#[cfg(unix)]
#[test]
fn confine_rejects_parentdir_surviving_into_a_nonexistent_remainder() {
    // The ParentDir-in-remainder rejection arm. On unix, a nonexistent
    // intermediate (`ghost`) means `ghost/../sibling` cannot be
    // canonicalized, and the `..` survives the nearest-existing-ancestor
    // walk into the re-joined remainder — where it is rejected outright
    // (the filesystem could not normalize it away). On Windows the OS
    // collapses `..` lexically before the resolver sees it, so this arm
    // is unix-shaped (it is the platform where the `..` actually survives).
    let root = tempdir().expect("tempdir");
    let surviving = root.path().join("ghost").join("..").join("sibling");

    let roots = vec![root.path().to_path_buf()];
    let err = confine(&surviving, &roots)
        .expect_err("a .. that survives into the remainder must be refused");
    assert!(
        matches!(err, ConfineError::NotPermitted(_)),
        "surviving .. must surface as NotPermitted, got {err:?}"
    );
}

#[cfg(unix)]
#[test]
fn confine_rejects_relative_path_with_no_existing_ancestor() {
    // The filesystem-root guard: a relative path whose ancestors never
    // exist walks up to an empty parent and then to `None` without ever
    // finding an existing ancestor — nothing legitimate resolves there,
    // so it is refused. (On unix `/` always exists, so an absolute path
    // never reaches this arm; a relative path with no real ancestor does.)
    let orphan = std::path::Path::new("nonexistent-rel-xyz-9f3a/deeper/leaf");

    // A real registered root exists, but the orphan resolves under none.
    let root = tempdir().expect("tempdir");
    let roots = vec![root.path().to_path_buf()];
    let err = confine(orphan, &roots)
        .expect_err("a relative path with no existing ancestor must be refused");
    assert!(
        matches!(err, ConfineError::NotPermitted(_)),
        "no-existing-ancestor must surface as NotPermitted, got {err:?}"
    );
}

#[cfg(windows)]
#[test]
fn confine_rejects_backslash_dotdot_escape() {
    // The Windows-native separator variant — v0.1 is Windows-first, so
    // the literal `..\..\` form is the production-surface traversal.
    let root = tempdir().expect("tempdir");
    let escape = PathBuf::from(format!("{}\\inner\\..\\..\\escaped", root.path().display()));

    let roots = vec![root.path().to_path_buf()];
    let err =
        confine(&escape, &roots).expect_err("backslash .. traversal must be refused on Windows");
    assert!(
        matches!(err, ConfineError::NotPermitted(_)),
        "backslash escape must surface as NotPermitted, got {err:?}"
    );
}
