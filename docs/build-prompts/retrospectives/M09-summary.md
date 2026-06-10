# M09 — Parent-Milestone Summary

> **Parent milestone:** M09 of M11 in `docs/MVP-v0.1.md` (the first ADR-0032 vertical slice)
> **Authored by:** Claude (per `CLAUDE.md` §19)
> **Aggregates:** M09.A, M09.B, M09.C, M09.D (+ D.fix iter1 + D.fix iter2) stage retrospectives + M09.V
> **Created at:** 2026-06-09 (build-machine local time)
> **Total elapsed:** ~7.5 h across stages (A 0.6 + B 0.6 + C 0.9 + D 0.8 + D.fix iter1 2.4 + D.fix iter2 ~2.2)
> **Estimated:** ~9 h (the four-stage prompt estimate; A/B/C/D at ~1.5 h each + a ~3 h D.fix allowance)

---

## Stage trail

| Stage | Status | Stage commit | Retrospective | Outcome |
|---|---|---|---|---|
| Stage A — blank-create an agent | Committed | `fe04140` | `M09.A-retrospective.md` | Sound |
| Stage B — file_access editor | Committed | `4f93a5e` | `M09.B-retrospective.md` | Sound |
| Stage C — attach a real MCP tool | Committed | `ab5957f` | `M09.C-retrospective.md` | Sound |
| Stage D — assembled vertical-slice IRL | Committed | `7ae11ea` | `M09.D-retrospective.md` | Sound (pending IRL → closed at D.fix iter2) |
| Stage D.fix iter1 — surface the MCP tool to the model | Committed | `15d4add` (+ UI `d65bb38`/`aecec03`) | `M09.D.fix-retrospective.md` | Reopened → iteration 2 |
| Stage D.fix iter2 — wire the MCP dispatcher's enforcer | Committed | `c76befd` (+ UI `784d367`/`95e9867`/`78f3fe1`) | `M09.D.fix2-retrospective.md` | Sound (re-IRL passed 2026-06-09) |
| Stage V — five-pass real-app verifier | Committed | `32a33a1` | `M09.V-retrospective.md` | Sound (0🔴 / 0🟡 / 2🟢) |
| execution-status flip | Committed | `73d2f3a` | — | the slice row flipped on the re-IRL |

All stages on parent-milestone feature branch `claude/m09-prep`. The M09 PR drafts after this summary + the gap-analysis entry land and surfaces all stage commits + retrospectives + this summary together.

---

## Aggregate scoring (sum across stages)

> **Scale note.** A/B/C scored the Pattern axis on the older `/35` rubric; D, D.fix iter1, and D.fix iter2 scored it on the `/40` rubric (an extra pattern item). Values are reported as scored; the per-axis mean below is computed within each scale and not cross-normalized.

### Process axis (/40)

| Stage | Total | /40 |
|---|---|---|
| Stage A | 40 | /40 |
| Stage B | 40 | /40 |
| Stage C | 40 | /40 |
| Stage D | 40 | /40 |
| Stage D.fix iter1 | 39 | /40 |
| Stage D.fix iter2 | 40 | /40 |
| **Mean** | **39.8** | /40 |

### Product axis (/40)

| Stage | Total | /40 |
|---|---|---|
| Stage A | 39 | /40 |
| Stage B | 39 | /40 |
| Stage C | 39 | /40 |
| Stage D | 39 | /40 |
| Stage D.fix iter1 | 39 | /40 |
| Stage D.fix iter2 | 39 | /40 |
| **Mean** | **39.0** | /40 |

### Pattern axis

| Stage | Total | Scale |
|---|---|---|
| Stage A | 33 | /35 |
| Stage B | 33 | /35 |
| Stage C | 33 | /35 |
| Stage D | 38 | /40 |
| Stage D.fix iter1 | 37 | /40 |
| Stage D.fix iter2 | 37 | /40 |
| **Mean (A/B/C)** | **33.0** | /35 (94%) |
| **Mean (D/D.fix×2)** | **37.3** | /40 (93%) |

The single recurring product-axis deduction across every stage is the same one (CI-matrix item 8: the Linux/macOS legs run on push, not locally — Windows is the build machine). Otherwise the work scored at or near ceiling.

---

## Cross-stage trends

### Friction patterns that recurred

- **The red-phase "lint the new test file first" gap, four stages running.** A (tsc cast TS2352, `683ce48`), B (prettier printWidth wrap, `be71df9`), C (cargo fmt re-wrap + tsc retype, `acd9a72`), D (full-program `npx eslint .` `no-unnecessary-type-assertion`, `92adc1a`) — each time a test-file-only mechanical lint surfaced only at the post-impl gate, fixed in a separate labelled fixup commit to keep the strict red→impl test diff EMPTY. The standing protocol nudge ("run `tsc --noEmit` / `prettier --check` / `cargo fmt` / the **full** `npx eslint .` on new test files before the red commit") is now confirmed across four consecutive stages and should graduate to a `<pre_red_mechanical_sweep>` step for code stages (gotcha candidate; see Decisions below).
- **Existing test fixtures break when a new required field/editor is added.** B (the pre-existing `NodeConfigPanel.test.tsx` fixture omitted `capabilities`, crashing 5 render tests once the editor rendered — `17355c4`); D.fix (default-collapsing the budget section unmounted `budget-cap-input`, breaking 5 vitest + 1 Playwright — `79f6fc2`). Both fixed as separate labelled test fixups. The `<construction_reachability_check>` could pre-flag "existing render-test fixtures will need updating."

### Pattern-level wins

- **"The substrate already executes; the gap is authoring."** The phase-doc thesis held: A/B/C were Palette/store/one-read-only-command changes; the dispatcher, tracked tier, and capabilities-granting were **already wired** in `test_framework` (`commands.rs:1769-1775`/`:1758`; `tester.rs:445`). M09 added **no execution-wiring under expectation** — exactly as designed.
- **Strict v1.11 two-commit TDD held on every code stage** (red→impl test-path diff EMPTY, including the binary-crate `#[cfg(test)]` byte-identical variant at C/D.fix). V confirmed it independently.
- **Hard Rule 8 caught a misdiagnosis pre-impl.** The D.fix iter1 re-IRL exposed a denial the phase doc had misdiagnosed ("the seam drops the tier"). The build escalated rather than implementing the wrong fix; the maintainer re-scoped to the real cause (the dispatcher's bare `CapabilityEnforcer::new()`), so the ADR-0008 two-iteration cap was **not** burned on a non-fix. This is the §4-rule-8 / §12 escalation path working as intended.

### Surprises across the parent milestone

- **The vertical slice took two D.fix iterations — and both were real, distinct execution holes the green tests never reached.** Iter1: the canvas-authored MCP tool was *dispatch-wired* but its *definition was never injected into the model's tool list*, so the model never emitted the call (`vertical_slice.e2e.ts` ran tool-free; `builder_mcp_tool.e2e.ts` was store-driven — neither hit the real-model-meets-real-MCP-tool path). Iter2: even injected + called, the `McpDispatcher`'s **own** enforcer was a bare `CapabilityEnforcer::new()` (default-Novice, no grants), denying the authored tool on L4 (tier) then L1 (grant-class) at any user tier. Both were found **only** by the maintainer real-app IRL — the precise reason rule 11 makes the IRL, not the green e2e, the authoritative close.
- **The "MCP dispatch executes" ledger claim was structurally true but behaviorally false end-to-end** until M09 — only the no-tools smoke had ever driven the dispatcher. M09's slice is the **first enforced-and-dispatched real MCP tool**; `docs/execution-status.md` was corrected accordingly (iter2).

### Hard gate violations across the milestone

- **No hard gate was bypassed.** G4 (real-app IRL observed) intentionally stayed open at D and D.fix iter1 (rule 11 — the flip waits on the maintainer observation, never tests-green), and closed at the 2026-06-09 re-IRL after iter2. The execution-status row flipped on that observation, not on green. No `--no-verify`, no skipped gate, no auto-commit.

---

## Time-box accuracy

| Stage | Estimated | Actual | Ratio |
|---|---|---|---|
| Stage A | 1.5 h | 0.6 h | 0.40 |
| Stage B | 1.5 h | 0.6 h | 0.40 |
| Stage C | 1.5 h | 0.9 h | 0.60 |
| Stage D | 1.5 h | 0.8 h | 0.53 |
| Stage D.fix iter1 | ~2 h | 2.4 h | 1.20 |
| Stage D.fix iter2 | ~2 h | ~2.2 h | 1.10 |
| **Total** | ~10 h | ~7.5 h | 0.75 |

A/B/C/D ran well under estimate (the authoring surfaces were genuinely small — the substrate carried the weight). The two D.fix iterations ran at/over estimate, which is the honest signature of "the real run path had two holes the green tests didn't reach" — diagnostic + escalation + a mutation-blocked enforcement change cost more than a clean stage. Total ratio 0.75 is within band; the estimation method is sound for authoring stages and slightly optimistic for assembled-IRL close stages that surface real composition gaps.

---

## Decisions to apply before the next parent milestone

### Coverage-policy reconciliation (mandatory check — per `CLAUDE.md` §6)

- [x] Coverage thresholds/exclusions: **unchanged this milestone** — nothing to reconcile. No `--ignore-filename-regex` value and no `--fail-under-lines` threshold changed anywhere in M09. The only gated `crates/**` touches (`runtime-main/src/builder/tester.rs` + `mod.rs` injection; `runtime-mcp/src/client/connection_resolver.rs`; the `src-tauri/src/commands.rs` enforcer/command) landed **inside existing package gates** — `runtime-main` 96.32% ≥ 95, `runtime-mcp` 96.00% ≥ 95, workspace 91.88% ≥ 80, Vitest `src/` 93.4% ≥ 80 (measured at D.fix iter2). The new `mcp_list_server_tools` command is a thin read-only wrapper over a `list_server_tools(name)` seam in `runtime-mcp` (`connection_resolver.rs`) — the seam carries the unit coverage; the `src-tauri` command is covered by the existing `tauri-shell` 50% patch gate. The seam-vs-wrapper split holds; no exclusion needed. A §C M09 entry is appended to `docs/coverage-policy.md`; **no new §B baseline** (no module entered a gate). The four canonical mirrors (CLAUDE.md §5 category list, §6 commands, `codecov.yml`, coverage-policy §A) are unchanged and verified byte-consistent.

### `CLAUDE.md` updates carrying forward

- **None applied this milestone.** Candidate (recurring, not yet written down): graduate the "lint the new test file before the red commit" nudge into the protocol as a `<pre_red_mechanical_sweep>` for code stages — confirmed across A/B/C/D + D.fix (five stages). Surfaced as a gotcha; defer the protocol edit to a focused `docs(stage-prompt-protocol)` pass.

### `TEMPLATE.md` / `STAGE-PROMPT-PROTOCOL.md` updates carrying forward

- The `<construction_reachability_check>` could add a sub-prompt: "name existing render-test fixtures / tests that will break when this stage adds a required field or default-collapses a section." Recurred at B and D.fix. Non-blocking.

### M10 stage prompts — known constraints to encode

- **M10 owns TD-059** (the gap-suspend-reads-PASS truthfulness gap — `tester.rs::fold_outcome` has no suspend arm) — already routed via the M09.D.fix `<scope_locks>` and now ledgered. M10's gap resolve→resume slice should derive a distinct `Suspended` verdict/state.
- **M10 inherits the budget-visible surface** (ADR-0032 — HITL steers) and plan-approval + plan task execution.

### Open issues filed

- None opened as GitHub issues. Tracked in `docs/tech-debt.md`: the two Stage-V 🟢 findings (**TD-058** weak `vertical_slice.e2e.ts` run-assertion; **TD-059** gap-suspend reads PASS → M10) + four **maintainer real-app IRL UX findings** the walkthrough surfaced (**TD-060** MCP "Add server" modal doesn't reflect a just-created server → M13; **TD-061** stale error-toast → M10; **TD-062** MCP palette label-wrap → M13; **TD-063** no canvas node-delete/undo → a future ADR-0032 authoring slice / M11). All six are in the immutable M09 gap-analysis fix-backlog with routing; none blocked the slice's functional close.

---

## Verdict

- [x] **Pattern held across M09.** Proceed to M10 with the protocol notes above queued (none blocking). Confidence in the prompt-driven approach: **high.** The milestone delivered the first canvas-authored real run, every behavioral claim is grounded by the maintainer real-app IRL (rule 11), the two D.fix iterations were both real execution holes the green tests didn't reach (and were closed honestly, with the ledger corrected), and Stage V (fresh-context, bias-guarded) produced zero 🔴/🟡 and two correctly-classified 🟢 (one in-design, one correctly-deferred-to-M10).

---

## User-review notes

> User reviews this summary as part of the M09 PR. Approval here gates M10.

User-review notes:

- [Empty until user reviews]

---

## Sign-off

**Claude:** This summary aggregates the per-stage retrospectives for M09 (A blank-create → B file_access editor → C attach a real MCP tool → D the assembled vertical-slice IRL, across D + D.fix iter1 + iter2 → V). It is my honest assessment of how the first ADR-0032 vertical slice went. M09 made the workbench **build and run** a real single-agent, MCP-data workflow from scratch: a from-scratch canvas agent, granted a `file_access.write` scope, with an installed MCP server's tool attached, runs in the Tester at the enforced tier and writes a real file from real MCP data within scope — denied outside it. The substrate already executed; the gap was authoring (three small surfaces) plus one read-only command, plus two real run-path holes the IRL surfaced and the D.fix iterations closed. No schema change; no coverage gate change; no ADR filed or flipped (ADR-0032 was accepted in the re-cut PR; M09 implements it). User review and approval pending. M10 (HITL steers — gap resolve→resume + plan-approval + plan task execution) does not begin until this summary is approved.

**Surfaced at:** 2026-06-09 (build-machine local time).
