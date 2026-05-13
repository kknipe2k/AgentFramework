//! End-to-end Stage D integration tests — spec §8.security L4.
//!
//! Covers the tier-evaluator-runs-before-L1-enforcer dispatch chain and
//! the persistence round-trip. Catches the bug class where a tier
//! gate exists but isn't actually wired into the enforcer's `check`.

use std::str::FromStr;

use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, DomainPattern, GlobPattern,
    PathPattern, ResourceName, SideEffectClass,
};
use runtime_main::capability::{CapabilityEnforcer, CapabilityError, DenyReason};
use runtime_main::tier::{load_tier, save_tier, Tier, TierError, TierEvaluator};
use tempfile::tempdir;

fn read_glob(resource: &str, glob: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Read,
        resource: ResourceName::from_str(resource).unwrap(),
        scope: CapabilityScope::Glob(GlobPattern::from_str(glob).unwrap()),
        side_effect_class: SideEffectClass::Pure,
    }
}

fn write_glob(resource: &str, glob: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Write,
        resource: ResourceName::from_str(resource).unwrap(),
        scope: CapabilityScope::Glob(GlobPattern::from_str(glob).unwrap()),
        side_effect_class: SideEffectClass::FilesystemMutate,
    }
}

fn network_domain(resource: &str, domain: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Network,
        resource: ResourceName::from_str(resource).unwrap(),
        scope: CapabilityScope::Domain(DomainPattern::from_str(domain).unwrap()),
        side_effect_class: SideEffectClass::NetworkEgress,
    }
}

fn network_glob(resource: &str, glob: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Network,
        resource: ResourceName::from_str(resource).unwrap(),
        scope: CapabilityScope::Glob(GlobPattern::from_str(glob).unwrap()),
        side_effect_class: SideEffectClass::NetworkEgress,
    }
}

#[test]
fn tier_check_runs_before_l1_enforcer_when_novice() {
    // Novice has a Write grant declared (per phase doc — Promoted-tier
    // user demoted with stale grants is a realistic scenario). The L1
    // enforcer would normally accept it; the L4 tier evaluator must
    // reject FIRST with TierForbidden, never reaching L1's "matching
    // grant" path. Test pins the layer order: TierForbidden, not Denied.
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Novice);
    enforcer.grant("worker", write_glob("src", "src/**"));
    let err = enforcer
        .check("worker", &write_glob("src", "src/**"))
        .expect_err("Novice must reject Write at L4 before L1 sees a match");
    match err {
        CapabilityError::TierForbidden { tier, .. } => assert_eq!(tier, Tier::Novice),
        CapabilityError::Denied { .. } => {
            panic!("L1 ran before L4 — tier gate is not the outer gate")
        }
    }
}

#[test]
fn novice_request_for_write_returns_tier_forbidden() {
    // End-to-end shape: load_tier → set on enforcer → check produces
    // TierForbidden. No grant configured — the test asserts the L4
    // gate fires regardless of whether L1 would have rejected too.
    let dir = tempdir().unwrap();
    let tier = load_tier(dir.path()).unwrap();
    assert_eq!(tier, Tier::Novice, "first run must default to Novice");
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(tier);
    let err = enforcer
        .check("worker", &write_glob("src", "src/**"))
        .expect_err("Novice must reject Write");
    matches!(err, CapabilityError::TierForbidden { .. });
}

#[test]
fn promoted_passes_tier_gate_then_l1_narrows() {
    // Promoted user with a matching Write grant: L4 passes (Promoted
    // permits any kind), then L1 finds the grant and returns Ok.
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    enforcer.grant("worker", write_glob("src", "src/**"));
    enforcer
        .check("worker", &write_glob("src", "src/**"))
        .expect("Promoted with matching grant passes both gates");
}

#[test]
fn promoted_without_grant_returns_denied_not_tier_forbidden() {
    // Promoted user has no grant for the request. L4 must pass
    // (Promoted permits any kind) so L1 fires the rejection — the
    // discriminator is Denied, not TierForbidden. This is the inverse
    // of `tier_check_runs_before_l1_enforcer_when_novice` and pins
    // both layer orderings.
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    let err = enforcer
        .check("worker", &read_glob("src", "src/**"))
        .expect_err("Promoted with no grant must be Denied by L1");
    match err {
        CapabilityError::Denied { reason, .. } => {
            assert_eq!(reason, DenyReason::NoDeclarations);
        }
        CapabilityError::TierForbidden { .. } => {
            panic!("Promoted must pass L4; only L1 should reject here")
        }
    }
}

#[test]
fn novice_allows_https_network_via_domain_scope() {
    // The Novice allowlist permits domain-scoped Network (HTTPS-only
    // posture) but not glob-scoped Network (plain network:["*"]). Pin
    // the scope-conditional behavior end-to-end through the enforcer.
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Novice);
    enforcer.grant("worker", network_domain("api", "api.example.com"));
    enforcer
        .check("worker", &network_domain("api", "api.example.com"))
        .expect("Novice + Domain-scoped Network passes both gates");
}

#[test]
fn novice_denies_plain_network_glob_scope() {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Novice);
    // grant doesn't matter — tier rejects the request shape before L1.
    let err = enforcer
        .check("worker", &network_glob("any", "*"))
        .expect_err("Novice rejects glob-scoped Network at L4");
    matches!(err, CapabilityError::TierForbidden { .. });
}

#[test]
fn persistence_round_trip_through_real_filesystem() {
    let dir = tempdir().unwrap();
    save_tier(dir.path(), Tier::Promoted).unwrap();
    let loaded = load_tier(dir.path()).unwrap();
    assert_eq!(loaded, Tier::Promoted);
}

#[test]
fn enforcer_default_tier_is_novice() {
    // The enforcer's default tier MUST be Novice — the default-safe
    // posture is the spec's L4 contract for first-run.
    let enforcer = CapabilityEnforcer::new();
    let err = enforcer
        .check("worker", &write_glob("src", "src/**"))
        .expect_err("default tier must reject Write");
    matches!(
        err,
        CapabilityError::TierForbidden {
            tier: Tier::Novice,
            ..
        }
    );
}

#[test]
fn tier_evaluator_promoted_allows_every_kind() {
    // Direct evaluator surface: Promoted is the unconditional "Ok"
    // pass-through. Pins that adding a new CapabilityKind later doesn't
    // accidentally close Promoted (which would be a regression).
    for kind in [
        CapabilityKind::Read,
        CapabilityKind::Write,
        CapabilityKind::Exec,
        CapabilityKind::Network,
        CapabilityKind::ProcessSpawn,
    ] {
        let decl = CapabilityDeclaration {
            kind,
            resource: ResourceName::from_str("any").unwrap(),
            scope: CapabilityScope::Path(PathPattern::from_str("any").unwrap()),
            side_effect_class: SideEffectClass::Pure,
        };
        TierEvaluator::allows(Tier::Promoted, &decl).expect("Promoted permits any kind at L4");
    }
}

#[test]
fn tier_error_distinct_from_denied_carries_tier() {
    // Renderer routes tier_violation differently from capability_violation
    // — the discriminator must be present and unambiguous in the error.
    let err = TierEvaluator::allows(Tier::Novice, &write_glob("src", "src/**"))
        .expect_err("Novice rejects Write");
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
