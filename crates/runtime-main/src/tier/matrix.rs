//! Tier allowlist tables — spec §8.security L4 (M05 Stage D).
//!
//! Data-driven matrix per phase-doc gotcha: "Matrix as data, not code".
//! Adding the v1.0+ Full tier means adding rows here, not nesting
//! if/else in the evaluator.
//!
//! Novice is the only tier with restrictions in v0.1 — Promoted is the
//! unconditional pass-through. The Full tier (post-v0.1 per §0d) would
//! gain its own table here.

use runtime_core::generated::capability::{CapabilityKind, CapabilityScope};

/// Discriminator for the scope-shape constraint a Novice allowance row applies.
///
/// `Any` means the row matches regardless of the request's
/// scope variant; `DomainOnly` means only `CapabilityScope::Domain`
/// requests satisfy the row (used for HTTPS-only Network in v0.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeShape {
    /// Any scope variant satisfies this row.
    Any,
    /// Only `CapabilityScope::Domain` satisfies this row.
    DomainOnly,
}

/// One row of the Novice tier allowlist. The evaluator walks
/// [`NOVICE_ALLOWED`] and accepts the request iff some row's
/// `kind` matches AND `scope_shape` is satisfied.
#[derive(Debug, Clone, Copy)]
pub struct NoviceAllowance {
    /// The coarse capability kind this row permits.
    pub kind: CapabilityKind,
    /// The scope-shape constraint this row applies.
    pub scope_shape: ScopeShape,
}

/// Novice tier allowlist (spec §8.security L4 + phase-doc D.3.1).
///
/// Forbidden by omission: `Write`, `Exec`, `ProcessSpawn`, plain (non-
/// Domain-scoped) `Network`. These map to the spec's "Promoted blocked
/// from `shell:true` and `network:["*"]`" — Novice is strictly more
/// restrictive than Promoted, so the same denials apply transitively.
pub const NOVICE_ALLOWED: &[NoviceAllowance] = &[
    NoviceAllowance {
        kind: CapabilityKind::Read,
        scope_shape: ScopeShape::Any,
    },
    NoviceAllowance {
        kind: CapabilityKind::Network,
        scope_shape: ScopeShape::DomainOnly,
    },
];

/// True iff `scope` satisfies `shape`. Pure function, table-friendly.
#[must_use]
pub const fn shape_matches(shape: ScopeShape, scope: &CapabilityScope) -> bool {
    match shape {
        ScopeShape::Any => true,
        ScopeShape::DomainOnly => matches!(scope, CapabilityScope::Domain(_)),
    }
}

/// True iff some row of [`NOVICE_ALLOWED`] permits the (kind, scope) pair.
#[must_use]
pub fn novice_table_permits(kind: CapabilityKind, scope: &CapabilityScope) -> bool {
    NOVICE_ALLOWED
        .iter()
        .any(|row| row.kind == kind && shape_matches(row.scope_shape, scope))
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::capability::{DomainPattern, GlobPattern, PathPattern};
    use std::str::FromStr;

    fn glob_scope(s: &str) -> CapabilityScope {
        CapabilityScope::Glob(GlobPattern::from_str(s).unwrap())
    }
    fn domain_scope(s: &str) -> CapabilityScope {
        CapabilityScope::Domain(DomainPattern::from_str(s).unwrap())
    }
    fn path_scope(s: &str) -> CapabilityScope {
        CapabilityScope::Path(PathPattern::from_str(s).unwrap())
    }

    #[test]
    fn novice_table_contains_read_any_scope() {
        // Read is allowed for every scope variant.
        assert!(novice_table_permits(
            CapabilityKind::Read,
            &glob_scope("src/**"),
        ));
        assert!(novice_table_permits(
            CapabilityKind::Read,
            &path_scope("src/lib/foo.rs"),
        ));
        assert!(novice_table_permits(
            CapabilityKind::Read,
            &domain_scope("api.example.com"),
        ));
    }

    #[test]
    fn novice_table_contains_network_domain_only() {
        // Network with Domain scope = OK; Network with Glob or Path scope = denied.
        assert!(novice_table_permits(
            CapabilityKind::Network,
            &domain_scope("api.example.com"),
        ));
        assert!(!novice_table_permits(
            CapabilityKind::Network,
            &glob_scope("*"),
        ));
        assert!(!novice_table_permits(
            CapabilityKind::Network,
            &path_scope("any"),
        ));
    }

    #[test]
    fn novice_table_excludes_write_exec_process_spawn() {
        // Forbidden-by-omission kinds — every scope variant must deny.
        for kind in [
            CapabilityKind::Write,
            CapabilityKind::Exec,
            CapabilityKind::ProcessSpawn,
        ] {
            assert!(!novice_table_permits(kind, &glob_scope("**")));
            assert!(!novice_table_permits(kind, &path_scope("any")));
            assert!(!novice_table_permits(
                kind,
                &domain_scope("any.example.com")
            ));
        }
    }

    #[test]
    fn shape_matches_any_accepts_every_variant() {
        assert!(shape_matches(ScopeShape::Any, &glob_scope("*")));
        assert!(shape_matches(ScopeShape::Any, &path_scope("any")));
        assert!(shape_matches(
            ScopeShape::Any,
            &domain_scope("any.example.com")
        ));
    }

    #[test]
    fn shape_matches_domain_only_rejects_glob_and_path() {
        assert!(!shape_matches(ScopeShape::DomainOnly, &glob_scope("*")));
        assert!(!shape_matches(ScopeShape::DomainOnly, &path_scope("any")));
        assert!(shape_matches(
            ScopeShape::DomainOnly,
            &domain_scope("api.example.com"),
        ));
    }

    #[test]
    fn novice_allowed_table_has_exactly_two_rows_in_v0_1() {
        // Pin the v0.1 table size — drift here means matrix expansion
        // happened and the test+CHANGELOG must catch up.
        assert_eq!(NOVICE_ALLOWED.len(), 2);
    }
}
