//! `AgentSdk` — drives a provider stream and emits typed `AgentEvent`s.
//!
//! Generic over [`LLMProvider`] so v1.0+ providers (`OpenAI`, local) slot in
//! behind the same trait without changes here. M02 ships single-turn only;
//! multi-turn tool-use loops land in M03+.
//!
//! Cancellation-safety: drop at any await point cleans up cleanly. The
//! drone IPC client is used only via send (no long-lived stream subscribed
//! by the SDK loop in M02).
//!
//! Test seam: [`AgentSdk::run_agent_with_provider_stream`] accepts a
//! pre-built `Stream<Item = ProviderEvent>` so tests inject deterministic
//! sequences without touching reqwest. Production wrapper
//! [`AgentSdk::run_agent`] constructs the real provider stream via
//! [`LLMProvider::stream`].
//!
//! M06.A wire-up (ADR-0009 closure):
//! - L1 enforcement at the dispatch boundary lives inside
//!   [`super::EventPipeline::with_enforcement`]; [`AgentSdk::with_capability_wiring`]
//!   constructs the wired pipeline. The SDK observes the emitted event
//!   sequence and routes `capability_violation` / `tier_violation` events
//!   through the [`HitlSeam::on_capability_violation`] M04.E trigger
//!   before resuming.
//! - L2a narrowing at the spawn boundary runs at session start in
//!   [`Self::spawn_framework_subagents`] — for every inline sub-agent
//!   declared in the framework whose `parent` matches the session
//!   root, the loader translates the parent's `Capabilities` block
//!   to grants, the child's `Capabilities` block to proposed grants,
//!   calls `narrow(parent, proposed)`, and emits `AgentSpawned` with
//!   the proposed-set serialized to `narrowed_from`. Widening attempts
//!   emit `CapabilityViolation` and skip the spawn.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::{Stream, StreamExt};
use runtime_core::event::{AgentEvent, CapabilityKindRef, TierRef, ToolSource};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::builtin_tools::{self, BuiltinExecError};
use super::event_pipeline::{EnforcementContext, EventPipeline};
use super::load_skill;
use super::mcp_dispatch::{
    apply_renderable, mcp_dispatch_error_event, renderable_needs_hitl, McpDispatchOutcome,
    McpToolDispatch, RenderableOutcome,
};
use super::request_capability::{
    self, handle_request_capability, parse_capability_kind, RequestCapabilityInvocation,
    RequestCapabilityResult,
};
use crate::budget::{
    BudgetEnforcer, BudgetScopeCap, DefaultLadder, DownshiftHook, RemainingBudget, ThresholdAction,
};
use crate::capability::{narrow, CapabilityEnforcer, CapabilityError, DenyReason};
use crate::drone_ipc::{DroneClient, DroneIpcError};
use crate::framework_loader::{
    capabilities_to_declarations, declaration_to_narrowed_from_str, inline_agents,
    parent_grants_for_agent, root_agent_role, FrameworkRef,
};
use crate::hitl::HitlSeam;
use crate::providers::{
    AgentConfig, ContentBlock, CostBreakdown, LLMProvider, Message, MessageRole, ProviderError,
    ProviderEvent, ToolResultContent,
};
use crate::tier::Tier;
use runtime_core::generated::framework::Framework;

/// Default wait for HITL responses on capability / tier violations
/// surfaced by the L1 wire-up. Long enough that a user can read the
/// modal + decide; short enough that a forgotten prompt does not hang
/// the SDK loop indefinitely. Matches the M04.E `HitlSeam` default
/// budgets used elsewhere.
const HITL_DEFAULT_WAIT: Duration = Duration::from_secs(3600);

/// Safety cap on the multi-turn agent-with-tools loop (M07.D2,
/// ADR-0011 d). Each turn that dispatches ≥1 MCP tool feeds the result
/// back and re-streams; a well-behaved model converges in a handful of
/// turns. This bounds a pathological tool-loop (model that never stops
/// requesting tools) so a session cannot spin unbounded. v0.1
/// single-session; generous enough to never clip a real workflow.
const MAX_AGENT_TURNS: usize = 16;

/// One MCP tool the run loop dispatched this turn, captured so the
/// multi-turn driver can feed the result back as the next turn's
/// `tool_result` (Anthropic message-history continuation — no new
/// `LLMProvider` trait method; the provider is stateless and the
/// conversation lives in [`AgentConfig::messages`]).
struct DispatchedTool {
    /// The provider-assigned `tool_use` id the result must reference.
    id: String,
    /// Resolved tool name (for the assistant `tool_use` block).
    name: String,
    /// Original call arguments (ride into the assistant `tool_use`).
    input: serde_json::Value,
    /// The MCP server's structured result (the user `tool_result`).
    value: serde_json::Value,
}

/// Tools dispatched during one provider-stream turn. Non-empty ⇒ the
/// multi-turn driver feeds these back and re-streams; empty ⇒ the model
/// stopped requesting tools and the session ends.
#[derive(Default)]
struct TurnFeedback {
    dispatched: Vec<DispatchedTool>,
    /// Set when a `request_capability` gap was raised this turn (M08.7 rung
    /// 4). The multi-turn driver SUSPENDS — issues no further provider turn
    /// — rather than feeding results back, so the session halts cleanly at
    /// the gap (recoverable from the snapshot per §1b).
    gap_suspended: bool,
    /// Set when a budget `HardStop` fired this turn (M08.7 rung 5). The
    /// multi-turn driver STOPS — issues no further provider turn — so the
    /// run halts at the cap. The load-bearing safety primitive: a
    /// `budget_exceeded` event firing licenses "the event fired", NOT "the
    /// run halted" (gotcha #66) — this break is the run-halt.
    budget_stopped: bool,
    /// Set when a budget `Suspend` (HITL at the suspend threshold) fired
    /// this turn (M08.7 rung 5). v0.1 records the suspend and halts
    /// (suspend-and-record) — the HITL **resume** half is NOT a dead-end:
    /// it is the same resolve-and-resume pattern as the rung-4 gap (suspend
    /// → human approves the budget → resume), folded into ADR-0029
    /// generalized to budget. Rung 5 records the suspend; the resume joins
    /// that rung.
    budget_suspended: bool,
    /// Set when a budget `Downshift` swapped the model via the ladder
    /// (M08.7 rung 5). The driver applies it to the next turn's
    /// [`AgentConfig::model`] so subsequent turns use the cheaper model.
    new_model: Option<String>,
}

/// Outcome of dispatching one `request_capability` `ToolUse` (M08.7 rung 4).
enum RequestCapabilityDisposition {
    /// The handler emitted a `*Missing` gap — the turn loop must suspend.
    Suspended,
    /// The invocation was malformed (empty name / justification); feed this
    /// error `tool_result` back and continue (NOT a suspend — the session
    /// is still healthy and the model can fix its call).
    Continue(DispatchedTool),
}

/// Collects the gap event(s) [`handle_request_capability`] emits so the run
/// loop can forward them through [`AgentSdk::emit`] — the same persist +
/// debug-log + renderer path every other event takes (so the gap is
/// recoverable per §1b). Reuses the built handler without reimplementing
/// its `match kind` arm.
#[derive(Default)]
struct GapEventCollector {
    events: std::sync::Mutex<Vec<AgentEvent>>,
}

impl GapEventCollector {
    /// Drain the collected events.
    fn take(&self) -> Vec<AgentEvent> {
        std::mem::take(
            &mut self
                .events
                .lock()
                .expect("gap collector mutex not poisoned"),
        )
    }
}

#[async_trait::async_trait]
impl crate::framework_loader::Emitter for GapEventCollector {
    async fn emit(&self, event: AgentEvent) {
        self.events
            .lock()
            .expect("gap collector mutex not poisoned")
            .push(event);
    }
}

/// Newtype wrapping a session UUID. Cheap to clone; serializes as a
/// hyphenated UUID string.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionId {
    /// Generate a fresh session id.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Hyphenated string form (matches `serde` serialization).
    #[must_use]
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

/// Errors raised by [`AgentSdk`].
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    /// Provider-side failure during stream open or while consuming events.
    #[error("provider error: {0}")]
    Provider(#[from] ProviderError),
    /// Drone IPC failure while emitting a snapshot trigger.
    #[error("drone IPC error: {0}")]
    Drone(#[from] DroneIpcError),
    /// The renderer-side `mpsc::Receiver` was dropped while the SDK was
    /// still emitting events.
    #[error("event channel closed")]
    EventChannelClosed,
}

/// Optional capability + spawn-narrowing wiring (M06.A).
#[derive(Clone)]
pub struct CapabilityWiring {
    /// L1 enforcer the `EventPipeline` gates `ToolUse` through.
    pub enforcer: Arc<CapabilityEnforcer>,
    /// Framework reference the loader walk + tool-capability lookup
    /// consumes.
    pub framework: FrameworkRef,
    /// HITL seam the SDK loop awaits on for `capability_violation` /
    /// `tier_violation` routing per M04.E `on_capability_violation`.
    pub hitl_seam: Arc<HitlSeam>,
    /// M08.7 rung 3 (ADR-0027): resolved skill bodies, keyed by skill
    /// name. The `LoadSkill` handler reads from here — the bodies are
    /// resolved once at load (ADR-0022 companions) and threaded in, never
    /// re-resolved. Empty for sessions with no skills.
    pub resolved_skills: BTreeMap<String, String>,
}

impl CapabilityWiring {
    /// Construct from the three Arc references the Tauri shell layer
    /// already holds at session start. `resolved_skills` defaults empty —
    /// attach it via [`Self::with_resolved_skills`].
    #[must_use]
    pub const fn new(
        enforcer: Arc<CapabilityEnforcer>,
        framework: FrameworkRef,
        hitl_seam: Arc<HitlSeam>,
    ) -> Self {
        Self {
            enforcer,
            framework,
            hitl_seam,
            resolved_skills: BTreeMap::new(),
        }
    }

    /// Attach resolved skill bodies (name → body) for the `LoadSkill`
    /// handler (M08.7 rung 3; ADR-0027). Additive over [`Self::new`] so
    /// every existing constructor call site stays byte-stable; the
    /// Tester's skill seam sets it.
    #[must_use]
    pub fn with_resolved_skills(mut self, resolved_skills: BTreeMap<String, String>) -> Self {
        self.resolved_skills = resolved_skills;
        self
    }
}

/// Agent SDK. Generic over the LLM provider so v1.0+ providers slot in
/// behind the same trait.
pub struct AgentSdk<P: LLMProvider> {
    provider: Arc<P>,
    event_tx: mpsc::Sender<AgentEvent>,
    drone_client: Arc<DroneClient>,
    session_id: SessionId,
    capability_wiring: Option<CapabilityWiring>,
    /// M06.F (ADR-0010 + ADR-0011) MCP-dispatch seam. The Tauri shell
    /// injects an `Arc<dyn McpToolDispatch>` via [`Self::with_mcp_dispatch`];
    /// the run loop intercepts `ProviderEvent::ToolUse` through it before
    /// the existing Stage A non-MCP L1 path. `None` (the M02 / smoke
    /// path) leaves the run loop pre-M06.F.
    mcp_dispatch: Option<Arc<dyn McpToolDispatch>>,
}

impl<P: LLMProvider + 'static> AgentSdk<P> {
    /// Construct without M06.A capability wiring. Smoke / streaming-only
    /// sessions (M02 production path; existing tests) consume this.
    #[must_use]
    pub const fn new(
        provider: Arc<P>,
        event_tx: mpsc::Sender<AgentEvent>,
        drone_client: Arc<DroneClient>,
        session_id: SessionId,
    ) -> Self {
        Self {
            provider,
            event_tx,
            drone_client,
            session_id,
            capability_wiring: None,
            mcp_dispatch: None,
        }
    }

    /// Inject the M06.F MCP-dispatch seam (ADR-0010 dependency
    /// inversion; ADR-0011 scopes this to the seam — the concrete
    /// `McpDispatcher` is constructed by the Tauri shell, an M07
    /// carry-forward). Builder over the existing constructors so the
    /// `*_with` shell seam composes it onto a wired-or-unwired SDK.
    #[must_use]
    pub fn with_mcp_dispatch(mut self, dispatch: Arc<dyn McpToolDispatch>) -> Self {
        self.mcp_dispatch = Some(dispatch);
        self
    }

    /// Construct with the M06.A L1 + L2a wire-up active.
    ///
    /// At session start the SDK walks `framework.agents[]` and emits
    /// `AgentSpawned` per inline sub-agent of the session root,
    /// running `narrow(parent_grants, proposed)` to gate widening.
    /// During streaming, every `ProviderEvent::ToolUse` flows through
    /// `enforcer.check(agent_id, &needed)` and routes per outcome.
    #[must_use]
    pub const fn with_capability_wiring(
        provider: Arc<P>,
        event_tx: mpsc::Sender<AgentEvent>,
        drone_client: Arc<DroneClient>,
        session_id: SessionId,
        wiring: CapabilityWiring,
    ) -> Self {
        Self {
            provider,
            event_tx,
            drone_client,
            session_id,
            capability_wiring: Some(wiring),
            mcp_dispatch: None,
        }
    }

    /// Production entry point — the multi-turn agent-with-tools loop
    /// (M07.D2, ADR-0011 d). Runs the session prelude ONCE, then streams
    /// the provider; when a turn dispatched ≥1 MCP tool through D1's
    /// concrete `McpDispatcher`, the results are fed back as the next
    /// turn's `tool_result` (Anthropic message-history continuation —
    /// the provider is stateless, the conversation lives in
    /// [`AgentConfig::messages`], so no new `LLMProvider` method is
    /// needed) and the loop re-streams. It terminates when a turn
    /// requests no tool (the model stopped) or `MAX_AGENT_TURNS` is
    /// hit. The smoke / no-tools path is the degenerate one-turn case.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::Provider`] if any turn's `stream` call fails;
    /// otherwise propagates errors from the prelude / stream drive.
    pub async fn run_agent(&self, mut config: AgentConfig) -> Result<(), SdkError> {
        let (agent_id, mut pipeline) = self.session_prelude().await?;
        // M08.7 rung 5: the session budget enforcer, built ONCE from the
        // framework's budget caps (when capability wiring carries a
        // framework with a `budget` block). It accumulates spend across
        // turns. `None` when no cap is configured — budget-less frameworks
        // and the un-wired smoke path stay byte-stable (no enforcement).
        let mut session_budget = self
            .capability_wiring
            .as_ref()
            .and_then(|w| build_session_budget(&w.framework));
        for _turn in 0..MAX_AGENT_TURNS {
            let stream = self.provider.stream(config.clone()).await?;
            let feedback = self
                .drive_stream(
                    stream,
                    &mut pipeline,
                    &agent_id,
                    session_budget.as_mut(),
                    &config.model,
                )
                .await?;
            // M08.7 rung 5: a budget Downshift swapped the model — apply it
            // so the NEXT provider.stream uses the cheaper model (the swap
            // the next turn actually uses; gotcha — observe it on the config,
            // not just the hook return).
            if let Some(model) = feedback.new_model {
                config.model = model;
            }
            if feedback.gap_suspended || feedback.budget_stopped || feedback.budget_suspended {
                // Halt the turn loop — issue no further provider turn. A
                // request_capability gap suspends (M08.7 rung 4); a budget
                // HardStop / Suspend stops at the cap (rung 5), EVEN if this
                // turn also dispatched a tool. The triggering event (gap /
                // budget_exceeded / budget_suspended) is persisted
                // (recoverable §1b); resolution (grant / approve-budget) is
                // the HITL resume surface (suspend-and-record — ADR-0029).
                break;
            }
            if feedback.dispatched.is_empty() {
                // Model requested no tool this turn → it has stopped.
                break;
            }
            // Feed the dispatched tools' results back as the next turn:
            // one assistant message carrying every `tool_use`, then one
            // user message carrying the matching `tool_result`s (the
            // Anthropic continuation contract).
            let assistant_blocks = feedback
                .dispatched
                .iter()
                .map(|d| ContentBlock::ToolUse {
                    id: d.id.clone(),
                    name: d.name.clone(),
                    input: d.input.clone(),
                })
                .collect();
            let user_blocks = feedback
                .dispatched
                .iter()
                .map(|d| ContentBlock::ToolResult {
                    tool_use_id: d.id.clone(),
                    content: ToolResultContent::Text(d.value.to_string()),
                    is_error: None,
                })
                .collect();
            config.messages.push(Message {
                role: MessageRole::Assistant,
                content: assistant_blocks,
            });
            config.messages.push(Message {
                role: MessageRole::User,
                content: user_blocks,
            });
        }
        for agent_event in pipeline.flush() {
            self.emit(agent_event).await?;
        }
        Ok(())
    }

    /// Test-seam variant. Accepts any pre-built `ProviderEvent` stream
    /// and drives exactly ONE turn (no multi-turn re-stream — a
    /// pre-built stream cannot be re-opened). Behaviorally identical to
    /// the pre-M07.D2 single-pass loop: prelude → drive one stream →
    /// flush.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::EventChannelClosed`] if the receiver was
    /// dropped, or [`SdkError::Drone`] if the snapshot trigger failed.
    pub async fn run_agent_with_provider_stream<S>(&self, stream: S) -> Result<(), SdkError>
    where
        S: Stream<Item = ProviderEvent> + Unpin,
    {
        let (agent_id, mut pipeline) = self.session_prelude().await?;
        // The single-turn seam carries no budget (no multi-turn spend
        // accumulation, no config to swap a model on — M08.7 rung 5).
        let _feedback = self
            .drive_stream(stream, &mut pipeline, &agent_id, None, "")
            .await?;
        for agent_event in pipeline.flush() {
            self.emit(agent_event).await?;
        }
        Ok(())
    }

    /// Session prelude (runs ONCE per session, before any turn): derive
    /// the runtime `agent_id`, emit the root `AgentSpawned`, trigger the
    /// start-of-session `SnapshotNow`, run the M06.A L2a sub-agent walk,
    /// and build the (optionally enforcement-wired) `EventPipeline`.
    ///
    /// # Errors
    ///
    /// [`SdkError::EventChannelClosed`] / [`SdkError::Drone`] as the
    /// emit / snapshot paths surface.
    async fn session_prelude(&self) -> Result<(String, EventPipeline), SdkError> {
        // M06.A wire-up: when the framework is present, the runtime
        // agent_id IS the framework's `session_root_agent` id so the
        // enforcer's grants (keyed by framework agent id) match the
        // dispatch path's runtime id. The pre-M06 UUID seed remains
        // for un-wired smoke sessions (no framework available).
        let agent_id = self.capability_wiring.as_ref().map_or_else(
            || format!("agent_{}", Uuid::new_v4()),
            |w| w.framework.session_root_agent.clone(),
        );
        // The root agent's display name: with capability wiring (the
        // Tester / a framework run) it is the framework root agent's
        // `role` — the same source `spawn_framework_subagents` uses for
        // every sub-agent (agent_sdk.rs:480 below). Without wiring (the
        // smoke / streaming-only session — commands.rs:233
        // `AgentSdk::new`) there is no framework, so the literal "smoke"
        // stays correct. M08.5 🔴-2.
        let agent_name = self.capability_wiring.as_ref().map_or_else(
            || "smoke".to_string(),
            |w| root_agent_role(&w.framework, &agent_id),
        );
        // The smoke / streaming-only spawn site predates the L2a wire-up
        // (M06 Stage A); top-level agents have no parent grants to narrow
        // against, so `narrowed_from` is empty here. The framework-walk
        // spawn site below (`spawn_framework_subagents`) populates it for
        // every sub-agent that flowed through `narrow()`.
        self.emit(AgentEvent::AgentSpawned {
            agent_id: agent_id.clone(),
            agent_name,
            parent_id: None,
            session_id: self.session_id.as_string(),
            narrowed_from: Vec::new(),
        })
        .await?;

        // Trigger a SnapshotNow on task start — once per session (the
        // multi-turn loop reuses this prelude's pipeline across turns).
        self.drone_client
            .send(runtime_core::drone::DroneCommand::SnapshotNow {
                reason: "task_started".to_string(),
                state_json: serde_json::json!({"agent_id": agent_id}),
            })
            .await?;

        // L2a wire-up — walk declared sub-agents and emit AgentSpawned
        // per child after running `narrow()`. Runs after the top-level
        // spawn so the child events appear after the root in the
        // event stream (renderer paints root then children).
        if let Some(wiring) = self.capability_wiring.as_ref() {
            self.spawn_framework_subagents(wiring, &agent_id).await?;
        }

        // The run loop retains `agent_id` (M06.F: the MCP-dispatch
        // interception needs it to emit agent_id-correct events, gotcha
        // #68); the pipeline takes a clone.
        let pipeline = self.capability_wiring.as_ref().map_or_else(
            || EventPipeline::new(agent_id.clone()),
            |wiring| {
                EventPipeline::with_enforcement(
                    agent_id.clone(),
                    EnforcementContext::new(
                        Arc::clone(&wiring.enforcer),
                        Arc::clone(&wiring.framework),
                    ),
                )
            },
        );
        Ok((agent_id, pipeline))
    }

    /// Drive ONE provider-stream turn through the MCP-dispatch
    /// interception + the Stage A pipeline, emitting events as it goes.
    /// Returns the [`TurnFeedback`] (MCP tools dispatched this turn) so
    /// the multi-turn caller can feed results back; the single-turn
    /// seam discards it.
    ///
    /// # Errors
    ///
    /// [`SdkError::EventChannelClosed`] if the receiver dropped.
    async fn drive_stream<S>(
        &self,
        mut stream: S,
        pipeline: &mut EventPipeline,
        agent_id: &str,
        session_budget: Option<&mut BudgetEnforcer>,
        model: &str,
    ) -> Result<TurnFeedback, SdkError>
    where
        S: Stream<Item = ProviderEvent> + Unpin,
    {
        let mut feedback = TurnFeedback::default();
        // M08.7 rung 5: accumulate this turn's token usage for budget
        // accounting. The Usage event is PEEKED (not consumed) — it still
        // flows to `pipeline.next_event` below (→ `TokenUsage`); the spend
        // is recorded at the turn boundary after the stream completes.
        let mut turn_input: u64 = 0;
        let mut turn_output: u64 = 0;
        while let Some(provider_event) = stream.next().await {
            if let ProviderEvent::Usage {
                input_tokens,
                output_tokens,
                ..
            } = &provider_event
            {
                turn_input += *input_tokens;
                turn_output += *output_tokens;
            }
            // M06.F (ADR-0010 + ADR-0011): when an MCP-dispatch seam is
            // injected, a `ProviderEvent::ToolUse` is offered to it
            // FIRST. `dispatch_if_mcp` returning `None` (not an MCP
            // tool) falls through to the existing Stage A non-MCP L1
            // path below, unchanged.
            if self.mcp_dispatch.is_some() {
                if let ProviderEvent::ToolUse { id, name, input } = &provider_event {
                    if let Some(dispatched) = self
                        .try_mcp_dispatch(agent_id, id, name, input.clone())
                        .await?
                    {
                        if let Some(d) = dispatched {
                            feedback.dispatched.push(d);
                        }
                        continue;
                    }
                }
            }
            // M08.7 rung 1: in-process built-in tool execution. A ToolUse
            // naming an in-process built-in (Read/Write) routes through the
            // capability-scoped executor BEFORE the emit-only pipeline
            // path. Only when capability wiring is present — the enforcer
            // IS the boundary (Hard Rule 8); the un-wired smoke path keeps
            // the pre-M08.7 emit-only behavior (no enforcer to check
            // against). Built-in results join `feedback.dispatched` exactly
            // as MCP results do, so the multi-turn loop re-streams them.
            if let Some(wiring) = self.capability_wiring.as_ref() {
                if let ProviderEvent::ToolUse { id, name, input } = &provider_event {
                    if builtin_tools::is_builtin_tool(name) {
                        if let Some(d) = self
                            .dispatch_builtin(wiring, agent_id, id, name, input.clone())
                            .await?
                        {
                            feedback.dispatched.push(d);
                        }
                        continue;
                    }
                }
            }
            // M08.7 rung 3 (ADR-0027): LoadSkill — read the agent's
            // already-resolved skill body under the `allowed_skills` gate
            // and inject it. Routes BEFORE the emit-only pipeline path,
            // only when capability wiring is present (the resolved skills +
            // the agent's `allowed_skills` live there). The body rides
            // back as a `tool_result` so it persists in the message
            // history across turns (the rung-1 feedback contract).
            if let Some(wiring) = self.capability_wiring.as_ref() {
                if let ProviderEvent::ToolUse { id, name, input } = &provider_event {
                    if name == load_skill::LOAD_SKILL_TOOL {
                        if let Some(d) = self
                            .dispatch_load_skill(wiring, agent_id, id, input.clone())
                            .await?
                        {
                            feedback.dispatched.push(d);
                        }
                        continue;
                    }
                }
            }
            // M08.7 rung 4 (spec §4b Layer 2): request_capability — the
            // auto-injected meta-tool an agent calls when it needs a
            // capability it lacks. Intercept it BEFORE pipeline.next_event
            // (which treats the undeclared meta-tool as a CapabilityViolation):
            // route it to the built handle_request_capability (emits the
            // *Missing gap with requested_via=request_capability) and SUSPEND
            // — the multi-turn loop issues no further provider turn
            // (suspend-and-record; recoverable §1b). Only with capability
            // wiring (a framework run — the agent_id is the requesting agent,
            // gotcha #68).
            if self.capability_wiring.is_some() {
                if let ProviderEvent::ToolUse { id, name, input } = &provider_event {
                    if name == request_capability::REQUEST_CAPABILITY_TOOL {
                        match self
                            .dispatch_request_capability(agent_id, id, input.clone())
                            .await?
                        {
                            RequestCapabilityDisposition::Suspended => {
                                feedback.gap_suspended = true;
                            }
                            RequestCapabilityDisposition::Continue(d) => {
                                feedback.dispatched.push(d);
                            }
                        }
                        continue;
                    }
                }
            }
            for agent_event in pipeline.next_event(provider_event) {
                let needs_hitl = matches!(
                    &agent_event,
                    AgentEvent::CapabilityViolation { .. } | AgentEvent::TierViolation { .. }
                );
                self.emit(agent_event).await?;
                if needs_hitl {
                    if let Some(wiring) = self.capability_wiring.as_ref() {
                        self.await_capability_violation_hitl(&wiring.hitl_seam)
                            .await;
                    }
                }
            }
        }
        // M08.7 rung 5: the per-turn budget boundary. The turn's stream has
        // completed; build the CostBreakdown from the peeked Usage tokens,
        // price it via the provider's `estimate_cost` (the same pricing the
        // real Anthropic wrapper applies), and feed the USD to the session
        // enforcer's `record_spend`. The returned ThresholdActions dispatch
        // to events + the model swap + the session stop.
        if let Some(budget) = session_budget {
            let breakdown = CostBreakdown::simple(turn_input, turn_output);
            let cost = self.provider.estimate_cost(&breakdown, model);
            self.dispatch_budget_actions(budget.record_spend(cost), model, &mut feedback)
                .await?;
        }
        Ok(feedback)
    }

    /// Dispatch the [`ThresholdAction`]s a turn's `record_spend` returned
    /// (M08.7 rung 5). Each maps to its existing `Budget*` `AgentEvent`
    /// (no schema change — all four pre-exist in `event.v1.json`) plus a
    /// side effect:
    /// - `Warn` → emit `BudgetWarn` (the renderer toast is event-driven;
    ///   the OS desktop notifier is TD-038, out of v0.1 scope).
    /// - `Downshift` → run the `opus → sonnet → haiku` ladder; on a swap
    ///   emit `BudgetDownshift` and set `feedback.new_model` so the NEXT
    ///   turn uses the cheaper model. When the ladder returns `None`
    ///   (already at the cheapest tier), there is no model change to
    ///   report, so no event fires.
    /// - `Suspend` → emit `BudgetSuspended` + set `budget_suspended`
    ///   (suspend-and-record; the HITL resume is ADR-0029 generalized to
    ///   budget).
    /// - `HardStop` → emit `BudgetExceeded` + set `budget_stopped` (the run
    ///   halts at the cap — the load-bearing safety primitive; gotcha #66).
    async fn dispatch_budget_actions(
        &self,
        actions: Vec<ThresholdAction>,
        model: &str,
        feedback: &mut TurnFeedback,
    ) -> Result<(), SdkError> {
        let mut current_model = model.to_string();
        for action in actions {
            match action {
                ThresholdAction::Warn {
                    spent_usd,
                    cap_usd,
                    percent,
                    ..
                } => {
                    self.emit(AgentEvent::BudgetWarn {
                        spent_usd,
                        cap_usd,
                        percent,
                    })
                    .await?;
                }
                ThresholdAction::Downshift {
                    spent_usd,
                    cap_usd,
                    percent,
                    ..
                } => {
                    let remaining = RemainingBudget {
                        spent_usd,
                        cap_usd,
                        avg_task_cost_usd: None,
                    };
                    if let Some(next) = DefaultLadder::new().next_model(&current_model, remaining) {
                        self.emit(AgentEvent::BudgetDownshift {
                            from_model: current_model.clone(),
                            to_model: next.clone(),
                            reason: format!("budget downshift at {percent}% of ${cap_usd:.4} cap"),
                        })
                        .await?;
                        current_model = next.clone();
                        feedback.new_model = Some(next);
                    }
                }
                ThresholdAction::Suspend {
                    spent_usd, cap_usd, ..
                } => {
                    self.emit(AgentEvent::BudgetSuspended { spent_usd, cap_usd })
                        .await?;
                    feedback.budget_suspended = true;
                }
                ThresholdAction::HardStop {
                    spent_usd, cap_usd, ..
                } => {
                    self.emit(AgentEvent::BudgetExceeded { spent_usd, cap_usd })
                        .await?;
                    feedback.budget_stopped = true;
                }
            }
        }
        Ok(())
    }

    /// Walk every inline sub-agent declared in the framework (sessions
    /// with no children produce no events). For each child, run
    /// `narrow(parent_grants, proposed)`; on Ok emit `AgentSpawned`
    /// with `narrowed_from` populated; on Err emit
    /// `CapabilityViolation` and skip the spawn.
    ///
    /// v0.1 walks every inline agent except the session-root one (the
    /// root spawned at the smoke emission above). The full per-parent
    /// child-only walk lands at M07 (registered agents + spawn driver).
    async fn spawn_framework_subagents(
        &self,
        wiring: &CapabilityWiring,
        parent_runtime_id: &str,
    ) -> Result<(), SdkError> {
        let session_root_id = wiring.framework.session_root_agent.as_str();
        let parent_grants =
            parent_grants_for_agent(&wiring.framework, session_root_id).unwrap_or_default();
        for child in inline_agents(&wiring.framework) {
            // Skip the session root — it spawned at the smoke emission
            // above; the L2a walk handles its descendants.
            if child.id.as_str() == session_root_id {
                continue;
            }
            let proposed = capabilities_to_declarations(&child.capabilities);
            let narrowed_from_strs: Vec<String> = proposed
                .iter()
                .map(declaration_to_narrowed_from_str)
                .collect();
            match narrow(&parent_grants, &proposed) {
                Ok(_narrowed_grants) => {
                    self.emit(AgentEvent::AgentSpawned {
                        agent_id: child.id.to_string(),
                        agent_name: child.role.to_string(),
                        parent_id: Some(parent_runtime_id.to_string()),
                        session_id: self.session_id.as_string(),
                        narrowed_from: narrowed_from_strs,
                    })
                    .await?;
                }
                Err(err) => {
                    let bad_decl = match &err {
                        crate::capability::NarrowingError::CapabilityNotHeldByParent {
                            proposed: bad,
                        } => bad,
                    };
                    self.emit(AgentEvent::CapabilityViolation {
                        agent_id: child.id.to_string(),
                        capability_kind: kind_to_ref(bad_decl.kind),
                        requested_action: format!(
                            "spawn sub-agent '{}' with capability '{}' on '{}'",
                            child.id.as_str(),
                            format!("{:?}", bad_decl.kind).to_lowercase(),
                            *bad_decl.resource,
                        ),
                        declared_scope: format!(
                            "parent '{session_root_id}' grants do not subsume the proposed child capability"
                        ),
                    })
                    .await?;
                    self.await_capability_violation_hitl(&wiring.hitl_seam)
                        .await;
                    // Sub-agent is NOT spawned. Loop continues to the
                    // next declared child so a single widening attempt
                    // does not block the rest of the framework load.
                }
            }
        }
        Ok(())
    }

    /// Await the HITL response for a `capability_violation` / `tier_violation`
    /// event. Per ADR-0007 in-process seam: the renderer's modal
    /// resolves the seam via `respond_hitl(prompt_id, choice)`. The
    /// `prompt_id` used here is the runtime agent id concatenated with
    /// the session id — sufficient for v0.1 single-session correlation;
    /// M07+ may introduce per-violation UUIDs.
    ///
    /// On `Err` (`NotFound` / `TimedOut` / Cancelled), the SDK loop logs
    /// and continues — the violation event has already been emitted to
    /// the renderer; the missing HITL resolution is non-fatal at the
    /// SDK level (renderer surfaces the event regardless).
    async fn await_capability_violation_hitl(&self, hitl_seam: &Arc<HitlSeam>) {
        let prompt_id = format!("capability_violation:{}", self.session_id.as_string());
        if let Err(e) = hitl_seam
            .await_response(&prompt_id, HITL_DEFAULT_WAIT)
            .await
        {
            tracing::warn!(
                error = %e,
                prompt_id = %prompt_id,
                "capability_violation HITL await did not resolve cleanly; continuing"
            );
        }
    }

    /// Offer one `ProviderEvent::ToolUse` to the injected MCP-dispatch
    /// seam. Returns:
    /// - `Ok(None)` — NOT an MCP tool: pure fall-through to the existing
    ///   Stage A non-MCP L1 path (the caller continues into
    ///   `pipeline.next_event`).
    /// - `Ok(Some(Some(DispatchedTool)))` — MCP `Invoked`: the run loop
    ///   emitted the agent_id-correct `ToolInvoked`/`ToolResult` itself
    ///   (gotcha #68: `McpDispatchOutcome::Invoked` carries no
    ///   `agent_id`), and the returned [`DispatchedTool`] is fed back as
    ///   the next turn's `tool_result` by the multi-turn driver.
    /// - `Ok(Some(None))` — MCP handled but no tool result to feed back
    ///   (`Blocked` / `Ambiguous` / transport `Err`).
    ///
    /// CQ-2 (M07.D2 — maintainer-decided "surgical, type-level"; the
    /// M06.V CQ-2/reuse-5 finding). The match over [`McpDispatchOutcome`]
    /// is exhaustive with NO catch-all: `Invoked` is handled directly
    /// (agent_id-correct), `Blocked`/`Ambiguous` map through
    /// [`apply_renderable`] over a [`RenderableOutcome`] that
    /// structurally cannot represent `Invoked` — so the dead
    /// empty-`agent_id` `Invoked` branch in `apply_mcp_dispatch` (the
    /// D-frozen wire-test contract, kept byte-stable for the ADR-0011
    /// D-freeze) is unreachable from production, and a future fourth
    /// variant is a compile error here rather than a silent drop.
    async fn try_mcp_dispatch(
        &self,
        agent_id: &str,
        tool_use_id: &str,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<Option<Option<DispatchedTool>>, SdkError> {
        let Some(dispatch) = self.mcp_dispatch.as_ref() else {
            return Ok(None);
        };
        // `dispatch_if_mcp` takes the framework's `mcp_aliases`
        // (§5a explicit-alias override). The Framework stores it as a
        // HashMap; the seam signature takes a BTreeMap (stable order).
        // No wiring (or no framework) ⇒ no aliases ⇒ empty map.
        let aliases: BTreeMap<String, String> = self
            .capability_wiring
            .as_ref()
            .map(|w| {
                w.framework
                    .mcp_aliases
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let Some(result) = dispatch
            .dispatch_if_mcp(agent_id, tool_name, args.clone(), &aliases)
            .await
        else {
            // Not an MCP tool — pure fall-through to the Stage A
            // non-MCP L1 path (caller continues into pipeline.next_event).
            return Ok(None);
        };

        // CQ-2: exhaustive, NO `_ =>` catch-all.
        match result {
            Ok(McpDispatchOutcome::Invoked {
                server,
                tool,
                value,
            }) => {
                self.emit(AgentEvent::ToolInvoked {
                    agent_id: agent_id.to_string(),
                    tool_name: tool.clone(),
                    source: ToolSource::Mcp,
                    server: Some(server),
                    input: args.clone(),
                })
                .await?;
                self.emit(AgentEvent::ToolResult {
                    agent_id: agent_id.to_string(),
                    tool_name: tool.clone(),
                    output: value.clone(),
                    duration_ms: 0,
                    tokens_in: None,
                    tokens_out: None,
                })
                .await?;
                Ok(Some(Some(DispatchedTool {
                    id: tool_use_id.to_string(),
                    name: tool,
                    input: args,
                    value,
                })))
            }
            Ok(McpDispatchOutcome::Blocked {
                agent_id: blocked_agent,
                server,
                tool,
                reason,
            }) => {
                let outcome = RenderableOutcome::Blocked {
                    agent_id: blocked_agent,
                    server,
                    tool,
                    reason,
                };
                let needs_hitl = renderable_needs_hitl(&outcome);
                for ev in apply_renderable(outcome, args) {
                    self.emit(ev).await?;
                }
                if needs_hitl {
                    if let Some(wiring) = self.capability_wiring.as_ref() {
                        self.await_capability_violation_hitl(&wiring.hitl_seam)
                            .await;
                    }
                }
                Ok(Some(None))
            }
            Ok(McpDispatchOutcome::Ambiguous { name, candidates }) => {
                for ev in apply_renderable(RenderableOutcome::Ambiguous { name, candidates }, args)
                {
                    self.emit(ev).await?;
                }
                Ok(Some(None))
            }
            Err(e) => {
                self.emit(mcp_dispatch_error_event(agent_id, tool_name, &e))
                    .await?;
                Ok(Some(None))
            }
        }
    }

    /// Execute one in-process built-in `ToolUse` (`Read`/`Write`) under
    /// capability scope and emit the agent-correct events. Returns
    /// `Some(DispatchedTool)` when there is a result to feed back —
    /// executed OR errored (both continue the multi-turn loop); `None`
    /// when the capability check blocked the op (no execution;
    /// `CapabilityViolation`/`TierViolation` emitted + HITL routed —
    /// mirrors the MCP `Blocked` shape so built-in and MCP converge on one
    /// feedback contract).
    async fn dispatch_builtin(
        &self,
        wiring: &CapabilityWiring,
        agent_id: &str,
        tool_use_id: &str,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<Option<DispatchedTool>, SdkError> {
        match builtin_tools::execute_builtin(&wiring.enforcer, agent_id, tool_name, &input) {
            Ok(value) => Ok(Some(
                self.emit_builtin_result(agent_id, tool_use_id, tool_name, input, value)
                    .await?,
            )),
            // The op ran but failed (malformed input / IO). Feed an error
            // tool_result back so the model can recover — the loop must not
            // break on a recoverable tool error.
            Err(BuiltinExecError::Op(msg)) => Ok(Some(
                self.emit_builtin_result(
                    agent_id,
                    tool_use_id,
                    tool_name,
                    input,
                    serde_json::json!({ "error": msg }),
                )
                .await?,
            )),
            // The capability check blocked the op — it never ran. Emit the
            // violation, route HITL, feed nothing back.
            Err(BuiltinExecError::Capability(err)) => {
                self.emit_builtin_capability_violation(agent_id, tool_name, &err)
                    .await?;
                self.await_capability_violation_hitl(&wiring.hitl_seam)
                    .await;
                Ok(None)
            }
        }
    }

    /// Handle one `LoadSkill` `ToolUse` (M08.7 rung 3; ADR-0027). Reads
    /// the agent's already-resolved skill body under the `allowed_skills`
    /// gate, emits `SkillLoaded` + the `ToolInvoked`/`ToolResult` pair, and
    /// returns a [`DispatchedTool`] so the body rides back as a
    /// `tool_result` (persisting in the message history across turns — the
    /// rung-1 feedback contract). A skill not in `allowed_skills` (or with
    /// no resolved body) feeds an error `tool_result` back so the loop
    /// survives; gap-event routing for a genuinely missing capability is
    /// rung 4 (`request_capability`), not rung 3.
    async fn dispatch_load_skill(
        &self,
        wiring: &CapabilityWiring,
        agent_id: &str,
        tool_use_id: &str,
        input: serde_json::Value,
    ) -> Result<Option<DispatchedTool>, SdkError> {
        let skill_name = input
            .get("skill_name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let allowed = agent_allowed_skills(&wiring.framework, agent_id);
        let value = match load_skill::load_skill(skill_name, &allowed, &wiring.resolved_skills) {
            Ok(loaded) => {
                self.emit(AgentEvent::SkillLoaded {
                    agent_id: agent_id.to_string(),
                    skill_name: loaded.name.clone(),
                    mode: None,
                })
                .await?;
                serde_json::json!({ "skill": loaded.name, "body": loaded.body })
            }
            Err(load_skill::LoadSkillError::NotAllowed(s)) => {
                serde_json::json!({ "error": format!("skill '{s}' is not in this agent's allowed_skills") })
            }
            Err(load_skill::LoadSkillError::NotResolved(s)) => {
                serde_json::json!({ "error": format!("skill '{s}' has no resolved body") })
            }
        };
        Ok(Some(
            self.emit_builtin_result(
                agent_id,
                tool_use_id,
                load_skill::LOAD_SKILL_TOOL,
                input,
                value,
            )
            .await?,
        ))
    }

    /// Handle one `request_capability` `ToolUse` (M08.7 rung 4; spec §4b
    /// Layer 2). Parses the meta-tool input
    /// (`{capability_name, capability_kind, reason}`) into a
    /// [`RequestCapabilityInvocation`] — the requesting `agent_id` is the
    /// dispatch context's id (gotcha #68: the requesting agent, not a
    /// hardcoded one) — routes it to the built
    /// [`handle_request_capability`] (REUSED, not reimplemented), and
    /// re-emits the resulting `*Missing` gap event through [`Self::emit`]
    /// so it persists in the drone signal chain (recoverable per §1b).
    ///
    /// Returns [`RequestCapabilityDisposition::Suspended`] when the handler
    /// emitted a gap (the turn loop must suspend), or
    /// [`RequestCapabilityDisposition::Continue`] carrying an error
    /// `tool_result` when the invocation was malformed (empty name /
    /// justification) so the model can recover without suspending.
    async fn dispatch_request_capability(
        &self,
        agent_id: &str,
        tool_use_id: &str,
        input: serde_json::Value,
    ) -> Result<RequestCapabilityDisposition, SdkError> {
        let name = input
            .get("capability_name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let kind = parse_capability_kind(
            input
                .get("capability_kind")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        let justification = input
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let invocation = RequestCapabilityInvocation {
            agent_id: agent_id.to_string(),
            kind,
            name,
            justification,
        };
        let collector = GapEventCollector::default();
        match handle_request_capability(invocation, &collector).await {
            Ok(RequestCapabilityResult::Pending) => {
                for event in collector.take() {
                    self.emit(event).await?;
                }
                Ok(RequestCapabilityDisposition::Suspended)
            }
            // Malformed request: the handler refuses to emit an
            // unconstructable gap. Feed an error tool_result back so the
            // model can fix its call — NOT a suspend (the session is healthy).
            Err(err) => {
                let value = serde_json::json!({ "error": err.to_string() });
                let dispatched = self
                    .emit_builtin_result(
                        agent_id,
                        tool_use_id,
                        request_capability::REQUEST_CAPABILITY_TOOL,
                        input,
                        value,
                    )
                    .await?;
                Ok(RequestCapabilityDisposition::Continue(dispatched))
            }
        }
    }

    /// Emit the `ToolInvoked` + `ToolResult` pair for an executed (or
    /// recoverably-errored) built-in and build the [`DispatchedTool`] the
    /// multi-turn loop feeds back.
    async fn emit_builtin_result(
        &self,
        agent_id: &str,
        tool_use_id: &str,
        tool_name: &str,
        input: serde_json::Value,
        value: serde_json::Value,
    ) -> Result<DispatchedTool, SdkError> {
        self.emit(AgentEvent::ToolInvoked {
            agent_id: agent_id.to_string(),
            tool_name: tool_name.to_string(),
            source: ToolSource::Builtin,
            server: None,
            input: input.clone(),
        })
        .await?;
        self.emit(AgentEvent::ToolResult {
            agent_id: agent_id.to_string(),
            tool_name: tool_name.to_string(),
            output: value.clone(),
            duration_ms: 0,
            tokens_in: None,
            tokens_out: None,
        })
        .await?;
        Ok(DispatchedTool {
            id: tool_use_id.to_string(),
            name: tool_name.to_string(),
            input,
            value,
        })
    }

    /// Map a built-in's capability `Err` to the agent-correct
    /// `CapabilityViolation` / `TierViolation` event — mirroring the
    /// `EventPipeline::translate_tool_use` copy so the renderer handles
    /// built-in and pipeline denials identically.
    async fn emit_builtin_capability_violation(
        &self,
        agent_id: &str,
        tool_name: &str,
        err: &CapabilityError,
    ) -> Result<(), SdkError> {
        let event = match err {
            CapabilityError::Denied {
                reason,
                agent_id: denied_id,
            } => {
                let declared_scope = match reason {
                    DenyReason::NoDeclarations => "no capabilities declared",
                    DenyReason::NoMatchingGrant => "declared grants do not cover this request",
                };
                AgentEvent::CapabilityViolation {
                    agent_id: pick_agent_id(denied_id, agent_id),
                    capability_kind: builtin_kind_ref(tool_name),
                    requested_action: format!("invoke built-in tool '{tool_name}'"),
                    declared_scope: declared_scope.to_string(),
                }
            }
            CapabilityError::TierForbidden {
                agent_id: tid,
                tier,
                capability_kind,
            } => AgentEvent::TierViolation {
                agent_id: pick_agent_id(tid, agent_id),
                tier: tier_to_ref(*tier),
                capability_kind: kind_to_ref(*capability_kind),
                attempted_action: format!("invoke built-in tool '{tool_name}' under {tier:?} tier"),
            },
        };
        self.emit(event).await
    }

    async fn emit(&self, event: AgentEvent) -> Result<(), SdkError> {
        // Minimal observability unblock (M08.7.A): surface each agent
        // event's salient payload at debug so a `RUST_LOG=debug` run is
        // watchable in the log. This is the IRL-only unblock — the full
        // in-app agent-output view (live-graph execution surface) is
        // M08.7b, not this. Off by default; never on the user's screen.
        log_event_debug(&event);
        // Persist BEFORE the renderer send so a slow/full renderer
        // channel cannot starve the drone signal sink. Additive: the
        // unchanged `event_tx.send` below is the in-mem-bus / renderer
        // sink (spec §11); `persist_signal` restores the drone /
        // signals+VDR / plan-projector sinks (M06.5 IRL 🔴-2).
        self.persist_signal(&event).await;
        self.event_tx
            .send(event)
            .await
            .map_err(|_| SdkError::EventChannelClosed)
    }

    /// Persist one `AgentEvent` to the drone `signals` table via the
    /// existing [`DroneClient::write_signal`] IPC, under the run's
    /// [`SessionId`].
    ///
    /// Best-effort by contract: a transient drone-IPC failure is logged
    /// and swallowed so it never aborts the agent run (M06.5 IRL 🔴-2;
    /// the renderer sink — the unchanged `event_tx.send` in
    /// [`Self::emit`] — keeps working through a drone blip). The drone's
    /// `handle_write_signal` runs the VDR + plan projectors in the same
    /// transaction, so this single call site restores three of spec
    /// §11's four sinks; the fourth (in-mem bus) is `event_tx.send`.
    ///
    /// The `AgentEvent → (signal_id, kind, event, context_type, payload)`
    /// mapping is derived from the established `write_signal` call sites
    /// (`crates/runtime-main/tests/recovery_lifecycle.rs:142-191`) + the
    /// `signals` table columns
    /// (`crates/runtime-drone/migrations/000_initial.sql`), not invented:
    /// `payload` is the serde-tagged event object (`AgentEvent` is
    /// `#[serde(tag = "type")]`, matching recovery's
    /// `json!({"type": …, …})` shape); `event` is that tag; `kind` is
    /// the coarse `signals.type` category; `session_id` is the run's id
    /// (same value carried by `AgentSpawned`, gotcha: a mismatched
    /// session id is as broken as no signal for recovery/replay).
    async fn persist_signal(&self, event: &AgentEvent) {
        let payload = match serde_json::to_value(event) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "signal payload serialize failed; skipping persist");
                return;
            }
        };
        let event_name = payload
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        if let Err(e) = self
            .drone_client
            .write_signal(
                Uuid::new_v4().to_string(),
                self.session_id.as_string(),
                signal_kind(&event_name).to_string(),
                event_name,
                "agent_loop".to_string(),
                payload,
            )
            .await
        {
            tracing::warn!(error = %e, "write_signal failed; continuing agent run");
        }
    }
}

/// Log one `AgentEvent`'s salient payload at `debug` (M08.7.A IRL-only
/// observability unblock; off by default, surfaced by `RUST_LOG=debug`).
///
/// Tool events log the tool name + the result that flows back to the model
/// (file content or error); agent text events log the reply text. This is
/// the minimal "watch a run in the log" affordance — the in-app agent-output
/// view is M08.7b. Other event types are not logged here (lifecycle /
/// capability events already surface through their own `tracing` warns).
fn log_event_debug(event: &AgentEvent) {
    match event {
        AgentEvent::ToolInvoked {
            tool_name, input, ..
        } => tracing::debug!(tool = %tool_name, input = %input, "tool invoked"),
        AgentEvent::ToolResult {
            tool_name, output, ..
        } => tracing::debug!(tool = %tool_name, output = %output, "tool result"),
        AgentEvent::StreamText { text, .. } => tracing::debug!(text = %text, "agent stream text"),
        AgentEvent::AgentComplete { result, .. } => {
            tracing::debug!(result = %result, "agent complete");
        }
        _ => {}
    }
}

/// Coarse `signals.type` category for an `AgentEvent`'s serde tag.
///
/// Mirrors the kinds the established `write_signal` call sites use
/// (`tests/recovery_lifecycle.rs`: `tool_invoked`→`"tool"`,
/// `plan_created`/`task_started`→`"agent"`, `decision`→`"decision"`)
/// and what `runtime_drone::vdr::is_projection_eligible` keys on
/// (`decision` | `verify` project to the VDR; everything else is a
/// plain signal). Derived, not invented.
fn signal_kind(event_name: &str) -> &'static str {
    if event_name.starts_with("tool_") {
        "tool"
    } else if event_name.starts_with("decision") {
        "decision"
    } else if event_name.starts_with("verify_") {
        "verify"
    } else {
        "agent"
    }
}

const fn kind_to_ref(
    k: runtime_core::generated::capability::CapabilityKind,
) -> runtime_core::event::CapabilityKindRef {
    use runtime_core::event::CapabilityKindRef;
    use runtime_core::generated::capability::CapabilityKind;
    match k {
        CapabilityKind::Read => CapabilityKindRef::Read,
        CapabilityKind::Write => CapabilityKindRef::Write,
        CapabilityKind::Exec => CapabilityKindRef::Exec,
        CapabilityKind::Network => CapabilityKindRef::Network,
        CapabilityKind::ProcessSpawn => CapabilityKindRef::ProcessSpawn,
    }
}

/// The `CapabilityKindRef` a built-in file tool's denial reports: `Read`
/// for `Read`, `Write` for `Write` (the kind the executor's request
/// declaration carried — distinct from the `Exec` the pre-rung-1
/// `ToolNotFound` pipeline fallback emits).
fn builtin_kind_ref(tool_name: &str) -> CapabilityKindRef {
    if tool_name == builtin_tools::WRITE_TOOL {
        CapabilityKindRef::Write
    } else {
        CapabilityKindRef::Read
    }
}

const fn tier_to_ref(t: Tier) -> TierRef {
    match t {
        Tier::Novice => TierRef::Novice,
        Tier::Promoted => TierRef::Promoted,
    }
}

/// The `allowed_skills` declared by the framework agent matching
/// `agent_id` — the `LoadSkill` capability gate (M08.7 rung 3). Empty when
/// the agent is not found or declares none.
fn agent_allowed_skills(framework: &FrameworkRef, agent_id: &str) -> Vec<String> {
    inline_agents(framework)
        .into_iter()
        .find(|a| a.id.as_str() == agent_id)
        .map(|a| a.allowed_skills.clone())
        .unwrap_or_default()
}

/// Build the session [`BudgetEnforcer`] from a framework's `budget` block
/// (M08.7 rung 5). Maps `session_usd_cap` / `framework_usd_cap` to scope
/// caps and the four percent thresholds (`NonZeroU64`) to the enforcer's
/// args. Returns `None` when the framework configures no cap — a
/// budget-less framework runs with no enforcement (byte-stable with the
/// pre-rung-5 loop). REUSES the built enforcer (CLAUDE.md scope lock); the
/// budget model is not rebuilt here.
fn build_session_budget(framework: &Framework) -> Option<BudgetEnforcer> {
    let budget = framework.budget.as_ref()?;
    let mut caps = Vec::new();
    if let Some(cap) = budget.session_usd_cap {
        caps.push(BudgetScopeCap::session(cap));
    }
    if let Some(cap) = budget.framework_usd_cap {
        caps.push(BudgetScopeCap::framework(cap));
    }
    if caps.is_empty() {
        return None;
    }
    let (warn, downshift, suspend, hard_stop) =
        budget
            .actions
            .as_ref()
            .map_or((None, None, None, None), |a| {
                (
                    a.warn_at_percent.and_then(|n| u32::try_from(n.get()).ok()),
                    a.downshift_at_percent
                        .and_then(|n| u32::try_from(n.get()).ok()),
                    a.hitl_at_percent.and_then(|n| u32::try_from(n.get()).ok()),
                    a.hard_stop_at_percent
                        .and_then(|n| u32::try_from(n.get()).ok()),
                )
            });
    Some(BudgetEnforcer::new(
        caps, warn, downshift, suspend, hard_stop,
    ))
}

/// Prefer the enforcer-carried agent id when present; fall back to the
/// dispatch agent id (mirrors the `audit_check_result` convention).
fn pick_agent_id(carried: &str, fallback: &str) -> String {
    if carried.is_empty() {
        fallback.to_string()
    } else {
        carried.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{
        AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
        ProviderSupport,
    };
    use async_trait::async_trait;
    use futures::stream::BoxStream;

    #[test]
    fn signal_kind_maps_each_coarse_category() {
        // Pins the derived AgentEvent→signals.type mapping (M06.5
        // 🔴-2). The assembled smoke path only exercises the "agent"
        // arm (agent_spawned/stream_text/agent_complete); these pin
        // the tool/decision/verify arms the M07 loop will hit so a
        // mis-categorized signal (e.g. a decision not reaching the
        // VDR projector, which keys on type="decision") is caught.
        assert_eq!(signal_kind("tool_invoked"), "tool");
        assert_eq!(signal_kind("tool_result"), "tool");
        assert_eq!(signal_kind("decision"), "decision");
        assert_eq!(signal_kind("decision_record"), "decision");
        assert_eq!(signal_kind("verify_passed"), "verify");
        assert_eq!(signal_kind("agent_spawned"), "agent");
        assert_eq!(signal_kind("stream_text"), "agent");
        assert_eq!(signal_kind("unknown"), "agent");
    }

    #[test]
    fn session_id_is_unique() {
        let a = SessionId::new();
        let b = SessionId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn session_id_serializes_as_string() {
        let s = SessionId::new();
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.starts_with('"') && json.ends_with('"'));
        assert_eq!(json.matches('-').count(), 4, "uuid hyphenation: {json}");
    }

    #[test]
    fn session_id_default_is_fresh() {
        let a = SessionId::default();
        let b = SessionId::default();
        assert_ne!(a, b, "Default impl must mint a new UUID each call");
    }

    /// In-process stub provider used to exercise the production
    /// `run_agent` wrapper without crossing reqwest. Returns a fixed
    /// 2-event sequence (`TextDelta` + `MessageStop`).
    struct InlineStub;

    #[async_trait]
    impl LLMProvider for InlineStub {
        #[allow(
            clippy::unnecessary_literal_bound,
            reason = "trait method returns &str by signature; literal &'static str must reborrow"
        )]
        fn name(&self) -> &str {
            "inline-stub"
        }
        fn supports(&self) -> ProviderSupport {
            ProviderSupport {
                tool_use: false,
                streaming: true,
                thinking: false,
            }
        }
        async fn stream(
            &self,
            _config: AgentConfig,
        ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
            Ok(Box::pin(futures::stream::iter(vec![
                ProviderEvent::TextDelta { text: "hi".into() },
                ProviderEvent::MessageStop {
                    stop_reason: "end_turn".into(),
                    total_tokens: None,
                },
            ])))
        }
        async fn count_tokens(&self, _m: &[Message]) -> Result<u64, ProviderError> {
            Ok(0)
        }
        async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
            Ok(Vec::new())
        }
        fn estimate_cost(&self, _b: &CostBreakdown, _m: &str) -> f64 {
            0.0
        }
    }

    #[tokio::test]
    async fn run_agent_drives_provider_stream_to_completion() {
        let provider = Arc::new(InlineStub);
        let drone = Arc::new(DroneClient::noop());
        let (tx, mut rx) = mpsc::channel(8);
        let sdk = AgentSdk::new(provider, tx, drone, SessionId::new());
        let config = AgentConfig {
            model: "x".into(),
            messages: vec![],
            max_tokens: 16,
            temperature: None,
            system_prompt: None,
            tools: vec![],
        };
        sdk.run_agent(config).await.expect("run_agent ok");
        drop(sdk);
        let mut events = Vec::new();
        while let Some(e) = rx.recv().await {
            events.push(e);
        }
        assert!(events
            .iter()
            .any(|e| matches!(e, AgentEvent::AgentSpawned { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, AgentEvent::AgentComplete { .. })));
    }

    #[tokio::test]
    async fn inline_stub_trait_methods_smoke() {
        // The InlineStub fixture above implements every LLMProvider
        // method to satisfy the trait. This test exercises each so the
        // fixture itself participates in the safety-primitive coverage
        // measurement (the lib-test compilation includes mod tests).
        let p = InlineStub;
        assert_eq!(p.name(), "inline-stub");
        let s = p.supports();
        assert!(s.streaming);
        assert!(!s.tool_use);
        assert!(!s.thinking);
        assert_eq!(p.count_tokens(&[]).await.unwrap(), 0);
        assert!(p.list_models().await.unwrap().is_empty());
        assert!((p.estimate_cost(&CostBreakdown::default(), "x") - 0.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn smoke_session_root_agent_is_named_smoke() {
        // M08.5.C.fix guard: the no-wiring path (`AgentSdk::new`, the
        // real smoke session — `src-tauri/src/commands.rs:233`
        // `run_smoke_session_with`) MUST keep emitting
        // `agent_name: "smoke"` on its root `AgentSpawned`. The
        // C.fix derivation applies ONLY when `capability_wiring` is
        // present; this test pins the byte-stability of the smoke
        // path against any future regression that derives the name
        // unconditionally.
        let provider = Arc::new(InlineStub);
        let drone = Arc::new(DroneClient::noop());
        let (tx, mut rx) = mpsc::channel(8);
        let sdk = AgentSdk::new(provider, tx, drone, SessionId::new());
        let config = AgentConfig {
            model: "x".into(),
            messages: vec![],
            max_tokens: 16,
            temperature: None,
            system_prompt: None,
            tools: vec![],
        };
        sdk.run_agent(config).await.expect("run_agent ok");
        drop(sdk);
        let mut root_name: Option<String> = None;
        while let Some(e) = rx.recv().await {
            if let AgentEvent::AgentSpawned {
                agent_name,
                parent_id: None,
                ..
            } = &e
            {
                root_name = Some(agent_name.clone());
                break;
            }
        }
        assert_eq!(
            root_name.as_deref(),
            Some("smoke"),
            "the un-wired smoke session must label its root agent \"smoke\" (byte-stable)"
        );
    }

    #[tokio::test]
    async fn end_of_stream_flushes_residual_text_buffer() {
        // Stream ends WITHOUT MessageStop, leaving text in the buffer.
        // The final `pipeline.flush()` must emit a StreamText.
        let provider = Arc::new(InlineStub);
        let drone = Arc::new(DroneClient::noop());
        let (tx, mut rx) = mpsc::channel(8);
        let sdk = AgentSdk::new(provider, tx, drone, SessionId::new());
        let stream = futures::stream::iter(vec![ProviderEvent::TextDelta {
            text: "residual".into(),
        }]);
        sdk.run_agent_with_provider_stream(stream)
            .await
            .expect("run ok");
        drop(sdk);
        let mut got_text = false;
        while let Some(e) = rx.recv().await {
            if let AgentEvent::StreamText { text, .. } = &e {
                if text == "residual" {
                    got_text = true;
                }
            }
        }
        assert!(got_text, "end-of-stream flush must emit residual buffer");
    }
}
