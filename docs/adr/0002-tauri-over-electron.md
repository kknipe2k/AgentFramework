# ADR-0002: Tauri + Rust over Electron + Node.js

**Status:** Accepted
**Date:** 2026-04-18
**Deciders:** @kknipe2k (with Claude analysis and recommendation)
**Tags:** stack, security, performance, oss, scope

## Context

The original spec (`agent-runtime-spec.md` ideation commit `2e36f5b`) prescribed an Electron + Node.js + TypeScript-everywhere stack. That choice was inherited from the initial design without re-examination.

When the project's positioning shifted to **open source from day one** (see `CONTRIBUTING.md`, §12 Engineering Charter, §13 Privacy & Telemetry), the implications for stack choice became more consequential:

- The runtime is a security-critical piece of software (executes generated code, runs continuously, handles API keys, mediates capability-restricted artifacts).
- Open-source projects are scrutinized harder than closed-source on dependency choices, attack surface, and security defaults.
- Users keep this app open all day; bundle size and idle RAM compound.
- §8.security L2 capability enforcement is a *core* primitive (not optional). It needs to be more than best-effort.

A separate consideration: I (Claude) am writing the bulk of the code. So "developer familiarity" — the strongest case for keeping a single-language TypeScript stack — was raised by the user as not a valid reason if the better-for-users choice is a different stack. Per their direction (paraphrased): "I don't care what's elegant from your perspective. Use best practice."

## Decision

We adopt **Tauri + Rust** for the runtime backend (main process, drone, sandbox), with **TypeScript + React + React Flow** preserved for the renderer (using the OS webview that Tauri exposes).

The Anthropic API client is implemented as **direct HTTP+SSE in Rust** using `reqwest` + `eventsource-stream`, not via a third-party SDK crate. This keeps the dependency surface small and removes a class of breaking-change risk.

Renderer ↔ main IPC: **Tauri's typed IPC commands + events** (allowlist-enforced via `tauri.conf.json`).
Main ↔ drone IPC: **Unix domain socket / Windows named pipe with framed JSON** (via `tokio_util` codec).

Capability enforcement gets a real two-layer story:
- **L2a (Application-level):** Rust intercepts every operation in `crates/runtime-main/src/capability/enforcer.rs`. Strong because there is no Node API in the renderer, no `eval`, and Rust's type system rules out whole classes of dynamic-dispatch attacks.
- **L2b (OS-level):** `runtime-sandbox` child process spawned with seccomp-bpf + landlock + namespaces (Linux), sandbox-exec (macOS), Job Objects + AppContainer (Windows). v0.1 ships L2a + process-boundary L2b; full OS sandboxing in v1.0.

## Consequences

### Positive
- **Bundle size:** ~10 MB vs Electron's ~150 MB. ~15× smaller.
- **RAM at idle:** ~50–80 MB vs Electron's ~400–600 MB. ~8× smaller.
- **Startup time:** sub-second cold start vs multi-second on Electron.
- **Security:** real OS-level sandboxing for capability enforcement. The §8.security L2 layer now actually delivers what it promises ("an artifact that declares `shell: false` literally cannot invoke a shell").
- **Battery:** doesn't ship a Chromium per app; uses the OS webview that's already running.
- **Trust posture for OSS:** Rust + Tauri signals serious-tools-for-serious-people; Electron's CVE history is a recurring concern in security-focused communities.
- **Dependency surface:** smaller. Direct HTTP client is fewer crates to track than an SDK shim.
- **No Node API in the renderer:** prompt-injection attempts via the webview have no path to escape (Electron's `nodeIntegration: true` history is a cautionary tale).

### Negative
- **Tauri ecosystem is younger** than Electron's. Some patterns (auto-update, plugins) are less battle-tested. Mitigated by Tauri 2.x stabilizing in 2024 and being used by major projects.
- **Rust learning curve** is real. Mitigated here because Claude is doing the typing; the user's direction time is what's gated by familiarity, not Claude's execution.
- **Anthropic SDK in Rust is community-maintained** (`anthropic-rs` exists but lags). Mitigated by hitting the HTTP API directly — small, stable surface.
- **Cross-OS webview behavior** has known edge cases (WebKit on macOS vs WebView2 on Windows vs WebKitGTK on Linux). Mitigated by testing per-OS in CI; v0.1 is Windows-only so this risk is bounded for the first release.
- **Frontend team must care about Tauri's IPC model** (typed commands, allowlist). Slightly more friction than Electron's "just call Node from the renderer" — but Electron's friction is exactly the problem we're avoiding.

### Neutral / future implications
- v1.0 multi-OS port is when most of the cross-platform Tauri risks materialize. v0.1 Windows-only buys us time to learn.
- A second LLM provider (OpenAI, Google, local-Ollama) is straightforward in Rust via the `LLMProvider` trait (§2c) — no Node SDK comparability needed.
- A future `examples/research/` or other framework that wants browser-like rendering capabilities benefits from being in a real webview, not a custom UI framework.

## Alternatives Considered

### Alternative A: Stick with Electron + Node + TypeScript
Original proposal. Familiar stack, large ecosystem.

**Rejected because:** the only argument for it was developer familiarity, which the user explicitly removed as a factor. Bundle/RAM/security/trust posture all favor Tauri once familiarity is off the table. For a security-critical OSS desktop runtime kept open all day, this stack's costs to users compound daily and over the project's lifetime.

### Alternative B: Native Rust UI (egui / Iced) without webview
Smallest possible binary. Most performant.

**Rejected because:** rebuilding React Flow's live-graph experience in egui or Iced is months of work (the graph IS the differentiator per the spec). Loses access to the React ecosystem, Tailwind, Vite, the entire frontend testing toolchain. Not worth the binary-size win when Tauri already gets us close.

### Alternative C: Bun + Electron (faster startup, native TS)
Replace Node with Bun for the main/drone; keep Electron's renderer.

**Rejected because:** still Electron-heavy. Doesn't fix bundle size or sandboxing. Bun's ecosystem is less mature than Node's for native modules (better-sqlite3, keytar). Marginal improvement, not a step change.

### Alternative D: Wails (Go + webview)
Go alternative to Tauri. Smaller community, less mature.

**Rejected because:** Tauri's community + tooling + Rust ecosystem (cargo, clippy, the entire crate registry) is stronger. Go is great but Rust's type system + borrow checker catch more bugs at compile time, which matters for a security-critical OSS project.

### Alternative E: Web app + Cloud backend (no desktop)
Skip the desktop runtime. Build a web app that calls Anthropic from a server.

**Rejected because:** the spec is explicit that this is a *local* runtime. Users have local files, local MCPs, local secrets. Cloud-hosted defeats the privacy-by-default posture (§13). Different product.

## Related

- Spec section: Tech Stack
- Spec section: §0c Development Loop (cargo tauri dev)
- Spec section: §1 Phase 1 Drone (Rust + tokio)
- Spec section: §1c Multi-Session & SQLite Concurrency (rusqlite WAL)
- Spec section: §1d IPC Channels (Tauri IPC + Unix socket)
- Spec section: §2 Phase 2 SDK Event Pipeline (direct HTTP+SSE)
- Spec section: §2c LLMProvider Abstraction (Rust trait)
- Spec section: §8.security L2 (the security argument that drove this decision)
- Spec section: Project Structure (Cargo workspace layout)
- ADR-0001: ARIA as Archetype (positioning that motivated re-examining stack choices)

## Notes

This ADR documents a stack-choice reversal made on 2026-04-18 after the OSS positioning was confirmed and after honest analysis of where Electron's trade-offs land for this specific product. The decision is reversible only via a successor ADR; a return to Electron would need to demonstrate that Tauri's real costs (younger ecosystem, less Anthropic-SDK maturity) exceed its real benefits (security, bundle, trust posture) — and that's not currently anywhere near true.
