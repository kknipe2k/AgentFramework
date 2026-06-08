# Proposal 0003 — Persistent, Retrievable Cross-Session Memory (Per-Runtime)

> **Status:** Proposed (back-pocket → v1.0 scope-candidate). Graduates the "lightweight built-in vector index via SQLite (sqlite-vec)" line in [`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §2.4 from a conditional future addition into a scoped memory surface.
> **Trigger:** JetBrains PyCharm "Top Agentic Frameworks for Building Applications 2026" defines an agentic framework as **Orchestration + Tools + Memory** — three co-equal pillars. A pillar-by-pillar grade of this runtime (see [`field-positioning-2026.md`](./field-positioning-2026.md) §1) scores Orchestration ✅ and Tools ✅, but **Memory 🟡** — the runtime's single weakest pillar against the field's own definition. Maintainer follow-up (2026-06-08) asked to pencil in cross-session memory, scoped per built runtime.
> **Scope:** Post-v0.1 only. None of this lands before M11. Per [§0d Release Scope Matrix](../../agent-runtime-spec.md) v0.1 additions require equivalent removals — this does not qualify.
> **Author:** kknipe2k (back-pocket exploration; Claude-drafted from a directed review).
> **Tags:** architecture, memory, persistence, retrieval, scope-candidate, sqlite-vec

---

## 1. Context

The 2026 field treats memory as a co-equal core capability, not an add-on. Every "Strong memory" framework in the JetBrains table (LangGraph, LlamaIndex, Haystack, Phidata) makes cross-step and cross-session retention central. This runtime has rich **within-session** state (drone snapshots, append-only SHA-chained recovery, the live graph) but **single-session lock** (§0d) means none of it is retrievable in a *later* session. An agent that ran yesterday remembers nothing today.

This is the cheap, high-value half of relaxing the single-session lock: **persistence across sessions is not the same problem as concurrent sessions**. It can land independently of any concurrent-multi-agent work (`runtime-capabilities-roadmap.md` §1.2).

The field also expects agents to "improve their behavior over time" (the roundup's PRAR "Reflect" pillar). This runtime deliberately forbids mid-run self-modification (`runtime-capabilities-roadmap.md` §3.4). Cross-session memory is the *safe* expression of "remembers and improves" — the agent accumulates retrievable data across runs without ever rewriting its own definition.

---

## 2. The gap, precisely

Two objects; the runtime has only the first:

1. **Within-session state (have it, rigorously).** Snapshots, recovery, the live graph — all scoped to one session, gone when the session closes for retrieval purposes.
2. **Cross-session memory (don't have it).** A store an agent can write to during a run and *retrieve from* in a future, unrelated session — keyed to the framework it belongs to.

**Scoping is the load-bearing decision, and the answer is: per built runtime (per framework).** Memory belongs to a specific framework the user has built, lives in that framework's own data directory, and is not shared across frameworks by default. This mirrors the existing path-agnostic persistence archetype (`docs/style.md`; M05.D `tier::persistence`): the memory module takes `path: &Path`; the Tauri shell resolves `app_local_data_dir().join("<framework>/memory.db")` and passes it in. No Tauri dep leaks into the core; unit tests use `tempfile` paths.

---

## 3. Why the runtime is well-positioned

- **Persistence archetype already exists.** A new memory module is the same shape as `tier::persistence` and `audit::file_path` — `&Path` in, SQLite out, shell resolves the directory. No new architecture.
- **SQLite is already the operational store** (`docs/persistence-architecture.md`); adding a memory table/db is incremental.
- **`sqlite-vec` was already floated** (`runtime-capabilities-roadmap.md` §2.4) for a lightweight built-in vector index — semantic retrieval reuses that, no external vector DB required, data stays local.
- **Observability is preserved for free.** Memory reads/writes are events on the live graph and in the audit trail — the runtime's existing instrumentation covers them.

---

## 4. The surface (sketch, not commitment)

- **Store.** SQLite in the framework's data dir. Keyword/recency retrieval first; **`sqlite-vec`** semantic retrieval when keyword recall is insufficient. Local-only; no phone-home (spec §13 / CLAUDE.md §4.4).
- **Write path is a declared capability.** The agent writes memory through a capability it must be granted (e.g. `memory:write`), so the sandbox + audit story stays intact — memory writes are scoped and observable like any other effect, not an ambient side channel.
- **Read path** surfaces retrieved memory into the agent's context at Perceive-time (the roundup's PRAR "Perceive" step), via the existing context-assembly seam.
- **Granularity.** Per-framework store; within it, optionally per-agent namespaces (decision point below).
- **Schema.** If memory entries become a first-class artifact, candidate `schemas/memory.v1.json` (typify-generated per §14; ADR per §11). If memory is purely operational (like audit/snapshots), it stays an internal store with no public schema. Recommend operational-first; promote to a schema only if users need to author/inspect memory as content.

---

## 5. Why this is the safe answer to "agents improve over time"

The field expects Reflect-and-improve. This runtime answers it on **two** safe axes, both offline relative to a single run:

- **Cross-session memory (this proposal)** — the agent accumulates retrievable experience across runs. Data, not self-modification.
- **Eval-as-surface ([`0002`](./0002-evaluation-as-product-surface.md))** — the framework's *behavior* is measured and regressions caught across model releases, driving an offline regenerate-and-reship loop.

Together they deliver "remembers and gets better" without ever touching the audit-integrity / no-self-modification lock (`runtime-capabilities-roadmap.md` §3.4). This proposal and `0002` are complementary halves of the same answer.

---

## 6. Out-of-scope locks (do NOT smuggle into v0.1, or let this drift into)

- **No concurrent sessions.** This is cross-session *persistence*, not concurrent multi-agent. The single-active-session lock (§0d) is untouched.
- **No self-modification.** Memory is data the agent reads/writes; it never rewrites the agent's definition, prompt, or capability rules.
- **No cross-framework memory by default.** Memory is per-built-runtime. A shared/global memory tier is a separate, later question.
- **No always-on telemetry / phone-home.** Memory is local to the user's data dir like any other run output.
- **No external vector DB requirement.** `sqlite-vec` keeps it self-contained; pointing at an external vector store stays an MCP-delegated option (`runtime-capabilities-roadmap.md` §2).

---

## 7. Decision points before formal scoping

1. **v1.0 commitment vs. back-pocket.** Recommendation: scope as an **early v1.0 milestone** — lowest-cost pillar-closer, independent of concurrency work.
2. **Per-agent vs. per-framework-shared memory** within a framework's store. Recommendation: per-framework store with optional per-agent namespaces.
3. **Operational store vs. first-class schema** (`schemas/memory.v1.json`). Recommendation: operational-first; promote only if users must author/inspect memory.
4. **Retention / expiry policy.** Unbounded growth vs. TTL vs. size cap vs. user-managed. Recommendation: size cap + user-managed clear, decided at scoping.
5. **Retrieval default.** Keyword/recency only, or `sqlite-vec` semantic from day one. Recommendation: ship keyword/recency first; add `sqlite-vec` when a real recall gap is demonstrated.

---

## 8. References

### Trigger
- JetBrains PyCharm — "Top Agentic Frameworks for Building Applications 2026" (Orchestration + Tools + Memory as the three core pillars).
- [`field-positioning-2026.md`](./field-positioning-2026.md) §1 (pillar grade), §3 (penciled memory scope).

### Internal cross-references
- [`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §2.4 (`sqlite-vec` quick-start RAG), §1.2 (single-session lock), §3.4 (no self-modification).
- [`0002-evaluation-as-product-surface.md`](./0002-evaluation-as-product-surface.md) — the complementary "improves over time" axis.
- `docs/persistence-architecture.md`; `docs/style.md` (path-agnostic persistence archetype, M05.D `tier::persistence`).
- `agent-runtime-spec.md` §0d (scope matrix), §13 (privacy/no telemetry), §14 (schema-as-source-of-truth).

---

## 9. Next steps

1. Maintainer review.
2. If accepted: graduate `runtime-capabilities-roadmap.md` §2.4 to a committed line; add to the post-M11 v1.0 scope discussion.
3. If accepted as a v1.0 milestone: draft the memory module against the `tier::persistence` archetype; decide schema-vs-operational; ADR if a schema is introduced.
4. If rejected: archive with a one-line rationale.
