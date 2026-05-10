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

Total revised estimate: ~25–35 hours estimated for B–G (A1+A2 actuals: ~4h). ~10–12 hours human direction (eight approval gates across A1–G + one PR review).

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

## Stage B — §3a Plan & Task primitive (schemas + events + FSM + projection-based persistence + WriteSignal IPC + structured emitter)

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens.

- <https://docs.rs/typify/latest/typify/> — typify codegen for plan.v1.json + task.v1.json (extends Stage A1 pattern)
- <https://json-schema.org/draft/2020-12/schema> — JSON Schema 2020-12 spec; plan.v1.json + task.v1.json author against this draft (matches existing schemas)
- <https://docs.rs/rusqlite/latest/rusqlite/> — rusqlite for the migration-runner architecture + new `plans` + `tasks` tables; verify `journal_mode = WAL` + `foreign_keys = ON` pragma pattern unchanged from M01.C `db.rs`
- <https://docs.rs/tokio/latest/tokio/sync/oneshot/index.html> — oneshot channel for the approval-gate seam (Stage E wires the seam to the HITL UI)

### B.1 Problem Statement

§3a Plan & Task primitive is M04's largest deliverable. Stage B builds the implementation end-to-end against spec §3a (data types + 11 events at lines 1417–1427 + approval-gate primitive + loop policy + failure escalation + graph integration + framework JSON) and spec §10 (plans + tasks SQLite DDL). Two M02/M03 carry-forward items fold in: WriteSignal IPC + structured-emitter migration. The phase doc framing in PR #56 ("8 of 11 plan/task events ALREADY exist; author only the 3 missing") was wrong; the real diff is **6 spec-canonical with shape mismatches + 2 codebase extras + 5 missing = 11 changes** (verified at authoring time against spec §3a lines 1417–1427 vs `crates/runtime-core/src/event.rs` + `schemas/event.v1.json`).

**Twelve work areas, all in scope:**

1. **Event-shape reconciliation against spec §3a (`schemas/event.v1.json` + downstream).** Schema is source-of-truth per CLAUDE.md §14. Per-variant decisions:
   - **6 spec-canonical with shape mismatches** → migrate to spec shape: `plan_created` (add `title` + `approval_required`); `plan_approved` (add `approved_by: 'user'|'auto'`); `task_escalated` (replace `reason` with `failure_count` + `max_failures`); `task_started`/`task_completed`/`task_failed` (no shape change beyond keeping the `plan_id` denormalization — see drift carve-out below).
   - **2 codebase extras** → drop `plan_rejected` (spec §1439 unifies cancel under `plan_aborted`); **keep `task_rolled_back`** as typed event (drift carve-out below).
   - **5 missing** → author: `plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped`.

2. **Spec drift carve-outs (two — both flagged for closeout `docs(spec):` PR per M01/M02/M03 precedent).** Stage B locks the engineering call:
   - **`task_rolled_back` typed event with `snapshot_id` field** — kept over spec §4a's stringly-typed `error: 'rolled_back_after_hook_<id>'` pattern. Spec text encodes structured info in error strings, which is the CLAUDE.md §9 "stringly-typed APIs" anti-pattern. Typed event + structured `snapshot_id` is sounder engineering. Stage D verify+rails work consumes the typed event. Spec drift target: §4a "After rollback, runtime emits `task_rolled_back { task_id, snapshot_id }`" (replaces stringly-typed `task_failed`).
   - **`task_*` events keep extra `plan_id` denormalization** — over spec §3a's lean `task_id` + `agent_id` shape. Denormalization makes events self-contained for downstream consumers (renderer, projector, replay) — no separate plan_id lookup or per-event index needed; same trade-off the codebase made for `tool_result.tool_invocation_id`-style cross-references in M02. Spec drift target: §3a event shapes "all task_* events include `plan_id` for projection self-containment."
   - Both drifts surface as ⚠️ adherence flags in M04 gap-analysis at closeout; the bundled post-M04 `docs(spec):` PR closes them.

3. **Plan + Task schemas (`schemas/plan.v1.json` + `schemas/task.v1.json`, new).** JSON Schema 2020-12 per spec §3a TS shapes + spec §10 DDL field shapes. `$id` follows the established `https://schemas.aria-runtime.dev/<name>.v1.json` pattern (M03.5.A retro decision; verify via `<fan_out_grep>` against existing schemas before writing the value). Validated string fields (`Plan.title` `minLength: 1`; `Task.title` `minLength: 1`) extracted to `$defs/<Name>` per A1 typify-friendliness gotcha (typify 0.6.2 panics on inline-validated strings).

4. **xtask codegen extension.** Add `plan` + `task` to the codegen list in `crates/xtask/src/main.rs` (Stage A1 archetype). Generated targets: `crates/runtime-core/src/generated/{plan,task}.rs` (typify) + `src/types/{plan,task}.ts` (json-schema-to-typescript). Top-level hand-curated `crates/runtime-core/src/event.rs` continues to be the primary `runtime_core::AgentEvent` type for callsites; Stage B hand-updates it to mirror schema shape after schema edits land. (Partial close of M03 "event.rs hand-maintained" carry-forward — A1 extended typify list; Stage B reconciles shape.)

5. **Migration runner architecture (new — closes implicit M01 gap).** No `migrations/` directory exists in `crates/runtime-drone/`; `db.rs::init_schema` uses `CREATE TABLE IF NOT EXISTS` with no version tracking. Stage B authors a versioned migration runner: `db.rs::run_migrations(conn)` reads `migrations/NNN_<name>.sql` files in lexical order, tracks applied versions in a `_migrations` table (`version INTEGER PRIMARY KEY`, `name TEXT NOT NULL`, `applied_at INTEGER NOT NULL`), applies each idempotently. `init_schema` becomes a wrapper that calls `run_migrations`; existing `CREATE TABLE IF NOT EXISTS` content moves into `migrations/000_initial.sql` to preserve the M01 baseline. First new migration (`migrations/001_plans_tasks.sql`) adds `plans` + `tasks` tables per spec §10 DDL. The phase doc's prior reference to "the existing M01.C migration runner pattern" was incorrect — the architecture is authored from scratch in Stage B.

6. **Plan + Task FSM (`crates/runtime-main/src/plan/state_machine.rs`, new — safety primitive).** Pure-logic module enforcing legal transitions per spec §3a:
   - **Plan**: `pending_approval → approved | aborted`; `approved → in_progress`; `in_progress → complete | aborted | awaiting_replan`; `awaiting_replan → in_progress | aborted`.
   - **Task**: `pending → running`; `running → done | failed | blocked | skipped`; `failed → pending` (retry within `max_failures`) `| escalated` (≥ `max_failures`); `blocked → pending` (after gap resolution); `skipped` / `done` / `escalated` are terminal.
   - Module exposes `PlanStateMachine::transition(plan, event) → Result<(), TransitionError>` and `TaskStateMachine::transition(task, event) → Result<(), TransitionError>`. Errors: `IllegalTransition { from, to }`, `UnknownEvent`, `MissingPrecondition { reason }`. No I/O, no async. ≥95% coverage gate per CLAUDE.md §5 (safety primitive).

7. **`WriteSignal` IPC variant + drone-side handler (closes M03 🟡 carry-forward).** New `runtime_core::DroneCommand::WriteSignal { signal_id, kind, source_id, context_type, payload }` variant per spec §2b signal shape. `crates/runtime-drone/src/command_handler.rs` handler arm: insert into `signals` table → call `vdr::project_signal(conn, signal_id)` (existing M03 projector) → call `plan_projector::project_signal(conn, signal_id)` (new, item 8) when the signal corresponds to a plan/task event. `crates/runtime-main/src/drone_ipc/client.rs` exposes `DroneClient::write_signal(...)` for SDK callers. Closes M03 gap-analysis 🟡 "vdr.rs projector wired at signal-write call-site."

8. **Plan/Task projector (`crates/runtime-drone/src/plan_projector.rs`, new — safety primitive — parallel to `vdr.rs`).** Drone-internal continuous projector that reads plan/task events from the `signals` table → UPSERTs `plans` + `tasks` rows. Idempotent via UNIQUE INDEX on `(event_type, plan_id|task_id)` + last-write-wins on status fields. Same architectural pattern as `vdr.rs::project_signal` (M03.E archetype). ≥95% coverage gate per CLAUDE.md §5. The projection is the read-model for renderer queries (Stage C SQL inspector) and recovery (item 11).

9. **SDK plan integration (`crates/runtime-main/src/sdk/agent_sdk.rs` + new `crates/runtime-main/src/sdk/plan_loop.rs`).** Drives the plan FSM from the agent loop:
   - On `plan_created` (parsed from structured-emitter output, item 10): if `approval_required`, emit `plan_approval_requested`, suspend on the approval-gate seam (item 12); on user approval, emit `plan_approved` and advance.
   - On `task_completed` (or any task-terminal): advance plan state machine; emit next `task_started` per task list, OR emit `plan_complete` when all done.
   - **`fresh_context_per_task` loop policy (only loop policy lit in v0.1 per spec §0d + CLAUDE.md §3 scope locks):** between tasks, clear the agent's `messages` vec; seed the next task with `system_prompt + plan_summary + completed_tasks_summary + current_task`. Plan state lives in SDK + projection (NOT in agent message history) — clearing messages does not lose plan state. The other two policies (`one_shot`, `continuous`) are enum variants only; the loop-policy seam returns `LoopPolicyError::NotImplemented` for both.
   - **Failure escalation per spec §3a:** on `task_failed`, increment `failure_count`; if `>= max_failures` (default 3), emit `task_escalated` (Stage E routes to HITL).

10. **Structured-emitter migration (closes M02 🟡 carry-forward).** New `crates/runtime-main/src/sdk/structured_emitter.rs` replaces `crates/runtime-main/src/sdk/decision_extractor.rs`'s line-by-line heuristic. Mechanism: prompt-template injects `<<DECISION>>...<<END>>` and `<<PLAN>>...<<END>>` delimited blocks; parser consumes blocks deterministically → emits `DecisionRecord` and `PlanCreated` events. Eliminates M02's false-positive concern (`Decision:` matched in code blocks / quoted content). The `decision_extractor` module is deleted; `event_pipeline.rs` callsite switches to the structured emitter.

11. **Snapshot integration + recovery semantics (per spec §1b literal — projection-aware).** Snapshot blob extends to include the projected plan/task state alongside the existing event log. `crates/runtime-drone/src/snapshot.rs::write` captures both. Recovery:
    - Load snapshot → restore plan/task projection in-memory + DB.
    - Replay any post-snapshot events from `signals` table.
    - Currently-running task (per spec §1b): set to `pending` unless its `task_completed` event is in the snapshot or replay window.
    - Tool-call uncertainty (per spec §1b): detect `tool_invoked` without matching `tool_result` → mark `tool_call_uncertain` flag for the renderer's UI prompt.

12. **Approval-gate seam (`crates/runtime-main/src/sdk/approval.rs`, new — Stage B exposes; Stage E wires the HITL UI).** `tokio::sync::oneshot` channel pattern: `ApprovalSeam` exposes `await_approval(plan_id) -> Result<ApprovalDecision, ApprovalError>` to the SDK; Stage E's HITL flow drives the seam from the renderer side. Stage B does NOT implement the HITL UI — only the seam the SDK awaits on. `ApprovalDecision` variants: `Approved`, `Revised(plan_id)` (revised plan ID — emits `plan_revised`), `Aborted` (emits `plan_aborted`).

**Renderer (`src/lib/graphStore.ts`):** the `applyEvent` exhaustive switch over `AgentEvent` variants gets 5 new cases + 6 changed cases + 2 dropped cases. `_exhaustive: never` is the forcing function (gotcha #36); the compiler catches missing cases. Stage B implements all cases as **pass-through state mutations** (no visual rendering — Stage C lights up the visual surface for plan/task events + ApprovalPanel).

**Success criterion:** schema event reconciliation lands cleanly (`cargo xtask regenerate-types --check` exits 0); plan + task schemas regenerate to deterministic Rust + TS; migration runner applies `001_plans_tasks.sql` idempotently (re-run is no-op via `_migrations` version tracking); FSM unit tests cover the exhaustive transition matrix at ≥95% line coverage; integration test (`plan_lifecycle.rs`) drives a 3-task plan end-to-end through `plan_created → plan_approval_requested → plan_approved (manual shim) → task_started/completed × 3 → plan_complete` with WriteSignal roundtrips and projection assertions; recovery integration test (`plan_recovery.rs`) kills the drone subprocess mid-plan, restarts, and verifies the projection rebuilds + currently-running-task semantics hold per spec §1b; structured-emitter unit tests cover the parser surface (delimited blocks; nested; malformed); graphStore exhaustive switch compiles after the 11-variant change.

**New artifacts:**
- `schemas/plan.v1.json`, `schemas/task.v1.json`
- `crates/runtime-core/src/generated/{plan,task}.rs` (regenerated)
- `src/types/{plan,task}.ts` (regenerated)
- `crates/runtime-main/src/plan/{mod,state_machine}.rs`
- `crates/runtime-main/src/sdk/{plan_loop,structured_emitter,approval}.rs`
- `crates/runtime-drone/src/plan_projector.rs`
- `crates/runtime-drone/migrations/` (new directory) + `migrations/000_initial.sql` + `migrations/001_plans_tasks.sql`
- `crates/runtime-main/tests/plan_lifecycle.rs`, `crates/runtime-main/tests/plan_recovery.rs`
- `crates/runtime-drone/tests/plan_projector.rs`, `crates/runtime-drone/tests/migration_runner.rs`

**Edited artifacts:**
- `schemas/event.v1.json` (13 oneOf changes: 6 migrate + 2 delete + 5 add)
- `crates/runtime-core/src/event.rs` (hand-curated wrapper mirrors schema)
- `crates/runtime-core/src/lib.rs` (re-exports `plan` + `task`)
- `crates/runtime-core/src/drone.rs` (`DroneCommand::WriteSignal` variant)
- `crates/runtime-core/src/generated/event.rs` (regenerated)
- `src/types/agent_event.ts` (regenerated)
- `crates/xtask/src/main.rs` (codegen list extension: `plan` + `task`)
- `crates/runtime-drone/src/db.rs` (migration runner; `init_schema` wraps `run_migrations`)
- `crates/runtime-drone/src/command_handler.rs` (`WriteSignal` arm; calls `vdr::project_signal` + `plan_projector::project_signal`)
- `crates/runtime-drone/src/lib.rs` (export `plan_projector`)
- `crates/runtime-drone/src/snapshot.rs` (projection-aware snapshot blob)
- `crates/runtime-main/src/sdk/agent_sdk.rs` (plan-loop integration; failure escalation; fresh_context wiring)
- `crates/runtime-main/src/sdk/event_pipeline.rs` (consume structured emitter; drop `decision_extractor` callsite)
- `crates/runtime-main/src/sdk/decision_extractor.rs` (deleted; replaced by `structured_emitter.rs`)
- `crates/runtime-main/src/drone_ipc/client.rs` (`DroneClient::write_signal` method)
- `src/lib/graphStore.ts` (applyEvent exhaustive switch — 5 new + 6 changed + 2 dropped variants; pass-through state)
- `CHANGELOG.md` (`[Unreleased]` notes M04 Stage B)

### B.2 Files to Change

| File | Change |
|---|---|
| `schemas/event.v1.json` | **Edited** — 13 `oneOf` changes: migrate 6 spec-canonical variants to spec shape; delete 2 codebase extras (`plan_rejected`); keep 1 codebase extra (`task_rolled_back`, drift carve-out); add 5 missing variants per spec §3a lines 1417–1427 |
| `schemas/plan.v1.json` | **New** — JSON Schema 2020-12 for Plan per spec §3a TS shape + spec §10 DDL; `$id` = `https://schemas.aria-runtime.dev/plan.v1.json`; validated strings in `$defs/<Name>` |
| `schemas/task.v1.json` | **New** — JSON Schema 2020-12 for Task per spec §3a TS shape + spec §10 DDL; same `$id` + `$defs` conventions |
| `crates/xtask/src/main.rs` | **Edited** — codegen list extension: add `plan` + `task` (Rust → `crates/runtime-core/src/generated/{plan,task}.rs`; TS → `src/types/{plan,task}.ts`) |
| `crates/runtime-core/src/generated/event.rs` | **Edited (regenerated)** — typify output reflects schema event reconciliation |
| `crates/runtime-core/src/generated/plan.rs` | **New (regenerated)** — typify output for `plan.v1.json` |
| `crates/runtime-core/src/generated/task.rs` | **New (regenerated)** — typify output for `task.v1.json` |
| `src/types/agent_event.ts` | **Edited (regenerated)** — json-schema-to-typescript output reflects schema event reconciliation |
| `src/types/plan.ts` | **New (regenerated)** — json-schema-to-typescript output for `plan.v1.json` |
| `src/types/task.ts` | **New (regenerated)** — json-schema-to-typescript output for `task.v1.json` |
| `crates/runtime-core/src/event.rs` | **Edited** — hand-curated `AgentEvent` wrapper mirrors schema (6 migrations + 2 deletions + 5 additions; carve-out for `task_rolled_back` + `task_*.plan_id`) |
| `crates/runtime-core/src/lib.rs` | **Edited** — top-level re-exports for `plan` + `task` (qualify-by-path per A1 retro decision: `pub use generated::{plan, task}`) |
| `crates/runtime-core/src/drone.rs` | **Edited** — new `DroneCommand::WriteSignal { signal_id, kind, source_id, context_type, payload }` variant per spec §2b signal shape |
| `crates/runtime-main/src/plan/mod.rs` | **New** — module root + public API |
| `crates/runtime-main/src/plan/state_machine.rs` | **New** — Plan/Task FSM per spec §3a (≥95% safety primitive) |
| `crates/runtime-main/src/sdk/plan_loop.rs` | **New** — drives plan FSM from agent loop; `fresh_context_per_task` implementation; failure escalation |
| `crates/runtime-main/src/sdk/structured_emitter.rs` | **New** — replaces `decision_extractor.rs` heuristic; parses `<<DECISION>>...<<END>>` + `<<PLAN>>...<<END>>` delimited blocks |
| `crates/runtime-main/src/sdk/approval.rs` | **New** — `ApprovalSeam` (tokio::sync::oneshot pattern) the SDK awaits on; Stage E wires HITL UI to the seam |
| `crates/runtime-main/src/sdk/agent_sdk.rs` | **Edited** — plan-loop integration; consume `ApprovalSeam`; consume `structured_emitter` |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | **Edited** — switch from `decision_extractor` callsite to `structured_emitter`; new event variants in translation paths |
| `crates/runtime-main/src/sdk/decision_extractor.rs` | **Deleted** — replaced by `structured_emitter.rs` (closes M02 🟡) |
| `crates/runtime-main/src/drone_ipc/client.rs` | **Edited** — new `DroneClient::write_signal(...)` method |
| `crates/runtime-drone/migrations/` | **New (directory)** — versioned migration files |
| `crates/runtime-drone/migrations/000_initial.sql` | **New** — preserves M01 baseline schema (existing `init_schema` content moves here verbatim) |
| `crates/runtime-drone/migrations/001_plans_tasks.sql` | **New** — first new migration; `plans` + `tasks` tables per spec §10 DDL |
| `crates/runtime-drone/src/db.rs` | **Edited** — migration-runner architecture: `run_migrations(conn)` + `_migrations` tracking table; `init_schema` becomes `run_migrations` wrapper |
| `crates/runtime-drone/src/plan_projector.rs` | **New** — drone-internal plan/task projector parallel to `vdr.rs` (≥95% safety primitive) |
| `crates/runtime-drone/src/command_handler.rs` | **Edited** — `WriteSignal` handler arm: insert into `signals` → call `vdr::project_signal` → call `plan_projector::project_signal` (when plan/task event) |
| `crates/runtime-drone/src/lib.rs` | **Edited** — export `plan_projector` |
| `crates/runtime-drone/src/snapshot.rs` | **Edited** — snapshot blob includes projected plan/task state alongside event log (per spec §1b recovery semantics) |
| `src/lib/graphStore.ts` | **Edited** — `applyEvent` exhaustive switch: 5 new + 6 changed + 2 dropped cases as pass-through state mutations; `_exhaustive: never` forcing function held |
| `crates/runtime-main/tests/plan_lifecycle.rs` | **New** — integration test: 3-task plan happy path; WriteSignal roundtrips; projection assertions |
| `crates/runtime-main/tests/plan_recovery.rs` | **New** — integration test: kill drone subprocess mid-plan; restart; verify projection rebuilds per spec §1b |
| `crates/runtime-drone/tests/plan_projector.rs` | **New** — projection idempotence; signal → plans/tasks row mapping |
| `crates/runtime-drone/tests/migration_runner.rs` | **New** — migration application; idempotent re-runs; version tracking |
| `tests/unit/graphStore.test.ts` | **Edited** — exhaustive `applyEvent` coverage for 11 new + 6 changed variants |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes M04 Stage B (event reconciliation + FSM + persistence + WriteSignal IPC + structured emitter + spec drift carve-outs) |

### B.3 Detailed Changes

#### `schemas/event.v1.json` — event-shape reconciliation (13 oneOf changes)

The schema is the source of truth (CLAUDE.md §14). Stage B re-authors the plan/task variants per the spec §3a lines 1417–1427 source, with two engineering carve-outs documented inline.

**6 spec-canonical variants — migrate to spec shape:**

| current schema variant (line ~) | spec §3a target shape | change |
|---|---|---|
| `plan_created { plan_id, task_count }` (~129) | `plan_created { plan_id, title, task_count, approval_required }` | add `title` (string, required), `approval_required` (boolean, required); keep `plan_id` + `task_count` |
| `plan_approved { plan_id }` (~140) | `plan_approved { plan_id, approved_by: 'user'\|'auto' }` | add `approved_by` enum (required) |
| `task_started { plan_id, task_id, agent_id }` (~161) | spec lean: `{ task_id, agent_id }`; **carve-out: keep `plan_id`** | drop nothing (carve-out); document drift |
| `task_completed { plan_id, task_id, duration_ms }` (~173) | spec lean: `{ task_id, duration_ms }`; **carve-out: keep `plan_id`** | drop nothing (carve-out); document drift |
| `task_failed { plan_id, task_id, error, failure_count }` (~185) | spec lean: `{ task_id, error, failure_count }`; **carve-out: keep `plan_id`** | drop nothing (carve-out); document drift |
| `task_escalated { plan_id, task_id, reason }` (~210) | `task_escalated { task_id, failure_count, max_failures }`; **carve-out: keep `plan_id`** | replace `reason` with `failure_count` (integer, required) + `max_failures` (integer, required); keep `plan_id` (carve-out) |

**2 codebase extras — disposition:**

| current schema variant (line ~) | disposition | rationale |
|---|---|---|
| `plan_rejected { plan_id, reason }` (~150) | **DROP** | spec §1439 unifies cancel under `plan_aborted`; renderer logic for "user said no" is the same regardless of phase |
| `task_rolled_back { plan_id, task_id, snapshot_id }` (~198) | **KEEP (drift carve-out)** | typed event with structured `snapshot_id` is sounder engineering than spec §4a's stringly-typed `error: 'rolled_back_after_hook_<id>'` (CLAUDE.md §9 anti-pattern: "stringly-typed APIs"). Spec drift target: §4a |

**5 missing variants — author per spec §3a:**

```json
{ "type": "object", "title": "PlanApprovalRequested", "properties": { "type": { "const": "plan_approval_requested" }, "plan_id": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "PlanRevised", "properties": { "type": { "const": "plan_revised" }, "plan_id": { "type": "string" }, "revision_reason": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "revision_reason", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "PlanAborted", "properties": { "type": { "const": "plan_aborted" }, "plan_id": { "type": "string" }, "reason": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "reason", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "PlanComplete", "properties": { "type": { "const": "plan_complete" }, "plan_id": { "type": "string" }, "duration_ms": { "type": "integer" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "duration_ms", "timestamp"], "additionalProperties": false },
{ "type": "object", "title": "TaskSkipped", "properties": { "type": { "const": "task_skipped" }, "plan_id": { "type": "string" }, "task_id": { "type": "string" }, "reason": { "type": "string" }, "timestamp": { "type": "integer" } }, "required": ["type", "plan_id", "task_id", "reason", "timestamp"], "additionalProperties": false }
```

(All 5 include `plan_id` per the denormalization carve-out for `task_*`; plan_* always carry `plan_id` in spec.)

After schema edits, run `cargo xtask regenerate-types` to propagate to `crates/runtime-core/src/generated/event.rs` + `src/types/agent_event.ts`. Hand-update `crates/runtime-core/src/event.rs` to mirror schema (the top-level wrapper used by callsites as `runtime_core::AgentEvent` is hand-curated; A1 typify-list extension created the generated parallel but did not retire the wrapper). Verify drift with `cargo xtask regenerate-types --check` exits 0.

#### `schemas/plan.v1.json` + `schemas/task.v1.json` — new schema files

Author each JSON Schema following the existing schema convention. Pre-flight `<fan_out_grep>` for `"$id"` across `schemas/*.v1.json` to confirm the `https://schemas.aria-runtime.dev/<name>.v1.json` base-URL pattern before writing the value (M03.5.A retro discipline).

Field shapes per spec §3a TS interfaces + spec §10 DDL:

- **Plan** (`plan.v1.json`): `id` (string, uuid), `session_id` (string, uuid), `title` (`$ref: '#/$defs/PlanTitle'` — `string` `minLength: 1` extracted to `$defs/PlanTitle` per A1 typify gotcha), `description?` (string), `status` (enum: `pending_approval | approved | in_progress | awaiting_replan | complete | aborted`), `approval_required` (boolean), `loop_policy` (enum: `one_shot | fresh_context_per_task | continuous` — only `fresh_context_per_task` lit in v0.1 per scope locks), `hitl_checkpoints` (array of strings), `risks` (array of strings), `created_by?` (string — agent_id or `'user'`), `created_at` (integer, unix ms), `approved_at?` (integer), `completed_at?` (integer).
- **Task** (`task.v1.json`): `id` (string, uuid), `plan_id` (string, uuid), `title` (`$ref: '#/$defs/TaskTitle'`), `status` (enum: `pending | running | done | failed | blocked | skipped | escalated`), `hitl` (boolean), `hitl_reason?` (string), `failure_count` (integer, default 0), `max_failures` (integer, default 3), `files_affected?` (array of glob strings), `acceptance_criteria?` (array of strings), `created_at` (integer), `started_at?` (integer), `completed_at?` (integer), `estimated_minutes?` (integer), `actual_minutes?` (integer).

#### `crates/xtask/src/main.rs` — codegen list extension

Add two entries to the existing 7-schema codegen list (`["common", "framework", "skill", "tool", "agent", "event", "error"]` post-A1 → 9-schema list):
- `("plan", "schemas/plan.v1.json")` → `crates/runtime-core/src/generated/plan.rs` + `src/types/plan.ts`
- `("task", "schemas/task.v1.json")` → `crates/runtime-core/src/generated/task.rs` + `src/types/task.ts`

Run `cargo xtask regenerate-types` to produce generated files. Run `--check` to verify deterministic output. Add `src/types/plan.ts` + `src/types/task.ts` to `.prettierignore` + `eslint.config.js` `ignores` list (matches `agent_event.ts` precedent per A1 retro).

#### `crates/runtime-drone/migrations/` + `db.rs` — migration runner architecture (NEW)

The phase doc's prior reference to "the existing M01.C migration runner pattern" was incorrect — `crates/runtime-drone/migrations/` does not exist; `db.rs::init_schema` uses `CREATE TABLE IF NOT EXISTS` with no version tracking. Stage B authors the architecture from scratch.

**Migration runner (`db.rs`):**

```rust
fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    // 1. Create _migrations table if not exists
    //    (version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at INTEGER NOT NULL)
    // 2. Read applied versions: SELECT version FROM _migrations
    // 3. Iterate migrations/*.sql in lexical order
    // 4. For each: parse leading NNN_<name>; skip if version applied; else exec + INSERT
    // 5. Wrap in transaction; rollback on error
}

fn init_schema(conn: &Connection) -> Result<(), DbError> {
    run_migrations(conn)
}
```

Migration files embedded via `include_str!("../migrations/000_initial.sql")` etc. (Build-time embed; no runtime filesystem dependency, matches the rusqlite + M01 single-binary deployment model.)

**`migrations/000_initial.sql`:** verbatim move of existing M01 `init_schema` content (8 tables: `sessions`, `snapshots`, `signals`, `heartbeats`, `vdr`, `token_usage`, `skills`, `mcp_servers`). Version 0 baseline. Idempotent re-application via the migration runner's `_migrations` check.

**`migrations/001_plans_tasks.sql`:** spec §10 DDL for `plans` + `tasks` tables. Plans table: `id TEXT PRIMARY KEY, session_id TEXT NOT NULL, title TEXT NOT NULL, description TEXT, status TEXT NOT NULL, approval_required INTEGER NOT NULL, loop_policy TEXT NOT NULL, hitl_checkpoints TEXT NOT NULL DEFAULT '[]', risks TEXT NOT NULL DEFAULT '[]', created_by TEXT, created_at INTEGER NOT NULL, approved_at INTEGER, completed_at INTEGER, FOREIGN KEY (session_id) REFERENCES sessions(id)`. Tasks table: per spec §10 + spec §3a Task TS shape. Plus indices for projector lookups: `CREATE UNIQUE INDEX idx_plans_id ON plans(id); CREATE UNIQUE INDEX idx_tasks_id ON tasks(id); CREATE INDEX idx_tasks_plan_id ON tasks(plan_id);`.

**Tests (`tests/migration_runner.rs`):** apply `000` + `001` to fresh DB; verify both rows in `_migrations`; re-apply (no-op verified by no INSERT); apply with `001` missing (only `000` applies); corrupt `_migrations` table behavior (clear `_migrations` row → migration re-applies which must be idempotent — this is the contract migration files honor).

#### `crates/runtime-core/src/drone.rs` — `DroneCommand::WriteSignal` variant

New variant per spec §2b signal shape:

```rust
WriteSignal {
    signal_id: String,           // uuid
    kind: SignalKind,            // existing 8-variant enum (Tool/Skill/Agent/Decision/Verify/Error/Hitl/Session)
    source_id: String,           // agent_id or session_id
    context_type: ContextType,   // existing enum (carry-forward reconciliation pending; M04 closeout)
    payload: serde_json::Value,  // type-erased event payload
},
```

Reuses existing `SignalKind` + `ContextType` from `crates/runtime-core/src/signal.rs`. The `ContextType` reconciliation with spec §2b (M02 carry-forward) is deferred to M04 closeout per the gap-analysis decision; Stage B uses the existing enum.

#### `crates/runtime-drone/src/command_handler.rs` + `plan_projector.rs` — IPC handler + projector

**Handler arm (`command_handler.rs`):** new `DroneCommand::WriteSignal` arm. Inside a transaction:

1. INSERT into `signals` table (matches `vdr.rs`-tested-shape).
2. Call `vdr::project_signal(conn, &signal_id)` for kinds 4 (Decision) + 5 (Verify) — existing M03 behavior.
3. Call `plan_projector::project_signal(conn, &signal_id)` for kinds whose payload is a plan/task event — new behavior.
4. Emit `SignalWritten` IPC response (or align with existing snapshot/event response shape).

**Projector (`plan_projector.rs`):** parallel to `vdr.rs::project_signal` (M03.E archetype):

```rust
pub fn project_signal(conn: &Connection, signal_id: &str) -> Result<(), ProjectorError> {
    // 1. SELECT signal payload by id
    // 2. Match payload.type:
    //    - "plan_created" → INSERT INTO plans (id, session_id, title, ...) ... ON CONFLICT(id) DO UPDATE
    //    - "plan_approved" → UPDATE plans SET status='approved', approved_at=... WHERE id=?
    //    - "plan_complete" → UPDATE plans SET status='complete', completed_at=... WHERE id=?
    //    - "plan_aborted" → UPDATE plans SET status='aborted' WHERE id=?
    //    - "task_started" → INSERT INTO tasks (...) ON CONFLICT(id) DO UPDATE SET status='running', started_at=?
    //    - "task_completed" → UPDATE tasks SET status='done', completed_at=?, actual_minutes=? WHERE id=?
    //    - "task_failed" → UPDATE tasks SET status='failed', failure_count=? WHERE id=?
    //    - "task_skipped" → UPDATE tasks SET status='skipped' WHERE id=?
    //    - "task_escalated" → UPDATE tasks SET status='escalated' WHERE id=?
    //    - "task_rolled_back" → UPDATE tasks SET status='failed' WHERE id=? + log snapshot_id reference
    //    - other → no-op
    // 3. Return Ok or ProjectorError::{InvalidPayload, DbError}
}
```

Idempotency invariants: every UPSERT path is safe to re-run (last-write-wins on `status` + timestamps; INSERT...ON CONFLICT handles re-projection). Tests (`tests/plan_projector.rs`): each event type → expected row state; double-projection no-op; out-of-order projection (task_completed before task_started) handled gracefully (UPSERT with status='done' wins).

#### `crates/runtime-main/src/sdk/structured_emitter.rs` — replaces `decision_extractor.rs`

Closes M02 🟡 carry-forward "Decision extractor → structured emitter migration." Mechanism:

- Prompt-template (loaded via framework.json or hand-coded in v0.1) injects delimited blocks the model emits:
  ```
  <<DECISION>>
    Decision: <text>
    Rationale: <text>
    Tool used: <text>
  <<END>>

  <<PLAN>>
    Plan ID: <uuid>
    Title: <text>
    Approval required: <boolean>
    Tasks:
      - <task1 title>
      - <task2 title>
  <<END>>
  ```
- Parser (`structured_emitter.rs::parse(text: &str) -> Vec<EmitterOutput>`) consumes blocks deterministically. Returns typed outputs: `EmitterOutput::Decision { decision, rationale, tool_used }` and `EmitterOutput::PlanCreation { plan_id, title, approval_required, task_titles }`.
- `event_pipeline.rs` switches its `flush_text_buffer` callsite from `decision_extractor::extract_decision` to `structured_emitter::parse`. The decision-record translation logic stays the same; only the input shape changes.
- `decision_extractor.rs` is **deleted** (closes the M02 false-positive concern: `Decision:` matched in code blocks / quoted content cannot trigger a false emit because parsing is delimiter-scoped, not line-scoped).

Tests (`structured_emitter.rs::tests`): single decision block; multiple decision blocks; nested blocks (rejected with parse error); malformed blocks (missing `<<END>>`, returns parse error); plan-creation block; mixed decision + plan blocks; empty input.

#### `crates/runtime-main/src/plan/state_machine.rs` — Plan/Task FSM

Pure-logic module. Two state machines:

**PlanStateMachine:**

```rust
pub fn transition(plan: &mut Plan, event: PlanEvent) -> Result<(), TransitionError> {
    match (&plan.status, event) {
        (PlanStatus::PendingApproval, PlanEvent::Approved) => { plan.status = PlanStatus::InProgress; Ok(()) }
        (PlanStatus::PendingApproval, PlanEvent::Aborted) => { plan.status = PlanStatus::Aborted; Ok(()) }
        (PlanStatus::InProgress, PlanEvent::Complete) => { plan.status = PlanStatus::Complete; Ok(()) }
        (PlanStatus::InProgress, PlanEvent::Aborted) => { plan.status = PlanStatus::Aborted; Ok(()) }
        (PlanStatus::InProgress, PlanEvent::AwaitingReplan(reason)) => { plan.status = PlanStatus::AwaitingReplan; Ok(()) }
        (PlanStatus::AwaitingReplan, PlanEvent::Revised) => { plan.status = PlanStatus::InProgress; Ok(()) }
        (PlanStatus::AwaitingReplan, PlanEvent::Aborted) => { plan.status = PlanStatus::Aborted; Ok(()) }
        (s, e) => Err(TransitionError::IllegalTransition { from: s.clone(), event: e })
    }
}
```

**TaskStateMachine:** parallel pattern over `TaskStatus` × `TaskEvent`.

Errors: `IllegalTransition { from, event }`, `MissingPrecondition { reason }`. No I/O, no async.

Tests: exhaustive transition matrix (every legal pair drives expected status; every illegal pair returns `IllegalTransition`); failure-escalation boundary (`failure_count = max_failures - 1` on `Failed` → still `Pending`; `failure_count = max_failures` → `Escalated`); plan-status invariants (`approval_required = false` skips `pending_approval`, starts at `in_progress` directly). ≥95% coverage gate.

#### `crates/runtime-main/src/sdk/plan_loop.rs` — agent-loop integration

Drives the plan FSM from the SDK's existing event loop. Hooks:

- After `structured_emitter::parse` yields `EmitterOutput::PlanCreation`: emit `plan_created`. If `approval_required`: emit `plan_approval_requested` immediately + suspend on `approval_seam.await_approval(plan_id).await?`. On `Approved`: emit `plan_approved { approved_by: 'user' }`. On `Revised`: emit `plan_revised { revision_reason }` + return to plan-creation flow. On `Aborted`: emit `plan_aborted { reason }`.
- After plan is `in_progress`: drive task execution loop per `loop_policy`:
  - **`fresh_context_per_task`** (only loop policy lit in v0.1): for each task in order, emit `task_started`; agent runs (via `agent_sdk::run_agent_with_provider_stream`); on terminal event, advance plan FSM. Between tasks, clear `messages` vec on the agent struct; reseed with `system_prompt + plan_summary + completed_tasks_summary + current_task` for the next task.
  - **`one_shot`** + **`continuous`**: return `LoopPolicyError::NotImplemented` (v0.1 scope per CLAUDE.md §3 + spec §0d).
- On `task_failed`: increment `failure_count`; if `>= max_failures` (default 3 per spec; framework JSON can override per task or session-wide): emit `task_escalated`. Stage E routes to HITL.
- On all tasks terminal-and-not-failed: emit `plan_complete { duration_ms }`.

Plan state lives in SDK + projection (NOT in agent message history). Clearing messages between tasks is a no-op for plan state.

#### `crates/runtime-main/src/sdk/approval.rs` — approval-gate seam

```rust
pub struct ApprovalSeam {
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<ApprovalDecision>>>>,
}

pub enum ApprovalDecision {
    Approved,
    Revised(String /* revision_reason */),
    Aborted(String /* reason */),
}

impl ApprovalSeam {
    pub async fn await_approval(&self, plan_id: &str) -> Result<ApprovalDecision, ApprovalError> {
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(plan_id.to_string(), tx);
        rx.await.map_err(|_| ApprovalError::Cancelled)
    }

    pub async fn resolve(&self, plan_id: &str, decision: ApprovalDecision) -> Result<(), ApprovalError> {
        let tx = self.pending.lock().await.remove(plan_id).ok_or(ApprovalError::NotFound)?;
        tx.send(decision).map_err(|_| ApprovalError::ReceiverDropped)
    }
}
```

Stage B exposes `ApprovalSeam`. Stage E wires the HITL UI: renderer's `approve_plan` / `revise_plan` / `abort_plan` Tauri commands call `approval_seam.resolve(...)`. Stage B does NOT implement the HITL UI.

Tests (`approval.rs::tests`): await + resolve happy path; cancel before resolve (drops sender); double-resolve (second returns `NotFound`); concurrent awaits on different plan_ids.

#### `crates/runtime-drone/src/snapshot.rs` — projection-aware snapshot

Existing `snapshot::write(conn, session_id, reason, state)` already accepts arbitrary `state: serde_json::Value`. Stage B extends the SDK-side state shape to include `{ "events": [...], "plans": [...], "tasks": [...] }` rather than just events.

On snapshot read at recovery: `snapshot::read_latest(conn, session_id)` returns the JSON; SDK loads `plans` + `tasks` arrays into the projection (or relies on the drone's already-projected `plans` + `tasks` tables — the snapshot is the authoritative state).

Tool-call uncertainty (per spec §1b): SDK detects `tool_invoked` events without matching `tool_result` in the replay window → marks `tool_call_uncertain` flag on the in-memory event. Renderer's UI prompt (Stage F) consumes the flag.

Currently-running task (per spec §1b): SDK rebuilds task state from snapshot + replay; if a task's last-emitted event is `task_started` and no `task_completed`/`task_failed`/etc. follows, set status to `pending` (NOT `running` — the agent process that was running it is dead).

#### `src/lib/graphStore.ts` — applyEvent exhaustive switch

The TypeScript `switch (event.type)` over `AgentEvent['type']` will compile-error after schema regeneration if any new variant is missing. Stage B implements all cases as **pass-through state mutations**:

- `plan_created`: insert `PlanNode { id: plan_id, title, status: 'pending_approval' | 'in_progress' /* per approval_required */, taskCount }` into `state.nodes`.
- `plan_approval_requested`: update PlanNode status → `'awaiting_approval'`.
- `plan_approved`: update PlanNode status → `'approved'` (Stage C makes this visually active).
- `plan_revised`: update PlanNode status → `'pending_approval'` + log revision_reason.
- `plan_aborted`: update PlanNode status → `'aborted'`.
- `plan_complete`: update PlanNode status → `'complete'` + record duration_ms.
- `task_started`: insert `TaskNode { id: task_id, plan_id, agent_id, status: 'running' }` linked to parent PlanNode via plan_id (the denormalization carve-out makes this lookup-free).
- `task_completed`: update TaskNode status → `'done'` + duration_ms.
- `task_failed`: update TaskNode status → `'failed'` + failure_count + error.
- `task_skipped`: update TaskNode status → `'skipped'` + reason.
- `task_escalated`: update TaskNode status → `'escalated'` + failure_count + max_failures.
- `task_rolled_back`: update TaskNode status → `'failed'` + snapshot_id reference (Stage D verify+rails consumes).

Stage C builds the visual treatment + ApprovalPanel + active-task animated edge from PlanNode → currently-running TaskNode. Stage B's job: make the store handle all 11 events without crashing, losing state, or failing the exhaustive check.

### B.4 Tests

#### Pedantic-pass preflight

Apply per `docs/gotchas.md` #21 to all new modules: `plan/state_machine.rs`, `plan/mod.rs`, `sdk/plan_loop.rs`, `sdk/structured_emitter.rs`, `sdk/approval.rs`, `plan_projector.rs`. Generated files (`generated/{plan,task,event}.rs`) exempt.

#### Unit tests (Rust)

- **`crates/runtime-main/src/plan/state_machine.rs::tests`** — exhaustive transition matrix:
  - Plan: every legal pair (PendingApproval×Approved → InProgress; PendingApproval×Aborted → Aborted; InProgress×Complete → Complete; etc.) drives expected status
  - Plan: every illegal pair (Complete×Approved; Aborted×Anything; etc.) returns `IllegalTransition`
  - Task: every legal pair (Pending×Started → Running; Running×Done; Running×Failed; Failed×Retry → Pending; Failed×Escalated; Blocked×Resolved → Pending)
  - Task: every illegal pair returns `IllegalTransition`
  - Failure escalation boundary: `failure_count = max_failures - 1` on `Failed` → still `Pending`; `failure_count = max_failures` → `Escalated` (with default `max_failures = 3`, so the 3rd failure does NOT escalate; the 4th does — verified explicitly)
  - Approval-required invariant: `approval_required = false` skips `pending_approval`; FSM starts at `in_progress` directly
- **`crates/runtime-main/src/sdk/structured_emitter.rs::tests`** — parser surface:
  - Single decision block (well-formed) → `[EmitterOutput::Decision { ... }]`
  - Multiple decision blocks → vec of decisions in order
  - Plan-creation block → `[EmitterOutput::PlanCreation { ... }]`
  - Mixed decision + plan blocks
  - Nested blocks (e.g., `<<DECISION>>` inside `<<PLAN>>`) → parse error `NestedBlock`
  - Malformed: missing `<<END>>` → parse error `Unterminated`
  - Malformed: unexpected text between blocks → preserved (not parsed; trailing text outside blocks is non-fatal)
  - Empty input → empty vec
  - **Forcing function** (closes M02 false-positive concern): "Decision: " text inside markdown code blocks (no `<<DECISION>>` wrapper) → empty vec (NOT a false positive)
- **`crates/runtime-main/src/sdk/approval.rs::tests`** — seam contract:
  - `await_approval` + `resolve(Approved)` happy path
  - `resolve` before `await_approval` → `NotFound`
  - Double-resolve same plan_id → second call returns `NotFound`
  - Concurrent `await_approval` on different plan_ids; `resolve` each independently
  - Sender dropped (channel cancelled) → `await_approval` returns `Cancelled`
- **`crates/runtime-drone/src/plan_projector.rs::tests`** — projection idempotence:
  - Each event type → expected row state in `plans` or `tasks`
  - Double-projection of same signal → no-op (UPSERT semantics; row state unchanged)
  - Out-of-order: `task_completed` before `task_started` projects, then `task_started` re-projects — last-write-wins on status; final state is `done` (a real session never emits this order; the test pins the contract)
  - Unknown event type → no-op (returns `Ok(())`)
  - Missing payload field → `ProjectorError::InvalidPayload`

#### Integration tests (Rust)

- **`crates/runtime-drone/tests/migration_runner.rs`** — migration architecture:
  - Apply `000_initial.sql` + `001_plans_tasks.sql` to fresh DB → both rows in `_migrations`; expected schema present
  - Re-apply on already-migrated DB → no-op (no INSERT into `_migrations`; tables unchanged)
  - Apply with only `000` registered → `001` runs; `_migrations` reflects both
  - Migration file not in directory but version in `_migrations` → graceful skip with warning (log; not a hard error)
  - Transaction rollback on migration failure (inject malformed SQL in a test migration; assert no partial state lands)
- **`crates/runtime-drone/tests/plan_projector.rs`** — drone-side roundtrip:
  - Spawn drone subprocess; send `WriteSignal { kind: Plan, payload: plan_created event }`; SELECT FROM plans; verify row matches event
  - Send sequence of plan/task events; SELECT FROM tasks; verify count + status fields per event sequence
  - Send unknown event kind; verify no projector side effects
- **`crates/runtime-main/tests/plan_lifecycle.rs`** — end-to-end plan flow:
  - Spawn drone subprocess + main; manual shim for `ApprovalSeam.resolve(Approved)`
  - Drive a 3-task plan via mock provider that emits structured `<<PLAN>>` block + per-task agent text
  - Assert event sequence: `plan_created` → `plan_approval_requested` → (shim resolves Approved) → `plan_approved` → `task_started` × 3 → `task_completed` × 3 → `plan_complete`
  - Assert SQLite state: `plans` table has 1 row (status='complete'); `tasks` table has 3 rows (all status='done'); `signals` table has all events; `vdr` table populated for any decision/verify signals
  - Failure path variant: 3rd task fails 4 times → `task_escalated` emitted; subsequent shim emits `task_skipped` → `plan_complete`
- **`crates/runtime-main/tests/plan_recovery.rs`** — recovery semantics per spec §1b:
  - Drive plan to mid-execution (after task_started[1], before task_completed[1])
  - Kill drone subprocess (SIGKILL on Unix; `Stop-Process -Force` on Windows)
  - Restart drone; SDK recovers
  - Assert: PlanNode status preserved; in-flight TaskNode set to `pending` (not `running`); projection rebuilt from snapshot; SDK can resume from `task_started` again
  - Tool-call uncertainty branch: kill between `tool_invoked` and `tool_result` → `tool_call_uncertain` flag set on the recovered event

#### Unit tests (TypeScript)

- **`tests/unit/graphStore.test.ts`** (extended) — exhaustive `applyEvent` coverage:
  - 1 case per new variant (5): `plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped` — assert resulting state has expected node+status changes
  - 1 case per changed variant (6): assert new fields are stored (e.g., `plan_created.title` lands on `PlanNode.title`; `task_escalated.failure_count` lands on `TaskNode.failureCount`)
  - 1 case per dropped variant: deleted `plan_rejected` is unreachable (compile-time — removed from `AgentEvent['type']`); `task_rolled_back` retained as drift carve-out (test asserts state lands)
  - `_exhaustive: never` line is the forcing function — no test needed (compile-time check covers it)

#### Schema drift gate

- **`cargo xtask regenerate-types --check`** must exit 0 after all schema edits land. Re-running `regenerate-types` (no `--check`) must produce byte-stable output.

#### Coverage targets

- `crates/runtime-main/src/plan/state_machine.rs` ≥95% (safety primitive per CLAUDE.md §5)
- `crates/runtime-main/src/sdk/structured_emitter.rs` ≥95% (replaces M02 safety primitive)
- `crates/runtime-main/src/sdk/approval.rs` ≥95% (new safety primitive: SDK suspension correctness)
- `crates/runtime-drone/src/plan_projector.rs` ≥95% (parallel to vdr.rs safety primitive)
- `crates/runtime-drone/src/db.rs` ≥95% (migration runner correctness; preserved per existing exclusion list)
- `crates/runtime-main` ≥95% maintained (per A2 baseline 98.09% line)
- `crates/runtime-drone` ≥95% maintained (per A2 baseline 95.86% line)
- workspace ≥80% maintained
- Generated files (`generated/{plan,task,event}.rs`) excluded via existing regex

### B.5 CLI Prompt

```xml
<work_stage_prompt id="M04.B">
  <context>
    Stage B of M04. §3a Plan & Task primitive — schema event-shape reconciliation (6 migrations + 2 deletions + 5 additions), Plan + Task FSM (≥95% safety primitive), projection-based persistence (new migration runner architecture + drone-internal plan/task projector parallel to vdr.rs), WriteSignal IPC variant + drone-side handler (closes M03 🟡), structured-emitter migration (closes M02 🟡), approval-gate seam (channel/oneshot the SDK awaits on; Stage E wires the HITL UI), fresh_context_per_task loop policy, failure escalation, snapshot-includes-projection recovery semantics per spec §1b. Two spec-drift carve-outs locked: typed task_rolled_back over stringly-typed error string; task_*.plan_id denormalization over spec's lean shape. Stage A2's commit must be on the milestone branch claude/m04-plan-verify-hitl-budget.
  </context>

  <pre_flight_check>
    <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
    <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage A2" subject (commit 2bf4d67 or successor)</check>
    <check name="a1_a2_artifacts_present">Test-Path crates/runtime-core/src/generated/error.rs (A1 codegen target) AND grep -q "DroneLifecycle" src-tauri/src/drone_lifecycle.rs (A2) must succeed</check>
    <check name="a1_namespace_decision_applied">grep -q "pub use generated::{agent, common, framework, skill, tool}" crates/runtime-core/src/lib.rs (A1 chose option (b) qualify-by-path)</check>
    <check name="a1_xtask_extended">grep -q "\"event\"" crates/xtask/src/main.rs AND grep -q "\"error\"" crates/xtask/src/main.rs (A1 extended codegen list to 7 schemas; B extends to 9 with plan + task)</check>
    <check name="schemas_drift_clean">cargo xtask regenerate-types --check exit 0 (A1+A2 codegen state durable before B's schema edits begin)</check>
    <check name="migrations_directory_absent">Test-Path crates/runtime-drone/migrations must FAIL — Stage B creates the directory + the migration runner architecture from scratch (the phase doc's prior reference to "existing M01.C migration runner" was incorrect)</check>
    <check name="event_diff_verified">grep -c "title.*plan_\\|title.*task_" schemas/event.v1.json must equal 8 (current state: 8 codebase variants — 6 spec-canonical with shape mismatches + 2 extras [plan_rejected, task_rolled_back]; Stage B reconciles to 11 spec-required with 1 retained extra)</check>
  </pre_flight_check>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Stage B sections B.1–B.4)</file>
    <file>agent-runtime-spec.md §3a (full section, especially Data types + Events lines 1417–1427 + Approval-gate primitive + Loop policy primitive + Failure escalation + Graph integration + Framework JSON), §1b (Recovery Semantics — projection rebuild + currently-running task + tool-call uncertainty), §2b (Signals + VDR — projector archetype), §10 (plans/tasks DDL)</file>
    <file>docs/MVP-v0.1.md §M4</file>
    <file>docs/gotchas.md (especially #14 snake_case discipline; #34 fmt-first; #41 grep-verify codebase claims; #42 origin partial view)</file>
    <file>docs/build-prompts/retrospectives/M04.A1-retrospective.md</file>
    <file>docs/build-prompts/retrospectives/M04.A2-retrospective.md (apply [END] Decisions)</file>
  </read_first>

  <read_reference>
    <file purpose="xtask codegen archetype Stage A1 established">crates/xtask/src/main.rs</file>
    <file purpose="existing schemas archetype + $id pattern; mirror for plan.v1.json + task.v1.json authoring">schemas/event.v1.json</file>
    <file purpose="error.v1.json post-A1 worked example for $defs validated-string extraction">schemas/error.v1.json</file>
    <file purpose="vdr.rs projector archetype — plan_projector.rs follows the same pattern">crates/runtime-drone/src/vdr.rs</file>
    <file purpose="db.rs current state — Stage B refactors init_schema into the new run_migrations runner">crates/runtime-drone/src/db.rs</file>
    <file purpose="command_handler.rs — Stage B adds the WriteSignal arm">crates/runtime-drone/src/command_handler.rs</file>
    <file purpose="snapshot.rs — Stage B extends the blob shape to include projected state">crates/runtime-drone/src/snapshot.rs</file>
    <file purpose="agent_sdk.rs — Stage B integrates plan_loop">crates/runtime-main/src/sdk/agent_sdk.rs</file>
    <file purpose="event_pipeline.rs — Stage B switches decision_extractor callsite to structured_emitter (NOT a phantom event_translation.rs)">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="decision_extractor.rs — DELETED in Stage B; reference for understanding the M02 heuristic before replacing">crates/runtime-main/src/sdk/decision_extractor.rs</file>
    <file purpose="graphStore applyEvent archetype M03.B established">src/lib/graphStore.ts</file>
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
    <grep pattern='"$id"' purpose="confirm $id base-URL pattern in schemas/*.v1.json before authoring plan.v1.json + task.v1.json (M03.5.A retro discipline)"/>
    <grep pattern="decision_extractor" purpose="enumerate every callsite that must switch to structured_emitter; expect event_pipeline.rs + agent_sdk.rs + tests"/>
    <grep pattern="DroneCommand::" purpose="enumerate every callsite that constructs DroneCommand variants; verify WriteSignal addition doesn't break exhaustiveness in match arms"/>
    <grep pattern="init_schema" purpose="enumerate callers; ensure refactor to run_migrations doesn't break the public API"/>
    <grep pattern="plan_rejected" purpose="enumerate references to the dropped variant; expect schema + event.rs + graphStore.ts; all must be removed"/>
  </fan_out_grep>

  <runtime_environment os="windows" note="Build agent on Windows 11; Test-Path replaces test -f; Get-ChildItem replaces ls; named pipe paths differ from Unix sockets in any drone-IPC test; subprocess SIGKILL is Stop-Process -Force on Windows for plan_recovery.rs test"/>

  <gotchas>
    <trap>AUDIT-CORRECTED: prior phase doc claim "8 of 11 already exist; author only 3 missing" was WRONG. Real diff (verified at authoring): 6 spec-canonical with shape mismatches (migrate); 2 codebase extras (drop plan_rejected; KEEP task_rolled_back per drift carve-out below); 5 missing (author plan_approval_requested, plan_revised, plan_aborted, plan_complete, task_skipped). Total 13 oneOf changes in schemas/event.v1.json.</trap>
    <trap>SPEC DRIFT CARVE-OUT (locked): keep `task_rolled_back` typed event with `snapshot_id` field over spec §4a's stringly-typed `error: 'rolled_back_after_hook_<id>'` pattern. Spec text encodes structured info in error strings — CLAUDE.md §9 anti-pattern. Typed event is sounder engineering. Stage D consumes the typed event. Drift target: spec §4a (closeout post-M04 docs(spec): PR).</trap>
    <trap>SPEC DRIFT CARVE-OUT (locked): keep `task_*.plan_id` denormalization over spec §3a's lean `task_id + agent_id` shape. Self-contained events for renderer/projector/replay; no separate plan_id lookup needed. Drift target: spec §3a (closeout post-M04 docs(spec): PR).</trap>
    <trap>AUDIT-CORRECTED: `crates/runtime-drone/migrations/` directory does NOT exist (verified via Test-Path). Phase doc's prior reference to "existing M01.C migration runner pattern" was wrong — there IS no migration runner. Stage B authors the architecture from scratch: `db.rs::run_migrations(conn)` + `_migrations` tracking table; `migrations/000_initial.sql` preserves M01 baseline (verbatim move of existing init_schema content); `migrations/001_plans_tasks.sql` adds plans + tasks per spec §10. Migration files embedded via `include_str!` (build-time embed; matches single-binary deployment).</trap>
    <trap>AUDIT folds A2 deferrals (closes M02 + M03 carry-forward in this milestone): (a) `WriteSignal` IPC variant + drone-side handler arm calling `vdr::project_signal` AND `plan_projector::project_signal`. (b) structured-emitter prompt-template module + AgentSdk plumbing replacing the M02 heuristic in decision_extractor.rs (DELETED). vdr lives in drone (not main; runtime-main has no rusqlite). Pattern: drone-internal projection at write-time, parallel to existing vdr architecture.</trap>
    <trap>Plan/Task projector (`plan_projector.rs`) is a SAFETY PRIMITIVE — coverage gate ≥95%. Pattern parallel to vdr.rs (M03.E archetype). Idempotent UPSERT semantics; out-of-order projection handled gracefully.</trap>
    <trap>AUDIT post-A1 namespace: A1 chose option (b) qualify-by-path. Generated `CmdError` + `AgentEvent` live under `runtime_core::generated::{error,event}`; top-level `error.rs::RuntimeError` + `event.rs::AgentEvent` stay hand-curated. Stage B's regen targets land at `crates/runtime-core/src/generated/{plan,task}.rs` per the same convention.</trap>
    <trap>EVENT.RS HAND-CURATED: top-level `crates/runtime-core/src/event.rs` is the wrapper used by callsites as `runtime_core::AgentEvent`. After schema edits + xtask regen produces `generated/event.rs`, hand-update the top-level event.rs to mirror schema. The two files coexist (qualify-by-path). M03 carry-forward "event.rs hand-maintained" — partial close in A1 (typify list extended); full close pending wrapper removal in a future milestone.</trap>
    <trap>A1's schema metadata pattern is the archetype for `plan.v1.json` + `task.v1.json` — variant titles PascalCased; validated string fields with `minLength` extracted to `$defs/<Name>` so typify 0.6.2 can name the validation newtype. See `schemas/error.v1.json` post-A1.</trap>
    <trap>v0.1 hardcodes STANDARD mode + fresh_context_per_task — schemas declare 3 loop policies but only fresh_context_per_task is lit; the `one_shot` and `continuous` variants in the schema are spec-prep, not v0.1 implementation. Stage B's loop-policy seam returns `LoopPolicyError::NotImplemented` for the other two.</trap>
    <trap>plan.v1.json + task.v1.json `$id` MUST follow `https://schemas.aria-runtime.dev/<name>.v1.json` (M03.5.A retro). Run `<fan_out_grep pattern='"$id"'/>` against existing schemas BEFORE authoring.</trap>
    <trap>graphStore.ts applyEvent exhaustive switch will compile-error on the 5 new + 6 changed + 2 dropped variants — `_exhaustive: never` is the forcing function (gotcha #36). Compiler catches missing/incorrect cases; rely on it. State mutations are pass-through (no visual treatment — Stage C lights up the visual surface for plan/task events + ApprovalPanel).</trap>
    <trap>Plan state machine is a SAFETY PRIMITIVE — coverage gate ≥95%. Document any exclusions inline (likely none — pure-logic module). Per CLAUDE.md §5 + M01.C precedent.</trap>
    <trap>Approval-gate seam is `tokio::sync::oneshot` channel pattern; Stage B exposes `ApprovalSeam::await_approval(plan_id)` for SDK to await on; Stage E wires the HITL UI to call `ApprovalSeam::resolve(plan_id, decision)` from the renderer's approve_plan/revise_plan/abort_plan Tauri commands. Do NOT implement the HITL UI in Stage B.</trap>
    <trap>fresh_context_per_task implementation — clearing the agent's `messages` vec mid-session must NOT clear the SDK's plan-state. Plan state lives in the SDK + projection (plans + tasks tables), NOT in the agent's message history.</trap>
    <trap>Recovery test (`plan_recovery.rs`) requires real subprocess kill + restart. On Windows: `Stop-Process -Force` (the M04.A2 retrospective's locked-binary issue applies — orphan drone subprocess on test re-run; per gotcha graduation candidate, Get-Process before re-running). On Unix: SIGKILL via `Child::start_kill`.</trap>
    <trap>Spec §1b recovery semantics — currently-running task set to `pending` (NOT `running`); tool-call uncertainty flag (`tool_call_uncertain`) on tool_invoked without matching tool_result. Both invariants tested in plan_recovery.rs.</trap>
    <trap>Snapshot blob shape extends to include projected `plans` + `tasks` arrays alongside event log. Drone projection is the read-model; snapshot is the persistence layer that recovery rebuilds from. Spec §1b: "Plan + task statuses restored from snapshot."</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT implement the orchestrator agent's prompt template — that's framework-JSON territory (loaded from examples/aria/framework.json at session start). Stage B provides the SDK machinery (FSM + event emission + loop policy + failure escalation + structured-emitter parser); framework JSON wires the orchestrator.</warning>
    <warning>DO NOT wire the renderer's PlanNode/TaskNode to active visual treatment — Stage B's graphStore changes are pass-through state mutations only. Stage C builds the visual surface + ApprovalPanel + animated edge from PlanNode → currently-running TaskNode.</warning>
    <warning>DO NOT implement the HITL UI — Stage E owns ApprovalPanel + the 9-trigger HITL flow. Stage B exposes only the `ApprovalSeam` channel/oneshot the SDK awaits on.</warning>
    <warning>DO NOT push between stages.</warning>
    <warning>DO NOT skip the migration runner architecture by inlining `CREATE TABLE IF NOT EXISTS` in init_schema. The runner with `_migrations` version tracking is the deliverable; subsequent migrations (Stage D verify+rails may add hooks tables; Stage E HITL may add hitl_requests; Stage F budget may add token_usage extensions) depend on the architecture being in place.</warning>
    <warning>DO NOT keep `decision_extractor.rs` alongside `structured_emitter.rs` as a fallback — the migration is full replacement (closes M02 false-positive concern). Delete the file; switch the event_pipeline.rs callsite.</warning>
  </execution_warnings>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage C: which Plan + Task fields the renderer's PlanNode/TaskNode actually need to render (likely: title, status, task_count for Plan; status, agent_id, failure_count for Task — subset of full struct); whether the ApprovalPanel needs additional fields beyond plan + risks + hitl_checkpoints (likely: estimated_minutes for total time display; created_by for source attribution); whether `_exhaustive: never` caught all 11 variants in graphStore (forcing function discipline); plan state machine coverage % achieved + any holdouts; structured-emitter false-positive elimination evidence (test that "Decision: " in code blocks doesn't trigger emit); WriteSignal IPC roundtrip latency observed; recovery test pass/fail + projection-rebuild time; whether the snapshot-includes-projection blob shape was sufficient or needs Stage F adjustment for budget recovery; spec-drift carve-outs status (both flagged for closeout docs(spec): PR).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M04-plan-verify-hitl-budget.md" section="B.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>git log --oneline main..HEAD (per CLAUDE.md §19 rule 7; build-machine state visible to any downstream orchestration)</item>
    <item>retrospective file listing (`Get-ChildItem docs/build-prompts/retrospectives/M04.*-retrospective.md`)</item>
    <item>gate results (each gate, pass/fail; FSM coverage % must be ≥95; structured_emitter coverage ≥95; approval coverage ≥95; plan_projector coverage ≥95; runtime-main + runtime-drone overall ≥95; workspace ≥80)</item>
    <item>schema drift check exit 0 (`cargo xtask regenerate-types --check`)</item>
    <item>generated file shape preview — first 30 lines each of crates/runtime-core/src/generated/{plan,task}.rs + src/types/{plan,task}.ts</item>
    <item>migration runner verification — `cargo test --package runtime-drone migration_runner` passes; show test output</item>
    <item>plan_lifecycle.rs integration test outcome — full 3-task plan flow end-to-end + failure-escalation variant</item>
    <item>plan_recovery.rs integration test outcome — kill mid-plan + restart; projection rebuilt; currently-running task = pending</item>
    <item>structured_emitter false-positive evidence — test asserting "Decision: " in code block doesn't emit</item>
    <item>spec-drift carve-out flag — both items (task_rolled_back typed; task_*.plan_id) noted as ⚠️ carry-forward to closeout gap-analysis</item>
    <item>retrospective with [END] decisions for Stage C</item>
    <item>draft commit message from B.6 (filled with session URL)</item>
    <item>"Stage M04.B is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### B.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime): M04 Stage B — §3a Plan & Task primitive (schemas + FSM + projection-based persistence + WriteSignal IPC + structured emitter)

Builds the §3a Plan & Task primitive end-to-end. Schema event-shape
reconciliation against spec §3a (lines 1417–1427). Two M02/M03
carry-forward items folded in (close M02 🟡 + M03 🟡). Two engineering
spec-drift carve-outs locked + flagged for closeout docs(spec): PR.

Schema event reconciliation (13 oneOf changes in schemas/event.v1.json):
- 6 spec-canonical migrated to spec shape: plan_created (+ title +
  approval_required); plan_approved (+ approved_by); task_escalated
  (replace reason with failure_count + max_failures); task_started /
  task_completed / task_failed (no shape change beyond drift carve-out)
- 2 codebase extras: drop plan_rejected (spec §1439 unifies cancel
  under plan_aborted); KEEP task_rolled_back as typed event with
  snapshot_id (drift carve-out — spec §4a's stringly-typed error
  pattern is CLAUDE.md §9 anti-pattern; typed event is sounder
  engineering)
- 5 missing authored: plan_approval_requested, plan_revised,
  plan_aborted, plan_complete, task_skipped
- task_*.plan_id denormalization kept (drift carve-out — self-contained
  events for renderer/projector/replay)
- Both drifts flagged for closeout post-M04 docs(spec): PR

New artifacts:
- schemas/plan.v1.json + schemas/task.v1.json (JSON Schema 2020-12;
  $id pattern matches existing schemas; validated strings in $defs/<Name>
  per A1 typify-friendliness gotcha)
- crates/runtime-core/src/generated/{plan,task}.rs (typify-regenerated)
- src/types/{plan,task}.ts (json-schema-to-typescript-regenerated)
- crates/runtime-main/src/plan/{mod,state_machine}.rs — Plan + Task FSM
  per spec §3a (≥95% safety primitive; exhaustive transition matrix)
- crates/runtime-main/src/sdk/plan_loop.rs — agent-loop integration;
  fresh_context_per_task; failure escalation (failure_count >=
  max_failures triggers task_escalated)
- crates/runtime-main/src/sdk/structured_emitter.rs — replaces M02
  decision_extractor.rs heuristic (closes M02 🟡); parses delimited
  <<DECISION>>...<<END>> + <<PLAN>>...<<END>> blocks; eliminates
  false-positive concern (Decision: in code blocks doesn't trigger emit)
- crates/runtime-main/src/sdk/approval.rs — ApprovalSeam (tokio oneshot
  channel) the SDK awaits on; Stage E wires HITL UI to the seam
- crates/runtime-drone/src/plan_projector.rs — drone-internal continuous
  projector parallel to vdr.rs (≥95% safety primitive); UPSERTs
  plans + tasks rows from plan/task signals; idempotent
- crates/runtime-drone/migrations/ (new directory) +
  migrations/000_initial.sql (preserves M01 baseline) +
  migrations/001_plans_tasks.sql (spec §10 DDL)

Migration runner architecture (closes implicit M01 gap; phase doc's
prior reference to "existing M01.C migration runner" was incorrect):
- crates/runtime-drone/src/db.rs::run_migrations(conn) reads
  migrations/NNN_<name>.sql files in lexical order; tracks applied
  versions in _migrations table; idempotent re-application
- init_schema becomes wrapper that calls run_migrations
- Migration files embedded via include_str! (build-time embed; matches
  single-binary deployment model)

WriteSignal IPC + drone-side handler (closes M03 🟡 vdr-projector-wired-
at-signal-write):
- crates/runtime-core/src/drone.rs: new DroneCommand::WriteSignal variant
- crates/runtime-drone/src/command_handler.rs: handler arm inserts into
  signals → calls vdr::project_signal → calls plan_projector::project_signal
  for plan/task events
- crates/runtime-main/src/drone_ipc/client.rs: new write_signal method

Snapshot integration + recovery (per spec §1b literal):
- crates/runtime-drone/src/snapshot.rs: blob shape extends to include
  projected plans + tasks arrays alongside event log
- Recovery: load snapshot → restore projection → replay post-snapshot
  events; currently-running task set to pending (not running);
  tool-call uncertainty (tool_call_uncertain flag on tool_invoked
  without matching tool_result)

Edits:
- crates/xtask/src/main.rs: codegen list extended from 7 to 9 schemas
  (added plan + task)
- crates/runtime-core/src/event.rs: hand-curated wrapper mirrors schema
  (qualify-by-path namespace per A1 retro decision retained)
- crates/runtime-core/src/lib.rs: re-exports plan + task
- crates/runtime-main/src/sdk/agent_sdk.rs: plan_loop integration;
  consume ApprovalSeam; consume structured_emitter
- crates/runtime-main/src/sdk/event_pipeline.rs: switch
  decision_extractor callsite to structured_emitter
- crates/runtime-main/src/sdk/decision_extractor.rs: DELETED (closes
  M02 🟡 structured-emitter migration)
- src/lib/graphStore.ts: applyEvent exhaustive switch handles 5 new +
  6 changed + 2 dropped variants as pass-through state mutations
  (Stage C builds visual treatment); _exhaustive: never forcing
  function held
- .prettierignore + eslint.config.js: ignore src/types/{plan,task}.ts
  per agent_event.ts precedent (A1 retro)

Tests:
- crates/runtime-main/src/plan/state_machine.rs::tests: exhaustive
  transition matrix; failure-escalation boundary; plan-status invariants
  (≥95% coverage met)
- crates/runtime-main/src/sdk/structured_emitter.rs::tests: parser
  surface (delimited blocks; nested; malformed; false-positive
  elimination)
- crates/runtime-main/src/sdk/approval.rs::tests: seam contract
- crates/runtime-drone/src/plan_projector.rs::tests: projection
  idempotence; out-of-order projection; UPSERT semantics
- crates/runtime-drone/tests/migration_runner.rs: apply + re-apply
  idempotence; transaction rollback on failure
- crates/runtime-drone/tests/plan_projector.rs: drone-side roundtrip
- crates/runtime-main/tests/plan_lifecycle.rs: end-to-end 3-task plan
  + failure-escalation variant
- crates/runtime-main/tests/plan_recovery.rs: kill drone subprocess
  mid-plan; restart; projection rebuilt; currently-running task =
  pending; tool-call uncertainty flag set
- tests/unit/graphStore.test.ts: extended applyEvent exhaustive
  coverage for 11 new + 6 changed variants

v0.1 scope locks held: STANDARD mode hardcoded, fresh_context_per_task
only (one_shot + continuous return LoopPolicyError::NotImplemented),
Novice + Promoted tiers only.

Spec drift items (carry-forward to M04 closeout docs(spec): PR):
- §3a event shapes: task_* events keep plan_id (denormalization for
  self-contained downstream consumers)
- §4a rollback: task_rolled_back as typed event with snapshot_id
  field (replaces stringly-typed task_failed with rolled_back error
  string)

Refs: M04-plan-verify-hitl-budget.md §B, spec §3a + §1b + §10, MVP §M4
Carry-forward closes: M02 🟡 structured-emitter migration; M03 🟡 vdr
  projector wired at signal-write; M01 implicit migration-runner gap
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
    <check name="arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Stages C/E/F all consume Arc&lt;DroneClient&gt; managed-state)</check>
    <check name="drone_lifecycle_present">Test-Path src-tauri/src/drone_lifecycle.rs (A2 deliverable)</check>
    <check name="plan_task_node_synthetic">Test-Path src/components/nodes/PlanNode.tsx AND Test-Path src/components/nodes/TaskNode.tsx (M03.C synthetic-state archetypes; Stage C drives live; verify state shapes match Stage B's graphStore.applyEvent additions before authoring)</check>
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
    <check name="arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Stage D consumes via RevertToSnapshot IPC dispatch)</check>
    <check name="audit_baseline_verify_events">grep -q "VerifyStarted\|VerifyPassed\|VerifyFailed\|RailTriggered" crates/runtime-core/src/event.rs (audit baseline; 4 events ALREADY exist — Stage D wires NOT re-authors as hook_*)</check>
    <check name="audit_baseline_revert_to_snapshot">grep -q "RevertToSnapshot" crates/runtime-core/src/drone.rs (audit baseline; variant ALREADY exists with RevertReason enum: HookRollback, UserRollback, GapRecovery — Stage D consumes, does NOT re-add)</check>
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
    <trap>AUDIT: event names ALREADY exist as `verify_started`/`verify_passed`/`verify_failed` + `rail_triggered` in `crates/runtime-core/src/event.rs` and `schemas/event.v1.json`. The original Phase doc planned new `hook_started`/`hook_passed`/`hook_failed` events — do NOT re-author. Adopt the existing `verify_*` names for the events. The Hook *primitive* (HookRef + HookCategory + Hook in `hook.v1.json`) keeps the "hook" terminology where it's the framework-config primitive that fires the events; the *event variants* stay as `verify_*` per codebase convention.</trap>
    <trap>AUDIT: `RevertToSnapshot` ALREADY exists in `crates/runtime-core/src/drone.rs::DroneCommand` with the correct `RevertReason` enum (`HookRollback`, `UserRollback`, `GapRecovery`). Stage D CONSUMES the existing variant — does NOT re-add it. Verify the `RevertReason` enum shape matches spec §4a + Stage D's needs; if any variant is missing, that's an additive edit, not a rebuild. Drone-side handler arm at `crates/runtime-drone/src/command_handler.rs` already exists; only extend if `RevertReason::HookRollback` shape changes.</trap>
    <trap>AUDIT: `pre_file_edit` is the genuinely-new firing point — verify spec §4a firing-point table currently lists 6 firing points; Stage D's `hook.v1.json` adds the 7th. Spec text edit either lands in-stage (if &lt;5 lines) or as a follow-up doc PR per retro decision.</trap>
    <trap>AUDIT scope summary: events done, drone command done. Hook primitive + Rails + don't-touch + pre_file_edit firing point + cross-platform shell wrapper are the substantial new work. Stage D is moderately smaller than the original Phase doc draft.</trap>
    <trap>shell.rs cross-platform — gotcha #32 cross-stack discipline applies. Verify pwsh.exe -NoProfile -Command semantics against current Microsoft PowerShell docs URL (WEBCHECK) BEFORE authoring; do NOT assume bash -c semantics carry over.</trap>
    <trap>JSONLogic operator allowlist (gotcha #18) — Rails check field. Operators beyond the allowlist return UnsupportedOperator; do NOT silently extend the operator set.</trap>
    <trap>Don't-touch glob matcher fires on pre_file_edit — every Write tool call routes through it BEFORE the OS write. If the rail evaluator is async, ensure the Write call awaits the rail decision; otherwise edits land before the rail blocks.</trap>
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
    <check name="arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Stage E consumes via respond_hitl IPC dispatch)</check>
    <check name="audit_baseline_hitl_event_naming">grep -q "HitlRequested\|HitlResolved" crates/runtime-core/src/event.rs AND ! grep -q "HitlResponse" crates/runtime-core/src/event.rs (audit baseline; codebase NAMES `hitl_resolved` NOT `hitl_response` — Stage E wires existing 2 events; if HitlResponse appears the codebase has drifted)</check>
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
    <trap>AUDIT event-name correction: the codebase has `hitl_resolved` (NOT `hitl_response` as the original Phase doc claimed). Adopt `hitl_resolved` throughout. The 3 NEW HITL events stay as planned: `hitl_timeout` + `notifier_dispatched` + `notifier_failed`. The existing pair is `hitl_requested` + `hitl_resolved` at `event.rs:281, :290`.</trap>
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
    <check name="arc_droneclient_managed">grep -q "manage(.*DroneClient\|manage(.*Arc" src-tauri/src/main.rs (A2 deliverable; Budget queries + Recovery resume both go through drone IPC)</check>
    <check name="audit_baseline_budget_events">grep -q "BudgetWarn\|BudgetDownshift\|BudgetSuspended\|BudgetExceeded" crates/runtime-core/src/event.rs (audit baseline; 4 events ALREADY exist — Stage F WIRES the new enforcer to fire these, does NOT re-add. If they're missing the codebase has drifted)</check>
    <check name="vdr_in_drone_only">Test-Path crates/runtime-drone/src/vdr.rs AND ! Test-Path crates/runtime-main/src/vdr.rs (audit baseline; vdr lives in drone NOT main; runtime-main has NO rusqlite dep — VDR access from main goes through drone IPC)</check>
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
    <file purpose="VDR projection — drone-side projector; uncertainty.rs queries via drone IPC since runtime-main has no rusqlite dep (NOT phantom runtime-main/src/vdr.rs)">crates/runtime-drone/src/vdr.rs</file>
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
    <trap>AUDIT: 4 budget event variants ALREADY EXIST in `crates/runtime-core/src/event.rs` + `schemas/event.v1.json` + `src/lib/graphStore.ts` (`budget_warning`, `budget_downshift`, `budget_suspended`, `budget_exceeded`). The original Phase doc said "add 4 new variants" — that's wrong. Stage F WIRES the budget enforcer to EMIT the existing events; does NOT re-add them. Schema drift check passes with no new variants; regen verifies.</trap>
    <trap>AUDIT path correction: `crates/runtime-main/src/vdr.rs` is a phantom. VDR lives in `crates/runtime-drone/src/vdr.rs`. `runtime-main` has NO `rusqlite` dependency — VDR access from main goes through drone IPC (likely via a new `QueryVdr { ... }` DroneCommand variant or by extending the existing `QuerySessionDb` if the SQL coverage is sufficient). Adjust X.2/X.3 paths and the IPC integration accordingly.</trap>
    <trap>AUDIT scope summary: events done; everything else (enforcer + recovery + UI) genuinely new. Recovery primitive (resume rebuilds history not re-execute, tool-call-uncertain UI prompt with 4-action options, MCP reconnect on resume, plan/capability state restoration) and budget enforcer (3 scopes + 4 threshold actions + downshift_hook + session header bar UI) are both genuinely new work. Token-cost computation uses Stage A2's real `count_tokens` endpoint with LRU per-message caching.</trap>
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
    <special_log>Decisions for Stage G (Phase Closeout): which M04 carry-forward items are fully closed vs need v1.0 escalation; whether the budget downshift_hook ladder configurability landed in framework JSON or stayed hardcoded; whether tool-call uncertainty UI surfaced any spec gaps in the 4-action semantics. Note: §1d long-lived events() reconnect note already CLOSED at A2 (v0.1 behavior locked: subscriptions do NOT survive reconnect; renderers resubscribe; integration test at crates/runtime-main/tests/drone_reconnect_events.rs); G's gap-analysis records the closure but does not need to re-validate.</special_log>
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

- [ ] **Hard Gate G1: do-not-commit-until-approved held** — every stage commit happened only after explicit user approval (8 approval gates across Stages A1, A2, B, C, D, E, F, G; original eight-stage plan included a separate A3 for vdr WriteSignal IPC + structured-emitter that was folded into Stage B per the post-A2 audit re-staging — net 7 work stages + Stage G closeout = 8 commits + 8 approval gates)
- [ ] User has reviewed each stage retrospective; scoring matches observable evidence
- [ ] M04-summary verdict is "Pattern held" (sound) or "Pattern held with friction"; not "Pattern strained"
- [ ] Three-artifact review per CLAUDE.md §20 complete: code diff + retrospectives/summary + gap-analysis entry all reviewed together
- [ ] PR creation deferred to explicit user instruction (do NOT auto-open per established convention)

---

*End of M04 specification + stage prompts. Eight stages on one parent-milestone branch (`claude/m04-plan-verify-hitl-budget`); Stage G is Phase Closeout per CLAUDE.md §20. PR drafts at end of Stage G and pushes after explicit approval. M05 (gap detection + capability enforcement) follows on a separate branch once this milestone merges.*
