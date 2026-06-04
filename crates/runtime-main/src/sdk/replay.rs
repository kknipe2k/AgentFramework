//! Replay: signal-log JSON → `AgentEvent` stream.
//!
//! Inverse of M02.D's [`crate::sdk::EventPipeline`]. Used by the Stage E
//! replay path: the renderer mounts with a known `session_id` (or, after
//! a restart that wiped the renderer's `lastSessionId`, the most-recent
//! persisted session — TD-044); main asks drone for the session's signals
//! (`ReadSignals`); drone returns them as JSON; main runs them through
//! this translator and re-emits each event via the existing Tauri
//! `agent_event` channel.
//!
//! Each signal row's `payload_json` IS a fully serialized [`AgentEvent`]:
//! the SDK's emit path persists `serde_json::to_value(&event)`, and
//! `AgentEvent` is internally tagged (`#[serde(tag = "type")]`) with
//! `#[serde(default)]` on its optional fields. Replay therefore round-trips
//! the payload straight back through serde rather than re-deriving each
//! variant by hand — the previous hand-rolled, field-by-field translator
//! read invented field names (`payload_json.event`) that the real signal
//! log never carried, so it silently produced ZERO events against real
//! data (TD-044: the reload→reconstruct chain never reconstructed).
//!
//! Pure function: no I/O. The caller bounds the input length (per
//! `docs/gotchas.md` #28). Idempotent at the renderer side:
//! `graphStore.applyEvent`'s idempotence property (Stage B/C tested) means
//! re-emitting the same events produces the same final graph state.

use runtime_core::event::AgentEvent;
use serde_json::Value;

/// Translate a slice of signal-shaped JSON values into `AgentEvent`s.
///
/// A signal whose `payload_json` is not a v0.1 `AgentEvent` — a
/// projector-only signal such as `token_usage`, a row with no payload, or
/// a legacy shape — fails to deserialize and is filtered silently rather
/// than panicked, so a partially-malformed or mixed log reconstructs the
/// parts it can.
#[must_use]
pub fn replay_signals_to_events(signals: &[Value]) -> Vec<AgentEvent> {
    signals.iter().filter_map(signal_to_event).collect()
}

fn signal_to_event(signal: &Value) -> Option<AgentEvent> {
    let payload = signal.get("payload_json")?;
    serde_json::from_value::<AgentEvent>(payload.clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::event::{AgentEvent, ToolSource};
    use serde_json::json;

    /// Wrap a `payload_json` value as the signal-row shape the drone's
    /// `ReadSignals` returns (`vdr::signals_for_session` parses the
    /// `payload_json` column into a JSON object).
    fn signal(payload: Value) -> Value {
        let mut row = serde_json::Map::new();
        row.insert("payload_json".into(), payload);
        Value::Object(row)
    }

    #[test]
    fn signal_with_no_payload_returns_none() {
        assert!(signal_to_event(&json!({})).is_none());
    }

    #[test]
    fn null_payload_returns_none() {
        assert!(signal_to_event(&json!({ "payload_json": null })).is_none());
    }

    #[test]
    fn non_agent_event_payload_is_filtered() {
        // A `token_usage` projector signal is not a v0.1 `AgentEvent`; it
        // must drop out of replay rather than crash it. (This row shape is
        // exactly what coexists with agent signals in a real session log —
        // observed IRL alongside agent_spawned/stream_text/agent_complete.)
        assert!(signal_to_event(&signal(json!({ "kind": "token_usage", "tokens": 5 }))).is_none());
    }

    #[test]
    fn real_agent_spawned_payload_round_trips() {
        // The EXACT on-disk shape the SDK emit path persists (dumped IRL
        // from the live session.sqlite during TD-044 triage): payload_json
        // is the serialized AgentEvent, internally tagged on `type`, with
        // `narrowed_from` skipped (it serializes only when non-empty).
        let s = signal(json!({
            "type": "agent_spawned",
            "agent_id": "agent_ed73c4c9",
            "agent_name": "smoke",
            "parent_id": null,
            "session_id": "88802c9f",
        }));
        match signal_to_event(&s).expect("translate") {
            AgentEvent::AgentSpawned {
                agent_id,
                agent_name,
                parent_id,
                narrowed_from,
                ..
            } => {
                assert_eq!(agent_id, "agent_ed73c4c9");
                assert_eq!(agent_name, "smoke");
                assert_eq!(parent_id, None);
                assert!(
                    narrowed_from.is_empty(),
                    "a skipped narrowed_from must default to empty"
                );
            }
            other => panic!("expected AgentSpawned, got {other:?}"),
        }
    }

    #[test]
    fn real_agent_complete_payload_round_trips() {
        // Real on-disk shape (dumped IRL): no `event` field — the variant
        // is tagged on `type`, and tokens_total rides as a bare number.
        let s = signal(json!({
            "type": "agent_complete",
            "agent_id": "agent_ed73c4c9",
            "result": "end_turn",
            "tokens_total": 34,
        }));
        match signal_to_event(&s).expect("translate") {
            AgentEvent::AgentComplete {
                agent_id,
                result,
                tokens_total,
            } => {
                assert_eq!(agent_id, "agent_ed73c4c9");
                assert_eq!(result, "end_turn");
                assert_eq!(tokens_total, Some(34));
            }
            other => panic!("expected AgentComplete, got {other:?}"),
        }
    }

    #[test]
    fn tool_invoked_round_trips_via_serde() {
        // Serialize a real AgentEvent → wrap as a signal → translate back.
        // Pins "payload_json IS a serialized AgentEvent" by construction
        // (behavior, not a fabricated field shape) — the §5 fix for the
        // tautology tests this rewrite replaces.
        let original = AgentEvent::ToolInvoked {
            agent_id: "a1".into(),
            tool_name: "search".into(),
            source: ToolSource::Mcp,
            server: Some("github".into()),
            input: json!({ "q": "x" }),
        };
        let payload = serde_json::to_value(&original).expect("serialize");
        match signal_to_event(&signal(payload)).expect("translate") {
            AgentEvent::ToolInvoked {
                source,
                server,
                tool_name,
                ..
            } => {
                assert!(matches!(source, ToolSource::Mcp));
                assert_eq!(server.as_deref(), Some("github"));
                assert_eq!(tool_name, "search");
            }
            other => panic!("expected ToolInvoked, got {other:?}"),
        }
    }

    #[test]
    fn replay_round_trips_a_serialized_event_slice() {
        // The end-to-end translator contract guarding reload→reconstruct
        // (TD-044): any serialized AgentEvent slice replays to the
        // identical events, in order. The smoke session's real signal
        // sequence (session_start → agent_spawned → agent_complete) is the
        // minimum the reload leg's `[data-testid^=agent-node-]` assertion
        // depends on.
        let events = vec![
            AgentEvent::SessionStart {
                session_id: "s1".into(),
                framework: "aria".into(),
                model: "haiku".into(),
            },
            AgentEvent::AgentSpawned {
                agent_id: "a1".into(),
                agent_name: "smoke".into(),
                parent_id: None,
                session_id: "s1".into(),
                narrowed_from: Vec::new(),
            },
            AgentEvent::AgentComplete {
                agent_id: "a1".into(),
                result: "ok".into(),
                tokens_total: Some(10),
            },
        ];
        let signals: Vec<Value> = events
            .iter()
            .map(|e| signal(serde_json::to_value(e).expect("serialize")))
            .collect();
        assert_eq!(
            replay_signals_to_events(&signals),
            events,
            "serialized AgentEvents must replay back identically"
        );
    }

    #[test]
    fn unknown_and_valid_signals_interleave_cleanly() {
        let signals = vec![
            signal(json!({ "kind": "token_usage", "tokens": 1 })), // filtered
            signal(json!({
                "type": "agent_spawned",
                "agent_id": "a1", "agent_name": "n", "parent_id": null, "session_id": "s1",
            })),
        ];
        let replayed = replay_signals_to_events(&signals);
        assert_eq!(replayed.len(), 1, "the non-event signal must be filtered");
        assert!(matches!(replayed[0], AgentEvent::AgentSpawned { .. }));
    }
}
