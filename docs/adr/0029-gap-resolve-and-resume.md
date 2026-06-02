# ADR-0029: Gap resolve-and-resume — the full front-to-back gap loop (v0.1)

**Status:** Proposed
**Date:** 2026-06-01
**Deciders:** @kknipe2k
**Tags:** capability, ipc, scope, architecture, hitl

## Context

The gap flow (`request_capability` → suspend) is the project's signature §0a
primitive: "gap detection that suspends the session cleanly when an agent
needs something it doesn't have." Rung 4 (ADR-0028) built the **suspend half**
— suspend-and-record (loop-break: halt + snapshot the run, recoverable per
§1b). ADR-0028 deferred the **resume half** (human resolves the gap → the
session continues) to M09.

**Maintainer direction (2026-06-01):** a runtime where a gap suspends and then
dead-ends is not a runtime. v0.1 must work **front-to-back, human-operable, no
AI required**: new-load → run → gap → **human grant/install/decline** → resume
→ complete. The mentor (M09) is the AI-assist layer *on top of* that loop and
is nothing until the loop closes. So **resolve-and-resume is pulled into v0.1**,
and this ADR decides its architecture — grounded in current best practice, not
improvised.

## Best-practice grounding (web research 2026-06-01)

Three independent bodies of practice converge on the same shape, and it matches
the substrate this project already has:

- **Durable checkpoint-and-resume** (LangGraph durable execution; Inngest;
  Google ADK long-running agents; Indium state-persistence): persist the
  complete execution state at the pause point; resume from the last checkpoint.
  → the project's **drone snapshot** (§1b — append-only, SHA-256-chained) *is*
  this checkpointer; no new persistence channel is needed.
- **Idempotency on resume (load-bearing rule):** *"do not re-execute tool calls
  that already have results in the history"* — resume **rebuilds** from the
  checkpoint and never re-runs completed side effects. → exactly the project's
  §1b "resume rebuilds (doesn't re-execute)." This is a hard correctness
  constraint on the resume path.
- **HITL = the suspend/resume primitive** (LangGraph `interrupt()` /
  `Command(resume=…)`): the agent pauses and stores state, awaits the human's
  value, and the resume **delivers that value back into the run as the
  interrupted call's result**. → the resolved `request_capability` returns a
  granted/denied `tool_result`.
- **Governed human re-authorization** (arXiv 2603.14332 *Governing Dynamic
  Capabilities* G1; arXiv 2603.20953 pre-action authorization; Oso
  context-aware permissions for AI agents): a change to an agent's capability
  set requires **explicit human re-authorization**; default standard-user +
  **governed elevation** for privileged actions. → a gap *is* a governed
  elevation: the human (the authorizing principal, no AI) grants the missing
  capability, the set changes, the runtime re-narrows, then resumes.

## Decision

v0.1 ships the **full gap loop, human-operable, no AI**, reusing the existing
substrate (drone snapshot §1b + `narrow()` + `HitlSeam` + the gap events):

1. **Suspend** (rung 4 / ADR-0028): `request_capability` → `ToolMissing` gap →
   loop-break + the run is snapshotted (the checkpoint).
2. **Resolve** (human, non-AI): the app surfaces the suspended gap (the
   live-graph node flips to `gap` + a resolution panel) with three **governed**
   actions — **Grant** (add the capability, re-narrowed to the requesting agent
   via the existing `narrow()`), **Install** (MCP gap: install + connect the
   server, then grant), **Decline** (deny). No AI mediates — the human is the
   authorizing principal.
3. **Resume**: on Grant/Install, the enforcer's grant set is updated
   (re-narrowed), the session **reloads from the snapshot** (§1b rebuild — *not*
   re-executing completed calls, the idempotency rule), the pending
   `request_capability` receives a **resolved `tool_result`** (granted), and the
   agent's loop re-enters at the next turn. On Decline, the pending call gets a
   **denied `tool_result`** and the agent continues (or the run ends) — clean
   either way.
4. **Recoverable across restart**: because the suspend is a snapshot, the
   resolution may occur across an app restart — the suspended session reloads
   from the snapshot chain.

The work splits across the two tracks already planned (sequencing in
ORCHESTRATOR §9):

- **Runtime resume** → an **M08.7 gap-resume rung**: the reload → deliver →
  continue mechanism, assembled-tested with a **stub resolution** (proves the
  engine resumes without any UI; the idempotency rebuild is the test's
  load-bearing assertion).
- **Resolution UI (the non-AI troubleshoot)** → the **app-workbench
  milestone**: the Grant/Install/Decline surface + the live-graph gap view,
  real-app IRL.

## Consequences

### Positive
- v0.1 becomes a **complete, human-operable runtime** (load → run → troubleshoot
  → resume → complete) — the actual product, not the suspend-only half.
- Best-practice-aligned (durable checkpoint-resume + idempotent rebuild +
  governed HITL re-authorization), reusing the existing substrate.
- The gap flow — the project's signature primitive — actually closes its loop.

### Negative
- Larger pre-M9 scope (the resume engine rung + the resolution UI) vs
  suspend-only. Justified: it is the product.
- The resume path carries a hard correctness constraint: **no re-execution of
  completed tool calls** on rebuild (idempotency). The gap-resume rung's
  assembled test must assert this.

### Neutral / future
- M09 (mentor) is re-cast as the **AI-assist layer on top of** the
  human-operable loop; **deprioritized** behind shipping the front-to-back
  runtime.
- **Supersedes ADR-0028's "resolve-and-resume deferred to M09."** ADR-0028
  remains the suspend-half decision; this ADR is the resume half + the full-loop
  architecture. (ADR-0028 lives on the M08.7 build branch and merges at the
  M08.7a PR; reconcile the cross-reference then.)

## Alternatives Considered

### Suspend-only for v0.1; resolve-and-resume at M09 (ADR-0028's original)
**Rejected:** a gap that cannot be resolved dead-ends the runtime — not a
shippable product. The maintainer's front-to-back requirement.

### AI-mediated gap resolution (the mentor resolves gaps)
**Rejected for the core loop:** the runtime must be human-operable without AI;
the human is the authorizing principal (governed-elevation best practice). The
mentor is a later assist layer, not a dependency of the core loop.

## Related

- ADR-0028 (gap-suspend, rung 4 — the suspend half this completes; on the M08.7 build branch)
- ADR-0019 (Tester isolated session — HITL/tier model the resume must respect)
- ADR-0022 (canonical framework representation — companions, relevant to Install)
- spec §1b (recovery rebuilds from snapshot, does not re-execute), §4 (gap flow), §8.security (`narrow()`)
- Web (2026-06-01): LangGraph interrupts + durable execution (docs.langchain.com); Inngest "Durable Execution… for AI Agents"; Google ADK long-running agents (developers.googleblog.com); Indium "7 State Persistence Strategies for Long-Running AI Agents (2026)"; arXiv 2603.14332 *Governing Dynamic Capabilities* (G1 human re-authorization); arXiv 2603.20953 pre-action authorization; Oso context-aware permissions for AI agents

## Notes

Directed by the maintainer 2026-06-01 ("pull it forward — a full runtime works
front-to-back, non-AI troubleshoot; M09 is fluff"). The architecture is
web-best-practice-grounded (above), not improvised. The detailed gap-resume rung
spec (X.3 changes, BDD) is authored at rung entry against the M08.7a evidence,
per the project's authored-at-entry pattern.
