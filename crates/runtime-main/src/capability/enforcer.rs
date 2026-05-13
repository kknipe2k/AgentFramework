//! L1 + L4 enforcer — spec §8.security (M05 Stage B + Stage D).
//!
//! [`CapabilityEnforcer`] owns per-agent capability grants + the current
//! user tier, and the `check(agent, requested)` predicate that runs
//! before every tool dispatch + sub-agent spawn. The check is two-layer:
//!
//! 1. **L4 (Stage D)** — [`crate::tier::TierEvaluator::allows`] gates
//!    the request by the user's tier. Novice rejects any kind outside
//!    its curated allowlist (Read + Domain-scoped Network); Promoted is
//!    a pass-through.
//! 2. **L1 (Stage B)** — if L4 passes, [`crate::capability::subsumes`]
//!    finds a matching grant in the per-agent grant map. Default-deny.
//!
//! Default-deny semantics are load-bearing (gotcha trap #1 from M05.B
//! stage prompt): an agent with no declared grants gets `Err(Denied {
//! reason: NoDeclarations })`, NOT `Ok`. The [`DenyReason`] discriminator
//! lets the renderer surface different copy for "you haven't declared
//! anything" vs "your declarations don't cover this".
//!
//! Event emission lives outside this module — Stage B mirrors the
//! framework_loader (M05.A) in-process emitter pattern: the enforcer
//! returns `Result`; the SDK consumer emits `capability_violation` on
//! `Err(Denied)` or `tier_violation` on `Err(TierForbidden)` before
//! invoking the HITL flow (gotcha trap #4 — event MUST emit BEFORE the
//! HITL prompt for renderer responsiveness).

use std::collections::HashMap;

use runtime_core::generated::capability::CapabilityDeclaration;

use crate::capability::declaration::subsumes;
use crate::capability::error::CapabilityError;
use crate::tier::{Tier, TierError, TierEvaluator};

/// Why a [`CapabilityEnforcer::check`] returned `Err`. Carried inside
/// [`CapabilityError::Denied`] so the renderer can surface different
/// copy per reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenyReason {
    /// The agent has no declarations recorded at all. Default-deny
    /// applies — agents must be explicitly granted before they can
    /// dispatch. This is the v0.1 boot state for any agent that hasn't
    /// been wired through the framework loader's grant pipeline.
    NoDeclarations,
    /// The agent has declarations, but none subsume the requested
    /// capability. A real mismatch — the agent declared X, attempted Y.
    NoMatchingGrant,
}

/// L1 + L4 capability enforcer.
///
/// Owns a `HashMap<AgentId, Vec<CapabilityDeclaration>>` of grants + the
/// user's current [`Tier`]. The `AgentId` key is a plain `String`
/// matching the
/// [`crate::sdk::request_capability::RequestCapabilityInvocation::agent_id`]
/// shape — no newtype yet at the runtime layer.
///
/// Cheap to construct via [`Self::new`]; cheap to clone (the underlying
/// `HashMap` clones the per-agent `Vec` of declarations).
///
/// The enforcer's tier starts at [`Tier::Novice`] (default-safe per
/// §8.security). The Tauri layer loads the persisted tier at app
/// startup and calls [`Self::set_tier`] before any dispatch runs; tier
/// transitions during a session also route through `set_tier`.
#[derive(Debug, Clone, Default)]
pub struct CapabilityEnforcer {
    grants_by_agent: HashMap<String, Vec<CapabilityDeclaration>>,
    current_tier: Tier,
}

impl CapabilityEnforcer {
    /// Construct an empty enforcer. Every agent is in default-deny state
    /// until [`Self::grant`] adds a declaration. Tier defaults to
    /// [`Tier::Novice`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Grant a capability to `agent`. Repeated grants append (do not
    /// deduplicate); the matching predicate is order-independent.
    pub fn grant(&mut self, agent: impl Into<String>, capability: CapabilityDeclaration) {
        self.grants_by_agent
            .entry(agent.into())
            .or_default()
            .push(capability);
    }

    /// Update the enforcer's current tier. Called at app startup with
    /// the value persisted in `tier.json`, and again on every
    /// successful tier transition.
    pub const fn set_tier(&mut self, tier: Tier) {
        self.current_tier = tier;
    }

    /// Read the enforcer's current tier — used by the Tauri
    /// `get_current_tier` command + the renderer's Settings panel.
    #[must_use]
    pub const fn current_tier(&self) -> Tier {
        self.current_tier
    }

    /// Two-layer L4 + L1 check.
    ///
    /// Layer ordering is the §8.security contract: the L4 tier gate
    /// runs BEFORE L1 so a Novice user with a stale Write grant is
    /// rejected with `TierForbidden`, not `Denied`. Renderer routes
    /// the two error shapes to distinct event variants.
    ///
    /// # Errors
    ///
    /// - [`CapabilityError::TierForbidden`] when the L4 tier gate
    ///   rejects the request (the user's tier excludes the requested
    ///   kind / scope shape).
    /// - [`CapabilityError::Denied`] with `reason: DenyReason::NoDeclarations`
    ///   when L4 passes but the agent has no entry in the grant map.
    /// - [`CapabilityError::Denied`] with `reason: DenyReason::NoMatchingGrant`
    ///   when L4 passes and the agent has declarations but none subsume
    ///   `requested`.
    pub fn check(
        &self,
        agent: &str,
        requested: &CapabilityDeclaration,
    ) -> Result<(), CapabilityError> {
        // L4 first — tier gate is the outer rejection. Stale grants
        // accumulated under a previous higher tier are correctly
        // blocked here when the user demotes back to Novice.
        TierEvaluator::allows(self.current_tier, requested).map_err(|e| match e {
            TierError::ForbiddenInTier {
                tier,
                capability_kind,
            } => CapabilityError::TierForbidden {
                agent_id: agent.to_string(),
                tier,
                capability_kind,
            },
        })?;
        // L1 — per-agent grant subsumption.
        let grants = self
            .grants_by_agent
            .get(agent)
            .ok_or_else(|| CapabilityError::Denied {
                agent_id: agent.to_string(),
                reason: DenyReason::NoDeclarations,
            })?;
        if grants.iter().any(|grant| subsumes(grant, requested)) {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                agent_id: agent.to_string(),
                reason: DenyReason::NoMatchingGrant,
            })
        }
    }

    /// How many grants are currently recorded for `agent`. Tests use
    /// this to assert grant accumulation; production callers don't need
    /// it.
    #[must_use]
    pub fn grant_count(&self, agent: &str) -> usize {
        self.grants_by_agent.get(agent).map_or(0, Vec::len)
    }

    /// Snapshot the grants for `agent` — used by the L2a narrowing path
    /// when an agent spawns a sub-agent and the SDK needs the parent's
    /// grant set to compute the child's narrowed grants.
    #[must_use]
    pub fn grants_for(&self, agent: &str) -> &[CapabilityDeclaration] {
        self.grants_by_agent.get(agent).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::capability::{
        CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, PathPattern,
        ResourceName, SideEffectClass,
    };
    use std::str::FromStr;

    fn read_src_glob() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("src").unwrap(),
            scope: CapabilityScope::Glob(GlobPattern::from_str("src/**").unwrap()),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    fn write_src_glob() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Write,
            resource: ResourceName::from_str("src").unwrap(),
            scope: CapabilityScope::Glob(GlobPattern::from_str("src/**").unwrap()),
            side_effect_class: SideEffectClass::FilesystemMutate,
        }
    }

    fn read_src_path(p: &str) -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("src").unwrap(),
            scope: CapabilityScope::Path(PathPattern::from_str(p).unwrap()),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    #[test]
    fn default_deny_when_no_declarations_for_agent() {
        let enforcer = CapabilityEnforcer::new();
        let err = enforcer
            .check("worker", &read_src_glob())
            .expect_err("no declarations must err");
        match err {
            CapabilityError::Denied { reason, agent_id } => {
                assert_eq!(reason, DenyReason::NoDeclarations);
                assert_eq!(agent_id, "worker");
            }
            CapabilityError::TierForbidden { .. } => {
                panic!("Read should pass L4 — only L1 should reject");
            }
        }
    }

    #[test]
    fn exact_match_grant_passes_check() {
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.grant("worker", read_src_glob());
        enforcer
            .check("worker", &read_src_glob())
            .expect("identical declaration must pass");
    }

    #[test]
    fn grant_for_one_agent_does_not_satisfy_other_agent() {
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.grant("worker", read_src_glob());
        let err = enforcer
            .check("intruder", &read_src_glob())
            .expect_err("other agent must default-deny");
        match err {
            CapabilityError::Denied { reason, .. } => {
                assert_eq!(reason, DenyReason::NoDeclarations);
            }
            CapabilityError::TierForbidden { .. } => {
                panic!("Read should pass L4 — only L1 should reject");
            }
        }
    }

    #[test]
    fn scope_widening_denied_when_request_falls_outside_glob() {
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.grant("worker", read_src_glob());
        // The grant covers `src/**`. The request targets `docs/foo.md`.
        // No matching grant.
        let request = read_src_path("docs/foo.md");
        let err = enforcer
            .check("worker", &request)
            .expect_err("outside-glob path must err");
        match err {
            CapabilityError::Denied { reason, .. } => {
                assert_eq!(reason, DenyReason::NoMatchingGrant);
            }
            CapabilityError::TierForbidden { .. } => {
                panic!("Read should pass L4 — only L1 should reject");
            }
        }
    }

    #[test]
    fn side_effect_class_mismatch_denied() {
        let mut enforcer = CapabilityEnforcer::new();
        // Stage D: Promote past the L4 gate so the L1 side-effect-class
        // mismatch — not the tier gate — produces the rejection. Under
        // Novice, the Write request would be rejected at L4 first
        // (TierForbidden), shadowing the L1 invariant this test pins.
        enforcer.set_tier(Tier::Promoted);
        // Granted read (pure); request write (filesystem_mutate).
        enforcer.grant("worker", read_src_glob());
        let err = enforcer
            .check("worker", &write_src_glob())
            .expect_err("read grant cannot satisfy write request");
        match err {
            CapabilityError::Denied { reason, .. } => {
                assert_eq!(reason, DenyReason::NoMatchingGrant);
            }
            CapabilityError::TierForbidden { .. } => {
                panic!("Promoted tier should not reject Write at L4");
            }
        }
    }

    #[test]
    fn default_tier_is_novice() {
        let enforcer = CapabilityEnforcer::new();
        assert_eq!(enforcer.current_tier(), Tier::Novice);
    }

    #[test]
    fn set_tier_updates_current_tier() {
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.set_tier(Tier::Promoted);
        assert_eq!(enforcer.current_tier(), Tier::Promoted);
        enforcer.set_tier(Tier::Novice);
        assert_eq!(enforcer.current_tier(), Tier::Novice);
    }

    #[test]
    fn novice_tier_rejects_write_with_tier_forbidden_error() {
        // Stage D: tier check runs BEFORE the L1 grant lookup. A
        // Novice user with a Write grant gets TierForbidden, never
        // reaching the L1 path that would have returned Ok.
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.grant("worker", write_src_glob());
        let err = enforcer
            .check("worker", &write_src_glob())
            .expect_err("Novice rejects Write at L4");
        match err {
            CapabilityError::TierForbidden {
                tier,
                capability_kind,
                agent_id,
            } => {
                assert_eq!(tier, Tier::Novice);
                assert_eq!(capability_kind, CapabilityKind::Write);
                assert_eq!(agent_id, "worker");
            }
            CapabilityError::Denied { .. } => {
                panic!("L1 ran before L4 — tier gate is not the outer gate");
            }
        }
    }

    #[test]
    fn promoted_tier_passes_l4_then_l1_finds_grant() {
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.set_tier(Tier::Promoted);
        enforcer.grant("worker", write_src_glob());
        enforcer
            .check("worker", &write_src_glob())
            .expect("Promoted with matching Write grant passes both gates");
    }

    #[test]
    fn tier_demotion_invalidates_previously_passing_check() {
        // Stage D pattern: a session that promoted, dispatched
        // successfully, then demoted must see subsequent Write requests
        // rejected — the tier change takes effect immediately.
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.set_tier(Tier::Promoted);
        enforcer.grant("worker", write_src_glob());
        enforcer
            .check("worker", &write_src_glob())
            .expect("under Promoted, Write passes");
        enforcer.set_tier(Tier::Novice);
        let err = enforcer
            .check("worker", &write_src_glob())
            .expect_err("after demotion to Novice, Write is forbidden");
        matches!(err, CapabilityError::TierForbidden { .. });
    }

    #[test]
    fn multi_call_invariant_both_succeed_in_sequence() {
        // Gotcha #69: IPC + stateful primitives need multi-call invariant
        // tests. Two sequential `check` calls against the same grant set
        // must both succeed; first-call mutation of internal state would
        // break the second call.
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.grant("worker", read_src_glob());
        enforcer
            .check("worker", &read_src_glob())
            .expect("first check");
        enforcer
            .check("worker", &read_src_glob())
            .expect("second check");
        // And a third for good measure.
        enforcer
            .check("worker", &read_src_glob())
            .expect("third check");
    }

    #[test]
    fn check_picks_satisfying_grant_among_many() {
        // Multiple grants — the matching one wins regardless of position.
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.grant("worker", write_src_glob()); // first — does not match
        enforcer.grant("worker", read_src_glob()); // second — matches
        assert_eq!(enforcer.grant_count("worker"), 2);
        enforcer
            .check("worker", &read_src_glob())
            .expect("any matching grant suffices");
    }

    #[test]
    fn grants_for_unknown_agent_returns_empty_slice() {
        let enforcer = CapabilityEnforcer::new();
        assert!(enforcer.grants_for("unknown").is_empty());
        assert_eq!(enforcer.grant_count("unknown"), 0);
    }

    #[test]
    fn grants_for_known_agent_returns_full_slice() {
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.grant("worker", read_src_glob());
        enforcer.grant("worker", write_src_glob());
        let grants = enforcer.grants_for("worker");
        assert_eq!(grants.len(), 2);
    }

    #[test]
    fn default_enforcer_has_no_grants() {
        let enforcer = CapabilityEnforcer::default();
        assert_eq!(enforcer.grant_count("anyone"), 0);
    }

    #[test]
    fn agent_id_carried_on_denied_error() {
        // The renderer needs the agent id to route the capability-violation
        // event to the right node; verify it round-trips through Err.
        let enforcer = CapabilityEnforcer::new();
        let err = enforcer
            .check("specific-agent-id", &read_src_glob())
            .expect_err("err");
        match err {
            CapabilityError::Denied { agent_id, .. } => {
                assert_eq!(agent_id, "specific-agent-id");
            }
            CapabilityError::TierForbidden { .. } => {
                panic!("Read should pass L4 — only L1 should reject");
            }
        }
    }
}
