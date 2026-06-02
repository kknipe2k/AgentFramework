# ADR-0028: Built-in tool execution contract

**Status:** Accepted
**Date:** 2026-06-02
**Deciders:** @kknipe2k
**Tags:** capability, security, scope, architecture

## Context

M08.7 (the execution engine) turned the runtime's painted primitives into
running ones. Stage A (rung 1) shipped the first executor — the in-process
built-in tools `Read`/`Write` (`execute_builtin` dispatches exactly these two;
`crates/runtime-main/src/sdk/builtin_tools.rs:90`) — and Stage B (rung 2) put
the capability gate in front of live execution. The shape Stage A froze is the
archetype **every** later rung mirrors (skill load, gap, budget, MCP dispatch):
a tool's `ToolUse` is executed and its result is fed back into the agent's next
turn. That contract was authored in code and exercised by the rung-1/2 assembled
tests, but the decision record meant to freeze it (the phase doc's "Two ADRs",
item 6) was never filed: the intended number `0026` was consumed by an unrelated
routing ADR (`0026-plan-loop-vdr-carry-forward-routing`, already Accepted), so
the built-in-execution contract had no ADR. M08.7.V flagged this (🟡 #1). This
ADR files the contract under the next free number, `0028`.

The decisions to freeze:
- **What "execute" means for a built-in** — the multi-turn feedback contract.
- **What authorizes a built-in** — the capability boundary.
- **Where a built-in runs** — in-process vs sandbox, and which tools are deferred.

## Decision

We freeze the built-in tool execution contract as Stage A implemented it and
every M08.7 rung mirrors.

1. **Multi-turn feedback contract.** A built-in `ToolUse` emitted by a turn is
   dispatched through `execute_builtin` (`runtime-main/src/sdk/builtin_tools.rs`),
   its result is collected into `TurnFeedback::dispatched`, and the run loop
   (`AgentSdk::run_agent`) feeds it back as the next turn's input — one assistant
   message carrying every `ToolUse`, then one user message carrying the matching
   `tool_result`s (the Anthropic continuation contract). The loop re-streams
   until a turn dispatches no tool (the model stopped) or `MAX_AGENT_TURNS` is
   reached. Built-ins route into `dispatched`, **not** to `pipeline.next_event` —
   the wire that distinguishes "executed and fed back" from "painted and dropped"
   (the M08.6 paints-not-executes bug). Every later rung (skill, gap, budget, MCP)
   is a branch of this same dispatch→feedback shape.

2. **`file_access` scope is the boundary.** A built-in is authorized by the
   requesting agent's capability declaration — specifically the `file_access`
   read/write globs for `Read`/`Write`, gated at L1 by the `CapabilityEnforcer`
   **before** the tool runs (rung 2). An out-of-scope path emits
   `CapabilityViolation` and the side effect never happens (no file read or
   written). The declared capability scope, not a separate per-call
   authorization step, is the enforcement point.

3. **In-process vs sandbox split; only `Read`/`Write` implemented; `Bash`
   deferred.** A built-in falls into one of three classes:
   - **Implemented in-process (rung 1):** `Read`/`Write` run in the main runtime
     today — bounded by the capability + path scope above, needing no separate OS
     sandbox. These are the only built-ins `execute_builtin` dispatches at M08.7a
     (`builtin_tools.rs:90`); the `execution-status.md` row-1 status reflects this.
   - **In-process class, not yet implemented:** `Glob`/`Grep`/`WebFetch` belong to
     the same in-process, capability+path-bounded class (no OS sandbox needed) but
     are **not yet wired** — they are a future in-process rung, not part of the
     M08.7a shipped set.
   - **Sandbox-class, deferred:** `Bash` (and any OS-process-spawning built-in) is
     **deferred to a separate ADR-class rung** — it crosses the sandbox boundary
     (`runtime-sandbox`; CLAUDE.md §10 / spec §8.security) and is not part of the
     v0.1 in-process set.

## Consequences

### Positive
- The execution archetype is recorded, so every later rung (M08.9 sub-agents /
  plan / hooks, and beyond) has one contract to mirror, not a re-invented one.
- The dispatch→feedback wire is named as the line between "executes" and
  "paints" — the regression the execution-status ledger and Stage V guard.

### Negative
- Files the contract late (at closeout, not Stage A). The code shipped and was
  tested first; this ADR documents already-running behavior rather than proposing
  new design — its `Proposed → Accepted` flip is a formality.

### Neutral / future
- `Bash` / OS-spawn built-ins are explicitly out of this contract and carry
  forward as their own ADR-class rung.
- Two refinements to the boundary are tracked as debt, not re-opened here:
  **TD-033** (the executor gates on `file_access` scope, not a distinct
  execution-time tool-authorization check) and **TD-035** (relative paths resolve
  against the process CWD, to be made workspace-relative in a later rung).

## Alternatives Considered

### Renumber the plan-loop-vdr ADR to free 0026
**Rejected:** `0026-plan-loop-vdr-carry-forward-routing` is already Accepted and
referenced; ADRs are immutable once accepted (CLAUDE.md §11). The built-in
contract takes the next free number (0028) instead.

### A per-call tool-authorization layer instead of file_access-scope-as-boundary
**Rejected for v0.1:** the capability `file_access` scope is the declared
boundary the framework author controls; a separate per-call auth layer is a
larger surface deferred (TD-033) — not needed for the v0.1 in-process built-in
set.

## Related

- Spec: §0b (Tool/Skill/Agent), §8.security L1 (capability gate), §6 (Framework JSON Loader)
- Phase doc: `docs/build-prompts/M08.7-execution-engine.md` (Stage A rung 1, Stage B rung 2)
- ADR-0027 (skill-into-context injection — the sibling M08.7 ADR; skills mirror this feedback contract)
- Prior art it freezes: `runtime-main/src/sdk/builtin_tools.rs`; `AgentSdk::run_agent` / `drive_stream` (`agent_sdk.rs`)
- TD-033 (execution-time tool-authorization), TD-035 (relative path resolution)
- Tests: `runtime-main/tests/builtin_tool_execution.rs` (rung 1), `capability_live_tool.rs` (rung 2)

## Notes

Filed late per M08.7.V finding 🟡 #1: the phase doc ("Two ADRs", item 6) reserved
number 0026 for this contract, but 0026 was used for
`plan-loop-vdr-carry-forward-routing`; this ADR takes the next free number
(0028). The phase-doc cross-references were repointed 0026→0028 in the same
closeout commit. The contract documented here is what rungs 1–2 shipped and
M08.7.V observed by execution (40/40 assembled tests), not new design.
