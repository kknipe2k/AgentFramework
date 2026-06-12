//! Anthropic Messages API provider — direct HTTP+SSE (spec §2c).
//!
//! No third-party Anthropic SDK (CLAUDE.md §15 trap #9 + spec §0d). Direct
//! API hits via `reqwest` + `eventsource-stream` keep the dependency surface
//! minimal and the breaking-change exposure flat.
//!
//! API key is loaded by the caller from OS keychain via `keyring` and held
//! in `secrecy::SecretString` so it never `Debug`-prints. The provider
//! lazily constructs `reqwest::Client` on first `stream()` call.

use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use std::sync::OnceLock;
use std::time::Duration;

use super::anthropic_sse;
use super::retry::{self, RetryPolicy};
use super::stream_guard;
use super::{
    AgentConfig, CostBreakdown, LLMProvider, Message, ModelCapabilities, ModelInfo, Pricing,
    ProviderError, ProviderEvent, ProviderSupport, ToolDef,
};

const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// TCP connect deadline (TD-054). NO global request timeout — a total
/// deadline would kill legitimately long streams; the per-event idle
/// timeout ([`stream_guard::IDLE_TIMEOUT`]) is the liveness instrument.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Direct HTTP+SSE Anthropic Messages API client.
pub struct AnthropicProvider {
    api_key: SecretString,
    base_url: String,
    http: OnceLock<reqwest::Client>,
    idle_timeout: Duration,
}

impl AnthropicProvider {
    /// Construct from an API key (loaded by caller from keychain).
    #[must_use]
    pub fn new(api_key: SecretString) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".into(),
            http: OnceLock::new(),
            idle_timeout: stream_guard::IDLE_TIMEOUT,
        }
    }

    /// Construct with an explicit base URL (for wiremock tests in Stage C).
    #[must_use]
    pub const fn with_base_url(api_key: SecretString, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            http: OnceLock::new(),
            idle_timeout: stream_guard::IDLE_TIMEOUT,
        }
    }

    /// Override the per-event idle timeout (the `with_base_url` test-seam
    /// precedent — the assembled stall regression runs in real time with a
    /// short window; production keeps [`stream_guard::IDLE_TIMEOUT`]).
    #[must_use]
    pub const fn with_idle_timeout(mut self, idle: Duration) -> Self {
        self.idle_timeout = idle;
        self
    }

    fn http_client(&self) -> &reqwest::Client {
        self.http.get_or_init(|| {
            reqwest::Client::builder()
                .pool_max_idle_per_host(2)
                .connect_timeout(CONNECT_TIMEOUT)
                .build()
                .expect("reqwest client builder cannot fail with default features")
        })
    }
}

/// Extract the `retry-after` header seconds, when present and numeric.
/// The default-60 semantics live in [`retry::map_http_error`] (covered).
fn retry_after_secs(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
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

    /// Real HTTP+SSE implementation. Constructs the request, sends it, and
    /// feeds the response byte stream into the private `anthropic_sse`
    /// module's `stream_events`. Production wrapper — excluded from the ≥95%
    /// coverage gate via `--ignore-filename-regex` because real-network hits
    /// are structurally untestable cross-platform. Wire-format logic lives
    /// in `anthropic_sse.rs` (covered by unit tests + wiremock integration).
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError::Auth`] on 401/403; [`ProviderError::RateLimit`]
    /// on 429 (with `retry_after_secs` parsed from the `retry-after` header,
    /// defaulting to 60); [`ProviderError::Overloaded`] on 529 /
    /// `overloaded_error`; [`ProviderError::Api`] on other non-2xx;
    /// [`ProviderError::Http`] on transport failure. 429/500/502/503/529
    /// are retried PRE-STREAM under the bounded [`retry::execute`] policy
    /// (TD-054) — the errors above are post-exhaustion surfaces.
    async fn stream(
        &self,
        config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        let body = AnthropicRequest::from_config(&config);
        let url = format!("{}/v1/messages", self.base_url);

        // Pre-stream bounded retry (TD-054): the op covers send → status
        // check only; a 2xx response exits the retry BEFORE the SSE layer
        // wraps the body, so a stream that yielded events is structurally
        // beyond replay. Mapping + backoff logic live in the covered
        // `retry` module; this excluded wrapper only delegates.
        let client = self.http_client();
        let api_key = &self.api_key;
        let url_ref = &url;
        let body_ref = &body;
        let response = retry::execute(&RetryPolicy::default(), move || async move {
            let response = client
                .post(url_ref)
                .header("x-api-key", api_key.expose_secret())
                .header("anthropic-version", ANTHROPIC_API_VERSION)
                .header("content-type", "application/json")
                .json(body_ref)
                .send()
                .await?;
            if !response.status().is_success() {
                let status = response.status().as_u16();
                let retry_after = retry_after_secs(response.headers());
                let body_text = response.text().await.unwrap_or_default();
                return Err(retry::map_http_error(status, retry_after, &body_text));
            }
            Ok(response)
        })
        .await?;

        // M07.D2 — the SSE layer emits ProviderEvent::Usage with empty
        // model / zero cost (it cannot price). This wrapper owns the
        // pricing table (estimate_cost), so it rewrites those fields
        // here. anthropic.rs is the coverage-excluded production
        // wrapper (CLAUDE.md §5 OS-call holdout), so the closure is
        // covered by the wiremock integration path, not the ≥95% gate.
        let model = config.model.clone();
        let events = anthropic_sse::stream_events(response.bytes_stream())
            .map(move |event| match event {
                ProviderEvent::Usage {
                    input_tokens,
                    output_tokens,
                    ..
                } => {
                    let cost_usd = self.estimate_cost(
                        &CostBreakdown {
                            input_tokens,
                            output_tokens,
                            ..CostBreakdown::default()
                        },
                        &model,
                    );
                    ProviderEvent::Usage {
                        input_tokens,
                        output_tokens,
                        model: model.clone(),
                        cost_usd,
                    }
                }
                other => other,
            })
            .boxed();
        // TD-054: per-event idle timeout at the provider boundary — a
        // stall surfaces the typed timeout through the loop's existing
        // ProviderEvent::Error → AgentError → clean-suspend path.
        Ok(stream_guard::with_idle_timeout(events, self.idle_timeout).boxed())
    }

    /// Real `POST /v1/messages/count_tokens` call per spec §2c.3 (added
    /// M03.5; M04 Stage A2 implements). Replaces the M02 chars/4
    /// approximation now that M04 budget enforcement (Stage F) requires
    /// the actual provider-side count.
    ///
    /// Per <https://platform.claude.com/docs/en/api/messages-count-tokens>:
    /// the response body is `{"input_tokens": <number>}`. The endpoint
    /// uses the same auth headers + `anthropic-version` as `/v1/messages`.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError::Auth`] on 401/403; [`ProviderError::RateLimit`]
    /// on 429 (with `retry_after_secs` parsed from the `retry-after` header,
    /// defaulting to 60); [`ProviderError::Overloaded`] on 529 /
    /// `overloaded_error`; [`ProviderError::Api`] on other non-2xx; and
    /// [`ProviderError::Api`] with a synthetic 0-status if the response
    /// body is missing the `input_tokens` field (provider regression).
    /// Retryable statuses get the same bounded [`retry::execute`] policy
    /// as `stream` (TD-054).
    async fn count_tokens(&self, messages: &[Message]) -> Result<u64, ProviderError> {
        let body = CountTokensRequest {
            // Use a default model when callers haven't set one — count_tokens
            // is independent of the message content's destination. The
            // pricing pages document Haiku-tokenizer drift is below 1%.
            model: "claude-haiku-4-5",
            messages,
        };
        let url = format!("{}/v1/messages/count_tokens", self.base_url);

        let client = self.http_client();
        let api_key = &self.api_key;
        let url_ref = &url;
        let body_ref = &body;
        let response = retry::execute(&RetryPolicy::default(), move || async move {
            let response = client
                .post(url_ref)
                .header("x-api-key", api_key.expose_secret())
                .header("anthropic-version", ANTHROPIC_API_VERSION)
                .header("content-type", "application/json")
                .json(body_ref)
                .send()
                .await?;
            if !response.status().is_success() {
                let status = response.status().as_u16();
                let retry_after = retry_after_secs(response.headers());
                let body_text = response.text().await.unwrap_or_default();
                return Err(retry::map_http_error(status, retry_after, &body_text));
            }
            Ok(response)
        })
        .await?;

        let parsed: CountTokensResponse = response.json().await.map_err(ProviderError::from)?;
        Ok(parsed.input_tokens)
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

/// `/v1/messages/count_tokens` request body. The endpoint accepts the
/// same `model` + `messages` pair as `/v1/messages` plus optional
/// `system` + `tools`, all of which the M04 budget path can extend
/// later. Only `model` + `messages` are required for the v0.1 pre-flight
/// budget check.
#[derive(Debug, Serialize)]
struct CountTokensRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
}

/// `/v1/messages/count_tokens` response body. The single `input_tokens`
/// field carries the count.
#[derive(Debug, serde::Deserialize)]
struct CountTokensResponse {
    input_tokens: u64,
}

/// Anthropic `/v1/messages` request body. Subset of the full API: the parts
/// the §M2 acceptance criteria require, plus tool support.
#[derive(Debug, Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<AnthropicTool<'a>>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct AnthropicTool<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: &'a serde_json::Value,
}

impl<'a> AnthropicRequest<'a> {
    fn from_config(config: &'a AgentConfig) -> Self {
        Self {
            model: &config.model,
            max_tokens: config.max_tokens,
            messages: &config.messages,
            system: config.system_prompt.as_deref(),
            temperature: config.temperature,
            tools: config.tools.iter().map(AnthropicTool::from_def).collect(),
            stream: true,
        }
    }
}

impl<'a> AnthropicTool<'a> {
    fn from_def(def: &'a ToolDef) -> Self {
        Self {
            name: &def.name,
            description: &def.description,
            input_schema: &def.input_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Pricing;
    use secrecy::SecretString;

    fn stub_provider() -> AnthropicProvider {
        AnthropicProvider::new(SecretString::from("sk-ant-test"))
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

    // The M02 `count_tokens_approximates_char_div_4` unit test was deleted
    // at M04 Stage A2 — `count_tokens` now hits the live
    // `/v1/messages/count_tokens` endpoint per spec §2c.3 and would fail
    // when run against `api.anthropic.com` from a unit test. Behavioral
    // coverage moved to `crates/runtime-main/tests/anthropic_wiremock.rs`
    // (4 cases: happy path, 401 auth, 429 rate-limit with retry-after,
    // missing-field response) which exercises the same wire-format path
    // through the real reqwest+json stack.

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
