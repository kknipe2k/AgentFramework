//! Anthropic Messages API provider (spec §2c).
//!
//! Stage B ships a STUB: `stream()` returns a hardcoded sequence of
//! `ProviderEvent`s. Stage C replaces the body with direct HTTP+SSE via
//! `reqwest` + `eventsource-stream`. The stub exists so Stages D/E can
//! depend on a stable interface before SSE work lands.

use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use secrecy::SecretString;

use super::{
    AgentConfig, ContentBlock, CostBreakdown, LLMProvider, Message, ModelCapabilities, ModelInfo,
    Pricing, ProviderError, ProviderEvent, ProviderSupport, ToolResultContent,
};

/// Direct HTTP+SSE Anthropic Messages API client.
///
/// API key is loaded from the OS keychain via `keyring` and held in
/// `SecretString` so it never `Debug`-prints. No third-party Anthropic SDK
/// is used (see CLAUDE.md §15 trap #9 + spec §0d).
pub struct AnthropicProvider {
    #[allow(dead_code, reason = "Stage C wires this into the HTTP client")]
    api_key: SecretString,
    #[allow(dead_code, reason = "Stage C uses this as the request base URL")]
    base_url: String,
}

impl AnthropicProvider {
    /// Construct from an API key (loaded by caller from keychain).
    #[must_use]
    pub fn new(api_key: SecretString) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".into(),
        }
    }

    /// Construct with an explicit base URL (for wiremock tests in Stage C).
    #[must_use]
    pub const fn with_base_url(api_key: SecretString, base_url: String) -> Self {
        Self { api_key, base_url }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    #[allow(
        clippy::unnecessary_literal_bound,
        reason = "trait signature borrows from self; other providers may return owned strings"
    )]
    fn name(&self) -> &str {
        "anthropic"
    }

    fn supports(&self) -> ProviderSupport {
        ProviderSupport {
            tool_use: true,
            streaming: true,
            thinking: true,
        }
    }

    /// STUB: returns a hardcoded `text_delta → message_stop` sequence.
    /// Stage C replaces with real HTTP+SSE implementation.
    async fn stream(
        &self,
        _config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        let events = vec![
            ProviderEvent::TextDelta {
                text: "Hello".into(),
            },
            ProviderEvent::TextDelta {
                text: " from".into(),
            },
            ProviderEvent::TextDelta {
                text: " stub.".into(),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
            },
        ];
        Ok(stream::iter(events).boxed())
    }

    async fn count_tokens(&self, messages: &[Message]) -> Result<u64, ProviderError> {
        // Stage B: rough char/4 approximation across all text content blocks.
        // Stage C uses the real /v1/messages/count_tokens endpoint.
        let total_chars: usize = messages
            .iter()
            .flat_map(|m| m.content.iter())
            .map(|block| match block {
                ContentBlock::Text { text } => text.len(),
                ContentBlock::Thinking { thinking, .. } => thinking.len(),
                ContentBlock::ToolUse { input, .. } => input.to_string().len(),
                ContentBlock::ToolResult { content, .. } => match content {
                    ToolResultContent::Text(s) => s.len(),
                    ToolResultContent::Blocks(_) => 0,
                },
                ContentBlock::Image { .. } => 0,
            })
            .sum();
        Ok((total_chars as u64).div_ceil(4))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        // Anthropic does not expose pricing via /v1/models — only model
        // metadata. Pricing is hardcoded against the docs page
        // (https://platform.claude.com/docs/en/about-claude/pricing) and
        // updated when the docs change. Verified 2026-05.
        //
        // Long-context surcharge eliminated 2026-03-13 — uniform per-token
        // rate across the full 1M window for Opus 4.6+ / Sonnet 4.6.
        Ok(vec![
            ModelInfo {
                id: "claude-opus-4-7".into(),
                display_name: "Claude Opus 4.7".into(),
                context_window: 1_000_000,
                pricing: Pricing {
                    input_per_million_usd: 5.0,
                    output_per_million_usd: 25.0,
                },
                capabilities: ModelCapabilities {
                    tool_use: true,
                    streaming: true,
                    thinking: true,
                    vision: true,
                },
            },
            ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Claude Sonnet 4.6".into(),
                context_window: 1_000_000,
                pricing: Pricing {
                    input_per_million_usd: 3.0,
                    output_per_million_usd: 15.0,
                },
                capabilities: ModelCapabilities {
                    tool_use: true,
                    streaming: true,
                    thinking: true,
                    vision: true,
                },
            },
            ModelInfo {
                id: "claude-haiku-4-5".into(),
                display_name: "Claude Haiku 4.5".into(),
                context_window: 200_000,
                pricing: Pricing {
                    input_per_million_usd: 1.0,
                    output_per_million_usd: 5.0,
                },
                capabilities: ModelCapabilities {
                    tool_use: true,
                    streaming: true,
                    thinking: false,
                    vision: true,
                },
            },
        ])
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "token counts stay <2^52 in any realistic cost estimate"
    )]
    #[allow(
        clippy::suboptimal_flops,
        reason = "explicit sum-of-products is more readable than mul_add chains; not in a hot path"
    )]
    fn estimate_cost(&self, b: &CostBreakdown, model: &str) -> f64 {
        // Cache-aware pricing per https://platform.claude.com/docs/en/about-claude/pricing
        // (verified 2026-05). Cache multipliers: 5m write 1.25× input,
        // 1h write 2× input, read 0.1× input.
        let pricing = match model {
            "claude-opus-4-7" | "claude-opus-4-6" | "claude-opus-4-5" => Pricing {
                input_per_million_usd: 5.0,
                output_per_million_usd: 25.0,
            },
            "claude-sonnet-4-6" | "claude-sonnet-4-5" => Pricing {
                input_per_million_usd: 3.0,
                output_per_million_usd: 15.0,
            },
            "claude-haiku-4-5" => Pricing {
                input_per_million_usd: 1.0,
                output_per_million_usd: 5.0,
            },
            _ => return 0.0,
        };
        let input_rate = pricing.input_per_million_usd / 1_000_000.0;
        let output_rate = pricing.output_per_million_usd / 1_000_000.0;

        (b.input_tokens as f64) * input_rate
            + (b.output_tokens as f64) * output_rate
            + (b.cache_5m_writes as f64) * input_rate * 1.25
            + (b.cache_1h_writes as f64) * input_rate * 2.0
            + (b.cache_reads as f64) * input_rate * 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{AgentConfig, Message, MessageRole, Pricing, ProviderEvent};
    use futures::StreamExt;
    use secrecy::SecretString;

    fn stub_provider() -> AnthropicProvider {
        AnthropicProvider::new(SecretString::from("sk-ant-test"))
    }

    fn stub_config() -> AgentConfig {
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

    #[tokio::test]
    async fn stub_stream_returns_text_then_stop() {
        let provider = stub_provider();
        let mut stream = provider.stream(stub_config()).await.unwrap();
        let mut events = vec![];
        while let Some(e) = stream.next().await {
            events.push(e);
        }
        assert!(matches!(
            events.first(),
            Some(ProviderEvent::TextDelta { .. })
        ));
        assert!(matches!(
            events.last(),
            Some(ProviderEvent::MessageStop { .. })
        ));
    }

    #[test]
    fn name_is_anthropic() {
        assert_eq!(stub_provider().name(), "anthropic");
    }

    #[test]
    fn supports_advertises_tool_use_streaming_thinking() {
        let s = stub_provider().supports();
        assert!(s.tool_use && s.streaming && s.thinking);
    }

    #[tokio::test]
    async fn count_tokens_approximates_char_div_4() {
        let provider = stub_provider();
        let messages = vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text {
                text: "hello world".into(),
            }],
        }];
        let count = provider.count_tokens(&messages).await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn list_models_returns_three_claude_4x_entries() {
        let models = stub_provider().list_models().await.unwrap();
        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|m| m.id == "claude-opus-4-7"));
        assert!(models.iter().any(|m| m.id == "claude-sonnet-4-6"));
        assert!(models.iter().any(|m| m.id == "claude-haiku-4-5"));
    }

    #[tokio::test]
    async fn list_models_pricing_values_correct() {
        let models = stub_provider().list_models().await.unwrap();
        let opus = models.iter().find(|m| m.id == "claude-opus-4-7").unwrap();
        let sonnet = models.iter().find(|m| m.id == "claude-sonnet-4-6").unwrap();
        let haiku = models.iter().find(|m| m.id == "claude-haiku-4-5").unwrap();
        assert_eq!(
            opus.pricing,
            Pricing {
                input_per_million_usd: 5.0,
                output_per_million_usd: 25.0
            }
        );
        assert_eq!(
            sonnet.pricing,
            Pricing {
                input_per_million_usd: 3.0,
                output_per_million_usd: 15.0
            }
        );
        assert_eq!(
            haiku.pricing,
            Pricing {
                input_per_million_usd: 1.0,
                output_per_million_usd: 5.0
            }
        );
    }

    #[test]
    fn estimate_cost_simple_for_haiku() {
        let provider = stub_provider();
        let b = CostBreakdown::simple(1_000_000, 1_000_000);
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 6.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_simple_for_sonnet() {
        let provider = stub_provider();
        let b = CostBreakdown::simple(1_000_000, 1_000_000);
        let cost = provider.estimate_cost(&b, "claude-sonnet-4-6");
        assert!((cost - 18.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_simple_for_opus() {
        let provider = stub_provider();
        let b = CostBreakdown::simple(1_000_000, 1_000_000);
        let cost = provider.estimate_cost(&b, "claude-opus-4-7");
        assert!((cost - 30.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_with_cache_writes_5m() {
        let provider = stub_provider();
        let b = CostBreakdown {
            input_tokens: 0,
            output_tokens: 0,
            cache_5m_writes: 1_000_000,
            cache_1h_writes: 0,
            cache_reads: 0,
        };
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 1.25).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_with_cache_writes_1h() {
        let provider = stub_provider();
        let b = CostBreakdown {
            cache_1h_writes: 1_000_000,
            ..Default::default()
        };
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 2.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_with_cache_reads() {
        let provider = stub_provider();
        let b = CostBreakdown {
            cache_reads: 1_000_000,
            ..Default::default()
        };
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 0.10).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_combined_cache_and_io() {
        let provider = stub_provider();
        let b = CostBreakdown {
            input_tokens: 100_000,
            output_tokens: 1_000_000,
            cache_5m_writes: 50_000,
            cache_1h_writes: 0,
            cache_reads: 500_000,
        };
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 5.2125).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_for_unknown_model_returns_zero() {
        let provider = stub_provider();
        let b = CostBreakdown::simple(1000, 1000);
        let cost = provider.estimate_cost(&b, "nonexistent-model");
        assert!(cost.abs() < f64::EPSILON, "got {cost}");
    }
}
