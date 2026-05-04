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

- `cargo test -p runtime-main` — unit tests + 8 wiremock-driven integration tests for the SSE state machine + provider HTTP path. Offline; no API key needed.
- `cargo test -p runtime-main --features integration` — adds the real-API smoke test in `tests/anthropic_smoke.rs` (gated). Requires a key in the OS keychain; CI never runs this.

## Real-API smoke test

The provider integration tests use `wiremock` for offline CI. To exercise the real Anthropic Messages API end-to-end:

1. Get an API key from <https://console.anthropic.com> (Settings → API Keys → Create Key).
2. Store it in the OS keychain under service `agent-runtime`, user `anthropic`:
   - **Windows:** open Credential Manager → Add a Generic Credential. Set `Internet or network address` to `agent-runtime` and `User name` to `anthropic`; paste the API key in `Password`.
   - **macOS / Linux:** any platform secret manager that writes the same `service=agent-runtime, user=anthropic` entry the `keyring` crate reads.
3. Run: `cargo test -p runtime-main --features integration --test anthropic_smoke`.

Cost per run: ~$0.001 against Haiku 4.5 ($1/$5 per million tokens). CI never runs this test; the wiremock tests in `tests/anthropic_wiremock.rs` cover the same wire-format paths offline.
