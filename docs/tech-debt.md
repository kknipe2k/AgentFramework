# Technical Debt — Append-Only Ledger

> 🟢 findings from Stage V verifier runs (per ADR-0008). Distinct from `docs/gap-analysis.md` (product↔spec drift) and `docs/gotchas.md` (don't-do-this patterns). Tech debt is "noted, not blocking" — code that works correctly per the spec but is structurally fragile, awkward to extend, or known-future-rework. Append-only: existing entries never edited or reordered.

---

## Format

Each entry is one section. Required fields:

```markdown
## TD-NNN — <short title>

**Date logged:** YYYY-MM-DD
**Found by:** Stage V verifier run M[NN].V (or "manual review", "post-merge IRL test", etc.)
**Pass that surfaced it:** Inventory | Wire | Behavior | Multi-call | (N/A if manual)
**Category:** structural | cosmetic | scalability | extensibility | observability | other
**Resolution status:** open | in-flight (PR #N) | resolved (commit hash) | superseded (TD-MMM)

### Description

One paragraph. Concrete: file paths, line numbers, the structural shape of the debt.

### Why it's debt not bug

What works correctly today. The spec is satisfied; code is functional. The debt is structural — what's hard to do next, not what's broken now.

### Recommended approach (when addressed)

One paragraph. Concrete: which files would change, estimated complexity, dependencies (if any).
```

## Numbering

`TD-001`, `TD-002`, … sequentially. Never reuse. Resolved entries stay; their `Resolution status` field is updated to `resolved (<commit hash>)` in a NEW entry that supersedes — the original entry text never changes.

## Append-only enforcement

CI gate (planned): a diff check parallel to `docs/gap-analysis.md` append-only verification. Until that gate ships, append-only is a code-review discipline.

---

## Entries

<!--
Initial seed: this ledger ships with ADR-0008. First entries land when M05.V runs (the first milestone shipped under v1.5 protocol with Stage V active). Until then, "None observed." is honest.
-->

## TD-001 — Viewport-width fix (M04 LG-04) unpinned by any test

**Date logged:** 2026-05-12
**Found by:** Stage V verifier run M04.V (finding #3)
**Pass that surfaced it:** Behavior
**Category:** observability (test-coverage gap: viewport-dependent CSS)
**Resolution status:** open

### Description

`src/styles.css:38` removed the `max-width: 720px` constraint from `main` per M04 IRL LG-04 ("View screen too small — needs to fill width of window"). The fix is in production code and visually self-evident on launch. No unit or integration test asserts (a) `main` lacks a narrow `max-width`, or (b) the graph canvas occupies expected pixel range at viewport widths ≥1280px. Vitest + jsdom doesn't simulate window width; the proper harness is Playwright at multiple viewport configs.

### Why it's debt not bug

The code is correct. The bug is fixed. Risk: a future CSS refactor could silently re-introduce a narrow `max-width` (or any other layout constraint) without any test catching it. Gotcha #70 codifies the pattern ("viewport / window-size assumptions in CSS don't show up in unit tests"), but the pattern lacks a structural guard.

### Recommended approach (when addressed)

Two options:
1. **Static-CSS-grep test** mirror of `every_status_class_has_a_corresponding_CSS_rule_in_styles_css` (in `tests/unit/components/BudgetHeaderBar.test.tsx`): read `src/styles.css` at test time, parse the `main` rule, assert no `max-width` below `1200px`. ~10 min.
2. **Playwright viewport-sweep** at ≥1280px asserting the graph canvas occupies the expected pixel range. Higher fidelity, ~30 min, depends on Playwright infrastructure being current (the M03 carry-forward `tauri-driver` E2E remains gated).

Default to (1) for v0.1; revisit (2) when M05+ adds Playwright suite expansions or M08 Builder Canvas surfaces.

## TD-002 — `read_signals` + `recover_session` lack per-method twice-in-sequence tests

**Date logged:** 2026-05-12
**Found by:** Stage V verifier run M04.V (finding #4)
**Pass that surfaced it:** Multi-call invariants
**Category:** observability (test-coverage gap: per-method multi-call invariants)
**Resolution status:** open

### Description

`crates/runtime-main/src/drone_ipc/client.rs::query_session_db_succeeds_twice_in_sequence` (PR #64 regression test) pins the multi-call invariant for one of three drone-IPC read methods. The other two methods that invoke `next_event` under the hood — `read_signals` (cold-start replay) and `recover_session` (RecoveryDialog) — lack analogous per-method tests. The underlying invariant is structurally pinned by `connection::next_event_returns_consecutive_events_without_consuming_reader`, so all three methods share the fix; the gap is belt-and-suspenders coverage, not a regression risk.

### Why it's debt not bug

The connection-level test proves the borrow-not-move semantics hold for ANY caller. Adding per-method twice-in-sequence tests is defense-in-depth: if a future refactor breaks the wrapper layer (e.g., `read_signals` introduces a new code path that bypasses `next_event`), the per-method test catches it before the connection-level test does. Today: the three methods compose `send_with_reconnect` + `await_event` + `next_event` identically; one regression test covers the structural invariant.

### Recommended approach (when addressed)

Mirror `query_session_db_succeeds_twice_in_sequence` at `crates/runtime-main/src/drone_ipc/client.rs:542` for `read_signals` and `recover_session`. Each test: spin up duplex-pair fixture with two pre-written response events; assert both calls return distinct expected payloads. ~20 min for both. Roll in at the next test-rationalization pass — likely M06 (MCP work adds new IPC primitives and is a natural moment to audit the IPC test surface).

## TD-003 — `respond_uncertainty` stale-id behavior parity vs `respond_hitl`

**Date logged:** 2026-05-12
**Found by:** Stage V verifier run M04.V (finding #5)
**Pass that surfaced it:** Multi-call invariants
**Category:** other (test-coverage gap: stale-id behavior parity — also a first-class user-domain question)
**Resolution status:** open (decision needed before test)

### Description

`src-tauri/src/commands.rs` has soft-Ok-on-stale-id tests for `respond_hitl` (`:1193`) and the plan-control commands (`approve_plan` `:999`, `revise_plan`, `abort_plan`): if the renderer fires a response for an `invocation_id` / `prompt_id` / `plan_id` whose awaiter has already timed out or been resolved, the command returns Ok rather than 500-ing the renderer. `respond_uncertainty_command_with` covers success path (`:1109`), unknown-action error (`:1123`), and drone-error propagation (`:1141`) — but has no analogous stale-`invocation_id` test. Two scenarios for the entry:
1. Recovery dialog fires; user takes ages to click; the uncertainty entry is resolved server-side (e.g., session-end cleanup); user finally clicks — what should happen?
2. Two renderer tabs both surface the same uncertainty prompt; one resolves first; the second's click arrives stale.

### Why it's debt not bug

The code paths that produce stale invocation_ids aren't currently exercised in production (single-session v0.1 per §0d; no session-end cleanup that prunes pending uncertainty entries). The contract for the stale case is unspecified — the spec §1b Recovery semantics doesn't mention it, and the build agent picked "let drone IPC propagate whatever error" (`commands.rs:1141`). Whether that's the right behavior is a user-domain question first; once decided, the test follows.

### Recommended approach (when addressed)

1. **First** — surface to maintainer: should `respond_uncertainty`'s stale-id contract mirror `respond_hitl` (soft-Ok-on-stale, no renderer error) or differ (propagate drone error)? Default recommendation: mirror `respond_hitl` for renderer consistency.
2. **Then** — add `respond_uncertainty_with_stale_invocation_id_returns_ok` (or `..._returns_drone_error` per (1)) at `src-tauri/src/commands.rs` mirroring the `respond_hitl_with_no_pending_awaiter_returns_ok` pattern. ~15 min.

## TD-004 — Verifier prompt template: `<scope_to_verify>` derivation guidance was unclear

**Date logged:** 2026-05-12
**Found by:** Stage V verifier run M04.V (finding #6 — protocol self-calibration)
**Pass that surfaced it:** Inventory
**Category:** other (verifier-template authoring drift — protocol refinement)
**Resolution status:** **resolved** — addressed in this same PR via the STAGE-V-VERIFIER-PROMPT-TEMPLATE.md "Choosing `<scope_to_verify>`" subsection. Entry retained for audit trail.

### Description

`docs/build-prompts/M04-V-prompt.md` (the first real-world V prompt) authored its `<scope_to_verify>` block by pattern-matching spec sections to plausible-sounding file paths. One match — "§4a Verify ⇒ `schemas/verification.v1.json`" — produced a file path that doesn't exist in the codebase (the M04 phase doc never claimed to ship it; §4a Verify uses event variants + the existing `Hook` types in `common.v1.json`). The M04.V Inventory pass surfaced this as a 🟢 (verifier-template authoring drift, not a code finding).

### Why it's debt not bug

The wrong scope didn't break the V run; the Inventory pass correctly flagged the discrepancy and continued. But for M05.V (the first non-grandfathered V run) the parameterized prompt should derive its file lists from the milestone's V.2 Scope-to-Verify table, which itself derives from each stage's X.2 Files-to-Change tables — NOT from per-spec-section pattern-matching. The template needs explicit authoring guidance for this.

### Recommended approach (when addressed)

Add a "Choosing `<scope_to_verify>`" subsection to `docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md`'s "Authoring guidance for the per-milestone parameterization" section. Specify the derivation chain: per-stage X.2 files-to-change → milestone V.2 scope-to-verify table → V prompt's `<scope_to_verify>` inline content (or `ref` if V.2 exists in the phase doc). Cite TD-004 as the bit-by-this incident. Apply BEFORE M05.V authoring.

## TD-005 — runtime-main `cargo llvm-cov` gate not Windows-local-measurable (gotcha #56 nested-cargo-build recurrence)

**Date logged:** 2026-05-16
**Found by:** Stage V verifier run M06.V (finding #3)
**Pass that surfaced it:** Behavior
**Category:** observability (CI-parity: per-crate coverage gate not locally executable on Windows)
**Resolution status:** open

### Description

`cargo llvm-cov --package runtime-main --ignore-filename-regex "…" --fail-under-lines 95` aborts (exit 101) at `crates/runtime-main/tests/drone_ipc_loopback.rs:63` — `assert!(status.success(), "drone build failed")`. `ensure_drone_built()` runs a nested `cargo build --bin runtime-drone` with `CARGO_TARGET_DIR` set by llvm-cov to `target/llvm-cov-target`; that instrumented nested build fails on the Windows host. The same 10 `drone_ipc_loopback` tests pass 10/10 under plain `cargo test -p runtime-main --lib --tests`. The `-- --skip drone_ipc_loopback` mitigation does not work: `--skip` matches test *names* (`connects_to_drone`, …), which lack the filename prefix, so the abort persists.

### Why it's debt not bug

Documented gotcha #56 ("Windows-local `cargo llvm-cov` may flake on subprocess-spawning tests"). CLAUDE.md §6 designates **CI Linux** as the authoritative coverage gate. The M06 runtime-main delta (event_pipeline L1 wire, agent_sdk `narrow`/`try_mcp_dispatch`, mcp_dispatch.rs) is fully exercised by green integration tests (sdk_capability_integration 4, sdk_narrowing_integration 4, mcp_dispatch_runloop 4, mcp_dispatch_wire 5) + 434 lib tests; the runtime-mcp per-crate gate independently passed at 97.16% ≥95%. No coverage regression — only a local-measurement infra gap.

### Recommended approach (when addressed)

Make `ensure_drone_built()` robust under llvm-cov: either (a) build the instrumented drone into the llvm-cov target dir before the assertion (respecting the llvm-cov-set `CARGO_TARGET_DIR`), or (b) pre-stage the drone binary as an llvm-cov fixture step, or (c) complete the gotcha #56 CI-workflow graduation already tracked in CLAUDE.md §6 so the local gap is moot. ~30–60 min; touches `crates/runtime-main/tests/drone_ipc_loopback.rs` + possibly the CI workflow. Natural moment: M07 (which adds the concrete MCP-dispatch construction and will re-audit the runtime-main coverage surface).

## TD-006 — V.3 / A.4.4 / CLAUDE.md §6 runtime-main `llvm-cov` regex inconsistency (`key_store.rs`)

**Date logged:** 2026-05-16
**Found by:** Stage V verifier run M06.V (finding #4)
**Pass that surfaced it:** Behavior
**Category:** cosmetic (gate-definition drift across canonical sources)
**Resolution status:** open

### Description

The M06 phase doc's V.3 Behavior-harness runtime-main coverage command (`docs/build-prompts/M06-mcp-basic.md:3210`) and the A.4.4 acceptance line append `|src.key_store\.rs` to the `--ignore-filename-regex`. The CLAUDE.md §6 canonical runtime-main gate regex omits `key_store.rs` (exclusions: `main.rs|generated|providers/anthropic.rs|drone_ipc/connection.rs|sandbox_ipc/connection.rs`). Three nominally-canonical sources disagree on one exclusion token.

### Why it's debt not bug

`key_store.rs` is an OS-keychain holdout already outside the runtime-main *patch* gate semantics; the line-count delta from including/excluding one OS-call wrapper is small and did not affect any M06.V finding (the gate was not locally measurable regardless — see TD-005). CI uses the CLAUDE.md §6 form, which is the hard-floor authority. Functionally inert; a consistency/maintenance hazard, not a correctness bug.

### Recommended approach (when addressed)

Pick one canonical runtime-main `--ignore-filename-regex` and make CLAUDE.md §6, M06 V.3, and M06 A.4.4 agree. Recommend the CLAUDE.md §6 form is authoritative (it is the CI-run command); correct V.3 + A.4.4 to drop `|src.key_store\.rs`, or add a one-line note in §6 if `key_store.rs` should in fact be excluded. ~10 min `docs:` edit. Roll into the M07 Stage A pre-flight alongside the M06.V 🟡 #2 X.2 truth-up.
