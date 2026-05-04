//! LLM provider abstraction (spec §2c).
//!
//! v1 ships a single `AnthropicProvider`; the trait abstracts the surface so
//! v1.0+ can add `OpenAI` / local model support without touching SDK callers.
//! All providers stream `ProviderEvent`s; the SDK layer (M02 Stage D) translates
//! these to `runtime_core::AgentEvent`s for the renderer.

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod anthropic;
mod anthropic_sse;

/// Provider-emitted streaming event. Internal to runtime-main; translated to
/// `AgentEvent` at the SDK boundary (Stage D).
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "serde_json::Value contains f64; cannot derive Eq"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderEvent {
    /// Incremental text delta from the model.
    TextDelta {
        /// The text fragment.
        text: String,
    },
    /// Model requested a tool be invoked.
    ToolUse {
        /// Provider-assigned tool-use id.
        id: String,
        /// Tool name.
        name: String,
        /// Input arguments object.
        input: serde_json::Value,
    },
    /// Tool result being fed back to the model (model-side; we mostly emit `ToolUse`).
    ToolResult {
        /// Matching tool-use id.
        id: String,
        /// Tool output value.
        output: serde_json::Value,
    },
    /// Extended-thinking chunk (Anthropic feature; only when supported + enabled).
    ThinkingDelta {
        /// Thinking-text fragment.
        text: String,
    },
    /// Model finished generating; reason in `stop_reason`.
    MessageStop {
        /// Provider stop reason (e.g., `end_turn`, `max_tokens`).
        stop_reason: String,
    },
    /// Provider-side error during the stream.
    Error {
        /// Error code (e.g., `rate_limit`, `overloaded`).
        code: String,
        /// Human-readable message.
        message: String,
    },
}

/// Capability flags reported by the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderSupport {
    /// Provider supports tool-use.
    pub tool_use: bool,
    /// Provider supports SSE streaming.
    pub streaming: bool,
    /// Provider supports extended thinking.
    pub thinking: bool,
}

/// Provider-side error variants. `thiserror`-derived for ergonomic propagation.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// HTTP transport failure.
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),
    /// SSE stream parse failure.
    #[error("SSE parse error: {0}")]
    Sse(String),
    /// Non-success HTTP status from the API.
    #[error("API returned error status {status}: {body}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Response body (truncated).
        body: String,
    },
    /// Authentication rejected (bad / missing API key).
    #[error("Authentication failed (check API key in keychain)")]
    Auth,
    /// Rate-limit hit; retry-after honors the provider's `retry-after` header when present.
    #[error("Rate limit hit; retry after {retry_after_secs}s")]
    RateLimit {
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },
    /// Request timeout.
    #[error("Request timed out after {0:?}")]
    Timeout(Duration),
    /// Caller passed an unknown model id.
    #[error("Invalid model: {0}")]
    InvalidModel(String),
    /// Provider returned a body we couldn't parse.
    #[error("Provider returned unparseable response: {0}")]
    Unparseable(String),
    /// Provider configuration error (e.g., missing endpoint).
    #[error("Provider configuration error: {0}")]
    Config(String),
    /// IO error from local sources (e.g., keychain).
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// JSON serde error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// One conversation message (user / assistant). Spec §2c.
///
/// Mirrors the Anthropic Messages API
/// (<https://docs.anthropic.com/en/api/messages>). System prompts are NOT
/// in the messages array — they go in `AgentConfig::system_prompt` (a
/// separate top-level field per the API).
///
/// `content` is `Vec<ContentBlock>` (not `String`) because the real API uses
/// typed content blocks for multi-part messages: text + images, tool calls
/// + tool results, etc. Single-text messages serialize as a 1-element vec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Author role.
    pub role: MessageRole,
    /// Ordered list of content blocks.
    pub content: Vec<ContentBlock>,
}

/// Message author. The Anthropic API only allows `user` and `assistant` in
/// the messages array; system prompts are a separate top-level parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Human-side message.
    User,
    /// Model-side message.
    Assistant,
}

/// Typed content block per Anthropic Messages API. The variants here match
/// the shapes the API accepts in request bodies AND produces in response
/// content arrays.
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "ToolUse.input is serde_json::Value (contains f64)"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text.
    Text {
        /// The text body.
        text: String,
    },

    /// Image, either as base64 source or URL source.
    Image {
        /// Where the image bytes come from.
        source: ImageSource,
    },

    /// Model-emitted tool invocation (in assistant messages).
    ToolUse {
        /// Provider-assigned tool-use id.
        id: String,
        /// Tool name.
        name: String,
        /// Input arguments object.
        input: serde_json::Value,
    },

    /// Tool result fed back to the model (in subsequent user message).
    ToolResult {
        /// Id of the `tool_use` block this result satisfies.
        tool_use_id: String,
        /// Output payload.
        content: ToolResultContent,
        /// Optional flag set when the tool errored.
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// Extended-thinking block (assistant-only; only when thinking enabled).
    Thinking {
        /// Reasoning text.
        thinking: String,
        /// Provider-emitted thinking signature for verification.
        signature: String,
    },
}

/// Source of an image content block — base64 or URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Inline base64 bytes.
    Base64 {
        /// MIME type (e.g., `image/png`).
        media_type: String,
        /// Base64-encoded image data.
        data: String,
    },
    /// HTTP(S) URL.
    Url {
        /// Image URL.
        url: String,
    },
}

/// Content of a tool result — either a string or a vec of content blocks
/// (e.g., tool returns text + image).
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "Blocks variant carries ContentBlock (serde_json::Value within)"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    /// Plain text result.
    Text(String),
    /// Multi-part result.
    Blocks(Vec<ContentBlock>),
}

/// Per-call agent configuration. Spec §2c.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Model id (e.g., `claude-haiku-4-5`).
    pub model: String,
    /// User+assistant turn history.
    pub messages: Vec<Message>,
    /// Hard cap on output tokens.
    pub max_tokens: u32,
    /// Sampling temperature; provider default if `None`.
    pub temperature: Option<f32>,
    /// Top-level system prompt (per Anthropic API; NOT in `messages`).
    pub system_prompt: Option<String>,
    /// Tool definitions advertised to the model.
    pub tools: Vec<ToolDef>,
}

/// Tool definition advertised to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    /// Tool name (matches `ToolUse.name`).
    pub name: String,
    /// Description shown to the model.
    pub description: String,
    /// JSON Schema for the input arguments.
    pub input_schema: serde_json::Value,
}

/// Pricing info per provider model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pricing {
    /// USD per million input tokens.
    pub input_per_million_usd: f64,
    /// USD per million output tokens.
    pub output_per_million_usd: f64,
}

/// Token-usage breakdown for `estimate_cost`. Cache-aware so M04 budget
/// integration just plumbs the values; no trait refactor needed.
///
/// Cache rates per Anthropic docs (verified 2026-05):
/// - 5-minute cache write: 1.25× input price
/// - 1-hour cache write:   2.0× input price
/// - Cache read:           0.1× input price
///
/// Unknown / unused cache fields default to 0 via [`CostBreakdown::simple`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CostBreakdown {
    /// Plain input tokens.
    pub input_tokens: u64,
    /// Output tokens.
    pub output_tokens: u64,
    /// 5-minute cache write tokens (1.25× input rate).
    pub cache_5m_writes: u64,
    /// 1-hour cache write tokens (2.0× input rate).
    pub cache_1h_writes: u64,
    /// Cache read tokens (0.1× input rate).
    pub cache_reads: u64,
}

impl CostBreakdown {
    /// Simple constructor for callers without cache awareness; cache fields zero.
    #[must_use]
    pub const fn simple(input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            cache_5m_writes: 0,
            cache_1h_writes: 0,
            cache_reads: 0,
        }
    }
}

/// Capability flags carried by a `ModelInfo`.
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent capability flags; not a state machine"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Tool-use supported.
    pub tool_use: bool,
    /// Streaming supported.
    pub streaming: bool,
    /// Extended thinking supported.
    pub thinking: bool,
    /// Vision (image inputs) supported.
    pub vision: bool,
}

/// Information about a single model offered by a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Stable id (used in API calls).
    pub id: String,
    /// Human-readable name.
    pub display_name: String,
    /// Total input-context capacity in tokens.
    pub context_window: u32,
    /// Pricing for this model.
    pub pricing: Pricing,
    /// Capability flags for this model.
    pub capabilities: ModelCapabilities,
}

/// LLM provider trait. Spec §2c.
///
/// All async methods must be cancellation-safe. Implementations should not
/// hold resources past `await` points that wouldn't survive a drop.
///
/// # Examples
///
/// ```
/// use runtime_main::providers::{LLMProvider, anthropic::AnthropicProvider};
/// use secrecy::SecretString;
///
/// let provider = AnthropicProvider::new(SecretString::from("sk-ant-..."));
/// assert_eq!(provider.name(), "anthropic");
/// ```
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider identifier (e.g., `"anthropic"`, `"openai"`).
    fn name(&self) -> &str;

    /// Capability flags for this provider.
    fn supports(&self) -> ProviderSupport;

    /// Open a streaming session against the provider. Stage C lands the real
    /// HTTP+SSE implementation; Stage B's stub returns a hardcoded sequence.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError::Http`], [`ProviderError::Auth`],
    /// [`ProviderError::RateLimit`], or [`ProviderError::Api`] from the real
    /// implementation. The Stage B stub never errors.
    async fn stream(
        &self,
        config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError>;

    /// Pre-flight token count for `messages`. Used by budget controls (M04).
    ///
    /// # Errors
    ///
    /// Returns provider error variants on transport / API failure. The Stage B
    /// stub uses a chars/4 approximation and never errors.
    async fn count_tokens(&self, messages: &[Message]) -> Result<u64, ProviderError>;

    /// List models the provider currently exposes (and their pricing).
    ///
    /// # Errors
    ///
    /// Returns provider error variants on transport / API failure. The Stage B
    /// stub uses a hardcoded table and never errors.
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;

    /// Estimate cost for a token-usage breakdown on a model. Cache-aware
    /// per Anthropic docs (5m write 1.25×, 1h write 2×, read 0.1× input).
    /// Callers without cache awareness use [`CostBreakdown::simple`].
    fn estimate_cost(&self, breakdown: &CostBreakdown, model: &str) -> f64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_event_round_trips() {
        let cases = vec![
            ProviderEvent::TextDelta {
                text: "hello".into(),
            },
            ProviderEvent::ToolUse {
                id: "tu_1".into(),
                name: "search".into(),
                input: serde_json::json!({"q": "rust"}),
            },
            ProviderEvent::ToolResult {
                id: "tu_1".into(),
                output: serde_json::json!({"ok": true}),
            },
            ProviderEvent::ThinkingDelta {
                text: "thinking...".into(),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".into(),
            },
            ProviderEvent::Error {
                code: "rate_limit".into(),
                message: "slow down".into(),
            },
        ];
        for event in cases {
            let json = serde_json::to_string(&event).unwrap();
            let back: ProviderEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event, back);
        }
    }

    #[test]
    fn provider_event_tag_is_snake_case() {
        let json = serde_json::to_string(&ProviderEvent::TextDelta { text: "x".into() }).unwrap();
        assert!(json.contains("\"type\":\"text_delta\""), "got: {json}");
    }

    #[test]
    fn content_block_round_trips() {
        let cases = vec![
            ContentBlock::Text {
                text: "hello".into(),
            },
            ContentBlock::Image {
                source: ImageSource::Base64 {
                    media_type: "image/png".into(),
                    data: "iVBORw0KGgo".into(),
                },
            },
            ContentBlock::ToolUse {
                id: "tu_2".into(),
                name: "search".into(),
                input: serde_json::json!({"q": "rust"}),
            },
            ContentBlock::ToolResult {
                tool_use_id: "tu_2".into(),
                content: ToolResultContent::Text("done".into()),
                is_error: Some(false),
            },
            ContentBlock::Thinking {
                thinking: "step-by-step".into(),
                signature: "sig_abc".into(),
            },
        ];
        for block in cases {
            let json = serde_json::to_string(&block).unwrap();
            let back: ContentBlock = serde_json::from_str(&json).unwrap();
            assert_eq!(block, back);
        }
    }
}
