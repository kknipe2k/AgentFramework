# M04 Plan + Verify + HITL + Budget — Specification + Stage Prompts

**Protocol version:** v1.3 (first milestone authored on the v1.3 XML stage-prompt schema; uses `<pre_flight_check>`, `<schema_drift_check>`, `<fan_out_grep>`, `<dependency_audit_check>`, and `<runtime_environment>` tags per `STAGE-PROMPT-PROTOCOL.md` v1.3).
**Date:** 2026-05-07 (initial authoring); 2026-05-08 (revised post-M03.5 codebase audit per `docs/gotchas.md` #41 + STAGE-PROMPT-PROTOCOL.md §10 v1.3 hardening rule).
**Status:** Design approved — A1–G all unexecuted; staging grounded in verified end-of-M03.5 codebase reality.
**Scope:** Build the four agentic primitives that turn a single-agent smoke into an actual agentic runtime: §3a Plan & Task (with `fresh_context_per_task` loop policy + 11 spec-canonical plan/task events of which 6 already exist in `event.rs` + ApprovalPanel), §4a Verify & Rails (Hook primitive with 7 firing points including new `pre_file_edit` + Rails hard/soft + don't-touch globs + existing `RevertToSnapshot` drone command already in `DroneCommand`), §6a HITL (9 trigger types + 3 UI variants Panel/Modal/Toast + 3 built-in notifiers terminal_bell/desktop/sound + plugin interface; codebase event names `hitl_requested` + `hitl_resolved` — NOT `hitl_response`), §2a Budget (3 scopes + 4 threshold actions + downshift_hook + 4 budget events already in `event.rs` — wired not added + UI header bar). Plus §1b Recovery (resume rebuilds history, tool-call-uncertain prompt). Seven stages on one feature branch (`claude/m04-plan-verify-hitl-budget`): A1 → A2 → B → C → D → E → F → G (closeout per CLAUDE.md §20). Spec §1b + §2a + §3a + §4a + §6a + MVP §M4 acceptance criteria.

---

## Background and Design Decision

> **Revised post-M03.5 codebase audit (2026-05-08).** Per `docs/gotchas.md` #41 + STAGE-PROMPT-PROTOCOL.md §10 v1.3 hardening rule (PR #51), every Phase doc claim about codebase reality is grep-verified at authoring time. The previous M04 draft (commit `875b84c`) framed itself as a "post-M04.A2 audit" and marked Stages A1 + A2 ✅ DONE; revisiting the codebase showed both were unexecuted — no `M04.A*` retrospectives in `docs/build-prompts/retrospectives/` (per CLAUDE.md §19 every stage produces one); `crates/xtask/src/main.rs:45` still names only `["common", "framework", "skill", "tool", "agent"]`; no `src-tauri/src/drone_lifecycle.rs`; no `src-tauri/src/lib.rs` (only `main.rs`); `src-tauri/src/commands.rs:1` still reads "Tauri command surface for M02 Stage E"; `CmdError` at `src-tauri/src/commands.rs:46` is still hand-maintained; `crates/runtime-main/src/providers/anthropic.rs:135` `count_tokens` still uses chars/4 approximation; the 3 `DroneClient::noop()` callsites at `src-tauri/src/commands.rs:166, 200, 247` remain. The audit findings recorded throughout this doc describe the codebase as of end-of-M03.5 (last shipped milestone), not as if any M04 stage had executed. M04 starts at A1 and proceeds through G; staging reflects what's actually in `event.rs`, `drone.rs`, `vdr.rs`, `graphStore.ts`, and `commands.rs` today. Per STAGE-PROMPT-PROTOCOL.md §10, audit-driven corrections live INSIDE each stage's `<work_stage_prompt>` XML (in `<context>`, `<pre_flight_check>`, `<read_reference>`, `<gotchas>`, and `<execution_warnings>` slots) — the build agent at execution time reads only the XML inside fenced ```xml blocks, not the narrative Markdown wrapper. Three audit findings the previous draft missed are surfaced here for the first time: (a) the "8 of 11 plan/task events already exist" framing is misleading — only 6 of the 11 spec §3a-canonical variants exist in `event.rs` (`PlanCreated`, `PlanApproved`, `TaskStarted`, `TaskCompleted`, `TaskFailed`, `TaskEscalated`); 2 codebase variants are unspecified extras (`PlanRejected`, `TaskRolledBack`); 5 spec variants are missing (`plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped`); (b) `crates/runtime-core/src/error.rs` already exists as a hand-curated `RuntimeError` (workspace internal error per `lib.rs:4–5` "Types in `event.rs`, `drone.rs`, and `error.rs` are hand-curated") — Stage A1's codegen target for the IPC `CmdError` is `crates/runtime-core/src/generated/error.rs` and the namespace clash with the existing `error.rs` is a Stage A1 design surface (rename + module split, decided at execution time); (c) `RevertReason::HookRollback` at `crates/runtime-core/src/drone.rs:266` is a unit variant in code; spec §4a's `revert_to_snapshot` description implies it should carry a `hook_id: String` field — Stage D's design surface (decided at execution time).

**Problem.** M03 lit up the live graph for the M02 single-agent smoke session — one AgentNode renders, click-to-inspect works, token weight scales. The other 10 node types (PlanNode, TaskNode, VerifyNode, HookNode, GapNode, HITLNode, MCPNode + four Plan/Task event types) render in unit tests with synthetic state but never light up live: no event source fires their corresponding `AgentEvent` variants. Spec §M4 declares the four primitives (plan, verify, HITL, budget) that produce those events. Loading `examples/aria/framework.json` and seeing a multi-task plan render with verify hooks firing post-task and the budget header bar tracking session spend is the M04 success surface.

**Solution.** Seven stages on one feature branch (`claude/m04-plan-verify-hitl-budget`), each a fresh Claude Code session per the v1.3 XML stage-prompt protocol. **Stage A1** closes M03 carry-forward 🟡 build-hygiene items: extends `crates/xtask/src/main.rs` codegen list (currently 5 schemas — common/framework/skill/tool/agent) to include `event.v1.json` + `error.v1.json`; regenerates types into `crates/runtime-core/src/generated/`; resolves the namespace clash between the existing hand-curated `crates/runtime-core/src/error.rs` (which holds `RuntimeError`, the workspace internal error) and the new generated `CmdError` (the IPC error) by either renaming the existing file (e.g., to `runtime_error.rs`) or splitting into modules — decision at A1 execution time; retrofits `await_event` `tokio::time::pause()` coverage; verifies `crates/runtime-drone/tests/integration*.rs` use `current_exe()`-derived paths cleanly. **Stage A2** lands production wiring: spawns the drone subprocess at Tauri startup (`Arc<DroneClient>` registered via `app.manage(...)` Tauri-managed-state — new sibling module `src-tauri/src/drone_lifecycle.rs` since `src-tauri/src/lib.rs` does NOT exist; spawn logic lands in the existing `main.rs::main()`); replaces 3 `DroneClient::noop()` callsites at `src-tauri/src/commands.rs:166, 200, 247` (`run_smoke_session`, `query_session_db`, `replay_session`) with real drone IPC; replaces `count_tokens` chars/4 approximation at `crates/runtime-main/src/providers/anthropic.rs:135` with real `POST /v1/messages/count_tokens` (wiremock-tested per spec §2c.3 added M03.5); refactors hand-maintained `CmdError` at `commands.rs:46` to re-export A1's generated type from `runtime-core`; renderer-side `unwrapCmdError` in `src/lib/ipc.ts` consumes the generated `CmdError` from `src/types/error.ts`; resolves long-lived `events()` reconnect behavior under integration test (closes M02 carry-forward 🟡 item; outcome locked as v0.1 behavior either way per CLAUDE.md §12 design-call discretion). **Stage B** builds the §3a Plan & Task primitive — authors `plan.v1.json` + `task.v1.json` schemas (xtask codegen extension over A1's archetype); authors the 5 missing spec event variants (`plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped`); wires the 6 existing spec-canonical variants (`PlanCreated`, `PlanApproved`, `TaskStarted`, `TaskCompleted`, `TaskFailed`, `TaskEscalated`) into the new plan state machine (does NOT re-author them); documents the disposition of the 2 codebase-only extras (`PlanRejected`, `TaskRolledBack` — likely keep both as additive) at execution time; plan state machine with `fresh_context_per_task` loop policy + failure escalation (≥95% safety primitive); plans + tasks SQLite tables (first migration creates the `crates/runtime-drone/migrations/` directory which does not yet exist); approval-gate seam exposed for Stage E. **Stage B also folds the original-A3 work** (WriteSignal IPC variant + drone-side handler arm calling `vdr::project_signal` + main-side emission path; structured-emitter prompt-template module + AgentSdk plumbing replacing the M02 heuristic in `crates/runtime-main/src/sdk/decision_extractor.rs`) — Stage B has scope slack from the 6 already-shipped events to absorb both. **Stage C** lights up the renderer surface — wires already-shipped `PlanNode`/`TaskNode` components (M03.C synthetic-state) to live event variants; authors `ApprovalPanel.tsx` for the plan approval gate; threads the approval flow renderer→main→drone→main→renderer via 3 new Tauri commands (`approve_plan`, `revise_plan`, `abort_plan`) using A2's `Arc<DroneClient>` managed state; Playwright E2E for the round-trip. **Stage D** builds §4a Verify & Rails — adopts existing `verify_started`/`passed`/`failed` + `rail_triggered` event names already in `event.rs:220–248` (codebase NAMES, NOT new `hook_*`); consumes existing `RevertToSnapshot` already in `DroneCommand` at `crates/runtime-core/src/drone.rs:183` with `RevertReason` enum at `:266` (existing handler arm at `crates/runtime-drone/src/command_handler.rs:64` — Stage D extends, does NOT re-add). New work: `hook.v1.json` schema; Hook primitive (HookRef + 7 firing points including new `pre_file_edit`); Rails (hard/soft + JSON-declared with JSONLogic operator allowlist); don't-touch glob matcher; VerifyNode + HookNode wired to the existing live events. The `RevertReason::HookRollback` shape (currently unit variant in code; spec §4a implies `hook_id: String`) is a Stage D design surface decided at execution time. **Stage E** builds §6a HITL — `hitl.v1.json` schema; 9 trigger types, 3 UI variants (Panel/Modal/Toast), notifier plugin interface, 3 built-in notifiers (terminal_bell/desktop via Tauri notification plugin v2/sound); 5 HITL events of which 2 already exist (`HitlRequested`, `HitlResolved` at `event.rs:281, 290` — codebase NAMES `hitl_resolved` NOT `hitl_response`) and 3 are new (`hitl_timeout`, `notifier_dispatched`, `notifier_failed`); failure-escalation flow (`task_escalated` → `on_failure_threshold` → `hitl_requested` → notifiers parallel → 1h default timeout). **Stage F** builds §2a Budget + §1b Recovery — `budget.v1.json` schema; budget enforcer wiring the 4 already-shipped budget events at `event.rs:321–353` (`BudgetWarn`, `BudgetDownshift`, `BudgetSuspended`, `BudgetExceeded` — Stage F wires NOT adds); 3 scopes + 4 threshold actions + downshift_hook + session header bar UI; Recovery primitive (resume rebuilds history per spec §1b WI-14, tool-call-uncertain UI prompt with retry/skip/mark-complete/abort, plan + capability state restoration; MCP reconnect-on-resume seam stubbed for M06). VDR access goes through drone IPC (`crates/runtime-drone/src/vdr.rs::project_signal` at `:50`); `runtime-main` has no `rusqlite` dep — there is no `crates/runtime-main/src/vdr.rs`. **Stage G** is Phase Closeout — gap-analysis entry per CLAUDE.md §20, M04 summary, three-artifact review, `<gotchas_graduation>` audit of A1–F per-stage gotchas.

**Why one PR for the parent milestone (not one PR per stage).** Same logic as M01–M03 — seven stages-as-commits-on-one-branch gives incremental discipline (each stage is reviewable; each stage retrospective surfaces friction early) without the overhead of seven PR reviews for one logical milestone. Consistent with the per-milestone-as-PR pattern in `docs/build-prompts/README.md`. M03 (six stages, ~10h actual) proved the pattern at scale.

**Why seven stages, not eight.** Original eight-stage plan included a separate A3 for vdr WriteSignal IPC + structured-emitter prompt-template wiring scoped as deferred-from-A2. The post-M03.5 codebase audit confirms Stage B has scope slack: 6 of the 11 spec-canonical plan/task events already exist in `event.rs`; 4 of 4 budget events; 4 of 4 verify+rails events; the existing `RevertToSnapshot` drone command + handler arm; the existing `HitlRequested` + `HitlResolved`. A3's work folds naturally into the revised Stage B without exceeding the single-session scope-split threshold. Net: 7 work stages + closeout, no separate A3. Calibrated estimate: ~32–45h. Naive estimate without the audit absorption would be ~39–54h; the audit reduces B/D/E/F net new work by ~10h (no re-authoring of already-shipped events; no re-adding of `RevertToSnapshot`; existing `graphStore.applyEvent` exhaustive-switch arms preserved) while Stage B absorbs ~3h of folded-from-A3 work — net trim ~7h vs naive.

**Why first milestone on v1.3 protocol.** M03.5 authored v1.3 (5 new tags + 3 anti-patterns); M04 is the first parent milestone where the new tags apply. Each stage's `X.5 CLI Prompt` uses `<pre_flight_check>` (Stages A2+ verify prior retro committed), `<schema_drift_check>` (every stage adding or modifying schemas — B, D, E, F), `<fan_out_grep>` (Stage A2 production-wiring DroneClient consumer enumeration; Stage F budget downshift_hook fan-out), `<dependency_audit_check>` (Stage E Tauri notification plugin), and `<runtime_environment>` (all stages pin `os="windows"` consistently — build agent runs on Windows per the established M01–M03.5 pattern).

**Key constraints.**
- §0d Release Scope Matrix — M04 is in scope. Out-of-scope items (gap detection + capability enforcement → M5; MCP basic → M6; registry import → M7; generators → M9) stay deferred. v0.1 STANDARD-mode hardcoded (CLAUDE.md §3 — `examples/aria/framework.json` per-mode overrides are honored at load but only `STANDARD` evaluates at runtime). v0.1 `fresh_context_per_task` only — `one_shot` and `continuous` loop policies in schema but not implemented.
- All M03 hard-gate inheritance — workspace ≥80%, runtime-drone ≥95%, runtime-main ≥95% with documented OS-call exclusions, frontend prettier+eslint+tsc strict + audit, codecov delta gates, gap-analysis append-only, vitest --coverage default, schemas-as-source-of-truth via `cargo xtask regenerate-types` — none relaxed. Plan state machine + capability enforcer are NEW safety primitives gated at ≥95% per CLAUDE.md §5; document exclusion lists per the M01.C / M02.C precedent.
- UI consistency carry-forward (Pre-M01 addendum via M01 entry, M02 SetupPanel/SmokeButton baseline, M03 InspectorPanel/graph) — all M04 panels (ApprovalPanel, GapPanel placeholder if needed, HITL Panel/Modal/Toast variants, BudgetHeaderBar) reuse existing component patterns and visual language; no per-feature re-skinning.
- tauri-driver E2E job stays disabled. Per M03 PR #47 closeout decision (CI job `if: false`'d), M04 does NOT attempt to re-enable. Renderer-Playwright (`e2e` job) remains the M04 E2E proof for new UI surfaces. The wdio v9 ↔ tauri-driver 2.x compat issue (tauri-apps/tauri#10670, #9203) stays a v1.0 / post-MVP carry-forward.
- Cross-stack integration discipline (gotcha #32 + STAGE-PROMPT-PROTOCOL.md §10 cross-stack rule). Every cross-stack code example in M04 stage prompts must be (a) verbatim-quoted from a known-working upstream example with commit SHA in a comment, OR (b) carry an `<execution_warnings>` "verify against upstream reference X before shipping" guard. M04's cross-stack risk surface includes: HITL renderer↔main IPC + Tauri notification plugin (Stage E), drone subprocess + `revert_to_snapshot` (Stages A2 + D), Hook `shell` execution + cross-platform PowerShell wrapper (Stage D), recovery dialog UI + `tool_call_uncertain` round-trip (Stage F), budget downshift_hook tool dispatch (Stage F), `count_tokens` real Anthropic endpoint (Stage A2), `pre_file_edit` rail interception (Stage D).

**License.** Apache 2.0; DCO sign-off (`git commit -s`) on every commit.

**Existing patterns to mirror.**
- M01 archetype: `crates/runtime-drone/src/snapshot.rs` + `db.rs` + `heartbeat.rs` + `command_handler.rs` (TDD-discipline + ≥95% coverage with documented OS-signal exclusions).
- M02 archetype: `crates/runtime-main/src/providers/anthropic_sse.rs` + `tests/anthropic_wiremock.rs` (`*_with` testable seam pattern + wire-format state machine + wiremock harness).
- M02 archetype: `crates/runtime-main/src/sdk/event_pipeline.rs` + `tests/sdk_event_translation.rs` (event-translation pipeline + bounded-stream test fixtures per `docs/gotchas.md` #28).
- M02 + M03.5 archetype: `src-tauri/src/commands.rs::set_api_key_with` + `run_smoke_session_with` (testable seam over Tauri command surface; `*_with` seam + wrapper over OS calls — matches the §13.5 Dev Logging instrumentation pattern).
- M02 architecture: `src/lib/ipc.ts::unwrapCmdError` (renderer-side typed error unwrap per `docs/gotchas.md` #30).
- M03 archetype: `src/components/InspectorPanel.tsx` + `src/components/nodes/*.tsx` (renderer component patterns + handle conventions per spec §3 + ARIA non-modal panel pattern).
- M03 archetype: M03.B–C synthetic-state testing pattern (`docs/gotchas.md` #36) — pass populated state directly into `<NodeComponent>` rather than dispatching events through the store. Stage C inverts this for already-shipped components since now the events DO exist; M04 wiring tests use the event path.
- M03.5 archetype: `STAGE-PROMPT-PROTOCOL.md` v1.3 tags applied per the table in this milestone's Background § "Why first milestone on v1.3 protocol".

**Pre-existing legacy file inventory.**

The renderer + Rust workspace are well-maintained. Carry-forward from M03 close (per gap-analysis.md M03 entry):

| File | Status | Disposition for M04 |
|---|---|---|
| `crates/xtask/src/main.rs` | M01 codegen pipeline; covers framework/skill/agent/tool/common schemas; does NOT cover event.v1.json (M03.A added but hand-maintained Rust types) or error.v1.json (M03.5 added; not yet in codegen) | **EXTEND in Stage A1** to add event.v1.json + error.v1.json codegen (Rust + TS) |
| `crates/runtime-core/src/event.rs` | M03 hand-maintained; should match event.v1.json shape | **REGENERATE in Stage A1** via xtask; validate byte-identical or near; address any drift discovered |
| `src/types/agent_event.ts` | M03.A regenerated from event.v1.json via xtask + json-schema-to-typescript | **REGENERATE in Stage A1** — no shape change expected (event.v1.json unchanged); confirms drift-check pipeline still clean |
| `src/types/error.ts` | DOES NOT EXIST — error.v1.json has no codegen target yet | **CREATE in Stage A1** via xtask extension; replaces hand-maintained `CmdError` interface in `src/lib/ipc.ts` |
| `crates/runtime-core/src/error.rs` | EXISTS — hand-curated `RuntimeError` (workspace internal error per `crates/runtime-core/src/lib.rs:4–5` "Types in `event.rs`, `drone.rs`, and `error.rs` are hand-curated"). Distinct from the IPC `CmdError` at `src-tauri/src/commands.rs:46` | **NAMESPACE CLASH in Stage A1.** A1's codegen target for the IPC `CmdError` is `crates/runtime-core/src/generated/error.rs` (mirrors typify's `generated/` convention). Existing `error.rs::RuntimeError` either renames (e.g., to `runtime_error.rs`) or absorbs into `lib.rs`. Decided at A1 execution time per Stage A1 `<gotchas>` |
| `src/lib/ipc.ts::unwrapCmdError` | M02; hand-maintained `CmdError` discriminated union | **REFACTOR in Stage A2** to import the generated `CmdError` type from `src/types/error.ts`; preserve unwrap semantics per gotcha #30 |
| `crates/runtime-drone/tests/integration*.rs` | M03.A current_exe()-derived paths landed | **VERIFY clean in Stage A1** — confirm no remaining `target/debug` literals; if any stragglers exist (Stage A1 of M03 missed some), retrofit |
| `crates/runtime-main/src/drone_ipc/client.rs::await_event` | M02; timeout path lacks `tokio::time::pause()` coverage | **ADD COVERAGE in Stage A1** — closes M03 carry-forward; archetype: `connection.rs::backoff_grows_exponentially_between_attempts` |
| `src-tauri/src/lib.rs` | DOES NOT EXIST — current Tauri shell is `src-tauri/src/main.rs` only (orchestration directly in `main()`); `lib.rs` was never authored | **NEW SIBLING MODULE in Stage A2.** A2 adds `src-tauri/src/drone_lifecycle.rs` (sibling of `main.rs`, NOT under a new `lib.rs`); spawn + `Arc<DroneClient>` registration + graceful-shutdown logic invoked from `main.rs::main()` |
| `src-tauri/src/commands.rs::query_session_db` + `replay_session` + `run_smoke_session` | M02/M03.E; 3 callsites at `src-tauri/src/commands.rs:166, 200, 247` all noop'd via `DroneClient::noop()` | **REFACTOR in Stage A2** — replace 3 noop callsites with real drone IPC dispatch via Tauri managed state (`tauri::State<'_, Arc<DroneClient>>`); SQL inspector + replay-from-signals + smoke session become end-to-end functional |
| `crates/runtime-main/src/sdk/decision_extractor.rs` | M02 heuristic line-by-line extractor | **REPLACE in Stage A2** with structured emitter — prompt template injects delimited block, SDK parses directly |
| `crates/runtime-main/src/providers/anthropic.rs::count_tokens` | M02 chars/4 approximation | **REPLACE in Stage A2** with real `POST /v1/messages/count_tokens` endpoint call (per spec §2c.3 added M03.5) |
| `crates/runtime-main/src/sdk/event_pipeline.rs::WriteSignal` | M02 + M03 — writes signal but does not project to VDR | **WIRE in Stage A2** — call `vdr::project_signal(conn, signal_id)` after each insert (per gap-analysis M03 entry 🟡) |
| `.github/workflows/ci.yml::e2e-tauri-driver` | M03 PR #47 disabled with `if: false` | **PRESERVE disabled** in M04 — defer re-enable to v1.0 / post-MVP per Key constraints |
| `examples/aria/framework.json` | Authored M01; references plan/task/verify/hitl/budget primitives that did not exist before M04 | **VERIFY loadable in Stage F** acceptance test — strip to v0.1-compatible (STANDARD mode hardcoded, no MCP, no generators referenced) per MVP §M4 |

No legacy from earlier milestones beyond the M03/M03.5 + M02 trees inventoried above.

---

## Document Structure

| Stage | Status | Summary | Estimated effort |
|---|---|---|---|
| **A1** | ⏳ NEXT | Build hygiene — extend `crates/xtask/src/main.rs:45` codegen list (currently 5 schemas) to include `event.v1.json` + `error.v1.json`; regenerate types into `crates/runtime-core/src/generated/`; resolve namespace clash with existing hand-curated `error.rs::RuntimeError` (decision at execution time); retrofit `await_event` `tokio::time::pause()` coverage; verify drone integration tests' `current_exe()`-derived paths clean | ~2.5h |
| **A2** | ⏳ | Production wiring — drone subprocess lifecycle at Tauri startup (`Arc<DroneClient>` Tauri-managed-state via new `src-tauri/src/drone_lifecycle.rs` sibling module + spawn from existing `main.rs::main()` since `lib.rs` does not exist); replace 3 `DroneClient::noop()` callsites at `src-tauri/src/commands.rs:166, 200, 247`; real `count_tokens` against `/v1/messages/count_tokens` (wiremock-tested); refactor hand-maintained `CmdError` to re-export A1's generated type; `unwrapCmdError` consumes generated `CmdError`; resolve long-lived `events()` reconnect behavior under integration test (closes M02 carry-forward 🟡 item) | ~4.5h |
| **B** | ⏳ | §3a Plan & Task primitive — author `plan.v1.json` + `task.v1.json` schemas; xtask codegen extension (over A1 archetype); author 5 missing spec event variants (`plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped`); wire 6 existing spec-canonical variants into the new state machine (NOT re-author); document disposition of 2 codebase extras (`PlanRejected`, `TaskRolledBack`) at execution time; plan state machine (safety primitive ≥95%); `fresh_context_per_task` loop policy; failure escalation; SQLite `plans` + `tasks` tables (first migration creates `crates/runtime-drone/migrations/`); approval-gate seam exposed for Stage E. **Folds original-A3 work:** WriteSignal IPC command + drone-side `vdr::project_signal` handler + main-side emission path; structured-emitter prompt-template module + AgentSdk plumbing replacing M02 heuristic in `decision_extractor.rs` | ~5–7h |
| **C** | ⏳ | §3a Plan UI + ApprovalPanel + graph wiring — wire already-shipped `PlanNode` + `TaskNode` (M03.C synthetic) to live event variants; `ApprovalPanel.tsx` renderer + approval-gate flow (renderer→main→drone→main→renderer); plan abort + revise flows via 3 new Tauri commands (`approve_plan`, `revise_plan`, `abort_plan`) using A2's `Arc<DroneClient>`; Playwright E2E. Adds `<pre_flight_check>` for `Arc<DroneClient>` from A2 | ~3–5h |
| **D** | ⏳ | §4a Verify & Rails — `hook.v1.json` schema (HookRef + HookCategory + Hook); Hook primitive with 7 firing points (existing 6 + new `pre_file_edit`); Rails primitive (hard/soft + JSON-declared with JSONLogic operator allowlist); don't-touch glob matcher; VerifyNode + HookNode wired to existing live events. **Audit-grounded:** event names are `verify_started`/`verify_passed`/`verify_failed` + `rail_triggered` already in `event.rs:220–248` (codebase NAMES, NOT new `hook_*`); `RevertToSnapshot` already in `DroneCommand` at `drone.rs:183` with `RevertReason` enum at `:266` and existing handler arm at `command_handler.rs:64` — Stage D extends, does NOT re-add. **Design surface:** `RevertReason::HookRollback` is a unit variant in code; spec §4a implies `hook_id: String` field — Stage D decides at execution time | ~5–7h |
| **E** | ⏳ | §6a HITL — `hitl.v1.json` schema (9 trigger types + 3 UI variants + notifier plugin interface); 3 built-in notifiers (terminal_bell/desktop via Tauri notification plugin v2/sound); 5 HITL events of which 2 already exist (`HitlRequested`, `HitlResolved` at `event.rs:281, 290` — codebase NAMES `hitl_resolved` NOT `hitl_response`) and 3 new (`hitl_timeout`, `notifier_dispatched`, `notifier_failed`); failure-escalation flow `task_escalated` → `on_failure_threshold` → `hitl_requested` → notifiers parallel → 1h timeout; HITL Panel + Modal + Toast renderer surfaces. Adds `<pre_flight_check>` for `Arc<DroneClient>` | ~5–7h |
| **F** | ⏳ | §2a Budget + §1b Recovery — `budget.v1.json` schema (3 scopes + 4 actions + downshift_hook); **wire enforcer to emit 4 already-shipped budget events** at `event.rs:321–353` (`BudgetWarn`, `BudgetDownshift`, `BudgetSuspended`, `BudgetExceeded` — Stage F WIRES not adds); session header bar UI; Recovery (resume rebuilds history per spec §1b WI-14; tool-call-uncertain UI prompt with retry/skip/mark-complete/abort; plan + capability state restoration; MCP reconnect-on-resume seam stubbed for M06). **Audit-grounded:** vdr access via drone IPC (vdr lives in `crates/runtime-drone/src/vdr.rs::project_signal` at `:50`); `runtime-main` has no `rusqlite` dep — there is no `runtime-main/src/vdr.rs`. Adds `<pre_flight_check>` for `Arc<DroneClient>` | ~4–6h |
| **G** | ⏳ | Phase Closeout — gap-analysis entry per CLAUDE.md §20 (cumulative product↔spec audit including M04 + cumulative review); `<gotchas_graduation>` v1.2 closeout subsection auditing all per-stage `<gotchas>` from A1–F (kept / graduated / resolved / expired); M03 + M03.5 carry-forward final disposition; `M04-summary.md` aggregating across stages; three-artifact review (CLAUDE.md §20) | ~2–3h |

Total estimate: ~32–45 hours for A1–G (calibrated). Human direction: ~10–12 hours across seven approval gates + one PR review.

**Estimation calibration (post-M03.5 codebase audit).** Naive estimate from spec scope alone would be ~39–54h; the audit reduces ~10h of B/D/E/F work that's already-shipped (no re-authoring of the 6 existing plan/task events; no re-authoring of 4 verify+rails events + `RevertToSnapshot`; no re-authoring of `HitlRequested`/`HitlResolved`; no re-authoring of 4 budget events; existing `graphStore.applyEvent` exhaustive-switch arms preserved); Stage B absorbs ~3h of folded-from-A3 work. Net calibrated: ~32–45h. Calibration ratios: M01 ran 0.3× of estimate; M02 0.7×; M03 0.32×; M03.5 0.14× (doc-only). Stages A1–F likely track 0.30×–0.50× (code-shipping with cross-stack glue per gotchas #29, #32, #41); Stage G ~0.20× (doc-only closeout per M03.F precedent). Realistic actual at calibrated ratios: ~10–18h.

---

## Implementation Workflow

Each stage runs through this exact cycle:

```
1. /clear                     — fresh context (only between stages)
2. Paste CLI Prompt below     — XML <work_stage_prompt> or <closeout_stage_prompt>
                                pasted into a fresh Claude Code session
3. WEBCHECK pass              — verify prompt's claims about API shapes /
                                version pins / best practices against the
                                URLs in the stage's WEBCHECK header before
                                writing code (per CLAUDE.md §12 web-first +
                                STAGE-PROMPT-PROTOCOL.md v1.2)
4. Pre-flight checks          — Stages A2+ run <pre_flight_check> verifications
                                (branch correct, prior retro committed, env
                                vars set) BEFORE any code per v1.3 protocol
5. Read prior stage retros    — Stage B+ reads M04.<prev>-retrospective.md
                                [END] Decisions section; applies decisions
                                BEFORE code
6. Schema drift check         — Stages adding/editing schemas/*.v1.json run
                                cargo xtask regenerate-types --check before
                                implementation per v1.3 <schema_drift_check>
7. Write failing tests first  — per CLAUDE.md §5 TDD discipline
8. cargo test --workspace +   — confirm new tests fail before any production
   npm run test                 code (red phase)
9. implement                  — Claude makes production changes
10. cargo test --workspace +  — all tests green
    npm run test
11. cargo clippy + fmt + audit — zero warnings
    + npm run lint + tsc        + frontend gates (run prettier --write +
                                  cargo fmt --all FIRST per gotcha #34)
12. cargo llvm-cov + npm test  — coverage thresholds met (workspace ≥80%,
    -- --coverage                runtime-drone ≥95%, runtime-main ≥95%,
                                src/ ≥80%; M04 plan state machine + capability
                                enforcer ≥95% per safety-primitive gate)
13. fill in retrospective      — docs/build-prompts/retrospectives/M04.<X>-retrospective.md
                                including the [END] Coverage holdouts and
                                [END] Decisions for next stage
14. commit (no push)           — exact commit message provided per stage X.6
15. user reviews + approves    — Claude does NOT push without approval
16. push (final stage only)    — Stage G push gates the M04 PR draft per
                                CLAUDE.md §20
```

**Rule:** If a new test passes before implementation, the test is wrong — stop and fix the test (CLAUDE.md §5 hard-fails on missing exports).

**Rule:** Stages are sequential. Stage B does not start until Stage A2's commit is on the feature branch (locally is sufficient; push is optional). Stage A2 does not start until Stage A1's commit is on the feature branch. The parent-milestone PR pushes only at the end of Stage G.

**Rule per CLAUDE.md §8:** Claude does not commit without user approval. After tests pass + retrospective filled, Claude surfaces the diff stat + retrospective + draft commit message. User approves; Claude commits.

**Rule per CLAUDE.md §19:** Each stage produces a retrospective; the final stage also produces an `M04-summary.md` aggregating across stages.

**Rule per CLAUDE.md §20:** Stage G's gap-analysis entry is **immutable** once committed. Future milestones report status updates via their Carry-forward sections; never edit M04's entry after merge. M01–M03 entries also remain immutable; M04 carry-forward absorption goes in the new M04 entry, not as edits to prior entries.

**Rule per spec §13.5 Dev Logging:** Every Rust binary modified in M04 keeps `tracing_subscriber::fmt::init()` at `main()`. Every Tauri command added in M04 logs entry / error / success. Every renderer `try { await invoke(...) } catch (e) { ... }` block logs `console.error('<context> error:', e)` before `unwrapCmdError(e)` dispatch.

**Rule per gotcha #32 + STAGE-PROMPT-PROTOCOL.md §10:** Cross-stack code examples in stage prompts (Tauri ↔ wdio config, OS-keychain feature flags, MCP JSON-RPC framing, OAuth flows, ESLint flat-config shape, etc.) must be (a) quoted verbatim from a working upstream example with commit SHA referenced in a comment above the example, OR (b) carry an explicit "verify against upstream reference X before shipping" instruction inside `<execution_warnings>`. M04's cross-stack surface is high — every stage's prompt review must verify this discipline before pasting to a fresh session.

---

<!-- ============================================================ -->
<!-- STAGE A1 — Build hygiene + xtask codegen + coverage retrofits  -->
<!-- ============================================================ -->

## Stage A1 — Build hygiene + xtask codegen extensions + coverage retrofits

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens. Per CLAUDE.md §12 web-first rule. If any claim below is stale, update this section in `M04-plan-verify-hitl-budget.md` BEFORE pasting Stage A1's CLI prompt — never let a fresh session work from a stale snapshot.

- <https://docs.rs/typify/latest/typify/> — confirm typify (Rust JSON-Schema → Rust types codegen) is the M03.A choice and is current; review API surface for adding new schemas to the codegen list
- <https://github.com/bcherny/json-schema-to-typescript> — confirm json-schema-to-typescript (Node CLI for JSON-Schema → TS types) is current; M03.A wired the Rust-side caller into xtask via `std::process::Command`; review for any breaking changes
- <https://docs.rs/tokio/latest/tokio/time/fn.pause.html> — confirm `tokio::time::pause()` API is unchanged (used in await_event timeout test); M01.C archetype `connection.rs::backoff_grows_exponentially_between_attempts` is the pattern reference
- <https://docs.rs/cargo-llvm-cov/latest/cargo_llvm_cov/> — confirm coverage tool API is unchanged from M03 baseline; check for any new flags relevant to per-test coverage attribution

### A1.1 Problem Statement

Three M03 carry-forward 🟡 build-hygiene items must close before Stage A2 (production wiring) starts:

1. **xtask codegen does not cover `event.v1.json` or `error.v1.json`.** M03.A added `event.v1.json` to the schemas tree but the codegen pipeline (`crates/xtask/src/main.rs`) still only handles framework/skill/agent/tool/common schemas — the Rust types in `crates/runtime-core/src/event.rs` are hand-maintained, and the TS types in `src/types/agent_event.ts` are generated via a one-off Node CLI invocation outside the xtask. M03.5 added `error.v1.json` (the new wire-format schema for `CmdError`) without any generated targets at all. Stage A1 extends xtask to codegen both schemas to both Rust and TS, regenerates the types, and validates the drift check is clean. Closes gap-analysis M03 🟡 entry "Extend xtask Rust typify list to include event.v1.json".

2. **`await_event` timeout path lacks `tokio::time::pause()` coverage.** M03 closed the `client.rs` 100% → 94.00% regression at Stage A but left the timeout-specific path untested under simulated time. Adding the `tokio::time::pause()` test brings coverage back to 100% and validates the timeout invariant deterministically. Closes gap-analysis M03 🟡 entry "tokio::time::pause() coverage for await_event timeout path".

3. **Drone integration tests verified clean.** M03.A retrofitted `crates/runtime-drone/tests/integration*.rs` to derive paths via `std::env::current_exe()` per gotcha #22. Stage A1 verifies no `target/debug` literals remain (drift check); if any stragglers exist, retrofit them.

Doc-only stage: `CHANGELOG.md` `[Unreleased]` notes the hygiene closures.

**Success criterion:** `cargo xtask regenerate-types --check` returns zero diff after a clean regen; `cargo llvm-cov --package runtime-main` reports `client.rs` at 100%; `grep -rn 'target/debug' crates/runtime-drone/tests/` returns zero matches; all gates pass.

**New artifacts:**
- `crates/runtime-core/src/generated/error.rs` (new; generated from `error.v1.json` via xtask — typify output. Goes under `generated/` per existing convention; the existing top-level `error.rs::RuntimeError` is a separate hand-curated module the namespace strategy resolves at execution time, see A1.3)
- `crates/runtime-core/src/generated/event.rs` (new; generated from `event.v1.json` via xtask — typify output. Replaces the hand-curated top-level `event.rs::AgentEvent` only IF the typify output matches the hand-curated shape. If drift is non-trivial — likely given typify's `oneOf`-derivation can produce wrappers the hand-curated form lacks — A1 commits the generated file under `generated/` and leaves the top-level `event.rs` in place; later stages migrate consumers. Decision at A1 execution time.)
- `src/types/error.ts` (new; generated from `error.v1.json` via xtask — json-schema-to-typescript output)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (extend the existing 5-schema codegen list at line 45 — currently `["common", "framework", "skill", "tool", "agent"]` — to include `event` + `error`; wire Rust typify + TS json-schema-to-typescript outputs)
- `crates/runtime-core/src/lib.rs` (export the new `generated::error` + `generated::event` paths; resolve the namespace strategy with the existing top-level `error.rs::RuntimeError` and `event.rs::AgentEvent` — strategies: (a) rename existing `error.rs` to `runtime_error.rs` and re-export `CmdError` from `generated::error`; (b) keep existing `error.rs` and put generated `CmdError` under `generated::error::CmdError` only; (c) absorb `RuntimeError` into `lib.rs` directly. Decision at execution time per Stage A1 `<gotchas>`)
- `crates/runtime-core/src/error.rs` (potentially renamed to `runtime_error.rs` per the namespace decision above — preserves existing `RuntimeError` shape; the file's contents do NOT change beyond the rename)
- `crates/runtime-core/src/event.rs` (potentially renamed to `event_legacy.rs` or absorbed into `lib.rs` if A1 decides to fully migrate to generated; otherwise stays in place — decision at execution time)
- `src/types/agent_event.ts` (regenerated via the existing M03.A pipeline path; verify drift-check clean)
- `crates/runtime-main/src/drone_ipc/client.rs` (add `tokio::time::pause()`-driven timeout test for `await_event` path; no production-code changes)
- `CHANGELOG.md` (`[Unreleased]` notes the M04 Stage A1 hygiene closures)

### A1.2 Files to Change

| File | Change |
|---|---|
| `crates/xtask/src/main.rs` | **Edited** — extend the 5-schema codegen list at `:45` with `"event"` + `"error"`; wire Rust typify + TS json-schema-to-typescript outputs (mirror existing pattern) |
| `crates/runtime-core/src/generated/error.rs` | **New** — generated from `error.v1.json` via xtask (5-variant tagged enum: `SetupRequired`, `Provider`, `Drone`, `KeyStore`, `Internal`) |
| `crates/runtime-core/src/generated/event.rs` | **New** — generated from `event.v1.json` via xtask. Likely to drift from existing hand-curated `crates/runtime-core/src/event.rs` (oneOf-derivation can produce wrappers); namespace strategy decided at A1 execution time per A1.3 |
| `src/types/error.ts` | **New** — generated from `error.v1.json` via xtask (5-variant discriminated union) |
| `crates/runtime-core/src/lib.rs` | **Edited** — declare `pub mod generated;` (it currently exists; its mod.rs adds re-exports for the new `error` + `event` submodules); resolve namespace strategy with existing top-level `error.rs::RuntimeError` + `event.rs::AgentEvent` — see A1.3 strategies |
| `crates/runtime-core/src/error.rs` | **Possibly renamed** to `runtime_error.rs` per A1 namespace decision; existing `RuntimeError` content unchanged otherwise |
| `crates/runtime-core/src/event.rs` | **Decision at execution time** — possibly stays as hand-curated source-of-truth (typify output drifts from this and is committed under `generated/` for forward use), or possibly removed in favor of the generated version (consumer migration becomes A2 work) |
| `src/types/agent_event.ts` | **Regenerated** via existing M03.A pipeline; drift-check clean |
| `crates/runtime-main/src/drone_ipc/client.rs` | **Edited (test only)** — add `tokio::time::pause()`-driven timeout test for `await_event` path |
| `crates/runtime-drone/tests/integration*.rs` | **Verified clean** — `grep -rn 'target/debug' crates/runtime-drone/tests/` returns zero matches per gotcha #22; retrofit if any remain |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes M04 Stage A1 hygiene closures + namespace decision outcome |

### A1.3 Detailed Changes

#### `crates/xtask/src/main.rs` — extend codegen list (audit-grounded)

The existing schemas list at `crates/xtask/src/main.rs:45` is `let schemas = ["common", "framework", "skill", "tool", "agent"];` (5 entries). Stage A1 extends it to 7 entries:

```rust
let schemas = ["common", "framework", "skill", "tool", "agent", "event", "error"];
```

Verify the surrounding loop produces output paths under `crates/runtime-core/src/generated/{name}.rs` (the existing convention; mod.rs already re-exports `agent`, `common`, `framework`, `skill`, `tool` — A1 adds `event` + `error` to that mod.rs). The TS pipeline downstream (existing M03.A path) emits to `src/types/agent_event.ts` for `event` and (new) `src/types/error.ts` for `error`.

The `--check` flag (drift detection) compares regenerated output to committed files via byte-diff; non-zero exit if any diff.

#### `crates/runtime-core/src/generated/error.rs` — new generated file (audit-grounded path)

**Path correction from previous draft.** Generated output goes under `crates/runtime-core/src/generated/error.rs` (mirrors the existing `generated/{agent,common,framework,skill,tool}.rs` pattern documented in `crates/runtime-core/src/generated/mod.rs`), NOT at top-level. The top-level `crates/runtime-core/src/error.rs` already exists as hand-curated `RuntimeError` (workspace internal error per `lib.rs:4–5`); putting `CmdError` at the top-level would clobber it.

Generated from `schemas/error.v1.json` (5-variant `oneOf`). The output is a Rust enum with `serde(tag = "type", rename_all = "snake_case")` matching the schema's encoding declared in the existing `src-tauri/src/commands.rs::CmdError:46` (which becomes a re-export of the generated type after Stage A2 wires it).

Expected enum shape (typify-generated; do not hand-edit):

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CmdError {
    SetupRequired,
    Provider { message: String },
    Drone { message: String },
    KeyStore { message: String },
    Internal { message: String },
}
```

If typify produces a different shape (e.g., struct variants with `Default`, or an extra `oneOf`-derived wrapper), accept the generated output and update consumers in Stage A2 — do not hand-edit the generated file.

#### Namespace clash decision — top-level `error.rs::RuntimeError` vs generated `generated::error::CmdError`

A1's most subtle decision. The existing top-level `crates/runtime-core/src/error.rs` holds:

```rust
pub enum RuntimeError { /* hand-curated workspace error variants */ }
```

A1's codegen lands `CmdError` at `crates/runtime-core/src/generated/error.rs`. Both files coexist by path. The decision is how `lib.rs` exposes the names so consumers (runtime-main, runtime-drone, src-tauri) can use both unambiguously. Three strategies:

(a) **Rename existing `error.rs` to `runtime_error.rs` (recommended baseline).** `lib.rs` then declares `pub mod runtime_error;` and `pub use runtime_error::RuntimeError;` for the workspace internal error. The generated path stays as `pub mod generated;` (already declared) which exposes `generated::error::CmdError`. Consumers of `RuntimeError` update to `runtime_error::RuntimeError` or use the re-export at crate root. Cleanest separation; mirrors the typify `generated/` convention. PR diff is one rename + a few `use` updates.

(b) **Keep existing `error.rs`, qualify generated `CmdError` only.** `lib.rs` keeps the existing `pub mod error;` for `RuntimeError`; `CmdError` is accessed only as `runtime_core::generated::error::CmdError` (long form). Consumers must use the qualified path. No renames; longer use-paths in callers. Simpler PR diff; uglier callsites.

(c) **Absorb `RuntimeError` into `lib.rs` directly.** `lib.rs` inlines `RuntimeError` (single enum, ~30 lines); deletes the existing `error.rs` file. Generated `CmdError` accessed as `generated::error::CmdError` or re-exported. Most disruptive; not recommended unless `RuntimeError` is genuinely tiny.

A1 picks one strategy at execution time, documents the choice in the retrospective `[END] Decisions for Stage A2` section, and applies it consistently. The Stage A1 prompt's `<gotchas>` surfaces this decision; the `<approval_surface>` includes the strategy choice as a review item.

#### `crates/runtime-core/src/generated/event.rs` — new generated file (likely drift-prone)

Generated from `schemas/event.v1.json` via typify. The current top-level `crates/runtime-core/src/event.rs` is hand-curated — 39 variants with extensive doc-comment cross-references to spec sections. Typify's `oneOf`-derivation may produce a structurally different shape (e.g., a wrapper enum + variant types as separate structs). If drift is non-trivial, A1 commits the generated file under `generated/` (per the codegen pipeline) but does NOT delete the top-level hand-curated `event.rs` in the same PR — the consumer migration is Stage A2 work, not A1's. If drift is trivial (byte-near-identical), A1 may choose to delete the top-level file in the same commit. Decision at execution time; surface in retrospective.

#### `src/types/error.ts` — new generated file

Generated from `schemas/error.v1.json` via json-schema-to-typescript. Expected output: a `CmdError` discriminated union matching the existing `src/lib/ipc.ts::CmdError` interface (which becomes a re-export after Stage A2 refactor). The generator may produce an `export type CmdError = { type: 'setup_required' } | { type: 'provider'; message: string } | ...` form or an interface-based form; accept whatever json-schema-to-typescript produces and update consumers in Stage A2.

#### `crates/runtime-core/src/lib.rs` — namespace updates

Currently: `pub mod drone; pub mod error; pub mod event; pub mod generated; pub mod signal;` plus re-exports `pub use error::RuntimeError; pub use event::{AgentEvent, ToolSource};`. Stage A1 updates per the chosen namespace strategy (typically option (a)): rename the existing `error.rs` declaration to `runtime_error`, leave `generated` as-is (its `mod.rs` adds `pub mod error; pub mod event;` declarations), and update the re-exports accordingly. Verify `pub mod generated;` still works after `mod.rs` adds two new declarations.

#### `crates/runtime-main/src/drone_ipc/client.rs` — `tokio::time::pause()` timeout test

Add a unit test inside the existing `#[cfg(test)] mod tests` block (or create one if absent). Pattern archetype: `crates/runtime-main/src/drone_ipc/connection.rs::backoff_grows_exponentially_between_attempts`.

Test body (sketch — adapt to the actual `await_event` signature):

```rust
#[tokio::test(start_paused = true)]
async fn await_event_returns_timeout_after_configured_duration() {
    // Given: a client with no events flowing
    let client = Client::with_test_seam(/* mock channel that never produces */);

    // When: we await an event with a 5s timeout
    let timeout = std::time::Duration::from_secs(5);
    let result = tokio::time::timeout(timeout, client.await_event(/* args */)).await;

    // Then: the timeout fires deterministically (paused-time clock advances)
    assert!(result.is_err(), "expected timeout, got {:?}", result);
}
```

Prefer `#[tokio::test(start_paused = true)]` over manual `tokio::time::pause()` calls — cleaner and matches the M01.C archetype.

If `await_event` already has tests using real-time waits, replace those with paused-time variants in the same change.

#### `crates/runtime-drone/tests/integration*.rs` — verify clean

Run: `grep -rn 'target/debug\|target/release' crates/runtime-drone/tests/ | grep -v current_exe`. Expected: zero matches. If matches surface, retrofit the matched lines to use `current_exe()`-derived paths per the M02.D + M03.A archetype at `crates/runtime-main/tests/drone_ipc_loopback.rs::drone_binary`.

#### `CHANGELOG.md` — `[Unreleased]` notes

Append to the existing `[Unreleased]` section a new bullet under an existing `### Added` subsection (or create a `### Build` subsection if more appropriate):

```markdown
- M04 Stage A1: extended xtask codegen to event.v1.json + error.v1.json (Rust typify + TS json-schema-to-typescript). New generated files: crates/runtime-core/src/error.rs + src/types/error.ts. Closes M03 carry-forward 🟡 build-hygiene items: hand-maintained event.rs replaced by codegen output; await_event timeout path covered via tokio::time::pause(); drone integration tests verified clean of target/debug literals.
```

### A1.4 Tests

#### Pedantic-pass preflight (no new modules introduced)

Stage A1 introduces `crates/runtime-core/src/error.rs` as a new generated module. Generated code is exempt from the pedantic preflight (covered by `--ignore-filename-regex "generated"` in the workspace coverage gate). Apply the preflight to any non-generated edits — `client.rs` test additions and xtask extension.

#### Test files

Stage A1 adds one test (the `await_event` timeout test) and verifies regen drift via `xtask regenerate-types --check`. No new test files; the test lands inside the existing `#[cfg(test)] mod tests` block in `client.rs`.

#### Coverage target

- `crates/runtime-main/src/drone_ipc/client.rs` returns to 100% (closes M03 holdout)
- workspace ≥80% maintained
- `runtime-main` ≥95% safety-primitive gate maintained (existing exclusions for `providers/anthropic.rs` + `drone_ipc/connection.rs`)
- Generated files (`crates/runtime-core/src/error.rs`, `crates/runtime-core/src/event.rs` regen) excluded via existing `--ignore-filename-regex "generated"` (verify the regex covers these — if not, extend per M01.C precedent)

### A1.5 CLI Prompt

Paste the XML block below into a fresh Claude Code session as the opening message. Per `STAGE-PROMPT-PROTOCOL.md` v1.3 — section-name refs, mandatory `<execution_steps>`, strict reference-first, plus v1.3 `<schema_drift_check>` and `<runtime_environment>` tags.

```xml
<work_stage_prompt id="M04.A1">
  <context>
    Stage A1 of M04 (Plan + Verify + HITL + Budget). First stage of M04; codebase is at end-of-M03.5. Build hygiene + xtask codegen extensions + coverage retrofits. Closes three M03 carry-forward 🟡 build-hygiene items so Stages A2–G focus on production wiring + new primitive surface. The xtask codegen list at `crates/xtask/src/main.rs:45` currently contains 5 schemas (`["common", "framework", "skill", "tool", "agent"]`) — A1 extends to 7 (adds `event` + `error`). Output goes under `crates/runtime-core/src/generated/` (mirrors existing typify convention). The existing top-level `crates/runtime-core/src/error.rs` (hand-curated `RuntimeError`) is a NAMESPACE CLASH — A1 resolves it (rename to `runtime_error.rs`, qualify by path, or absorb into `lib.rs`; decision documented in retrospective). The existing top-level `event.rs` may diverge from typify output; A1 commits the generated file and decides whether to keep or remove the hand-curated version. Stage A2 does not start until Stage A1's commit is on the milestone branch `claude/m04-plan-verify-hitl-budget`. First milestone authored on the v1.3 XML stage-prompt protocol — uses `<schema_drift_check>` + `<runtime_environment>` tags below.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Background, Document Structure, Implementation Workflow, Pre-existing legacy file inventory, Stage A1 sections A1.1–A1.4)</file>
    <file>agent-runtime-spec.md §0–§0d, §1d, §2c, §13.5</file>
    <file>docs/MVP-v0.1.md §M4</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #14 snake_case discipline; #41 grep-verify-claims — the rule that produced this audit-grounded prompt)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="M03.A xtask codegen archetype to extend; line 45 has the 5-schema list">crates/xtask/src/main.rs</file>
    <file purpose="existing typify-generated module structure; A1 adds 'error' + 'event' submodules to the existing mod.rs">crates/runtime-core/src/generated/mod.rs</file>
    <file purpose="EXISTING hand-curated RuntimeError — namespace clash with generated CmdError; A1 resolves">crates/runtime-core/src/error.rs</file>
    <file purpose="EXISTING hand-curated AgentEvent (39 variants); typify output may drift; A1 decides keep/remove">crates/runtime-core/src/event.rs</file>
    <file purpose="lib.rs declares the existing pub mod error/event/generated; A1 updates per namespace strategy">crates/runtime-core/src/lib.rs</file>
    <file purpose="schema source for new error type codegen target">schemas/error.v1.json</file>
    <file purpose="hand-maintained CmdError at line 46 that A1's generated type will replace in Stage A2">src-tauri/src/commands.rs</file>
    <file purpose="tokio::time::pause() archetype for await_event timeout test">crates/runtime-main/src/drone_ipc/connection.rs</file>
    <file purpose="current_exe() archetype for any drone integration test retrofits">crates/runtime-main/tests/drone_ipc_loopback.rs</file>
  </read_reference>

  <read_prior_milestones>
    <gap_analysis_carry_forward milestone="M03"/>
    <gap_analysis_carry_forward milestone="M03.5"/>
    <milestone_summary milestone="M03" section="Decisions to apply before the next parent milestone"/>
  </read_prior_milestones>

  <deliverable ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="A1.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="A1.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="Key constraints"/>

  <gates milestone="M04"/>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="audit_baseline_xtask">grep -q '\["common", "framework", "skill", "tool", "agent"\]' crates/xtask/src/main.rs (confirms A1 starts from the verified end-of-M03.5 baseline; if the grep fails the codebase has drifted between authoring and execution and A1's plan needs revisiting)</check>
    <check name="audit_baseline_error_rs">grep -q "pub enum RuntimeError" crates/runtime-core/src/error.rs (confirms the existing hand-curated error.rs is unchanged from audit baseline; namespace strategy applies as written)</check>
    <check name="audit_baseline_generated_dir">Test-Path crates/runtime-core/src/generated AND Test-Path crates/runtime-core/src/generated/mod.rs (confirms the typify generated/ convention exists; A1 extends mod.rs)</check>
    <check name="schemas_present">Test-Path schemas/event.v1.json AND Test-Path schemas/error.v1.json</check>
  </pre_flight_check>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <runtime_environment os="windows" note="Build agent runs on Windows 11 per the established M01-M03.5 pattern; Select-String is the assumed grep equivalent throughout the prompt; Test-Path replaces test -f. The PowerShell-via-bash discipline from M03.5.A retro applies — variable expansion in heredoc-wrapped scripts must be safe under bash's expansion rules."/>

  <gotchas>
    <trap>NAMESPACE CLASH: existing top-level crates/runtime-core/src/error.rs holds RuntimeError (hand-curated workspace error per lib.rs:4-5); generated CmdError lands at crates/runtime-core/src/generated/error.rs (per existing generated/ convention). A1 picks the namespace strategy: (a) rename top-level to runtime_error.rs (recommended baseline); (b) qualify generated by full path; (c) absorb RuntimeError into lib.rs. Decision documented in retrospective [END] Decisions; PR diff reflects choice.</trap>
    <trap>EVENT.RS DRIFT LIKELY: the existing top-level event.rs (39 hand-curated variants with extensive doc comments) will likely diverge from typify oneOf-derived output. A1 commits the generated version under generated/event.rs but does NOT delete the hand-curated top-level file in the same PR if drift is non-trivial — consumer migration is Stage A2 work. If drift is trivial, A1 may delete the top-level in the same commit. Surface in retrospective.</trap>
    <trap>Stage A1's job is to close M03 build-hygiene carry-forward + extend xtask codegen, not to start Stage A2's production wiring — resist scope creep into drone subprocess spawning even if the regenerated types make it tempting. The 3 DroneClient::noop() callsites at src-tauri/src/commands.rs:166, 200, 247 stay noop'd until A2.</trap>
    <trap>typify-generated Rust types may not match the hand-maintained event.rs byte-for-byte — accept the generated output and update consumers in subsequent stages rather than hand-editing the generated file (gotcha #14 snake_case schema discipline applies here)</trap>
    <trap>json-schema-to-typescript may produce a TS shape that differs from the M02 hand-maintained CmdError interface (e.g., interface vs type alias, strict vs loose discriminator) — Stage A2 owns the consumer refactor; A1 only commits the generated output</trap>
    <trap>tokio::time::pause() — prefer #[tokio::test(start_paused = true)] over manual pause() at test start (cleaner pattern; M01.C connection.rs archetype works but the start_paused form is the v1.3 baseline). Document the choice in retro.</trap>
    <trap>Per gotcha #41 (grep-verify-claims): every codebase-state claim in this stage's deliverable section is verified against end-of-M03.5 reality. If the verifications in &lt;pre_flight_check&gt; fail (codebase has drifted between authoring and execution), surface the drift before proceeding — do NOT improvise around it.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT touch src-tauri/src/commands.rs::CmdError in Stage A1 — that's Stage A2's refactor (replace with re-export of generated type). Stage A1 only commits the generated output file.</warning>
    <warning>DO NOT regenerate framework/skill/agent/tool/common schemas — only event.v1.json + error.v1.json get extended codegen. The existing 5 schemas in xtask main.rs:45 stay untouched in their codegen.</warning>
    <warning>DO NOT push between stages — Stage A1 commits locally only. The push happens at end of Stage G per CLAUDE.md §8 + §20.</warning>
    <warning>The cargo xtask regenerate-types --check command must produce zero diff after the regen step — if there's persistent drift between regen passes, the codegen is non-deterministic and needs fixing (sorted fields, normalized whitespace, deterministic comments) before committing.</warning>
    <warning>Stage A1's commit MUST include docs/build-prompts/retrospectives/M04.A1-retrospective.md in the staged files (per M03.5.B retro [END] decision — the Stage A1 retrospective being untracked at commit time is the M03.5 drift this stage explicitly closes via &lt;pre_flight_check&gt; for Stage A2).</warning>
  </execution_warnings>

  <time_box estimate_hours="2.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage A2: NAMESPACE STRATEGY chosen for the error.rs clash (a/b/c per Stage A1's gotchas + A1.3); whether the existing top-level event.rs was deleted or kept alongside generated/event.rs (A2 may need to migrate consumers); typify drift discovered in event.rs regen (was it pre-existing M03.A drift or new from this regen? confirm in diff); whether json-schema-to-typescript output requires Stage A2 consumer refactor (likely yes given M02 hand-maintained CmdError interface predates the schema); whether the await_event timeout test surfaces any other timeout-related bugs in client.rs not covered by existing tests; whether the drone integration test current_exe() retrofit was clean or revealed additional path-derivation issues; outcome of cargo xtask regenerate-types --check post-implement (zero diff or non-zero with reason).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="A1.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD) including the M04.A1-retrospective.md file in staged set</item>
    <item>gate results (each gate, pass/fail, key numbers including the new client.rs coverage 100%)</item>
    <item>schema drift check output — cargo xtask regenerate-types --check exit code + diff if any</item>
    <item>NAMESPACE STRATEGY chosen (a/b/c per A1.3) with one-paragraph rationale</item>
    <item>generated file shape preview — first 30 lines of crates/runtime-core/src/generated/error.rs + first 30 lines of src/types/error.ts so the human can spot-check shape</item>
    <item>event.rs regen disposition — diff between top-level hand-curated event.rs and generated/event.rs; statement of whether the top-level was kept, deleted, or absorbed</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage A2)</item>
    <item>draft commit message from M04-plan-verify-hitl-budget.md A1.6 Commit Message section (filled with session URL)</item>
    <item>explicit statement: "Stage M04.A1 is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### A1.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
chore(workspace): M04 Stage A1 — build hygiene + xtask codegen extensions

Closes M03 carry-forward 🟡 build-hygiene items so Stage A2 (production
wiring) can focus on the real surface. No new feature surface; codegen
extensions + namespace resolution + coverage retrofit + drift verification.

Carry-forward closures:
- crates/xtask/src/main.rs: extended the 5-schema codegen list at line
  45 to 7 entries (adds "event" + "error"); Rust typify outputs to
  crates/runtime-core/src/generated/{event,error}.rs (mirrors existing
  generated/ convention); TS json-schema-to-typescript outputs to
  src/types/{agent_event,error}.ts.
- crates/runtime-core/src/generated/error.rs: new generated CmdError
  (5-variant tagged enum); namespace strategy resolves the clash with
  the existing hand-curated crates/runtime-core/src/error.rs (which
  holds RuntimeError, the workspace internal error). Strategy chosen:
  <NAMESPACE_STRATEGY> (a: rename top-level error.rs to runtime_error.rs
  / b: keep top-level, qualify generated by full path /
  c: absorb RuntimeError into lib.rs). PR diff reflects the choice.
- crates/runtime-core/src/generated/event.rs: new generated AgentEvent.
  Top-level hand-curated event.rs disposition: <KEEP_OR_REMOVE>
  (kept alongside if typify drift is non-trivial; consumer migration
  is Stage A2 work).
- src/types/error.ts: new generated CmdError discriminated union.
- crates/runtime-main/src/drone_ipc/client.rs: tokio::time::pause()-
  driven test for await_event timeout path. Closes 100% → 94% regression
  on client.rs coverage from M03.D retro.
- crates/runtime-drone/tests/integration*.rs: verified clean of
  target/debug literals (per docs/gotchas.md #22; M03.A retrofit
  confirmed durable).

CHANGELOG.md [Unreleased] reflects the closures. Namespace strategy
choice documented in retrospective [END] Decisions section.

Refs: M04-plan-verify-hitl-budget.md §A1, gap-analysis.md M03 entry 🟡
(xtask event.v1.json codegen + await_event coverage), CLAUDE.md §14
(schemas-as-source-of-truth)
Retrospective: docs/build-prompts/retrospectives/M04.A1-retrospective.md

https://claude.ai/code
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE A2 — Production wiring                                   -->
<!-- ============================================================ -->

## Stage A2 — Production wiring (drone subprocess + count_tokens + events() reconnect + CmdError consumption)

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.rs/tokio/latest/tokio/process/struct.Command.html> — confirm `tokio::process::Command` API for drone subprocess spawn is unchanged from M01.C (already in use); review `kill_on_drop` + child stdio handling
- <https://v2.tauri.app/develop/state-management/> — confirm Tauri 2.x managed-state API is unchanged; `Arc<DroneClient>` registered via `app.manage(...)` and accessed in commands via `tauri::State<'_, Arc<DroneClient>>`
- <https://docs.anthropic.com/en/api/messages-count-tokens> — confirm the `POST /v1/messages/count_tokens` endpoint URL + request shape + response shape are current; Stage A2 wires the real call
- <https://docs.rs/reqwest/latest/reqwest/> — `reqwest::Client::post` API is unchanged from M02.C; confirm
- <https://docs.rs/keyring/latest/keyring/> — keyring 3.6 (per gotcha #29 + Cargo.toml workspace pin) — confirm no breaking change relevant to this stage (Stage A2 doesn't touch keyring directly; included for cross-stack discipline)

### A2.1 Problem Statement

Stage A2 wires the production paths M03 deferred via `DroneClient::noop()`, plus closes three M02/M03 carry-forward 🟡 items that block downstream stages. **Two items the previous draft scoped here (vdr WriteSignal IPC wiring + structured-emitter prompt template) are folded into Stage B per the post-M03.5 audit re-staging** — see Background § "Why seven stages, not eight" — because Stage B has scope slack from already-shipped events and the WriteSignal IPC variant naturally belongs alongside Stage B's plan/task schema authoring.

1. **Drone subprocess lifecycle at Tauri startup.** M03.E shipped 3 `DroneClient::noop()` callsites at `src-tauri/src/commands.rs:166, 200, 247` (`run_smoke_session`, `query_session_db`, `replay_session`); Stage A2 spawns the real `runtime-drone` subprocess at app startup, registers `Arc<DroneClient>` as Tauri managed state, and wires graceful shutdown on app exit. **`src-tauri/src/lib.rs` does NOT exist** (audit-confirmed); current Tauri shell is `src-tauri/src/main.rs` directly. A2 adds new sibling module `src-tauri/src/drone_lifecycle.rs` and invokes its `spawn`/`shutdown` from the existing `main.rs::main()`. SQL inspector + replay-from-signals + smoke session become end-to-end functional. Closes gap-analysis M03 🟡 entry "Production drone subprocess wiring at Tauri startup".

2. **Real `count_tokens` Anthropic endpoint.** M02 ships a chars/4 approximation at `crates/runtime-main/src/providers/anthropic.rs:135`. Stage A2 implements the real call to `POST /v1/messages/count_tokens` per spec §2c.3 (added M03.5). Wiremock test covers happy path + error mapping. M04 budget enforcement (Stage F) depends on this. Closes gap-analysis M02 🟡 entry "count_tokens → real /v1/messages/count_tokens endpoint".

3. **Long-lived `events()` reconnect resolution.** Per spec §1d ⚠️ note (updated M03.5 from M03 to M04 carry-forward): does the renderer's long-lived `agent_event` subscription survive a mid-session main↔drone reconnect? Stage A2 establishes the answer through a deliberate integration test (kill drone subprocess mid-session, verify the renderer continues to receive events after reconnect). Test-driven decision: if survival works as-implemented, the ⚠️ note becomes a closed item; if not, document the v0.1 behavior (renderer resubscribes on reconnect via M03's replay_session pattern) and update spec text. Reconnect logic lives at `crates/runtime-main/src/drone_ipc/connection.rs` (audit-confirmed; `event_translation.rs` is a phantom path — actual translation pipeline is `crates/runtime-main/src/sdk/event_pipeline.rs`). Closes gap-analysis M02 🟡 entry "Long-lived events() subscription survives reconnect".

4. **`unwrapCmdError` consumes generated types + Tauri command surface refactor.** Stage A1 generates `crates/runtime-core/src/generated/error.rs` (per the typify `generated/` convention; the existing top-level `error.rs::RuntimeError` is unrelated workspace internal error) + `src/types/error.ts`. Stage A2 refactors `src/lib/ipc.ts::unwrapCmdError` to import the generated `CmdError` type from `src/types/error.ts` rather than the M02 hand-maintained interface. The hand-maintained `CmdError` enum at `src-tauri/src/commands.rs:46` becomes `pub use runtime_core::generated::error::CmdError` (path depends on A1's namespace strategy outcome — see A1 retro). Preserves unwrap semantics per gotcha #30 (renderer-side typed error unwrap). Closes the consumer-refactor portion of A1's `error.rs` codegen.

**Items folded into Stage B per audit re-staging (NOT in A2):**
- VDR projector wiring at signal-write call-site (was previous A2.1 #2). Architecture: main emits a `WriteSignal` IPC variant → drone runs `vdr::project_signal` internally (`crates/runtime-drone/src/vdr.rs::project_signal:50`). `runtime-main` has no `rusqlite` dep, so direct main-side projection is structurally infeasible. WriteSignal IPC variant authoring + drone handler arm + main-side emission path all land in Stage B alongside the plan-state-machine signal emissions.
- Decision extractor → structured emitter migration (was previous A2.1 #3). Replacement of `crates/runtime-main/src/sdk/decision_extractor.rs` heuristic with regex on `<<DECISION>>...<<END>>` delimited blocks; prompt-template module authoring; AgentSdk plumbing. Folded into Stage B because the structured emitter is the cross-stack glue that lets the SDK detect plan-creation events from the orchestrator agent's text — same authoring stage as the plan state machine that consumes the decisions.

**Success criterion:** drone subprocess spawns at Tauri startup; the 3 `DroneClient::noop()` callsites at `commands.rs:166, 200, 247` invoke real drone IPC and return real data; wiremock-backed `count_tokens` test passes against the real endpoint shape; long-lived events() reconnect behavior is documented + tested (with spec §1d ⚠️ note disposition either closed or updated); `unwrapCmdError` uses generated types; `commands.rs::CmdError` is a re-export of the generated type; all gates pass.

**New artifacts:**
- `src-tauri/src/drone_lifecycle.rs` (new sibling of `main.rs`; subprocess spawn + lifecycle + graceful shutdown)
- `crates/runtime-main/tests/drone_reconnect_events.rs` (new integration test for long-lived events() survival)

**Edited artifacts:**
- `src-tauri/src/main.rs` (spawn drone at app startup via new `drone_lifecycle` module; register `Arc<DroneClient>` as Tauri managed state via `app.manage(...)`; wire graceful shutdown on `RunEvent::ExitRequested`)
- `src-tauri/src/commands.rs` (replace 3 `DroneClient::noop()` callsites at lines 166, 200, 247 with `tauri::State<'_, Arc<DroneClient>>` parameter + real IPC dispatch; replace hand-maintained `CmdError` enum at line 46 with re-export of A1's generated type from `runtime-core`)
- `crates/runtime-main/src/providers/anthropic.rs` (implement real `count_tokens` at line 135 against `POST /v1/messages/count_tokens`)
- `crates/runtime-main/src/drone_ipc/connection.rs` or `crates/runtime-main/src/sdk/event_pipeline.rs` (long-lived events() reconnect handling per A2.1 #3 — exact location depends on test outcome)
- `crates/runtime-main/tests/anthropic_wiremock.rs` (add `count_tokens` happy-path + error tests)
- `src/lib/ipc.ts` (refactor `unwrapCmdError` to consume generated `CmdError` from `src/types/error.ts`)
- Possibly `agent-runtime-spec.md` §1d (update or close the ⚠️ long-lived events() note based on Stage A2's test outcome)

### A2.2 Files to Change

| File | Change |
|---|---|
| `src-tauri/src/main.rs` | **Edited** — spawn drone subprocess at app startup (calls into new `drone_lifecycle` module); register `Arc<DroneClient>` via `app.manage(...)`; graceful shutdown on `RunEvent::ExitRequested` |
| `src-tauri/src/drone_lifecycle.rs` | **New** (sibling of `main.rs`) — `DroneLifecycle::spawn`, `DroneLifecycle::shutdown`, RAII drop guard for graceful exit |
| `src-tauri/src/commands.rs` | **Edited** — replace 3 `DroneClient::noop()` callsites at lines 166, 200, 247 (`run_smoke_session`, `query_session_db`, `replay_session`) with `tauri::State<'_, Arc<DroneClient>>` parameter + real IPC dispatch; replace hand-maintained `CmdError` enum at line 46 with re-export of A1's generated type — exact path depends on A1 namespace strategy (`pub use runtime_core::generated::error::CmdError;` or via re-export at `runtime_core::error::CmdError` if A1 chose strategy (a) renamed-aware) |
| `crates/runtime-main/src/providers/anthropic.rs` | **Edited** — implement `count_tokens` at line 135 against `POST /v1/messages/count_tokens` |
| `crates/runtime-main/src/drone_ipc/connection.rs` or `crates/runtime-main/src/sdk/event_pipeline.rs` | **Edited (conditional)** — long-lived events() reconnect handling per A2.1 #3; exact location surfaced by the integration test |
| `crates/runtime-main/tests/anthropic_wiremock.rs` | **Edited** — add `count_tokens` happy-path + error tests |
| `crates/runtime-main/tests/drone_reconnect_events.rs` | **New** — integration test for long-lived events() reconnect |
| `src/lib/ipc.ts` | **Edited** — `unwrapCmdError` consumes generated `CmdError` from `src/types/error.ts` |
| `agent-runtime-spec.md` §1d | **Edited (conditional)** — update or close the ⚠️ long-lived events() note based on test outcome |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes M04 Stage A2 production wiring |

### A2.3 Detailed Changes

#### `src-tauri/src/drone_lifecycle.rs` — new module

Per Tauri 2.x managed-state docs (verbatim shape per <https://v2.tauri.app/develop/state-management/>): module exposes `DroneLifecycle::spawn(app: &AppHandle, db_path: &Path) -> Result<Arc<DroneClient>, CmdError>` that:

1. Generates a unique session_id (UUID v4)
2. Computes the IPC socket path (Unix: `/tmp/runtime-drone-<sid>.sock`; Windows: `\\.\pipe\runtime-drone-<sid>`)
3. Spawns `runtime-drone` via `tokio::process::Command::new("runtime-drone").args(["--session-id", &sid, "--db-path", db_path.to_str().unwrap(), "--ipc-socket", &sock]).kill_on_drop(true).spawn()`
4. Connects a `DroneClient` to the socket (with retry per M01.C reconnect semantics — 5 attempts, 200ms→3.2s exp backoff)
5. Returns `Arc<DroneClient>` for managed-state registration

`DroneLifecycle::shutdown` sends graceful shutdown signal (drone's existing `Shutdown` IPC command) then awaits `Child::wait()` with timeout fallback to SIGKILL.

Drop guard pattern: a `DroneLifecycle` struct holding `Child` + `Arc<DroneClient>` implements `Drop` to call `shutdown` on app exit.

Tracing: `tracing::info!("drone subprocess spawned"; pid = child.id(), socket = sock)` at spawn; `tracing::warn!` on shutdown timeout fallback. Per spec §13.5 Dev Logging.

#### `src-tauri/src/main.rs` — Tauri `setup` hook (audit-grounded path)

**Path correction from previous draft.** `src-tauri/src/lib.rs` does NOT exist (audit-confirmed); the current Tauri shell is `src-tauri/src/main.rs` only — `main()` directly calls `tauri::Builder::default().invoke_handler(...).run(tauri::generate_context!())`. A2 keeps this orchestration shape and adds the spawn logic to `main.rs` itself, NOT a new `lib.rs`.

Locate the existing `tauri::Builder::default()` chain in `main.rs`. Add a `.setup(|app| { ... })` block (before the existing `.invoke_handler(...)`) that:

1. Resolves the SQLite db path via existing path-resolution helper
2. Calls `DroneLifecycle::spawn(app.handle(), &db_path)` → `Arc<DroneClient>`
3. Registers via `app.manage(drone_client.clone())`
4. Stores the `DroneLifecycle` instance for graceful shutdown (likely via a `OnceLock<Mutex<Option<DroneLifecycle>>>` static or similar — match existing app-state pattern)

Add an `.on_window_event(...)` or `.run(|_app, event| match event { ... })` handler for `RunEvent::ExitRequested` that calls `DroneLifecycle::shutdown` before propagating exit. Verify the exact Tauri 2.x event hook name + signature against current docs before authoring (cross-stack discipline per gotcha #32).

Tracing: log app-startup + drone-spawn correlation per §13.5.

A2 may choose to refactor `main.rs` into a thinner orchestration + a `lib.rs` if the Tauri 2.x examples increasingly recommend it, but **the default is to keep `main.rs` as the orchestration site and add `drone_lifecycle.rs` as a sibling**. Document the choice in the retrospective if A2 deviates.

#### `src-tauri/src/commands.rs` — replace 3 noop callsites + CmdError re-export

For each of the 3 `DroneClient::noop()` callsites at `commands.rs:166, 200, 247` (`run_smoke_session`, `query_session_db`, `replay_session`):

- Add `client: tauri::State<'_, Arc<DroneClient>>` parameter
- Replace the `DroneClient::noop()` construction with `&*client` (deref the Tauri state) for real IPC dispatch via `client.<method>().await`
- Map drone IPC errors to `CmdError::Drone { message }`

Replace the existing hand-maintained `pub enum CmdError { ... }` block at `commands.rs:46` with a re-export of A1's generated type. Exact path depends on A1's namespace strategy outcome (recorded in M04.A1 retrospective):

- If A1 chose strategy (a) renamed-aware: `pub use runtime_core::generated::error::CmdError;`
- If A1 chose strategy (b) qualified path: same, `pub use runtime_core::generated::error::CmdError;`
- If A1 chose strategy (c) absorbed: `pub use runtime_core::CmdError;` (re-exported at crate root)

Verify `runtime-core` is in `src-tauri/Cargo.toml` workspace dependencies (M03 added it; verify with `cargo tree -p agent-runtime --depth 1` before authoring).

The existing `CmdError::Internal(...)` constructor calls in this file (and consumers across `runtime-main`, `runtime-drone`) may need shape adjustment if the generated enum produces `Internal { message: String }` (struct variant) rather than `Internal(String)` (tuple variant). The `<fan_out_grep pattern="CmdError::"/>` in the stage prompt enumerates all callsites — update them in the same commit so the variant shape is consistent across crates.

**Note:** vdr projector wiring at WriteSignal call-site, and the decision-extractor → structured-emitter migration, are folded into Stage B per the audit re-staging — see Stage A2.1 "Items folded into Stage B." A2 does NOT touch `crates/runtime-main/src/sdk/event_pipeline.rs::WriteSignal` (Stage B's territory) or `crates/runtime-main/src/sdk/decision_extractor.rs` (Stage B replaces with structured emitter alongside the prompt-template module + AgentSdk plumbing).

#### `crates/runtime-main/src/providers/anthropic.rs` — real `count_tokens`

Per <https://docs.anthropic.com/en/api/messages-count-tokens>:

```rust
async fn count_tokens(&self, messages: &[Message]) -> Result<u64, ProviderError> {
    let req_body = json!({
        "model": self.model,
        "messages": messages,
    });
    let response = self.client
        .post("https://api.anthropic.com/v1/messages/count_tokens")
        .header("x-api-key", &self.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&req_body)
        .send()
        .await
        .map_err(ProviderError::from)?;
    if !response.status().is_success() {
        return Err(ProviderError::Api { status: response.status().as_u16(), body: response.text().await.unwrap_or_default() });
    }
    let body: serde_json::Value = response.json().await.map_err(ProviderError::from)?;
    body.get("input_tokens")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ProviderError::Api { status: 0, body: "missing input_tokens in response".into() })
}
```

Verify the exact request shape + response field name against the cited URL before authoring — the `input_tokens` field name and `2023-06-01` API version are both subject to upstream change.

#### `crates/runtime-main/tests/anthropic_wiremock.rs` — count_tokens tests

Add tests for the new endpoint per the existing wiremock harness pattern. Happy path, 401 auth error, 429 rate limit, malformed response. Match the structure of existing `anthropic_wiremock.rs` tests.

#### Long-lived events() reconnect — `crates/runtime-main/src/drone_ipc/connection.rs` or `event_pipeline.rs`

**Path correction from previous draft.** The previous draft named `crates/runtime-main/src/sdk/event_translation.rs` — that file does NOT exist (audit-confirmed phantom path). The actual translation pipeline is `crates/runtime-main/src/sdk/event_pipeline.rs`; the reconnect logic lives at `crates/runtime-main/src/drone_ipc/connection.rs`. A2 either extends the connection-side reconnect or the event_pipeline-side translation flow depending on what the integration test surfaces.

Per spec §1d ⚠️ note (M04 carry-forward). The existing flow takes ProviderEvents and emits AgentEvents through the bounded-stream pattern from M02. Stage A2's question: if drone↔main reconnects mid-session, does the renderer's `listen('agent_event', ...)` callback continue to receive events?

Test-driven approach (preferred): write the integration test first (in `tests/drone_reconnect_events.rs`) that:

1. Spawns drone, connects main, subscribes renderer-side via the existing IPC pattern
2. Starts a session that emits events
3. Kills the drone subprocess mid-session (SIGTERM via `Child::kill()`)
4. Spawns a fresh drone (simulating Tauri's auto-restart, or invokes existing reconnect logic)
5. Continues the session
6. Asserts renderer continues to receive events

If the test passes as-implemented (M01.C reconnect logic + Tauri event emission already handles this), close the spec ⚠️ note. If not, the test surfaces what's broken and Stage A2 implements the fix (likely involves resubscribing on reconnect or buffering events during the gap).

#### `src/lib/ipc.ts` — generated CmdError consumption

Replace the hand-maintained `interface CmdError { ... }` with `import type { CmdError } from '../types/error';` (the import path A1 generated). Update `unwrapCmdError` if the generated shape differs from the hand-maintained one — json-schema-to-typescript may produce a discriminated union form (`{ type: 'setup_required' } | { type: 'provider'; message: string } | ...`) rather than a single interface with optional fields; the discriminator key (`type`) matches the M02 hand-maintained shape but the variant-specific fields may be tightened. Preserve all behavior of the helper per gotcha #30.

#### `agent-runtime-spec.md` §1d — close or update the ⚠️ note

Conditional on Stage A2's test outcome:
- If long-lived events() survives reconnect: change the ⚠️ note from "pending (M04 carry-forward)" to "resolved at M04.A2; integration test at crates/runtime-main/tests/drone_reconnect_events.rs"
- If not: keep the ⚠️ note but document the v0.1 behavior (renderer resubscribes on reconnect via M03's replay_session pattern) and update the carry-forward target to v1.0.

#### `CHANGELOG.md` — `[Unreleased]` notes

Append:

```markdown
- M04 Stage A2: production wiring — drone subprocess lifecycle at Tauri startup; replaced DroneClient::noop() callsites in query_session_db + replay_session; vdr.rs projector wired at signal-write call-site; decision extractor migrated from heuristic to structured emitter; real /v1/messages/count_tokens endpoint replaces chars/4 approximation; long-lived events() reconnect resolved; src/lib/ipc.ts::unwrapCmdError consumes generated CmdError types from src/types/error.ts. SQL inspector + replay-from-signals + decision projection are now end-to-end functional.
```

### A2.4 Tests

#### Pedantic-pass preflight

Per `docs/gotchas.md` #21. Stage A2 introduces `src-tauri/src/drone_lifecycle.rs` (new module); apply the preflight checklist to it. The other edited modules pre-exist and inherit their existing pedantic-clean state.

#### Test files

Stage A2 adds:

- `crates/runtime-main/tests/drone_reconnect_events.rs` — integration test for long-lived events() survival across drone restart
- New tests inside `crates/runtime-main/tests/anthropic_wiremock.rs` for `count_tokens` (happy path + auth error + rate limit + malformed response)
- (Structured-emitter unit tests are folded into Stage B; not in A2)

Test sketches (full content authored in stage):

```rust
// crates/runtime-main/tests/drone_reconnect_events.rs (new)
#[tokio::test]
async fn renderer_continues_receiving_events_after_drone_restart() {
    // Setup: drone + main + renderer subscription
    let (drone, client) = spawn_drone_with_test_db().await;
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received.clone();
    tokio::spawn(async move {
        client.events().for_each(|e| async {
            received_clone.lock().await.push(e);
        }).await;
    });

    // Phase 1: emit events
    client.start_session(...).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let phase1_count = received.lock().await.len();
    assert!(phase1_count > 0);

    // Mid-session: kill drone, spawn fresh
    drone.kill().await;
    let (drone2, _client2) = spawn_drone_with_test_db().await; // same db
    tokio::time::sleep(Duration::from_millis(500)).await; // reconnect window

    // Phase 2: continue session, expect events to flow
    client.continue_session(...).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let phase2_count = received.lock().await.len();
    assert!(phase2_count > phase1_count, "no events received after reconnect");

    drone2.kill().await;
}
```

#### Coverage target

- workspace ≥80% maintained
- `runtime-main` ≥95% — new code in `providers/anthropic.rs::count_tokens` (real impl) is covered via wiremock per the M02.C precedent (`providers/anthropic.rs` real-network construction stays in the existing exclusion list per CLAUDE.md §5; wire-format logic moves to `anthropic_sse.rs`-style coverage if the count_tokens path warrants extraction). Reconnect-handling code surfaces depending on the integration test outcome — covered by either unit tests (`drone_ipc/connection.rs::send_with_reconnect` archetype) or by the integration test itself.
- New file `src-tauri/src/drone_lifecycle.rs`: unit tests via testable seam pattern (`DroneLifecycle::spawn_with(spawn_fn, ...)` taking a process-spawn closure for testability). Real OS-spawn wrapper excluded per the M02 `tauri-shell` exception in `codecov.yml` (the patch-gate covers `src-tauri/**` at 50% target rather than the workspace 80% baseline).
- New integration test `drone_reconnect_events.rs`: integration test (not subject to coverage gate; correctness is the assertion).

### A2.5 CLI Prompt

Paste the XML block below into a fresh Claude Code session as the opening message.

```xml
<work_stage_prompt id="M04.A2">
  <context>
    Stage A2 of M04 (Plan + Verify + HITL + Budget). Production wiring — drone subprocess lifecycle at Tauri startup with `Arc<DroneClient>` Tauri-managed-state via new `src-tauri/src/drone_lifecycle.rs` sibling module + spawn from existing `main.rs::main()` (note: `src-tauri/src/lib.rs` does NOT exist per audit — A2 does NOT introduce one); replaces 3 `DroneClient::noop()` callsites at `src-tauri/src/commands.rs:166, 200, 247`; implements real `count_tokens` against `/v1/messages/count_tokens` (wiremock-tested); resolves long-lived events() reconnect carry-forward via integration test (path is `crates/runtime-main/src/sdk/event_pipeline.rs` + `drone_ipc/connection.rs`, NOT phantom `event_translation.rs`); refactors `commands.rs::CmdError` to re-export A1's generated type from `runtime-core/src/generated/error.rs`; renderer-side `unwrapCmdError` consumes generated `CmdError` from `src/types/error.ts`. **Two items the previous draft scoped here are folded into Stage B per audit re-staging:** vdr WriteSignal IPC variant + drone-side handler arm calling `vdr::project_signal` (vdr lives in `crates/runtime-drone/src/vdr.rs`; `runtime-main` has no `rusqlite` dep); structured-emitter prompt-template module + AgentSdk plumbing replacing `decision_extractor.rs` heuristic. Stage B does not start until Stage A2's commit is on the milestone branch `claude/m04-plan-verify-hitl-budget`.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage A1" subject</check>
    <check name="prior_retrospective_staged">git log -1 --name-only must include docs/build-prompts/retrospectives/M04.A1-retrospective.md (per M03.5.B retro [END] decision — closes the M03.5.A drift pattern where retrospective files were untracked at commit time)</check>
    <check name="anthropic_key_set">Test-Path env:ANTHROPIC_API_KEY must succeed (count_tokens wiremock tests do not require a valid key, but the smoke test path does)</check>
    <check name="generated_error_present">Test-Path crates/runtime-core/src/generated/error.rs must succeed (A1 deliverable; path is generated/, NOT top-level — top-level error.rs is hand-curated RuntimeError)</check>
    <check name="generated_ts_present">Test-Path src/types/error.ts must succeed (A1 deliverable)</check>
    <check name="lib_rs_does_not_exist">! Test-Path src-tauri/src/lib.rs (asserts the audit baseline; if lib.rs materialized between A1 and A2 the A2 plan adapts to the new shape)</check>
    <check name="vdr_in_drone_only">Test-Path crates/runtime-drone/src/vdr.rs AND ! Test-Path crates/runtime-main/src/vdr.rs (confirms vdr architecture)</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage A2 sections A2.1–A2.4)</file>
    <file>agent-runtime-spec.md §1c, §1d, §2b, §2c (especially §2c.3), §13.5</file>
    <file>docs/gotchas.md (especially #29 keyring; #30 unwrapCmdError; #31 tracing init; #32 cross-stack; #41 grep-verify-claims)</file>
    <file>docs/build-prompts/retrospectives/M04.A1-retrospective.md (apply [END] Decisions, especially the namespace strategy outcome)</file>
  </read_first>

  <read_reference>
    <file purpose="M01.C drone subprocess spawn archetype + reconnect semantics">crates/runtime-drone/src/main.rs</file>
    <file purpose="current Tauri shell — main.rs is the orchestration site (no lib.rs); A2 adds drone_lifecycle.rs sibling and calls into it from main.rs">src-tauri/src/main.rs</file>
    <file purpose="Tauri command shell pattern + 3 DroneClient::noop() callsites at lines 166, 200, 247 to replace; CmdError at line 46 to re-export">src-tauri/src/commands.rs</file>
    <file purpose="existing DroneClient + reconnect logic to extend; testable seam send_with_reconnect at this file">crates/runtime-main/src/drone_ipc/connection.rs</file>
    <file purpose="actual event-translation pipeline (NOT phantom event_translation.rs); reconnect-flow extension may land here">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="existing Anthropic provider HTTP+SSE archetype to extend with count_tokens at line 135 (currently chars/4 approximation)">crates/runtime-main/src/providers/anthropic.rs</file>
    <file purpose="existing wiremock harness pattern + 8 existing tests; A2 adds count_tokens tests">crates/runtime-main/tests/anthropic_wiremock.rs</file>
    <file purpose="renderer-side error unwrap that needs to consume generated types">src/lib/ipc.ts</file>
    <file purpose="generated error types Stage A1 produced; Stage A2 imports">src/types/error.ts</file>
    <file purpose="generated CmdError type Stage A1 produced; A2's commands.rs re-exports — exact path depends on A1 namespace strategy outcome (see A1 retro)">crates/runtime-core/src/generated/error.rs</file>
  </read_reference>

  <read_prior_stages>
    <retrospective stage="A1" milestone="M04"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="A2.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="A2.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="Key constraints"/>

  <gates milestone="M04"/>

  <self_correction_budget>3</self_correction_budget>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <fan_out_grep>
    <grep pattern="DroneClient::noop" purpose="all callsites of the noop stub being replaced; expect query_session_db, replay_session, possibly tests"/>
    <grep pattern="CmdError::" purpose="all enum-variant-construction sites; if generated enum shape differs (e.g., Internal { message } vs Internal(String)) all callers update together"/>
    <grep pattern="count_tokens" purpose="all callers of LLMProvider::count_tokens; chars/4 approximation behavior is preserved at the trait level but real impl may surface latency that callers should handle"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="reqwest" required_features="rustls,rustls-native-certs,json,stream"/>
    <dep name="tokio" required_features="process,time,sync,io-util"/>
  </dependency_audit_check>

  <runtime_environment os="windows" note="Build agent runs on Windows 11; drone subprocess uses named pipe \\.\pipe\runtime-drone-<sid> (not Unix socket); Test-Path replaces test -f"/>

  <gotchas>
    <trap>typify-generated CmdError variant shapes (e.g., Internal { message: String } vs Internal(String) tuple) must match across the runtime-main + drone + Tauri commands callsites — fan_out_grep above catches these; do NOT silently leave one callsite with the old shape</trap>
    <trap>Drone subprocess kill_on_drop(true) is mandatory — without it, the subprocess outlives the Tauri app on crash and leaves stale .sock/.pipe files; gotcha #29-style silent failure mode in production</trap>
    <trap>count_tokens against the real endpoint — verify the exact response field name (input_tokens vs token_count vs other) against https://docs.anthropic.com/en/api/messages-count-tokens BEFORE authoring; do NOT assume the M03.5 spec text §2c.3 is verbatim correct (it's design-doc not API spec)</trap>
    <trap>Long-lived events() reconnect — the test outcome drives the spec edit. The actual translation pipeline is `event_pipeline.rs` (NOT phantom `event_translation.rs`); reconnect logic lives at `drone_ipc/connection.rs`. If the test reveals broken-as-implemented, do NOT silently fix without surfacing to the user — this is a v0.1 behavior decision and may warrant scoping to v1.0</trap>
    <trap>src-tauri/src/lib.rs does NOT exist in current shell — DO NOT introduce one as part of A2's spawn-logic placement. Spawn logic lands in main.rs::main() directly + new drone_lifecycle.rs sibling. If the upstream Tauri 2.x examples now strongly recommend lib.rs, surface as a design call rather than silently restructuring</trap>
    <trap>vdr WriteSignal IPC + structured-emitter prompt template are FOLDED INTO STAGE B per audit re-staging — DO NOT pre-author them in A2 even if the production wiring naturally cuts that direction; respect the staging boundary so Stage B's authoring discipline holds</trap>
    <trap>CmdError re-export path depends on A1 namespace strategy outcome (recorded in M04.A1 retrospective): if A1 chose strategy (a) renamed-aware, use pub use runtime_core::generated::error::CmdError; same for (b) qualified path; if (c) absorbed, use pub use runtime_core::CmdError. Read the A1 retro before writing this line.</trap>
    <trap>Per gotcha #41 (grep-verify-claims): every codebase claim in this stage's deliverable section is verified against end-of-M03.5 reality. If the &lt;pre_flight_check&gt; verifications fail (codebase has drifted between authoring and execution), surface the drift before proceeding — do NOT improvise around it.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT regenerate framework/skill/agent/tool/common/event/error schemas — Stage A1 already did event + error; existing schemas were already done in M01–M03</warning>
    <warning>DO NOT call /v1/messages/count_tokens against the live API in tests — wiremock only. Live calls are reserved for the smoke test in src-tauri (which gates on ANTHROPIC_API_KEY presence)</warning>
    <warning>DO NOT push between stages — Stage A2 commits locally only. Push happens at end of Stage G per CLAUDE.md §8 + §20</warning>
    <warning>DO NOT touch crates/runtime-main/src/sdk/decision_extractor.rs (Stage B's structured-emitter replacement) or crates/runtime-main/src/sdk/event_pipeline.rs::WriteSignal (Stage B's vdr WriteSignal IPC wiring) — both folded into Stage B per audit re-staging</warning>
    <warning>The drone subprocess spawn at Tauri setup is the highest-risk surface in M04 — if startup hangs or races with renderer mount, surface immediately rather than working around (e.g., hidden setTimeout in renderer); the user explicitly approved high-risk-first staging in the M04 plan</warning>
    <warning>Stage A2's commit MUST include docs/build-prompts/retrospectives/M04.A2-retrospective.md in the staged files (per M03.5.B retro [END] decision)</warning>
  </execution_warnings>

  <time_box estimate_hours="4.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage B: long-lived events() reconnect outcome (resolved or v0.1 deferred? path location of the fix if any — connection.rs or event_pipeline.rs); whether count_tokens response field name matched the M03.5 spec §2c.3 wording or required spec follow-up; whether drone subprocess startup latency on cold-start affects renderer mount UX (Stage F may need a loading state); CmdError re-export shape — confirmation of A1 namespace strategy applied consistently across runtime-main + drone + Tauri commands; any cross-stack glue points the agent had to verbatim-quote from upstream rather than authoring (cite the upstream source in the retro per gotcha #32); confirmation that Stage A2 did NOT pre-author Stage B's vdr WriteSignal IPC or structured-emitter work.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="A2.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD) including M04.A2-retrospective.md in staged set</item>
    <item>gate results (each gate, pass/fail, key numbers including wiremock test count + drone_reconnect_events.rs outcome)</item>
    <item>schema drift check output — cargo xtask regenerate-types --check exit code (must be 0)</item>
    <item>fan_out_grep results — DroneClient::noop / CmdError:: / count_tokens callsite counts before vs after refactor (target: 0 noop callsites remaining at commands.rs:166, 200, 247; CmdError:: variant shapes consistent across crates)</item>
    <item>long-lived events() reconnect test outcome — pass (closed) or fail (v0.1 behavior documented + spec updated); cite the file:line where the fix landed if any (connection.rs vs event_pipeline.rs)</item>
    <item>spec §1d ⚠️ note disposition — closed or updated (cite line)</item>
    <item>CmdError re-export verification — `pub use` line in commands.rs matches A1 namespace strategy outcome</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage B)</item>
    <item>draft commit message from M04-plan-verify-hitl-budget.md A2.6 Commit Message section (filled with session URL)</item>
    <item>explicit statement: "Stage M04.A2 is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### A2.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime+renderer): M04 Stage A2 — production wiring

Replaces M03's DroneClient::noop() seams with real drone subprocess
lifecycle + closes three M02/M03 carry-forward 🟡 production-wiring
items. SQL inspector + replay-from-signals + smoke session are now
end-to-end functional.

Two items the previous draft scoped here (vdr WriteSignal IPC + 
structured-emitter prompt template) are folded into Stage B per the
post-M03.5 audit re-staging — Stage B has scope slack from already-
shipped events to absorb both.

Production wiring:
- src-tauri/src/main.rs + drone_lifecycle.rs (new sibling module):
  drone subprocess spawned at Tauri setup hook via tokio::process::
  Command; Arc<DroneClient> registered as Tauri managed state; graceful
  shutdown on RunEvent::ExitRequested. kill_on_drop(true) per docs/
  gotchas.md drone-subprocess discipline. Note: src-tauri/src/lib.rs
  does NOT exist; spawn logic lands in main.rs directly.
- src-tauri/src/commands.rs: 3 DroneClient::noop() callsites at lines
  166, 200, 247 (run_smoke_session, query_session_db, replay_session)
  take tauri::State<'_, Arc<DroneClient>> and dispatch real drone IPC;
  CmdError enum at line 46 becomes pub use runtime_core::generated::
  error::CmdError (path per A1 namespace strategy).
- crates/runtime-main/src/providers/anthropic.rs: count_tokens at line
  135 calls POST /v1/messages/count_tokens (per spec §2c.3 added
  M03.5); chars/4 approximation removed. wiremock-tested.
- crates/runtime-main/src/drone_ipc/connection.rs OR sdk/event_pipeline.rs
  + tests/drone_reconnect_events.rs (new integration test): long-lived
  events() reconnect resolved [or documented as v0.1 behavior — see
  retro]. Spec §1d ⚠️ note [closed at this commit / updated to reflect
  v0.1 behavior].
- src/lib/ipc.ts: unwrapCmdError consumes generated CmdError type from
  src/types/error.ts; preserves gotcha #30 unwrap semantics.

Carry-forward closures:
- M03 🟡 Production drone subprocess wiring at Tauri startup
- M02 🟡 count_tokens → real /v1/messages/count_tokens endpoint
- M02 🟡 Long-lived events() subscription survives reconnect

Folded into Stage B (NOT closed in A2):
- M03 🟡 vdr.rs projector wired at signal-write call-site (Stage B
  authors WriteSignal IPC variant + drone-side handler arm calling
  vdr::project_signal; runtime-main has no rusqlite dep so direct
  main-side projection is structurally infeasible)
- M02 🟡 Decision extractor → structured emitter migration (Stage B
  authors prompt-template module + AgentSdk plumbing alongside the
  plan state machine that consumes the structured decisions)

Spec edits (conditional on test outcome):
- §1d ⚠️ note disposition

Refs: M04-plan-verify-hitl-budget.md §A2, gap-analysis.md M03 + M02
entries 🟡 (3 carry-forward items closed in A2; 2 deferred to Stage B
per audit re-staging)
Retrospective: docs/build-prompts/retrospectives/M04.A2-retrospective.md

https://claude.ai/code
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE B — §3a Plan & Task primitive                            -->
<!-- ============================================================ -->

## Stage B — §3a Plan & Task primitive (schemas + types + 5 missing events + state machine + persistence + folded-A3 work)

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.rs/typify/latest/typify/> — typify codegen for plan.v1.json + task.v1.json (extends Stage A1 pattern)
- <https://json-schema.org/draft/2020-12/schema> — JSON Schema 2020-12 spec; plan.v1.json + task.v1.json author against this draft (matches existing schemas)
- <https://docs.rs/rusqlite/latest/rusqlite/> — rusqlite for new `plans` + `tasks` table migrations; verify `journal_mode = WAL` + `foreign_keys = ON` pragma pattern unchanged from M01.C `db.rs`
- <https://docs.rs/regex/latest/regex/> — structured-emitter regex pattern for `<<DECISION>>...<<END>>` delimited blocks (folded from original-A3 work); confirm `regex` crate is in workspace deps

### B.1 Problem Statement

§3a Plan & Task primitive is the single largest deliverable in M04. Spec §3a (with M03.5's DDL addition) locks the field shapes. Stage B builds the implementation end-to-end AND folds the original-A3 work (WriteSignal IPC + structured emitter) per the post-M03.5 audit re-staging:

1. **Schemas** — author `schemas/plan.v1.json` + `schemas/task.v1.json` per spec §3a TypeScript shapes + M03.5 DDL. Extend `crates/xtask/src/main.rs` codegen list per the Stage A1 archetype. Generated targets: `crates/runtime-core/src/generated/plan.rs` + `crates/runtime-core/src/generated/task.rs` (Rust; under `generated/` per the typify convention A1 confirmed); `src/types/plan.ts` + `src/types/task.ts` (TS).

2. **5 missing spec event variants + 6 already-shipped wired + 2 codebase extras dispositioned** (audit-grounded). Per the post-M03.5 audit: `crates/runtime-core/src/event.rs` already contains 6 spec-canonical variants (`PlanCreated:141`, `PlanApproved:148`, `TaskStarted:160`, `TaskCompleted:169`, `TaskFailed:178`, `TaskEscalated:198`) + 2 codebase-only extras NOT in spec (`PlanRejected:153`, `TaskRolledBack:189`). Stage B authors the 5 spec variants missing from `event.v1.json` + `event.rs`: `plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped`. Stage B WIRES the 6 existing variants into the new state machine (does NOT re-author them). Stage B documents the disposition of the 2 extras at execution time — likely keep both as additive (`PlanRejected` covers the user-cancels-at-approval-gate flow distinct from `plan_aborted` in-progress; `TaskRolledBack` covers rollback via verify hooks). Renderer `graphStore.applyEvent` exhaustive switch already handles the 8 existing variants; Stage B adds 5 new cases (gotcha #36 `_exhaustive: never` forces it).

3. **Plan state machine** — `crates/runtime-main/src/plan/state_machine.rs` (new module; verify `crates/runtime-main/src/plan/` does not yet exist before authoring per `<pre_flight_check>`) implements the FSM over Plan.status + Task.status transitions per spec §3a. Safety primitive — ≥95% coverage gate per CLAUDE.md §5 (declare exclusions inline if any; pure-logic module so likely 100%).

4. **fresh_context_per_task loop policy** — only loop policy lit in v0.1 per spec §0d + CLAUDE.md §3. Implementation: after each `task_completed`, the SDK clears the agent's message history and starts the next task with the full plan + completed-tasks summary in the system prompt. The `one_shot` and `continuous` variants in the schema return `NotImplemented` at this seam.

5. **Failure escalation** — `failure_count++` on `task_failed`; if `>= max_failures` → emit `task_escalated` (routed to HITL in Stage E). Default `max_failures = 3` per spec §3a.

6. **SQLite persistence** — migrations land `plans` + `tasks` tables per the DDL added to spec §10 in M03.5. **First migration creates the `crates/runtime-drone/migrations/` directory** — audit-confirmed the directory does not yet exist. Drone-side migration runner (existing `crates/runtime-drone/src/db.rs::run_migrations` from M01.C) picks up the new migration file. Verify migration version increment is `001` (no existing-numbered-sequence to fit into).

7. **Approval-gate primitive** — when `Plan.approval_required = true` and a `plan_created` fires, the runtime emits `plan_approval_requested` and SUSPENDS the plan until `plan_approved` (via HITL flow — Stage E wires this; Stage B exposes the suspend/resume seam as a channel/oneshot the SDK awaits on, NOT the HITL UI).

8. **WriteSignal IPC + drone-side `vdr::project_signal` handler arm + main-side emission path** (folded from original-A3). Stage B authors the new `WriteSignal { signal_id, ... }` variant on `DroneCommand` enum at `crates/runtime-core/src/drone.rs` (verified: no `WriteSignal` variant present today); adds the handler arm to `crates/runtime-drone/src/command_handler.rs` (existing arms: `SnapshotNow`, `GracefulShutdown`, `SpawnProcess`, `StopProcess`, `SetActivityTimeout`, `RevertToSnapshot:64`, `QuerySessionDb`, `ReadSignals`); the new arm calls `vdr::project_signal(&conn, signal_id)` from `crates/runtime-drone/src/vdr.rs:50`. Main-side emission lands at `crates/runtime-main/src/sdk/event_pipeline.rs` — when the SDK writes a Decision signal, it emits `DroneCommand::WriteSignal` rather than direct rusqlite (which `runtime-main` doesn't have as a dep). Closes M03 carry-forward 🟡 "vdr.rs projector wired at signal-write call-site" (folded from A2's deferrals into Stage B per audit re-staging).

9. **Structured-emitter prompt-template module + AgentSdk plumbing** (folded from original-A3). Replaces the M02 line-by-line heuristic in `crates/runtime-main/src/sdk/decision_extractor.rs` with regex-based delimited-block extraction matching `<<DECISION>>{json}<<END>>` per the M03.5 spec §2b ⚠️ note. New module `crates/runtime-main/src/sdk/prompt_template.rs` injects the format instructions into the system prompt. AgentSdk plumbing wires the template into the prompt-builder. Closes M02 carry-forward 🟡 "Decision extractor → structured emitter migration" (folded from A2's deferrals).

**Success criterion:** unit tests cover plan state machine transitions exhaustively (hot path + every error transition); SDK can spawn a 3-task plan that emits `plan_created` → `plan_approval_requested` → (manual approval shim) → `plan_approved` → `task_started`/`task_completed` × 3 → `plan_complete`; SQLite contains the plan + task rows with correct status transitions; structured emitter parses delimited blocks correctly; WriteSignal IPC round-trip emits `vdr` projection on drone side; coverage gate met (≥95% on `state_machine.rs`; ≥95% maintained on `runtime-main`; ≥95% maintained on `runtime-drone`).

**New artifacts:**
- `schemas/plan.v1.json`, `schemas/task.v1.json` (new)
- `crates/runtime-core/src/generated/plan.rs`, `crates/runtime-core/src/generated/task.rs` (new; generated under `generated/` per A1 convention)
- `src/types/plan.ts`, `src/types/task.ts` (new; generated)
- `crates/runtime-main/src/plan/mod.rs`, `crates/runtime-main/src/plan/state_machine.rs` (new module)
- `crates/runtime-main/src/sdk/prompt_template.rs` (new; folded-A3 structured-emitter prompt-template module)
- `crates/runtime-drone/migrations/001_plans_tasks.sql` (new; first migration file — creates the `migrations/` directory which does not yet exist)
- `crates/runtime-main/tests/plan_lifecycle.rs` (new integration test)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (extend codegen list with `plan` + `task` schemas; generated/ outputs)
- `schemas/event.v1.json` (add 5 missing spec event variants to the `oneOf`; do NOT touch the 8 existing plan/task variants)
- `crates/runtime-core/src/event.rs` OR `crates/runtime-core/src/generated/event.rs` (regenerated; depends on A1's namespace strategy outcome)
- `crates/runtime-core/src/drone.rs` (add `WriteSignal { signal_id: String, ... }` variant to `DroneCommand` enum at line 152+; folded-A3 work)
- `crates/runtime-drone/src/command_handler.rs` (add handler arm for `WriteSignal`; calls `vdr::project_signal`; folded-A3 work)
- `crates/runtime-main/src/sdk/event_pipeline.rs` (emit `DroneCommand::WriteSignal` at signal-write call-site; folded-A3 work)
- `crates/runtime-main/src/sdk/decision_extractor.rs` (replace heuristic with regex-based structured emitter; folded-A3 work)
- `crates/runtime-main/src/sdk/agent_sdk.rs` or `mod.rs` (wire plan state machine into SDK event loop; wire structured emitter via prompt template)
- `src/types/agent_event.ts` (regenerated with 5 new variants)
- `src/lib/graphStore.ts` (extend `applyEvent` exhaustive switch with 5 new cases — even if rendering wiring lands in Stage C, the store must compile under `_exhaustive: never`)
- `CHANGELOG.md` (`[Unreleased]` notes M04 Stage B Plan & Task primitive + folded-A3 work)

### B.2 Files to Change

| File | Change |
|---|---|
| `schemas/plan.v1.json` | **New** — JSON Schema 2020-12 for Plan per spec §3a + M03.5 DDL field shapes |
| `schemas/task.v1.json` | **New** — JSON Schema 2020-12 for Task per spec §3a + M03.5 DDL field shapes |
| `crates/xtask/src/main.rs` | **Edited** — extend codegen list (currently 7 entries post-A1) with `plan` + `task` (Rust outputs to `generated/`; TS outputs to `src/types/`) |
| `schemas/event.v1.json` | **Edited** — add 5 missing spec event variants (`plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped`); existing 8 plan/task variants stay unchanged |
| `crates/runtime-core/src/generated/{plan,task}.rs` | **New (regen)** — typify outputs |
| `crates/runtime-core/src/event.rs` OR `generated/event.rs` | **Edited (regen)** — adds 5 new variants; path depends on A1 namespace strategy |
| `crates/runtime-core/src/drone.rs` | **Edited** — add `WriteSignal { signal_id: String, ... }` variant to `DroneCommand` (line 152+); folded-A3 |
| `crates/runtime-core/src/lib.rs` | **Edited** — declare new `plan` + `task` re-exports per A1 namespace strategy |
| `src/types/{plan,task,agent_event}.ts` | **Edited (regen)** — json-schema-to-typescript output |
| `crates/runtime-main/src/plan/mod.rs` | **New** — module root + public API |
| `crates/runtime-main/src/plan/state_machine.rs` | **New** — Plan/Task FSM per spec §3a (≥95% safety primitive) |
| `crates/runtime-main/src/sdk/prompt_template.rs` | **New** — folded-A3 structured-emitter prompt template module |
| `crates/runtime-main/src/sdk/decision_extractor.rs` | **Edited (rewrite)** — replace M02 heuristic with regex-based structured emitter for `<<DECISION>>...<<END>>` |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | **Edited** — emit `DroneCommand::WriteSignal` at signal-write call-site; folded-A3 |
| `crates/runtime-main/src/sdk/agent_sdk.rs` (or `mod.rs`) | **Edited** — wire state machine into SDK event loop; failure-escalation logic; structured-emitter prompt-template plumbing |
| `crates/runtime-drone/src/command_handler.rs` | **Edited** — add `WriteSignal` handler arm (calls `vdr::project_signal`); existing arms at lines 64+ stay |
| `crates/runtime-drone/migrations/001_plans_tasks.sql` | **New** — first migration file; creates the `migrations/` directory (currently does not exist) |
| `src/lib/graphStore.ts` | **Edited** — extend `applyEvent` exhaustive switch with 5 new cases |
| `crates/runtime-main/tests/plan_lifecycle.rs` | **New** — integration test for plan-end-to-end flow |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes |

### B.3 Detailed Changes

#### `schemas/plan.v1.json` + `schemas/task.v1.json` — new schema files

Author each JSON Schema following the existing `schemas/*.v1.json` shape (`$schema`, `$id` per the established `https://schemas.aria-runtime.dev/<name>.v1.json` pattern per gotcha caught in M03.5.A retro, `title`, `description`, `properties`, `required`, `additionalProperties: false`). Field shapes match spec §3a TypeScript interfaces + M03.5 SQLite DDL:

- **Plan**: `id` (string, uuid), `session_id` (string, uuid), `title` (string), `description?` (string), `status` (enum: 6 values), `approval_required` (boolean), `loop_policy` (enum: 3 values; only `fresh_context_per_task` lit in v0.1 per scope locks), `hitl_checkpoints` (array of strings), `risks` (array of strings), `created_by?` (string), `created_at` (integer, unix ms), `approved_at?` (integer), `completed_at?` (integer).
- **Task**: `id` (string, uuid), `plan_id` (string, uuid), `title` (string), `status` (enum: 6 values), `hitl` (boolean), `hitl_reason?` (string), `failure_count` (integer, default 0), `max_failures` (integer, default 3), `files_affected?` (array of glob strings), `acceptance_criteria?` (array of strings), `created_at` (integer), `started_at?` (integer), `completed_at?` (integer), `estimated_minutes?` (integer), `actual_minutes?` (integer).

Pre-flight: `<schema_drift_check>` on Stage A1 + A2 outputs must be clean before authoring (verifies Stage A1 + A2's xtask state is durable).

#### `crates/xtask/src/main.rs` — codegen list extension

Add two entries to the existing 7-entry codegen list (post-A1: `["common", "framework", "skill", "tool", "agent", "event", "error"]` at line 45):

```rust
let schemas = ["common", "framework", "skill", "tool", "agent", "event", "error", "plan", "task"];
```

Outputs land at `crates/runtime-core/src/generated/{plan,task}.rs` + `src/types/{plan,task}.ts` per the existing A1 archetype. Run `cargo xtask regenerate-types` to produce; run `--check` to verify deterministic output.

#### `schemas/event.v1.json` — 5 missing spec event variants (audit-grounded)

**Audit-grounded.** The existing `oneOf` already contains 8 plan/task variants (`PlanCreated`, `PlanApproved`, `PlanRejected`, `TaskStarted`, `TaskCompleted`, `TaskFailed`, `TaskRolledBack`, `TaskEscalated` — verified by `grep '"const":' schemas/event.v1.json`). Stage B adds the 5 spec-canonical variants missing from the schema:

```json
{ "type": "object", "title": "plan_approval_requested", "properties": { "type": { "const": "plan_approval_requested" }, "plan_id": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "plan_revised", "properties": { "type": { "const": "plan_revised" }, "plan_id": { "type": "string" }, "revision_reason": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "revision_reason", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "plan_aborted", "properties": { "type": { "const": "plan_aborted" }, "plan_id": { "type": "string" }, "abort_reason": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "abort_reason", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "plan_complete", "properties": { "type": { "const": "plan_complete" }, "plan_id": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "task_skipped", "properties": { "type": { "const": "task_skipped" }, "task_id": { "type": "string" }, "skip_reason": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "task_id", "skip_reason", "timestamp"], "additionalProperties": false }
```

Field shapes per spec §3a Events subsection. Run `cargo xtask regenerate-types` to propagate to the regen target (`event.rs` or `generated/event.rs` per A1 namespace strategy).

**Disposition of 2 codebase-only extras (`PlanRejected`, `TaskRolledBack`):** keep both as additive variants. `PlanRejected` semantically distinct from `plan_aborted` (rejected = user declined at approval gate; aborted = in-progress plan stops). `TaskRolledBack` semantically distinct from `task_failed` (rolled back via verify hook revert; failed = task execution errored). Document both in the retrospective + propose a spec follow-up to canonicalize them.

#### `crates/runtime-main/src/plan/state_machine.rs` — Plan/Task FSM

The FSM enforces the legal transitions per spec §3a:

- Plan: `pending_approval` → `approved` | `aborted`; `approved` → `in_progress`; `in_progress` → `complete` | `aborted` | `awaiting_replan`; `awaiting_replan` → `in_progress` (after revise) | `aborted`.
- Task: `pending` → `running`; `running` → `done` | `failed` | `blocked` | `skipped`; `failed` → `pending` (retry within max_failures) | `escalated` (≥ max_failures); `blocked` → `pending` (after gap resolution); `skipped` is terminal.

Module exposes `PlanStateMachine::transition(plan: &mut Plan, event: PlanEvent) -> Result<(), TransitionError>` and `TaskStateMachine::transition(task: &mut Task, event: TaskEvent) -> Result<(), TransitionError>`. Errors: `IllegalTransition { from, to }`, `UnknownEvent`, `MissingPrecondition { reason }`.

Pure module — no I/O, no async. Drives the SDK event loop's plan-state updates.

Test plan: exhaustive transition matrix (legal + illegal pairs); failure-escalation boundary (max_failures=3 → 4th failure emits `task_escalated`); plan-status invariants (e.g., `approval_required=false` skips `pending_approval`). ≥95% coverage gate (safety primitive per CLAUDE.md §5).

#### `crates/runtime-main/src/sdk/mod.rs` — SDK event loop integration

Locate the SDK's existing event-emit logic. Add plan-state hooks:

- After each `agent_text_complete` of an "orchestrator" agent (the agent whose role is plan creation), parse for plan creation per the M03.5 prompt-template structured-emitter pattern (Stage A2 deliverable; reuse the regex). On detection: emit `plan_created` (with the parsed Plan); if `approval_required`, immediately emit `plan_approval_requested` and SUSPEND the plan (Stage E wires the suspend/resume seam to HITL; Stage B exposes the channel/oneshot the SDK awaits on).
- After `task_completed` (or any task-terminal event): advance the plan state machine and emit the next event(s) per the FSM; when all tasks done, emit `plan_complete`.
- `fresh_context_per_task`: between `task_completed[N]` and `task_started[N+1]`, clear the agent's `messages` vec and seed with `system_prompt + plan_summary + completed_tasks_summary + current_task`.

Do not implement the orchestrator agent's prompt template here — that's framework-JSON territory (loaded via `examples/aria/framework.json`). The SDK provides the FSM + event emission + loop-policy machinery; framework JSON wires it.

#### `crates/runtime-drone/migrations/001_plans_tasks.sql` — first migration file

**Audit-grounded.** The `crates/runtime-drone/migrations/` directory does NOT yet exist (verified). Stage B's first migration creates the directory; version `001` (no existing-numbered-sequence to fit into).

Author the SQL migration matching the M03.5 §10 spec DDL verbatim. The migration runner is the existing `crates/runtime-drone/src/db.rs::run_migrations` (M01.C). Verify the runner correctly picks up files from `migrations/` (it currently looks for the directory; an absent directory is treated as zero migrations, which is fine for M03 but Stage B's run must succeed once the file exists).

#### `crates/runtime-core/src/drone.rs` — `WriteSignal` IPC variant (folded-A3)

Locate the existing `pub enum DroneCommand` at line 152. The current variants are: `SnapshotNow`, `GracefulShutdown`, `SpawnProcess`, `StopProcess`, `SetActivityTimeout`, `RevertToSnapshot`, `QuerySessionDb`, `ReadSignals` (8 variants). Add a 9th:

```rust
WriteSignal {
    signal_id: String,
    session_id: String,
    /// Decision-signal payload as serde-serialized JSON; drone-side handler
    /// inserts into `signals` table then calls `vdr::project_signal`.
    payload: String,
},
```

The schema source (`schemas/drone.v1.json` if present, or `runtime-core` hand-curated `drone.rs` per `lib.rs:4-5`) gets updated correspondingly — `drone.rs` is hand-curated so this is a direct edit. Document the variant added in the retrospective.

#### `crates/runtime-drone/src/command_handler.rs` — `WriteSignal` handler arm (folded-A3)

The existing `pub async fn run` at this file has arms for `SnapshotNow`, `GracefulShutdown`, etc. — including `RevertToSnapshot` at line 64 (audit-confirmed). Add a new arm for `WriteSignal`:

```rust
DroneCommand::WriteSignal { signal_id, session_id, payload } => {
    // Insert into signals table (existing M01.C pattern via db.rs)
    db::insert_signal(&conn, &signal_id, &session_id, &payload).await?;
    // Project to VDR; non-fatal failure per spec §2b separation of concerns
    if let Err(e) = vdr::project_signal(&conn, &signal_id) {
        tracing::warn!(signal_id = %signal_id, error = ?e, "vdr projection failed");
    }
    Ok(DroneEvent::SignalLog { signal_id })
}
```

`vdr::project_signal` lives at `crates/runtime-drone/src/vdr.rs:50` (audit-confirmed). The handler returns `DroneEvent::SignalLog` (existing variant) so main-side tracking is unchanged.

#### `crates/runtime-main/src/sdk/event_pipeline.rs` — main-side WriteSignal emission (folded-A3)

Locate the existing decision-signal write path. Replace direct rusqlite calls (or stubs from M02/M03) with `DroneCommand::WriteSignal` emission via the `DroneClient` from Stage A2's managed state:

```rust
let cmd = DroneCommand::WriteSignal {
    signal_id: signal.id.clone(),
    session_id: ctx.session_id.clone(),
    payload: serde_json::to_string(&signal)?,
};
client.send_command(cmd).await?;
```

Non-blocking: a drone-side projection failure logs but does not fail the signal write per the handler arm above + spec §2b.

#### `crates/runtime-main/src/sdk/decision_extractor.rs` — structured emitter (folded-A3)

Replace the M02 line-by-line heuristic at this file with regex-based delimited-block extraction. Pattern:

```rust
// Match the structured-decision block injected by the prompt template:
// <<DECISION>>
// {
//   "type": "...",
//   "subject": "...",
//   "rationale": "..."
// }
// <<END>>
static DECISION_BLOCK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<<DECISION>>\s*(\{.*?\})\s*<<END>>").unwrap()
});

pub fn extract_decisions(text: &str) -> Vec<Decision> {
    DECISION_BLOCK
        .captures_iter(text)
        .filter_map(|cap| serde_json::from_str::<Decision>(&cap[1]).ok())
        .collect()
}
```

Note the `(?s)` flag (DOTALL mode) so the regex can match multi-line JSON inside the delimiters.

Unit tests: round-trip a known decision through the regex; multi-decision text; malformed-JSON tolerance (skip + log, do not error); no-decision text returns empty.

#### `crates/runtime-main/src/sdk/prompt_template.rs` — new module (folded-A3)

New module exposing `build_system_prompt(base_prompt: &str, decision_format: &str) -> String` that injects the `<<DECISION>>...<<END>>` format instructions into the system prompt. Called by `agent_sdk.rs` when the SDK builds the agent's initial system message.

The format-instruction text is verbatim per the M03.5 spec §2b ⚠️ note (do NOT alter the delimiter format without a paired spec edit).

#### `src/lib/graphStore.ts` — applyEvent exhaustive switch (5 new cases)

The `applyEvent(state, event)` function already has cases for the 8 existing plan/task variants (verified). Stage B adds 5 new cases for the spec-missing variants:

- `plan_approval_requested`: mark the existing PlanNode (created by the prior `plan_created` case) as `awaiting_approval`
- `plan_revised`: bump the PlanNode's `revision_count`; render side-effect lands in Stage C
- `plan_aborted`: mark the PlanNode as `aborted`; cascade to all child TaskNodes that aren't terminal
- `plan_complete`: mark the PlanNode as `complete`
- `task_skipped`: mark the TaskNode as `skipped`

Stage B implements the case bodies as **pass-through to graph state** (no visual treatment yet — Stage C lights up the surface). The 8 existing cases are NOT touched. The `_exhaustive: never` discriminator ensures the compiler enforces all 5 are handled.

### B.4 Tests

#### Pedantic-pass preflight

Apply per `docs/gotchas.md` #21 to the new modules: `plan/state_machine.rs`, `plan/mod.rs`, `prompt_template.rs`. Generated files exempt.

#### Test files

- `crates/runtime-main/src/plan/state_machine.rs` — unit tests for legal/illegal transitions; failure-escalation boundary; plan-status invariants
- `crates/runtime-main/src/sdk/decision_extractor.rs` — unit tests for structured-emitter regex (round-trip; multi-decision text; malformed-JSON tolerance; no-decision returns empty)
- `crates/runtime-main/src/sdk/prompt_template.rs` — unit tests for system-prompt builder
- `crates/runtime-main/tests/plan_lifecycle.rs` (new integration test) — full plan flow: orchestrator emits `plan_created` → approval requested → approved (manual shim) → 3 tasks executed → `plan_complete`; SQLite assertions after each phase via the new `WriteSignal` IPC path; verifies `vdr` table populated drone-side
- `crates/runtime-drone/src/command_handler.rs` — extended unit tests for the new `WriteSignal` arm (calls vdr; non-fatal projection failure; signal inserted regardless)
- `tests/unit/graphStore.test.ts` (extended) — applyEvent exhaustive coverage for the 5 new variants; state assertions; existing 8 variants unchanged

#### Coverage target

- `crates/runtime-main/src/plan/state_machine.rs` ≥95% (safety primitive per CLAUDE.md §5)
- `crates/runtime-main/src/sdk/decision_extractor.rs` ≥95% (structured emitter; pure-logic, easy to fully cover)
- `crates/runtime-main` ≥95% maintained
- `crates/runtime-drone` ≥95% maintained (new `WriteSignal` arm + `vdr::project_signal` integration testable via the existing seam pattern)
- workspace ≥80% maintained
- Generated files excluded via existing regex

### B.5 CLI Prompt

```xml
<work_stage_prompt id="M04.B">
  <context>
    Stage B of M04. §3a Plan & Task primitive — largest deliverable in M04 by file count + LOC. Audit-grounded scope: author 5 missing spec event variants (`plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped` — currently absent from `schemas/event.v1.json` + `crates/runtime-core/src/event.rs`); WIRE the 6 spec-canonical variants already in `event.rs` (`PlanCreated:141`, `PlanApproved:148`, `TaskStarted:160`, `TaskCompleted:169`, `TaskFailed:178`, `TaskEscalated:198`) into the new state machine — do NOT re-author them; document disposition of the 2 codebase-only extras NOT in spec (`PlanRejected:153`, `TaskRolledBack:189`) in the retrospective. Plus: `plan.v1.json` + `task.v1.json` schemas, plan/task FSM (≥95% safety primitive), `fresh_context_per_task` loop policy, failure escalation (`max_failures = 3`), plans + tasks SQLite tables (first migration creates the `crates/runtime-drone/migrations/` directory which does not yet exist), approval-gate seam exposed for Stage E. **Folds the original-A3 work** per audit re-staging: WriteSignal IPC variant on `DroneCommand` enum at `crates/runtime-core/src/drone.rs:152+` + drone-side handler arm at `crates/runtime-drone/src/command_handler.rs` (existing arms include `RevertToSnapshot:64`) calling `vdr::project_signal` at `crates/runtime-drone/src/vdr.rs:50` + main-side emission path at `crates/runtime-main/src/sdk/event_pipeline.rs`; structured-emitter prompt-template module replacing M02 heuristic in `crates/runtime-main/src/sdk/decision_extractor.rs` (per spec §2b ⚠️ note — `<<DECISION>>...<<END>>` delimiter format); new `crates/runtime-main/src/sdk/prompt_template.rs` module + AgentSdk plumbing. Stage A2's commit must be on the milestone branch claude/m04-plan-verify-hitl-budget. Plan state machine is a NEW safety primitive subject to ≥95% coverage gate.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage A2" subject</check>
    <check name="prior_retrospective_staged">git log -1 --name-only must include docs/build-prompts/retrospectives/M04.A2-retrospective.md (per M03.5.B retro [END] decision)</check>
    <check name="a1_artifacts_present">Test-Path crates/runtime-core/src/generated/error.rs AND Test-Path src/types/error.ts (A1 deliverables; path under generated/ NOT top-level since top-level error.rs is hand-curated RuntimeError)</check>
    <check name="a2_drone_lifecycle_present">Test-Path src-tauri/src/drone_lifecycle.rs (A2 deliverable; sibling of main.rs, NOT under lib.rs which doesn't exist)</check>
    <check name="a2_arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Stage B's plan-lifecycle integration test consumes this state)</check>
    <check name="a2_count_tokens_real">grep -q "messages/count_tokens" crates/runtime-main/src/providers/anthropic.rs (A2 deliverable; budget calculations downstream require)</check>
    <check name="schemas_drift_clean">cargo xtask regenerate-types --check exit 0 (A1+A2 codegen state durable; Stage B regenerate is over the same baseline)</check>
    <check name="audit_baseline_event_inventory">grep '"const":' schemas/event.v1.json | grep -E "plan_|task_" | wc -l must equal 8 (audit baseline; Stage B adds 5 new = 13 total post-implement; if not 8 the codebase has drifted between authoring and execution and Stage B's plan needs revisiting)</check>
    <check name="audit_baseline_no_writesignal">! grep -q "WriteSignal" crates/runtime-core/src/drone.rs (audit baseline; Stage B adds the variant)</check>
    <check name="audit_baseline_no_migrations_dir">! Test-Path crates/runtime-drone/migrations (audit baseline; Stage B's first migration creates it)</check>
    <check name="audit_baseline_no_plan_module">! Test-Path crates/runtime-main/src/plan (audit baseline; Stage B creates it)</check>
    <check name="vdr_in_drone">Test-Path crates/runtime-drone/src/vdr.rs (the drone-side projector Stage B's WriteSignal handler calls)</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage B sections B.1–B.4)</file>
    <file>agent-runtime-spec.md §3a (full section, especially Data types + Events + Approval-gate primitive + Loop policy primitive + Failure escalation + Graph integration + Framework JSON), §10 (plans/tasks DDL added M03.5), §2b (signals + structured-emitter delimiter format ⚠️ note)</file>
    <file>docs/MVP-v0.1.md §M4</file>
    <file>docs/gotchas.md (especially #14 snake_case discipline; #34 fmt-first; #36 synthetic-state inversion — Stage B starts the live-event path so the inversion no longer applies; #41 grep-verify-claims)</file>
    <file>docs/build-prompts/retrospectives/M04.A1-retrospective.md (apply [END] Decisions, especially A1 namespace strategy outcome — affects how Stage B's regenerate paths land)</file>
    <file>docs/build-prompts/retrospectives/M04.A2-retrospective.md (apply [END] Decisions, especially the A1 strategy applied to commands.rs + the long-lived events() reconnect outcome)</file>
  </read_first>

  <read_reference>
    <file purpose="xtask codegen archetype Stage A1 established at line 45; Stage B extends to 9 schemas">crates/xtask/src/main.rs</file>
    <file purpose="existing schemas archetype to mirror for plan + task schema authoring; verify $id base URL convention before writing new schemas">schemas/event.v1.json</file>
    <file purpose="hand-curated event variants — 8 plan/task already present (6 spec + 2 extras); Stage B WIRES not re-authors">crates/runtime-core/src/event.rs</file>
    <file purpose="hand-curated DroneCommand enum at line 152+; Stage B adds WriteSignal as new variant">crates/runtime-core/src/drone.rs</file>
    <file purpose="db.rs migration runner archetype; Stage B's first migration file lands under new migrations/ directory">crates/runtime-drone/src/db.rs</file>
    <file purpose="existing command-handler arms (including RevertToSnapshot at line 64); Stage B adds WriteSignal arm">crates/runtime-drone/src/command_handler.rs</file>
    <file purpose="vdr::project_signal at line 50; Stage B's WriteSignal handler calls this">crates/runtime-drone/src/vdr.rs</file>
    <file purpose="actual event-translation pipeline (NOT phantom event_translation.rs); Stage B emits DroneCommand::WriteSignal from here">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="M02 heuristic decision extractor; Stage B replaces with regex on `&lt;&lt;DECISION&gt;&gt;...&lt;&lt;END&gt;&gt;` per spec §2b">crates/runtime-main/src/sdk/decision_extractor.rs</file>
    <file purpose="AgentSdk where the structured-emitter prompt-template plumbing lands">crates/runtime-main/src/sdk/agent_sdk.rs</file>
    <file purpose="graphStore applyEvent archetype with 8 existing plan/task cases; Stage B adds 5 new cases">src/lib/graphStore.ts</file>
    <file purpose="if the A1 namespace strategy renamed the existing error.rs (option a), this provides the archetype for the plan/task generated paths">crates/runtime-core/src/lib.rs</file>
  </read_reference>

  <read_prior_stages>
    <retrospective stage="A1" milestone="M04"/>
    <retrospective stage="A2" milestone="M04"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="B.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="B.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="Key constraints"/>

  <gates milestone="M04"/>

  <self_correction_budget>3</self_correction_budget>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <fan_out_grep>
    <grep pattern="AgentEvent::" purpose="exhaustive matches that must add 5 new arms (plan_approval_requested, plan_revised, plan_aborted, plan_complete, task_skipped); existing 8 plan/task arms stay"/>
    <grep pattern="case 'plan_\|case 'task_\|case &quot;plan_\|case &quot;task_" purpose="graphStore exhaustive switch must add 5 new cases without breaking the 8 existing"/>
    <grep pattern="DroneCommand::" purpose="exhaustive matches; WriteSignal IPC variant adds a new arm; existing 8 arms stay"/>
    <grep pattern="&quot;\$id&quot;" purpose="confirm schema $id base URL convention before authoring plan.v1.json + task.v1.json (M03.5.A retro: https://schemas.aria-runtime.dev/&lt;name&gt;.v1.json)"/>
  </fan_out_grep>

  <runtime_environment os="windows" note="Build agent on Windows 11; Test-Path replaces test -f; named pipe paths differ from Unix sockets in any drone-IPC test; PowerShell-via-bash discipline applies"/>

  <gotchas>
    <trap>AUDIT-GROUNDED EVENT INVENTORY: 6 spec-canonical plan/task variants ALREADY exist in event.rs (PlanCreated, PlanApproved, TaskStarted, TaskCompleted, TaskFailed, TaskEscalated). Stage B WIRES these into the state machine — do NOT re-author them. The 5 NEW variants Stage B adds are the spec-missing ones (plan_approval_requested, plan_revised, plan_aborted, plan_complete, task_skipped). The 2 codebase-only extras (PlanRejected, TaskRolledBack) stay; document disposition in retrospective.</trap>
    <trap>FOLDED-A3 WORK: vdr WriteSignal IPC + structured emitter are Stage B's territory per audit re-staging (NOT Stage A2's). DroneCommand::WriteSignal is a NEW variant on the enum; the drone-side handler arm calls vdr::project_signal at runtime-drone/src/vdr.rs:50 (vdr lives in drone, NOT main; runtime-main has no rusqlite dep). Structured-emitter delimiter format (`&lt;&lt;DECISION&gt;&gt;...&lt;&lt;END&gt;&gt;`) is verbatim per spec §2b ⚠️ note — do NOT change without paired spec edit.</trap>
    <trap>v0.1 hardcodes STANDARD mode + fresh_context_per_task — schemas declare 3 loop policies but only fresh_context_per_task is lit; the `one_shot` and `continuous` variants in the schema are spec-prep, not v0.1 implementation. Stage B's loop-policy seam returns NotImplemented for the other two.</trap>
    <trap>plan.v1.json + task.v1.json $id MUST follow https://schemas.aria-runtime.dev/&lt;name&gt;.v1.json pattern (M03.5.A retro decision; verified by `&lt;fan_out_grep pattern='&quot;$id&quot;'/&gt;` above against existing schemas BEFORE authoring).</trap>
    <trap>5 new event variants in event.v1.json — the renderer's graphStore.applyEvent exhaustive switch will fail to compile if any case is missing. This is the forcing function (gotcha #36 _exhaustive: never); rely on the compiler. Existing 8 cases stay; do NOT touch them.</trap>
    <trap>Plan state machine is a SAFETY PRIMITIVE — coverage gate ≥95%. Document any exclusions inline (likely none — pure-logic module). Per CLAUDE.md §5 + M01.C precedent.</trap>
    <trap>Approval-gate seam (Stage B's deliverable) must be the channel/oneshot the SDK awaits on, NOT the HITL UI itself (Stage E). Stage B's SDK code calls `approval_seam.await_approval(plan_id).await?` and Stage E wires the seam to the HITL flow. Do NOT implement the HITL UI in Stage B.</trap>
    <trap>fresh_context_per_task implementation — clearing the agent's `messages` vec mid-session must NOT clear the SDK's plan-state. Plan state lives in the SDK + SQLite (drone-side), NOT in the agent's message history.</trap>
    <trap>migrations/ directory does NOT exist yet (audit-confirmed). Stage B's migration is the first; version `001`. The db.rs::run_migrations runner currently treats absent dir as zero migrations — Stage B's run is the first non-zero invocation.</trap>
    <trap>WriteSignal IPC payload field shape — `payload: String` (serde-serialized JSON) is the natural shape since `runtime-main` doesn't have rusqlite for direct typed insertion. Drone-side handler deserializes if needed for typed inserts. Document the shape choice in retro.</trap>
    <trap>Per gotcha #41 (grep-verify-claims): every codebase-state claim in this stage's deliverable section is verified against end-of-A2 reality via &lt;pre_flight_check&gt;. If verifications fail (codebase drifted between A2 commit and Stage B execution), surface before proceeding.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT implement the orchestrator agent's prompt template — that's framework-JSON territory (loaded from examples/aria/framework.json at session start). Stage B provides the SDK machinery (state machine + event emission + loop policy + failure escalation + structured-emitter regex); framework JSON wires the orchestrator.</warning>
    <warning>DO NOT wire the renderer's PlanNode/TaskNode to active visual treatment — Stage B's graphStore changes are pass-through state updates only. Stage C builds the visual surface + ApprovalPanel.</warning>
    <warning>DO NOT touch the existing 8 plan/task variants in event.rs or event.v1.json — Stage B's regen target adds only the 5 missing spec variants. The 2 codebase-only extras (PlanRejected, TaskRolledBack) stay; do NOT remove or rename.</warning>
    <warning>DO NOT change the structured-emitter delimiter format (`&lt;&lt;DECISION&gt;&gt;...&lt;&lt;END&gt;&gt;`) without a paired edit to spec §2b ⚠️ note — surface as a design call rather than silently changing.</warning>
    <warning>DO NOT push between stages — Stage B commits locally only.</warning>
    <warning>Stage B's commit MUST include docs/build-prompts/retrospectives/M04.B-retrospective.md in the staged files.</warning>
  </execution_warnings>

  <time_box estimate_hours="6"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage C: which plan-state fields the renderer's PlanNode actually needs to render (likely subset of the full Plan struct — token-spend, status badge, task count); whether the approval-gate seam exposed in Stage B requires renderer-side state reflection (likely yes — the ApprovalPanel needs the plan + risks + hitl_checkpoints); whether _exhaustive: never caught all 5 new event variants in graphStore (forcing function discipline); plan state machine coverage % achieved + any holdouts; disposition of PlanRejected + TaskRolledBack codebase extras (kept additive? renamed? removed via spec follow-up?); WriteSignal payload field shape decision (String JSON vs typed enum); structured-emitter regex DOTALL flag handling.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="B.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD) including M04.B-retrospective.md in staged set</item>
    <item>gate results (each gate, pass/fail; plan state machine coverage % must be ≥95; runtime-drone ≥95% maintained with WriteSignal arm covered)</item>
    <item>schema drift check exit 0 (regenerated 9 schemas: existing 7 from A1 + new plan + task)</item>
    <item>fan_out_grep results — AgentEvent::/case 'plan_*'/case 'task_*' counts before vs after; expected: 8 existing → 13 (5 added)</item>
    <item>WriteSignal IPC integration verification — drone handler arm exists; main-side emission at event_pipeline.rs replaces direct rusqlite (or stub); plan_lifecycle.rs integration test confirms `vdr` table populated after each Decision signal</item>
    <item>generated file shape preview — first 30 lines of crates/runtime-core/src/generated/plan.rs + plan.ts</item>
    <item>plan_lifecycle.rs integration test outcome — full 3-task plan flow end-to-end; SQLite assertions per phase</item>
    <item>structured-emitter unit test outcome — round-trip + multi-decision + malformed-JSON + no-decision cases all pass</item>
    <item>disposition decisions for PlanRejected + TaskRolledBack codebase extras (kept additive / renamed / removed)</item>
    <item>retrospective with [END] decisions for Stage C</item>
    <item>draft commit message from B.6 (filled with session URL)</item>
    <item>"Stage M04.B is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### B.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime): M04 Stage B — §3a Plan & Task primitive + folded-A3 work

Builds the §3a Plan & Task primitive end-to-end + folds the original-A3
work (WriteSignal IPC + structured emitter) per the post-M03.5 audit
re-staging. Plan state machine is a new safety primitive at ≥95%
coverage per CLAUDE.md §5.

New artifacts:
- schemas/plan.v1.json + schemas/task.v1.json (JSON Schema 2020-12;
  $id follows https://schemas.aria-runtime.dev/<name>.v1.json
  convention per M03.5.A retro decision)
- crates/runtime-core/src/generated/plan.rs + generated/task.rs
  (typify-generated; under generated/ per A1 convention — top-level
  paths reserved for hand-curated modules per lib.rs:4-5)
- src/types/plan.ts + task.ts (json-schema-to-typescript-generated)
- crates/runtime-main/src/plan/state_machine.rs — Plan + Task FSM per
  spec §3a (legal transitions, illegal-transition errors, exhaustive
  transition matrix in unit tests; ≥95% coverage gate met)
- crates/runtime-main/src/plan/mod.rs
- crates/runtime-main/src/sdk/prompt_template.rs (folded-A3) —
  structured-emitter prompt-template module injects <<DECISION>>...
  <<END>> format instructions per spec §2b ⚠️ note
- crates/runtime-drone/migrations/001_plans_tasks.sql — first
  migration file; creates the migrations/ directory which did not
  yet exist (audit-confirmed); matches spec §10 DDL added M03.5
- crates/runtime-main/tests/plan_lifecycle.rs — integration test for
  plan-end-to-end flow (orchestrator emits plan_created → approval
  requested → approved → 3 tasks executed → plan_complete; SQLite
  assertions per phase via the new WriteSignal IPC path)

Edits (audit-grounded):
- crates/xtask/src/main.rs: codegen list extended from 7 entries
  (post-A1) to 9 entries (adds plan + task)
- schemas/event.v1.json: 5 missing spec event variants added
  (plan_approval_requested, plan_revised, plan_aborted, plan_complete,
  task_skipped). The 8 already-present plan/task variants stay; 6 are
  spec-canonical (PlanCreated, PlanApproved, TaskStarted, TaskCompleted,
  TaskFailed, TaskEscalated) and 2 are codebase-only extras
  (PlanRejected — distinct from plan_aborted; TaskRolledBack —
  distinct from task_failed). Disposition documented in retro.
- crates/runtime-core/src/event.rs (or generated/event.rs per A1
  namespace strategy): regenerated with 5 new variants; existing
  variants unchanged
- crates/runtime-core/src/drone.rs (folded-A3): WriteSignal { signal_id,
  session_id, payload } variant added to DroneCommand enum at line
  152+; enum grows from 8 to 9 variants
- crates/runtime-drone/src/command_handler.rs (folded-A3): WriteSignal
  handler arm added; calls vdr::project_signal at vdr.rs:50; non-fatal
  projection failure logged via tracing::warn! per spec §2b
- crates/runtime-main/src/sdk/event_pipeline.rs (folded-A3): main-side
  emits DroneCommand::WriteSignal at signal-write call-site (replaces
  direct rusqlite or M02 stub; runtime-main has no rusqlite dep)
- crates/runtime-main/src/sdk/decision_extractor.rs (folded-A3): M02
  line-by-line heuristic replaced with regex on <<DECISION>>(.*?)<<END>>
  delimited blocks; (?s) DOTALL flag for multi-line JSON
- crates/runtime-main/src/sdk/agent_sdk.rs: plan state machine wired
  into SDK event loop; failure-escalation logic (failure_count >=
  max_failures triggers task_escalated); fresh_context_per_task loop
  policy (clears agent messages between tasks; preserves SDK plan
  state in SQLite); structured-emitter prompt-template plumbing
- src/types/agent_event.ts: regenerated with 5 new variants
- src/lib/graphStore.ts: applyEvent exhaustive switch extended with
  5 new cases as pass-through state updates (Stage C builds visual
  treatment); _exhaustive: never forcing function held; existing 8
  cases unchanged

Approval-gate seam exposed (channel/oneshot the SDK awaits on); Stage E
wires the seam to HITL UI.

Carry-forward closures (folded from A2 deferrals into Stage B per audit
re-staging):
- M03 🟡 vdr.rs projector wired at signal-write call-site (via new
  WriteSignal IPC variant + drone-side handler arm calling
  vdr::project_signal; main-side emission at event_pipeline.rs)
- M02 🟡 Decision extractor → structured emitter migration (regex on
  <<DECISION>>...<<END>> delimited blocks per spec §2b ⚠️ note;
  prompt-template module injects format instructions)

v0.1 scope locks held: STANDARD mode hardcoded, fresh_context_per_task
only (one_shot + continuous return NotImplemented), Novice + Promoted
tiers only.

Refs: M04-plan-verify-hitl-budget.md §B, spec §3a, §2b ⚠️, §10 (DDL),
gap-analysis.md M03 + M02 entries 🟡 (2 carry-forward items closed via
fold from A2)
Retrospective: docs/build-prompts/retrospectives/M04.B-retrospective.md

https://claude.ai/code

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE C — §3a Plan UI + ApprovalPanel + graph wiring           -->
<!-- ============================================================ -->

## Stage C — §3a Plan UI + ApprovalPanel + graph wiring (renderer surface for plan/task events)

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://reactflow.dev/api-reference/types/node> — confirm React Flow v12 Node + custom-node API for PlanNode/TaskNode visual upgrades unchanged from M03.B
- <https://v2.tauri.app/develop/calling-rust/> — confirm Tauri 2.x `invoke` API for the approval-flow round-trip (renderer → main → drone → main → renderer); used in `approve_plan` + `revise_plan` + `abort_plan` commands

### C.1 Problem Statement

Stage B exposed the approval-gate seam (channel/oneshot the SDK awaits on); Stage C builds the renderer surface that resolves it.

1. **Wire PlanNode + TaskNode to live event variants.** M03.B-C shipped the components with synthetic-state tests (gotcha #36); Stage B's graphStore now forwards 13 plan/task events (8 already-shipped + 5 newly authored by Stage B) through `applyEvent`. Stage C consumes those state updates: PlanNode shows status badge + task count + cumulative token spend; TaskNode shows status + HITL flag + failure_count when > 0. Per spec §3a Graph integration. **Pre-flight verifies PlanNode/TaskNode synthetic-state assumption still holds** — the audit scope didn't deep-read the components, only confirmed they exist; if Stage B's wiring emitted state shapes that differ from M03.C synthetic fixtures, Stage C's tests follow the new shape.

2. **Build ApprovalPanel.** When `plan_approval_requested` fires, the renderer surfaces an ApprovalPanel showing: plan title, description, risks, hitl_checkpoints, the full task list. User actions: Approve / Revise / Cancel. ARIA non-modal pattern (matches M03.D InspectorPanel discipline). Per spec §3a Approval-gate primitive.

3. **Approval-flow round-trip.** Renderer dispatches one of three Tauri commands on user action:
   - `approve_plan(plan_id)` — main routes to drone → drone resolves the SDK's approval seam → SDK emits `plan_approved` → renderer re-receives via existing event subscription
   - `revise_plan(plan_id, revisions)` — main routes to drone; SDK emits `plan_revised` then awaits new approval
   - `abort_plan(plan_id, reason)` — main routes to drone; SDK emits `plan_aborted`; plan terminates
4. **Plan abort + replan + revise flows.** Wire the three Tauri commands per CLAUDE.md §8.security model (capability declarations on commands; no user data leaked beyond the plan_id + the user's typed revisions).

**Success criterion:** Loading a fixture session that emits `plan_approval_requested`, the ApprovalPanel surfaces; clicking Approve dispatches `approve_plan` and the panel dismisses on `plan_approved` receipt; PlanNode + TaskNode reflect live state transitions through the 3-task plan execution; Playwright E2E test covers the happy path; Vitest tests cover the panel's state machine + the three Tauri command unwrap paths.

**New artifacts:**
- `src/components/ApprovalPanel.tsx` (new)
- `tests/e2e/plan_approval.spec.ts` (new Playwright test)
- `tests/unit/ApprovalPanel.test.tsx` (new Vitest)
- `tests/unit/nodes/PlanNode.test.tsx` + `TaskNode.test.tsx` (extended; Stage B's pass-through state now drives live rendering)

**Edited artifacts:**
- `src/components/nodes/PlanNode.tsx`, `src/components/nodes/TaskNode.tsx` (visual upgrades from synthetic-state to live-event-driven rendering)
- `src/lib/ipc.ts` (add `invokeApprovePlan`, `invokeRevisePlan`, `invokeAbortPlan` typed wrappers)
- `src-tauri/src/commands.rs` (3 new commands; route to drone via `Arc<DroneClient>` per A2 pattern)
- `src/App.tsx` (mount ApprovalPanel based on graph state)
- `CHANGELOG.md`

### C.2 Files to Change

| File | Change |
|---|---|
| `src/components/ApprovalPanel.tsx` | **New** — non-modal panel; shows plan + risks + hitl_checkpoints + task list; Approve/Revise/Cancel actions |
| `src/components/nodes/PlanNode.tsx` | **Edited** — render live state from graphStore (status badge, task count, token spend) |
| `src/components/nodes/TaskNode.tsx` | **Edited** — render live state (status, HITL flag, failure_count) |
| `src/lib/ipc.ts` | **Edited** — add `invokeApprovePlan`, `invokeRevisePlan`, `invokeAbortPlan` |
| `src-tauri/src/commands.rs` | **Edited** — `approve_plan`, `revise_plan`, `abort_plan` Tauri commands; dispatch to drone via existing `Arc<DroneClient>` |
| `src/App.tsx` | **Edited** — mount ApprovalPanel conditionally based on graph state |
| `tests/unit/ApprovalPanel.test.tsx` | **New** — Vitest coverage for panel state machine + action dispatch |
| `tests/unit/nodes/PlanNode.test.tsx`, `TaskNode.test.tsx` | **Edited** — extend with live-event tests (gotcha #36 inversion: now the events DO exist, test via `applyEvent` path) |
| `tests/e2e/plan_approval.spec.ts` | **New** — Playwright happy-path E2E |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes |

### C.3 Detailed Changes

#### `src/components/ApprovalPanel.tsx` — non-modal approval panel

Layout per M03.D InspectorPanel discipline (right-side overlay, dismissible, ARIA `aria-modal="false"`):

```tsx
interface ApprovalPanelProps {
  plan: Plan;
  onApprove: () => void;
  onRevise: (revisions: string) => void;
  onAbort: (reason: string) => void;
  onDismiss: () => void;
}

export function ApprovalPanel({ plan, onApprove, onRevise, onAbort, onDismiss }: ApprovalPanelProps) {
  // Render: plan.title, plan.description, plan.risks (bulleted), plan.hitl_checkpoints (bulleted),
  //         plan.tasks (list with title + acceptance_criteria), three action buttons.
  // Approve: simple click handler.
  // Revise: opens a textarea inline; submit dispatches onRevise(text).
  // Abort: opens a textarea for reason; submit dispatches onAbort(text).
  // Dismiss: optional — does NOT abort; just hides the panel; SDK keeps awaiting.
  // Render in a fixed-position right-side overlay matching InspectorPanel.tsx style.
}
```

ARIA: `role="region"`, `aria-label="Plan approval"`, `aria-modal="false"`. Keyboard: Tab cycles focusable elements; Escape calls onDismiss (not onAbort).

#### `src/components/nodes/PlanNode.tsx` — visual upgrade

Existing M03.C synthetic-state component now drives off `graphStore` plan state. Render per spec §3 Node Types + §3a Graph integration:

- Status badge (color-coded: pending_approval=amber, approved=blue, in_progress=green, complete=gray, aborted=red, awaiting_replan=amber)
- Task count: `{completed}/{total}` (e.g., `2/3`)
- Cumulative token spend (sum of token_in + token_out across child agents; reuses M03.D `tokenScale.ts` weight)
- Title (truncate at 40 chars; full title in InspectorPanel)

Handle convention per spec §3 (top-target / bottom-source for branching nodes); already shipped in M03.B-C — no edge-handle changes.

#### `src/components/nodes/TaskNode.tsx` — visual upgrade

Render per spec §3a:
- Status badge (pending=gray, running=blue, done=green, blocked=amber, failed=red, skipped=gray-strikethrough)
- HITL flag icon when `hitl=true`
- Failure-count badge when `failure_count > 0` (e.g., `⚠ 2/3`)
- Title (truncate at 30 chars)

#### `src-tauri/src/commands.rs` — 3 new Tauri commands

```rust
#[tauri::command]
pub async fn approve_plan(
    plan_id: String,
    client: tauri::State<'_, Arc<DroneClient>>,
) -> Result<(), CmdError> {
    tracing::info!("approve_plan plan_id={}", plan_id);
    client.approve_plan(plan_id).await.map_err(CmdError::from)
}

// Similar shape for revise_plan(plan_id, revisions: String) and abort_plan(plan_id, reason: String).
```

Per CLAUDE.md §8.security + spec §13.5 dev logging: every command logs entry/error/success. Capability adherence: these commands take user-typed text (revisions, reason); pass through to drone as opaque strings; drone-side validates length + sanitizes per existing string-handling pattern.

Drone-side: extend the IPC command enum + handler with `ApprovePlan { plan_id }`, `RevisePlan { plan_id, revisions }`, `AbortPlan { plan_id, reason }`. Each resolves the SDK's approval seam (Stage B's deliverable) with the corresponding outcome.

#### `src/App.tsx` — mount ApprovalPanel

Subscribe to graphStore's plan state. When any plan has `status === 'pending_approval'` AND no other panel is active, render `<ApprovalPanel plan={...} ... />`. On user action, dispatch the corresponding `invoke*` and on success update local UI state (panel dismisses on `plan_approved` event arrival via existing event subscription).

#### `tests/e2e/plan_approval.spec.ts` — Playwright happy path

Test flow: load app → run a fixture smoke session that triggers `plan_approval_requested` (use a scripted fixture; do NOT call live Anthropic) → assert ApprovalPanel visible → click Approve → assert panel dismisses → assert PlanNode status badge transitions to `approved`.

Per gotcha #23, this is renderer-level Playwright (Vite dev server + module-mocked Tauri); not desktop-shell tauri-driver (still disabled per Key constraints).

### C.4 Tests

#### Test files

- `tests/unit/ApprovalPanel.test.tsx` — render assertions (plan fields visible); action-dispatch assertions (mocked `invoke*` calls); keyboard navigation (Escape dismisses, Tab cycles)
- `tests/unit/nodes/PlanNode.test.tsx` extended — live state (status badge transitions, task count, token spend); previous synthetic-state tests preserved as documentation of state shape
- `tests/unit/nodes/TaskNode.test.tsx` extended — live state (status, HITL flag, failure-count badge)
- `tests/e2e/plan_approval.spec.ts` — Playwright happy path

#### Coverage target

- `src/` ≥80% maintained (ApprovalPanel + node components covered by Vitest)
- `runtime-main` + `runtime-drone` ≥95% maintained (3 new Tauri command + 3 new drone IPC variants tested via `*_with` seams + drone integration tests)

### C.5 CLI Prompt

```xml
<work_stage_prompt id="M04.C">
  <context>
    Stage C of M04. §3a Plan UI + ApprovalPanel + graph wiring. Wires Stage B's plan/task event surface to the renderer; builds ApprovalPanel and the approve/revise/abort Tauri command round-trip. Stage B's commit must be on the milestone branch.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage B" subject</check>
    <check name="prior_retrospective_staged">git log -1 --name-only must include docs/build-prompts/retrospectives/M04.B-retrospective.md</check>
    <check name="plan_state_machine_present">Test-Path crates/runtime-main/src/plan/state_machine.rs (Stage B deliverable)</check>
    <check name="plan_schema_present">Test-Path schemas/plan.v1.json AND Test-Path schemas/task.v1.json (Stage B deliverables)</check>
    <check name="approval_seam_exposed">grep -q "approval_seam\|ApprovalSeam\|await_approval" crates/runtime-main/src/plan/mod.rs (Stage B's seam channel must exist; Stage C consumes via the new Tauri commands)</check>
    <check name="arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Stage C's 3 new Tauri commands consume tauri::State&lt;'_, Arc&lt;DroneClient&gt;&gt;)</check>
    <check name="plan_task_node_synthetic">Test-Path src/components/nodes/PlanNode.tsx AND Test-Path src/components/nodes/TaskNode.tsx (M03.C synthetic-state archetypes; Stage C drives live)</check>
    <check name="schema_drift_clean">cargo xtask regenerate-types --check exit 0</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage C sections C.1–C.4)</file>
    <file>agent-runtime-spec.md §3 (Graph Behavior + Visual Design Principles + InspectorPanel layout), §3a (Approval-gate primitive + Graph integration)</file>
    <file>docs/gotchas.md (especially #35 React Flow + happy-dom; #36 synthetic-state inversion now lifted; #37 trust TS narrowing; #41 grep-verify-claims)</file>
    <file>docs/build-prompts/retrospectives/M04.B-retrospective.md (apply [END] Decisions, especially: which plan-state fields PlanNode actually needs to render; ApprovalPanel state-shape requirements; PlanRejected/TaskRolledBack disposition)</file>
  </read_first>

  <read_reference>
    <file purpose="M03.D InspectorPanel layout archetype + ARIA non-modal pattern + dismissal semantics">src/components/InspectorPanel.tsx</file>
    <file purpose="existing PlanNode synthetic-state component to drive live (M03.C archetype; Stage B may have added pass-through state-update calls)">src/components/nodes/PlanNode.tsx</file>
    <file purpose="existing TaskNode synthetic-state component to drive live">src/components/nodes/TaskNode.tsx</file>
    <file purpose="Tauri command archetype + Arc&lt;DroneClient&gt; managed-state pattern from A2; Stage C adds 3 new commands following same shape">src-tauri/src/commands.rs</file>
    <file purpose="renderer-side typed invoke wrapper archetype with generated CmdError unwrap (A2 pattern)">src/lib/ipc.ts</file>
    <file purpose="Playwright renderer-level archetype with module-mocked Tauri">tests/e2e/smoke.spec.ts</file>
    <file purpose="graphStore applyEvent extended in Stage B with 5 new cases (5 missing spec variants); 8 existing plan/task cases unchanged">src/lib/graphStore.ts</file>
    <file purpose="M03.D tokenScale.ts archetype for token-spend rendering on PlanNode (DO NOT reinvent the formula renderer-side; import the helper)">src/lib/tokenScale.ts</file>
    <file purpose="Stage B's plan state machine + approval_seam — Stage C's 3 new Tauri commands resolve the seam via drone IPC">crates/runtime-main/src/plan/mod.rs</file>
  </read_reference>

  <read_prior_stages>
    <retrospective stage="A1" milestone="M04"/>
    <retrospective stage="A2" milestone="M04"/>
    <retrospective stage="B" milestone="M04"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="C.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="C.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="Key constraints"/>

  <gates milestone="M04"/>

  <self_correction_budget>3</self_correction_budget>

  <runtime_environment os="windows"/>

  <gotchas>
    <trap>ApprovalPanel is NON-MODAL (aria-modal="false") — graph stays interactive underneath. Don't follow Modal pattern from gotcha-37 territory; mirror M03.D InspectorPanel.</trap>
    <trap>Stage B's graphStore handles 8 existing + 5 new plan/task event variants as pass-through state. Stage C consumes the state — does NOT re-implement event handling. If a new visual treatment needs a new state field, add it to Stage B's graphStore (via an amendment commit pre-Stage-C if needed) rather than computing it renderer-side.</trap>
    <trap>The 3 new Tauri commands (approve_plan, revise_plan, abort_plan) take user-typed strings (revisions + reason). Pass-through opaque per CLAUDE.md §8.security; do NOT parse/interpret the user input on the renderer side beyond length validation. Drone-side validates length + sanitizes per existing string-handling pattern.</trap>
    <trap>The 3 new Tauri commands consume `tauri::State&lt;'_, Arc&lt;DroneClient&gt;&gt;` registered by A2; do NOT re-spawn drone subprocesses or construct fresh DroneClient instances. The managed-state pattern A2 established is the only approved access path.</trap>
    <trap>Drone-side IPC for the 3 new commands — Stage C extends the DroneCommand enum (in `crates/runtime-core/src/drone.rs`) with `ApprovePlan`, `RevisePlan`, `AbortPlan` variants that resolve Stage B's approval_seam. Each new variant grows the exhaustive match in `command_handler.rs` — fan-out grep `DroneCommand::` enumerates all matches.</trap>
    <trap>Playwright test uses module-mocked Tauri (renderer-level), NOT tauri-driver (still disabled per Key constraints). Reuse the M02.E test setup pattern.</trap>
    <trap>Token-spend on PlanNode reuses M03.D tokenScale.ts — DO NOT re-implement the formula renderer-side; import the helper.</trap>
    <trap>Per gotcha #41 (grep-verify-claims): every codebase claim in this stage is verified against post-B reality via &lt;pre_flight_check&gt;. If verifications fail, surface drift before proceeding.</trap>
  </gotchas>

  <fan_out_grep>
    <grep pattern="DroneCommand::" purpose="exhaustive matches; Stage C's 3 new variants (ApprovePlan, RevisePlan, AbortPlan) extend the enum"/>
    <grep pattern="invoke('approve_plan\|invoke('revise_plan\|invoke('abort_plan" purpose="renderer-side invoke calls; should be exactly 3 callsites (one per command); ipc.ts wrappers are the canonical entry"/>
  </fan_out_grep>

  <execution_warnings>
    <warning>DO NOT touch the SDK approval seam (Stage B's deliverable) — Stage C only consumes its result via the event stream. The drone-side resolution of the seam happens via the 3 new IPC commands.</warning>
    <warning>DO NOT add new graph state fields without Stage B amendment — if Stage C needs them, surface and pause; the right fix is in Stage B's store.</warning>
    <warning>DO NOT touch the existing 8 plan/task graphStore cases — Stage B authored 5 new cases as pass-through; Stage C drives them visual.</warning>
    <warning>DO NOT push between stages.</warning>
    <warning>Stage C's commit MUST include docs/build-prompts/retrospectives/M04.C-retrospective.md in the staged files.</warning>
  </execution_warnings>

  <time_box estimate_hours="4"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage D: any new gotchas about React Flow + non-modal panels (gotcha #35 act() warnings still apply); whether the approval round-trip latency is acceptable (renderer→Tauri→drone→SDK→drone→Tauri→renderer); any UI patterns for the multi-action panel (Approve/Revise/Cancel) that future panels (Verify rollback prompt, HITL panels) should mirror.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="C.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD) including M04.C-retrospective.md in staged set</item>
    <item>gate results (each gate, pass/fail; src/ coverage maintained ≥80%; runtime-main + runtime-drone ≥95% with 3 new IPC arms covered)</item>
    <item>fan_out_grep results — DroneCommand:: + invoke('approve_plan/etc count verification</item>
    <item>Playwright plan_approval.spec.ts pass result</item>
    <item>screenshot or DOM snapshot of ApprovalPanel rendered with a sample plan</item>
    <item>retrospective with [END] decisions for Stage D</item>
    <item>draft commit message from C.6</item>
    <item>"Stage M04.C is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### C.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(renderer): M04 Stage C — §3a Plan UI + ApprovalPanel + graph wiring

Wires Stage B's plan/task event surface to the renderer. PlanNode +
TaskNode upgrade from M03.C synthetic-state to live-event-driven
rendering; ApprovalPanel surfaces plan_approval_requested events;
3 new Tauri commands (approve_plan, revise_plan, abort_plan) route
the user's decision back through main → drone → SDK approval seam.

New artifacts:
- src/components/ApprovalPanel.tsx — non-modal right-side panel per
  M03.D InspectorPanel discipline (aria-modal="false"); shows plan
  title + description + risks + hitl_checkpoints + task list;
  Approve/Revise/Cancel actions; ARIA + keyboard nav.
- tests/e2e/plan_approval.spec.ts — Playwright happy-path E2E
  (renderer-level; module-mocked Tauri per gotcha #23 + Key
  constraints; tauri-driver stays disabled).
- tests/unit/ApprovalPanel.test.tsx — Vitest coverage for panel
  state machine + action dispatch.

Edits:
- src/components/nodes/PlanNode.tsx + TaskNode.tsx: live state
  rendering (status badges, task count, token spend via M03.D
  tokenScale.ts; failure-count badge; HITL flag). Existing
  synthetic-state tests preserved.
- src/lib/ipc.ts: invokeApprovePlan/RevisePlan/AbortPlan typed
  wrappers using generated CmdError unwrap (Stage A2 pattern).
- src-tauri/src/commands.rs: 3 new commands routing to drone via
  Arc<DroneClient> Tauri-managed-state (Stage A2 pattern).
- src/App.tsx: ApprovalPanel mount on graphStore plan state.

Approval round-trip: renderer → main → drone → SDK approval seam
(Stage B) → emits plan_approved/plan_revised/plan_aborted →
renderer re-receives via existing event subscription → panel
dismisses.

Refs: M04-plan-verify-hitl-budget.md §C, spec §3 + §3a
Retrospective: docs/build-prompts/retrospectives/M04.C-retrospective.md

https://claude.ai/code
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE D — §4a Verify & Rails primitive                         -->
<!-- ============================================================ -->

## Stage D — §4a Verify & Rails primitive (hooks + rails + don't-touch; consume existing verify_* events + RevertToSnapshot)

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.rs/tokio/latest/tokio/process/struct.Command.html> — `tokio::process::Command` for shell hook execution (cross-platform); already in use for drone subprocess (Stage A2)
- <https://learn.microsoft.com/en-us/powershell/scripting/lang-spec/chapter-01> — PowerShell wrapper for Windows shell hooks per spec §M4 ("PowerShell wrapper" acceptance criterion); verify exact invocation pattern (`pwsh -NoProfile -Command "..."` vs `powershell.exe -NoProfile -Command`)
- <https://docs.rs/globset/latest/globset/> — globset for don't-touch glob matching; verify version pin or vendor decision
- <https://json-schema.org/draft/2020-12/schema> — hook.v1.json author against this draft

### D.1 Problem Statement

§4a Verify & Rails is a major M04 deliverable. Spec §4a locks the primitives (Hook + HookRef + HookCategory + 7 firing points + Rails hard/soft + don't-touch); Stage D builds them. **Audit-grounded scope:** the 4 verify+rails event variants (`VerifyStarted`, `VerifyPassed`, `VerifyFailed`, `RailTriggered`) ALREADY exist in `crates/runtime-core/src/event.rs:220–248` and `schemas/event.v1.json` — Stage D WIRES them to the new Hook executor + Rails evaluator (does NOT re-author or rename them as `hook_*`). The `RevertToSnapshot` drone command + `RevertReason` enum + handler arm at `crates/runtime-drone/src/command_handler.rs:64` ALREADY exist in `crates/runtime-core/src/drone.rs:183, :266` — Stage D CONSUMES them (does NOT re-add). The genuinely-new work: Hook primitive, Rails primitive, don't-touch glob matcher, `pre_file_edit` firing point, cross-platform shell wrapper.

1. **Hook primitive** — `schemas/hook.v1.json` declares `HookRef = shell{command,timeout_ms?,cwd?} | tool{tool_name,input?} | agent{agent_id,prompt?}` and `HookCategory = verify | lint | build | test | custom`. Hook execution: shell variant spawns subprocess via `tokio::process::Command` with cross-platform PowerShell wrapper on Windows (`pwsh -NoProfile -Command "..."`); tool variant invokes the runtime tool dispatcher; agent variant spawns a child agent.

2. **Seven firing points** — existing 6 (`pre_task`, `post_task`, `post_file_edit`, `pre_commit`, `pre_agent_spawn`, `session_end`) + new `pre_file_edit` for don't-touch interception. Spec §4a's firing-point table currently lists 6; Stage D's `hook.v1.json` adds the 7th. Spec edit lands in-stage (if <5 lines) or as a post-M04 follow-up doc PR (analogous to M03.5 pattern) — decision in retro.

3. **Rails primitive** — `Rails { hard: Rail[], soft: Rail[] }` declared in framework JSON. Each `Rail` has `id`, `fires_on` (firing-point reference), `check` (JSONLogic expression), `message`. Hard rails block; soft rails warn. Emits the existing `rail_triggered { rail_id, policy: 'hard' | 'soft', firing_point, message, agent_id? }` event from `event.rs:241` (already there; Stage D fires it from the new Rails evaluator).

4. **Don't-touch primitive** — glob array in framework JSON; built-in pre-edit rail; fires on the new `pre_file_edit` firing point. Implementation: every `Write` tool call from any agent intercepts at the rail evaluator; if any glob matches, emit `rail_triggered { rail_id: 'dont_touch', policy: 'hard' }` and block the edit.

5. **`RevertToSnapshot` drone command — CONSUME, do NOT re-add.** The variant already exists at `crates/runtime-core/src/drone.rs:183` with the `RevertReason` enum at `:266` (variants: `HookRollback`, `UserRollback`, `GapRecovery` — all currently UNIT variants, no fields). The drone-side handler arm exists at `crates/runtime-drone/src/command_handler.rs:64`. Stage D consumes the existing variant from the Hook executor's `on_failure: rollback` path. **Design surface (decided at execution time):** spec §4a's `revert_to_snapshot` description implies `HookRollback` should carry a `hook_id: String` field (so the resulting `task_failed` event can name which hook caused the rollback); code currently has it as a unit variant. Stage D either (a) extends `RevertReason::HookRollback` to carry `hook_id`, paired with a small spec text edit confirming the field, OR (b) keeps the unit variant and updates spec to match codebase, OR (c) defers the field-add to a v1.0 milestone. Decision documented in retrospective.

6. **VerifyNode + HookNode wired to live events.** Already shipped as M03.C synthetic-state components; Stage D wires them to the existing 4 live event variants (`verify_started`, `verify_passed`, `verify_failed`, `rail_triggered` — codebase NAMES, NOT `hook_*`). The `graphStore.applyEvent` exhaustive switch already handles these 4 variants per M03 — Stage D leaves the switch alone; only the visual treatment in `VerifyNode.tsx` + `HookNode.tsx` changes.

**Success criterion:** Loading a fixture framework JSON with a `post_task` hook ("`bash .aria/verify.sh`" on Linux/macOS; `pwsh -NoProfile -Command ".aria\verify.ps1"` on Windows) running after each task; pass → emits `verify_passed` → next task; fail with `on_failure: rollback` → drone reverts via existing `RevertToSnapshot`, task retries; rail violations on `pre_file_edit` fire `rail_triggered` events with don't-touch glob match; Playwright + integration tests cover the flows; ≥95% coverage on the new hooks/ + rails/ modules (capability-enforcer-adjacent safety primitives per CLAUDE.md §5).

**New artifacts:**
- `schemas/hook.v1.json` (new)
- `crates/runtime-core/src/generated/hook.rs` (new; generated under `generated/` per A1 convention)
- `src/types/hook.ts` (new; generated)
- `crates/runtime-main/src/hooks/mod.rs`, `crates/runtime-main/src/hooks/executor.rs`, `crates/runtime-main/src/hooks/rails.rs`, `crates/runtime-main/src/hooks/dont_touch.rs` (new)
- `crates/runtime-main/src/hooks/shell.rs` (new — cross-platform shell wrapper)
- `crates/runtime-main/tests/hook_integration.rs` (new — full lifecycle integration test)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (codegen list: add `hook`)
- `crates/runtime-main/src/sdk/agent_sdk.rs` (wire Hook executor at firing-point sites; intercept Write tool dispatch for `pre_file_edit`)
- `src/components/nodes/VerifyNode.tsx`, `src/components/nodes/HookNode.tsx` (synthetic-state → live-event-driven; already-shipped event variants drive)
- Possibly `crates/runtime-core/src/drone.rs` (`RevertReason::HookRollback` shape decision — extend with `hook_id: String` per execution-time call)
- Possibly `agent-runtime-spec.md` §4a (add `pre_file_edit` to firing-point table — in-stage if <5 lines, else follow-up PR; AND/OR edit to confirm `hook_id` field on `HookRollback` if Stage D extends the variant)
- `CHANGELOG.md`

**Items NOT in Stage D scope (audit-grounded; would have been incorrectly listed in the previous draft):**
- Authoring `hook_started`/`hook_passed`/`hook_failed`/`rail_triggered` events — they ALREADY exist as `verify_started`/`verify_passed`/`verify_failed`/`rail_triggered` per codebase NAMES.
- Authoring `RevertToSnapshot` variant on `DroneCommand` — already at `drone.rs:183`.
- Authoring drone-side handler for `RevertToSnapshot` — already at `command_handler.rs:64`.
- Extending `graphStore.applyEvent` for verify/rails events — already there per M03.

### D.2 Files to Change

| File | Change |
|---|---|
| `schemas/hook.v1.json` | **New** — Hook + HookRef + HookCategory + Rail per spec §4a |
| `crates/runtime-core/src/generated/hook.rs`, `src/types/hook.ts` | **New** — generated under `generated/` per A1 convention |
| `crates/xtask/src/main.rs` | **Edited** — codegen list extends from 9 entries (post-B) to 10 entries (adds `hook`) |
| `crates/runtime-main/src/hooks/{mod,executor,rails,dont_touch,shell}.rs` | **New** — Hook/Rails/don't-touch implementation |
| `crates/runtime-main/src/sdk/agent_sdk.rs` | **Edited** — wire Hook executor at firing-point sites (existing `pre_task`/`post_task`/etc); intercept Write tool dispatch for new `pre_file_edit` |
| `crates/runtime-main/tests/hook_integration.rs` | **New** — full lifecycle integration test |
| `src/components/nodes/VerifyNode.tsx`, `HookNode.tsx` | **Edited** — synthetic-state → live-event-driven wiring (events already exist; only visual treatment changes) |
| `crates/runtime-core/src/drone.rs` | **Edited (conditional)** — `RevertReason::HookRollback` may extend with `hook_id: String` field per Stage D design decision; otherwise unchanged |
| `agent-runtime-spec.md` §4a | **Edited (or follow-up note)** — add `pre_file_edit` to firing-point table (in-stage if <5 lines, else post-M04 follow-up PR); plus optional confirmation of `HookRollback` field-add per design decision |
| `CHANGELOG.md` | **Edited** — Stage D summary; design-decision outcomes noted |

**Files explicitly NOT in this table** (audit-grounded; already exist per codebase reality):
- `schemas/event.v1.json` — Stage D does NOT touch (4 verify+rails events already present)
- `crates/runtime-core/src/event.rs` — Stage D does NOT touch
- `src/types/agent_event.ts` — Stage D does NOT regenerate (no new event variants)
- `crates/runtime-drone/src/snapshot.rs` — Stage D does NOT extend (RevertToSnapshot handler at `command_handler.rs:64` already calls into this; no new handler logic needed unless Stage D extends `RevertReason::HookRollback`)
- `src/lib/graphStore.ts` — Stage D does NOT touch (verify/rails cases already handled per M03)

### D.3 Detailed Changes

#### `schemas/hook.v1.json` — Hook + HookRef + HookCategory + Rail schema

Author per spec §4a TypeScript shapes. Key invariants:
- HookRef discriminator on `type` (shell|tool|agent) per `serde(tag="type", rename_all="snake_case")` convention
- Firing points enum: `pre_task`, `post_task`, `pre_file_edit`, `post_file_edit`, `pre_commit`, `pre_agent_spawn`, `session_end` (7 values; new `pre_file_edit` is the don't-touch interception point)
- Hook on_failure enum: `block | warn | rollback`
- Rail check field: opaque string (JSONLogic expression evaluated at runtime per gotcha #18 — operator allowlist)

Pre-flight per gotcha #14: snake_case schema discipline; verify $id pattern.

#### `crates/runtime-main/src/hooks/shell.rs` — cross-platform shell execution

**Cross-stack glue point — gotcha #32 verbatim-quote-or-verify discipline applies.** Per upstream `tokio::process::Command` docs (URL in WEBCHECK):

```rust
// Cross-platform shell wrapper. On Windows uses pwsh.exe -NoProfile -Command;
// on Linux/macOS uses bash -c. Verify against tokio::process docs URL in
// WEBCHECK before authoring; pwsh.exe is preferred over powershell.exe per
// Microsoft's PowerShell 7+ guidance (URL in WEBCHECK).
pub async fn execute_shell(
    command: &str,
    timeout_ms: Option<u64>,
    cwd: Option<&Path>,
) -> Result<HookOutcome, HookError> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = tokio::process::Command::new("pwsh.exe");
        c.args(["-NoProfile", "-Command", command]);
        c
    } else {
        let mut c = tokio::process::Command::new("bash");
        c.args(["-c", command]);
        c
    };
    if let Some(d) = cwd { cmd.current_dir(d); }
    cmd.kill_on_drop(true);
    // ... timeout + spawn + wait + capture + map to HookOutcome
}
```

`<execution_warnings>` flags this with explicit "verify against pwsh -NoProfile semantics on the target Windows version before shipping" guard per gotcha #32.

#### `crates/runtime-main/src/hooks/dont_touch.rs` — pre_file_edit interception

Globset-backed matcher. Every Write tool call (existing tool dispatcher, location TBD) routes through the rail evaluator BEFORE the OS write. If any glob in `framework.dont_touch` matches the target path:
- Emit `rail_triggered { rail_id: 'dont_touch', policy: 'hard', firing_point: 'pre_file_edit', message: '<path> matches dont_touch glob: <pattern>' }`
- Block the write; return error to the calling agent

Coverage: ≥95% per safety-primitive gate. Tests: matched-glob blocks; unmatched-glob allows; multi-glob match (first match wins; emit only once); empty dont_touch list passes through.

#### `crates/runtime-main/src/hooks/rails.rs` — Rails primitive

Hard + soft rail evaluation per JSONLogic operator allowlist (gotcha #18: `var, ==, !=, <, <=, >, >=, and, or, not, in, +, -, *, /`). Anything beyond the allowlist returns `RailError::UnsupportedOperator`. Hard rails on match → block + emit `rail_triggered { policy: 'hard', ... }`. Soft rails on match → warn + emit `rail_triggered { policy: 'soft', ... }`.

#### `crates/runtime-core/src/drone.rs` + `command_handler.rs` — CONSUME existing RevertToSnapshot (audit-grounded)

**Audit-grounded.** The variant + handler already exist:

```rust
// crates/runtime-core/src/drone.rs:183 — ALREADY PRESENT
pub enum DroneCommand {
    // ... 7 other variants
    RevertToSnapshot {
        snapshot_id: String,
        reason: RevertReason,
    },
    // ...
}

// crates/runtime-core/src/drone.rs:266 — ALREADY PRESENT (UNIT variants)
pub enum RevertReason {
    HookRollback,
    UserRollback,
    GapRecovery,
}
```

The drone-side handler arm is at `crates/runtime-drone/src/command_handler.rs:64` (verified). Stage D's Hook executor invokes the existing variant via `client.send_command(DroneCommand::RevertToSnapshot { snapshot_id, reason: RevertReason::HookRollback })` from the `on_failure: rollback` path; no new variant authoring needed.

**Design decision at execution time — `RevertReason::HookRollback` shape.** Spec §4a's `revert_to_snapshot` description implies `HookRollback` should carry a `hook_id: String` field so downstream `task_failed { error: 'rolled_back_after_hook_<hook_id>' }` events can name the cause. Code currently has it as a unit variant. Stage D picks one of three strategies and documents the choice in retro:

(a) **Extend `RevertReason::HookRollback` to carry `hook_id`** — paired with a small spec text edit confirming the field. Migration: existing `RevertReason::HookRollback` callsites (likely zero in current code; the variant is unused pre-M04) update to `RevertReason::HookRollback { hook_id: ... }`. Cleanest semantically.

(b) **Keep the unit variant; update spec to match** — spec §4a edit removes the `hook_id` implication; `task_failed` event uses a generic "rolled_back_after_hook" error string without hook_id. Less informative downstream but minimal code change.

(c) **Defer the field-add to v1.0** — keep both code and spec as-is for v0.1 (each says different things); add a v1.0-roadmap note. Trades short-term correctness for longest deferral.

Recommended baseline: (a). Decision documented in retrospective + applied consistently across `drone.rs`, `command_handler.rs` (if existing handler arm uses the variant), spec §4a, and Stage D's Hook executor `on_failure: rollback` path.

#### Spec §4a follow-up (firing-point table)

Spec §4a's firing-point table currently lists 6 firing points; Stage D's `hook.v1.json` adds `pre_file_edit` as the 7th. Two options:
1. Land a small spec edit in Stage D (add `pre_file_edit` to the table; <5 lines).
2. Defer to a post-M04 doc PR (analogous to M03.5 pattern).

Decision per retro: option 1 if the spec edit is < 5 lines and self-contained; option 2 if it ripples to other §4a content.

If Stage D also extends `RevertReason::HookRollback` per the design decision above, both edits land in the same spec PR (in-stage or follow-up).

### D.4 Tests

#### Pedantic-pass preflight

Apply to: `hooks/executor.rs`, `hooks/rails.rs`, `hooks/dont_touch.rs`, `hooks/shell.rs`. Generated files exempt.

#### Test files

- `crates/runtime-main/src/hooks/{executor,rails,dont_touch}.rs` — unit tests for each module per CLAUDE.md §5 default test plan (N unit + M integration)
- `crates/runtime-main/src/hooks/shell.rs` — `*_with` testable seam (`execute_shell_with(spawn_fn, ...)`); cross-platform tests via cfg-gated mock spawner
- `crates/runtime-main/tests/hook_integration.rs` (new) — full hook lifecycle: post_task hook fires after task_completed; on_failure: rollback drives drone revert_to_snapshot; verify task retries
- `tests/unit/nodes/VerifyNode.test.tsx`, `HookNode.test.tsx` extended with live-event paths

#### Coverage target

- `crates/runtime-main/src/hooks/` ≥95% (capability-enforcer-adjacent safety primitive)
- workspace ≥80%
- `runtime-main` ≥95% maintained
- `shell.rs` real OS-spawn wrapper excluded per existing M02 + A2 pattern (testable seam covered; wrapper structurally untestable cross-platform)

### D.5 CLI Prompt

```xml
<work_stage_prompt id="M04.D">
  <context>
    Stage D of M04. §4a Verify & Rails primitive. Audit-grounded scope: 4 verify+rails event variants ALREADY EXIST in `crates/runtime-core/src/event.rs` (`VerifyStarted:220`, `VerifyPassed:227`, `VerifyFailed:234`, `RailTriggered:241`) — Stage D WIRES them to the new Hook executor + Rails evaluator (does NOT re-author or rename to `hook_*`). `RevertToSnapshot` ALREADY EXISTS at `crates/runtime-core/src/drone.rs:183` with `RevertReason` enum at `:266` (variants: `HookRollback`, `UserRollback`, `GapRecovery` — all currently UNIT variants); existing handler arm at `crates/runtime-drone/src/command_handler.rs:64`. Stage D CONSUMES the existing variant from the Hook executor's `on_failure: rollback` path; does NOT re-add. Genuinely-new work: Hook primitive (HookRef + 3 variants × 7 firing points including the new `pre_file_edit`); Rails primitive (hard/soft + JSONLogic operator allowlist per gotcha #18); don't-touch glob matcher (firing on the new `pre_file_edit` point); cross-platform PowerShell shell wrapper (gotcha #32 cross-stack discipline). Design surface: `RevertReason::HookRollback` is currently a UNIT variant in code; spec §4a implies `hook_id: String` field — Stage D decides at execution time per D.3 (extend variant, update spec to match, or defer to v1.0). Cross-stack risk: shell hook execution + cross-platform PowerShell wrapper + Tool dispatcher integration for pre_file_edit. Stage C's commit must be on the milestone branch.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage C" subject</check>
    <check name="prior_retrospective_staged">git log -1 --name-only must include docs/build-prompts/retrospectives/M04.C-retrospective.md</check>
    <check name="pwsh_available">where.exe pwsh.exe must succeed (or fall back to powershell.exe with explicit retro decision)</check>
    <check name="audit_baseline_verify_events">grep -q "VerifyStarted\|VerifyPassed\|VerifyFailed\|RailTriggered" crates/runtime-core/src/event.rs (audit baseline; 4 events must all be present — Stage D does NOT re-author them)</check>
    <check name="audit_baseline_revert_to_snapshot">grep -q "RevertToSnapshot" crates/runtime-core/src/drone.rs (audit baseline; variant must be present at line 183 — Stage D consumes, does NOT re-add)</check>
    <check name="audit_baseline_revert_handler">grep -q "DroneCommand::RevertToSnapshot" crates/runtime-drone/src/command_handler.rs (audit baseline; existing handler arm at line 64 — Stage D extends only if HookRollback variant gains a hook_id field)</check>
    <check name="audit_baseline_revert_reason_unit">grep -A 5 "pub enum RevertReason" crates/runtime-core/src/drone.rs | grep -q "HookRollback,$" (audit baseline; HookRollback is currently a UNIT variant — Stage D decides at execution whether to extend)</check>
    <check name="arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Stage D's Hook executor consumes via the managed-state)</check>
    <check name="schema_drift_clean">cargo xtask regenerate-types --check exit 0</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage D sections D.1–D.4)</file>
    <file>agent-runtime-spec.md §4a (full section, especially firing-point table + revert_to_snapshot description for HookRollback shape decision)</file>
    <file>docs/MVP-v0.1.md §M4 (PowerShell wrapper acceptance criterion)</file>
    <file>docs/gotchas.md (especially #18 JSONLogic operator allowlist; #32 cross-stack discipline applies to shell.rs; #41 grep-verify-claims)</file>
    <file>docs/build-prompts/retrospectives/M04.C-retrospective.md (apply [END] Decisions)</file>
  </read_first>

  <read_reference>
    <file purpose="cross-stack archetype: tokio::process::Command from Stage A2 drone subprocess spawn">src-tauri/src/drone_lifecycle.rs</file>
    <file purpose="EXISTING RevertToSnapshot variant at line 183 + RevertReason enum at line 266 (unit variants); Stage D consumes; design decision on HookRollback shape lives here">crates/runtime-core/src/drone.rs</file>
    <file purpose="EXISTING handler arm at line 64; Stage D's Hook executor invokes via DroneClient; only extend if HookRollback variant gains hook_id field">crates/runtime-drone/src/command_handler.rs</file>
    <file purpose="snapshot read API used by the existing RevertToSnapshot handler; Stage D extends if the field-add design decision lands">crates/runtime-drone/src/snapshot.rs</file>
    <file purpose="EXISTING 4 verify+rails event variants at event.rs:220-248; Stage D wires Hook executor to fire these (NOT new hook_* events)">crates/runtime-core/src/event.rs</file>
    <file purpose="existing tool dispatcher where don't-touch pre_file_edit interception lands; locate via fan_out_grep on Write tool dispatch">crates/runtime-main/src/sdk/agent_sdk.rs</file>
    <file purpose="VerifyNode/HookNode synthetic components from M03.C; Stage D drives live with existing event variants">src/components/nodes/VerifyNode.tsx</file>
    <file purpose="VerifyNode/HookNode synthetic components from M03.C">src/components/nodes/HookNode.tsx</file>
    <file purpose="graphStore exhaustive switch already handles 4 verify/rails events per M03; Stage D does NOT touch this file">src/lib/graphStore.ts</file>
  </read_reference>

  <read_prior_stages>
    <retrospective stage="A1" milestone="M04"/>
    <retrospective stage="A2" milestone="M04"/>
    <retrospective stage="B" milestone="M04"/>
    <retrospective stage="C" milestone="M04"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="D.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="D.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="Key constraints"/>

  <gates milestone="M04"/>

  <self_correction_budget>3</self_correction_budget>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <fan_out_grep>
    <grep pattern="firing_point" purpose="all firing-point references — enum + matchers + tests; new pre_file_edit needs all callsites updated to handle the 7th value"/>
    <grep pattern="DroneCommand::RevertToSnapshot" purpose="exhaustive matches; Stage D's Hook executor adds a new callsite (do NOT add a new enum variant — the existing variant is consumed)"/>
    <grep pattern="RevertReason::" purpose="all RevertReason variant constructions; if Stage D extends HookRollback to carry hook_id, all callsites update together"/>
    <grep pattern="invoke('write_file\|Write\\.\\(\|tool_dispatch.*write\|dispatch.*[Ww]rite" purpose="locate the Write tool dispatch site for pre_file_edit interception (do NOT trust 'Write tool' literal pattern alone — actual symbol may vary)"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="globset" min_version="0.4"/>
    <dep name="tokio" required_features="process,time"/>
  </dependency_audit_check>

  <runtime_environment os="windows" note="PowerShell wrapper required per MVP §M4 acceptance criterion; pwsh.exe preferred over powershell.exe; verify availability at pre_flight_check"/>

  <gotchas>
    <trap>AUDIT-GROUNDED EVENT NAMES: 4 verify+rails event variants ALREADY EXIST in event.rs (VerifyStarted:220, VerifyPassed:227, VerifyFailed:234, RailTriggered:241). Stage D WIRES the new Hook executor + Rails evaluator to fire THESE — does NOT author new hook_started/passed/failed events. The Hook *primitive* (HookRef + HookCategory + Hook in hook.v1.json) is genuinely new; the *event variants* are not.</trap>
    <trap>AUDIT-GROUNDED RevertToSnapshot: variant ALREADY EXISTS at drone.rs:183 with handler arm at command_handler.rs:64. Stage D CONSUMES the existing variant — does NOT re-add. The fan_out_grep on `DroneCommand::RevertToSnapshot` above counts callsites; Stage D's Hook executor adds a new callsite, not a new arm.</trap>
    <trap>DESIGN DECISION at execution time — RevertReason::HookRollback shape. Currently a UNIT variant (drone.rs:266); spec §4a implies hook_id: String field. Stage D picks: (a) extend variant + update spec; (b) keep unit + update spec to match; (c) defer to v1.0. Document choice in retro [END] Decisions; apply consistently across drone.rs, command_handler.rs (if existing handler uses the variant), spec §4a, Hook executor on_failure: rollback path. Recommended baseline: (a).</trap>
    <trap>shell.rs cross-platform — gotcha #32 cross-stack discipline applies. Verify pwsh.exe -NoProfile -Command semantics against current Microsoft PowerShell docs URL (WEBCHECK) BEFORE authoring; do NOT assume bash -c semantics carry over.</trap>
    <trap>JSONLogic operator allowlist (gotcha #18) — Rails check field. Operators beyond the allowlist return UnsupportedOperator; do NOT silently extend the operator set.</trap>
    <trap>Don't-touch glob matcher fires on pre_file_edit — every Write tool call routes through it BEFORE the OS write. If the rail evaluator is async, ensure the Write call awaits the rail decision; otherwise edits land before the rail blocks. Locate the Write dispatch site via fan_out_grep above (do NOT trust the literal 'Write tool' pattern — actual symbol may differ).</trap>
    <trap>Spec §4a's firing-point table doesn't list pre_file_edit — Stage D adds it via hook.v1.json; spec text edit is either in-stage (if &lt;5 lines) or a follow-up doc PR (decide per retro). Do NOT silently add pre_file_edit to the spec without a deliberate edit + commit message note.</trap>
    <trap>Per gotcha #41 (grep-verify-claims): every codebase claim in this stage is verified against post-C reality via &lt;pre_flight_check&gt;. The audit baseline checks (verify_events, RevertToSnapshot, RevertReason unit variants) are load-bearing — if any fail, surface drift before proceeding.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT author new event variants for hook_started/hook_passed/hook_failed — the existing verify_started/passed/failed at event.rs:220-234 are the codebase NAMES. Wire the new Hook executor to fire these; do NOT rename or add parallel variants.</warning>
    <warning>DO NOT re-add RevertToSnapshot variant or its handler arm — both already exist (drone.rs:183, command_handler.rs:64). Stage D's Hook executor invokes the existing variant; the only conditional drone-side edit is if RevertReason::HookRollback gains a hook_id field.</warning>
    <warning>DO NOT execute shell hooks against the user's actual filesystem in tests — use test fixtures + the *_with seam to inject mock spawners. Real shell execution is reserved for the integration test in a tempdir.</warning>
    <warning>DO NOT extend the JSONLogic operator allowlist — operators beyond gotcha #18's set need a deliberate spec edit + ADR (CLAUDE.md §11).</warning>
    <warning>DO NOT touch graphStore.ts — the 4 verify/rails event cases are already handled per M03; Stage D's wiring only changes VerifyNode.tsx + HookNode.tsx visual treatment.</warning>
    <warning>DO NOT push between stages.</warning>
    <warning>Stage D's commit MUST include docs/build-prompts/retrospectives/M04.D-retrospective.md in the staged files.</warning>
  </execution_warnings>

  <time_box estimate_hours="6.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage E: RevertReason::HookRollback shape decision (a/b/c per D.3) and rationale; spec §4a firing-point table edit disposition (in-stage or follow-up PR); spec §4a HookRollback edit (paired with the variant decision); any cross-platform shell-execution surprises (gotcha #32 territory); whether the JSONLogic operator allowlist needed extension (if so, surface the operator + use case for ADR consideration); whether the Write tool dispatch site for pre_file_edit interception was where fan_out_grep located it (file:line cited); revert_to_snapshot integration with Stage F recovery flow (the same mechanism may apply to v0.1 startup recovery — flag if Stage F should reuse).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="D.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD) including M04.D-retrospective.md in staged set</item>
    <item>gate results (each gate; hooks/ coverage ≥95%; runtime-main + runtime-drone ≥95% maintained)</item>
    <item>schema drift check exit 0</item>
    <item>fan_out_grep results — firing_point + DroneCommand::RevertToSnapshot + RevertReason:: + Write-dispatch callsite counts</item>
    <item>integration test outcome — hook_integration.rs full lifecycle (post_task → fail → rollback via existing RevertToSnapshot → retry)</item>
    <item>cross-platform shell test outcome (Windows pwsh.exe + Linux bash via test fixtures)</item>
    <item>RevertReason::HookRollback shape decision — option a/b/c chosen + rationale + paired spec §4a edit</item>
    <item>spec §4a firing-point table disposition (in-stage edit or follow-up PR)</item>
    <item>retrospective with [END] decisions for Stage E</item>
    <item>draft commit message from D.6</item>
    <item>"Stage M04.D is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### D.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime+renderer): M04 Stage D — §4a Verify & Rails primitive

Builds the §4a Verify & Rails primitive end-to-end. Hook primitive
(shell|tool|agent variants × 7 firing points) + Rails (hard/soft +
JSONLogic-evaluated) + don't-touch glob matcher (new pre_file_edit
firing point). Wires the new executor + evaluator to the 4 already-
shipped verify+rails event variants in event.rs (does NOT author new
hook_* events). Consumes the existing RevertToSnapshot drone command
from drone.rs:183 (does NOT re-add). VerifyNode + HookNode upgraded
from M03.C synthetic to live-event-driven.

New artifacts:
- schemas/hook.v1.json (Hook + HookRef + HookCategory + Rail)
- crates/runtime-core/src/generated/hook.rs + src/types/hook.ts (generated)
- crates/runtime-main/src/hooks/{mod,executor,rails,dont_touch,shell}.rs
- crates/runtime-main/tests/hook_integration.rs (full lifecycle)

Edits (audit-grounded):
- crates/xtask/src/main.rs: codegen list extends to 10 entries (adds hook)
- crates/runtime-main/src/sdk/agent_sdk.rs: Hook executor wired at firing-
  point sites; pre_file_edit interception inserted at the Write tool
  dispatch (located via fan_out_grep)
- src/components/nodes/VerifyNode.tsx + HookNode.tsx: live-event wiring
  (consumes existing 4 verify_*/rail_triggered events at event.rs:220-241)
- crates/runtime-core/src/drone.rs (CONDITIONAL per HookRollback design
  decision): RevertReason::HookRollback shape — <DECISION> chose option
  <a/b/c> per D.3 (extend variant with hook_id / keep unit / defer v1.0)
- agent-runtime-spec.md §4a: pre_file_edit added to firing-point table
  [in-stage / deferred to follow-up PR per retro decision]; AND/OR edit
  to confirm RevertReason::HookRollback shape per design decision

Items NOT in this commit (audit-grounded):
- schemas/event.v1.json — 4 verify+rails events already there
- crates/runtime-core/src/event.rs — already there per M03; not regen'd
- crates/runtime-drone/src/command_handler.rs — RevertToSnapshot arm
  already at line 64; only extends if HookRollback gains hook_id field
- src/lib/graphStore.ts — verify+rails cases already there per M03

Cross-stack glue: shell.rs cross-platform shell execution per gotcha
#32 — pwsh.exe -NoProfile -Command on Windows; bash -c on Linux/macOS.
Verified against tokio::process::Command docs + Microsoft PowerShell
docs (WEBCHECK).

Coverage: hooks/ ≥95% (capability-enforcer-adjacent safety primitive
per CLAUDE.md §5); shell.rs OS-spawn wrapper excluded with documented
rationale (testable seam covered via *_with pattern; wrapper
structurally untestable cross-platform per A2 + M02 precedent).

Refs: M04-plan-verify-hitl-budget.md §D, spec §4a, MVP §M4
Retrospective: docs/build-prompts/retrospectives/M04.D-retrospective.md

https://claude.ai/code

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE E — §6a HITL primitive                                   -->
<!-- ============================================================ -->

## Stage E — §6a HITL primitive (9 triggers + 3 UI variants + 3 notifiers + plugin interface)

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://v2.tauri.app/plugin/notification/> — Tauri 2.x notification plugin for the `desktop` notifier; verify install + capability + permission-prompt invocation pattern unchanged
- <https://v2.tauri.app/develop/calling-rust/> — Tauri command pattern for HITL response round-trip (renderer → main → drone → SDK HITL seam)
- <https://json-schema.org/draft/2020-12/schema> — hitl.v1.json against this draft

### E.1 Problem Statement

§6a HITL is the third-largest M04 deliverable. Spec §6a locks the primitive (structured object with 9 trigger types + 3 UI variants + notifier plugin interface); Stage E builds it.

1. **Schema** — `schemas/hitl.v1.json` declares: `HitlPolicy` (per-trigger config), `HitlTrigger` enum (9 values), `HitlUiVariant` enum (`panel | modal | toast`), `HitlNotifier` plugin shape (`type` discriminator + `notify(event)` method signature documented). Per spec §6a.

2. **Nine trigger types** per spec §6a (lines 1962–1997): `on_gap` (blocks for missing tool/skill resolution), `on_risky_tool` (per-tool list in framework), `on_dont_touch_edit` (chained from §4a don't-touch rail), `on_failure_threshold` (chained from §3a task_escalated), `on_capability_violation` (M5 hook; Stage E exposes the seam, M5 wires it), `on_budget_threshold` (chained from §2a), `on_plan_approval` (already wired in Stage C), `per_task` (HITL gate before each task), `per_epic` (HITL gate at plan boundaries).

3. **Three UI variants** — `Panel` (full takeover, dim graph), `Modal` (floating dialog, blocks adjacent), `Toast` (non-blocking auto-dismiss). Per-trigger default UI in framework JSON; mode-keyed overrides per spec §6a (M04 v0.1 STANDARD-only — overrides honored at load but only STANDARD evaluates).

4. **Notifier plugin interface** — `HitlNotifier { type, notify(event: HitlNotifyEvent) -> async Result<(), NotifierError> }`. `HitlNotifyEvent { trigger, session_id, question, options, context, timeout_at }`. Built-in v1 notifiers: `terminal_bell`, `desktop` (Tauri notification plugin per WEBCHECK URL), `sound`. Plugin notifiers from `notifiers/` dir under §8.security model (M9 generators wire this; M04 ships only built-ins).

5. **Five HITL events — 2 already shipped + 3 new (audit-grounded).** Existing 2 in `crates/runtime-core/src/event.rs:281, :290`: `HitlRequested`, `HitlResolved` (codebase NAMES — NOT `hitl_response` as the previous draft claimed). Stage E WIRES these (does NOT re-author or rename). 3 NEW variants added to `schemas/event.v1.json`: `hitl_timeout`, `notifier_dispatched`, `notifier_failed`. Regen propagates the 3 new variants only.

6. **Failure-escalation flow** — `task_escalated` (Stage B) → `on_failure_threshold` HITL trigger evaluates → `hitl_requested` event → notifiers fire in parallel → 1h default timeout → on response: route to `task_started` (retry) / `task_skipped` / `plan_aborted` per user choice. Wire the SDK's HITL seam (Stage B exposed) to the HITL flow.

7. **Three renderer surfaces** — `HITLPanel`, `HITLModal`, `HITLToast`. Each consumes `hitl_requested` events and dispatches `respond_hitl(prompt_id, choice)` Tauri command. Reuse M03.D + M04.C non-modal patterns where applicable.

**Success criterion:** Loading a fixture with `on_failure_threshold` trigger → simulating 3 task failures → HITL Panel surfaces; user clicks Skip → `task_skipped` emits; plan continues. `desktop` notifier fires OS notification on Windows + Linux. Coverage gate met (capability-enforcer-adjacent ≥95%).

**New artifacts:**
- `schemas/hitl.v1.json` (new)
- `crates/runtime-core/src/generated/hitl.rs`, `src/types/hitl.ts` (new; generated under `generated/` per A1 convention)
- `crates/runtime-main/src/hitl/{mod,policy,seam,notifiers}.rs` (new module)
- `crates/runtime-main/src/hitl/notifiers/{terminal_bell,desktop,sound}.rs` (new)
- `src/components/HITLPanel.tsx`, `src/components/HITLModal.tsx`, `src/components/HITLToast.tsx` (new)
- `crates/runtime-main/tests/hitl_failure_escalation.rs` (new integration)
- `tests/e2e/hitl_failure_escalation.spec.ts` (new)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (codegen list: extend from 10 entries post-D to 11 with `hitl`)
- `schemas/event.v1.json` (add ONLY 3 new variants: `hitl_timeout`, `notifier_dispatched`, `notifier_failed`. Existing `HitlRequested` + `HitlResolved` at `event.rs:281, :290` stay unchanged)
- `crates/runtime-core/src/event.rs` OR `crates/runtime-core/src/generated/event.rs` (regenerated with 3 new variants per A1 namespace strategy)
- `src/types/agent_event.ts` (regen with 3 new variants)
- `src/lib/graphStore.ts` (exhaustive switch adds 3 new cases; existing `hitl_requested` + `hitl_resolved` cases stay)
- `crates/runtime-core/src/drone.rs` (add `RespondHitl { prompt_id, choice }` variant to `DroneCommand` enum)
- `crates/runtime-drone/src/command_handler.rs` (add `RespondHitl` handler arm — resolves the SDK's HITL seam)
- `src-tauri/src/commands.rs` (`respond_hitl` Tauri command using A2's `Arc<DroneClient>` managed state)
- `src-tauri/tauri.conf.json` + `capabilities/default.json` (Tauri notification plugin capability per current Tauri 2.x docs at WEBCHECK URL)
- `package.json` (add `@tauri-apps/plugin-notification` per Tauri 2.x docs)
- `Cargo.toml` workspace + `src-tauri/Cargo.toml` (`tauri-plugin-notification`)
- `crates/runtime-main/src/sdk/agent_sdk.rs` (wire `task_escalated` → HITL trigger evaluation; `on_failure_threshold` flow lives here)
- `src/App.tsx` (mount HITL surfaces conditionally on `hitl_requested` event arrival)
- `CHANGELOG.md`

### E.2 Files to Change

| File | Change |
|---|---|
| `schemas/hitl.v1.json` | **New** — HitlPolicy + HitlTrigger (9) + HitlUiVariant (3) + HitlNotifier shape |
| `crates/runtime-core/src/hitl.rs`, `src/types/hitl.ts` | **New (generated)** |
| `crates/xtask/src/main.rs` | **Edited** — codegen list: hitl |
| `schemas/event.v1.json` | **Edited** — 3 new variants (hitl_timeout, notifier_dispatched, notifier_failed) |
| `crates/runtime-main/src/hitl/{mod,policy,seam,notifiers}.rs` | **New** — HITL primitive + seam + notifier dispatch |
| `crates/runtime-main/src/hitl/notifiers/{terminal_bell,desktop,sound}.rs` | **New** — 3 built-in notifiers |
| `src/components/HITLPanel.tsx`, `HITLModal.tsx`, `HITLToast.tsx` | **New** — 3 UI variants |
| `src-tauri/src/commands.rs` | **Edited** — `respond_hitl` command |
| `src-tauri/tauri.conf.json` | **Edited** — notification plugin permission |
| `package.json`, `src-tauri/Cargo.toml`, `Cargo.toml` (workspace) | **Edited** — Tauri notification plugin deps |
| `src/lib/graphStore.ts` | **Edited** — exhaustive switch +3 variants |
| `src/App.tsx` | **Edited** — conditional mount of HITL surfaces |
| `tests/e2e/hitl_failure_escalation.spec.ts` | **New** — Playwright happy path |
| `CHANGELOG.md` | **Edited** |

### E.3 Detailed Changes

#### `schemas/hitl.v1.json` — HITL primitive schema

Author per spec §6a. Discriminator: `HitlTrigger.type` over the 9 trigger values; `HitlUiVariant` is a freestanding enum referenced from each trigger's policy. Notifier plugin shape uses a discriminated `type` for the 3 built-ins (`terminal_bell`, `desktop`, `sound`) plus an open extension point for plugin notifiers (M9 territory; v0.1 doesn't load external plugins).

`$id` per the established `https://schemas.aria-runtime.dev/hitl.v1.json` pattern.

#### `crates/runtime-main/src/hitl/seam.rs` — HITL seam

The SDK's HITL approval seam (analogous to Stage B's plan-approval seam): exposes a channel/oneshot the SDK awaits on while a HITL prompt is outstanding. Stage E wires this to the HITL flow:

1. Trigger fires (e.g., `on_failure_threshold` chained from `task_escalated`)
2. Seam emits `hitl_requested` event with prompt_id + question + options + timeout_at
3. Seam fires all configured notifiers in parallel (terminal_bell / desktop / sound)
4. Seam awaits user response via `respond_hitl(prompt_id, choice)` Tauri command OR timeout (default 1h)
5. On response: emit `hitl_resolved` (codebase NAME) + route per `choice` to `task_started`/`task_skipped`/`plan_aborted`
6. On timeout: emit `hitl_timeout` + treat as configured fallback (default: `plan_aborted`)

#### `crates/runtime-main/src/hitl/notifiers/desktop.rs` — Tauri notification plugin

**Cross-stack glue point — gotcha #32 verbatim-quote-or-verify discipline applies.** Per Tauri 2.x notification plugin docs (URL in WEBCHECK), the Rust-side dispatch uses the plugin's API:

```rust
// Verify against https://v2.tauri.app/plugin/notification/ before authoring;
// API surface and permission-prompt semantics may have evolved.
pub async fn dispatch(event: HitlNotifyEvent) -> Result<(), NotifierError> {
    use tauri_plugin_notification::NotificationExt;
    // The exact builder pattern is documented at the WEBCHECK URL — copy
    // verbatim from the upstream example, do not invent.
    // Fallback: if the plugin returns a permission-not-granted error,
    // emit notifier_failed and continue (notifier failures are non-fatal).
}
```

`<execution_warnings>` flags this with explicit "verify against Tauri 2.x notification plugin docs current at <DATE> before shipping" per gotcha #32. Permission flow: app requests notification permission at first dispatch; subsequent calls reuse granted permission.

#### `crates/runtime-main/src/hitl/notifiers/{terminal_bell,sound}.rs` — built-in notifiers

`terminal_bell`: writes `\x07` (ASCII BEL) to stderr; works cross-platform without deps.

`sound`: plays a short beep via `rodio` or similar (verify dep choice during implementation; preference is no new deps if a stdlib path exists). Cross-platform sound is non-trivial — if the implementation requires a new dep, dependency_audit_check covers it.

#### Three renderer UI variants

`HITLPanel.tsx`: full right-side overlay (similar to ApprovalPanel from Stage C); dims the graph; ARIA non-modal (graph remains queryable but interaction routes to panel).

`HITLModal.tsx`: floating centered dialog; blocks interaction with the graph behind it (`aria-modal="true"`); closeable via Escape.

`HITLToast.tsx`: non-blocking auto-dismiss notification; appears in corner; clicking expands to full prompt; auto-dismiss after 30s if not interacted with (treated as `notifier_failed` if user doesn't respond).

Each consumes `hitl_requested` events and dispatches `respond_hitl(prompt_id, choice)` on action.

#### `src-tauri/src/commands.rs` — `respond_hitl`

```rust
#[tauri::command]
pub async fn respond_hitl(
    prompt_id: String,
    choice: String,
    client: tauri::State<'_, Arc<DroneClient>>,
) -> Result<(), CmdError> {
    tracing::info!("respond_hitl prompt_id={} choice={}", prompt_id, choice);
    client.respond_hitl(prompt_id, choice).await.map_err(CmdError::from)
}
```

Drone-side: extend IPC enum with `RespondHitl { prompt_id, choice }` variant; handler resolves the SDK's HITL seam.

#### Failure-escalation flow wiring

In `crates/runtime-main/src/sdk/mod.rs` (or wherever Stage B's plan state machine integrates): on `task_escalated` event, invoke HITL trigger evaluation. If `on_failure_threshold` is configured → fire HITL flow per E.1 #6.

### E.4 Tests

#### Pedantic-pass preflight

Apply to all new modules (`hitl/{mod,policy,seam,notifiers/*}.rs`).

#### Test files

- `crates/runtime-main/src/hitl/seam.rs` — unit tests for seam state machine (request → response | timeout); routing logic
- `crates/runtime-main/src/hitl/notifiers/desktop.rs` — `*_with` testable seam (mock notification dispatcher); real Tauri plugin call in integration test only (excluded from unit coverage per the OS-call holdout pattern)
- `crates/runtime-main/tests/hitl_failure_escalation.rs` (new integration test) — full flow: 3 task failures → HITL request → user-choice shim returns Skip → task_skipped → plan continues
- `tests/unit/HITLPanel.test.tsx`, `HITLModal.test.tsx`, `HITLToast.test.tsx` — render + action-dispatch + ARIA + keyboard nav
- `tests/e2e/hitl_failure_escalation.spec.ts` — Playwright happy path

#### Coverage target

- `crates/runtime-main/src/hitl/` ≥95% (capability-enforcer-adjacent safety primitive per CLAUDE.md §5)
- `notifiers/desktop.rs` real OS-call path excluded with documented rationale
- workspace ≥80%, runtime-main ≥95% maintained

### E.5 CLI Prompt

```xml
<work_stage_prompt id="M04.E">
  <context>
    Stage E of M04. §6a HITL primitive. Audit-grounded scope: 2 HITL events ALREADY EXIST in `crates/runtime-core/src/event.rs:281, :290` (`HitlRequested`, `HitlResolved` — codebase NAMES; NOT `hitl_response` as the previous draft claimed) — Stage E WIRES these (does NOT re-author or rename). 3 NEW HITL events authored: `hitl_timeout`, `notifier_dispatched`, `notifier_failed`. 9 trigger types + 3 UI variants (Panel/Modal/Toast) + notifier plugin interface + 3 built-in notifiers (terminal_bell/desktop via Tauri notification plugin v2/sound). Wires Stage B's HITL seam to the failure-escalation flow (`task_escalated` → `on_failure_threshold` → `hitl_requested` → notifiers parallel → 1h timeout → routes to `task_started`/`task_skipped`/`plan_aborted`). New `respond_hitl` Tauri command using A2's `Arc<DroneClient>` managed state; new `RespondHitl` variant on `DroneCommand` + drone-side handler arm resolving the seam. Cross-stack risk: Tauri notification plugin is the textbook gotcha #32 case (verify against https://v2.tauri.app/plugin/notification/ verbatim). Stage D's commit must be on the milestone branch.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage D" subject</check>
    <check name="prior_retrospective_staged">git log -1 --name-only must include docs/build-prompts/retrospectives/M04.D-retrospective.md (per M03.5.B retro [END] decision)</check>
    <check name="hooks_present">Test-Path crates/runtime-main/src/hooks/mod.rs (Stage D deliverable; Stage E's HITL flow chains from hooks/rails on_failure_threshold)</check>
    <check name="audit_baseline_hitl_event_naming">grep -q "HitlRequested\|HitlResolved" crates/runtime-core/src/event.rs AND ! grep -q "HitlResponse" crates/runtime-core/src/event.rs (audit baseline; codebase NAMES `hitl_resolved` NOT `hitl_response` — Stage E wires existing names; if HitlResponse appears the codebase has drifted)</check>
    <check name="audit_baseline_task_escalated">grep -q "TaskEscalated" crates/runtime-core/src/event.rs (audit baseline; Stage B wired this event; Stage E's on_failure_threshold flow consumes)</check>
    <check name="approval_seam_archetype">grep -q "approval_seam\|ApprovalSeam\|await_approval" crates/runtime-main/src/plan/mod.rs (Stage B's seam archetype Stage E mirrors for the HITL seam)</check>
    <check name="arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Stage E's respond_hitl Tauri command consumes)</check>
    <check name="schema_drift_clean">cargo xtask regenerate-types --check exit 0</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage E sections E.1–E.4)</file>
    <file>agent-runtime-spec.md §6a (full section), §3a Failure escalation primitive (cross-ref into HITL — task_escalated → on_failure_threshold)</file>
    <file>docs/MVP-v0.1.md §M4 (HITL escalation acceptance criterion)</file>
    <file>docs/gotchas.md (especially #32 cross-stack discipline; Tauri notification plugin is the textbook case; #41 grep-verify-claims)</file>
    <file>docs/build-prompts/retrospectives/M04.D-retrospective.md (apply [END] Decisions, especially any cross-platform shell-execution surprises that may inform notifier-platform discipline)</file>
  </read_first>

  <read_reference>
    <file purpose="Stage B approval-gate seam archetype — HITL seam mirrors the pattern (channel/oneshot the SDK awaits on)">crates/runtime-main/src/plan/state_machine.rs</file>
    <file purpose="Stage B's seam channel exposed for downstream wiring">crates/runtime-main/src/plan/mod.rs</file>
    <file purpose="EXISTING HITL events at lines 281, 290; Stage E wires NOT re-authors">crates/runtime-core/src/event.rs</file>
    <file purpose="EXISTING DroneCommand enum to extend with RespondHitl variant; existing arms include WriteSignal (Stage B) + ApprovePlan/RevisePlan/AbortPlan (Stage C)">crates/runtime-core/src/drone.rs</file>
    <file purpose="EXISTING command_handler arms; Stage E adds RespondHitl arm">crates/runtime-drone/src/command_handler.rs</file>
    <file purpose="Stage C ApprovalPanel non-modal pattern for HITLPanel; reuse the discipline (aria-modal=false)">src/components/ApprovalPanel.tsx</file>
    <file purpose="existing Tauri command archetype with Arc&lt;DroneClient&gt; state from A2 + Stage C 3 new commands">src-tauri/src/commands.rs</file>
    <file purpose="VerifyNode/HookNode wiring archetype from Stage D for HITLNode (already-shipped synthetic) live wiring">src/components/nodes/HITLNode.tsx</file>
    <file purpose="graphStore exhaustive switch — Stage E adds 3 new HITL cases; existing hitl_requested + hitl_resolved cases stay">src/lib/graphStore.ts</file>
  </read_reference>

  <read_prior_stages>
    <retrospective stage="A1" milestone="M04"/>
    <retrospective stage="A2" milestone="M04"/>
    <retrospective stage="B" milestone="M04"/>
    <retrospective stage="C" milestone="M04"/>
    <retrospective stage="D" milestone="M04"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="E.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="E.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="Key constraints"/>

  <gates milestone="M04"/>

  <self_correction_budget>3</self_correction_budget>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <fan_out_grep>
    <grep pattern="AgentEvent::HitlRequested\|AgentEvent::HitlResolved" purpose="confirm existing event variants present; Stage E wires NOT re-authors"/>
    <grep pattern="task_escalated\|TaskEscalated" purpose="all callsites of the failure-escalation event — Stage B emits, Stage E consumes via HITL trigger evaluation"/>
    <grep pattern="HitlTrigger::" purpose="all enum-variant constructions; 9 triggers must be exhaustively handled"/>
    <grep pattern="DroneCommand::" purpose="exhaustive matches; Stage E adds RespondHitl variant"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="tauri-plugin-notification" min_version="2.0"/>
  </dependency_audit_check>

  <runtime_environment os="windows" note="desktop notifier uses Windows Toast Notifications via Tauri plugin; verify pwsh-side permission state if first-run flow differs from Linux"/>

  <gotchas>
    <trap>AUDIT-GROUNDED HITL EVENT NAMES: codebase has `HitlRequested` + `HitlResolved` at event.rs:281, :290 — NOT `hitl_response` as the previous draft (PR #52) claimed. Stage E WIRES the existing 2 events; adds 3 NEW ones (`hitl_timeout`, `notifier_dispatched`, `notifier_failed`). Do NOT rename `HitlResolved` to `HitlResponse` or vice versa.</trap>
    <trap>Tauri notification plugin is the textbook gotcha #32 case — verify the install + capability + permission-prompt flow against https://v2.tauri.app/plugin/notification/ verbatim BEFORE authoring desktop.rs. The previous M04 cross-stack failures (M03 tauri-driver) cost three iteration cycles; do not repeat.</trap>
    <trap>Notifier failures are NON-FATAL — emit notifier_failed event and continue. Don't propagate notifier errors up; the HITL seam still resolves on user response or timeout regardless of which notifiers fired.</trap>
    <trap>HITL Panel/Modal/Toast — pick the right ARIA pattern per variant. Panel: aria-modal="false" (graph stays queryable). Modal: aria-modal="true" (blocks adjacent). Toast: role="status" or "alert" depending on urgency.</trap>
    <trap>v0.1 STANDARD mode hardcoded — mode-keyed HITL overrides in framework JSON are loaded but not evaluated. Implementation: load + validate + ignore non-STANDARD overrides; do NOT silently apply LITE/CONFIG defaults.</trap>
    <trap>9 trigger types — exhaustive matching required throughout SDK. Compiler enforces via Rust enum exhaustiveness; add WireMock-style tests if the dispatch logic uses string-keyed lookup that can drift.</trap>
    <trap>RespondHitl IPC variant is NEW on DroneCommand — drone-side handler arm resolves the SDK's HITL seam (mirrors Stage C's ApprovePlan/RevisePlan/AbortPlan pattern). fan_out_grep `DroneCommand::` catches exhaustive matches.</trap>
    <trap>Per gotcha #41 (grep-verify-claims): every codebase claim verified against post-D reality via &lt;pre_flight_check&gt;. The `audit_baseline_hitl_event_naming` check is load-bearing — if it fails (HitlResponse appears), the codebase drifted between authoring and execution; surface before proceeding.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT rename HitlResolved to HitlResponse anywhere — codebase NAMES are HitlRequested + HitlResolved. The previous M04 draft was wrong; this rewrite corrects it.</warning>
    <warning>DO NOT add new event variants for hitl_requested or hitl_resolved — they already exist at event.rs:281, :290. Stage E adds ONLY 3 new variants: hitl_timeout, notifier_dispatched, notifier_failed.</warning>
    <warning>DO NOT implement the M5 capability-violation HITL trigger — Stage E exposes the seam (on_capability_violation in the enum) but the trigger source is M5's deliverable. Mark Stage E's coverage of that trigger as "seam-only" in retro.</warning>
    <warning>DO NOT load external notifier plugins from notifiers/ dir — that's M9 generators territory. Stage E ships only the 3 built-ins; plugin loader returns NotImplemented for external types.</warning>
    <warning>DO NOT push between stages.</warning>
    <warning>Stage E's commit MUST include docs/build-prompts/retrospectives/M04.E-retrospective.md in the staged files.</warning>
  </execution_warnings>

  <time_box estimate_hours="6.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage F: any cross-stack surprises with Tauri notification plugin (gotcha #32 territory); whether the desktop notifier permission flow needs first-run UX integration with M10; the 1h default HITL timeout — should it be per-trigger configurable in v0.1 or v1.0?; coverage gate disposition for notifiers/desktop.rs OS-call holdout.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="E.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD) including M04.E-retrospective.md in staged set</item>
    <item>gate results (each gate; hitl/ coverage ≥95%; runtime-main + runtime-drone ≥95% maintained with new RespondHitl arm covered)</item>
    <item>schema drift check exit 0 (regenerated 11 schemas: existing 10 from D + new hitl)</item>
    <item>fan_out_grep results — HitlRequested + HitlResolved confirmed present (audit baseline); HitlResponse absent (audit baseline); task_escalated + HitlTrigger:: + DroneCommand:: callsite counts</item>
    <item>integration test outcome — hitl_failure_escalation.rs full flow (3 failures → HITL prompt → Skip → plan continues)</item>
    <item>desktop notifier OS-permission flow outcome (test environment may not have permission; document fallback)</item>
    <item>retrospective with [END] decisions for Stage F</item>
    <item>draft commit message from E.6</item>
    <item>"Stage M04.E is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### E.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime+renderer): M04 Stage E — §6a HITL primitive

Builds the §6a HITL primitive end-to-end. Audit-grounded scope: 2 HITL
events ALREADY EXIST at event.rs:281, :290 (HitlRequested, HitlResolved
— codebase NAMES `hitl_resolved` NOT `hitl_response`); Stage E WIRES
these. 3 NEW HITL events authored: hitl_timeout, notifier_dispatched,
notifier_failed. 9 trigger types + 3 UI variants (Panel/Modal/Toast) +
notifier plugin interface + 3 built-in notifiers (terminal_bell,
desktop via Tauri notification plugin, sound). Wires Stage B's HITL
seam to the failure-escalation flow.

New artifacts:
- schemas/hitl.v1.json (HitlPolicy + 9 HitlTrigger + 3 HitlUiVariant +
  HitlNotifier plugin shape)
- crates/runtime-core/src/generated/hitl.rs + src/types/hitl.ts
  (generated under generated/ per A1 convention)
- crates/runtime-main/src/hitl/{mod,policy,seam,notifiers}.rs
- crates/runtime-main/src/hitl/notifiers/{terminal_bell,desktop,sound}.rs
- src/components/HITLPanel.tsx + HITLModal.tsx + HITLToast.tsx
- crates/runtime-main/tests/hitl_failure_escalation.rs (integration)
- tests/e2e/hitl_failure_escalation.spec.ts (Playwright)

Edits (audit-grounded):
- schemas/event.v1.json: 3 NEW variants (hitl_timeout,
  notifier_dispatched, notifier_failed). The existing HitlRequested +
  HitlResolved variants at event.rs:281, :290 stay UNCHANGED — Stage E
  wires the new HITL flow to fire them.
- crates/runtime-core/src/event.rs (or generated/event.rs per A1
  namespace strategy): regenerated with 3 new variants
- crates/runtime-core/src/drone.rs: RespondHitl { prompt_id, choice }
  variant added to DroneCommand
- crates/runtime-drone/src/command_handler.rs: RespondHitl handler arm
  resolves the SDK's HITL seam
- src-tauri/src/commands.rs: respond_hitl Tauri command using A2's
  Arc<DroneClient> managed state
- src-tauri/tauri.conf.json + capabilities/default.json: notification
  plugin capability
- Cargo.toml workspace + src-tauri/Cargo.toml + package.json:
  tauri-plugin-notification + @tauri-apps/plugin-notification deps
- crates/runtime-main/src/sdk/agent_sdk.rs: task_escalated → HITL
  trigger evaluation; on_failure_threshold flow drives the seam
- src/lib/graphStore.ts: exhaustive switch +3 new cases (hitl_timeout,
  notifier_dispatched, notifier_failed); existing hitl_requested +
  hitl_resolved cases stay
- src/App.tsx: conditional HITL surface mount on hitl_requested arrival

Failure-escalation flow: task_escalated (Stage B) → on_failure_threshold
trigger evaluates → hitl_requested + notifiers fire in parallel → user
response or 1h timeout → routes to task_started (retry) / task_skipped /
plan_aborted.

Cross-stack glue: notifiers/desktop.rs uses Tauri notification plugin per
gotcha #32 verbatim-quote discipline; verified against
https://v2.tauri.app/plugin/notification/ at authoring time.

v0.1 scope: STANDARD mode hardcoded; mode-keyed HITL overrides loaded but
not evaluated. on_capability_violation trigger seam exposed; M5 wires the
trigger source.

Coverage: hitl/ ≥95% (capability-enforcer-adjacent safety primitive);
notifiers/desktop.rs real-Tauri-call path excluded with documented
rationale (testable seam covered via *_with pattern; OS-call wrapper
structurally untestable per A2 + M02 + Stage D precedent).

Refs: M04-plan-verify-hitl-budget.md §E, spec §6a, MVP §M4
Retrospective: docs/build-prompts/retrospectives/M04.E-retrospective.md

https://claude.ai/code

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE F — §2a Budget + §1b Recovery                            -->
<!-- ============================================================ -->

## Stage F — §2a Budget + §1b Recovery (cost controls + resume from snapshot)

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.anthropic.com/en/api/messages-count-tokens> — confirm `count_tokens` endpoint (Stage A2 wired) handles budget-enforcement query patterns; no rate-limit issues for per-message pre-flight checks
- <https://json-schema.org/draft/2020-12/schema> — budget.v1.json against this draft

### F.1 Problem Statement

Two primitives bundled in Stage F: §2a Budget (medium-sized; mostly Rust + UI header bar) + §1b Recovery (medium-sized; mostly Rust state-restoration logic). Both depend on Stage A2 production wiring + Stage E HITL flow. **Audit-grounded scope:** the 4 budget event variants ALREADY EXIST in `crates/runtime-core/src/event.rs:321–353` (`BudgetWarn`, `BudgetDownshift`, `BudgetSuspended`, `BudgetExceeded`) — Stage F WIRES the new budget enforcer to fire these (does NOT re-author or rename them). VDR access goes through drone IPC: `crates/runtime-drone/src/vdr.rs::project_signal:50` is the projector; `runtime-main` has no `rusqlite` dep — there is no `runtime-main/src/vdr.rs`. Stage F's recovery uncertainty handler queries VDR via the existing `DroneCommand::QuerySessionDb` IPC variant (or a new `QueryVdr` variant if simple-SQL coverage is insufficient — Stage F's design call).

1. **Budget primitive** — `schemas/budget.v1.json` declares `BudgetActions { warn_at_percent? (def 50), downshift_at_percent? (def 75), hitl_at_percent? (def 90), hard_stop_at_percent? (def 100) }`. Three scopes per spec §2a: per-session ($5 default), per-framework, per-day-global (user setting). Tightest cap wins; budget tracking via real `count_tokens` (Stage A2 wired).

2. **Four budget actions** — `warn` emits toast notification; `downshift` invokes the model-selector hook (default `opus → sonnet → haiku` ladder); `hitl` triggers `on_budget_threshold` HITL flow (Stage E wired); `hard_stop` triggers immediate agent kill via drone `stop_process` + emits `budget_exceeded`.

3. **Four budget events — ALREADY SHIPPED, Stage F WIRES (audit-grounded).** `BudgetWarn:321`, `BudgetDownshift:330`, `BudgetSuspended:339`, `BudgetExceeded:346` all exist in `event.rs` (note: codebase NAMES use `BudgetWarn` — NOT `BudgetWarning` — the previous draft's `budget_warning` was wrong). `graphStore.applyEvent` exhaustive switch already handles these 4 cases per M03. Stage F authors the budget enforcer; the enforcer fires the existing events. Schema drift check passes with no new event variants added.

4. **Session header bar** — `src/components/BudgetHeaderBar.tsx` (new); shows current spend / cap with color gradient (green < 50%, amber 50–75%, red 75–90%, dark red > 90%). Per spec §2a Graph integration.

5. **Recovery — resume rebuilds history.** Per spec §1b WI-14: on session restore, the snapshot's `messages` + `tool_calls` + `tool_results` load into SDK history "as if they had already happened"; model generates next turn fresh; tools NOT re-invoked. Snapshot read API at `crates/runtime-drone/src/snapshot.rs` (existing M01.C); Stage F's `recovery/resume.rs` queries via the existing `DroneCommand::ReadSignals` IPC variant.

6. **Recovery — tool-call uncertainty UI.** Detect `tool_invoked` without paired `tool_result` (signal pair invariant violation per spec §2b). Stage F's uncertainty handler queries VDR via drone IPC (NOT direct rusqlite — `runtime-main` has no `rusqlite` dep). The drone-side `vdr::project_signal:50` already projects signals; Stage F's query path is `DroneCommand::QuerySessionDb { sql }` or extends with a new `QueryVdr { ... }` variant if simple-SQL coverage is insufficient (decision at execution time). Mark VDR row `tool_call_uncertain: true`; UI prompt with `[r]etry/[s]kip/[m]ark complete/[a]bort` options; record as `tool_call_uncertainty_resolved` decision signal.

7. **Recovery — MCP reconnection.** v0.1 doesn't ship MCP servers (M06 territory). Stage F exposes a no-op stub: a `MaybeReconnectMcp` seam called on resume that returns `Ok(())` for v0.1 (no servers configured). M06 wires the seam. Stage F's stub includes a TODO comment + `<dependencies>` reference to M06.

8. **Recovery — plan + capability state restoration.** Plan + task statuses from snapshot's `plans` + `tasks` tables (Stage B authored); running task reset to `pending` unless `task_completed` was in snapshot; loop policy resumes; capability `scope: 'session'` carries over, `scope: 'once'` cleared.

**Success criterion:** Loading a fixture with `budget.session_usd_cap = $1.00` + simulated text streaming → budget header bar transitions through color gradient as spend accumulates → at 50%/75%/90% the corresponding existing events fire (`BudgetWarn`/`BudgetDownshift`/`BudgetSuspended`) → at 100% session hard-stops with `BudgetExceeded`. Recovery: closing app mid-session + reopening → recovery dialog offers resume → resumed session continues from last snapshot with task pointer reset; tool-call-uncertain prompt surfaces if a signal pair was orphaned at crash time; user picks Skip → session continues without re-running the tool. Coverage gate met.

**New artifacts:**
- `schemas/budget.v1.json` (new)
- `crates/runtime-core/src/generated/budget.rs`, `src/types/budget.ts` (new; generated under `generated/` per A1 convention)
- `crates/runtime-main/src/budget/{mod,enforcer}.rs` (new module)
- `crates/runtime-main/src/recovery/{mod,resume,uncertainty,mcp_reconnect_stub}.rs` (new module; `mcp_reconnect_stub.rs` is the v0.1 no-op stub for M06)
- `src/components/BudgetHeaderBar.tsx`, `src/components/RecoveryDialog.tsx`, `src/components/UncertaintyPrompt.tsx` (new)
- `crates/runtime-main/tests/budget_threshold.rs`, `crates/runtime-main/tests/recovery_lifecycle.rs` (new integration)
- `tests/e2e/budget_threshold.spec.ts`, `tests/e2e/recovery_uncertainty.spec.ts` (new Playwright)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (codegen list extends from 11 entries post-E to 12 with `budget`)
- `crates/runtime-main/src/sdk/agent_sdk.rs` (wire budget enforcer at signal-write site; emits the 4 already-shipped events; chains 90% threshold to Stage E HITL seam)
- `crates/runtime-core/src/drone.rs` (CONDITIONAL: add `QueryVdr { ... }` variant if simple-SQL coverage of `QuerySessionDb` insufficient for uncertainty queries — execution-time call)
- `crates/runtime-drone/src/command_handler.rs` (CONDITIONAL: handler arm if Stage F adds `QueryVdr` variant)
- `src-tauri/src/commands.rs` (`request_resume`, `respond_uncertainty`, `set_global_budget` Tauri commands using A2's `Arc<DroneClient>` managed state)
- `src/App.tsx` (mount BudgetHeaderBar always; mount RecoveryDialog on cold-start with prior snapshot; mount UncertaintyPrompt on tool_call_uncertain detection)
- Possibly `agent-runtime-spec.md` §1b ⚠️ note disposition (final status — closed via Stage A2 outcome documented in M03.5)
- `CHANGELOG.md`

**Items NOT in Stage F scope (audit-grounded; would have been incorrectly listed in the previous draft):**
- Authoring `budget_warning`/`budget_downshift`/`budget_suspended`/`budget_exceeded` events — they ALREADY exist (with codebase NAMES `BudgetWarn` etc., NOT `BudgetWarning`) at `event.rs:321–353`.
- `crates/runtime-main/src/vdr.rs` — phantom path; vdr lives in drone (`crates/runtime-drone/src/vdr.rs:50`); Stage F accesses via drone IPC.
- `schemas/event.v1.json` — Stage F does NOT touch (no new event variants).
- `crates/runtime-core/src/event.rs` — Stage F does NOT regenerate.
- `src/types/agent_event.ts` — Stage F does NOT regenerate.
- `src/lib/graphStore.ts` — Stage F does NOT touch (4 budget cases already there).
- `crates/runtime-drone/src/snapshot.rs` — Stage F does NOT extend the read API; existing M01.C read path is sufficient (resume queries via `DroneCommand::ReadSignals`).

### F.2 Files to Change

| File | Change |
|---|---|
| `schemas/budget.v1.json` | **New** — BudgetActions + 3 scopes + downshift_hook per spec §2a |
| `crates/runtime-core/src/generated/budget.rs`, `src/types/budget.ts` | **New (generated)** under `generated/` per A1 convention |
| `crates/xtask/src/main.rs` | **Edited** — extend codegen list from 11 entries (post-E) to 12 with `budget` |
| `crates/runtime-main/src/budget/{mod,enforcer}.rs` | **New** — budget enforcement loop; emits the 4 already-shipped events |
| `crates/runtime-main/src/recovery/{mod,resume,uncertainty,mcp_reconnect_stub}.rs` | **New** — recovery primitive; mcp_reconnect_stub is v0.1 no-op for M06 |
| `crates/runtime-main/src/sdk/agent_sdk.rs` | **Edited** — wire budget enforcer at signal-write site; chains 90% threshold to Stage E HITL seam |
| `crates/runtime-core/src/drone.rs` | **Edited (conditional)** — `QueryVdr { ... }` variant if simple-SQL coverage of QuerySessionDb insufficient for uncertainty queries (execution-time decision) |
| `crates/runtime-drone/src/command_handler.rs` | **Edited (conditional)** — `QueryVdr` handler arm if Stage F adds the variant |
| `src/components/BudgetHeaderBar.tsx`, `RecoveryDialog.tsx`, `UncertaintyPrompt.tsx` | **New** — 3 UI surfaces |
| `src-tauri/src/commands.rs` | **Edited** — `request_resume`, `respond_uncertainty`, `set_global_budget` Tauri commands using A2's `Arc<DroneClient>` |
| `src/App.tsx` | **Edited** — mount BudgetHeaderBar always; conditional RecoveryDialog (cold-start with prior snapshot) + UncertaintyPrompt (tool_call_uncertain) |
| `crates/runtime-main/tests/{budget_threshold,recovery_lifecycle}.rs` | **New** — integration tests |
| `tests/e2e/budget_threshold.spec.ts`, `recovery_uncertainty.spec.ts` | **New** — Playwright |
| `CHANGELOG.md` | **Edited** |

**Files explicitly NOT in this table** (audit-grounded; already exist per codebase reality):
- `schemas/event.v1.json` — Stage F does NOT touch (4 budget events already present at lines 321–353)
- `crates/runtime-core/src/event.rs` — Stage F does NOT regenerate (no new variants)
- `src/types/agent_event.ts` — Stage F does NOT regenerate
- `src/lib/graphStore.ts` — Stage F does NOT touch (4 budget cases already handled per M03)
- `crates/runtime-drone/src/snapshot.rs` — Stage F does NOT extend (existing M01.C read path is sufficient via DroneCommand::ReadSignals)
- `crates/runtime-main/src/vdr.rs` — does NOT exist; vdr is `crates/runtime-drone/src/vdr.rs:50` accessed via drone IPC

### F.3 Detailed Changes

#### `schemas/budget.v1.json` — Budget primitive schema

Author per spec §2a. `BudgetActions` with 4 percent thresholds (defaults per spec); 3 scopes (per-session / per-framework / per-day-global); `downshift_hook` field referencing a tool ID by name.

#### `crates/runtime-main/src/budget/enforcer.rs` — Budget enforcement loop (audit-grounded)

Hooks into the SDK's signal-write path (Stage B's WriteSignal IPC emission). After every signal that carries `tokens_in` + `tokens_out`:

1. Compute current spend (sum across session, lookup framework, lookup global per-day)
2. For each scope, check tightest cap; if any threshold crossed, emit the corresponding **already-shipped** event variant from `event.rs:321–353`:
   - 50% → emit `BudgetWarn` (codebase NAME — NOT `BudgetWarning` as the previous draft claimed)
   - 75% → emit `BudgetDownshift` + invoke downshift_hook (model swap via tool dispatch)
   - 90% → emit `BudgetSuspended` + trigger `on_budget_threshold` HITL flow (Stage E wired)
   - 100% → emit `BudgetExceeded` + drone `stop_process` (immediate kill)

Cost computation uses real `count_tokens` (Stage A2 `messages/count_tokens` endpoint) cached per-message with LRU per session.

Stage F does NOT add new event variants; the 4 budget events all exist post-M03. The schema-drift check passes with no `event.v1.json` edits.

#### `crates/runtime-main/src/recovery/resume.rs` — Resume from snapshot

Per spec §1b: load snapshot → reconstruct SDK message history → seed agent with prior state → model generates next turn fresh. Tool calls NOT re-invoked (per WI-14 lock).

Plan state restoration: load plan + tasks from SQLite; running task reset to `pending` unless `task_completed` in snapshot; loop policy resumes.

Capability state restoration: scope-session capabilities carry over; scope-once capabilities cleared.

#### `crates/runtime-main/src/recovery/uncertainty.rs` — Tool-call uncertainty handler (audit-grounded)

Detect: `tool_invoked` signal without paired `tool_result` at crash time. The detection query runs against VDR via drone IPC — `runtime-main` has no `rusqlite` dep, so direct SQL is structurally infeasible. Two paths (decision at execution time):

(a) **Use existing `DroneCommand::QuerySessionDb { sql }`** with a SELECT against the VDR-projected table for orphaned tool_invoked rows. Simple SQL coverage; no new IPC variant.

(b) **Add new `DroneCommand::QueryVdr { ... }` variant** with a typed query interface. Cleaner but adds IPC surface; only worth it if simple-SQL of (a) is insufficient.

Recommended baseline: (a). Decision documented in retro.

Once orphaned signals are found, mark VDR row `tool_call_uncertain: true` (via another drone IPC dispatch). Surface UI prompt:

```
Tool call "X" was in flight when the session was interrupted.
[r] retry the call
[s] skip — assume failed
[m] mark complete — assume succeeded with no result
[a] abort the session
```

User response → emit `tool_call_uncertainty_resolved` decision signal (via Stage B's `DroneCommand::WriteSignal` IPC variant) with the chosen action; route accordingly.

#### `src/components/BudgetHeaderBar.tsx` — UI

Top-of-screen horizontal bar. Shows: current spend (e.g., "$2.34"), cap (e.g., "$5.00"), percent, color gradient. Tooltip on hover shows scope breakdown (session / framework / day). Click opens a settings panel for global per-day cap (M10 first-run UX wires this; Stage F exposes the seam via `set_global_budget` command).

#### `src/components/RecoveryDialog.tsx` — Cold-start recovery

On app launch, check for prior snapshot. If present, surface `<RecoveryDialog>`: "Previous session detected. Resume?" with Resume / Discard / Cancel options. Resume invokes `request_resume(session_id)` Tauri command.

#### `src/components/UncertaintyPrompt.tsx` — Tool-call uncertainty UI

Modal dialog with the 4 options above. Dispatches `respond_uncertainty(prompt_id, choice)` on action.

### F.4 Tests

#### Test files

- `crates/runtime-main/src/budget/enforcer.rs` — unit tests for threshold crossings + scope precedence + cap-tightest-wins logic
- `crates/runtime-main/tests/budget_threshold.rs` — integration: simulated spend hits 50/75/90/100 thresholds in order; events fire in order; downshift_hook dispatches model swap
- `crates/runtime-main/src/recovery/{resume,uncertainty}.rs` — unit tests for state restoration (full coverage of snapshot fields → SDK state); uncertainty detection (paired-signal invariant)
- `crates/runtime-main/tests/recovery_lifecycle.rs` — integration: write snapshot mid-session → reload → verify SDK state matches; verify tool calls not re-invoked
- `tests/unit/BudgetHeaderBar.test.tsx`, `RecoveryDialog.test.tsx`, `UncertaintyPrompt.test.tsx` — render + dispatch
- `tests/e2e/budget_threshold.spec.ts` + `recovery_uncertainty.spec.ts` — Playwright happy paths

#### Coverage target

- `crates/runtime-main/src/budget/` ≥95% (capability-enforcer-adjacent)
- `crates/runtime-main/src/recovery/` ≥95% (snapshot/recovery code path per CLAUDE.md §5)
- workspace ≥80% maintained

### F.5 CLI Prompt

```xml
<work_stage_prompt id="M04.F">
  <context>
    Stage F of M04. §2a Budget + §1b Recovery. Audit-grounded scope: 4 budget event variants ALREADY EXIST at `crates/runtime-core/src/event.rs:321–353` (`BudgetWarn`, `BudgetDownshift`, `BudgetSuspended`, `BudgetExceeded` — codebase NAMES; the previous draft's `budget_warning` was wrong). Stage F WIRES the new budget enforcer to fire these (does NOT re-author or rename). VDR access via drone IPC: `runtime-main` has NO `rusqlite` dep — there is no `runtime-main/src/vdr.rs`. The drone-side projector is at `crates/runtime-drone/src/vdr.rs:50` (Stage B's WriteSignal IPC variant + handler arm wired the path). Recovery uncertainty queries VDR via existing `DroneCommand::QuerySessionDb { sql }` (or new `QueryVdr` variant if simple-SQL coverage insufficient — execution-time decision). Budget primitive: 3 scopes + 4 threshold actions + downshift_hook + UI header bar. Recovery primitive: resume rebuilds history (per spec §1b WI-14; tools NOT re-invoked) + tool-call-uncertain UI (4 actions) + MCP reconnect SEAM as v0.1 no-op stub for M06 + plan/capability state restoration. Token-cost via Stage A2's real `count_tokens` endpoint with LRU per-message cache. 90% threshold chains to Stage E's HITL `on_budget_threshold` flow. Hard-stop at 100% via drone `stop_process`. Stage E's commit must be on the milestone branch.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage E" subject</check>
    <check name="prior_retrospective_staged">git log -1 --name-only must include docs/build-prompts/retrospectives/M04.E-retrospective.md (per M03.5.B retro [END] decision)</check>
    <check name="hitl_seam_present">Test-Path crates/runtime-main/src/hitl/seam.rs (Stage E exposes the on_budget_threshold trigger; Stage F's 90% action drives it)</check>
    <check name="count_tokens_real">grep -q "messages/count_tokens" crates/runtime-main/src/providers/anthropic.rs (Stage A2 wired the real endpoint; budget enforcement depends on it)</check>
    <check name="audit_baseline_budget_events">grep -q "BudgetWarn\|BudgetDownshift\|BudgetSuspended\|BudgetExceeded" crates/runtime-core/src/event.rs (audit baseline; 4 events must all be present — Stage F wires NOT re-authors. If `BudgetWarning` appears the codebase has drifted)</check>
    <check name="audit_baseline_no_runtime_main_vdr">! Test-Path crates/runtime-main/src/vdr.rs (audit baseline; vdr lives in drone NOT main; Stage F accesses via drone IPC)</check>
    <check name="audit_baseline_drone_vdr_present">Test-Path crates/runtime-drone/src/vdr.rs (audit baseline; the projector Stage F's recovery uncertainty handler queries via drone IPC)</check>
    <check name="audit_baseline_writesignal_present">grep -q "WriteSignal" crates/runtime-core/src/drone.rs (Stage B deliverable; Stage F's recovery uncertainty handler emits tool_call_uncertainty_resolved decisions via this IPC path)</check>
    <check name="arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Stage F's 3 new Tauri commands consume)</check>
    <check name="schema_drift_clean">cargo xtask regenerate-types --check exit 0</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage F sections F.1–F.4)</file>
    <file>agent-runtime-spec.md §2a (full section), §1b (Recovery Semantics — Resume rebuilds history per WI-14; Tool calls in flight at crash time; MCP reconnection; Plan state restoration; Capability state), §2b (signals + structured-emitter for the tool_call_uncertainty_resolved decision shape)</file>
    <file>docs/MVP-v0.1.md §M4 (budget + recovery acceptance criteria)</file>
    <file>docs/gotchas.md (especially #15 Resume rebuilds history, doesn't re-execute; #41 grep-verify-claims)</file>
    <file>docs/build-prompts/retrospectives/M04.E-retrospective.md (apply [END] Decisions, especially: any Tauri notification plugin cross-stack surprises that may inform the budget UI integration)</file>
  </read_first>

  <read_reference>
    <file purpose="Stage A2 count_tokens implementation at line 135 (real endpoint); budget enforcement queries">crates/runtime-main/src/providers/anthropic.rs</file>
    <file purpose="Stage E HITL seam that on_budget_threshold trigger drives at 90% threshold">crates/runtime-main/src/hitl/seam.rs</file>
    <file purpose="EXISTING snapshot read API at runtime-drone (M01.C); resume.rs queries via DroneCommand::ReadSignals (do NOT extend the read API directly; runtime-main has no rusqlite)">crates/runtime-drone/src/snapshot.rs</file>
    <file purpose="EXISTING VDR projector at vdr.rs:50; uncertainty.rs queries via drone IPC (runtime-main has no rusqlite dep — there is NO runtime-main/src/vdr.rs)">crates/runtime-drone/src/vdr.rs</file>
    <file purpose="EXISTING DroneCommand enum to potentially extend with QueryVdr variant if simple-SQL coverage of QuerySessionDb is insufficient (execution-time decision)">crates/runtime-core/src/drone.rs</file>
    <file purpose="EXISTING 4 budget event variants at event.rs:321-353; Stage F wires NOT re-authors">crates/runtime-core/src/event.rs</file>
    <file purpose="Stage B's WriteSignal IPC path that the tool_call_uncertainty_resolved decision emits through">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="Stage C/E renderer surface mounting pattern for BudgetHeaderBar/RecoveryDialog/UncertaintyPrompt">src/App.tsx</file>
    <file purpose="graphStore exhaustive switch — Stage F does NOT touch (4 budget cases already there per M03)">src/lib/graphStore.ts</file>
  </read_reference>

  <read_prior_stages>
    <retrospective stage="A1" milestone="M04"/>
    <retrospective stage="A2" milestone="M04"/>
    <retrospective stage="B" milestone="M04"/>
    <retrospective stage="C" milestone="M04"/>
    <retrospective stage="D" milestone="M04"/>
    <retrospective stage="E" milestone="M04"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="F.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="F.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="Key constraints"/>

  <gates milestone="M04"/>

  <self_correction_budget>3</self_correction_budget>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <runtime_environment os="windows"/>

  <fan_out_grep>
    <grep pattern="AgentEvent::Budget" purpose="confirm 4 budget event variants present (BudgetWarn, BudgetDownshift, BudgetSuspended, BudgetExceeded); Stage F wires NOT re-authors"/>
    <grep pattern="DroneCommand::QuerySessionDb\|DroneCommand::ReadSignals" purpose="existing IPC surface uncertainty handler queries through; verify simple-SQL coverage before deciding to add QueryVdr"/>
    <grep pattern="DroneCommand::WriteSignal" purpose="Stage B's IPC path that tool_call_uncertainty_resolved decisions emit through"/>
  </fan_out_grep>

  <gotchas>
    <trap>AUDIT-GROUNDED BUDGET EVENT NAMES: codebase has BudgetWarn (NOT BudgetWarning), BudgetDownshift, BudgetSuspended, BudgetExceeded at event.rs:321-353. Stage F WIRES the new enforcer to fire these — does NOT author new variants or rename. The previous draft (PR #52) had `budget_warning` which was wrong.</trap>
    <trap>VDR ACCESS VIA DRONE IPC: runtime-main has NO rusqlite dep; there is NO runtime-main/src/vdr.rs. Recovery uncertainty queries route through DroneCommand::QuerySessionDb (or new QueryVdr variant if simple-SQL insufficient). Do NOT add rusqlite to runtime-main's Cargo.toml as a workaround.</trap>
    <trap>Recovery rebuilds HISTORY, not EXECUTION — gotcha #15. Tool calls in the snapshot are loaded into SDK message history as if they already happened; the model generates the NEXT turn fresh. Do NOT re-invoke tools on resume.</trap>
    <trap>Tool-call uncertainty detection is paired-signal invariant — `tool_invoked` without `tool_result`. The 4 user actions (retry/skip/mark/abort) must each emit a distinct `tool_call_uncertainty_resolved` decision signal so the VDR projection has the audit trail. Decision emission goes through Stage B's WriteSignal IPC path (runtime-main has no rusqlite).</trap>
    <trap>Budget tightest-cap-wins — if session cap=$5, framework cap=$3, day-global cap=$10, the framework cap wins. Implementation: compute (cap, scope) for all active scopes; min(cap) wins.</trap>
    <trap>Budget downshift_hook — invokes a runtime tool (the model-selector). The default ladder (opus → sonnet → haiku) is HARDCODED in the hook implementation OR configurable per framework JSON; pick the simpler v0.1 path and document choice in retro.</trap>
    <trap>BudgetExceeded → drone stop_process is the EMERGENCY KILL path. After hard_stop, the session is unrecoverable — UI must surface a clear "session terminated due to budget" message, not just a silent stop.</trap>
    <trap>MCP reconnect on resume — v0.1 ships NO MCP servers (M06 territory). Stage F's mcp_reconnect_stub.rs is a NO-OP returning Ok(()); add a TODO comment + &lt;dependencies&gt; reference to M06. Do NOT implement reconnect logic for non-existent servers.</trap>
    <trap>Per gotcha #41 (grep-verify-claims): every codebase claim verified against post-E reality via &lt;pre_flight_check&gt;. The audit baselines (budget events present, no runtime-main/src/vdr.rs, drone vdr present, WriteSignal present) are load-bearing — if any fail, surface drift before proceeding.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT add new event variants for budget — all 4 already exist at event.rs:321-353 with codebase NAMES (BudgetWarn, NOT BudgetWarning). Schema drift check verifies no new event.v1.json variants.</warning>
    <warning>DO NOT touch graphStore.ts — the 4 budget event cases are already handled per M03.</warning>
    <warning>DO NOT add rusqlite to runtime-main's Cargo.toml — VDR access goes through drone IPC. There is no runtime-main/src/vdr.rs.</warning>
    <warning>DO NOT call live Anthropic /v1/messages/count_tokens in tests — budget enforcement uses cached counts; tests use a fixture cache. Live calls reserved for the smoke test.</warning>
    <warning>DO NOT implement the model-selector tool itself — that's framework-JSON territory. Stage F provides the downshift_hook seam (invokes a tool by name); the tool implementation is in framework JSON or future M9 generators.</warning>
    <warning>DO NOT implement MCP reconnect logic — Stage F's mcp_reconnect_stub.rs is a NO-OP for v0.1 (no servers). M06 wires the actual reconnect.</warning>
    <warning>DO NOT push between stages.</warning>
    <warning>Stage F's commit MUST include docs/build-prompts/retrospectives/M04.F-retrospective.md in the staged files.</warning>
  </execution_warnings>

  <time_box estimate_hours="6"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage G (Phase Closeout): which M04 carry-forward items are fully closed vs need v1.0 escalation; whether the budget downshift_hook ladder configurability landed in framework JSON or stayed hardcoded; whether tool-call uncertainty UI surfaced any spec gaps in the 4-action semantics; final disposition of the §1d long-lived events() reconnect note (Stage A2 may have closed it; F validates).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="F.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD) including M04.F-retrospective.md in staged set</item>
    <item>gate results (each gate; budget/ + recovery/ coverage ≥95%; runtime-main + runtime-drone ≥95% maintained)</item>
    <item>schema drift check exit 0 (regenerated 12 schemas: existing 11 from E + new budget; NO event.v1.json edits since 4 budget events already exist)</item>
    <item>fan_out_grep results — AgentEvent::Budget* count = 4 (audit baseline); QuerySessionDb/ReadSignals/WriteSignal IPC callsites</item>
    <item>uncertainty handler IPC choice — option (a) QuerySessionDb sufficient OR option (b) new QueryVdr variant added; rationale</item>
    <item>budget downshift_hook ladder configurability — hardcoded vs framework-JSON-configurable; rationale</item>
    <item>integration test outcomes — budget_threshold.rs (50/75/90/100 events fire in order with codebase event NAMES BudgetWarn/Downshift/Suspended/Exceeded); recovery_lifecycle.rs (snapshot → resume; tool calls not re-invoked per WI-14)</item>
    <item>e2e test outcomes — budget_threshold.spec.ts + recovery_uncertainty.spec.ts</item>
    <item>retrospective with [END] decisions for Stage G</item>
    <item>draft commit message from F.6</item>
    <item>"Stage M04.F is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### F.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime+renderer): M04 Stage F — §2a Budget + §1b Recovery

Bundles two primitives in one stage. Budget primitive (3 scopes + 4
threshold actions + downshift_hook + UI header bar) closes spec §2a.
Recovery primitive (resume rebuilds history not re-execute per WI-14,
tool-call-uncertain UI with 4 actions, MCP reconnect SEAM as v0.1
no-op stub for M06, plan + capability state restoration) closes spec
§1b.

Audit-grounded scope: 4 budget event variants ALREADY EXIST at
event.rs:321-353 (BudgetWarn — NOT BudgetWarning, BudgetDownshift,
BudgetSuspended, BudgetExceeded). Stage F WIRES the new enforcer to
fire these; does NOT author new variants. VDR access via drone IPC
(runtime-main has no rusqlite; there is no runtime-main/src/vdr.rs).

New artifacts:
- schemas/budget.v1.json
- crates/runtime-core/src/generated/budget.rs + src/types/budget.ts
  (generated under generated/ per A1 convention)
- crates/runtime-main/src/budget/{mod,enforcer}.rs
- crates/runtime-main/src/recovery/{mod,resume,uncertainty,
  mcp_reconnect_stub}.rs (mcp_reconnect_stub is v0.1 no-op for M06)
- src/components/BudgetHeaderBar.tsx + RecoveryDialog.tsx +
  UncertaintyPrompt.tsx
- crates/runtime-main/tests/budget_threshold.rs +
  recovery_lifecycle.rs (integration)
- tests/e2e/budget_threshold.spec.ts +
  recovery_uncertainty.spec.ts (Playwright)

Edits (audit-grounded):
- crates/xtask/src/main.rs: codegen list extends from 11 entries
  (post-E) to 12 with `budget`
- crates/runtime-main/src/sdk/agent_sdk.rs: budget enforcer wired at
  signal-write site; emits the 4 already-shipped events; chains 90%
  threshold to Stage E HITL seam
- crates/runtime-core/src/drone.rs (CONDITIONAL): QueryVdr variant if
  simple-SQL coverage of QuerySessionDb insufficient for uncertainty
  queries; otherwise unchanged
- crates/runtime-drone/src/command_handler.rs (CONDITIONAL): QueryVdr
  handler arm if Stage F adds the variant
- src-tauri/src/commands.rs: request_resume, respond_uncertainty,
  set_global_budget Tauri commands using A2's Arc<DroneClient>
- src/App.tsx: mount BudgetHeaderBar always; conditional RecoveryDialog
  (cold-start with prior snapshot) + UncertaintyPrompt
  (tool_call_uncertain)

Items NOT in this commit (audit-grounded):
- schemas/event.v1.json — 4 budget events already there
- crates/runtime-core/src/event.rs — already there per M03; not regen'd
- src/types/agent_event.ts — not regen'd
- src/lib/graphStore.ts — 4 budget cases already handled per M03
- crates/runtime-drone/src/snapshot.rs — existing read API sufficient

Budget enforcement uses Stage A2's real count_tokens endpoint;
threshold crossings emit existing events (BudgetWarn at 50%,
BudgetDownshift + downshift_hook at 75% / opus→sonnet→haiku ladder,
BudgetSuspended + Stage E HITL flow at 90%, BudgetExceeded + drone
stop_process at 100%).

Recovery rebuilds SDK message history from snapshot per WI-14; tools
NOT re-invoked. Tool-call uncertainty detection via paired-signal
invariant (tool_invoked without tool_result) — query routes through
drone IPC (existing QuerySessionDb or new QueryVdr per execution-time
decision); 4-action prompt; user choice emits
tool_call_uncertainty_resolved decision via Stage B's WriteSignal IPC.

MCP reconnect on resume is a NO-OP STUB for v0.1 (no MCP servers);
M06 wires the actual reconnect.

Coverage: budget/ + recovery/ ≥95% (safety primitive per CLAUDE.md §5).

Refs: M04-plan-verify-hitl-budget.md §F, spec §2a + §1b WI-14, MVP §M4
Retrospective: docs/build-prompts/retrospectives/M04.F-retrospective.md

https://claude.ai/code

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE G — Phase Closeout: Gap Analysis                         -->
<!-- ============================================================ -->

## Stage G — Phase Closeout: Gap Analysis

> **Per CLAUDE.md §20.** This stage runs after Stages A1–F commit and the M04-summary.md aggregation lands. It produces one new entry in `docs/gap-analysis.md`. The gap analysis commit is the final commit on the parent-milestone branch — it gates the PR push.

### G.1 Problem Statement

Generate the M04 entry in `docs/gap-analysis.md`. Cumulative review of code-vs-spec across all milestones to date (M01 + M02 + M03 + M03.5 + M04) — not just M04. Append-only — never edit prior entries.

Per STAGE-PROMPT-PROTOCOL.md v1.2 closeout schema, this stage's `<gap_analysis_requirements>` includes the mandatory `<gotchas_graduation>` subsection auditing every per-stage `<gotchas>` from M04.A1 through M04.F with a disposition (kept | graduated | resolved | expired).

Three carry-forward dispositions are mandatory:
- **M02 carry-forward** — items still open at M03.5 close; M04 closes most via Stage A2 (production wiring) + Stage B (Plan model). Final disposition recorded.
- **M03 carry-forward** — items absorbed by M04 Stage A1 (build hygiene) + Stage A2 (production wiring). Final disposition recorded.
- **M03.5 carry-forward** — validator script v1.4 + verification-regex dry-run + estimation-calibration + per-stage gotchas. Disposition: forward to M05+.

### G.2 Files to Change

| File | Change |
|---|---|
| `docs/gap-analysis.md` | **Edited (append-only)** — new M04 section appended at the bottom per the entry template at the top of the file |
| `docs/build-prompts/retrospectives/M04-summary.md` | **New** — parent-milestone roll-up across Stages A1–F; verdict per CLAUDE.md §19 |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes M04 gap-analysis entry was added |

### G.3 Detailed Changes

The M04 entry follows the six-section template defined at the top of `docs/gap-analysis.md`. Do NOT diverge from the template; do NOT skip sections (write "None observed." if a section truly has nothing to report).

**Process:**

1. Re-read `agent-runtime-spec.md` end-to-end (especially §1b, §2a, §3a, §4a, §6a — sections this milestone touched) plus prior milestone sections that M04 may have affected (§2c.3 token tracking from M03.5; §1d events() reconnect; §10 plans/tasks DDL).
2. Read every file produced or edited across Stages A1–F (and prior milestones if cumulative review surfaces issues there).
3. Read prior `docs/gap-analysis.md` entries (M01, M02, M03, M03.5 absent — M03.5 is doc-only) in full to know what's outstanding.
4. Draft the new entry per the template + add the `<gotchas_graduation>` audit subsection.
5. Author `M04-summary.md` aggregating Stage A1–F retrospectives with verdict.
6. Run the append-only check locally before surfacing: `git show origin/main:docs/gap-analysis.md > /tmp/gap-base.md && diff /tmp/gap-base.md <(head -n "$(wc -l < /tmp/gap-base.md)" docs/gap-analysis.md)` — must be empty.

### G.4 Tests

No new code tests. Verification is the append-only check (CI-enforced) plus user review of the entry's substance.

#### Coverage target

N/A — documentation stage.

### G.5 CLI Prompt

```xml
<closeout_stage_prompt id="M04.G">
  <context>
    Stage G of M04 — Phase Closeout: Gap Analysis. Produces the immutable M04 entry in docs/gap-analysis.md per CLAUDE.md §20. This is the FINAL commit on the milestone branch and gates the PR push. Per STAGE-PROMPT-PROTOCOL.md v1.2 closeout schema, includes mandatory <gotchas_graduation> audit of all M04.A1–F per-stage gotchas.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="all_work_stages_committed">git log --oneline main..HEAD must show 7 commits (Stages A1, A2, B, C, D, E, F) — Stage G's commit is the closing 8th</check>
    <check name="all_retros_committed">Test-Path docs/build-prompts/retrospectives/M04.A1-retrospective.md through M04.F-retrospective.md (7 files; per M03.5.B retro [END] decision each retro file was staged with its own stage's commit — verify via git log --name-only)</check>
    <check name="all_retros_in_history">for s in A1 A2 B C D E F; do git log --diff-filter=A --name-only main..HEAD -- docs/build-prompts/retrospectives/M04.$s-retrospective.md must show one commit; done (closes the M03.5.A drift pattern where retrospective was untracked at stage commit time)</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md (especially §20 Gap Analysis Protocol)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (especially closeout-stage schema + <gotchas_graduation> rule)</file>
    <file>docs/gap-analysis.md (header + entry template + ALL prior M01-M03 entries in full)</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage G sections G.1–G.4)</file>
    <file>agent-runtime-spec.md (skim end-to-end; deep on §1b, §2a, §3a, §4a, §6a)</file>
    <file>docs/MVP-v0.1.md §M4 (verify all acceptance criteria check off)</file>
  </read_first>

  <read_reference>
    <file purpose="prior closeout archetype">docs/build-prompts/M03-live-graph.md (§F Phase Closeout)</file>
    <file purpose="prior summary archetype">docs/build-prompts/retrospectives/M03-summary.md</file>
    <file purpose="prior gap-analysis entry archetype">docs/gap-analysis.md (M03 entry)</file>
  </read_reference>

  <cumulative_reads>
    <commit_log>git log --oneline main..HEAD (all 7 work-stage commits)</commit_log>
    <retrospectives_path>docs/build-prompts/retrospectives/M04.A1-retrospective.md through M04.F-retrospective.md</retrospectives_path>
    <prior_milestone_entries>M01, M02, M03 entries in docs/gap-analysis.md (M03.5 has no entry per its design)</prior_milestone_entries>
  </cumulative_reads>

  <deliverables>
    <item>docs/gap-analysis.md M04 entry (append-only, 6 sections including <gotchas_graduation>)</item>
    <item>docs/build-prompts/retrospectives/M04-summary.md (parent-milestone roll-up; verdict per CLAUDE.md §19)</item>
    <item>CHANGELOG.md [Unreleased] note that M04 gap-analysis entry was added</item>
  </deliverables>

  <gap_analysis_requirements ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="G.3 Detailed Changes">
    <gotchas_graduation>
      <stage_review id="A1">
        <gotcha>Each per-stage <gotchas> trap from M04.A1's prompt audited here</gotcha>
        <disposition>kept | graduated | resolved | expired</disposition>
        <target>Where it lands forward (graduated → docs/gotchas.md #N; resolved → fixed at <commit>; expired → rationale beyond bare n/a)</target>
      </stage_review>
      <stage_review id="A2">
        <gotcha>...</gotcha>
        <disposition>...</disposition>
        <target>...</target>
      </stage_review>
      <stage_review id="B"><gotcha>...</gotcha><disposition>...</disposition><target>...</target></stage_review>
      <stage_review id="C"><gotcha>...</gotcha><disposition>...</disposition><target>...</target></stage_review>
      <stage_review id="D"><gotcha>...</gotcha><disposition>...</disposition><target>...</target></stage_review>
      <stage_review id="E"><gotcha>...</gotcha><disposition>...</disposition><target>...</target></stage_review>
      <stage_review id="F"><gotcha>...</gotcha><disposition>...</disposition><target>...</target></stage_review>
    </gotchas_graduation>
  </gap_analysis_requirements>

  <append_only_verification>
    <command>git show origin/main:docs/gap-analysis.md > /tmp/gap-base.md</command>
    <command>diff /tmp/gap-base.md <(head -n "$(wc -l < /tmp/gap-base.md)" docs/gap-analysis.md)</command>
    <expected>empty diff (prior entries unchanged)</expected>
  </append_only_verification>

  <three_artifact_review>
    <artifact>code diff across Stages A1-F (cumulative)</artifact>
    <artifact>per-stage retrospectives + M04-summary.md</artifact>
    <artifact>M04 gap-analysis entry (this stage's deliverable)</artifact>
    <note>Per CLAUDE.md §20: all three artifacts reviewed together; pushback on any blocks the PR until revised.</note>
  </three_artifact_review>

  <scope_locks ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="Key constraints"/>

  <gates milestone="M04"/>

  <self_correction_budget>2</self_correction_budget>

  <runtime_environment os="windows"/>

  <gotchas>
    <trap>docs/gap-analysis.md is APPEND-ONLY per CLAUDE.md §20. Prior M01, M02, M03 entries are IMMUTABLE. M04's carry-forward entries reference prior items by milestone tag (e.g., "M02 yellow X — resolved at <commit>"); never edit prior entries directly.</trap>
    <trap><gotchas_graduation> requires every prior stage in the milestone to have a <stage_review id="..."> entry — even if a stage had no gotchas (use <gotcha>None observed.</gotcha><disposition>n/a</disposition> in that case). Validator catches missing stages by counting stage headings in this Phase doc.</trap>
    <trap>Severity in Fix backlog is non-elastic. If everything is "Important," re-prioritize. Critical = "must fix before next milestone starts." A pile of Criticals is a signal the milestone shouldn't have shipped; surface that honestly.</trap>
    <trap>M03.5 has no gap-analysis entry by design (doc/protocol-only). Don't try to audit M03.5 in M04's entry — its outputs are inputs to M04 (validator script v1.4 deliverable, etc.) but M03.5 itself is not a §20-bound parent milestone.</trap>
  </gotchas>

  <execution_warnings>
    <warning>This is the FINAL commit on the M04 branch. After commit + approval, push the branch (first push for the milestone). PR draft surfaces; PR creation waits for explicit go-ahead per CLAUDE.md §20 + the established convention from M03.5.</warning>
    <warning>DO NOT close the M03.5 carry-forward items if M04 didn't fully close them — they forward to M05+ via the new M04 entry's Carry-forward section.</warning>
  </execution_warnings>

  <time_box estimate_hours="2.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for M05 prompt authoring: which M04 carry-forward items inform M05 stage decomposition; which v1.3 protocol tags surfaced friction (candidate v1.4 changes); whether the M04 +20% time-box buffer was honored or ratio drifted; final disposition of the validator-script v1.4 deliverable (still M05 carry-forward).</special_log>
    <m_summary_required>true</m_summary_required>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="G.6 Commit Message"/>

  <approval_surface>
    <item>The full M04 gap-analysis entry text</item>
    <item>append-only check output (must be empty)</item>
    <item>M04-summary.md contents (parent-milestone verdict)</item>
    <item>3-artifact review per CLAUDE.md §20: code diff stats; retrospective summaries; gap-analysis entry</item>
    <item>draft commit message from G.6</item>
    <item>draft PR description (do NOT open the PR)</item>
    <item>"Stage M04.G is ready. I will not commit until you approve. Once committed, prior gap-analysis entries are immutable forever per CLAUDE.md §20. After approval I will push the milestone branch and surface the PR draft; PR creation waits for explicit go-ahead."</item>
  </approval_surface>
</closeout_stage_prompt>
```

### G.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
docs(gap-analysis): M04 — append cumulative product+spec audit

Per CLAUDE.md §20. Reviews codebase to date (M01 + M02 + M03 + M03.5 +
M04) against agent-runtime-spec.md. Records adherence findings, spec
gaps, and prioritized fix backlog. This entry is immutable — future
milestones report status via Carry-forward.

Includes <gotchas_graduation> audit of M04.A1-F per-stage gotchas with
disposition (kept | graduated | resolved | expired) per
STAGE-PROMPT-PROTOCOL.md v1.2 closeout schema.

Final disposition for M02 + M03 carry-forward 🟡 items closed in M04
Stages A1 + A2 + B; M03.5 carry-forward (validator script v1.4 +
verification-regex dry-run + estimation calibration) forwards to M05.

Refs: M04-plan-verify-hitl-budget.md §G, all M04.A1-F retros, M04-summary.md

https://claude.ai/code
EOF
)"
```

---

## Summary Table

| Stage | Status | New Files | Edited Files | Tests Added | Effort |
|---|---|---|---|---|---|
| **A1** Build hygiene | ⏳ NEXT | 3 (`generated/error.rs`, `generated/event.rs`, `error.ts` — under `generated/` submodule per A1 convention; namespace clash with existing top-level hand-curated `error.rs::RuntimeError` resolved at execution time) | 4 (xtask, lib.rs, client.rs test, CHANGELOG) | 1 unit (`await_event` timeout via `tokio::test(start_paused=true)`) | ~2.5h |
| **A2** Production wiring | ⏳ | 2 (`drone_lifecycle.rs` sibling of `main.rs` since `lib.rs` does not exist; `drone_reconnect_events.rs` integration test) | 5 (Tauri main.rs + commands.rs replacing 3 noop callsites at :166/:200/:247, anthropic.rs count_tokens at :135, ipc.ts unwrapCmdError, possibly spec §1d) | 4 wiremock count_tokens + reconnect integration + drone_lifecycle unit | ~4.5h |
| **B** §3a Plan/Task primitive (folds original-A3 work) | ⏳ | 7 (plan + task schemas, generated, plan/state_machine.rs, first migration creates `migrations/` dir, prompt_template.rs, plan_lifecycle.rs integration) | ~9 (xtask +2 to 9 entries, event.rs adds 5 missing variants, drone.rs adds WriteSignal variant, command_handler.rs adds WriteSignal arm, event_pipeline.rs emits WriteSignal, decision_extractor.rs structured emitter, agent_sdk.rs plan integration, graphStore +5 cases, lib.rs) | exhaustive FSM + plan_lifecycle.rs integration + structured-emitter unit + WriteSignal IPC integration | ~5–7h |
| **C** Plan UI + ApprovalPanel | ⏳ | 4 (ApprovalPanel + 3 test files including Playwright) | ~6 (PlanNode/TaskNode visual, ipc.ts +3 wrappers, commands.rs +3 commands, drone.rs +3 IPC variants, command_handler.rs +3 arms, App.tsx) | Vitest + Playwright + 3 IPC integration | ~3–5h |
| **D** §4a Verify & Rails (4 events + RevertToSnapshot already shipped) | ⏳ | 6 (hook.v1.json + generated + 4 hooks/ modules + hook_integration.rs) | ~3 (xtask +1 to 10, agent_sdk.rs hook integration + pre_file_edit, VerifyNode/HookNode visual, possibly drone.rs HookRollback shape + spec §4a pre_file_edit row) | exhaustive rails + hooks integration + cross-platform shell | ~5–7h |
| **E** §6a HITL (HitlRequested + HitlResolved already shipped) | ⏳ | 11 (hitl.v1.json + generated + 4 hitl/ modules + 3 notifiers + 3 panels + hitl_failure_escalation.rs + Playwright) | ~9 (xtask +1 to 11, event.rs +3 new HITL variants only, drone.rs +RespondHitl, command_handler.rs +RespondHitl arm, commands.rs +respond_hitl, deps, agent_sdk.rs flow, App.tsx, graphStore +3 cases) | unit + integration + Playwright + cross-stack discipline | ~5–7h |
| **F** §2a Budget + §1b Recovery (4 budget events already shipped; vdr-via-drone-IPC) | ⏳ | 9 (budget.v1.json + generated + budget/ + recovery/ with mcp_reconnect_stub for M06 + 3 panels + budget_threshold.rs + recovery_lifecycle.rs + 2 Playwright) | ~6 (xtask +1 to 12, agent_sdk.rs budget enforcer, conditionally drone.rs +QueryVdr + handler arm, commands.rs +3 commands, App.tsx; NO event.rs/graphStore/snapshot.rs touches per audit) | unit + integration + Playwright | ~4–6h |
| **G** Phase Closeout | ⏳ | 1 (M04-summary.md) | 2 (gap-analysis.md append-only, CHANGELOG.md) | None (doc-only) | ~2–3h |
| **Total** | All ⏳ | ~43 new files | ~44 edited files | 30+ tests across unit/integration/Playwright | ~32–45h estimated; ~10–18h actual at calibration ratios (M01 0.3× / M02 0.7× / M03 0.32×; doc-only Stage G at 0.20×) |

---

## Verification Checklist

Before approving the M04 PR (Stage G's surface), verify:

### Automated (gates)

- [ ] `cargo fmt --all -- --check` — zero diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- [ ] `cargo build --workspace` — succeeds on Linux/macOS/Windows × stable + MSRV
- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo llvm-cov` — workspace ≥80%, runtime-drone ≥95%, runtime-main ≥95% with documented exclusions
- [ ] M04 safety primitives ≥95%: `crates/runtime-main/src/plan/state_machine.rs` (Stage B), `crates/runtime-main/src/hooks/` (Stage D), `crates/runtime-main/src/hitl/` (Stage E), `crates/runtime-main/src/budget/` + `recovery/` (Stage F)
- [ ] `cargo audit` clean, `cargo deny check` clean
- [ ] `npx prettier --check '**/*.{ts,tsx,js,jsx,json}'` — clean
- [ ] `npx eslint .` — clean
- [ ] `npx tsc --noEmit` — clean
- [ ] `npm run test` (Vitest) — all tests pass; `src/` ≥80% coverage
- [ ] `npm audit --audit-level=high` — zero high/critical
- [ ] `npm run test:e2e` (Playwright) — all renderer-level E2E tests pass (plan_approval, hitl_failure_escalation, budget_threshold, recovery_uncertainty)
- [ ] `cargo xtask regenerate-types --check` — zero diff (schema-as-source-of-truth invariant)
- [ ] CI green on all OS × toolchain cells; `e2e-tauri-driver` job stays disabled per Key constraints
- [ ] Codecov delta gates pass (no regression > 0.5pp on any gated crate)

### Manual

- [ ] All MVP §M4 acceptance criteria checked off:
  - [ ] Loads `examples/aria/framework.json` (v0.1-stripped); orchestrator spawns; planner generates 3-task plan; HITL approval surfaces; user approves; tasks execute
  - [ ] Each `task_completed` triggers `post_task` hook (PowerShell `bash .aria/verify.sh` shim returning 0); pass → next task; fail with `on_failure: rollback` → drone reverts → retry
  - [ ] `failure_count >= max_failures` → HITL escalation panel; user picks retry/skip/abort
  - [ ] Budget threshold breach → `BudgetWarn` event + toast at 50%, `BudgetDownshift` at 75%, `BudgetSuspended` HITL approval at 90%, `BudgetExceeded` + drone stop_process at 100% (codebase event NAMES per `event.rs:321–353` — NOT `BudgetWarning`)
  - [ ] User closes app mid-session; reopens; recovery dialog offers resume; resumed session continues from last snapshot with task pointer reset
- [ ] All M04 stage retrospectives present (A1, A2, B, C, D, E, F) and filled in
- [ ] `M04-summary.md` aggregates across stages with verdict ("Pattern held" / "Pattern held with friction" / "Pattern strained")
- [ ] `docs/gap-analysis.md` M04 entry committed; prior entries (M01, M02, M03) unchanged (CI append-only check passes)
- [ ] `<gotchas_graduation>` audit complete — every M04.A1-F stage has a `<stage_review>` entry with disposition
- [ ] M04 PR description references all 8 stage commits + retrospectives + summary + gap-analysis entry
- [ ] CHANGELOG `[Unreleased]` reflects what M04 actually delivered
- [ ] M02 + M03 carry-forward final disposition recorded in M04 gap-analysis entry's Carry-forward section
- [ ] M03.5 carry-forward (validator script v1.4 + verification-regex dry-run + estimation calibration) forwarded to M05

### Approval gate (per CLAUDE.md §19)

- [ ] **Hard Gate G1: do-not-commit-until-approved held** — every stage commit happened only after explicit user approval (8 approval gates across Stages A1, A2, B, C, D, E, F, G; original eight-stage plan included a separate A3 for vdr WriteSignal IPC + structured emitter that was folded into Stage B per the post-M03.5 codebase audit re-staging — net 7 work stages + Stage G closeout = 8 commits + 8 approval gates)
- [ ] User has reviewed each stage retrospective; scoring matches observable evidence
- [ ] M04-summary verdict is "Pattern held" (sound) or "Pattern held with friction"; not "Pattern strained"
- [ ] Three-artifact review per CLAUDE.md §20 complete: code diff + retrospectives/summary + gap-analysis entry all reviewed together
- [ ] PR creation deferred to explicit user instruction (do NOT auto-open per established convention)

---

*End of M04 specification + stage prompts. Eight stages on one parent-milestone branch (`claude/m04-plan-verify-hitl-budget`); Stage G is Phase Closeout per CLAUDE.md §20. PR drafts at end of Stage G and pushes after explicit approval. M05 (gap detection + capability enforcement) follows on a separate branch once this milestone merges.*
