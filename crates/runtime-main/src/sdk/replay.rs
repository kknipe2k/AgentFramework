//! Replay: signal-log JSON → `AgentEvent` stream.
//!
//! Inverse of M02.D's [`crate::sdk::EventPipeline`]. Used by the Stage E
//! replay path: the renderer mounts with a known `session_id`; main asks
//! drone for the session's signals (`ReadSignals`); drone returns them
//! as JSON; main runs them through this translator and re-emits each
//! event via the existing Tauri `agent_event` channel.
//!
//! Pure function: no I/O, no allocation outside the result `Vec`. The
//! caller bounds the input length (per `docs/gotchas.md` #28).
//!
//! Idempotent at the renderer side: `graphStore.applyEvent`'s
//! idempotence property (Stage B/C tested) means re-emitting the same
//! events produces the same final graph state.

use runtime_core::event::{AgentEvent, ToolSource};
use serde_json::Value;

/// Translate a slice of signal-shaped JSON values into `AgentEvent`s.
///
/// Signals that don't map to a v0.1 `AgentEvent` variant — or that lack
/// required fields — are filtered silently rather than panicked, so a
/// partially-malformed log reconstructs the parts it can.
#[must_use]
pub fn replay_signals_to_events(signals: &[Value]) -> Vec<AgentEvent> {
    signals.iter().filter_map(signal_to_event).collect()
}

fn signal_to_event(signal: &Value) -> Option<AgentEvent> {
    let sig_type = signal.get("type").and_then(Value::as_str)?;
    let payload = signal.get("payload_json")?;
    match sig_type {
        "agent" => translate_agent(payload),
        "tool" => translate_tool(payload),
        "skill" => translate_skill(payload),
        "decision" => translate_decision(payload),
        "session" => translate_session(payload),
        _ => None,
    }
}

fn translate_agent(payload: &Value) -> Option<AgentEvent> {
    let event = payload.get("event").and_then(Value::as_str)?;
    let agent_id = payload.get("agent_id").and_then(Value::as_str)?.to_string();
    match event {
        "spawned" => {
            let agent_name = payload
                .get("agent_name")
                .and_then(Value::as_str)
                .unwrap_or("agent")
                .to_string();
            let session_id = payload
                .get("session_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let parent_id = payload
                .get("parent_id")
                .and_then(Value::as_str)
                .map(String::from);
            Some(AgentEvent::AgentSpawned {
                agent_id,
                agent_name,
                parent_id,
                session_id,
            })
        }
        "complete" => {
            let result = payload
                .get("result")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let tokens_total = payload.get("tokens_total").and_then(Value::as_u64);
            Some(AgentEvent::AgentComplete {
                agent_id,
                result,
                tokens_total,
            })
        }
        "error" => {
            let error = payload
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            Some(AgentEvent::AgentError { agent_id, error })
        }
        _ => None,
    }
}

fn translate_tool(payload: &Value) -> Option<AgentEvent> {
    let agent_id = payload.get("agent_id").and_then(Value::as_str)?.to_string();
    let tool_name = payload
        .get("tool_name")
        .and_then(Value::as_str)?
        .to_string();
    let source_str = payload
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("builtin");
    let source = match source_str {
        "mcp" => ToolSource::Mcp,
        "generated" => ToolSource::Generated,
        _ => ToolSource::Builtin,
    };
    let server = payload
        .get("server")
        .and_then(Value::as_str)
        .map(String::from);
    let input = payload.get("input").cloned().unwrap_or(Value::Null);
    Some(AgentEvent::ToolInvoked {
        agent_id,
        tool_name,
        source,
        server,
        input,
    })
}

fn translate_skill(payload: &Value) -> Option<AgentEvent> {
    let agent_id = payload.get("agent_id").and_then(Value::as_str)?.to_string();
    let skill_name = payload
        .get("skill_name")
        .and_then(Value::as_str)?
        .to_string();
    let mode = payload
        .get("mode")
        .and_then(Value::as_str)
        .map(String::from);
    Some(AgentEvent::SkillLoaded {
        agent_id,
        skill_name,
        mode,
    })
}

fn translate_decision(payload: &Value) -> Option<AgentEvent> {
    let agent_id = payload.get("agent_id").and_then(Value::as_str)?.to_string();
    let decision = payload.get("decision").and_then(Value::as_str)?.to_string();
    let rationale = payload
        .get("rationale")
        .and_then(Value::as_str)?
        .to_string();
    let tool_used = payload
        .get("tool_used")
        .and_then(Value::as_str)
        .map(String::from);
    Some(AgentEvent::DecisionRecord {
        agent_id,
        decision,
        rationale,
        tool_used,
    })
}

fn translate_session(payload: &Value) -> Option<AgentEvent> {
    let event = payload.get("event").and_then(Value::as_str)?;
    if event != "start" {
        // session_end is not in the v0.1 AgentEvent union; ignore
        // gracefully so partial logs don't crash replay.
        return None;
    }
    let session_id = payload
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let framework = payload
        .get("framework")
        .and_then(Value::as_str)
        .unwrap_or("aria")
        .to_string();
    let model = payload
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Some(AgentEvent::SessionStart {
        session_id,
        framework,
        model,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn signal_with_no_type_returns_none() {
        assert!(signal_to_event(&json!({})).is_none());
    }

    #[test]
    fn signal_with_no_payload_returns_none() {
        assert!(signal_to_event(&json!({"type": "agent"})).is_none());
    }

    #[test]
    fn agent_spawned_with_explicit_parent_id_round_trips() {
        let s = json!({
            "type": "agent",
            "payload_json": {
                "event": "spawned",
                "agent_id": "child",
                "agent_name": "n",
                "session_id": "s1",
                "parent_id": "root",
            }
        });
        let evt = signal_to_event(&s).expect("translate");
        if let AgentEvent::AgentSpawned { parent_id, .. } = evt {
            assert_eq!(parent_id.as_deref(), Some("root"));
        } else {
            panic!("expected AgentSpawned");
        }
    }

    #[test]
    fn tool_invoked_with_mcp_source_recovers_server() {
        let s = json!({
            "type": "tool",
            "payload_json": {
                "agent_id": "a1",
                "tool_name": "search",
                "source": "mcp",
                "server": "github",
                "input": {"q": "x"},
            }
        });
        let evt = signal_to_event(&s).expect("translate");
        if let AgentEvent::ToolInvoked { source, server, .. } = evt {
            assert!(matches!(source, ToolSource::Mcp));
            assert_eq!(server.as_deref(), Some("github"));
        } else {
            panic!("expected ToolInvoked");
        }
    }

    #[test]
    fn agent_complete_carries_tokens_total_when_present() {
        let s = json!({
            "type": "agent",
            "payload_json": {
                "event": "complete",
                "agent_id": "a1",
                "result": "done",
                "tokens_total": 1234,
            }
        });
        let evt = signal_to_event(&s).expect("translate");
        if let AgentEvent::AgentComplete { tokens_total, .. } = evt {
            assert_eq!(tokens_total, Some(1234));
        } else {
            panic!("expected AgentComplete");
        }
    }

    #[test]
    fn agent_error_translates_with_empty_error_default() {
        let s = json!({
            "type": "agent",
            "payload_json": {"event": "error", "agent_id": "a1"}
        });
        let evt = signal_to_event(&s).expect("translate");
        if let AgentEvent::AgentError { agent_id, error } = evt {
            assert_eq!(agent_id, "a1");
            assert_eq!(error, "");
        } else {
            panic!("expected AgentError");
        }
    }

    #[test]
    fn session_end_event_is_filtered() {
        let s = json!({
            "type": "session",
            "payload_json": {"event": "end"}
        });
        assert!(signal_to_event(&s).is_none());
    }
}
