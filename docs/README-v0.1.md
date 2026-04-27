<!--
This file is the v0.1 README. It moves to /README.md at v0.1 ship time,
replacing the existing shell-ARIA README (which moves to
archive/aria-shell/README.md). Until then, both READMEs coexist.
-->

# Agent Runtime

A desktop runtime for agentic AI workflows. Live graph, gap detection, capability sandboxing.

> **Status:** v0.1.0 Windows Preview — pre-release. Built openly.

[![CI](https://github.com/kknipe2k/AgentFramework/actions/workflows/ci.yml/badge.svg)](https://github.com/kknipe2k/AgentFramework/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

<!--
Demo recording goes here once M11 lands. 30–90 second screen capture per
docs/MVP-v0.1.md "Demo recording plan."
-->

![Demo placeholder — drop in once M11 lands](docs/assets/demo-placeholder.png)

## What it does

Loads a framework that defines agents, skills, and tools. Runs a session against your codebase or task. Renders the agent's reasoning as a live graph — every spawned subagent, every tool call, every skill loaded. When the agent needs a capability it doesn't have, the session **suspends cleanly** and shows you exactly what's missing — you fix the gap, the session resumes from the last snapshot.

The differentiator is the **workbench**: novices build their first agentic process by dragging Tools / Skills / Agents onto a canvas and letting a generator write what doesn't exist yet. Experienced users edit the framework JSON directly, import skills from any URL, and use the canvas as visualization.

Both share one app. There's no separate "novice mode" / "power-user mode."

## Status — what works, what doesn't

This is **v0.1.0 Windows Preview**. Honest scope:

### What works
- Drone process for session survival (heartbeat, snapshots, recovery from crashes)
- Live graph rendering of agent execution
- Gap detection: agent requests a capability it doesn't have → session suspends → you install → resume
- Plan → HITL approve → execute → verify → commit cycle
- Capability disclosure on every artifact (plain English at install time)
- Capability enforcement (the runtime intercepts every operation against declared capabilities)
- Sandboxed validation of generated artifacts before install
- Tier-gated install: Novice (manual review every install) and Promoted (auto-accept validated artifacts within bounds)
- Tool / Skill / Agent generators with two-tier review
- Builder Canvas with drag-drop palette, live JSON preview, sandboxed Tester
- MCP server: add by URL, connect, agent uses its tools
- Import skills/agents/tools by URL or local file
- `examples/aria/` reference framework demonstrating the spec end-to-end
- Apache 2.0, signed Windows installer

### What doesn't work yet (deferred to v1.0)
- macOS / Linux — Windows-only in v0.1
- OS-level sandboxing (seccomp / landlock / sandbox-exec) — process boundary only in v0.1; full OS sandbox in v1.0
- Operator tier (full auto-accept of any validated artifact) — v0.1 ships Novice + Promoted only
- Multiple concurrent sessions — single session in v0.1
- Mode router (LITE / STANDARD / FULL / FULL+) — STANDARD hardcoded in v0.1
- Continuous-loop policy (Ralph-style) — `fresh_context_per_task` only in v0.1
- Anthropic skills upstream search UI — v0.1 imports by URL/file only
- Auto-update — manual download from GitHub Releases in v0.1
- `examples/ralph/` framework — present in repo but doesn't run on v0.1 (continuous loop is v1.0)

### What's deferred to v2.0+
- Pluggable community registries
- Plugin system for custom node types
- Team / collaboration mode
- Remote / CI execution
- OpenTelemetry export
- OpenAI / Google / local-Ollama providers (Anthropic only in v0.1 and v1.0)

The full scope matrix is in [`agent-runtime-spec.md` §0d](agent-runtime-spec.md).

## Why this exists

Today's agent loops burn tokens hitting failures with no audit trail. When an agent gets stuck, you usually start over — no record of what it tried, no way to replay decisions, no signal of *what specifically* it lacked.

This runtime captures every decision, surfaces missing capabilities cleanly instead of letting the agent flail, and lets a user (novice or experienced) close those gaps via the workbench rather than starting from scratch each time.

The reference framework `examples/aria/` reconstructs an existing shell-based agentic system ([ARIA](archive/aria-shell/) — moved here at v0.1 release) using only runtime primitives. If that reconstruction works, the runtime's primitives are sufficient for general agentic systems, not just one product's flavor.

## Install

> v0.1.0 hasn't released yet. Once it has:

1. Download the signed `.msi` from [GitHub Releases](https://github.com/kknipe2k/AgentFramework/releases).
2. Verify the signature with `signtool verify /pa /v <file>.msi` (the cert details will be in the release notes).
3. Run the installer. Welcome screen → API key → import or skip → first session.

API key required: get one at [console.anthropic.com](https://console.anthropic.com/settings/keys). Stored in OS keychain; never written to disk by this runtime.

## Try the workbench (novice path)

1. Welcome → "Build my own."
2. Empty Builder canvas. Tutorial overlay walks through the next steps.
3. Click **Generate Tool** → describe what you need ("fetch the contents of a URL"). Generator produces a `tool.md`. Review the capability disclosure ("This tool will: make HTTPS requests to declared hosts. It will NOT: read files, run shell, spawn agents."). Click Install.
4. Click **Generate Skill** → describe ("summarize web articles"). Same flow.
5. Drag both onto the orchestrator agent already on canvas.
6. Click **Test** → enter "summarize https://example.com" → live graph runs → Tester reports pass with output.

You built and ran an agentic process without editing JSON.

## Try the workbench (experienced path)

1. Welcome → "Use ARIA template."
2. Open Settings → switch tier to Promoted (one-time warning explained; accept).
3. Builder → Import → paste GitHub raw URL of any third-party `skill.md` (subject to your trust judgment). Skill is fetched, validated, sandbox-tested, auto-installed because Promoted tier accepts validated artifacts within bounds.
4. Open a real codebase in another window. Click **Start Session** → describe your task.
5. Session runs: orchestrator → planner → analyzer → implementer → verify-app subagents pipeline through tasks.
6. Mid-task, agent calls `request_capability { kind: 'tool', name: 'something' }`. GapPanel opens. Click "Generate Tool" inline; describe; auto-installed (Promoted). Resume.
7. Session completes. Report-writer generates a summary at session end. Audit log is inspectable; full VDR queryable.

## How to build (contributors)

Once the Cargo workspace lands ([M1 in the build checklist](docs/MVP-v0.1.md)):

```bash
git clone https://github.com/kknipe2k/AgentFramework.git
cd AgentFramework
cargo build --workspace
npm install
cargo tauri dev
```

Tests:
```bash
cargo test --workspace
npm run test
```

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full contributor guide and quality gates.

## Architecture (one-paragraph version)

Tauri shell. Rust backend across three processes: **main** (Tauri event loop, Anthropic HTTP+SSE client, MCP client, framework loader, capability enforcement); **drone** (per-session, owns SQLite, snapshots, recovery, IPC); **sandbox** (per-artifact, OS-isolated, runs L3 validation). TypeScript + React + React Flow renderer in the OS webview. Persistence in SQLite (WAL mode). IPC: Tauri typed commands renderer↔main; framed JSON over Unix socket / Windows named pipe main↔drone. No telemetry. No analytics. API keys in OS keychain only.

Full architecture: [`agent-runtime-spec.md`](agent-runtime-spec.md), in particular §1–§4 for runtime, §5–§7 for tools/MCP/registry, §8 for security, §11 for failure modes.

## Disclosures

- **AI-assisted development.** Most of this codebase was written with Claude Code (Anthropic). Direction, review, scope decisions, and final acceptance are by the human maintainer; Claude does the typing. This is disclosed in every commit message format and in [`CONTRIBUTING.md`](CONTRIBUTING.md).
- **Privacy:** the runtime collects nothing about you. No analytics, no telemetry, no crash reporter. Your prompts go to Anthropic's API (your key); your data stays local. See spec §13 for specifics.
- **Provider scope:** v0.1 supports Anthropic only. OpenAI / Google / local-Ollama in v2.0+. The `LLMProvider` trait is in place from day one.

## Prior art

- **[Boris Cherny's subagent patterns](https://www.builder.io/blog/claude-code)** — informed the orchestrator + analyzer/implementer/verify-app/simplifier agent decomposition.
- **[Ralph (autonomous loop pattern)](https://github.com/ghuntley/ralph-wiggum)** — informed the continuous-loop framework (`examples/ralph/`, deferred to v1.0).
- **The existing shell-based ARIA framework** ([`.aria/`](archive/aria-shell/) at v0.1 release) — provided the reference behavior the runtime's primitives must support. ARIA in turn integrates Boris's patterns and Ralph's loop with safety rails, HITL, offline RL, and a verification pipeline.

## License

[Apache 2.0](LICENSE). Patent grant included. DCO sign-off required for contributions ([`CONTRIBUTING.md`](CONTRIBUTING.md)).

## Roadmap

- **v0.1.0 Windows Preview** — current target ([build checklist](docs/MVP-v0.1.md))
- **v1.0** — multi-OS (Linux + macOS), full L2b OS sandboxing, Operator tier, MCP collision UI, Anthropic upstream search, mode router, multi-session, continuous-loop policy, Sigstore signed releases. Estimated 6–12 months after v0.1.0.
- **v2.0+** — pluggable registries, plugin nodes, team/collab, remote/CI execution, OTel, additional providers. Multi-year horizon.

## Spec, ADRs, and history

- [`agent-runtime-spec.md`](agent-runtime-spec.md) — the contract.
- [`docs/MVP-v0.1.md`](docs/MVP-v0.1.md) — what we're building first and how.
- [`docs/adr/`](docs/adr/) — architecture decision records (immutable rationale).
- [`CHANGELOG.md`](CHANGELOG.md) — what's landed; updated per release.
- [`SECURITY.md`](SECURITY.md) — disclosure flow + threat model summary.

## Get involved

Open an issue or PR. We use GitHub Discussions once v0.1 ships. No chat server yet (Slack/Discord/Matrix may be added when there's a community to support).

This is a solo-maintainer project for now; response times vary. Quality > volume of contributions.
