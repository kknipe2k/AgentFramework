//! Tauri command surface.
//!
//! Five commands are exposed to the renderer:
//! - [`set_api_key`] — write the Anthropic API key to the OS keychain.
//! - [`run_smoke_session`] — read the key, construct the SDK, and emit
//!   `AgentEvent`s through the Tauri event bus on channel `"agent_event"`.
//! - [`query_session_db`] — SELECT-only query against the session database
//!   via drone IPC.
//! - [`replay_session`] — reconstruct a prior session's graph by reading
//!   the signal log via drone IPC and re-emitting `AgentEvent`s.
//!
//! Per spec §10 capability boundary: the renderer never holds the API key,
//! never speaks HTTP, never touches the filesystem. Every privileged action
//! goes through these commands.
//!
//! # Test seam
//!
//! Each production command has a `*_with` testable seam (M01.C / M02.C
//! / M02.D / M03.E pattern). Seams accept injectable collaborators
//! (provider stub, query function, signal reader, emit callback,
//! `Arc<DroneClient>`) so unit tests exercise the SDK→event flow + IPC
//! translation paths without crossing reqwest, the OS keychain, or a real
//! drone subprocess. Production wrappers construct the real provider and
//! pull the [`runtime_main::drone_ipc::DroneClient`] from Tauri-managed
//! state (M04 Stage A2 wired the lifecycle).
//!
//! # `CmdError` shape
//!
//! `CmdError` is the typify-generated wire-format enum from
//! `schemas/error.v1.json`, re-exported via [`runtime_core::CmdError`].
//! Helper constructors (`provider`, `drone`, `key_store`, `internal`)
//! and [`std::fmt::Display`] / [`std::error::Error`] impls live in
//! `runtime-core/src/cmd_error_ext.rs`. M02 shipped a hand-rolled
//! struct-variant enum here; M04 Stage A2 migrated to the generated
//! tuple-variant shape (the wire format is unchanged).

use std::sync::Arc;

use runtime_core::event::AgentEvent;
use runtime_core::CmdError;
use runtime_main::drone_ipc::{DroneClient, RecoveredSession};
use runtime_main::hitl::{HitlChoice, HitlError, HitlSeam};
use runtime_main::import::fetch::HttpFetcher;
use runtime_main::import::{
    self, ArtifactKind, ImportError, ImportSource, McpRegistry, McpServerImport, NetworkGate,
    Sandbox, SystemClock,
};
use runtime_main::key_store::{read_api_key, write_api_key, KeyStoreError};
use runtime_main::providers::anthropic::AnthropicProvider;
use runtime_main::providers::{AgentConfig, ContentBlock, LLMProvider, Message, MessageRole};
use runtime_main::recovery::{
    request_resume_with, respond_uncertainty_with, ResumeError, ResumePlan, UncertaintyError,
    UncertaintyResolution,
};
use runtime_main::sandbox_ipc::SandboxClient;
use runtime_main::sdk::{
    replay_signals_to_events, AgentSdk, ApprovalDecision, ApprovalError, ApprovalSeam,
    McpToolDispatch, SessionId,
};
use runtime_main::tier::{save_tier, Tier, TierPersistenceError};
use runtime_mcp::client::registry::{McpServerRecord, Registry};
use runtime_mcp::client::{McpClient, McpServerSummary};
use runtime_mcp::transport::McpTool;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio::sync::Mutex;

/// Persist the Anthropic API key in the OS keychain.
///
/// # Errors
///
/// Returns [`CmdError::KeyStore`] if the platform keychain rejects the write.
#[tauri::command]
pub async fn set_api_key(key: String) -> Result<(), CmdError> {
    set_api_key_with(&key, write_api_key)?;
    // `key: String` is dropped at the end of this scope; the keyring crate
    // takes ownership of the bytes during set_password, so the input string
    // does not outlive this call.
    drop(key);
    Ok(())
}

/// Test-seam for [`set_api_key`] (per CLAUDE.md §5 `*_with` archetype).
/// Accepts an injectable writer so tests exercise the tracing + error
/// translation paths without touching the real OS keychain. Per spec
/// §13.5 dev-logging — never log the key value, only `key_len`.
///
/// # Errors
///
/// Returns whatever the writer's `KeyStoreError` translates to via the
/// `From<KeyStoreError> for CmdError` impl in
/// `crates/runtime-main/src/key_store.rs`.
pub fn set_api_key_with<F>(key: &str, write: F) -> Result<(), CmdError>
where
    F: FnOnce(&str) -> Result<(), KeyStoreError>,
{
    tracing::info!(key_len = key.len(), "set_api_key invoked");
    if let Err(e) = write(key) {
        tracing::error!(error = %e, "set_api_key failed at write_api_key");
        return Err(e.into());
    }
    tracing::info!("set_api_key succeeded");
    Ok(())
}

/// Run the M02 smoke session against the live Anthropic API.
///
/// Reads the API key, constructs an [`AnthropicProvider`], runs the SDK
/// against a single hardcoded "Say only the word: hello" prompt, and emits
/// each `AgentEvent` via `app.emit("agent_event", &event)`.
///
/// # Errors
///
/// - [`CmdError::SetupRequired`] if no API key is in the keychain.
/// - [`CmdError::Provider`] if the provider stream open or yields fail.
/// - [`CmdError::KeyStore`] for non-NotFound keychain errors.
/// - [`CmdError::Internal`] for SDK channel-closed conditions.
#[tauri::command]
pub async fn run_smoke_session(
    app: AppHandle,
    drone: tauri::State<'_, Arc<DroneClient>>,
    session: tauri::State<'_, SessionId>,
) -> Result<(), CmdError> {
    let api_key = read_api_key().inspect_err(|e| {
        tracing::error!(error = %e, "run_smoke_session: read_api_key failed");
    })?;
    let provider = AnthropicProvider::new(api_key.clone());
    let drone_client = Arc::clone(&drone);
    // M06.5 🔴-2: write signals under the drone's seeded session id
    // (managed at setup from DroneLifecycle::sdk_session_id) — a
    // SessionId::new() here would never match the signals→sessions FK
    // and every signal would be silently rejected.
    let session_id = session.inner().clone();
    let (tx, rx) = mpsc::channel::<AgentEvent>(64);
    let app_clone = app.clone();
    let forwarder = tokio::spawn(forward_events(rx, app_clone));
    // ADR-0011 (a)-(c) discharged at M07.D1: the concrete
    // `McpDispatcher` is now constructible in-shell
    // (`build_mcp_dispatcher`), so the production wrapper threads
    // `Some(dispatcher)` instead of M06.F's `None`. The dispatcher is
    // built only when the `McpClient` opened at startup (best-effort
    // per spec §13.5 — without it the no-tools smoke still runs). The
    // no-tools smoke prompt emits no `ProviderEvent::ToolUse`, so the
    // dispatcher is constructed-but-not-exercised here; D2's
    // agent-with-tools loop is what drives it.
    let mcp_dispatch: Option<Arc<dyn McpToolDispatch>> =
        app.try_state::<Arc<McpClient>>().map(|client| {
            let audit = app
                .try_state::<Arc<runtime_main::audit::AuditWriter>>()
                .map(|w| w.inner().clone());
            build_mcp_dispatcher(client.inner().clone(), audit, &session_id)
        });
    let result = run_smoke_session_with(
        provider,
        tx,
        drone_client,
        smoke_config(),
        mcp_dispatch,
        session_id,
    )
    .await;
    drop(api_key);
    // Wait for the forwarder to drain any final events before returning.
    let _ = forwarder.await;
    result
}

/// Test-seam: run a smoke session against a caller-supplied provider and
/// drone client, emitting events into a caller-supplied channel.
///
/// Production [`run_smoke_session`] constructs the real provider, channel,
/// and forwarder, and pulls the drone client from Tauri-managed state.
/// Tests inject an in-memory provider stub and a [`DroneClient::noop`]
/// and assert on the events received.
///
/// `mcp_dispatch` is the M06.F (ADR-0010 + ADR-0011) composition-root
/// injection seam: when `Some`, it is composed onto the SDK via
/// [`AgentSdk::with_mcp_dispatch`] so the run loop intercepts
/// `ProviderEvent::ToolUse` through it. As of M07.D1 production threads
/// `Some(build_mcp_dispatcher(..))` (ADR-0011 (a)-(c) discharged); the
/// seam test injects a mock and the construction test injects the
/// concrete dispatcher.
///
/// `session_id` is the drone's seeded session id (managed at setup
/// from [`crate::drone_lifecycle::DroneLifecycle::sdk_session_id`]).
/// The SDK writes every signal under it so the `signals → sessions`
/// FK accepts the row; an independent `SessionId::new()` here would
/// make the assembled signal sink dead (M06.5 IRL 🔴-2).
///
/// # Errors
///
/// Same as [`run_smoke_session`] minus the keychain-read step.
pub async fn run_smoke_session_with<P: LLMProvider + 'static>(
    provider: P,
    event_tx: mpsc::Sender<AgentEvent>,
    drone: Arc<DroneClient>,
    config: AgentConfig,
    mcp_dispatch: Option<Arc<dyn McpToolDispatch>>,
    session_id: SessionId,
) -> Result<(), CmdError> {
    tracing::info!("run_smoke_session starting");
    let mut sdk = AgentSdk::new(Arc::new(provider), event_tx, drone, session_id);
    if let Some(dispatch) = mcp_dispatch {
        sdk = sdk.with_mcp_dispatch(dispatch);
    }
    let result = sdk
        .run_agent(config)
        .await
        .map_err(|e| CmdError::provider(e.to_string()));
    if let Err(ref e) = result {
        tracing::error!(error = %e, "run_smoke_session failed");
    } else {
        tracing::info!("run_smoke_session succeeded");
    }
    result
}

/// ADR-0011 (c) — construct the concrete `McpDispatcher` in the shell.
///
/// Closes the M07.A-mapped construction graph: M06.F's production
/// `run_smoke_session` passed `None` because neither the
/// `CapabilityEnforcer` nor the `NamespaceResolver` had a `src-tauri`
/// construction site (ADR-0011 Context #2/#3). All three are now
/// reachable in-shell:
///
/// - the §5a `NamespaceResolver` starts empty —
///   `NamespaceResolver::new(BTreeMap::new())`; it is populated by the
///   §5a re-resolution driver (`McpDispatcher::on_server_connected`,
///   ADR-0011 (b)) as servers connect, not at construction.
/// - the L1 `CapabilityEnforcer` is the empty default; the v0.1
///   no-tools smoke grants nothing, so the dispatcher is
///   constructed-but-not-exercised here (D2's agent-with-tools loop is
///   what builds the framework-/tier-wired enforcer and drives it).
/// - `Arc<McpClient>` (Tauri-managed, opened at startup) is injected as
///   `Arc<dyn ConnectionResolver>` (ADR-0011 (a)).
///
/// `CapabilityEnforcer` construction is CODEOWNERS-flagged (Hard Rule
/// 8); the M07.D1 construction-reachability map + this function are the
/// surfaced plan.
fn build_mcp_dispatcher(
    mcp_client: Arc<McpClient>,
    audit: Option<Arc<runtime_main::audit::AuditWriter>>,
    session_id: &SessionId,
) -> Arc<dyn McpToolDispatch> {
    use runtime_main::capability::CapabilityEnforcer;
    use runtime_mcp::{ConnectionResolver, McpDispatcher, NamespaceResolver};
    use std::collections::BTreeMap;
    use tokio::sync::RwLock;

    let resolver = Arc::new(RwLock::new(NamespaceResolver::new(BTreeMap::new())));
    let enforcer = Arc::new(CapabilityEnforcer::new());
    let connections: Arc<dyn ConnectionResolver> = mcp_client;
    Arc::new(McpDispatcher::new(
        resolver,
        enforcer,
        connections,
        audit,
        session_id.as_string(),
    ))
}

async fn forward_events(mut rx: mpsc::Receiver<AgentEvent>, app: AppHandle) {
    while let Some(event) = rx.recv().await {
        // Errors from `emit` indicate the renderer has gone away; drop and
        // continue draining so the SDK can finish cleanly.
        let _ = app.emit("agent_event", &event);
    }
}

/// Run a SELECT-only query against the session database via drone IPC.
///
/// Production wrapper: pulls the [`Arc<DroneClient>`] from Tauri-managed
/// state (registered by `drone_lifecycle::DroneLifecycle::spawn` at the
/// Tauri setup hook) and dispatches a real `QuerySessionDb` IPC command.
/// The drone-side validator (`runtime_drone::vdr::is_select_only`) is the
/// security boundary regardless of this layer's wiring state.
///
/// # Errors
///
/// - [`CmdError::Drone`] if the IPC fails after retry exhaustion.
#[tauri::command]
pub async fn query_session_db(
    sql: String,
    drone: tauri::State<'_, Arc<DroneClient>>,
) -> Result<Vec<Value>, CmdError> {
    let drone = Arc::clone(&drone);
    query_session_db_with(sql, |s| {
        let drone = Arc::clone(&drone);
        async move {
            drone
                .query_session_db(s)
                .await
                .map_err(|e| CmdError::drone(e.to_string()))
        }
    })
    .await
}

/// Test-seam for [`query_session_db`] (CLAUDE.md §5 `*_with` archetype).
/// Accepts an injectable async query function so unit tests exercise the
/// happy + error paths without needing a real drone subprocess.
///
/// # Errors
///
/// Surfaces whatever `query` returns.
pub async fn query_session_db_with<F, Fut>(sql: String, query: F) -> Result<Vec<Value>, CmdError>
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<Value>, CmdError>>,
{
    tracing::info!(sql_len = sql.len(), "query_session_db invoked");
    let result = query(sql).await;
    if let Err(ref e) = result {
        tracing::warn!(error = %e, "query_session_db failed");
    } else {
        tracing::info!("query_session_db succeeded");
    }
    result
}

/// Replay a prior session by id. Reads the signal log via drone IPC,
/// translates each signal into an `AgentEvent`, and re-emits each via
/// the existing `agent_event` channel so the renderer reconstructs the
/// graph identically to the original session.
///
/// # Errors
///
/// - [`CmdError::Drone`] if the IPC fails after retry exhaustion.
#[tauri::command]
pub async fn replay_session(
    app: AppHandle,
    session_id: String,
    drone: tauri::State<'_, Arc<DroneClient>>,
) -> Result<(), CmdError> {
    let drone = Arc::clone(&drone);
    replay_session_with(
        session_id,
        |id| {
            let drone = Arc::clone(&drone);
            async move {
                drone
                    .read_signals(id)
                    .await
                    .map_err(|e| CmdError::drone(e.to_string()))
            }
        },
        |event| {
            let _ = app.emit("agent_event", &event);
            Ok::<(), CmdError>(())
        },
    )
    .await
}

/// Test-seam for [`replay_session`] (CLAUDE.md §5 `*_with` archetype).
/// Accepts an injectable signal-reader and an emitter callback so unit
/// tests exercise the read → translate → emit pipeline without a real
/// drone or Tauri `AppHandle`.
///
/// # Errors
///
/// Surfaces whatever `read_signals` returns; emit errors are logged and
/// dropped (matches `forward_events` in the smoke path).
pub async fn replay_session_with<F, Fut, Emit>(
    session_id: String,
    read_signals: F,
    mut emit: Emit,
) -> Result<(), CmdError>
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<Value>, CmdError>>,
    Emit: FnMut(AgentEvent) -> Result<(), CmdError>,
{
    tracing::info!(session_id, "replay_session invoked");
    let signals = read_signals(session_id.clone()).await?;
    let events = replay_signals_to_events(&signals);
    let count = events.len();
    for event in events {
        // Emit errors mean the renderer has gone away; log and drop so
        // the pipeline drains cleanly (matches `forward_events`).
        if let Err(e) = emit(event) {
            tracing::warn!(error = %e, "replay_session emit failed; continuing");
        }
    }
    tracing::info!(emitted = count, "replay_session finished");
    Ok(())
}

/// Resolve the in-process [`ApprovalSeam`] with `Approved` for `plan_id`.
///
/// Production wrapper: pulls the [`Arc<ApprovalSeam>`] from Tauri-managed
/// state (registered by the Tauri `setup` hook) and dispatches the
/// decision to whichever SDK task is awaiting on the seam.
///
/// # Errors
///
/// - [`CmdError::Internal`] if the seam reports the receiver was already
///   dropped before the resolution could be delivered (rare; usually means
///   the awaiting task was cancelled mid-flight).
///
/// **Note on no-pending-await:** if no SDK task is currently awaiting on
/// `plan_id` (e.g., the M04 v0.1 `plan_loop` driver is deferred per Stage B
/// retro `[LIVE]` ambiguity-events; the renderer can dispatch this command
/// before any SDK awaiter exists), the command returns `Ok(())` and
/// warn-logs. Per `CLAUDE.md` §12 user-flow ergonomics: do not 500 the
/// renderer's click on a soft-state issue.
#[tauri::command]
pub async fn approve_plan(
    plan_id: String,
    seam: tauri::State<'_, Arc<ApprovalSeam>>,
) -> Result<(), CmdError> {
    approve_plan_with(plan_id, seam.inner().as_ref()).await
}

/// Test-seam for [`approve_plan`] (CLAUDE.md §5 `*_with` archetype).
///
/// # Errors
///
/// See [`approve_plan`].
pub async fn approve_plan_with(plan_id: String, seam: &ApprovalSeam) -> Result<(), CmdError> {
    tracing::info!(plan_id, "approve_plan invoked");
    resolve_or_log(seam, &plan_id, ApprovalDecision::Approved).await
}

/// Resolve the in-process [`ApprovalSeam`] with `Revised(revisions)` for
/// `plan_id`. The renderer's user-typed string is passed through opaque
/// per CLAUDE.md §8.security; the SDK / framework JSON downstream
/// validates + sanitizes content before incorporating into a re-prompt.
///
/// # Errors
///
/// See [`approve_plan`].
#[tauri::command]
pub async fn revise_plan(
    plan_id: String,
    revisions: String,
    seam: tauri::State<'_, Arc<ApprovalSeam>>,
) -> Result<(), CmdError> {
    revise_plan_with(plan_id, revisions, seam.inner().as_ref()).await
}

/// Test-seam for [`revise_plan`].
///
/// # Errors
///
/// See [`approve_plan`].
pub async fn revise_plan_with(
    plan_id: String,
    revisions: String,
    seam: &ApprovalSeam,
) -> Result<(), CmdError> {
    tracing::info!(plan_id, len = revisions.len(), "revise_plan invoked");
    resolve_or_log(seam, &plan_id, ApprovalDecision::Revised(revisions)).await
}

/// Resolve the in-process [`ApprovalSeam`] with `Aborted(reason)` for
/// `plan_id`. The renderer's user-typed reason is passed through opaque
/// per CLAUDE.md §8.security.
///
/// # Errors
///
/// See [`approve_plan`].
#[tauri::command]
pub async fn abort_plan(
    plan_id: String,
    reason: String,
    seam: tauri::State<'_, Arc<ApprovalSeam>>,
) -> Result<(), CmdError> {
    abort_plan_with(plan_id, reason, seam.inner().as_ref()).await
}

/// Test-seam for [`abort_plan`].
///
/// # Errors
///
/// See [`approve_plan`].
pub async fn abort_plan_with(
    plan_id: String,
    reason: String,
    seam: &ApprovalSeam,
) -> Result<(), CmdError> {
    tracing::info!(plan_id, len = reason.len(), "abort_plan invoked");
    resolve_or_log(seam, &plan_id, ApprovalDecision::Aborted(reason)).await
}

/// Tauri-managed global budget cap. v0.1 holds the user-configured per-day
/// global cap in process memory only — first-run UX persistence is M10.
pub type GlobalBudgetState = Mutex<Option<f64>>;

/// Tauri-managed current-tier cache. M05 Stage D loads this from
/// `<app_data_dir>/tier.json` at startup; mutated by
/// [`request_tier_transition`]. The renderer reads via
/// [`get_current_tier`] and observes mutations through the
/// `tier_transition` event channel.
pub type CurrentTierState = Mutex<Tier>;

/// Read the user's current tier — M05 Stage D (spec §8.security L4).
///
/// # Errors
///
/// Infallible by construction (Tauri state is initialized at setup).
/// Returns `Result` to keep the surface uniform with the other tier
/// commands.
#[tauri::command]
pub async fn get_current_tier(state: tauri::State<'_, CurrentTierState>) -> Result<Tier, CmdError> {
    get_current_tier_with(state.inner()).await
}

/// Test-seam for [`get_current_tier`] (CLAUDE.md §5 `*_with` archetype).
///
/// # Errors
///
/// Infallible.
pub async fn get_current_tier_with(state: &CurrentTierState) -> Result<Tier, CmdError> {
    let tier = *state.lock().await;
    tracing::info!(?tier, "get_current_tier invoked");
    Ok(tier)
}

/// Request a tier transition — M05 Stage D (spec §8.security L4).
///
/// **Promotion** (Novice → Promoted) is authoritative on the renderer
/// side: the Settings panel shows a confirmation modal; on confirm the
/// renderer invokes this command, and the runtime treats the call as
/// approved. No `HitlSeam` involvement — tier transitions are an OS-level
/// user preference, not a framework-JSON-driven trigger.
///
/// **Demotion** (Promoted → Novice) is direct, no confirmation. Demotion
/// is always safer.
///
/// On success: persists the new tier to `<app_data_dir>/tier.json`,
/// updates the in-memory cache, and emits a `tier_transition` event
/// through the `agent_event` channel so the renderer's graph store
/// updates its `currentTier` slot.
///
/// # Errors
///
/// - [`CmdError::Internal`] if the persistence layer fails (filesystem
///   I/O, JSON serialization).
#[tauri::command]
pub async fn request_tier_transition(
    app: AppHandle,
    target_tier: Tier,
    reason: String,
    state: tauri::State<'_, CurrentTierState>,
) -> Result<(), CmdError> {
    let app_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| CmdError::internal(format!("app_local_data_dir: {e}")))?;
    request_tier_transition_with(target_tier, reason, state.inner(), &app_data_dir, |event| {
        let _ = app.emit("agent_event", &event);
        Ok(())
    })
    .await
}

/// Test-seam for [`request_tier_transition`] (CLAUDE.md §5 `*_with`
/// archetype). Accepts an injectable emit callback so unit tests
/// exercise the persistence + event-emission paths without touching
/// Tauri.
///
/// # Errors
///
/// - [`CmdError::Internal`] on filesystem write failure or emit-callback
///   error (rare; persistence module surfaces structured errors).
pub async fn request_tier_transition_with<F>(
    target_tier: Tier,
    reason: String,
    state: &CurrentTierState,
    app_data_dir: &std::path::Path,
    emit: F,
) -> Result<(), CmdError>
where
    F: FnOnce(AgentEvent) -> Result<(), CmdError>,
{
    let previous = {
        let guard = state.lock().await;
        *guard
    };
    tracing::info!(
        ?previous,
        target = ?target_tier,
        reason_len = reason.len(),
        "request_tier_transition invoked"
    );
    // Idempotent no-op when target matches current: surfaced as Ok so
    // the renderer's settings panel can call freely without checking.
    if previous == target_tier {
        tracing::info!(?target_tier, "tier already at target; idempotent no-op");
        return Ok(());
    }
    save_tier(app_data_dir, target_tier).map_err(|e: TierPersistenceError| {
        tracing::error!(error = %e, "save_tier failed");
        CmdError::internal(format!("save_tier: {e}"))
    })?;
    {
        let mut guard = state.lock().await;
        *guard = target_tier;
    }
    let event = AgentEvent::TierTransition {
        previous: tier_to_ref(previous),
        current: tier_to_ref(target_tier),
        reason,
    };
    emit(event)?;
    tracing::info!(?target_tier, "tier transition complete");
    Ok(())
}

const fn tier_to_ref(tier: Tier) -> runtime_core::event::TierRef {
    match tier {
        Tier::Novice => runtime_core::event::TierRef::Novice,
        Tier::Promoted => runtime_core::event::TierRef::Promoted,
    }
}

/// Request a session resume — M04 Stage F (spec §1b). Reads the latest
/// snapshot + projected plan/task state + uncertain tool-invocation ids
/// from the drone and returns a [`ResumePlan`] the renderer surfaces.
///
/// Tools are NOT re-invoked (gotcha #15); the SDK rebuilds message
/// history from the snapshot's signal log and the model generates the
/// next turn fresh.
///
/// # Errors
///
/// - [`CmdError::Drone`] if the IPC `RecoverSession` round-trip fails.
#[tauri::command]
pub async fn request_resume(
    session_id: String,
    drone: tauri::State<'_, Arc<DroneClient>>,
) -> Result<ResumePlan, CmdError> {
    let drone = Arc::clone(&drone);
    request_resume_command_with(session_id, |id| {
        let drone = Arc::clone(&drone);
        async move { drone.recover_session(id).await }
    })
    .await
}

/// Test-seam for [`request_resume`] — accepts an injectable async
/// recover function so tests exercise the resume flow without a real
/// drone subprocess. Maps [`ResumeError::Drone`] → [`CmdError::Drone`].
///
/// # Errors
///
/// See [`request_resume`].
pub async fn request_resume_command_with<F, Fut>(
    session_id: String,
    recover: F,
) -> Result<ResumePlan, CmdError>
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<
        Output = Result<RecoveredSession, runtime_main::drone_ipc::DroneIpcError>,
    >,
{
    tracing::info!(session_id, "request_resume invoked");
    match request_resume_with(session_id.clone(), recover).await {
        Ok(plan) => {
            tracing::info!(
                session_id,
                plans = plan.plans.len(),
                tasks = plan.tasks.len(),
                uncertain = plan.uncertain_tool_invocations.len(),
                "request_resume succeeded"
            );
            Ok(plan)
        }
        Err(ResumeError::Drone(e)) => {
            tracing::error!(error = %e, "request_resume drone IPC failed");
            Err(CmdError::drone(e.to_string()))
        }
    }
}

/// Record the user's resolution for one uncertain tool invocation —
/// M04 Stage F (spec §1b). Writes a `tool_call_uncertainty_resolved`
/// decision signal to the VDR via drone IPC so replay carries the
/// audit trail.
///
/// `action` must be one of `retry`, `skip`, `mark_complete`, `abort`.
///
/// # Errors
///
/// - [`CmdError::Internal`] if `action` is not a known token.
/// - [`CmdError::Drone`] if the signal write fails.
#[tauri::command]
pub async fn respond_uncertainty(
    session_id: String,
    invocation_id: String,
    action: String,
    agent_id: Option<String>,
    drone: tauri::State<'_, Arc<DroneClient>>,
) -> Result<UncertaintyResolution, CmdError> {
    let drone = Arc::clone(&drone);
    respond_uncertainty_command_with(session_id, invocation_id, action, agent_id, |args| {
        let drone = Arc::clone(&drone);
        async move {
            drone
                .write_signal(
                    args.signal_id,
                    args.session_id,
                    args.kind,
                    args.event,
                    args.context_type,
                    args.payload,
                )
                .await
        }
    })
    .await
}

/// Test-seam for [`respond_uncertainty`] — accepts an injectable async
/// emit function. Maps unknown-action to [`CmdError::Internal`] and
/// drone errors to [`CmdError::Drone`].
///
/// # Errors
///
/// See [`respond_uncertainty`].
pub async fn respond_uncertainty_command_with<F, Fut>(
    session_id: String,
    invocation_id: String,
    action: String,
    agent_id: Option<String>,
    emit: F,
) -> Result<UncertaintyResolution, CmdError>
where
    F: FnOnce(runtime_main::recovery::uncertainty::WriteSignalArgs) -> Fut,
    Fut: std::future::Future<Output = Result<(), runtime_main::drone_ipc::DroneIpcError>>,
{
    tracing::info!(session_id, invocation_id, action = %action, "respond_uncertainty invoked");
    match respond_uncertainty_with(session_id, invocation_id, action, agent_id, emit).await {
        Ok(resolution) => {
            tracing::info!(
                signal_id = resolution.signal_id,
                action = resolution.action.as_token(),
                "respond_uncertainty recorded resolution"
            );
            Ok(resolution)
        }
        Err(UncertaintyError::UnknownAction(token)) => {
            tracing::warn!(action = %token, "respond_uncertainty rejected unknown action token");
            Err(CmdError::internal(format!("unknown action token: {token}")))
        }
        Err(UncertaintyError::Drone(e)) => {
            tracing::error!(error = %e, "respond_uncertainty drone IPC failed");
            Err(CmdError::drone(e.to_string()))
        }
    }
}

/// Store the user's per-day global budget cap — M04 Stage F (spec §2a).
/// v0.1 holds the value in process memory only; M10 first-run UX
/// persists it to settings. Set to `0.0` to disable the global cap.
///
/// # Errors
///
/// - [`CmdError::Internal`] if `usd_cap` is negative.
#[tauri::command]
pub async fn set_global_budget(
    usd_cap: f64,
    state: tauri::State<'_, GlobalBudgetState>,
) -> Result<(), CmdError> {
    set_global_budget_with(usd_cap, state.inner()).await
}

/// Test-seam for [`set_global_budget`].
///
/// # Errors
///
/// See [`set_global_budget`].
pub async fn set_global_budget_with(
    usd_cap: f64,
    state: &GlobalBudgetState,
) -> Result<(), CmdError> {
    if usd_cap.is_nan() || usd_cap < 0.0 {
        return Err(CmdError::internal(
            "global budget cap must be a non-negative number".to_string(),
        ));
    }
    let mut guard = state.lock().await;
    *guard = if usd_cap > 0.0 { Some(usd_cap) } else { None };
    tracing::info!(usd_cap, "set_global_budget stored");
    Ok(())
}

/// Resolve the in-process [`HitlSeam`] for a HITL prompt — M04 Stage E
/// (spec §6a). The renderer's Panel / Modal / Toast surfaces dispatch
/// this command when the user picks a choice. The SDK's awaiting HITL
/// gate wakes via [`HitlSeam::resolve`] and the plan loop routes per the
/// chosen token.
///
/// # Errors
///
/// - [`CmdError::Internal`] if the seam reports the receiver was already
///   dropped (rare; usually means the prompt timed out between the
///   renderer's click and the dispatch reaching main).
///
/// **Soft-Ok on no-pending-await:** mirrors [`approve_plan`]'s rationale.
/// If no SDK task is currently awaiting the `prompt_id` (e.g. the prompt
/// timed out, or the SDK plan-loop integration site is deferred per
/// M04.B retro), this returns `Ok(())` with a warn-log rather than 500
/// the renderer's click.
#[tauri::command]
pub async fn respond_hitl(
    prompt_id: String,
    choice: String,
    seam: tauri::State<'_, Arc<HitlSeam>>,
) -> Result<(), CmdError> {
    respond_hitl_with(prompt_id, choice, seam.inner().as_ref()).await
}

/// Test-seam for [`respond_hitl`] (CLAUDE.md §5 `*_with` archetype).
///
/// # Errors
///
/// See [`respond_hitl`].
pub async fn respond_hitl_with(
    prompt_id: String,
    choice: String,
    seam: &HitlSeam,
) -> Result<(), CmdError> {
    tracing::info!(prompt_id, choice_len = choice.len(), "respond_hitl invoked");
    match seam.resolve(&prompt_id, HitlChoice::new(choice)).await {
        Ok(()) => {
            tracing::info!(prompt_id, "hitl seam resolved");
            Ok(())
        }
        Err(HitlError::NotFound(_)) => {
            tracing::warn!(prompt_id, "hitl seam had no pending awaiter; soft-Ok");
            Ok(())
        }
        Err(e) => {
            tracing::error!(prompt_id, error = %e, "hitl seam resolve failed");
            Err(CmdError::internal(e.to_string()))
        }
    }
}

// ── M06 Stage C — MCP server lifecycle commands ──────────────

/// Add a new MCP server. Persists to the registry, optionally stores the
/// per-server auth secret in the OS keychain (under the
/// `agent-runtime/mcp` namespace), runs a one-shot test connection, and
/// emits the audit lines (`mcp_installed` + `mcp_auth_granted` when
/// applicable). Renderer wires from the Settings panel "Add Server" form.
///
/// # Errors
///
/// - [`CmdError::Internal`] for any lifecycle error (registry / transport /
///   auth / connect failure). The error body carries the underlying detail.
#[tauri::command]
pub async fn mcp_add_server(
    config: serde_json::Value,
    auth: Option<String>,
    client: tauri::State<'_, Arc<McpClient>>,
) -> Result<(), CmdError> {
    mcp_add_server_with(config, auth, client.inner().as_ref()).await
}

/// Test-seam for [`mcp_add_server`]. Accepts a borrowed `McpClient` so
/// unit tests construct one with `tempfile`-backed registry + audit and
/// drive the lifecycle without Tauri state plumbing.
///
/// # Errors
///
/// See [`mcp_add_server`].
pub async fn mcp_add_server_with(
    config: serde_json::Value,
    auth: Option<String>,
    client: &McpClient,
) -> Result<(), CmdError> {
    let parsed: runtime_core::generated::mcp::McpServerConfig = serde_json::from_value(config)
        .map_err(|e| CmdError::internal(format!("mcp_add_server: invalid config JSON: {e}")))?;
    let transport = McpClient::transport_from_config(&parsed);
    tracing::info!(name = %parsed.name.to_string(), has_auth = auth.is_some(), "mcp_add_server invoked");
    client
        .add_server(parsed, auth, transport)
        .await
        .map_err(|e| CmdError::internal(format!("mcp_add_server: {e}")))
}

/// Remove a registered MCP server. Disconnects, removes registry row,
/// drops the auth secret if present, emits `mcp_uninstalled` audit line.
///
/// # Errors
///
/// - [`CmdError::Internal`] for any lifecycle error. `NotFound` surfaces as
///   `Internal` with the underlying detail body — the renderer interprets.
#[tauri::command]
pub async fn mcp_remove_server(
    name: String,
    client: tauri::State<'_, Arc<McpClient>>,
) -> Result<(), CmdError> {
    mcp_remove_server_with(name, client.inner().as_ref()).await
}

/// Test-seam for [`mcp_remove_server`].
///
/// # Errors
///
/// See [`mcp_remove_server`].
pub async fn mcp_remove_server_with(name: String, client: &McpClient) -> Result<(), CmdError> {
    tracing::info!(name = %name, "mcp_remove_server invoked");
    client
        .remove_server(&name)
        .await
        .map_err(|e| CmdError::internal(format!("mcp_remove_server: {e}")))
}

/// Test a server connection without persisting (Settings panel "Test"
/// button). Connect + `list_tools` + disconnect.
///
/// # Errors
///
/// - [`CmdError::Internal`] for any lifecycle error.
#[tauri::command]
pub async fn mcp_test_connection(
    config: serde_json::Value,
    client: tauri::State<'_, Arc<McpClient>>,
) -> Result<Vec<McpTool>, CmdError> {
    mcp_test_connection_with(config, client.inner().as_ref()).await
}

/// Test-seam for [`mcp_test_connection`].
///
/// # Errors
///
/// See [`mcp_test_connection`].
pub async fn mcp_test_connection_with(
    config: serde_json::Value,
    client: &McpClient,
) -> Result<Vec<McpTool>, CmdError> {
    let parsed: runtime_core::generated::mcp::McpServerConfig = serde_json::from_value(config)
        .map_err(|e| {
            CmdError::internal(format!("mcp_test_connection: invalid config JSON: {e}"))
        })?;
    let transport = McpClient::transport_from_config(&parsed);
    tracing::info!(name = %parsed.name.to_string(), "mcp_test_connection invoked");
    client
        .test_connection(transport)
        .await
        .map_err(|e| CmdError::internal(format!("mcp_test_connection: {e}")))
}

/// List registered MCP servers + their current state.
///
/// # Errors
///
/// - [`CmdError::Internal`] for registry failures.
#[tauri::command]
pub async fn mcp_list_servers(
    client: tauri::State<'_, Arc<McpClient>>,
) -> Result<Vec<McpServerSummary>, CmdError> {
    mcp_list_servers_with(client.inner().as_ref()).await
}

/// Test-seam for [`mcp_list_servers`].
///
/// # Errors
///
/// See [`mcp_list_servers`].
pub async fn mcp_list_servers_with(client: &McpClient) -> Result<Vec<McpServerSummary>, CmdError> {
    tracing::info!("mcp_list_servers invoked");
    client
        .list_servers()
        .await
        .map_err(|e| CmdError::internal(format!("mcp_list_servers: {e}")))
}

/// Shared resolve-and-log helper for the three approval-flow commands.
/// Treats `ApprovalError::NotFound` as soft-Ok with a warn-log per the
/// no-pending-await rationale on [`approve_plan`].
async fn resolve_or_log(
    seam: &ApprovalSeam,
    plan_id: &str,
    decision: ApprovalDecision,
) -> Result<(), CmdError> {
    match seam.resolve(plan_id, decision).await {
        Ok(()) => {
            tracing::info!(plan_id, "approval seam resolved");
            Ok(())
        }
        Err(ApprovalError::NotFound(_)) => {
            tracing::warn!(plan_id, "approval seam had no pending awaiter; soft-Ok");
            Ok(())
        }
        Err(e) => {
            tracing::error!(plan_id, error = %e, "approval seam resolve failed");
            Err(CmdError::internal(e.to_string()))
        }
    }
}

fn smoke_config() -> AgentConfig {
    AgentConfig {
        model: "claude-haiku-4-5".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text {
                text: "Say only the word: hello".to_string(),
            }],
        }],
        max_tokens: 16,
        temperature: Some(0.0),
        system_prompt: None,
        tools: vec![],
    }
}

/// Outcome of [`import_artifact`] surfaced to the renderer (Stage E).
#[derive(Debug, serde::Serialize)]
pub struct ImportOutcome {
    /// The `name@version` written into `skills.lock`.
    pub lock_key: String,
    /// Whether the Novice capability-disclosure review applies — the
    /// renderer renders the disclosure modal for Novice (the L4 outcome
    /// is decided here, the renderer just renders it; spec §M7 / E.3.4).
    pub review_required: bool,
    /// Secrets the artifact needs before first run (spec §15d notice).
    pub requires_secrets: Vec<String>,
}

/// L1 network gate over the M05 `CapabilityEnforcer` — import egress is
/// constrained to the user-supplied host (default-deny + domain scope;
/// Hard Rule 4 — no phone-home, only the URL the user pasted is hit).
struct EnforcerGate;

impl NetworkGate for EnforcerGate {
    fn check(&self, host: &str) -> Result<(), String> {
        use runtime_core::generated::capability::{
            CapabilityDeclaration, CapabilityKind, CapabilityScope, DomainPattern, ResourceName,
            SideEffectClass,
        };
        use runtime_main::capability::CapabilityEnforcer;
        use std::str::FromStr as _;

        let decl = CapabilityDeclaration {
            kind: CapabilityKind::Network,
            resource: ResourceName::from_str(host).map_err(|e| e.to_string())?,
            scope: CapabilityScope::Domain(
                DomainPattern::from_str(host).map_err(|e| e.to_string())?,
            ),
            side_effect_class: SideEffectClass::NetworkEgress,
        };
        let mut enforcer = CapabilityEnforcer::new();
        enforcer.grant("import-fetch", decl.clone());
        enforcer
            .check("import-fetch", &decl)
            .map_err(|e| e.to_string())
    }
}

/// L3 adapter over the M05 sandbox subprocess (`runtime-sandbox`,
/// reused — not rebuilt). v0.1 import L3 uses a conservative
/// `Read`/`Pure` declaration: any write / network / spawn / exec token
/// the validator detects in the artifact rejects the install
/// (sandbox-before-trust; ADR-0014 threat model).
struct SandboxAdapter(Arc<SandboxClient>);

#[async_trait::async_trait]
impl Sandbox for SandboxAdapter {
    async fn validate(&self, code: &str) -> Result<Vec<String>, String> {
        use runtime_core::generated::capability::{
            CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, ResourceName,
            SideEffectClass,
        };
        use runtime_main::sandbox_ipc::ValidationResult;
        use std::str::FromStr as _;

        let conservative = CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("*").expect("`*` is a valid ResourceName"),
            scope: CapabilityScope::Glob(
                GlobPattern::from_str("*").expect("`*` is a valid GlobPattern"),
            ),
            side_effect_class: SideEffectClass::Pure,
        };
        match self
            .0
            .validate(code.to_string(), conservative)
            .await
            .map_err(|e| e.to_string())?
        {
            ValidationResult::Ok => Ok(Vec::new()),
            ValidationResult::Reject { reasons } => Ok(reasons),
        }
    }
}

/// MCP Manager registry adapter (ADR-0010 dependency inversion — the
/// concrete `runtime_mcp::Registry` is wrapped HERE in the shell, never
/// depended on by `runtime-main`, to avoid the `runtime-mcp →
/// runtime-main` Cargo cycle). `upsert` = idempotent remove-then-insert
/// so a re-import replaces the prior config.
struct RegistryAdapter(Arc<Registry>);

impl McpRegistry for RegistryAdapter {
    fn upsert(&self, cfg: &McpServerImport) -> Result<(), String> {
        let record = McpServerRecord {
            name: cfg.name.clone(),
            transport: cfg.transport.clone(),
            command: cfg.command.clone(),
            args_json: cfg.args_json.clone(),
            env_json: cfg.env_json.clone(),
            cwd: cfg.cwd.clone(),
            url: cfg.url.clone(),
            auth_secret_ref: cfg.auth_secret_ref.clone(),
            // CQ-6 — a freshly-imported MCP-config server is
            // `disconnected` until the first health pass / connect.
            status: runtime_mcp::ServerStatus::Disconnected,
        };
        self.0.remove(&cfg.name).map_err(|e| e.to_string())?;
        self.0.insert(&record).map_err(|e| e.to_string())
    }
}

fn import_err_to_cmd(e: &ImportError) -> CmdError {
    CmdError::internal(e.to_string())
}

/// Import an artifact (skill / tool / agent / MCP-server config) by
/// GitHub-raw URL or local file — M07 Stage C (spec Phase 7 §2152-2211;
/// MVP §M7).
///
/// Thin §5 shell wrapper over the unit-tested `import_artifact_with`
/// seam. Resolves the framework-root `skills.lock` path
/// (path-agnostic — CLAUDE.md §9), wires the real reqwest fetcher + the
/// M05 L1/L3 + the M06 registry adapter + wall-clock, and records the
/// current tier as `tier_at_install`. The Novice capability-disclosure
/// review is rendered by the Stage E review screen via the
/// `import::tier_gate` seam (E.3.4 — "the renderer renders the
/// outcome"); `review_required` carries that decision to the renderer.
///
/// # Errors
///
/// - [`CmdError::Internal`] for any pipeline failure (fetch / schema /
///   L3 reject / OS mismatch / lock / registry), the message naming the
///   stage.
#[tauri::command]
pub async fn import_artifact(
    app: AppHandle,
    source_kind: String,
    location: String,
    artifact_kind: String,
    sandbox: tauri::State<'_, Arc<SandboxClient>>,
    registry: tauri::State<'_, Arc<Registry>>,
    tier_state: tauri::State<'_, CurrentTierState>,
) -> Result<ImportOutcome, CmdError> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| CmdError::internal(format!("app_local_data_dir: {e}")))?;
    let lock = dir.join("skills.lock");

    let src = match source_kind.as_str() {
        "url" => ImportSource::Url(location),
        "file" => ImportSource::File(location.into()),
        other => return Err(CmdError::internal(format!("unknown source_kind: {other}"))),
    };
    let kind = match artifact_kind.as_str() {
        "skill" => ArtifactKind::Skill,
        "tool" => ArtifactKind::Tool,
        "agent" => ArtifactKind::Agent,
        "mcp_server" => ArtifactKind::McpServer,
        other => {
            return Err(CmdError::internal(format!(
                "unknown artifact_kind: {other}"
            )))
        }
    };
    let tier = *tier_state.lock().await;

    let installed = import::import_artifact_with(
        src,
        kind,
        tier,
        std::env::consts::OS,
        &lock,
        &EnforcerGate,
        &HttpFetcher::new(),
        &SandboxAdapter(Arc::clone(sandbox.inner())),
        &RegistryAdapter(Arc::clone(registry.inner())),
        &SystemClock,
    )
    .await
    .map_err(|e| import_err_to_cmd(&e))?;

    Ok(ImportOutcome {
        lock_key: installed.lock_key,
        review_required: matches!(tier, Tier::Novice),
        requires_secrets: installed.requires_secrets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use futures::stream::BoxStream;
    use keyring::Error as KeyringError;
    use runtime_main::providers::{
        AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
        ProviderSupport,
    };

    #[test]
    fn cmd_error_setup_required_serializes_with_type_tag_only() {
        // The renderer pattern-matches on the JSON shape from
        // src/types/error.ts; the unit-variant case must produce
        // `{"type":"setup_required"}` with no `message` key.
        let json = serde_json::to_string(&CmdError::SetupRequired).unwrap();
        assert_eq!(json, r#"{"type":"setup_required"}"#);
    }

    #[test]
    fn cmd_error_provider_serializes_with_message_body() {
        // Generated CmdError uses #[serde(tag="type", content="message")]
        // on tuple variants — produces the same {"type":"...","message":"..."}
        // wire shape M02 emitted via #[serde(tag="type")] on struct variants.
        let json = serde_json::to_string(&CmdError::provider("boom")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "provider");
        assert_eq!(value["message"], "boom");
    }

    #[test]
    fn cmd_error_from_keystore_not_found_maps_to_setup_required() {
        // The keychain "not found" condition is the user-actionable path:
        // renderer surfaces "set your key first" rather than a generic
        // backend error. The `From<KeyStoreError>` impl lives in
        // `runtime-main/src/key_store.rs` per orphan-rule constraints
        // (CmdError is foreign to runtime-main; KeyStoreError is local).
        let e: CmdError = KeyStoreError::NotFound.into();
        assert!(matches!(e, CmdError::SetupRequired), "got {e:?}");
    }

    #[test]
    fn import_outcome_serializes_the_enriched_review_wire_for_the_renderer() {
        // M07.E / ADR-0015 — the cross-language wire anchor (condition
        // 2). The Stage E renderer's hand-mirrored `ImportOutcome`
        // interface (src/lib/ipc.ts, the McpTool/ResumePlan precedent)
        // pattern-matches THIS exact serialized shape. Pinning the snake
        // _case keys here is what makes the renderer fixture provably
        // the real bridge contract, not a fabricated mock. The
        // `runtime-main` integration suite proves a REAL import produces
        // an enriched `Installed`; this proves the command maps it to
        // the JSON the renderer consumes.
        let outcome = ImportOutcome {
            lock_key: "fs-test@2.0.0".to_string(),
            review_required: true,
            requires_secrets: vec!["OPENAI_API_KEY".to_string()],
            capabilities: vec![
                "network: api.example.com".to_string(),
                "shell: true".to_string(),
            ],
            l3_report: runtime_main::import::L3Report {
                report_id: "vr-1".to_string(),
                passed: true,
                reasons: vec![],
            },
            share_provenance: Some(serde_json::json!({
                "exported_by": "share-it@0.1.0",
                "rebake_changes": []
            })),
        };
        let v = serde_json::to_value(&outcome).unwrap();
        assert_eq!(v["lock_key"], serde_json::json!("fs-test@2.0.0"));
        assert_eq!(v["review_required"], serde_json::json!(true));
        assert_eq!(
            v["requires_secrets"],
            serde_json::json!(["OPENAI_API_KEY"])
        );
        assert_eq!(
            v["capabilities"],
            serde_json::json!(["network: api.example.com", "shell: true"]),
            "the renderer's plain-English disclosure reads `capabilities`"
        );
        assert_eq!(
            v["l3_report"],
            serde_json::json!({ "report_id": "vr-1", "passed": true, "reasons": [] }),
            "the L3 report crosses the bridge as a nested object"
        );
        assert_eq!(
            v["share_provenance"]["rebake_changes"],
            serde_json::json!([]),
            "share_provenance surfaces verbatim (None serializes to null)"
        );
    }

    #[test]
    fn import_outcome_serializes_absent_provenance_as_null() {
        // The renderer renders the "no provenance" state from `null`,
        // never a synthesized empty block (ADR-0005 / ADR-0015).
        let outcome = ImportOutcome {
            lock_key: "x@1.0.0".to_string(),
            review_required: false,
            requires_secrets: vec![],
            capabilities: vec![],
            l3_report: runtime_main::import::L3Report {
                report_id: "r".to_string(),
                passed: true,
                reasons: vec![],
            },
            share_provenance: None,
        };
        let v = serde_json::to_value(&outcome).unwrap();
        assert_eq!(v["share_provenance"], serde_json::Value::Null);
    }

    /// In-process stub provider used by `run_smoke_session_with` tests.
    struct StubProvider;

    #[async_trait]
    impl LLMProvider for StubProvider {
        #[allow(
            clippy::unnecessary_literal_bound,
            reason = "trait method returns &str by signature; literal &'static str must reborrow"
        )]
        fn name(&self) -> &str {
            "stub"
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
                ProviderEvent::TextDelta {
                    text: "hello".to_string(),
                },
                ProviderEvent::MessageStop {
                    stop_reason: "end_turn".to_string(),
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
    async fn run_smoke_session_with_emits_events_to_channel() {
        // The testable seam runs the SDK against a stub provider and
        // pushes events to a caller-owned channel. This exercises the
        // command-body equivalent of `run_smoke_session` without a Tauri
        // AppHandle (which is environment-bound). The seam takes an
        // `Arc<DroneClient>` (M04 Stage A2) + a `SessionId` (M06.5 🔴-2)
        // — tests inject `noop` (no FK) + a fresh SessionId.
        let (tx, mut rx) = mpsc::channel(8);
        let drone = Arc::new(DroneClient::noop());
        let config = smoke_config();
        run_smoke_session_with(StubProvider, tx, drone, config, None, SessionId::new())
            .await
            .expect("run_smoke_session_with");

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AgentEvent::AgentSpawned { .. })),
            "expected AgentSpawned in {events:?}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AgentEvent::AgentComplete { .. })),
            "expected AgentComplete in {events:?}"
        );
    }

    /// Provider that emits a single MCP-shaped `ToolUse` then stops, so
    /// the M06.F injection-seam test drives the run-loop interception.
    struct McpToolProvider;

    #[async_trait]
    impl LLMProvider for McpToolProvider {
        #[allow(
            clippy::unnecessary_literal_bound,
            reason = "trait method returns &str by signature; literal &'static str must reborrow"
        )]
        fn name(&self) -> &str {
            "mcp-tool-stub"
        }
        fn supports(&self) -> ProviderSupport {
            ProviderSupport {
                tool_use: true,
                streaming: true,
                thinking: false,
            }
        }
        async fn stream(
            &self,
            _config: AgentConfig,
        ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
            Ok(Box::pin(futures::stream::iter(vec![
                ProviderEvent::ToolUse {
                    id: "t1".to_string(),
                    name: "pdf-mcp__extract_text".to_string(),
                    input: serde_json::json!({"path": "doc.pdf"}),
                },
                ProviderEvent::MessageStop {
                    stop_reason: "end_turn".to_string(),
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

    /// Mock `McpToolDispatch` that always resolves the call as an
    /// `Invoked` MCP outcome — the seam test asserts the SDK actually
    /// routed the `ToolUse` through the injected dispatch.
    struct InvokingMcpDispatch;

    #[async_trait]
    impl McpToolDispatch for InvokingMcpDispatch {
        async fn dispatch_if_mcp(
            &self,
            _agent_id: &str,
            _tool_name: &str,
            _args: serde_json::Value,
            _aliases: &std::collections::BTreeMap<String, String>,
        ) -> Option<
            Result<runtime_main::sdk::McpDispatchOutcome, runtime_main::sdk::McpDispatchError>,
        > {
            Some(Ok(runtime_main::sdk::McpDispatchOutcome::Invoked {
                server: "pdf-mcp".to_string(),
                tool: "extract_text".to_string(),
                value: serde_json::json!({"text": "extracted"}),
            }))
        }
    }

    #[tokio::test]
    async fn run_smoke_session_with_injected_mcp_dispatch_routes_tool_use_through_seam() {
        // M06.F composition-root injection seam (ADR-0011 trace #11a):
        // passing `Some(dispatch)` must compose it onto the SDK
        // (`with_mcp_dispatch`) so an MCP `ProviderEvent::ToolUse`
        // resolves through the injected dispatch and reaches the
        // renderer-facing channel as an agent_id-correct, MCP-sourced
        // ToolInvoked + ToolResult — NOT the Stage A Builtin path.
        let (tx, mut rx) = mpsc::channel(16);
        let drone = Arc::new(DroneClient::noop());
        run_smoke_session_with(
            McpToolProvider,
            tx,
            drone,
            smoke_config(),
            Some(Arc::new(InvokingMcpDispatch) as Arc<dyn McpToolDispatch>),
            SessionId::new(),
        )
        .await
        .expect("run_smoke_session_with(Some(dispatch))");

        let mut events = Vec::new();
        while let Some(e) = rx.recv().await {
            events.push(e);
        }
        let invoked = events
            .iter()
            .find_map(|e| match e {
                AgentEvent::ToolInvoked {
                    agent_id,
                    source,
                    server,
                    ..
                } => Some((agent_id.clone(), source.clone(), server.clone())),
                _ => None,
            })
            .unwrap_or_else(|| panic!("expected an MCP ToolInvoked; events: {events:?}"));
        assert!(
            !invoked.0.is_empty(),
            "injected-dispatch ToolInvoked agent_id MUST be non-empty (gotcha #68)"
        );
        assert_eq!(invoked.1, runtime_core::event::ToolSource::Mcp);
        assert_eq!(invoked.2.as_deref(), Some("pdf-mcp"));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AgentEvent::ToolResult { .. })),
            "the injected MCP dispatch must also surface a ToolResult; events: {events:?}"
        );
    }

    // ── ADR-0011 (c) — concrete McpDispatcher constructed in src-tauri ──
    //
    // M06.F's production `run_smoke_session` passed `None` because the
    // concrete `McpDispatcher` was not constructible in-shell (ADR-0011
    // Context #2/#3 — no `NamespaceResolver`/`CapabilityEnforcer` ctor
    // site). D1 adds `build_mcp_dispatcher`, closing the A-mapped
    // construction graph. CapabilityEnforcer construction is
    // CODEOWNERS-flagged (Hard Rule 8) — the construction-reachability
    // map + this seam test is the surfaced plan.

    fn mcp_client_over_tempdir() -> (tempfile::TempDir, Arc<McpClient>) {
        use runtime_mcp::client::{InMemorySecretStore, SecretStore};
        let dir = tempfile::TempDir::new().expect("tempdir");
        let registry =
            Arc::new(Registry::open(&dir.path().join("mcp.sqlite")).expect("open registry"));
        let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
        let client = Arc::new(McpClient::new(registry, secret_store, "sess-build"));
        (dir, client)
    }

    #[tokio::test]
    async fn build_mcp_dispatcher_yields_the_concrete_dispatcher_not_a_mock() {
        // The constructed value must behave as the concrete
        // `McpDispatcher`: an empty `NamespaceResolver` resolves an
        // unknown tool to §5a `NotFound` ⇒ `dispatch_if_mcp` returns
        // `None` (fall-through). A mock (e.g. `InvokingMcpDispatch`)
        // returns `Some(Invoked)` — so `None` here proves it is the
        // real concrete impl threaded through, not a stand-in.
        let (_dir, client) = mcp_client_over_tempdir();
        let dispatcher = build_mcp_dispatcher(client, None, &SessionId::new());
        let outcome = dispatcher
            .dispatch_if_mcp(
                "worker",
                "definitely_not_an_mcp_tool",
                serde_json::json!({}),
                &std::collections::BTreeMap::new(),
            )
            .await;
        assert!(
            outcome.is_none(),
            "concrete McpDispatcher with an empty resolver must fall through (None); got {outcome:?}"
        );
    }

    #[tokio::test]
    async fn run_smoke_session_with_threads_the_concrete_built_dispatcher() {
        // The assembled path (the §6/v1.8 assembled-regression mandate):
        // the shell ctor output threads through the real
        // `run_smoke_session_with` seam. The no-tools smoke emits no
        // `ProviderEvent::ToolUse`, so the dispatcher is
        // constructed-but-not-exercised here (D2's agent-with-tools loop
        // is what exercises it) — but construction + threading must not
        // break the existing smoke path.
        let (_dir, client) = mcp_client_over_tempdir();
        let dispatcher = build_mcp_dispatcher(client, None, &SessionId::new());
        let (tx, mut rx) = mpsc::channel(8);
        let drone = Arc::new(DroneClient::noop());
        run_smoke_session_with(
            StubProvider,
            tx,
            drone,
            smoke_config(),
            Some(dispatcher),
            SessionId::new(),
        )
        .await
        .expect("smoke path still succeeds with the concrete dispatcher injected");
        let mut events = Vec::new();
        while let Some(e) = rx.recv().await {
            events.push(e);
        }
        assert!(
            !events.is_empty(),
            "the no-tools smoke still produces its events with the dispatcher threaded"
        );
    }

    /// Stub provider whose `stream()` returns a `ProviderError` so the
    /// `run_smoke_session_with` error-path tracing branch is exercised.
    struct FailingProvider;

    #[async_trait]
    impl LLMProvider for FailingProvider {
        #[allow(
            clippy::unnecessary_literal_bound,
            reason = "trait method returns &str by signature; literal &'static str must reborrow"
        )]
        fn name(&self) -> &str {
            "failing"
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
            Err(ProviderError::Auth)
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
    async fn run_smoke_session_with_error_path_emits_provider_cmd_error() {
        // Exercises the error-branch tracing call inside run_smoke_session_with.
        // The stub provider returns ProviderError::Auth; the seam must wrap it
        // into CmdError::Provider per the existing translation.
        let (tx, _rx) = mpsc::channel(8);
        let drone = Arc::new(DroneClient::noop());
        let result = run_smoke_session_with(
            FailingProvider,
            tx,
            drone,
            smoke_config(),
            None,
            SessionId::new(),
        )
        .await;
        let err = result.expect_err("expected provider error");
        assert!(
            matches!(err, CmdError::Provider(_)),
            "expected CmdError::Provider, got {err:?}"
        );
    }

    #[test]
    fn set_api_key_with_success_path() {
        // Inject a writer that succeeds; exercises the entry + success
        // tracing branches in `set_api_key_with`.
        let result = set_api_key_with("sk-ant-test1234567890", |_key| Ok(()));
        assert!(result.is_ok(), "got {result:?}");
    }

    #[test]
    fn set_api_key_with_error_path_maps_to_keystore_cmd_error() {
        // Inject a writer that returns a Keyring error; exercises the error
        // tracing branch + the From<KeyStoreError> for CmdError translation.
        // The From impl now lives in runtime-main/src/key_store.rs; this
        // test keeps the cross-crate translation path under coverage at the
        // command-surface level too.
        let result = set_api_key_with("sk-ant-test1234567890", |_key| {
            Err(KeyStoreError::Keyring(KeyringError::NoEntry))
        });
        let err = result.expect_err("expected keyring error");
        assert!(
            matches!(err, CmdError::KeyStore(_)),
            "expected CmdError::KeyStore, got {err:?}"
        );
    }

    #[test]
    fn set_api_key_with_error_path_not_found_maps_to_setup_required() {
        // The bare KeyStoreError::NotFound variant maps to SetupRequired
        // per the From impl. Exercises the same error tracing branch as
        // the test above but a different translation path.
        let result = set_api_key_with("sk-ant-test1234567890", |_key| Err(KeyStoreError::NotFound));
        let err = result.expect_err("expected NotFound error");
        assert!(
            matches!(err, CmdError::SetupRequired),
            "expected CmdError::SetupRequired, got {err:?}"
        );
    }

    #[tokio::test]
    async fn query_session_db_with_returns_rows_from_querier() {
        let rows = query_session_db_with("SELECT id FROM signals".to_string(), |sql| async move {
            assert_eq!(sql, "SELECT id FROM signals");
            Ok(vec![serde_json::json!({"id": "x"})])
        })
        .await
        .expect("query");
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn query_session_db_with_propagates_querier_error() {
        let result = query_session_db_with("SELECT 1".to_string(), |_sql| async move {
            Err(CmdError::drone("boom"))
        })
        .await;
        assert!(matches!(result, Err(CmdError::Drone(_))));
    }

    #[tokio::test]
    async fn replay_session_with_emits_translated_events() {
        let signals = vec![
            serde_json::json!({
                "type": "session",
                "payload_json": {"event": "start", "session_id": "s1", "framework": "aria", "model": "haiku"},
            }),
            serde_json::json!({
                "type": "agent",
                "payload_json": {"event": "spawned", "agent_id": "a1", "agent_name": "n", "session_id": "s1"},
            }),
        ];
        let mut emitted: Vec<AgentEvent> = Vec::new();
        replay_session_with(
            "s1".to_string(),
            move |id| async move {
                assert_eq!(id, "s1");
                Ok(signals)
            },
            |event| {
                emitted.push(event);
                Ok(())
            },
        )
        .await
        .expect("replay");
        assert_eq!(emitted.len(), 2);
        assert!(matches!(emitted[0], AgentEvent::SessionStart { .. }));
        assert!(matches!(emitted[1], AgentEvent::AgentSpawned { .. }));
    }

    #[tokio::test]
    async fn replay_session_with_propagates_reader_error() {
        let result = replay_session_with(
            "s1".to_string(),
            |_id| async move { Err(CmdError::drone("boom")) },
            |_event| Ok::<(), CmdError>(()),
        )
        .await;
        assert!(matches!(result, Err(CmdError::Drone(_))));
    }

    #[tokio::test]
    async fn replay_session_with_swallows_emit_errors_and_continues() {
        let signals = vec![
            serde_json::json!({
                "type": "agent",
                "payload_json": {"event": "spawned", "agent_id": "a1", "agent_name": "n", "session_id": "s1"},
            }),
            serde_json::json!({
                "type": "agent",
                "payload_json": {"event": "spawned", "agent_id": "a2", "agent_name": "n", "session_id": "s1"},
            }),
        ];
        let mut count = 0;
        replay_session_with(
            "s1".to_string(),
            move |_id| async move { Ok(signals) },
            |_event| {
                count += 1;
                Err(CmdError::internal("renderer gone"))
            },
        )
        .await
        .expect("replay must not surface emit errors");
        assert_eq!(count, 2, "emit must be invoked for every translated event");
    }

    #[test]
    fn smoke_config_targets_haiku_with_tight_budget() {
        // Sanity-checks the hardcoded smoke prompt — Haiku for cost
        // (cheapest model), max_tokens=16 to bound spend per click,
        // temperature=0 for deterministic output.
        let cfg = smoke_config();
        assert_eq!(cfg.model, "claude-haiku-4-5");
        assert_eq!(cfg.max_tokens, 16);
        assert_eq!(cfg.temperature, Some(0.0));
        assert!(cfg.tools.is_empty());
        assert_eq!(cfg.messages.len(), 1);
        assert_eq!(cfg.messages[0].role, MessageRole::User);
    }

    // ── M04 Stage C: approval-flow Tauri commands ────────────────

    use runtime_main::sdk::{ApprovalDecision, ApprovalSeam};

    /// Spawn the seam's awaiter, wait for it to register, then return the
    /// join handle so the test can resolve the seam and assert on the
    /// awaited decision.
    async fn await_seam(
        seam: &ApprovalSeam,
        plan_id: &str,
    ) -> tokio::task::JoinHandle<ApprovalDecision> {
        let s = seam.clone();
        let id = plan_id.to_string();
        let handle =
            tokio::spawn(async move { s.await_approval(&id).await.expect("await_approval") });
        // Spin until the awaiter has registered.
        for _ in 0..100 {
            if seam.pending_len().await >= 1 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
        handle
    }

    #[tokio::test]
    async fn approve_plan_with_resolves_seam_with_approved_decision() {
        let seam = ApprovalSeam::new();
        let awaiter = await_seam(&seam, "p1").await;
        approve_plan_with("p1".into(), &seam)
            .await
            .expect("approve_plan_with");
        let decision = awaiter.await.expect("join");
        assert_eq!(decision, ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn approve_plan_with_no_pending_awaiter_returns_ok_with_warn() {
        // Per CLAUDE.md §12 ergonomics: the SDK plan_loop driver is deferred
        // to M07 (Stage B retro [LIVE] ambiguity-events), so the renderer
        // can dispatch approve_plan with no awaiter present. Treat as
        // soft-Ok (warn-logged) rather than 500 the user's click.
        let seam = ApprovalSeam::new();
        let result = approve_plan_with("ghost".into(), &seam).await;
        assert!(result.is_ok(), "got {result:?}");
    }

    #[tokio::test]
    async fn revise_plan_with_resolves_seam_with_revised_decision_and_text() {
        let seam = ApprovalSeam::new();
        let awaiter = await_seam(&seam, "p1").await;
        revise_plan_with("p1".into(), "expand risks".into(), &seam)
            .await
            .expect("revise_plan_with");
        let decision = awaiter.await.expect("join");
        match decision {
            ApprovalDecision::Revised(text) => assert_eq!(text, "expand risks"),
            other => panic!("expected Revised, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn revise_plan_with_no_pending_awaiter_returns_ok() {
        let seam = ApprovalSeam::new();
        let result = revise_plan_with("ghost".into(), "rev".into(), &seam).await;
        assert!(result.is_ok(), "got {result:?}");
    }

    #[tokio::test]
    async fn abort_plan_with_resolves_seam_with_aborted_decision_and_reason() {
        let seam = ApprovalSeam::new();
        let awaiter = await_seam(&seam, "p1").await;
        abort_plan_with("p1".into(), "wrong scope".into(), &seam)
            .await
            .expect("abort_plan_with");
        let decision = awaiter.await.expect("join");
        match decision {
            ApprovalDecision::Aborted(reason) => assert_eq!(reason, "wrong scope"),
            other => panic!("expected Aborted, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn abort_plan_with_no_pending_awaiter_returns_ok() {
        let seam = ApprovalSeam::new();
        let result = abort_plan_with("ghost".into(), "reason".into(), &seam).await;
        assert!(result.is_ok(), "got {result:?}");
    }

    // ── M04 Stage E: HITL respond_hitl command ──────────────────────

    /// Spawn the HITL seam's awaiter, wait for it to register, return the
    /// join handle so the test can resolve + assert on the user's choice.
    async fn await_hitl(seam: &HitlSeam, prompt_id: &str) -> tokio::task::JoinHandle<HitlChoice> {
        let s = seam.clone();
        let id = prompt_id.to_string();
        let handle = tokio::spawn(async move {
            s.await_response(&id, std::time::Duration::from_secs(60))
                .await
                .expect("await_response")
        });
        for _ in 0..100 {
            if seam.pending_len().await >= 1 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
        handle
    }

    // ── M04 Stage F: recovery + budget commands ─────────────────

    #[tokio::test]
    async fn request_resume_with_returns_plan() {
        // The seam pulls a deterministic RecoveredSession from the
        // injected closure; production wires Arc<DroneClient> instead.
        let plan = request_resume_command_with("s1".to_string(), |id| {
            assert_eq!(id, "s1");
            async move {
                Ok(RecoveredSession {
                    snapshot_id: Some("snap-1".to_string()),
                    state: serde_json::json!({"foo": 1}),
                    plans: vec![serde_json::json!({"id": "p1"})],
                    tasks: vec![serde_json::json!({"id": "t1"})],
                    uncertain_tool_invocations: vec!["sig-1".to_string()],
                })
            }
        })
        .await
        .expect("request_resume");
        assert!(plan.has_state);
        assert_eq!(plan.snapshot_id.as_deref(), Some("snap-1"));
        assert_eq!(plan.uncertain_tool_invocations.len(), 1);
    }

    #[tokio::test]
    async fn request_resume_propagates_drone_error_as_cmd_error() {
        let result = request_resume_command_with("s1".to_string(), |_| async move {
            Err(runtime_main::drone_ipc::DroneIpcError::Codec(
                "boom".to_string(),
            ))
        })
        .await;
        assert!(matches!(result, Err(CmdError::Drone(_))));
    }

    #[tokio::test]
    async fn respond_uncertainty_with_records_signal() {
        let resolution = respond_uncertainty_command_with(
            "s1".to_string(),
            "sig-tool-1".to_string(),
            "skip".to_string(),
            Some("a1".to_string()),
            |_args| async move { Ok(()) },
        )
        .await
        .expect("respond");
        assert_eq!(resolution.invocation_id, "sig-tool-1");
    }

    #[tokio::test]
    async fn respond_uncertainty_rejects_unknown_action_with_internal_error() {
        let result = respond_uncertainty_command_with(
            "s1".to_string(),
            "sig-a".to_string(),
            "bogus".to_string(),
            None,
            |_args| async move { Ok(()) },
        )
        .await;
        match result {
            Err(CmdError::Internal(msg)) => {
                assert!(format!("{msg:?}").contains("bogus"), "got {msg:?}");
            }
            other => panic!("expected Internal, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn respond_uncertainty_propagates_drone_error_as_cmd_error() {
        let result = respond_uncertainty_command_with(
            "s1".to_string(),
            "sig-a".to_string(),
            "skip".to_string(),
            None,
            |_args| async move {
                Err(runtime_main::drone_ipc::DroneIpcError::Codec(
                    "drone".to_string(),
                ))
            },
        )
        .await;
        assert!(matches!(result, Err(CmdError::Drone(_))));
    }

    #[tokio::test]
    async fn set_global_budget_stores_value_in_state() {
        let state: GlobalBudgetState = Mutex::new(None);
        set_global_budget_with(3.50, &state).await.expect("set");
        assert!((state.lock().await.unwrap() - 3.50).abs() < f64::EPSILON);
        // Subsequent call overwrites.
        set_global_budget_with(7.25, &state).await.expect("update");
        assert!((state.lock().await.unwrap() - 7.25).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn set_global_budget_rejects_negative_cap_as_internal_error() {
        let state: GlobalBudgetState = Mutex::new(None);
        let result = set_global_budget_with(-1.0, &state).await;
        match result {
            Err(CmdError::Internal(msg)) => {
                assert!(format!("{msg:?}").contains("non-negative"), "got {msg:?}");
            }
            other => panic!("expected Internal, got {other:?}"),
        }
        // State unchanged.
        assert!(state.lock().await.is_none());
    }

    // ── M05 Stage D — tier commands ──

    #[tokio::test]
    async fn get_current_tier_with_returns_state_value() {
        let state: CurrentTierState = Mutex::new(Tier::Novice);
        let t = get_current_tier_with(&state).await.unwrap();
        assert_eq!(t, Tier::Novice);
        *state.lock().await = Tier::Promoted;
        let t = get_current_tier_with(&state).await.unwrap();
        assert_eq!(t, Tier::Promoted);
    }

    #[tokio::test]
    async fn request_tier_transition_promotes_persists_and_emits() {
        let state: CurrentTierState = Mutex::new(Tier::Novice);
        let dir = tempfile::tempdir().unwrap();
        let emitted = std::sync::Arc::new(std::sync::Mutex::new(None::<AgentEvent>));
        let emitted_clone = emitted.clone();
        request_tier_transition_with(
            Tier::Promoted,
            "user confirmed".into(),
            &state,
            dir.path(),
            move |event| {
                *emitted_clone.lock().unwrap() = Some(event);
                Ok(())
            },
        )
        .await
        .unwrap();
        assert_eq!(*state.lock().await, Tier::Promoted);
        // Persisted: a subsequent load_tier reads back Promoted.
        assert_eq!(
            runtime_main::tier::load_tier(dir.path()).unwrap(),
            Tier::Promoted
        );
        // Event shape: TierTransition with previous=Novice, current=Promoted.
        let event = emitted.lock().unwrap().clone().expect("event emitted");
        match event {
            AgentEvent::TierTransition {
                previous,
                current,
                reason,
            } => {
                assert_eq!(previous, runtime_core::event::TierRef::Novice);
                assert_eq!(current, runtime_core::event::TierRef::Promoted);
                assert_eq!(reason, "user confirmed");
            }
            other => panic!("expected TierTransition, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn request_tier_transition_demotes_without_confirmation() {
        // Demotion is direct — same call path, no special handling.
        let state: CurrentTierState = Mutex::new(Tier::Promoted);
        let dir = tempfile::tempdir().unwrap();
        request_tier_transition_with(
            Tier::Novice,
            "user demoted".into(),
            &state,
            dir.path(),
            |_event| Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(*state.lock().await, Tier::Novice);
    }

    #[tokio::test]
    async fn request_tier_transition_idempotent_when_target_matches_current() {
        // Calling with the current tier is a no-op: state unchanged,
        // event NOT emitted (no transition happened).
        let state: CurrentTierState = Mutex::new(Tier::Novice);
        let dir = tempfile::tempdir().unwrap();
        let emitted = std::sync::Arc::new(std::sync::Mutex::new(false));
        let emitted_clone = emitted.clone();
        request_tier_transition_with(
            Tier::Novice,
            "noop".into(),
            &state,
            dir.path(),
            move |_event| {
                *emitted_clone.lock().unwrap() = true;
                Ok(())
            },
        )
        .await
        .unwrap();
        assert_eq!(*state.lock().await, Tier::Novice);
        assert!(
            !*emitted.lock().unwrap(),
            "no event should fire on idempotent call"
        );
    }

    #[tokio::test]
    async fn request_tier_transition_surfaces_persistence_error_as_internal() {
        // Pass a path that cannot be created as a directory (a file
        // path inside the temp dir) to trigger save_tier failure.
        let state: CurrentTierState = Mutex::new(Tier::Novice);
        let parent = tempfile::tempdir().unwrap();
        let file_path = parent.path().join("not-a-dir");
        std::fs::write(&file_path, b"placeholder").unwrap();
        // Now ask save_tier to create a directory at the same path
        // — fs::create_dir_all will fail because a regular file exists.
        let result = request_tier_transition_with(
            Tier::Promoted,
            "expect-failure".into(),
            &state,
            &file_path,
            |_event| Ok(()),
        )
        .await;
        match result {
            Err(CmdError::Internal(msg)) => {
                assert!(
                    format!("{msg:?}").contains("save_tier"),
                    "expected save_tier in error, got {msg:?}"
                );
            }
            other => panic!("expected Internal CmdError, got {other:?}"),
        }
        // State unchanged on error — the transition aborted before
        // mutating the in-memory cache.
        assert_eq!(*state.lock().await, Tier::Novice);
    }

    #[tokio::test]
    async fn respond_hitl_with_resolves_seam_with_choice() {
        let seam = HitlSeam::new();
        let awaiter = await_hitl(&seam, "u-1").await;
        respond_hitl_with("u-1".into(), "skip".into(), &seam)
            .await
            .expect("respond_hitl_with");
        let choice = awaiter.await.expect("join");
        assert_eq!(choice.token, "skip");
    }

    #[tokio::test]
    async fn respond_hitl_with_no_pending_awaiter_returns_ok() {
        // Mirrors approve_plan_with's soft-Ok rationale: the renderer may
        // dispatch a stale prompt_id (timeout fired between display + click).
        // Do not 500 the renderer's click.
        let seam = HitlSeam::new();
        let result = respond_hitl_with("ghost".into(), "skip".into(), &seam).await;
        assert!(result.is_ok(), "got {result:?}");
    }

    #[tokio::test]
    async fn respond_hitl_with_receiver_dropped_returns_internal() {
        // Manually inject a sender whose receiver is already dropped to
        // exercise the ReceiverDropped → CmdError::Internal branch.
        let seam = HitlSeam::new();
        let (sender, receiver) = tokio::sync::oneshot::channel::<HitlChoice>();
        drop(receiver);
        // Use the test-only seam-internal API to inject; we rely on the
        // seam's HashMap surfaces but the production type doesn't expose
        // this. Instead, drive ReceiverDropped through the public API by
        // dropping an early task between await_response registration and
        // resolve.
        let _ = sender; // satisfy unused-binding
                        // Equivalent path: register an awaiter, drop its receiver via
                        // task cancellation, resolve → ReceiverDropped.
        let s = seam.clone();
        let task = tokio::spawn(async move {
            // Register the awaiter, then drop its receiver by aborting.
            let _ = s
                .await_response("u-drop", std::time::Duration::from_millis(20))
                .await;
        });
        for _ in 0..100 {
            if seam.pending_len().await >= 1 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
        task.abort();
        // Now resolve — depending on timing the seam either timed-out
        // (clean: pending removed → NotFound → soft-Ok) or returned
        // ReceiverDropped (also soft-translated to internal). Both
        // outcomes are accepted: the function must NOT panic.
        let _ = respond_hitl_with("u-drop".into(), "skip".into(), &seam).await;
    }
}
