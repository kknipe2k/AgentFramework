//! `request_capability` meta-tool — spec §4b Layer 2.
//!
//! Agent-visible meta-tool the LLM invokes when it realizes mid-task that
//! it needs a tool / skill / MCP server / sub-agent it doesn't have. The
//! SDK does NOT route this to an LLM tool call: it intercepts the
//! invocation inline, emits the appropriate `*_missing` event with
//! `requested_via: request_capability` + `severity: requested`, and
//! returns [`RequestCapabilityResult::Pending`] to the caller. The HITL
//! seam (`on_gap` trigger, wired in M04 Stage E `crates/runtime-main/src/hitl/`)
//! routes the gap event to the user; the SDK turn loop awaits resolution
//! before resuming.
//!
//! Spec §4b text restricts `capability_kind` to `tool | skill`; M05.A
//! extends to four kinds (`tool | skill | mcp | agent`) to match the
//! schema's `*_missing` variant set. Divergence surfaced in M05.A
//! retrospective for maintainer decision (analogous to M04.V Decision 2).

use runtime_core::event::{AgentEvent, GapSeverityRef, GapSourceRef};
use thiserror::Error;

use crate::framework_loader::Emitter;

/// One of the four primitive kinds an agent can request mid-session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityKind {
    /// Callable primitive (built-in or MCP-served).
    Tool,
    /// Instructional context primitive — loaded into the agent's context.
    Skill,
    /// MCP server (v0.1 user installs externally; M06 adds in-runtime
    /// install flow).
    Mcp,
    /// Sub-agent to spawn (v0.1 requires the agent to be declared in the
    /// framework JSON's `agents[]`).
    Agent,
}

/// Structured invocation the agent passes when calling `request_capability`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestCapabilityInvocation {
    /// Agent making the request.
    pub agent_id: String,
    /// Which kind of capability is being requested.
    pub kind: CapabilityKind,
    /// Best-effort name of the missing primitive (the agent's guess).
    pub name: String,
    /// Free-text rationale ("Why do I need this?"). Surfaced verbatim in
    /// the HITL prompt + `GapPanel` UI so the user can decide whether to
    /// grant / install / decline.
    pub justification: String,
}

/// Result of the meta-tool dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestCapabilityResult {
    /// Gap event emitted; SDK turn loop should await the HITL resolution
    /// before resuming the agent's tool result. This is the only v0.1
    /// outcome — the meta-tool is non-blocking on the dispatch side and
    /// blocks at the awaiter.
    Pending,
}

/// Errors raised by [`handle_request_capability`].
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum RequestCapabilityError {
    /// The provided `name` was empty. The schema's `SuggestedAction`
    /// requires `minLength: 1`; the meta-tool composes the suggested
    /// action from name + justification, so an empty name would produce
    /// an unconstructable event.
    #[error("request_capability invocation has empty `name`")]
    EmptyName,
    /// The `justification` was empty. v0.1 makes justification mandatory
    /// per spec §4b "Agent's stated reason shown" — the renderer needs
    /// it to surface the gap in the HITL prompt.
    #[error("request_capability invocation has empty `justification`")]
    EmptyJustification,
}

/// Dispatch a `request_capability` invocation: emit the appropriate
/// `*_missing` event via `emitter` and return
/// [`RequestCapabilityResult::Pending`].
///
/// # Errors
///
/// - [`RequestCapabilityError::EmptyName`] if `invocation.name` is empty.
/// - [`RequestCapabilityError::EmptyJustification`] if
///   `invocation.justification` is empty.
pub async fn handle_request_capability(
    invocation: RequestCapabilityInvocation,
    emitter: &impl Emitter,
) -> Result<RequestCapabilityResult, RequestCapabilityError> {
    if invocation.name.is_empty() {
        return Err(RequestCapabilityError::EmptyName);
    }
    if invocation.justification.is_empty() {
        return Err(RequestCapabilityError::EmptyJustification);
    }

    let suggested_text = format!(
        "Agent '{agent}' requested {kind_label} '{name}': {why}",
        agent = invocation.agent_id,
        kind_label = match invocation.kind {
            CapabilityKind::Tool => "tool",
            CapabilityKind::Skill => "skill",
            CapabilityKind::Mcp => "mcp server",
            CapabilityKind::Agent => "sub-agent",
        },
        name = invocation.name,
        why = invocation.justification,
    );
    let event = match invocation.kind {
        CapabilityKind::Tool => AgentEvent::ToolMissing {
            agent_id: invocation.agent_id,
            tool_name: invocation.name,
            severity: GapSeverityRef::Requested,
            suggested_action: suggested_text,
            requested_via: GapSourceRef::RequestCapability,
        },
        CapabilityKind::Skill => AgentEvent::SkillMissing {
            agent_id: invocation.agent_id,
            skill_name: invocation.name,
            severity: GapSeverityRef::Requested,
            suggested_action: suggested_text,
            requested_via: GapSourceRef::RequestCapability,
        },
        CapabilityKind::Mcp => AgentEvent::McpMissing {
            agent_id: invocation.agent_id,
            server_name: invocation.name,
            severity: GapSeverityRef::Requested,
            suggested_action: suggested_text,
            requested_via: GapSourceRef::RequestCapability,
        },
        CapabilityKind::Agent => AgentEvent::AgentMissing {
            agent_id: invocation.agent_id,
            missing_agent_id: invocation.name,
            severity: GapSeverityRef::Requested,
            suggested_action: suggested_text,
            requested_via: GapSourceRef::RequestCapability,
        },
    };
    emitter.emit(event).await;
    Ok(RequestCapabilityResult::Pending)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct CollectingEmitter {
        events: Mutex<Vec<AgentEvent>>,
    }

    #[async_trait::async_trait]
    impl Emitter for CollectingEmitter {
        async fn emit(&self, event: AgentEvent) {
            self.events.lock().expect("no poisoning").push(event);
        }
    }

    fn invocation(kind: CapabilityKind, name: &str) -> RequestCapabilityInvocation {
        RequestCapabilityInvocation {
            agent_id: "worker".into(),
            kind,
            name: name.into(),
            justification: "needed to complete current task".into(),
        }
    }

    #[tokio::test]
    async fn tool_kind_emits_tool_missing_with_requested_via_source() {
        let emitter = CollectingEmitter::default();
        let res =
            handle_request_capability(invocation(CapabilityKind::Tool, "fetch_prs"), &emitter)
                .await
                .expect("ok");
        assert_eq!(res, RequestCapabilityResult::Pending);

        let events = emitter.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            AgentEvent::ToolMissing {
                tool_name,
                severity,
                requested_via,
                ..
            } => {
                assert_eq!(tool_name, "fetch_prs");
                assert_eq!(*severity, GapSeverityRef::Requested);
                assert_eq!(*requested_via, GapSourceRef::RequestCapability);
            }
            other => panic!("expected ToolMissing, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn all_four_kinds_routed() {
        let cases = [
            (CapabilityKind::Tool, "fetch_prs"),
            (CapabilityKind::Skill, "rag"),
            (CapabilityKind::Mcp, "pdf-mcp"),
            (CapabilityKind::Agent, "report-writer"),
        ];
        for (kind, name) in cases {
            let emitter = CollectingEmitter::default();
            handle_request_capability(invocation(kind, name), &emitter)
                .await
                .expect("ok");
            let events = emitter.events.lock().unwrap();
            assert_eq!(events.len(), 1);
            match (kind, &events[0]) {
                (CapabilityKind::Tool, AgentEvent::ToolMissing { tool_name, .. }) => {
                    assert_eq!(tool_name, name);
                }
                (CapabilityKind::Skill, AgentEvent::SkillMissing { skill_name, .. }) => {
                    assert_eq!(skill_name, name);
                }
                (CapabilityKind::Mcp, AgentEvent::McpMissing { server_name, .. }) => {
                    assert_eq!(server_name, name);
                }
                (
                    CapabilityKind::Agent,
                    AgentEvent::AgentMissing {
                        missing_agent_id, ..
                    },
                ) => {
                    assert_eq!(missing_agent_id, name);
                }
                (k, e) => panic!("kind {k:?} did not route to matching event: {e:?}"),
            }
        }
    }

    #[tokio::test]
    async fn severity_requested_for_all_kinds() {
        for kind in [
            CapabilityKind::Tool,
            CapabilityKind::Skill,
            CapabilityKind::Mcp,
            CapabilityKind::Agent,
        ] {
            let emitter = CollectingEmitter::default();
            handle_request_capability(invocation(kind, "x"), &emitter)
                .await
                .expect("ok");
            let severity = {
                let events = emitter.events.lock().unwrap();
                match &events[0] {
                    AgentEvent::ToolMissing { severity, .. }
                    | AgentEvent::SkillMissing { severity, .. }
                    | AgentEvent::McpMissing { severity, .. }
                    | AgentEvent::AgentMissing { severity, .. } => *severity,
                    other => panic!("unexpected event: {other:?}"),
                }
            };
            assert_eq!(
                severity,
                GapSeverityRef::Requested,
                "request_capability gaps carry severity=Requested per spec §4b severity matrix",
            );
        }
    }

    #[tokio::test]
    async fn empty_name_returns_err() {
        let emitter = CollectingEmitter::default();
        let mut inv = invocation(CapabilityKind::Tool, "");
        inv.name = String::new();
        let err = handle_request_capability(inv, &emitter).await.unwrap_err();
        assert_eq!(err, RequestCapabilityError::EmptyName);
        assert!(emitter.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn empty_justification_returns_err() {
        let emitter = CollectingEmitter::default();
        let mut inv = invocation(CapabilityKind::Tool, "fetch_prs");
        inv.justification = String::new();
        let err = handle_request_capability(inv, &emitter).await.unwrap_err();
        assert_eq!(err, RequestCapabilityError::EmptyJustification);
        assert!(emitter.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn suggested_action_carries_agent_id_kind_name_and_justification() {
        let emitter = CollectingEmitter::default();
        handle_request_capability(
            RequestCapabilityInvocation {
                agent_id: "planner".into(),
                kind: CapabilityKind::Skill,
                name: "rag".into(),
                justification: "needed to retrieve repo context".into(),
            },
            &emitter,
        )
        .await
        .expect("ok");

        let action = {
            let events = emitter.events.lock().unwrap();
            match &events[0] {
                AgentEvent::SkillMissing {
                    suggested_action, ..
                } => suggested_action.clone(),
                other => panic!("expected SkillMissing, got {other:?}"),
            }
        };
        assert!(action.contains("planner"), "{action}");
        assert!(action.contains("skill"), "{action}");
        assert!(action.contains("rag"), "{action}");
        assert!(action.contains("retrieve repo context"), "{action}");
    }
}
