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
**Resolution status:** **resolved** (M07.A red/test commit `348d1ef`) — `read_signals_succeeds_twice_in_sequence` + `recover_session_succeeds_twice_in_sequence` added to `crates/runtime-main/src/drone_ipc/client.rs` in-source `mod tests`, mirroring `query_session_db_succeeds_twice_in_sequence`. Entry retained for audit trail.

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
**Resolution status:** **resolved** (M07.A red/test commit `348d1ef`) — structural close via recommended approach (b): the six runtime-main integration test files that spawn `runtime-drone` were de-duplicated onto a shared `crates/runtime-main/tests/common/mod.rs` fixture that builds the drone once into a dedicated `target/drone-fixture` dir (never the parent run's target dir → no build-lock contention) with the workspace manifest + package pinned (CWD-independent; closes the `no bin target named runtime-drone` resolution failure) and the llvm-cov instrumentation env stripped. Measured Windows-local at M07.A: runtime-main 95.73% line ≥ 95 (exit 0); previously the gate aborted before any measurement. Not a local-only `--test-threads` flag — CI-parity-safe. Entry retained for audit trail.

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
**Resolution status:** **resolved** (this M07.A impl/green commit) — the stray `|src.key_store\.rs` token was dropped from all 6 runtime-main `--ignore-filename-regex` occurrences in `docs/build-prompts/M06-mcp-basic.md` so the M06 phase doc matches the canonical CLAUDE.md §6 form. The four canonical mirrors (`docs/coverage-policy.md` §A, CLAUDE.md §5 + §6, `codecov.yml`) were already byte-consistent and never carried the token — only the non-mirror phase-doc copy had drifted, so the v1.8 four-mirror sync rule is satisfied vacuously (no mirror value change). `docs/coverage-policy.md` §C M07.A entry records the reconcile. Entry retained for audit trail.

### Description

The M06 phase doc's V.3 Behavior-harness runtime-main coverage command (`docs/build-prompts/M06-mcp-basic.md:3210`) and the A.4.4 acceptance line append `|src.key_store\.rs` to the `--ignore-filename-regex`. The CLAUDE.md §6 canonical runtime-main gate regex omits `key_store.rs` (exclusions: `main.rs|generated|providers/anthropic.rs|drone_ipc/connection.rs|sandbox_ipc/connection.rs`). Three nominally-canonical sources disagree on one exclusion token.

### Why it's debt not bug

`key_store.rs` is an OS-keychain holdout already outside the runtime-main *patch* gate semantics; the line-count delta from including/excluding one OS-call wrapper is small and did not affect any M06.V finding (the gate was not locally measurable regardless — see TD-005). CI uses the CLAUDE.md §6 form, which is the hard-floor authority. Functionally inert; a consistency/maintenance hazard, not a correctness bug.

### Recommended approach (when addressed)

Pick one canonical runtime-main `--ignore-filename-regex` and make CLAUDE.md §6, M06 V.3, and M06 A.4.4 agree. Recommend the CLAUDE.md §6 form is authoritative (it is the CI-run command); correct V.3 + A.4.4 to drop `|src.key_store\.rs`, or add a one-line note in §6 if `key_store.rs` should in fact be excluded. ~10 min `docs:` edit. Roll into the M07 Stage A pre-flight alongside the M06.V 🟡 #2 X.2 truth-up.

## TD-007 — `kind_to_ref` / `tier_to_ref` duplicated across three modules

**Date logged:** 2026-05-16
**Found by:** Stage G `<simplify_pass>` (code-quality agent CQ-4)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural (cross-module duplication on the safety-primitive wire surface)
**Resolution status:** open

### Description

`kind_to_ref(CapabilityKind) -> CapabilityKindRef` appears in `crates/runtime-main/src/sdk/event_pipeline.rs:304` and `crates/runtime-main/src/sdk/agent_sdk.rs:529`; `tier_to_ref` appears in `event_pipeline.rs` and `src-tauri/src/commands.rs:530`. All four are identical `const fn` match expressions duplicated because no shared location exports them.

### Why it's debt not bug

Functionally correct; clippy's `non_exhaustive_patterns` red-flags a missed copy at compile time when a new `CapabilityKind`/`Tier` variant is added, so the maintenance risk is compile-time-caught, not silent. Maintainer deferred (chose the CQ-1+EFF-9-only subset at M06.G) because the extract touches the M06.V-verified ADR-0009 wire path (`event_pipeline.rs`/`agent_sdk.rs`) and the M06 PR diff should stay scoped to V-verified surfaces.

### Recommended approach (when addressed)

Add `pub(crate) kind_to_ref` + `tier_to_ref` to `crates/runtime-main/src/sdk/mod.rs`; the two sdk sub-modules import from there; `commands.rs` either imports the promoted `pub` form or keeps its 2-line `Tier→TierRef` inline. ~10-line extract + 2 import changes. Natural window: M07 (which re-audits the runtime-main coverage surface and re-touches the dispatch wire).

## TD-008 — `apply_mcp_dispatch` Invoked-arm is dead in production (empty `agent_id`)

**Date logged:** 2026-05-16
**Found by:** Stage G `<simplify_pass>` (code-quality CQ-2 / code-reuse reuse-5)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural (type-unsafe dead path guarded only by a comment)
**Resolution status:** open (M07 — bundle with ADR-0011 live dispatch)

### Description

`crates/runtime-main/src/sdk/mcp_dispatch.rs:113` `apply_mcp_dispatch`'s `Invoked` arm builds `ToolInvoked`+`ToolResult` with `agent_id: String::new()`. The run loop special-cases `Invoked` (`agent_sdk.rs:478`) and emits agent_id-correct events directly (the gotcha #68 fix), so the function's `Invoked` arm is reached only by the D-frozen `mcp_dispatch_wire.rs` integration test. Any future caller of `apply_mcp_dispatch` with `Invoked` silently produces broken (empty-`agent_id`) events; the inline comment is the only guard.

### Why it's debt not bug

Not a runtime bug today — the only production caller (the run loop) never routes `Invoked` through `apply_mcp_dispatch`, and the behavior is documented inline (gotcha #68 / ADR-0010 rationale). The type-level fix (split a `Blocked | Ambiguous` enum so `apply_mcp_dispatch` cannot be called with `Invoked`) perturbs the M06.V-verified D-frozen `mcp_dispatch_wire.rs` test, so it was deferred at M06.G rather than landed in the V-verified milestone PR.

### Recommended approach (when addressed)

At M07, alongside the ADR-0011 (a)–(d) concrete-`McpDispatcher`-construction + live-dispatch wire: introduce `McpNonInvokedOutcome { Blocked, Ambiguous }`, make `apply_mcp_dispatch` take that (exhaustive-by-construction), delete the dead `Invoked` arm + its comment, and update the `mcp_dispatch_wire.rs` integration test to the new shape in the same commit (the D-frozen contract is intentionally re-opened at M07 when the live path lands). Bundle with TD-009 (`ServerStatus` enum) since both land with the M07 dispatch/health state machine.

## TD-009 — `status` is a stringly-typed `String` where a `ServerStatus` enum belongs

**Date logged:** 2026-05-16
**Found by:** Stage G `<simplify_pass>` (code-quality CQ-6)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural (stringly-typed lifecycle state)
**Resolution status:** open (M07 — health-ping state machine)

### Description

`crates/runtime-mcp/src/client/registry.rs` `McpServerRecord.status` and `crates/runtime-mcp/src/client/mod.rs` `McpServerSummary.status` are both `String` with the enumerated value set documented only in a comment (`"configured" | "connected" | "errored" | "disabled" | "failed"`). A typo (`"error"` vs `"errored"`) could reach the DB or the renderer untyped.

### Why it's debt not bug

No active bug: M06 writes `status` exactly once as `"configured"` at insert; `run_health_pass` does not yet write status transitions (it updates `last_connected_at` on success and drops the connection on failure without writing `"errored"` to the column). The stringly-typed risk only materializes when the lifecycle state machine starts mutating `status`.

### Recommended approach (when addressed)

Introduce `ServerStatus { Configured, Connected, Errored, Disabled, Failed }` in `registry.rs` with `serde(rename_all = "snake_case")`, used by both `McpServerRecord` and `McpServerSummary`. Land it together with the M07 health-ping state machine that writes `"connected"`/`"errored"` to the column (the missing status-update is the same M07 work). Bundle with TD-008.

## TD-010 — Constructor-shape inconsistency + `now_unix_ms` third copy in `runtime-mcp`

**Date logged:** 2026-05-16
**Found by:** Stage G `<simplify_pass>` (code-quality CQ-3/CQ-10, code-reuse reuse-1)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural (constructor duplication + helper duplication)
**Resolution status:** open

### Description

(a) `McpClient::new` vs `McpClient::new_with_audit` (`client/mod.rs:75`) is a dual constructor differing only in `audit: Option<Arc<AuditWriter>>`; the `new` docstring itself says "prefer `new_with_audit`" and the no-audit path "doesn't exist" in production. (b) `McpDispatcher::new` (`dispatch.rs:104`) takes 5 positional params with `audit: Option<…>` `None` at 5 of 7 test sites — inconsistent with the `AgentSdk::new` + `with_mcp_dispatch` builder shape. (c) `now_unix_ms()` is defined a third time privately in `client/registry.rs:37` (i64) — `runtime_main::audit::entry::now_unix_ms` (u64, public) is already in scope (M06.C `run_health_pass` calls it at `client/mod.rs:341`).

### Why it's debt not bug

All three are pure structural/cosmetic; zero runtime cost, no behavior difference. The dual constructor and 5-arg constructor work correctly; the `now_unix_ms` third copy is a one-liner with an i64/u64 cast difference already handled at the existing call site.

### Recommended approach (when addressed)

Collapse `McpClient` to a single `new(…, audit: Option<Arc<AuditWriter>>)`; convert `McpDispatcher::new` to `new(resolver, enforcer, connections, session_id)` + `with_audit(writer) -> Self` mirroring `AgentSdk`; have `registry.rs` call `runtime_main::audit::entry::now_unix_ms` (cast at the site as line 341 already does) and delete the private copy. Pure refactor with no behavior change; roll into any M07 `runtime-mcp` touch.

## TD-011 — `ipc.ts` `McpTool`/`McpServerSummary` are hand-written non-schema mirrors

**Date logged:** 2026-05-16
**Found by:** Stage G `<simplify_pass>` (code-reuse reuse-7)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** extensibility (schema-coverage gap vs CLAUDE.md §14)
**Resolution status:** open

### Description

`src/lib/ipc.ts:13–30` hand-writes the TS interfaces `McpTool` + `McpServerSummary` mirroring `runtime_mcp::transport::McpTool` + `runtime_mcp::client::McpServerSummary`. Per CLAUDE.md §14 cross-bridge types should be schema-generated, but these response shapes are not in `schemas/mcp.v1.json` (the schema covers `McpServerConfig`, not the transport/client response types). The drift is documented inline at `ipc.ts:8`.

### Why it's debt not bug

Deliberate + documented; the structs are `transport`/`client` response types, not the persisted config the schema covers. The hand-maintenance is small and the doc-comment flags it. No correctness risk today.

### Recommended approach (when addressed)

If a future stage adds these response shapes to `schemas/mcp.v1.json` (or a sibling schema), regenerate the TS types and delete the hand-written mirrors. Otherwise accept the documented deliberate exception. Re-evaluate at M07 (registry import re-touches the MCP wire types) or M08 (Builder Canvas consumes the tool list).

## TD-012 — `runtime-mcp` namespace/dispatch/health-pass efficiency cluster

**Date logged:** 2026-05-16
**Found by:** Stage G `<simplify_pass>` (efficiency agent EFF-1, EFF-2/3, EFF-4)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** scalability (cold/warm-path waste, no v0.1 impact)
**Resolution status:** open (EFF-4 → M07; EFF-1/2/3 → tech-debt)

### Description

(EFF-1) `try_mcp_dispatch` (`crates/runtime-main/src/sdk/agent_sdk.rs:455`) rebuilds a `BTreeMap<String,String>` from the framework `mcp_aliases` on every `ProviderEvent::ToolUse` — warm path, O(N) clone, N immutable for the run-loop lifetime. (EFF-2/3) `crates/runtime-mcp/src/namespace/mod.rs:148/159` `connect_server` does two full O(S*T) `ambiguous_short_names()` scans per connect; `disconnect_server` does the same and the result is provably empty (the inline comment documents it). (EFF-4) `client/mod.rs:341` `run_health_pass` issues K sequential SQLite `UPDATE`s (one per pinged server) where a single `UPDATE … WHERE name IN (…)` would do.

### Why it's debt not bug

All correct; all negligible at v0.1 single-session, few-aliases, handful-of-servers scale. EFF-1 is a warm-path constant; EFF-2/3 are cold-path (connect/disconnect events); EFF-4 is zero-cost at single-server.

### Recommended approach (when addressed)

EFF-1: hoist the `aliases` binding out of the event loop (or pass the framework `&HashMap` by reference into `dispatch_if_mcp`) — one-line lift. EFF-2/3: `disconnect_server` becomes `self.connected.remove(server); vec![]` (+ optional `debug_assert!` for the documented invariant); `connect_server` limits the post-insert scan to the newly-added server's tool names. EFF-4: batch the `last_alive` updates into one SQL call. Land EFF-4 with the M07 multi-server health state machine (bundle with TD-008/TD-009); EFF-1/2/3 in any M07+ `runtime-mcp` cleanup pass.

## TD-013 — Renderer MCP store-update efficiency (graphStore + MCPNode)

**Date logged:** 2026-05-16
**Found by:** Stage G `<simplify_pass>` (efficiency agent EFF-5, EFF-6)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** scalability (no v0.1 impact; React/Zustand re-render hygiene)
**Resolution status:** open

### Description

(EFF-5) `src/lib/graphStore.ts:893` — every `tool_result` event (including non-MCP results, where `activeMcpCalls` is empty/unchanged) runs `Object.fromEntries(Object.entries(activeMcpCalls).filter(…))` and returns a new `activeMcpCalls` slice, defeating Zustand reference-equality even when nothing changed. (EFF-6) `src/components/nodes/MCPNode.tsx:26–28` uses two `useGraphStore` subscriptions (one `useShallow`-wrapped, one not) where a single combined `useShallow` selector returning `{ connStatus, callActive }` would halve the subscription overhead.

### Why it's debt not bug

Both correct; immaterial at v0.1 single-agent single-session node counts. EFF-5's map has ≤ one entry per connected server; EFF-6's double subscription is two cheap selectors. No incorrect render, just avoidable churn.

### Recommended approach (when addressed)

EFF-5: add an early-return guard — if `activeMcpCalls` is empty, skip the filter and return the unmodified slice (reference-stable). EFF-6: combine into one `useShallow((s) => ({ connStatus: s.currentMcpServers[name]?..., callActive: s.activeMcpCalls[name] }))`. Roll into the M08 Builder Canvas renderer pass or any M07+ renderer touch; pairs with the M05.F `<test_isolation_audit>` / Zustand-selector gotcha cluster.
