//! Drop-mid-stream cancellation-safety tests for `AgentSdk`.
//!
//! Per `docs/build-prompts/M02-event-pipeline.md` §D.4:
//! - Drop the `run_agent_with_provider_stream` future at every observable
//!   point: mid-`TextDelta`, after `ToolUse`, mid-`MessageStop`.
//! - Verify no panic.
//! - Verify the event channel is dropped cleanly (receiver sees Closed).
//!
//! These tests bypass the network and the drone IPC; the `AgentSdk` is
//! constructed with a no-op stub provider (never invoked because we drive
//! `run_agent_with_provider_stream` directly) and a no-op `DroneClient`
//! that short-circuits `send()` without touching a socket.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use runtime_core::event::AgentEvent;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{
    AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
    ProviderSupport,
};
use runtime_main::sdk::{AgentSdk, SessionId};
use tokio::sync::mpsc;
use tokio::time::timeout;

/// Stub provider — never invoked by these tests (we call
/// `run_agent_with_provider_stream` directly). Required only because
/// `AgentSdk` is generic over `LLMProvider`.
struct StubProvider;

#[async_trait]
impl LLMProvider for StubProvider {
    #[allow(
        clippy::unnecessary_literal_bound,
        reason = "trait method returns &str by signature; literal &'static str must be reborrowed"
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
        Ok(Box::pin(futures::stream::empty()))
    }
    async fn count_tokens(&self, _messages: &[Message]) -> Result<u64, ProviderError> {
        Ok(0)
    }
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        Ok(Vec::new())
    }
    fn estimate_cost(&self, _b: &CostBreakdown, _m: &str) -> f64 {
        0.0
    }
}

fn make_sdk(buffer: usize) -> (AgentSdk<StubProvider>, mpsc::Receiver<AgentEvent>) {
    let (tx, rx) = mpsc::channel(buffer);
    let provider = Arc::new(StubProvider);
    let drone = Arc::new(DroneClient::noop());
    let sdk = AgentSdk::new(provider, tx, drone, SessionId::new());
    (sdk, rx)
}

#[tokio::test]
async fn drops_immediately_no_panic() {
    let (sdk, mut rx) = make_sdk(16);
    let stream = futures::stream::iter(vec![ProviderEvent::TextDelta { text: "x".into() }]);
    {
        let _fut = sdk.run_agent_with_provider_stream(stream);
        // Future created but never polled; drop here.
    }
    drop(sdk);
    // Receiver drains to Closed.
    let result = timeout(Duration::from_secs(1), async {
        while rx.recv().await.is_some() {}
    })
    .await;
    assert!(result.is_ok(), "receiver should drain to Closed in <1s");
}

#[tokio::test]
async fn drops_mid_text_burst_no_panic() {
    let (sdk, mut rx) = make_sdk(16);
    // Bounded burst: 200 short text deltas. Long enough that a 10ms
    // cancellation timeout can land mid-burst on most machines, but small
    // enough to bound test memory if the timeout misses.
    let burst: Vec<ProviderEvent> = (0..200)
        .map(|_| ProviderEvent::TextDelta { text: "x".into() })
        .collect();
    let stream = futures::stream::iter(burst).chain(futures::stream::pending());
    let _ = timeout(
        Duration::from_millis(10),
        sdk.run_agent_with_provider_stream(stream),
    )
    .await;
    drop(sdk);
    while rx.recv().await.is_some() {}
    // No panic = pass.
}

#[tokio::test]
async fn drops_after_tool_use_no_panic() {
    let (sdk, mut rx) = make_sdk(16);
    let stream = futures::stream::iter(vec![
        ProviderEvent::ToolUse {
            id: "tu_1".into(),
            name: "n".into(),
            input: serde_json::json!({}),
        },
        ProviderEvent::TextDelta {
            text: "buffered-but-cancelled".into(),
        },
    ])
    .chain(futures::stream::pending());
    let _ = timeout(
        Duration::from_millis(100),
        sdk.run_agent_with_provider_stream(stream),
    )
    .await;
    drop(sdk);
    while rx.recv().await.is_some() {}
}

#[tokio::test]
async fn completes_cleanly_when_stream_ends() {
    let (sdk, mut rx) = make_sdk(16);
    let stream = futures::stream::iter(vec![
        ProviderEvent::TextDelta {
            text: "partial".into(),
        },
        ProviderEvent::MessageStop {
            stop_reason: "end_turn".into(),
            total_tokens: None,
        },
    ]);
    sdk.run_agent_with_provider_stream(stream)
        .await
        .expect("clean run");
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
async fn channel_back_pressure_does_not_panic_on_drop() {
    // 1-cap channel; sends back-pressure quickly. Cancel mid-flight.
    let (sdk, mut rx) = make_sdk(1);
    let stream = futures::stream::iter(vec![
        ProviderEvent::TextDelta { text: "a".into() },
        ProviderEvent::ThinkingDelta { text: "b".into() },
        ProviderEvent::ThinkingDelta { text: "c".into() },
        ProviderEvent::ThinkingDelta { text: "d".into() },
    ]);
    let _ = timeout(
        Duration::from_millis(50),
        sdk.run_agent_with_provider_stream(stream),
    )
    .await;
    drop(sdk);
    while rx.recv().await.is_some() {}
}
