//! `ProviderEvent` → `AgentEvent` translator with consecutive-`TextDelta`
//! bundling. Pure logic; no I/O. Spec §2 event taxonomy.
//!
//! Bundling: consecutive `ProviderEvent::TextDelta`s collapse into one
//! `AgentEvent::StreamText` per non-text event boundary. Without this the
//! renderer gets spammed with one event per token; with it, one event per
//! "burst of text" which matches user expectation for streaming UX.
//!
//! Structured emission: when a text bundle flushes, the M04 Stage B
//! [`crate::sdk::parse_structured`] parser scans for `<<DECISION>>` /
//! `<<PLAN>>` delimited blocks and emits `AgentEvent::DecisionRecord` /
//! plan-creation events *in addition to* the [`AgentEvent::StreamText`]
//! (the raw text is always preserved). M02's line-level
//! `decision_extractor` heuristic was replaced (closes M02 🟡
//! false-positive carry-forward).

use runtime_core::event::{AgentEvent, ToolSource};

use super::structured_emitter::{parse_structured, EmitterOutput};
use crate::providers::ProviderEvent;

/// Stateful translator. Hold one per agent stream; call
/// [`Self::next_event`] for each incoming `ProviderEvent`, then
/// [`Self::flush`] at end-of-stream to drain any buffered text.
pub struct EventPipeline {
    agent_id: String,
    text_buffer: String,
}

impl EventPipeline {
    /// Construct a pipeline scoped to a single agent.
    #[must_use]
    pub const fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            text_buffer: String::new(),
        }
    }

    /// Translate one `ProviderEvent` into zero-or-more `AgentEvent`s.
    ///
    /// Internal bundling state is mutated; call [`Self::flush`] at
    /// end-of-stream to drain.
    pub fn next_event(&mut self, event: ProviderEvent) -> Vec<AgentEvent> {
        let mut output = Vec::new();
        match event {
            ProviderEvent::TextDelta { text } => {
                self.text_buffer.push_str(&text);
            }
            ProviderEvent::ThinkingDelta { text } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::StreamText {
                    agent_id: self.agent_id.clone(),
                    text,
                });
            }
            ProviderEvent::ToolUse { id: _, name, input } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::ToolInvoked {
                    agent_id: self.agent_id.clone(),
                    tool_name: name,
                    source: ToolSource::Builtin,
                    server: None,
                    input,
                });
            }
            ProviderEvent::ToolResult {
                id,
                output: result,
                tokens_in,
                tokens_out,
            } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::ToolResult {
                    agent_id: self.agent_id.clone(),
                    tool_name: format!("tool_{id}"),
                    output: result,
                    duration_ms: 0,
                    tokens_in,
                    tokens_out,
                });
            }
            ProviderEvent::MessageStop {
                stop_reason,
                total_tokens,
            } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::AgentComplete {
                    agent_id: self.agent_id.clone(),
                    result: stop_reason,
                    tokens_total: total_tokens,
                });
            }
            ProviderEvent::Error { code, message } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::AgentError {
                    agent_id: self.agent_id.clone(),
                    error: format!("{code}: {message}"),
                });
            }
        }
        output
    }

    /// Drain any buffered text. Call at end-of-stream so the final burst
    /// reaches the renderer.
    pub fn flush(&mut self) -> Vec<AgentEvent> {
        let mut output = Vec::new();
        self.flush_text_buffer(&mut output);
        output
    }

    fn flush_text_buffer(&mut self, output: &mut Vec<AgentEvent>) {
        if self.text_buffer.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.text_buffer);
        // Structured emitter: extract any well-formed delimited blocks.
        // Malformed blocks return Err — log + continue (the raw text
        // still reaches the renderer; downstream just doesn't get a
        // typed event for the malformed block).
        match parse_structured(&text) {
            Ok(outputs) => {
                for out in outputs {
                    if let EmitterOutput::Decision {
                        decision,
                        rationale,
                        tool_used,
                    } = out
                    {
                        output.push(AgentEvent::DecisionRecord {
                            agent_id: self.agent_id.clone(),
                            decision,
                            rationale,
                            tool_used,
                        });
                    }
                    // PlanCreation outputs are surfaced through the
                    // SDK's plan_loop (Stage B+); the event_pipeline's
                    // job stops at decision translation.
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "structured emitter parse failed; raw text still forwarded");
            }
        }
        output.push(AgentEvent::StreamText {
            agent_id: self.agent_id.clone(),
            text,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_flush_emits_nothing() {
        let mut p = EventPipeline::new("a1".into());
        assert!(p.flush().is_empty());
    }

    #[test]
    fn lone_text_delta_flushes_on_message_stop() {
        let mut p = EventPipeline::new("a1".into());
        let pre = p.next_event(ProviderEvent::TextDelta { text: "hi".into() });
        assert!(pre.is_empty(), "text deltas buffer until a boundary");
        let post = p.next_event(ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
            total_tokens: None,
        });
        assert!(post
            .iter()
            .any(|e| matches!(e, AgentEvent::StreamText { .. })));
        assert!(post
            .iter()
            .any(|e| matches!(e, AgentEvent::AgentComplete { .. })));
    }
}
