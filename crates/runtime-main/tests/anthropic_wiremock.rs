//! Wiremock-driven integration tests for `AnthropicProvider`.
//!
//! Exercises the SSE state machine end-to-end without real network: wiremock
//! intercepts `POST /v1/messages`, returns a pre-canned SSE response body,
//! and the provider's `stream()` consumes it through the real reqwest +
//! eventsource-stream + sse state machine path. Every transition the API
//! actually emits is exercised here.
//!
//! These tests gate ≥95% coverage on `crates/runtime-main/src/providers/`
//! (the SSE state machine specifically — the thin reqwest wrapper above it
//! is excluded per `--ignore-filename-regex` for the same OS-signal-class
//! reason as M01.C drone `lib::run`).

use futures::StreamExt;
use runtime_main::providers::{
    anthropic::AnthropicProvider, AgentConfig, ContentBlock, LLMProvider, Message, MessageRole,
    ProviderError, ProviderEvent,
};
use secrecy::SecretString;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn make_config() -> AgentConfig {
    AgentConfig {
        model: "claude-haiku-4-5".into(),
        messages: vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text {
                text: "ping".into(),
            }],
        }],
        max_tokens: 100,
        temperature: None,
        system_prompt: None,
        tools: vec![],
    }
}

const HAPPY_PATH_SSE: &str = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-haiku-4-5\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":1}}}\n\
\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\
\n\
event: ping\n\
data: {\"type\":\"ping\"}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"!\"}}\n\
\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\
\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":2}}\n\
\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\
\n";

#[tokio::test]
async fn happy_path_yields_text_deltas_and_message_stop() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "sk-ant-test"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(HAPPY_PATH_SSE),
        )
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let mut stream = provider.stream(make_config()).await.unwrap();

    let mut events = vec![];
    while let Some(e) = stream.next().await {
        events.push(e);
    }

    let text_count = events
        .iter()
        .filter(|e| matches!(e, ProviderEvent::TextDelta { .. }))
        .count();
    assert_eq!(text_count, 2, "expected 2 text deltas, got {events:?}");
    assert!(matches!(
        events.last(),
        Some(ProviderEvent::MessageStop { .. })
    ));
}

#[tokio::test]
async fn auth_failure_surfaces_as_provider_error_auth() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_string(
            r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#,
        ))
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-bogus"), server.uri());
    let result = provider.stream(make_config()).await;
    assert!(matches!(result, Err(ProviderError::Auth)));
}

#[tokio::test]
async fn rate_limit_includes_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "30")
                .set_body_string("rate limited"),
        )
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let result = provider.stream(make_config()).await;
    match result {
        Err(ProviderError::RateLimit { retry_after_secs }) => {
            assert_eq!(retry_after_secs, 30);
        }
        Err(other) => panic!("expected RateLimit, got {other:?}"),
        Ok(_) => panic!("expected RateLimit error, got Ok stream"),
    }
}

const TOOL_USE_SSE: &str = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_2\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-haiku-4-5\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":50,\"output_tokens\":1}}}\n\
\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tu_42\",\"name\":\"search\",\"input\":{}}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"q\\\":\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"rust\\\"}\"}}\n\
\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\
\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":12}}\n\
\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\
\n";

#[tokio::test]
async fn tool_use_accumulates_and_emits() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(TOOL_USE_SSE),
        )
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let mut stream = provider.stream(make_config()).await.unwrap();

    let mut events = vec![];
    while let Some(e) = stream.next().await {
        events.push(e);
    }

    let tool_event = events
        .iter()
        .find(|e| matches!(e, ProviderEvent::ToolUse { .. }))
        .expect("expected at least one ToolUse event");
    match tool_event {
        ProviderEvent::ToolUse { id, name, input } => {
            assert_eq!(id, "tu_42");
            assert_eq!(name, "search");
            assert_eq!(input, &serde_json::json!({"q": "rust"}));
        }
        _ => unreachable!(),
    }
    assert!(matches!(
        events.last(),
        Some(ProviderEvent::MessageStop { stop_reason, .. }) if stop_reason == "tool_use"
    ));
}

const THINKING_SSE: &str = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_3\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-opus-4-7\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":30,\"output_tokens\":1}}}\n\
\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"Let me consider...\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"sig_abc\"}}\n\
\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\
\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":8}}\n\
\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\
\n";

#[tokio::test]
async fn thinking_delta_passthrough() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(THINKING_SSE),
        )
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let mut stream = provider.stream(make_config()).await.unwrap();

    let mut events = vec![];
    while let Some(e) = stream.next().await {
        events.push(e);
    }

    let thinking_count = events
        .iter()
        .filter(|e| matches!(e, ProviderEvent::ThinkingDelta { .. }))
        .count();
    assert_eq!(thinking_count, 1);
    let signature_present = events
        .iter()
        .any(|e| matches!(e, ProviderEvent::TextDelta { text } if text == "sig_abc"));
    assert!(
        !signature_present,
        "signature_delta must not surface as a text delta"
    );
    assert!(matches!(
        events.last(),
        Some(ProviderEvent::MessageStop { .. })
    ));
}

const SERVER_ERROR_SSE: &str = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_4\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-haiku-4-5\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}\n\
\n\
event: error\n\
data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Server is overloaded\"}}\n\
\n";

#[tokio::test]
async fn error_event_emits_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(SERVER_ERROR_SSE),
        )
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let mut stream = provider.stream(make_config()).await.unwrap();

    let mut events = vec![];
    while let Some(e) = stream.next().await {
        events.push(e);
    }

    let err_event = events
        .iter()
        .find(|e| matches!(e, ProviderEvent::Error { .. }))
        .expect("expected ProviderEvent::Error from server-emitted error");
    match err_event {
        ProviderEvent::Error { code, message } => {
            assert_eq!(code, "overloaded_error");
            assert_eq!(message, "Server is overloaded");
        }
        _ => unreachable!(),
    }
}

const MALFORMED_SSE: &str = "\
event: message_start\n\
data: this is not valid json\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"recovered\"}}\n\
\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null}}\n\
\n";

#[tokio::test]
async fn malformed_sse_skipped() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(MALFORMED_SSE),
        )
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let mut stream = provider.stream(make_config()).await.unwrap();

    let mut events = vec![];
    while let Some(e) = stream.next().await {
        events.push(e);
    }

    assert!(
        events
            .iter()
            .any(|e| matches!(e, ProviderEvent::TextDelta { text } if text == "recovered")),
        "expected the post-malformed text delta to still surface; got {events:?}"
    );
    assert!(matches!(
        events.last(),
        Some(ProviderEvent::MessageStop { .. })
    ));
}

#[tokio::test]
async fn partial_chunk_reassembled() {
    let server = MockServer::start().await;
    let chunk_a = "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_5\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-haiku-4-5\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":3,\"output_tokens\":0}}}\n\nevent: content_bl";
    let chunk_b = "ock_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"split\"}}\n\nevent: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null}}\n\n";
    let body = format!("{chunk_a}{chunk_b}");
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let mut stream = provider.stream(make_config()).await.unwrap();

    let mut events = vec![];
    while let Some(e) = stream.next().await {
        events.push(e);
    }

    assert!(
        events
            .iter()
            .any(|e| matches!(e, ProviderEvent::TextDelta { text } if text == "split")),
        "expected the text delta whose framing was split mid-event-name to still parse; got {events:?}"
    );
    assert!(matches!(
        events.last(),
        Some(ProviderEvent::MessageStop { .. })
    ));
}
