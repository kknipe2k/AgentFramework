<!--
Per-parent-milestone summary template. Claude creates this at the
end of the FINAL stage of a parent milestone (per CLAUDE.md §19).
The user reviews alongside the M02 PR description.
-->

# M02 — Parent-Milestone Summary

> **Parent milestone:** M02 of M11 in `docs/MVP-v0.1.md`
> **Authored by:** Claude (per `CLAUDE.md` §19)
> **Aggregates:** M02.A, M02.B, M02.C, M02.D, M02.E stage retrospectives
> **Created at:** 2026-05-04 (UTC)
> **Total elapsed:** ~8.8 h (sum of stage sessions)
> **Estimated:** ~13 h calibrated (per `M02-event-pipeline.md` Summary Table); MVP-v0.1.md M02 budget was 30–40 human-equivalent hours

---

## Stage trail

| Stage | Status | Stage commit | Retrospective | Outcome |
|---|---|---|---|---|
| Stage A | Committed | `85a711b` | `M02.A-retrospective.md` | Sound |
| Stage B | Committed | `3e84125` | `M02.B-retrospective.md` | Sound |
| Stage C | Committed | `1fa0c53` | `M02.C-retrospective.md` | Sound |
| Stage D | Committed | `3b29268` | `M02.D-retrospective.md` | Sound |
| Stage E | Committed | `4bd809a` | `M02.E-retrospective.md` | Sound but rough |

All stages on parent-milestone feature branch `claude/m02-event-pipeline`. The M02 PR drafts after this summary lands and surfaces all stage commits + retrospectives + this summary + the M02 gap-analysis entry together.

---

## Aggregate scoring (across stages A–E)

### Process axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | 39 | /40 |
| Stage B | 39 | /40 |
| Stage C | 39 | /40 |
| Stage D | 39 | /40 |
| Stage E | 37 | /40 |
| **Mean** | **38.6** | /40 |

### Product axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | 39 | /40 |
| Stage B | 40 | /40 |
| Stage C | 40 | /40 |
| Stage D | 40 | /40 |
| Stage E | 38 | /40 |
| **Mean** | **39.4** | /40 |

### Pattern axis

| Stage | Total | /35 |
|---|---|---|
| Stage A | 29 | /35 |
| Stage B | 29 | /35 |
| Stage C | 30 | /35 |
| Stage D | 30 | /35 |
| Stage E | 28 | /35 |
| **Mean** | **29.2** | /35 |

All five stages cleared the soft-gate floors (Process ≥30, Product ≥32, Pattern ≥25). All cleared every hard gate (G1 do-not-commit-until-approved, G2 no Sev-5 friction, G3 no protocol drift, G4 stage completion, G5 no axis row below 3).

---

## Cross-stage trends

### Friction patterns that recurred

- **Pedantic+nursery clippy "first-pass tax" on every new module.** A → 3 lints; B → 8 lints; C → 5 lints; D → 8 lints; E → 5 lints + serde tagged-newtype rejection. Each one self-corrected within one round, but the cumulative cost is real. The patterns are now well-known: `redundant_pub_crate` (private-module visibility), `derive_partial_eq_without_eq` (`serde_json::Value` containment), `cast_precision_loss`/`suboptimal_flops` (token-count arithmetic), `struct_excessive_bools` (capability flags), `missing_const_for_fn` (two-line ctors), `unnecessary_literal_bound` (trait `&str` returns), `doc_markdown` (type names in plain prose), `unused_async` (cross-platform cfg variants), `default_trait_access` (`Default::default()` calls), `match_wildcard_for_single_variants`. Stages C/D/E each promoted one or more to the "extended pedantic-pass checklist" carry-forward. **Decision:** consolidate into a single CLAUDE.md §15 / `docs/gotchas.md` entry before M03 so the pattern doesn't re-discover itself stage-by-stage.

- **Prompt-snippet drift on fast-moving tooling.** B (reqwest 0.12→0.13 feature rename `rustls-tls` → `rustls`); E (Tauri 2.x E2E pattern was Electron-shaped; ESLint 9 default flat-config; happy-dom 15.x critical CVE; Vite project-root index.html; serde tag-shape rejection on newtype variants). The CLAUDE.md §12 web-first rule held — both stages caught the drift before code shipped — but the prompt body had stale information. **Decision:** `TEMPLATE.md` should grow a `WEBCHECK:` header listing URLs to verify before every stage that touches a fast-moving tooling surface.

- **Coverage delta and `*_with` test-seam scaffolding.** The pattern from M01.C was promoted to documented guidance in `docs/style.md` at Stage A and applied verbatim in C, D, and E. Coverage outcomes confirm the shape: M02.C `anthropic_sse.rs` 98.33%; M02.D `agent_sdk.rs` 99.42% / `event_pipeline.rs` 100% / `decision_extractor.rs` 100% / `drone_ipc/client.rs` 100%; M02.E command surface participates via `run_smoke_session_with`. The pattern is now demonstrated FOUR times across two distinct I/O substrates (named pipes, HTTP+SSE, in-process streams, Tauri command bodies). **Decision:** `TEMPLATE.md` should default the test plan for any stage adding a new safety primitive to the dual-coverage shape (N unit tests for the testable seam + M integration tests for end-to-end behavior).

### Pattern-level wins

- **`*_with` test-seam pattern is robust cross-substrate.** Drone (M01.C — Unix sockets / Windows named pipes), provider HTTP+SSE (M02.C — wiremock-fed reqwest body), AgentSdk (M02.D — `Stream<ProviderEvent>` injected), drone IPC client (M02.D — `tokio::io::duplex` halves), Tauri command surface (M02.E — `LLMProvider` + `mpsc::Sender` injected). The pattern's exclusion-list discipline in CLAUDE.md §5 is mature: every excluded module names a structurally infeasible OS-call wrapper, and CI's `--ignore-filename-regex` enforces consistency.

- **Schemas-as-source-of-truth held for Rust types.** Adding `mcp_servers` to the SQLite schema in M02.A (closing the M01.B gap) plus extending `AgentEvent` with `ToolSource` + `AgentSpawned.session_id` in M02.D were both done as additive schema changes; the typify-driven generation pipeline picked them up cleanly. Frontend types remain hand-mirrored (M03 codegen target — see Decisions).

- **Coverage gates raised without regression.** runtime-drone safety primitive 96.84% across all five stages (per-module baselines preserved: snapshot 100%, db 99.30%, heartbeat 98.88%, command_handler 98.13%, ipc 86.89%). runtime-main safety primitive activated at Stage C (95.47%), grew to 99.37% at Stage D, held at 99.37% at Stage E. Workspace coverage moved from M01's 96.17% → 94.51% at M02.E (the slight drop reflects the `key_store.rs` keychain-call exclusion + the larger non-safety surface added by the renderer).

- **Do-not-commit-until-approved held perfectly.** All five stages reached "surface only" at session end; user explicitly approved each before commit. Five hard-gate G1 evaluations, five passes.

### Surprises across the parent milestone

- **`signature_delta` (Anthropic thinking signature) is a real wire event** that should be parsed and silently dropped — covered by M02.C's state machine. Spec §2c does not document this; flagged in the gap-analysis Spec Review for the post-M02 `docs(spec):` PR.

- **Tauri 2.x's E2E ecosystem (`tauri-driver` + WebdriverIO) is in deep conflict with Playwright `_electron`-style tests** and unsupported on macOS. The M02 prompt was authored before the ecosystem stabilized; Stage E swapped to renderer-level Playwright + Vitest App-state-machine tests, deferring full desktop-shell E2E to M03. This is the single largest M03-blocking carry-forward.

- **Coverage workspace gate exposed a cross-package subprocess-spawn bug** (M02.D `drone_ipc_loopback.rs` hardcoded `target/debug/runtime-drone.exe` but `cargo llvm-cov --workspace` uses `target/llvm-cov-target/`). Fix: derive the binary path from `std::env::current_exe()`. Generalizable pattern that should also retrofit `crates/runtime-drone/tests/integration*.rs` before M03.

- **vitest test-ordering flake** (App.test.tsx mock-state pollution) caught a real Vitest+RTL idiom: capturing a button's DOM ref before awaited re-render gives a stale view. The fix (sync on a visible-in-DOM indicator before re-querying) is now codified in the M02.E retro Decisions section and should land in CLAUDE.md §15.

### Hard gate violations across the milestone

- **None.** All five stages cleared G1–G5. The Stage E "Sound but rough" verdict reflects soft-gate ratings on Pattern Axis #6 + #7 (fresh-session reproducibility + carry-forward tax to M03), not a hard-gate failure.

---

## Time-box accuracy

| Stage | Estimated | Actual | Ratio |
|---|---|---|---|
| Stage A | 1.5–3 h | 1.9 h | 0.6×–1.3× |
| Stage B | 1.5–3 h | 0.6 h | 0.2×–0.4× |
| Stage C | 3–4 h | 0.9 h | 0.2×–0.3× |
| Stage D | 2.5–4 h | 2.4 h | 0.6×–1.0× |
| Stage E | 3 h | 3.0 h | 1.0× |
| **Total** | ~13 h calibrated | ~8.8 h | **~0.7×** |

Total ratio 0.7× — under the calibrated estimate. M01-summary's 0.3× recalibration applied: M01 ran 10.5h actual vs 30–40h human-equivalent estimate (~0.3×); M02 ran 8.8h actual vs the 13h M02-prompt calibrated estimate (~0.7×). The 0.3× and 0.7× numbers aren't comparable directly — M01 used the human-equivalent estimate, M02 used the agent-time calibrated estimate. The post-M01 calibration (multiply human-time-equivalent by ~0.3×) held for the M02 deliverable's complexity. Stages B + C came in at 0.2×–0.3× of *their* calibrated estimates, suggesting the per-stage estimates can be tightened further when the work is type-surface authoring (B) or wire-format-tracking (C) rather than novel architecture.

**Correction for next parent milestone:** keep the human-time-to-agent-time 0.3× ratio as the calibration anchor. For per-stage estimates within an agent-time budget, "type-surface authoring" stages can be estimated at 0.5–1 h actual; "novel architecture" stages at 2–3 h actual; "frontend-tooling-touching" stages at 3 h actual (E was 1.0× of estimate, which matches the friction profile).

---

## Decisions to apply before the next parent milestone

The following are the cumulative carry-forwards from M02.A–E `[END] Decisions` sections. Each is owner-tagged and target-milestone-tagged. The Stage F gap-analysis entry classifies these into the Fix Backlog by severity.

### `CLAUDE.md` updates carrying forward

- **§15 / `docs/gotchas.md`:** Consolidate the recurring clippy pedantic+nursery patterns into a single "common clippy traps" subsection. Cover: `redundant_pub_crate` (use plain `pub` in private mods), `derive_partial_eq_without_eq` (`serde_json::Value` containment requires `#[allow]` with rationale), `unused_async` (cross-platform cfg variants where one branch awaits and the other doesn't), `default_trait_access` (use `HashMap::default()` over `Default::default()`), `match_wildcard_for_single_variants` (bind explicitly), `cast_precision_loss`/`suboptimal_flops`/`struct_excessive_bools`/`missing_const_for_fn`/`unnecessary_literal_bound`/`doc_markdown` patterns. (Sources: A friction r1, B friction r1, C friction r1, D friction r1, E friction r1+r2.)

- **§15:** "Subprocess-spawning integration tests must derive paths from `std::env::current_exe()`, never `target/debug/<binary>` literals — `cargo llvm-cov --workspace` uses a distinct target dir." Cite `crates/runtime-main/tests/drone_ipc_loopback.rs::drone_binary` as the archetype. Retrofit `crates/runtime-drone/tests/integration*.rs` before M03. (Source: M02.D Decisions §`CLAUDE.md` updates.)

- **§15:** "Tauri 2.x E2E uses `tauri-driver` + WebdriverIO per official docs; Playwright `_electron` is Electron-specific and won't drive a Tauri 2.x window. macOS unsupported. M02.E ships renderer-level Playwright against the Vite dev server; full desktop-shell E2E is M03 carry-forward." (Source: M02.E friction r1.)

- **§15:** "ESLint 9 default is flat config (`eslint.config.js`); legacy `.eslintrc.cjs` requires `ESLINT_USE_FLAT_CONFIG=false`." (Source: M02.E friction r3.)

- **§15:** "Vite's project root contains `index.html`, not `src/index.html`. Script tag references `/src/main.tsx` (absolute path under root)." (Source: M02.E friction r9.)

- **§15:** "`serde(tag = \"type\")` requires struct-shape variants, not newtype-wrapped primitives. `Provider(String)` errors at runtime; `Provider { message: String }` works." (Source: M02.E friction r7.)

- **§15:** "Vitest+RTL: sync on a visible-in-DOM indicator (`findByLabelText`) before re-querying dependent UI; capturing a button's DOM ref BEFORE awaited re-render leaves it stale even when `waitFor(.toBeEnabled())` passes." Cite `tests/unit/App.test.tsx::save_key_then_run_smoke_renders_event_list` as the archetype. (Source: M02.E friction r8.)

- **§15:** "Test fixtures using `futures::stream::repeat` against a stateful pipeline (in-memory accumulator) will OOM. Use `take(N)` or `iter([...]).chain(pending())` to bound the stream while still triggering the cancellation timeout." Cite `tests/sdk_cancellation.rs::drops_mid_text_burst_no_panic` as the archetype (~34 GB allocated before harness aborted in M02.D r1). (Source: M02.D friction r1.)

- **§5 (already partially landed at M02.A):** runtime-main safety primitive baseline now codified at 99.37% with three exclusions documented (`providers/anthropic.rs` reqwest+SSE wrapper; `drone_ipc/connection.rs::open` cfg-platform OS call; `key_store.rs` keyring-call wrapper). Stage F preserves this in the Adherence-to-spec section.

### `TEMPLATE.md` updates carrying forward

- **Add a "Coverage holdouts" subsection** to the per-stage retrospective template so per-module baselines + new exclusions are recorded in one place rather than scattered across CLAUDE.md §5 + per-stage [END] Decisions. (Source: A Decisions, C Decisions, D Decisions.)

- **Default test plan for stages adding a new safety primitive**: "(N) unit tests for the testable seam (`*_with` / `from_streams`) + (M) integration tests for end-to-end behavior." Pattern proven across M01.C / M02.A / M02.C / M02.D / M02.E. (Source: D Decisions, E Decisions.)

- **Add a `WEBCHECK:` header** to milestone-prompt template for stages that touch fast-moving tooling surfaces (Tauri, Vite, ESLint, Vitest, Playwright, npm ecosystem, reqwest, keyring). Lists URLs to verify against the prompt body before code. Per CLAUDE.md §12 web-first rule. (Source: M02.E surprise event 1, recurring pattern.)

- **Add a "Pre-existing legacy file inventory"** subsection to milestone prompts so fresh sessions catch tracked-but-orphaned files (like `src/counter.{js,test.js}`) before they trip prettier / eslint. (Source: M02.E friction r5.)

- **Add an extended pedantic-pass checklist** preface to test plans for new modules (consolidates A/B/C/D/E patterns above).

### M03 stage prompts — known constraints to encode

- **M03 prompt MUST pin Tauri 2.x desktop-shell E2E** as `tauri-driver` + WebdriverIO, matrix Linux + Windows (no macOS — unsupported by tauri-driver). The four `test.skip()`-with-rationale Playwright tests in `tests/e2e/smoke.spec.ts` are the carry-forward set. (Source: M02.E friction r1, M02.E Decisions §M03.)

- **M03 prompt should add `event.v1.json` schema + `cargo xtask regenerate-types` for TS types** so `src/types/agent_event.ts` ceases to be hand-mirrored. The hand-mirrored shape held for M02 because schema is currently stable; the `ToolSource` + `AgentSpawned.session_id` additions in M02.D would have silently drifted the TS side under any pressure. Per CLAUDE.md §14. (Source: M02.E surprise event 5, M02.E Decisions §"Was the IPC TypeScript type sync a source of bugs?")

- **M03 prompt should evaluate Vite 5 → 7 bump** for the dev-server esbuild CVE (currently `npm audit --audit-level=high` filters out the moderate vulns; security fix lands in vite 6+/8+). (Source: M02.E surprise event 4.)

- **M03 prompt should re-evaluate `keyring 3.6 → 4.0` upgrade** when the multi-platform CI matrix exercises real keychain calls. M02 stayed on 3.6 deliberately. (Source: M02.B Decisions, M02.D Decisions, M02.E ambiguity event.)

- **M03 prompt should re-evaluate the `secrecy/serde` workspace feature** — currently advertised but unused at every M02 stage. If M03 doesn't use it either, drop in a `chore(workspace):` PR. (Source: M02.B/C/D/E Decisions.)

- **M03 prompt should embed UI consistency carry-forward** (Pre-M01 addendum) — all M03 modals/screens reuse existing component patterns and visual language; no per-feature re-skinning. (Source: Pre-M01 addendum carry-forward via M01 entry.)

- **M03 prompt must enable `vitest --coverage` by default** in the `test` script so the `vitest.config.ts` 80% threshold is enforced (currently the threshold is configured but only triggers when `--coverage` is passed). (Source: M02.E Decisions §"Frontend coverage 80% threshold".)

- **M03 prompt should include the M02 carry-forward delete** of `src/counter.{js,test.js}` (legacy CommonJS files conflicting with the new `"type": "module"`; currently in `.prettierignore` + `eslint.config.js ignores`). (Source: M02.E friction r5.)

- **M03 prompt should reference the existing `tests/integration*.rs` fixture pattern** for any new subprocess-spawning tests (current_exe-derived paths, not `cargo run`). (Source: M02.D ambiguity event 2.)

### Open issues filed

None during M02. All carry-forwards are tracked here, in the per-stage retros, and in the M02 gap-analysis entry. No GitHub issues opened (per CLAUDE.md §12 "do it autonomously" — the milestone-prompt + retro + gap-analysis trio is the issue tracker).

---

## Verdict

Mark one:

- [ ] **Pattern held across M02.** Proceed to M03.1 with the protocol updates above applied. Confidence in the prompt-driven approach: high.
- [x] **Pattern held but with friction.** Apply soft-gate fixes from stage retrospectives before M03.1. Confidence: medium.
- [ ] **Pattern strained.** A hard gate failed in one or more stages; or aggregate scores indicate sustained pattern-level friction. Spend a session iterating on `CLAUDE.md` / `TEMPLATE.md` BEFORE M03.1. Confidence: low until protocol is updated.

**Why "held but with friction":** All 5 stages cleared every hard gate. Aggregate scores are healthy (Process 38.6/40, Product 39.4/40, Pattern 29.2/35). But Stage E specifically scored "Sound but rough" with 8 Sev-2-or-3 prompt-drift items (Tauri 2.x E2E framework, ESLint flat-config, happy-dom version, Vite root convention, serde struct-shape, vitest mock-ordering, prettier scope, index.html location), several of which a less-experienced fresh session could have missed. The Stage E retrospective itself flagged Process Axis #2 = 3 (Read-first list correctly orienting Claude) and Pattern #6/#7 = 3 (fresh-session reproducibility, carry-forward tax to M03).

The protocol-iteration session before M03.1 should land:

1. The CLAUDE.md §15 / `docs/gotchas.md` consolidation (clippy traps, Tauri 2.x E2E, ESLint flat-config, Vite root, serde tag-shape, Vitest+RTL idiom, subprocess fixtures, OOM-bound test fixtures).
2. The `TEMPLATE.md` updates (Coverage holdouts subsection, default safety-primitive test plan, WEBCHECK: header, legacy-file inventory).
3. The M03 prompt itself (tauri-driver + WebdriverIO + Linux+Windows matrix; event.v1.json schema + codegen; Vite/keyring/secrecy/keyring-3→4 re-evaluation; vitest --coverage default; UI consistency carry-forward; counter-file deletion).

These are mechanical edits informed by the M02 retros + this summary + the M02 gap-analysis entry; they should fit in a single short session before M03.1 opens.

---

## User-review notes

> User reviews this summary as part of the final stage's PR. Approval here gates M03.

User-review notes:

- [Empty until user reviews]

---

## Sign-off

**Claude:** This summary aggregates the per-stage retrospectives for M02. It is my honest assessment of how the parent milestone went and what the protocol should carry forward. Aggregate scores are healthy and all hard gates passed; Stage E's prompt-drift items justify a "held but with friction" verdict and a short protocol-iteration session before M03.1 opens. User review and approval pending. The next parent milestone (M03) does not begin until this summary is approved.

**Surfaced at:** 2026-05-04 (UTC).
