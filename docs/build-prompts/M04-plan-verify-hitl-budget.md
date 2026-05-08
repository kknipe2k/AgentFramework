# M04 Plan + Verify + HITL + Budget — Specification + Stage Prompts

**Protocol version:** v1.3 (first milestone authored on the v1.3 XML stage-prompt schema; uses `<pre_flight_check>`, `<schema_drift_check>`, `<fan_out_grep>`, `<dependency_audit_check>`, and `<runtime_environment>` tags per `STAGE-PROMPT-PROTOCOL.md` v1.3).
**Date:** 2026-05-07
**Status:** Design approved — implement one stage at a time, in order
**Scope:** Build the four agentic primitives that turn a single-agent smoke into an actual agentic runtime: §3a Plan & Task (with `fresh_context_per_task` loop policy + 11 plan/task events + ApprovalPanel), §4a Verify & Rails (Hook primitive with 7 firing points including new `pre_file_edit` + Rails hard/soft + don't-touch globs + `revert_to_snapshot` drone command), §6a HITL (9 trigger types + 3 UI variants Panel/Modal/Toast + 3 built-in notifiers terminal_bell/desktop/sound + plugin interface), §2a Budget (3 scopes + 4 threshold actions + downshift_hook + 4 budget events + UI header bar). Plus §1b Recovery (resume rebuilds history, tool-call-uncertain prompt) and the M03 production-wiring carry-forward (drone subprocess + Arc<DroneClient> + vdr.rs projector + decision-extractor migration + real `count_tokens` endpoint + long-lived `events()` reconnect). Eight stages on one feature branch (`claude/m04-plan-verify-hitl-budget`); Stage G is Phase Closeout per CLAUDE.md §20. Spec §1b + §2a + §3a + §4a + §6a + MVP §M4 acceptance criteria.

---

## Background and Design Decision

**Problem.** M03 lit up the live graph for the M02 single-agent smoke session — one AgentNode renders, click-to-inspect works, token weight scales. The other 10 node types (PlanNode, TaskNode, VerifyNode, HookNode, GapNode, HITLNode, MCPNode + four Plan/Task event types) render in unit tests with synthetic state but never light up live: no event source fires their corresponding `AgentEvent` variants. Spec §M4 declares the four primitives (plan, verify, HITL, budget) that produce those events. Loading `examples/aria/framework.json` and seeing a multi-task plan render with verify hooks firing post-task and the budget header bar tracking session spend is the M04 success surface.

**Solution.** Eight stages on one feature branch (`claude/m04-plan-verify-hitl-budget`), each a fresh Claude Code session per the v1.3 XML stage-prompt protocol. Stage A is split into A1 (build hygiene — xtask codegen extensions for `event.v1.json` + `error.v1.json` + drone-test retrofits + `tokio::time::pause()` coverage closures) and A2 (production wiring — drone subprocess lifecycle at Tauri startup, `Arc<DroneClient>` Tauri-managed-state, vdr.rs projector at signal-write call-site, decision-extractor → structured emitter migration, real `count_tokens` endpoint, long-lived `events()` reconnect resolution). Stage B builds the §3a Plan & Task primitive end-to-end: `Plan` + `Task` types in `runtime-core` (typified from new `plan.v1.json` + `task.v1.json`), 11 plan/task events added to `event.v1.json`, plan state machine with `fresh_context_per_task` loop policy, failure escalation, plans + tasks SQLite tables (per spec §10 DDL added in M03.5). Stage C lights up the renderer surface — wires already-shipped PlanNode/TaskNode from M03 to the new event variants, builds the ApprovalPanel for plan approval gate, threads the approval flow renderer→main→drone→main→renderer. Stage D builds §4a Verify & Rails: Hook primitive (HookRef + 7 firing points including new `pre_file_edit`), Rails (hard/soft + JSON-declared), don't-touch glob matcher, `revert_to_snapshot` drone command + reason enum, VerifyNode + HookNode wired to the new `hook_*` + `rail_triggered` events. Stage E builds §6a HITL: 9 trigger types, 3 UI variants (Panel/Modal/Toast), notifier plugin interface, 3 built-in notifiers (terminal_bell/desktop via Tauri notification plugin/sound), 5 HITL events (`hitl_requested`/`hitl_response` existing + `hitl_timeout` + `notifier_dispatched` + `notifier_failed` new), failure-escalation flow (`task_escalated` → `on_failure_threshold` → `hitl_requested` → notifiers parallel → 1h default timeout). Stage F builds §2a Budget + §1b Recovery: Budget (3 scopes + 4 threshold actions + downshift_hook + 4 budget events + session header bar), Recovery (resume rebuilds history not re-execute, tool-call-uncertain UI prompt, MCP reconnect on resume, plan state restoration). Stage G is Phase Closeout — gap-analysis entry per CLAUDE.md §20, M04 summary, three-artifact review.

**Why one PR for the parent milestone (not one PR per stage).** Same logic as M01–M03 — eight stages-as-commits-on-one-branch gives incremental discipline (each stage is reviewable; each stage retrospective surfaces friction early) without the overhead of eight PR reviews for one logical milestone. Consistent with the per-milestone-as-PR pattern in `docs/build-prompts/README.md`. M03 (six stages, ~10h actual) proved the pattern at scale.

**Why eight stages, not fewer.** Five primitives + drone Phase 2 wiring + Phase Closeout is genuinely more surface area than M03's live graph. Calibrated estimate: ~39–54h actual (~3.5×–5× M03 actual) reflecting the new domain density and cross-stack risk surface. Splitting into eight stages keeps each in the 3–8h range per CLAUDE.md §5 single-session budget. Stage A's 6–8h carry-forward absorption was further split into A1 (build hygiene only, ~2–3h) + A2 (production wiring only, ~4–5h) per the scope-split rule in `docs/build-prompts/TEMPLATE.md` — the M03 retro pattern of "carry-forward + new deps" hit ~3h baseline; M04's carry-forward absorption is 2× that scope and warrants the split.

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

| Stage | Summary | Estimated effort |
|---|---|---|
| **A1** | Build hygiene — extend xtask to codegen `event.v1.json` + `error.v1.json` (Rust + TS); regenerate types; close `await_event` `tokio::time::pause()` coverage; verify drone integration test current_exe() paths clean; update CHANGELOG | ~2–3h |
| **A2** | Production wiring — drone subprocess lifecycle at Tauri startup with `Arc<DroneClient>` Tauri-managed-state; replace `DroneClient::noop()` callsites in `query_session_db` + `replay_session`; wire `vdr::project_signal` at `WriteSignal` call-site; replace heuristic decision extractor with structured emitter; implement real `count_tokens` against `/v1/messages/count_tokens`; resolve long-lived `events()` reconnect carry-forward; refactor `unwrapCmdError` to use generated types | ~4–5h |
| **B** | §3a Plan & Task primitive — `plan.v1.json` + `task.v1.json` schemas; xtask codegen extension; Plan/Task Rust types in runtime-core; 11 plan/task events added to `event.v1.json` (regenerate types); plan state machine; `fresh_context_per_task` loop policy; failure escalation; SQLite `plans` + `tasks` tables (per §10 DDL added M03.5); approval-gate primitive; runtime-main + drone consumers | ~4–6h |
| **C** | §3a Plan UI + ApprovalPanel + graph wiring — wire already-shipped `PlanNode` + `TaskNode` (M03.C synthetic) to live event variants; ApprovalPanel renderer + approval-gate flow (renderer→main→drone→main→renderer); plan abort + replan + revise flows | ~3–5h |
| **D** | §4a Verify & Rails — `hook.v1.json` schema (HookRef + HookCategory + Hook); Hook primitive with 7 firing points (existing 6 + new `pre_file_edit`); 3 hook events; Rails primitive (hard/soft + JSON-declared); don't-touch glob matcher; `revert_to_snapshot` drone command + reason enum; VerifyNode + HookNode wired to live events | ~6–8h |
| **E** | §6a HITL — `hitl.v1.json` schema (9 trigger types + 3 UI variants + notifier plugin interface); 3 built-in notifiers (terminal_bell/desktop via Tauri notification plugin/sound); 5 HITL events (3 new); failure-escalation flow `task_escalated` → `on_failure_threshold` → `hitl_requested` → notifiers parallel → 1h timeout; HITL Panel + Modal + Toast renderer surfaces | ~6–8h |
| **F** | §2a Budget + §1b Recovery — `budget.v1.json` schema (3 scopes + 4 actions + downshift_hook); 4 budget events; session header bar UI; Recovery (resume rebuilds history per spec §1b; tool-call-uncertain UI prompt with retry/skip/mark-complete/abort options; MCP reconnect on resume; plan state restoration; capability state restoration) | ~5–7h |
| **G** | Phase Closeout — gap-analysis entry per CLAUDE.md §20 (cumulative product↔spec audit including M04 + cumulative review); `<gotchas_graduation>` v1.2 closeout subsection auditing all per-stage `<gotchas>` from A1–F (kept | graduated | resolved | expired); M03 + M03.5 carry-forward final disposition; M04-summary.md aggregating across stages; three-artifact review (CLAUDE.md §20) | ~2–3h |

Total: ~32–45 hours estimated. ~10–12 hours human direction (eight approval gates + one PR review).

**Estimation calibration.** M01: estimated 29–46h, ran ~9–14h (ratio 0.3×). M02: estimated 13h, ran ~8.8h (ratio 0.7×). M03: estimated 25–31h, ran ~10h (ratio 0.32×). M03.5: estimated 6–8h, ran ~1h (doc-only mini-milestone, ratio 0.14×). M04 actuals likely track between M03's 0.32× ratio (most stages are code-shipping) and M03.5's 0.14× ratio (Stage A1 is doc/codegen-only). Per the user's locked +20% buffer: budget for ~12–17h actual across A1–G. Stage A1's likely ratio: 0.14× (closer to M03.5; ~25–35min actual on a 2–3h estimate). Stages B–F ratio: 0.30×–0.50× (cross-stack glue density adds overhead vs M03's React-only stages). Stage G ratio: 0.20× (doc-only closeout per M03.F).

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

## Stage D — §4a Verify & Rails primitive (hooks + rails + don't-touch + revert_to_snapshot)

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
<!-- Stages E–G to follow in Chunk 3                                -->
<!-- ============================================================ -->

*Stages E (§6a HITL), F (§2a Budget + §1b Recovery), and G (Phase Closeout) authored in Chunk 3 per the chunked-authoring decision (checkpoint after Stage D draft surfaced).*

---

*End of M04 prompt — Chunk 2 (Stages B + C + D appended). Stages E–G + Summary Table + Verification Checklist authored in Chunk 3.*
