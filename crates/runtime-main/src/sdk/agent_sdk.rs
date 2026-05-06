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

use std::sync::Arc;

use futures::stream::{Stream, StreamExt};
use runtime_core::event::AgentEvent;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::event_pipeline::EventPipeline;
use crate::drone_ipc::{DroneClient, DroneIpcError};
use crate::providers::{AgentConfig, LLMProvider, ProviderError, ProviderEvent};

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

/// Agent SDK. Generic over the LLM provider so v1.0+ providers slot in
/// behind the same trait.
pub struct AgentSdk<P: LLMProvider> {
    provider: Arc<P>,
    event_tx: mpsc::Sender<AgentEvent>,
    drone_client: Arc<DroneClient>,
    session_id: SessionId,
}

impl<P: LLMProvider + 'static> AgentSdk<P> {
    /// Construct with explicit collaborators. Tests inject a no-op
    /// drone via [`DroneClient::noop`].
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
    /// # Errors
    ///
    /// Returns [`SdkError::EventChannelClosed`] if the receiver was
    /// dropped, or [`SdkError::Drone`] if the snapshot trigger failed.
    pub async fn run_agent_with_provider_stream<S>(&self, mut stream: S) -> Result<(), SdkError>
    where
        S: Stream<Item = ProviderEvent> + Unpin,
    {
        let agent_id = format!("agent_{}", Uuid::new_v4());
        self.emit(AgentEvent::AgentSpawned {
            agent_id: agent_id.clone(),
            agent_name: "smoke".to_string(),
            parent_id: None,
            session_id: self.session_id.as_string(),
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

        let mut pipeline = EventPipeline::new(agent_id);
        while let Some(provider_event) = stream.next().await {
            for agent_event in pipeline.next_event(provider_event) {
                self.emit(agent_event).await?;
            }
        }
        for agent_event in pipeline.flush() {
            self.emit(agent_event).await?;
        }
        Ok(())
    }

    async fn emit(&self, event: AgentEvent) -> Result<(), SdkError> {
        self.event_tx
            .send(event)
            .await
            .map_err(|_| SdkError::EventChannelClosed)
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
