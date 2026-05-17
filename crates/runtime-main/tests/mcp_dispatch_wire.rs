//! M06.D — the runtime-main side of ADR-0010: the
//! `McpDispatchOutcome` → `AgentEvent` mapping the SDK run loop applies
//! after the injected `Arc<dyn McpToolDispatch>` returns. The concrete
//! dispatcher's resolve/check/invoke/audit behavior is tested in
//! `runtime-mcp`; this pins the wire that turns its outcome into the
//! events the renderer consumes.

use runtime_core::event::{AgentEvent, ToolSource};
use runtime_main::sdk::{
    apply_mcp_dispatch, mcp_dispatch_error_event, outcome_needs_hitl, McpDispatchError,
    McpDispatchOutcome,
};
use serde_json::json;

#[test]
fn invoked_outcome_maps_to_tool_invoked_then_tool_result_with_mcp_source() {
    let events = apply_mcp_dispatch(
        McpDispatchOutcome::Invoked {
            server: "pdf-mcp".to_string(),
            tool: "extract_text".to_string(),
            value: json!({"text": "hi"}),
        },
        json!({"q": "needle"}),
    );
    assert_eq!(events.len(), 2, "Invoked → ToolInvoked + ToolResult");
    match &events[0] {
        AgentEvent::ToolInvoked {
            tool_name,
            source,
            server,
            input,
            ..
        } => {
            assert_eq!(tool_name, "extract_text");
            assert_eq!(*source, ToolSource::Mcp);
            assert_eq!(server.as_deref(), Some("pdf-mcp"));
            assert_eq!(
                *input,
                json!({"q": "needle"}),
                "original args ride into ToolInvoked"
            );
        }
        other => panic!("events[0] must be ToolInvoked, got {other:?}"),
    }
    match &events[1] {
        AgentEvent::ToolResult {
            tool_name, output, ..
        } => {
            assert_eq!(tool_name, "extract_text");
            assert_eq!(*output, json!({"text": "hi"}));
        }
        other => panic!("events[1] must be ToolResult, got {other:?}"),
    }
}

#[test]
fn blocked_outcome_maps_to_capability_violation_then_mcp_request_blocked() {
    let events = apply_mcp_dispatch(
        McpDispatchOutcome::Blocked {
            agent_id: "worker".to_string(),
            server: "pdf-mcp".to_string(),
            tool: "extract_text".to_string(),
            reason: "no capabilities declared".to_string(),
        },
        json!({}),
    );
    assert_eq!(
        events.len(),
        2,
        "Blocked → CapabilityViolation + McpRequestBlocked (single deny, two events)"
    );
    match &events[0] {
        AgentEvent::CapabilityViolation { agent_id, .. } => {
            assert_eq!(agent_id, "worker");
        }
        other => panic!("events[0] must be CapabilityViolation, got {other:?}"),
    }
    match &events[1] {
        AgentEvent::McpRequestBlocked {
            agent_id,
            server,
            tool,
            reason,
        } => {
            assert_eq!(agent_id, "worker");
            assert_eq!(server, "pdf-mcp");
            assert_eq!(tool, "extract_text");
            assert_eq!(reason, "no capabilities declared");
        }
        other => panic!("events[1] must be McpRequestBlocked, got {other:?}"),
    }
}

#[test]
fn ambiguous_outcome_maps_to_tool_alias_ambiguous() {
    let events = apply_mcp_dispatch(
        McpDispatchOutcome::Ambiguous {
            name: "extract_text".to_string(),
            candidates: vec![
                "pdf-mcp__extract_text".to_string(),
                "image-mcp__extract_text".to_string(),
            ],
        },
        json!({}),
    );
    assert_eq!(events.len(), 1);
    match &events[0] {
        AgentEvent::ToolAliasAmbiguous { name, candidates } => {
            assert_eq!(name, "extract_text");
            assert_eq!(candidates.len(), 2);
        }
        other => panic!("expected ToolAliasAmbiguous, got {other:?}"),
    }
}

#[test]
fn only_blocked_outcome_needs_hitl() {
    assert!(outcome_needs_hitl(&McpDispatchOutcome::Blocked {
        agent_id: "w".into(),
        server: "s".into(),
        tool: "t".into(),
        reason: "r".into(),
    }));
    assert!(!outcome_needs_hitl(&McpDispatchOutcome::Invoked {
        server: "s".into(),
        tool: "t".into(),
        value: json!(null),
    }));
    assert!(!outcome_needs_hitl(&McpDispatchOutcome::Ambiguous {
        name: "n".into(),
        candidates: vec!["a".into(), "b".into()],
    }));
}

#[test]
fn dispatch_error_maps_to_tool_error_event() {
    let ev = mcp_dispatch_error_event(
        "worker",
        "pdf-mcp__extract_text",
        &McpDispatchError::Transport("connection refused".to_string()),
    );
    match ev {
        AgentEvent::ToolError {
            agent_id,
            tool_name,
            error,
        } => {
            assert_eq!(agent_id, "worker");
            assert_eq!(tool_name, "pdf-mcp__extract_text");
            assert!(
                error.contains("connection refused"),
                "error text must carry the transport cause, got {error}"
            );
        }
        other => panic!("expected ToolError, got {other:?}"),
    }
}
