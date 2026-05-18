//! M07 Stage B — `skills.lock` integrity primitive (spec §2181–2216).
//!
//! Behavioral contract tests for the path-agnostic `skills_lock` module:
//! SRI-encoded SHA-256 content hashing, read/write of the framework-root
//! lock file, verify-on-load (happy + tamper + unknown-artifact), the
//! byte-identical canonical-serialization reproducibility invariant
//! (spec §2204/§2216 — the lock is checked into the user's framework
//! repo, so two installs of the same set must produce an identical
//! file), and the schema-faithful `artifact_hash_mismatch` event the
//! load path emits on drift.
//!
//! Generated types are constructed via `serde_json` from schema-shaped
//! JSON literals rather than typify struct literals so the contract
//! pins the *wire shape* (the schema is the source of truth) and not
//! typify's incidental Rust field/builder naming.
//!
//! Strict-TDD (CLAUDE.md §6, v1.8 two-commit): every test here lands in
//! the red commit; the impl commit touches zero `**/tests/**` files.

use runtime_core::event::AgentEvent;
use runtime_core::generated::skills_lock::{LockEntry, SkillsLock};
use runtime_main::skills_lock::{self, LockError};
use serde_json::json;

/// SHA-256("") known vector → SRI base64. Locks cross-platform
/// determinism: the digest of the empty input is a fixed RFC-test
/// constant, and the SRI encoding (standard base64, `sha256-` prefix)
/// must be byte-identical on Windows / Linux / macOS.
const EMPTY_SRI: &str = "sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=";

fn lock_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("skills.lock")
}

fn url_entry(content_hash: &str) -> LockEntry {
    serde_json::from_value(json!({
        "kind": "skill",
        "source": { "type": "url", "url": "https://example.com/a.md" },
        "content_hash": content_hash,
        "installed_at": "2026-05-18T14:23:00Z",
        "tier_at_install": "promoted",
        "validation_report_id": "vr-789xyz"
    }))
    .expect("schema-shaped url LockEntry deserializes")
}

fn file_entry(content_hash: &str) -> LockEntry {
    serde_json::from_value(json!({
        "kind": "tool",
        "source": { "type": "file", "path": "./tools/b.md" },
        "content_hash": content_hash,
        "installed_at": "2026-05-18T15:00:00Z",
        "tier_at_install": "novice",
        "validation_report_id": "vr-000aaa"
    }))
    .expect("schema-shaped file LockEntry deserializes")
}

// ── content_hash: SRI sha256-<base64>, deterministic, cross-platform ──

#[test]
fn content_hash_matches_known_sri_vector_for_empty_input() {
    // Cross-machine reproducibility (spec §2204): the digest of a fixed
    // input is a fixed string. If this drifts, every lock written on a
    // different machine would mismatch on load.
    assert_eq!(skills_lock::content_hash(b""), EMPTY_SRI);
}

#[test]
fn content_hash_is_deterministic_and_input_sensitive() {
    let a1 = skills_lock::content_hash(b"artifact-bytes-A");
    let a2 = skills_lock::content_hash(b"artifact-bytes-A");
    let b = skills_lock::content_hash(b"artifact-bytes-B");
    assert_eq!(a1, a2, "same bytes must hash identically (determinism)");
    assert_ne!(a1, b, "different bytes must hash differently");
}

#[test]
fn content_hash_is_sri_prefixed_standard_base64() {
    let h = skills_lock::content_hash(b"anything");
    let rest = h
        .strip_prefix("sha256-")
        .expect("hash must carry the swappable `sha256-` SRI algorithm prefix");
    // Schema pattern: ^sha256-[A-Za-z0-9+/]+={0,2}$ — standard base64
    // (not URL-safe) of a 32-byte digest is 43 chars + one '=' pad.
    assert!(
        rest.len() == 44 && rest.ends_with('='),
        "expected 44-char padded standard-base64 SHA-256 body, got {rest:?}"
    );
    assert!(
        rest.chars().all(|c| c.is_ascii_alphanumeric()
            || c == '+'
            || c == '/'
            || c == '='),
        "SRI body must be standard base64 alphabet, got {rest:?}"
    );
}

// ── read / write_entry: path-agnostic round-trip + multi-call ──

#[test]
fn write_entry_creates_lock_when_absent_and_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = lock_path(&dir);
    assert!(!path.exists(), "precondition: no lock file yet");

    let h = skills_lock::content_hash(b"A");
    skills_lock::write_entry(&path, "alpha@1.0.0", url_entry(&h)).expect("first write creates");
    assert!(path.exists(), "write_entry must create the lock file");

    let lock: SkillsLock = skills_lock::read(&path).expect("read back");
    let value = serde_json::to_value(&lock).expect("re-serialize");
    assert_eq!(value["version"], json!(1), "schema-faithful version field");
    assert_eq!(
        value["installed"]["alpha@1.0.0"]["content_hash"],
        json!(h),
        "round-tripped entry preserves the SRI content_hash"
    );
    assert_eq!(value["installed"]["alpha@1.0.0"]["kind"], json!("skill"));
}

#[test]
fn write_entry_appends_second_entry_preserving_first() {
    // Multi-call invariant (gotcha #69): a second install must not clobber
    // the first — the lock accumulates entries across installs.
    let dir = tempfile::tempdir().unwrap();
    let path = lock_path(&dir);
    skills_lock::write_entry(&path, "alpha@1.0.0", url_entry(&skills_lock::content_hash(b"A")))
        .expect("write #1");
    skills_lock::write_entry(&path, "beta@2.0.0", file_entry(&skills_lock::content_hash(b"B")))
        .expect("write #2");

    let lock: SkillsLock = skills_lock::read(&path).expect("read");
    let v = serde_json::to_value(&lock).unwrap();
    assert!(v["installed"]["alpha@1.0.0"].is_object(), "first entry survived");
    assert!(v["installed"]["beta@2.0.0"].is_object(), "second entry present");
}

#[test]
fn write_entry_replaces_same_key_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let path = lock_path(&dir);
    skills_lock::write_entry(&path, "alpha@1.0.0", url_entry(&skills_lock::content_hash(b"old")))
        .expect("write old");
    let new_hash = skills_lock::content_hash(b"new");
    skills_lock::write_entry(&path, "alpha@1.0.0", url_entry(&new_hash)).expect("rewrite");

    let lock = skills_lock::read(&path).expect("read");
    let v = serde_json::to_value(&lock).unwrap();
    assert_eq!(v["installed"]["alpha@1.0.0"]["content_hash"], json!(new_hash));
    assert_eq!(
        v["installed"].as_object().unwrap().len(),
        1,
        "re-install of the same key replaces, not duplicates"
    );
}

#[test]
fn read_missing_file_returns_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let err = skills_lock::read(&lock_path(&dir)).expect_err("missing lock must err");
    assert!(matches!(err, LockError::Io(_)), "got {err:?}");
}

#[test]
fn read_malformed_json_returns_parse_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = lock_path(&dir);
    std::fs::write(&path, "{ this is not json").unwrap();
    let err = skills_lock::read(&path).expect_err("malformed lock must err");
    assert!(matches!(err, LockError::Parse(_)), "got {err:?}");
}

#[test]
fn write_entry_into_missing_parent_dir_returns_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nope").join("skills.lock");
    let err = skills_lock::write_entry(&path, "x@1.0.0", url_entry(EMPTY_SRI))
        .expect_err("missing parent dir must err");
    assert!(matches!(err, LockError::Io(_)), "got {err:?}");
}

// ── verify-on-load: happy / tamper / unknown ──

#[test]
fn verify_happy_path_is_ok() {
    let dir = tempfile::tempdir().unwrap();
    let path = lock_path(&dir);
    let bytes = b"the real artifact bytes";
    skills_lock::write_entry(&path, "pkg@1.0.0", url_entry(&skills_lock::content_hash(bytes)))
        .expect("write");
    skills_lock::verify(&path, "pkg@1.0.0", bytes).expect("matching bytes verify Ok");
}

#[test]
fn verify_tamper_returns_hash_mismatch_carrying_sri_expected_and_actual() {
    let dir = tempfile::tempdir().unwrap();
    let path = lock_path(&dir);
    let good = b"the real artifact bytes";
    let expected = skills_lock::content_hash(good);
    skills_lock::write_entry(&path, "pkg@1.0.0", url_entry(&expected)).expect("write");

    let tampered = b"the TAMPERED artifact bytes";
    let actual = skills_lock::content_hash(tampered);
    let err = skills_lock::verify(&path, "pkg@1.0.0", tampered).expect_err("tamper must block");

    match err {
        LockError::HashMismatch {
            artifact_ref,
            expected: e,
            actual: a,
        } => {
            assert_eq!(artifact_ref, "pkg@1.0.0");
            assert_eq!(e, expected, "expected = the locked SRI hash");
            assert_eq!(a, actual, "actual = the recomputed SRI hash of the drifted bytes");
            // The load path maps this 1:1 onto the schema-faithful
            // blocking event (spec §2214). Construct + assert wire shape.
            let evt = AgentEvent::ArtifactHashMismatch {
                artifact_ref: artifact_ref.clone(),
                expected: e.clone(),
                actual: a.clone(),
            };
            let jv = serde_json::to_value(&evt).expect("event serializes");
            assert_eq!(jv["type"], json!("artifact_hash_mismatch"));
            assert_eq!(jv["artifact_ref"], json!("pkg@1.0.0"));
            assert_eq!(jv["expected"], json!(expected));
            assert_eq!(jv["actual"], json!(actual));
        }
        other => panic!("expected HashMismatch, got {other:?}"),
    }
}

#[test]
fn verify_unknown_artifact_returns_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let path = lock_path(&dir);
    skills_lock::write_entry(&path, "known@1.0.0", url_entry(EMPTY_SRI)).expect("write");
    let err =
        skills_lock::verify(&path, "ghost@9.9.9", b"x").expect_err("unknown ref must not silently pass");
    assert!(matches!(err, LockError::NotFound(ref r) if r == "ghost@9.9.9"), "got {err:?}");
}

#[test]
fn verify_propagates_read_error_when_lock_absent() {
    let dir = tempfile::tempdir().unwrap();
    let err = skills_lock::verify(&lock_path(&dir), "pkg@1.0.0", b"x")
        .expect_err("verify against a missing lock must err, not pass");
    assert!(matches!(err, LockError::Io(_)), "got {err:?}");
}

// ── reproducibility invariant (spec §2204 / §2216) ──

#[test]
fn canonical_serialization_is_byte_identical_regardless_of_install_order() {
    // Two machines install the same artifact set in different orders.
    // The lock is checked into VCS (spec §2216) — the on-disk bytes MUST
    // be identical or every cross-machine diff is noise / a false drift.
    let d1 = tempfile::tempdir().unwrap();
    let p1 = lock_path(&d1);
    skills_lock::write_entry(&p1, "zeta@9.0.0", file_entry(&skills_lock::content_hash(b"Z")))
        .unwrap();
    skills_lock::write_entry(&p1, "alpha@1.0.0", url_entry(&skills_lock::content_hash(b"A")))
        .unwrap();

    let d2 = tempfile::tempdir().unwrap();
    let p2 = lock_path(&d2);
    skills_lock::write_entry(&p2, "alpha@1.0.0", url_entry(&skills_lock::content_hash(b"A")))
        .unwrap();
    skills_lock::write_entry(&p2, "zeta@9.0.0", file_entry(&skills_lock::content_hash(b"Z")))
        .unwrap();

    let b1 = std::fs::read(&p1).unwrap();
    let b2 = std::fs::read(&p2).unwrap();
    assert_eq!(
        b1, b2,
        "lock bytes must be byte-identical regardless of install order"
    );
    // And keys must be sorted (mergeable-lockfile best practice — git
    // auto-resolves concurrent adds rather than conflicting).
    let text = String::from_utf8(b1).unwrap();
    assert!(
        text.find("alpha@1.0.0").unwrap() < text.find("zeta@9.0.0").unwrap(),
        "entries must be alphabetically ordered by name@version"
    );
}

#[test]
fn canonical_round_trip_parses_back_equal() {
    let dir = tempfile::tempdir().unwrap();
    let path = lock_path(&dir);
    skills_lock::write_entry(&path, "alpha@1.0.0", url_entry(&skills_lock::content_hash(b"A")))
        .unwrap();
    let first = std::fs::read(&path).unwrap();
    let parsed = skills_lock::read(&path).expect("read");
    // Re-write the parsed value through write_entry's canonical path and
    // confirm the bytes are stable (parse∘serialize is a fixed point).
    let dir2 = tempfile::tempdir().unwrap();
    let path2 = lock_path(&dir2);
    let v = serde_json::to_value(&parsed).unwrap();
    let h = v["installed"]["alpha@1.0.0"]["content_hash"]
        .as_str()
        .unwrap()
        .to_string();
    skills_lock::write_entry(&path2, "alpha@1.0.0", url_entry(&h)).unwrap();
    assert_eq!(
        first,
        std::fs::read(&path2).unwrap(),
        "serialize∘parse∘serialize must be a byte-stable fixed point"
    );
}

// ── schema is the source of truth (CLAUDE.md §14) ──

#[test]
fn skills_lock_schema_has_spec_faithful_shape() {
    // The lock key is spec §2200 `installed` (NOT phase-doc `entries`),
    // SriHash is the SRI pattern, $id matches the repo base-URL convention.
    let root = env!("CARGO_MANIFEST_DIR");
    let schema_path = std::path::Path::new(root)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("schemas/skills-lock.v1.json");
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).expect("read schema"))
            .expect("schema is valid JSON");

    assert_eq!(
        schema["$id"],
        json!("https://schemas.aria-runtime.dev/skills-lock.v1.json"),
        "$id must follow the schemas.aria-runtime.dev base-URL convention"
    );
    let required = schema["required"].as_array().expect("required[]");
    assert!(
        required.contains(&json!("installed")),
        "spec §2200 names the map key `installed`, not `entries`"
    );
    assert_eq!(
        schema["$defs"]["SriHash"]["pattern"],
        json!("^sha256-[A-Za-z0-9+/]+={0,2}$"),
        "SriHash must enforce the SRI algorithm prefix at the schema level"
    );
}

#[test]
fn generated_skills_lock_type_round_trips_a_spec_faithful_fixture() {
    let fixture = json!({
        "version": 1,
        "installed": {
            "pdf_summarizer@1.0.0": {
                "kind": "skill",
                "source": { "type": "url", "url": "https://example.com/pdf.md" },
                "content_hash": EMPTY_SRI,
                "installed_at": "2026-04-18T14:23:00Z",
                "tier_at_install": "promoted",
                "validation_report_id": "vr-789xyz"
            },
            "local_tool@0.2.0": {
                "kind": "tool",
                "source": { "type": "file", "path": "./tools/local.md" },
                "content_hash": EMPTY_SRI,
                "installed_at": "2026-04-18T14:24:00Z",
                "tier_at_install": "novice",
                "validation_report_id": "vr-aaa111"
            }
        }
    });
    let lock: SkillsLock =
        serde_json::from_value(fixture.clone()).expect("spec-faithful fixture deserializes");
    let reser = serde_json::to_value(&lock).expect("re-serialize");
    assert_eq!(reser, fixture, "generated type is a lossless mirror of the schema shape");

    // A bare-hex content_hash (spec sketch §2205 used `sha256:<hex>`)
    // must FAIL the SRI pattern — ADR-0014 deliberately tightened this.
    let mut bad = fixture;
    bad["installed"]["pdf_summarizer@1.0.0"]["content_hash"] = json!("sha256:deadbeef");
    assert!(
        serde_json::from_value::<SkillsLock>(bad).is_err(),
        "non-SRI hash must be rejected by the schema-derived type"
    );
}
