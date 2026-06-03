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

## TD-014 — `token_usage` projector at a flat path; third `SignalRow`-shaped struct

**Date logged:** 2026-05-19
**Found by:** Stage V verifier run M07.V (finding 🟢 #6) + Stage G `<simplify_pass>` (code-reuse RU-M07-2)
**Pass that surfaced it:** Inventory (V #6); N/A (closeout simplify pass)
**Category:** structural (cosmetic)
**Resolution status:** open

### Description

`crates/runtime-drone/src/token_usage.rs` ships at the crate's top level, not under a `projectors/` sub-module — the M07 phase doc D2.3.3 named `projectors/token_usage.rs`. The actual layout is consistent with the crate's established convention (`vdr.rs` and `plan_projector.rs` are also flat); the `projectors/` directory does not exist. Separately, `token_usage.rs` defines its own `SignalRow { event, session_id, timestamp, payload_json }` struct + private `read_signal_row` — the third such definition across `vdr.rs` (4 columns), `plan_projector.rs` (2 columns) and `token_usage.rs` (4 columns). Each projector selects a different column set from `signals`, so the structs cannot share one definition without widening to optional fields.

### Why it's debt not bug

Functional correctness is verified end-to-end (the M07.D2 assembled regression asserts `token_usage > 0`); `command_handler.rs` wires the projector correctly. The layout matches the crate's existing flat convention rather than the phase doc's aspirational `projectors/` sub-module. The `SignalRow` column-set differences are genuine, not copy-paste — no free extraction exists at three callers.

### Recommended approach (when addressed)

When the projector count grows past three, a `projectors/` sub-module pass: collapse `vdr` / `plan_projector` / `token_usage` into `projectors::{vdr,plan,token_usage}` with public re-exports preserved for source compatibility; at that point evaluate a shared `SignalRow` with optional columns. Cosmetic; no behavior change. Natural window: M08+ when a fourth projector lands.

## TD-015 — import-pipeline structural cluster (error erasure, nested-Option, arg count, redundant clones)

**Date logged:** 2026-05-19
**Found by:** Stage G `<simplify_pass>` (code-quality CQ-M07-2 / CQ-M07-3 / CQ-M07-6, efficiency EFF-M07-9)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural
**Resolution status:** open (M07.5 — bundle with the import-lifecycle restructure)

### Description

Four structural items on the M07.C/D2 import path: (a) `import_err_to_cmd` (`src-tauri/src/commands.rs:1143`) collapses every `ImportError` variant — `TierReviewRequired`, `OsMismatch`, `L3`, `Fetch` — to `CmdError::internal(e.to_string())`, discarding the variant signal the renderer could act on. (b) `try_mcp_dispatch` (`crates/runtime-main/src/sdk/agent_sdk.rs:571`) returns `Result<Option<Option<DispatchedTool>>, SdkError>` — a double-`Option` encoding a three-state discriminant (MCP-not-applicable / MCP-handled-no-feedback / MCP-invoked) that should be a named enum. (c) `import_artifact_with` (`crates/runtime-main/src/import/mod.rs:591`) carries `#[allow(clippy::too_many_arguments)]` (10 params). (d) `validate()` (`import/mod.rs:~408`) clones the parsed `serde_json::Value` once per schema variant where only the firing branch needs it.

### Why it's debt not bug

All four are correct today: `import_err_to_cmd` surfaces a usable message (just not a typed variant); the double-`Option` is exhaustively handled at the one call site; the 10-arg seam is injected once; `validate`'s clones are immaterial at v0.1 (import runs once per user action). The cluster is bundled because M07.5 (the V 🔴 #1 fix-cycle) re-opens the import-command surface — `import_artifact_with` must change signature for the `pending_review_id` split anyway, which is the natural moment to land all four.

### Recommended approach (when addressed)

At M07.5: expand `import_err_to_cmd` to a proper variant match (mirror `KeyStoreError → CmdError`); introduce `enum TryMcpResult { NotMcp, Handled, Dispatched(DispatchedTool) }`; group `import_artifact_with`'s injected seams into two builder structs (network-side / install-side) to drop the `#[allow]`; thread the parsed `Value` so `validate` clones at most once. All land in the M07.5 fix-cycle commit cluster.

## TD-016 — `connection_resolver.rs` — `record_to_transport` duplication + malformed-JSON swallowing

**Date logged:** 2026-05-19
**Found by:** Stage G `<simplify_pass>` (code-reuse RU-M07-1, code-quality CQ-M07-7)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural + observability
**Resolution status:** open

### Description

`crates/runtime-mcp/src/client/connection_resolver.rs` `record_to_transport` (≈line 57) re-implements stdio/http transport construction, duplicating the builder chain in `McpClient::transport_from_config` + `config_to_record` (`client/mod.rs`). The two paths are structurally isomorphic but separated because `connection_resolver` reads the flattened `McpServerRecord` row shape while `transport_from_config` reads the typify `McpServerConfig` — a new transport type added to one branch silently misses the other. Separately, `record_to_transport` parses `args_json` / `env_json` with `serde_json::from_str(s).unwrap_or_default()` — a malformed registry row yields an empty `Vec` / `BTreeMap` rather than a connect error.

### Why it's debt not bug

`connection_resolver.rs` ships at ≥95 (in the runtime-mcp gate) and the ADR-0011 (a) `impl ConnectionResolver` is M07.V-verified. The `record_to_transport` divergence is load-bearing (two genuinely different input shapes); a clean fix needs `McpServerRecord` to carry a `to_transport()` method. The `unwrap_or_default()` is not a v0.1 bug — registry rows are written by the runtime's own typed insert path, so a malformed `args_json` cannot occur today; the swallowing only matters if a future external writer or a migration corrupts a row. Returning an error is a behavior change on a Stage-V-verified safety primitive, so it is deliberately not an apply-now closeout refactor.

### Recommended approach (when addressed)

Give `McpServerRecord` a `to_transport()` method and have both `connection_resolver` and (where shapes allow) `McpClient` route through it. Replace `unwrap_or_default()` with `McpError::connect_failed(format!("malformed args_json for '{}': {e}", record.name))`. A `runtime-mcp` internal refactor; roll into any M07+ `runtime-mcp` touch.

## TD-017 — Tier / capability string-mapping duplication growth

**Date logged:** 2026-05-19
**Found by:** Stage G `<simplify_pass>` (code-reuse RU-M07-3 / RU-M07-4 / RU-M07-5)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural (extends TD-007)
**Resolution status:** open

### Description

Three single-site mapping duplications added in M07: (a) `tier_wire(Tier) -> &'static str` (`crates/runtime-main/src/import/mod.rs:~503`) is a third distinct `Tier → string` match (after `tier_to_ref` in `event_pipeline.rs` + `commands.rs` — TD-007), in a third location, mapping to the install-lock wire string. (b) `RegistryAdapter::upsert` (`src-tauri/src/commands.rs:1124`) copies `McpServerImport → McpServerRecord` field-by-field (8 fields) with no `From` impl. (c) `capability_summary` (`import/mod.rs:~466`) iterates a hard-coded capability-key list (`["tools_called", "network", "spawn_agents"]`) that mirrors the `schemas/capability.v1.json` enumeration.

### Why it's debt not bug

All correct and all single-site: `tier_wire` is semantically distinct from the event-wire `TierRef` (different output type); the `McpServerImport → McpServerRecord` conversion has one call site (below the fourth-use abstraction threshold); `capability_summary` is a deliberate best-effort human-facing summary with no generated accessor to reuse. None is a copy of existing code — they are parallel mappings of the same enums.

### Recommended approach (when addressed)

Fold into the TD-007 resolution: when `pub(crate) tier_to_ref` is extracted to `sdk/mod.rs`, move `tier_wire` to `tier/mod.rs` as a `pub(crate)` sibling. Add a `From<&McpServerImport> for McpServerRecord` if a second conversion site appears (a bulk-import path). Leave `capability_summary` until a new capability kind forces a touch. Low priority; no runtime cost.

## TD-018 — M07 efficiency micro-waste (alias-map rebuild, double clone, panel scans, silent timestamp)

**Date logged:** 2026-05-19
**Found by:** Stage G `<simplify_pass>` (efficiency EFF-M07-7 / EFF-M07-8 / EFF-M07-10, code-quality CQ-M07-5)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** scalability + observability
**Resolution status:** open

### Description

Four low-impact waste sites: (a) `try_mcp_dispatch` (`crates/runtime-main/src/sdk/agent_sdk.rs:~579`) rebuilds the `mcp_aliases` `BTreeMap<String,String>` from the framework on every `ProviderEvent::ToolUse` (warm loop path; aliases are immutable for the run-loop lifetime — the same shape as EFF-1 / TD-012, now also in the M07 `try_mcp_dispatch` wrapper). (b) double `args.clone()` at the `dispatch_if_mcp` call site (`agent_sdk.rs:~592` + `~612`) — `args` feeds both the dispatch call and the `ToolInvoked` event. (c) `ImportPanel.tsx` (≈line 38) runs `Object.values(s.imports)` + `.find` + two `.filter` passes — three O(N) scans in the render body. (d) `token_usage.rs` (≈line 120) does `timestamp.parse::<i64>().unwrap_or(0)` — a corrupt row gets epoch-0 silently rather than a logged warning.

### Why it's debt not bug

All correct; all immaterial at v0.1 scale (single-session, a handful of aliases / imports / servers). (a) is a warm-path constant; (b) is two cheap `serde_json::Value` clones per tool use; (c)'s N is ≤ a handful of imports; (d)'s corrupt-row case cannot occur with the runtime's own typed signal-write path. (b) touches the Stage-V-verified `McpToolDispatch` trait signature, so it is a maintainer's-call refactor, not apply-now.

### Recommended approach (when addressed)

(a) hoist the `aliases` binding out of the event loop (or pass `&HashMap` into `dispatch_if_mcp`) — pairs with EFF-1 / TD-012. (b) construct `ToolInvoked.input` before the dispatch call, or have the trait take `&Value` — bundle with the TD-015 `try_mcp_dispatch` enum work at M07.5. (c) combine into one reduce pass behind a stable `useShallow` selector — pairs with TD-013. (d) add a `tracing::warn!` before `.unwrap_or(0)`, mirroring the `vdr.rs` projector observability pattern. All low priority; roll into the relevant M07+ touch.

## TD-019 — `builderStore.replaceFramework` does not re-trigger continuous validation

**Date logged:** 2026-05-21
**Found by:** Stage V verifier run M08.V (finding 🟢 #3)
**Pass that surfaced it:** Wire
**Category:** observability (stale validation badges after a JSON-tab edit / load)
**Resolution status:** open

### Description

`src/lib/builderStore.ts:466` `replaceFramework: (fw) => set({ framework: fw })` — the action a valid `JsonView` JSON-tab edit and the Inspector's Load both call — does NOT call `scheduleValidation()`, unlike the three canvas-mutation actions `addNode` / `updateNode` / `connectEdge`. The M08.D2 phase doc (D2.3.4) specifies the debounced continuous-validation trigger fires on "every `framework` mutation"; `replaceFramework` is a `framework` mutation that omits it. After a JSON-tab edit or a `load_framework`, the canvas red badges (`builderStore.validation`) reflect the pre-edit state until the user clicks the explicit Validate button or makes a canvas edit.

### Why it's debt not bug

MVP §M8 criterion 6 (the canvas re-derives on a JSON edit) still passes — the `canvasNodes` / `canvasEdges` projection is unconditional and updates correctly. The explicit Inspector Validate button works. Only the *badge freshness* lags after a JSON / load path; a workaround (click Validate) exists. No incorrect computation, only stale display.

### Recommended approach (when addressed)

Add a `scheduleValidation()` call to `replaceFramework` (the same trigger `addNode` / `updateNode` / `connectEdge` use). One-line change in `src/lib/builderStore.ts`. Roll into any M09+ builder-store touch.

## TD-020 — `builderStore.removeNode` is a permanent no-op stub

**Date logged:** 2026-05-21
**Found by:** Stage V verifier run M08.V (finding 🟢 #4)
**Pass that surfaced it:** Multi-call invariants
**Category:** extensibility (dead public-interface surface)
**Resolution status:** open

### Description

`src/lib/builderStore.ts:498` `removeNode: () => set((s) => s)` is a typed no-op exposed on the `BuilderState` public interface, documented "node deletion is not in D2's scope; a later stage fills it" (ADR-0020 — Stage C ships the canvas-mutation actions as no-op stubs D1/D2 fill; `removeNode` is the one D1/D2 never filled). No component calls it (grep: zero non-store references), so it is harmless dead interface surface today.

### Why it's debt not bug

Node deletion is genuinely out of M08 scope — no MVP §M8 criterion requires it, and nothing calls `removeNode`, so the no-op cannot misbehave. The risk is latent: a future delete affordance wired to `removeNode` before it is implemented would silently no-op (no error, no deletion).

### Recommended approach (when addressed)

Implement `removeNode` (drop the node from `framework` + `nodePositions`, prune any edge referencing it) when node deletion enters scope, or remove it from the `BuilderState` interface until then. Roll into whichever milestone adds canvas node deletion.

## TD-021 — `kind_to_ref` / `capability_kind_label` capability-label duplication (M08 simplify pass)

**Date logged:** 2026-05-21
**Found by:** M08.H closeout `<simplify_pass>` (code-reuse axis — RU-M08-1, RU-M08-2)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural
**Resolution status:** open

### Description

Two related capability-label duplications surfaced in the M08 simplify pass. **RU-M08-1:** `const fn kind_to_ref(k: CapabilityKind) -> CapabilityKindRef` is a byte-identical five-arm match defined independently in two private `sdk` submodules — `crates/runtime-main/src/sdk/agent_sdk.rs:756` and `crates/runtime-main/src/sdk/event_pipeline.rs:322`. This is **pre-M08 duplication** (TD-007's class — `kind_to_ref`/`tier_to_ref` mapping growth) and was not introduced by M08; the simplify pass re-surfaced it. **RU-M08-2:** `const fn capability_kind_label(kind: CapabilityKindRef) -> &'static str` (`crates/runtime-main/src/builder/tester.rs:207`) hand-maps `CapabilityKindRef` to a snake_case label string — `CapabilityKindRef` is a hand-rolled `runtime-core` enum with no `Display` impl, so the Tester derives the label by hand.

### Why it's debt not bug

Both are correct today. `kind_to_ref`'s match is compiler-exhaustiveness-checked — a new `CapabilityKind` variant forces a compile error in both copies, so they cannot silently diverge. `capability_kind_label` is isolated, indirectly tested, and its comment already explains why it exists. No behavior risk.

### Recommended approach (when addressed)

Fold into TD-007's resolution: a single `pub(crate)` capability-label helper in `runtime-main/src/sdk/mod.rs` (or a `sdk/capability_labels.rs`) for `kind_to_ref`; for `capability_kind_label`, the right fix is a `Display` or `label()` impl on `CapabilityKindRef` in `runtime-core` — a cross-crate change best landed when `runtime-core`'s event types are next touched.

## TD-022 — `BuilderToolNode` / `BuilderSkillNode` structural duplication (M08 simplify pass)

**Date logged:** 2026-05-21
**Found by:** M08.H closeout `<simplify_pass>` (code-reuse axis — RU-M08-3)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural
**Resolution status:** open

### Description

`src/components/builder/nodes/BuilderToolNode.tsx` and `BuilderSkillNode.tsx` are structurally identical — `Handle(top) + NodeValidationBadge + div.{type}-node__name + Handle(bottom)`. The only differences are the CSS class names (`tool-node`/`skill-node`, `builder-tool-node`/`builder-skill-node`) and the `data-testid` prefix. A shared `BuilderLeafNode` wrapper taking `baseClass` / `label` / `testIdPrefix` would collapse the leaf node components.

### Why it's debt not bug

Both components render correctly and are 100%-covered. The CSS classes are intentionally distinct (`tool-node__name` is monospace, `skill-node__name` is italic), so the components are not byte-identical *presentations* — only structurally parallel. The net gain of collapsing two ~30-line files is small.

### Recommended approach (when addressed)

Re-evaluate at M09: if `BuilderHitlNode` and `BuilderHookNode` exhibit the same leaf-node shape (four leaf-alike components), extract a `BuilderLeafNode` wrapper. Touches `builderNodeTypes` registration and the class-name-querying tests. Skip if the four nodes diverge.

## TD-023 — Builder canvas-projection / summary efficiency cluster (M08 simplify pass)

**Date logged:** 2026-05-21
**Found by:** M08.H closeout `<simplify_pass>` (efficiency axis — EFF-M08-1, EFF-M08-4, EFF-M08-5, EFF-M08-6)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** scalability
**Resolution status:** open

### Description

Four immaterial-at-v0.1-scale efficiency findings. **EFF-M08-1** (`src/lib/builderStore.ts:327`): `canvasEdges()` rebuilds the `nodeIds` `Set` from a re-projected node list on every call. **EFF-M08-4** (`crates/runtime-main/src/builder/validate.rs:87`): `serde_json::from_value(doc.clone())` clones the whole JSON tree — `from_value` consumes the value, so the clone is forced by the `&Value` parameter; the Tauri command wrapper could pass an owned `Value` instead. **EFF-M08-5** (`crates/runtime-main/src/builder/summary.rs:111/114`): `parent_grants_for_agent` is re-computed once per child inside the spawn-edge loop — a parent with K children pays K re-computations. **EFF-M08-6** (`summary.rs:120`): `parent_caps.clone()` per child edge, pairing with EFF-M08-5.

### Why it's debt not bug

All four are correct. v0.1 framework sizes are tens of nodes and tens of spawn edges; the redundant work is microseconds. None is on a tight render loop that would be perceptible. The canvas projection's module-level memoization (correctly keyed on `framework`/`nodePositions` identity) already prevents the `useSyncExternalStore` infinite-loop hazard.

### Recommended approach (when addressed)

EFF-M08-5/6 is the only one worth a one-line hoist (compute `parent_grants_for_agent` once outside the inner `for child_id` loop in `summary.rs`); the rest are immaterial. Address opportunistically on any M09+ builder-backend touch, or leave until a framework grows to hundreds of nodes (a v1.0 multi-framework-comparison concern).

## TD-024 — `builderStore` code-quality cluster (M08 simplify pass)

**Date logged:** 2026-05-21
**Found by:** M08.H closeout `<simplify_pass>` (code-quality axis — CQ-M08-1, CQ-M08-2, CQ-M08-3, CQ-M08-4)
**Pass that surfaced it:** N/A (closeout simplify pass)
**Category:** structural
**Resolution status:** open

### Description

Four low/medium structural smells in the Builder store and crate surface. **CQ-M08-1** (`src/lib/builderStore.ts:484`): `updateNode` extracts the agent id with an ad-hoc `nodeId.replace(/^agent:/,'')` regex instead of the already-exported `parseNodeId` helper used elsewhere; the regex passes a non-agent id (e.g. `tool:Read`) through unchanged, causing a silent no-op match against `agents`. **CQ-M08-2** (`builderStore.ts:327`): `memoizedCanvasEdges` accepts a `nodeIds: Set<string>` parameter the memoization cache key ignores (the cache keys on `framework` alone) — a signature trap for a future caller. **CQ-M08-3** (`crates/runtime-main/src/builder/mod.rs:44`): `fold_outcome` / `load_verified_artifact` are re-exported from the `builder` crate surface but have no external consumer — unnecessary public-API surface. **CQ-M08-4** (`builderStore.ts:69`): `updateNode`'s `patch` parameter is `Record<string, unknown>` — stringly-typed; `Partial<Pick<Agent, 'role'|'model'|'allowed_tools'|'allowed_skills'|'spawns'>>` would compile-guard the keys.

### Why it's debt not bug

All four are correct under current callers. `updateNode`'s only callers (`NodeConfigPanel.tsx`) pass agent ids and known-correct `patch` keys; `memoizedCanvasEdges` has one caller passing the matching `nodeIds`; the over-exported seams are harmless. None touches a safety-critical surface, and none is a behavior change today.

### Recommended approach (when addressed)

Land at the next builder-store touch (M09+) alongside `removeNode`/TD-020: route `updateNode` through `parseNodeId` (CQ-M08-1 — the highest-value fix, since a filled-in `removeNode` could broaden the caller set); either drop the `nodeIds` parameter from `memoizedCanvasEdges` or document the cache ignores it (CQ-M08-2); narrow the `builder` crate re-exports (CQ-M08-3); type `patch` as a `Partial<Pick<Agent,…>>` (CQ-M08-4).

## TD-025 — Smoke run too fast to observe streaming (M07-IRL 🟢 #1)

**Date logged:** 2026-05-21
**Found by:** post-M07.5 gate-7 IRL walk-through (`docs/M07-irl-findings.md` 🟢 #1); logged at the M08.H closeout (the M07-IRL routing to `docs/tech-debt.md` was not completed during the M07-IRL pass)
**Pass that surfaced it:** N/A (post-merge IRL test)
**Category:** observability
**Resolution status:** open

### Description

The smoke prompt ("say only the word: hello") is a ~1-second round trip, so token streaming into the live graph is not perceptible — the agent node goes spawned → complete with nothing visibly streaming. Not a defect (the minimal prompt is by design and the end state verifies correctly), but the live-graph streaming surface is hard to observe out of the box.

### Why it's debt not bug

The smoke session works correctly and its end state is verified; the streaming pipeline is exercised by the M02/M03 tests. Only the *demo observability* of streaming is poor.

### Recommended approach (when addressed)

Ship a longer demo prompt, or a replay-at-speed control on the live graph. Cosmetic / demo-quality; queue for v1.0 or any milestone touching the smoke-session UX.

## TD-026 — No bundled importable example artifact (M07-IRL 🟢 #4)

**Date logged:** 2026-05-21
**Found by:** post-M07.5 gate-7 IRL walk-through (`docs/M07-irl-findings.md` 🟢 #4); logged at the M08.H closeout
**Pass that surfaced it:** N/A (post-merge IRL test)
**Category:** other (out-of-box UX)
**Resolution status:** open

### Description

The import URL field carries only a placeholder; the repo's own skills are `.md` files (the import pipeline validates JSON), and no example importable JSON artifact ships. A user cannot exercise the import feature out-of-the-box without sourcing or hosting a JSON artifact externally.

### Why it's debt not bug

The import pipeline works correctly against a valid JSON artifact (verified IRL with a hand-made gist). Only the first-run discoverability of the feature is poor.

### Recommended approach (when addressed)

Ship an example importable artifact in the repo (a minimal valid agent/tool/skill JSON), or pre-fill a working example URL in the import field. Pairs naturally with M09's Generators (a generated artifact is itself an importable example) or the §14 first-run UX work.

## TD-027 — Graph minimap renders blank / unthemed (M07-IRL 🟢 #8)

**Date logged:** 2026-05-21
**Found by:** post-M07.5 gate-7 IRL walk-through (`docs/M07-irl-findings.md` 🟢 #8); logged at the M08.H closeout
**Pass that surfaced it:** N/A (post-merge IRL test)
**Category:** cosmetic
**Resolution status:** open

### Description

The React Flow `MiniMap` on the live graph renders as a blank white square — it functions as a click-to-navigate control but shows no miniature node representation, and it is unthemed (white against the dark canvas).

### Why it's debt not bug

The minimap's navigation function works; only the miniature rendering + theming is missing. The live graph itself renders correctly.

### Recommended approach (when addressed)

Pass a `nodeColor` / `nodeStrokeColor` callback to React Flow's `<MiniMap>` keyed on node kind, and theme the minimap background to the dark canvas tokens. Small CSS + one prop. Applies to both the live `GraphCanvas` and (if a minimap is added) the Builder canvas; roll into any M09+ canvas touch.

## TD-028 — `walker.rs::agents_item_id` + `capability_map::inline_agents` `FrameworkAgentsItem::Object` branches structurally unreachable on a loaded framework post-ADR-0022

**Date logged:** 2026-05-26
**Found by:** Stage V verifier run M08.6.V (finding 🟢 #3)
**Pass that surfaced it:** Wire
**Category:** structural
**Resolution status:** open

### Description

`crates/runtime-main/src/framework_loader/walker.rs:169` matches `FrameworkAgentsItem::Object { id, .. }` to extract the agent id; `crates/runtime-main/src/framework_loader/capability_map.rs:324-333` filters `framework.agents.iter()` to `FrameworkAgentsItem::Agent(_)` only and silently drops `FrameworkAgentsItem::Object` variants. Per ADR-0022 (Accepted at M08.6 Stage B), the in-memory `Framework` post-`load_framework` carries only `Agent(_)` variants — the `Object` variant exists only in `framework.json` on disk (re-split at save). The branches are defensive M08-era code that survived the milestone.

### Why it's debt not bug

The functions read structural fields (`id`) and use a defensive filter — they do NOT read `.md` files, parse YAML frontmatter, or perform reference resolution. ADR-0022's single-resolution-boundary invariant is not violated. The grep for `serde_yaml` + frontmatter parsing returns only `crates/runtime-main/src/builder/persist.rs` and its test files. The branches are correct under any input; they just have no production caller for the Object arm post-ADR-0022.

### Recommended approach (when addressed)

Once the renderer surface is confirmed never to produce `Object` variants in `builderStore.framework.agents[]` (it doesn't today — `builderAgent` always emits inline shapes), tighten both branches to `unreachable!("post-ADR-0022 load resolves all agent refs to inline; Object variant should not appear in an in-memory Framework")` with a SAFETY-comment naming the ADR. ~10 minutes; roll in at any M09+ touch of `framework_loader/`.

## TD-029 — No e2e-tauri assertion that Inspector capability summary populates on loaded `examples/aria/`

**Date logged:** 2026-05-26
**Found by:** Stage V verifier run M08.6.V (finding 🟢 #4)
**Pass that surfaced it:** Behavior
**Category:** observability

**Resolution status:** open

### Description

M08.6 Stage A's triage of 🟡 #5 (blank Inspector capability summary on loaded ARIA) predicted Stage B's loader-resolution would incidentally populate it: `framework_capability_summary` (`crates/runtime-main/src/builder/summary.rs:73-74`) reads `capability_map::inline_agents`, which post-ADR-0022 returns all 8 ARIA agents (where pre-B it returned zero because every entry was `FrameworkAgentsItem::Object` and filtered out). The structural fix is sound and Stage V's Wire pass confirmed it. But no test asserts the rendered Inspector capability-summary text is non-empty on a loaded `examples/aria/` — neither vitest (no Inspector-pane capability-summary unit tests exist) nor e2e-tauri (`builder_load_aria.e2e.ts` asserts canvas layout + edges; `builder_palette_aria.e2e.ts` asserts Palette items; neither asserts Inspector content).

### Why it's debt not bug

The structural cause is gone (path-ref agents resolve to inline; `inline_agents` returns them; `framework_capability_summary` aggregates over real `Capabilities` blocks). The IRL acceptance bar — load ARIA → see ARIA's workflow — does not include "Inspector capability summary is non-empty" as a hard checkbox; the panel that wasn't part of M08.6's deliverable will incidentally populate when the maintainer loads ARIA post-merge. The debt is missing automated verification at the rendered-UI layer, not a defect.

### Recommended approach (when addressed)

Add a renderer or e2e-tauri assertion that, after loading `examples/aria/`, the Inspector capability-summary surface contains a non-empty list of declared tools / agents / hosts. Cheap (~15 min) to fold into M09's Stage 0 real-app discovery walk (since 🔴 #4 / 🟡 #6 tier-state work will exercise the Inspector pane anyway) or into any future Inspector touch.

## TD-030 — `split_frontmatter` candidate for extraction to `runtime-core` (M08.6 simplify pass)

**Date logged:** 2026-05-26
**Found by:** M08.6 Stage F closeout `<simplify_pass>` (RU-M08.6-1)
**Pass that surfaced it:** N/A (closeout simplify)
**Category:** extensibility
**Resolution status:** open

### Description

`split_frontmatter` lives at `crates/runtime-main/src/builder/persist.rs:422-429` as a 7-line `pub fn split_frontmatter(text: &str) -> Option<(&str, &str)>`. A near-identical pattern exists in `crates/runtime-core/tests/round_trip.rs:140` (test-local `text.splitn(3, "---\n")` with CRLF normalization at line 139). Two callers today: one production (M08.6.B's loader resolution), one test (round-trip serde verification).

### Why it's debt not bug

The function is small + correctly contained at the load boundary that needs it; the test-local copy is appropriate for a serde round-trip test (no cross-crate dependency). The split pattern is durable but not yet a reusable utility — extracting now would create a `runtime-core → {test + persist}` dependency that is architecturally sound but currently low-impact (per CLAUDE.md §9 "wait for the fourth" abstraction discipline).

### Recommended approach (when addressed)

Revisit when a fourth independent caller emerges — likely candidates: M09 artifact generators parsing `.md` bodies on write/preview; any v1.0 Share It module reading framework artifacts. At that point, move `split_frontmatter` to `crates/runtime-core/src/lib.rs` as a public utility; re-export from `runtime-main`; delete the test-local copy. ~15 minutes when the fourth call site lands.

## TD-031 — Load/save directory-escape asymmetry could carry an inline rationale comment (M08.6 simplify pass)

**Date logged:** 2026-05-26
**Found by:** M08.6 Stage F closeout `<simplify_pass>` (EFF-M08.6-1)
**Pass that surfaced it:** N/A (closeout simplify)
**Category:** observability (documentation hygiene)
**Resolution status:** open

### Description

`is_outside_framework_dir` (`crates/runtime-main/src/builder/persist.rs:243-249`) guards `save_framework` writes — three call sites for agents / tools / skills emission, refusing to write outside the framework directory. `read_referenced_md` (`crates/runtime-main/src/builder/persist.rs:374-380`) deliberately does NOT call the guard — Ralph's `../aria/tools/aria_verify.md` cross-framework reads are explicitly permitted by ADR-0022. The asymmetry is intentional (writes are scoped; reads cross frameworks); the rationale lives in ADR-0022 but is not inline at the call sites.

### Why it's debt not bug

The code is correct under ADR-0022. The asymmetry is structurally clear to a maintainer familiar with the ADR (the `format!("agents/{}.md", ...)` write pattern + the `is_outside_framework_dir` guard at writes only). A fresh session would have to read ADR-0022 to understand why reads cross `..` and writes don't.

### Recommended approach (when addressed)

If a fresh session encounters confusion (test-driven clarity per CLAUDE.md §9 "Don't write [comments] unless removing them would confuse a future reader"), add a one-line comment at `read_referenced_md` referencing ADR-0022's cross-framework-read decision and a parallel one-line at `is_outside_framework_dir` referencing the write-scope decision. Pure documentation; zero behavior change. Roll into any future `persist.rs` touch.

## TD-032 — `applyLoadedFramework` layout-seeding inline boilerplate (M08.6 simplify pass)

**Date logged:** 2026-05-26
**Found by:** M08.6 Stage F closeout `<simplify_pass>` (CQ-M08.6-1)
**Pass that surfaced it:** N/A (closeout simplify)
**Category:** structural (premature-abstraction candidate)
**Resolution status:** open

### Description

`applyLoadedFramework` (`src/lib/builderStore.ts` — the M08.6.D-added load-only seam) projects nodes twice (once with empty `nodePositions` to feed `projectCanvasEdges`, then `layoutGraph(nodes, edges)`, then hand-writes positions back into a `Record<string, Position>`). The pattern is currently inline; a helper `seedPositionsFromLayout(fw: Framework): Record<string, Position>` would compress the 11-line `set()` callback to 3 lines.

### Why it's debt not bug

The current inline form is transparent and makes the ADR-0020 load-only seeding visible at the point of use (Stage D's surface event #1: the seam-choice rationale is what carries the invariant — "auto-layout fires on LOAD only, not on every framework mutation"). Extracting now would bury the layout logic one level deeper without a second caller — same trade-off as the M08 simplify pass's CQ-M08-1 (the `updateNode` ad-hoc regex was a real inconsistency but its fix was best landed at the next builder-store touch, not patched in isolation at closeout).

### Recommended approach (when addressed)

If a second caller emerges (likely candidates: an M09 Tester re-layout button; an "open recent framework" Inspector hook; a "reset layout" canvas affordance), extract `seedPositionsFromLayout` to `src/lib/layout.ts` adjacent to `layoutGraph`, replace both call sites with the helper, ensure the load-only contract stays documented at each call site. ~20 min when the second caller lands.

## TD-033 — Built-in tool executor gates on `file_access` scope, not execution-time tool-authorization

**Date logged:** 2026-05-29
**Found by:** M08.7.A green-phase impl (zero-propagation triage of a finding surfaced while wiring the executor; maintainer-approved disposition)
**Pass that surfaced it:** N/A (manual review during rung-1 implementation)
**Category:** other (defense-in-depth / capability-model layering)
**Resolution status:** open (v1.0 hardening)

### Description

`execute_builtin` (`crates/runtime-main/src/sdk/builtin_tools.rs`) gates an in-process built-in `Read`/`Write` op on the **`file_access`** capability scope only — it builds a `(Read|Write, filesystem, Path, …)` declaration and runs it through `CapabilityEnforcer::check`. It does NOT additionally check an execution-time **tool-invocation authorization** (the `allowed_tools` / `Exec/<toolname>` grant the M06 pipeline path checks for non-built-in tools). An agent that emits a `Read` `ToolUse` for a path inside its `file_access.read` scope executes the read even if `Read` were absent from its `allowed_tools`.

### Why it's debt not bug

In v0.1 the only execution path is the Anthropic provider via the Tester, and `allowed_tools` IS gated at **advertisement**: `test_agent_config` advertises a built-in `ToolDef` only for names in the agent's `allowed_tools` (`builtin_tool_defs`), so a well-behaved model only emits `ToolUse` for tools it was advertised. The model cannot emit a `Read` `ToolUse` for a tool the framework didn't authorize, so the file_access check is the operative boundary and no v0.1 path bypasses authorization. The §1.4 BDD specifies `file_access` as the gate, and rung 1 implements exactly that.

The gap is defense-in-depth for **non-advertisement / non-Anthropic paths**: a future provider that emits un-advertised tool calls, a replay/scripted path that injects a `ToolUse` directly, or a malicious model ignoring the advertised set would reach `execute_builtin` with only the file_access check between it and the filesystem (still scoped, but not authorization-checked).

### Recommended approach (when addressed)

v1.0 hardening: have the built-in branch in `drive_stream` (or `execute_builtin`) additionally verify the tool is in the agent's `allowed_tools` / has the `Exec/<toolname>` grant before running — making built-in execution require BOTH tool-invocation authorization AND file_access scope (the same two-dimension model the spec implies). Land it alongside the v1.0 provider-abstraction work (the first non-Anthropic provider is the trigger) or any execution path that does not route tool advertisement through `builtin_tool_defs`. Small change; touches `crates/runtime-main/src/sdk/agent_sdk.rs` + `builtin_tools.rs`.

## TD-034 — No agent output is visible in the running app (IRL observability gap)

**Date logged:** 2026-05-29
**Found by:** M08.7.A rung-1 close (the IRL is unobservable — a real run cannot be watched in the assembled app)
**Pass that surfaced it:** N/A (rung-1 IRL prep)
**Category:** observability
**Resolution status:** open, **MAINTAINER-CONFIRMED 2026-05-31** (rung-1 IRL, running app) — the Tester panel shows the run **completed** but renders **NO agent reply**; the agent's output (tool invocation, tool result, streamed text quoting `[package]`, completion) was visible **only via `RUST_LOG`**, never in-app. Mitigated by debug-log; full fix routed to M08.7b.

### Description

A rung-1 built-in-tool run produces no agent output a human can observe in the running app. Agent events (`StreamText`, `ToolInvoked`/`ToolResult`, `AgentComplete`) reach only (a) the renderer graph and (b) the throwaway Tester SQLite DB — neither surfaces the agent's text or tool results to the user:

- The Tester graph nodes are not clickable, so a `ToolInvoked(Read)` / `ToolResult` node exposes no payload.
- The live / main canvas streams no agent text — `StreamText` does not render as visible output.
- The Tester DB is created in a tempdir and discarded after the run, so the persisted signals are not inspectable post-hoc.

This blocks IRL observability: the maintainer cannot confirm by watching the app that the agent actually read the file and fed the contents back.

### Why it's debt not bug

The execution path is correct and proven by the assembled regression tests (`tests/builtin_tool_execution.rs`: the agent reads the file, the contents feed back as a `tool_result`, the final text quotes them). The gap is purely **surfacing** — the events are produced and emitted; nothing in v0.1's UI renders them in a watchable form. No behavior is wrong; the observable-evidence channel for a live run is missing.

### Recommended approach (when addressed)

Minimal mitigation (landed at M08.7.A): `log_event_debug` in `crates/runtime-main/src/sdk/agent_sdk.rs::emit` logs each event's salient payload at `debug`, so `RUST_LOG=debug` makes a run watchable in the log. Full fix (M08.7b): surface agent output in the live-graph execution view — render `StreamText` as visible agent text and make tool nodes expose their input/result payload — so a run is observable in-app without a debug log.

## TD-035 — Relative file paths resolve against the app CWD (≠ repo root), undefined for later rungs

**Date logged:** 2026-05-29
**Found by:** M08.7.A rung-1 IRL watch (RUST_LOG=debug, real Anthropic model)
**Pass that surfaced it:** N/A (IRL observation)
**Category:** other (path-resolution semantics)
**Resolution status:** open (forward-looking — no rung-1 defect)

### Description

During the rung-1 IRL, the built-in `Read` executor resolved a **relative** path against the **app's working directory**, which is NOT the repo root. Observed evidence: a relative `test-read.txt` came back **not-found** while `Cargo.toml` was **found** — confirming the app CWD is somewhere other than the project root a user would assume. Relative paths therefore resolve against an app-determined directory, not the project tree.

### Why it's debt not bug

Rung 1 is correct as-is: `execute_builtin` is path-string-parameterised and capability-checked (`crates/runtime-main/src/sdk/builtin_tools.rs`). Whatever path string the model emits is what `std::fs::read_to_string` resolves and what the `Path`-scoped `file_access` check evaluates — `globset` matches on the same string, so scope enforcement and the read agree. No path is read outside scope; nothing silently escapes the capability boundary. The IRL confirmed the happy path with `Cargo.toml`. The finding is purely that **relative-path resolution semantics are undefined/implicit** — fine while every IRL path is absolute or CWD-relative-by-luck, latent the moment a rung introduces a user-facing relative-path convention.

### Recommended approach (when addressed)

For any later rung that accepts **relative** paths from the user or model (a workspace-root convention, an in-app file picker, a "read this project file" affordance): define explicitly what relative paths resolve against — most likely a framework/workspace root the Tauri shell resolves (the CLAUDE.md §9 "Tauri-shell-resolves-directory" archetype) and passes into the executor — rather than inheriting the process CWD. Resolve relative paths against that root (and scope-check against it) so a user's `./data/x.txt` means the same thing regardless of where the app was launched. Decide alongside the rung that first surfaces relative paths; until then, absolute paths are unambiguous and rung 1 needs no change.

## TD-036 — Production never wires the user's tier into the run-loop enforcer (Tester + smoke always run at Novice)

**Date logged:** 2026-05-31
**Found by:** M08.7.B rung-2 ground-at-red (grep for `set_tier` callers; maintainer-approved disposition — Option 1, test-seam + log TD)
**Pass that surfaced it:** N/A (ground-at-red investigation during rung-2 implementation)
**Category:** other (painted-not-wired — capability/tier enforcement; cf. [[TD-034]])
**Resolution status:** open (routed to the live-session / tier-wiring rung)

### Description

No production code path calls `CapabilityEnforcer::set_tier` anywhere — grep confirms the only callers are `enforcer.rs`'s own unit tests. Both run-loop entry points build a fresh `CapabilityEnforcer::new()` and never set its tier:

- `run_test_session_with` (`crates/runtime-main/src/builder/tester.rs`) — the Builder Tester.
- `run_smoke_session_with` (`src-tauri/src/commands.rs`) — the live smoke session.

`CapabilityEnforcer::new()` defaults to `Tier::Novice` (`enforcer.rs:64`), so **every** production agent run executes at Novice regardless of the user's actual tier. The app *tracks* the user's tier (`CurrentTierState` / the `get_current_tier` command), but that value is never pushed into the enforcer that gates execution. The enforcer's own doc-comment (`enforcer.rs:65-66`) describes the intended wire ("the Tauri layer loads the persisted tier at app startup and calls `set_tier` before any dispatch runs") — that wire was never built.

### Why it's debt not bug

In v0.1 the observable consequence is *conservative*, not unsafe: Novice is the most restrictive tier (it forbids Write and most non-Read kinds at L4), so a run-loop stuck at Novice can only **under**-grant, never over-grant — it cannot let an agent do something the user's actual tier forbids. The capability boundary is intact; it is just pinned to the safe end. A Promoted user simply cannot exercise Promoted-tier execution (e.g. a built-in Write) through the assembled app yet — the feature is absent, not mis-enforced.

This is the same "painted, never wired" class as the M08.7 thesis ([[TD-034]]): the tier primitive (`Tier`, `TierEvaluator`, `set_tier`, the L4 gate) is fully built and unit-tested; only the production wire from the persisted/tracked tier into the run-loop enforcer is missing.

Rung 2's consequence: the Promoted-tier `file_access`-scope Write denial is only expressible through the **test-path** seam `run_test_session_with_tier` (the assembled integration test sets `Tier::Promoted`). The real-app Tester runs at Novice, so a real-app out-of-scope Write is **tier**-denied (`TierViolation`) before the scope gate is reached — the scope-gate `CapabilityViolation(Write)` is NOT observable in the running app today. Rung 2 deliberately did NOT wire production tier (Hard Rule 8 / ADR-0019 — out of the in-process-Read/Write scope lock).

### Recommended approach (when addressed)

Wire the tracked/persisted user tier into the run-loop enforcer at the live-session / tier-wiring rung: have the Tauri shell read `CurrentTierState` (or `tier.json`) and pass the tier into `run_test_session_with_tier` (already plumbed) and an analogous `run_smoke_session_with` tier param, calling `enforcer.set_tier(tier)` before the first dispatch — and re-apply on a tier transition mid-session (the `tier::transition` path already documents routing through `set_tier`). That makes runtime execution actually gate on the user's tier, and lets the maintainer observe the Promoted scope-gate `CapabilityViolation(Write)` in the running app (pairs with [[TD-034]]'s in-app agent-output surfacing — both are needed for a real-app IRL of the scope gate). Touches `src-tauri/src/commands.rs` (the `test_framework` + `run_smoke_session` commands) + `tester.rs` (already has the tier seam). ADR-0019 amendment if the Tester's tier model (always-Novice-sandbox vs user-tier) is a product decision.

## TD-037 — Real-app Builder-Tester cannot thread skill bodies (v0.1 canvas authors no companions); skill-load real-app IRL deferred

**Date logged:** 2026-05-31
**Found by:** M08.7.C rung-3 close — construction_reachability_check on the production `companions → resolved_skills` join (maintainer-directed: "wire it, or STOP and surface if walled deeper than expected — we'll re-tier")
**Pass that surfaced it:** N/A (rung-3 production-wire reachability check)
**Category:** other (painted-not-wired — renderer↔main + canvas state; cf. [[TD-034]], [[TD-036]])
**Resolution status:** open (re-tiered — the real-app Builder-Tester skill IRL is deferred to the canvas-skill-body rung; rung 3's behavior close rests on the live eval instead)

### Description

Rung 3's `LoadSkill` handler reads the resolved skill body from a
`resolved_skills` map threaded onto `CapabilityWiring`. The assembled
test (`skill_load_execution.rs`) and the live eval (`skill_load_live.rs`)
pass that map directly. The **production** path — the Tauri
`test_framework` command building the map from the loaded framework's
companions and passing it to `run_test_session_with_skills` — is **NOT
wired**, because the companions are not reachable there:

- `test_framework(framework_doc: Framework, task)` (`src-tauri/src/commands.rs:1630`)
  receives only a bare `Framework` straight from the canvas (spec Phase 9
  "does NOT need to save first") — **not** a `LoadedFramework`. No
  companions.
- The renderer holds a `companions` array (it passes it to
  `save_framework`), but `testFramework` (`src/lib/ipc.ts:618`) sends only
  `{ frameworkDoc, task }`.
- Decisively: in v0.1 the canvas **authors no skill bodies at all** —
  `src/lib/ipc.ts:504-505`: "`companions` defaults to `[]` — the v0.1
  canvas authors no inline markdown bodies (M09's Generators will)." So
  there is no skill-body companion in renderer state to thread, even with
  an IPC change.

Wiring only the backend join would be a **dead path** the v0.1 renderer
can never feed (always `[]`) — the exact paint-not-execute anti-pattern
M08.7 exists to stop (rule 11). It was therefore NOT built; the finding
was surfaced and re-tiered per the maintainer's standing instruction.

### Why it's debt not bug

Rung 3 is correct and proven: the `LoadSkill` handler + the `drive_stream`
branch + the injection-into-context are exercised by the assembled test
(CI, structural) and the `#[ignore]`d live eval (real Anthropic,
behavior-change — the encoded IRL). The gap is purely the **production
renderer→shell threading** of an already-resolved skill body, which v0.1
cannot feed because the canvas has no skill bodies (by design — M09
Generators author them). No behavior is wrong; the in-app Builder-Tester
"load a skill, watch it change behavior" affordance is **absent**, not
mis-wired.

### Recommended approach (when addressed)

At the rung that gives the canvas skill bodies (M09 Generators, or a
canvas skill-body editor), complete the chain together: (1) the canvas
holds/authors skill-body companions; (2) `testFramework` +
`test_framework` accept `companions: Vec<Companion>` (the IPC contract
change); (3) `test_framework` joins `framework.skills[].path ↔
companions[].file_name` into a `resolved_skills` map and calls
`run_test_session_with_skills` (the seam is already plumbed); (4) an
`e2e-tauri/` regression drives the real app (per ADR-0021). Until then,
the skill-load behavior close is the live eval (`skill_load_live.rs`),
run with `ANTHROPIC_API_KEY` set — a real-model observation through the
real `run_test_session_with_skills → run_agent → drive_stream` path,
just not through the Builder UI. Pairs with [[TD-034]] (no in-app agent
output) — both are needed for an in-Builder skill IRL.

## TD-038 — Budget threshold has no OS desktop notifier (v0.1 = in-app event/toast only)

**Date logged:** 2026-06-01
**Found by:** M08.7.E rung-5 ground-at-entry — the ThresholdAction→event dispatch mapping (phase doc E.3.2 names "budget_warn event + a notifier dispatch (the NotifierDispatched path)")
**Pass that surfaced it:** N/A (rung-5 dispatch-mapping scope decision, surfaced before red)
**Category:** other (scoped-out side effect — OS integration; cf. the desktop-notifier seam)
**Resolution status:** open (scoped out of rung 5 — the in-app budget events cover v0.1; the OS desktop notification is the deferred bit)

### Description

Rung 5 wires the four budget `ThresholdAction`s to their existing
`AgentEvent` variants — `Warn → BudgetWarn`, `Downshift →
BudgetDownshift`, `Suspend → BudgetSuspended`, `HardStop →
BudgetExceeded` (all pre-existing in `schemas/event.v1.json` +
`crates/runtime-core/src/event.rs` — no schema change). The renderer's
budget toast / header-bar amber shift is **event-driven** by `BudgetWarn`
(the schema doc for the warn threshold: "surfaces a toast notification +
shifts the BudgetHeaderBar color toward amber").

The phase doc E.3.2 also names "a notifier dispatch (the
`NotifierDispatched` path)" for the Warn action. That was **deliberately
NOT wired** in rung 5: `AgentEvent::NotifierDispatched`
(`event.rs:596`) is structurally bound to a `HitlTriggerRef` (it reports
"a notifier successfully fired **for a HITL request**") — emitting it for
a non-HITL budget warning would mean fabricating a trigger that does not
fit the budget-warn semantics. The in-app toast (driven by `BudgetWarn`)
is the v0.1 user-visible signal; the **OS desktop notification** (the
`terminal_bell` / `desktop` / `sound` notifier types `NotifierDispatched`
carries) is the deferred surface.

### Why it's debt not bug

No behavior is wrong: the budget warning IS surfaced to the user (the
`BudgetWarn` event → renderer toast/header bar). The gap is purely the
OS-level desktop notification — a nicety on top of the in-app signal, not
a missing enforcement. The load-bearing rung-5 safety primitive
(`HardStop` halts the run; `BudgetExceeded` + no further provider turns)
is unaffected.

### Recommended approach (when addressed)

When the desktop-notifier seam is generalized beyond HITL (the same seam
M04.E's `on_budget_threshold` HITL notifier uses), route the budget
`Warn` / `Suspend` actions through it. That likely wants either a
budget-flavored `NotifierDispatched` trigger variant (a §11 schema
change — ADR + version bump) or a separate budget-notifier event. Pairs
with the budget-`Suspend` resume work folded into the gap resolve-and-resume
rung ([[ADR-0029]], generalized to budget).

---

## TD-039 — Rung 1 has no explicit "two reads in one session both feed back" multi-call test

**Date logged:** 2026-06-02
**Found by:** M08.7.V multi_call pass (🟢 #2)
**Pass that surfaced it:** Multi-call invariants
**Category:** other (test-completeness)
**Resolution status:** open

### Description

The built-in-tool multi-call invariant — *two `Read`s in one session both
feed their results back* — has no dedicated assertion in
`runtime-main/tests/builtin_tool_execution.rs`. The mechanism is proven
**indirectly**: the single-read→quote path (rung 1) exercises one
dispatch→feedback cycle, and rungs 3/5 exercise repeated dispatch across
turns. There is no explicit two-reads-both-feed-back assertion for the
built-in surface specifically.

### Why it's debt not bug

Rung 1 is correct and proven (17/17 assembled tests; the agent quotes the
file it Read). The multi-turn loop demonstrably re-streams until a turn
dispatches no tool, so the feedback cycle repeats by construction. The gap
is a missing explicit multi-call test for the built-in surface, not a
behavior defect — M08.7.V graded it 🟢, not a blocker.

### Recommended approach (when addressed)

Add a `builtin_tool_execution.rs` case where the agent issues two `Read`s
across two turns and assert each result appears in its respective next-turn
`AgentConfig`. Low effort; fold into any rung-1 touch.

---

## TD-040 — Rung 4 "recoverable" BDD leg proven indirectly (no assembled snapshot-rebuild assertion)

**Date logged:** 2026-06-02
**Found by:** M08.7.V behavior pass (🟢 #3)
**Pass that surfaced it:** Behavior
**Category:** other (test-completeness)
**Resolution status:** open

### Description

The rung-4 gap BDD has a "recoverable" leg (the suspended gap is persisted
per §1b so the session can later resume). `gap_detection_execution.rs`
proves the gap event is persisted (via `self.emit` → the signals sink) and
the session suspends cleanly (exactly one provider turn; no `ToolInvoked`
for `request_capability`), but it does **not** assert an assembled
snapshot-**rebuild** (reload from the snapshot chain → the suspended state
reconstructs). The load-bearing "suspend cleanly" half is grounded; the
"recoverable" half is inferred from the persistence call, not observed by a
rebuild.

### Why it's debt not bug

Rung 4's headline behavior — `request_capability` suspends the session
cleanly — is grounded by execution (6/6 assembled tests). Recovery rebuild
is a §1b mechanism already exercised elsewhere (`recovery_lifecycle`
tests). The gap is a missing rung-4-specific rebuild assertion. The gap
**resume** path is itself M08.9.F (ADR-0029) — this TD is about the test
assertion, not the unbuilt resume.

### Recommended approach (when addressed)

When M08.9.F (gap resolve-and-resume) lands, its assembled test asserts the
snapshot-rebuild → resolved `tool_result` → continue; that test subsumes
this leg. Until then, optionally add a `gap_detection_execution.rs`
assertion that the suspended gap is present in the persisted snapshot chain.

---

## TD-041 — `ToolUse` JSON input-field extraction repeats across dispatch handlers

**Date logged:** 2026-06-02
**Found by:** M08.7 closeout simplify-pass (RU-M08.7-1)
**Pass that surfaced it:** Simplify-pass (reuse)
**Category:** reuse
**Resolution status:** open

### Description

`dispatch_load_skill` and `dispatch_request_capability` in
`crates/runtime-main/src/sdk/agent_sdk.rs` both extract a string field from
the `ToolUse` `input` JSON with the same shape
(`input.get(key).and_then(Value::as_str).unwrap_or_default().to_string()`).
`builtin_tools.rs` uses a narrower single-field variant. A shared
`tool_input_str(input, key)` helper would compress the two-to-three sites.

### Why it's debt not bug

Both sites are small, correct, and clear in place. Per CLAUDE.md §9
("wait for the fourth") the pattern is below the extraction bar today — two
production sites + one near-variant. No behavior concern; this is a reuse
opportunity that only pays off once a fourth caller lands.

### Recommended approach (when addressed)

When a fourth `ToolUse`-input-parsing handler emerges (M08.9 sub-agent /
plan dispatch, or v1.0 generators), extract `tool_input_str` to a shared
helper at the dispatch boundary that needs it.

---

## TD-042 — `dispatch_budget_actions` carries `current_model` mutation across the action loop

**Date logged:** 2026-06-02
**Found by:** M08.7 closeout simplify-pass (EFF-M08.7-1)
**Pass that surfaced it:** Simplify-pass (efficiency/clarity)
**Category:** other (clarity-on-future-extension)
**Resolution status:** open

### Description

`dispatch_budget_actions` (`crates/runtime-main/src/sdk/agent_sdk.rs`)
seeds `current_model` from the turn's model and mutates it in place on a
`Downshift` action so a subsequent action in the same `actions` vec sees the
updated model. The logic is correct for today's action set (`Warn` /
`Downshift` / `Suspend`), but the cross-iteration mutable state would become
subtle if a future rung adds a third model-aware action or allows an action
to repeat.

### Why it's debt not bug

The current form is correct and necessary for the warn→downshift ordering;
the simplify-pass explicitly recommended no refactor now. This is a
clarity flag for future extension, not a defect.

### Recommended approach (when addressed)

If a second model-aware budget action lands (or the action set grows),
revisit whether the per-action model should be derived rather than carried
as mutable loop state.

## TD-043 — `TesterModal` not migrated onto the `Modal` primitive

**Date logged:** 2026-06-03
**Found by:** M08.8.B (Light Instrument visual foundation)
**Pass that surfaced it:** Stage-B implementation (Modal-primitive consolidation)
**Category:** other (consolidation / duplicate-implementation)
**Resolution status:** RESOLVED at M08.8.B.fix — `TesterModal` now renders
`<Modal size="full" testId="tester-modal">`; the hand-rolled `.tester-modal`
overlay CSS + its `tester-modal__*` test pins were removed (the unit + the
Playwright close-selector rewrite rode the B.fix RED commit). Two modal
implementations no longer coexist.

### Description

M08.8.B shipped the `TesterModal` full-screen *visual* via a CSS repaint of
its existing `.tester-modal*` classes (z-300, 96vw × 92vh, Light Instrument
tokens), NOT a migration onto the reusable `Modal` primitive
(`<Modal size="full">`, B.3.4). Two modal implementations therefore coexist:
the `Modal` primitive (overlay/scrim/z-index/focus-trap/Esc, used by
`MCPServerAddModal`) and `TesterModal`'s hand-rolled overlay.

### Why it's debt not bug

The migration is **test-bearing**: `TesterModal.test.tsx` pins
`tester-close` / `tester-modal__close` / `tester-modal__header` (testids +
a CSS-rule-exists check), and strict-TDD (`CLAUDE.md` §5) forbids editing
those tests in an impl commit. A clean swap onto `Modal` would change those
selectors, so it cannot land as a pure repaint mid-stage. The shipped CSS
repaint is correct and full-screen; this is a consolidation flag, not a
defect.

### Recommended approach (when addressed)

A later stage's **red phase** rewrites the `tester-modal__*` tests against
the `Modal` primitive's structure (the primitive's `data-testid`/role +
a Tester-owned close affordance), then the impl migrates `TesterModal` onto
`<Modal size="full">` and deletes the duplicate `.tester-modal` overlay CSS.
Ref B.3.4.


---

## TD-044 — `smoke.e2e.ts` "reload reconstructs the graph" is key-dependent + races the post-reload rebuild

**Date logged:** 2026-06-03
**Found by:** M08.8.B.fix (first full `test:e2e:tauri` run to completion)
**Pass that surfaced it:** real-app `tauri-driver` suite (design-conformance close)
**Category:** test-infra (real-app spec flake / environment dependency)
**Resolution status:** open — pre-existing; NOT a B.fix exposure (deferred, do not chase in B.fix)

### Description

`tests/e2e-tauri/smoke.e2e.ts` → "reload reconstructs the graph from persisted
signals" waits for `[data-testid^="agent-node-"]` after a window reload and
times out (15s) when the key-present branch runs: the spec first performs a
**real Anthropic smoke run** (it needs a live `ANTHROPIC_API_KEY`/keychain
entry), persists signals, reloads, and expects the graph to rebuild from the
persisted signal chain. On the build machine (key present) the post-reload
rebuild did not surface a node in the window; CI runs keyless and skips the
real-run leg, so this never blocked merge.

### Why it's debt not a B.fix bug

B.fix is renderer/CSS only — `git diff 47e10a2..HEAD --name-only` shows it
touched **zero** persistence / signal-replay / `commands.rs` / store
reconstruction code (only `styles.css`, 5 components, fonts, tests). The
reload-rebuild path is unchanged since before M08.8; this spec was simply
never run to green in A/B (B's retro left `test:e2e:tauri` "pending"), so the
first completed run surfaced a **pre-existing** key-present timing/rebuild
issue. The `agent-node-*` testid the spec waits on was not altered by the
restyle.

### Recommended approach (when addressed)

Separate triage (not B.fix): (1) confirm whether the post-reload rebuild
genuinely fails or merely races (add a `waitUntil` on the reconstructed graph,
mirroring the TD-044-sibling `builder_load_aria` deflake), and (2) decide the
key-present vs CI-keyless contract for this leg — gate the real-run+reload
assertion behind a key-present guard so the spec is deterministic in both
environments.
