//! M09.5.D red — the ASSEMBLED adversarial acceptance for TD-054: a
//! stalling provider must not hang the run loop. The session suspends
//! cleanly — `run_agent` returns, the typed idle timeout is in the trace
//! as an `AgentError`, and no further provider turn is issued.
//!
//! This drives the REAL composition: `AnthropicProvider` (real reqwest +
//! SSE stack against a raw stalling TCP server) → `AgentSdk::run_agent`
//! (the production multi-turn loop) → the event channel the renderer
//! consumes. The v1.8 falsifiable hypothesis: "the loop's existing
//! `ProviderEvent::Error` → `AgentError` → break path is the clean-suspend
//! consumer for the provider-boundary idle timeout" — this test must
//! disprove it if wrong.
//!
//! Red expectation: FAILS TO COMPILE — `AnthropicProvider::with_idle_timeout`
//! does not exist yet (the §5-endorsed hard-fail red; the behavioral hang
//! proof runs in `provider_resilience.rs`, which compiles today). The
//! short idle override (250ms) exists because the assembled path runs in
//! real time: pausing the clock across `run_agent`'s connect risks
//! spurious `connect_timeout` auto-advance fires (tokio documented hazard).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use runtime_core::event::AgentEvent;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::anthropic::AnthropicProvider;
use runtime_main::providers::{AgentConfig, ContentBlock, Message, MessageRole};
use runtime_main::sdk::{AgentSdk, SessionId};
use secrecy::SecretString;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

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

/// Same raw stall server as `provider_resilience.rs` (wiremock cannot hold
/// a response open mid-body): serves the SSE prelude, then holds the
/// socket forever. Counts served requests so the no-further-turn assertion
/// has an observable.
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
                std::future::pending::<()>().await;
            });
        }
    });
    (addr, requests)
}

/// D.4 scenario 1: "a mid-stream stall suspends cleanly instead of
/// hanging." Outer 10s real-time guard; idle override 250ms.
#[tokio::test]
async fn stalled_provider_suspends_session_cleanly() {
    let (addr, requests) = spawn_stall_server(STALL_SSE_PREFIX).await;
    let provider = AnthropicProvider::with_base_url(
        SecretString::from("sk-ant-test"),
        format!("http://{addr}"),
    )
    .with_idle_timeout(Duration::from_millis(250));

    let (tx, mut rx) = mpsc::channel::<AgentEvent>(256);
    let sdk = AgentSdk::new(
        Arc::new(provider),
        tx,
        Arc::new(DroneClient::noop()),
        SessionId::new(),
    );

    let run = tokio::time::timeout(Duration::from_secs(10), sdk.run_agent(make_config())).await;
    let run_result = run.expect(
        "TD-054: run_agent hung on the stalled stream — the idle timeout never reached the loop",
    );
    run_result.expect(
        "the idle timeout surfaces as an in-trace AgentError and the session ends Ok — \
         not as a run_agent Err",
    );

    // Drop the SDK so its event sender closes and the drain terminates.
    drop(sdk);
    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        events.push(event);
    }

    let agent_error = events.iter().find_map(|e| match e {
        AgentEvent::AgentError { error, .. } => Some(error.clone()),
        _ => None,
    });
    let error = agent_error.expect(
        "the trace must carry the typed timeout as an AgentError (clean suspend observable)",
    );
    assert!(
        error.contains("provider_idle_timeout"),
        "machine tag must reach the trace, got: {error}"
    );
    assert!(
        error.contains("provider idle timeout"),
        "plain-language phrase must reach the trace (rider 2 / close-gate design review), got: {error}"
    );
    assert!(
        error.contains("250"),
        "the Timeout Display's duration figure must reach the trace (rider 2), got: {error}"
    );

    assert_eq!(
        requests.load(Ordering::SeqCst),
        1,
        "the suspended session must issue NO further provider turn"
    );
}
