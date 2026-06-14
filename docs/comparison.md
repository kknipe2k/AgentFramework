# Comparison — orchestration languages and visual builders

Where this runtime sits relative to the tools people most often ask us to
compare against: the code/DSL orchestration layer (**LangGraph**,
**Barnum**) and the visual-builder layer (**n8n**, **Flowise**,
**LangFlow**, **Dify**, **Copilot Studio**).

This doc consolidates the prior-art and positioning fragments that were
scattered across the repo — `docs/README-v0.1.md` ("Prior art"),
`docs/proposals/runtime-capabilities-roadmap.md` §1.5 (swarm positioning),
`docs/adr/0022-canonical-framework-representation.md` (the rejected n8n
single-file model), `docs/adr/0029-gap-resolve-and-resume.md` (LangGraph
`interrupt()` as design prior-art), and `docs/workbench-delivery-plan.md`
(the builder-canvas lineage study) — and discharges the long-form
comparison that `docs/launch/stage-1-first-signal.md` deliberately defers
("Different scope… Curious what you'd want compared").

## Status & honesty convention (read first)

Per `CLAUDE.md` §4.11 (grounded claims), every runtime capability below
carries a status tag:

| Tag | Meaning |
|---|---|
| **Built** | Merged in a shipped milestone (M01–M08.9). Milestone cited. |
| **Partial** | Merged, but with a known deferral/waiver or only a sub-path shipped. |
| **Specified** | Designed in spec/ADR/roadmap; **not yet built** (the M09–M13 spine, or v1.0+). |

These tags are **milestone-level**, derived from `git log`, `CHANGELOG.md`,
and the retrospective summaries as of **2026-06-14** — they are **not**
per-feature verifications against the running, assembled app. Authoritative
live state lives in `CHANGELOG.md` `[Unreleased]`, `docs/MVP-v0.1.md`,
`docs/gap-analysis.md`, and `git log` — read those before trusting any tag
here. As of this writing the runtime is **mid-build**: M01–M08.9 are
merged; **M09 (author one agent from scratch and run it for real) is in
active development**; M10–M13 (HITL-steered runs, sub-agents, the
hooks/rails + shell-exec verify loop, ship) are specified per ADR-0031 /
ADR-0032 and **not yet built**.

Competitor facts are from public sources as of **mid-2026** (linked at the
bottom). Barnum's details are drawn from its published site summaries — its
homepage returned HTTP 403 to our fetcher, so those specifics are
second-hand and should be confirmed against the source.

## 1. The crux: three different *kinds* of thing

The comparison only makes sense once you see that these tools aren't the
same category:

- **Code / DSL orchestration — you *program* the orchestration.**
  LangGraph is a **library** (Python/JS); Barnum is a **typed DSL**
  (TypeScript handlers composed with primitives). The "language" is code
  you write and run.
- **Visual builders — you *wire* a canvas.** n8n, Flowise, LangFlow, Dify,
  and Copilot Studio give you a drag-and-drop graph of nodes; the canvas
  generates or drives a runnable flow (usually served headless behind an
  API or chat endpoint).
- **This runtime — a *governed declarative runtime* that runs an
  orchestration language expressed as data.** Per `agent-runtime-spec.md`
  §0, it is explicitly **not a framework and not a library** — "the way the
  JVM is to Java, or Deno is to TypeScript." Orchestration is **declarative
  JSON** (`framework.v1.json` + `skill.md` / `agent.md` / `tool.md`
  artifacts, JSONLogic for conditionals), and a runtime owns the parts the
  other tools leave to the author: capability enforcement, crash recovery,
  gap handling, tiered human gates, audit. Its visual surface is primarily
  a **live execution graph** (Built, M3) plus a **build-time Builder
  canvas** (Built, M8).

So the one-line trichotomy: **LangGraph/Barnum are things you *program*;
n8n/Flowise/LangFlow/Dify/Copilot are canvases you *wire*; this is a
runtime that *runs a declarative orchestration language under
governance*** — with both a live graph and a builder. The axis that
separates it from everything else is **code vs. canvas vs. data**, and the
fact that safety/recovery/capability are **runtime properties no author can
skip**, not library features each author must remember to wire.

## 2. At a glance

| Tool | Category | You author in | Execution model | Visual surface | Safety/governance posture | Hosting / provider |
|---|---|---|---|---|---|---|
| **This runtime** | Governed declarative runtime | Declarative JSON (framework + skill/agent/tool artifacts) | Runtime executes the framework; drone owns survival + snapshot recovery; fresh context per task | **Live execution graph** (Built M3) + Builder canvas (Built M8) | Capability sandbox, tiered human gate, rails, `dont_touch`, budget, gap-suspend — **runtime-enforced** (mix of Built/Partial/Specified) | Local desktop (Tauri); Anthropic only (v0.1) |
| **LangGraph** | Code library | Python / JS (StateGraph API) | Cyclic graph; **shared mutable state** object; checkpointer for persistence | Code-first; LangGraph Studio for viz | Not a core concern; `interrupt()` (HITL) + checkpointing are first-class | Runs anywhere; provider-agnostic; LangGraph Platform (hosted) |
| **Barnum** | Typed DSL | TypeScript handlers + `.then`/`.map`/`.iterate`/`loop`/`branch`/`tryCatch` | **State machine**; each handler in its own isolated subprocess; Rust runtime drives it | Code-first | Subprocess isolation for predictability/security; not a governance product | Self-run; LLM-agnostic |
| **n8n** | Workflow automation (+ agents) | Visual canvas + JS/Python code nodes | Node graph; AI is one step in a larger automation | Mature node canvas | Enterprise RBAC/self-host; not agent-capability sandboxing | Self-host / cloud (fair-code license); any model |
| **Flowise** | Visual agent builder | Drag-drop canvas (LangChain/LlamaIndex under the hood) | Agent/chain flow; HITL a strength | Canvas | App-level; no capability sandbox | Self-host / cloud; any model |
| **LangFlow** | Visual agent builder | Drag-drop canvas (Python/LangChain) | Agent/chain flow; exports to Python/API | Canvas | App-level | Self-host / cloud (DataStax); any model |
| **Dify** | All-in-one LLMOps platform | Visual workflow/chatflow + prompt/RAG IDE | Workflow + agent + RAG + observability + publish | Canvas + app console | Platform-level (workspaces, logs); not per-agent capability sandbox | Self-host / cloud; many models |
| **Copilot Studio** | Enterprise low-code agent SaaS | Visual designer (topics/actions/connectors) | Unified canvas: deterministic actions + branching + AI steps; multi-agent in-designer; A2A | Canvas | **Enterprise governance** via Power Platform admin (security, DLP, audit) | Microsoft cloud; multi-model (Project Polaris) |

## 3. Orchestration languages — LangGraph and Barnum

**LangGraph** is the code-first baseline. You assemble a `StateGraph` of
nodes and edges in Python/JS, threading a **shared mutable state** object
(typically a `TypedDict` with reducers) through the graph; conditional
edges route control flow; checkpointers give persistence; `interrupt()`
gives human-in-the-loop; subgraphs compose. It is unopinionated, mature,
provider-agnostic, and maximally flexible for arbitrary cyclic topologies.
Its model is the **opposite** of this runtime's on two axes: state is a
central shared object every node reads/writes (vs. fresh, scoped context
per task here), and orchestration logic lives in *your* code (vs. in the
runtime here). Notably, ADR-0029 cites LangGraph's `interrupt()` +
durable-execution model as **prior art** for this runtime's
gap-resolve-and-resume design — so on HITL-as-a-suspend-primitive the two
converge by intent.

**Barnum** bills itself as "the programming language for orchestrating
agents." You define an asynchronous workflow that is **effectively a state
machine**, composed from type-safe primitives — `.then()` (sequential),
`.iterate()` / `.map()` (fan-out), plus `loop`, `branch`, `tryCatch`.
Handlers are built-in primitives or TypeScript async functions, and **each
handler runs in its own isolated subprocess**; a **Rust runtime** dispatches
handlers, collects results, and advances the machine. No handler sees
another's context — "the agent performing a refactor never sees the full
workflow, just its input." The explicit design split is **agents for
judgment, deterministic code for the rest.**

**The non-obvious finding: this runtime is architecturally closer to Barnum
than to LangGraph.** Both are **Rust runtimes driving a state machine with
per-unit subprocess/context isolation and a deliberate
"deterministic-code-does-the-rote-work / agents-do-the-judgment" split** —
the same reaction against the single-bloated-context agent loop. The
divergence is what each wraps that engine in:

- **Barnum keeps you in code** (typed TS composition) — more *expressive*
  for arbitrary control flow, aimed squarely at developers who want precise
  async orchestration.
- **This keeps you in data** (declarative framework JSON) — less expressive
  for exotic topologies, but the orchestration *guarantees* (capability
  enforcement, gap-suspend, recovery, audit, tiered HITL) become **runtime
  properties no author can skip or get wrong**, and the authoring surface
  extends past developers to the Builder canvas.

LangGraph sits apart from both: shared-state, single-process, code-first,
and unopinionated about isolation and governance.

## 4. Visual builders — n8n, Flowise, LangFlow, Dify, Copilot Studio

The 2026 public comparisons converge on a useful taxonomy within this
cluster, and on one decision rule:

- **n8n** — a **workflow-automation** platform (400+ integrations) that
  added agent nodes. Wins when AI is *one step* in a larger ops automation
  and you want deep connectors. Fair-code license; self-host or cloud.
- **Flowise** — a **visual agent builder** on LangChain/LlamaIndex.
  Strong for LangChain-style prototyping on a canvas; cited as the pick
  when **human-in-the-loop is a hard requirement**.
- **LangFlow** — a **visual agent builder**, Python/LangChain-native
  (DataStax). The pick for **Python teams** prioritizing community and
  ecosystem; exports to Python/API.
- **Dify** — an **all-in-one LLMOps platform**: visual workflow/chatflow +
  RAG + prompt IDE + observability + app publishing. Cited as the fastest
  path to a **shipped app** because the knowledge base, debugging, and
  publishing are built in.
- **Copilot Studio** — Microsoft's **enterprise low-code agent SaaS**. In
  the 2026 wave, agent nodes embed directly in workflow graphs (one unified
  visual canvas of deterministic actions + branching + AI steps),
  multi-agent orchestration is first-class in the designer (no SDK),
  Studio-built and Agent-Framework-SDK-built agents are **A2A
  wire-compatible**, computer-using agents drive app UIs directly, and
  governance/security/DLP run through Power Platform admin at enterprise
  scale.

**The decision rule the comparisons land on:** *if your process is mostly
defined and deterministic, with agents handling specific reasoning inside
fixed steps, a **workflow** platform fits; if you need agents that reason
dynamically, delegate by context, and coordinate as a team, you need a tool
built around that model from the start.* This runtime is firmly in the
second camp (agent-native, dynamic spawn) — but with two differences from
every tool above:

1. **The visual model is inverted.** The builders give you a **static
   design canvas** that you then run (often headless). This runtime's
   primary visual surface is a **live execution graph** (Built, M3) — it
   renders agent/skill/tool spawning in real time and **suspends visibly on
   a capability gap** — with the build-time **Builder canvas** (Built, M8)
   as a second surface. You watch the run, not just the design.
2. **Safety is in the runtime, not the canvas.** A flow exported from n8n /
   Flowise / LangFlow / Dify enforces whatever the author wired. Here,
   capability narrowing, tiered human gates, rails, `dont_touch`, and budget
   caps are **runtime-enforced regardless of what the framework JSON says**
   — the author cannot widen them by forgetting to add a guard node.

For the record, the Builder's design explicitly studied this cluster
(`docs/workbench-delivery-plan.md` — n8n / Flowise / LangFlow / Dify /
Copilot Studio palette + JSON + integration models), and ADR-0022
**rejected n8n's single-file-per-workflow model** in favor of a canonical
multi-artifact framework representation.

## 5. What this runtime does that the others don't (first-class, runtime-enforced)

Each item below is a runtime property, not a library feature an author
opts into — tagged for honesty per §4.11.

- **Live execution graph** of agent/skill/tool spawning, rendered in real
  time. **Built (M3).** No tool above renders a *live run* as its primary
  surface; their canvases are design-time.
- **Gap detection → clean suspend → Builder → resume.** When an agent needs
  a capability it lacks, the runtime raises it as a first-class event.
  *Detection + block* is **Built (M5)** (`request_capability` fires a
  GapNode; `capability_violation` blocks). The full *resolve→resume* loop
  (suspend, fix in Builder, resume without re-executing) is **Specified
  (M10 / ADR-0029)** — not yet built.
- **Capability sandbox + agent→agent narrowing + tiered human gate** (the
  §8 L1–L5 model). **Partial** — capability declaration/enforcement and the
  Novice/Promoted tier landed at M05 and the Tester now runs at the tracked
  tier (M08.8/M08.9), but L1/L2a SDK-wire and a tier gate were deferred
  under ADR-0009 / ADR-0016, and OS-level (L2b) sandboxing is v1.0. A child
  agent cannot exceed its parent's grants by design.
- **Drone-owned crash survival.** A dedicated drone process owns SQLite
  snapshots and recovery; **resume rebuilds from snapshots rather than
  re-executing.** **Built (M01).**
- **Rails / `dont_touch` / budget / failure-escalation** as declarative
  framework primitives. Foundations landed at M4; the full hooks/rails
  **firing engine + controlled shell exec** is **Specified (M12)**.
- **Orchestration-as-data.** The framework is declarative JSON validated
  against `schemas/framework.v1.json`, with types generated from the
  schema (not hand-written). **Built**; canonical representation per
  ADR-0022.
- **Decision trace / VDR projection + a native signal taxonomy.** Built
  (M3/M8) — an append-only event stream projected into a verifiable
  decision record.
- **Local-first, zero-telemetry, keys-in-OS-keychain.** **Built.** No
  analytics, no crash reporter (spec §13); prompts go to Anthropic on the
  user's own key; data stays local.
- **One authoring surface for novice and expert.** The Builder canvas is
  **Built (M8)**; the full *author-from-scratch-and-run* path is **In
  progress (M09).**

## 6. Where the others are clearly stronger (honest)

- **LangGraph** — mature, huge ecosystem, provider-agnostic from day one,
  arbitrary cyclic control flow, streaming, time-travel, hosted platform.
  For maximum flexibility *today*, it wins.
- **Barnum** — a genuine typed programming model; `.then`/`.map`/`tryCatch`
  composition is more powerful than declarative JSON + JSONLogic for complex
  control flow, while keeping the isolation benefits.
- **n8n** — 400+ integrations and ops-automation breadth this runtime has no
  intention of matching (it delegates integrations to MCP).
- **Flowise / LangFlow** — fast LangChain-style prototyping on a canvas,
  large component libraries, instant deploy-as-API.
- **Dify** — all-in-one LLMOps (RAG + agents + observability + publishing);
  the fastest path from idea to a shipped, monitored app.
- **Copilot Studio** — enterprise governance at scale, M365 / Power
  Platform integration, a connector ecosystem, in-designer multi-agent +
  A2A wire-compatibility, and computer-use agents — plus genuine
  **non-developer** accessibility at enterprise grade.
- **All of them ship today.** This runtime is **mid-build** — v0.1 is not
  yet released, and the full author-and-run + verify loop is M09–M13. The
  comparison above is largely "this runtime *as specified*" vs. "the others
  *as they exist*." Keep that asymmetry in mind.

## 7. One-line positioning

LangGraph is a **flexible code library**; Barnum is a **typed orchestration
language**; n8n / Flowise / LangFlow are **visual flow / agent builders**;
Dify is an **all-in-one LLMOps app platform**; Copilot Studio is an
**enterprise low-code agent SaaS**; and this is a **local-first, governed,
declarative runtime with a live execution graph** — trading raw
expressiveness and ecosystem for runtime-enforced capability / safety /
recovery guarantees and a novice→expert visual surface. That trade is
exactly the bet `agent-runtime-spec.md` §0 makes.

## Sources

Public sources as of mid-2026 (competitor specifics are theirs, not ours):

- Barnum — <https://barnum-circus.github.io/> (homepage 403'd our fetcher; details from published summaries)
- LangGraph — <https://langchain-ai.github.io/langgraph/>
- n8n — <https://n8n.io/>
- Flowise — <https://flowiseai.com/>
- LangFlow — <https://www.langflow.org/>
- Dify — <https://dify.ai/>
- Copilot Studio — <https://learn.microsoft.com/en-us/microsoft-copilot-studio/> and the [2026 release wave 1 overview](https://learn.microsoft.com/en-us/power-platform/release-plan/2026wave1/microsoft-copilot-studio/)
- Visual-builder comparisons — [MadAppGang (Flowise/LangFlow/n8n/Sim, 2026)](https://madappgang.com/blog/open-source-visual-agent-builders-compared-flowise-vs-langflow-vs-n8n-vs-sim-studio-in-2026/), [Hugging Face (n8n/Flowise/LangFlow, enterprises)](https://huggingface.co/blog/daya-shankar/n8n-vs-flowise-vs-langflow-enterprises), [ToolHalla (Dify/Flowise/LangFlow, 2026)](https://toolhalla.ai/blog/dify-vs-flowise-vs-langflow-2026)

Internal positioning this doc consolidates: `agent-runtime-spec.md` §0 /
§0a / §0b; `docs/adr/0022-canonical-framework-representation.md`;
`docs/adr/0029-gap-resolve-and-resume.md`;
`docs/proposals/runtime-capabilities-roadmap.md` §1.5;
`docs/workbench-delivery-plan.md`; `docs/README-v0.1.md` ("Prior art");
`docs/launch/stage-1-first-signal.md`.
