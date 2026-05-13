//! Tier evaluator — spec §8.security L4 (M05 Stage D).
//!
//! Stateless predicate: given a tier and a `CapabilityDeclaration`,
//! return `Ok(())` if the tier permits the request at all,
//! `Err(TierError::ForbiddenInTier)` otherwise.
//!
//! The evaluator sits BEFORE the L1+L2a enforcer in the dispatch chain:
//! tier check → enforcer check → dispatch. A Promoted user with a
//! `write` declaration still passes through L1; a Novice user requesting
//! `write` is rejected at L4 before L1 even runs.
//!
//! The Novice matrix lives in `matrix.rs` as a data table — adding a
//! v1.0+ Full tier means adding a table, not nesting if/else here.

use runtime_core::generated::capability::CapabilityDeclaration;

use crate::tier::error::TierError;
use crate::tier::matrix;

/// The two tiers shipped in v0.1 per §0d release scope. Full tier is
/// post-v0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Default-safe tier — curated allowlist (read + HTTPS-only network).
    /// First-run default; new users start here.
    Novice,
    /// Full capability surface — Promoted user toggled the auto-accept
    /// settings. L1 still narrows by declaration; the tier gate is a
    /// pass-through.
    Promoted,
}

impl Default for Tier {
    /// First-run default per §8.security spirit (default-safe). The
    /// persistence layer ([`crate::tier::load_tier`]) returns this when
    /// `tier.json` is absent.
    fn default() -> Self {
        Self::Novice
    }
}

/// Stateless L4 evaluator. The single public surface is
/// [`Self::allows`]; all matrix bookkeeping is inside [`crate::tier::matrix`].
pub struct TierEvaluator;

impl TierEvaluator {
    /// Check whether `tier` permits the requested capability AT ALL.
    /// Returns `Ok(())` when the tier permits; `Err(TierError)`
    /// otherwise. Called BEFORE the L1 enforcer's check — tier acts as
    /// the outer gate.
    ///
    /// # Errors
    ///
    /// - [`TierError::ForbiddenInTier`] when the request's
    ///   `capability_kind` (combined with its `scope` shape) is not
    ///   present in the Novice allowlist. Promoted always returns `Ok`.
    pub fn allows(tier: Tier, capability: &CapabilityDeclaration) -> Result<(), TierError> {
        match tier {
            Tier::Promoted => Ok(()),
            Tier::Novice => {
                if matrix::novice_table_permits(capability.kind, &capability.scope) {
                    Ok(())
                } else {
                    Err(TierError::ForbiddenInTier {
                        tier,
                        capability_kind: capability.kind,
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::capability::{
        CapabilityKind, CapabilityScope, DomainPattern, GlobPattern, PathPattern, ResourceName,
        SideEffectClass,
    };
    use std::str::FromStr;

    fn decl(
        kind: CapabilityKind,
        scope: CapabilityScope,
        side: SideEffectClass,
    ) -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind,
            resource: ResourceName::from_str("res").unwrap(),
            scope,
            side_effect_class: side,
        }
    }

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
    fn novice_allows_read() {
        let c = decl(
            CapabilityKind::Read,
            glob_scope("src/**"),
            SideEffectClass::Pure,
        );
        TierEvaluator::allows(Tier::Novice, &c).expect("Novice permits Read");
    }

    #[test]
    fn novice_denies_write() {
        let c = decl(
            CapabilityKind::Write,
            glob_scope("src/**"),
            SideEffectClass::FilesystemMutate,
        );
        let err = TierEvaluator::allows(Tier::Novice, &c).expect_err("Novice rejects Write");
        match err {
            TierError::ForbiddenInTier {
                tier,
                capability_kind,
            } => {
                assert_eq!(tier, Tier::Novice);
                assert_eq!(capability_kind, CapabilityKind::Write);
            }
        }
    }

    #[test]
    fn novice_allows_https_network() {
        let c = decl(
            CapabilityKind::Network,
            domain_scope("api.example.com"),
            SideEffectClass::NetworkEgress,
        );
        TierEvaluator::allows(Tier::Novice, &c).expect("Domain-scoped Network passes Novice");
    }

    #[test]
    fn novice_denies_plain_network() {
        // Glob-scoped Network — the "network:[*]" case the spec calls out.
        let c = decl(
            CapabilityKind::Network,
            glob_scope("*"),
            SideEffectClass::NetworkEgress,
        );
        TierEvaluator::allows(Tier::Novice, &c).expect_err("Glob-scoped Network denied for Novice");
    }

    #[test]
    fn novice_denies_path_scoped_network() {
        // Path scope is also outside the Domain-only allowlist row.
        let c = decl(
            CapabilityKind::Network,
            path_scope("api"),
            SideEffectClass::NetworkEgress,
        );
        TierEvaluator::allows(Tier::Novice, &c).expect_err("Path-scoped Network denied for Novice");
    }

    #[test]
    fn novice_denies_exec_and_process_spawn() {
        for kind in [CapabilityKind::Exec, CapabilityKind::ProcessSpawn] {
            let c = decl(kind, path_scope("any"), SideEffectClass::Pure);
            TierEvaluator::allows(Tier::Novice, &c).expect_err("Novice denies Exec / ProcessSpawn");
        }
    }

    #[test]
    fn promoted_allows_all_kinds() {
        for kind in [
            CapabilityKind::Read,
            CapabilityKind::Write,
            CapabilityKind::Exec,
            CapabilityKind::Network,
            CapabilityKind::ProcessSpawn,
        ] {
            let c = decl(kind, glob_scope("**"), SideEffectClass::Pure);
            TierEvaluator::allows(Tier::Promoted, &c).expect("Promoted permits every kind at L4");
        }
    }

    #[test]
    fn tier_default_is_novice() {
        assert_eq!(Tier::default(), Tier::Novice);
    }

    #[test]
    fn tier_serialize_lowercase() {
        // Serde rename_all = lowercase pins the wire format for tier.json.
        let json = serde_json::to_string(&Tier::Novice).unwrap();
        assert_eq!(json, "\"novice\"");
        let json = serde_json::to_string(&Tier::Promoted).unwrap();
        assert_eq!(json, "\"promoted\"");
    }

    #[test]
    fn tier_deserialize_lowercase() {
        let t: Tier = serde_json::from_str("\"novice\"").unwrap();
        assert_eq!(t, Tier::Novice);
        let t: Tier = serde_json::from_str("\"promoted\"").unwrap();
        assert_eq!(t, Tier::Promoted);
    }

    #[test]
    fn tier_equality_and_copy() {
        let a = Tier::Novice;
        let b = a;
        assert_eq!(a, b);
    }
}
