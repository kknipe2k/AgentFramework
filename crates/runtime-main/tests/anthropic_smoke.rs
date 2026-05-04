//! Real-API smoke test for `AnthropicProvider`.
//!
//! Gated by `--features integration` — CI never runs this. Requires a real
//! Anthropic API key in the OS keychain (service `agent-runtime`, user
//! `anthropic`).
//!
//! Cost per run: ~$0.001 against Haiku 4.5 ($1/$5 per `MTok`).

#![cfg(feature = "integration")]

use futures::StreamExt;
use keyring::Entry;
use runtime_main::providers::{
    anthropic::AnthropicProvider, AgentConfig, ContentBlock, LLMProvider, Message, MessageRole,
    ProviderEvent,
};
use secrecy::SecretString;

#[tokio::test]
async fn smoke_real_api_hello() {
    let key = Entry::new("agent-runtime", "anthropic")
        .expect("keyring::Entry::new should succeed")
        .get_password()
        .expect("API key not in keychain — set service=agent-runtime user=anthropic");

    let provider = AnthropicProvider::new(SecretString::from(key));
    let config = AgentConfig {
        model: "claude-haiku-4-5".into(),
        messages: vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text {
                text: "Say only the word: hello".into(),
            }],
        }],
        max_tokens: 16,
        temperature: Some(0.0),
        system_prompt: None,
        tools: vec![],
    };

    let mut stream = provider
        .stream(config)
        .await
        .expect("stream() should succeed against real API");
    let mut text_deltas = 0_usize;
    let mut message_stops = 0_usize;

    while let Some(event) = stream.next().await {
        match event {
            ProviderEvent::TextDelta { .. } => text_deltas += 1,
            ProviderEvent::MessageStop { .. } => message_stops += 1,
            _ => {}
        }
    }

    assert!(text_deltas >= 1, "expected ≥1 text delta, got 0");
    assert_eq!(message_stops, 1, "expected exactly 1 MessageStop");
}
