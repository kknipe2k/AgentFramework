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
use runtime_core::event::{AgentEvent, ToolSource};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::event_pipeline::{EnforcementContext, EventPipeline};
use super::mcp_dispatch::{
    apply_mcp_dispatch, mcp_dispatch_error_event, outcome_needs_hitl, McpDispatchOutcome,
    McpToolDispatch,
};
use crate::capability::{narrow, CapabilityEnforcer};
use crate::drone_ipc::{DroneClient, DroneIpcError};
use crate::framework_loader::{
    capabilities_to_declarations, declaration_to_narrowed_from_str, inline_agents,
    parent_grants_for_agent, FrameworkRef,
};
use crate::hitl::HitlSeam;
use crate::providers::{AgentConfig, LLMProvider, ProviderError, ProviderEvent};

/// Default wait for HITL responses on capability / tier violations
/// surfaced by the L1 wire-up. Long enough that a user can read the
/// modal + decide; short enough that a forgotten prompt does not hang
/// the SDK loop indefinitely. Matches the M04.E `HitlSeam` default
/// budgets used elsewhere.
const HITL_DEFAULT_WAIT: Duration = Duration::from_secs(3600);

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
}

impl CapabilityWiring {
    /// Construct from the three Arc references the Tauri shell layer
    /// already holds at session start.
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
        }
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

    /// Production entry point. Constructs the provider stream and delegates
    /// to [`Self::run_agent_with_provider_stream`].
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::Provider`] if the provider's `stream` call
    /// fails; otherwise propagates errors from
    /// [`Self::run_agent_with_provider_stream`].
    pub async fn run_agent(&self, config: AgentConfig) -> Result<(), SdkError> {
        let stream = self.provider.stream(config).await?;
        self.run_agent_with_provider_stream(stream).await
    }

    /// Test-seam variant. Accepts any pre-built `ProviderEvent` stream.
    ///
    /// Emits an `AgentSpawned` first, then drives the pipeline until the
    /// stream ends, flushes any buffered text, and returns. The drone
    /// receives one `SnapshotNow` at the start.
    ///
    /// When constructed via [`Self::with_capability_wiring`], also walks
    /// the framework's inline sub-agents at session start (per the
    /// M06.A L2a wire-up) before the streaming loop begins.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::EventChannelClosed`] if the receiver was
    /// dropped, or [`SdkError::Drone`] if the snapshot trigger failed.
    pub async fn run_agent_with_provider_stream<S>(&self, mut stream: S) -> Result<(), SdkError>
    where
        S: Stream<Item = ProviderEvent> + Unpin,
    {
        // M06.A wire-up: when the framework is present, the runtime
        // agent_id IS the framework's `session_root_agent` id so the
        // enforcer's grants (keyed by framework agent id) match the
        // dispatch path's runtime id. The pre-M06 UUID seed remains
        // for un-wired smoke sessions (no framework available).
        let agent_id = self.capability_wiring.as_ref().map_or_else(
            || format!("agent_{}", Uuid::new_v4()),
            |w| w.framework.session_root_agent.clone(),
        );
        // The smoke / streaming-only spawn site predates the L2a wire-up
        // (M06 Stage A); top-level agents have no parent grants to narrow
        // against, so `narrowed_from` is empty here. The framework-walk
        // spawn site below (`spawn_framework_subagents`) populates it for
        // every sub-agent that flowed through `narrow()`.
        self.emit(AgentEvent::AgentSpawned {
            agent_id: agent_id.clone(),
            agent_name: "smoke".to_string(),
            parent_id: None,
            session_id: self.session_id.as_string(),
            narrowed_from: Vec::new(),
        })
        .await?;

        // Trigger a SnapshotNow on task start. M02 single-turn only — one
        // task per session — so this fires once.
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
        let mut pipeline = self.capability_wiring.as_ref().map_or_else(
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
        while let Some(provider_event) = stream.next().await {
            // M06.F (ADR-0010 + ADR-0011): when an MCP-dispatch seam is
            // injected, a `ProviderEvent::ToolUse` is offered to it
            // FIRST. `dispatch_if_mcp` returning `None` (not an MCP
            // tool) falls through to the existing Stage A non-MCP L1
            // path below, unchanged.
            if self.mcp_dispatch.is_some() {
                if let ProviderEvent::ToolUse { name, input, .. } = &provider_event {
                    if self
                        .try_mcp_dispatch(&agent_id, name, input.clone())
                        .await?
                    {
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
        for agent_event in pipeline.flush() {
            self.emit(agent_event).await?;
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
    /// seam. Returns `Ok(true)` when the call was an MCP tool and the
    /// run loop fully handled it (the event must NOT also flow through
    /// the Stage A pipeline); `Ok(false)` when it is not an MCP tool
    /// (pure fall-through to the existing Stage A non-MCP L1 path).
    ///
    /// ADR-0010 dependency-inversion seam; ADR-0011 scopes this to the
    /// SDK side. `dispatch_if_mcp` returning `None` (not an MCP tool) is
    /// a pure fall-through (`Ok(false)`). `Some(Ok(Invoked))` → the run
    /// loop emits the success `ToolInvoked`/`ToolResult` itself with the
    /// `agent_id` it holds (gotcha #68: `McpDispatchOutcome::Invoked`
    /// carries no `agent_id`; `apply_mcp_dispatch`'s empty-agent_id Invoked
    /// branch is the D-frozen wire-test contract only, never shipped to
    /// the renderer). `Some(Ok(Blocked|Ambiguous))` → the D-frozen
    /// `apply_mcp_dispatch` mapping (Blocked carries its own `agent_id`;
    /// Ambiguous emits `ToolAliasAmbiguous`, no `agent_id` field — so no
    /// empty-agent_id reaches the renderer either way), and Blocked
    /// routes the existing `on_capability_violation` HITL trigger
    /// (ADR-0007, no new seam — mirrors the Stage A violation routing).
    /// `Some(Err)` → `mcp_dispatch_error_event` (a `ToolError`).
    async fn try_mcp_dispatch(
        &self,
        agent_id: &str,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<bool, SdkError> {
        let Some(dispatch) = self.mcp_dispatch.as_ref() else {
            return Ok(false);
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
            return Ok(false);
        };

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
                    input: args,
                })
                .await?;
                self.emit(AgentEvent::ToolResult {
                    agent_id: agent_id.to_string(),
                    tool_name: tool,
                    output: value,
                    duration_ms: 0,
                    tokens_in: None,
                    tokens_out: None,
                })
                .await?;
            }
            Ok(outcome) => {
                let needs_hitl = outcome_needs_hitl(&outcome);
                for ev in apply_mcp_dispatch(outcome, args) {
                    self.emit(ev).await?;
                }
                if needs_hitl {
                    if let Some(wiring) = self.capability_wiring.as_ref() {
                        self.await_capability_violation_hitl(&wiring.hitl_seam)
                            .await;
                    }
                }
            }
            Err(e) => {
                self.emit(mcp_dispatch_error_event(agent_id, tool_name, &e))
                    .await?;
            }
        }
        Ok(true)
    }

    async fn emit(&self, event: AgentEvent) -> Result<(), SdkError> {
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
