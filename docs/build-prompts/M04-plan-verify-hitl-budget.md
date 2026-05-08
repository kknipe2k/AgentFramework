# M04 Plan + Verify + HITL + Budget — Specification + Stage Prompts

**Protocol version:** v1.3 (first milestone authored on the v1.3 XML stage-prompt schema; uses `<pre_flight_check>`, `<schema_drift_check>`, `<fan_out_grep>`, `<dependency_audit_check>`, and `<runtime_environment>` tags per `STAGE-PROMPT-PROTOCOL.md` v1.3).
**Date:** 2026-05-07 (initial authoring); 2026-05-08 (revised post-M04.A2 audit per `docs/gotchas.md` #41 + STAGE-PROMPT-PROTOCOL.md §10 v1.3 hardening rule)
**Status:** Design approved — A1 + A2 committed on milestone branch; B–G revised to match codebase reality per post-A2 audit
**Scope:** Build the four agentic primitives that turn a single-agent smoke into an actual agentic runtime: §3a Plan & Task (with `fresh_context_per_task` loop policy + 11 plan/task events + ApprovalPanel), §4a Verify & Rails (Hook primitive with 7 firing points including new `pre_file_edit` + Rails hard/soft + don't-touch globs + existing `RevertToSnapshot` drone command — already in `DroneCommand`), §6a HITL (9 trigger types + 3 UI variants Panel/Modal/Toast + 3 built-in notifiers terminal_bell/desktop/sound + plugin interface), §2a Budget (3 scopes + 4 threshold actions + downshift_hook + 4 budget events + UI header bar). Plus §1b Recovery (resume rebuilds history, tool-call-uncertain prompt). Seven stages on one feature branch (`claude/m04-plan-verify-hitl-budget`): A1 + A2 committed; B + C + D + E + F + G remaining. Stage G is Phase Closeout per CLAUDE.md §20. Spec §1b + §2a + §3a + §4a + §6a + MVP §M4 acceptance criteria.

---

## Background and Design Decision

> **Revised post-M04.A2 audit (2026-05-08).** A2's execution surfaced that the original Phase doc claimed integration points that didn't exist (`event_translation.rs`, `prompt_template.rs`, `WriteSignal` IPC variant, `vdr.rs` in `runtime-main`). A post-A2 codebase audit then found ~30% of M04's "new work" already landed incidentally during M03 — gap-analysis ledger is append-only and items "carry forward" but don't auto-close when work happens incidentally. Per `docs/gotchas.md` #41 + STAGE-PROMPT-PROTOCOL.md §10 v1.3 hardening rule (PR #51), Stages B + C + D + E + F have been revised to match codebase reality. Summary of audit-driven changes: (1) Stage B was 11 NEW plan/task events — 8 already exist in `event.rs`; revised to "wire 8 + add 3 missing"; folds in the previously-deferred A2 work (vdr WriteSignal IPC + structured-emitter prompt template) since Stage B has scope slack. (2) Stage D was 4 new `hook_*` events + `RevertToSnapshot` drone command — events already exist as `verify_*` + `rail_triggered`; `RevertToSnapshot` already in `DroneCommand`. Adopt codebase names; skip re-adding. (3) Stage E was `hitl_response` event — codebase has `hitl_resolved`. Rename. (4) Stage F was 4 new budget events — all already exist; revised to "wire enforcer + author recovery primitive"; vdr access via drone IPC, not `runtime-main/src/vdr.rs` (phantom). (5) Stage G unchanged. Original eight-stage plan is now seven (no A3; A3 work folded into B). Path corrections throughout: `event_translation.rs` → `event_pipeline.rs`. Audit findings serve as the grep-verification proof per gotcha #41.

**Problem.** M03 lit up the live graph for the M02 single-agent smoke session — one AgentNode renders, click-to-inspect works, token weight scales. The other 10 node types (PlanNode, TaskNode, VerifyNode, HookNode, GapNode, HITLNode, MCPNode + four Plan/Task event types) render in unit tests with synthetic state but never light up live: no event source fires their corresponding `AgentEvent` variants. Spec §M4 declares the four primitives (plan, verify, HITL, budget) that produce those events. Loading `examples/aria/framework.json` and seeing a multi-task plan render with verify hooks firing post-task and the budget header bar tracking session spend is the M04 success surface.

**Solution.** Seven stages on one feature branch (`claude/m04-plan-verify-hitl-budget`), each a fresh Claude Code session per the v1.3 XML stage-prompt protocol. **Stage A1 (DONE)** closed M03 build-hygiene carry-forward: xtask codegen extensions for `event.v1.json` + `error.v1.json`, drone-test retrofits, `tokio::time::pause()` coverage. **Stage A2 (DONE)** landed production wiring: drone subprocess lifecycle at Tauri startup with `Arc<DroneClient>` Tauri-managed-state, replaced `DroneClient::noop()` callsites, real `count_tokens` Anthropic endpoint, `unwrapCmdError` consumes generated `CmdError`, long-lived `events()` reconnect locked as v0.1 behavior (subscribers must resubscribe). **Stage B** builds the §3a Plan & Task primitive — wires the 8 already-shipped plan/task event variants in `event.rs`, adds the 3 missing variants, authors `plan.v1.json` + `task.v1.json` schemas (xtask codegen), plan state machine with `fresh_context_per_task` loop policy, failure escalation, plans + tasks SQLite tables (per spec §10 DDL added in M03.5; first migration also creates the `crates/runtime-drone/migrations/` directory). **Stage B also folds in A2's deferred work** (vdr WriteSignal IPC command + handler + main-side emission path; structured-emitter prompt-template module + AgentSdk plumbing) — Stage B has scope slack from already-shipped events to absorb it. **Stage C** lights up the renderer surface — wires already-shipped PlanNode/TaskNode (M03.C synthetic) to live event variants, builds the ApprovalPanel for plan approval gate, threads the approval flow renderer→main→drone→main→renderer; adds `<pre_flight_check>` for `Arc<DroneClient>` from A2. **Stage D** builds §4a Verify & Rails — adopts existing `verify_*` + `rail_triggered` event names (NOT new `hook_*` per audit; events already in `event.rs`); skips re-adding `RevertToSnapshot` drone command (already in `DroneCommand` with `RevertReason` enum). New work: Hook primitive (HookRef + 7 firing points including new `pre_file_edit`), Rails (hard/soft + JSON-declared), don't-touch glob matcher, VerifyNode + HookNode wired to the existing events. **Stage E** builds §6a HITL — 9 trigger types, 3 UI variants (Panel/Modal/Toast), notifier plugin interface, 3 built-in notifiers (terminal_bell/desktop via Tauri notification plugin/sound), 5 HITL events (`hitl_requested` + `hitl_resolved` existing — NOT `hitl_response`; codebase name is `hitl_resolved`; plus `hitl_timeout` + `notifier_dispatched` + `notifier_failed` new), failure-escalation flow (`task_escalated` → `on_failure_threshold` → `hitl_requested` → notifiers parallel → 1h default timeout); `<pre_flight_check>` for `Arc<DroneClient>`. **Stage F** builds §2a Budget + §1b Recovery — 4 budget events ALREADY exist in `event.rs` + `graphStore.ts` (audit confirmed); Stage F wires the budget enforcer to emit them (NOT to add them). VDR access goes through drone IPC, not `runtime-main/src/vdr.rs` (phantom path; vdr lives in `runtime-drone/src/vdr.rs`). New work: budget enforcer logic + 3 scopes + 4 threshold actions + downshift_hook + session header bar UI; Recovery primitive (resume rebuilds history not re-execute, tool-call-uncertain UI prompt, MCP reconnect on resume, plan state restoration). **Stage G** is Phase Closeout — gap-analysis entry per CLAUDE.md §20, M04 summary, three-artifact review, `<gotchas_graduation>` audit of A1–F per-stage gotchas.

**Why one PR for the parent milestone (not one PR per stage).** Same logic as M01–M03 — seven stages-as-commits-on-one-branch gives incremental discipline (each stage is reviewable; each stage retrospective surfaces friction early) without the overhead of seven PR reviews for one logical milestone. Consistent with the per-milestone-as-PR pattern in `docs/build-prompts/README.md`. M03 (six stages, ~10h actual) proved the pattern at scale.

**Why seven stages, not eight.** Original plan was eight (A1, A2, B, C, D, E, F, G) plus a proposed A3 for VDR + structured-emitter wiring deferred from A2. Post-A2 audit revealed Stage B has scope slack (8/11 plan/task events already exist) — A3's work folds naturally into the revised Stage B without exceeding scope-split threshold. Net: 7 stages, no separate A3. Calibrated estimate: ~25–35h actual (down from original ~39–54h after the audit removed already-done work from B/D/F).

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
| `crates/runtime-core/src/error.rs` | DOES NOT EXIST — `CmdError` enum lives at `src-tauri/src/commands.rs` | **CREATE in Stage A1** via xtask extension; runtime-main + runtime-drone consumers can now reference shared error types |
| `src/lib/ipc.ts::unwrapCmdError` | M02; hand-maintained `CmdError` discriminated union | **REFACTOR in Stage A2** to import the generated `CmdError` type from `src/types/error.ts`; preserve unwrap semantics per gotcha #30 |
| `crates/runtime-drone/tests/integration*.rs` | M03.A current_exe()-derived paths landed | **VERIFY clean in Stage A1** — confirm no remaining `target/debug` literals; if any stragglers exist (Stage A1 of M03 missed some), retrofit |
| `crates/runtime-main/src/drone_ipc/client.rs::await_event` | M02; timeout path lacks `tokio::time::pause()` coverage | **ADD COVERAGE in Stage A1** — closes M03 carry-forward; archetype: `connection.rs::backoff_grows_exponentially_between_attempts` |
| `src-tauri/src/lib.rs` | M02 Tauri shell setup; runs `DroneClient::noop()` in M03 | **REFACTOR in Stage A2** — spawn drone subprocess at app startup, manage `Arc<DroneClient>` via Tauri managed state, graceful shutdown on app exit |
| `src-tauri/src/commands.rs::query_session_db` + `replay_session` | M03.E; both noop'd via `DroneClient::noop()` | **REFACTOR in Stage A2** — replace noop with real drone IPC dispatch; SQL inspector + replay-from-signals become end-to-end functional |
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
| **A1** | ✅ DONE | Build hygiene — xtask codegen extensions for `event.v1.json` + `error.v1.json` (Rust + TS); regenerated types into `crates/runtime-core/src/generated/`; closed `await_event` `tokio::time::pause()` coverage; verified drone integration test current_exe() paths clean | ~1h actual |
| **A2** | ✅ DONE | Production wiring — drone subprocess lifecycle at Tauri startup with `Arc<DroneClient>` Tauri-managed-state; replaced `DroneClient::noop()` callsites in `query_session_db` + `replay_session` + `run_smoke_session`; real `count_tokens` against `/v1/messages/count_tokens` (wiremock-tested); CmdError migration (full); `unwrapCmdError` consumes generated `CmdError`; long-lived `events()` reconnect locked as v0.1 behavior (subscribers must resubscribe). **Two items deferred from A2 fold into Stage B per post-A2 audit:** vdr WriteSignal IPC + structured-emitter prompt template | ~3h actual |
| **B** | ⏳ NEXT | §3a Plan & Task primitive — author `plan.v1.json` + `task.v1.json` schemas; xtask codegen; Plan/Task Rust types in runtime-core; **wire 8 already-shipped plan/task events + add 3 missing variants** (audit confirmed 8/11 already in `event.rs`); plan state machine (safety primitive ≥95%); `fresh_context_per_task` loop policy; failure escalation; SQLite `plans` + `tasks` tables (first migration creates `crates/runtime-drone/migrations/`); approval-gate seam. **Folds A2 deferrals:** WriteSignal IPC command + handler + main-side emission path; structured-emitter prompt-template module + AgentSdk plumbing | ~5–7h |
| **C** | ⏳ | §3a Plan UI + ApprovalPanel + graph wiring — wire already-shipped `PlanNode` + `TaskNode` (M03.C synthetic) to live event variants; ApprovalPanel renderer + approval-gate flow (renderer→main→drone→main→renderer); plan abort + replan + revise flows. Adds `<pre_flight_check>` for `Arc<DroneClient>` from A2 | ~3–5h |
| **D** | ⏳ | §4a Verify & Rails — `hook.v1.json` schema (HookRef + HookCategory + Hook); Hook primitive with 7 firing points (existing 6 + new `pre_file_edit`); Rails primitive (hard/soft + JSON-declared); don't-touch glob matcher; VerifyNode + HookNode wired to existing live events. **Audit-corrected:** event names are `verify_started/passed/failed` + `rail_triggered` (already in `event.rs` per audit), NOT new `hook_*`; `RevertToSnapshot` already exists in `DroneCommand` with `RevertReason` enum — Stage D consumes it, does NOT re-add | ~5–7h |
| **E** | ⏳ | §6a HITL — `hitl.v1.json` schema (9 trigger types + 3 UI variants + notifier plugin interface); 3 built-in notifiers (terminal_bell/desktop via Tauri notification plugin/sound); HITL events (codebase has `hitl_requested` + `hitl_resolved` — NOT `hitl_response`; plus 3 new: `hitl_timeout` + `notifier_dispatched` + `notifier_failed`); failure-escalation flow `task_escalated` → `on_failure_threshold` → `hitl_requested` → notifiers parallel → 1h timeout; HITL Panel + Modal + Toast renderer surfaces. Adds `<pre_flight_check>` for `Arc<DroneClient>` | ~5–7h |
| **F** | ⏳ | §2a Budget + §1b Recovery — `budget.v1.json` schema (3 scopes + 4 actions + downshift_hook); **wire enforcer to emit 4 already-shipped budget events** (audit confirmed all 4 in `event.rs` + `graphStore.ts`); session header bar UI; Recovery (resume rebuilds history per spec §1b; tool-call-uncertain UI prompt with retry/skip/mark-complete/abort options; MCP reconnect on resume; plan state restoration; capability state restoration). **Audit-corrected:** vdr access via drone IPC (vdr lives in `runtime-drone/src/vdr.rs`, NOT `runtime-main/src/vdr.rs` — phantom path). Adds `<pre_flight_check>` for `Arc<DroneClient>` | ~4–6h |
| **G** | ⏳ | Phase Closeout — gap-analysis entry per CLAUDE.md §20 (cumulative product↔spec audit including M04 + cumulative review); `<gotchas_graduation>` v1.2 closeout subsection auditing all per-stage `<gotchas>` from A1–F (kept | graduated | resolved | expired); M03 + M03.5 carry-forward final disposition; M04-summary.md aggregating across stages; three-artifact review (CLAUDE.md §20) | ~2–3h |

Total revised estimate: ~25–35 hours estimated for B–G (A1+A2 actuals: ~4h). ~10–12 hours human direction (seven approval gates + one PR review).

**Estimation calibration (revised post-A2 audit).** Original M04: 32–45h calibrated, 12–17h actual at M03 0.32× ratio + 20% buffer. Post-audit: scope reduction in B/D/F removes ~10h of "already-done" work that the original Phase doc claimed as new. Revised total: ~25–35h calibrated, ~8–12h actual. M01 ran 0.3× of estimate; M02 0.7×; M03 0.32×; M03.5 0.14× (doc-only); M04.A1 0.4× actual on 2.5h estimate; M04.A2 0.67× actual on 4.5h estimate. Stages B–F likely track 0.30×–0.50× (code-shipping with cross-stack glue); Stage G ~0.20× (doc-only closeout per M03.F).

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
- `crates/runtime-core/src/error.rs` (new; generated from `error.v1.json` via xtask)
- `src/types/error.ts` (new; generated from `error.v1.json` via xtask)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (add event.v1.json + error.v1.json to codegen list; wire Rust typify + TS json-schema-to-typescript outputs)
- `crates/runtime-core/src/lib.rs` (export the new `error` module if codegen produces a freestanding file; otherwise verify integration with existing `event` module)
- `crates/runtime-core/src/event.rs` (regenerated; verify byte-near-identical to current; address any drift)
- `src/types/agent_event.ts` (regenerated; verify byte-near-identical)
- `crates/runtime-main/src/drone_ipc/client.rs` (add `tokio::time::pause()`-driven timeout test; no production-code changes)
- `CHANGELOG.md` (`[Unreleased]` notes the M04 Stage A1 hygiene closures)

### A1.2 Files to Change

| File | Change |
|---|---|
| `crates/xtask/src/main.rs` | **Edited** — extend codegen list with `event.v1.json` + `error.v1.json` (Rust typify + TS json-schema-to-typescript outputs) |
| `crates/runtime-core/src/error.rs` | **New** — generated from `error.v1.json` via xtask (5-variant tagged enum) |
| `src/types/error.ts` | **New** — generated from `error.v1.json` via xtask (5-variant discriminated union) |
| `crates/runtime-core/src/lib.rs` | **Edited (if needed)** — export the new `error` module per the codegen file structure |
| `crates/runtime-core/src/event.rs` | **Edited (regen)** — verify byte-near-identical to current hand-maintained shape; address drift |
| `src/types/agent_event.ts` | **Edited (regen)** — verify byte-near-identical |
| `crates/runtime-main/src/drone_ipc/client.rs` | **Edited (test only)** — add `tokio::time::pause()`-driven timeout test for `await_event` path |
| `crates/runtime-drone/tests/integration*.rs` | **Verified clean (no edits expected)** — confirm zero `target/debug` literals; retrofit if any remain |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes M04 Stage A1 hygiene closures |

### A1.3 Detailed Changes

#### `crates/xtask/src/main.rs` — extend codegen list

Locate the existing schemas list in the xtask codegen function (M03.A added it; structure is a `&[(name, path)]` slice or similar). Add two entries after the existing entries:

- `("event", "schemas/event.v1.json")` — outputs to `crates/runtime-core/src/event.rs` (Rust) + `src/types/agent_event.ts` (TS)
- `("error", "schemas/error.v1.json")` — outputs to `crates/runtime-core/src/error.rs` (Rust) + `src/types/error.ts` (TS)

The Rust output uses typify (existing M03.A integration); TS output uses json-schema-to-typescript via Node CLI invocation (existing M03.A pattern via `std::process::Command::new("npx").args(["json-schema-to-typescript", schema_path]).output()`).

The `--check` flag (drift detection) compares regenerated output to committed file via byte-diff; non-zero exit if any diff.

#### `crates/runtime-core/src/error.rs` — new generated file

Generated from `schemas/error.v1.json` (5-variant `oneOf`). The output is a Rust enum with `serde(tag = "type", rename_all = "snake_case")` matching the schema's `serde` encoding declared in the existing `src-tauri/src/commands.rs::CmdError` (which becomes a re-export of the generated type after Stage A2 wires it).

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

#### `src/types/error.ts` — new generated file

Generated from `schemas/error.v1.json` via json-schema-to-typescript. Expected output: a `CmdError` discriminated union matching the existing `src/lib/ipc.ts::CmdError` interface (which becomes a re-export after Stage A2 refactor). The generator may produce an `export type CmdError = { type: 'setup_required' } | { type: 'provider'; message: string } | ...` form or an interface-based form; accept whatever json-schema-to-typescript produces and update consumers in Stage A2.

#### `crates/runtime-core/src/lib.rs` — export `error` module

Add `pub mod error;` if the codegen produces a freestanding `error.rs` file. Verify the existing `pub mod event;` line is unchanged (regen of `event.rs` should not affect the module declaration).

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
    Stage A1 of M04 (Plan + Verify + HITL + Budget). Build hygiene + xtask codegen extensions + coverage retrofits. Closes three M03 carry-forward 🟡 build-hygiene items so Stages A2-G focus on production wiring + new primitive surface. Stage A2 does not start until Stage A1's commit is on the milestone branch claude/m04-plan-verify-hitl-budget. First milestone authored on the v1.3 XML stage-prompt protocol — uses <schema_drift_check> + <runtime_environment> tags below.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Background, Document Structure, Implementation Workflow, Pre-existing legacy file inventory, Stage A1 sections A1.1–A1.4)</file>
    <file>agent-runtime-spec.md §0–§0d, §1d, §2c, §13.5</file>
    <file>docs/MVP-v0.1.md §M4</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="M03.A xtask codegen archetype to extend">crates/xtask/src/main.rs</file>
    <file purpose="hand-maintained event types about to be regenerated; verify near-byte-identical post-regen">crates/runtime-core/src/event.rs</file>
    <file purpose="schema source for new error type codegen target">schemas/error.v1.json</file>
    <file purpose="hand-maintained CmdError that error.rs will replace in Stage A2">src-tauri/src/commands.rs</file>
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

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <runtime_environment os="windows" note="Build agent runs on Windows 11 per the established M01-M03.5 pattern; Select-String is the assumed grep equivalent throughout the prompt; Test-Path replaces test -f"/>

  <gotchas>
    <trap>Stage A1's job is to close M03 build-hygiene carry-forward + extend xtask codegen, not to start Stage A2's production wiring — resist scope creep into drone subprocess spawning even if the regenerated types make it tempting</trap>
    <trap>typify-generated Rust types may not match the hand-maintained event.rs byte-for-byte — accept the generated output and update consumers in subsequent stages rather than hand-editing the generated file (gotcha #14 snake_case schema discipline applies here)</trap>
    <trap>json-schema-to-typescript may produce a TS shape that differs from the M02 hand-maintained CmdError interface (e.g., interface vs type alias, strict vs loose discriminator) — Stage A2 owns the consumer refactor; A1 only commits the generated output</trap>
    <trap>tokio::time::pause() requires #[tokio::test(start_paused = true)] OR explicit tokio::time::pause() at test start — the latter pattern from M01.C is acceptable but the former is cleaner; pick one and document the choice</trap>
    <trap>If event.rs regen produces drift from the hand-maintained version, that's M03.A drift — surface the diff in the retrospective so future schema edits don't recur the issue; do NOT silently accept changes that affect runtime behavior without flagging them</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT touch src-tauri/src/commands.rs::CmdError in Stage A1 — that's Stage A2's refactor (replace with re-export of generated type). Stage A1 only commits the generated output file.</warning>
    <warning>DO NOT regenerate framework/skill/agent/tool/common schemas — only event.v1.json + error.v1.json get extended codegen. Existing schemas were already regenerated in M01–M03.</warning>
    <warning>DO NOT push between stages — Stage A1 commits locally only. The push happens at end of Stage G per CLAUDE.md §8 + §20.</warning>
    <warning>The cargo xtask regenerate-types --check command must produce zero diff after the regen step — if there's persistent drift between regen passes, the codegen is non-deterministic and needs fixing (sorted fields, normalized whitespace, deterministic comments) before committing.</warning>
  </execution_warnings>

  <time_box estimate_hours="2.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage A2: any drift discovered between hand-maintained event.rs and regen output (if so, was it pre-existing or did regen introduce it?); whether json-schema-to-typescript output requires Stage A2 consumer refactor (likely yes given M02 hand-maintained shape predates the schema); whether the await_event timeout test surfaces any other timeout-related bugs in client.rs that weren't covered by the existing tests; whether the drone integration test current_exe() retrofit was clean or revealed additional path-derivation issues.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="A1.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers including the new client.rs coverage 100%)</item>
    <item>schema drift check output — cargo xtask regenerate-types --check exit code + diff if any</item>
    <item>generated file shape preview — first 30 lines of crates/runtime-core/src/error.rs + first 30 lines of src/types/error.ts so the human can spot-check shape</item>
    <item>any drift discovered in event.rs regen (diff with original hand-maintained content, or "byte-identical")</item>
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
extensions + coverage retrofit + drift verification.

Carry-forward closures:
- crates/xtask/src/main.rs: extended codegen to event.v1.json +
  error.v1.json (Rust typify + TS json-schema-to-typescript). Replaces
  hand-maintained crates/runtime-core/src/event.rs with regen output;
  adds new generated crates/runtime-core/src/error.rs + src/types/
  error.ts (consumers refactor in Stage A2).
- crates/runtime-main/src/drone_ipc/client.rs: tokio::time::pause()-
  driven test for await_event timeout path. Closes 100% → 94% regression
  on client.rs coverage from M03.D retro.
- crates/runtime-drone/tests/integration*.rs: verified clean of
  target/debug literals (per docs/gotchas.md #22; M03.A retrofit
  confirmed durable).

CHANGELOG.md [Unreleased] reflects the closures. No source-code behavior
changes; codegen output may differ structurally from hand-maintained
event.rs in trivial ways (sorted derive order, doc-comment style) —
verify byte-near-identical via diff and document any meaningful drift.

Refs: M04-plan-verify-hitl-budget.md §A1, gap-analysis.md M03 entry 🟡
(xtask event.v1.json codegen + await_event coverage)
Retrospective: docs/build-prompts/retrospectives/M04.A1-retrospective.md

https://claude.ai/code
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE A2 — Production wiring                                   -->
<!-- ============================================================ -->

## Stage A2 — Production wiring (drone subprocess + vdr.rs projector + decision extractor + count_tokens + events() reconnect)

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.rs/tokio/latest/tokio/process/struct.Command.html> — confirm `tokio::process::Command` API for drone subprocess spawn is unchanged from M01.C (already in use); review `kill_on_drop` + child stdio handling
- <https://v2.tauri.app/develop/state-management/> — confirm Tauri 2.x managed-state API is unchanged; `Arc<DroneClient>` registered via `app.manage(...)` and accessed in commands via `tauri::State<'_, Arc<DroneClient>>`
- <https://docs.anthropic.com/en/api/messages-count-tokens> — confirm the `POST /v1/messages/count_tokens` endpoint URL + request shape + response shape are current; Stage A2 wires the real call
- <https://docs.rs/reqwest/latest/reqwest/> — `reqwest::Client::post` API is unchanged from M02.C; confirm
- <https://docs.rs/keyring/latest/keyring/> — keyring 3.6 (per gotcha #29 + Cargo.toml workspace pin) — confirm no breaking change relevant to this stage (Stage A2 doesn't touch keyring directly; included for cross-stack discipline)

### A2.1 Problem Statement

Stage A2 wires the production paths M03 deferred via `DroneClient::noop()`, plus closes four M02/M03 carry-forward 🟡 items that block downstream stages:

1. **Drone subprocess lifecycle at Tauri startup.** M03.E shipped `DroneClient::noop()` for the Tauri command seams (`query_session_db`, `replay_session`); Stage A2 spawns the real `runtime-drone` subprocess at app startup, registers `Arc<DroneClient>` as Tauri managed state, and wires graceful shutdown on app exit. SQL inspector + replay-from-signals become end-to-end functional. Closes gap-analysis M03 🟡 entry "Production drone subprocess wiring at Tauri startup".

2. **VDR projector wired at signal-write call-site.** M03 added the `vdr` module + projection logic but never called it from `WriteSignal`. Stage A2 calls `vdr::project_signal(conn, signal_id)` after each insert in `crates/runtime-main/src/sdk/event_pipeline.rs`. Decisions are now actually projected. Closes gap-analysis M03 🟡 entry "vdr.rs projector wired at signal-write call-site".

3. **Decision extractor → structured emitter migration.** M02 ships a heuristic line-by-line text-scan extractor at `crates/runtime-main/src/sdk/decision_extractor.rs`. Stage A2 replaces it with a structured emitter: prompt template injects a delimited block (e.g., `<<DECISION>>...<<END>>`); SDK parses the block directly via regex. Reduces extraction false-positive rate; matches spec §2b ⚠️ note added in M03.5. Closes gap-analysis M02 🟡 entry "Decision extractor → structured emitter migration".

4. **Real `count_tokens` Anthropic endpoint.** M02 ships a chars/4 approximation in `crates/runtime-main/src/providers/anthropic.rs::count_tokens`. Stage A2 implements the real call to `POST /v1/messages/count_tokens` per spec §2c.3 (added M03.5). Wiremock test covers happy path + error mapping. M04 budget enforcement (Stage F) depends on this. Closes gap-analysis M02 🟡 entry "count_tokens → real /v1/messages/count_tokens endpoint".

5. **Long-lived `events()` reconnect resolution.** Per spec §1d ⚠️ note (updated M03.5 from M03 to M04 carry-forward): does the renderer's long-lived `agent_event` subscription survive a mid-session main↔drone reconnect? Stage A2 establishes the answer through a deliberate integration test (kill drone subprocess mid-session, verify the renderer continues to receive events after reconnect). Test-driven decision: if survival works as-implemented, the ⚠️ note becomes a closed item; if not, document the v0.1 behavior (renderer resubscribes on reconnect via M03's replay_session pattern) and update spec text. Closes gap-analysis M02 🟡 entry "Long-lived events() subscription survives reconnect".

6. **`unwrapCmdError` consumes generated types.** Stage A1 generated `crates/runtime-core/src/error.rs` + `src/types/error.ts`. Stage A2 refactors `src/lib/ipc.ts::unwrapCmdError` to import the generated `CmdError` type from `src/types/error.ts` rather than the M02 hand-maintained interface. Preserves unwrap semantics per gotcha #30 (renderer-side typed error unwrap). Closes the consumer-refactor portion of A1's `error.rs` codegen.

**Success criterion:** drone subprocess spawns at Tauri startup; `query_session_db` + `replay_session` invoke real drone IPC and return real data; `vdr` table populates after every signal write; structured decision emitter parses delimited blocks correctly under unit test; wiremock-backed `count_tokens` test passes against the real endpoint shape; long-lived events() reconnect behavior is documented + tested; `unwrapCmdError` uses generated types; all gates pass.

**New artifacts:**
- `src-tauri/src/drone_lifecycle.rs` (new; subprocess spawn + lifecycle + graceful shutdown)
- `crates/runtime-main/tests/drone_reconnect_events.rs` (new integration test for long-lived events() survival)

**Edited artifacts:**
- `src-tauri/src/lib.rs` (spawn drone at app startup; register `Arc<DroneClient>` as Tauri managed state)
- `src-tauri/src/commands.rs` (replace `DroneClient::noop()` in `query_session_db` + `replay_session`; replace hand-maintained `CmdError` enum with re-export of generated type from `runtime-core`)
- `crates/runtime-main/src/sdk/event_pipeline.rs` (call `vdr::project_signal` at WriteSignal)
- `crates/runtime-main/src/sdk/decision_extractor.rs` (replace heuristic with structured emitter)
- `crates/runtime-main/src/providers/anthropic.rs` (implement real `count_tokens` against `/v1/messages/count_tokens`)
- `crates/runtime-main/src/sdk/event_translation.rs` or equivalent (long-lived events() reconnect handling — verify or implement per A2.1 #5)
- `crates/runtime-main/tests/anthropic_wiremock.rs` (add `count_tokens` happy-path + error tests)
- `src/lib/ipc.ts` (refactor `unwrapCmdError` to consume generated `CmdError` from `src/types/error.ts`)
- Possibly `agent-runtime-spec.md` §1d (update or close the ⚠️ long-lived events() note based on Stage A2's test outcome)

### A2.2 Files to Change

| File | Change |
|---|---|
| `src-tauri/src/lib.rs` | **Edited** — spawn drone subprocess at `setup` hook; register `Arc<DroneClient>` via `app.manage(...)` |
| `src-tauri/src/drone_lifecycle.rs` | **New** — `DroneLifecycle::spawn`, `DroneLifecycle::shutdown`, RAII drop guard for graceful exit |
| `src-tauri/src/commands.rs` | **Edited** — replace `DroneClient::noop()` in `query_session_db` + `replay_session` with `tauri::State<Arc<DroneClient>>` parameter; replace hand-maintained `CmdError` enum with `pub use runtime_core::error::CmdError` |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | **Edited** — call `vdr::project_signal(conn, signal_id)` after WriteSignal insert |
| `crates/runtime-main/src/sdk/decision_extractor.rs` | **Edited (rewrite)** — structured-emitter parser (regex-based delimited-block extraction) replaces line-by-line heuristic |
| `crates/runtime-main/src/providers/anthropic.rs` | **Edited** — implement `count_tokens` against `POST /v1/messages/count_tokens` |
| `crates/runtime-main/src/sdk/event_translation.rs` | **Edited** — long-lived events() reconnect handling per A2.1 #5 |
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

#### `src-tauri/src/lib.rs` — Tauri `setup` hook

Locate the existing `tauri::Builder::default()` chain. Add a `.setup(|app| { ... })` block that:

1. Resolves the SQLite db path via existing path-resolution helper
2. Calls `DroneLifecycle::spawn(app.handle(), &db_path)` → `Arc<DroneClient>`
3. Registers via `app.manage(drone_client.clone())`
4. Stores the `DroneLifecycle` instance for graceful shutdown (likely via a `OnceLock<Mutex<Option<DroneLifecycle>>>` static or similar — match existing app-state pattern)

Add an `on_window_event` or `on_run_event` handler for `RunEvent::ExitRequested` that calls `DroneLifecycle::shutdown` before propagating exit. Verify the exact Tauri 2.x event hook name + signature against current docs before authoring.

Tracing: log app-startup + drone-spawn correlation per §13.5.

#### `src-tauri/src/commands.rs` — replace noop'd commands

For both `query_session_db` and `replay_session`:

- Add `client: tauri::State<'_, Arc<DroneClient>>` parameter
- Replace `DroneClient::noop()` body with real IPC dispatch via `client.<method>().await`
- Map drone IPC errors to `CmdError::Drone { message }`

Replace the existing `pub enum CmdError { ... }` block with:

```rust
pub use runtime_core::error::CmdError;
```

(Verify `runtime-core` is already in `Cargo.toml` dependencies; M03 added it. If error.rs lives at a different path post-Stage-A1 codegen, adjust accordingly.)

The existing `CmdError::Internal(...)` constructor calls in this file may need shape adjustment if the generated enum has `Internal { message: String }` rather than `Internal(String)` — match the generated output.

#### `crates/runtime-main/src/sdk/event_pipeline.rs` — vdr projector wiring

Locate the `WriteSignal` execution path (typically inside the SDK event loop where signals get inserted into SQLite). After the existing `INSERT INTO signals ...` operation succeeds:

```rust
vdr::project_signal(&conn, signal_id)
    .map_err(|e| tracing::warn!("vdr projection failed for signal {signal_id}: {e}"))
    .ok();
```

Non-blocking: a projection failure is logged but does not fail the signal write (signals are forensic, VDR is a projection). Per spec §2b separation of concerns.

#### `crates/runtime-main/src/sdk/decision_extractor.rs` — structured emitter

Replace the existing heuristic with regex-based delimited-block extraction. Pattern:

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
    Regex::new(r"<<DECISION>>\s*(\{.*?\})\s*<<END>>").unwrap()
});

pub fn extract_decisions(text: &str) -> Vec<Decision> {
    DECISION_BLOCK
        .captures_iter(text)
        .filter_map(|cap| serde_json::from_str::<Decision>(&cap[1]).ok())
        .collect()
}
```

The prompt-template injection (where the model is instructed to emit decisions in the delimited form) lands in `crates/runtime-main/src/sdk/prompt_template.rs` or equivalent — locate the existing system-prompt builder and add the decision-format instructions to the system prompt.

Unit tests: round-trip a known decision through the regex; multi-decision text; malformed-JSON tolerance (skip + log); no-decision text returns empty.

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

#### `crates/runtime-main/src/sdk/event_translation.rs` — events() reconnect

Per spec §1d ⚠️ note (M04 carry-forward). The existing event translation flow takes ProviderEvents and emits AgentEvents. Stage A2's question: if drone↔main reconnects mid-session, does the renderer's `listen('agent_event', ...)` callback continue to receive events?

Test-driven approach (preferred): write the integration test first (in `tests/drone_reconnect_events.rs`) that:

1. Spawns drone, connects main, subscribes renderer-side via the existing IPC pattern
2. Starts a session that emits events
3. Kills the drone subprocess mid-session (SIGTERM via `Child::kill()`)
4. Spawns a fresh drone (simulating Tauri's auto-restart, or invokes existing reconnect logic)
5. Continues the session
6. Asserts renderer continues to receive events

If the test passes as-implemented (M01.C reconnect logic + Tauri event emission already handles this), close the spec ⚠️ note. If not, the test surfaces what's broken and Stage A2 implements the fix (likely involves resubscribing on reconnect or buffering events during the gap).

#### `src/lib/ipc.ts` — generated CmdError consumption

Replace the hand-maintained `interface CmdError { ... }` with `import type { CmdError } from '../types/error';`. Update `unwrapCmdError` if the generated shape differs from the hand-maintained one (likely the discriminator key matches but the variant shape may differ slightly). Preserve all behavior of the helper per gotcha #30.

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
- New unit tests inside `crates/runtime-main/src/sdk/decision_extractor.rs` for structured emitter (round-trip; multi-decision; malformed-JSON tolerance; no-decision)

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
- `runtime-main` ≥95% — new code in `event_pipeline.rs` (vdr projector call), `decision_extractor.rs` (structured emitter), `providers/anthropic.rs::count_tokens` (real impl) all covered. The `count_tokens` real-network path is covered via wiremock per the M02.C precedent (`providers/anthropic.rs` real-network construction stays in the existing exclusion list).
- New file `src-tauri/src/drone_lifecycle.rs`: unit tests via testable seam pattern (`DroneLifecycle::spawn_with(spawn_fn, ...)` taking a process-spawn closure for testability). Real OS-spawn wrapper excluded per the M02 `tauri-shell` exception in `codecov.yml`.
- New integration test `drone_reconnect_events.rs`: integration test (not subject to coverage gate; correctness is the assertion).

### A2.5 CLI Prompt

Paste the XML block below into a fresh Claude Code session as the opening message.

```xml
<work_stage_prompt id="M04.A2">
  <context>
    Stage A2 of M04 (Plan + Verify + HITL + Budget). Production wiring — drone subprocess lifecycle at Tauri startup with Arc<DroneClient> Tauri-managed-state; replaces DroneClient::noop() callsites in query_session_db + replay_session; wires vdr::project_signal at WriteSignal; replaces heuristic decision extractor with structured emitter; implements real count_tokens against /v1/messages/count_tokens; resolves long-lived events() reconnect carry-forward; refactors unwrapCmdError to consume generated CmdError types from src/types/error.ts (Stage A1 set up the generation; A2 wires it). Stage B does not start until Stage A2's commit is on the milestone branch claude/m04-plan-verify-hitl-budget.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage A1" subject</check>
    <check name="anthropic_key_set">Test-Path env:ANTHROPIC_API_KEY must succeed (count_tokens wiremock tests need a valid-looking key; live test optional)</check>
    <check name="generated_files_present">Test-Path crates/runtime-core/src/error.rs must succeed (Stage A1 deliverable)</check>
    <check name="generated_ts_present">Test-Path src/types/error.ts must succeed (Stage A1 deliverable)</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage A2 sections A2.1–A2.4)</file>
    <file>agent-runtime-spec.md §1c, §1d, §2b, §2c (especially §2c.3), §13.5</file>
    <file>docs/gotchas.md (especially #29 keyring; #30 unwrapCmdError; #31 tracing init; #32 cross-stack)</file>
    <file>docs/build-prompts/retrospectives/M04.A1-retrospective.md (apply [END] Decisions)</file>
  </read_first>

  <read_reference>
    <file purpose="M01.C drone subprocess spawn archetype + reconnect semantics">crates/runtime-drone/src/main.rs</file>
    <file purpose="Tauri command shell pattern + *_with seam archetype">src-tauri/src/commands.rs</file>
    <file purpose="existing DroneClient + reconnect logic to extend">crates/runtime-main/src/drone_ipc/connection.rs</file>
    <file purpose="existing Anthropic provider HTTP+SSE archetype to extend with count_tokens">crates/runtime-main/src/providers/anthropic.rs</file>
    <file purpose="existing wiremock harness pattern">crates/runtime-main/tests/anthropic_wiremock.rs</file>
    <file purpose="vdr projector module that needs wiring at WriteSignal">crates/runtime-main/src/vdr.rs</file>
    <file purpose="renderer-side error unwrap that needs to consume generated types">src/lib/ipc.ts</file>
    <file purpose="generated error types Stage A1 produced; Stage A2 imports">src/types/error.ts</file>
    <file purpose="generated error types Stage A1 produced; Stage A2 re-exports from commands.rs">crates/runtime-core/src/error.rs</file>
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
    <trap>vdr::project_signal failure should NOT fail the WriteSignal — signals are forensic, VDR is a projection; log the failure via tracing::warn! and continue per spec §2b separation of concerns</trap>
    <trap>The structured-emitter prompt template injection (in prompt_template.rs or equivalent) is the cross-stack glue point — verbatim per the format spec'd in M03.5 ⚠️ note (delimited block <<DECISION>>...<<END>>); do NOT change the delimiter format without updating the M03.5 spec text first via a follow-up doc PR</trap>
    <trap>count_tokens against the real endpoint — verify the exact response field name (input_tokens vs token_count vs other) against https://docs.anthropic.com/en/api/messages-count-tokens BEFORE authoring; do NOT assume the M03.5 spec text §2c.3 is verbatim correct (it's design-doc not API spec)</trap>
    <trap>Long-lived events() reconnect — the test outcome drives the spec edit. If the test reveals broken-as-implemented, do NOT silently fix without surfacing to the user — this is a v0.1 behavior decision and may warrant scoping to v1.0</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT regenerate framework/skill/agent/tool/common/event/error schemas — Stage A1 already did event + error; existing schemas were already done in M01–M03</warning>
    <warning>DO NOT call /v1/messages/count_tokens against the live API in tests — wiremock only. Live calls are reserved for the smoke test in src-tauri (which gates on ANTHROPIC_API_KEY presence)</warning>
    <warning>DO NOT push between stages — Stage A2 commits locally only. Push happens at end of Stage G per CLAUDE.md §8 + §20</warning>
    <warning>The drone subprocess spawn at Tauri setup is the highest-risk surface in M04 — if startup hangs or races with renderer mount, surface immediately rather than working around (e.g., hidden setTimeout in renderer); the user explicitly approved high-risk-first staging in the M04 plan</warning>
  </execution_warnings>

  <time_box estimate_hours="4.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage B: long-lived events() reconnect outcome (resolved or v0.1 deferred?); whether the structured-emitter delimiter format needed adjustment from the M03.5 spec text; whether count_tokens response field name matched the M03.5 spec §2c.3 wording or required spec follow-up; whether drone subprocess startup latency on cold-start affects renderer mount UX (Stage F may need a loading state); any cross-stack glue points the agent had to verbatim-quote from upstream rather than authoring (cite the upstream source in the retro).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="A2.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers including wiremock test count + drone_reconnect_events.rs outcome)</item>
    <item>schema drift check output — cargo xtask regenerate-types --check exit code (must be 0)</item>
    <item>fan_out_grep results — DroneClient::noop / CmdError:: / count_tokens callsite counts before vs after refactor (target: 0 noop callsites remaining; CmdError:: variant shapes consistent across crates)</item>
    <item>long-lived events() reconnect test outcome — pass (closed) or fail (v0.1 behavior documented + spec updated)</item>
    <item>spec §1d ⚠️ note disposition — closed or updated (cite line)</item>
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
lifecycle + closes four M02/M03 carry-forward 🟡 production-wiring
items. SQL inspector + replay-from-signals + decision projection are
now end-to-end functional.

Production wiring:
- src-tauri/src/lib.rs + drone_lifecycle.rs (new): drone subprocess
  spawned at Tauri setup hook via tokio::process::Command; Arc<DroneClient>
  registered as Tauri managed state; graceful shutdown on app exit.
  kill_on_drop(true) per docs/gotchas.md drone-subprocess discipline.
- src-tauri/src/commands.rs: query_session_db + replay_session take
  tauri::State<Arc<DroneClient>> and dispatch real drone IPC; CmdError
  enum becomes pub use runtime_core::error::CmdError (Stage A1 codegen).
- crates/runtime-main/src/sdk/event_pipeline.rs: vdr::project_signal
  called at WriteSignal site; projection failure logged but does not
  fail the signal write per spec §2b.
- crates/runtime-main/src/sdk/decision_extractor.rs: structured-emitter
  parser (regex on <<DECISION>>...<<END>> delimited blocks) replaces
  M02 line-by-line heuristic; prompt template updated.
- crates/runtime-main/src/providers/anthropic.rs: count_tokens calls
  POST /v1/messages/count_tokens (per spec §2c.3 added M03.5); chars/4
  approximation removed. wiremock-tested.
- crates/runtime-main/src/sdk/event_translation.rs +
  tests/drone_reconnect_events.rs (new): long-lived events() reconnect
  resolved [or documented as v0.1 behavior — see retro]. Spec §1d
  ⚠️ note [closed at this commit / updated to reflect v0.1 behavior].
- src/lib/ipc.ts: unwrapCmdError consumes generated CmdError type from
  src/types/error.ts; preserves gotcha #30 unwrap semantics.

Carry-forward closures:
- M03 🟡 Production drone subprocess wiring at Tauri startup
- M03 🟡 vdr.rs projector wired at signal-write call-site
- M02 🟡 Decision extractor → structured emitter migration
- M02 🟡 count_tokens → real /v1/messages/count_tokens endpoint
- M02 🟡 Long-lived events() subscription survives reconnect

Spec edits (conditional on test outcome):
- §1d ⚠️ note disposition

Refs: M04-plan-verify-hitl-budget.md §A2, gap-analysis.md M03 + M02
entries 🟡 (5 carry-forward items closed)
Retrospective: docs/build-prompts/retrospectives/M04.A2-retrospective.md

https://claude.ai/code
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE B — §3a Plan & Task primitive                            -->
<!-- ============================================================ -->

## Stage B — §3a Plan & Task primitive (schemas + types + events + state machine + persistence)

> **🔧 Audit corrections (post-M04.A2 audit, 2026-05-08).** Apply these to the X.2/X.3/X.4 sections below — the original prose in those sections predates the audit and contains drift.
> 1. **8 of 11 plan/task events ALREADY exist in `crates/runtime-core/src/event.rs` and `schemas/event.v1.json`** (audit-confirmed). Identify the 3 missing variants by diffing spec §3a's 11-event list against the existing `event.rs` enum + `event.v1.json` `oneOf`. Author only the missing 3; the existing 8 get state-machine wiring not re-authoring.
> 2. **Path correction:** `event_translation.rs` is a phantom — actual translation lives in `crates/runtime-main/src/sdk/event_pipeline.rs`. Read references and any X.3 instructions that name `event_translation.rs` should be read as `event_pipeline.rs`.
> 3. **Folds A2 deferrals** (per user-approved post-A2 sequencing): WriteSignal IPC command variant + drone-side handler + main-side emission path; structured-emitter prompt-template module + AgentSdk plumbing. Stage B has scope slack from already-shipped events to absorb these. New schema items: WriteSignal added to drone IPC enum; structured-emitter regex + prompt-template module new in `crates/runtime-main/src/sdk/`.
> 4. **`crates/runtime-drone/migrations/` directory does not exist yet** (audit-confirmed). The first migration file Stage B authors creates the directory; no existing-numbered-sequence to fit into.
> 5. **Plan state machine ≥95% safety primitive coverage** is unchanged. Stage B's authoring doesn't reduce the safety gate.
>
> All other X.1–X.6 content stays. Audit-driven scope reduction is real (8/11 events done) but offset by the A2 deferrals folding in. Net Stage B scope: similar size to original Phase doc, different content mix.

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.rs/typify/latest/typify/> — typify codegen for plan.v1.json + task.v1.json (extends Stage A1 pattern)
- <https://json-schema.org/draft/2020-12/schema> — JSON Schema 2020-12 spec; plan.v1.json + task.v1.json author against this draft (matches existing schemas)
- <https://docs.rs/rusqlite/latest/rusqlite/> — rusqlite for new `plans` + `tasks` table migrations; verify `journal_mode = WAL` + `foreign_keys = ON` pragma pattern unchanged from M01.C `db.rs`

### B.1 Problem Statement

§3a Plan & Task primitive is the single largest deliverable in M04. Spec §3a (with M03.5's DDL addition) locks the field shapes; Stage B builds the implementation end-to-end:

1. **Schemas** — author `schemas/plan.v1.json` + `schemas/task.v1.json` per spec §3a TypeScript shapes + M03.5 DDL. Extend `crates/xtask/src/main.rs` codegen list per the Stage A1 pattern. Generated targets: `crates/runtime-core/src/plan.rs` + `crates/runtime-core/src/task.rs` (Rust); `src/types/plan.ts` + `src/types/task.ts` (TS).

2. **Eleven new event variants** — `plan_created`, `plan_approval_requested`, `plan_approved`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_started`, `task_completed`, `task_failed`, `task_skipped`, `task_escalated` added to `schemas/event.v1.json`. Regen propagates to `event.rs` + `agent_event.ts`. Renderer `graphStore.applyEvent` exhaustive switch must handle all 11 (gotcha #36 _exhaustive: never forces it).

3. **Plan state machine** — `crates/runtime-main/src/plan/state_machine.rs` (new module) implements the FSM over Plan.status transitions per spec §3a. Safety primitive — ≥95% coverage gate per CLAUDE.md §5 (declare exclusions inline if any).

4. **fresh_context_per_task loop policy** — only loop policy lit in v0.1 per spec §0d + CLAUDE.md §3. Implementation: after each `task_completed`, the SDK clears the agent's message history and starts the next task with the full plan + completed-tasks summary in the system prompt.

5. **Failure escalation** — `failure_count++` on `task_failed`; if `>= max_failures` → emit `task_escalated` (routed to HITL in Stage E). Default `max_failures = 3` per spec §3a.

6. **SQLite persistence** — migrations land `plans` + `tasks` tables per the DDL added to spec §10 in M03.5. Drone-side migration runner (existing M01.C pattern) picks up the new migration files.

7. **Approval-gate primitive** — when `Plan.approval_required = true` and a `plan_created` fires, the runtime emits `plan_approval_requested` and SUSPENDS the plan until `plan_approved` (via HITL flow — Stage E wires this; Stage B exposes the suspend/resume seam).

**Success criterion:** unit tests cover plan state machine transitions exhaustively (hot path + every error transition); SDK can spawn a 3-task plan that emits `plan_created` → `plan_approval_requested` → (manual approval shim) → `plan_approved` → `task_started`/`task_completed` × 3 → `plan_complete`; SQLite contains the plan + task rows with correct status transitions; coverage gate met.

**New artifacts:**
- `schemas/plan.v1.json`, `schemas/task.v1.json` (new)
- `crates/runtime-core/src/plan.rs`, `crates/runtime-core/src/task.rs` (new; generated)
- `src/types/plan.ts`, `src/types/task.ts` (new; generated)
- `crates/runtime-main/src/plan/mod.rs`, `crates/runtime-main/src/plan/state_machine.rs` (new)
- `crates/runtime-drone/migrations/00X_plans_tasks.sql` (new; X = next available)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (extend codegen list with plan + task schemas)
- `schemas/event.v1.json` (add 11 plan/task event variants)
- `crates/runtime-core/src/event.rs` (regenerated)
- `src/types/agent_event.ts` (regenerated)
- `src/lib/graphStore.ts` or equivalent (extend `applyEvent` exhaustive switch for 11 new variants — even if rendering wiring lands in Stage C, the store must compile under `_exhaustive: never`)
- `crates/runtime-main/src/sdk/mod.rs` or equivalent (wire plan state machine into SDK event loop)
- `CHANGELOG.md` (`[Unreleased]` notes M04 Stage B Plan & Task primitive)

### B.2 Files to Change

| File | Change |
|---|---|
| `schemas/plan.v1.json` | **New** — JSON Schema 2020-12 for Plan per spec §3a + M03.5 DDL field shapes |
| `schemas/task.v1.json` | **New** — JSON Schema 2020-12 for Task per spec §3a + M03.5 DDL field shapes |
| `crates/xtask/src/main.rs` | **Edited** — add `plan` + `task` to codegen list (Rust + TS targets) |
| `schemas/event.v1.json` | **Edited** — add 11 plan/task event variants to the `oneOf` |
| `crates/runtime-core/src/{plan,task,event}.rs` + `lib.rs` | **Edited (regen)** — typify output for new + updated schemas; module exports |
| `src/types/{plan,task,agent_event}.ts` | **Edited (regen)** — json-schema-to-typescript output |
| `crates/runtime-main/src/plan/mod.rs` | **New** — module root + public API |
| `crates/runtime-main/src/plan/state_machine.rs` | **New** — Plan/Task FSM per spec §3a |
| `crates/runtime-main/src/sdk/mod.rs` (or where plan integration lands) | **Edited** — wire state machine into SDK event loop; failure-escalation logic |
| `crates/runtime-drone/migrations/00X_plans_tasks.sql` | **New** — migration for plans + tasks tables per M03.5 spec §10 DDL |
| `src/lib/graphStore.ts` | **Edited** — extend `applyEvent` exhaustive switch for 11 new variants |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes |

### B.3 Detailed Changes

#### `schemas/plan.v1.json` + `schemas/task.v1.json` — new schema files

Author each JSON Schema following the existing `schemas/*.v1.json` shape (`$schema`, `$id` per the established `https://schemas.aria-runtime.dev/<name>.v1.json` pattern per gotcha caught in M03.5.A retro, `title`, `description`, `properties`, `required`, `additionalProperties: false`). Field shapes match spec §3a TypeScript interfaces + M03.5 SQLite DDL:

- **Plan**: `id` (string, uuid), `session_id` (string, uuid), `title` (string), `description?` (string), `status` (enum: 6 values), `approval_required` (boolean), `loop_policy` (enum: 3 values; only `fresh_context_per_task` lit in v0.1 per scope locks), `hitl_checkpoints` (array of strings), `risks` (array of strings), `created_by?` (string), `created_at` (integer, unix ms), `approved_at?` (integer), `completed_at?` (integer).
- **Task**: `id` (string, uuid), `plan_id` (string, uuid), `title` (string), `status` (enum: 6 values), `hitl` (boolean), `hitl_reason?` (string), `failure_count` (integer, default 0), `max_failures` (integer, default 3), `files_affected?` (array of glob strings), `acceptance_criteria?` (array of strings), `created_at` (integer), `started_at?` (integer), `completed_at?` (integer), `estimated_minutes?` (integer), `actual_minutes?` (integer).

Pre-flight: `<schema_drift_check>` on Stage A1 + A2 outputs must be clean before authoring (verifies Stage A1 + A2's xtask state is durable).

#### `crates/xtask/src/main.rs` — codegen list extension

Add two entries to the existing codegen list (Stage A1 archetype):
- `("plan", "schemas/plan.v1.json")` → `crates/runtime-core/src/plan.rs` + `src/types/plan.ts`
- `("task", "schemas/task.v1.json")` → `crates/runtime-core/src/task.rs` + `src/types/task.ts`

Run `cargo xtask regenerate-types` to produce the generated files. Run `--check` to verify deterministic output.

#### `schemas/event.v1.json` — 11 new event variants

Locate the existing `oneOf` array of event variants. Append 11 new variants:

```json
{ "type": "object", "title": "plan_created", "properties": { "type": { "const": "plan_created" }, "plan_id": { "type": "string" }, "agent_id": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "agent_id", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "plan_approval_requested", "properties": { "type": { "const": "plan_approval_requested" }, "plan_id": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "timestamp"], "additionalProperties": false },
... (9 more, mirror per spec §3a event shapes)
```

Field shapes per spec §3a Events subsection (lines 1349–1359 per M03.5 reference). Run `cargo xtask regenerate-types` to propagate to `event.rs` + `agent_event.ts`.

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

#### `crates/runtime-drone/migrations/00X_plans_tasks.sql` — DDL migration

Author the SQL migration matching the M03.5 §10 spec DDL verbatim. The migration runner is the existing `crates/runtime-drone/src/db.rs::run_migrations` (M01.C). Verify migration version increment (`X` = next available).

#### `src/lib/graphStore.ts` — applyEvent exhaustive switch

The `applyEvent(state, event)` function uses TS `switch (event.type)` over `AgentEvent['type']`. Adding 11 new variants triggers a `_exhaustive: never` compile error if any case is missing. Stage B implements the case bodies as **pass-through to graph state** (no UI rendering yet — Stage C lights up the visual surface):

- `plan_created`: insert a `PlanNode` placeholder into the graph state (no edges yet)
- `plan_approval_requested`: mark the PlanNode as `awaiting_approval`
- `plan_approved`: mark the PlanNode as `approved`; render becomes active
- `task_started`: insert a `TaskNode` linked to the parent PlanNode
- ... etc per spec §3a graph integration

Stage C builds out the actual visual treatment + ApprovalPanel; Stage B's job is to ensure the store handles all 11 events without crashing or losing state.

### B.4 Tests

#### Pedantic-pass preflight

Apply per `docs/gotchas.md` #21 to the new modules: `plan/state_machine.rs`, `plan/mod.rs`, generated files exempt.

#### Test files

- `crates/runtime-main/src/plan/state_machine.rs` — unit tests for legal/illegal transitions; failure-escalation boundary; plan-status invariants
- `crates/runtime-main/tests/plan_lifecycle.rs` (new integration test) — full plan flow: orchestrator emits `plan_created` → approval requested → approved (manual shim) → 3 tasks executed → `plan_complete`; SQLite assertions after each phase
- `tests/unit/graphStore.test.ts` (extended) — applyEvent exhaustive coverage for all 11 new variants; state assertions

#### Coverage target

- `crates/runtime-main/src/plan/state_machine.rs` ≥95% (safety primitive per CLAUDE.md §5)
- `crates/runtime-main` ≥95% maintained
- workspace ≥80% maintained
- Generated files excluded via existing regex

### B.5 CLI Prompt

```xml
<work_stage_prompt id="M04.B">
  <context>
    Stage B of M04. §3a Plan & Task primitive — schemas + types + 11 new events + state machine + fresh_context_per_task loop policy + failure escalation + SQLite persistence + approval-gate seam. Largest deliverable in M04 by file count + LOC. Stage A2's commit must be on the milestone branch claude/m04-plan-verify-hitl-budget. Plan state machine is a NEW safety primitive subject to ≥95% coverage gate.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage A2" subject</check>
    <check name="a1_a2_artifacts_present">Test-Path crates/runtime-core/src/error.rs (A1) AND grep -q "DroneLifecycle" src-tauri/src/drone_lifecycle.rs (A2) must succeed</check>
    <check name="schemas_drift_clean">cargo xtask regenerate-types --check exit 0 (A1+A2 codegen state durable)</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage B sections B.1–B.4)</file>
    <file>agent-runtime-spec.md §3a (full section, especially Data types + Events + Approval-gate primitive + Loop policy primitive + Failure escalation + Graph integration + Framework JSON), §10 (plans/tasks DDL added M03.5)</file>
    <file>docs/MVP-v0.1.md §M4</file>
    <file>docs/gotchas.md (especially #14 snake_case discipline; #34 fmt-first; #36 synthetic-state inversion — Stage B starts the live-event path so the inversion no longer applies)</file>
    <file>docs/build-prompts/retrospectives/M04.A1-retrospective.md</file>
    <file>docs/build-prompts/retrospectives/M04.A2-retrospective.md (apply [END] Decisions)</file>
  </read_first>

  <read_reference>
    <file purpose="xtask codegen archetype Stage A1 established">crates/xtask/src/main.rs</file>
    <file purpose="existing schemas archetype to mirror for plan + task schema authoring">schemas/event.v1.json</file>
    <file purpose="existing FSM/state-machine archetype if any in runtime-main; otherwise spec §3a is the contract">crates/runtime-main/src</file>
    <file purpose="db.rs migration runner archetype">crates/runtime-drone/src/db.rs</file>
    <file purpose="graphStore applyEvent archetype Stage M03.B established">src/lib/graphStore.ts</file>
    <file purpose="renderer event-translation pipeline that propagates new event variants">crates/runtime-main/src/sdk/event_translation.rs</file>
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

  <runtime_environment os="windows" note="Build agent on Windows 11; Test-Path replaces test -f; named pipe paths differ from Unix sockets in any drone-IPC test"/>

  <gotchas>
    <trap>v0.1 hardcodes STANDARD mode + fresh_context_per_task — schemas declare 3 loop policies but only fresh_context_per_task is lit; the `one_shot` and `continuous` variants in the schema are spec-prep, not v0.1 implementation. Stage B's loop-policy seam returns NotImplemented for the other two.</trap>
    <trap>plan.v1.json $id MUST follow https://schemas.aria-runtime.dev/<name>.v1.json pattern (M03.5.A retro decision). Verify against existing schemas BEFORE authoring.</trap>
    <trap>11 new event variants in event.v1.json — the renderer's graphStore.applyEvent exhaustive switch will fail to compile if any case is missing. This is the forcing function (gotcha #36 _exhaustive: never); rely on the compiler to catch missing cases rather than running tests.</trap>
    <trap>Plan state machine is a SAFETY PRIMITIVE — coverage gate ≥95%. Document any exclusions inline (likely none — pure-logic module). Per CLAUDE.md §5 + M01.C precedent.</trap>
    <trap>Approval-gate seam (Stage B's deliverable) must be the channel/oneshot the SDK awaits on, NOT the HITL UI itself (Stage E). Stage B's SDK code calls `approval_seam.await_approval(plan_id).await?` and Stage E wires the seam to the HITL flow. Do NOT implement the HITL UI in Stage B.</trap>
    <trap>fresh_context_per_task implementation — clearing the agent's `messages` vec mid-session must NOT clear the SDK's plan-state. Plan state lives in the SDK + SQLite, NOT in the agent's message history.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT implement the orchestrator agent's prompt template — that's framework-JSON territory (loaded from examples/aria/framework.json at session start). Stage B provides the SDK machinery (state machine + event emission + loop policy + failure escalation); framework JSON wires the orchestrator.</warning>
    <warning>DO NOT wire the renderer's PlanNode/TaskNode to active visual treatment — Stage B's graphStore changes are pass-through state updates only. Stage C builds the visual surface + ApprovalPanel.</warning>
    <warning>DO NOT push between stages.</warning>
  </execution_warnings>

  <time_box estimate_hours="5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage C: which plan-state fields the renderer's PlanNode actually needs to render (likely subset of the full Plan struct — token-spend, status badge, task count); whether the approval-gate seam exposed in Stage B requires renderer-side state reflection (likely yes — the ApprovalPanel needs the plan + risks + hitl_checkpoints); whether _exhaustive: never caught all 11 new event variants in graphStore (forcing function discipline); plan state machine coverage % achieved + any holdouts.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="B.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail; plan state machine coverage % must be ≥95)</item>
    <item>schema drift check exit 0</item>
    <item>generated file shape preview — first 30 lines of crates/runtime-core/src/plan.rs + plan.ts</item>
    <item>plan_lifecycle.rs integration test outcome — full 3-task plan flow end-to-end</item>
    <item>retrospective with [END] decisions for Stage C</item>
    <item>draft commit message from B.6 (filled with session URL)</item>
    <item>"Stage M04.B is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### B.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime): M04 Stage B — §3a Plan & Task primitive (schemas + types + state machine + persistence)

Builds the §3a Plan & Task primitive end-to-end. Largest single
deliverable in M04 by file count + LOC. Plan state machine is a new
safety primitive at ≥95% coverage per CLAUDE.md §5.

New artifacts:
- schemas/plan.v1.json + schemas/task.v1.json (JSON Schema 2020-12;
  $id follows https://schemas.aria-runtime.dev/<name>.v1.json
  convention per M03.5.A retro decision)
- crates/runtime-core/src/plan.rs + task.rs (typify-generated)
- src/types/plan.ts + task.ts (json-schema-to-typescript-generated)
- crates/runtime-main/src/plan/state_machine.rs — Plan + Task FSM per
  spec §3a (legal transitions, illegal-transition errors, exhaustive
  transition matrix in unit tests; ≥95% coverage gate met)
- crates/runtime-main/src/plan/mod.rs
- crates/runtime-drone/migrations/00X_plans_tasks.sql — matches
  spec §10 DDL added in M03.5

Edits:
- crates/xtask/src/main.rs: codegen list extended with plan + task
- schemas/event.v1.json: 11 plan/task event variants added
  (plan_created, plan_approval_requested, plan_approved, plan_revised,
  plan_aborted, plan_complete, task_started, task_completed,
  task_failed, task_skipped, task_escalated)
- crates/runtime-core/src/event.rs + src/types/agent_event.ts:
  regenerated with 11 new variants
- crates/runtime-main/src/sdk/mod.rs: plan state machine wired into
  SDK event loop; failure-escalation logic (failure_count >=
  max_failures triggers task_escalated); fresh_context_per_task loop
  policy (clears agent messages between tasks; preserves SDK plan
  state in SQLite)
- src/lib/graphStore.ts: applyEvent exhaustive switch handles all 11
  new variants as pass-through state updates (Stage C builds visual
  treatment); _exhaustive: never forcing function held

Approval-gate seam exposed (channel/oneshot the SDK awaits on); Stage E
wires the seam to HITL UI.

v0.1 scope locks held: STANDARD mode hardcoded, fresh_context_per_task
only (one_shot + continuous return NotImplemented), Novice + Promoted
tiers only.

Refs: M04-plan-verify-hitl-budget.md §B, spec §3a, MVP §M4
Retrospective: docs/build-prompts/retrospectives/M04.B-retrospective.md

https://claude.ai/code
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE C — §3a Plan UI + ApprovalPanel + graph wiring           -->
<!-- ============================================================ -->

## Stage C — §3a Plan UI + ApprovalPanel + graph wiring (renderer surface for plan/task events)

> **🔧 Audit corrections (post-M04.A2 audit, 2026-05-08).**
> 1. **Add `<pre_flight_check>` for `Arc<DroneClient>` Tauri-managed-state** (Stage A2 wired it; Stages C/E/F all consume it). Verification: `Test-Path src-tauri/src/drone_lifecycle.rs` succeeds; `grep -q "manage(.*DroneClient" src-tauri/src/main.rs` returns a match.
> 2. **Verify PlanNode/TaskNode synthetic-state assumption still holds** before authoring — the audit scope didn't deeply read the components, only confirmed they exist. If Stage B's revised wiring emitted state shapes that differ from M03.C synthetic fixtures, Stage C's tests need to follow the new state shape.
> 3. Otherwise scope appears intact per audit.

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://reactflow.dev/api-reference/types/node> — confirm React Flow v12 Node + custom-node API for PlanNode/TaskNode visual upgrades unchanged from M03.B
- <https://v2.tauri.app/develop/calling-rust/> — confirm Tauri 2.x `invoke` API for the approval-flow round-trip (renderer → main → drone → main → renderer); used in `approve_plan` + `revise_plan` + `abort_plan` commands

### C.1 Problem Statement

Stage B exposed the approval-gate seam (channel/oneshot the SDK awaits on); Stage C builds the renderer surface that resolves it.

1. **Wire PlanNode + TaskNode to live event variants.** M03.B-C shipped the components with synthetic-state tests (gotcha #36); Stage B's graphStore now forwards 11 plan/task events through `applyEvent`. Stage C consumes those state updates: PlanNode shows status badge + task count + cumulative token spend; TaskNode shows status + HITL flag + failure_count when > 0. Per spec §3a Graph integration.

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
    <check name="plan_state_machine_present">Test-Path crates/runtime-main/src/plan/state_machine.rs must succeed</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage C sections C.1–C.4)</file>
    <file>agent-runtime-spec.md §3 (Graph Behavior + Visual Design Principles + InspectorPanel layout), §3a (Approval-gate primitive + Graph integration)</file>
    <file>docs/gotchas.md (especially #35 React Flow + happy-dom; #36 synthetic-state inversion now lifted; #37 trust TS narrowing)</file>
    <file>docs/build-prompts/retrospectives/M04.B-retrospective.md (apply [END] Decisions)</file>
  </read_first>

  <read_reference>
    <file purpose="M03.D InspectorPanel layout archetype + ARIA non-modal pattern + dismissal semantics">src/components/InspectorPanel.tsx</file>
    <file purpose="existing PlanNode synthetic-state component to drive live">src/components/nodes/PlanNode.tsx</file>
    <file purpose="existing TaskNode synthetic-state component to drive live">src/components/nodes/TaskNode.tsx</file>
    <file purpose="Tauri command archetype + error unwrap pattern">src-tauri/src/commands.rs</file>
    <file purpose="renderer-side typed invoke wrapper archetype">src/lib/ipc.ts</file>
    <file purpose="Playwright renderer-level archetype with module-mocked Tauri">tests/e2e/smoke.spec.ts</file>
    <file purpose="graphStore applyEvent extended in Stage B">src/lib/graphStore.ts</file>
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
    <trap>Stage B's graphStore handles 11 new event variants as pass-through state. Stage C consumes the state — does NOT re-implement event handling. If a new visual treatment needs a new state field, add it to Stage B's graphStore (via an amendment commit pre-Stage-C if needed) rather than computing it renderer-side.</trap>
    <trap>The 3 Tauri commands take user-typed strings (revisions + reason). Pass-through opaque per CLAUDE.md §8.security; do NOT parse/interpret the user input on the renderer side beyond length validation.</trap>
    <trap>Playwright test uses module-mocked Tauri (renderer-level), NOT tauri-driver (still disabled per Key constraints). Reuse the M02.E test setup pattern.</trap>
    <trap>Token-spend on PlanNode reuses M03.D tokenScale.ts — DO NOT re-implement the formula renderer-side; import the helper.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT touch the SDK approval seam (Stage B's deliverable) — Stage C only consumes its result via the event stream. The drone-side resolution of the seam happens via the 3 new IPC commands.</warning>
    <warning>DO NOT add new graph state fields without Stage B amendment — if Stage C needs them, surface and pause; the right fix is in Stage B's store.</warning>
    <warning>DO NOT push between stages.</warning>
  </execution_warnings>

  <time_box estimate_hours="4"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage D: any new gotchas about React Flow + non-modal panels (gotcha #35 act() warnings still apply); whether the approval round-trip latency is acceptable (renderer→Tauri→drone→SDK→drone→Tauri→renderer); any UI patterns for the multi-action panel (Approve/Revise/Cancel) that future panels (Verify rollback prompt, HITL panels) should mirror.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="C.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail; src/ coverage maintained)</item>
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

## Stage D — §4a Verify & Rails primitive (hooks + rails + don't-touch; consume existing RevertToSnapshot)

> **🔧 Audit corrections (post-M04.A2 audit, 2026-05-08).** Apply these to the X.2/X.3 sections below.
> 1. **Event names already exist as `verify_started/passed/failed` + `rail_triggered`** in `crates/runtime-core/src/event.rs` and `schemas/event.v1.json` (audit-confirmed). The original Phase doc planned new `hook_started/passed/failed` events — adopt the existing `verify_*` names instead. The Hook *primitive* (HookRef + HookCategory + Hook in `hook.v1.json`) keeps the "hook" terminology where appropriate (the framework-config primitive that fires the events); the *event variants* stay as `verify_*` per codebase convention.
> 2. **`RevertToSnapshot` already exists in `crates/runtime-core/src/drone.rs::DroneCommand`** with the correct `RevertReason` enum (`HookRollback`, `UserRollback`, `GapRecovery`) (audit-confirmed). Stage D consumes the existing variant — does NOT re-add it. Verify the `RevertReason` enum shape matches spec §4a + Stage D's needs; if any variant is missing, that's an additive edit, not a rebuild.
> 3. **`pre_file_edit` is the genuinely-new firing point** — verify spec §4a firing-point table currently lists 6 firing points; Stage D's `hook.v1.json` adds the 7th. Spec text edit either lands in-stage (if <5 lines) or as a follow-up doc PR per retro decision.
> 4. **Add `<pre_flight_check>` for `Arc<DroneClient>` Tauri-managed-state** (consumed via `RevertToSnapshot` IPC dispatch).
> 5. Hook executor (shell|tool|agent variants), Rails (hard/soft + JSONLogic), don't-touch glob matcher are all genuinely new per the audit.
>
> Net: Stage D scope is moderately smaller than original — events done, drone command done. Hook primitive + Rails + don't-touch + pre_file_edit firing point + cross-platform shell wrapper are still substantial new work.

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.rs/tokio/latest/tokio/process/struct.Command.html> — `tokio::process::Command` for shell hook execution (cross-platform); already in use for drone subprocess (Stage A2)
- <https://learn.microsoft.com/en-us/powershell/scripting/lang-spec/chapter-01> — PowerShell wrapper for Windows shell hooks per spec §M4 ("PowerShell wrapper" acceptance criterion); verify exact invocation pattern (`pwsh -NoProfile -Command "..."` vs `powershell.exe -NoProfile -Command`)
- <https://docs.rs/globset/latest/globset/> — globset for don't-touch glob matching; verify version pin or vendor decision
- <https://json-schema.org/draft/2020-12/schema> — hook.v1.json author against this draft

### D.1 Problem Statement

§4a Verify & Rails is the second-largest M04 deliverable. Spec §4a locks the primitives (Hook + HookRef + HookCategory + 7 firing points + Rails hard/soft + don't-touch + revert_to_snapshot drone command); Stage D builds them.

1. **Hook primitive** — `schemas/hook.v1.json` declares `HookRef = shell{command,timeout_ms?,cwd?} | tool{tool_name,input?} | agent{agent_id,prompt?}` and `HookCategory = verify | lint | build | test | custom`. Hook execution: shell variant spawns subprocess via `tokio::process::Command` with cross-platform PowerShell wrapper on Windows (`pwsh -NoProfile -Command "..."`); tool variant invokes the runtime tool dispatcher; agent variant spawns a child agent.

2. **Seven firing points** — existing 6 (`pre_task`, `post_task`, `post_file_edit`, `pre_commit`, `pre_agent_spawn`, `session_end`) + new `pre_file_edit` for don't-touch interception. Spec §4a's firing-point table (line 1689) lists 6; Stage D's hook.v1.json adds the 7th and a follow-up doc PR (post-M04) updates the spec text. Document this drift in retro decisions.

3. **Rails primitive** — `Rails { hard: Rail[], soft: Rail[] }` declared in framework JSON. Each `Rail` has `id`, `fires_on` (firing-point reference), `check` (JSONLogic expression), `message`. Hard rails block; soft rails warn. Emits `rail_triggered { rail_id, policy: 'hard' | 'soft', firing_point, message, agent_id? }`.

4. **Don't-touch primitive** — glob array in framework JSON; built-in pre-edit rail; fires on `pre_file_edit` firing point. Implementation: every `Write` tool call from any agent intercepts at the rail evaluator; if any glob matches, emit `rail_triggered { rail_id: 'dont_touch', policy: 'hard' }` and block the edit.

5. **revert_to_snapshot drone command** — new variant: `RevertToSnapshot { snapshot_id, reason: 'hook_rollback' | 'user_rollback' | 'gap_recovery' }`. Drone-side handler restores state from the named snapshot; emits `task_failed { error: 'rolled_back_after_hook_<hook_id>' }` for hook_rollback case.

6. **VerifyNode + HookNode wired** — already shipped as M03.C synthetic-state components; Stage D wires them to live `hook_started` / `hook_passed` / `hook_failed` / `rail_triggered` events.

**Success criterion:** Loading a fixture framework JSON with a `post_task` hook ("`bash .aria/verify.sh`" on Linux/macOS; `pwsh -NoProfile -Command ".aria\verify.ps1"` on Windows) running after each task; pass → next task; fail with `on_failure: rollback` → drone reverts to snapshot, task retries; rail violations on `pre_file_edit` fire `rail_triggered` events with don't-touch glob match; Playwright + integration tests cover the flows; ≥95% coverage on the new modules (rails + hooks are part of the capability enforcer surface; safety-primitive gate per CLAUDE.md §5).

**New artifacts:**
- `schemas/hook.v1.json` (new)
- `crates/runtime-core/src/hook.rs` (new; generated)
- `src/types/hook.ts` (new; generated)
- `crates/runtime-main/src/hooks/mod.rs`, `crates/runtime-main/src/hooks/executor.rs`, `crates/runtime-main/src/hooks/rails.rs`, `crates/runtime-main/src/hooks/dont_touch.rs` (new)
- `crates/runtime-main/src/hooks/shell.rs` (new — cross-platform shell wrapper)
- `crates/runtime-drone/src/snapshot.rs` (extended with `revert_to_snapshot` handler)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (codegen list: hook)
- `schemas/event.v1.json` (add `hook_started`, `hook_passed`, `hook_failed`, `rail_triggered`)
- `crates/runtime-core/src/event.rs` + `src/types/agent_event.ts` (regen)
- Drone IPC schema (extend with `RevertToSnapshot` variant + handler)
- `src/components/nodes/VerifyNode.tsx`, `src/components/nodes/HookNode.tsx` (live-event wiring)
- `src/lib/graphStore.ts` (extend exhaustive switch for 4 new variants)
- Spec §4a (add `pre_file_edit` to firing-point table — or note as M04 follow-up doc PR per retro decision)
- `CHANGELOG.md`

### D.2 Files to Change

| File | Change |
|---|---|
| `schemas/hook.v1.json` | **New** — Hook + HookRef + HookCategory + Rail per spec §4a |
| `crates/runtime-core/src/hook.rs`, `src/types/hook.ts` | **New** — generated |
| `crates/xtask/src/main.rs` | **Edited** — codegen list extension |
| `schemas/event.v1.json` | **Edited** — 4 new event variants |
| `crates/runtime-main/src/hooks/{mod,executor,rails,dont_touch,shell}.rs` | **New** — Hook/Rails/don't-touch implementation |
| `crates/runtime-drone/src/snapshot.rs` | **Edited** — add `revert_to_snapshot` handler |
| Drone IPC enum (location TBD per existing layout) | **Edited** — add `RevertToSnapshot` variant |
| `src/components/nodes/VerifyNode.tsx`, `HookNode.tsx` | **Edited** — live-event wiring |
| `src/lib/graphStore.ts` | **Edited** — exhaustive switch + 4 new variants |
| `agent-runtime-spec.md` §4a | **Edited (or follow-up note)** — add `pre_file_edit` to firing-point table; if deferred, document as follow-up PR target |
| `CHANGELOG.md` | **Edited** |

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

#### `crates/runtime-drone/src/snapshot.rs` — revert_to_snapshot handler

Extend with new IPC variant + handler:

```rust
pub enum DroneCommand {
    // ... existing variants
    RevertToSnapshot { snapshot_id: SnapshotId, reason: RevertReason },
}

pub enum RevertReason {
    HookRollback { hook_id: String },
    UserRollback,
    GapRecovery,
}
```

Handler restores state from the named snapshot (existing snapshot read path) + emits `task_failed { error: 'rolled_back_after_hook_<hook_id>' }` (for HookRollback case) or `task_started` (re-emit for retry).

#### Spec §4a follow-up (or in-stage)

Spec §4a's firing-point table at line 1689 lists 6 firing points; Stage D's hook.v1.json adds `pre_file_edit` as the 7th. Two options:
1. Land a small spec edit in Stage D (add `pre_file_edit` to the table)
2. Defer to a post-M04 doc PR (analogous to M03.5 pattern)

Decision per retro: option 1 if the spec edit is < 5 lines and self-contained; option 2 if it ripples to other §4a content.

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
    Stage D of M04. §4a Verify & Rails primitive. Hook (shell|tool|agent variants × 7 firing points) + Rails (hard/soft + JSONLogic) + don't-touch glob matcher (new pre_file_edit firing point) + revert_to_snapshot drone command. VerifyNode/HookNode wiring to live events. Cross-stack risk: shell hook execution + cross-platform PowerShell wrapper. Stage C's commit must be on the milestone branch.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage C" subject</check>
    <check name="pwsh_available">where.exe pwsh.exe must succeed (or fall back to powershell.exe with explicit retro decision)</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage D sections D.1–D.4)</file>
    <file>agent-runtime-spec.md §4a (full section)</file>
    <file>docs/MVP-v0.1.md §M4 (PowerShell wrapper acceptance criterion)</file>
    <file>docs/gotchas.md (especially #18 JSONLogic operator allowlist; #32 cross-stack discipline applies to shell.rs)</file>
    <file>docs/build-prompts/retrospectives/M04.C-retrospective.md</file>
  </read_first>

  <read_reference>
    <file purpose="cross-stack archetype: tokio::process::Command from Stage A2 drone subprocess spawn">src-tauri/src/drone_lifecycle.rs</file>
    <file purpose="snapshot module to extend with revert_to_snapshot">crates/runtime-drone/src/snapshot.rs</file>
    <file purpose="existing tool dispatcher where don't-touch interception lands">crates/runtime-main/src/sdk</file>
    <file purpose="VerifyNode/HookNode synthetic components from M03.C">src/components/nodes/VerifyNode.tsx</file>
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
    <grep pattern="firing_point" purpose="all firing-point references — enum + matchers + tests; new pre_file_edit needs all callsites updated"/>
    <grep pattern="DroneCommand::" purpose="all drone command variant constructions — RevertToSnapshot adds a new variant; exhaustive matches must add the case"/>
    <grep pattern="Write tool" purpose="locate the Write tool dispatch site where pre_file_edit interception inserts"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="globset" min_version="0.4"/>
    <dep name="tokio" required_features="process,time"/>
  </dependency_audit_check>

  <runtime_environment os="windows" note="PowerShell wrapper required per MVP §M4 acceptance criterion; pwsh.exe preferred over powershell.exe; verify availability at pre_flight_check"/>

  <gotchas>
    <trap>shell.rs cross-platform — gotcha #32 cross-stack discipline applies. Verify pwsh.exe -NoProfile -Command semantics against current Microsoft PowerShell docs URL (WEBCHECK) BEFORE authoring; do NOT assume bash -c semantics carry over.</trap>
    <trap>JSONLogic operator allowlist (gotcha #18) — Rails check field. Operators beyond the allowlist return UnsupportedOperator; do NOT silently extend the operator set.</trap>
    <trap>Don't-touch glob matcher fires on pre_file_edit — every Write tool call routes through it BEFORE the OS write. If the rail evaluator is async, ensure the Write call awaits the rail decision; otherwise edits land before the rail blocks.</trap>
    <trap>revert_to_snapshot is a NEW DroneCommand variant — fan_out_grep above catches exhaustive-match callsites. Verify all matches add the new arm.</trap>
    <trap>Spec §4a's firing-point table doesn't list pre_file_edit — Stage D adds it via hook.v1.json; spec text edit is either in-stage or a follow-up doc PR (decide per retro). Do NOT silently add pre_file_edit to the spec without a deliberate edit + commit message note.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT execute shell hooks against the user's actual filesystem in tests — use test fixtures + the *_with seam to inject mock spawners. Real shell execution is reserved for the integration test in a tempdir.</warning>
    <warning>DO NOT extend the JSONLogic operator allowlist — operators beyond gotcha #18's set need a deliberate spec edit + ADR (CLAUDE.md §11).</warning>
    <warning>DO NOT push between stages.</warning>
  </execution_warnings>

  <time_box estimate_hours="6.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage E: any cross-platform shell-execution surprises (gotcha #32 territory); whether the JSONLogic operator allowlist needed extension (if so, surface the operator + use case for ADR consideration); spec §4a firing-point table edit disposition (in-stage or follow-up PR); revert_to_snapshot integration with Stage F recovery flow (the same mechanism may apply to v0.1 startup recovery).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="D.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate; hooks/ coverage ≥95%)</item>
    <item>schema drift check exit 0</item>
    <item>fan_out_grep results — firing_point + DroneCommand:: + Write tool callsite counts</item>
    <item>integration test outcome — hook_integration.rs full lifecycle (post_task → fail → rollback → retry)</item>
    <item>cross-platform shell test outcome (Windows pwsh.exe + Linux bash via test fixtures)</item>
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
firing point) + revert_to_snapshot drone command. VerifyNode + HookNode
upgraded from M03.C synthetic to live-event-driven.

New artifacts:
- schemas/hook.v1.json (Hook + HookRef + HookCategory + Rail)
- crates/runtime-core/src/hook.rs + src/types/hook.ts (generated)
- crates/runtime-main/src/hooks/{mod,executor,rails,dont_touch,shell}.rs
- Drone command RevertToSnapshot { snapshot_id, reason: hook_rollback |
  user_rollback | gap_recovery }
- crates/runtime-main/tests/hook_integration.rs (full lifecycle)

Edits:
- schemas/event.v1.json: 4 new variants (hook_started, hook_passed,
  hook_failed, rail_triggered)
- crates/runtime-core/src/event.rs + src/types/agent_event.ts: regen
- crates/runtime-drone/src/snapshot.rs: revert_to_snapshot handler;
  emits task_failed for hook_rollback case
- src/components/nodes/VerifyNode.tsx + HookNode.tsx: live-event wiring
- src/lib/graphStore.ts: exhaustive switch handles 4 new variants
- agent-runtime-spec.md §4a: pre_file_edit added to firing-point table
  [in-stage / deferred to follow-up PR per retro decision]

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
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE E — §6a HITL primitive                                   -->
<!-- ============================================================ -->

## Stage E — §6a HITL primitive (9 triggers + 3 UI variants + 3 notifiers + plugin interface)

> **🔧 Audit corrections (post-M04.A2 audit, 2026-05-08).**
> 1. **Event name correction:** the codebase has `hitl_resolved` (not `hitl_response` as the original Phase doc claimed). Adopt `hitl_resolved` throughout Stage E. The 3 NEW HITL events stay as planned: `hitl_timeout` + `notifier_dispatched` + `notifier_failed`. The existing pair is `hitl_requested` + `hitl_resolved`.
> 2. **Add `<pre_flight_check>` for `Arc<DroneClient>` Tauri-managed-state** (consumed via `respond_hitl` IPC dispatch).
> 3. Tauri notification plugin cross-stack discipline (gotcha #32) unchanged — verify against https://v2.tauri.app/plugin/notification/ at authoring time per existing prompt body.
> 4. Otherwise scope intact per audit.

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

5. **Five HITL events** — existing 2 (`hitl_requested`, `hitl_response`) + 3 new (`hitl_timeout`, `notifier_dispatched`, `notifier_failed`). Added to `event.v1.json`; regen propagates.

6. **Failure-escalation flow** — `task_escalated` (Stage B) → `on_failure_threshold` HITL trigger evaluates → `hitl_requested` event → notifiers fire in parallel → 1h default timeout → on response: route to `task_started` (retry) / `task_skipped` / `plan_aborted` per user choice. Wire the SDK's HITL seam (Stage B exposed) to the HITL flow.

7. **Three renderer surfaces** — `HITLPanel`, `HITLModal`, `HITLToast`. Each consumes `hitl_requested` events and dispatches `respond_hitl(prompt_id, choice)` Tauri command. Reuse M03.D + M04.C non-modal patterns where applicable.

**Success criterion:** Loading a fixture with `on_failure_threshold` trigger → simulating 3 task failures → HITL Panel surfaces; user clicks Skip → `task_skipped` emits; plan continues. `desktop` notifier fires OS notification on Windows + Linux. Coverage gate met (capability-enforcer-adjacent ≥95%).

**New artifacts:**
- `schemas/hitl.v1.json` (new)
- `crates/runtime-core/src/hitl.rs`, `src/types/hitl.ts` (new; generated)
- `crates/runtime-main/src/hitl/{mod,policy,seam,notifiers}.rs` (new)
- `crates/runtime-main/src/hitl/notifiers/{terminal_bell,desktop,sound}.rs` (new)
- `src/components/HITLPanel.tsx`, `src/components/HITLModal.tsx`, `src/components/HITLToast.tsx` (new)
- `tests/e2e/hitl_failure_escalation.spec.ts` (new)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (codegen list: hitl)
- `schemas/event.v1.json` (3 new event variants)
- `crates/runtime-core/src/event.rs` + `src/types/agent_event.ts` (regen)
- `src/lib/graphStore.ts` (exhaustive switch +3 variants)
- `src-tauri/src/commands.rs` (`respond_hitl` Tauri command)
- `src-tauri/tauri.conf.json` (Tauri notification plugin permission per WEBCHECK URL)
- `package.json` (add `@tauri-apps/plugin-notification` per Tauri 2.x docs)
- `Cargo.toml` workspace + `src-tauri/Cargo.toml` (`tauri-plugin-notification`)
- `src/App.tsx` (mount HITL surfaces conditionally)
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
5. On response: emit `hitl_response` + route per `choice` to `task_started`/`task_skipped`/`plan_aborted`
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
    Stage E of M04. §6a HITL primitive. 9 trigger types + 3 UI variants (Panel/Modal/Toast) + notifier plugin interface + 3 built-in notifiers (terminal_bell/desktop/sound). Wires Stage B's HITL seam to the failure-escalation flow. Cross-stack risk: Tauri notification plugin (gotcha #32 applies). Stage D's commit must be on the milestone branch.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage D" subject</check>
    <check name="hooks_present">Test-Path crates/runtime-main/src/hooks/mod.rs must succeed</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage E sections E.1–E.4)</file>
    <file>agent-runtime-spec.md §6a (full section), §3a Failure escalation primitive (cross-ref into HITL)</file>
    <file>docs/MVP-v0.1.md §M4 (HITL escalation acceptance criterion)</file>
    <file>docs/gotchas.md (especially #32 cross-stack discipline; Tauri notification plugin is the textbook case)</file>
    <file>docs/build-prompts/retrospectives/M04.D-retrospective.md</file>
  </read_first>

  <read_reference>
    <file purpose="Stage B approval-gate seam archetype — HITL seam mirrors the pattern">crates/runtime-main/src/plan/state_machine.rs</file>
    <file purpose="Stage C ApprovalPanel non-modal pattern for HITLPanel">src/components/ApprovalPanel.tsx</file>
    <file purpose="existing Tauri command archetype with Arc<DroneClient> state">src-tauri/src/commands.rs</file>
    <file purpose="VerifyNode/HookNode wiring archetype from Stage D for HITLNode (already-shipped synthetic) live wiring">src/components/nodes/HITLNode.tsx</file>
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
    <grep pattern="task_escalated" purpose="all callsites of the failure-escalation event — Stage B emits, Stage E consumes via HITL trigger evaluation"/>
    <grep pattern="HitlTrigger::" purpose="all enum-variant constructions; 9 triggers must be exhaustively handled"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="tauri-plugin-notification" min_version="2.0"/>
  </dependency_audit_check>

  <runtime_environment os="windows" note="desktop notifier uses Windows Toast Notifications via Tauri plugin; verify pwsh-side permission state if first-run flow differs from Linux"/>

  <gotchas>
    <trap>Tauri notification plugin is the textbook gotcha #32 case — verify the install + capability + permission-prompt flow against https://v2.tauri.app/plugin/notification/ verbatim BEFORE authoring desktop.rs. The previous M04 cross-stack failures (M03 tauri-driver) cost three iteration cycles; do not repeat.</trap>
    <trap>Notifier failures are NON-FATAL — emit notifier_failed event and continue. Don't propagate notifier errors up; the HITL seam still resolves on user response or timeout regardless of which notifiers fired.</trap>
    <trap>HITL Panel/Modal/Toast — pick the right ARIA pattern per variant. Panel: aria-modal="false" (graph stays queryable). Modal: aria-modal="true" (blocks adjacent). Toast: role="status" or "alert" depending on urgency.</trap>
    <trap>v0.1 STANDARD mode hardcoded — mode-keyed HITL overrides in framework JSON are loaded but not evaluated. Implementation: load + validate + ignore non-STANDARD overrides; do NOT silently apply LITE/CONFIG defaults.</trap>
    <trap>9 trigger types — exhaustive matching required throughout SDK. Compiler enforces via Rust enum exhaustiveness; add WireMock-style tests if the dispatch logic uses string-keyed lookup that can drift.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT implement the M5 capability-violation HITL trigger — Stage E exposes the seam (on_capability_violation in the enum) but the trigger source is M5's deliverable. Mark Stage E's coverage of that trigger as "seam-only" in retro.</warning>
    <warning>DO NOT load external notifier plugins from notifiers/ dir — that's M9 generators territory. Stage E ships only the 3 built-ins; plugin loader returns NotImplemented for external types.</warning>
    <warning>DO NOT push between stages.</warning>
  </execution_warnings>

  <time_box estimate_hours="6.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage F: any cross-stack surprises with Tauri notification plugin (gotcha #32 territory); whether the desktop notifier permission flow needs first-run UX integration with M10; the 1h default HITL timeout — should it be per-trigger configurable in v0.1 or v1.0?; coverage gate disposition for notifiers/desktop.rs OS-call holdout.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="E.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate; hitl/ coverage ≥95%)</item>
    <item>schema drift check exit 0</item>
    <item>fan_out_grep results — task_escalated + HitlTrigger:: callsite counts</item>
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

Builds the §6a HITL primitive end-to-end. 9 trigger types + 3 UI variants
(Panel/Modal/Toast) + notifier plugin interface + 3 built-in notifiers
(terminal_bell, desktop via Tauri notification plugin, sound). Wires
Stage B's HITL seam to the failure-escalation flow.

New artifacts:
- schemas/hitl.v1.json (HitlPolicy + 9 HitlTrigger + 3 HitlUiVariant +
  HitlNotifier plugin shape)
- crates/runtime-core/src/hitl.rs + src/types/hitl.ts (generated)
- crates/runtime-main/src/hitl/{mod,policy,seam,notifiers}.rs
- crates/runtime-main/src/hitl/notifiers/{terminal_bell,desktop,sound}.rs
- src/components/HITLPanel.tsx + HITLModal.tsx + HITLToast.tsx
- crates/runtime-main/tests/hitl_failure_escalation.rs (integration)
- tests/e2e/hitl_failure_escalation.spec.ts (Playwright)

Edits:
- schemas/event.v1.json: 3 new variants (hitl_timeout, notifier_dispatched,
  notifier_failed). hitl_requested + hitl_response existing.
- crates/runtime-core/src/event.rs + src/types/agent_event.ts: regen
- src-tauri/src/commands.rs: respond_hitl Tauri command
- src-tauri/tauri.conf.json: notification plugin capability
- Cargo.toml workspace + src-tauri/Cargo.toml + package.json:
  tauri-plugin-notification + @tauri-apps/plugin-notification deps
- src/lib/graphStore.ts: exhaustive switch +3 variants
- src/App.tsx: conditional HITL surface mount

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
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE F — §2a Budget + §1b Recovery                            -->
<!-- ============================================================ -->

## Stage F — §2a Budget + §1b Recovery (cost controls + resume from snapshot)

> **🔧 Audit corrections (post-M04.A2 audit, 2026-05-08).**
> 1. **All 4 budget event variants ALREADY exist** in `crates/runtime-core/src/event.rs` + `schemas/event.v1.json` + `src/lib/graphStore.ts` (audit-confirmed: `budget_warning`, `budget_downshift`, `budget_suspended`, `budget_exceeded`). Stage F's original X.3 said "add 4 new variants" — that's wrong. Stage F wires the budget enforcer to EMIT the existing events, does NOT re-add them. The `<schema_drift_check>` should still pass (no new variants); regen verifies.
> 2. **Path correction:** `crates/runtime-main/src/vdr.rs` is a phantom (audit-confirmed). VDR lives in `crates/runtime-drone/src/vdr.rs`. `runtime-main` has NO rusqlite dependency — VDR access from main goes through drone IPC (likely via a new `QueryVdr { ... }` DroneCommand variant or by extending the existing `QuerySessionDb` if VDR queries are simple enough). Adjust X.2/X.3 paths and the IPC integration accordingly.
> 3. **Add `<pre_flight_check>` for `Arc<DroneClient>` Tauri-managed-state** (Budget queries + Recovery resume both go through drone IPC).
> 4. Recovery primitive (resume rebuilds history not re-execute, tool-call-uncertain UI prompt with 4-action options, MCP reconnect on resume, plan/capability state restoration) is genuinely new — original X.3 stays accurate for that half.
> 5. Budget enforcer logic (3 scopes + 4 threshold actions + downshift_hook + session header bar UI) is genuinely new — original X.3 stays accurate for that half. Token-cost computation uses Stage A2's real `count_tokens` endpoint with LRU per-message caching.
>
> Net: Stage F scope is moderately smaller — events done; everything else (enforcer + recovery + UI) genuinely new. Path correction critical (vdr in drone, not main).

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.anthropic.com/en/api/messages-count-tokens> — confirm `count_tokens` endpoint (Stage A2 wired) handles budget-enforcement query patterns; no rate-limit issues for per-message pre-flight checks
- <https://json-schema.org/draft/2020-12/schema> — budget.v1.json against this draft

### F.1 Problem Statement

Two primitives bundled in Stage F: §2a Budget (medium-sized; mostly Rust + UI header bar) + §1b Recovery (medium-sized; mostly Rust state-restoration logic). Both depend on Stage A2 production wiring + Stage E HITL flow.

1. **Budget primitive** — `schemas/budget.v1.json` declares `BudgetActions { warn_at_percent? (def 50), downshift_at_percent? (def 75), hitl_at_percent? (def 90), hard_stop_at_percent? (def 100) }`. Three scopes per spec §2a: per-session ($5 default), per-framework, per-day-global (user setting). Tightest cap wins; budget tracking via real `count_tokens` (Stage A2 wired).

2. **Four budget actions** — `warn` emits toast notification; `downshift` invokes the model-selector hook (default `opus → sonnet → haiku` ladder); `hitl` triggers `on_budget_threshold` HITL flow (Stage E wired); `hard_stop` triggers immediate agent kill via drone `stop_process` + emits `budget_exceeded`.

3. **Four budget events** — `budget_warning`, `budget_downshift`, `budget_suspended`, `budget_exceeded` added to `event.v1.json`. Regen propagates.

4. **Session header bar** — `src/components/BudgetHeaderBar.tsx` (new); shows current spend / cap with color gradient (green < 50%, amber 50–75%, red 75–90%, dark red > 90%). Per spec §2a Graph integration.

5. **Recovery — resume rebuilds history.** Per spec §1b: on session restore, the snapshot's `messages` + `tool_calls` + `tool_results` load into SDK history "as if they had already happened"; model generates next turn fresh; tools NOT re-invoked. Per spec §1b WI-14 lock.

6. **Recovery — tool-call uncertainty UI.** Detect `tool_invoked` without paired `tool_result` (signal pair invariant violation); mark VDR row `tool_call_uncertain: true`; UI prompt with `[r]etry/[s]kip/[m]ark complete/[a]bort` options; record as `tool_call_uncertainty_resolved` decision signal.

7. **Recovery — MCP reconnection.** On resume, attempt MCP server reconnect; on failure → emit `tool_missing` via gap flow (M5 wires gap flow; Stage F exposes the seam).

8. **Recovery — plan + capability state restoration.** Plan + task statuses from snapshot; running task reset to `pending` unless `task_completed` was in snapshot; loop policy resumes; capability `scope: 'session'` carries over, `scope: 'once'` cleared.

**Success criterion:** Loading a fixture with `budget.session_usd_cap = $1.00` + simulated text streaming → budget header bar transitions through color gradient as spend accumulates → at 50%/75%/90% the corresponding events fire → at 100% session hard-stops. Recovery: closing app mid-session + reopening → recovery dialog offers resume → resumed session continues from last snapshot with task pointer reset; tool-call-uncertain prompt surfaces if a signal pair was orphaned at crash time; user picks Skip → session continues without re-running the tool. Coverage gate met.

**New artifacts:**
- `schemas/budget.v1.json` (new)
- `crates/runtime-core/src/budget.rs`, `src/types/budget.ts` (new; generated)
- `crates/runtime-main/src/budget/{mod,enforcer}.rs` (new)
- `crates/runtime-main/src/recovery/{mod,resume,uncertainty}.rs` (new)
- `src/components/BudgetHeaderBar.tsx`, `src/components/RecoveryDialog.tsx`, `src/components/UncertaintyPrompt.tsx` (new)
- `tests/e2e/budget_threshold.spec.ts`, `tests/e2e/recovery_uncertainty.spec.ts` (new)

**Edited artifacts:**
- `crates/xtask/src/main.rs` (codegen list: budget)
- `schemas/event.v1.json` (4 new budget event variants)
- `crates/runtime-core/src/event.rs` + `src/types/agent_event.ts` (regen)
- `src/lib/graphStore.ts` (exhaustive switch +4 variants)
- `crates/runtime-drone/src/snapshot.rs` (extend with resume-rebuild path; reuses existing read API)
- `src-tauri/src/commands.rs` (`request_resume`, `respond_uncertainty`, `set_global_budget` Tauri commands)
- `src/App.tsx` (mount BudgetHeaderBar always; mount RecoveryDialog on cold-start with prior snapshot; mount UncertaintyPrompt on tool_call_uncertain)
- Spec §1b ⚠️ note disposition (final status — closed via Stage A2 outcome documented in M03.5)
- `CHANGELOG.md`

### F.2 Files to Change

| File | Change |
|---|---|
| `schemas/budget.v1.json` | **New** |
| `crates/runtime-core/src/budget.rs`, `src/types/budget.ts` | **New (generated)** |
| `crates/xtask/src/main.rs` | **Edited** — budget codegen |
| `schemas/event.v1.json` | **Edited** — 4 new variants |
| `crates/runtime-main/src/budget/{mod,enforcer}.rs` | **New** — budget enforcement loop |
| `crates/runtime-main/src/recovery/{mod,resume,uncertainty}.rs` | **New** — resume + uncertainty handler |
| `crates/runtime-drone/src/snapshot.rs` | **Edited** — resume-rebuild path (reuses read API) |
| `src/components/BudgetHeaderBar.tsx`, `RecoveryDialog.tsx`, `UncertaintyPrompt.tsx` | **New** |
| `src-tauri/src/commands.rs` | **Edited** — 3 new commands |
| `src/lib/graphStore.ts` | **Edited** — exhaustive switch |
| `src/App.tsx` | **Edited** — UI mounting |
| `tests/e2e/budget_threshold.spec.ts`, `recovery_uncertainty.spec.ts` | **New** |
| `CHANGELOG.md` | **Edited** |

### F.3 Detailed Changes

#### `schemas/budget.v1.json` — Budget primitive schema

Author per spec §2a. `BudgetActions` with 4 percent thresholds (defaults per spec); 3 scopes (per-session / per-framework / per-day-global); `downshift_hook` field referencing a tool ID by name.

#### `crates/runtime-main/src/budget/enforcer.rs` — Budget enforcement loop

Hooks into the SDK's signal-write path (Stage A2 wired vdr). After every signal that carries `tokens_in` + `tokens_out`:

1. Compute current spend (sum across session, lookup framework, lookup global per-day)
2. For each scope, check tightest cap; if any threshold crossed:
   - 50% → emit `budget_warning`
   - 75% → emit `budget_downshift` + invoke downshift_hook (model swap via tool dispatch)
   - 90% → emit `budget_suspended` + trigger `on_budget_threshold` HITL flow (Stage E wired)
   - 100% → emit `budget_exceeded` + drone `stop_process` (immediate kill)

Cost computation uses real `count_tokens` (Stage A2 endpoint) cached per-message with LRU per session.

#### `crates/runtime-main/src/recovery/resume.rs` — Resume from snapshot

Per spec §1b: load snapshot → reconstruct SDK message history → seed agent with prior state → model generates next turn fresh. Tool calls NOT re-invoked (per WI-14 lock).

Plan state restoration: load plan + tasks from SQLite; running task reset to `pending` unless `task_completed` in snapshot; loop policy resumes.

Capability state restoration: scope-session capabilities carry over; scope-once capabilities cleared.

#### `crates/runtime-main/src/recovery/uncertainty.rs` — Tool-call uncertainty handler

Detect: `tool_invoked` signal without paired `tool_result` at crash time. Mark VDR row `tool_call_uncertain: true`. Surface UI prompt:

```
Tool call "X" was in flight when the session was interrupted.
[r] retry the call
[s] skip — assume failed
[m] mark complete — assume succeeded with no result
[a] abort the session
```

User response → emit `tool_call_uncertainty_resolved` decision signal with the chosen action; route accordingly.

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
    Stage F of M04. §2a Budget + §1b Recovery. Budget primitive (3 scopes + 4 threshold actions + downshift_hook + 4 events + UI header bar) + Recovery primitive (resume rebuilds history not re-execute, tool-call-uncertain UI, MCP reconnect seam, plan + capability state restoration). Stage E's commit must be on the milestone branch.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage E" subject</check>
    <check name="hitl_seam_present">Test-Path crates/runtime-main/src/hitl/seam.rs must succeed (Stage E exposes the on_budget_threshold trigger)</check>
    <check name="count_tokens_real">grep -q "messages/count_tokens" crates/runtime-main/src/providers/anthropic.rs (Stage A2 wired the real endpoint; budget enforcement depends on it)</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage F sections F.1–F.4)</file>
    <file>agent-runtime-spec.md §2a (full section), §1b (Recovery Semantics — Resume rebuilds history; Tool calls in flight at crash time; MCP reconnection; Plan state restoration; Capability state)</file>
    <file>docs/MVP-v0.1.md §M4 (budget + recovery acceptance criteria)</file>
    <file>docs/gotchas.md (especially #15 Resume rebuilds history, doesn't re-execute)</file>
    <file>docs/build-prompts/retrospectives/M04.E-retrospective.md</file>
  </read_first>

  <read_reference>
    <file purpose="Stage A2 count_tokens implementation that budget enforcement queries">crates/runtime-main/src/providers/anthropic.rs</file>
    <file purpose="Stage E HITL seam that on_budget_threshold trigger drives">crates/runtime-main/src/hitl/seam.rs</file>
    <file purpose="snapshot read API to extend for resume path">crates/runtime-drone/src/snapshot.rs</file>
    <file purpose="VDR projection that uncertainty.rs uses to find orphaned signals">crates/runtime-main/src/vdr.rs</file>
    <file purpose="Stage C/E renderer surface mounting pattern for BudgetHeaderBar/RecoveryDialog/UncertaintyPrompt">src/App.tsx</file>
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

  <gotchas>
    <trap>Recovery rebuilds HISTORY, not EXECUTION — gotcha #15. Tool calls in the snapshot are loaded into SDK message history as if they already happened; the model generates the NEXT turn fresh. Do NOT re-invoke tools on resume.</trap>
    <trap>Tool-call uncertainty detection is paired-signal invariant — `tool_invoked` without `tool_result`. The 4 user actions (retry/skip/mark/abort) must each emit a distinct `tool_call_uncertainty_resolved` decision signal so the VDR projection has the audit trail.</trap>
    <trap>Budget tightest-cap-wins — if session cap=$5, framework cap=$3, day-global cap=$10, the framework cap wins. Implementation: compute (cap, scope) for all active scopes; min(cap) wins.</trap>
    <trap>Budget downshift_hook — invokes a runtime tool (the model-selector). The default ladder (opus → sonnet → haiku) is HARDCODED in the hook implementation OR configurable per framework JSON; pick the simpler v0.1 path and document choice in retro.</trap>
    <trap>budget_exceeded → drone stop_process is the EMERGENCY KILL path. After hard_stop, the session is unrecoverable — UI must surface a clear "session terminated due to budget" message, not just a silent stop.</trap>
    <trap>MCP reconnect on resume — if no MCP servers are configured (v0.1 STANDARD), this is a no-op. v0.1 doesn't ship MCP; the seam exists so M5/M6 can fill it without resume-flow refactor.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT call live Anthropic /v1/messages/count_tokens in tests — budget enforcement uses cached counts; tests use a fixture cache. Live calls reserved for the smoke test.</warning>
    <warning>DO NOT implement the model-selector tool itself — that's framework-JSON territory. Stage F provides the downshift_hook seam (invokes a tool by name); the tool implementation is in framework JSON or future M9 generators.</warning>
    <warning>DO NOT push between stages.</warning>
  </execution_warnings>

  <time_box estimate_hours="6"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage G (Phase Closeout): which M04 carry-forward items are fully closed vs need v1.0 escalation; whether the budget downshift_hook ladder configurability landed in framework JSON or stayed hardcoded; whether tool-call uncertainty UI surfaced any spec gaps in the 4-action semantics; final disposition of the §1d long-lived events() reconnect note (Stage A2 may have closed it; F validates).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="F.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate; budget/ + recovery/ coverage ≥95%)</item>
    <item>schema drift check exit 0</item>
    <item>integration test outcomes — budget_threshold.rs (50/75/90/100 events fire in order); recovery_lifecycle.rs (snapshot → resume; tool calls not re-invoked)</item>
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
threshold actions + downshift_hook + 4 events + UI header bar) closes
spec §2a. Recovery primitive (resume rebuilds history not re-execute
per WI-14, tool-call-uncertain UI with 4 actions, MCP reconnect seam,
plan + capability state restoration) closes spec §1b.

New artifacts:
- schemas/budget.v1.json
- crates/runtime-core/src/budget.rs + src/types/budget.ts (generated)
- crates/runtime-main/src/budget/{mod,enforcer}.rs
- crates/runtime-main/src/recovery/{mod,resume,uncertainty}.rs
- src/components/BudgetHeaderBar.tsx + RecoveryDialog.tsx +
  UncertaintyPrompt.tsx
- crates/runtime-main/tests/budget_threshold.rs +
  recovery_lifecycle.rs (integration)
- tests/e2e/budget_threshold.spec.ts +
  recovery_uncertainty.spec.ts (Playwright)

Edits:
- schemas/event.v1.json: 4 new variants (budget_warning,
  budget_downshift, budget_suspended, budget_exceeded)
- src-tauri/src/commands.rs: request_resume, respond_uncertainty,
  set_global_budget
- src/lib/graphStore.ts: exhaustive switch +4 variants
- src/App.tsx: mount BudgetHeaderBar always; conditional
  RecoveryDialog (cold-start with prior snapshot) +
  UncertaintyPrompt (tool_call_uncertain)

Budget enforcement uses Stage A2's real count_tokens endpoint;
threshold crossings emit events + invoke downshift_hook (default
opus→sonnet→haiku ladder) at 75% / route to Stage E HITL flow at 90% /
hard-stop at 100%.

Recovery rebuilds SDK message history from snapshot; tools NOT
re-invoked per gotcha #15. Tool-call uncertainty detection via
paired-signal invariant (tool_invoked without tool_result) surfaces
4-action prompt; user choice emits tool_call_uncertainty_resolved
decision signal.

Coverage: budget/ + recovery/ ≥95% (safety primitive per CLAUDE.md §5).

Refs: M04-plan-verify-hitl-budget.md §F, spec §2a + §1b, MVP §M4
Retrospective: docs/build-prompts/retrospectives/M04.F-retrospective.md

https://claude.ai/code
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
    <check name="all_work_stages_committed">git log --oneline main..HEAD must show 7 commits (Stages A1, A2, B, C, D, E, F)</check>
    <check name="all_retros_committed">Test-Path docs/build-prompts/retrospectives/M04.A1-retrospective.md through M04.F-retrospective.md (7 files)</check>
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
| **A1** Build hygiene | ✅ DONE | 2 (`error.rs`, `error.ts` generated under `generated/` submodule) | 6 (xtask, regen, client.rs test, CHANGELOG) | 1 unit (`await_event` timeout) | ~1h actual |
| **A2** Production wiring | ✅ DONE | 4 (`drone_lifecycle.rs`, `drone_reconnect_events.rs`, `cmd_error_ext.rs`, retro) | 12 (Tauri shell, sdk modules, anthropic, ipc.ts, spec, key_store) | 4 wiremock + 2 reconnect integration + 13 cmd_error + 9 ipc.ts + unit | ~3h actual |
| **B** §3a Plan/Task primitive (revised: folds A2 deferrals) | ⏳ NEXT | 7+ (plan/task schemas + generated + state machine + first migration creates `migrations/` dir + structured-emitter prompt-template module) | ~8 (xtask, event regen for 3 missing variants, sdk wiring for 8 already-shipped events + new WriteSignal IPC, graphStore) | exhaustive FSM + integration plan_lifecycle.rs + structured-emitter regex tests | ~5–7h |
| **C** Plan UI + ApprovalPanel | ⏳ | 4 (ApprovalPanel + 3 test files) | ~6 (PlanNode/TaskNode, ipc.ts, commands, App.tsx) | Vitest + Playwright | ~3–5h |
| **D** §4a Verify & Rails (revised: events + drone command done) | ⏳ | 6 (hook schema + generated + 4 hook modules + integration test) | ~7 (xtask, VerifyNode/HookNode, graphStore, spec §4a `pre_file_edit` row) | exhaustive rails + hook integration | ~5–7h |
| **E** §6a HITL (revised: hitl_resolved adopted) | ⏳ | 12 (hitl schema + generated + 4 hitl modules + 3 notifiers + 3 panels + 2 e2e) | ~9 (xtask, event regen for 3 new HITL variants, deps, commands, App.tsx, graphStore) | unit + integration + Playwright | ~5–7h |
| **F** §2a Budget + §1b Recovery (revised: events done, vdr-via-drone-IPC) | ⏳ | 9 (budget schema + generated + 4 modules + 3 panels + 2 e2e) | ~7 (xtask, drone IPC for VDR access, commands, App.tsx, graphStore) | unit + integration + Playwright | ~4–6h |
| **G** Phase Closeout | ⏳ | 1 (M04-summary.md) | 2 (gap-analysis.md, CHANGELOG.md) | None (doc-only) | ~2–3h |
| **Total (revised)** | A1+A2 done | ~39 new files (down from ~49 — events-already-done removes some generated targets) | ~46 edited (down from ~54) | 30+ tests across unit/integration/Playwright | ~25–35h estimated; ~8–12h actual at M03 0.32× ratio + 20% buffer (down from 32–45h / 12–17h after audit) |

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
  - [ ] Budget threshold breach → `budget_warning` toast at 50%, `budget_downshift` at 75%, `budget_suspended` HITL approval at 90%
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

- [ ] **Hard Gate G1: do-not-commit-until-approved held** — every stage commit happened only after explicit user approval (7 approval gates across Stages A1, A2, B, C, D, E, F, G; revised post-A2 — original plan was 8 with a separate A3 stage that was folded into B per audit)
- [ ] User has reviewed each stage retrospective; scoring matches observable evidence
- [ ] M04-summary verdict is "Pattern held" (sound) or "Pattern held with friction"; not "Pattern strained"
- [ ] Three-artifact review per CLAUDE.md §20 complete: code diff + retrospectives/summary + gap-analysis entry all reviewed together
- [ ] PR creation deferred to explicit user instruction (do NOT auto-open per established convention)

---

*End of M04 specification + stage prompts. Eight stages on one parent-milestone branch (`claude/m04-plan-verify-hitl-budget`); Stage G is Phase Closeout per CLAUDE.md §20. PR drafts at end of Stage G and pushes after explicit approval. M05 (gap detection + capability enforcement) follows on a separate branch once this milestone merges.*
