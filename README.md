# Agent Runtime

> Local Tauri desktop runtime for agentic AI workflows. Live graph of agent execution, capability sandboxing, gap detection that suspends the session cleanly when an agent needs something it doesn't have.

## What this is

A local desktop application (Windows v0.1; Linux + macOS post-v1.0) that runs agentic AI workflows defined as JSON frameworks. Each session executes inside a sandboxed runtime with:

- **Live graph** — every agent spawn, tool invocation, plan/task transition, verify result, and HITL gate renders on the canvas in real time (React Flow v12).
- **Capability enforcement** — tools declare what they need; the runtime narrows agent→agent capability transfers; sandbox process enforces L3 validation for generated artifacts.
- **Gap detection** — when an agent needs a skill, tool, or MCP server it doesn't have, the session suspends cleanly with a structured prompt to fix the framework.
- **Direct Anthropic streaming** — no third-party SDK; `reqwest` + `eventsource-stream` straight to the API.

## What this isn't

A chatbot wrapper. A framework. A general-purpose terminal. A low-code tool for non-technical users in v0.1. The runtime executes what exists; it doesn't modify itself mid-run.

## Stack

- **Shell:** Tauri 2.x (OS webview)
- **Backend:** Rust 1.95.0 (workspace at `crates/`, pinned via `rust-toolchain.toml`)
- **Async:** tokio
- **Frontend:** React 18 + TypeScript + React Flow v12 + Tailwind + Vite
- **LLM client:** direct HTTP + SSE to Anthropic via `reqwest` + `eventsource-stream`
- **Persistence:** SQLite (WAL mode) via `rusqlite`
- **IPC:** Tauri typed IPC (renderer ↔ main); Unix socket / Windows named pipe with framed JSON (main ↔ drone)

Stack rationale: [ADR-0002](docs/adr/0002-tauri-rust-over-electron.md).

## Status

In flight. Per [`docs/MVP-v0.1.md`](docs/MVP-v0.1.md):

- [x] **M01 Foundation** — Cargo workspace + 5 crates + typify codegen + drone Phase 1 + Tauri 2.x shell + React skeleton
- [x] **M02 Event Pipeline** — `LLMProvider` trait + `AnthropicProvider` + IPC + OS keychain + smoke session against live Anthropic
- [x] **M03 Live Graph** — React Flow v12 + 11 node types + SQL inspector + cold-start replay + dagre layout
- [x] **M04 Plan / Verify / HITL / Budget** — Plan FSM + Verify hooks + Rails + HITL (3 UI variants × 9 triggers) + Budget enforcer + Recovery
- [x] **M04.5 Protocol Iteration** — `docs/gotchas.md` graduations + ADR-0007 (in-process HITL seam) + STAGE-PROMPT-PROTOCOL v1.4
- [ ] **M05 Gap + Capability** — §4b gap detection + capability enforcer (L1–L5) + sandbox subprocess + tier system + audit log + GapPanel *(next)*
- [ ] **M06–M11** — MCP basic, Registry import, Workbench Builder Canvas, Generators, First-run + polish, Signed installer *(planned)*

The runtime binary builds; M04 acceptance criteria are met end-to-end. See [`CHANGELOG.md`](CHANGELOG.md) for milestone-by-milestone history.

## Quick start

Prerequisites: Rust 1.95.0 (pinned via `rust-toolchain.toml`), Node 20+, an Anthropic API key.

```bash
git clone https://github.com/kknipe2k/AgentFramework.git
cd AgentFramework
npm install
npm run tauri dev
```

First cold build: 5–10 minutes (~444 transitive Rust deps). Subsequent runs: ~30 seconds. The Tauri window opens automatically; save your API key in the setup panel — it's stored in the OS keychain via `keyring` 3.x, never in plaintext.

For a full IRL feature walkthrough (50+ manual test cases covering M01→M04): [`docs/M04-irl-test-plan.md`](docs/M04-irl-test-plan.md).

## Where to read more

| Document | Purpose |
|---|---|
| [`agent-runtime-spec.md`](agent-runtime-spec.md) | The contract. Capability matrix (§0a), scope matrix (§0d), all phases. |
| [`docs/MVP-v0.1.md`](docs/MVP-v0.1.md) | v0.1 milestone breakdown + acceptance criteria. |
| [`CLAUDE.md`](CLAUDE.md) | Working agreement for AI-assisted development: hard rules, TDD discipline, quality gates, PR workflow. |
| [`docs/adr/`](docs/adr/) | Architecture decision records (currently 0001–0007). |
| [`docs/gotchas.md`](docs/gotchas.md) | 65 named traps surfaced during M01–M04 build. |
| [`docs/gap-analysis.md`](docs/gap-analysis.md) | Cumulative product↔spec audit, per-milestone, append-only. |
| [`STAGE-PROMPT-PROTOCOL.md`](STAGE-PROMPT-PROTOCOL.md) | XML schema for milestone stage prompts (v1.4). |
| [`SECURITY.md`](SECURITY.md) | Vulnerability disclosure process. |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contributing guide + DCO sign-off. |

## ARIA — reference framework

[`examples/aria/framework.json`](examples/aria/framework.json) reconstructs the shell-based ARIA framework as the runtime's reference archetype. ARIA is the test case that proves the §0a Capability Matrix is complete — every primitive listed there must be reconstructible into ARIA's behavior without modifying the runtime. See [ADR-0001](docs/adr/0001-aria-as-archetype.md) for positioning.

The original shell-based ARIA implementation lives at [`.aria/`](.aria/) (moves to `archive/aria-shell/` at v0.1 ship time per CLAUDE.md §10). It runs independently of the runtime; the two products coexist. See [`.aria/CLAUDE.md`](.aria/CLAUDE.md) for ARIA-specific instructions.

## Telemetry

None. The runtime collects nothing about the user — no analytics, no crash reporters, no usage metrics. Adding any phone-home would require an ADR with a public dashboard plan + opt-in mechanism. Default: don't. See `agent-runtime-spec.md` §13 for the full privacy stance.

## License + contributing

Apache 2.0 — see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

Contributions accepted via PR with DCO sign-off (`git commit -s`). See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the engineering charter (TDD, quality gates, schema-as-source-of-truth, ADRs for capability / IPC / schema changes).

Working with Claude Code? [`CLAUDE.md`](CLAUDE.md) is loaded automatically and defines the project's hard rules + PR workflow + quality gates.
