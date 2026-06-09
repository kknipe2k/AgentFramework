# Execution status — the paints → executes capability ledger

> **What this is.** The single living source of truth for **which runtime
> primitives actually execute** (vs merely *paint* — emit events for the
> live graph — or *check* capabilities without running). It is the
> enforcement surface of **CLAUDE.md §4 rule 11** (grounded-claims): a
> primitive is **not "done"** until its row flips from `paints` to
> `executes` **with an eval cite** — an assembled-app observation, not a
> code citation. Reading code that *should* execute licenses nothing
> about runtime behavior (rule 11). This row flips only when an eval has
> **run the assembled path and observed the behavior**.
>
> **Why it exists.** M08.6 shipped a full milestone "Sound, 0🔴"; the
> post-M08.6 IRL walk (`docs/M08.6-irl-findings.md`) found **7🔴** and the
> load-bearing fact that *the runtime paints most of what it claims* —
> only provider-streaming + MCP execute. Nothing in the build machinery
> ever ran the assembled app. This ledger makes "executes — observed"
> a tracked, regression-guarded status instead of an assumption.
>
> **Lifespan.** **Seeded at M08.7; persists through v0.1 ship.** The
> M08.7 ladder closes the six `paints` rows below (its ending is "ARIA
> runs the v0.1 subset"); the ledger then remains the capability-status
> surface through **M09–M13** (the ADR-0032 vertical re-cut — execution
> breadth is now wired *per slice*: gap-resume + plans at M10, sub-agents
> at M11, hooks/rails + shell-exec at M12). Any milestone that adds or
> changes a runtime primitive adds or re-opens a row here.

---

## Single source of truth — relationship to the M08.7 rung table

This ledger is **not** a second copy of the
[`docs/build-prompts/M08.7-execution-engine.md`](build-prompts/M08.7-execution-engine.md)
rung table. Their jobs differ and so do their lifespans:

| | M08.7 rung table | This ledger |
|---|---|---|
| Answers | *how do we build each primitive* (the eval-first ladder) | *does each primitive execute, observed* |
| Status basis | code reality cited at authoring | **assembled-app observation (eval cite)** |
| Lifespan | M08.7-local; retired at M08.7 close | **durable — M08.7 → v0.1 ship** |

The rung table is the **build plan**; this ledger is the **observed
capability status**. While M08.7 is active, build-plan detail lives in
the rung table; the **flip to `executes`** is recorded here (the cluster
close-gate of each rung is what produces the eval that licenses the
flip — `docs/cluster-pattern.md` §1 step 4). To avoid drift, do not
duplicate build-plan detail here — link the rung.

---

## The ledger

**Status vocabulary.**

- `paints` — emits live-graph events (and/or passes a capability check)
  but performs **no real work**; the file is never read, the child never
  runs, the skill is never injected, the gap never fires. The cited code
  is the *current* grounded reality, **not** an execution claim.
- `executes — observed in assembled app, eval E-NN` — the assembled path
  has **run** and the **observable behavior / side effect** was seen
  (rule 11; the assertion is on disk/behavior, not on the emitted event —
  `docs/cluster-pattern.md` §4). `E-NN` cites the rung's assembled
  regression (the eval). **A row may not carry this status without a
  live `E-NN` cite.**

**Eval numbering.** `E-NN` is assigned as each rung lands its assembled
regression test; the cite is the test's file + name (the permanent
encoded eval, so the behavior can never silently regress). Landed:
**E-01** (rung 1) + **E-02** (rung 2) — row 1 (built-in tools) flipped to
`executes — observed (assembled)`; **E-03** (rung 3) — row 3 (skills)
flipped to `executes — observed` (CI structural injection + a real-model
behavior eval + maintainer IRL); **E-04** (rung 4) — row 4 (gaps) flipped
to `executes — observed (suspend-and-record)` (CI assembled wire + suspend
+ a real-model gap eval + maintainer IRL 2026-06-01: `request_capability`
→ `tool_missing(deploy)` → clean suspend); **E-05** (rung 5) — the **Budget**
row (Adjacent / out-of-ladder) flipped to `executes — observed at the
assembled run loop (engine)` — **engine-tested only**: the CI assembled wire
observes warn/downshift/hard-stop/idempotent, but the live real-model eval is
`#[ignore]d` and there was **no maintainer real-app IRL this rung**; the in-app
budget surface is deferred to M08.8. The remaining rows (2 sub-agents,
5 plans, 6 hooks/rails) stay `paints`.

### The six paints-not-executes primitives (the M08.6-IRL seed)

| # | Primitive | v0.1 target behavior | Status | Built by (rung) | Grounding at seed (code cite — NOT an execution claim) |
|---|---|---|---|---|---|
| 1 | **Built-in tools** (`Read`/`Write`; `Bash`/`Glob`/`Grep`/`WebFetch` deferred) | An allowed built-in runs and feeds its result back into the agent's next turn; a blocked one does not run (no side effect) | `executes — observed (assembled integration test), eval E-01 + E-02` — **real-app execution view observed at M08.8.A (TD-034 closed); the run-loop tier is now the user's tracked tier (TD-036 closed at M08.8.C)** ⚠️ the *positive* in-app scope-gate `CapabilityViolation(Write)` real-app-watch is key/live-model-dependent — assembled + maintainer-IRL grounded, not CI-driven (see [Observed in app](#observed-in-app--the-m088-app-workbench-surfaces) → Tier positive enforcement) | rung 1 (in-process `Read`/`Write` execute) + rung 2 (capability gate on live exec); `Bash`/OS-spawn = separate ADR-class rung | **E-01** `crates/runtime-main/tests/builtin_tool_execution.rs` (rung 1 — an allowed `Read` runs, the file contents feed back as a `tool_result` + the agent quotes them; rung 1 additionally real-app-watched a live Anthropic model quote `Cargo.toml` under `RUST_LOG=debug`, but the run is UI-unobservable — TD-034; + maintainer IRL 2026-05-31 (RUST_LOG chain: tool invoked `Read(Cargo.toml)` → tool result(content) → agent stream text quoted `[package]` → agent complete)). **E-02** `crates/runtime-main/tests/capability_live_tool.rs` (rung 2 — a Promoted out-of-scope `Write` emits `CapabilityViolation{Write}` + leaves **no file on disk**; an in-scope `Write` writes the file with its content). Assertions on the file side effect, not the event (rule 11 / §4). Scope-gate `CapabilityViolation` is assembled-observed only — the real-app Tester runs at Novice (TD-036), so the real-app scope-watch is deferred to the tier-wire + UI rung. |
| 2 | **Sub-agents** | Root spawns a child with narrowed grants; the child runs its own loop and returns a summary | `paints` — eval E-?? pending rung 6 close | rung 6 (sequential spawn ⭐) | `spawn_framework_subagents` emits `AgentSpawned` and stops; no child execution context is created (`agent_sdk.rs:467`) |
| 3 | **Skills** (`LoadSkill`) | Loading a skill injects its markdown into context and changes agent behavior | `executes — observed, eval E-03` ⚠️ **real-app/Builder-Tester IRL deferred (TD-037 — needs M09 canvas skill bodies)** | rung 3 (`LoadSkill` handler) | **E-03** `crates/runtime-main/tests/skill_load_execution.rs` (CI — the skill body is present in the turn-2 `AgentConfig`: injection-into-context, structural) + `crates/runtime-main/tests/skill_load_live.rs` (`#[ignore]`d — real Anthropic, behavior-change) + **maintainer IRL 2026-05-31** (`LoadSkill(shout)` → reply ALL CAPS, observed: *"HELLO! WELCOME, AND THANKS FOR STOPPING BY!"*). Per ADR-0027 the body rides back as the `LoadSkill` `tool_result`, persisting in `config.messages` across turns; capability-gated by `allowed_skills`. Behavior-change asserted, not the `SkillLoaded` event alone (rule 11 / gotcha #66). The real-app **Builder-Tester** skill IRL is deferred — the v0.1 canvas authors no skill bodies to thread (TD-037). |
| 4 | **Gaps** (`request_capability`) | An unheld capability raises a gap event and suspends the session cleanly + recoverably | `executes — observed (suspend-and-record), eval E-04 + maintainer IRL 2026-06-01 (request_capability → tool_missing(deploy) → clean suspend)` ⚠️ **resolve-and-resume (the resume half) is the scheduled gap-resume rung — ADR-0029 (`docs/adr/0029-gap-resolve-and-resume.md`); NOT wired this rung** | rung 4 (wire into the run loop) | **E-04** `crates/runtime-main/tests/gap_detection_execution.rs` (CI assembled — a `request_capability` `ToolUse` routes to `handle_request_capability` (not `pipeline.next_event`), the `*Missing` gap fires with `requested_via=request_capability`, and the loop **suspends**: one provider turn, clean `Ok`, suspend **even when the same turn dispatched a tool** + the four kind arms + the malformed no-suspend path) + `crates/runtime-main/tests/gap_detection_live.rs` (`#[ignore]`d — real Anthropic) + **maintainer IRL 2026-06-01** (a real model lacking `deploy` called `request_capability` → `ToolMissing(deploy, requested_via=request_capability)` → **clean suspend**, watched). The SUSPEND behavior is asserted (one turn, clean halt, gap left **unresolved** — no `tool_result` fed back), not the event alone (rule 11 / gotcha #66). resolve-and-resume = scheduled gap-resume rung (ADR-0029). |
| 5 | **Plans** (`drive_plan`) | An approved plan drives real task execution (`TaskStarted`/`TaskCompleted` → `PlanComplete`) | `paints` — eval E-?? pending rung 7 close | rung 7 (ADR-0026) | `drive_plan` advances the FSM but its docstring states it "runs no tasks" and it has **no production caller** (M08.V 🟡 #2; `plan_loop.rs:79`,`:128-129`) |
| 6 | **Hooks / rails** | A hook fires on its trigger; a rail-violating action is blocked | `paints` — eval E-?? pending rung 8 close | rung 8 (firing engine) | Defined in framework JSON; **no firing engine** (`M08.7-execution-engine.md` §Background) |

> **Row 5 routing note (resolved — ADR-0026).** `plan_loop` production
> wiring is owned by **M08.7 rung 7** (zero-propagation re-route,
> superseding M08.V Decision 2's →M9.A). `TestOutcome.vdr` population —
> a Tester output field, not one of the six execution primitives, so it
> has no row here — stays **M9 Stage A** (ADR-0026; the earlier
> ORCHESTRATOR §9 "vdr → rung 6" note was a mislabel: rung 6 is
> sequential spawn). The behavior remains `paints` until rung 7's
> append-on-close eval observes it (see Maintenance protocol).

### Already executes (recorded for the complete capability picture)

| Primitive | v0.1 target behavior | Status | Durable eval | Grounding (code cite) |
|---|---|---|---|---|
| **Provider streaming** (single-agent multi-turn) | An agent streams multi-turn until a turn dispatches no tool (or `MAX_AGENT_TURNS`) | `executes` (cited) — durable eval E-?? pending rung 0 baseline | rung 0 (smoke baseline) | `AgentSdk::run_agent` runs the prelude then `provider.stream` → `drive_stream` per turn (`agent_sdk.rs:256`,`:261`) |
| **MCP tool dispatch** | An MCP tool executes and its result feeds back into the next turn | `executes` (cited) — modulo M08.6 #23 Windows `npx`; durable eval E-?? pending rung 9 | rung 9 (fix #23) | `try_mcp_dispatch` calls the `McpToolDispatch` seam, emits `ToolInvoked`/`ToolResult`, returns a `DispatchedTool` the loop feeds back (`agent_sdk.rs:576`,`:269-294`) |

> Code citations above establish *current execution*, but per rule 11 a
> **durable** `executes — observed` status still requires the ladder's
> assembled regression to cover the path (rungs 0 + 9). Until that eval
> lands, the status is `executes (cited)`, not `executes — observed,
> eval E-NN`.

### Adjacent / out-of-ladder

| Primitive | Status | Note |
|---|---|---|
| **Budget** (warn / downshift / hard-stop) | `executes — observed at the assembled run loop (engine), eval E-05` ⚠️ **engine-tested only — the live real-model eval is `#[ignore]d`; NO maintainer real-app IRL this rung; the in-app budget surface (Settings cap + cross-restart persistence, M08.6-IRL #22) is deferred to M08.8 (app-workbench)** | rung 5 (M08.7.E — per-turn budget at the run loop) | **E-05** `crates/runtime-main/tests/budget_runloop_execution.rs` (CI assembled — `record_spend` accumulates across turns; the threshold halts the run + issues **no further turn**; `Downshift` swaps the model for subsequent turns; `Warn` emits `BudgetWarn` and does **not** stop; idempotent across turns — assertions on the counted-turns halt + the emitted action, not a painted event) + `crates/runtime-main/tests/budget_live.rs` (`#[ignore]`d — a real Anthropic run under a tiny cap hard-stops with no runaway; E-05 live, **not** CI-run, no maintainer IRL watch this rung). Grounded-by-execution at the **assembled-stub** granularity (V ran it); the real-model halt + the in-app cap surface are the M08.8 app-observe. Rung 5 grounded budget as tracked-but-not-enforced-at-the-loop at entry (rule 11), then wired enforcement. |
| **Modes** (router) | out of v0.1 scope | STANDARD hardcoded (§0d, gotcha #3); no router until v1.0 |

---

## Observed in app — the M08.8 app-workbench surfaces

> **What this bar adds (M08.8).** M08.7 flipped the rows above to `executes
> — observed` at the **engine** granularity (assembled `E-NN` evals + per-rung
> RUST_LOG/IRL); the assembled *app* was not the observation surface. M08.8
> made the engine **observable + operable in the running Tauri app** for the
> built surfaces, and closes each on the **real Tauri app driven via
> `tauri-driver`** (ADR-0021; the merge-blocking `e2e-tauri-driver` job) + the
> maintainer IRL — never Playwright-mock-green, never code-read (rule 11). Each
> row below cites its `tests/e2e-tauri/*.e2e.ts` regression + the
> observation date. Per the M08.8.V retro (2026-06-05), the real-app suite ran
> **22/22 assertions green on a clean Novice baseline** (10/10 spec files);
> the 3 deltas seen as-found on a contaminated machine were stale-`tier.json`
> persisted state (V 🟡 #1), fixed at this closeout (the hermetic
> `beforeSession` seed in `wdio.conf.ts`).

| Surface (M08.8 stage) | Real-app E2E | Observed in the running app | Grounding |
|---|---|---|---|
| **Execution view** — `stream_text` → Output rail; tool `{input,output}` on a clickable node; two-axis node render (A) | `tests/e2e-tauri/execution_view.e2e.ts` (2/2) | Agent stream text rendered live in the Output rail; a `Read` tool-node click surfaced its `input` + returned contents; tool node `data-kind=tool` + `data-status=complete`; same rail over the scoped Tester store. **Closes TD-034.** | grounded-by-real-app (V 2026-06-05 + maintainer IRL) |
| **Light Instrument visual foundation** — light token system, 52/232/360 three-pane shell, node cards, Toast, Modal-with-untruncated-labels (B) | `tests/e2e-tauri/visual_foundation.e2e.ts` (3/3) | Light canvas (not the dark M03 `:root`); three-pane shell present; node card `surface-0`; MCP modal "Cancel" untruncated (#24 closed); a pushed Toast appears bottom-right + auto-dismisses. | grounded-by-real-app (V 2026-06-05 + maintainer "looks like the mockup" IRL) |
| **Complete restyle** — every surface to its `workbench.css` spec; IBM Plex bundled; `TesterModal`→`Modal` (B.fix) | `tests/e2e-tauri/design_conformance.e2e.ts` (5/5) | IBM Plex computed on the brand (bundled woff2 wins, not fallback); segmented view-switch; node duration in Plex Mono; hairline borders (<1.5px); Tester is `modal--full` + `role=dialog`. **Closes TD-043.** | grounded-by-real-app (V 2026-06-05 + maintainer IRL) |
| **Reload reconstructs the graph** — replay-on-mount from persisted signals (B.fix2) | `tests/e2e-tauri/smoke.e2e.ts` test 6 (6/6, incl. a live Anthropic run) | A real run → reload → the graph reconstructs from the persisted signal log (`data-status=complete` after `reloadSession`). **Closes TD-044.** | grounded-by-real-app, live session (V 2026-06-05) |
| **Tier — UI truth + display seed** — topbar tier chip state-visible; truthful "Promote" at Novice (#20); the display seeds the enforced tier across a reload (#19) (C / C.fix) | `tests/e2e-tauri/tier_enforcement.e2e.ts` (2/2) + `tier_display.e2e.ts` (1/1) | Topbar tier chip shows the current tier; button reads truthful "Promote" at Novice (not "Promote to Promoted"); promote → display "Demote"/"Promoted" + a tier-change toast → reload → display still Promoted (the C.fix mount seed fired). On a clean Novice baseline. | grounded-by-real-app (V 2026-06-05 + maintainer IRL) |
| **Tier — positive enforcement** — a Promoted run reaches the L1 **scope** gate (a Promoted in-scope Write lands; an out-of-scope Write is `CapabilityViolation`); a Novice run is **tier**-denied first (C) | assembled: `src-tauri/src/commands.rs::test_framework_with_at_promoted_…` / `…_at_novice_…` (the scope-vs-tier gate contrast; neither writes the file out of scope) | The tier the run loop enforces is the user's tracked tier (TD-036 closed: `test_framework` → `test_framework_with` → `run_test_session_with_tier` → `enforcer.set_tier`). **NOT real-app-driven by V** — the positive Promoted-Write-lands path needs a live Anthropic round-trip CI does not provide (CLAUDE.md §10); grounded by the assembled gate-contrast test + the maintainer IRL. | assembled + maintainer IRL (hypothesis from V's seat — labeled, not real-app-driven by V) |

> **Not flipped (superseded by ADR-0031 — recorded, not closed).** The
> in-app **budget surface** (D), **gap resolve-and-resume** Grant/Install/
> Decline (E), and the **save-path** 🔴 (F) were re-cut out of v0.1 and never
> built — their DESIGN.md surfaces (budget bar, transport gap/incident ticks
> driving resolution, gap node → banner → Grant) are out-of-scope absences,
> not findings. They route to **M11 / M13** (ADR-0031). The transport shell +
> run banner shipped at B; their budget/gap *behaviors* did not.

---

## Maintenance protocol

1. **A row flips to `executes — observed, eval E-NN` only at a cluster
   close** — when the rung's assembled regression has run and the
   maintainer IRL-watched the behavior (`docs/cluster-pattern.md` §1 step
   4). "Tests green" alone does not flip a row. On flip, the primitive
   **joins the append-on-close cumulative execution-regression eval**
   (`docs/cluster-pattern.md` §9): every subsequent rung close re-runs the
   accumulated suite, so a flipped row that later regresses re-opens to
   `paints`.
2. **The flip cites the eval** (the assembled regression test's file +
   name). A flip without a live `E-NN` cite is a rule-11 violation and is
   rejected at the cluster's surface.
3. **New primitives add a row** (seeded `paints` with a grounding code
   cite). A regression that breaks an `executes` primitive **re-opens its
   row** to `paints` until re-observed.
4. This ledger is part of the **milestone IRL-confirm + Stage V
   assembled-execution pass** review surface (`docs/cluster-pattern.md`
   §3; the v1.9 fifth verifier pass): V confirms each `executes` row's
   eval actually runs the assembled path.
