# Review: `agent-runtime-spec.md`

**Reviewer:** Claude (Opus 4.7)
**Date:** 2026-04-18
**Branch:** `claude/review-aria-agentic-wrapper-Xz7Y3`
**Subject commit:** `2e36f5b Create agent-runtime-spec.md`
**Spec size:** 795 lines, 29 KB

---

## Context (what this review is measured against)

The ideation commit lands a single-author product spec describing a **desktop Electron runtime** (drone process, SDK event pipeline, React Flow live graph, gap detection, agent builder, SQLite VDR store). It is the only content in this "review-aria-agentic-wrapper" branch — no code, no RFC discussion, no ADR, no migration notes.

The existing repo is a mature shell-based orchestration system:

- **13,190 LOC** across shell + python
  - `aria-engine.sh` 717L, `ralph.sh` 1066L, `common.sh` 1602L
  - `verify.sh` 701L, `model-selector.sh` 1177L
  - `serve-dashboard.py` 1066L, `offline-learner.py` 629L
- **17 skills** with semantic trigger docs and mode variants
- **4 subagents** (`analyzer`, `implementer`, `verify-app`, `code-simplifier`)
- **Thompson-Sampling offline RL** (`.aria/learned/priors/`)
- **Signal Schema v2** with 8 signal types, decision trace, reconciliation
- **Router/mode system** (LITE / STANDARD / FULL / FULL+) with explicit sizing matrix in `CLAUDE.md`
- **Rails system** (`.aria/rails/safety.json`) with hard/soft blocks
- **Ralph PRD-driven autonomous loop** with HITL fallback
- **Web dashboard** (`:8420`) tying it all together

**The spec never references any of this. That is the single most important observation in this review.**

---

## What's Working

### 1. The drone-process idea is genuinely good

Phase 1's dedicated survival process — heartbeat, append-only snapshots, PID ownership, SIGTERM emergency snapshot, recovery offered on relaunch — is the right pattern and is *missing* from the existing ARIA. Shell-ARIA has checkpoints (git stash in `verify.sh`) but has no equivalent to "laptop closed mid-run, resume from snapshot." This would be a real capability gain. The `DroneEvent`/`DroneCommand` typed message protocol (L142–161) is clean.

### 2. Build-time vs runtime separation is architecturally sound

> "Skill finding, writing, and testing is a **build-time activity** ... The runtime executes what exists. It does not modify itself mid-run" (L9).

This is a principled stance that avoids the whole class of self-modification bugs. The existing codebase blurs this line — skill-writer-style autonomous generation doesn't currently exist, but the spec draws a good boundary if this is ever added.

### 3. Clean suspension on gap is a good UX primitive

The GapNode → pause → Builder → resume flow (Phase 4) is better than the ARIA equivalent (3-failure escalation to HITL, which is reactive). Detecting "this session needs a capability we don't have" *before* burning 3 failures is a real improvement.

### 4. Event-typed SDK wrapper is the right design

The single `AgentEvent` union in Phase 2 (L183–198) is the correct place to draw the abstraction. Having graph, VDR, dashboard, and cost tracking all subscribe to one typed event stream is clean.

This parallels Signal Schema v2 in `.aria/docs/SIGNAL-SCHEMA-V2.md` — the spec's event taxonomy is actually *less rich* than what ARIA already defines (no `context_type`, no retry chains, no skill-load events distinct from skill-invoke), which is a regression to address (see gaps).

### 5. Secrets vault via OS keychain (keytar) is correct

Short section (L649–659) but the policy is right: main process only, never renderer, never in snapshots/VDR. ARIA today has no equivalent — API keys live in env vars and are implicitly trusted.

### 6. Persistence schema is sensible for v1

The SQLite schema (L570–644) is adequate. `state_hash` for snapshot deduplication is a nice touch. Using `auth_key_ref` (reference, not value) in `mcp_servers` is the right call.

---

## What's Not Working

### 1. The spec ignores the existing codebase entirely

This is the fundamental problem. A reader of just this spec would not know ARIA exists. Concretely:

- **Framework JSON (L378–415)** is a flat object with `system_prompt`, `tools`, `agents[]`, `skills[]` (as string names). This cannot express ARIA's mode router (LITE/STANDARD/FULL/FULL+), HITL policies beyond `"on_gap"`, verification pipelines, rails, mode-variant behaviors in skills, sizing criteria, or the planning/execution split. "Aria ships as the built-in default framework" (L426) is not feasible with this schema without a large extension.

- **Skills in the spec are tools** (L502–533 `skill.md` has `input_schema`/`output_schema` frontmatter). Skills in ARIA are context-loaded markdown prompts with semantic triggers and mode variations — closer to prompts-as-code than callable tools. These are incompatible models called the same thing. The spec needs to either:
  - (a) acknowledge skills-as-prompts is a separate concept and name the callable thing "tool," or
  - (b) explicitly subsume the existing ARIA skill model and show how `planning.md` or `executing.md` would be represented.

- **No planning model.** ARIA's central contract is: plan → HITL-approve → execute one task at a time → verify after each → commit → next. The spec has `AgentNode` and `HITLNode` but no plan, no tasks, no acceptance criteria, no approval gate. The whole verify-then-proceed discipline that makes ARIA trustworthy is absent.

- **No verification or rails.** The word "verify" appears only in Phase 1 (heartbeat status verification) and Phase 7 (skill install validation). `.aria/verify.sh` (701L, with git-stash rollback), `.aria/rails-executor.sh`, the 5-layer verification levels, and `.aria/rails/safety.json` have no analog in the spec. Without these, it's a visualizer for a chatbot loop.

- **No budget/cost enforcement.** Spec has `token_usage` table and `token_usage` event, but no budget cap, no per-framework limit, no model-downshift logic. ARIA's `model-selector.sh` (1177L) and `offline-learner.py` (Thompson Sampling) are the single most sophisticated subsystem in the repo and have no place in the spec.

- **Mode router is absent.** `CLAUDE.md`'s sizing matrix (tasks × LOC × files × deps × auth scope → LITE/STANDARD/FULL/FULL+) drives ARIA's entire behavior. The spec has one mode.

### 2. "Skill missing" detection is underspecified

`skill_missing` is the trigger for the GapNode + session suspension (Phase 4). The spec never says **how** a missing skill is detected. Options:

- **Static:** framework JSON declares `skills: ["foo"]`, local library lacks `foo` → gap at load time. Fine but shallow; this is just pre-flight validation, not runtime gap detection.
- **Dynamic:** model emits a structured "I need a skill called X" tool call. This requires a meta-tool (`request_capability`) injected into every prompt, plus training/system-prompt discipline so the model actually uses it. Spec doesn't mention this.
- **Heuristic:** agent repeatedly fails or asks for something outside its toolset. Requires a classifier.

**The strongest differentiator in the spec hinges on a mechanism that isn't designed.** This needs a section.

### 3. Autonomous skill writer has no security model

Phase 8 (L491–499) describes autonomous skill generation by Claude, validated in a sandbox, then installed. The validator is described as (L449–455):

- Parse skill.md → valid format
- Run against mock inputs in isolated sandbox
- Check output schema matches declared schema
- **"Flag dangerous patterns (undeclared network calls, exec, etc.)"**

Static analysis of LLM-generated code for "dangerous patterns" is an unsolved problem; regex on `exec(`/`require('child_process')` is not a security boundary. If skills can execute code, the threat model needs to be explicit: container? Deno-style permissions? No I/O at all (pure JSON transforms)?

Without that, autonomous skill writing is an arbitrary-code-execution pipeline with a LLM at the top. **This is the single most dangerous feature in the spec and it gets ~8 lines.**

**Recommendation:** either
- (a) autonomous mode is gated behind a mandatory human approval step with a full diff review, or
- (b) skills run in a capability-restricted runtime (Deno `--allow-*`, or WASM sandbox), or
- (c) skills are declarative-only (no code, just prompts + tool-use specs) for v1.

### 4. Registry is imaginary

L470–473 lists registries:
- Anthropic skills repo index (real)
- `mcp.so/api/search` (real-ish)
- "community registries — pluggable" (imaginary)
- "agent.md registry — GitHub index of community agent definitions" (L438) — **this doesn't exist**

Trust chain, signature verification, malicious-package protection, version pinning/lockfile, dependency resolution across skills — all undefined. Shipping a platform whose Phase 7 depends on community registries that don't exist is a hard pivot risk; at minimum, ship v1 with a local-only library and one vetted upstream (Anthropic skills).

### 5. MCP manager design is shallow

"Tool namespace collision detection and resolution" (L356) — no algorithm given. Prefix by server name? First-wins? Fail loud? Per-server tool approval (some servers should be read-only, some can write) not addressed. Retry/backoff policy is "configurable" but no defaults or rate-limit handling shown.

### 6. Provider abstraction is aspirational

"Claude family first, frontier model agnostic long term" (L11) — but every code example imports `@anthropic-ai/sdk` directly. No port layer, no provider interface, no token-counting abstraction. "Long term" is doing a lot of work here; it will be painful to add later if `AgentSDK` wraps Anthropic concretely.

### 7. Concurrency / multi-session model missing

- Can a user run two frameworks simultaneously?
- Is the drone one-per-session or singleton?
- If singleton, how does it track concurrent session state?
- If per-session, how do multiple drones coordinate over one SQLite file (WAL is fine but not mentioned)?
- What happens if two sessions both claim the same MCP server with different auth configs?

### 8. IPC choice (stdio newline-JSON) is brittle

Starting prompt L764: "Accept commands from main process via `process.stdin` (newline-delimited JSON) ... Emit events ... via `process.stdout`."

Any incidental print-to-stdout in the drone (library warnings, a stray `console.log`) corrupts the stream. Standard fix: dedicated IPC channel (`child_process.fork` gives you `process.send`/`.on('message')`), or Unix socket, or a tiny named-pipe protocol. Spec should specify.

### 9. Observability regresses from what ARIA has today

The spec's VDR (L228–242) has:
- id, session_id, agent_id, timestamp, decision, rationale, tool, input, output, token_cost, outcome, snapshot_id.

ARIA's Signal Schema v2 has **8 distinct signal types** with pre/post events, retry chains, parent-signal correlation, context classification (`skill|framework|code|search|verify|commit|subagent`), output previews, duration, etc.

The spec should explicitly inherit or re-derive Signal Schema v2, not re-invent a weaker version.

### 10. The starting prompt (L752–780) under-specifies and over-commits

It tells Claude to write tests for the drone, but tests for Electron IPC + SQLite concurrency + SIGTERM handling in a single first session is ambitious. More importantly, **it doesn't tell Claude to read the existing ARIA codebase first** — which is where all the prior art for verification, rails, signal schema, and HITL lives.

---

## Critical Gaps (must address before anyone writes Phase 1 code)

Ordered by how much damage they do if ignored.

### 1. Relationship to existing ARIA — undefined

Replace? Coexist? Migrate? This is a binary decision that drives everything else. Options:

- **Replace:** delete `.aria/`, port skills to new format, rewrite in TS. Highest value, highest risk, loses 9 months of shell tooling.
- **Wrapper:** runtime shells out to `.aria/verify.sh`, `.aria/ralph/ralph.sh`, etc. Electron is the face, ARIA is the engine. Pragmatic; easiest Phase 0.
- **Parallel:** new product alongside ARIA, different user. Dilutes focus.

**Pick one and write it into Section 1.** Without this, every subsequent design choice is ambiguous.

### 2. Verification and safety rails story

ARIA's differentiator is "autonomous but safe." The spec removes the rails. At minimum:

- A `VerifyNode` type in the graph, triggered after each agent action that modifies files
- A rails concept in the framework JSON (hard blocks, soft warnings)
- Rollback primitive (the drone already has snapshots — extend with a "revert to snapshot N" command)
- Project-context / don't-touch zones

### 3. Planning model

Add a `PlanNode` / task model to the graph. Even in LITE mode, most real work breaks into tasks that want individual verify + commit. Without this, the graph is a flat execution trace, not a plan-driven workflow.

### 4. Disambiguate "skill" vs "tool"

Two options:

- Call callable things **tools** (MCP, in-process functions with input/output schemas) and **skills** remains the ARIA concept (context-loaded markdown with triggers and mode variations). Framework JSON then has `tools: [...]` and `skills: [...]` distinct.
- Keep one term but be explicit: skills are callable with schemas (spec's current model), and the ARIA skills/ directory will be renamed/reworked.

The current spec silently collapses these and will cause developer confusion.

### 5. Gap detection mechanism

Write down how `skill_missing` is actually emitted. The honest answer is probably: inject a meta-tool (`request_capability` or `tool_not_available_for`) into every system prompt, detect it in the event pipeline. Fine — but say so, with the system-prompt template.

### 6. Skill-writer security model

Either downgrade autonomous mode to "AI drafts → mandatory human review diff → install" in v1, or specify a real sandbox (Deno permissions, WASM, container). Don't ship autonomous arbitrary-skill-install with regex-based "danger detection."

### 7. Budget / cost enforcement

Add per-framework token caps, per-session USD cap, model-downshift thresholds. Reuse `offline-learner.py` logic if possible. A runtime without budget controls will produce $500 surprise bills.

---

## Important (should address in spec before coding)

1. **Signal schema unification.** Inherit Signal Schema v2 explicitly, or document what's dropped and why.
2. **IPC channel.** Specify `child_process.fork` + `process.send`, or socket, not stdio-JSON.
3. **Multi-session semantics.** One drone per session vs singleton; SQLite WAL mode; MCP-server sharing rules.
4. **MCP tool namespace collision rule.** Prefix by server name, with explicit override allowed. Written down.
5. **Registry trust chain.** For v1, pin to one vetted source (Anthropic skills repo). Defer community registries to v2 with a proper design.
6. **Provider abstraction.** Introduce `LLMProvider` interface in Phase 2 with Anthropic as the only implementation, rather than baking `@anthropic-ai/sdk` into `AgentSDK`.
7. **Recovery correctness.** "Resume rebuilds SDK message history from snapshot, reconnects MCPs, restores graph state" (L133) — if the model used tool results that depended on external state (a web fetch), replay is non-deterministic. Document that resume rebuilds *history* but execution continues fresh, not that the session re-runs.
8. **Mode router import.** Port the LITE/STANDARD/FULL/FULL+ sizing as a framework-level concept or explicitly discard it with rationale.
9. **HITL beyond `on_gap`.** Add HITL policy modes: `on_gap`, `on_risky_tool`, `per_task`, `never`. Spec has one value.
10. **Dev-loop story.** How does someone iterate on a framework without packaging Electron? Need `npm run dev` with hot reload across main/renderer/drone.

---

## Nice-to-Haves (augmentation)

- **Graph replay** from VDR + snapshots — rewatch a session step by step. Existing dashboard has some of this; native replay in the live graph would be stronger.
- **Plugin node types.** Let frameworks define custom node renderers.
- **Team / collaboration mode.** Session shareable via exported `.aria-session` bundle (snapshots + VDR + graph state).
- **Remote / CI execution.** Run a framework headlessly on a server, stream events to a connected Electron client.
- **Skill signing and lockfile.** `skills.lock` with hashes; signed skill manifests from trusted registries.
- **Telemetry export.** OpenTelemetry-compatible span export for enterprise users who want to centralize traces.
- **Diff-view for autonomous skill writer.** Even gated autonomous mode benefits from a side-by-side diff + test-run preview.
- **Graph export to PNG/SVG** for postmortems and docs.

---

## Spec-Level Quality Issues

- **L743 typo:** `AGENT_RUNTIME_SPEC.md` in project structure vs actual filename `agent-runtime-spec.md`.
- **L385 model ID:** `claude-sonnet-4-20250514` — if this spec is meant to ship in 2026, update to current IDs (`claude-sonnet-4-6` or `claude-opus-4-7`).
- **L46–48 schema duplication:** SQLite table `sessions.snapshot_count` duplicates data you can `COUNT(*)` from snapshots. Minor but the schema has similar minor redundancies.
- **No pseudocode for the core reconciliation:** how events → graph state → VDR → dashboard refresh. One sequence diagram is worth 100 lines of prose here.
- **No failure modes section** for what happens when the Anthropic API is down and the drone is healthy but can't do anything. The drone error matrix (L165–172) is one row for this; the runtime needs a matching "degraded mode" UX.

---

## Bottom Line

The spec describes a credible product vision (Electron runtime with live graph, drone-backed session survival, build-time/runtime split) and has two genuinely strong architectural ideas (the drone, build-time isolation). **It is worth building.**

But as a plan for **this repo, right now, as ARIA's next chapter**, it is 60% there:

- **Strong:** drone, build-time split, gap → suspend UX, event pipeline shape, SQLite persistence.
- **Missing:** how any of this relates to the existing ARIA codebase (the #1 issue), verification/rails/rollback (ARIA's whole point), planning model, budget/cost, mode router, skill-vs-tool clarity, skill-writer security model, gap-detection mechanism.
- **Risky handwaves:** community registries, "frontier-model agnostic long term," autonomous skill generation, stdio-JSON IPC.

Before anyone starts Phase 1, the spec needs a preceding section that answers:

> **What happens to the existing ARIA codebase, and which of its capabilities (verify.sh, rails, planning, mode router, offline RL, signal schema v2, subagents) are inherited vs replaced vs dropped?**

Everything else follows from that answer.

---

## Appendix: Prior-Art Inventory (what the spec should inherit)

| Existing ARIA capability | File / location | Spec equivalent | Gap |
|---|---|---|---|
| Verification pipeline | `.aria/verify.sh` (701L) | — | Missing |
| Safety rails | `.aria/rails/safety.json`, `rails-executor.sh` | — | Missing |
| Planning + HITL approval | `.aria/skills/planning.md`, `current-plan.json` | `HITLNode` (weak) | Missing plan model |
| Mode router (LITE/STANDARD/FULL/FULL+) | `CLAUDE.md` sizing matrix | — | Missing |
| Subagent orchestration | `.claude/agents/{analyzer,implementer,verify-app,code-simplifier}.md` | `AgentNode` (generic) | Roles undefined |
| Signal schema v2 (8 types) | `.aria/docs/SIGNAL-SCHEMA-V2.md` | `AgentEvent` (weaker) | Inherit & extend |
| Decision trace | `.aria/state/decisions.jsonl` | VDR table | Re-invented weaker |
| Offline RL (Thompson Sampling) | `.aria/lib/offline-learner.py` (629L) | — | Missing |
| Model selector (budget-aware) | `.aria/model-selector.sh` (1177L) | — | Missing |
| Ralph autonomous loop | `.aria/ralph/ralph.sh` (1066L) | — | No autonomy story |
| HITL notification system | `.aria/hitl.sh` (632L) | `HITLNode` (graph-only) | No notifier |
| Git checkpoint / rollback | `.aria/git-ops.sh` (531L) | — | Partial via snapshots |
| Rails hooks | `.claude/hooks/aria-rails.sh` | — | Missing |
| Dashboard | `.aria/dashboard/index.html` (1352L), `serve-dashboard.py` (1066L) | Live graph | Replace or unify |

---

*End of review.*

