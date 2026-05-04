# `runtime-main`

The Tauri main process for Agent Runtime. Hosts the agent SDK, the LLM provider abstraction, and (in later milestones) the framework loader, capability enforcer, and MCP manager.

## What's here

- `providers/mod.rs` — the `LLMProvider` trait, `ProviderEvent` enum, `ProviderError` (thiserror-derived), and supporting types (`AgentConfig`, `Message`, `ContentBlock`, `ModelInfo`, `Pricing`, `CostBreakdown`, `ProviderSupport`) per spec §2c.
- `providers/anthropic.rs` + `providers/anthropic_sse.rs` — `AnthropicProvider` against the Anthropic Messages API (HTTP+SSE; no third-party SDK).
- `sdk/` (M02 Stage D) — `AgentSdk<P: LLMProvider>` agent loop, `EventPipeline` translator, `extract_decision` heuristic.
- `drone_ipc/` (M02 Stage D) — `DroneClient` main-side IPC client connecting to the M01 drone subprocess.
- `key_store.rs` (M02 Stage E) — OS-keychain-backed Anthropic API key storage via the `keyring` crate. Reads return `SecretString` so the key never `Debug`-prints; the platform backend is Linux Secret Service / macOS Keychain Services / Windows Credential Manager.

## Agent SDK

`AgentSdk<P>` wraps any `LLMProvider`, drives the agent loop, and emits typed `runtime_core::AgentEvent`s on a `tokio::sync::mpsc::Sender<AgentEvent>` for the renderer to consume.

```ignore
use std::sync::Arc;
use runtime_core::event::AgentEvent;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{AgentConfig, anthropic::AnthropicProvider};
use runtime_main::sdk::{AgentSdk, SessionId};
use secrecy::SecretString;
use tokio::sync::mpsc;

# async fn _example(api_key: SecretString, drone_addr: &str, config: AgentConfig) {
let provider = Arc::new(AnthropicProvider::new(api_key));
let drone    = Arc::new(DroneClient::connect(drone_addr).await.unwrap());
let (tx, _rx): (mpsc::Sender<AgentEvent>, _) = mpsc::channel(64);
let sdk = AgentSdk::new(provider, tx, drone, SessionId::new());
sdk.run_agent(config).await.unwrap();
# }
```

### `ProviderEvent` → `AgentEvent` mapping

| `ProviderEvent`                    | `AgentEvent` emitted                        | Notes |
|---|---|---|
| `TextDelta { text }`               | (buffered)                                  | Bundled into a single `StreamText` per non-text boundary. |
| `ThinkingDelta { text }`           | `StreamText { text }`                       | Flushes any buffered text first. |
| `ToolUse { name, input, .. }`      | `ToolInvoked { tool_name, source: Builtin, server: None, input }` | Flushes buffer first. M06 refines `source` based on registry. |
| `ToolResult { id, output }`        | `ToolResult { tool_name: "tool_{id}", output, duration_ms: 0 }` | M02 always emits `0` duration; real timing lands in M03. |
| `MessageStop { stop_reason }`      | `AgentComplete { result: stop_reason }`     | Flushes buffer first. Always terminal. |
| `Error { code, message }`          | `AgentError { error: "{code}: {message}" }` | Flushes buffer first. Terminal alongside `MessageStop`. |

A `StreamText` flush also runs the `extract_decision` heuristic; when a `Decision:`/`Rationale:` pair is present, an additional `DecisionRecord` precedes the `StreamText`.

### Drone IPC

`DroneClient` mirrors the M01 drone-server framing (`tokio_util::codec::LinesCodec` over Unix domain socket / Windows named pipe). M02 only sends `DroneCommand::SnapshotNow` on `task_started`; further commands wire as M03+ subsystems land. Reconnect: 5 attempts with exponential backoff (200ms → 400ms → 800ms → 1.6s); `DroneIpcError::Disconnected { retries }` surfaces when the budget is exhausted.

`DroneClient::noop()` is a test-only constructor whose `send` short-circuits to `Ok(())` — use in tests that exercise the SDK loop without a real drone.

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

## Key store + Tauri command surface (M02 Stage E)

The renderer never holds the API key, never speaks HTTP, never touches the filesystem. Privileged actions go through the Tauri command surface in `src-tauri/src/commands.rs`:

- `set_api_key(key: String)` — writes the key to `agent-runtime/anthropic` in the OS keychain via `runtime_main::key_store::write_api_key`.
- `run_smoke_session()` — reads the key, constructs an `AnthropicProvider`, drives the SDK against a hardcoded "Say only the word: hello" prompt with `claude-haiku-4-5` + `max_tokens: 16` + `temperature: 0`, and emits each `AgentEvent` via `app.emit("agent_event", &event)`.

Both commands return `Result<(), CmdError>` where `CmdError` serializes as `{"type":"setup_required"|"provider"|"key_store"|...}` for renderer pattern-matching.

The testable seam `commands::run_smoke_session_with(provider, event_tx, config)` accepts an injectable `LLMProvider` + channel — production wraps it with the keychain read + Tauri `AppHandle` event forwarder.
