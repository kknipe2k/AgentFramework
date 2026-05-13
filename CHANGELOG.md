# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — M05.B §8.security L1 + L2a Capability Enforcer (new safety primitive ≥95%)

Code + schema. M05 Stage B ships the in-process capability enforcer + L2a
narrowing evaluator as a new safety primitive at 100% per-module coverage.
Default-deny semantics: an agent with no declared grants is rejected; the
asymmetric `parent.subsumes(child)` predicate is the load-bearing
invariant proptest-verified. Renderer applyEvent branches lit up for
`capability_violation` + `capability_grant` (previously no-op).

- **`schemas/capability.v1.json`** *(new)* — `CapabilityDeclaration`
  shape: `kind` (`CapabilityKind` enum: `read | write | exec | network |
  process_spawn`), `resource` (newtype `ResourceName`, minLength 1),
  `scope` (`CapabilityScope` oneOf: `GlobScope { glob }` / `DomainScope
  { domain }` / `PathScope { path }`), `side_effect_class`
  (`SideEffectClass` enum: `pure | filesystem_mutate | network_egress |
  process_spawn | irreversible`). Per gotcha #43 every validated inline
  string is extracted to a titled `$def` so typify generates clean
  newtypes (`GlobPattern`, `DomainPattern`, `PathPattern`,
  `ResourceName`).
- **`schemas/event.v1.json`** — `capability_violation` enriched with
  `agent_id` + `capability_kind` + `requested_action` + `declared_scope`
  (was: `declared` + `attempted`). `capability_grant` enriched with
  optional `parent_agent_id` + `granted_to` + `capability_kind` +
  `resource` + optional `narrowed_from` (was: `agent_id` +
  `capability` + `scope`). New `$defs`: `CapabilityKindRef`,
  `RequestedAction`, `DeclaredScope`, `GrantedResource`.
- **`crates/runtime-core/src/generated/{capability,event}.rs` +
  `src/types/{capability,agent_event}.ts`** — regenerated via
  `cargo xtask regenerate-types`.
- **`crates/runtime-core/src/event.rs`** — hand-rolled canonical
  `AgentEvent` mirrors the schema enrichment. `CapabilityViolation` +
  `CapabilityGrant` variants replaced with the enriched payload. Added
  `CapabilityKindRef` enum (5 values; follows the
  `HitlTriggerRef` / `GapSeverityRef` cross-schema mirror pattern).
- **`crates/runtime-main/src/capability/`** *(new module)* —
  - `mod.rs` — module root + re-exports.
  - `declaration.rs` — pure-function `subsumes(parent, requested)` +
    `scope_contains(outer, inner)` per-variant containment (glob via
    `globset::Glob`; domain with leading-`.` subdomain support; path
    prefix-with-separator). 15 unit tests; 100% line coverage.
  - `enforcer.rs` — `CapabilityEnforcer` struct + `check` /
    `grant` / `grants_for` / `grant_count` API. Default-deny: agent
    with no entries gets `Err(Denied { reason: NoDeclarations })`.
    `DenyReason` discriminates `NoDeclarations` vs `NoMatchingGrant`
    for renderer copy. 11 unit tests; 100% line coverage.
  - `narrowing.rs` — `narrow(parent, proposed)` evaluator enforces
    "child grants ⊆ parent grants" on sub-agent spawn. Short-circuits
    on first uncovered proposed declaration. 7 unit tests + 2 proptest
    properties (`property_narrowing_preserves_invariant` +
    `property_widening_always_denied`); 100% line coverage.
  - `error.rs` — `CapabilityError::Denied { agent_id, reason }` +
    `NarrowingError::CapabilityNotHeldByParent { proposed }`.
- **`crates/runtime-main/tests/capability_enforcer_smoke.rs`** *(new)*
  — 6 integration tests stand in for the SDK's eventual `dispatch_tool`
  wrap (D1 in M05.B retrospective — no production dispatch path in v0.1
  yet). Covers: grant→success+capability_grant emission;
  no-grants→denial+capability_violation emission BEFORE err returns
  (gotcha trap #4 ordering); declarations-exist-but-no-match path;
  L2a narrowing emits per-narrowed-grant; widening denied; multi-call
  invariant.
- **`crates/runtime-core/tests/round_trip.rs`** — extended
  `agent_event_capability_violation_round_trip` for the new shape +
  added `agent_event_capability_grant_{root,narrowed}_round_trip`.
- **`src/lib/graphStore.ts`** — replaced the M04-era no-op cases for
  `capability_violation` + `capability_grant` with real `applyEvent`
  branches. New state slots: `capabilityViolations:
  Record<agentId, CapabilityViolationRecord>` (last-write-wins per
  agent) + `capabilityGrants: CapabilityGrantRecord[]` (append-only
  log). `clear()` resets both.
- **`tests/unit/graphStore.test.ts`** — new "capability events (M05
  Stage B)" describe block: 6 tests covering violation state recording,
  grant log append, narrowed-grant metadata, multi-call append-only
  invariant, and clear reset.
- **`.prettierignore` + `eslint.config.js`** — added
  `src/types/capability.ts` to both ignore lists per gotcha #44.
- **`crates/xtask/src/main.rs`** — wired `capability` into the
  schemas list + the TS-targets list so `cargo xtask regenerate-types`
  produces the new Rust + TS bindings.

D1 (SDK wire-up): the production SDK has no `dispatch_tool` /
`spawn_sub_agent` path to wrap yet (M02-shipped single-turn streaming
only); the smoke test stands in as the canonical wrapping shape so
the enforcer's check + grant + narrow contract is exercised end-to-end.
The phase doc's `<execution_warnings>` explicitly authorizes this
scoping. M06+ wires the enforcer to the live dispatch path when
multi-turn tool loops land.

Not in this stage: sandbox subprocess (Stage C1+C2), tier system
(Stage D), audit log (Stage E), capability-violation modal +
CapabilityBadge UI (Stage F).

Coverage: workspace 94.29% line; runtime-drone 95.79% line;
runtime-main 97.16% line; `capability/declaration.rs` 100%;
`capability/enforcer.rs` 100%; `capability/narrowing.rs` 100%.
All ≥ gates.

### Added — M05.A §4b Gap Detection (framework_loader + request_capability meta-tool + schema enrichment + M04.V carry-forwards)

Code + schema. M05 Stage A wires spec §4b Layer 1 (framework_loader) +
Layer 2 (request_capability meta-tool) gap detection end-to-end. Enriched
gap-event payload per spec §4b severity matrix; new `mcp_missing` +
`agent_missing` variants; `ContextType` reconciled with spec §2b
(M02/M03/M04 carry-forward closed); M04.V Decision 1 absorbed via
TaskNode regression test.

- **`schemas/event.v1.json`** — added `mcp_missing` + `agent_missing`
  event variants; enriched the four `*_missing` variants with `severity`
  (`GapSeverity` enum: `critical | important | advisory | requested`),
  `suggested_action` (validated minLength 1), and `requested_via`
  (`GapSource` enum: `loader | request_capability`). New `$defs`:
  `GapSeverity`, `GapSource`, `SuggestedAction`. Per gotcha #43 the
  validated string extracts to `$defs/SuggestedAction` so typify generates
  a clean newtype.
- **`crates/runtime-core/src/generated/event.rs` + `src/types/agent_event.ts`**
  — regenerated via `cargo xtask regenerate-types`.
- **`crates/runtime-core/src/event.rs`** — hand-rolled canonical
  `AgentEvent` union mirrors the schema enrichment. Added
  `GapSeverityRef` + `GapSourceRef` enums (following the existing
  `HitlTriggerRef` cross-schema mirror pattern). The four `*_missing`
  variants gain the enriched payload.
- **`crates/runtime-main/src/framework_loader/`** *(new module)* —
  - `mod.rs` — `Emitter` trait (in-process event seam) +
    `load_and_validate` async wrapper + `load_and_validate_str` test seam.
  - `walker.rs` — pure-function walker over `Framework`: checks every
    inline `Agent`'s `allowed_tools[]` / `allowed_skills[]` / `spawns[]`
    against the framework's declared primitive sets, returns `Vec<Gap>`
    with per-kind severity per spec §4b severity matrix. `mcp_missing`
    is Layer-2-only in v0.1 (v0.1 framework schema declares no MCP
    servers; M06 adds Layer-1 emission). 9 unit tests; 100% line on
    `to_event` mapping.
  - `error.rs` — `FrameworkLoadError { Io, Json, GapsFound }`.
- **`crates/runtime-main/src/sdk/request_capability.rs`** *(new module)*
  — spec §4b Layer 2 meta-tool. `CapabilityKind { Tool, Skill, Mcp, Agent }`
  + `RequestCapabilityInvocation` + `handle_request_capability` emits the
  matching `*_missing` event with `severity: Requested` +
  `requested_via: RequestCapability` and returns `Pending`. M05.A
  authoring decision: meta-tool accepts 4 kinds (spec §4b text says 2 —
  surfaced in retro for reconciliation). 6 unit tests.
- **`crates/runtime-main/tests/framework_loader_smoke.rs`** *(new)* —
  integration test against `examples/aria/framework.json`: valid framework
  loads with zero gaps + multi-call invariant (gotcha #69).
- **`crates/runtime-core/src/signal.rs::ContextType`** — reconciled with
  spec §2b. Old variants (`AgentLoop / SkillLoad / ToolInvoke /
  HookExecute / PlanCreate / HitlPrompt / SessionLifecycle`) replaced by
  spec set (`Skill / Framework / Code / Search / Verify / Commit /
  Subagent`). M02 + M03 + M04 carry-forward CLOSED.
- **`src/lib/graphStore.ts`** — lit up `applyEvent` branches for all
  four `*_missing` + `gap_resolved`. GapNodes mount keyed by
  `${kind}:${missingName}:${agentId}` (idempotent re-emission;
  loader-vs-meta-tool re-emission of same gap collapses with latest
  severity). `GapNodeData` extended with `agentId`, `severity`,
  `suggestedAction`, `requestedVia`; `kind` widened to the 4-variant
  union.
- **`src/components/nodes/GapNode.tsx`** — renders severity-tier CSS
  modifier class + `suggestedAction` text + DOM-readable
  `data-kind` / `data-severity` / `data-requested-via` discriminators so
  e2e + unit tests pin the wire-path contract (gotcha #66 / #68).
- **`tests/unit/nodes/TaskNode.test.tsx`** — added M04.V Decision 1
  regression test `renders_task_id_prefix_fallback_when_title_is_empty`
  pinning the LG-02 IRL fix at `TaskNode.tsx:27`.
- **`tests/unit/nodes/GapNode.test.tsx`** — 7 tests for enriched payload
  rendering + 4-kind visual differentiation + accessibility.
- **`tests/unit/graphStore.test.ts`** — 7 tests for the new gap-event
  applyEvent branches (per-kind mount + idempotence + latest-wins on
  re-emit + `gap_resolved` dismissal + safe-noop on unknown kind).
- **`.prettierignore` + `eslint.config.js`** — added `src/types/budget.ts`
  to ignore lists (M04.F oversight surfaced by M05.A regeneration; per
  gotcha #44 every generated TS file goes in both ignore lists).

M04.V Decision 2 (§4a `hook_*` vs `verify_*` spec/code naming) surfaced
in `docs/build-prompts/retrospectives/M05.A-retrospective.md` for
maintainer adjudication — no code change in this stage.

Not in this stage: capability enforcer (Stage B), sandbox subprocess
(Stage C1+C2), tier system (Stage D), audit log (Stage E), GapPanel UI +
CapabilityBadge (Stage F).

Coverage: workspace 94.06% line; runtime-drone 95.79% line;
runtime-main 96.94% line; `framework_loader/walker.rs` 98.18% line;
`framework_loader/mod.rs` 99.17% line; `sdk/request_capability.rs`
98.33% line. All ≥ gates.

### Added — M04.6 protocol iteration (Stage V Verifier introduced, validator extended, M04 IRL bug patterns graduated to gotchas)

Documentation + protocol. Adds the Stage V (Verifier) ceremony between work
stages and closeout for M05 onward; extends the existing schema validator
to recognize the new third schema variant; graduates seven M04 IRL bug
patterns into `docs/gotchas.md`. M01–M04 grandfathered as v1.0/v1.2/v1.3/v1.4
per their existing banners — M04 receives a retroactive V run (no Phase doc
edit, no gap-analysis edit; findings land in
`docs/build-prompts/retrospectives/M04.V-retrospective.md` only). Per
ADR-0008 + `STAGE-PROMPT-PROTOCOL.md` v1.5 §14.

- **`docs/adr/0008-milestone-stage-v-verifier.md`** *(new)* — records the
  decision to adopt Stage V; three alternatives rejected on substance
  (strengthened-closeout, continuous per-stage hooks, automated-coverage-only).
  Calibrated against M04's five IRL bug classes; fourth Behavior pass added
  in round 4 of the design review after the BudgetHeaderBar-CSS bug was
  identified as static-uncatchable.
- **`STAGE-PROMPT-PROTOCOL.md`** — v1.4 → v1.5 bump. Adds the
  `<verifier_stage_prompt>` schema variant (new §14) with five required tags
  (`<scope_to_verify>`, `<verification_passes>`, `<findings_format>`,
  `<merge_gate>`, plus common tags adapted for V), three forbidden tags
  (`<read_prior_stages>`, `<deliverable>`, `<test_plan_required>`), and four
  verification passes (Inventory + Wire + Behavior + Multi-call invariants).
  §4 (programmatic extraction) + §5 (the schemas) updated to acknowledge the
  third variant. v1.5 changelog entry in §15 (renamed from §14 Versioning).
- **`bin/validate-stage-prompts.mjs`** — extended to recognize
  `<verifier_stage_prompt>` as a third root variant + enforce the bias-guard
  rule (V's `<read_first>` must NOT reference per-stage retros, milestone
  summaries, or gap-analysis ledger paths).
- **`docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md`** *(new)* — parameterized
  prompt template. Per-milestone V prompt is a copy + parameterization of
  this template into the milestone's V.5 section. Documents the
  clear-and-paste session pattern (the bias guard).
- **`docs/build-prompts/TEMPLATE.md`** — adds the Stage V section template
  parallel to Stage A–D + Closeout. Six subsections (V.1 Problem statement,
  V.2 Scope to verify, V.3 Verification passes, V.4 Findings format, V.5 CLI
  prompt, V.6 Commit message). M01–M04 phase docs predate the protocol and
  are grandfathered; M05+ phase docs include the Stage V section.
- **`docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md`**
  *(new)* — per-V retrospective shape. Verification axes (coverage adequacy,
  finding signal-to-noise, fresh-context discipline) replace work axes.
  Outcomes (Sound / Sound-but-rough / Friction-heavy / Not-ready) feed merge
  gating rather than next-stage prep.
- **`docs/tech-debt.md`** *(new)* — append-only ledger for Stage V 🟢
  findings. Distinct from `gap-analysis.md` (product↔spec drift) and
  `gotchas.md` (don't-do-this patterns). Seeded empty.
- **`docs/gotchas.md`** — entries #66–#72 graduated from M04 IRL findings:
  meta-gotcha "tests-pass-but-contract-fails is a distinct bug class"; the
  CSS-rule-missing pattern; the wrong-field-read pattern; the multi-call
  invariant pattern; the viewport-CSS-assumption pattern; the schema-field-
  missing-renders-blank pattern; the tokio-duplex-EOF-propagation pattern
  (the bug that hung CI on PR #64). Each ties to a specific M04 IRL test ID
  + PR #64 regression test.
- **`CLAUDE.md`** — §17 (reference index) adds the four new files
  (`STAGE-V-VERIFIER-PROMPT-TEMPLATE.md`, `VERIFIER-RETROSPECTIVE-TEMPLATE.md`,
  `tech-debt.md`, validator script). §19 (retrospective protocol) adds
  Stage V as a fourth retro type and introduces the inline summary of the
  protocol with pointers to ADR-0008 + `STAGE-PROMPT-PROTOCOL.md` §14.

### Added — M04 Stage G (Phase Closeout — gap analysis + parent-milestone summary)

Final stage of M04. Documentation-only; no code changes. Per CLAUDE.md §20:

- **`docs/gap-analysis.md`** — appended the immutable M04 entry. Cumulative
  product↔spec audit across M01 + M02 + M03 + M03.5 + M04. Six sections per
  the entry template (Codebase deep dive; Adherence to spec; Spec review
  forward-looking; Fix backlog; Carry-forward from prior milestones; Sign-off)
  plus the second-of-its-kind `<gotchas_graduation>` subsection covering 42
  per-stage gotchas + friction events across A1–F with disposition (12
  resolved, 23 graduated, 7 kept, 0 expired). M02 + M03 carry-forward final
  disposition recorded; M03.5 carry-forwards (v1.3 protocol tags + 12
  docs/gotchas.md graduations) all applied; v1.4 protocol candidates surfaced
  (`<architecture_check>` + `<schema_audit>` + `<schema_root_check>` +
  `<phase_doc_inventory_audit>` + `<safety_primitive_coverage_path>`).
- **`docs/build-prompts/retrospectives/M04-summary.md`** — new parent-milestone
  roll-up aggregating M04.A1–F retrospectives. Aggregate scoring (Process
  38.43/40, Product 37.43/40, Pattern 29.57/35); time-box accuracy (~0.55×
  mean ratio, ~20h actual against 36h estimated); cross-stage trends; verdict
  `Pattern held across M04`. Decisions to apply before M05 enumerated.
- **CHANGELOG.md** — this entry.

Append-only invariant verified: `git show origin/main:docs/gap-analysis.md`
diff against the head N lines of the local file returns empty (prior M01 +
M02 + M03 entries unchanged).

### Added — M04 Stage F (§2a Budget + §1b Recovery — cost controls + resume from snapshot)

Seventh stage of M04. Bundles two primitives in one stage: §2a Budget (3 scopes
+ 4 threshold actions + downshift_hook + UI header bar) + §1b Recovery (resume
rebuilds history not re-execute per WI-14 + tool-call-uncertain UI prompt with
4 actions + MCP reconnect seam + plan/capability state restoration).

**Decisions documented in M04.F retrospective:**
- The existing 4 budget event variants (`budget_warn`, `budget_downshift`,
  `budget_suspended`, `budget_exceeded`) had a provisional minimal shape that
  diverges from spec §2a (`scope` field missing on all; `spent_usd`/`cap_usd`
  missing on `BudgetDownshift`; the `budget_warn` discriminator should be
  `budget_warning` per spec). Stage F WIRES the existing events as-is rather
  than reshaping; the divergence becomes a Stage G gap-analysis carry-forward
  entry. Rationale: Stage F's deliverable is the enforcer + recovery; touching
  existing public event shapes would balloon scope.
- The downshift hook ladder is hardcoded in `runtime-main/src/budget/hook.rs`
  (`DefaultLadder`) per the spec §2a `opus → sonnet → haiku` rule. Framework
  JSON's `framework.budget.downshift_hook.tool_name` is read at schema-codegen
  time but the framework-tool-dispatch wiring is deferred to M5/M9 generators —
  the hook trait exists at the seam so later milestones can plug in a
  framework-defined tool without changing call sites.
- Drone IPC adds a new `DroneCommand::RecoverSession` + `DroneEvent::SessionRecovered`
  pair rather than reusing `QuerySessionDb` or `ReadSignals`. The drone-side
  `snapshot::recover_session_state` (shipped M04 Stage B) was already complete;
  Stage F exposes it via the IPC variant and consumes it through a
  `DroneClient::recover_session(session_id)` method.

**New artifacts:**
- **`schemas/budget.v1.json`** (new) — `BudgetPolicy` (session/framework caps +
  actions + downshift_hook), `BudgetActions` (4 percent thresholds with spec §2a
  defaults), `BudgetScope` enum (session/framework/global), `DownshiftHook`
  (type=`tool` + tool_name). Concrete `type: object` at root per Stage E
  gotcha #29 carry-forward.
- **`crates/runtime-core/src/generated/budget.rs`** (typify),
  **`src/types/budget.ts`** (json-schema-to-typescript) — generated and
  re-exported via `runtime_core::budget` + `src/types/budget.ts`.
- **`crates/runtime-main/src/budget/`** (new module) — four files:
  - `mod.rs` — re-exports.
  - `enforcer.rs` — `BudgetEnforcer` with 3-scope tightest-cap-wins evaluation,
    4 threshold actions (Warn/Downshift/Suspend/HardStop) emitted in firing
    order. Idempotent: re-recording spend at the same percent does not re-fire.
    `record_spend_with_scopes(incremental_usd, framework_spend, global_spend)`
    accepts caller-supplied per-scope running totals. `cost_from(breakdown,
    input_per_million, output_per_million)` pure helper for cache-aware cost
    math.
  - `cost.rs` — `CostCache` LRU keyed by `CostKey` (stable hash of message
    content list). Capacity-0 cache disables caching cleanly.
  - `hook.rs` — `DownshiftHook` trait + `DefaultLadder` implementing the
    spec §2a `opus → sonnet → haiku` rule (tier-classified by model-id prefix).
    Sonnet → Haiku triggers only when `remaining < 10% AND
    avg_task_cost > remaining/3`. `RemainingBudget` carries `spent_usd`,
    `cap_usd`, optional `avg_task_cost_usd`.
- **`crates/runtime-main/src/recovery/`** (new module) — three files:
  - `mod.rs` — re-exports.
  - `resume.rs` — `request_resume_with(session_id, recover)` coordinates a
    session resume against the supplied async `recover` callback. Returns
    `ResumePlan { snapshot_id, plans, tasks, uncertain_tool_invocations,
    has_state }`. `reconnect_mcp_servers(session_id)` is the v0.1 no-op seam
    (M5/M6 wire the real path).
  - `uncertainty.rs` — `ToolCallUncertaintyAction` enum (4 spec §1b actions:
    Retry/Skip/MarkComplete/Abort) + `respond_uncertainty_with(...)` which
    writes a `tool_call_uncertainty_resolved` decision signal via the supplied
    emit callback. Returns `UncertaintyResolution { signal_id, action,
    invocation_id }`.
- **`src/components/BudgetHeaderBar.tsx`** — sticky top-of-screen bar with
  color gradient (ok/warn/downshift/suspended/exceeded). Tooltip surfaces
  scope breakdown; click reveals settings form for global per-day cap (calls
  `set_global_budget`). Exceeded status surfaces the "Session terminated due
  to budget" banner. Renders only when a budget event has landed.
- **`src/components/RecoveryDialog.tsx`** — cold-start surface. Reads
  `localStorage.lastSessionId` on mount; surfaces Resume/Discard prompt;
  Resume calls `invokeRequestResume(sessionId)` and seeds the renderer's
  `uncertainInvocations` list from the returned `ResumePlan`.
- **`src/components/UncertaintyPrompt.tsx`** — modal dialog iterating
  `state.uncertainInvocations`. Each invocation presents the 4 spec §1b action
  buttons; click dispatches `respond_uncertainty` and removes from the list.
  Counter shows remaining invocations.
- **`crates/runtime-main/tests/budget_threshold.rs`** (new integration test)
  — drives the enforcer with deterministic spend deltas; asserts the 4
  threshold actions fire in 50→75→90→100 order; downshift_hook invokes exactly
  once at 75%; tightest-cap-wins with framework scope.
- **`crates/runtime-main/tests/recovery_lifecycle.rs`** (new integration test)
  — full round-trip via real drone subprocess: write 3 signals (plan + task +
  stranded tool_invoked) + snapshot → recover via IPC → assert `ResumePlan`
  populated correctly → resolve uncertainty with `skip` → assert the
  resolution signal lands without re-invoking the tool (spec §1b + gotcha #15
  invariant).
- **`tests/unit/components/{BudgetHeaderBar,RecoveryDialog,UncertaintyPrompt}.test.tsx`**
  (new vitest specs) — 14 + 8 + 9 = 31 cases covering color gradient + status
  transitions + settings form + dialog resume/discard + uncertainty action
  routing + error surfaces.
- **`tests/e2e/{budget_threshold,recovery_uncertainty}.spec.ts`** (new
  Playwright specs) — 4 + 4 = 8 cases driving `window.__graphStore` to verify
  surface-on-state-change + ARIA attributes.

**Schema edits:**
- `crates/xtask/src/main.rs` — `budget` added to schemas list (Rust + TS
  codegen).

**Drone IPC additions:**
- `crates/runtime-core/src/drone.rs` — `DroneCommand::RecoverSession {
  session_id }` + `DroneEvent::SessionRecovered { snapshot_id, state, plans,
  tasks, uncertain_tool_invocations }`. DroneEvent variant count bumps to 10
  (round_trip.rs guard updated).
- `crates/runtime-drone/src/command_handler.rs` —
  `handle_recover_session(conn, session_id, event_tx)` calls existing
  `snapshot::recover_session_state` and emits the new `SessionRecovered`
  event.
- `crates/runtime-main/src/drone_ipc/client.rs` —
  `DroneClient::recover_session(session_id)` async method + `RecoveredSession`
  mirror struct + `await_recovery` event-filter helper.

**Tauri shell additions:**
- `src-tauri/src/main.rs` — `GlobalBudgetState` (Tauri-managed `Mutex<Option<f64>>`)
  registered alongside seams; new `request_resume` + `respond_uncertainty` +
  `set_global_budget` commands added to `invoke_handler`.
- `src-tauri/src/commands.rs` — three new Tauri commands + `*_with` testable
  seams. `set_global_budget` rejects NaN / negative caps with `CmdError::Internal`.

**Renderer state:**
- `src/lib/graphStore.ts` — 4 budget event cases now drive `state.budget:
  BudgetState | null` (spentUsd, capUsd, percent, status). New
  `uncertainInvocations: UncertainInvocation[]` field plus
  `recordUncertainInvocation` and `resolveUncertainInvocation` actions.
  Exhaustive `_exhaustive: never` switch holds.
- `src/lib/ipc.ts` — `invokeRequestResume`, `invokeRespondUncertainty`,
  `invokeSetGlobalBudget` wrappers + `ResumePlan`, `UncertaintyResolution`,
  `UncertaintyAction` types.
- `src/App.tsx` — mounts `<BudgetHeaderBar />` at the top of the page,
  `<RecoveryDialog />` (self-managing via localStorage), and
  `<UncertaintyPrompt sessionId={lastSessionId} />`.

**Quality gates (M04 Stage F — measured):**
- workspace coverage: 93.75% line (≥80% ✓)
- runtime-main coverage: 96.66% line (≥95% ✓)
- runtime-drone coverage: 95.79% line (≥95% ✓)
- per-module new safety primitives: budget/cost.rs 100%, budget/enforcer.rs
  98.90%, budget/hook.rs 100%, recovery/resume.rs 96.46%, recovery/uncertainty.rs
  98.48% — all ≥95%
- vitest: 249 passed (+34 from new component tests)
- Playwright: 27 passed (+8 new — 4 budget_threshold + 4 recovery_uncertainty)
- cargo test: ~300 passing including 3 new budget integration tests + 2 new
  recovery integration tests + 2 new drone command_handler tests
- cargo fmt / clippy / audit / deny / schema-drift: all green

**Carry-forward to Stage G gap-analysis:**
- Budget event shapes diverge from spec §2a (missing `scope`, `spent_usd`/`cap_usd`
  on Downshift, `budget_warn` vs spec's `budget_warning`). Document as 🟡
  Important.
- Downshift hook framework-tool-dispatch wiring deferred to M5/M9 — note the
  `DownshiftHook::tool_name` field reads but doesn't dispatch in v0.1.
- §1d long-lived `events()` reconnect note: ALREADY CLOSED at A2 — Stage G
  records the closure but does not need to re-validate (integration test at
  `crates/runtime-main/tests/drone_reconnect_events.rs`).

### Added — M04 Stage E (§6a HITL primitive — 9 trigger types + 3 UI variants + notifier plugin interface)

Sixth stage of M04. Builds the §6a HITL primitive end-to-end: 9-trigger policy evaluator + `HitlSeam` (oneshot-channel gate mirroring Stage B's `ApprovalSeam`) + notifier plugin interface + 3 built-in notifiers (`terminal_bell`, `desktop` via Tauri 2.x notification plugin, `sound`) + 3 renderer surfaces (Panel non-modal / Modal aria-modal=true / Toast role=status with 30s auto-dismiss) + `respond_hitl` Tauri command + failure-escalation integration test driving `task_escalated` → `on_failure_threshold` → seam-resolve-with-Skip end-to-end.

**Decisions documented in M04.E retrospective:**
- The phase doc's `respond_hitl` example threaded the call through `Arc<DroneClient>` IPC. Mirrored Stage B's `ApprovalSeam` pattern instead: the seam is `tokio::sync::oneshot`-backed and lives in-process — Tauri-managed `Arc<HitlSeam>` registered in `src-tauri/src/main.rs` setup hook, resolved directly by `respond_hitl` without a drone round-trip. Same architectural rationale as Stage C's approve_plan/revise_plan/abort_plan path.
- The pre-Stage-E `HitlRequested`/`HitlResolved` events had a provisional minimal shape (`prompt`, `hitl_kind`, `agent_id`, `response`, `duration_ms`). No live producers existed (audit-verified). Replaced with the spec §6a `HitlNotifyEvent`-aligned shape: `prompt_id` (correlation id), `trigger` (HitlTriggerRef), `agent_id` (nullable for plan-scoped triggers), `question`, `options[]`, `ui_variant`, `timeout_at_unix_ms` on `HitlRequested`; `prompt_id` + `choice` + `duration_ms` on `HitlResolved`.

**New artifacts:**
- **`schemas/hitl.v1.json`** (new) — `HitlPolicy` (9 trigger configs + notifier list + `timeout_seconds` + `default_action_on_timeout`), `HitlTrigger` enum (9 values, locked), `HitlUiVariant` enum (panel/modal/toast), `HitlNotifierType` enum (terminal_bell/desktop/sound/plugin), `HitlNotifier` shape, `HitlTriggerPolicy` (enabled + ui override + trigger-specific fields tools/threshold/percent).
- **`crates/runtime-core/src/generated/hitl.rs`** (generated via typify), **`src/types/hitl.ts`** (generated via json-schema-to-typescript).
- **`crates/runtime-main/src/hitl/`** (new module) — five files:
  - `mod.rs` — re-exports + module documentation.
  - `seam.rs` — `HitlSeam` (oneshot-channel gate). `await_response(prompt_id, wait)` registers the awaiter + races against a tokio timeout; on timeout the registration is removed before returning so a late `resolve` returns `NotFound` not `ReceiverDropped`. `resolve(prompt_id, choice)`, `cancel(prompt_id)`, `pending_len()` mirror `ApprovalSeam`'s contract.
  - `policy.rs` — 9-trigger policy evaluator. `HitlPolicyEvaluator::evaluate(policy, context)` returns `Some(ResolvedTrigger { trigger, ui_variant, timeout_seconds, default_action })` when enabled + trigger-specific preconditions met (risky-tool allowlist match; failure-threshold count ≥ threshold; budget percent ≥ percent), else `None`. `default_ui_for(trigger)` encodes the spec §6a default-UI-per-trigger table. Tool-pattern matcher supports exact and trailing-wildcard forms (`Bash:rm`, `WebFetch:*`).
  - `notifiers/mod.rs` — `HitlNotifier` trait (`fn notifier_type() -> &str`; `async fn notify(event) -> Result<(), NotifierError>`). `NotifierRegistry::build(configs)` skips disabled entries and rejects `plugin` type with `NotifierError::PluginNotSupported` per the v0.1 / M9 deferral. `dispatch_all(event)` fires every notifier in parallel (`futures::join_all`) and returns per-notifier `NotifierOutcome` (notifier_type + Result); errors are NON-FATAL — every notifier runs regardless of which ones fail.
  - `notifiers/terminal_bell.rs` — writes ASCII BEL (`\x07`) to stderr via the `emit_bell_with` testable seam.
  - `notifiers/sound.rs` — v0.1 BEL stub (same audible bell, `notifier_type = "sound"`). Cross-platform sound playback deferred to v1.0 / M11.
  - `notifiers/desktop.rs` — Tauri 2.x notification plugin wrapper. `Desktop::with_dispatcher(closure)` accepts an injectable async dispatcher; production wires the closure to call `tauri_plugin_notification::NotificationExt::notification()` (real Tauri call lives in `src-tauri`); tests inject in-memory stubs. `compose_title_body(event)` produces the title + body strings (body truncated at 240 chars with `…`). Per CLAUDE.md §5 OS-call wrapper-vs-seam pattern: the testable seam is covered; the real Tauri call path is structurally untestable cross-platform.
- **`crates/runtime-main/tests/hitl_failure_escalation.rs`** (new integration test) — full lifecycle: drive a `TaskState` to `Escalated` via 3 Failed events → evaluate `on_failure_threshold` policy fires → build registry from framework JSON + observer notifier → dispatch → seam await → resolve with `skip` → assert FSM rejects further events on terminal state. Also: timeout-without-response surfaces `HitlError::TimedOut`; plugin notifier type rejected at registry-build time.
- **`src/components/HITLPanel.tsx`** — non-modal (`aria-modal="false"`) full-takeover panel. Renders one button per option; falls back to a textarea form when `options[]` is empty. Escape dismisses locally without resolving the seam (seam keeps awaiting; same Stage C ApprovalPanel pattern).
- **`src/components/HITLModal.tsx`** — floating modal dialog with `aria-modal="true"`, `aria-labelledby`/`aria-describedby`, Escape closes.
- **`src/components/HITLToast.tsx`** — `role="status"` + `aria-live="polite"`. Renders a summary button when collapsed; clicking expands to options. Auto-dismisses after 30s of no interaction (renderer-local; the SDK seam keeps awaiting until its own timeout).
- **`tests/e2e/hitl_failure_escalation.spec.ts`** (new Playwright spec) — 6 tests driving `window.__graphStore` to verify panel/modal/toast surface on `hitl_requested`, dismiss on `hitl_resolved`, ARIA-modal attributes per variant, notifier-record attach on `notifier_dispatched`, and Escape-dismiss-without-resolving.

**Schema edits:**
- `schemas/event.v1.json` — provisional `HitlRequested`/`HitlResolved` shape replaced with the spec §6a-aligned shape (prompt_id, trigger, options, ui_variant, timeout_at_unix_ms on Requested; prompt_id, choice on Resolved); 3 new variants added (`hitl_timeout`, `notifier_dispatched`, `notifier_failed`); 2 new shared $defs (`HitlTriggerRef`, `HitlUiVariantRef`) — typify-friendly enum-in-$defs pattern per M04.D precedent.
- `crates/runtime-core/src/event.rs` (hand-curated) — mirrored to match schema; new public enums `HitlTriggerRef` and `HitlUiVariantRef`.
- `crates/xtask/src/main.rs` — `hitl` added to the schemas list (Rust + TS codegen).

**Tauri shell (Stage E wiring):**
- `src-tauri/src/main.rs` — registers `tauri_plugin_notification::init()` plugin; registers `Arc<HitlSeam>` Tauri-managed state alongside `Arc<ApprovalSeam>`; `respond_hitl` added to the `invoke_handler` list.
- `src-tauri/src/commands.rs` — new `respond_hitl(prompt_id, choice)` Tauri command + `respond_hitl_with(prompt_id, choice, &HitlSeam)` testable seam. Soft-Ok on no-pending-awaiter (same rationale as `approve_plan`).
- `src-tauri/capabilities/default.json` — `notification:default` permission added so the desktop notifier reaches the OS notification system. Locked-down capability list otherwise unchanged.
- `Cargo.toml` workspace + `src-tauri/Cargo.toml` — `tauri-plugin-notification = "2"`. Install + capability + permission API verified verbatim against <https://v2.tauri.app/plugin/notification/> at 2026-05-10 per gotcha #32.
- `package.json` — `@tauri-apps/plugin-notification ^2.0.0` (renderer-side dep for future renderer-driven notifications).

**Renderer:**
- `src/lib/graphStore.ts` — 5 formerly-no-op event cases now drive live state: `hitl_requested` inserts into `pendingHitl: Record<promptId, PendingHitl>`; `hitl_resolved` / `hitl_timeout` delete; `notifier_dispatched` / `notifier_failed` append to `notifierRecords: Record<promptId, NotifierRecord[]>` per matching trigger. `clear()` resets both. The exhaustive `_exhaustive: never` switch holds.
- `src/lib/ipc.ts` — `invokeRespondHitl(promptId, choice)` wrapper.
- `src/App.tsx` — mounts `<HITLPanel />` alongside `<ApprovalPanel />` and `<HITLModal />` / `<HITLToast />` at the App layout root.

**Tests:**
- HITL Rust unit tests: `hitl/seam.rs` 12 cases (await→resolve, timeout, double-resolve, cancel, receiver-dropped, concurrent, ReceiverDropped path); `hitl/policy.rs` 22 cases (9-trigger exhaustive happy path + 3 trigger-specific precondition tables + UI override + missing/disabled → None + matches_tool variants); `hitl/notifiers/mod.rs` 8 cases (build empty / disabled / each built-in / plugin reject / dispatch parallel / continue-on-failure / outcome shape); `hitl/notifiers/{terminal_bell,sound,desktop}.rs` 6 cases each.
- `crates/runtime-main/tests/hitl_failure_escalation.rs` 3 cases — full failure-escalation lifecycle + seam-timeout + build-rejects-plugin.
- `src-tauri/src/commands.rs` 3 new cases — `respond_hitl_with` resolve / no-pending-awaiter / receiver-dropped paths.
- `tests/unit/components/HITLPanel.test.tsx` 14 cases — render shape, ARIA, options, textarea fallback, Escape, error display, internal `_testing` helpers.
- `tests/unit/components/HITLModal.test.tsx` 9 cases — ARIA modal attributes, Escape, error display.
- `tests/unit/components/HITLToast.test.tsx` 9 cases — collapsed summary, expand-on-click, auto-dismiss timer, error display.
- `tests/unit/graphStore.test.ts` extended with 7 new cases — HITL request/resolve/timeout/notifier-dispatched/notifier-failed/trigger-routing/clear-resets.
- `tests/unit/ipc.test.ts` extended with 1 case — `invokeRespondHitl` arg-shape.
- `tests/e2e/hitl_failure_escalation.spec.ts` 6 Playwright cases.

**Coverage (measured locally, Linux CI runs same gates):**
- workspace ≥80% — actual 93.83% line.
- runtime-main ≥95% — actual 97.17% line (hitl/seam.rs 99.47%, hitl/policy.rs 99.81%, hitl/notifiers/desktop.rs 100%, hitl/notifiers/mod.rs 92.78%, hitl/notifiers/sound.rs 94.34%, hitl/notifiers/terminal_bell.rs 94.44%).
- runtime-drone ≥95% — actual 95.89% line.
- src/ ≥80% — actual 97.8% line.

**Coverage holdout:**
- `notifiers/desktop.rs` real-Tauri-call path: the `Desktop::with_dispatcher` testable seam is covered to 100% line; production wiring (the closure built around `tauri_plugin_notification::NotificationExt::notification()`) lives in `src-tauri` and is exercised end-to-end only by the production renderer, not by the workspace coverage gate. Same OS-call wrapper-vs-seam holdout pattern as `providers/anthropic.rs` + `key_store.rs` + `drone_ipc/connection.rs` + `hooks/shell.rs::TokioShellSpawner::spawn`.

**v0.1 scope:**
- STANDARD mode hardcoded — mode-keyed HITL policy overrides in framework JSON are loaded + validated but only STANDARD is evaluated.
- `on_capability_violation` trigger seam exposed but not wired — M5 wires the trigger source.
- Plugin notifiers from `notifiers/` dir return `PluginNotSupported` — M9 generators wire the plugin loader.
- 1h default HITL timeout (`HitlPolicy::timeout_seconds = 3600`) — per-trigger override deferred to v1.0.

### Added — M04 Stage D (§4a Verify & Rails primitive — Hook executor + JSONLogic-evaluated rails + don't-touch + revert_to_snapshot)

Fifth stage of M04. Builds the §4a Verify & Rails primitive end-to-end. Hook executor (shell|tool|agent variants × 7 firing points) + Rails (hard/soft + JSONLogic operator allowlist per gotcha #18) + globset-backed don't-touch glob matcher (new `pre_file_edit` firing point) + drone-side `RevertToSnapshot` handler extended to consume `RevertReason::HookRollback { hook_id }`. VerifyNode + HookNode upgrade from M03.C synthetic-state stubs to live-event-driven; rail_triggered events accumulate into a new `triggeredRails` store field for M05 capability-enforcer surface.

**Audit-grounded scope reductions vs. the original phase doc draft:**
- The phase doc planned a new `schemas/hook.v1.json`. Audit verified `Hook`, `HookRef`, `HookCategory`, `HookOnFailure`, `JsonLogicExpression` are ALREADY declared in `common.v1.json` and generated to `runtime_core::generated::common`; `Rail` is in `framework.v1.json`. Stage D consumes the existing types — no new schema file.
- The phase doc's planned new `hook_started`/`hook_passed`/`hook_failed` events would have duplicated the existing `verify_started`/`verify_passed`/`verify_failed` variants. Stage D extends the existing variant fields per spec §4a rather than re-authoring (audit gotcha decision).
- The Write-tool dispatcher integration site (`runtime-main/src/sdk/`) does not exist in v0.1 — the SDK drives LLM streaming + structured-emitter parsing only. Stage D ships `DontTouchEvaluator` as a callable primitive that the future capability enforcer (M05) and plan loop (M07) will route through; the evaluator itself is fully tested standalone.

**New artifacts:**
- **`crates/runtime-main/src/hooks/`** (new module) — five files:
  - `mod.rs` — re-exports + module documentation.
  - `shell.rs` — cross-platform shell wrapper. Windows uses `powershell.exe -NoProfile -Command "<command>"` (Windows PowerShell 5.1; pwsh.exe was unavailable on the M04.D build machine — documented retro decision); Linux/macOS uses `bash -c "<command>"`. Flag spelling + semantics verified verbatim against Microsoft's `about_PowerShell_exe` reference (M04.D WEBCHECK). Testable seam `execute_shell_with(spawner, ...)` accepts a `ShellSpawner` trait for unit tests; production `execute_shell` is the OS-spawn wrapper excluded from coverage gates per the M02/A2 wrapper-vs-seam pattern. Timeout via `tokio::time::timeout` + `kill_on_drop(true)`.
  - `executor.rs` — single entry point `execute_hook(hook, ctx, deps)` returning `HookOutcome::Passed { hook_id, duration_ms, output_preview? }` or `HookOutcome::Failed { hook_id, duration_ms, error, on_failure }`. Output-preview truncated at `OUTPUT_PREVIEW_MAX_BYTES = 512` with UTF-8-boundary-safe slicing. Tool / Agent variants surface as `HookError::DeferredVariant("tool:<name>")` / `("agent:<id>")` until M05 / M07 wire those dispatch paths.
  - `rails.rs` — JSONLogic evaluator. Operator allowlist locked to the gotcha #18 set (`var, ==, !=, <, <=, >, >=, and, or, not, in, +, -, *, /`). Operators outside the allowlist return `RailError::UnsupportedOperator`; missing `var` paths return `RailError::MissingVar`; division by zero → `RailError::Malformed`. `evaluate_rail(check, facts) -> RailOutcome::Triggered | Quiet` is the rail-evaluation surface; truthy table locks `Bool(false) | Null | 0 | "" | [] | {}` as falsy.
  - `dont_touch.rs` — `DontTouchEvaluator::new(patterns)` + `evaluate(path) -> DontTouchDecision::Allow | Block { matched_pattern }`. Globset-backed; case-insensitive matching for cross-platform consistency (Windows FS doesn't care about case; Linux does — runtime stays consistent regardless of host OS). Pattern recovery via `GlobSet::matches` index. Multi-glob match: first-by-index wins (single emit). Empty pattern list returns Allow for every path.
- **`crates/runtime-main/tests/hook_integration.rs`** (new) — full lifecycle integration test (pure-Rust, no real subprocess). Covers: post_task hook passes → `HookOutcome::Passed`; post_task hook fails with on_failure=rollback → `HookOutcome::Failed { on_failure: Rollback }` (the SDK uses `hook_id` to dispatch `RevertReason::HookRollback` to the drone); post_file_edit lint hook with on_failure=warn → no rollback flag; `RevertReason::HookRollback { hook_id }` round-trips serde correctly; `RevertReason::UserRollback` + `GapRecovery` stay unit-shape per audit decision.

**Schema edits:**
- `schemas/event.v1.json`:
  - `verify_started` extended: `category` (HookCategoryRef enum) + `firing_point` (string) required; `level` made optional + nullable. Spec §4a `hook_started` field set adopted.
  - `verify_passed` extended: optional nullable `output_preview`.
  - `verify_failed` extended: `duration_ms` + `on_failure` (OnFailureRef enum) required.
  - `rail_triggered`: `severity` field renamed to `policy` (RailPolicy enum hard|soft) per spec §4a; `firing_point` (string) required + `agent_id` optional nullable added.
  - Three new shared $defs: `HookCategoryRef`, `OnFailureRef`, `RailPolicy` (typify panics on inline enum properties — pattern follows `ApprovedBy` from M04 Stage B).
- `schemas/framework.v1.json` — `pre_file_edit` added to the `hooks` object as the 7th firing point (spec §4a).
- `crates/runtime-core/src/event.rs` (hand-curated) — mirrored to match schema.
- `crates/runtime-core/src/drone.rs` (hand-curated) — `RevertReason::HookRollback` migrated from unit variant to `HookRollback { hook_id: String }` per spec §4a; serde tag changed to `kind` (was implicit untagged) for forward compatibility with downstream variants. The audit baseline check (M04.D pre-flight) confirmed the variant existed; the field addition is the additive edit the audit gotcha called out.
- `crates/runtime-drone/src/command_handler.rs` — `handle_revert` extended to consume `&RevertReason`. Reason string in the emitted `SnapshotWritten` event now carries the variant (e.g., `"revert:hook_rollback:verify"` for HookRollback; `"revert:user_rollback"` / `"revert:gap_recovery"` for the unit variants). The actual `task_failed` emit per spec §4a happens at the SDK side (M07 plan loop) — drone's role is limited to confirming the snapshot exists.

**Renderer:**
- `src/lib/graphStore.ts` — four formerly-no-op event cases (`verify_started`/`passed`/`failed` + `rail_triggered`) now drive live state. `verify_started` with `category === 'verify'` creates/updates a VerifyNode (id `verify:<hook_id>`); other categories create/update a HookNode (id `hook:<hook_id>`). `verify_passed`/`failed` update whichever node type exists for that hook_id. `rail_triggered` appends to a new `triggeredRails: TriggeredRail[]` store field (cleared by `clear()`); M05 surfaces them in the capability-enforcer UI. Idempotent re-emit: re-emitting `verify_started` for the same hook_id resets to `active` + clears duration/error fields (retry-after-rollback path). VerifyNodeData extended with `firingPoint`, `outputPreview`, `error`, `onFailure`; HookNodeData extended with `firingPoint`, `durationMs`, `error`. Existing exhaustive `_exhaustive: never` switch held — TS compiler errors on any new schema variant per gotcha #36.
- `src/components/nodes/VerifyNode.tsx` + `HookNode.tsx` — render the new fields. VerifyNode shows `outputPreview` on `pass`, `error` + `onFailure` badge (block/warn/rollback color) on `fail`. HookNode shows `error` on `error` status. Both nodes expose `data-firing-point` for E2E selectors.
- `src/types/agent_event.ts` (regen) — three new types `HookCategoryRef`, `OnFailureRef`, `RailPolicy` exported alongside extended `VerifyStarted` / `VerifyPassed` / `VerifyFailed` / `RailTriggered` interfaces.

**Spec edit (in-stage, < 5 lines):**
- `agent-runtime-spec.md` §4a firing-point table — `pre_file_edit` row added between `post_task` and `post_file_edit` with description "Built-in `dont_touch` rail interception" matching the M04.D scope.

**Tests:**
- 4 new `crates/runtime-main/src/hooks/` modules with inline unit tests (~50 cases total): shell.rs covers SpawnArgs construction across platforms + dispatch + propagation; dont_touch.rs covers empty/matched/unmatched/recursive/multi-glob/case-insensitive/invalid-glob; rails.rs covers all 15 allowlisted operators + nested expressions + literal pass-through + truthy table; executor.rs covers shell pass/fail per on_failure variant + tool/agent deferred + output preview truncation (ASCII + UTF-8 boundary).
- `crates/runtime-main/tests/hook_integration.rs` (7 cases) — full lifecycle.
- `tests/unit/graphStore.test.ts` extended with 11 new cases for verify/hook/rail event flows (verify-vs-non-verify routing; pass/fail field carry; idempotent re-emit; rail accumulation; clear() reset).
- `tests/unit/nodes/VerifyNode.test.tsx` + `HookNode.test.tsx` extended for new fields (output preview, error+on_failure badge, firing-point data attribute, level-null omission).

**Coverage targets (per CLAUDE.md §5 safety-primitive gate):**
- `crates/runtime-main/src/hooks/` collectively ≥95% via per-module unit tests (the executor's shell-spawn path is covered via the `*_with` seam against a fake spawner; the production `TokioShellSpawner::spawn` real-OS wrapper is the OS-spawn holdout per the M02/A2 precedent).
- workspace ≥80%, runtime-main ≥95%, runtime-drone ≥95% maintained.

### Added — M04 Stage C (§3a Plan UI + ApprovalPanel + graph wiring — renderer surface for plan/task events)

Fourth stage of M04. Wires Stage B's plan/task event surface to the renderer. The non-modal `ApprovalPanel` resolves Stage B's `ApprovalSeam` via three new Tauri commands; PlanNode + TaskNode upgrade from synthetic-state stubs to live-driven visual treatments. One technical-best-practice decision documented: the seam is resolved in-process (the seam is `tokio::sync::oneshot`-backed and lives in `runtime-main`; cross-process oneshots don't exist), not via drone IPC as the phase doc text suggested. Per CLAUDE.md §12 own-technical-decisions; the architectural mismatch is documented in the M04.C retrospective.

- **`src/components/ApprovalPanel.tsx`** (new) — non-modal right-side overlay per M03.D InspectorPanel discipline. ARIA `role="region"`, `aria-label="Plan approval"`, `aria-modal="false"`. Surfaces when any plan in graphStore reaches `awaiting_approval`. Three actions: Approve dispatches `invokeApprovePlan`; Revise opens an inline textarea for free-text revisions then submits via `invokeRevisePlan`; Cancel plan opens an inline textarea for reason then submits via `invokeAbortPlan`. ESC dismisses panel-locally (does NOT abort — the SDK keeps awaiting). Panel auto-dismisses when the plan's status transitions out of `awaiting_approval` (via existing event subscription). Free-text passes through opaque per CLAUDE.md §8.security; renderer-side validation limited to length cap (2000) + non-empty trim before submit.
- **`src/components/nodes/PlanNode.tsx`** (edited) — visual upgrade from synthetic-state stub to live-driven rendering. Status badge (text label) displayed alongside title; per-status border color across all 7 PlanStatus values (`pending_approval`, `awaiting_approval`, `awaiting_replan` → amber/gap; `approved`, `in_progress` → blue/active; `complete` → green; `aborted` → red); revision/abort reason rendered on `awaiting_replan` + `aborted`; duration rendered on `complete`; title truncated at 40 chars (full title in InspectorPanel via existing JSON dump). Cumulative per-plan token spend deferred — adding it would require a Stage B `PlanNodeData` amendment, which the prompt's `<gotchas>` trap explicitly forbids; documented in retrospective.
- **`src/components/nodes/TaskNode.tsx`** (edited) — visual upgrade. All 7 TaskStatus values (`pending`, `running`, `done`, `blocked`, `failed`, `skipped`, `escalated`) drive border color + class; `escalated` adds the gap-pulse animation reusing the existing keyframe. Failure-count badge `⚠ N/M` (or `⚠ N` when `maxFailures = null`) renders when `failureCount > 0`. HITL flag preserved from M03. Duration rendered on `done`. Title truncated at 30 chars. The `skipped` status now surfaces line-through text-decoration per spec §3a strikethrough convention.
- **`src/lib/ipc.ts`** (edited) — three new typed wrappers: `invokeApprovePlan(planId)`, `invokeRevisePlan(planId, revisions)`, `invokeAbortPlan(planId, reason)`. Argument names align with the Tauri command parameter snake_case-to-camelCase mapping (`{ planId }`, `{ planId, revisions }`, `{ planId, reason }`).
- **`src-tauri/src/commands.rs`** (edited) — three new Tauri commands `approve_plan`, `revise_plan`, `abort_plan`. Each takes `tauri::State<'_, Arc<ApprovalSeam>>` and dispatches through a `*_with` testable seam (CLAUDE.md §5 archetype) that takes `&ApprovalSeam` directly. Per-command tracing entry/error/success per spec §13.5. Shared `resolve_or_log` helper treats `ApprovalError::NotFound` as soft-Ok with warn-log (rationale: Stage B retro [LIVE] ambiguity-events deferred the SDK plan_loop driver to M07; the renderer can dispatch the command before any awaiter is wired — do not 500 the user's click on a soft-state issue per CLAUDE.md §12 user-flow ergonomics). 6 unit tests cover happy-path (resolve seam → returns Ok with the right `ApprovalDecision` variant) + no-pending-await path (returns Ok).
- **`src-tauri/src/main.rs`** (edited) — Tauri `setup` hook registers an `Arc<ApprovalSeam>` ahead of drone spawn (the seam has no I/O so construction is infallible; registering early keeps the command layer wired even if drone spawn fails). The 3 new commands added to `invoke_handler`.
- **`src/App.tsx`** (edited) — mounts `<ApprovalPanel />` inside `.graph-layout` next to InspectorPanel. Exposes `window.__graphStore = useGraphStore` as a testing affordance for `tests/e2e/plan_approval.spec.ts` — module-level mocking across the `@tauri-apps/api` ESM boundary doesn't work in Playwright (Vitest covers the click→invoke linkage); the affordance lets the E2E spec drive graph state via `page.evaluate`. Always-on (no `import.meta.env.DEV` gate per CLAUDE.md §9 anti-pattern: the store carries no secrets, the same data is already inspectable via React DevTools, and feature-flag shims that don't earn their cost are out).
- **`src/styles.css`** — new `.approval-panel*` styles (right-overlay matching `.inspector-panel`, amber border per spec §3 Visual Design, action button per-action color encoding); PlanNode `__status` / `__reason` / `__duration` lines + `awaiting_*` border-color rules; TaskNode `__failure-badge` / `__duration` lines + `--escalated` gap-pulse animation + `--skipped` line-through.
- **Tests** — `tests/unit/components/ApprovalPanel.test.tsx` (10 cases: hidden when no awaiting plan, hidden on `pending_approval` pre-request, surfaces on `awaiting_approval`, ARIA region + aria-modal=false, Approve/Revise/Abort dispatch the right `invoke*` with the right args, ESC dismisses without aborting, auto-dismiss on `plan_approved` state transition, single-instance enforcement on multi-pending). `tests/unit/nodes/PlanNode.test.tsx` extended (status class across all 7 values; status badge text matching; revision reason rendering; abort reason rendering; duration formatting on `complete`). `tests/unit/nodes/TaskNode.test.tsx` extended (status class across all 7 values; failure-count badge format `⚠ N/M`; failure-count without denominator; badge omitted when count is 0; duration on `done`). `tests/e2e/plan_approval.spec.ts` (3 cases: panel surfaces, panel dismisses on state transition, PlanNode `data-status` transitions through the approval flow). 6 new Rust unit tests in `src-tauri/src/commands.rs` for the `*_with` seams.

### Added — M04 Stage B (§3a Plan & Task primitive — schemas + FSM + projection-based persistence + WriteSignal IPC + structured emitter)

Third stage of M04. Builds the §3a Plan & Task primitive end-to-end against spec (events at lines 1417–1427 + approval-gate primitive + loop policy + failure escalation + graph integration) and spec §10 (plans + tasks DDL). Two M02/M03 carry-forward items fold in: WriteSignal IPC + structured-emitter migration. Two engineering spec-drift carve-outs locked + flagged for closeout `docs(spec):` PR.

- **Event-shape reconciliation** (`schemas/event.v1.json` — 13 oneOf changes). 6 spec-canonical migrated: `plan_created` (+ `title` + `approval_required`); `plan_approved` (+ `approved_by` enum); `task_escalated` (replace `reason` with `failure_count` + `max_failures`); `task_started` / `task_completed` / `task_failed` keep shape (drift carve-out). 2 codebase extras: drop `plan_rejected` (unified under `plan_aborted`); KEEP `task_rolled_back` as typed event with `snapshot_id` (drift carve-out). 5 missing authored: `plan_approval_requested`, `plan_revised`, `plan_aborted`, `plan_complete`, `task_skipped`. Both drifts flagged for closeout `docs(spec):` PR. New `ApprovedBy` enum + `task_*.plan_id` denormalization preserved for self-contained downstream consumers.
- **`schemas/plan.v1.json` + `schemas/task.v1.json`** (new) — JSON Schema 2020-12 per spec §3a TS shape + spec §10 DDL. `$id` follows the established `https://schemas.aria-runtime.dev/<name>.v1.json` pattern. Validated string fields (`PlanTitle`, `TaskTitle` `minLength: 1`) extracted to `$defs/<Name>` per A1 typify-friendliness gotcha.
- **`crates/xtask/src/main.rs`** — codegen list extended from 7 to 9 schemas (added `plan` + `task`). Generated targets: `crates/runtime-core/src/generated/{plan,task}.rs` (typify) + `src/types/{plan,task}.ts` (json-schema-to-typescript). New TS files added to `.prettierignore` + `eslint.config.js` ignores per the agent_event.ts precedent.
- **`crates/runtime-drone/migrations/`** (new directory) + `db.rs::run_migrations` migration runner architecture. `_migrations` table tracks applied versions (version INTEGER PK, name TEXT, applied_at INTEGER). Migration files embedded via `include_str!` (build-time embed, single-binary deployment). Each migration runs in its own transaction with rollback-on-failure. `migrations/000_initial.sql` preserves M01 baseline (verbatim move of existing `init_schema` content — 8 tables: sessions, snapshots, signals, heartbeats, vdr, token_usage, skills, mcp_servers). `migrations/001_plans_tasks.sql` adds `plans` + `tasks` per spec §10 DDL with CHECK constraints on status enums + plan FK on tasks. The phase doc's prior reference to "existing M01.C migration runner pattern" was incorrect — architecture authored from scratch in Stage B.
- **`crates/runtime-main/src/plan/`** (new) — Plan + Task FSM. Pure-logic `PlanStateMachine` + `TaskStateMachine` enforce legal transitions per spec §3a. Failure-escalation boundary: `failure_count >= max_failures` (default 3) transitions Failed→Escalated. Safety primitive: 99.28% line coverage (≥95% gate met). 28 unit tests covering exhaustive transition matrix + illegal-transition rejection + failure-escalation boundary + terminal-state invariants. v0.1 hardcodes `fresh_context_per_task` (only loop policy lit) per CLAUDE.md §3 + spec §0d.
- **`crates/runtime-main/src/sdk/structured_emitter.rs`** (new) — replaces M02's `decision_extractor.rs` heuristic (DELETED — closes M02 🟡 carry-forward). Mechanism: parser consumes `<<DECISION>>...<<END>>` and `<<PLAN>>...<<END>>` delimited blocks deterministically. False-positive elimination: `Decision:` text in markdown code blocks / quoted content cannot trigger emit unless wrapped in delimiters (test pins this contract — `unstructured_decision_text_does_not_emit_decision_record`). Safety primitive: 95.92% line coverage (≥95% gate met). 21 unit tests covering single + multiple + nested + malformed + mixed + empty inputs + the false-positive-elimination forcing function.
- **`crates/runtime-main/src/sdk/approval.rs`** (new) — `ApprovalSeam` (oneshot channel pattern). SDK calls `await_approval(plan_id)` to suspend on a pending plan-approval HITL gate; renderer (Stage E wires the UI) calls `resolve(plan_id, decision)` to deliver the user's choice. `ApprovalDecision` variants: `Approved`, `Revised(reason)`, `Aborted(reason)`. Errors: `NotFound`, `Cancelled`, `ReceiverDropped`. Cancel + double-resolve + receiver-dropped paths exercised. Safety primitive: 99.02% line coverage (≥95% gate met). 11 unit tests including concurrent awaits on different plan IDs.
- **`crates/runtime-drone/src/plan_projector.rs`** (new) — drone-internal continuous projector parallel to `vdr.rs` (M03.E archetype). Reads plan/task signals from the `signals` table and UPSERTs `plans` + `tasks` rows. Idempotent semantics: every projection path uses `INSERT ... ON CONFLICT(id) DO UPDATE`. Out-of-order projection: terminal task statuses (`done`, `skipped`, `escalated`) preserved on subsequent `task_started` re-projection (CASE in UPDATE). Safety primitive: 97.88% line coverage (≥95% gate met). 18 unit tests covering each event type + idempotence + out-of-order + missing-field error paths.
- **`runtime_core::DroneCommand::WriteSignal`** (new IPC variant) — `{ signal_id, session_id, kind, event, context_type, payload }`. Drone-side handler in `crates/runtime-drone/src/command_handler.rs` inserts into `signals` table (INSERT OR IGNORE for idempotence) → calls `vdr::project_signal` → calls `plan_projector::project_signal` inside the same lock guard. Both projectors gracefully handle `SignalNotFound` as no-ops (race tolerance). Closes M03 🟡 carry-forward "vdr projector wired at signal-write call-site." 3 unit tests cover happy-path projection, idempotence on duplicate signal_id, and decision-payload → vdr roundtrip.
- **`crates/runtime-main/src/drone_ipc/client.rs::write_signal`** (new method) — fire-and-forget `DroneClient::write_signal(...)` exposing the IPC variant to SDK callers. No-op short-circuit for `DroneClient::noop()`.
- **`crates/runtime-drone/src/snapshot.rs::recover_session_state`** (new helper) + `RecoveredSession` struct — implements spec §1b recovery semantics. Reads latest snapshot + projected `plans` + `tasks` rows; **currently-running tasks (`status = 'running'`) are normalized to `pending`** (the agent process that was running them is gone). Tool-call uncertainty: `tool_invoked` signals lacking matching `tool_result` surfaced as `uncertain_tool_invocations` (renderer Stage F prompts retry/skip/mark-complete/abort). 8 unit tests covering no-snapshot path, latest-by-timestamp ordering, terminal-status preservation, and uncertainty heuristic edge cases.
- **`crates/runtime-main/src/sdk/event_pipeline.rs`** — `flush_text_buffer` now calls `parse_structured` (deletes `decision_extractor` callsite). Plan-creation outputs are surfaced for downstream `plan_loop` consumption (Stage B leaves the integration point; framework JSON wires the orchestrator). Malformed delimiter blocks log a warning + still forward the raw text as `StreamText` (no event silently dropped).
- **`src/lib/graphStore.ts::applyEvent`** — exhaustive switch handles 5 new + 6 changed + 2 dropped variants (compile-time `_exhaustive: never` forcing function held). Stage B mutations are pass-through state only (Stage C lights up the visual surface + ApprovalPanel + animated edge from PlanNode → currently-running TaskNode). PlanNodeData + TaskNodeData extended with new fields (`approvalRequired`, `lastTransitionReason`, `durationMs`, `agentId`, `failureCount`, `maxFailures`, `lastError`, `rollbackSnapshotId`). New `awaiting_approval` + `awaiting_replan` PlanStatus + `escalated` TaskStatus. 14 new vitest cases covering each event type's state mutation.
- **Integration tests** — `crates/runtime-drone/tests/migration_runner.rs` (8 cases: fresh apply, idempotent re-apply, version tracking, table existence, M01 baseline preservation, run-on-existing-conn, plans/tasks status CHECK constraints). `crates/runtime-main/tests/plan_lifecycle.rs` (2 cases: full 3-task happy path through plan_created → plan_approved → 3× task_started/completed → plan_complete via real drone subprocess + WriteSignal IPC; failure-escalation variant). `crates/runtime-main/tests/plan_recovery.rs` (2 cases: kill drone mid-plan, verify currently-running task recovered as `pending` per spec §1b; tool_invoked without matching tool_result marked uncertain).
- **Coverage** — workspace 93.44% line (≥80% ✓); runtime-main 97.83% line (≥95% ✓); runtime-drone 96.01% line (≥95% ✓). Per-safety-primitive: state_machine.rs 99.28%, approval.rs 99.02%, structured_emitter.rs 95.92%, plan_projector.rs 97.88%, snapshot.rs 98.15%, db.rs 97.20%, command_handler.rs 94.64%.

Spec-drift carry-forward to M04 closeout `docs(spec):` PR:
- §3a event shapes: `task_*` events keep `plan_id` (denormalization for self-contained downstream consumers).
- §4a rollback: `task_rolled_back` as typed event with `snapshot_id` field (replaces stringly-typed `task_failed` with `error: 'rolled_back_after_hook_<id>'` — CLAUDE.md §9 anti-pattern).

### Changed — M04 phase doc surgical fix: audit-corrections moved into XML (doc-only)

Surgical follow-up to the PR #53 revert (PR #54). Original ask from the user that PR #53 over-shot is now executed correctly: the `🔧 Audit corrections (post-M04.A2 audit)` callout blocks above each X.5 prompt section in stages B/C/D/E/F are dropped; their substantive corrections moved INTO the corresponding `<work_stage_prompt>` XML slots (`<gotchas>`, `<pre_flight_check>`, `<read_reference>`) where the build agent reads them at execution time. Plus three small audit-grounded refinements: Stage B `<pre_flight_check>` adds A1 namespace-decision check (`pub use generated::{agent, common, framework, skill, tool}`) + post-A1 7-schema xtask check; Stage F `<gotchas>` notes §1d ⚠️ note already closed at A2 (subscriptions don't survive reconnect; renderers resubscribe); Verification Checklist Hard Gate G1 corrected to "8 approval gates" (was "7"). 1 file edited, all 5 callout blocks removed, equivalent or stronger content embedded inside the XML the build agent actually parses.

### Added — Post-M04-PR-#53-revert protocol gap closure (doc-only)

Closes the cross-session-blindness failure mode that produced PR #53 (M04 phase doc rewrite based on origin's stale view of project state; reverted via PR #54). 4 narrowly-scoped doc edits across CLAUDE.md, STAGE-PROMPT-PROTOCOL.md, docs/gotchas.md.

- **CLAUDE.md §8 — new "Phase-doc-edit pre-flight (cross-machine state check)" subsection.** Mandatory before any edit to `docs/build-prompts/M[NN]-*.md` larger than ~50 lines or affecting any X.5 stage prompt: orchestration session MUST surface a request for the user to paste `git log --oneline main..HEAD` from the build machine; retrospective-file presence on the build machine is the source of truth for "stage X executed," not git visibility on origin. Banned failure mode: inferring stage status from origin's silence.
- **CLAUDE.md §19 — new rule 7: every stage end surface includes cross-machine state by default.** Specifically `git log --oneline main..HEAD` (commits ahead of main on the active milestone branch) + `ls docs/build-prompts/retrospectives/M[NN].*-retrospective.md` (retrospective files present). Closes the gap structurally: when the user pastes a stage surface to any downstream orchestration session, that session sees actual project state instead of inferring from origin.
- **STAGE-PROMPT-PROTOCOL.md §10 — new v1.4 hardening rule.** "Build-machine state must be confirmed before phase-doc edits." Companion to v1.3 grep-verify-claims rule: v1.3 covers WHAT codebase claims need verification; v1.4 covers WHICH codebase to verify against when origin and build-machine diverge. Validator does not enforce mechanically; authoring discipline backed by gotcha #42 + §19 rule 7.
- **docs/gotchas.md #42** (companion to #41). "Origin is a partial view of project state when CLAUDE.md §8 forbids per-stage pushes." Pattern bit M04 PR #53 → revert PR #54.

### Added — M04 Stage A2 (production wiring — drone subprocess + count_tokens + CmdError migration + reconnect lock)

Second stage of M04. Wires production paths M03 deferred via `DroneClient::noop()` and migrates the Tauri command surface to consume the typify-generated `CmdError` from Stage A1. Two phase-doc-named items deferred to a downstream stage after surface-and-confirm with the user (the integration points didn't exist in the codebase): VDR projector wiring at WriteSignal (no `WriteSignal` IPC command yet) and structured-emitter decision extractor (no prompt-template builder yet) — both fold into Stage B's plan/verify primitives where the missing primitives land naturally.

- **`src-tauri/src/drone_lifecycle.rs`** (new) — `DroneLifecycle` owns the spawned `runtime-drone` subprocess for an app session. `DroneLifecycle::spawn_with` is the testable seam (CLAUDE.md §5 `*_with` archetype) accepting injectable spawn + connect closures; `DroneLifecycle::spawn` is the production wrapper (locates the binary alongside `current_exe()` per gotcha #22, `tokio::process::Command` with `kill_on_drop(true)` failsafe, `connect_with_retry` exponential-backoff). `shutdown` sends `DroneCommand::GracefulShutdown` then awaits `Child::wait` with a 3s timeout fallback to `start_kill`. Cross-platform IPC addressing: Unix socket at `<temp>/runtime-drone-<sid>.sock`; Windows named pipe at `\\.\pipe\runtime-drone-<sid>`. Unit-tested via `spawn_with` seam (8 tests covering args composition, spawn-failure propagation, connect-failure propagation, shutdown idempotence, address-uniqueness invariants, ENOENT-on-cleanup tolerance).
- **`src-tauri/src/main.rs`** — Tauri `setup` hook spawns the drone subprocess, registers `Arc<DroneClient>` as Tauri-managed state, and stores the `Mutex<Option<DroneLifecycle>>` for graceful shutdown. `RunEvent::ExitRequested` handler `take()`s the lifecycle and runs `shutdown()` synchronously inside the Tauri runtime so the drone gets its handshake before the host exits. SQLite db path resolves under `app.path().app_local_data_dir()` (created on first run).
- **`src-tauri/src/commands.rs`** — `query_session_db` + `replay_session` now take `tauri::State<'_, Arc<DroneClient>>` and dispatch real drone IPC (M03's `DroneClient::noop()` shim removed from production code; remains a test affordance in `runtime-main`). `run_smoke_session` similarly takes the managed state and threads it through `run_smoke_session_with`. Hand-rolled struct-variant `pub enum CmdError` removed; replaced with `pub use runtime_core::CmdError` (typify-generated tuple variants over the `ErrorMessage` newtype). All ~17 callsites updated for the tuple shape.
- **`crates/runtime-core/src/cmd_error_ext.rs`** (new) — inherent constructors (`provider`, `drone`, `key_store`, `internal`) + `Display` + `std::error::Error` impls for the typify-generated `CmdError`. The constructors substitute `"(no message)"` for empty strings so the `ErrorMessage` `minLength: 1` schema constraint never panics in callsite ergonomics. Wire format unchanged from M02 (`{"type":"...","message":"..."}` for non-unit variants). `runtime-core/src/lib.rs` re-exports `CmdError` + `ErrorMessage` at the crate root (no name collision with the hand-curated `RuntimeError`). 13 unit tests covering constructors, `Display` parity with the M02 `thiserror` messages, `Error` trait wiring, and wire-shape preservation.
- **`crates/runtime-main/src/key_store.rs`** — `From<KeyStoreError> for CmdError` impl (orphan-rule placement: `KeyStoreError` is local to `runtime-main`; `CmdError` is foreign in `runtime-core`). `NotFound` maps to `SetupRequired`; `Keyring` wraps with `Display` body via `key_store(...)` helper. 2 new tests assert both translation paths.
- **`crates/runtime-main/src/providers/anthropic.rs`** — `count_tokens` calls `POST /v1/messages/count_tokens` per spec §2c.3 (added M03.5). Replaces the M02 chars/4 approximation now that M04 budget enforcement (Stage F) requires the actual provider-side count. Verified against <https://platform.claude.com/docs/en/api/messages-count-tokens>: response is `{"input_tokens": <number>}`; same `x-api-key` + `anthropic-version: 2023-06-01` headers as `/v1/messages`. The obsolete `count_tokens_approximates_char_div_4` unit test deleted (chars/4 path no longer exists; live-network unit test would fail in CI). Behavioral coverage moved to `tests/anthropic_wiremock.rs`.
- **`crates/runtime-main/tests/anthropic_wiremock.rs`** — 4 new wiremock tests for the count_tokens endpoint: happy path returning `input_tokens` field, 401 → `ProviderError::Auth`, 429 with `retry-after: 45` → `ProviderError::RateLimit`, missing `input_tokens` field → error (defends against upstream shape regression that would otherwise silently report 0 tokens and under-report budget pressure). Pattern parallels the existing `/v1/messages` tests.
- **`crates/runtime-main/tests/drone_reconnect_events.rs`** (new) — 2 integration tests covering the long-lived `events()` subscription survival question (spec §1d ⚠️ note, M04 carry-forward). Test-driven decision: subscriptions do NOT survive reconnect — the single-consumer `take_event_stream` design binds the subscriber to the original reader half; on drone restart that reader EOFs and the stream terminates. v0.1 application pattern: subscribers resubscribe on reconnect. The renderer's `agent_event` channel is fed by `forward_events` / `replay_session` per task — no app-layer reliance on cross-reconnect survival. Cross-platform `#![cfg(any(unix, windows))]` matching `drone_ipc_loopback.rs`.
- **`agent-runtime-spec.md` §1d** — ⚠️ "long-lived events() subscription pending" note replaced with the v0.1 behavior lock (resubscribe on reconnect; survival deferred to v1.0). Test reference: `drone_reconnect_events.rs`.
- **`src/lib/ipc.ts`** — `unwrapCmdError` consumes the typify-generated `CmdError` from `src/types/error.ts` (M04 Stage A1 codegen) instead of the M02 hand-maintained interface. New `isCmdError` type-guard checks the discriminator against the literal union (`'setup_required' | 'provider' | 'drone' | 'key_store' | 'internal'`); preserves M02 `setup_required` user-actionable phrasing + `${type}: ${message}` rendering for body-carrying variants. Falls through to a plain `message` field check then to `String(e)` for non-CmdError shapes.
- **`tests/unit/ipc.test.ts`** — 9 new Vitest tests for `unwrapCmdError` covering all 5 generated `CmdError` variants, the `Error`-instance path, the plain-object-with-message compatibility path, and the last-resort `String()` fallback.
- **`src-tauri/Cargo.toml`** — adds `process` + `time` + `io-util` + `fs` features to `tokio` (drone subprocess spawn / shutdown timeout / async stdout-stderr handling), and `uuid` workspace dep with `v4` feature for session-id generation in `drone_lifecycle::compute_ipc_addr`.

Closes carry-forward 🟡 entries:

1. M03 🟡 "Production drone subprocess wiring at Tauri startup" — DONE.
2. M02 🟡 "count_tokens → real /v1/messages/count_tokens endpoint" — DONE.
3. M02 🟡 "Long-lived events() subscription survives reconnect" — RESOLVED (test-driven v0.1 behavior lock; spec §1d updated).

Deferred (re-listed in M04.A2 retrospective Carry-forward):

- M03 🟡 "vdr.rs projector wired at signal-write call-site" — defers to a future stage (no `WriteSignal` IPC command exists yet; landing this requires schema additions to `runtime_core::DroneCommand` + drone-side handler + main-side persistence path).
- M02 🟡 "Decision extractor → structured emitter migration" — defers to a future stage (no central prompt-template builder; without injection a regex on `<<DECISION>>...<<END>>` blocks would always return empty).

### Added — M04 Stage A1 (build hygiene + xtask codegen extensions + coverage retrofits)

First implementation stage of M04 (Plan + Verify + HITL + Budget). Closes three M03 carry-forward 🟡 build-hygiene items so Stages A2–G focus on production wiring + new primitive surface. Doc + codegen + test additions; no shipped runtime behavior change.

- **`crates/xtask/src/main.rs`** — extends Rust schemas codegen list with `event` + `error` (was `[common, framework, skill, tool, agent]`); extends TS targets list with `("error", error.v1.json)`. New generated artifacts: `crates/runtime-core/src/generated/event.rs` + `crates/runtime-core/src/generated/error.rs` (typify) + `src/types/error.ts` (json-schema-to-typescript). The hand-curated `crates/runtime-core/src/event.rs` (with proptest module + per-variant docs) and `crates/runtime-core/src/error.rs` (`RuntimeError` thiserror enum) remain unchanged — Stage A1 commits the generated parallel artifacts; consumer reconciliation is downstream-stage scope.
- **`crates/runtime-core/src/lib.rs`** — replaces `pub use generated::*;` with explicit `pub use generated::{agent, common, framework, skill, tool};`. Necessary because the new `generated::event` and `generated::error` modules collide with the top-level `pub mod event;` / `pub mod error;` if glob-re-exported. Generated `CmdError` + typify-`AgentEvent` reachable via `runtime_core::generated::{event,error}` for Stage A2 consumers.
- **`crates/runtime-core/src/generated/mod.rs`** — adds `pub mod event;` + `pub mod error;` declarations.
- **`schemas/error.v1.json`** — metadata clarification (no validation behavior change): variant `title` fields PascalCased (`SetupRequired` / `Provider` / `Drone` / `KeyStore` / `Internal`) so typify derives Rust enum variant names cleanly; `message` string extracted to `$defs/ErrorMessage` so typify can name the `minLength: 1` validation newtype (typify 0.6.2 panics on root-oneOf string subschemas with validation but no name source). Same wire format, same `const` discriminator values, same `additionalProperties: false`.
- **`crates/runtime-main/src/drone_ipc/client.rs`** — adds `await_event_timeout_when_peer_silent` test using `#[tokio::test(start_paused = true)]` + duplex peer kept alive (not dropped) + paused-time advance past 5s. Asserts `Err(DroneIpcError::Io)` with `ErrorKind::TimedOut`. Distinguishes the 5s timeout branch from the EOF branch covered by the existing `read_signals_stream_close_surfaces_as_error_not_hang`. Closes M03 carry-forward 🟡 "tokio::time::pause() coverage for await_event timeout path"; `client.rs` line coverage 94.00% (M03.E baseline) → 96.75% (M04.A1).
- **`crates/runtime-drone/tests/integration*.rs`** — verified clean of `target/debug` literals via grep. Only matches are doc comments at `tests/integration.rs:16–17` describing the gotcha #22 rationale; production code uses `current_exe()`-derived paths per the M02.D + M03.A archetype. No retrofit needed.
- **`.prettierignore` + `eslint.config.js`** — append `src/types/error.ts` to the existing generated-TS ignore lists (matches the `agent_event.ts` precedent at lines 25 + eslint.config.js:24). Prettier sees `error.ts` as ignored so its json-schema-to-typescript double-quote output doesn't trip the `singleQuote: true` rule; eslint sees it as ignored so its `/* eslint-disable */` header doesn't surface as an unused-directive warning.

Closes M03 gap-analysis 🟡 entries:

1. "Extend xtask Rust typify list to include `event.v1.json`" — DONE (event added; error added as bonus).
2. "tokio::time::pause()-driven coverage for `await_event` timeout path" — DONE (client.rs 94.00% → 96.75% with new deterministic timeout test).
3. "Retrofit `crates/runtime-drone/tests/integration*.rs` to `current_exe()`-derived paths" — VERIFIED still clean (M03.A retrofit durable; only doc-comment mentions of `target/debug` remain).

Carry-forward to Stage A2: src-tauri/src/commands.rs::CmdError replacement with re-export of generated CmdError; src/lib/ipc.ts::unwrapCmdError refactor to consume generated `CmdError` from `src/types/error.ts`; eventual reconciliation of hand-curated `event.rs` with typify-generated `generated/event.rs`.

### Added — M03.5 (Pre-M04 prep — doc/protocol-only mini-milestone)

Two-stage doc/protocol prep landing the doc-level debt M03 closeout flagged plus the next iteration of the stage-prompt protocol, before M04 prompt authoring begins. Doc-only — no source code touched, no gap-analysis entry (per CLAUDE.md §20 the immutable ledger is reserved for code-shipping milestones).

- **Stage A — combined doc PR.** 22 surgical edits across 3 existing files plus 1 new schema file. Spec polish (M03 carry-forward): §2c.3 token tracking + count_tokens M04-deferral; §3 InspectorPanel layout + per-node-type handle conventions (M03.B–C–D shipped); §1c ⚠️ production drone wiring deferred to M04 Stage A2; §2b SQL inspector lexical validation rationale; §3 replay-from-signals expanded model; §10 ⚠️ v0.1 renderer-side localStorage exception. M02 carry-forward (still open at M03 close): §3a + §10 plans/tasks SQLite DDL; §957 ⚠️ decision extractor → structured emitter migration; §1120 ⚠️ ContextType reconciliation expanded; §839 ⚠️ long-lived events() reconnect M03→M04 update; new `schemas/error.v1.json` (CmdError wire format) + §1d reference. Gotchas graduation: 8 entries (#33–#40) graduated from per-stage M03 retros to durable `docs/gotchas.md`. CLAUDE.md §15 stale-count refresh "32 → 40".
- **Stage B — STAGE-PROMPT-PROTOCOL.md v1.3 iteration.** Five additive optional tags (`<pre_flight_check>`, `<schema_drift_check>`, `<fan_out_grep>`, `<dependency_audit_check>`, `<runtime_environment>`) in §7 work-stage-only, informed by M01–M03 friction. Three new anti-patterns in §13 covering the v1.3-introduced failure shapes. v1.3 hardening rule appended to §10. v1.3 validator behavior added to §11 errors + warnings. v1.3 changelog entry at top of §14. Lean-validator pattern continued from v1.2 — structural-only checks; cross-checks deferred to v1.4. M01–M03 prompts continue to validate unchanged under v1.3 (additive contract preserved). M04 is the first milestone authored on v1.3.

### Fixed — M03.F (post-merge CI fixes on PR #47)

Two post-merge CI fixes on the M03 PR. Both surfaced after Stage F landed; neither is in scope for the M03 gap-analysis entry (immutable per CLAUDE.md §20) and both will reappear as M04 carry-forward.

- **`wdio.conf.ts`** — fixes `browserName` capability. Stage F shipped `browserName: process.platform === 'win32' ? 'edge' : 'webkit2gtk'` per the M03 build prompt §F.3 example. A first fix attempt set `browserName: 'wry'` based on a misreading of the Tauri 2.x WebDriver docs page; that also failed CI (Linux returned `Failed to match capabilities` from POST /session, Windows returned `no msedge binary at <APP_BIN_PATH>`). Per the **official** Tauri 2.x WebDriver example (<https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/Tests/WebDriver/Example/webdriverio.mdx>) the capabilities object intentionally **omits** `browserName` entirely — `tauri-driver` constructs the native value when proxying to the platform driver (WebKitWebDriver on Linux, msedgedriver against WebView2 on Windows). Final fix: `browserName` removed from the capabilities object. Same fix applied to the M03 build-prompt example at `docs/build-prompts/M03-live-graph.md` §F.3 so future readers don't repeat either bug.
- **`crates/runtime-drone/tests/integration.rs`** — fixes 4 clippy errors that fired on Linux (stable + MSRV) + macOS (stable) but not Windows because the test file is `#![cfg(unix)]`. Two `clippy::redundant_closure_for_method_calls` (`.and_then(|r| r.ok())` → `.and_then(Result::ok)`) and two `clippy::collapsible_match` (nested `if let Ok(evt) = … { if let DroneEvent::Variant { … } = evt { … } }` collapsed to single `if let Ok(DroneEvent::Variant { … }) = …`). Source-level fixes per CLAUDE.md §7 anti-patterns; no `#[allow(...)]`.
- **`.github/workflows/ci.yml`** — disables the `e2e-tauri-driver` job (`if: false && …` combined with the existing condition) per CLAUDE.md §7 self-correction-budget escalation. After three iterations on `wdio.conf.ts` capabilities (`'edge' / 'webkit2gtk'` → `'wry'` → omit per the official Tauri 2.x docs) Linux + Windows still failed for two independent reasons (Linux: tauri-driver could not exec the built app binary; Windows: msedgedriver not on PATH). Upstream wdio v9 + tauri-driver 2.x compatibility is unresolved (tauri-apps/tauri#10670, tauri-apps/tauri#9203); the only confirmed-working community example pins wdio@7. Job definition + `wdio.conf.ts` + `tests/e2e-tauri/**` + `npm run test:e2e:tauri` script all remain in the tree so M04 carry-forward fix work has the existing infrastructure to iterate against. The renderer-level Playwright `e2e` job remains the E2E proof for M03's deliverables (live graph, VDR projection, SQL inspector, replay-on-mount); tauri-driver was additive desktop-shell coverage, not an M03 acceptance gate.

### Added — M03.F (Tauri-driver E2E + Phase Closeout)

Final stage of M03. Two workstreams in one commit: full Tauri 2.x desktop-shell E2E framework + M03 Phase Closeout artifacts.

- **`tests/e2e-tauri/smoke.e2e.ts` (NEW)** — six WebdriverIO v9 + mocha + chai E2E tests covering the M03 user-facing surfaces: app launch + SetupPanel; save-key flow + ✓ keychain indicator; smoke happy path with real Anthropic API call (graph renders); click AgentNode → InspectorPanel; SQL inspector executes SELECT; reload reconstructs graph from persisted signals via M03.E's replay-on-mount path. Tests 3 + 6 require `ANTHROPIC_TEST_KEY` repo secret in CI (~$0.001 per run × 2 OS).
- **`wdio.conf.ts` (NEW)** — WebdriverIO v9 config. Spawns `tauri-driver` as a service per <https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/>. Per-platform `browserName` (`webkit2gtk` on Linux, `edge` on Windows). macOS early-exit (`process.exit(0)`) so `npm run test:e2e:tauri` is a no-op there rather than a hard failure — `tauri-driver` is upstream-unsupported on macOS.
- **`.github/workflows/ci.yml`** — new `e2e-tauri-driver` job. Linux + Windows matrix (no macOS). Linux installs WebKitGTK driver + Xvfb; Windows uses pre-installed msedgedriver + Edge WebView2. Both build the app with `npx tauri build --no-bundle`, install `tauri-driver` via `cargo install --locked`, then run `npm run test:e2e:tauri` (Linux wraps in `xvfb-run`).
- **`tests/e2e/smoke.spec.ts`** — deletes the four `test.skip()`-with-rationale entries that M02.E carried forward; keeps the three active renderer-level Playwright tests (page load + password input type + smoke disabled without key). Two test types now cover two layers: Playwright (Vite dev server, fast feedback, all 3 OSes) + WebdriverIO (built Tauri binary, full integration, Linux + Windows).
- **`package.json` + `package-lock.json`** — adds devDeps `@wdio/cli ^9`, `@wdio/globals ^9`, `@wdio/local-runner ^9`, `@wdio/mocha-framework ^9`, `@wdio/spec-reporter ^9`, `webdriverio ^9`, `chai ^5`, `@types/mocha ^10`. New script `test:e2e:tauri`. Workspace `overrides.serialize-javascript: ^7.0.5` patches the only high-severity audit finding from the new mocha tree (transitive in @wdio/mocha-framework — pre-7.0.5 RCE/DoS advisories GHSA-5c6j-r48x-rmvq + GHSA-qj8w-gfj5-8c6v).
- **`eslint.config.js`** — extends the `**/*.config.{ts,js}` override to also match `wdio.conf.ts` (`.conf.ts` not `.config.ts`); sets `parserOptions.projectService: false` for that file so projectService doesn't error on a config not in the tsconfig include.
- **`docs/build-prompts/retrospectives/M03.F-retrospective.md` (NEW)** — Stage F process retro. Covers tauri-driver setup smoothness, gap-analysis authoring, CI workflow extension. Distinct from M03-summary.md (which aggregates across all six stages).
- **`docs/build-prompts/retrospectives/M03-summary.md` (NEW)** — per `SUMMARY-TEMPLATE.md`. Aggregates three-axis scores across A–F; cross-stage trends; pattern wins + surprises; time-box accuracy; ~12 explicit decisions to apply before M04 authoring; verdict.
- **`docs/gap-analysis.md`** — appended M03 entry per CLAUDE.md §20. Six required sections + new v1.2 protocol `<gotchas_graduation>` subsection (28 stage-gotcha entries across A–F with disposition: kept | graduated | resolved | expired). Append-only — M01 + M02 entries unchanged.

This commit is the FINAL commit on `claude/m03-live-graph` per CLAUDE.md §20. The gap-analysis entry is **immutable** once committed; future milestones report status via Carry-forward sections only.

Refs: `docs/build-prompts/M03-live-graph.md` §F; `agent-runtime-spec.md` §3 + §13; `CLAUDE.md` §8 + §20; `STAGE-PROMPT-PROTOCOL.md` v1.2 (closeout schema + gotchas_graduation subsection); `docs/gotchas.md` #23 (tauri-driver matrix); `docs/MVP-v0.1.md` §M3 acceptance criteria.

### Added — M03.E (VDR projection + SQL inspector + replay)

Largest stage of M03. Three pieces, one stage: drone-internal VDR projection (decision + verify signals → vdr table); renderer-side SELECT-only SQL inspector over the session database; graph persistence via replay-from-signals on app mount. Ships the architecture + full unit/integration test coverage; production drone subprocess wiring is M04+ scope (Tauri commands wrap a `DroneClient::noop()` for v0.1 — the test seams exercise the full chain).

- **`crates/runtime-drone/src/vdr.rs` (NEW)** — projection module + read-only SQL helpers. `project_signal(conn, signal_id)` projects decision + verify signals into the `vdr` table; `project_session(conn, session_id)` is the per-session bulk variant. Idempotent: re-projecting a signal-id is a no-op (UNIQUE INDEX on `vdr.contributing_signal_id`). `signals_for_session` returns signals as JSON for the `ReadSignals` command path; `execute_select` runs validated SELECTs and returns rows keyed by column name. **`is_select_only` is parser-based, not regex-based** (Stage E E.1 Decision #3): rejects compound semicolons, `pragma_*`, and any statement that doesn't `prepare()` to a `column_count() > 0` shape.
- **`crates/runtime-drone/src/db.rs`** — adds `contributing_signal_id TEXT` column on the `vdr` table + `CREATE UNIQUE INDEX IF NOT EXISTS idx_vdr_contributing_signal` for projection idempotence. Existing schema preserved verbatim. New public `init_in_existing(conn)` helper lets integration tests pre-seed the database from a separate process before the drone subprocess opens it.
- **`crates/runtime-drone/src/command_handler.rs`** — handles two new `DroneCommand` variants. `QuerySessionDb { sql }` validates SELECT-only, runs `execute_select`, replies with `DroneEvent::QueryResult { rows }` (or `Alert(Critical)` on rejection / failure). `ReadSignals { session_id }` calls `signals_for_session` and replies with `DroneEvent::SignalLog { signals }`. UTF-8-safe `truncate_for_log` helper for alert messages with non-ASCII content.
- **`crates/runtime-drone/tests/vdr_projection.rs` (NEW)** — 6 tests cover the full projection contract: decision-signal-yields-row, verify-signal-yields-row, non-projection-eligible-signal-yields-nothing, idempotent-on-re-run, full-session-projection, SELECT-only-validator-rejects-6-attack-vectors.
- **`crates/runtime-drone/tests/integration.rs`** — 2 new Unix-only subprocess roundtrip tests (`query_session_db_roundtrip_returns_rows`, `read_signals_roundtrip_preserves_ordering`). Pre-seed the database via `init_in_existing`, spawn the drone, send the command over the socket, parse the response.
- **`crates/runtime-core/src/drone.rs`** — `DroneCommand` gains `QuerySessionDb { sql }` + `ReadSignals { session_id }`; `DroneEvent` gains `QueryResult { rows: Vec<Value> }` + `SignalLog { signals: Vec<Value> }`. Both event payloads use `serde_json::Value` (Eq impl-bearing as of recent serde_json) so `Eq` derive on `DroneEvent` holds.
- **`crates/runtime-main/src/sdk/replay.rs` (NEW)** — `replay_signals_to_events(&[Value]) -> Vec<AgentEvent>`. Pure-function inverse of M02.D's EventPipeline. Handles agent (spawned/complete/error), tool, skill, decision, session_start signal types. Missing-required-fields signals are filtered, not panicked; unknown signal types skipped silently per spec §2b "more types may exist".
- **`crates/runtime-main/tests/sdk_replay.rs` (NEW)** — 4 tests: per-signal-type translation; ordering preserved across translation; missing-fields filtered; 100-signal log translates without OOM (bounded `Vec` per `docs/gotchas.md` #28).
- **`crates/runtime-main/src/drone_ipc/client.rs`** — adds `query_session_db(sql)` + `read_signals(session_id)` methods. Send the command, await the matching response on the event stream (Heartbeats and unrelated events are skipped via `await_event` filter), 5-second timeout. Noop mode short-circuits to empty `Vec`. New `Connection::is_noop()` accessor.
- **`src-tauri/src/commands.rs`** — adds `query_session_db(sql)` + `replay_session(session_id)` Tauri commands. Both have `*_with` testable seams per CLAUDE.md §5 archetype: `query_session_db_with(sql, querier)` takes an injectable async function; `replay_session_with(session_id, read_signals, emit)` takes injectable signal-reader and emitter callbacks. Production wrappers route through `DroneClient::noop()` (M04+ wires a real drone subprocess).
- **`src-tauri/src/main.rs`** — registers both new commands in the `tauri::Builder::invoke_handler` macro.
- **`src/components/SqlInspector.tsx` (NEW)** — renderer-side SQL inspector. Textarea for SQL, Execute button, results table or error paragraph. ARIA-compliant (`role="alert"` for the error path; explicit `aria-label` for the textarea). Disables Execute while a query is in flight (debounce discipline — rapid clicks fire only one IPC call). 5 unit tests cover the contract.
- **`src/lib/ipc.ts`** — adds `invokeQuerySessionDb(sql)` + `invokeReplaySession(sessionId)` wrappers. 2 new ipc.test.ts tests cover the call-shape contract.
- **`src/App.tsx`** — adds replay-on-mount `useEffect` that reads `localStorage.lastSessionId` and calls `invokeReplaySession`; the `subscribeAgentEvents` handler now writes `event.session_id` to localStorage on `session_start` so the next mount can replay. Adds `<SqlInspector>` below the graph + inspector layout. 2 new App.test.tsx tests.
- **localStorage scope**. M03 uses webview-scoped localStorage for `lastSessionId` — sufficient for v0.1 (single-instance, single-user); M04+ may persist last-session-id in SQLite if cross-instance state is needed.

Stage E does NOT bump `schemas/event.v1.json` (per the prompt's `<execution_warnings>`); Stage D's bump is the last for M03. The renderer's data-shape interfaces in `graphStore.ts` are unchanged.

Refs: `docs/build-prompts/M03-live-graph.md` §E; `agent-runtime-spec.md` §1 (drone) §2b (signals + VDR) §3 (graph behavior); `docs/MVP-v0.1.md` §M3 acceptance criteria; CLAUDE.md §5 `*_with` archetype + §6 cargo deny no-new-deps; `docs/gotchas.md` #21 #27 #28.

### Added — M03.D (Inspector panel + token weight + dagre layout)

Three pieces that make the live graph interactive: click-to-inspect side panel; token-spend visualization (CSS `transform: scale()` per cumulative tokens); zoom/pan + MiniMap + dagre layout. Adds a schema bump on `tool_result` + `agent_complete`, hand-extends the Rust `AgentEvent`, and threads token data through the runtime-main `ProviderEvent` + `EventPipeline` from Anthropic's existing `message_delta.usage` tracking.

- **`schemas/event.v1.json`** — additive minor in-place bump per `schemas/README.md`. `tool_result` gains optional `tokens_in?` + `tokens_out?`; `agent_complete` gains optional `tokens_total?`. `$id` unchanged. `cargo xtask regenerate-types` updates `src/types/agent_event.ts` accordingly.
- **`crates/runtime-core/src/event.rs`** — hand-extended (`event.v1.json` is in the TS codegen list, not the Rust typify list per Stage A) so the Rust enum matches the schema. New fields are `Option<u64>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` so M02-era payloads continue to deserialize and absent fields don't pollute the wire format.
- **`crates/runtime-main/src/providers/mod.rs::ProviderEvent`** — `ToolResult` gains `tokens_in: Option<u64>` + `tokens_out: Option<u64>`; `MessageStop` gains `total_tokens: Option<u64>`. Internal-to-runtime-main; not a schema concern.
- **`crates/runtime-main/src/providers/anthropic_sse.rs`** — `SseState` accumulates input + output tokens across `message_start.usage` + `message_delta.usage` (the Anthropic SSE running totals); `translate(MessageDelta)` attaches `cumulative_tokens` to the emitted `ProviderEvent::MessageStop.total_tokens`. Two new unit tests (cumulative accumulation; missing-usage stays `None`).
- **`crates/runtime-main/src/sdk/event_pipeline.rs`** — translation forwards token fields: `ProviderEvent::ToolResult { tokens_in, tokens_out }` → `AgentEvent::ToolResult.tokens_in/tokens_out`; `ProviderEvent::MessageStop { total_tokens }` → `AgentEvent::AgentComplete.tokens_total`. Three new tests in `crates/runtime-main/tests/sdk_event_translation.rs`.
- **`src/lib/layout.ts` (NEW)** — pure dagre wrapper. `layoutGraph(nodes, edges) => GraphNode[]` runs `@dagrejs/dagre` v2 with `rankdir: 'TB'`, returns nodes with computed top-left positions (translated from dagre's center-based coords). Empty-graph fast-path; deterministic for a given input. 4 unit tests cover the contract.
- **`src/components/InspectorPanel.tsx` (NEW)** — right-rail ARIA-compliant non-modal dialog. Subscribes to `selectedNodeId` + node-data slice via Zustand selectors (single source of truth pattern from Stage B preserved). `role="dialog"` + `aria-modal="false"` + `aria-label="node inspector"`. ESC + close-button both clear the store's selection; focus moves to the panel root on open per WAI APG dialog pattern. Renders `selectedNode.data` as JSON; Stage E will extend with VDR-correlated decision history. 6 unit tests cover render, ESC, close-button, ARIA attrs, and JSON content.
- **`src/lib/tokenScale.ts` (NEW)** — pure helper. `tokenScale(totalTokens) => clamp(0.8, 1 + tokens/1000, 1.5)`. Shared between AgentNode + ToolNode so the scale logic is covered once and identical across consumers.
- **`src/lib/graphStore.ts`** — `AgentNodeData` gains `tokensIn` + `tokensOut` + `tokensTotal` (cumulative across the agent's tool calls + the session-total reported on `agent_complete`); `ToolNodeData` gains `tokensIn` + `tokensOut` (per-call). `applyEvent('tool_result')` populates the tool-node fields and accumulates the parent agent's totals. `applyEvent('agent_complete')` populates `tokensTotal` when `tokens_total` is present. Missing fields default to 0 (the schema's `Option<u64>` surfaces as `?? 0`). Existing graphStore tests preserved; 4 new tests cover the token paths.
- **`src/components/GraphCanvas.tsx`** — adds `<MiniMap nodeStrokeWidth={3} pannable zoomable />` alongside existing `<Background>` + `<Controls>`. New `useMemo` runs `layoutGraph(nodes, edges)` keyed on `[nodes.length, edges.length]` so layout reruns only on graph-shape changes (status flips + token-spend updates don't churn the layout). Per React Flow v12 layouting guide.
- **`src/components/nodes/AgentNode.tsx` + `ToolNode.tsx`** — apply `style={{ transform: scale(...), transformOrigin: 'center' }}` per cumulative tokens via `tokenScale`. `aria-label` + `data-status` + `data-testid` preserved.
- **`src/App.tsx`** — wraps `<GraphCanvas>` + `<InspectorPanel>` in a flexbox row so the panel sits to the right of the canvas. SetupPanel + SmokeButton + handleSetKey + handleSmoke unchanged. 2 new App-level tests (selecting a node opens the inspector; ESC closes it).
- **`src/styles.css`** — adds `.graph-layout` flexbox row, `.inspector-panel` right-rail rules (panel header + close button + JSON `<pre>` styling), and a small `.react-flow__minimap` border rule to align with the palette. The 11 existing node-type styles + edge keyframes are preserved verbatim.
- **`package.json`** — `@dagrejs/dagre ^2.0.0` (the maintained DagreJs-org fork; verified via WEBCHECK against <https://github.com/dagrejs/dagre>).

Refs: `docs/build-prompts/M03-live-graph.md` §D; `agent-runtime-spec.md` §3 Behavior + Visual Design; `schemas/README.md` additive in-place bump policy; `docs/gotchas.md` #21.

### Added — M03.C (Remaining 8 node types + animated edges + color encoding)

Lights up the rest of spec §3's node-type set. After Stage C, all 11 node types ship as renderable components: AgentNode, ToolNode, SkillNode (Stage B) + MCPNode, GapNode, HITLNode, PlanNode, TaskNode, VerifyNode, HookNode, FrameworkNode (this stage). graphStore.applyEvent extended for the two events that already exist in the v0.1 schema: `session_start` → FrameworkNode (graph root); `tool_invoked` with `source='mcp'` → lazy parent MCPNode. The remaining six components (Gap, HITL, Plan, Task, Verify, Hook) ship as renderable but their event-driven wiring lands at M4 (plan/task/verify/hook events) and M5 (gap events) when the schema gains those variants.

- **`src/components/nodes/{MCPNode,GapNode,HITLNode,PlanNode,TaskNode,VerifyNode,HookNode,FrameworkNode}.tsx` (NEW)** — eight new React Flow custom-node components mirroring the Stage B AgentNode archetype (`Handle` + `Position` + `data-testid` + `data-status` + ARIA label). Two specialize: HITLNode is `role="alert"` + `aria-live="assertive"` per WAI APG (blocking input affordance); GapNode adds `data-kind` (`tool_missing` / `skill_missing`) + the `gap-node--gap` class drives the `@keyframes gap-pulse` animation per spec §3 Behavior ("GapNode appears immediately on tool_missing"). FrameworkNode is the graph root — source handle only (no upstream parent in v0.1).
- **`src/lib/graphStore.ts`** — extended:
  - Eight new data interfaces (`MCPNodeData`, `GapNodeData`, `HITLNodeData`, `PlanNodeData`, `TaskNodeData`, `VerifyNodeData`, `HookNodeData`, `FrameworkNodeData`) plus typed `Node<...>` aliases.
  - `GraphNode` discriminated union grown from 3 → 11 variants. `EdgeData.kind` enum gains `'agent-mcp'`.
  - `applyEvent('session_start')` promoted from no-op to spawn FrameworkNode at root with id `framework:<name>`. Idempotent on duplicate session_start.
  - `applyEvent('tool_invoked')` extended: `source: 'mcp'` + `server` set lazily spawns an MCPNode with id `mcp:<server>` and wires agent → MCP + MCP → tool edges (NOT agent → tool); same MCP server reused across multiple tools (one MCPNode + one agent→MCP edge + one MCP→tool edge per tool). Non-MCP tools keep Stage B's agent → tool routing.
  - Animated-edge state machine: every `tool_invoked`-created edge has `animated: true`; `tool_result` clears the flag on the inbound edge (matches by target so both agent→tool and MCP→tool shapes resolve uniformly).
  - Coverage: 96.37%+ preserved on the safety primitive.
- **`src/components/GraphCanvas.tsx`** — `nodeTypes` map grown from 3 → 11 entries (one per spec §3 node type). Map definition kept at module level per @xyflow/react v12 docs (Stage B trap re-applies with 11 types — inline definition triggers per-render remount).
- **`src/styles.css`** — extended:
  - Spec §3 Visual Design palette in `:root` CSS custom properties (`--node-active`, `--node-complete`, `--node-error`, `--node-gap`, `--node-hitl` + base bg/border/fg). Existing AgentNode + ToolNode rules refactored to use `var(--node-...)` so future stages adjust the palette in one place.
  - Eight new node-type style blocks each with `--<status>` modifiers (Plan/Task/Verify use type-specific status enums per spec §3a + §4a; MCP/Hook/Framework use the shared `active/complete/error` palette).
  - GapNode `gap-pulse` keyframe (1.4s amber pulse) + HITLNode bright/white modifier per spec §3.
  - `.react-flow__edge.animated` keyframe (`dash-flow` 1s linear) for active-call animation; `.react-flow__edge--dashed` static dashed style for skill-load edges.
- **Tests** — `tests/unit/graphStore.test.ts` (7 new tests: `session_start_spawns_FrameworkNode_at_root` + idempotent; MCP lazy spawn + reuse across tools; animated-edge lifecycle on `tool_invoked`/`tool_result` for both agent→tool and MCP→tool shapes); `tests/unit/nodes/{MCP,Gap,HITL,Plan,Task,Verify,Hook,Framework}Node.test.tsx` (5 tests each = 40 new component tests; HITLNode + GapNode + FrameworkNode have specialized assertions per their spec §3 specializations); `tests/unit/App.test.tsx` updated to assert FrameworkNode lands when `session_start` arrives in the smoke happy-path.
- **Synthetic-state testing pattern locked.** Tests for the six event-less components (Gap, HITL, Plan, Task, Verify, Hook) pass populated state directly to `<NodeComponent>` rather than dispatching events through the store. M4+ wires events to these components without renderer-test churn.

Refs: `docs/build-prompts/M03-live-graph.md` §C; `agent-runtime-spec.md` §3 (Node Types + Behavior + Visual Design); `docs/MVP-v0.1.md` §M3; `docs/gotchas.md` #21 + #27.

### Added — M03.B (React Flow + Zustand foundation + 3 basic node types)

Lays the foundation for the live graph. Replaces M02's flat `<ul>` event list with a React Flow canvas backed by a Zustand store. Three of the eleven spec §3 node types ship: AgentNode, ToolNode, SkillNode. The remaining eight (MCP, Gap, HITL, Plan, Task, Verify, Hook, Framework) land in Stage C.

- **`src/lib/graphStore.ts` (NEW)** — Zustand v5 store; the canonical source of graph state. Exports `applyEvent(event)`, `clear()`, `selectNode(id)` actions plus `nodes` / `edges` / `selectedNodeId` slices. `applyEvent` is the single entry point for translating `AgentEvent` into node + edge mutations. Idempotent on duplicate events; exhaustive over the 36-variant discriminated union via TS `_exhaustive: never` check. Stage B handles 6 variants as render mutations (`agent_spawned` + parent edge; `agent_complete`/`agent_error` status flips; `tool_invoked` + edge; `tool_result` complete + duration; `skill_loaded` + dashed edge); the remaining 30 are explicit no-ops Stage C/D/M4+ light up. Coverage: 96.37% line.
- **`src/components/nodes/AgentNode.tsx` (NEW)** — React Flow v12 custom node with `Handle` + `Position` primitives. Renders agent name + 8-char-truncated id + status class. ARIA-labeled. `data-testid` + `data-status` for E2E selectability (Stage F).
- **`src/components/nodes/ToolNode.tsx` (NEW)** — same shape, renders tool name + duration (when complete).
- **`src/components/nodes/SkillNode.tsx` (NEW)** — dashed outline (`skill-node--dashed` class) per spec §3 Behavior; no flow animation. Renders skill name + mode-variant (when present).
- **`src/components/GraphCanvas.tsx` (NEW)** — wraps `<ReactFlow>` from `@xyflow/react`; subscribes to the store via Zustand selectors (`useGraphStore((s) => s.nodes)` form) so re-renders trigger only on the relevant slice change. `nodeTypes` map defined at module level per @xyflow/react v12 docs (inline definition forces per-render remounts and kills the streaming UX). Includes `<Background />` + `<Controls />`. `onNodeClick` / `onPaneClick` wired to `selectNode` for Stage D's inspector seam.
- **`src/App.tsx`** — refactored: Zustand store replaces the M02 `useReducer`. SetupPanel + SmokeButton + handleSetKey + handleSmoke + `console.error` + `unwrapCmdError` preserved verbatim. Heading flipped from "M02 smoke" to "M03 live graph".
- **`src/styles.css`** — appended graph canvas + 3 node-type styles per spec §3 Visual Design (dark background, color-encoded status: blue=active, green=complete, red=error; dashed SkillNode outline). M02 component styles preserved.
- **Tests** — `tests/unit/graphStore.test.ts` (13 tests covering each Stage B AgentEvent branch + idempotence + clear/select + an exhaustive no-op coverage test for the other 27 schema variants); `tests/unit/nodes/{Agent,Tool,Skill}Node.test.tsx` (5 tests each: render + status classes + accessibility + handles); `tests/unit/App.test.tsx` refactored to assert on `useGraphStore.getState().nodes` instead of listitem count; `tests/unit/components.test.tsx` refactored — dropped EventList tests, added GraphCanvas empty-state smoke. 41 frontend tests pass; coverage 93.47% global, 96.37% on graphStore primitive.
- **Deletions** — `src/lib/eventReducer.ts`, `src/components/EventList.tsx`, `tests/unit/eventReducer.test.ts` (replaced by graphStore architecture).

Refs: `docs/build-prompts/M03-live-graph.md` §B; `agent-runtime-spec.md` §3; `CLAUDE.md` §5 (TDD discipline) §14 (schemas-as-source-of-truth — schema's snake_case field names used throughout); `docs/gotchas.md` #21 (clippy traps — N/A for TS), #25 (Vite root — preserved), #26 (serde tag-shape — N/A for TS), #27 (Vitest+RTL DOM-ref staleness — observed via `act()` wrap of synchronous Zustand dispatch in App.test.tsx).

### Added — M03.A (Build hygiene + carry-forward closures + new deps)

Closes the M02 🟡 Important carry-forward items + adds the deps Stages B–F need. No React Flow code yet; that lands in Stage B. Per `docs/gap-analysis.md` M02 entry §"Carry-forward to M03 prep" + `M02-summary.md` §"Decisions to apply before the next parent milestone".

- **`schemas/event.v1.json` (NEW)** — canonical AgentEvent schema covering all variants of `runtime_core::event::AgentEvent` (session/agent/tool/skill/plan/task/mode/verify/rails/gap/HITL/capability/budget/stream/decision/token + `ToolSource` enum). Source-of-truth for renderer TypeScript types per `CLAUDE.md` §14 schemas-as-source-of-truth. Replaces hand-mirrored `src/types/agent_event.ts`.
- **`crates/xtask/src/main.rs`** — extends `regenerate-types` + `regenerate-types --check` to also generate TypeScript types via `npx --yes json-schema-to-typescript`. New testable seam `regenerate_typescript_types_with(schemas, output_dir, runner, check)` mirrors the M01.C / M02.C / M02.D / M02.E `*_with` archetype; production wires `runner = run_npx_json_schema_to_typescript`. Drift list merges with the existing typify Rust-codegen drift list so a single bail message covers both Rust and TS regressions.
- **`src/types/agent_event.ts`** — regenerated. Hand-mirrored content replaced by `cargo xtask regenerate-types` output. Header banner makes the generated nature explicit; `.prettierignore` + `eslint.config.js` exclude the path so prettier/eslint don't fight the codegen formatter. The drift check in CI catches future divergence between schema and generated TS.
- **`crates/xtask/tests/check_drift.rs`** — Case 4 added: mutates `src/types/agent_event.ts`, runs `regenerate-types --check`, asserts non-zero exit, restores. Mirrors existing Case 3 for Rust drift.
- **`crates/runtime-drone/tests/integration.rs` + `integration_windows.rs`** — `drone_binary()` retrofitted to derive paths from `std::env::current_exe()` instead of `CARGO_MANIFEST_DIR`-relative `target/debug/runtime-drone`. Per `docs/gotchas.md` #22: `cargo llvm-cov --workspace` uses a distinct target dir that breaks hard-coded paths. Archetype: `crates/runtime-main/tests/drone_ipc_loopback.rs::drone_binary`.
- **`package.json`** — `"test"` script flipped from `vitest run` → `vitest run --coverage` so the 80% threshold in `vitest.config.ts` is enforced on every run (M02.E carry-forward — the threshold was configured but only triggered when `--coverage` was passed explicitly).
- **`src/counter.{js,test.js}`** — deleted. Legacy CommonJS files predating the M02 `"type": "module"` flip; were carried forward via `.prettierignore` + `eslint.config.js ignores`. The ignore-list entries are now removed.
- **Workspace `Cargo.toml`** — `secrecy` dropped the `serde` feature. Per `docs.rs/secrecy/0.10`: `SecretString` does NOT serialize via serde by default (the feature requires the `SerializableSecret` marker trait, which no M02 code implements). The feature was dead weight; verified by grep on `secrecy::Serialize` / `serialize_with` / `Deserialize` over `SecretString`.
- **`package.json`** — Vite 5.4 → ^7.1.0 (the dev-server esbuild advisory in 5.x is in the moderate-vulns chain that `npm audit --audit-level=high` already filters out, but the bump closes the M02.E surprise event 4 carry-forward). Vite 8 (Rolldown) is GA but out-of-scope per the M03 stage prompt's `<execution_warnings>`; defer to M04+.
- **`package.json`** — added `@xyflow/react ^12.10.0` + `zustand ^5.0.0` (production deps for Stages B–F React Flow + state management) + `json-schema-to-typescript ^15.0.0` (devDep used by the new xtask TS codegen). `keyring 3.6` stays per the M03 stage prompt's `<execution_warnings>` — 4.0 has breaking API surface and is deferred to a dedicated chore PR after M03 ships.

Refs: `docs/build-prompts/M03-live-graph.md` §A; `agent-runtime-spec.md` §3 §13.5; `CLAUDE.md` §5 §14; `docs/gotchas.md` #21–#28 (especially #22 `current_exe`); `M02-summary.md` §"Decisions to apply before the next parent milestone"; `docs/gap-analysis.md` M02 entry §"Carry-forward to M03 prep".

### Fixed — Post-M02 smoke-test live debugging

Live debugging a "[object Object]" smoke-test failure in the M02 desktop app surfaced four overlapping issues. All four are fixed here in one PR; the underlying spec/process gap (dev-logging discipline) is locked into the spec so future milestones don't repeat the silent-stub trap.

- **`Cargo.toml` — keyring 3.x platform features.** Bare `keyring = "3.6"` ships NO platform backend by default; the workspace dep was missing the `apple-native` / `windows-native` / `sync-secret-service` features. Result: the keyring crate compiled but used a stub backend that silently succeeded on writes and returned `NoEntry` on reads. Symptoms in M02 dev: "Save key ✓ stored in OS keychain" then `setup_required` on smoke test. Fix opts into all three OS backends (one-line change to the workspace dep). Per `docs/gotchas.md` #29.
- **`src/lib/ipc.ts` — typed `unwrapCmdError` helper.** Tauri renderer's `catch(e)` receives a serde-tagged JS object (e.g., `{type: "setup_required"}` or `{type: "provider", message: "..."}`); `e instanceof Error` is `false`; `String(e)` yields `"[object Object]"`. The new `unwrapCmdError(e: unknown): string` helper exhaustively handles `Error` instances, `CmdError` shape (with type + optional message), generic objects with `message`, and falls back to `String(e)`. Exported so M03+ command surfaces reuse it. Type definition matches the actual `CmdError` enum in `src-tauri/src/commands.rs`. Per `docs/gotchas.md` #30.
- **`src/App.tsx` — error logging at every `catch`.** Both `handleSetKey` and `handleSmoke` now `console.error('<context> error:', e)` before user-facing dispatch. Critical for diagnostics: without this, structured errors collapse to `"[object Object]"` in the UI with zero signal in the DevTools console. The change pairs with the `unwrapCmdError` helper — together they ensure every renderer-side error has a console log AND a user-readable string.
- **`src-tauri/src/main.rs` — `tracing_subscriber::fmt::init()`.** M02 wired `tracing::info!` / `tracing::error!` calls inside Tauri commands but never initialized the subscriber, so the calls emitted to a null sink. The fix adds an `init_tracing()` function called at the top of `main()` with `EnvFilter`-based level config (default `info` globally, `debug` for project crates; `RUST_LOG` overrides). Adds `tracing` + `tracing-subscriber` (with `env-filter`, `fmt` features) to `src-tauri/Cargo.toml`. Per `docs/gotchas.md` #31.
- **`src-tauri/src/commands.rs` — minimum-viable command-level instrumentation.** `set_api_key` and `run_smoke_session` now log entry (`info!`), failure paths (`error!` with `error = %e` + which sub-step), and success (`info!`). API key VALUES are never logged (only `key_len` for `set_api_key`); `SecretString` wrapping ensures `Debug` output is `[REDACTED]`. Per `agent-runtime-spec.md` §13.5 (new this PR).

### Documentation — Spec §13.5 + gotchas #29–#31

Locks the dev-logging discipline that the M02 live debugging exposed as a structural gap.

- **`agent-runtime-spec.md` §13.5 Dev Logging** — new subsection inside §13 Privacy & Telemetry. Documents the dev/release boundary (zero-telemetry remains in force), the `tracing_subscriber::fmt::init()` requirement at every Rust binary's `main()`, the per-Tauri-command instrumentation pattern (entry / success / error logs), the renderer-side `console.error` + `unwrapCmdError` pattern, the secrets-redaction invariant (`SecretString` for API keys; structural-only logging for user content), what release mode does differently (JSON formatter, log files at `$DATA_DIR/logs/{date}/`), and what dev mode does NOT do (no telemetry, no automatic diagnostics, no phone-home-on-crash). Includes the per-milestone logging-requirements gate that §13.5 reviews land in closeout stages.
- **`docs/gotchas.md`** — three new entries (#29, #30, #31) consolidating the M02 live debugging traps:
  - **#29** keyring 3.x stub backend (no platform features by default)
  - **#30** Tauri renderer's `catch(e)` gets non-Error objects from serde-tagged enums
  - **#31** Tauri main process binary needs `tracing_subscriber::fmt::init()` in `main()`

### Documentation — Post-M02 spec lock + ADR-0006

Per `M02-summary.md` Decisions + `docs/gap-analysis.md` M02 entry Fix
Backlog. Locks the M02 architectural decisions into the spec so M03+
implementations don't have to re-decide. Pairs with the
post-M02-protocol-iteration PR (gotchas + retrospective + template
carry-forwards) — both PRs are pre-M03 housekeeping.

- **`agent-runtime-spec.md` §2c LLMProvider Abstraction** — two new
  subsections locking the M02 SSE wire-format + ProviderEvent semantics:
  - **§2c.1 Anthropic SSE wire format** — full event-set table with
    payload + ProviderEvent mapping; specific call-out for
    `signature_delta` (verifier-only; consumed silently) and `ping` (SSE
    keep-alive; consumed silently). Pre-M02 spec drafts didn't document
    these; M02 implementation discovered them live and they tripped
    fresh implementations as "unknown event type" warnings.
  - **§2c.2 ProviderEvent::Error semantics** — locks `Error` as
    **terminal**: stream yields Error then terminates without
    MessageStop. Retry logic lives in AgentSdk task layer, not provider
    layer (cost-runaway + correctness rationale documented). Adds
    cancellation-safety language: provider stream is cancellation-safe;
    dropping mid-burst drops the underlying reqwest::Response.
- **`agent-runtime-spec.md` §2b Signals & VDR Projection** — adds a
  ⚠️ note flagging the `signal::ContextType` enum's divergence from
  spec's `context.type ∈ {skill, framework, code, search, verify,
  commit, subagent}` set. M02's runtime scaffold uses operation-context
  variants (`AgentLoop / SkillLoad / ToolInvoke / HookExecute /
  PlanCreate / HitlPrompt / SessionLifecycle`); reconciliation deferred
  to M04 closeout when emission integration provides evidence on which
  shape is correct.
- **`agent-runtime-spec.md` §1d IPC Channels** — new "Reconnect
  semantics" subsection documenting the 5-attempt 200ms→3.2s
  exponential backoff M02.D landed in `DroneClient::send_with_reconnect`,
  the `*_with` testable seam pattern, and the open M03-blocking
  question on long-lived events() subscription survival across
  reconnect.
- **`agent-runtime-spec.md` §10 Persistence Layer** — adds ⚠️ note
  alongside the `mcp_servers` table definition flagging the divergence
  from the documented 7-field shape (the shipped table is 22 fields)
  and pointing readers to ADR-0006 for the full rationale.
- **`agent-runtime-spec.md` Project Structure** — runtime-main module
  listing updated to reflect M02 actuals (sdk/, providers/, drone_ipc/,
  key_store.rs, etc.) plus per-file milestone tags (M02 / M04 / M05 /
  M06 / M06+ / M07 / M09) so readers can see what's shipped vs what's
  forward-looking.
- **`docs/adr/0006-mcp-servers-schema.md`** — new ADR (Accepted)
  documenting the 22-field `mcp_servers` schema's divergence from spec
  §10's 7-field shape. Per-field rationale table covers transport set
  (stdio/http/sse/streamable_http), stdio-vs-remote mutual-exclusion
  CHECK, OAuth refresh state persistence, capability discovery cache,
  scope/plugin_id, retry+timeout policy, lifecycle audit fields. Four
  alternatives rejected (match 7-field exactly; split tables; single
  JSON column; defer to M06 Stage A) with explicit reasoning. Target
  was "before M06 Stage A"; landed during post-M02 housekeeping.
- **`docs/MVP-v0.1.md` §M2 / §M3** — Tauri 2.x E2E framework note.
  §M2 Out-of-scope clarifies M02 ships renderer-level Playwright
  against Vite dev server (`@tauri-apps/api` module-mocked); full
  desktop-shell E2E is M03 carry-forward. §M3 deliverable adds
  `tauri-driver` + WebdriverIO matrix (Linux + Windows; macOS
  unsupported), wires the four `test.skip()` carry-forward Playwright
  tests, adds CI E2E acceptance criterion. §M3 out-of-scope adds
  "macOS Tauri-shell E2E (tauri-driver does not support macOS —
  deferred indefinitely)".

### Documentation — Post-M02 protocol iteration

Per `CLAUDE.md` §19 + `M02-summary.md` Verdict ("Pattern held but with
friction") prescribed protocol-iteration session before M03 authoring opens.
Lands the carry-forward decisions from M02.A–E retrospectives into the
shared protocol docs so M03 stages don't re-discover the same friction.

- **`docs/gotchas.md`** — eight new entries (#21–#28) consolidating M02
  carry-forwards: clippy pedantic+nursery patterns (compound entry covering
  9 sub-patterns), `current_exe()`-derived subprocess test paths, Tauri 2.x
  E2E uses `tauri-driver` + WebdriverIO (not Playwright `_electron`), ESLint
  9 flat-config default, Vite root convention, `serde(tag = "type")` requires
  struct-shape variants, Vitest+RTL DOM-ref-staleness pattern, bound
  test-fixture streams.
- **`docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md`** — new
  `[END] Coverage holdouts` subsection between Threshold evaluation and
  Decisions for next stage. Records workspace + per-package coverage
  actuals, exclusions added this stage, current exclusion list, per-module
  baselines (preserved-or-improved invariant per CLAUDE.md §5), and a
  doc-to-CI drift check. Replaces the historical scatter across CLAUDE.md
  §5 + per-stage `[END] Decisions`.
- **`docs/build-prompts/TEMPLATE.md`** — four additions to the milestone
  prompt template:
  - **`WEBCHECK:` header** at each stage's title block — required when the
    stage touches fast-moving tooling surfaces (npm / Tauri / esbuild /
    Vite / etc.). Lists authoritative URLs to web-verify against the prompt
    body before the fresh session opens. Per CLAUDE.md §12 web-first rule.
  - **Pre-existing legacy file inventory** subsection in milestone-level
    Background — required when this milestone touches a tree a prior
    milestone created. Lists tracked-but-orphaned files that prettier /
    eslint will scan with disposition (delete / preserve / refactor).
  - **Pedantic-pass preflight** checklist in the Stage X.4 Tests section
    template — clippy pedantic+nursery patterns to verify against new
    modules before writing the test plan. Cross-references gotchas.md #21.
  - **Default test plan for stages adding a new safety primitive** — codifies
    the M01.C / M02.A / M02.C / M02.D / M02.E pattern: "N unit tests for
    the testable seam + M integration tests for end-to-end behavior."
  - **Doc-to-CI invariant** addition under Safety primitive coverage gate —
    requires updating CI workflow + CLAUDE.md §5 + per-stage retro
    Coverage holdouts subsection in the SAME commit when adding a new
    coverage exclusion. Cites the M02.E `key_store.rs` drift bug as the
    cautionary tale.

### Added — M02.E (Tauri shell + skeleton renderer + frontend CI gates + Playwright)

- `package.json` + full frontend tooling (Vite 5.4, TypeScript 5.6 strict,
  React 18.3, Vitest 2.1, Playwright 1.48, ESLint 9 flat-config, Prettier 3,
  `@testing-library/react`, `@testing-library/user-event`,
  `@testing-library/jest-dom`, `happy-dom` 20.x).
- `src/` skeleton renderer:
  - `App.tsx` composes `SetupPanel` + `SmokeButton` + `EventList`; state via
    `useReducer` over a pure reducer (`lib/eventReducer.ts`) with full
    immutability.
  - `lib/ipc.ts` — typed wrappers over `@tauri-apps/api/core::invoke` and
    `@tauri-apps/api/event::listen`.
  - `lib/eventReducer.ts` — pure reducer + `Action` discriminated union.
  - `types/agent_event.ts` — TypeScript discriminated union mirroring
    `runtime_core::AgentEvent` v0.1 subset (10 variants Stage D emits +
    `ToolSource` enum). M03+ regenerates from `schemas/event.v1.json` via
    `cargo xtask regenerate-types` per CLAUDE.md §14.
  - `components/{EventList,SetupPanel,SmokeButton}.tsx` — minimal
    accessible markup; password-input invariant for the API-key field.
  - `index.html` + `styles.css` + `main.tsx` (React 18 root).
- `crates/runtime-main/src/key_store.rs` — OS-keychain-backed API key
  storage via the `keyring` crate. Reads return `SecretString` so the key
  never `Debug`-prints; `delete_api_key()` is idempotent (treats `NoEntry`
  as success). 2 unit tests + 2 keychain-gated `#[ignore]` tests for
  read-after-write round-trip + missing-entry → `NotFound` mapping.
- `src-tauri/src/commands.rs` — `set_api_key` and `run_smoke_session`
  Tauri commands. `CmdError` serializes with `serde(tag = "type")` for
  renderer pattern-matching. The testable seam
  `run_smoke_session_with(provider, event_tx, config)` (M01.C / M02.C /
  M02.D `*_with` archetype) accepts an injectable `LLMProvider` + channel
  so unit tests exercise the SDK→event flow without crossing reqwest or
  the Tauri `AppHandle`.
- `src-tauri/capabilities/default.json` — locked-down capability set:
  `core:default` + `core:event:{default,allow-listen,allow-emit}` only.
  No `shell:*`, no `fs:*`, no `http:*`, no `dialog:*`. Per spec §10
  capability boundary; CLAUDE.md §15 trap #10 (Tauri allowlist is the
  security boundary).
- Tests:
  - 14 frontend unit tests across `tests/unit/eventReducer.test.ts` (10)
    + `tests/unit/ipc.test.ts` (5) — pure reducer immutability + IPC
    wrapper call-shape + subscriber lifecycle.
  - 11 component tests across `tests/unit/components.test.tsx` (8)
    covering password-input invariant, save-button enabled-when-key-min-
    length, EventList aria-label + data-event-type attrs, all 10
    `AgentEvent` variant render paths.
  - 2 App-level state-machine tests in `tests/unit/App.test.tsx` —
    save-key-then-run-smoke happy path + command-error surface (mocks
    `@tauri-apps/api` at the module level).
  - 4 Playwright renderer-level E2E tests in `tests/e2e/smoke.spec.ts`:
    renderer-loads-with-setup-visible, password-input-type, smoke-
    disabled-without-key, save-key-then-run-disables-button-during-run.
  - 4 Rust unit tests in `src-tauri/src/commands.rs::tests` —
    `cmd_error_serializes_with_type_tag`, `from_keystore_not_found_maps_
    to_setup_required`, `run_smoke_session_with_emits_events_to_channel`,
    `smoke_config_targets_haiku_with_tight_budget`.
  - 2 unit tests + 2 keychain-gated tests in
    `crates/runtime-main/src/key_store.rs::tests`.
- CI:
  - `frontend` job (existed from M02.A) updated to run prettier on
    `**/*.{ts,tsx,js,jsx,json}` (markdown + YAML excluded via
    `.prettierignore` since markdown is checked by the existing
    `markdown-lint` job and YAML is structurally validated by the
    Actions runner).
  - `e2e` job added — installs Playwright + Chromium, runs
    `npm run test:e2e` against the Vite dev server.
  - `runtime-main` coverage gate exclusion list extended to add
    `src/key_store.rs` (the keychain-call paths are platform-bound and
    `#[ignore]`-gated). runtime-main remains at 99.37% line; workspace
    at 94.51% line. CLAUDE.md §5 + §6 updated.

### Documentation — M02.E

- `CLAUDE.md` §6 — frontend gates section made authoritative; E2E gates
  subsection added with the Tauri 2.x platform note (full desktop-shell
  E2E requires `tauri-driver` + WebdriverIO per the official Tauri docs;
  Stage E ships renderer-level Playwright against the dev server).
- `crates/runtime-main/README.md` — `key_store` module documented;
  Tauri command surface (`set_api_key` + `run_smoke_session` +
  `CmdError` shape) documented with the testable-seam pattern note.
- `docs/MVP-v0.1.md` §M2 — acceptance criteria all marked `[x]`; the
  `tool_invoked (LoadSkill)` sub-criterion noted as M03+ work since
  skills don't exist at M02.

### Status — M02.E

Stage E is the final implementation stage of M02. Stage F (Phase Closeout:
Gap Analysis) follows in a fresh session per CLAUDE.md §20.

### Documentation — M02.F (Phase Closeout: Gap Analysis)

- `docs/gap-analysis.md` — M02 entry appended (commit `4bd809a`, Stage E).
  Cumulative product+spec audit across M01 + M02 per CLAUDE.md §20. Six
  sections: codebase deep dive, adherence to spec (1 ❌ "None observed";
  multiple ✅ holds for zero-telemetry, direct Anthropic API, SSE wire
  format, Tauri capability lockdown, schemas-as-source-of-truth, `*_with`
  test-seam pattern, coverage delta gating, mcp_servers + Windows IPC test
  + .gitattributes carry-forwards closed; ⚠️ items for the M04-deferred
  decision-extractor heuristic, count_tokens approximation, EventPipeline
  ToolResult duration_ms placeholder, ContextType enum diverging from spec
  §2b's value set (M04 closeout reconciles — direction undetermined),
  mcp_servers schema deliberately richer than spec §11 (ADR before M06),
  vitest threshold not yet enforced by default, Tauri 2.x desktop-shell
  E2E deferred to M03, hand-mirrored TS AgentEvent), spec review
  (forward-looking — signature_delta + ping events, IPC reconnect surface,
  ProviderEvent::Error terminal semantics, Phase 3 spec expansion, Session
  FSM diagram, plan model field shapes, model deprecation policy, error.v1
  schema), fix backlog (0 Critical; ~17 🟡 Important spanning M03/M04 prep
  + post-M02 docs(spec) PR + CLAUDE.md+TEMPLATE.md consolidation; ~5 🟢
  Nice-to-have including vite 5→7 bump and counter.{js,test.js} cleanup),
  carry-forward (M01 🟡 mcp_servers / coverage delta gating / *_with /
  Windows drone integration test / .gitattributes / post-M01 docs(spec)
  PR all RESOLVED; M01 🟡 Phase 3 spec expansion / Session FSM diagram /
  UI consistency STILL OPEN per their target milestone). Append-only
  invariant verified locally and via `git diff origin/main` (line 706+ is
  pure addition). Per CLAUDE.md §20 the entry is **immutable** once
  committed.
- `docs/build-prompts/retrospectives/M02-summary.md` — new. Per-parent-
  milestone roll-up aggregating M02.A–E retrospectives. Mean Process
  38.6/40, Product 39.4/40, Pattern 29.2/35; verdict **"Pattern held but
  with friction"** (all hard gates passed; Stage E sound-but-rough due to
  8 Sev-2-or-3 prompt-drift items including Tauri 2.x E2E framework, ESLint
  flat-config, Vite root convention, serde tag-shape, Vitest+RTL idiom).
  Decisions to apply before M03.1 authoring documented.

### Status — M02.F

Stage F is the final stage of M02 per CLAUDE.md §20. The Stage F commit is
the last on `claude/m02-event-pipeline`; the M02 PR push is gated on this
commit. The M02 PR aggregates all stage commits + per-stage retrospectives
+ M02-summary.md + the new gap-analysis entry for three-artifact review
per CLAUDE.md §19 + §20.

### Added — M02.D (AgentSdk + drone IPC client + event translation)

- `crates/runtime-main/src/sdk/agent_sdk.rs` — `AgentSdk<P: LLMProvider>`
  agent loop. Generic over the provider trait so v1.0+ providers slot in
  unchanged. Constructs the provider stream in `run_agent(config)`; the
  test-seam variant `run_agent_with_provider_stream(stream)` accepts any
  pre-built `Stream<ProviderEvent>` (mirrors the M01.C / M02.C `*_with`
  archetype). Emits `AgentSpawned` first, drives the `EventPipeline` to
  exhaustion, flushes buffered text. `SessionId` newtype wraps `Uuid`.
- `crates/runtime-main/src/sdk/event_pipeline.rs` — pure
  `ProviderEvent` → `AgentEvent` translator. Consecutive `TextDelta`s
  bundle into a single `StreamText` per non-text event boundary;
  flushed on `ThinkingDelta`, `ToolUse`, `ToolResult`, `MessageStop`,
  `Error`, and end-of-stream. Decision extraction runs at every flush
  and prepends a `DecisionRecord` when matching markers are present.
- `crates/runtime-main/src/sdk/decision_extractor.rs` — first-line
  `Decision:`/`Rationale:` heuristic per spec §2 `decision_record`.
  Pure function; line-by-line scan tolerates intervening blank lines
  and leading whitespace; last `Decision:`/`Rationale:` pair wins.
  Optional `Tool used:` capture. Property test verifies no panic on
  arbitrary input. M04 verify+rails replaces the heuristic with a
  structured emitter.
- `crates/runtime-main/src/drone_ipc/client.rs` — `DroneClient` main-
  side IPC client. Connects via `DroneClient::connect(addr)` (cfg-
  platform Unix `UnixStream` / Windows `NamedPipeClient`). Test-only
  `DroneClient::noop()` short-circuits all sends. `events()` returns a
  single-consumer stream of `Result<DroneEvent, DroneIpcError>`.
- `crates/runtime-main/src/drone_ipc/connection.rs` — connection state
  machine + reconnect policy. Exponential backoff: 200ms → 400ms →
  800ms → 1.6s (4 sleeps for 5 attempts; no trailing sleep). Surfaces
  `DroneIpcError::Disconnected { retries }` on exhaustion.
  `Connection::from_streams` is the testable seam taking already-opened
  read+write halves; the `open()` cfg-platform OS-call wrapper is the
  coverage holdout.
- `crates/runtime-core/src/event.rs` — `ToolSource { Builtin, Mcp,
  Generated }` enum added; `AgentEvent::ToolInvoked` gains `source` +
  `server` fields; `AgentEvent::AgentSpawned` gains `session_id`.
  Property tests round-trip the new shape per the M01.B pattern.
- `crates/runtime-main/tests/sdk_event_translation.rs` — 20 table-
  driven translation tests + 1 proptest covering bundling boundaries,
  decision extraction, error-path translation, multi-tool sequencing,
  buffer drain semantics, agent-id propagation.
- `crates/runtime-main/tests/sdk_cancellation.rs` — 5 drop-mid-stream
  cancellation-safety tests using `tokio::time::timeout` +
  `futures::stream::iter` patterns. Verifies no panic on drop, channel
  drains to `Closed`, back-pressure does not panic.
- `crates/runtime-main/tests/drone_ipc_loopback.rs` — 10 end-to-end
  tests spawning the M01 `runtime-drone` binary, exercising every
  `DroneCommand` variant, the `SnapshotWritten` event surface, and the
  reconnect / disconnect surface paths.
- `runtime-main` safety-primitive coverage gate extended to span
  `sdk/` and `drone_ipc/`. Exclusions: `providers/anthropic.rs` (Stage
  C real-network wrapper) plus `drone_ipc/connection.rs::open` (cfg-
  platform OS-call holdout); the testable seam
  `Connection::send_with_reconnect` is fully covered. CI gate +
  `CLAUDE.md` §5 updated.

### Changed — M02.D

- `crates/runtime-main/Cargo.toml` — add `tokio-util` (`codec`
  feature), `uuid` (`v4` + `serde`), `tempfile` + `rusqlite[bundled]`
  (dev-deps for the loopback test).
- `crates/runtime-main/src/lib.rs` — top-level module declarations
  (`pub mod sdk; pub mod drone_ipc;`).
- `crates/runtime-main/README.md` — appended §"Agent SDK" with the
  `ProviderEvent` ↔ `AgentEvent` mapping table and `DroneClient`
  reconnect policy notes.

### Added — M02.C (AnthropicProvider real HTTP+SSE)

- `crates/runtime-main/src/providers/anthropic_sse.rs` — SSE state
  machine + parser. `SseEvent` enum mirrors the Anthropic Messages API
  wire format (`message_start`, `content_block_start/delta/stop`,
  `message_delta/stop`, `ping`, `error`). `SseState` accumulates tool
  input partial-JSON deltas across `content_block_delta` events; emits
  the complete `ToolUse` on `ContentBlockStop`. `signature_delta` is
  parsed and silently dropped (verifier-only payload).
- `crates/runtime-main/src/providers/anthropic.rs::stream` — real
  HTTP+SSE implementation. Direct `reqwest` + `eventsource-stream`; no
  third-party Anthropic SDK. Lazy `OnceLock<reqwest::Client>` per
  provider instance. Maps non-2xx responses: 401/403 → `Auth`; 429 →
  `RateLimit { retry_after_secs }` (parsed from the `retry-after`
  header, default 60); other → `Api { status, body }`.
- `crates/runtime-main/tests/anthropic_wiremock.rs` — 8 wiremock-driven
  integration tests covering happy path, auth failure, rate limit, tool
  use accumulation, thinking + signature passthrough, server-emitted
  error, malformed bytes skipped, and partial-chunk reassembly.
- `crates/runtime-main/tests/anthropic_smoke.rs` — real-API smoke gated
  by `--features integration`; reads keychain entry
  `agent-runtime/anthropic`. CI never runs this; cost ~$0.001 per run
  against Haiku 4.5.
- `runtime-main` added to the safety-primitive coverage gate matrix
  (≥95% line) with `src/providers/anthropic.rs` excluded as the
  real-network production wrapper. CI gate + CLAUDE.md §5 updated.

### Changed — M02.C

- `crates/runtime-main/Cargo.toml` — add `bytes` dep (direct, was
  transitive via reqwest) for the SSE state machine's stream type
  bound, and `wiremock` dev-dep for the integration tests.
- Workspace `Cargo.toml` — pin `wiremock = "0.6"` in
  `[workspace.dependencies]`.
- `crates/runtime-main/README.md` — add real-API smoke-test section
  with platform-specific keychain setup notes.
- `crates/runtime-main/src/providers/anthropic.rs` — remove the now-
  obsolete `stub_stream_returns_text_then_stop` test (the wiremock
  `happy_path_yields_text_deltas_and_message_stop` covers the same
  end-to-end shape against the real HTTP+SSE pipeline).

### Added — M02.B (LLMProvider trait + AnthropicProvider stub)

- `crates/runtime-main/src/providers/mod.rs` — `LLMProvider` trait,
  `ProviderEvent` enum (`TextDelta` / `ToolUse` / `ToolResult` /
  `ThinkingDelta` / `MessageStop` / `Error`), `ProviderError`
  (thiserror-derived), and supporting types (`AgentConfig`, `Message`,
  `ContentBlock`, `ImageSource`, `ToolResultContent`, `ModelInfo`,
  `Pricing`, `CostBreakdown`, `ProviderSupport`, `ModelCapabilities`)
  per spec §2c.
- `crates/runtime-main/src/providers/anthropic.rs` — `AnthropicProvider`
  shell. `SecretString`-wrapped API key; stub `stream()` returning
  hardcoded `TextDelta + MessageStop` sequence; hardcoded `list_models()`
  (Opus 4.7, Sonnet 4.6, Haiku 4.5); char-based `count_tokens()`;
  cache-aware `estimate_cost()` (5m write 1.25× / 1h write 2× / read
  0.1× input). Stage C replaces the stub body with real HTTP+SSE.
- `crates/runtime-main/README.md` — public API documentation per
  CLAUDE.md §6.
- Workspace dependencies (no third-party Anthropic SDK): `reqwest`
  (rustls-tls + json + stream), `eventsource-stream`, `async-trait`,
  `secrecy`, `keyring`, plus a path-dep entry for `runtime-core`.

### Added — M02.A (Build hygiene + scaffolding)

- `crates/runtime-core/src/signal.rs` — Signal Schema v2 type scaffold per
  spec §2b (8-variant `Signal` enum + `ContextType` + correlation field
  types `PreSignalId` / `ParentSignalId` / `RetryOfSignalId`). Emission
  integration is M04+ work; M02.A ships the type surface so M03+ work can
  import without churn.
- `crates/runtime-core/src/drone.rs::HeartbeatStatus` typed enum
  (`Ok`/`Degraded`/`Stalled`) replaces the prior `String`. Implements
  `Display` + `FromStr` so SQLite text storage round-trips through the
  enum. Closes M01 gap-analysis Important "HeartbeatStatus typed enum"
  per spec §1d (PR #36 closeout).
- `crates/runtime-drone/src/db.rs::init_schema` — 8th SQLite table
  `mcp_servers` per spec §11:2435-2444 + MCP best-practice (Claude Code
  / Claude Desktop / VS Code MCP client schemas). 22 columns covering
  identity, transport-specific config (stdio/http/sse/streamable_http),
  authentication (keychain refs, never literal secrets), connection
  lifecycle, timeouts, scope tracking, capability cache; SQL CHECK
  constraints enforce the stdio-vs-remote mutual exclusion. Schema only;
  MCP client lands in M06.
- `crates/runtime-drone/tests/integration_windows.rs` — Windows-platform
  end-to-end test exercising `ipc::accept_loop` over named pipe: spawns
  drone, sends `SnapshotNow`, verifies SQLite row, sends
  `GracefulShutdown`, verifies clean exit. Sister to the existing
  `tests/integration.rs` Unix SIGTERM lifecycle test; together they
  cover §0d Windows-only release scope.

### Changed — M02.A

- `crates/runtime-drone/src/command_handler.rs::run` accepts an optional
  `oneshot::Sender<&'static str>` and signals it on `GracefulShutdown`,
  driving full drone-process exit through the IPC channel. `run_inner`
  selects between the OS-signal future and the IPC-shutdown future to
  unify cross-platform graceful shutdown.
- Workspace coverage gate adds delta-gating from M02 onward (Codecov
  project: `target: auto`, `threshold: 0.5%`; patch: `target: 80%`).
  Per-crate Codecov flag uploads added for `runtime-drone` and
  `runtime-main`. Documented in `CLAUDE.md` §5 "Coverage delta gating
  (from M02 onward)".

### Documentation — M02.A

- `docs/style.md` — `*_with` / `*_inner` test-seam pattern documented
  as the canonical TDD-friendly approach to OS-driven async functions.
  Cites M01.C archetype at `crates/runtime-drone/src/{lib,shutdown}.rs`
  and codification commit `1dec4ba`.
- `.gitattributes` — explicit LF normalization for `*.rs`, `*.toml`,
  `*.json`, `*.md`, `*.yml`, `*.sh`, `*.bash`, `*.py`, `*.html`, `*.css`,
  `*.js`. Closes M01 gap-analysis Important "line-ending normalization".
- `.gitignore` — `src-tauri/gen/schemas/` excluded; the four
  Tauri-generated files (`acl-manifests.json`, `capabilities.json`,
  `desktop-schema.json`, `windows-schema.json`) untracked but kept on
  disk. Closes M01 PR #36 follow-up "src-tauri/gen/schemas/ should be
  gitignored to prevent future drift".

### Added

- **Spec §15 Sharing & Distribution + ADR-0005** — three sharing tiers
  declared (runtime-to-runtime in v0.1 via M07; headless CLI
  `agent-runtime-cli` in v1.0; WASM in v2.0+); cross-OS portability
  rules (POSIX-only paths, `compatible_os` declaration); the "Share It"
  module forward-declared as v1.0 deliverable in M08+. Four additive
  optional fields in `schemas/framework.v1.json` (`requires_secrets`,
  `runtime_dependency_class`, `compatible_os`, `share_provenance`)
  ship as v0.1 schema groundwork so M03–M07 frameworks are
  forward-compatible with the v1.0 headless CLI and Share It module
  without schema migration. Minor in-place schema bump per
  `schemas/README.md` versioning policy; `$id` unchanged. MVP-v0.1.md
  §M07 updated to emit `share_provenance` on export and validate the
  four fields on import; §M08 forward-declares the Share It module.
  Generated Rust types (`crates/runtime-core/src/generated/framework.rs`)
  and TypeScript types (`src/types/framework.ts`) **must be regenerated
  via `cargo xtask regenerate-types` before this changeset's PR merges**
  — the type-drift CI gate (per CLAUDE.md §14) blocks merge otherwise.
  Regen happens on a Rust-capable machine (Windows / macOS / Linux);
  the agent environment that authored the schema/spec/ADR changes does
  not have a usable cargo toolchain.
- **M01 Foundation milestone** — Cargo workspace with five member crates
  (`runtime-core`, `runtime-main`, `runtime-drone`, `runtime-sandbox`,
  `xtask`) plus Tauri stub at `src-tauri/`, workspace lints (deny
  warnings, forbid unsafe except sandbox, clippy pedantic + nursery),
  and a `cargo-deny` policy. `rust-toolchain.toml` pins channel to
  `stable`; MSRV enforcement lives in workspace `Cargo.toml`.
- **Type-generation pipeline** — `cargo xtask regenerate-types` reads
  `schemas/*.v1.json` via [`typify`](https://crates.io/crates/typify)
  and writes to `crates/runtime-core/src/generated/`. CI runs
  `--check` on every PR to fail on any drift between committed types
  and freshly regenerated output.
- **Hand-curated event taxonomy in `runtime-core`** — `AgentEvent`
  (full variant list per spec §2 + §2a + §2b + §3a + §3b + §4a + §4b
  + §6a + §8.security), `DroneEvent` + `DroneCommand` per spec §1d,
  `RuntimeError` via `thiserror`.
- **Drone Phase 1 (`runtime-drone`)** — heartbeat task (5s tokio
  interval) writing `heartbeats` rows and emitting
  `DroneEvent::Heartbeat`; append-only snapshot writer with SHA-256
  `state_hash`; platform-specific IPC server (Unix domain socket on
  Linux/macOS, Windows named pipe via `tokio::net::windows::named_pipe`)
  with framed JSON-newline via `tokio_util::codec::LinesCodec` and
  malformed-input tolerance (emits `Alert`, keeps server alive);
  SIGTERM / SIGINT / CTRL_BREAK / CTRL_C handler with best-effort
  emergency snapshot before exit. SQLite WAL pragmas applied in correct
  order (`journal_mode → synchronous → busy_timeout → foreign_keys`);
  7-table schema (`sessions`, `snapshots`, `signals`, `heartbeats`,
  `vdr`, `token_usage`, `skills`).
- **Runtime-drone safety-primitive coverage gate** — ≥95% line with
  `lib.rs` + `shutdown.rs` excluded (OS-signal orchestrators exercised
  end-to-end by the Unix subprocess integration test). Per-module
  baseline (M01.C measured): `snapshot.rs` 100%, `db.rs` 98.82%,
  `heartbeat.rs` 98.59%, `command_handler.rs` 97.94%, `ipc.rs` 84.70%.
  Workspace coverage gate: ≥80% line, generated code and binary stubs
  excluded.
- **Fuzz harness** — cargo-fuzz `drone_command_decode` target for the
  IPC frame decoder with 6 seed corpus entries (one per
  `DroneCommand` variant). CI fuzz-smoke job runs 30s on every PR;
  scheduled `fuzz-nightly.yml` workflow runs 1 hour at 04:00 UTC and
  uploads the corpus on failure.
- **Per-crate READMEs** — `runtime-core`, `runtime-drone`, and `xtask`
  document the public API surface, IPC protocol, SQLite schema,
  manual smoke procedure, platform-specific details, and the
  coverage requirement.

### Tests

- **Schema round-trip tests** — `examples/aria/framework.json`,
  `examples/ralph/framework.json`, and 19 skill / agent / tool
  frontmatter files all round-trip through generated `runtime-core`
  types via the serialize-deserialize-serialize stability check.
- **Property tests** — `proptest` round trips for `AgentEvent`,
  `DroneEvent`, `DroneCommand`, including the newline-delimited JSON
  codec wire format.
- **Drift-check positive and negative cases** in `xtask`.
- **Drone unit tests** (22 total) — WAL pragmas, schema, snapshot
  append-only and SHA-256 hash, heartbeat interval, IPC encode /
  decode, command dispatch, malformed-input → `Alert`, broadcast
  lagged path.
- **Subprocess-spawn integration test** (`tests/integration.rs`,
  `#[cfg(unix)]`) — drone responds to SIGTERM with an emergency
  snapshot.
- **Fuzz target compiles and runs** — `cargo +nightly fuzz build`
  succeeds on Linux/macOS/Windows; `cargo +nightly fuzz run … -- 
  -max_total_time=30` exits 0 with no panics on Linux CI.

### Documentation

- Per-crate READMEs (`runtime-core`, `runtime-drone`, `xtask`).
- M01 Foundation specification + per-stage prompts at
  `docs/build-prompts/M01-foundation.md` (Stages A through E).
- M01 Phase Closeout: cumulative gap analysis appended to
  `docs/gap-analysis.md` per `CLAUDE.md` §20 (append-only living
  document). Gates the M01 PR. CI gains a `gap-analysis-append-only`
  job that enforces the immutability of prior entries on every PR.
- Per-stage retrospectives at
  `docs/build-prompts/retrospectives/M01.{A,B,C,D}-retrospective.md`
  + parent-milestone summary at `M01-summary.md` (per `CLAUDE.md` §19).
- Comprehensive product specification (`agent-runtime-spec.md`)
  covering project positioning, capability matrix, three-concept model
  (Tool/Skill/Agent), dev loop, release scope matrix, drone, recovery,
  multi-session, IPC, event pipeline, budget, signals/VDR,
  LLMProvider abstraction, live graph, plan/task primitive,
  mode/sizing, gap detection, verify/rails, MCP manager, framework
  loader, HITL policy, registry, generators with 5-layer security,
  builder canvas, persistence, secrets vault, reconciliation/degraded
  modes, engineering charter, privacy/telemetry, first-run UX.
- JSON Schema source-of-truth files in `schemas/` (Draft 2020-12):
  `common.v1.json`, `skill.v1.json`, `tool.v1.json`, `agent.v1.json`,
  `framework.v1.json`. All 19 example artifacts validate.
- `examples/aria/` reference framework reconstructing every row of
  the capability matrix.
- `examples/ralph/` sibling framework demonstrating the
  `loop_policy: continuous` variant; reuses `aria/` tools and skills
  via `source: external`.
- `docs/MVP-v0.1.md` build checklist (11 milestones; novice-and-
  experienced two-path success criterion).
- Engineering Charter in spec §12; Privacy & Telemetry in §13
  (zero telemetry by default); First-Run UX state machine in §14.
- ADR template + ADRs 0001–0004 (ARIA-as-archetype, Tauri-over-
  Electron, Engineering Charter adoption, defer paid code-signing).
- OSS scaffolding: `LICENSE` (Apache 2.0), `NOTICE`,
  `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CONTRIBUTING.md`.

### Changed

- **Code-signing posture for v0.1: deferred** (per ADR-0004). v0.1
  ships unsigned `.msi` with SHA-256 checksums and Sigstore provenance
  attestations via GitHub Actions OIDC. Paid Windows EV code-signing
  revisited at v0.5+ when adoption is proven. Affects:
  `docs/MVP-v0.1.md` M11 acceptance + risk register R4;
  `docs/README-v0.1.md` install instructions (SmartScreen-warning
  explainer + checksum/cosign verification steps);
  `.github/workflows/release.yml` (drops signing secrets, adds
  SHA-256 generation + `actions/attest-build-provenance@v1`);
  spec §0d distribution row.

### Status

M01 Foundation milestone complete. M02 (event pipeline +
`AnthropicProvider` + Tauri shell + `AgentEvent` flow) is the next
milestone.

---

## Versioning

- **0.x** — pre-stable. Schemas may change; APIs are not guaranteed compatible across 0.x versions.
- **1.0+** — semver strict. Breaking changes to framework JSON schema, AgentEvent union, or any `pub` Rust API require a major bump.

## Release artifacts

Once releases begin (v0.1.0 Windows Preview), each release will include:
- Signed Windows installer (`.msi`) at v0.1; macOS `.dmg` and Linux AppImage from v1.0.
- SBOM in CycloneDX format.
- Source tarball.
- SLSA Level 3 provenance attestations from v1.0.
