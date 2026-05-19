//! Anthropic SSE event parsing + translation to `ProviderEvent`.
//!
//! The Anthropic Messages API emits a specific SSE event sequence on
//! `POST /v1/messages` with `stream: true`:
//!
//! ```text
//! message_start
//! → (content_block_start → content_block_delta* → content_block_stop)+
//! → message_delta → message_stop
//! ```
//!
//! plus `ping` keep-alives anywhere and `error` for server-side errors. See
//! <https://docs.anthropic.com/en/api/messages-streaming>.
//!
//! This module exposes the test seam:
//!
//! - [`parse_sse_data`] — pure JSON-string-to-[`SseEvent`] parser.
//! - [`SseState::translate`] — pure `SseEvent` → `Option<ProviderEvent>`
//!   translator with internal state for accumulating tool-input partial-JSON
//!   deltas.
//! - [`stream_events`] — `*_with`-style entry: caller injects the byte stream
//!   (real reqwest stream OR wiremock-fed bytes); function yields
//!   `ProviderEvent`s. Same translation logic exercised both ways.
//!
//! The thin production wrapper in `anthropic.rs::stream` constructs the real
//! `reqwest::Client` and feeds its byte stream into [`stream_events`]. That
//! wrapper is the OS-signal-equivalent holdout (real network is structurally
//! infeasible to test cross-platform) and is excluded from the ≥95% coverage
//! gate per the M01.C codification (commit `1dec4ba`).

use eventsource_stream::Eventsource;
use futures::stream::{Stream, StreamExt};
use serde::Deserialize;

use super::{ProviderError, ProviderEvent};

/// Anthropic SSE event types per the Messages API streaming spec.
///
/// Each event arrives as `event: <name>\ndata: <json>\n\n`. The `type` field
/// in the JSON matches the SSE event name; we deserialize from the JSON only
/// (the `event:` line is informational and `eventsource-stream` reassembles
/// the framing).
#[allow(
    dead_code,
    reason = "metadata fields (id, model, usage, stop_sequence) are part of the wire format and accepted for forward-compat; not all are surfaced as ProviderEvents at Stage C"
)]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    MessageStart {
        message: SseMessage,
    },
    ContentBlockStart {
        index: usize,
        content_block: SseContentBlockStart,
    },
    ContentBlockDelta {
        index: usize,
        delta: SseDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: SseMessageDelta,
        usage: Option<SseUsage>,
    },
    MessageStop,
    Ping,
    Error {
        error: SseError,
    },
}

#[allow(
    dead_code,
    reason = "id/model/usage are part of the wire format; not surfaced at Stage C but accepted for forward-compat"
)]
#[derive(Debug, Clone, Deserialize)]
pub struct SseMessage {
    pub id: String,
    pub model: String,
    #[serde(default)]
    pub usage: SseUsage,
}

#[allow(
    dead_code,
    reason = "ContentBlockStart variant payload fields are accepted for forward-compat; only id/name carry through to ProviderEvent::ToolUse"
)]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseContentBlockStart {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    Thinking {
        thinking: String,
    },
}

#[allow(
    clippy::enum_variant_names,
    reason = "variant names mirror the Anthropic Messages API delta-type names verbatim (text_delta, input_json_delta, thinking_delta, signature_delta); renaming would break the serde tag mapping"
)]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseDelta {
    TextDelta {
        text: String,
    },
    InputJsonDelta {
        partial_json: String,
    },
    ThinkingDelta {
        thinking: String,
    },
    SignatureDelta {
        #[allow(
            dead_code,
            reason = "signature is parsed for forward-compat but intentionally not surfaced; it's a verifier-only payload (see https://docs.anthropic.com/en/docs/build-with-claude/extended-thinking)"
        )]
        signature: String,
    },
}

#[allow(
    dead_code,
    reason = "stop_sequence is part of the wire format; not surfaced at Stage C"
)]
#[derive(Debug, Clone, Deserialize)]
pub struct SseMessageDelta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SseUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SseError {
    #[serde(rename = "type")]
    pub kind: String,
    pub message: String,
}

/// Per-stream parsing state. Tracks open content blocks so partial-JSON
/// tool-input deltas can be accumulated into a complete `ToolUse` event,
/// and accumulates `usage` data from `message_start` + `message_delta` so
/// `MessageStop` can carry a running total-token count forward.
#[derive(Debug, Default)]
pub struct SseState {
    open_blocks: std::collections::HashMap<usize, OpenBlock>,
    /// Sum of `input_tokens` + `output_tokens` seen across `message_start`
    /// and `message_delta` events. `None` until any usage data arrives.
    /// Surfaced on the `MessageStop` translation so downstream code can
    /// attach the count to `AgentEvent::AgentComplete.tokens_total`.
    cumulative_tokens: Option<u64>,
    /// Separate input/output accumulation (M07.D2). Anthropic reports
    /// them independently (`message_start.usage.input_tokens` +
    /// `message_delta.usage.output_tokens`); surfaced as
    /// [`ProviderEvent::Usage`] just before the terminal `MessageStop`
    /// so the SDK can persist a real `token_usage` row (closes M06.5).
    input_tokens: u64,
    output_tokens: u64,
    /// One buffered event (the `MessageStop` deferred behind the
    /// `Usage` it co-occurs with on the terminal `message_delta`).
    /// Drained by [`stream_events`] before the next SSE frame.
    pending: Option<ProviderEvent>,
}

#[derive(Debug)]
enum OpenBlock {
    Text,
    ToolUse {
        id: String,
        name: String,
        input_buffer: String,
    },
    Thinking,
}

impl SseState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Translate one `SseEvent` to zero-or-one `ProviderEvent`. State is
    /// mutated to accumulate tool-input partial-JSON deltas; the complete
    /// `ToolUse` is emitted on the corresponding `ContentBlockStop`. Token
    /// `usage` from `message_start` + `message_delta` is summed into
    /// `cumulative_tokens` and attached to the `MessageStop` translation.
    pub fn translate(&mut self, event: SseEvent) -> Option<ProviderEvent> {
        match event {
            SseEvent::MessageStart { message } => {
                self.add_usage(&message.usage);
                None
            }
            SseEvent::MessageStop | SseEvent::Ping => None,

            SseEvent::ContentBlockStart {
                index,
                content_block,
            } => {
                let open = match content_block {
                    SseContentBlockStart::Text { .. } => OpenBlock::Text,
                    SseContentBlockStart::ToolUse { id, name, .. } => OpenBlock::ToolUse {
                        id,
                        name,
                        input_buffer: String::new(),
                    },
                    SseContentBlockStart::Thinking { .. } => OpenBlock::Thinking,
                };
                self.open_blocks.insert(index, open);
                None
            }

            SseEvent::ContentBlockDelta { index, delta } => match delta {
                SseDelta::TextDelta { text } => Some(ProviderEvent::TextDelta { text }),
                SseDelta::ThinkingDelta { thinking } => {
                    Some(ProviderEvent::ThinkingDelta { text: thinking })
                }
                SseDelta::SignatureDelta { .. } => None,
                SseDelta::InputJsonDelta { partial_json } => {
                    if let Some(OpenBlock::ToolUse { input_buffer, .. }) =
                        self.open_blocks.get_mut(&index)
                    {
                        input_buffer.push_str(&partial_json);
                    }
                    None
                }
            },

            SseEvent::ContentBlockStop { index } => {
                let removed = self.open_blocks.remove(&index)?;
                if let OpenBlock::ToolUse {
                    id,
                    name,
                    input_buffer,
                } = removed
                {
                    let input = if input_buffer.is_empty() {
                        serde_json::Value::Object(serde_json::Map::new())
                    } else {
                        serde_json::from_str(&input_buffer)
                            .unwrap_or(serde_json::Value::String(input_buffer))
                    };
                    Some(ProviderEvent::ToolUse { id, name, input })
                } else {
                    None
                }
            }

            SseEvent::MessageDelta { delta, usage } => {
                if let Some(u) = usage.as_ref() {
                    self.add_usage(u);
                }
                delta.stop_reason.map(|stop_reason| {
                    let stop = ProviderEvent::MessageStop {
                        stop_reason,
                        total_tokens: self.cumulative_tokens,
                    };
                    // M07.D2 — when usage was reported, surface a
                    // ProviderEvent::Usage FIRST (the production
                    // token-bearing signal the drone `token_usage`
                    // projector persists), deferring MessageStop. model
                    // / cost_usd are filled by the AnthropicProvider
                    // wrapper that owns the pricing table; the SSE
                    // layer cannot price.
                    if self.input_tokens == 0 && self.output_tokens == 0 {
                        stop
                    } else {
                        self.pending = Some(stop);
                        ProviderEvent::Usage {
                            input_tokens: self.input_tokens,
                            output_tokens: self.output_tokens,
                            model: String::new(),
                            cost_usd: 0.0,
                        }
                    }
                })
            }

            SseEvent::Error { error } => Some(ProviderEvent::Error {
                code: error.kind,
                message: error.message,
            }),
        }
    }

    fn add_usage(&mut self, usage: &SseUsage) {
        self.input_tokens = self.input_tokens.saturating_add(usage.input_tokens);
        self.output_tokens = self.output_tokens.saturating_add(usage.output_tokens);
        let delta = usage.input_tokens.saturating_add(usage.output_tokens);
        if delta == 0 {
            return;
        }
        self.cumulative_tokens = Some(self.cumulative_tokens.unwrap_or(0).saturating_add(delta));
    }
}

/// Parse a single SSE `data:` JSON payload into an [`SseEvent`].
///
/// `eventsource-stream` already reassembles event frames upstream; this
/// function only decodes the `data` JSON.
///
/// # Errors
///
/// Returns [`ProviderError::Sse`] when the payload is not valid JSON or does
/// not match the [`SseEvent`] tagged-enum shape (e.g., unknown `type`).
pub fn parse_sse_data(data: &str) -> Result<SseEvent, ProviderError> {
    serde_json::from_str(data).map_err(|e| ProviderError::Sse(e.to_string()))
}

/// Convert an injected byte stream into a stream of [`ProviderEvent`]s.
///
/// `*_with`-style test seam: the production wrapper in `anthropic.rs` feeds
/// this with `reqwest::Response::bytes_stream()`; tests feed it with
/// pre-canned wiremock bytes. Same translation logic exercised both ways.
///
/// Malformed individual events are silently skipped (with the bytes already
/// consumed by `eventsource-stream`'s framer); this lets a single bad event
/// in the middle of a stream not blow up the whole session. Server-side
/// `error` SSE events surface as [`ProviderEvent::Error`] (via the state
/// machine) so the caller still gets the signal.
pub fn stream_events<S, E>(byte_stream: S) -> impl Stream<Item = ProviderEvent>
where
    S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    let event_stream = byte_stream.eventsource();
    futures::stream::unfold(
        (event_stream, SseState::new()),
        |(mut es, mut state)| async move {
            // M07.D2 — drain a deferred event (the MessageStop buffered
            // behind the Usage it co-occurs with) before the next frame.
            if let Some(buffered) = state.pending.take() {
                return Some((buffered, (es, state)));
            }
            while let Some(event_result) = es.next().await {
                let Ok(event) = event_result else {
                    continue;
                };
                let Ok(parsed) = parse_sse_data(&event.data) else {
                    continue;
                };
                if let Some(provider_event) = state.translate(parsed) {
                    return Some((provider_event, (es, state)));
                }
            }
            None
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_message_start() {
        let data = r#"{"type":"message_start","message":{"id":"msg_1","model":"claude-haiku-4-5","usage":{"input_tokens":25,"output_tokens":1}}}"#;
        let event = parse_sse_data(data).unwrap();
        assert!(matches!(event, SseEvent::MessageStart { .. }));
    }

    #[test]
    fn parses_text_delta() {
        let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"hello"}}"#;
        let event = parse_sse_data(data).unwrap();
        assert!(matches!(
            event,
            SseEvent::ContentBlockDelta { index: 0, .. }
        ));
    }

    #[test]
    fn parses_ping() {
        let event = parse_sse_data(r#"{"type":"ping"}"#).unwrap();
        assert!(matches!(event, SseEvent::Ping));
    }

    #[test]
    fn parses_error() {
        let data = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;
        let event = parse_sse_data(data).unwrap();
        assert!(matches!(event, SseEvent::Error { .. }));
    }

    #[test]
    fn parses_message_delta_with_stop_reason() {
        let data = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":15}}"#;
        let event = parse_sse_data(data).unwrap();
        assert!(matches!(event, SseEvent::MessageDelta { .. }));
    }

    #[test]
    fn translate_text_delta_emits_provider_event() {
        let mut state = SseState::new();
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta { text: "hi".into() },
        };
        let out = state.translate(evt);
        assert!(matches!(out, Some(ProviderEvent::TextDelta { .. })));
    }

    #[test]
    fn translate_ping_returns_none() {
        let mut state = SseState::new();
        assert!(state.translate(SseEvent::Ping).is_none());
    }

    #[test]
    fn translate_message_delta_emits_message_stop() {
        let mut state = SseState::new();
        let evt = SseEvent::MessageDelta {
            delta: SseMessageDelta {
                stop_reason: Some("end_turn".into()),
                stop_sequence: None,
            },
            usage: None,
        };
        let out = state.translate(evt);
        assert!(matches!(out, Some(ProviderEvent::MessageStop { .. })));
    }

    #[test]
    fn cumulative_tokens_attached_to_message_stop() {
        // Stage D: SseState accumulates input + output tokens across
        // message_start + message_delta and surfaces the running total
        // on the MessageStop translation.
        let mut state = SseState::new();
        // message_start carries initial usage.
        let _ = state.translate(SseEvent::MessageStart {
            message: SseMessage {
                id: "m1".into(),
                model: "haiku".into(),
                usage: SseUsage {
                    input_tokens: 25,
                    output_tokens: 1,
                },
            },
        });
        // message_delta with usage adds to the running total and triggers
        // the MessageStop translation.
        let out = state.translate(SseEvent::MessageDelta {
            delta: SseMessageDelta {
                stop_reason: Some("end_turn".into()),
                stop_sequence: None,
            },
            usage: Some(SseUsage {
                input_tokens: 0,
                output_tokens: 15,
            }),
        });
        match out {
            Some(ProviderEvent::MessageStop { total_tokens, .. }) => {
                assert_eq!(total_tokens, Some(41));
            }
            other => panic!("expected MessageStop with total_tokens, got {other:?}"),
        }
    }

    #[test]
    fn missing_usage_keeps_total_tokens_none() {
        let mut state = SseState::new();
        let out = state.translate(SseEvent::MessageDelta {
            delta: SseMessageDelta {
                stop_reason: Some("end_turn".into()),
                stop_sequence: None,
            },
            usage: None,
        });
        match out {
            Some(ProviderEvent::MessageStop { total_tokens, .. }) => {
                assert_eq!(total_tokens, None);
            }
            other => panic!("expected MessageStop, got {other:?}"),
        }
    }

    #[test]
    fn translate_error_emits_provider_error_event() {
        let mut state = SseState::new();
        let evt = SseEvent::Error {
            error: SseError {
                kind: "overloaded_error".into(),
                message: "slow down".into(),
            },
        };
        let out = state.translate(evt);
        assert!(matches!(out, Some(ProviderEvent::Error { .. })));
    }

    #[test]
    fn tool_use_accumulates_partial_json_then_emits_on_stop() {
        let mut state = SseState::new();
        state.translate(SseEvent::ContentBlockStart {
            index: 0,
            content_block: SseContentBlockStart::ToolUse {
                id: "tu_1".into(),
                name: "search".into(),
                input: serde_json::Value::Object(serde_json::Map::new()),
            },
        });
        state.translate(SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#"{"q":"#.into(),
            },
        });
        state.translate(SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#""rust"}"#.into(),
            },
        });
        let out = state.translate(SseEvent::ContentBlockStop { index: 0 });
        match out {
            Some(ProviderEvent::ToolUse { id, name, input }) => {
                assert_eq!(id, "tu_1");
                assert_eq!(name, "search");
                assert_eq!(input, serde_json::json!({"q": "rust"}));
            }
            other => panic!("expected ToolUse, got {other:?}"),
        }
    }

    #[test]
    fn signature_delta_is_silent() {
        let mut state = SseState::new();
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::SignatureDelta {
                signature: "abc".into(),
            },
        };
        assert!(state.translate(evt).is_none());
    }

    #[test]
    fn malformed_data_returns_sse_error() {
        let result = parse_sse_data("not json at all");
        assert!(matches!(result, Err(ProviderError::Sse(_))));
    }
}
