# ADR-0007: In-process HITL seam architecture (Approval + HITL flows)

**Status:** Accepted
**Date:** 2026-05-11
**Deciders:** @kknipe2k (maintainer); architectural call surfaced during M04 Stages B/C/E
**Tags:** ipc, architecture, hitl, approval, capability

## Context

M04 introduced two human-in-the-loop suspension points where the SDK's async work waits for a user decision before continuing:

1. **Plan approval gate** (spec §3a) — when `Plan.approval_required = true`, the SDK emits `plan_approval_requested`, suspends, and resumes on `plan_approved` / `plan_revised` / `plan_aborted` from the user.
2. **Failure-escalation HITL** (spec §6a) — when a task fails ≥ `max_failures` times, the SDK emits `task_escalated`, suspends, and resumes on the user's choice (retry / skip / abort).

Both flows share the same structural shape: the SDK awaits, the renderer presents UI, the user picks, the SDK resumes. The implementation question is **where the wait-and-resolve primitive lives**.

The M04 phase doc's initial wording (pre-audit) implied these flows would route through drone IPC: renderer → Tauri command → drone IPC → drone holds suspension state → drone signals main → main resumes. M04 Stages C and E surfaced the question explicitly and chose a different architecture: keep the seam in `runtime-main` as a `tokio::sync::oneshot` channel, resolve fully in-process via the Tauri command, and inform the drone of the outcome via the existing `WriteSignal` IPC path (M04 Stage B).

The architectural call is forward-applicable to M05 (capability-violation HITL), M06 (MCP user-prompt flows), and M07 (framework-loader user prompts). Pinning it now keeps M05+ stages from re-litigating.

## Decision

**HITL seams live in `runtime-main` as in-process `tokio::sync::oneshot` channels. Renderer → Tauri command → `seam.resolve(decision)` is fully in-process; the drone receives the user's decision via the existing `WriteSignal` IPC path, not via approval-specific IPC.**

Concretely:

- `runtime-main` exposes two seam types: `ApprovalSeam` (M04 Stage B) and `HitlSeam` (M04 Stage E). Each maintains a `HashMap<prompt_id, oneshot::Sender<Decision>>` for in-flight suspensions.
- The SDK's plan loop / failure-escalation flow calls `seam.await_decision(prompt_id) -> Decision`. The future resolves when the renderer drives the seam from the other side.
- Renderer-side Tauri commands (`approve_plan`, `revise_plan`, `abort_plan`, `respond_hitl`) accept a managed-state `Arc<ApprovalSeam>` / `Arc<HitlSeam>` and call `seam.resolve(prompt_id, decision)` directly. The Tauri command does not roundtrip through the drone.
- After the SDK resumes, it emits the outcome event (`plan_approved`, `plan_revised`, `task_started` for retry, etc.) via the existing event-emit path. That event hits `WriteSignal` IPC and lands in `signals` + projection tables (Stage B's plan_projector / Stage E's HITL projector if added later) for audit + replay.
- **The drone is the audit log, not the orchestrator.** It records what happened; it does not own the suspension state.

## Consequences

### Positive

- **Simpler IPC topology.** One signal-write path for all HITL outcomes instead of approval-specific IPC variants. Reduces variant count in `DroneCommand` from "WriteSignal + ApproveDecision + HitlDecision + ..." to just `WriteSignal`.
- **Lower latency.** Renderer click → seam resolve is microseconds; drone-mediated would add ~ms of IPC roundtrip + drone-side state lookup.
- **No drone-side state machine for HITL.** Drone stays focused on persistence + projection; HITL state lives where the SDK that awaits it lives.
- **Recovery is uniform.** Audit trail in `signals` table replays through the existing event pipeline; no separate HITL state to restore.
- **Aligns with single-session v0.1 scope** (§0d). One main process, one SDK awaiter, one renderer client.

### Negative

- **Main-process death loses pending HITL.** If `runtime-main` crashes while an `ApprovalSeam` future is pending, the user's pending decision is lost — the renderer will see a stale prompt with no live awaiter. v0.1 single-session means this is "session aborted, restart"; v1.0 multi-session would need a different mechanism (likely drone-persisted prompt-state + re-binding at session resume).
- **Drone cannot drive a HITL prompt.** If the drone needs to surface a user prompt (e.g., snapshot-corruption recovery, drone-side capability violation), it can't use the same seam — drone would need to send an IPC event to main, which would then create an `ApprovalSeam` entry. Acceptable in v0.1 (drone-driven prompts aren't in scope); revisit if the drone surface grows.
- **Renderer-side state injection for tests.** Without drone IPC in the approval round-trip, renderer-level Playwright tests that drive approval flows need a state-injection affordance (M04 graduated gotcha #54 — `window.__graphStore`). Vitest-level tests mock the Tauri command directly.

### Neutral / future implications

- **M05 capability-violation HITL** (spec §6a trigger `on_capability_violation`) follows this pattern: when the capability enforcer fires a violation, it routes through the existing `HitlSeam` (already shipped in M04.E) — no new IPC variant.
- **M06 MCP user prompts** (per MCP spec's `elicitation` flow) follow this pattern: create a new `McpPromptSeam` or extend `HitlSeam`.
- **M07 framework-loader prompts** (e.g., "this framework declares uses of skills you haven't loaded — proceed?") follow this pattern.
- **v1.0 multi-session may require revisit.** If session resume needs to restore pending HITL across main-process restarts, the seam state will need persistence (drone-side or other). Track as v1.0 carry-forward; v0.1 ships in-process only.

## Alternatives Considered

### Alternative A: Drone-mediated HITL

Renderer → Tauri command → drone IPC (`HitlResponse { prompt_id, decision }`) → drone holds suspension state → drone signals main via a separate IPC event → main resumes.

**Rejected because:** adds an IPC roundtrip with no benefit in v0.1; doubles the variant count in `DroneCommand` for each new HITL type; couples drone to UI-flow state machines (which it has no reason to own); makes recovery harder (drone must persist + restore in-flight HITL prompts on every snapshot, then re-bind to a new SDK awaiter after restart).

### Alternative B: Synchronous Tauri-only with no SDK suspension

Tauri command holds the awaiting future itself; SDK plan-loop polls a "decision is ready" flag instead of awaiting.

**Rejected because:** the SDK plan loop is fundamentally async (calls into the agent loop, which awaits LLM streaming); poll-based suspension would either busy-wait or add an internal channel anyway. The `tokio::sync::oneshot` pattern is the idiomatic async-Rust expression of "wait for one decision."

### Alternative C: Cross-process pipe to a separate "hitl-daemon" process

Spawn a dedicated process to hold HITL state; main + drone both communicate with it via IPC.

**Rejected because:** introduces a third long-running process for v0.1 single-session without benefit; the seam state is naturally co-located with the SDK that awaits it.

## Related

- Spec sections: §3a (Plan approval gate); §6a (HITL primitive + 9 triggers); §1b (Recovery semantics — currently-running task = pending)
- M04 Phase doc: `docs/build-prompts/M04-plan-verify-hitl-budget.md` §B.3 (ApprovalSeam exposure), §C.3 (renderer wiring), §E.3 (HitlSeam wiring)
- M04 Retrospectives: `M04.B-retrospective.md`, `M04.C-retrospective.md` (architectural decision), `M04.E-retrospective.md` (HitlSeam mirroring)
- M04 Commits: `962525e` (Stage B `ApprovalSeam`), `1138486` (Stage C in-process Tauri command), `2996cff` (Stage E `HitlSeam`)
- Prior ADRs: none directly superseded; ADR-0002 (Tauri over Electron) and ADR-0003 (Engineering Charter) are the architectural context this builds on

## Notes

Surfaced as M04 closeout gap-analysis 🟡 backlog item ("in-process seam architecture ADR"). The decision was *de facto* made at M04 Stages B/C/E during execution per CLAUDE.md §12 ("own technical decisions"); this ADR records it formally so M05+ stages don't re-litigate.

The drone's role as "audit log, not orchestrator" applies broadly. Same architectural pattern: M03's VDR projector (drone records decision/verify signals — doesn't drive the decisions); M04's plan/task projector (drone records plan-state changes — doesn't drive the FSM). HITL seam in-process fits the same shape.
