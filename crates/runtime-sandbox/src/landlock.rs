//! Linux landlock filesystem fence — §8.security L3 OS isolation (Linux).
//!
//! Restricts the sandbox subprocess's filesystem access via the Landlock
//! LSM (Linux Security Module, available since kernel 5.13). The
//! installed ruleset is the inverse of seccomp: instead of an allowlist
//! of syscalls, it's an allowlist of *paths* + access rights. Anything
//! outside the allowed paths returns `EACCES` regardless of file
//! permissions — even if running as root.
//!
//! v0.1 policy: allow read + write + create + remove under the IPC
//! socket's parent directory only. The validator is pure (no IO), and
//! the subprocess's only filesystem need is creating / accepting on the
//! Unix domain socket. Everything else (`/etc`, `/home`, `/proc`'s
//! mutating endpoints, anywhere the parent didn't explicitly grant) is
//! denied.
//!
//! Tightening this policy further is an M09 carry-forward — once
//! generators wire production callers we can scope per-validation.
//!
//! ## Compatibility
//!
//! The `landlock` crate uses `CompatLevel::BestEffort` by default: on a
//! kernel without Landlock support `restrict_self` returns `Ok` with a
//! `RulesetStatus::NotEnforced`. This module preserves that behavior —
//! kernels older than 5.13 silently fall through (the sandbox still
//! runs; just without the filesystem fence). The seccomp filter
//! (`crate::seccomp` — bare backticks per gotcha #55) is the primary
//! safety net; landlock is
//! defense-in-depth.

#![cfg(target_os = "linux")]

use std::path::Path;

use landlock::{
    path_beneath_rules, Access, AccessFs, Ruleset, RulesetAttr, RulesetCreated, RulesetCreatedAttr,
    RulesetStatus, ABI,
};

use crate::error::SandboxError;

/// ABI level pinned for v0.1. Landlock ABI v3 lands in Linux 6.2 and
/// adds `Truncate` plus `Refer` semantics. Pinning v3 keeps the rule set
/// stable; older kernels degrade gracefully via `CompatLevel::BestEffort`.
const ABI_LEVEL: ABI = ABI::V3;

/// The access bits granted under each allowed path: read + write + create
/// regular files / dirs + remove. Wide enough for the IPC socket bind
/// path (mkdir + unlink + create + open), narrow enough to deny
/// `make_sock` for sockets in other directories or `execute` permission.
fn rw_access(abi: ABI) -> AccessFs {
    AccessFs::ReadFile
        | AccessFs::ReadDir
        | AccessFs::WriteFile
        | AccessFs::MakeReg
        | AccessFs::MakeDir
        | AccessFs::RemoveFile
        | AccessFs::RemoveDir
        | AccessFs::MakeSock
        | AccessFs::Refer
        | AccessFs::from_all(abi).intersection(AccessFs::Truncate)
}

/// Build (but do not install) a landlock ruleset that grants read+write
/// access on the supplied paths and denies everything else.
///
/// Returns a [`RulesetCreated`] handle that the caller can either
/// `restrict_self()` (commit) or drop (no-op — useful for tests).
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if the kernel rejects the rule
/// set, the access mask is invalid, or a supplied path cannot be opened.
pub fn build_ruleset(allowed_paths: &[&Path]) -> Result<RulesetCreated, SandboxError> {
    let abi = ABI_LEVEL;
    let access = rw_access(abi);
    let created = Ruleset::default()
        .handle_access(access)
        .map_err(|e| SandboxError::Isolation(format!("landlock handle_access: {e}")))?
        .create()
        .map_err(|e| SandboxError::Isolation(format!("landlock create: {e}")))?
        .add_rules(path_beneath_rules(allowed_paths, access))
        .map_err(|e| SandboxError::Isolation(format!("landlock add_rules: {e}")))?;
    Ok(created)
}

/// Build the ruleset and commit it to the calling thread (`restrict_self`).
/// All subsequent syscalls — and syscalls on any thread spawned by this
/// one — are restricted to the allowed paths.
///
/// The `PR_SET_NO_NEW_PRIVS` flag is set automatically by the landlock
/// crate before commit, matching the kernel's requirement.
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if the ruleset cannot be created
/// or installed. A `NotEnforced` status (older kernel without Landlock)
/// is logged at `warn` level but NOT treated as an error — the seccomp
/// filter remains the primary safety net.
pub fn install(allowed_paths: &[&Path]) -> Result<(), SandboxError> {
    let created = build_ruleset(allowed_paths)?;
    let status = created
        .restrict_self()
        .map_err(|e| SandboxError::Isolation(format!("landlock restrict_self: {e}")))?;
    match status.ruleset {
        RulesetStatus::FullyEnforced => {
            tracing::info!(paths = allowed_paths.len(), "landlock ruleset enforced");
        }
        RulesetStatus::PartiallyEnforced => {
            tracing::warn!(
                paths = allowed_paths.len(),
                "landlock ruleset partially enforced (kernel ABI < pinned)"
            );
        }
        RulesetStatus::NotEnforced => {
            tracing::warn!("landlock not enforced (kernel < 5.13); seccomp remains primary fence");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_ruleset_succeeds_for_existing_path() {
        // /tmp exists on every Linux CI runner; build_ruleset must
        // succeed against it. Use a fresh tempdir to avoid coupling to
        // /tmp's exact state.
        let dir = tempfile::TempDir::new().expect("tempdir");
        let p = dir.path();
        let result = build_ruleset(&[p]);
        assert!(
            result.is_ok(),
            "build_ruleset failed against {}: {:?}",
            p.display(),
            result.err()
        );
    }

    #[test]
    fn build_ruleset_succeeds_for_multiple_paths() {
        let dir1 = tempfile::TempDir::new().expect("tempdir1");
        let dir2 = tempfile::TempDir::new().expect("tempdir2");
        let result = build_ruleset(&[dir1.path(), dir2.path()]);
        assert!(
            result.is_ok(),
            "build_ruleset failed for multiple paths: {:?}",
            result.err()
        );
    }

    #[test]
    fn build_ruleset_rejects_nonexistent_path() {
        // A path that doesn't exist cannot have a PathBeneath rule
        // attached; the kernel rejects the file descriptor open.
        let bogus = Path::new("/nonexistent/agent-runtime/m05-c2-landlock-test");
        let result = build_ruleset(&[bogus]);
        assert!(
            result.is_err(),
            "build_ruleset should fail for nonexistent path"
        );
    }

    #[test]
    fn rw_access_grants_read_and_write() {
        let access = rw_access(ABI_LEVEL);
        assert!(access.contains(AccessFs::ReadFile));
        assert!(access.contains(AccessFs::WriteFile));
        assert!(access.contains(AccessFs::ReadDir));
        assert!(access.contains(AccessFs::MakeReg));
        assert!(access.contains(AccessFs::RemoveFile));
        // Execute must NOT be granted — even within the socket dir.
        assert!(
            !access.contains(AccessFs::Execute),
            "rw_access leaks execute permission"
        );
    }
}
