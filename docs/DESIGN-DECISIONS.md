# Agentic Workbench — Design Decisions

> Living record of decisions made during the planning session that reviewed
> `agent-runtime-spec.md`. This document is the source of truth for the
> *current* state of the design. The original spec remains as a historical
> snapshot of the initial vision.

**Status:** Planning phase complete. Ready to begin v0.1 implementation.
**Scope of v0.1:** ~80–100 hours of implementation work. Windows-only.
Quick Agent framework end-to-end with narration, budget caps, and
first-run experience.

---

## Product, in one sentence

A desktop workbench where agentic-literate non-coders assemble, run, and
share agent workflows — with multiple models working in parallel, every
action visible and interruptible, every run safely budgeted, and a
troubleshoot buddy one click away. Runs on Windows day one.

---

## Item 1 — Target User & Positioning

**User:** Agentic-literate non-coder. Understands agents, skills, MCP, HITL
as concepts. Does not write JSON, markdown, or code.

**Product framing:** A *workbench* for building, running, and sharing
agentic workflows — not a runtime with a builder bolted on. The Builder is
the beating heart of the product.

**Acceptance test for the whole architecture:**

> *Can a non-coder rebuild ARIA inside this app, save it, and share it with
> someone else who can load and run it?*

**Consequences:**

- No JSON or markdown authoring in the primary flow.
- Plain-English narration; no jargon.
- First-Run Experience is Phase 0, mandatory: framework card gallery,
  guided first task, no blank canvas.
- Sharing is a first-class verb, not an export menu item.

---

## Item 2 — Architectural Primitives & Runtime Context

**Design principle:** The runtime is a platform; a framework is a guest.
Frameworks ask the runtime for everything through a single gatekeeper.
This is what enforces policies, enables narration, and makes the Builder
possible.

### Primitive set (the vocabulary the whole app shares)

| Category     | Primitives                                              |
|--------------|---------------------------------------------------------|
| Agents       | Orchestrator, Subagent, Agent role                      |
| Capabilities | Skill, MCP tool, HITL checkpoint                        |
| Memory       | Short-term (session-scoped KV), Long-term (persistent KV) |
| Knowledge    | Corpus (with citations), RAG skill                      |
| Governance   | Policy, access rules, budget caps                       |
| Lifecycle    | Session, snapshot, recovery                             |

### `RuntimeContext` — the single gatekeeper

Exposed to every agent. Provides: `llm`, `memory`, `corpus`, `skills`,
`mcp`, `hitl`, `policy`, `emit`, `snapshot`. Agents never import the
Anthropic SDK, never touch SQLite, never open files. Everything goes
through the context.

### `LLMProvider` interface

Ships Anthropic-only in v1, but defined model-agnostic from day one so
OpenAI/Gemini plug in as additional provider classes later.

### Per-agent model assignment

Each agent role declares its own model. Multiple models from multiple
providers can run in parallel on one task, all feeding one event pipeline,
one memory layer, one graph, one budget.

### Primitive event pipeline

Adds events for memory reads/writes, corpus queries, policy violations,
orchestration plans, subagent delegation, and parallel-running clusters.
Every event has a deterministic `humanSummary()` for the narration layer.

---

## Item 2b — Committees & Workbench Optimizer

### Committees (first-class multi-model decision primitive)

A named group of agents with a decision rule. Six patterns: jury, debate,
critic loop, chain-of-verification, ensemble merge, tiebreaker. Members
can be same-model × N, different models, or different providers. The
Builder renders a committee as a cluster block.

**New events:** `committee_convened`, `committee_member_responded`,
`committee_disagreement`, `committee_decided`.

### Workbench Optimizer (opt-in, propose-only)

Separate subsystem that watches runs and suggests improvements: model
downgrades, parallelism opportunities, committee right-sizing, skill
caching, gap patterns, HITL noise, cost spikes.

- Never modifies anything automatically.
- Off by default.
- Local-only, never phones home.
- v1 is pattern mining + bandit math, not deep RL.
- Requires outcome labeling from Phase 2 onward.

### The synergy

Committees generate labeled training data for the Optimizer for free.
Over time, a user who runs with committees and the Optimizer gets a
workbench that actively learns which models work best for their tasks and
their preferences — no training pipeline, no fine-tuning.

---

## Item 3 — Phase 6.5: Prove Pluralism

Phase 6.5 is the architectural gate between "runtime is built" and
"invest in the Builder." Four starter frameworks ship in Phase 6.5, each
a manifest running on the one generic executor, each exercising different
primitives:

| Framework           | What it does for users                          | Primitives stressed                                        |
|---------------------|--------------------------------------------------|------------------------------------------------------------|
| Quick Agent (ReAct) | Fast one-shot tasks; smart chat with actions   | Minimum viable — single agent + skills                     |
| Research Assistant  | Upload docs, ask questions, get cited answers  | Corpus, RAG skill, long-term memory                        |
| Visual ARIA         | Plan, approve, execute, verify with checkpoints| Every primitive, every HITL pattern, subagent hierarchy    |
| Buddy               | On-demand troubleshoot helper (button)         | Single-shot strategy, runtime dogfooding itself            |

Falsification checklist governs acceptance. Failing any checklist item
means fixing the abstraction, not the framework.

---

## Item 4 — Phase 3: Budget Caps & Plain-English Narration

### Budget caps (four tiers)

| Cap         | Scope                | Default  | Behavior            |
|-------------|----------------------|----------|---------------------|
| Per-run     | One task execution   | $1.00    | Soft pause + HITL   |
| Per-session | Working session      | $10.00   | Soft pause + HITL   |
| Daily       | Rolling 24h          | $25.00   | Hard stop           |
| Monthly     | Rolling 30 days      | $100.00  | Hard stop           |

- Pre-run estimator (range, not point) with one-click "save ~40%" rebalance.
- Persistent cost widget during runs, turns amber/red on approach.
- Caps enforced in `RuntimeContext` — frameworks cannot bypass.
- Per-model cost breakdown.

### Plain-English narration (deterministic, three registers)

- Primary UI surface is the narration panel; the live graph is a toggleable
  secondary view.
- Every event has a `humanSummary()` — **deterministic templating, not
  LLM-generated.**
- Three registers: Headline / Detailed / Technical.
- "What just happened?" button at run completion → single-paragraph recap.
- "Why did it do that?" button on any node/event → plain-English decision
  trace.
- The narration transcript doubles as the shareable workflow documentation
  format.

---

## Item 5 — Three-Tier Survival System

**Two mechanical tiers + one on-demand LLM tier.**

```
┌─────────────────────────────────────────┐
│  Buddy (on-demand LLM, pay-per-click)   │
│  • "Explain this crash"                 │
│  • "Why did it do that?"                │
│  • "Help me fix this"                   │
│  • Zero cost until invoked              │
└─────────────────────────────────────────┘
              ▲ user clicks a button
              │
┌─────────────────────────────────────────┐
│  Process Supervisor (deterministic)     │
│  • Child lifecycle, stdio, restart      │
│  • Can die, Core restarts it            │
│  • $0 cost                              │
└─────────────────────────────────────────┘
              ▲ monitored by
              │
┌─────────────────────────────────────────┐
│  Drone Core (deterministic, tiny)       │
│  • Heartbeats, snapshots, recovery      │
│  • Never dies                           │
│  • $0 cost                              │
└─────────────────────────────────────────┘
```

**Rule:** Events flow upward. Authority flows downward. The Buddy consumes
events and proposes; it has no authority to restart anything. All
survival-critical decisions are deterministic.

- Core is a provably correct watchdog.
- Supervisor can die safely — Core restarts it; sessions recover from
  snapshots.
- Buddy is itself a manifest (single-shot strategy, four skills) —
  dogfoods the architecture.
- Ambient cost of the whole survival system: **$0.**
- Any LLM spend is user-initiated and shown up front.

---

## Item 6 — Frameworks Are Manifests, Not Code

### The retraction

The original spec (and early Item 2 design) treated frameworks as programs
implementing a `FrameworkAdapter` interface. This was wrong. It led to
unnecessary complexity and was inconsistent with the non-coder user model.

### The correct model

- A framework is a **manifest** — declarative data describing agent roles,
  skills, memory, corpora, committees, policies, HITL points, and the
  orchestration strategy.
- The runtime has **one generic executor** that reads any manifest and
  runs it.
- Orchestration strategies (ReAct, plan-then-execute, single-shot, jury,
  debate, chain-of-verification) are **built into the executor** — the
  manifest picks one from a dropdown.
- **Skills are the only place external code lives** — a skill's
  implementation can be a Python script, a TypeScript function, an MCP
  tool, a shell command, whatever.
- Custom Python agents exist as *skills*, not as frameworks.

### Consequences

- **Windows support is native on day one** — no WSL, no bash, no
  cross-platform pain.
- The `ExternalProcessAdapter` is deferred out of v1.
- ARIA is rebuilt as a native manifest assembled in the Builder (Visual
  ARIA). The existing `.aria/` bash scripts stay as a reference spec, not
  a runtime dependency.
- Phase 6.5 gets much cheaper — "four manifests, four strategies, one
  executor" instead of "four language-specific adapters."
- The Builder acceptance test becomes easier — non-coders cannot fail to
  "write code" because there is no code to write.
- No jargon leaks to the user. "TypeScript," "Python," "adapter," "wire
  protocol" never appear in the app.

### What stays pluggable across languages

Three places, all at the skill level:

1. **Skills** — any language, sandboxed, schema-typed.
2. **MCP servers** — external, language-agnostic by design.
3. **Custom agents exposed as skills** — Python programs, callable by any
   framework through the uniform skill interface.

---

## Updated Phase Order

| Phase | Thing                                                                 |
|-------|-----------------------------------------------------------------------|
| 0     | First-Run Experience + Framework Card Gallery                         |
| 1a    | Drone Core (heartbeats, snapshots, recovery)                          |
| 1b    | Process Supervisor (child lifecycle, restart policies)                |
| 2     | Event Pipeline + `LLMProvider` + `RuntimeContext` + outcome labeling  |
| 3     | Narration Panel + Live Graph (secondary) + budget caps + cost widget  |
| 4     | Gap Detection + clean suspension                                      |
| 5     | MCP Manager ("Tool Connections") + Corpus storage + embeddings        |
| 6     | Framework Manifest Loader + generic executor + orchestration strategies |
| 6.5   | Prove pluralism — four starter manifests run on one executor          |
| 7     | Builder: Visual Canvas (primary authoring surface for manifests)      |
| 8     | Builder: Skill Writer (conversational-autonomous)                     |
| 9     | Builder: Registry Search + Skill Marketplace                          |
| 10    | Workbench Optimizer (suggestions inbox, analyzers, rollback)          |
| 11+   | `ExternalProcessAdapter` (if needed), more strategies, more providers |

---

## Ten Things the Design Promises the User

1. Runs native on Windows from day one.
2. Four useful starter frameworks in the gallery immediately.
3. You never see code, JSON, or markdown unless you ask to.
4. Every run shows estimated cost before you confirm.
5. Four-tier budget caps protect against runaway spend.
6. Plain-English narration of everything happening, in real time.
7. "What just happened?" and "Why did it do that?" buttons on every run.
8. Multiple models from multiple providers can work in parallel on one task.
9. Committees (jury, debate, critic loop) are one Builder block away.
10. A troubleshoot Buddy is one click away when anything confuses you;
    never charged unless you use it.

Plus two opt-in advanced capabilities:

- The Optimizer can watch runs and suggest improvements over time.
- Custom Python agents or MCP tools can be added as skills any framework
  can call.

---

## Strategic Context (Not Architectural, But Worth Memorializing)

### Relationship to Claude Code

- **Adjacent, not identical.** Shares primitives (skills, hooks, MCP,
  subagents, Agent SDK) but differs in audience (non-coders vs developers),
  interface (visual vs CLI), extension model (manifests vs code), and
  promises (narration, budget caps, trust layer).
- Build **on top of** Anthropic primitives where possible, not around
  them. Positioning: "the visual workbench that runs Claude Agent SDK
  workflows."
- Compete where Anthropic will be slowest: non-coder accessibility,
  visual-first, framework-agnostic.

### Open source

- **Decision:** Open source, Apache 2.0 license.
- Removes "Anthropic eats this" risk — makes you an API-usage ally rather
  than a competitor.
- Gives non-coders a trust signal closed products can't match.
- Community-contributed framework manifests become the moat.
- Monetization deferred: build the thing, find users, the model will
  reveal itself (hosted version, open core, marketplace, support — all
  available later).

### v0.1 scope and budget

- **Target:** ~80–100 hours of implementation work for a non-embarrassing
  Windows-only v0.1.
- **Fits in v0.1:** Phase 0 first-run + Phase 1a drone core + Phase 1b
  supervisor + Phase 2 event pipeline + Phase 3 narration/graph/cost +
  Phase 6 manifest loader + one working framework (Quick Agent) +
  Windows packaging.
- **Not in v0.1:** Visual Builder, Research Assistant, Visual ARIA, Buddy,
  committees, Optimizer, Skill Writer, MCP Manager UI, `.workflow`
  export/import. These total ~250–400 additional hours.
- **Platform:** Windows only for v0.1. Code written platform-agnostic;
  macOS untested until a Mac collaborator appears. README will say
  "Windows tested; macOS untested, PRs welcome."

---

## Build Order for v0.1 (The Thin Slice)

Proposed execution order — slice-first, not skeleton-first, so there's
something clickable early:

1. Project bootstrap (Electron + Vite + TS + React + Tailwind + SQLite) ~5h
2. `LLMProvider` interface + Anthropic implementation + streaming ~5h
3. `RuntimeContext` + generic executor + ReAct strategy ~9h
4. Event pipeline + deterministic `humanSummary()` templates ~7h
5. Narration panel UI (primary view) ~7h
6. Drone Core + basic snapshots ~7h
7. Process Supervisor ~7h
8. Live graph v1 (React Flow, toggleable) ~7h
9. Cost widget + 4-tier budget caps + pre-run estimator ~5h
10. First-run experience + card gallery (one card) ~5h
11. Quick Agent framework manifest + skill definitions ~3h
12. Integration testing + bug fixing ~10h
13. Polish pass ~5h
14. Windows packaging + installer ~6h

Total: ~88 hours (within the 80–100 budget).

---

*Last updated: during the planning session that replaced the original
spec's code-first framework model with the manifest-based executor model.
Incorporates all six items and two strategic addenda (Claude Code
positioning, open source).*
