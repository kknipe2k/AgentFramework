//! M09.5.D red — TD-054 (review C5) adversarial provider-resilience tests.
//!
//! These tests encode the HOSTILE network cases the happy-path wiremock
//! suite (`anthropic_wiremock.rs`) never exercises:
//!
//! - a provider that stops sending bytes mid-stream must surface a TYPED
//!   idle timeout (`ProviderError::Timeout` identity, carried in a
//!   `ProviderEvent::Error`) instead of parking the consumer forever;
//! - a transient HTTP 529 (`overloaded_error`) must be retried invisibly
//!   (pre-stream, bounded), and retry exhaustion must surface typed.
//!
//! Red expectation (the review finding reproduced): the stall test fails
//! as "stream stalled — no typed timeout surfaced" (auto-advance fires the
//! outer virtual guard because no idle timer exists today); the 529 tests
//! fail on request count (no retry exists — exactly one request is seen).
//!
//! The stall fixture is a raw `TcpListener` HTTP/1.1 server (wiremock
//! cannot hold a response body open mid-stream): it serves the SSE prelude
//! then parks on `std::future::pending` — deliberately NOT a tokio timer,
//! so `tokio::time::pause()` auto-advance never wakes it.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use runtime_main::providers::{
    anthropic::AnthropicProvider, AgentConfig, ContentBlock, LLMProvider, Message, MessageRole,
    ProviderEvent,
};
use secrecy::SecretString;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

/// SSE prelude the stall server emits before going silent: enough for the
/// state machine to yield exactly one `ProviderEvent::TextDelta`.
const STALL_SSE_PREFIX: &str = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_stall\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-haiku-4-5\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":1}}}\n\
\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\
\n";

const HAPPY_PATH_SSE: &str = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-haiku-4-5\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":1}}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\
\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":2}}\n\
\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\
\n";

/// Minimal HTTP/1.1 server: accepts connections, reads the request head,
/// writes a 200 `text/event-stream` response with `body` — then HOLDS the
/// socket open forever (read-until-close framing, close never comes).
/// Returns the bound address and a served-request counter.
async fn spawn_stall_server(body: &'static str) -> (std::net::SocketAddr, Arc<AtomicUsize>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind stall server");
    let addr = listener.local_addr().expect("local addr");
    let requests = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&requests);
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65536];
                let mut read = 0usize;
                // Read until the header terminator; the tiny JSON body
                // rides in the same segments and is irrelevant here.
                loop {
                    let n = match sock.read(&mut buf[read..]).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => n,
                    };
                    read += n;
                    if buf[..read].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                counter.fetch_add(1, Ordering::SeqCst);
                let head = "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\n\r\n";
                if sock.write_all(head.as_bytes()).await.is_err() {
                    return;
                }
                if sock.write_all(body.as_bytes()).await.is_err() {
                    return;
                }
                let _ = sock.flush().await;
                // Hold the connection open WITHOUT a timer: paused-clock
                // auto-advance must never wake this task.
                std::future::pending::<()>().await;
            });
        }
    });
    (addr, requests)
}

/// TD-054 adversarial acceptance, provider level: a mid-stream stall must
/// surface a typed idle timeout and end the stream — today it hangs.
///
/// Mechanics: the stream is established and the first event consumed in
/// real time; then the clock pauses. Under a paused clock the runtime
/// auto-advances to the NEAREST pending timer when idle. Post-impl that is
/// the 90s idle timer (the typed timeout fires); today no idle timer
/// exists, so the only timer is the 600s outer guard — the guard firing IS
/// the reproduced hang, in milliseconds of real time.
#[tokio::test]
async fn stalled_stream_surfaces_typed_timeout_and_fuses() {
    let (addr, requests) = spawn_stall_server(STALL_SSE_PREFIX).await;
    let provider = AnthropicProvider::with_base_url(
        SecretString::from("sk-ant-test"),
        format!("http://{addr}"),
    );
    let mut stream = provider
        .stream(make_config())
        .await
        .expect("stream must open against the stall server");

    let first = tokio::time::timeout(Duration::from_secs(5), stream.next())
        .await
        .expect("first event must arrive in real time")
        .expect("stream must yield the prelude event");
    assert!(
        matches!(first, ProviderEvent::TextDelta { .. }),
        "prelude should yield a TextDelta, got {first:?}"
    );

    tokio::time::pause();

    let next = tokio::time::timeout(Duration::from_secs(600), stream.next()).await;
    let Ok(event) = next else {
        panic!(
            "TD-054 red: stream stalled — no typed timeout surfaced within 600s (virtual); \
             the run loop would hang forever on this await"
        );
    };
    let Some(ProviderEvent::Error { code, message }) = event else {
        panic!("expected the typed idle-timeout Error event, got {event:?}");
    };
    assert_eq!(code, "provider_idle_timeout", "machine tag (rider 2)");
    assert!(
        message.contains("provider idle timeout"),
        "plain-language phrase must live in the message (rider 2), got: {message}"
    );
    assert!(
        message.contains("90"),
        "message must carry the Timeout Display's duration figure (rider 2), got: {message}"
    );

    // Rider 1(b): after the synthetic timeout event the stream is FUSED —
    // no further item, ever.
    let after = tokio::time::timeout(Duration::from_secs(600), stream.next()).await;
    assert!(
        matches!(after, Ok(None)),
        "stream must end after the timeout event (fused), got {after:?}"
    );
    assert_eq!(
        requests.load(Ordering::SeqCst),
        1,
        "a stalled stream is mid-stream failure — it must never be replayed"
    );
}

/// TD-054: a transient 529 is retried invisibly — the caller sees one
/// successful stream; the server sees exactly two requests. Red: no retry
/// exists, the server sees one request and the caller sees Err(Api 529).
#[tokio::test]
async fn overloaded_529_then_success_retries_invisibly() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(529).set_body_string(
            r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#,
        ))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(HAPPY_PATH_SSE),
        )
        .with_priority(2)
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let result = provider.stream(make_config()).await;

    let seen = server
        .received_requests()
        .await
        .expect("request recording on")
        .len();
    assert_eq!(
        seen, 2,
        "expected one invisible pre-stream retry (2 requests), got {seen} — 529 is not retried"
    );

    let mut stream = result.expect("529-then-200 must succeed after one backoff");
    let mut text = 0usize;
    let mut stops = 0usize;
    while let Some(event) = stream.next().await {
        match event {
            ProviderEvent::TextDelta { .. } => text += 1,
            ProviderEvent::MessageStop { .. } => stops += 1,
            _ => {}
        }
    }
    assert_eq!(text, 1, "exactly one outcome reaches the consumer");
    assert_eq!(stops, 1, "exactly one MessageStop reaches the consumer");
}

/// TD-054: retry is BOUNDED — three attempts total, then the typed
/// overload error surfaces. Red: one attempt, generic Api error.
#[tokio::test]
async fn overload_exhaustion_surfaces_typed_after_three_attempts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(529).set_body_string(
            r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#,
        ))
        .mount(&server)
        .await;

    let provider =
        AnthropicProvider::with_base_url(SecretString::from("sk-ant-test"), server.uri());
    let result = provider.stream(make_config()).await;

    let seen = server
        .received_requests()
        .await
        .expect("request recording on")
        .len();
    assert_eq!(
        seen, 3,
        "expected exactly three bounded attempts, got {seen}"
    );

    let err = match result {
        Ok(_) => panic!("exhausted retries must surface an error, got Ok stream"),
        Err(e) => e,
    };
    let display = err.to_string().to_lowercase();
    assert!(
        display.contains("overloaded"),
        "the surfaced error must be the typed overload (plain-language Display), got: {display}"
    );
}
