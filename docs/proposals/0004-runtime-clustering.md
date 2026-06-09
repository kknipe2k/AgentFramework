# Proposal 0004 — Clustering Runtimes Under a Super-Orchestrator (A2A-Based)

> **Status:** Proposed (back-pocket). Same-trust-domain clustering → v1.5–v2.0 scope-candidate; cross-org federation → v3.0+ (stays where [`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §1.4 puts it). Graduates the peer-to-peer / cross-runtime swarm lines (§1.3 / §1.4) toward a concrete wire-protocol commitment (A2A) rather than a bespoke IPC topology.
> **Trigger:** JetBrains PyCharm "Top Agentic Frameworks for Building Applications 2026" places this runtime in the high-control cluster (LangGraph / OpenAI Agents SDK / Semantic Kernel) and frames the 2026 decision as "how much control, autonomy, and governance your systems require." The broader field standardized on Google's **A2A** (Agent2Agent) for cross-framework/cross-runtime delegation. Maintainer follow-up (2026-06-08) asked to pencil in "ability to cluster runtimes under super-orchestrators."
> **Scope:** Post-v0.1 only. None of this lands before M11. Per [§0d Release Scope Matrix](../../agent-runtime-spec.md) v0.1 additions require equivalent removals — this does not qualify.
> **Author:** kknipe2k (back-pocket exploration; Claude-drafted from a directed review).
> **Tags:** architecture, multi-runtime, orchestration, a2a, capability-federation, scope-candidate

---

## 1. Context

A "super-orchestrator" coordinating multiple Agent Runtime instances is the runtime's existing **agent→agent** delegation model lifted one level to **runtime→runtime**. The runtime is multi-agent-first with capability narrowing on Agent→Agent edges (`runtime-capabilities-roadmap.md` §1; ARIA's 8-agent hierarchy is the proof). Clustering applies the same delegation-with-narrowing pattern across a runtime boundary instead of an agent boundary.

`runtime-capabilities-roadmap.md` already anticipates this on two horizons:
- **Peer-to-peer swarm (§1.3, v1.5+)** — agents/runtimes message each other, needing a new IPC topology.
- **Cross-runtime swarm (§1.4, v3.0+)** — your stack talks to someone else's, needing an inter-runtime trust model.

This proposal sharpens both with one decision: **don't invent the wire protocol — adopt A2A.**

---

## 2. The gap, precisely

Current agent-to-agent comms is **one-shot spawn**: parent calls child with input, child returns output, no ongoing channel (`runtime-capabilities-roadmap.md` §1.3). There is no notion of a runtime exposing itself as a delegatable endpoint, and no inter-runtime identity or capability-federation model.

Two distinct objects, neither present:

1. **Same-trust-domain clustering (near-term, tractable).** One owner runs several Agent Runtime instances; a super-orchestrator delegates tasks across them. Single trust boundary, single owner — the hard cross-org problems (identity federation, cross-org capability trust) don't arise yet.
2. **Cross-org federation (far-term, hard).** Different owners' runtimes interoperate. Needs inter-runtime identity, message routing across machine/org boundaries, cross-org capability federation — `runtime-capabilities-roadmap.md` §1.4's v3.0 territory. Out of scope for this proposal beyond naming it.

---

## 3. Why adopt A2A instead of a bespoke topology

`runtime-capabilities-roadmap.md` §1.3 framed peer-to-peer as "needs new IPC topology" — implying a bespoke broadcast/pub-sub layer. The field has since standardized on **A2A** as the framework-agnostic protocol for one agent to delegate to another regardless of framework. Adopting it instead of inventing IPC means:

- **Ecosystem interoperability for free.** A clustered Agent Runtime can delegate to — and be delegated to by — agents built on LangGraph, ADK, CrewAI, LlamaIndex, Semantic Kernel, AutoGen. Bespoke IPC only ever talks to other Agent Runtimes.
- **No protocol-maintenance burden.** A2A is externally specified and versioned; the runtime implements an endpoint rather than owning a protocol.
- **Consistency with the existing MCP posture.** The runtime already speaks MCP (M6) for tool/data integration; A2A is the analogous open standard for agent/runtime delegation. Same "adopt the open standard, don't reinvent" instinct.

Each runtime exposes an A2A endpoint; the super-orchestrator delegates tasks and collects results over it. The drone IPC layer stays internal; A2A is the *inter*-runtime wire.

---

## 4. Why the runtime is well-positioned

Capability containment, audit-trail integrity, and plan reconciliation are already solved at the architecture level for agents (`runtime-capabilities-roadmap.md` §1.5). Clustering reuses those guarantees one level up rather than re-solving safety per-feature:

- **Capability narrowing already exists on delegation edges** — extend it to the runtime→runtime edge (the super-orchestrator's grant to a child runtime narrows what that runtime, and transitively its agents, may do).
- **Audit-trail integrity** — each runtime keeps its own append-only SHA-chained trail; the super-orchestrator's delegations are events on its own graph. The composite audit story is the union, not a new mechanism.
- **Plan reconciliation when a child diverges** — the existing reconciliation machinery (spec §11) is the per-runtime analog of what a super-orchestrator needs across runtimes.

This is the same "designed safety-first, swarms as natural composition" argument §1.5 makes, applied to clusters.

---

## 5. The surface (sketch, not commitment)

- **A2A endpoint per runtime.** Each Agent Runtime can expose an A2A server endpoint (opt-in, capability-gated). Reuses the headless seam (M7-deferred CLI / server shape — see [`field-positioning-2026.md`](./field-positioning-2026.md) §2).
- **Super-orchestrator = an Agent Runtime instance** (recommended — dogfoods the model). Its agents delegate to child runtimes via A2A the way they spawn sub-agents today.
- **Capability federation in the schema.** Cross-runtime grants expressed in the framework schema — a schema change (ADR per §11, new/bumped schema per §14). The grant narrows; it never widens.
- **Identity + trust model.** Same-trust-domain first (shared owner, shared secret/keyring identity — reuses the existing keyring posture). Cross-org identity federation is explicitly deferred.

---

## 6. Out-of-scope locks (do NOT smuggle into v0.1, or let this drift into)

- **No cross-org federation in this proposal.** Same-trust-domain only; cross-org stays §1.4 v3.0+.
- **No concurrent-multi-agent dependency confusion.** Clustering can layer on top of concurrency work but is a distinct concern; don't bundle the §1.2 single-session-lock lift into this.
- **No bespoke protocol.** If A2A is adopted, the runtime implements the standard; it does not fork or extend it proprietarily without an ADR.
- **No self-modification across the boundary.** A child runtime executes what exists; a super-orchestrator delegates tasks, it does not rewrite a child's definition mid-run (`runtime-capabilities-roadmap.md` §3.4 applies at the cluster level too).
- **No telemetry / phone-home** introduced by the endpoint (spec §13).

---

## 7. Decision points before formal scoping

1. **A2A vs. bespoke IPC.** Recommendation: **A2A** (this proposal's thesis) — ecosystem interop + no protocol-maintenance burden.
2. **A2A version pinning + conformance.** Which A2A version; how the runtime declares conformance. Decided at scoping against the then-current spec.
3. **Super-orchestrator shape.** A full Agent Runtime instance (recommended — dogfoods) vs. a thinner dedicated coordinator.
4. **Capability-federation schema.** How cross-runtime grants are expressed (new schema vs. extension of the framework schema). ADR + schema change either way (§11/§14).
5. **Trust horizon for v1.5–2.0.** Same-trust-domain only (recommended) vs. attempting limited cross-org earlier.

---

## 8. References

### Trigger
- JetBrains PyCharm — "Top Agentic Frameworks for Building Applications 2026" (high-control cluster; "control, autonomy, governance" framing).
- [`field-positioning-2026.md`](./field-positioning-2026.md) §4 (penciled clustering scope), §2 (headless/server seam reuse).
- Google A2A (Agent2Agent) protocol — the open inter-agent/inter-runtime delegation standard the field standardized on.

### Internal cross-references
- [`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §1.3 (peer-to-peer swarm), §1.4 (cross-runtime swarm), §1.5 (why well-positioned for swarms), §3.4 (no self-modification).
- `agent-runtime-spec.md` §0d (scope matrix), §11 (plan reconciliation; ADR triggers), §13 (privacy), §14 (schema-as-source-of-truth), §8.security (capability model).
- M6 MCP integration — the precedent for adopting an open protocol rather than inventing one.

---

## 9. Next steps

1. Maintainer review.
2. If accepted: graduate `runtime-capabilities-roadmap.md` §1.3/§1.4 toward an A2A-based commitment; add to the post-M11 scope discussion (v1.5–2.0 same-trust-domain).
3. If accepted as a milestone: file the A2A-adoption ADR (§11) + capability-federation schema (§14); size the endpoint against the headless seam.
4. If rejected: archive with a one-line rationale.
