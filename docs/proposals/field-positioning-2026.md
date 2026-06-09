# Field Positioning — Agent Runtime vs. the 2026 Agentic-Framework Field

> **Status:** Positioning note + two back-pocket scope-candidates (cross-session memory; runtime clustering). Drafted against the JetBrains PyCharm "Top Agentic Frameworks for Building Applications 2026" roundup. Not a commitment to ship; a positioning artifact + forward-look. None of the scope-candidates land before M11 per §0d.
> **Why this exists:** the roundup defines an agentic framework as **Orchestration + Tools + Memory**, facilitating **multi-agent coordination + HITL + observability/control/reproducibility**, and closes with "the key decision is no longer whether to use agents, but how much control, autonomy, and governance your systems require." That conclusion is this runtime's thesis. This note (1) writes the runtime's entry in the article's own voice + table, (2) answers whether a `pip install` distribution is reachable from the current architecture, (3) pencils in persistent cross-session memory and runtime clustering.

---

## 1. The runtime, written as an entry in the article

### Agent Runtime

- **Core design:** Graph-based + planner orchestration, safety-first.
- **Philosophy:** Governed, observable, sandboxed execution — the same way for novices and experts.

Agent Runtime is not a library you import into an application; it is a local desktop runtime (Tauri + Rust) that executes agentic processes under OS-level isolation while drawing a live graph of everything that happens. Where most frameworks start from agent autonomy and add controls afterward, Agent Runtime starts from the controls: every tool declares its capabilities, every agent runs inside a seccomp / landlock / Job Objects sandbox, and the plan state machine pauses cleanly the moment an agent needs something it has not been granted (gap detection). Execution is reconstructable — append-only, SHA-chained snapshots let a session be replayed without re-running tools.

It deliberately trades emergent autonomy for trust, auditability, and reproducibility. The runtime executes what exists; it does not modify itself mid-run. That makes it predictable and debuggable at the cost of the "agent improves itself on the fly" behavior other frameworks lean into — improvement is an offline eval-and-reship loop, not a mid-run mutation.

#### Strengths
- OS-level capability sandboxing — least privilege, deny by default (most of the field has nothing equivalent).
- Live execution graph + signal/VDR observability + full audit trail.
- Strong HITL: plan → verify → approve → resume, with budget gates.
- Reproducibility via replayable, SHA-chained snapshots.
- Clean gap-detection / suspend instead of silent failure or capability escalation.

#### Limitations
- Single-session today; cross-session memory is light (see §3).
- Anthropic-only, Windows-first in v0.1 (deliberate scope locks, not architecture).
- Not embeddable via `pip` / package import today — it is a runtime + workbench, not a library (see §2 for whether that can change).
- Constrained by design: less emergent / open-ended behavior than role- or chain-based frameworks.

#### Best applications
- Building and running agentic processes where control, audit, and safety are non-negotiable.
- Novice-to-expert authoring of agent frameworks in one workbench.
- Regulated / sensitive contexts where "what ran, why, and under what permissions" must be answerable.

### Added to the article's table

| Framework | Orchestration model | Multi-agent support | Memory capabilities | HITL support | Best used for |
|--|--|--|--|--|--|
| **Agent Runtime** | Graph-based + planner | Hierarchical (concurrent: roadmap) | Light–Moderate today (cross-session: roadmap) | Strong | Governed, observable, sandboxed building *and running* of agentic processes |

This places Agent Runtime in the article's **high-control cluster — LangGraph, OpenAI Agents SDK, Semantic Kernel** — and its nearest analog by philosophy is **Semantic Kernel** (governance, safety, observability, human oversight; trades emergence for trust). The defining difference: Semantic Kernel is a library integrated into enterprise systems; Agent Runtime is a sandboxed product + workbench with an OS-level safety boundary none of the library frameworks carry.

---

## 2. Can we get to `pip install` later — or is it an architecture change we're too far along to make?

**Short answer: it is reachable, and you are not too far along. It is additive, not a rewrite.** The reason is structural and already in place.

### Why the architecture is already friendly to it

The orchestration + safety engine lives in the **Rust workspace crates** (`runtime-core`, `runtime-main`, `runtime-drone`, `runtime-sandbox`, `runtime-mcp`), not in the Tauri/JS shell. The shell is one consumer of that engine. Three existing decisions make a second consumer (a Python package) feasible without re-architecting:

1. **Path-agnostic persistence (the established archetype).** Persistence modules take `path: &Path`; the Tauri shell resolves the directory and passes it in. A Python binding would resolve and pass a path the same way — no Tauri dependency leaks into the core.
2. **Pluggable `LLMProvider` trait.** The model layer is already an interface, not hardwired to the shell.
3. **Multi-process by design.** Main ↔ drone ↔ sandbox already talk over framed JSON IPC (Unix socket / Windows named pipe). The engine is not trapped inside the webview event loop.
4. **The headless seam is already named.** The M7-deferred headless CLI is exactly the entry point a package or server would reuse.

What would make a `pip install` *hard* is the inverse of all of the above — orchestration logic stranded in the React/JS layer, or coupled to the webview. That is not your situation.

### Two viable shapes (this is the real decision, and it is product-scope, not architecture)

- **(A) Python client → runtime server.** Lift the runtime's existing IPC to a local server; ship a thin Python client (`pip install agent-runtime-client`) that drives it. Best fit for your multi-process sandbox model — the sandbox keeps spawning isolated subprocesses; Python never holds unsafe state. Lowest risk; reuses the IPC you already have.
- **(B) Python-native bindings via PyO3 / maturin.** Compile the Rust core into a wheel (the path `pydantic-core`, `polars`, `ruff`, `tokenizers` all take). In-process orchestration callable from Python; the sandbox still spawns subprocesses. More "frameworky" ergonomics; heavier packaging (manylinux wheels; landlock/seccomp need Linux kernel features, Job Objects need Windows — so the sandbox story complicates the wheel matrix).

### The honest caveats (all scope-locks, not arch dead-ends)

- **Single-session lock** must be lifted for any server/embedded use — that is roadmap work (see §3/§4), not a redesign.
- **OS-specific sandbox** is harder to ship as a portable package than as a desktop app you control. Distributable, but the wheel/platform matrix is real work.
- **Entering the library market = entering LangGraph/Semantic Kernel's competition** with its own maintenance surface. That is a *business* decision; the engineering is feasible.

**Verdict:** keep building. The core/shell split you already enforce is precisely what keeps the `pip`-install door open. Recommend shape **(A)** when/if you choose to walk through it — it reuses the existing IPC and preserves the subprocess sandbox boundary. Treat it as a post-v1.0 product decision, not an architecture commitment you must make now.

---

## 3. Pencilled in: persistent, retrievable memory across sessions

**Scope (penciled, v1.0 candidate).** Memory that survives session end and is retrievable in later sessions — closing the "Memory" pillar the article names as one of three core capabilities (and the runtime's current weak pillar).

- **Scoping: per built runtime (per framework) — confirmed as the right model.** Memory belongs to a specific framework/runtime the user has built, stored in that runtime's own data directory. Not global, not cross-framework by default. This matches the path-agnostic persistence archetype: a new memory module takes `&Path`, the shell resolves `app_local_data_dir().join("<framework>/memory.db")`.
- **Store:** SQLite (consistent with `persistence-architecture.md`); semantic retrieval via `sqlite-vec` (already floated in `runtime-capabilities-roadmap.md` §2.4) when keyword recall is insufficient. Keeps data sovereign — memory never leaves the machine, no phone-home (spec §13).
- **Respects the locks:** memory is *data the agent reads/writes*, not a change to the agent's definition — so it does **not** violate "the runtime does not modify itself mid-run." Memory reads/writes are events on the live graph and in the audit trail (observability preserved).
- **Relationship to the single-session lock:** persistence *across* sessions ≠ concurrent sessions. This is the cheap half of lifting the lock and can land independently of concurrent multi-agent work.
- **Open decision points:** retention/expiry policy; whether memory is per-agent or per-framework-shared; write-gating (does the agent write memory freely, or through a declared capability?). Recommend: declared capability to write memory, so the sandbox/audit story stays intact.

---

## 4. Pencilled in: clustering runtimes under a super-orchestrator

**Scope (penciled, v1.5–v2.0 candidate for same-trust-domain; v3.0 for cross-org).** A parent "super-orchestrator" coordinating multiple Agent Runtime instances — hierarchical multi-runtime, the agent→agent model lifted one level to runtime→runtime.

- **What's already there:** the runtime is multi-agent-first with capability narrowing on Agent→Agent edges (`runtime-capabilities-roadmap.md` §1). A super-orchestrator is the same delegation-with-narrowing pattern applied across a runtime boundary instead of an agent boundary.
- **Wire protocol — adopt A2A, don't invent one.** The field standardized on Google's **A2A** (Agent2Agent) for cross-framework/cross-runtime delegation. Each runtime exposes an A2A endpoint; the super-orchestrator delegates tasks and receives results over it. This is strictly better than a bespoke peer-to-peer IPC topology (roadmap §1.3) because it makes clustered runtimes interoperable with the broader ecosystem, not just with each other.
- **What's needed (the real work):** inter-runtime **identity + trust model** + **capability federation** — i.e., the super-orchestrator's grant to a child runtime narrows what that runtime (and transitively its agents) may do. Same-owner / single-trust-domain clustering is the near-term, tractable version; cross-org federation is the far-term, hard version (roadmap §1.4 keeps that at v3.0).
- **Why the runtime is well-positioned:** capability containment, audit-trail integrity, and plan reconciliation are solved at the architecture level for agents already. Clustering re-uses those guarantees one level up rather than inventing safety per-feature — the same argument roadmap §1.5 makes for swarms.
- **Open decision points:** A2A version pinning; whether the super-orchestrator is itself an Agent Runtime instance (recommended — dogfoods the model) or a thinner coordinator; how cross-runtime capability grants are expressed in the schema (ADR + schema change per §11/§14).

---

## 5. What to do with this note

- **§1 (article entry):** reusable for launch / positioning copy. Closest competitor framing = Semantic Kernel.
- **§2 (pip distribution):** decision is *product-scope*, not architecture; door is open; revisit post-v1.0. No action needed now beyond not closing the door (keep the core/shell split clean; keep the headless seam).
- **§3 + §4:** fold into `runtime-capabilities-roadmap.md` (memory → §2.4 graduation; clustering → §1.3/§1.4 with the A2A alignment) and, if accepted, promote each to a numbered proposal with decision points. Cross-session memory pairs naturally with proposal `0002` (eval-as-surface) as the runtime's answer to the article's "agents improve over time" expectation — offline, audited, not mid-run.
