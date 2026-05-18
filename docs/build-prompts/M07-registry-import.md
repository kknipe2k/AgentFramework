# M07 — Registry Import

> **Protocol version:** v1.8 (per `STAGE-PROMPT-PROTOCOL.md` v1.8 — first milestone authored under v1.8; the three new authoring-time audit slots `<construction_reachability_check>` / `<wire_signature_audit>` / `<wire_trace_vs_adr_reconcile>` + the `<phase_doc_inventory_audit shape=…>` extension are used where they bite; closeout carries the required `<simplify_pass>` (v1.6) and `<coverage_policy_reconciliation>` (v1.8) children; Stage V runs the **mandatory** `--features integration` reference-MCP-server smoke per the M06.V Dec-7 codification).

## Background and Design Decision

M07 delivers **Registry import** (MVP §M7, week 13; spec Phase 7 §2156 + `skills.lock` §2181–2211): import an artifact by GitHub raw URL or local file → fetch → schema-validate → L3 sandbox → tier-gate review → install → `skills.lock` updated, with hash validation on every subsequent load. It is also where the **ADR-0011 (a)–(d) concrete-construction carry-forward** is discharged — the agent-with-tools loop + concrete `McpDispatcher` construction in `src-tauri` that M06 Stage F explicitly deferred ("the explicit M07 carry-forward — not a Stage F miss"). M07 is therefore both the headline registry feature and the milestone that closes the largest open architectural carry-forward in the ledger.

### What this milestone produces

1. **`skills.lock` integrity primitive** (Stage B) — the lock file at framework root `{ "name@version": { kind, source, content_hash, installed_at, tier_at_install, validation_report_id } }`; `schemas/skills-lock.v1.json` (schema-as-source-of-truth, ADR-0014); hash validation on artifact load; the `artifact_hash_mismatch` event variant that blocks load with a re-install prompt. Path-agnostic persistence (CLAUDE.md §9). Safety primitive (artifact integrity) → ≥95%.
2. **Import pipeline backend** (Stage C) — fetch-by-URL (capability-gated `reqwest` GET) + file-pick → `schemas/{skill,tool,agent}.v1.json` validation → L3 sandbox (reuse `runtime-sandbox` from M05) → tier-gate (reuse L4 from M05) → install → `skills.lock` write (Stage B). The §15d/§15c metadata (`requires_secrets`, `runtime_dependency_class`, `compatible_os`) read + validated on import; `compatible_os` mismatch with the host OS surfaces a blocking error per spec §15c. `share_provenance` export/import groundwork (ADR-0005; runtime-to-runtime only — no rebake, no Share It module). Same flow for skill / tool / agent / MCP-server-config (the last installs into the M06 MCP Manager).
3. **Agent-with-tools loop + concrete MCP dispatch** (Stages **D1**+**D2**) — discharges ADR-0011 (a)–(d). **D1** (a–c): the concrete `McpDispatcher` constructed in `src-tauri`, `impl ConnectionResolver for McpClient`, the shell `CapabilityEnforcer`/`NamespaceResolver` construction sites. **D2** (d): the multi-turn agent loop replacing the no-tools smoke path, consuming D1's dispatcher. This is the production driver M06 carried forward; D2 is also where signal→`token_usage` projection lands (absorbs the M06.5 `token_usage` open finding).
4. **Import panel renderer** (Stage E) — Builder Import panel: paste-GitHub-raw-URL dialog, local-file picker, tier-gate review screen (capability disclosure + L3 report), `share_provenance` trust-signal block, hash-mismatch reinstall/remove prompt.
5. **Stage V** four-pass verifier (first under v1.8 — the `--features integration` reference-MCP-server smoke is **mandatory** in the Behavior pass, since a real dispatch path exists by D).
6. **Stage G** closeout — M07 summary + gap-analysis entry + `<simplify_pass>` + `<coverage_policy_reconciliation>`.

### What's not in scope

- Anthropic upstream search UI (v1.0 — MVP §M7 "No Anthropic-upstream search UI in v0.1; just URL/file import").
- Pluggable community registries (v2.0); Sigstore signature verification (v1.0).
- The Share It module with rebake / per-OS bundle generation (v1.0, paired with the headless CLI — MVP §M8 forward declaration). v0.1 export is **runtime-to-runtime only**: `share_provenance` is populated and surfaced, nothing is rebaked.
- The 🔴-1 / 🔴-2 real-app IRL re-confirmation (M06.5 carry-forward) — that rides the **post-M07 IRL pass**, not M07 work; tracked only.
- Headless `agent-runtime-cli` (v1.0, post-M11).

### Why six work stages (D split D1/D2) + V + closeout

`skills.lock` (B) is an independent integrity primitive with its own schema + ≥95% gate — distinct from the import pipeline (C) that consumes it, distinct again from the ADR-0011 architectural discharge (D1/D2), and from the renderer (E). Stage A is the canonical carry-forward-absorption stage (the M06 gap-analysis routes ~everything to "M07 Stage A"). Each is a clean red→green boundary; bundling B+C would muddy the schema/primitive gate, bundling C+D would conflate "import an artifact" with "execute an agent that uses it". **The ADR-0011 discharge is split D1/D2** (M05.C1/C2 precedent): D1 = the construction half (`impl ConnectionResolver for McpClient` + concrete `McpDispatcher` ctor in `src-tauri` + CQ-6/EFF-4 — ADR-0011 a–c, closes the A-mapped construction graph); D2 = the agent-with-tools loop consuming it + the `token_usage` projector + CQ-2 (ADR-0011 d / M06.5). One stage would span three crates + two CODEOWNERS surfaces (capability-enforcer construction in D1, runtime-drone projector in D2) + the largest carry-forward in the ledger — too large for one coherent red→green unit; the split gives each a clean construction-graph boundary (D1 makes the inputs reachable; D2 consumes them).

### Carry-forward absorbed (M07 Stage A unless noted)

From `docs/gap-analysis.md` (M06 entry + M06.V) + `M06.5-summary.md`:

- **ADR-0011 (a)–(d) concrete-construction** — Stage **D1** (a–c: `ConnectionResolver` + concrete `McpDispatcher` ctor) + **D2** (d: the agent-with-tools loop discharge); A lays the construction-graph groundwork via `<construction_reachability_check>` (inputs M07 makes reachable; D1 closes it, D2 consumes it).
- **M06.V 🟡 #1** (§5a re-resolution-on-connect no production driver) — Stage A maps it; **Stage D1** discharges it: ADR-0011(b) — the `McpClient`→`NamespaceResolver` connect/disconnect re-resolution + `tool_alias_ambiguous`-on-connect wire; M07.V Wire trace #6 = expected endpoint.
- **M06.V 🟡 #2** (X.2 path drift `mcp_dispatch_integration.rs`) — Stage A pre-flight X.2 truth-up (M05.V-#3 precedent); bundle with **TD-006**.
- **M06.5 `token_usage` open finding** — Stage **D2** (no production `token_usage` writer; sole INSERT is `#[cfg(test)]` in `vdr.rs`; the D2 agent-with-tools loop is the first production token-bearing signal source — the projector lands there).
- **M04 🟡** `plan_loop.rs` driver / `HitlContext::BudgetThreshold`→`BudgetWarn`; **M04 🟡** `runtime-main/src/drone_ipc/client.rs` per-module coverage; **M05 🟡** `capability/enforcer.rs` `TierForbidden` audit-branch coverage — Stage A.
- **post-M05/M06 `docs(spec):` PR (~22 entries)** — gap-analysis: "Open before M07 Stage A." Encoded as a Stage A `<pre_flight_check>` prerequisite (separate PR; M07.A does not start until it merges).
- **TD-002** (`read_signals`/`recover_session` twice-in-sequence), **TD-005** (runtime-main llvm-cov not Windows-local-measurable, gotcha #56), **TD-006** (runtime-main llvm-cov regex `key_store.rs` inconsistency) — Stage A.
- **CQ-6** (`status: String` → `ServerStatus` enum) + **EFF-4** (`run_health_pass` K sequential vs 1 batched) — Stage **D1** (the health-ping/client surface, lands with the resolver wire). **CQ-2/reuse-5** (`apply_mcp_dispatch` Invoked-arm dead in production; split `Blocked|Ambiguous`/`DispatchOutcome` enum) — Stage **D2** (lands with the agent-with-tools loop).
- **runtime-mcp `transport/mod.rs` baseline** (87.50% post-CQ-1) — Stage **D1**: add a `rmcp_tool_to_mcp_tool` unit test OR extend the CLAUDE.md §5 / `docs/coverage-policy.md` §C rationale; reconcile via the v1.8 closeout `<coverage_policy_reconciliation>`.

### Key constraints

- **v1.8 protocol.** Stage prompts use the new audit slots where they bite: Stage A/D1/D2 `<construction_reachability_check>` (the ADR-0011 wire is the canonical "inject X into Y — are ctor inputs reachable" case; A maps it false, D1 closes it true, D2 consumes it — the slot documents the construction graph completing across the A→D1→D2 chain) + D1 `<wire_trace_vs_adr_reconcile>` (the §5a re-resolution trace authored against `McpDispatcher`, NOT `McpClient` — the exact M06.V Dec-6 lesson). Stage E `<wire_signature_audit>` + `<phase_doc_inventory_audit shape=…>` (Import-panel IPC wrappers + store slots pinned to the actual wire).
- **Schema-as-source-of-truth (CLAUDE.md §14, Hard Rule 5).** `schemas/skills-lock.v1.json` + the `artifact_hash_mismatch` event variant are authored as schema; Rust/TS types generated via `cargo xtask regenerate-types`; committed alongside. New schema → ADR (ADR-0014) + schema gate.
- **Reuse, don't rebuild.** L3 sandbox = `runtime-sandbox` (M05); tier-gate = L4 (M05); user prompts = `HitlSeam` (M04.E, ADR-0007 in-process seam — no new IPC variant); MCP-server-config import = the M06 MCP Manager. Network fetch is capability-gated (the import fetch declares `network` capability; enforced through the M05 L1 enforcer).
- **No `unsafe` outside `runtime-sandbox`** (Hard Rule 7). **No telemetry** (Hard Rule 4) — the import fetch hits only the user-supplied URL; no phone-home.
- **CODEOWNERS-flagged paths** (Hard Rule 8): D1 touches capability-enforcer construction + the dispatch seam; D2 touches the runtime-drone `token_usage` projector + the agent loop — each surfaces its construction-graph plan first (the `<construction_reachability_check>` IS that plan).
- **No Co-Authored-By; DCO `-s`; session-URL footer.** Strict v1.8 two-commit `<tdd_discipline>` on every code stage (B/C/D1/D2; A where it ships testable code; E renderer per the v1.7 default).
- **Windows is v0.1 target; CI all three OSes.** gotcha #56 (runtime-main llvm-cov not Windows-local-measurable) is a Stage A TD-005 item — its structural close (ensure_drone_built llvm-cov-robust OR the CI graduation) lands in A.

## Document Structure

| Stage | Scope | Strict TDD | Effort | Coverage gate |
|---|---|---|---|---|
| **A** | Carry-forward absorption: docs(spec) PR pre-flight + X.2 truth-up (TD-006) + M04/M05 🟡 (plan_loop, drone_ipc cov, enforcer TierForbidden) + TD-002/005 + the ADR-0011 construction-graph groundwork (`<construction_reachability_check>`) | yes (where it ships code) | 4–6 h | workspace ≥80; maintain runtime-main ≥95 |
| **B** | `skills.lock` integrity primitive + `schemas/skills-lock.v1.json` (ADR-0014) + `artifact_hash_mismatch` event + hash-on-load | yes (v1.8 two-commit) | 6–8 h | new ≥95 on the `skills_lock` module (safety primitive) |
| **C** | Import pipeline backend: URL/file fetch → schema-validate → L3 → tier-gate → install → lock update; §15c metadata; share_provenance (ADR-0005) | yes (v1.8 two-commit) | 7–9 h | runtime-main ≥95 on the import pipeline; workspace ≥80 |
| **D1** | ADR-0011 a–c construction: `impl ConnectionResolver for McpClient` (§5a re-resolution) + concrete `McpDispatcher` ctor in `src-tauri` (real `NamespaceResolver`/`CapabilityEnforcer`) + CQ-6 (`ServerStatus`) + EFF-4 (batched `run_health_pass`) — closes the A-mapped construction graph | yes (v1.8 two-commit) | 4–5 h | runtime-mcp ≥95; `transport/mod.rs` baseline reconcile |
| **D2** | ADR-0011 d: multi-turn agent-with-tools loop consuming D1's `McpDispatcher` + `token_usage` projector (closes M06.5) + CQ-2 (`DispatchOutcome` enum split) + the assembled-app regression (real loop+drone, `token_usage>0`) | yes (v1.8 two-commit) | 4–5 h | runtime-main ≥95 (loop + projector); drone gate (projector) |
| **E** | Renderer: Builder Import panel (URL dialog + file picker + tier-gate review + share_provenance + hash-mismatch prompt) + Playwright | yes (v1.7 default) | 5–6 h | renderer ≥80 (vitest) |
| **V** | Verifier — four passes; **mandatory `--features integration` reference-MCP-server smoke** (M06.V Dec 7, v1.8) | n/a | 2–4 h | n/a |
| **G** | Closeout — gap-analysis entry + M07 summary + `<simplify_pass>` + `<coverage_policy_reconciliation>` | n/a | 3–4 h | n/a |

Total ~35–47 h estimated (~22–30 h actual at M06's converging ~0.7× calibration; Stages D1+D2 carry a novel-integration bump for the ADR-0011 construction + agent-loop wire — the split adds minor boundary overhead but each is a coherent red→green unit).

## Implementation Workflow

Project-wide protocol (CLAUDE.md §3–§6, §8, §16, §19; not restated per stage):

1. **Read first** — each stage's `<read_first>`; Stage B+ reads the prior stage's retrospective `[END] Decisions` and applies them (CLAUDE.md §19 rule 1).
2. **Strict v1.8 two-commit TDD** on B/C/D (+ A where it ships code; E per the v1.7 renderer default): failing tests → standalone `test(M07.X): …` commit → red-phase surface → impl WITHOUT touching test files → gate ordering → impl commit whose body proves `git diff <red>..<impl> -- '**/tests/**'` EMPTY → final surface. Net-new/mechanical test changes in a separate labelled follow-up.
3. **Schema-as-source-of-truth** — schema edits → `cargo xtask regenerate-types` → commit generated with the schema (CLAUDE.md §14).
4. **v1.6 canonical gate ordering** (CLAUDE.md §6): `cargo fmt --all` → `cargo clippy --fix --allow-dirty -p <crate>` → `cargo clippy --workspace --all-targets -- -D warnings` → test/doc/audit/deny → `cargo llvm-cov clean --workspace` (gotcha #81) → the llvm-cov gates → frontend. CI-parity is a hard rule; cite any divergence inline with a gotcha reference.
5. **Surface, don't commit, until approved** (Hard Rule 1). Every stage surface includes cross-machine state: `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M07.*-retrospective.md` (CLAUDE.md §19 rule 7). Do not push between stages.
6. **Stage V in a fresh CLI session** (the bias guard); V's `<read_first>` deliberately omits prior retros/summary/gap-analysis but DOES consume the phase doc's `<scope_change>` + `<wire_trace_vs_adr_reconcile>` blocks (v1.8 STAGE-V template).
7. **Stage G** runs `<simplify_pass>` (M07.A..HEAD diff) + the v1.8 `<coverage_policy_reconciliation>` (sync `docs/coverage-policy.md` §B/§C + CLAUDE.md §5/§6 + `codecov.yml` for any gate change — esp. the runtime-mcp `transport/mod.rs` baseline + the new `skills_lock` ≥95 gate).
8. **ADR-0014** (skills.lock integrity model + new schema) filed in Stage B, `Proposed → Accepted` in the M07 PR before merge (CLAUDE.md §11).

---

## Pre-existing legacy file inventory

Grep-verified at authoring time against `origin/main` at `e72c762` (post-M06.6 / v1.8 merge). Files M07 stages CONSUME or REFERENCE (not create); shape claims are factual as of the authoring snapshot.

| File | Purpose | M07 stage that touches it |
|---|---|---|
| `crates/runtime-main/src/sdk/agent_sdk.rs` | SDK loop; the no-tools smoke path is the only `AgentSdk` construction (ADR-0011 Context #4); `emit`→`persist_signal` choke point (M06.5.B) | D2 (multi-turn agent-with-tools loop replaces the no-tools path; `token_usage` projector) |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | `ProviderEvent::ToolUse` → `AgentEvent` (M02); M06.F MCP-dispatch interception seam | D (concrete dispatch interception in the real loop) |
| `crates/runtime-main/src/capability/enforcer.rs` | L1 enforcer; 94.24% line with the `TierForbidden` audit branch uncovered (M05.E) | A (TierForbidden audit-branch test) + D1 (constructed in `src-tauri` for the concrete `McpDispatcher`) |
| `crates/runtime-main/src/capability/narrowing.rs` | L2a `narrow()` (M05.B) | D (NamespaceResolver / enforcer construction path) |
| `crates/runtime-main/src/drone_ipc/client.rs` | drone IPC client; M04 🟡 per-module coverage + TD-002 (`read_signals`/`recover_session` twice-in-sequence) + TD-005 (gotcha #56 not Windows-local-measurable) | A |
| `crates/runtime-main/src/plan/…` (`plan_loop.rs`) | M04 🟡 driver unwired; `HitlContext::BudgetThreshold` (schema-driven — check before rename) | A (wire driver; `BudgetWarn` rename) |
| `crates/runtime-main/tests/mcp_dispatch_integration.rs` | M06.V 🟡#2 X.2 path drift + TD-006 (runtime-main llvm-cov regex `key_store.rs` inconsistency) | A (X.2 truth-up; M05.V-#3 precedent) |
| `crates/runtime-main/tests/smoke_signal_persistence.rs` | the M06.5 real-drone-subprocess assembled-regression harness archetype | D (mirror for the agent-with-tools assembled regression; the §6/v1.8 mandate) |
| `crates/runtime-mcp/src/client/…` | M06 MCP client + registry; no `impl ConnectionResolver for McpClient` (ADR-0011 Context #1); `status: String` (CQ-6); `run_health_pass` K-sequential (EFF-4) | C (registry = MCP-config import target) + D (`impl ConnectionResolver`, `ServerStatus` enum, batched health pass) |
| `crates/runtime-mcp/src/transport/mod.rs` | 87.50% line post-M06.G CQ-1 (the `rmcp_tool_to_mcp_tool` baseline carry-forward) | D (add the unit test OR extend the §5/coverage-policy rationale; G reconciles) |
| `crates/runtime-sandbox/` | L3 validation sandbox (M05.C) | C (reused for import L3 — not rebuilt) |
| `crates/runtime-main/src/tier/` | L4 tier system (M05.D) | C (reused for the import tier-gate — not rebuilt) |
| `crates/runtime-main/src/hitl/seam.rs` | `HitlSeam` (M04.E, ADR-0007 in-process seam) | C (reused for any import prompt; no new IPC/seam variant) |
| `src-tauri/src/commands.rs` | `run_smoke_session` (no-tools); shell command surface | C (`import_artifact` command added) + D1 (concrete `McpDispatcher` ctor) + D2 (`run_session_with` loop consumes it) + E (renderer pins the signature) |
| `src-tauri/src/drone_lifecycle.rs` | drone construction + the M06.5.B `sdk_session_id` sharing | D1 (concrete `McpDispatcher` / enforcer / resolver ctor sites) |
| `crates/runtime-core/src/generated/…` + `src/types/…` | typify / json-schema-to-typescript generated types | B (regenerated after `schemas/skills-lock.v1.json` + `artifact_hash_mismatch`; `cargo xtask regenerate-types --check` clean) |
| `schemas/{skill,tool,agent,framework}.v1.json` | artifact schemas | C (import validation targets); B (new `skills-lock.v1.json` sibling per the `$id` convention) |
| `docs/adr/0010-mcp-dispatch-dependency-inversion.md` + `0011-m06-stage-f-scope.md` | the dependency-inversion seam + the (a)–(d) concrete-construction carry-forward | A (map via `<construction_reachability_check>`) + D (discharge) |
| `docs/adr/0005-*.md` | sharing metadata (`share_provenance`, §15d) | C (export populates / import surfaces; runtime-to-runtime only) |
| `docs/MVP-v0.1.md` §M7 + `agent-runtime-spec.md` §2152–2211 / §5a / §15c / §2c.3 | acceptance criteria + the spec sections M07 implements | All stages (V later checks against them) |
| `docs/coverage-policy.md` (+ CLAUDE.md §5/§6, `codecov.yml`) | the four-mirror coverage source-of-truth | A (TD-006 reconcile) + B/C/D (gate changes) + G (`<coverage_policy_reconciliation>`) |
| `STAGE-PROMPT-PROTOCOL.md` v1.8 + `docs/build-prompts/M06.6-protocol-v1-8.md` | the protocol M07 is the first parent milestone authored under | All stages — prompts adopt the v1.8 audit slots inline |

---

## Stage A — Carry-forward absorption + ADR-0011 construction-graph groundwork

### A.1 Problem Statement

The M06 gap-analysis routes a backlog to "M07 Stage A": the X.2 truth-up (M06.V 🟡#2 + TD-006), the M04/M05 🟡 coverage/driver items, TD-002/005, and the construction-graph groundwork for the ADR-0011 discharge that Stages D1/D2 perform. Per gap-analysis, the **post-M05/M06 `docs(spec):` PR (~22 entries) must be open before M07 Stage A** — encoded as a `<pre_flight_check>` (M07.A does not start until that PR merges; it is a separate PR, not M07 work). Stage A clears the debt and authors the `<construction_reachability_check>` that proves which ADR-0011 constructor inputs are (not yet) reachable today and which Stage D1 makes reachable (D2 consumes) — the v1.8 mechanism whose absence caused the M06.F over-framing.

### A.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/plan/…` | exists | M04 🟡: wire the `plan_loop.rs` driver; rename `HitlContext::BudgetThreshold`→`BudgetWarn` (schema-driven if it is a generated variant — check `schemas/` first). |
| `crates/runtime-main/src/drone_ipc/client.rs` | exists | M04 🟡 per-module coverage close; TD-002 (`read_signals`/`recover_session` twice-in-sequence); TD-005 structural close (gotcha #56 — make `ensure_drone_built` llvm-cov-robust or complete the CI graduation). |
| `crates/runtime-main/src/capability/enforcer.rs` | exists | M05 🟡: the `TierForbidden`-then-audit branch test (lifts 94.24% within the runtime-main gate). |
| `crates/runtime-main/tests/mcp_dispatch_integration.rs` | exists | M06.V 🟡#2 X.2 truth-up: correct the path drift; bundle TD-006 (runtime-main llvm-cov regex `key_store.rs` reconcile — sync CLAUDE.md §5/§6 + `docs/coverage-policy.md`). |
| `docs/coverage-policy.md` / `CLAUDE.md` §5/§6 / `codecov.yml` | exist | TD-006 reconcile only (no gate value change unless A surfaces one — then the v1.8 four-mirror sync rule applies). |
| `CHANGELOG.md` | exists | `[Unreleased]` Stage A entry. |
| `docs/build-prompts/retrospectives/M07.A-retrospective.md` | new | Stage A retrospective. |

### A.3 Detailed Changes

#### A.3.1 M04 🟡 — `plan_loop.rs` driver wire + `BudgetThreshold`→`BudgetWarn`

Before A, `crates/runtime-main/src/plan/` ships the plan state machine but no driver loop consumes it in the production session path (M04 🟡 "plan_loop.rs driver"). After A, the smoke/session entrypoint drives the plan loop:

```rust
// crates/runtime-main/src/plan/plan_loop.rs — illustrative shape:
pub async fn drive_plan(
    plan: &mut PlanState,
    hitl: &HitlSeam,
    emit: &impl Fn(AgentEvent),
) -> Result<(), PlanError> {
    while let Some(task) = plan.next_ready_task() {
        if plan.requires_approval(&task) {
            match hitl.await_decision(plan.approval_prompt(&task)).await {
                HitlDecision::Approve => plan.mark_running(&task),
                HitlDecision::Reject => { plan.mark_skipped(&task); continue; }
            }
        }
        emit(AgentEvent::TaskStarted { task_id: task.id.clone(), /* … */ });
        // task execution is M07.D2's agent-with-tools loop; A only wires the driver shell
    }
    Ok(())
}
```

`HitlContext::BudgetThreshold` → `BudgetWarn`: check `schemas/` first — if it is a generated variant (it is, per `schemas/event.v1.json` `hitl_context`), rename in the schema + `cargo xtask regenerate-types`, do **not** hand-edit `runtime-core` (Hard Rule 5). Minor in-`v1` bump; document in A.3.5.

#### A.3.2 M04 🟡 + TD-002 + TD-005 — `drone_ipc/client.rs`

Three coupled items on one file:
- **M04 🟡 per-module coverage close** — `client.rs` is below its per-module line baseline; add the missing error-path tests (reconnect-exhausted, codec-error on `SignalLog`).
- **TD-002** — `read_signals` / `recover_session` are not exercised twice-in-sequence; add a test that calls each twice on the same client to pin the no-single-use-state invariant.
- **TD-005** — `cargo llvm-cov` for runtime-main is not Windows-local-measurable (gotcha #56: `ensure_drone_built()` doesn't place the drone bin in the llvm-cov target). Structural close: make `ensure_drone_built()` llvm-cov-target-aware **or** complete the gotcha #56 CI graduation. Not a permanent local `--test-threads` flag (CI-parity hard rule).

```rust
// crates/runtime-main/tests/ — TD-002 shape:
#[tokio::test]
async fn read_signals_twice_in_sequence_is_stable() {
    let (client, _drone) = spawn_test_drone().await;
    let a = client.read_signals(sid.clone()).await.expect("first");
    let b = client.read_signals(sid.clone()).await.expect("second");  // must not error / not be empty-due-to-consumed-state
    assert_eq!(a, b);
}
```

#### A.3.3 M05 🟡 — `capability/enforcer.rs` `TierForbidden` audit branch

`enforcer.rs` is 94.24% line within the runtime-main ≥95 gate; the uncovered lines are the `TierForbidden`-then-`audit_check_result` branch (M05.E left it; M06.A's L1 wire-up covered `enforcer.check` but not this specific arm). Add an integration test that drives a Promoted-tier-forbidden capability through `check` and asserts (a) `CapabilityError::TierForbidden` and (b) the audit writer received the `tier_forbidden` entry:

```rust
#[tokio::test]
async fn tier_forbidden_emits_audit_entry() {
    let (enforcer, audit_spy) = enforcer_with_audit_spy(Tier::Promoted);
    let err = enforcer.check("agent", &[forbidden_for_promoted()]).unwrap_err();
    assert!(matches!(err, CapabilityError::TierForbidden { .. }));
    assert_eq!(audit_spy.entries().await, vec![AuditKind::TierForbidden]);
}
```

#### A.3.4 M06.V 🟡#2 — X.2 truth-up on `mcp_dispatch_integration.rs` (M05.V-#3 precedent)

Focused path/line correction only (no behavior change), exactly the M05.V-#3 pattern: the M06 D.2 line-1887 reference + the V.3 Behavior-harness crate scope drifted from where `mcp_dispatch_integration.rs` actually lives. Correct the cited paths/lines in place; every other line stays unchanged. The `docs/gap-analysis.md` M06 entry already records the discrepancy at closeout per §20 append-only; this moves the canonical X.2 record to match what shipped, closing the 🟡#2 carry-forward. Bundle with A.3.5 (TD-006).

#### A.3.5 TD-006 — runtime-main llvm-cov regex `key_store.rs` four-mirror reconcile

TD-006: the runtime-main `--ignore-filename-regex` conceptually excludes `key_store.rs` (OS-keychain holdout) but the regex string is inconsistent across the mirrors. Per the v1.8 CLAUDE.md §6 four-mirror rule, reconcile in **one** commit: `docs/coverage-policy.md` §A/§C (append the M07.A reconcile note), CLAUDE.md §5 category list + §6 `cargo llvm-cov --package runtime-main` command, and `codecov.yml`. No gate-value change — string consistency only; if A surfaces a genuine value change, the same four-mirror sync applies and G's `<coverage_policy_reconciliation>` verifies it.

#### A.3.6 The `<construction_reachability_check>` map (load-bearing)

The v1.8 mechanism whose absence caused the M06.F over-framing. For each ADR-0011 wire, state the constructor, whether its inputs are reachable on `main` today, and the resolution (Stage D1 for the a–c construction; Stage D2 consumes it for the d loop). On `main` they are **not** reachable (ADR-0011 Context #1–#4): no `impl ConnectionResolver for McpClient`; `McpDispatcher::new` requires `Arc<RwLock<NamespaceResolver>>` + `Arc<CapabilityEnforcer>` with no `src-tauri` ctor site; the only `AgentSdk` construction is the no-tools smoke path. A authors this as the `<construction_reachability_check>` block in the A.5 prompt; D1 inverts it (inputs become reachable, file:line) and D2 consumes them. This converts M06.F's implicit over-reach into an explicit, Stage-V-readable construction graph — A ships **no** McpDispatcher code (that is D1); A only documents the graph.

### A.4 Tests

Each absorbed item gets the test its 🟡/TD demanded: `plan_loop` driver behavior test; `drone_ipc/client.rs` per-method twice-in-sequence test (TD-002) lifting the per-module gate; `enforcer.rs` `TierForbidden`-then-audit test; the corrected `mcp_dispatch_integration.rs` path compiles + runs. Acceptance: every cited 🟡/TD has a named green test or a documented structural close; full v1.6 canonical gate suite green; runtime-main ≥95 maintained; the docs(spec) PR pre-flight confirmed merged; the `<construction_reachability_check>` enumerates the ADR-0011 wires with `inputs_reachable="false — Stage D1 makes reachable; D2 consumes"`.

### A.5 CLI Prompt

```xml
<work_stage_prompt id="M07.A">
  <context>
    M07 Stage A — clear the M06 gap-analysis carry-forward backlog
    routed to "M07 Stage A" and author the ADR-0011 construction-graph
    groundwork. NOT the registry feature (B–E) and NOT the ADR-0011
    discharge (D) — A absorbs debt + documents the construction graph
    so D's wire is authored against a V-readable reachability map. The
    post-M05/M06 docs(spec): PR (~22 entries) is a pre-flight
    prerequisite — M07.A does not start until it merges.
  </context>
  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — the three new audit slots; §975 ordering)</file>
    <file>docs/build-prompts/M07-registry-import.md (Background, Key constraints, Implementation Workflow, Stage A A.1–A.4)</file>
    <file>docs/gap-analysis.md (M06 entry + M06.V carry-forward: the exact 🟡/TD items + line cites)</file>
    <file>docs/adr/0010-mcp-dispatch-dependency-inversion.md + docs/adr/0011-m06-stage-f-scope.md (the (a)-(d) concrete-construction carry-forward this stage maps, D discharges)</file>
    <file>docs/build-prompts/retrospectives/M06.V-retrospective.md (🟡#1 §5a re-resolution driver; 🟡#2 X.2 drift)</file>
    <file>docs/tech-debt.md (TD-002/005/006 exact text)</file>
    <file>docs/coverage-policy.md (before any TD-006 gate-text touch — the four-mirror sync rule)</file>
    <file>docs/gotchas.md (#56 runtime-main llvm-cov Windows-local; #66; #81)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>
  <read_prior_milestones>
    <gap_analysis_carry_forward milestone="M06"/>
  </read_prior_milestones>
  <deliverable ref="docs/build-prompts/M07-registry-import.md" section="A.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <tdd_discipline strict="true">
    <red_phase>One standalone `test(M07.A): …` commit with the failing tests for every code-bearing carry-forward (plan_loop driver, drone_ipc TD-002, enforcer TierForbidden). Pure path/doc reconciles (X.2/TD-006) are not test-bearing — note them as such. Right-reason failure per CLAUDE.md §5; surface for red approval.</red_phase>
    <green_phase>Impl without touching test files; impl commit body proves `git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**'` EMPTY. TD-006/X.2 path corrections + any four-mirror coverage sync in the impl or a labelled follow-up. No Co-Authored-By.</green_phase>
  </tdd_discipline>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M07-registry-import.md" section="A.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M07-registry-import.md" section="Key constraints"/>
  <gates milestone="M07"/>
  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs"/>
  </coverage_gate>
  <self_correction_budget>3</self_correction_budget>
  <construction_reachability_check>
    <wire claim="concrete McpDispatcher ctor in src-tauri" constructor="McpDispatcher::new (no src-tauri ctor on main)" inputs_reachable="false — Arc&lt;RwLock&lt;NamespaceResolver&gt;&gt; + Arc&lt;CapabilityEnforcer&gt; have no shell construction site (ADR-0011 Context #2/#3)" resolution="Stage D1 constructs them"/>
    <wire claim="impl ConnectionResolver for McpClient" constructor="(absent on main — ADR-0011 Context #1)" inputs_reachable="false" resolution="Stage D1 (the §5a re-resolution driver — M06.V 🟡#1)"/>
    <wire claim="agent-with-tools loop replacing the no-tools smoke path" constructor="AgentSdk::new (src-tauri/src/commands.rs — currently no-tools smoke only)" inputs_reachable="false — M07 scope per ADR-0009/0011" resolution="Stage D2 (consumes D1's concrete McpDispatcher)"/>
  </construction_reachability_check>
  <adr_triggers>
    <trigger>None new in A. ADR-0010/0011 already accepted; A maps their carry-forward, D discharges it. ADR-0014 (skills.lock) is filed in Stage B.</trigger>
  </adr_triggers>
  <gotchas>
    <trap>#56 — runtime-main llvm-cov not Windows-local-measurable; TD-005 structural close is in A (ensure_drone_built robust OR CI graduation), not a permanent local flag.</trap>
    <trap>#81 — `cargo llvm-cov clean --workspace` before the coverage gates.</trap>
    <trap>TD-006 four-mirror sync: any runtime-main regex touch updates docs/coverage-policy.md §C + CLAUDE.md §5/§6 + codecov.yml in THIS commit (v1.8 §6 rule).</trap>
    <trap>Schema-driven types: if `HitlContext::BudgetThreshold` is a generated variant, rename in `schemas/` + regenerate, do NOT hand-edit runtime-core (Hard Rule 5).</trap>
  </gotchas>
  <pre_flight_check>
    <check name="branch">git rev-parse --abbrev-ref HEAD == claude/m07-registry-import (cut from main after v1.8 + the docs(spec) PR merged)</check>
    <check name="docs_spec_pr_merged">the post-M05/M06 docs(spec): PR (~22 entries) MUST be merged to main before A starts (gap-analysis "Open before M07 Stage A"); confirm via git log on main</check>
    <check name="v1_8_protocol">grep -n 'construction_reachability_check' STAGE-PROMPT-PROTOCOL.md must hit (v1.8 landed via M06.6)</check>
  </pre_flight_check>
  <phase_doc_inventory_audit>
    <claim type="file" path="crates/runtime-main/src/drone_ipc/client.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/capability/enforcer.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/tests/mcp_dispatch_integration.rs" verified="true"/>
  </phase_doc_inventory_audit>
  <runtime_environment os="windows"/>
  <time_box hours="4-6"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>List each absorbed 🟡/TD with its green test or documented structural close. Confirm the docs(spec) PR pre-flight. State the `<construction_reachability_check>` map + that Stage D1 owns each `inputs_reachable="false"` construction resolution (D2 consumes). Note any four-mirror coverage sync done.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M07-registry-import.md" section="A.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + M07.* retro listing)</item>
    <item>strict-TDD invariant: git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**' EMPTY</item>
    <item>per-item carry-forward closure table (🟡/TD → test/structural-close)</item>
    <item>the `<construction_reachability_check>` map (ADR-0011 wires; Stage D1 construction resolutions, D2 consumes)</item>
    <item>gate results (v1.6 order; runtime-main ≥95; CI-parity, cite any divergence); any four-mirror coverage sync</item>
    <item>M07.A retrospective [END]</item>
    <item>explicit: "Stage M07.A is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### A.6 Commit Message

```
feat(runtime): M07 Stage A — clear M06 carry-forward + ADR-0011 construction-graph groundwork

Absorbs the M06 gap-analysis "M07 Stage A" backlog: M04 plan_loop
driver + BudgetWarn rename, M04 drone_ipc/client.rs per-module
coverage + TD-002, M05 enforcer.rs TierForbidden audit branch,
M06.V 🟡#2 X.2 truth-up + TD-006 reconcile, TD-005 (gotcha #56
structural close). Authors the v1.8 <construction_reachability_check>
mapping the ADR-0011 (a)-(d) wires whose constructor inputs are not
reachable on main today — Stage D1 makes them reachable, D2 consumes.

docs(spec): PR pre-flight confirmed merged. No new ADR (ADR-0010/0011
accepted; A maps, D discharges). Strict v1.8 two-commit TDD:
git diff <red>..<impl> -- '**/tests/**' EMPTY. runtime-main ≥95
maintained; any four-mirror coverage sync per CLAUDE.md §6.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage B — `skills.lock` integrity primitive + ADR-0014

### B.1 Problem Statement

Spec §2181–2204: every installed artifact records to a `skills.lock` at framework root; the lock is committed to the user's framework repo for reproducible cross-machine installs; hash validation runs on **every artifact load** and a mismatch blocks the load with a reinstall prompt. No such primitive exists on `main`. It is a new artifact shape → new schema (`schemas/skills-lock.v1.json`) + ADR-0014; an integrity safety primitive → ≥95%. It must be path-agnostic (CLAUDE.md §9): the module takes `path: &Path`; the Tauri shell resolves the framework-root lock path.

### B.2 Files to Change

| File | Status | Change |
|---|---|---|
| `schemas/skills-lock.v1.json` | new | The lock schema: map of `name@version` → `{ kind, source, content_hash, installed_at, tier_at_install, validation_report_id }`. |
| `schemas/common.v1.json` (or event schema) | exists | Add the `artifact_hash_mismatch` event variant (schema-as-source-of-truth). |
| generated type files | regenerated | `cargo xtask regenerate-types` output committed alongside the schema. |
| `crates/runtime-main/src/skills_lock/…` | new | Path-agnostic lock read/write/verify + the SHA-256 content-hash + `verify_on_load` returning the `artifact_hash_mismatch` signal on drift. |
| `docs/adr/0014-skills-lock-integrity.md` | new | The lock format + hash-blocks-load decision; `Proposed`→`Accepted` in the M07 PR. |
| `CHANGELOG.md` / retro | exist/new | Stage B entries. |

### B.3 Detailed Changes

#### B.3.1 Schema — `schemas/skills-lock.v1.json`

Author the schema first (Hard Rule 5); match the existing `$id` base-URL convention (grep `"$id"` across `schemas/` per the v1.8 authoring discipline). Field set per spec §2181:

```jsonc
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://agent-runtime.local/schemas/skills-lock.v1.json",
  "title": "SkillsLock",
  "type": "object",
  "required": ["version", "entries"],
  "properties": {
    "version": { "const": 1 },
    "entries": {
      "type": "object",
      "description": "map of \"name@version\" → lock entry",
      "additionalProperties": { "$ref": "#/$defs/LockEntry" }
    }
  },
  "$defs": {
    "LockEntry": {
      "type": "object",
      "required": ["kind", "source", "content_hash", "installed_at", "tier_at_install", "validation_report_id"],
      "properties": {
        "kind": { "type": "string", "enum": ["skill", "tool", "agent", "mcp_server"] },
        "source": { "$ref": "#/$defs/Source" },
        "content_hash": { "$ref": "#/$defs/SriHash" },
        "installed_at": { "type": "string", "format": "date-time" },
        "tier_at_install": { "type": "string", "enum": ["novice", "promoted"] },
        "validation_report_id": { "type": "string", "minLength": 1 }
      }
    },
    "Source": {
      "oneOf": [
        { "type": "object", "required": ["type", "url"], "properties": { "type": { "const": "url" }, "url": { "type": "string", "format": "uri" } } },
        { "type": "object", "required": ["type", "path"], "properties": { "type": { "const": "file" }, "path": { "type": "string" } } }
      ]
    },
    "SriHash": { "type": "string", "pattern": "^sha256-[A-Za-z0-9+/]+={0,2}$", "description": "SRI-style algorithm-prefixed base64 digest" }
  }
}
```

`Source` is a struct-shaped discriminated union (gotcha #26 — `serde(tag = "type")` needs struct variants). `SriHash` pattern enforces the algorithm prefix at the schema level so a bare hex digest fails validation.

#### B.3.2 Event variant — `artifact_hash_mismatch`

Add to `schemas/event.v1.json` (minor in-`v1` bump; regenerate Rust + TS types):

```jsonc
{
  "type": "object",
  "title": "ArtifactHashMismatch",
  "required": ["type", "artifact_ref", "expected", "actual"],
  "properties": {
    "type": { "const": "artifact_hash_mismatch" },
    "artifact_ref": { "type": "string", "minLength": 1, "description": "name@version of the drifted artifact" },
    "expected": { "$ref": "skills-lock.v1.json#/$defs/SriHash" },
    "actual": { "$ref": "skills-lock.v1.json#/$defs/SriHash" }
  }
}
```

Cross-schema `$ref` to `skills-lock.v1.json#/$defs/SriHash` — the v1.6 `<schema_ref_audit>` slot verifies it resolves at authoring time.

#### B.3.3 Module surface — `crates/runtime-main/src/skills_lock/mod.rs`

Path-agnostic (CLAUDE.md §9 — the module takes `&Path`; the Tauri shell resolves the framework-root lock path):

```rust
//! skills.lock — per-framework artifact integrity ledger (spec §2181-2204).
//! Path-agnostic: callers pass the resolved lock path; the Tauri shell
//! resolves `<framework_root>/skills.lock`. Hash is SRI-encoded so the
//! algorithm is self-describing and swappable (npm/SRI convention).

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("hash mismatch for {artifact_ref}: expected {expected}, got {actual}")]
    HashMismatch { artifact_ref: String, expected: String, actual: String },
    #[error("lock io: {0}")] Io(String),
    #[error("lock parse: {0}")] Parse(String),
}

/// SRI-encode a SHA-256 over the canonical artifact bytes: "sha256-<base64>".
pub fn content_hash(artifact_bytes: &[u8]) -> String { /* sha256 → base64 → prefix */ }

pub fn read(path: &Path) -> Result<SkillsLock, LockError>;
pub fn write_entry(path: &Path, key: &str, entry: LockEntry) -> Result<(), LockError>;

/// Verify on load. Parses the stored hash's algorithm prefix; recomputes
/// over `artifact_bytes`; on drift returns HashMismatch so the load path
/// can emit `artifact_hash_mismatch` and block + prompt reinstall.
pub fn verify(path: &Path, artifact_ref: &str, artifact_bytes: &[u8]) -> Result<(), LockError>;
```

#### B.3.4 Canonical serialization (the reproducibility invariant)

`SkillsLock` serializes with **sorted keys** and a stable field order so two installs of the same set produce a byte-identical file (spec §2204 "reproducible installs across machines"; checked into the user's framework repo). Mergeability is prioritized over compactness — one entry per line, alphabetical by `name@version` — the established lockfile best practice so git auto-resolves concurrent adds rather than conflicting. The test asserts **byte-identical** round-trip, not struct-equality.

#### B.3.5 ADR-0014

Records: (1) hash-blocks-load (integrity > availability for installed artifacts — a tampered artifact must not silently run); (2) the **SRI-encoded SHA-256** choice + upgrade-safe rationale (SHA-256 is the uv/Go/Cargo ecosystem-common digest; npm uses SHA-512 via SRI; the `sha256-` prefix keeps the algorithm swappable without a schema break — bare hex would not); (3) the canonical-serialization reproducibility invariant; (4) the threat model + staged posture (TOFU vs a mutable ref → sandbox-before-trust + tier-gate + hash-lock-on-first-install now; Sigstore/SLSA/TUF provenance is the v1.0 layer attached at the `share_provenance` seam — explicitly deferred, not missed). `Proposed → Accepted` in the M07 PR.

### B.4 Tests

Schema validation (fixture lock files validate/round-trip against `schemas/skills-lock.v1.json`); SHA-256 determinism + cross-platform stability; `verify` happy path + tamper → `artifact_hash_mismatch` emitted; canonical-serialization byte-identical round-trip (the reproducibility invariant); path-agnostic (tempfile-backed). Acceptance: `skills_lock` module ≥95; schema gate green; ADR-0014 present; v1.8 two-commit invariant proven; types generated not hand-written.

### B.5 CLI Prompt

```xml
<work_stage_prompt id="M07.B">
  <context>
    M07 Stage B — the skills.lock integrity primitive (spec §2181–2204):
    schema-defined lock file, SHA-256 hash validation on artifact load,
    artifact_hash_mismatch event blocking load with reinstall prompt,
    reproducible across machines. New schema + ADR-0014. Safety
    primitive ≥95, path-agnostic (CLAUDE.md §9). NOT the import
    pipeline (C consumes this) and NOT Sigstore (v1.0).
  </context>
  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8)</file>
    <file>docs/build-prompts/M07-registry-import.md (Background, Key constraints, Stage B B.1–B.4)</file>
    <file>agent-runtime-spec.md §2181–2211 (skills.lock format + reproducibility + L1–L5 install)</file>
    <file>schemas/README.md (versioning policy) + schemas/common.v1.json + an existing schemas/*.v1.json for the $id/shape convention</file>
    <file>docs/adr/0000-template.md</file>
    <file>crates/runtime-main/src/audit/file_path.rs (the path-agnostic + Tauri-resolves-dir archetype, CLAUDE.md §9)</file>
    <file>docs/build-prompts/retrospectives/M07.A-retrospective.md</file>
    <file>docs/gotchas.md (#80 idempotent migrations not relevant; #81 llvm-cov clean) + RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>
  <read_prior_stages><stage id="M07.A" retro="docs/build-prompts/retrospectives/M07.A-retrospective.md"/></read_prior_stages>
  <deliverable ref="docs/build-prompts/M07-registry-import.md" section="B.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <tdd_discipline strict="true">
    <red_phase>Standalone `test(M07.B): …` — schema round-trip, SHA-256 determinism, verify happy+tamper, canonical-serialization byte-identical, path-agnostic tempfile. Right-reason red (unresolved import / unimplemented). Surface for red approval.</red_phase>
    <green_phase>Schema → regenerate-types → module impl; no test-file edits in the impl commit; body proves `git diff <red>..<impl> -- '**/tests/**'` EMPTY. ADR-0014 Proposed→Accepted in the PR. No Co-Authored-By.</green_phase>
  </tdd_discipline>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M07-registry-import.md" section="B.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M07-registry-import.md" section="Key constraints"/>
  <gates milestone="M07"/>
  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="module" name="skills_lock" target_lines="95" ignore_filename_regex="src.main\.rs|generated"/>
  </coverage_gate>
  <self_correction_budget>3</self_correction_budget>
  <adr_triggers>
    <trigger>ADR-0014 (skills.lock integrity model + new schema schemas/skills-lock.v1.json + artifact_hash_mismatch event) — new artifact shape + schema (CLAUDE.md §11/§14). File in B; Proposed→Accepted in the M07 PR. New ≥95 gate on `skills_lock` → the v1.8 four-mirror sync (docs/coverage-policy.md §B/§C + CLAUDE.md §5/§6 + codecov.yml) lands at closeout G per `<coverage_policy_reconciliation>`.</trigger>
  </adr_triggers>
  <schema_drift_check>cargo xtask regenerate-types --check must be clean after the schema add (committed types == regenerated)</schema_drift_check>
  <gotchas>
    <trap>Hard Rule 5 — skills-lock types + artifact_hash_mismatch are GENERATED from schema; never hand-write in runtime-core/src/types.</trap>
    <trap>CLAUDE.md §9 — the module takes path: &Path; the Tauri shell resolves the framework-root path. tempfile-backed unit tests.</trap>
    <trap>Reproducibility (§2204) — canonical/sorted serialization; assert byte-identical round-trip, not just struct-equality.</trap>
    <trap>#81 — `cargo llvm-cov clean` before the ≥95 measurement.</trap>
  </gotchas>
  <pre_flight_check>
    <check name="branch">HEAD == claude/m07-registry-import; M07.A impl commit present</check>
    <check name="schema_convention">grep '"$id"' an existing schemas/*.v1.json to match the base-URL pattern before writing skills-lock.v1.json</check>
  </pre_flight_check>
  <runtime_environment os="windows"/>
  <time_box hours="6-8"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the canonical-serialization choice + the cross-machine reproducibility test. Confirm types generated (regenerate-types --check clean). ADR-0014 status. Note the new `skills_lock` ≥95 gate for the G `<coverage_policy_reconciliation>`.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M07-registry-import.md" section="B.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state</item>
    <item>strict-TDD invariant EMPTY proof</item>
    <item>schema + generated-types diff (regenerate-types --check clean)</item>
    <item>ADR-0014 (Proposed) summary</item>
    <item>gate results incl. skills_lock ≥95; the new gate flagged for G four-mirror sync</item>
    <item>M07.B retrospective [END]; explicit "Stage M07.B is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### B.6 Commit Message

```
feat(runtime): M07 Stage B — skills.lock integrity primitive (ADR-0014)

schemas/skills-lock.v1.json + artifact_hash_mismatch event
(schema-as-source-of-truth; types regenerated). Path-agnostic
skills_lock module: SHA-256 content hash, verify-on-load,
canonical serialization for byte-identical cross-machine
reproducibility (spec §2204), artifact_hash_mismatch on drift to
block load + prompt reinstall. ADR-0014 (Accepted in the M07 PR).
Safety primitive ≥95. Strict v1.8 two-commit TDD:
git diff <red>..<impl> -- '**/tests/**' EMPTY. New gate recorded
for the G coverage-policy reconciliation.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage C — Import pipeline backend

### C.1 Problem Statement

MVP §M7 lines 213–224: paste a GitHub raw URL (or pick a local file) → fetch → schema-validate → L3 sandbox → tier-gate review → install → `skills.lock` updated; same flow for skill/tool/agent/MCP-server-config; `requires_secrets`/`runtime_dependency_class`/`compatible_os` read + validated (compatible_os mismatch → blocking error, spec §15c); framework export populates `share_provenance`, import surfaces it (ADR-0005, runtime-to-runtime only). The backend pipeline does not exist; it composes M05 (L3 sandbox, L4 tier-gate) + Stage B (lock) + a capability-gated fetch.

### C.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/import/…` | new | The pipeline: `fetch(source: ImportSource)` (URL via capability-gated reqwest GET / file read) → `validate(kind, bytes)` against `schemas/{skill,tool,agent}.v1.json` → `l3_sandbox` (reuse `runtime-sandbox`) → `tier_gate` (reuse L4) → `install` → Stage B `skills_lock` write. §15c `compatible_os` check → blocking `ImportError::OsMismatch`. |
| `schemas/*` | exists | If `share_provenance` / §15d metadata fields are not already in the framework/artifact schemas, add per ADR-0005 (schema + regenerate). |
| Tauri command (`src-tauri/src/commands.rs`) | exists | `import_artifact` command (thin shell wrapper over the pipeline; the `*_with` seam is the unit-tested core, the wrapper is the §5 tauri-shell holdout). |
| `CHANGELOG.md` / retro | exist/new | Stage C entries. |

### C.3 Detailed Changes

#### C.3.1 Pipeline surface — `crates/runtime-main/src/import/mod.rs`

Each stage is a pure/seam-testable function taking injected fakes; the real reqwest GET + real FS + the Tauri command wrapper are the §5 OS-call holdouts (covered by the seam unit tests + the `--features integration` smoke):

```rust
pub enum ImportSource { Url(String), File(PathBuf) }
pub enum ArtifactKind { Skill, Tool, Agent, McpServer }

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("fetch failed: {0}")] Fetch(String),
    #[error("schema invalid: {0}")] SchemaInvalid(String),
    #[error("L3 sandbox failed: {0}")] L3(String),
    #[error("tier-gated: review required")] TierReviewRequired(TierReview),
    #[error("compatible_os mismatch: artifact {artifact:?} vs host {host}")]
    OsMismatch { artifact: Vec<String>, host: String },   // §15c — BLOCKING
}

/// Seam: caller injects the fetcher (real reqwest in prod, fake in tests).
pub async fn fetch_with(src: &ImportSource, get: &impl Fetcher) -> Result<Vec<u8>, ImportError>;
/// Validate raw bytes against schemas/{skill,tool,agent}.v1.json by kind.
pub fn validate(kind: ArtifactKind, bytes: &[u8]) -> Result<ValidatedArtifact, ImportError>;
/// Reuse M05 runtime-sandbox; injected for tests.
pub async fn l3_with(a: &ValidatedArtifact, sb: &impl Sandbox) -> Result<L3Report, ImportError>;
/// Reuse M05 L4 tier; Novice → TierReviewRequired (renderer review screen).
pub fn tier_gate(a: &ValidatedArtifact, tier: Tier) -> Result<(), ImportError>;
/// Install + Stage-B skills.lock write (path-agnostic; shell resolves the root).
pub fn install_with(a: &ValidatedArtifact, rep: L3Report, lock: &Path) -> Result<(), ImportError>;
```

The pipeline composes these in `import_artifact_with` (the unit-tested seam the Tauri command wraps):

```rust
pub async fn import_artifact_with(
    src: ImportSource, kind: ArtifactKind, tier: Tier, host_os: &str, lock: &Path,
    get: &impl Fetcher, sb: &impl Sandbox, reg: &impl McpRegistry,
) -> Result<Installed, ImportError> {
    let bytes = fetch_with(&src, get).await?;                       // C.3.2 (network-gated)
    let art   = validate(kind, &bytes)?;                            // schemas/{skill,tool,agent}.v1.json
    if !art.compatible_os.iter().any(|o| o == host_os) {            // §15c — BLOCKING (C.3.3)
        return Err(ImportError::OsMismatch { artifact: art.compatible_os.clone(), host: host_os.into() });
    }
    let report = l3_with(&art, sb).await?;                          // reuse runtime-sandbox (M05)
    tier_gate(&art, tier)?;                                         // reuse L4 (M05) → TierReviewRequired for Novice
    if matches!(kind, ArtifactKind::McpServer) {
        reg.upsert(art.as_mcp_config()?)?;                          // M06 MCP Manager registry — reuse
    }
    install_with(&art, report.clone(), lock)?;                      // → skills_lock::write_entry (B.3.3)
    Ok(Installed { lock_key: art.name_at_version(), report })
}
```

Each `?` is a distinct `ImportError` the renderer (E) maps to a phase; `TierReviewRequired` is not an error-stop but a "renderer shows the review modal" outcome (E.3.4). MCP-server-config import (`ArtifactKind::McpServer`) routes into the M06 MCP Manager registry (`runtime-mcp::client::registry`) — reuse, do not rebuild.

#### C.3.2 Network capability gate (Hard Rule 4)

```rust
pub async fn fetch_with(src: &ImportSource, get: &impl Fetcher) -> Result<Vec<u8>, ImportError> {
    match src {
        ImportSource::File(p) => std::fs::read(p).map_err(|e| ImportError::Fetch(e.to_string())),
        ImportSource::Url(u) => {
            enforcer.check(IMPORT_AGENT, &[CapabilityDeclaration::network(host_of(u)?)])  // M05 L1
                .map_err(|e| ImportError::Fetch(format!("network capability denied: {e}")))?;
            get.get(u).await.map_err(|e| ImportError::Fetch(e.to_string()))               // ONLY the user URL
        }
    }
}
```

No unguarded egress; only the user-supplied URL is hit; no phone-home (Hard Rule 4). The real `reqwest` `Fetcher` impl lives in `src/import/fetch.rs` — the new runtime-main OS-call-holdout exclusion (C.3.5); `fetch_with` itself is seam-tested with an injected `Fetcher`.

#### C.3.3 §15c `compatible_os` + §15d metadata

`compatible_os` mismatch is a **blocking** `ImportError::OsMismatch` (C.3.1, not a warning — the install halts before L3, spec §15c). `requires_secrets` + `runtime_dependency_class` (§15d) are parsed off the validated artifact and carried into the install result so the tier-gate review screen (E.3.4) renders the secrets notice before first run:

```rust
struct ArtifactMeta {                       // parsed in validate(), §15d
    requires_secrets: Vec<String>,          // → E review "provision before run"
    runtime_dependency_class: DepClass,     // surfaced read-only
    compatible_os: Vec<String>,             // → §15c gate (C.3.1)
}
```

`compatible_os` is checked **before** L3 (cheap reject before the expensive sandbox run).

#### C.3.4 `share_provenance` (ADR-0005, runtime-to-runtime only)

On framework export, populate:

```jsonc
"share_provenance": {
  "exported_at": "<rfc3339>",
  "exported_by": "share-it@0.1.0",
  "for_runtime_class": "desktop_runtime",
  "for_os": ["windows", "macos", "linux"],
  "rebake_changes": []
}
```

(MVP §M7 line 215.) `rebake_changes: []` always — v0.1 export is runtime-to-runtime; no rebaking, no Share It module (v1.0). On import the block is parsed and handed to the renderer as a trust signal (E). If `share_provenance`/§15d fields are not already in the framework/artifact schemas, add per ADR-0005 (schema + `cargo xtask regenerate-types`; `<schema_ref_audit>`).

#### C.3.5 Coverage exclusion + threat model

`src/import/fetch.rs` (the real-reqwest `Fetcher` impl) is a new runtime-main OS-call-holdout exclusion (seam-tested via `fetch_with` + injected fakes; real path via the `--features integration` smoke). Per the v1.8 CLAUDE.md §6 four-mirror rule, the new `--ignore-filename-regex` term syncs across `docs/coverage-policy.md` §A/§C + CLAUDE.md §5/§6 + `codecov.yml` in **this** stage's commit; G's `<coverage_policy_reconciliation>` verifies it.

**Threat model (explicit, per the supply-chain best-practice review).** Import-by-raw-URL is **trust-on-first-use against a mutable ref** — a GitHub raw URL points at a branch/tag an attacker could redirect (current guidance: prefer immutable SHA-pins over mutable tags). v0.1's layered mitigation is the SLSA "progressively stronger guarantees" model: (1) **sandbox-before-trust** — L3 runs the artifact's declared examples in `runtime-sandbox` before install; (2) **tier-gate review** — Novice sees the capability disclosure + L3 report before accepting; (3) **hash-lock-on-first-install** — `skills.lock` pins the SRI-encoded content hash so any *subsequent* tamper is blocked (Stage B `artifact_hash_mismatch`). Cryptographic provenance (Sigstore keyless signing + SLSA build attestation + TUF trust-root) is the **v1.0** layer and attaches at the existing `share_provenance` seam (ADR-0005) — explicitly deferred, not missed. ADR-0014 records this threat model + the staged posture.

### C.4 Tests

URL-fetch (wiremock) + file-pick happy paths; schema-invalid → rejected with the validation report; L3 failure → blocked; tier-gate forbidden → review-required; successful install → `skills.lock` entry written (Stage B integration); `compatible_os` mismatch → `OsMismatch` blocking error (spec §15c); `share_provenance` round-trips export→import; MCP-server-config import lands in the M06 registry. Acceptance: pipeline seams ≥95 within runtime-main; workspace ≥80; wiremock (no live network in the gate; `--features integration` opt-in only); v1.8 invariant proven.

### C.5 CLI Prompt

```xml
<work_stage_prompt id="M07.C">
  <context>
    M07 Stage C — the import pipeline backend: URL/file fetch →
    schema-validate → L3 (reuse runtime-sandbox) → tier-gate (reuse
    L4) → install → skills.lock (Stage B). §15c compatible_os blocking
    check; ADR-0005 share_provenance export/import (runtime-to-runtime
    only — NO rebake, NO Share It module). MCP-server-config import
    reuses the M06 MCP Manager. NOT the renderer (E) and NOT the agent
    loop (D).
  </context>
  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8)</file>
    <file>docs/build-prompts/M07-registry-import.md (Background, Key constraints, Stage C C.1–C.4)</file>
    <file>agent-runtime-spec.md §2152–2211 (Phase 7) + §15/§15c/§15d (share_provenance, compatible_os, requires_secrets, runtime_dependency_class)</file>
    <file>docs/adr/0005-*.md (sharing metadata) + docs/adr/0007 (in-process HitlSeam — reuse for any prompt)</file>
    <file>crates/runtime-sandbox/ (L3 reuse surface) + crates/runtime-main/src/tier/ (L4 reuse) + the M07.B skills_lock module</file>
    <file>crates/runtime-mcp/src/client/registry.rs (MCP-server-config import target)</file>
    <file>docs/build-prompts/retrospectives/M07.B-retrospective.md + RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>
  <read_prior_stages><stage id="M07.B" retro="docs/build-prompts/retrospectives/M07.B-retrospective.md"/></read_prior_stages>
  <deliverable ref="docs/build-prompts/M07-registry-import.md" section="C.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <tdd_discipline strict="true">
    <red_phase>Standalone `test(M07.C): …` — fetch (wiremock URL + file), schema-invalid reject, L3 fail block, tier forbidden, install→lock, OsMismatch, share_provenance round-trip, MCP-config→M06 registry. Right-reason red. Surface.</red_phase>
    <green_phase>Seam-first impl; no test-file edits in impl commit; body proves `**/tests/**` diff EMPTY. No Co-Authored-By.</green_phase>
  </tdd_discipline>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M07-registry-import.md" section="C.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M07-registry-import.md" section="Key constraints"/>
  <gates milestone="M07"/>
  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs"/>
  </coverage_gate>
  <self_correction_budget>3</self_correction_budget>
  <adr_triggers>
    <trigger>ADR-0005 already accepted (sharing metadata) — C implements it, no new ADR unless a schema add for share_provenance/§15d fields triggers §14 (then bump schema + ADR + four-mirror if a gate moves). The `src.import.fetch\.rs` exclusion (real-reqwest OS-call holdout, seam-tested) is a NEW runtime-main gate exclusion → the v1.8 four-mirror sync (CLAUDE.md §5/§6 + docs/coverage-policy.md §A/§C + codecov.yml) in THIS stage's commit, reconciled at G.</trigger>
  </adr_triggers>
  <gotchas>
    <trap>Hard Rule 4 — the fetch hits ONLY the user-supplied URL; network is capability-gated through the M05 L1 enforcer; no phone-home.</trap>
    <trap>Reuse: L3 = runtime-sandbox, L4 = tier, prompts = HitlSeam (ADR-0007), MCP-config = M06 registry. Do NOT rebuild any of these.</trap>
    <trap>§15c — compatible_os mismatch is a BLOCKING error, not a warning.</trap>
    <trap>The new src.import.fetch.rs exclusion: sync all four coverage mirrors in this commit (v1.8 §6 rule); G verifies.</trap>
    <trap>#81 — llvm-cov clean before gates.</trap>
  </gotchas>
  <pre_flight_check>
    <check name="branch">HEAD == claude/m07-registry-import; M07.B impl commit present</check>
    <check name="reuse_surfaces">grep-confirm runtime-sandbox L3 entrypoint + tier L4 + M06 registry add API exist before wiring</check>
  </pre_flight_check>
  <runtime_environment os="windows"/>
  <time_box hours="7-9"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the seam/wrapper split + the new src.import.fetch.rs exclusion + the four-mirror sync done. Confirm L3/L4/HitlSeam/M06-registry reuse (no rebuild). share_provenance round-trip + §15c blocking behavior pinned.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M07-registry-import.md" section="C.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state; strict-TDD EMPTY proof</item>
    <item>pipeline seam/wrapper split + the new coverage exclusion + four-mirror sync diff</item>
    <item>reuse confirmation (L3/L4/HitlSeam/M06 registry — not rebuilt)</item>
    <item>gate results (runtime-main ≥95; workspace ≥80; CI-parity)</item>
    <item>M07.C retrospective [END]; explicit "Stage M07.C is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### C.6 Commit Message

```
feat(runtime): M07 Stage C — import pipeline backend (URL/file → validate → L3 → tier → install → lock)

Composes the import pipeline over M05 (L3 sandbox, L4 tier) +
M07.B skills.lock + a capability-gated fetch. §15c compatible_os
mismatch blocks; ADR-0005 share_provenance populated on export,
surfaced on import (runtime-to-runtime only). MCP-server-config
import reuses the M06 MCP Manager. New src.import.fetch.rs
OS-call-holdout exclusion synced across all four coverage mirrors
(v1.8 §6). Strict v1.8 two-commit TDD: '**/tests/**' diff EMPTY.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage D1 — Concrete dispatch construction + `ConnectionResolver` (ADR-0011 a–c)

### D1.1 Problem Statement

ADR-0011 Context #1–#3: on `main` there is no `impl ConnectionResolver for McpClient`, and `McpDispatcher::new` requires `Arc<RwLock<NamespaceResolver>>` + `Arc<CapabilityEnforcer>` with no `src-tauri` construction site. D1 builds the **construction half** — the concrete dispatcher becomes constructible in the shell and the §5a re-resolution-on-connect driver (M06.V 🟡#1) exists — so D2's agent-with-tools loop has a real `Arc<dyn McpToolDispatch>` to consume. CQ-6 (`ServerStatus` enum) + EFF-4 (batched `run_health_pass`) land here because they live in the `runtime-mcp` client/health-ping surface D1 touches. D1 ships **no** agent loop and **no** `token_usage` projector (that is D2) — it closes ADR-0011 (a)–(c) only.

### D1.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-mcp/src/client/…` | exists | `impl ConnectionResolver for McpClient` (connect/disconnect re-resolution + `tool_alias_ambiguous`-on-connect); `ServerStatus` enum (CQ-6); batched `run_health_pass` (EFF-4). |
| `src-tauri/src/commands.rs` / `drone_lifecycle.rs` | exists | Construct `Arc<RwLock<NamespaceResolver>>` + `Arc<CapabilityEnforcer>` + `McpDispatcher`; expose it as `Arc<dyn McpToolDispatch>` through the `run_session_with` seam (replacing M06.F's `None`). Shell wrapper = §5 holdout; the `*_with` seam is unit-tested. |
| `CHANGELOG.md` / retro | exist/new | Stage D1 entries. |

### D1.3 Detailed Changes

#### D1.3.1 `impl ConnectionResolver for McpClient` (ADR-0011 Context #1; M06.V 🟡#1)

`ConnectionResolver` is the M06-shipped trait `McpDispatcher` consumes to keep the namespace fresh; on `main` `McpClient` does not implement it (ADR-0011 Context #1). The build reads the M06-shipped `ConnectionResolver` trait + `NamespaceResolver` surface verbatim first (illustrative shape):

```rust
// crates/runtime-mcp/src/client/connection_resolver.rs (new)
#[async_trait::async_trait]
impl ConnectionResolver for McpClient {
    /// Called by McpDispatcher when a server connection comes up.
    /// Snapshots the server's tool set into the resolver and returns
    /// every short name that is NEWLY ambiguous across the connected set.
    async fn on_connect(&self, server: &str) -> Result<Vec<NewAmbiguity>, McpError> {
        let tools: Vec<McpTool> = self.list_tools(server).await?;          // rmcp list_tools
        let names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        let mut resolver = self.resolver.write().await;                    // Arc<RwLock<NamespaceResolver>>
        resolver.add_server(server, names);                               // updates connected_servers
        Ok(resolver.re_evaluate_short_names())                            // M06.D NamespaceResolver API
    }

    /// Called when a server drops; short names may become UNambiguous again.
    async fn on_disconnect(&self, server: &str) -> Result<(), McpError> {
        self.resolver.write().await.remove_server(server);
        Ok(())
    }
}

// The dispatch/connect site translates each NewAmbiguity → an event:
for amb in client.on_connect(server).await? {
    emit(AgentEvent::ToolAliasAmbiguous { name: amb.short, candidates: amb.candidates });
}
```

This is the §5a re-resolution-on-connect driver M06.V 🟡#1 named ("no production driver" was the finding; this impl + the emit site is the driver). The build must NOT invent the `NamespaceResolver`/`NewAmbiguity` API — it reads the M06.D-shipped `namespace/mod.rs` and uses `add_server`/`remove_server`/`re_evaluate_short_names` verbatim (gotcha #74 — derive from shipped code, not memory). The Wire trace for this is authored against **`McpDispatcher`** (the resolver lives there per ADR-0010), NOT `McpClient` — `McpClient` only *impls* the trait. That is the `<wire_trace_vs_adr_reconcile>` reconciliation of M06.V Dec 6 (trace #6), declared in D1.5.

#### D1.3.2 Concrete `McpDispatcher` construction in `src-tauri` (ADR-0011 Context #2/#3)

`McpDispatcher::new` requires `Arc<RwLock<NamespaceResolver>>` + `Arc<CapabilityEnforcer>`; ADR-0011 Context #2/#3 record neither has a `src-tauri` construction site. D1 adds the ctor site and threads a real `Arc<dyn McpToolDispatch>` through a new `run_session_with` seam (the M06.F `run_smoke_session_with` passed `None`):

```rust
// src-tauri/src/commands.rs — illustrative; build reads the M06-shipped
// McpDispatcher::new signature + run_smoke_session_with verbatim first.
fn build_mcp_dispatcher(
    framework: &Framework,
    tier: Tier,
    registry: Arc<Registry>,                 // M06 MCP Manager registry (session.sqlite, ADR-0012)
    audit: Option<Arc<dyn AuditWriter>>,
) -> Arc<dyn McpToolDispatch> {
    let resolver = Arc::new(RwLock::new(NamespaceResolver::new(BTreeMap::new())));
    let enforcer = Arc::new(CapabilityEnforcer::from_framework(framework, tier));   // L1, M05.B
    let client   = Arc::new(McpClient::new(registry));                             // impls ConnectionResolver (D1.3.1)
    Arc::new(McpDispatcher::new(client, resolver, enforcer, audit))                // ADR-0010 concrete impl
}

// run_smoke_session (the shell command) now:
let dispatcher = build_mcp_dispatcher(&framework, tier, registry.clone(), audit.clone());
run_session_with(provider, tx, drone, smoke_config(), Some(dispatcher)).await
//                                                     ^^^^ was None at M06.F (ADR-0011)
```

`build_mcp_dispatcher` is the unit-tested seam (the dispatcher is constructible + correctly threaded; assert it is the concrete `McpDispatcher`, not `None`/mock). The `run_smoke_session` shell command wrapping it is the §5 tauri-shell holdout (50% patch gate, `docs/coverage-policy.md` §D). `CapabilityEnforcer` construction is CODEOWNERS-flagged (Hard Rule 8) — D1.3 + the `<construction_reachability_check>` is the surfaced plan. D1 wires the construction; D2 builds the loop `run_session_with` drives.

#### D1.3.3 CQ-6 — `status: String` → `ServerStatus` enum

`McpServerRecord`/`McpServerSummary` carry `status: String`; M06.B already defined the enum at the schema level (`schemas/mcp.v1.json` `McpServerStatus`). CQ-6 makes the Rust side use the generated enum instead of a bare `String`:

```rust
// generated from schemas/mcp.v1.json#/$defs/McpServerStatus (typify) —
// do NOT hand-write (Hard Rule 5); CQ-6 swaps the field type + call sites.
pub enum ServerStatus { Connected, Disconnected, HealthPending, Error }
// McpServerRecord { …, status: ServerStatus }   // was: status: String
```

The health-ping state machine writes `ServerStatus::Connected` / `Error` (D1.3.4) instead of `"connected"`/`"errored"` string literals; the registry serialization round-trips the enum. If the schema needs a minor touch (it should not — M06.B shipped it), schema-first + `cargo xtask regenerate-types`.

#### D1.3.4 EFF-4 — batched `run_health_pass`

`run_health_pass` does K sequential SQLite `UPDATE`s (one per server). Replace with one batched transaction so the multi-server path (M07 registry can hold >1 server) is correct + atomic:

```rust
// before: for s in servers { conn.execute("UPDATE mcp_servers SET status=?…", …)?; }
// after:
let tx = conn.transaction()?;
{
    let mut up = tx.prepare("UPDATE mcp_servers SET status=?1, last_alive=?2 WHERE name=?3")?;
    for s in &servers { up.execute(params![s.status, s.last_alive, s.name])?; }
}
tx.commit()?;   // one fsync, atomic across the pass; status is ServerStatus (CQ-6)
```

Zero-cost at v0.1 single-server but lands now with the multi-server-capable resolver wire.

#### D1.3.5 Construction-graph closure (the inverse of Stage A's map)

A's `<construction_reachability_check>` recorded every ADR-0011 input `inputs_reachable="false"`. D1.5's `<construction_reachability_check>` records each now `inputs_reachable="true — constructed at src-tauri/src/commands.rs:NNN"` (resolver, enforcer, dispatcher) — D1 is where the (a)–(c) graph closes. D2 consumes it.

#### D1.3.6 Schema discipline

No new event variant expected in D1 (`tool_alias_ambiguous` shipped at M06.D). If the `ServerStatus` schema needs a minor touch, schema-first + `cargo xtask regenerate-types`; `<schema_ref_audit>` for any cross-`$ref`. Avoid gratuitous schema churn (Hard Rule 5).

### D1.4 Tests

`ConnectionResolver` re-resolution: connect two servers exposing a colliding short name → `on_connect` returns the `NewAmbiguity`; disconnect → resolver no longer ambiguous. The concrete-construction seam test: `run_session_with(Some(dispatcher))` threads a real `McpDispatcher` (constructed via the shell ctor path under test) — assert it is the concrete impl, not a mock/`None`. CQ-6 `ServerStatus` round-trips; EFF-4 batched pass writes all rows in one transaction. Acceptance: runtime-mcp ≥95 (ConnectionResolver + ServerStatus); the construction seam covered (the §5 shell wrapper is the holdout); v1.8 two-commit invariant proven; any four-mirror coverage sync (runtime-mcp `transport/mod.rs` baseline) landed here + verified at G.

### D1.5 CLI Prompt

```xml
<work_stage_prompt id="M07.D1">
  <context>
    M07 Stage D1 — the ADR-0011 (a)–(c) construction half: impl
    ConnectionResolver for McpClient (§5a re-resolution-on-connect,
    M06.V 🟡#1) + concrete McpDispatcher constructed in src-tauri
    (real NamespaceResolver + CapabilityEnforcer, replacing M06.F's
    None) + CQ-6 (ServerStatus enum) + EFF-4 (batched run_health_pass).
    NO agent loop, NO token_usage projector (that is D2). Closes the
    A-mapped construction graph (inputs become reachable). Touches
    CODEOWNERS-flagged capability-enforcer construction — the
    construction-reachability map IS the surfaced plan (Hard Rule 8).
  </context>
  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — construction_reachability_check + wire_trace_vs_adr_reconcile)</file>
    <file>docs/build-prompts/M07-registry-import.md (Background, Key constraints, Stage D1 D1.1–D1.4)</file>
    <file>docs/adr/0010-mcp-dispatch-dependency-inversion.md + docs/adr/0011-m06-stage-f-scope.md (Context #1–#3 — the construction this stage closes, verbatim)</file>
    <file>agent-runtime-spec.md §5a (Tool Namespace Resolution / re-resolution) + §8.security L1/L2a</file>
    <file>docs/build-prompts/retrospectives/M06.V-retrospective.md (🟡#1 §5a driver; Dec 6 wire-trace-vs-ADR)</file>
    <file>docs/build-prompts/retrospectives/M07.A-retrospective.md (the construction-reachability map A authored — D1 inverts it) + M07.C-retrospective.md</file>
    <file>docs/gap-analysis.md (CQ-6/EFF-4 + transport/mod.rs baseline carry-forward text) + RETROSPECTIVE-TEMPLATE.md + gotchas #56/#68/#81</file>
  </read_first>
  <read_prior_stages>
    <stage id="M07.A" retro="docs/build-prompts/retrospectives/M07.A-retrospective.md"/>
    <stage id="M07.C" retro="docs/build-prompts/retrospectives/M07.C-retrospective.md"/>
  </read_prior_stages>
  <deliverable ref="docs/build-prompts/M07-registry-import.md" section="D1.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <tdd_discipline strict="true">
    <red_phase>Standalone `test(M07.D1): …` — ConnectionResolver re-resolution (connect/disconnect, tool_alias_ambiguous), the concrete-construction seam (run_session_with threads a real McpDispatcher), CQ-6 ServerStatus, EFF-4 batched. Right-reason red. Surface for red approval.</red_phase>
    <green_phase>Impl ConnectionResolver + the shell ctor + CQ-6/EFF-4; no test-file edits in impl commit; body proves `git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**'` EMPTY. No Co-Authored-By.</green_phase>
  </tdd_discipline>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M07-registry-import.md" section="D1.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M07-registry-import.md" section="Key constraints"/>
  <gates milestone="M07"/>
  <coverage_gate>
    <gate scope="package" name="runtime-mcp" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.lib\.rs|src.transport.stdio\.rs|src.transport.http\.rs|src.client.auth_keyring\.rs|src.client.lifecycle\.rs"/>
  </coverage_gate>
  <self_correction_budget>3</self_correction_budget>
  <construction_reachability_check>
    <wire claim="concrete McpDispatcher constructible in src-tauri" constructor="McpDispatcher::new (src-tauri/src/commands.rs — added this stage)" inputs_reachable="true — NamespaceResolver + CapabilityEnforcer constructed in src-tauri this stage (closes ADR-0011 Context #2/#3)" resolution="discharged in D1; D2's loop consumes it"/>
    <wire claim="impl ConnectionResolver for McpClient" constructor="runtime-mcp client (added this stage)" inputs_reachable="true" resolution="discharged in D1 (M06.V 🟡#1 §5a driver)"/>
  </construction_reachability_check>
  <wire_trace_vs_adr_reconcile>
    <trace id="6" assumes="McpClient drives namespace re-resolution" adr_checked="ADR-0010" superseded="true — resolver lives in McpDispatcher" resolution="author the §5a re-resolution trace against McpDispatcher; McpClient only impls ConnectionResolver (M06.V Dec 6)"/>
  </wire_trace_vs_adr_reconcile>
  <adr_triggers>
    <trigger>None new — D1 discharges ADR-0010/0011 (a)–(c) (accepted). If D1 moves the runtime-mcp transport/mod.rs gate baseline, the v1.8 four-mirror sync lands here + verified at G.</trigger>
  </adr_triggers>
  <gotchas>
    <trap>M06.V Dec 6 — the §5a re-resolution trace is authored against McpDispatcher, NOT McpClient (the &lt;wire_trace_vs_adr_reconcile&gt; above).</trap>
    <trap>Hard Rule 8 — the CapabilityEnforcer construction is CODEOWNERS-flagged; the construction map + D1.3 is the plan-first surface.</trap>
    <trap>#81 — llvm-cov clean before gates.</trap>
  </gotchas>
  <pre_flight_check>
    <check name="branch">HEAD == claude/m07-registry-import; M07.A + M07.C impl commits present</check>
    <check name="adr0011_inputs">grep -rn "CapabilityEnforcer|NamespaceResolver" src-tauri/src/ — confirm A's map premise (zero ctor sites) still holds before D1 constructs them</check>
  </pre_flight_check>
  <runtime_environment os="windows"/>
  <time_box hours="4-5"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>ADR-0011 (a)–(c) construction map closed (each input now reachable, file:line). §5a trace authored against McpDispatcher. CQ-6/EFF-4 disposition. Any transport/mod.rs four-mirror sync. Confirm NO agent loop / NO token_usage shipped (that is D2).</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M07-registry-import.md" section="D1.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state; strict-TDD '**/tests/**' EMPTY proof</item>
    <item>ADR-0011 (a)–(c) construction-graph closure map (file:line per input)</item>
    <item>ConnectionResolver re-resolution + concrete-construction seam test results</item>
    <item>CQ-6/EFF-4 disposition; any transport/mod.rs four-mirror sync</item>
    <item>gate results (runtime-mcp ≥95; CI-parity)</item>
    <item>M07.D1 retrospective [END]; explicit "Stage M07.D1 is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### D1.6 Commit Message

```
feat(runtime): M07 Stage D1 — ADR-0011 (a)-(c) construction: ConnectionResolver + concrete McpDispatcher

impl ConnectionResolver for McpClient (§5a re-resolution-on-connect,
M06.V 🟡#1); concrete McpDispatcher constructed in src-tauri (real
NamespaceResolver + CapabilityEnforcer, replacing M06.F's None) —
closes the A-mapped ADR-0011 (a)-(c) construction graph (inputs now
reachable). CQ-6 (ServerStatus enum), EFF-4 (batched run_health_pass).
No agent loop / no token_usage (D2). §5a trace authored against
McpDispatcher per M06.V Dec 6. Strict v1.8 two-commit TDD:
'**/tests/**' diff EMPTY.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage D2 — Agent-with-tools loop + `token_usage` projector (ADR-0011 d / M06.5)

### D2.1 Problem Statement

The only `AgentSdk` construction is the no-tools smoke path (ADR-0011 Context #4). D2 builds the multi-turn **agent-with-tools loop** that consumes D1's concrete `Arc<dyn McpToolDispatch>` — discharging ADR-0011 (d). M06.5 established no production code writes `token_usage` (sole INSERT is `#[cfg(test)]` in `vdr.rs`); the real loop is the first production token-bearing signal source, so the `token_usage` projector lands here, closing the M06.5 open finding. CQ-2 (the dead `Invoked` arm) is removed at the type level with the loop's outcome enum. D2's assembled-app regression is what makes M07.V's mandatory `--features integration` smoke real.

### D2.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/sdk/…` | exists | The multi-turn agent-with-tools loop; the `DispatchOutcome` enum split (CQ-2 — no dead `Invoked` arm). |
| `crates/runtime-drone/src/…` | exists | The `token_usage` projector in `handle_write_signal` (CODEOWNERS-flagged; not a new `DroneCommand`). |
| `crates/runtime-main/tests/…` (new) | new | The assembled-app regression (real drone subprocess + stub provider), mirroring `smoke_signal_persistence.rs`. |
| `CHANGELOG.md` / retro | exist/new | Stage D2 entries. |

### D2.3 Detailed Changes

#### D2.3.1 Multi-turn agent-with-tools loop — `crates/runtime-main/src/sdk/`

The M06.F `run_agent_with_provider_stream` is single-pass (no tool-result feedback — the no-tools smoke path). D2 makes it multi-turn: a `ProviderEvent::ToolUse` is offered to D1's concrete dispatcher first; the result is fed back into the provider stream as a new turn; the loop continues until the model stops requesting tools. The build reads the M06.F-shipped stream loop + `dispatch_if_mcp` signature verbatim first (gotcha #74):

```rust
// crates/runtime-main/src/sdk/agent_sdk.rs — run_session_with's loop body.
// `dispatcher: Arc<dyn McpToolDispatch>` is D1's concrete McpDispatcher
// (Some(..) — M06.F passed None). `provider` is the multi-turn stream.
let mut turn = provider.start(config).await?;
loop {
    match turn.next().await? {
        ProviderEvent::ToolUse { id, name, input } => {
            match dispatcher.dispatch_if_mcp(&agent_id, &name, input.clone(), &aliases).await {
                // MCP tool resolved + dispatched: feed the result back as the
                // next turn's tool_result so the model can continue (multi-turn).
                Some(Ok(DispatchOutcome::Dispatched(value))) => {
                    self.emit(AgentEvent::ToolResultReceived { id, result: Some(value.clone()), error: None }).await?;
                    turn = provider.continue_with_tool_result(id, value).await?;   // ← the multi-turn step
                }
                Some(Ok(DispatchOutcome::Blocked(cv))) => {
                    self.emit(AgentEvent::CapabilityViolation(cv.clone())).await?;
                    self.emit(AgentEvent::McpRequestBlocked { server: cv.server, tool: cv.tool, reason: cv.reason }).await?;
                    self.hitl.await_decision(cv.into_prompt()).await?;            // existing on_capability_violation trigger
                }
                Some(Ok(DispatchOutcome::Ambiguous(candidates))) => {
                    self.emit(AgentEvent::ToolAliasAmbiguous { name, candidates }).await?;
                }
                Some(Err(e)) => self.emit(AgentEvent::ToolResultReceived { id, result: None, error: Some(e.to_string()) }).await?,
                None => { self.default_tool_dispatch(id, name, input).await?; }   // not an MCP tool — Stage-A L1 path
            }
        }
        ProviderEvent::Done => break,
        other => self.emit(other.into_agent_event()).await?,   // M06.5.B emit→persist_signal choke point (signals + token_usage signal)
    }
}
```

Every `self.emit` flows the M06.5.B `persist_signal` choke point (signals land in the drone); the token-bearing signal among them is what D2.3.3's projector consumes. Reuse `dispatch_if_mcp` (M06.D) verbatim — D2 changes the *caller* (real dispatcher, multi-turn feedback), not the dispatch primitive.

#### D2.3.2 CQ-2 — `DispatchOutcome` enum split (dead `Invoked` arm removed)

M06.D's outcome type carried an `Invoked` arm that was dead in production (constructed only with an empty `agent_id` — gap-analysis CQ-2/reuse-5). D2 splits it so the dead path cannot regress at the type level:

```rust
// crates/runtime-main/src/sdk/mcp_dispatch.rs
pub enum DispatchOutcome {
    Dispatched(serde_json::Value),   // tool resolved + invoked OK → feed back
    Blocked(CapViolation),           // L1/tier denied → McpRequestBlocked + HITL
    Ambiguous(Vec<String>),          // §5a short-name collision → ToolAliasAmbiguous
}
// REMOVED: `Invoked` — was only ever constructed with agent_id="" (dead in
// production; M06.V CQ-2/reuse-5). The D2.3.1 match is now exhaustive with
// NO catch-all arm, so reintroducing a dead variant is a compile error.
```

The exhaustive match (no `_ =>`) is the regression guard: any future variant forces every call site to handle it.

#### D2.3.3 `token_usage` projector — `crates/runtime-drone` (CODEOWNERS; closes M06.5)

M06.5 established no production code writes `token_usage` (sole INSERT is `#[cfg(test)]` in `vdr.rs`). The loop's token-bearing signal already flows through the drone's `handle_write_signal` (the `vdr` + `plan` projectors see it). Add a **third projector** parallel to those — **not** a new `DroneCommand` (no IPC-protocol change → no §11 ADR):

```rust
// crates/runtime-drone/src/projectors/token_usage.rs (new) — mirrors
// vdr::project_signal / plan_projector::project_signal exactly.
pub fn project_signal(tx: &rusqlite::Transaction, sig: &Signal) -> Result<(), DroneError> {
    if let Some(tu) = sig.token_usage() {            // the existing token-bearing signal kind
        tx.execute(
            "INSERT INTO token_usage(session_id, agent_id, input_tokens, output_tokens, ts)
             VALUES (?1,?2,?3,?4,?5)",
            params![sig.session_id, sig.agent_id, tu.input, tu.output, sig.ts],
        )?;
    }
    Ok(())
}
// wired into handle_write_signal alongside vdr::project_signal +
// plan_projector::project_signal — same transaction, same call site.
```

`runtime-drone` is CODEOWNERS-flagged (Hard Rule 8) — D2.3 + the `<construction_reachability_check>` is the surfaced plan. The build confirms at authoring which existing signal kind already carries token counts (the `vdr` projector path is the reference) and projects from it; it does **not** add a new signal variant unless none carries tokens (Hard Rule 5 — schema-first if it must).

#### D2.3.4 The assembled-app regression (the v1.8/§6 mandate; the M07.V enabler)

Mirror `crates/runtime-main/tests/smoke_signal_persistence.rs` (the M06.5 real-drone-subprocess harness) — drive the **real** loop + **real** drone, not a mock seam (the Stage-V blind spot the §6 mandate exists to kill):

```rust
#[tokio::test(flavor = "multi_thread")]
async fn agent_with_tools_loop_persists_signals_and_token_usage() {
    let drone = RealDroneSubprocess::spawn(&db_path).await;          // M06.5 harness
    let provider = StubProvider::script(vec![                        // NO live Anthropic
        ProviderEvent::ToolUse { id: "1".into(), name: "fs__read".into(), input: json!({"path":"/x"}) },
        ProviderEvent::Done,
    ]);
    let dispatcher = build_mcp_dispatcher(&fw, Tier::Promoted, registry, audit); // D1 ctor — CONCRETE
    run_session_with(provider, tx, drone.client(), cfg, Some(dispatcher)).await.unwrap();

    let db = drone.open_db();
    assert!(db.count("signals", &session_id) > 0, "signals persisted under run id");
    assert!(db.count("token_usage", &session_id) > 0, "token_usage > 0 — M06.5 closed");   // the falsifiable hypothesis
    assert!(dispatched_via_concrete_mcp_dispatcher(&db), "real McpDispatcher, not a mock");
    // + ConnectionResolver re-resolves on connect with tool_alias_ambiguous on collision
}
```

The phase-doc root cause ("the loop persists token_usage") is a **falsifiable hypothesis the assembled test must disprove** (v1.8 §6) — if it passes only with a mock dispatcher, the test is wrong, not the code. This is the test that makes M07.V's mandatory `--features integration` smoke meaningful.

#### D2.3.5 Construction-graph consumption

D2.5's `<construction_reachability_check>` records the loop **consuming** D1's now-reachable `Arc<dyn McpToolDispatch>` (constructor `run_session_with` at `src-tauri/src/commands.rs:NNN`, `inputs_reachable="true — D1 constructed it"`). This is the closure endpoint of the A→D1→D2 construction graph.

#### D2.3.6 Schema discipline

Prefer the existing token-bearing signal; only add an `event.v1.json` variant if the loop genuinely emits a new observable, schema-first + `cargo xtask regenerate-types` + `<schema_ref_audit>`. No gratuitous churn (Hard Rule 5).

### D2.4 Tests

The D2.3.4 assembled regression is the headline acceptance: real loop + real drone, `token_usage > 0`, signals persisted under the run id, dispatch via the concrete `McpDispatcher`, `ConnectionResolver` re-resolves. CQ-2: the `DispatchOutcome` match is exhaustive with no `Invoked`/catch-all (a compile-time guarantee + a test that each arm is reachable). Acceptance: runtime-main ≥95 (loop + the projector wiring on the main side); the drone projector covered by the drone gate; `--features integration` reference-MCP-server smoke runnable (the M07.V endpoint); v1.8 two-commit invariant proven.

### D2.5 CLI Prompt

```xml
<work_stage_prompt id="M07.D2">
  <context>
    M07 Stage D2 — ADR-0011 (d): the multi-turn agent-with-tools loop
    consuming D1's concrete McpDispatcher + the token_usage projector
    (closes the M06.5 open finding — the real loop is the first
    production token-bearing signal source) + CQ-2 (DispatchOutcome
    enum split, dead Invoked-arm removed). The assembled-app regression
    (real loop + real drone, token_usage>0) is what makes M07.V's
    mandatory --features integration smoke real. Touches CODEOWNERS
    runtime-drone (the projector) — D2.3 + the construction map is the
    surfaced plan (Hard Rule 8); NOT an IPC-protocol change (no new
    DroneCommand), so no §11 ADR.
  </context>
  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — construction_reachability_check; the §6 assembled-regression mandate)</file>
    <file>docs/build-prompts/M07-registry-import.md (Background, Key constraints, Stage D2 D2.1–D2.4)</file>
    <file>docs/adr/0011-m06-stage-f-scope.md (Context #4 — the no-tools-loop carry-forward this stage discharges)</file>
    <file>agent-runtime-spec.md §2c.3 (token tracking) + §8.security L1/L2a</file>
    <file>docs/build-prompts/retrospectives/M06.5-summary.md (token_usage finding; the assembled-app-regression mandate, lines 156–173)</file>
    <file>docs/build-prompts/retrospectives/M07.D1-retrospective.md (the construction graph D1 closed — D2 consumes it)</file>
    <file>crates/runtime-main/tests/smoke_signal_persistence.rs (the M06.5 real-drone assembled-regression harness archetype) + RETROSPECTIVE-TEMPLATE.md + gotchas #56/#66/#68/#81</file>
  </read_first>
  <read_prior_stages>
    <stage id="M07.D1" retro="docs/build-prompts/retrospectives/M07.D1-retrospective.md"/>
  </read_prior_stages>
  <deliverable ref="docs/build-prompts/M07-registry-import.md" section="D2.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <tdd_discipline strict="true">
    <red_phase>Standalone `test(M07.D2): …` — the ASSEMBLED regression (real drone subprocess + stub provider ToolUse): dispatch via the concrete McpDispatcher (D1, not a mock), signals persist, token_usage>0, ConnectionResolver re-resolves; CQ-2 exhaustive match. Right-reason red. Surface for red approval.</red_phase>
    <green_phase>Impl the loop + CQ-2 enum + the runtime-drone token_usage projector; no test-file edits in impl commit; body proves `git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**'` EMPTY. drone projector is CODEOWNERS — D2.3 + the construction map is the surfaced plan. No Co-Authored-By.</green_phase>
  </tdd_discipline>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M07-registry-import.md" section="D2.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M07-registry-import.md" section="Key constraints"/>
  <gates milestone="M07"/>
  <coverage_gate>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs"/>
  </coverage_gate>
  <self_correction_budget>3</self_correction_budget>
  <construction_reachability_check>
    <wire claim="agent-with-tools loop consumes Arc&lt;dyn McpToolDispatch&gt;" constructor="run_session_with (src-tauri/src/commands.rs:NNN — D1 wired it)" inputs_reachable="true — D1 constructed + injected the concrete McpDispatcher" resolution="discharged in D2 (ADR-0011 d); closure endpoint of the A→D1→D2 graph"/>
  </construction_reachability_check>
  <adr_triggers>
    <trigger>None new — D2 discharges ADR-0011 (d). The runtime-drone token_usage projector is CODEOWNERS (Hard Rule 8) but NOT an IPC-protocol change (no new DroneCommand — it projects from an existing token-bearing signal, parallel to vdr/plan projectors), so no §11 ADR.</trigger>
  </adr_triggers>
  <gotchas>
    <trap>#66 / M06.5 mandate — the regression MUST drive the assembled loop through a real drone subprocess, not a mock McpToolDispatch seam (that already passes; it is the Stage-V blind spot). Assert token_usage>0, not just signals>0.</trap>
    <trap>Hard Rule 8 — runtime-drone projector is CODEOWNERS; D2.3 + the construction map is the plan-first surface. No new DroneCommand (no §11 ADR).</trap>
    <trap>CQ-2 — split DispatchOutcome so the dead Invoked-arm is gone at the type level; exhaustive match, no catch-all.</trap>
    <trap>#56/#81 — runtime-main llvm-cov Windows-local (TD-005 closed in A; cite inline if still flaky); llvm-cov clean first.</trap>
  </gotchas>
  <pre_flight_check>
    <check name="branch">HEAD == claude/m07-registry-import; M07.D1 impl commit present (the concrete McpDispatcher D2 consumes)</check>
    <check name="d1_dispatcher">grep -rn "McpDispatcher::new" src-tauri/src/ — confirm D1 actually constructed it before D2's loop consumes it</check>
  </pre_flight_check>
  <runtime_environment os="windows"/>
  <time_box hours="4-5"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Confirm the assembled regression drives the real loop+drone (not a mock) and asserts token_usage>0 (M06.5 closed — cite the test). CQ-2 enum disposition (no Invoked arm). The A→D1→D2 construction graph closure endpoint. token_usage projector: confirm projector-not-new-DroneCommand.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M07-registry-import.md" section="D2.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state; strict-TDD '**/tests/**' EMPTY proof</item>
    <item>assembled-regression proof: real loop+drone, token_usage>0, signals persisted, dispatch via concrete McpDispatcher, ConnectionResolver re-resolves</item>
    <item>ADR-0011 (d) discharge + A→D1→D2 construction-graph closure</item>
    <item>CQ-2 disposition; token_usage projector (CODEOWNERS plan honored; no new DroneCommand)</item>
    <item>gate results (runtime-main ≥95; drone gate; CI-parity)</item>
    <item>M07.D2 retrospective [END]; explicit "Stage M07.D2 is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### D2.6 Commit Message

```
feat(runtime): M07 Stage D2 — ADR-0011 (d): agent-with-tools loop + token_usage projector

Multi-turn agent-with-tools loop consuming D1's concrete
McpDispatcher (discharges ADR-0011 d, replaces the no-tools smoke
path); runtime-drone token_usage projector closing the M06.5 open
finding (the real loop is the first production token-bearing signal
source; projector over an existing signal — no new DroneCommand, no
§11 ADR). CQ-2 (DispatchOutcome enum split, dead Invoked-arm
removed). Assembled regression drives the real loop + real drone
subprocess (not a mock): token_usage>0, signals persisted,
ConnectionResolver re-resolves — the M07.V integration-smoke
enabler. Strict v1.8 two-commit TDD: '**/tests/**' diff EMPTY.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```


## Stage E — Renderer: Builder Import panel

### E.1 Problem Statement

MVP §M7 lines 213/223: an `Import` panel in the Builder — paste a GitHub raw URL or pick a local file → the install flow surfaces a tier-gate review screen (capability disclosure plain-English + L3 report + Install/Reject), the `share_provenance` block as a trust signal, and a hash-mismatch reinstall/remove prompt. The renderer has no Import panel. Per the v1.8 `<wire_signature_audit>` + `<phase_doc_inventory_audit shape=…>` (the M06.E lesson): the Tauri command params and store-slot shapes are pinned to the actual wire BEFORE authoring component pseudocode.

### E.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/components/ImportPanel.tsx` (+ tier-gate review modal, share-provenance badge, hash-mismatch prompt) | new | The panel + review screen + trust-signal block + reinstall prompt. |
| `src/state/…` | exists | Import state slot (shape pinned via `<phase_doc_inventory_audit shape=…>`). |
| `src/lib/ipc.ts` (or equivalent) | exists | `import_artifact` wrapper — params pinned to the actual Stage C Tauri command via `<wire_signature_audit>`. |
| Playwright + vitest | new | E2E (paste-URL → review → install → palette) + unit. |
| `CHANGELOG.md` / retro | exist/new | Stage E entries. |

### E.3 Detailed Changes

#### E.3.1 Wire pinning (v1.8 discipline — BEFORE pseudocode)

Pin to the **shipped** Stage C/D1/D2 wire, not assumptions (the M06.E drift lesson — five drifts at E because pseudocode preceded wire-verification):
- `<wire_signature_audit>` (E.5): read `src-tauri/src/commands.rs` for the committed `import_artifact` signature; record `actual_params` verbatim.
- `<phase_doc_inventory_audit shape=…>` (E.5): read `src/state/` for the committed import state slot; record its actual TS type (e.g. `Record<string, ImportStatus>`, not an assumed `Map`).

#### E.3.2 IPC wrapper + store slot — `src/lib/ipc.ts` + `src/state/`

Pinned from the shipped Stage C/D2 wire (E.3.1), not assumed:

```ts
// src/lib/ipc.ts — params PINNED from the Stage C src-tauri/src/commands.rs
// import_artifact signature at authoring time (do NOT assume {config}).
export type ImportSource = { kind: 'url'; url: string } | { kind: 'file'; path: string };
export async function importArtifact(src: ImportSource, kind: ArtifactKind): Promise<ImportResult> {
  return invoke('import_artifact', { src, kind });   // <PINNED arg names from the shipped command>
}

// src/state/ — slot SHAPE pinned (M06.E lesson: it is a Record, not a Map):
type ImportState = Record<string, {                 // key = name@version
  phase: 'idle'|'fetching'|'validating'|'l3'|'tier_review'|'installed'|'error';
  report?: L3Report; provenance?: ShareProvenance; error?: string;
}>;
```

#### E.3.3 `src/components/ImportPanel.tsx`

```tsx
export function ImportPanel() {
  const items = useStore(s => s.import);                       // the pinned Record slot
  const [src, setSrc] = useState<ImportSource>();
  const onSubmit = async () => {
    const r = await importArtifact(src!, inferKind(src!));      // ipc.ts wrapper (E.3.2)
    // phase transitions are driven by the event stream reducer (E.3.5),
    // not local state — single source of truth.
  };
  return (
    <Panel>
      <UrlInput onChange={v => setSrc({ kind:'url', url:v })}/>           {/* paste GitHub raw URL */}
      <FilePicker onPick={p => setSrc({ kind:'file', path:p })}/>          {/* @tauri-apps/api dialog (gotcha #23: Playwright-mocked) */}
      {Object.entries(items).map(([k,it]) => <ImportRow key={k} item={it}/>)}
    </Panel>
  );
}
```

Two entry affordances; phase comes from the store (E.3.5 reducer), not component-local state — so a backend phase change can't desync the UI.

#### E.3.4 Tier-gate review modal (reuse, don't rebuild)

The review modal **reuses the M05 capability-disclosure plain-English component** (the same one M05.F used for `capability_violation`) — do not rebuild plain-English disclosure:

```tsx
function TierReviewModal({ item }: { item: ImportItem }) {
  return (
    <Modal>
      <CapabilityDisclosure caps={item.report.capabilities}/>   {/* M05 component, reused verbatim */}
      <L3ReportView report={item.report}/>
      {item.requiresSecrets?.length ? <RequiresSecretsNotice names={item.requiresSecrets}/> : null}  {/* §15d, C.3.3 */}
      <ShareProvenanceBadge p={item.provenance}/>                {/* E.3.5 */}
      <button onClick={install}>Install</button><button onClick={reject}>Reject</button>
    </Modal>
  );
}
```

Novice always sees it; Promoted-within-bounds gets the auto-accept toast (the L4 outcome from C.3.1's `tier_gate` — `TierReviewRequired` vs auto-accept is decided backend-side, the renderer just renders the outcome).

#### E.3.5 `share_provenance` badge + the event-driven phase reducer

```ts
// src/state/ reducer — NEW branches over the EXISTING event stream
// (no new Tauri command; Stage B's artifact_hash_mismatch + C's import phases).
case 'artifact_hash_mismatch':
  draft.import[e.artifact_ref] = { ...draft.import[e.artifact_ref], phase:'error',
    error:`hash mismatch — expected ${e.expected}, got ${e.actual}` };
  draft.blockedArtifacts.add(e.artifact_ref);     // block use until reinstall/remove
  break;
case 'import_phase':                              // C pipeline progress
  draft.import[e.ref] = { ...draft.import[e.ref], phase:e.phase, report:e.report, provenance:e.provenance };
  break;
```

`ShareProvenanceBadge` renders `exported_by`, `for_runtime_class`, `for_os`, and `rebake_changes: []` as "no rebaking (runtime-to-runtime)" — read-only trust signal, not a gate in v0.1. The hash-mismatch path blocks the artifact's use and surfaces a Reinstall / Remove prompt; no new backend (wiring over Stage B's event + Stage C's `import_artifact`).

#### E.3.6 Tests

Playwright (Vite dev server, `@tauri-apps/api` module-mocked — M02 Stage E pattern; full desktop-shell E2E stays the documented gotcha #23 carry-forward):

```ts
test('paste URL → review → install → palette', async ({ page }) => {
  mockInvoke('import_artifact', urlScenario);                  // pinned signature (E.3.2)
  await page.fill('[data-t=url]', RAW_URL); await page.click('[data-t=import]');
  await expect(page.locator('[data-t=cap-disclosure]')).toBeVisible();   // M05 component
  await expect(page.locator('[data-t=share-provenance]')).toContainText('no rebaking');
  await page.click('[data-t=install]');
  await expect(page.locator('[data-t=palette]')).toContainText('fs-test');
});
test('artifact_hash_mismatch blocks use + prompts reinstall', async ({ page }) => {
  emitEvent({ type:'artifact_hash_mismatch', artifact_ref:'fs@1', expected:'sha256-a', actual:'sha256-b' });
  await expect(page.locator('[data-t=reinstall-prompt]')).toBeVisible();
});
```

Vitest unit for the panel/review/badge + the two reducer branches. Renderer ≥80 (vitest).

### E.4 Tests

Playwright (Vite dev server, `@tauri-apps/api` mocked per the M02 Stage E pattern): paste a raw URL → review screen renders capability disclosure + share_provenance → Install → artifact appears in palette; hash-mismatch event → reinstall prompt blocks. Vitest unit for the panel/review/badge. Acceptance: renderer ≥80 (vitest); Playwright green; `<wire_signature_audit>` + `shape=` claims match the shipped Stage C wire; v1.7 two-commit (renderer default).

### E.5 CLI Prompt

```xml
<work_stage_prompt id="M07.E">
  <context>
    M07 Stage E — the Builder Import panel renderer: paste-GitHub-raw-URL
    dialog + local-file picker + tier-gate review screen (capability
    disclosure + L3 report) + share_provenance trust-signal block +
    hash-mismatch reinstall prompt. Wiring over the Stage C
    import_artifact command + the Stage B artifact_hash_mismatch event.
    NO new backend. v1.8 wire-signature + shape audits pin the actual
    wire before pseudocode (the M06.E drift lesson).
  </context>
  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — wire_signature_audit + phase_doc_inventory_audit shape=)</file>
    <file>docs/build-prompts/M07-registry-import.md (Background, Key constraints, Stage E E.1–E.4)</file>
    <file>src-tauri/src/commands.rs (the SHIPPED import_artifact signature from Stage C — pin params, do not assume)</file>
    <file>src/state/ (the SHIPPED import state slot type — pin shape)</file>
    <file>the M05 capability-disclosure renderer component (reuse) + the M06.E renderer pattern (MCPNode/Settings) for the panel idiom</file>
    <file>docs/build-prompts/retrospectives/M07.C-retrospective.md + M07.D1-retrospective.md + M07.D2-retrospective.md + RETROSPECTIVE-TEMPLATE.md + gotchas #23 (Playwright cannot drive Tauri window)</file>
  </read_first>
  <read_prior_stages>
    <stage id="M07.C" retro="docs/build-prompts/retrospectives/M07.C-retrospective.md"/>
    <stage id="M07.D1" retro="docs/build-prompts/retrospectives/M07.D1-retrospective.md"/>
    <stage id="M07.D2" retro="docs/build-prompts/retrospectives/M07.D2-retrospective.md"/>
  </read_prior_stages>
  <deliverable ref="docs/build-prompts/M07-registry-import.md" section="E.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <tdd_discipline strict="true">
    <red_phase>Standalone `test(M07.E): …` — Playwright (paste URL→review→install→palette; hash-mismatch→prompt) + vitest unit. Right-reason red (component absent). Surface.</red_phase>
    <green_phase>Impl the panel; no test-file edits in impl commit; '**/tests/**' diff EMPTY. No Co-Authored-By.</green_phase>
  </tdd_discipline>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M07-registry-import.md" section="E.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M07-registry-import.md" section="Key constraints"/>
  <gates milestone="M07"/>
  <coverage_gate><gate scope="renderer" target_lines="80" ignore_filename_regex="generated"/></coverage_gate>
  <self_correction_budget>3</self_correction_budget>
  <wire_signature_audit>
    <wrapper ipc_command="import_artifact" actual_params="PIN from the Stage C-shipped src-tauri/src/commands.rs signature — do NOT assume" phase_doc_assumed="(author fills from the shipped signature at authoring time)"/>
  </wire_signature_audit>
  <phase_doc_inventory_audit>
    <claim type="store_slot" path="src/state/" symbol="(import slot)" shape="PIN from the Stage C/D2-shipped TS type" verified="true"/>
    <claim type="ipc_command" path="src-tauri/src/commands.rs" symbol="import_artifact" verified="true"/>
  </phase_doc_inventory_audit>
  <gotchas>
    <trap>#23 — Playwright drives the Vite dev server with @tauri-apps/api mocked; it cannot drive the Tauri window. Full desktop-shell E2E stays the documented carry-forward.</trap>
    <trap>M06.E drift — pin the actual import_artifact params + state-slot shape from the SHIPPED Stage C/D1/D2 code before writing pseudocode (the whole point of the v1.8 audits).</trap>
    <trap>Reuse the M05 capability-disclosure component; do not rebuild plain-English disclosure.</trap>
  </gotchas>
  <pre_flight_check>
    <check name="branch">HEAD == claude/m07-registry-import; M07.C + M07.D1 + M07.D2 impl commits present (the wire E pins must already be shipped)</check>
  </pre_flight_check>
  <runtime_environment os="windows"/>
  <time_box hours="5-6"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the actual import_artifact params + state-slot shape pinned (from shipped code, not assumed) and that the panel matches them. Confirm M05 disclosure-component reuse. Playwright/vitest results.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M07-registry-import.md" section="E.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state; strict-TDD '**/tests/**' EMPTY proof</item>
    <item>wire_signature_audit + shape= reconciliation against the shipped Stage C/D1/D2 wire</item>
    <item>Playwright + vitest results; renderer ≥80</item>
    <item>M07.E retrospective [END]; explicit "Stage M07.E is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### E.6 Commit Message

```
feat(renderer): M07 Stage E — Builder Import panel (URL/file → tier-gate review → install)

Import panel: paste-GitHub-raw-URL dialog + local-file picker +
tier-gate review (M05 capability-disclosure reuse + L3 report) +
share_provenance trust-signal block + artifact_hash_mismatch
reinstall prompt. Wiring over the Stage C import_artifact command +
Stage B event; no new backend. v1.8 wire_signature_audit +
phase_doc_inventory_audit shape= pinned to the SHIPPED wire (the
M06.E drift lesson). Playwright + vitest; renderer ≥80. Strict
v1.8 two-commit TDD: '**/tests/**' diff EMPTY.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage V — Verifier (four passes; mandatory integration smoke)

### V.1 Problem Statement

First V under v1.8. The M06.V Dec-7 codification (now in `STAGE-V-VERIFIER-PROMPT-TEMPLATE.md`) makes the `--features integration` reference-MCP-server smoke **mandatory** in the Behavior pass — a real dispatch path exists by Stage D2, so the mock-only escape is closed. The Dec-6 standing rule applies: a primitive delivered + tested but with the production driver absent, root-caused to an accepted ADR's named carry-forward → 🟡-with-mandatory-enumeration. V also consumes the phase doc's `<wire_trace_vs_adr_reconcile>` + `<construction_reachability_check>` + `<scope_change>` blocks (bias-guarded read-list).

### V.2 Scope to verify

The M07 deliverables A, B, C, D1, D2, E against spec Phase 7 + MVP §M7 acceptance + ADR-0010/0011/0014: skills.lock integrity + reproducibility; the import pipeline (URL/file → validate → L3 → tier → install → lock; §15c; share_provenance); the ADR-0011 (a)–(c) construction discharge (D1: concrete `McpDispatcher` + `ConnectionResolver`) + (d) loop discharge (D2: assembled loop dispatches through the concrete `McpDispatcher`; token_usage>0; ConnectionResolver re-resolution); the Import panel.

### V.5 CLI Prompt

```xml
<verifier_stage_prompt id="M07.V">
  <context>
    M07 Stage V — fresh-context four-pass contract-fidelity verifier,
    first under v1.8. Bias guard: this session deliberately does NOT
    read prior M07 retrospectives / summary / gap-analysis. It DOES
    read the phase doc's <construction_reachability_check> /
    <wire_trace_vs_adr_reconcile> / <scope_change> blocks (v1.8
    STAGE-V template). The Behavior pass MUST run the --features
    integration reference-MCP-server smoke (M06.V Dec-7 codified — a
    real dispatch path exists by D; mock-only is no longer acceptable).
    Apply the Dec-6 standing rule for any delivered/driver-absent/
    accepted-ADR-carry-forward finding.
  </context>
  <read_first>
    <file>STAGE-PROMPT-PROTOCOL.md §14 (verifier schema) + §8 v1.8 codified standing rules</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (four passes + bias guard)</file>
    <file>docs/adr/0010 + 0011 + 0014 (the architecture/carry-forward V reconciles deliverables against)</file>
    <file>docs/build-prompts/M07-registry-import.md (Background, all stages A.1–E.4 incl. D1.1–D1.4 + D2.1–D2.4, V.1/V.2, AND every `<construction_reachability_check>` / `<wire_trace_vs_adr_reconcile>` / `<scope_change>` block — NOT any retrospective references)</file>
    <file>agent-runtime-spec.md §2152–2211 (Phase 7) + §5a + §15c + §2c.3</file>
    <file>docs/MVP-v0.1.md §M7 (acceptance criteria)</file>
    <file>docs/style.md + docs/gotchas.md + docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md + docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md (v1.8 — the Dec-6/Dec-7 standing rules) + docs/tech-debt.md</file>
  </read_first>
  <scope_to_verify ref="docs/build-prompts/M07-registry-import.md" section="V.2 Scope to verify"/>
  <verification_passes>
    <pass name="inventory">Every MVP §M7 acceptance item + ADR-0010/0011/0014 deliverable has a shipped artifact; read `<construction_reachability_check>` / `<scope_change>` — authorized carry-forwards are NOT gaps.</pass>
    <pass name="wire">Trace skills.lock write→verify-on-load→hash-mismatch-block; import URL/file→validate→L3→tier→install→lock; the ADR-0011 concrete-dispatch path end-to-end. Reconcile every Wire trace against ADR-0010/0011 (the §5a re-resolution trace MUST be against McpDispatcher, not McpClient — Dec-6).</pass>
    <pass name="behavior">MANDATORY `--features integration` reference-MCP-server smoke (M06.V Dec-7): a real MCP tool dispatches through the assembled loop; token_usage>0; hash-mismatch blocks load; §15c compatible_os mismatch blocks. Not mock-only.</pass>
    <pass name="multi_call_invariants">skills.lock reproducible byte-identical across two installs/machines; ConnectionResolver re-resolves idempotently on reconnect; double-import is safe.</pass>
  </verification_passes>
  <findings_format ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md" section="Findings"/>
  <merge_gate red_blocks="true" dfix_iteration_cap="2" waiver_path="docs/adr/NNNN-waiver-M07-finding-N.md"/>
  <gates milestone="M07"/>
  <self_correction_budget>3</self_correction_budget>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md">
    <special_log>First V under v1.8. Record: did the mandatory --features integration smoke run (N/M, NOT 0/0 — the M06.V Dec-7 endpoint)? Did the v1.8 `<construction_reachability_check>` / `<wire_trace_vs_adr_reconcile>` read-list consumption work (any 🔴 that was actually a documented construction carry-forward)? Apply + record the Dec-6 standing rule for any delivered/driver-absent finding. Protocol-calibration observations for v1.9.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M07-registry-import.md" section="V.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls M07.*-retrospective.md)</item>
    <item>findings by severity; per-pass counts</item>
    <item>the mandatory integration-smoke result (N/M, not 0/0)</item>
    <item>ADR-0011 (a)–(d) discharge confirmation (assembled dispatch path verified) + Dec-6 rulings if any</item>
    <item>VERIFIER-RETROSPECTIVE [END] + merge recommendation: "Proceed to G" | "Open D.fix for 🔴 &lt;cite&gt;" | "Re-tier"</item>
    <item>explicit: "Stage M07.V is ready. I will not commit until you approve."</item>
  </approval_surface>
</verifier_stage_prompt>
```

### V.6 Commit Message

```
verify(M07): in-band V — four passes + mandatory --features integration smoke (v1.8)

First V under v1.8: the reference-MCP-server integration smoke ran
(not 0/0 — M06.V Dec-7 endpoint). Findings: <N🔴 N🟡 N🟢>.
ADR-0011 (a)-(d) discharge verified via the assembled dispatch
path. Dec-6 standing rule applied to <…>. Disposition: <Proceed
to G | D.fix>.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage G — Closeout

### G.1 Problem Statement

Aggregate M07 across A, B, C, D1, D2, E + V; append the immutable M07 gap-analysis entry (six sections + `gotchas_graduation` A,B,C,D1,D2,E); run the v1.6 `<simplify_pass>` against `M07.A..HEAD`; perform the v1.8 `<coverage_policy_reconciliation>` (new `skills_lock` ≥95 gate from B, the `src.import.fetch.rs` exclusion from C, the runtime-mcp `transport/mod.rs` baseline from D1 — all four mirrors synced + the `docs/coverage-policy.md` §B/§C entries appended); draft the M07 PR.

### G.5 CLI Prompt

```xml
<closeout_stage_prompt id="M07.G">
  <context>
    M07 Stage G closeout — milestone summary + immutable gap-analysis
    entry + v1.6 simplify_pass + the v1.8 coverage_policy_reconciliation
    (M07 added/changed gates: skills_lock ≥95 [B], src.import.fetch.rs
    exclusion [C], transport/mod.rs baseline [D]). Draft the M07 PR;
    do not open until asked.
  </context>
  <cumulative_reads>
    <codebase>entire shipped codebase through M07 (cumulative across M01–M06.6 merges)</codebase>
    <spec>agent-runtime-spec.md (focus §2152–2211 Phase 7 + §5a + §15c + §2c.3 + §8.security)</spec>
    <gap_analysis>docs/gap-analysis.md (ALL prior entries — M01..M06)</gap_analysis>
    <retrospectives>docs/build-prompts/retrospectives/M07.*-retrospective.md (A,B,C,D,E,V — closeout reads these; V did not)</retrospectives>
    <summary>docs/build-prompts/retrospectives/M07-summary.md (authored this stage)</summary>
    <tech_debt>docs/tech-debt.md (cumulative; M07.V 🟢 present)</tech_debt>
    <coverage_policy>docs/coverage-policy.md (§A/§B/§C — the four-mirror reconciliation target)</coverage_policy>
  </cumulative_reads>
  <read_first>
    <file>CLAUDE.md (§5/§6 coverage source-of-truth + §20 Gap Analysis)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (§8 closeout tags; v1.6 simplify_pass; v1.8 coverage_policy_reconciliation)</file>
    <file>docs/build-prompts/M07-registry-import.md (entire phase doc)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md + SUMMARY-TEMPLATE.md (incl. the v1.8 coverage-policy reconciliation check) + docs/gap-analysis.md template + docs/adr/0008</file>
  </read_first>
  <scope_locks ref="docs/build-prompts/M07-registry-import.md" section="Key constraints"/>
  <gates milestone="M07"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>M07 closed the largest open architectural carry-forward (ADR-0011 (a)–(d)) + shipped the registry-import feature + skills.lock primitive + first v1.8 V. Aggregate per-primitive coverage. Cite the simplify_pass outcome + the coverage_policy_reconciliation (which gates changed, all four mirrors synced, §B/§C appended).</special_log>
  </retrospective_requirements>
  <deliverables>
    <milestone_summary>docs/build-prompts/retrospectives/M07-summary.md (per SUMMARY-TEMPLATE.md; aggregates A, B, C, D1, D2, E + V; scores axes; verdict)</milestone_summary>
    <gap_analysis_entry>docs/gap-analysis.md (append immutable M07 entry; six sections; gotchas_graduation A,B,C,D1,D2,E; ADR-0011 (a)–(d) discharge cited in Adherence-to-spec; the post-M07 IRL re-confirm of 🔴-1/🔴-2 + token_usage-now-closed recorded in Carry-forward)</gap_analysis_entry>
    <coverage_policy_reconciliation>M07 changed gates: skills_lock ≥95 (B), src.import.fetch.rs exclusion (C), transport/mod.rs baseline (D). Append docs/coverage-policy.md §C M07 entry + §B baselines; verify CLAUDE.md §5 categories + §6 commands + codecov.yml byte-consistent (the v1.8 four-mirror rule). If any drift, fix in this commit.</coverage_policy_reconciliation>
    <simplify_pass>
      <invoke skill="simplify" against="milestone cumulative diff (M07.A..HEAD)"/>
      <surface kind="refactor_proposals" examples="import-pipeline stage duplication / dispatcher construction parallel surfaces / dead arms post-CQ-2 / premature abstractions"/>
      <approval_required>true</approval_required>
      <commit_on_approval>focused refactor commit on the same branch before PR opens</commit_on_approval>
      <defer_unapproved_to>docs/tech-debt.md (ADR-0008 🟢 ledger)</defer_unapproved_to>
    </simplify_pass>
    <pr_description>draft only; do not open PR until explicitly asked</pr_description>
  </deliverables>
  <gap_analysis_requirements ref="CLAUDE.md" section="20. Gap Analysis Protocol">
    <gotchas_graduation>
      <stage_review id="A"/>
      <stage_review id="B"/>
      <stage_review id="C"/>
      <stage_review id="D1"/>
      <stage_review id="D2"/>
      <stage_review id="E"/>
    </gotchas_graduation>
    <special_check>V→closeout handoff: 🟡→Carry-forward, 🟢→tech-debt, 🔴→D.fix-or-waiver before closeout. ADR-0011 (a)–(c) discharge (D1) + (d) discharge (D2) status explicitly cited in Adherence-to-spec. The M06.5 token_usage finding: record as RESOLVED at D2 (with the assembled-regression cite) in Carry-forward.</special_check>
    <special_check>Run the v1.6 simplify_pass against M07.A..HEAD; apply approved subset before PR; defer rest to tech-debt. Run the v1.8 coverage_policy_reconciliation; the SUMMARY-TEMPLATE coverage check must be ticked.</special_check>
  </gap_analysis_requirements>
  <append_only_verification>
    <local_check>prior content of docs/gap-analysis.md must be a literal prefix of HEAD before commit (the M07 entry only appends)</local_check>
    <ci_check name="gap-analysis-append-only">fails if any prior line is modified</ci_check>
  </append_only_verification>
  <three_artifact_review>
    <artifact>code diff (cumulative across M07 stages A, B, C, D1, D2, E + V findings absorbed + any Simplify-pass refactor commit)</artifact>
    <artifact>per-stage retrospectives (M07.A through M07.E) + Stage V retro + M07 summary</artifact>
    <artifact>new gap-analysis M07 entry — flagged "IMMUTABLE once committed" (ADR-0011 discharge + token_usage-resolved cited)</artifact>
    <pushback_blocks_pr>true</pushback_blocks_pr>
  </three_artifact_review>
  <self_correction_budget>3</self_correction_budget>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M07-registry-import.md" section="G.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls M07.*-retrospective.md + M07-summary.md)</item>
    <item>the immutable gap-analysis M07 entry (six sections; ADR-0011 discharge + token_usage-resolved cited)</item>
    <item>coverage_policy_reconciliation: gates changed, four mirrors synced, §B/§C appended (SUMMARY check ticked)</item>
    <item>simplify_pass proposals + approved/deferred split</item>
    <item>three-artifact review bundle (code diff + retros/summary + gap-analysis) + draft PR description</item>
    <item>explicit: "Stage M07.G is ready. I will not commit until you approve."</item>
  </approval_surface>
</closeout_stage_prompt>
```

### G.6 Commit Message

```
docs(closeout): M07 — gap-analysis + summary + simplify_pass + coverage-policy reconciliation

M07 summary (A, B, C, D1, D2, E + V) + immutable gap-analysis entry (six sections;
gotchas_graduation A,B,C,D1,D2,E; ADR-0011 (a)–(d) discharge + the
M06.5 token_usage finding RESOLVED-at-D2 cited in Adherence/Carry-forward;
🔴-1/🔴-2 real-app re-confirm carried to the post-M07 IRL pass).
v1.6 simplify_pass against M07.A..HEAD (approved subset applied).
v1.8 coverage_policy_reconciliation: skills_lock ≥95 (B) +
src.import.fetch.rs exclusion (C) + transport/mod.rs baseline (D)
synced across docs/coverage-policy.md §B/§C + CLAUDE.md §5/§6 +
codecov.yml. PR drafted (not opened).

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```
