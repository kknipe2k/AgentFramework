# `runtime-main`

The Tauri main process for Agent Runtime. Hosts the agent SDK, the LLM provider abstraction, and (in later milestones) the framework loader, capability enforcer, and MCP manager.

## What's here (M02 Stage B)

- `providers/mod.rs` — the `LLMProvider` trait, `ProviderEvent` enum, `ProviderError` (thiserror-derived), and supporting types (`AgentConfig`, `Message`, `ContentBlock`, `ModelInfo`, `Pricing`, `CostBreakdown`, `ProviderSupport`) per spec §2c.
- `providers/anthropic.rs` — `AnthropicProvider` shell. Stage B ships a stub with hardcoded events; Stage C lands the real HTTP+SSE implementation against the Anthropic Messages API (no third-party SDK).

## Adding a provider

Implement the `LLMProvider` trait. Keep the impl behind a feature flag if it pulls a heavy transitive tree. The trait does not assume anything about transport — `AnthropicProvider` uses HTTP+SSE; a hypothetical `LocalLlamaProvider` could use a local Unix socket.

## Security notes

- API keys are passed as `secrecy::SecretString` so they never `Debug`-print or appear in logs.
- The actual key value is loaded from the OS keychain at startup (Stage E wires this in for the smoke session).
- No literal API keys in environment variables, files, or source. CLAUDE.md §13 + spec §13 zero-telemetry rule.

## Tests

- `cargo test -p runtime-main` — unit tests for the provider trait, ProviderEvent serde, and the stub Anthropic implementation.
- `cargo test -p runtime-main --features integration` — Stage C adds wiremock-driven integration tests + an opt-in real-API smoke test.
