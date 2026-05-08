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
use runtime_main::drone_ipc::DroneClient;
use runtime_main::key_store::{read_api_key, write_api_key, KeyStoreError};
use runtime_main::providers::anthropic::AnthropicProvider;
use runtime_main::providers::{AgentConfig, ContentBlock, LLMProvider, Message, MessageRole};
use runtime_main::sdk::{replay_signals_to_events, AgentSdk, SessionId};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

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
) -> Result<(), CmdError> {
    let api_key = read_api_key().inspect_err(|e| {
        tracing::error!(error = %e, "run_smoke_session: read_api_key failed");
    })?;
    let provider = AnthropicProvider::new(api_key.clone());
    let drone_client = Arc::clone(&drone);
    let (tx, rx) = mpsc::channel::<AgentEvent>(64);
    let app_clone = app.clone();
    let forwarder = tokio::spawn(forward_events(rx, app_clone));
    let result = run_smoke_session_with(provider, tx, drone_client, smoke_config()).await;
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
/// # Errors
///
/// Same as [`run_smoke_session`] minus the keychain-read step.
pub async fn run_smoke_session_with<P: LLMProvider + 'static>(
    provider: P,
    event_tx: mpsc::Sender<AgentEvent>,
    drone: Arc<DroneClient>,
    config: AgentConfig,
) -> Result<(), CmdError> {
    tracing::info!("run_smoke_session starting");
    let sdk = AgentSdk::new(Arc::new(provider), event_tx, drone, SessionId::new());
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
        // AppHandle (which is environment-bound). The seam now also takes
        // an `Arc<DroneClient>` per M04 Stage A2 — tests inject `noop`.
        let (tx, mut rx) = mpsc::channel(8);
        let drone = Arc::new(DroneClient::noop());
        let config = smoke_config();
        run_smoke_session_with(StubProvider, tx, drone, config)
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
        let result = run_smoke_session_with(FailingProvider, tx, drone, smoke_config()).await;
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
}
