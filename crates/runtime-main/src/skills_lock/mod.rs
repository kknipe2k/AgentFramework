//! `skills.lock` — per-framework artifact integrity ledger (spec
//! §2181-2216; M07 Stage B; ADR-0014).
//!
//! Path-agnostic: callers pass the resolved lock path; the Tauri shell
//! resolves `<framework_root>/skills.lock` (CLAUDE.md §9 archetype,
//! mirroring `audit::file_path`). The content hash is SRI-encoded
//! (`sha256-<standard-base64>`) so the algorithm is self-describing and
//! swappable without a schema break (npm/SRI convention; ADR-0014).
//!
//! The lock is checked into the user's framework repo alongside the
//! framework JSON (spec §2216). `write_entry` serializes with sorted
//! `name@version` keys and a stable field order so two installs of the
//! same artifact set produce a byte-identical file (spec §2204
//! reproducible cross-machine installs) and git auto-resolves
//! concurrent adds rather than conflicting.
//!
//! `verify` recomputes the hash on every artifact load; a mismatch is
//! a HARD block (integrity > availability) returned as
//! `LockError::HashMismatch`, which the load path maps onto the
//! schema-faithful `AgentEvent::ArtifactHashMismatch` and refuses to
//! run the drifted bytes.

/// `LockError` — the integrity-block error surface.
pub mod error;

use std::path::Path;

use base64::Engine as _;
use runtime_core::generated::skills_lock::{LockEntry, SkillsLock};
use sha2::{Digest, Sha256};

pub use error::LockError;

/// SRI-encode a SHA-256 over the artifact bytes: `sha256-<base64>`.
///
/// Standard base64 (the `+/=` alphabet, not URL-safe) per the SRI
/// convention and the `schemas/skills-lock.v1.json` `SriHash` pattern.
/// Deterministic and platform-independent: the same bytes produce the
/// same string on Windows / Linux / macOS, which is what makes a lock
/// written on one machine verifiable on another (spec §2204).
#[must_use]
pub fn content_hash(artifact_bytes: &[u8]) -> String {
    let digest = Sha256::digest(artifact_bytes);
    format!(
        "sha256-{}",
        base64::engine::general_purpose::STANDARD.encode(digest)
    )
}

/// Read and parse the `skills.lock` at `path`.
///
/// # Errors
///
/// - `LockError::Io` when the file cannot be read (missing lock,
///   permission denied). A missing lock is an `Io` error of kind
///   `NotFound`; `write_entry` treats that case as "start a fresh
///   lock", but callers that *read* a lock expect it to exist.
/// - `LockError::Parse` when the bytes are not valid JSON or do not
///   match the schema-derived `SkillsLock` shape (e.g. a non-SRI
///   `content_hash` fails the generated newtype's pattern check).
pub fn read(path: &Path) -> Result<SkillsLock, LockError> {
    let text = std::fs::read_to_string(path)?;
    let lock = serde_json::from_str(&text)?;
    Ok(lock)
}

/// Insert (or replace) the `name@version` → entry mapping and rewrite
/// the lock in canonical form.
///
/// Creates the lock file with a fresh `{ "version": 1, "installed": {}
/// }` skeleton if it does not exist yet (the first install in a
/// framework). Re-installing the same `name@version` replaces the entry
/// in place rather than duplicating it.
///
/// # Errors
///
/// - `LockError::Io` when the existing lock cannot be read for a
///   reason other than "does not exist", or when the rewrite cannot be
///   written (missing parent directory, permission denied).
/// - `LockError::Parse` when the existing lock file is corrupt.
pub fn write_entry(path: &Path, key: &str, entry: LockEntry) -> Result<(), LockError> {
    let mut lock = match read(path) {
        Ok(lock) => lock,
        Err(LockError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => empty_lock()?,
        Err(other) => return Err(other),
    };
    lock.installed.insert(key.to_string(), entry);
    std::fs::write(path, to_canonical_string(&lock)?)?;
    Ok(())
}

/// Verify an artifact's bytes against its locked content hash.
///
/// Recompute the SRI hash over `artifact_bytes` and compare it to the
/// hash stored for `artifact_ref` in the lock at `path`.
///
/// # Errors
///
/// - `LockError::Io` / `LockError::Parse` propagated from `read`
///   (a missing or corrupt lock blocks the load — it never silently
///   passes).
/// - `LockError::NotFound` when `artifact_ref` has no lock entry (an
///   un-locked artifact must not load by virtue of an absent record).
/// - `LockError::HashMismatch` when the recomputed hash differs from
///   the locked hash — the load path maps this to
///   `AgentEvent::ArtifactHashMismatch` and BLOCKS the artifact.
pub fn verify(path: &Path, artifact_ref: &str, artifact_bytes: &[u8]) -> Result<(), LockError> {
    let lock = read(path)?;
    let entry = lock
        .installed
        .get(artifact_ref)
        .ok_or_else(|| LockError::NotFound(artifact_ref.to_string()))?;
    let expected = entry.content_hash.as_str();
    let actual = content_hash(artifact_bytes);
    if expected == actual {
        Ok(())
    } else {
        Err(LockError::HashMismatch {
            artifact_ref: artifact_ref.to_string(),
            expected: expected.to_string(),
            actual,
        })
    }
}

/// An empty lock skeleton (`version: 1`, no entries). Built through the
/// schema-derived type so the `version` const stays the single source
/// of truth rather than a hand-typed literal.
fn empty_lock() -> Result<SkillsLock, LockError> {
    let lock = serde_json::from_value(serde_json::json!({ "version": 1, "installed": {} }))?;
    Ok(lock)
}

/// Serialize the lock canonically (spec §2204/§2216 reproducibility):
/// `installed` keys sorted alphabetically by `name@version`, stable
/// field order, pretty-printed (one entry per line region) for git
/// mergeability over compactness. The explicit `BTreeMap`-style sort
/// makes the output independent of the `HashMap` iteration order AND of
/// whether `serde_json`'s `preserve_order` feature is enabled anywhere
/// in the workspace — two installs of the same set are byte-identical.
fn to_canonical_string(lock: &SkillsLock) -> Result<String, LockError> {
    let mut keys: Vec<&String> = lock.installed.keys().collect();
    keys.sort();
    let mut installed = serde_json::Map::new();
    for k in keys {
        installed.insert(k.clone(), serde_json::to_value(&lock.installed[k])?);
    }
    let mut root = serde_json::Map::new();
    root.insert("version".to_string(), lock.version.clone());
    root.insert(
        "installed".to_string(),
        serde_json::Value::Object(installed),
    );
    let text = serde_json::to_string_pretty(&serde_json::Value::Object(root))?;
    Ok(format!("{text}\n"))
}
