//! Table-driven `ProviderEvent` → `AgentEvent` translation tests.
//!
//! Per `docs/build-prompts/M02-event-pipeline.md` §D.4 — every transition
//! including consecutive-`TextDelta` bundling, decision-pattern extraction,
//! tool-use boundary flushing, error-path translation, and a proptest
//! "no input sequence panics."
//!
//! These tests bypass the network and the drone IPC; they exercise
//! `EventPipeline` directly so the translation is verified in isolation.

use proptest::prelude::*;
use runtime_core::event::{AgentEvent, ToolSource};
use runtime_main::providers::ProviderEvent;
use runtime_main::sdk::EventPipeline;

const AGENT: &str = "agent_test";

fn pipe() -> EventPipeline {
    EventPipeline::new(AGENT.to_string())
}

fn run(events: Vec<ProviderEvent>) -> Vec<AgentEvent> {
    let mut p = pipe();
    let mut out = Vec::new();
    for e in events {
        out.extend(p.next_event(e));
    }
    out.extend(p.flush());
    out
}

// ── Bundling ────────────────────────────────────────────────────────────

#[test]
fn lone_text_then_message_stop_flushes_one_stream_text() {
    let out = run(vec![
        ProviderEvent::TextDelta { text: "hi".into() },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    let text_count = out
        .iter()
        .filter(|e| matches!(e, AgentEvent::StreamText { .. }))
        .count();
    assert_eq!(text_count, 1, "got: {out:?}");
    assert!(matches!(out.last(), Some(AgentEvent::AgentComplete { .. })));
}

#[test]
fn multiple_text_deltas_bundle_to_one_stream_text() {
    let out = run(vec![
        ProviderEvent::TextDelta { text: "hel".into() },
        ProviderEvent::TextDelta { text: "lo ".into() },
        ProviderEvent::TextDelta {
            text: "world".into(),
        },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    let stream_texts: Vec<&str> = out
        .iter()
        .filter_map(|e| match e {
            AgentEvent::StreamText { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(stream_texts, ["hello world"], "events: {out:?}");
}

#[test]
fn text_then_tool_then_text_flushes_at_each_boundary() {
    let out = run(vec![
        ProviderEvent::TextDelta {
            text: "before".into(),
        },
        ProviderEvent::ToolUse {
            id: "tu_1".into(),
            name: "search".into(),
            input: serde_json::json!({"q": "rust"}),
        },
        ProviderEvent::TextDelta {
            text: "after".into(),
        },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    // Sequence: StreamText("before"), ToolInvoked, StreamText("after"), AgentComplete.
    let kinds: Vec<&str> = out
        .iter()
        .map(|e| match e {
            AgentEvent::StreamText { .. } => "text",
            AgentEvent::ToolInvoked { .. } => "tool",
            AgentEvent::AgentComplete { .. } => "complete",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, vec!["text", "tool", "text", "complete"]);
}

#[test]
fn tool_use_first_no_leading_stream_text() {
    let out = run(vec![
        ProviderEvent::ToolUse {
            id: "tu_1".into(),
            name: "search".into(),
            input: serde_json::json!({"q": "rust"}),
        },
        ProviderEvent::MessageStop {
            stop_reason: "tool_use".into(),
        },
    ]);
    assert!(matches!(out.first(), Some(AgentEvent::ToolInvoked { .. })));
    assert!(!out
        .iter()
        .any(|e| matches!(e, AgentEvent::StreamText { .. })));
}

#[test]
fn empty_stream_yields_nothing() {
    assert!(run(vec![]).is_empty());
}

// ── Variant routing ─────────────────────────────────────────────────────

#[test]
fn thinking_delta_emits_stream_text_and_flushes_buffer() {
    let out = run(vec![
        ProviderEvent::TextDelta { text: "x".into() },
        ProviderEvent::ThinkingDelta {
            text: "step".into(),
        },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    // x is flushed (because thinking is a non-text boundary), then the
    // thinking text is emitted, then the message-stop fires AgentComplete.
    let kinds: Vec<&str> = out
        .iter()
        .map(|e| match e {
            AgentEvent::StreamText { .. } => "text",
            AgentEvent::AgentComplete { .. } => "complete",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, vec!["text", "text", "complete"]);
}

#[test]
fn error_event_emits_agent_error_and_flushes_buffer() {
    let out = run(vec![
        ProviderEvent::TextDelta { text: "buf".into() },
        ProviderEvent::Error {
            code: "overloaded".into(),
            message: "slow down".into(),
        },
    ]);
    assert!(out
        .iter()
        .any(|e| matches!(e, AgentEvent::StreamText { .. })));
    let err = out
        .iter()
        .find_map(|e| match e {
            AgentEvent::AgentError { error, .. } => Some(error.clone()),
            _ => None,
        })
        .expect("AgentError emitted");
    assert!(err.contains("overloaded"));
    assert!(err.contains("slow down"));
}

#[test]
fn tool_result_passthrough_emits_tool_result() {
    let out = run(vec![ProviderEvent::ToolResult {
        id: "tu_1".into(),
        output: serde_json::json!({"ok": true}),
    }]);
    let result = out.iter().find_map(|e| match e {
        AgentEvent::ToolResult { output, .. } => Some(output.clone()),
        _ => None,
    });
    assert_eq!(result, Some(serde_json::json!({"ok": true})));
}

#[test]
fn message_stop_carries_stop_reason_into_result() {
    let out = run(vec![ProviderEvent::MessageStop {
        stop_reason: "max_tokens".into(),
    }]);
    let result = out.iter().find_map(|e| match e {
        AgentEvent::AgentComplete { result, .. } => Some(result.clone()),
        _ => None,
    });
    assert_eq!(result, Some("max_tokens".into()));
}

// ── Tool-invoked source defaulting ──────────────────────────────────────

#[test]
fn tool_invoked_carries_builtin_source_default_at_m02() {
    let out = run(vec![ProviderEvent::ToolUse {
        id: "tu_1".into(),
        name: "search".into(),
        input: serde_json::json!({"q": "x"}),
    }]);
    let source = out.iter().find_map(|e| match e {
        AgentEvent::ToolInvoked { source, .. } => Some(source.clone()),
        _ => None,
    });
    // M02 defaults to Builtin; M06 refines based on the tool registry.
    assert_eq!(source, Some(ToolSource::Builtin));
}

// ── Decision extraction ─────────────────────────────────────────────────

#[test]
fn decision_pattern_in_text_emits_decision_record() {
    let text = "Decision: pick haiku\nRationale: cost-sensitive task";
    let out = run(vec![
        ProviderEvent::TextDelta { text: text.into() },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    // Decision extraction emits BOTH DecisionRecord and StreamText (the raw
    // text is always preserved; the decision is a parallel structured signal).
    let dr = out
        .iter()
        .find_map(|e| match e {
            AgentEvent::DecisionRecord {
                decision,
                rationale,
                ..
            } => Some((decision.clone(), rationale.clone())),
            _ => None,
        })
        .expect("DecisionRecord emitted");
    assert_eq!(dr.0, "pick haiku");
    assert_eq!(dr.1, "cost-sensitive task");
    assert!(out
        .iter()
        .any(|e| matches!(e, AgentEvent::StreamText { .. })));
}

#[test]
fn no_decision_pattern_no_decision_record() {
    let out = run(vec![
        ProviderEvent::TextDelta {
            text: "just plain text\n".into(),
        },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    assert!(!out
        .iter()
        .any(|e| matches!(e, AgentEvent::DecisionRecord { .. })));
}

// ── Multi-tool sequencing ───────────────────────────────────────────────

#[test]
fn consecutive_tool_uses_emit_multiple_tool_invoked_no_spurious_text() {
    let out = run(vec![
        ProviderEvent::ToolUse {
            id: "tu_1".into(),
            name: "search".into(),
            input: serde_json::json!({}),
        },
        ProviderEvent::ToolUse {
            id: "tu_2".into(),
            name: "fetch".into(),
            input: serde_json::json!({}),
        },
        ProviderEvent::MessageStop {
            stop_reason: "tool_use".into(),
        },
    ]);
    let tool_count = out
        .iter()
        .filter(|e| matches!(e, AgentEvent::ToolInvoked { .. }))
        .count();
    assert_eq!(tool_count, 2);
    assert!(!out
        .iter()
        .any(|e| matches!(e, AgentEvent::StreamText { .. })));
}

// ── Buffer flushing ─────────────────────────────────────────────────────

#[test]
fn buffer_flushed_on_explicit_flush_call() {
    let mut p = pipe();
    let pre = p.next_event(ProviderEvent::TextDelta { text: "x".into() });
    assert!(pre.is_empty(), "buffered until boundary");
    let drained = p.flush();
    assert!(drained
        .iter()
        .any(|e| matches!(e, AgentEvent::StreamText { .. })));
}

#[test]
fn flush_after_drain_emits_nothing() {
    let mut p = pipe();
    p.next_event(ProviderEvent::TextDelta { text: "x".into() });
    p.flush();
    assert!(p.flush().is_empty());
}

// ── Bundling preserves text content exactly ─────────────────────────────

#[test]
fn bundled_stream_text_preserves_concatenation_order() {
    let out = run(vec![
        ProviderEvent::TextDelta { text: "1".into() },
        ProviderEvent::TextDelta { text: "2".into() },
        ProviderEvent::TextDelta { text: "3".into() },
        ProviderEvent::TextDelta { text: "4".into() },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    let combined = out.iter().find_map(|e| match e {
        AgentEvent::StreamText { text, .. } => Some(text.clone()),
        _ => None,
    });
    assert_eq!(combined, Some("1234".into()));
}

#[test]
fn agent_id_propagates_to_every_event() {
    let out = run(vec![
        ProviderEvent::TextDelta { text: "hi".into() },
        ProviderEvent::ToolUse {
            id: "tu_1".into(),
            name: "n".into(),
            input: serde_json::json!({}),
        },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    for e in &out {
        let id = match e {
            AgentEvent::StreamText { agent_id, .. }
            | AgentEvent::ToolInvoked { agent_id, .. }
            | AgentEvent::AgentComplete { agent_id, .. } => Some(agent_id.as_str()),
            _ => None,
        };
        assert_eq!(id, Some(AGENT));
    }
}

// ── Edge cases ──────────────────────────────────────────────────────────

#[test]
fn empty_text_delta_does_not_emit_empty_stream_text() {
    let out = run(vec![
        ProviderEvent::TextDelta {
            text: String::new(),
        },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    // Empty buffer flush must not emit a zero-length StreamText.
    assert!(!out
        .iter()
        .any(|e| matches!(e, AgentEvent::StreamText { text, .. } if text.is_empty())));
}

#[test]
fn error_after_tool_use_still_flushes_clean() {
    let out = run(vec![
        ProviderEvent::ToolUse {
            id: "tu_1".into(),
            name: "n".into(),
            input: serde_json::json!({}),
        },
        ProviderEvent::Error {
            code: "x".into(),
            message: "y".into(),
        },
    ]);
    assert!(out
        .iter()
        .any(|e| matches!(e, AgentEvent::ToolInvoked { .. })));
    assert!(out
        .iter()
        .any(|e| matches!(e, AgentEvent::AgentError { .. })));
}

#[test]
fn message_stop_after_message_stop_emits_two_agent_completes() {
    // Defensive: provider should not normally re-emit MessageStop, but if a
    // pathological stream does, the pipeline does not panic and emits the
    // mapped AgentComplete each time.
    let out = run(vec![
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
        },
    ]);
    let count = out
        .iter()
        .filter(|e| matches!(e, AgentEvent::AgentComplete { .. }))
        .count();
    assert_eq!(count, 2);
}

// ── Property test ───────────────────────────────────────────────────────

fn arb_provider_event() -> impl Strategy<Value = ProviderEvent> {
    prop_oneof![
        any::<String>().prop_map(|text| ProviderEvent::TextDelta { text }),
        any::<String>().prop_map(|text| ProviderEvent::ThinkingDelta { text }),
        (any::<String>(), any::<String>()).prop_map(|(id, name)| ProviderEvent::ToolUse {
            id,
            name,
            input: serde_json::json!({}),
        }),
        any::<String>().prop_map(|id| ProviderEvent::ToolResult {
            id,
            output: serde_json::json!({"ok": true}),
        }),
        any::<String>().prop_map(|stop_reason| ProviderEvent::MessageStop { stop_reason }),
        (any::<String>(), any::<String>())
            .prop_map(|(code, message)| ProviderEvent::Error { code, message }),
    ]
}

proptest! {
    #[test]
    fn arbitrary_event_sequence_never_panics(events in proptest::collection::vec(arb_provider_event(), 0..20)) {
        let _ = run(events);
    }
}
