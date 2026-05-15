//! Per-method unit fixture for the §8.security L1 + L2a enforcer.
//!
//! Exercises the enforcer + narrowing primitives + the in-process
//! emitter contract through a stand-in `dispatch_with_check` wrapper.
//! Pre-M06.A this file ALSO claimed to be the "what the SDK will call"
//! surface; M06 Stage A (ADR-0009 closure) ports those scenarios to
//! real call-path integration tests in
//! `crates/runtime-main/tests/sdk_capability_integration.rs` (L1 wire-up)
//! and `crates/runtime-main/tests/sdk_narrowing_integration.rs` (L2a
//! wire-up). Those are now the canonical wire-trace surfaces; this
//! file remains as the per-method unit fixture for the enforcer +
//! narrowing primitives — i.e., the fixture that exercises the
//! emitter contract independently of the production SDK loop.
//!
//! Gotcha trap #4 from M05.B stage prompt: `capability_violation`
//! events MUST emit BEFORE the HITL flow returns control. This test
//! pins the ordering: the emitter records the event, the wrapping
//! routine returns Err, and the test asserts both observations.
//!
//! Gotcha #66 contract-fidelity: the M06.A integration tests in
//! `sdk_*_integration.rs` are the load-bearing gate against the
//! M05.V-class "primitive ships green; production path missing" bug.
//! This smoke remains as a lower-level guard against per-method
//! regressions; it intentionally does NOT exercise the SDK loop.

use runtime_core::event::{AgentEvent, CapabilityKindRef};
use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, PathPattern, ResourceName,
    SideEffectClass,
};
use runtime_main::capability::{narrow, CapabilityEnforcer, CapabilityError, DenyReason};
use runtime_main::framework_loader::Emitter;
use std::str::FromStr;
use std::sync::Mutex;

/// Test emitter that records every emitted event in declaration order.
#[derive(Default)]
struct RecordingEmitter {
    events: Mutex<Vec<AgentEvent>>,
}

impl RecordingEmitter {
    fn snapshot(&self) -> Vec<AgentEvent> {
        self.events.lock().expect("no poisoning").clone()
    }
}

#[async_trait::async_trait]
impl Emitter for RecordingEmitter {
    async fn emit(&self, event: AgentEvent) {
        self.events.lock().expect("no poisoning").push(event);
    }
}

fn read_src_glob() -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Read,
        resource: ResourceName::from_str("src").unwrap(),
        scope: CapabilityScope::Glob(GlobPattern::from_str("src/**").unwrap()),
        side_effect_class: SideEffectClass::Pure,
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

/// Stand-in for the SDK's eventual `dispatch_tool` wrap (D1 in M05.B
/// retro). Production wrapper will look identical: `check` →
/// `emit_grant` on Ok / `emit_violation + HITL` on Err. Smoke tests
/// here ensure the event emission ordering contract is honored before
/// the SDK wires it.
async fn dispatch_with_check(
    enforcer: &CapabilityEnforcer,
    emitter: &impl Emitter,
    agent_id: &str,
    requested: &CapabilityDeclaration,
) -> Result<(), CapabilityError> {
    match enforcer.check(agent_id, requested) {
        Ok(()) => {
            // The grant event names the agent + the granted resource.
            emitter
                .emit(AgentEvent::CapabilityGrant {
                    parent_agent_id: None,
                    granted_to: agent_id.to_string(),
                    capability_kind: kind_to_ref(requested.kind),
                    resource: (*requested.resource).clone(),
                    narrowed_from: None,
                })
                .await;
            Ok(())
        }
        Err(err) => {
            match &err {
                CapabilityError::Denied {
                    reason,
                    agent_id: agent,
                } => {
                    emitter
                        .emit(AgentEvent::CapabilityViolation {
                            agent_id: agent.clone(),
                            capability_kind: kind_to_ref(requested.kind),
                            requested_action: format!(
                                "requested {kind:?} on '{resource}'",
                                kind = requested.kind,
                                resource = *requested.resource
                            ),
                            declared_scope: match reason {
                                DenyReason::NoDeclarations => {
                                    "no capabilities declared".to_string()
                                }
                                DenyReason::NoMatchingGrant => {
                                    "declared grants do not cover this request".to_string()
                                }
                            },
                        })
                        .await;
                }
                CapabilityError::TierForbidden {
                    agent_id: agent,
                    tier,
                    capability_kind,
                } => {
                    // Stage D — L4 tier gate rejected before L1. Route
                    // to the tier_violation event variant; the renderer
                    // surfaces a tier-violation modal in the Settings
                    // panel instead of the capability-violation modal.
                    emitter
                        .emit(AgentEvent::TierViolation {
                            agent_id: agent.clone(),
                            tier: match tier {
                                runtime_main::tier::Tier::Novice => {
                                    runtime_core::event::TierRef::Novice
                                }
                                runtime_main::tier::Tier::Promoted => {
                                    runtime_core::event::TierRef::Promoted
                                }
                            },
                            capability_kind: kind_to_ref(*capability_kind),
                            attempted_action: format!(
                                "requested {kind:?} on '{resource}' under {tier:?} tier",
                                kind = capability_kind,
                                resource = *requested.resource,
                            ),
                        })
                        .await;
                }
            }
            Err(err)
        }
    }
}

const fn kind_to_ref(k: CapabilityKind) -> CapabilityKindRef {
    match k {
        CapabilityKind::Read => CapabilityKindRef::Read,
        CapabilityKind::Write => CapabilityKindRef::Write,
        CapabilityKind::Exec => CapabilityKindRef::Exec,
        CapabilityKind::Network => CapabilityKindRef::Network,
        CapabilityKind::ProcessSpawn => CapabilityKindRef::ProcessSpawn,
    }
}

#[tokio::test]
async fn tool_call_with_grant_succeeds_and_emits_capability_grant() {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.grant("worker", read_src_glob());
    let emitter = RecordingEmitter::default();

    dispatch_with_check(
        &enforcer,
        &emitter,
        "worker",
        &read_src_path("src/lib/foo.rs"),
    )
    .await
    .expect("grant covers request; dispatch must succeed");

    let events = emitter.snapshot();
    assert_eq!(events.len(), 1);
    match &events[0] {
        AgentEvent::CapabilityGrant {
            granted_to,
            capability_kind,
            resource,
            parent_agent_id,
            narrowed_from,
        } => {
            assert_eq!(granted_to, "worker");
            assert_eq!(*capability_kind, CapabilityKindRef::Read);
            assert_eq!(resource, "src");
            assert!(parent_agent_id.is_none());
            assert!(narrowed_from.is_none());
        }
        other => panic!("expected CapabilityGrant, got {other:?}"),
    }
}

#[tokio::test]
async fn tool_call_without_grant_denied_and_emits_capability_violation_before_err_returns() {
    // Gotcha trap #4: emit MUST come before the err return so the
    // renderer surfaces the violation state even if the awaiter never
    // unblocks. We assert (a) event observable, (b) err observable —
    // and that (a) was recorded BEFORE the test sees (b) because the
    // emitter's `emit` is awaited before the `Err(...)` returns.
    let enforcer = CapabilityEnforcer::new(); // no grants
    let emitter = RecordingEmitter::default();

    let err = dispatch_with_check(&enforcer, &emitter, "worker", &read_src_glob())
        .await
        .expect_err("no grants = default-deny");

    let events = emitter.snapshot();
    assert_eq!(events.len(), 1, "violation must be emitted before err");
    match &events[0] {
        AgentEvent::CapabilityViolation {
            agent_id,
            capability_kind,
            requested_action,
            declared_scope,
        } => {
            assert_eq!(agent_id, "worker");
            assert_eq!(*capability_kind, CapabilityKindRef::Read);
            assert!(!requested_action.is_empty());
            assert!(declared_scope.contains("no capabilities declared"));
        }
        other => panic!("expected CapabilityViolation, got {other:?}"),
    }

    // The err carries the default-deny reason.
    match err {
        CapabilityError::Denied { reason, .. } => {
            assert_eq!(reason, DenyReason::NoDeclarations);
        }
        CapabilityError::TierForbidden { .. } => {
            panic!("Read should pass L4 (Novice allows Read); L1 should reject");
        }
    }
}

#[tokio::test]
async fn dispatch_emits_no_matching_grant_violation_when_declarations_exist_but_dont_cover() {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.grant("worker", read_src_glob());
    let emitter = RecordingEmitter::default();

    // Request something outside the granted glob.
    let outside = read_src_path("docs/foo.md");
    let err = dispatch_with_check(&enforcer, &emitter, "worker", &outside)
        .await
        .expect_err("outside-glob request must err");

    let events = emitter.snapshot();
    assert_eq!(events.len(), 1);
    match &err {
        CapabilityError::Denied { reason, .. } => {
            assert_eq!(*reason, DenyReason::NoMatchingGrant);
        }
        CapabilityError::TierForbidden { .. } => {
            panic!("Read should pass L4 (Novice allows Read); L1 should reject");
        }
    }

    match &events[0] {
        AgentEvent::CapabilityViolation { declared_scope, .. } => {
            assert!(
                declared_scope.contains("declared grants do not cover"),
                "{declared_scope}"
            );
        }
        other => panic!("expected CapabilityViolation, got {other:?}"),
    }
}

#[tokio::test]
async fn narrowing_to_child_emits_one_capability_grant_per_subset_declaration() {
    // Spec §8.security L2a: a parent spawns a sub-agent and narrows a
    // subset of its grants to the child. Each narrowed declaration
    // should surface as a `capability_grant` event with
    // `parent_agent_id` populated.
    let parent_grants = vec![read_src_glob()];
    let proposed_child_grants = vec![read_src_path("src/lib/foo.rs")];

    let narrowed =
        narrow(&parent_grants, &proposed_child_grants).expect("subset child grants narrow OK");
    assert_eq!(narrowed.len(), 1);

    let emitter = RecordingEmitter::default();
    for grant in &narrowed {
        emitter
            .emit(AgentEvent::CapabilityGrant {
                parent_agent_id: Some("orchestrator".to_string()),
                granted_to: "subagent".to_string(),
                capability_kind: kind_to_ref(grant.kind),
                resource: (*grant.resource).clone(),
                narrowed_from: Some("parent grant: read src/** (glob)".to_string()),
            })
            .await;
    }

    let events = emitter.snapshot();
    assert_eq!(events.len(), 1);
    match &events[0] {
        AgentEvent::CapabilityGrant {
            parent_agent_id,
            granted_to,
            narrowed_from,
            ..
        } => {
            assert_eq!(parent_agent_id.as_deref(), Some("orchestrator"));
            assert_eq!(granted_to, "subagent");
            assert!(narrowed_from.is_some());
        }
        other => panic!("expected CapabilityGrant, got {other:?}"),
    }
}

#[tokio::test]
async fn narrowing_rejects_child_widening_parent_scope() {
    // Parent: narrow read on a specific path.
    // Child: read glob covering more than the parent.
    // Must err with CapabilityNotHeldByParent.
    use runtime_main::capability::NarrowingError;

    let parent = vec![read_src_path("src/lib/foo.rs")];
    let proposed = vec![read_src_glob()];
    let err = narrow(&parent, &proposed).expect_err("child widening denied");
    match err {
        NarrowingError::CapabilityNotHeldByParent { proposed: p } => {
            assert_eq!(p.kind, CapabilityKind::Read);
            assert!(matches!(p.scope, CapabilityScope::Glob(_)));
        }
    }
}

#[tokio::test]
async fn multi_call_sequence_both_dispatches_succeed_and_emit_separately() {
    // Gotcha #69: stateful primitives need multi-call invariant tests.
    // Two sequential dispatches against the same grant set must each
    // emit their own grant event.
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.grant("worker", read_src_glob());
    let emitter = RecordingEmitter::default();

    dispatch_with_check(&enforcer, &emitter, "worker", &read_src_path("src/a.rs"))
        .await
        .expect("first dispatch");
    dispatch_with_check(&enforcer, &emitter, "worker", &read_src_path("src/b.rs"))
        .await
        .expect("second dispatch");

    let events = emitter.snapshot();
    assert_eq!(events.len(), 2);
    for evt in &events {
        assert!(matches!(evt, AgentEvent::CapabilityGrant { .. }));
    }
}
