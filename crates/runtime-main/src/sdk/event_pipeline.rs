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
//!
//! M06.A L1 wire-up (ADR-0009 closure): when constructed via
//! [`EventPipeline::with_enforcement`], a `ProviderEvent::ToolUse`
//! first translates to a `Vec<CapabilityDeclaration>` via
//! [`crate::framework_loader::capabilities_for_tool`], then runs
//! [`crate::capability::CapabilityEnforcer::check`]. On `Ok` the
//! pipeline emits both [`AgentEvent::CapabilityGrant`] and the existing
//! [`AgentEvent::ToolInvoked`]. On
//! [`crate::capability::CapabilityError::Denied`] the pipeline emits
//! [`AgentEvent::CapabilityViolation`] and **omits** `ToolInvoked` —
//! the renderer must not paint a tool node for a blocked dispatch. On
//! [`crate::capability::CapabilityError::TierForbidden`] the pipeline
//! emits [`AgentEvent::TierViolation`] and similarly omits
//! `ToolInvoked`. HITL routing for the violation cases lives in the
//! SDK run loop ([`crate::sdk::AgentSdk`]) which observes the emitted
//! event sequence and awaits the corresponding `HitlSeam` resolution
//! before resuming.

use std::sync::Arc;

use runtime_core::event::{AgentEvent, CapabilityKindRef, ToolSource};
use runtime_core::generated::capability::CapabilityKind;

use super::structured_emitter::{parse_structured, EmitterOutput};
use crate::capability::{CapabilityEnforcer, CapabilityError};
use crate::framework_loader::{capabilities_for_tool, CapabilityLookupError, FrameworkRef};
use crate::providers::ProviderEvent;
use crate::tier::Tier;

/// Optional L1 enforcement wiring (M06.A).
#[derive(Clone)]
pub struct EnforcementContext {
    enforcer: Arc<CapabilityEnforcer>,
    framework: FrameworkRef,
}

impl EnforcementContext {
    /// Build the wiring from the pre-constructed enforcer + framework
    /// references the SDK already holds.
    #[must_use]
    pub const fn new(enforcer: Arc<CapabilityEnforcer>, framework: FrameworkRef) -> Self {
        Self {
            enforcer,
            framework,
        }
    }
}

/// Stateful translator. Hold one per agent stream; call
/// [`Self::next_event`] for each incoming `ProviderEvent`, then
/// [`Self::flush`] at end-of-stream to drain any buffered text.
pub struct EventPipeline {
    agent_id: String,
    text_buffer: String,
    enforcement: Option<EnforcementContext>,
}

impl EventPipeline {
    /// Construct a pipeline scoped to a single agent without L1
    /// enforcement. Pre-M06.A behavior; smoke / unit tests + the M02
    /// production path that doesn't yet wire a framework consume this.
    #[must_use]
    pub const fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            text_buffer: String::new(),
            enforcement: None,
        }
    }

    /// Construct a pipeline with the M06.A L1 wire-up active. Per
    /// `ProviderEvent::ToolUse`, the pipeline runs
    /// `enforcer.check(agent_id, &needed)` and routes per outcome.
    #[must_use]
    pub const fn with_enforcement(agent_id: String, enforcement: EnforcementContext) -> Self {
        Self {
            agent_id,
            text_buffer: String::new(),
            enforcement: Some(enforcement),
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
                self.translate_tool_use(name, input, &mut output);
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

    /// Translate a `ProviderEvent::ToolUse` to the appropriate
    /// [`AgentEvent`] sequence. Without `self.enforcement`, behaves as
    /// pre-M06.A and emits a single
    /// [`AgentEvent::ToolInvoked`]. With `self.enforcement` set,
    /// gates the dispatch through L1 + L4 and emits the wire-up
    /// sequence per ADR-0009 closure.
    fn translate_tool_use(
        &self,
        name: String,
        input: serde_json::Value,
        output: &mut Vec<AgentEvent>,
    ) {
        let Some(ctx) = self.enforcement.as_ref() else {
            // Pre-M06.A behavior — no gate.
            output.push(AgentEvent::ToolInvoked {
                agent_id: self.agent_id.clone(),
                tool_name: name,
                source: ToolSource::Builtin,
                server: None,
                input,
            });
            return;
        };

        let needed = match capabilities_for_tool(&ctx.framework, &name) {
            Ok(decls) => decls,
            Err(CapabilityLookupError::ToolNotFound { name: missing }) => {
                // Tool not declared in framework.tools[]. Surface as a
                // CapabilityViolation using NoMatchingGrant copy — the
                // renderer's existing GapPanel + capability-violation
                // modal handles both. Per gotcha #66: contract failure
                // must surface as a deniable event, not a silent skip.
                output.push(AgentEvent::CapabilityViolation {
                    agent_id: self.agent_id.clone(),
                    capability_kind: CapabilityKindRef::Exec,
                    requested_action: format!("invoke tool '{missing}'"),
                    declared_scope: format!("tool '{missing}' not declared in framework.tools[]"),
                });
                return;
            }
        };

        // L1 gate. Per the M05.B + Stage D contract, `check` runs L4
        // first (TierForbidden when the tier excludes the kind) then
        // L1 (Denied when no grant subsumes); we route both Err shapes
        // to their distinct event variants per the M05.D `tier_violation`
        // / M05.B `capability_violation` separation.
        let needed_decl = needed
            .first()
            .expect("capabilities_for_tool returns ≥1 decl");
        match ctx.enforcer.check(&self.agent_id, needed_decl) {
            Ok(()) => {
                output.push(AgentEvent::CapabilityGrant {
                    parent_agent_id: None,
                    granted_to: self.agent_id.clone(),
                    capability_kind: kind_to_ref(needed_decl.kind),
                    resource: (*needed_decl.resource).clone(),
                    narrowed_from: None,
                });
                output.push(AgentEvent::ToolInvoked {
                    agent_id: self.agent_id.clone(),
                    tool_name: name,
                    source: ToolSource::Builtin,
                    server: None,
                    input,
                });
            }
            Err(CapabilityError::Denied { reason, agent_id }) => {
                let scope_copy = match reason {
                    crate::capability::DenyReason::NoDeclarations => {
                        "no capabilities declared".to_string()
                    }
                    crate::capability::DenyReason::NoMatchingGrant => {
                        "declared grants do not cover this request".to_string()
                    }
                };
                output.push(AgentEvent::CapabilityViolation {
                    agent_id,
                    capability_kind: kind_to_ref(needed_decl.kind),
                    requested_action: format!("invoke tool '{name}'"),
                    declared_scope: scope_copy,
                });
            }
            Err(CapabilityError::TierForbidden {
                agent_id,
                tier,
                capability_kind,
            }) => {
                output.push(AgentEvent::TierViolation {
                    agent_id,
                    tier: tier_to_ref(tier),
                    capability_kind: kind_to_ref(capability_kind),
                    attempted_action: format!("invoke tool '{name}' under {tier:?} tier"),
                });
            }
        }
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

const fn kind_to_ref(k: CapabilityKind) -> CapabilityKindRef {
    match k {
        CapabilityKind::Read => CapabilityKindRef::Read,
        CapabilityKind::Write => CapabilityKindRef::Write,
        CapabilityKind::Exec => CapabilityKindRef::Exec,
        CapabilityKind::Network => CapabilityKindRef::Network,
        CapabilityKind::ProcessSpawn => CapabilityKindRef::ProcessSpawn,
    }
}

const fn tier_to_ref(t: Tier) -> runtime_core::event::TierRef {
    match t {
        Tier::Novice => runtime_core::event::TierRef::Novice,
        Tier::Promoted => runtime_core::event::TierRef::Promoted,
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

    #[test]
    fn unwired_tool_use_emits_only_tool_invoked() {
        // Pre-M06.A pipeline (no enforcement) preserves M02 behavior:
        // a single ToolInvoked event, no CapabilityGrant.
        let mut p = EventPipeline::new("a1".into());
        let events = p.next_event(ProviderEvent::ToolUse {
            id: "t1".into(),
            name: "Read".into(),
            input: serde_json::json!({"path": "src/lib.rs"}),
        });
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], AgentEvent::ToolInvoked { .. }));
    }
}
