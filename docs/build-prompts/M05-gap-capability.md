# M05 — Gap Detection + Capability Enforcement

> **Protocol version:** v1.5 (per `STAGE-PROMPT-PROTOCOL.md` v1.5 — first milestone authored with Stage V Verifier section per ADR-0008).
> **Branch:** `claude/m05-gap-capability-phase-doc`
> **Builds on:** M04 (Plan / Verify / HITL / Budget) merged at `2765d78`; M04.V retrospective merged at `4162085`; M04.V findings cleanup merged at `48d6050`.
> **MVP scope:** `docs/MVP-v0.1.md` §M5.

---

## Background and Design Decision

### What this milestone produces

M05 lights up the **§4b gap detection + §8.security capability enforcement** primitives that the spec names as the central runtime safety surface. Until M05 the runtime executes whatever framework JSON declares without checking that referenced skills/tools/MCP/sub-agents exist OR that an agent's invocation is allowed by its capability declaration; M05 closes both gaps. End-to-end:

- **Loader** parses a framework JSON (or schema-derived shape), walks it, and emits a **gap event** for every referenced primitive that doesn't resolve (`tool_missing`, `skill_missing`, `mcp_missing`, `agent_missing` — the last is new). The session **suspends** on first gap; the user fixes the framework (or declines via HITL) and the session resumes from the gap point.
- **`request_capability` meta-tool** gives an agent a structured way to ASK for a capability mid-session. It emits a gap event the same way the loader does; the renderer + HITL pipeline route it identically.
- **Capability enforcer** (the L1+L2a layers of §8.security) sits in front of every tool dispatch + sub-agent spawn. It reads the calling agent's capability declaration, looks up the requested action's declared shape, and rejects mismatches with a `capability_violation` event + HITL prompt. L3 sandbox (out-of-process validation for generated artifacts) is its own stage. L4 tier system + L5 provenance/audit are subsequent stages.
- **Tier system** caps which capabilities can be granted at all per the user's persona (Novice = curated subset; Promoted = full surface). The spec scopes v0.1 to two tiers; Full tier ships post-v0.1.
- **Audit log** appends every gap, every capability_grant, every capability_violation to `skills.audit.jsonl` per spec §13.5 dev-logging.
- **UI** surfaces the runtime state to the human: `GapPanel` (right-side list of unresolved gaps with resolution affordances), `CapabilityBadge` (per-node tier indicator), capability-violation modal (the M04.E HITL modal variant reused per ADR-0007).

### What's not in scope

- **L3 sandbox subprocess** for executing GENERATED artifacts at runtime (the §8.security L3 layer — its own stage C below). The capability enforcer (L1+L2a) handles the in-process check; L3 is the out-of-process boundary.
- **MCP server lifecycle** — that's M06. M05 only emits `mcp_missing` events when the framework references an MCP server that isn't installed; M06 actually launches MCP processes.
- **Framework editing UI** — the Builder Canvas is M08. M05 ships GapPanel as a passive surface that shows what's missing; resolution happens by editing the framework JSON file (manual) or via the `request_capability` meta-tool (programmatic).
- **Generators** that produce new tools/skills from spec descriptions — M09. M05's `request_capability` meta-tool emits an *event*; M09 wires that event to an artifact-generation flow.

### Why six work stages + V + closeout (with C split into C1 + C2)

The early structuring used eight stages (A1/A2 split plus separate stages for §4b detection / framework_loader / request_capability / severity matrix). Per M04.5 design-review push-back, splitting A1/A2 for "build hygiene + production wiring" is M04-specific pattern-matching — M05's first stage genuinely is one coherent work surface: wiring §4b detection + framework_loader + request_capability while absorbing M04.V carry-forwards. Single Stage A. The rest of M05 splits along §8.security's L-layers:

- **A** — §4b detection (framework_loader + request_capability + severity matrix) + new schemas (`gap.v1.json` for the enriched gap-event payload; `agent_missing` event variant; `request_capability` tool definition) + M04 carry-forward absorption + M04.V Decision 1 (TaskNode regression test) + M04.V Decision 2 (§4a `hook_*` vs `verify_*` reconciliation note)
- **B** — §8.security L1 + L2a capability enforcer (NEW safety primitive ≥95% coverage)
- **C1** — §8.security L3 sandbox crate plumbing + main-side IPC client + lifecycle (NEW safety primitive ≥95% on sandbox_ipc + plumbing files)
- **C2** — §8.security L3 cross-platform OS isolation: seccomp + landlock + Job Objects (NEW safety primitive ≥95% extending the runtime-sandbox gate to OS-isolation files)
- **D** — §8.security L4 tier system (Novice + Promoted only, per §0d)
- **E** — §8.security L5 provenance + `skills.audit.jsonl` audit log
- **F** — UI: `GapPanel` + capability-violation modal wiring + `CapabilityBadge`
- **V** — Stage V Verifier (per v1.5 protocol — first milestone shipped with Stage V from authoring; runs against this milestone's deliverables in fresh CLI session)
- **G** — Closeout (gap-analysis entry + parent-milestone summary)

Stages B, C1, C2 are all new safety primitives at ≥95%. Stage C is the only stage split into sub-stages (C1/C2) — and only because the L3 sandbox is two coherent surfaces with different dependencies, test harnesses, and failure modes: subprocess plumbing (C1) is cross-platform pure Rust + tokio; OS isolation (C2) requires per-platform native libraries (libseccomp-rs, landlock, winapi) + per-platform CI matrix. C1 ships the boundary without isolation; C2 layers isolation on. Stage A is NOT similarly split — A's surface is one coherent piece per the design-review push-back. The C1/C2 split is the exception, not a pattern to repeat.

### M04.V audit-driven absorbtions

Per the v1.5 protocol's gap-analysis → next-milestone carry-forward chain, two M04.V 🟡 findings absorb here:

- **🟡 #1 TaskNode empty-title regression test** — small (~10 min) Stage A task; mirrors PR #64's `reads_tokensTotal_for_visual_scale_not_tokensIn_plus_tokensOut` shape. New test in `tests/unit/nodes/TaskNode.test.tsx`.
- **🟡 #2 Spec §4a `hook_*` vs codebase `verify_*` reconciliation** — Stage A surfaces the question for maintainer adjudication. Decision is product-naming. Default recommendation: update spec §4a text to say `verify_*` (the code is internally consistent; renaming the events would cascade across schema + Rust + TS + tests for cosmetic gain). Captured in M05.A retrospective for maintainer review; no code change unless decision goes the other way.

The 4 🟢 findings from M04.V (TD-001..004 in `docs/tech-debt.md`) are forward-applicable but NOT absorbed into Stage A — they're handled in their natural-incorporation windows (TD-001 viewport pinning when Playwright suite expands; TD-002 read_signals/recover_session per-method tests at M06 IPC work; TD-003 respond_uncertainty parity once maintainer decides the contract; TD-004 already resolved).

### Key constraints

- **v0.1 single-session** per §0d. No multi-session capability grants; no cross-session audit-log queries.
- **Two tiers only** (Novice + Promoted) per §0d. Full tier surface is post-v0.1.
- **STANDARD mode hardcoded** per §0d. M05 doesn't ship mode-aware capability gates.
- **Anthropic-only** per §0d. No provider-specific capability shapes.
- **`fresh_context_per_task` loop policy** per §0d. Stage E audit log appends per task boundary, not per turn.
- **Safety-primitive coverage gates ≥95%** for the L1+L2a enforcer (Stage B), the L3 sandbox (Stage C), and the L4 tier system (Stage D). Per CLAUDE.md §5 + Codecov gates.
- **Schema-as-source-of-truth** per CLAUDE.md §14 — every new event variant lives in `schemas/event.v1.json`; Rust + TS bindings regenerate via `cargo xtask regenerate-types`.

---

## Document Structure

| Stage | Scope | Effort | Coverage gate |
|---|---|---|---|
| **A** | §4b detection wiring (framework_loader + request_capability) + 2 new event variants + M04.V carry-forwards | 5–7h | workspace ≥80% |
| **B** | §8.security L1 + L2a capability enforcer (new safety primitive) | 4–6h | per-crate ≥95% on `runtime-main::capability` |
| **C1** | §8.security L3 sandbox crate plumbing + main-side IPC client + lifecycle (new safety primitive) | 4–6h | per-module ≥95% on `runtime-main::sandbox_ipc` + plumbing files in `runtime-sandbox` |
| **C2** | §8.security L3 cross-platform OS isolation (seccomp + landlock + Job Objects) (new safety primitive) | 4–6h | per-crate ≥95% on `runtime-sandbox` extended to cover OS-isolation files |
| **D** | §8.security L4 tier system (Novice + Promoted only) | 3–4h | per-module ≥95% on tier evaluator |
| **E** | §8.security L5 provenance + `skills.audit.jsonl` audit log | 3–4h | workspace ≥80% |
| **F** | UI: `GapPanel` + capability-violation modal + `CapabilityBadge` | 3–5h | renderer ≥80% (vitest) |
| **V** | Verifier — four-pass contract-fidelity check against the milestone (per ADR-0008) | 2–4h | N/A (verification stage; no code shipped) |
| **G** | Closeout — gap-analysis entry + M05 summary | 1–2h | N/A |

Total: ~30–45 hours of code execution across stages A–F; ~3–6 hours for V + closeout. M01–M04 calibrated at ~13–20h per milestone (smaller scope each); M05 is genuinely the largest yet because of stages B + C being net-new safety primitives.

---

## Implementation Workflow

Apply these rules consistently across every stage. Don't restate per-stage; they're the project-wide protocol (CLAUDE.md §3, §4, §5, §6, §8, §16, §19).

1. **Read first.** Each stage's `<read_first>` declares what to read before code. Read those files. Stage B+ also reads the prior stage's retrospective `[END] Decisions` section and applies decisions before writing code (CLAUDE.md §19 rule 1).
2. **TDD.** Write a failing behavior test FIRST. Run it; confirm it fails for the right reason. Then implement. Then refactor. Each red-green-refactor cycle is 5–15 min; if longer, the test is too big — split it (CLAUDE.md §5).
3. **Schema-as-source-of-truth.** Every new event variant or shared type goes in `schemas/*.v1.json`. Don't hand-author Rust types in `runtime-core` or TS types in `src/types/`. After schema edits, run `cargo xtask regenerate-types` and commit the generated changes alongside the schema change (CLAUDE.md §14).
4. **Safety-primitive ≥95% coverage** — Stages B, C, D each add new safety primitives. Per-module coverage gates apply. Use the per-package `cargo llvm-cov` exclusion pattern from M01 (skip OS-call wrappers; baseline the testable seams) per `CLAUDE.md` §5 quality gates.
5. **No `unsafe` outside `crates/runtime-sandbox/`.** Stage C's sandbox subprocess needs `unsafe` for seccomp / landlock / Job Objects integration; every block requires a `// SAFETY:` comment. All other crates: `forbid(unsafe_code)`.
6. **Don't loosen capabilities to make a test pass.** If a test fails because a capability is narrower than the test expected, the test is wrong (or the capability is correctly narrow). Per CLAUDE.md §10.
7. **Pre-flight checks every stage.** Stage prompts include `<pre_flight_check>` (env vars, branch state, prior commits), `<phase_doc_inventory_audit>` (every X.2 file path verified against `git ls-files` before code work), `<architecture_check>` where HOW-claims are load-bearing (Stage B + C). v1.4 + v1.5 tags.
8. **Surface, don't commit, until approved.** Per CLAUDE.md §4 Hard Rule 1. Each stage's `<approval_surface>` declares what the agent shows the human; the human approves before commit lands.
9. **Cross-machine state on every surface.** Every stage end MUST surface `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M05.*-retrospective.md` so any downstream session has real state, not origin's partial view (CLAUDE.md §19 rule 7).
10. **Stage V runs in fresh CLI session.** The user clears the session and pastes the V prompt fresh (the bias guard). V deliberately doesn't read prior retrospectives — the discipline is structurally enforced by the validator's bias-guard rule (`STAGE-PROMPT-PROTOCOL.md` §14).

---

## Pre-existing legacy file inventory

Grep-verified at authoring time. Files M05 stages CONSUME or REFERENCE (not create); shape claims about each are factual as of `origin/main` `48d6050`.

| File | Purpose | M05 stage that touches it |
|---|---|---|
| `crates/runtime-core/src/generated/framework.rs` | typify-generated Rust types from `schemas/framework.v1.json` (Framework, Skill, Tool, MCPServer, AgentDecl) | A (read; not edited — schema is source) |
| `schemas/framework.v1.json` | Framework JSON schema (skill/tool/mcp/agent declarations) | A (read; consumed by framework_loader) |
| `schemas/event.v1.json` | All event variants (M01–M04 events) | A (edited — add `agent_missing` + enriched `gap`-class fields) |
| `crates/runtime-core/src/generated/event.rs` | typify-generated Rust types from `schemas/event.v1.json` | A (regenerated after schema edit; not hand-edited) |
| `src/types/agent_event.ts` | `json-schema-to-typescript`-generated TS types from `schemas/event.v1.json` | A (regenerated; not hand-edited) |
| `src/lib/graphStore.ts` | Zustand store + `applyEvent` reducer; lines 1156+ have no-op cases for `gap_resolved`, `capability_violation`, `capability_grant`, `skill_missing`, `tool_missing` (M04-era prep) | A (no-op cases lit up; new GapNode emissions; renamed/extended for new event payloads) |
| `crates/runtime-main/src/sdk/` | SDK module — agent loop, tool dispatch, message history | A (extended with framework_loader hookup + request_capability meta-tool) + B (capability enforcer wraps tool dispatch) |
| `crates/runtime-main/src/hitl/` | HitlSeam + policy (9-trigger lookup) | A (on_gap trigger emits HITL prompt on first gap); B (on_capability_violation trigger emits HITL on enforcer reject) |
| `crates/runtime-core/src/signal.rs` | Signal enum + ContextType — note the M02/M03 carry-forward "ContextType reconcile with spec §2b" is STILL DEFERRED; absorb in A | A (reconcile per M02/M03 carry-forward) |
| `crates/runtime-drone/src/db.rs` | Drone-side SQLite schema | E (add `audit_log` table or equivalent for skills.audit.jsonl persistence — TBD between flat file and SQLite) |
| `crates/runtime-sandbox/` | M01-scaffolded skeleton crate (currently empty — `lib.rs` is `pub fn placeholder() {}` or similar) | C (lit up with seccomp/landlock/Job Objects subprocess) |
| `src/components/nodes/GapNode.tsx` | M03-shipped 11th node type; declared in graphStore as `gap` but no event emits it in M04 | A (gap events emit GapNode; F adds GapPanel rendering for the same data) |
| `src/components/HITLModal.tsx` | M04.E modal HITL UI variant | F (reused for capability_violation modal per ADR-0007) |
| `docs/MVP-v0.1.md` §M5 | M05 acceptance criteria | A (read; verifier later checks against it) |
| `agent-runtime-spec.md` §4b, §8.security L1–L5 | Spec sections this milestone implements | All stages (consult continuously) |
| `examples/aria/framework.json` | Reference framework — exercises all primitives | A (framework_loader smoke test); V Behavior pass (load + verify no gaps for a valid framework) |
| `examples/ralph/framework.json` | Sibling reference framework | V Behavior pass (cross-framework verification) |

---

## Stage A — §4b Detection wiring (framework_loader + request_capability + new schemas + M04 carry-forwards)

### A.1 Problem Statement

Wire §4b gap detection into the runtime. The §0a Capability Matrix names four gap primitives (skill / tool / mcp / agent); v0.1 declared variants for three (`skill_missing`, `tool_missing`, `mcp_missing — already in schema`) plus `gap_resolved` and `capability_violation`/`capability_grant`. M05.A adds the fourth (`agent_missing`), enriches all four with the severity + suggestion payload the spec §4b describes, and wires a `framework_loader` module that walks a framework JSON pre-session and emits a gap event for every unresolved reference. Plus the `request_capability` meta-tool — an agent's structured way to ask for a capability mid-session. Plus absorbs M04.V 🟡 carry-forwards.

Concrete deliverables:
1. `schemas/event.v1.json` — new `agent_missing` event variant; enrich `skill_missing` + `tool_missing` + `mcp_missing` + new `agent_missing` with `severity`, `suggested_action` per spec §4b severity matrix; regenerate Rust + TS bindings.
2. `crates/runtime-main/src/framework_loader/` — new module. Parses a framework JSON (uses existing `runtime_core::generated::framework::*` types), walks declared skills/tools/mcps/agents, emits gap events for unresolved references. Returns `Ok(())` if all references resolve; emits gap events + returns `Err(FrameworkLoadError::Gaps)` if any unresolved.
3. `crates/runtime-main/src/sdk/request_capability.rs` — new module. The `request_capability` meta-tool implementation. Agent invokes it with `{ kind: "tool" | "skill" | "mcp" | "agent", name: string, justification: string }`; the SDK emits the appropriate `*_missing` event with `severity: "requested"`; the HITL pipeline (M04.E `on_gap` trigger) prompts the user; user resolves; SDK proceeds.
4. `crates/runtime-core/src/signal.rs::ContextType` — reconcile with spec §2b per M02/M03 carry-forward. Currently this enum has variants that don't perfectly match the spec's `context_type` taxonomy; resolve drift.
5. `src/lib/graphStore.ts` — replace no-op cases for `tool_missing`, `skill_missing`, `mcp_missing`, `gap_resolved` with real `applyEvent` branches that mount/dismiss GapNodes per gap event; add `agent_missing` case.
6. `tests/unit/nodes/TaskNode.test.tsx` — add the M04.V Decision 1 regression test (`renders_task_id_prefix_fallback_when_title_is_empty`).
7. Stage A retrospective surface includes the M04.V Decision 2 maintainer-decision question (spec §4a `hook_*` vs codebase `verify_*`).

Not in this stage:
- The capability enforcer (Stage B's L1+L2a)
- The sandbox subprocess (Stage C's L3)
- The tier evaluator (Stage D's L4)
- The audit-log writer (Stage E's L5)
- The GapPanel renderer + CapabilityBadge (Stage F)

### A.2 Files to Change

| File | Status | Change |
|---|---|---|
| `schemas/event.v1.json` | exists | Edit: add `agent_missing` variant; enrich the four `*_missing` variants with `severity` + `suggested_action`; add `requested_via` discriminator (`"loader"` or `"request_capability"`) |
| `crates/runtime-core/src/generated/event.rs` | exists | Regenerate via `cargo xtask regenerate-types` |
| `src/types/agent_event.ts` | exists | Regenerate via the TS codegen step |
| `crates/runtime-main/src/framework_loader/mod.rs` | **new** | Loader struct, `load_and_validate` method, error type, gap-emission via `WriteSignal` |
| `crates/runtime-main/src/framework_loader/walker.rs` | **new** | Pure-function walker: framework + (declared_skills, declared_tools, declared_mcps, declared_agents) → Vec<Gap> |
| `crates/runtime-main/src/framework_loader/error.rs` | **new** | FrameworkLoadError enum (Io, Json, GapsFound{gaps: Vec<Gap>}) |
| `crates/runtime-main/src/sdk/request_capability.rs` | **new** | Meta-tool wrapper: builds the appropriate `*_missing` event with `requested_via: "request_capability"`; routes through SDK emitter |
| `crates/runtime-main/src/sdk/mod.rs` | exists | Edit: expose `request_capability` module; wire it into the agent loop's tool-dispatch surface as a built-in meta-tool |
| `crates/runtime-main/src/lib.rs` | exists | Edit: `pub mod framework_loader;` |
| `crates/runtime-core/src/signal.rs` | exists | Edit: reconcile `ContextType` with spec §2b per M02/M03 carry-forward |
| `src/lib/graphStore.ts` | exists | Edit: real applyEvent branches for `tool_missing`/`skill_missing`/`mcp_missing`/`agent_missing`/`gap_resolved` (currently no-op) |
| `src/components/nodes/GapNode.tsx` | exists | Edit: render severity + suggested_action + agent_id from the enriched payload |
| `tests/unit/nodes/TaskNode.test.tsx` | exists | Add: `renders_task_id_prefix_fallback_when_title_is_empty` test (M04.V Decision 1) |
| `tests/unit/nodes/GapNode.test.tsx` | exists | Add: tests for the four kinds × the new payload fields |
| `tests/unit/lib/graphStore.test.ts` | exists | Add: applyEvent branches for the four gap event variants + idempotence |
| `crates/runtime-main/src/framework_loader/tests.rs` | **new** | Unit tests for the walker + loader |
| `crates/runtime-main/tests/framework_loader_smoke.rs` | **new** | Integration test against `examples/aria/framework.json` (loader produces zero gaps for the valid reference framework) |
| `crates/runtime-main/src/sdk/request_capability_tests.rs` | **new** OR inline in `request_capability.rs` `#[cfg(test)] mod tests` | Unit tests for the meta-tool dispatch + event-shape construction |
| `docs/build-prompts/retrospectives/M05.A-retrospective.md` | **new** | Stage A retrospective; per RETROSPECTIVE-TEMPLATE.md |
| `CHANGELOG.md` | exists | Edit: `[Unreleased]` notes M05.A — gap detection + framework_loader + request_capability + M04.V carry-forward absorption |

Effort budget: ~5–7 hours of code execution (largest piece is the framework_loader walker tests).

### A.3 Detailed Changes

#### A.3.1 Schema edit: enrich gap-event variants

`schemas/event.v1.json` currently declares `skill_missing`, `tool_missing`, `mcp_missing`, `gap_resolved`, `capability_violation`, `capability_grant` with minimal payload (most v0.1-stub-only). Stage A enriches the four `*_missing` variants and adds `agent_missing`:

```jsonc
// In the union of event variants in event.v1.json:
{
  "type": "object",
  "title": "ToolMissing",
  "required": ["type", "agent_id", "tool_name", "severity", "requested_via"],
  "properties": {
    "type": { "const": "tool_missing" },
    "agent_id": { "$ref": "common.v1.json#/$defs/AgentId" },
    "tool_name": { "$ref": "common.v1.json#/$defs/NonEmptyString" },
    "severity": { "$ref": "#/$defs/GapSeverity" },
    "suggested_action": { "type": "string", "minLength": 1 },
    "requested_via": { "$ref": "#/$defs/GapSource" }
  }
}
// + parallel shapes for skill_missing, mcp_missing, agent_missing
// + add to $defs:
"GapSeverity": { "type": "string", "enum": ["critical", "important", "advisory", "requested"] },
"GapSource":   { "type": "string", "enum": ["loader", "request_capability"] }
```

The `requested_via` discriminator distinguishes loader-driven gaps (session-start scan) from request_capability-driven gaps (mid-session). HITL routes both to `on_gap` trigger; severity drives default-action selection.

Per gotcha #43 (typify root-oneOf + validated string): `GapSeverity` and `GapSource` defined as `$defs` with title — typify generates named enums, not anonymous newtypes.

#### A.3.2 New module: `crates/runtime-main/src/framework_loader/`

The loader is a pure-function-plus-thin-IO wrapper:

```rust
// crates/runtime-main/src/framework_loader/mod.rs
pub mod walker;
pub mod error;

use error::FrameworkLoadError;
use runtime_core::generated::framework::Framework;
use std::path::Path;

/// Load a framework JSON from disk + walk it for gaps.
/// Returns Ok(()) only if every reference resolves; otherwise
/// emits one gap event per unresolved reference via the supplied
/// emitter and returns `FrameworkLoadError::GapsFound`.
pub async fn load_and_validate<E>(
    path: &Path,
    emitter: &E,
) -> Result<Framework, FrameworkLoadError>
where
    E: Emitter,
{
    let raw = tokio::fs::read_to_string(path).await?;
    let framework: Framework = serde_json::from_str(&raw)?;
    let gaps = walker::walk(&framework);
    if gaps.is_empty() {
        return Ok(framework);
    }
    for gap in &gaps {
        emitter.emit(gap.to_event()).await;
    }
    Err(FrameworkLoadError::GapsFound { count: gaps.len() })
}
```

The `walker` module is the testable seam — pure function over the parsed `Framework`. Emitter is injected to keep the unit tests free of IO.

#### A.3.3 New module: `crates/runtime-main/src/sdk/request_capability.rs`

The meta-tool sits inside the SDK's tool dispatch surface — when an agent invokes `request_capability`, the SDK doesn't route to an LLM tool call; it routes inline to this module:

```rust
// crates/runtime-main/src/sdk/request_capability.rs
pub async fn handle_request_capability(
    invocation: RequestCapabilityInvocation,
    emitter: &impl Emitter,
) -> Result<RequestCapabilityResult, RequestCapabilityError> {
    let event = match invocation.kind {
        Kind::Tool => AgentEvent::ToolMissing {
            agent_id: invocation.agent_id,
            tool_name: invocation.name,
            severity: Severity::Requested,
            suggested_action: invocation.justification,
            requested_via: Source::RequestCapability,
        },
        // ... parallel for Skill, Mcp, Agent
    };
    emitter.emit(event).await;
    // HITL pipeline picks this up via on_gap trigger (M04.E);
    // SDK awaits the resolution event.
    Ok(RequestCapabilityResult::Pending)
}
```

The meta-tool's contract is "emit event + return Pending"; the actual resolution flow lives in the HITL seam + the user's response. This keeps the meta-tool's testable seam narrow.

#### A.3.4 ContextType reconcile (M02/M03 carry-forward)

`crates/runtime-core/src/signal.rs::ContextType` was scaffolded in M02 with an enum whose variants don't perfectly match spec §2b's `context_type` taxonomy. The M02 retrospective + M03 retrospective + M04 gap-analysis all list this as 🟡 deferred. Resolution in Stage A:

1. Read spec §2b end-to-end. Enumerate the `context_type` values the spec names.
2. Compare to current enum variants. Identify drift.
3. Either (a) update the enum to match spec exactly (likely; safer), OR (b) update spec to match the enum's variants (only if maintainer signals).
4. Regenerate any typify-touched downstream code if the enum is schema-derived.
5. Surface the spec/code reconciliation in the Stage A retrospective alongside M04.V Decision 2.

#### A.3.5 graphStore.ts gap-event branches

Currently the four `*_missing` cases plus `gap_resolved` + `capability_violation` + `capability_grant` are no-op (`graphStore.ts:1156+`). Stage A lights them up:

- `tool_missing` / `skill_missing` / `mcp_missing` / `agent_missing` — mount a GapNode with the enriched payload. GapNode-id keyed by `${kind}:${name}:${agent_id}` so re-emissions for the same gap are idempotent.
- `gap_resolved` — dismiss the GapNode + append to the inspector's resolution-history list. The dismissal is keyed by the original gap's id.
- `capability_violation` / `capability_grant` — Stage B wires; Stage A leaves them as no-op-with-TODO comment.

#### A.3.6 M04.V Decision 1 absorbed: TaskNode regression test

Add to `tests/unit/nodes/TaskNode.test.tsx`:

```typescript
it('renders_task_id_prefix_fallback_when_title_is_empty', () => {
  renderTask({ ...baseData, title: '' });
  // M04 IRL LG-02: TaskNodes from `task_started` events showed as blank.
  // Fix lives at TaskNode.tsx:27 — `const displayTitle = title || \`task ${taskId.slice(0,8)}\``;
  // Caught by M04.V Finding #1.
  const root = screen.getByTestId(`task-node-${baseData.taskId}`);
  expect(root).toHaveTextContent(/task [a-f0-9-]{1,8}/);
  expect(root).toHaveAttribute('aria-label', expect.stringMatching(/task task [a-f0-9-]{1,8}/));
});
```

Per gotcha #66 (tests-pass-but-contract-fails) the test asserts the actual DOM output, not just the JS-layer behavior.

#### A.3.7 M04.V Decision 2 surfaced for maintainer

The spec §4a vs codebase naming drift (`hook_*` events in spec text; `verify_*` events in code) is a product-naming decision. Stage A doesn't change code — instead, the Stage A retrospective's `[END] Decisions` section surfaces the trade-off explicitly:

> **Spec §4a event naming reconciliation:** Spec §4a describes `hook_*` events; `schemas/event.v1.json` lines 332–369 declare `verify_*` events with `category` as the discriminator across all hook categories. Two paths:
>
> (a) **Update spec §4a text to `verify_*`** (recommended): code is internally consistent; renaming events would cascade across schema + Rust + TS + tests + renderer for cosmetic gain. Estimated cost: ~10 min spec edit.
>
> (b) **Rename to `hook_*` in `event.v1.1.json`**: aligns with spec but adds a v1 → v1.1 migration for one cosmetic naming choice. Estimated cost: ~3h across schema + Rust + TS + renderer + tests.
>
> Recommendation: (a). Maintainer adjudicates in M05.A approval.

If maintainer picks (b), Stage A's scope grows to absorb the rename. Default flow: (a).

### A.4 Tests

Behavior tests first, implementation tests second (CLAUDE.md §5). Per M04.V Finding #6 + gotcha #66, every test asserts an observable contract, not implementation-internal state.

| Test | Type | Catches |
|---|---|---|
| `framework_loader::walker::tests::walks_valid_framework_emits_zero_gaps` | Unit (Rust) | Walker shape + base case |
| `walker::tests::unresolved_skill_reference_emits_skill_missing` | Unit (Rust) | Skill gap detection |
| `walker::tests::unresolved_tool_reference_emits_tool_missing` | Unit (Rust) | Tool gap detection |
| `walker::tests::unresolved_mcp_reference_emits_mcp_missing` | Unit (Rust) | MCP gap detection |
| `walker::tests::unresolved_subagent_reference_emits_agent_missing` | Unit (Rust) | NEW: Agent gap detection |
| `walker::tests::severity_critical_for_loader_gaps` | Unit (Rust) | Severity matrix — loader gaps default critical |
| `walker::tests::multiple_gaps_in_one_walk_all_surface` | Unit (Rust) | Walker doesn't short-circuit on first gap |
| `tests/framework_loader_smoke.rs::valid_aria_framework_loads_with_zero_gaps` | Integration (Rust) | End-to-end against `examples/aria/framework.json` |
| `tests/framework_loader_smoke.rs::two_consecutive_loads_succeed` | Integration (Rust) | Multi-call invariant per gotcha #69 |
| `request_capability::tests::tool_kind_emits_tool_missing_with_requested_via_source` | Unit (Rust) | Meta-tool routing for tool kind |
| `request_capability::tests::all_four_kinds_routed` | Unit (Rust) | Parameterized over all 4 kinds |
| `request_capability::tests::severity_requested_for_meta_tool_gaps` | Unit (Rust) | Severity differentiates loader vs meta-tool |
| `tests/unit/lib/graphStore.test.ts::applies_tool_missing_event_mounts_gap_node` | Unit (TS) | applyEvent branch + GapNode mount |
| `graphStore.test.ts::applies_gap_resolved_dismisses_gap_node` | Unit (TS) | gap_resolved branch |
| `graphStore.test.ts::tool_missing_idempotent_on_re_emission` | Unit (TS) | Idempotence per spec |
| `tests/unit/nodes/TaskNode.test.tsx::renders_task_id_prefix_fallback_when_title_is_empty` | Unit (TSX) | **M04.V Decision 1 absorbed** |
| `tests/unit/nodes/GapNode.test.tsx::renders_severity_and_suggested_action` | Unit (TSX) | New payload fields surface in DOM |
| `tests/unit/nodes/GapNode.test.tsx::four_kinds_render_distinguishable_visual` | Unit (TSX) | tool/skill/mcp/agent visually distinct |

Coverage gate (per CLAUDE.md §6):
- Workspace ≥80% (Rust + TS). Stage A's new code lands inside workspace gate.
- `runtime-main::framework_loader::walker` aims for ≥95% — it's pure-function logic with no IO; high coverage is cheap.

### A.5 CLI Prompt

Paste the XML block below into a fresh Claude Code session as the opening message. Per `STAGE-PROMPT-PROTOCOL.md` v1.5 — Stage A is a work-stage prompt; uses v1.4 protocol tags plus v1.5 awareness that this milestone's V section will execute before closeout.

```xml
<work_stage_prompt id="M05.A">
  <context>
    Stage A of M05 (Gap Detection + Capability Enforcement). Wires §4b gap
    detection: framework_loader (parses framework JSON, walks declared
    primitives, emits gap events for unresolved references) + request_capability
    meta-tool (an agent's structured way to ask for a capability mid-session) +
    schema enrichment (severity + suggested_action + requested_via discriminator
    on the four *_missing variants; adds new `agent_missing` variant). Absorbs
    two M04.V 🟡 carry-forwards: Decision 1 (TaskNode empty-title regression
    test) and Decision 2 (§4a hook_*/verify_* spec-vs-code reconciliation
    surface to maintainer). Reconciles the M02/M03 deferred ContextType
    enum drift with spec §2b. Stage B does not start until this stage's
    commit is on the milestone branch `claude/m05-gap-capability-phase-doc`.
    First milestone authored under v1.5 protocol with Stage V section
    declared in this Phase doc.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M05-gap-capability.md (Background, Document Structure, Implementation Workflow, Pre-existing legacy file inventory, Stage A sections A.1–A.4)</file>
    <file>agent-runtime-spec.md §0–§0d, §2b (ContextType), §4b (Gap detection), §8.security (the L1–L5 layer overview — full L1+L2a in Stage B but Stage A wires the loader that triggers enforcement)</file>
    <file>docs/MVP-v0.1.md §M5 (acceptance criteria)</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #43 typify root-oneOf, #51 schema-derived enums, #66–#72 the M04 IRL bug patterns)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="schema-as-source-of-truth pattern + xtask regenerate-types flow">crates/xtask/src/main.rs</file>
    <file purpose="existing event variants and union shape to extend">schemas/event.v1.json</file>
    <file purpose="typify-generated baseline; do NOT hand-edit — regenerate from schema">crates/runtime-core/src/generated/event.rs</file>
    <file purpose="framework JSON shape the loader consumes">crates/runtime-core/src/generated/framework.rs</file>
    <file purpose="emitter pattern + signal-write integration archetype">crates/runtime-main/src/budget/enforcer.rs</file>
    <file purpose="HITL seam wire archetype — on_gap trigger lookup">crates/runtime-main/src/hitl/policy.rs</file>
    <file purpose="reference framework — loader's valid-input smoke target">examples/aria/framework.json</file>
    <file purpose="graphStore applyEvent reducer + the existing no-op cases for the four *_missing variants (M04-era prep)">src/lib/graphStore.ts</file>
    <file purpose="TaskNode empty-title fallback to regression-test (M04.V Decision 1)">src/components/nodes/TaskNode.tsx</file>
  </read_reference>

  <read_prior_milestones>
    <gap_analysis_carry_forward milestone="M04"/>
    <milestone_summary milestone="M04" section="Decisions to apply before the next parent milestone"/>
    <verifier_retrospective milestone="M04">docs/build-prompts/retrospectives/M04.V-retrospective.md — Findings #1 (🟡 TaskNode test) + #2 (🟡 §4a reconciliation) absorb here; Findings #3–#6 are tech-debt logged at docs/tech-debt.md TD-001..004 and do not require Stage A action</verifier_retrospective>
  </read_prior_milestones>

  <deliverable ref="docs/build-prompts/M05-gap-capability.md" section="A.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M05-gap-capability.md" section="A.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M05-gap-capability.md" section="Key constraints"/>

  <gates milestone="M05"/>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch_state" gate="git rev-parse --abbrev-ref HEAD must equal claude/m05-gap-capability-phase-doc"/>
    <check name="prior_milestone_merged" gate="git log origin/main --oneline | head -5 must include the M04 closeout merge and the M04.V retrospective merge"/>
    <check name="rust_toolchain" gate="cargo --version must report 1.95.0 per rust-toolchain.toml"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <inventory_row path="schemas/event.v1.json" status="exists"/>
    <inventory_row path="crates/runtime-core/src/generated/event.rs" status="exists"/>
    <inventory_row path="src/types/agent_event.ts" status="exists"/>
    <inventory_row path="crates/runtime-main/src/framework_loader/mod.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/framework_loader/walker.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/framework_loader/error.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/sdk/request_capability.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/lib.rs" status="exists"/>
    <inventory_row path="crates/runtime-core/src/signal.rs" status="exists"/>
    <inventory_row path="src/lib/graphStore.ts" status="exists"/>
    <inventory_row path="src/components/nodes/GapNode.tsx" status="exists"/>
    <inventory_row path="src/components/nodes/TaskNode.tsx" status="exists"/>
    <inventory_row path="tests/unit/nodes/TaskNode.test.tsx" status="exists"/>
    <inventory_row path="tests/unit/nodes/GapNode.test.tsx" status="exists"/>
    <inventory_row path="tests/unit/lib/graphStore.test.ts" status="exists"/>
    <inventory_row path="crates/runtime-main/tests/framework_loader_smoke.rs" status="new"/>
    <inventory_row path="examples/aria/framework.json" status="exists"/>
  </phase_doc_inventory_audit>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <schema_audit>
    <survey pattern='"GapSeverity"' purpose="confirm GapSeverity $def not already declared in any schemas/*.v1.json before authoring as new $def in event.v1.json"/>
    <survey pattern='"GapSource"' purpose="confirm GapSource $def not already declared elsewhere"/>
    <survey pattern='"agent_missing"' purpose="confirm agent_missing event variant not already declared (only the three existing *_missing variants should match)"/>
  </schema_audit>

  <schema_root_check/>

  <architecture_check>
    <claim description="framework_loader emits gap events via Emitter trait, NOT directly via drone IPC — same pattern as M04 budget enforcer and HITL seam (ADR-0007 in-process pattern)" verify="grep -rn 'impl Emitter for' crates/runtime-main/src/ ; expect at least one impl in framework_loader/mod.rs after Stage A lands"/>
    <claim description="request_capability is a META-tool: SDK routes it inline without LLM round-trip; NOT a normal tool dispatched via tool_invoked event" verify="grep -n 'request_capability' crates/runtime-main/src/sdk/mod.rs ; expect mod declaration + tool-dispatch routing distinct from tool_invoked path"/>
    <claim description="HITL on_gap trigger already wired in M04.E; Stage A reuses (does NOT re-declare)" verify="grep -n 'on_gap' crates/runtime-main/src/hitl/policy.rs ; expect existing entry from M04.E"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="tool_missing|skill_missing|mcp_missing" purpose="enumerate all consumers of the existing three *_missing variants (graphStore no-op cases, schema variants, any test fixtures) before enriching their payload — confirm the rename + enrichment doesn't break anywhere outside the known sites"/>
    <grep pattern="case 'gap_resolved'|case 'capability_violation'|case 'capability_grant'" purpose="confirm graphStore is the only consumer of these no-op variants (other no-op cases per graphStore.ts:1156+ pattern)"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="no new crates" required_features="N/A" min_version="N/A" audit="cargo deny check still passes; Stage A introduces no new third-party dependencies (framework_loader uses tokio + serde + thiserror — all already in workspace)"/>
  </dependency_audit_check>

  <runtime_environment os="windows" note="Build agent runs on Windows 11 per M01-M04 pattern; Select-String for grep; Test-Path for file checks; commands assume PowerShell shell with the bash tool available for cross-platform fallback (gotcha #56 cargo llvm-cov subprocess flake)"/>

  <gotchas>
    <trap>Schema-derived enum mismatch: writing test fixtures with stringly-typed enum values can drift from the generated TS enum. Per gotcha #51, pre-flight grep the schema $defs and use the generated TS enum's exact string values (`'critical' | 'important' | 'advisory' | 'requested'`). Don't hand-author the test fixture string literals.</trap>
    <trap>typify panics on validated inline strings inside event-variant `oneOf`. Per gotcha #43 — `suggested_action` is a validated string (`minLength: 1`); declare it inline only if typify accepts it cleanly. If panics surface, extract to `$defs/SuggestedAction` with a title.</trap>
    <trap>graphStore no-op cases that get filled in: the existing `case 'tool_missing':` / etc. lines at graphStore.ts:1156+ are no-op fallthroughs to the default. Replace each with a real handler — do NOT just delete the case label, which would change the default-case behavior.</trap>
    <trap>ContextType reconcile: spec §2b is the source of truth for `context_type` values. If the spec disagrees with the current enum, update the enum (not the spec) UNLESS maintainer explicitly signals otherwise. Surface the diff in the Stage A retrospective for review.</trap>
    <trap>HITL on_gap trigger reuse: do NOT redefine `on_gap` in policy.rs. It already exists from M04.E. Wire framework_loader + request_capability emitter to fire the existing trigger via the existing HitlSeam.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT load examples/aria/framework.json + run smoke session as integration test in Stage A — the full agent loop integration is M07's plan_loop driver work. Stage A's smoke is the framework_loader walker against the static JSON; the SDK agent loop integration lives in Stage B+ stages and M07.</warning>
    <warning>DO NOT add a new schema file (e.g., schemas/gap.v1.json). The gap event variants live in the existing event.v1.json union. Adding a new schema file triggers the M04.D-class collision pattern that gotcha #41 + the schema_audit slot are structured to prevent.</warning>
  </execution_warnings>

  <time_box estimate_hours="6"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>M04.V Decision 2 (spec §4a hook_*/verify_* reconciliation) maintainer-decision surface; M02/M03 ContextType reconcile outcome (which direction the drift was resolved); whether the framework_loader walker tests caught any sub-agent declaration edge cases not enumerated in spec §4b; whether the request_capability meta-tool's "return Pending; await HITL response" pattern composes cleanly with the existing HitlSeam (or requires a new seam variant)</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="A.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M05.*-retrospective.md)</item>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (cargo fmt + clippy + test + doc + frontend lint/typecheck/test + validator + schema_drift_check, each pass/fail with key numbers)</item>
    <item>retrospective filled-in [END] section (three-axis scoring + verdict + decisions for Stage B + spec §4a reconciliation surface + ContextType reconcile outcome)</item>
    <item>draft commit message from M05-gap-capability.md A.6 Commit Message section (filled with session URL)</item>
    <item>explicit statement: "Stage M05.A is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### A.6 Commit Message

```
feat(runtime): M05 Stage A — §4b gap detection + framework_loader + request_capability + M04.V carry-forwards

Wires §4b gap detection end-to-end. New module
crates/runtime-main/src/framework_loader/ walks a parsed framework
JSON, emits gap events for unresolved references; new module
crates/runtime-main/src/sdk/request_capability.rs gives agents a
structured meta-tool to ask for a capability mid-session. Both emit
the same enriched gap event shape (severity + suggested_action +
requested_via discriminator) and route through the existing HITL
on_gap trigger from M04.E (no new seam variant).

Schema:
- schemas/event.v1.json: add agent_missing variant; enrich the four
  *_missing variants with severity, suggested_action, requested_via.
- crates/runtime-core/src/generated/event.rs + src/types/agent_event.ts:
  regenerated.

Renderer:
- src/lib/graphStore.ts: light up the four *_missing + gap_resolved
  applyEvent branches (previously no-op per M04-era prep).
- src/components/nodes/GapNode.tsx: render severity + suggested_action
  + agent_id from enriched payload.

M04.V carry-forwards absorbed:
- M04.V Decision 1: regression test
  `renders_task_id_prefix_fallback_when_title_is_empty` added to
  tests/unit/nodes/TaskNode.test.tsx. Pins the LG-02 IRL fix
  (TaskNode.tsx:27).
- M04.V Decision 2: spec §4a hook_*/verify_* reconciliation surfaced
  in this stage's retrospective for maintainer adjudication.
  Default: update spec §4a text to `verify_*` (code is internally
  consistent). Maintainer decision recorded in M05.A retrospective.

M02/M03 carry-forward absorbed:
- ContextType enum reconciled with spec §2b
  (crates/runtime-core/src/signal.rs). Spec/code drift documented in
  retrospective.

Tests:
- 7 unit tests for the walker (per gap kind + severity matrix +
  multiple-gaps-no-shortcircuit)
- 2 integration tests (valid-framework smoke + multi-call invariant)
- 3 unit tests for request_capability meta-tool routing
- 4 graphStore.ts tests for new applyEvent branches
- 1 TaskNode regression test (M04.V Decision 1)
- 2 GapNode tests for enriched payload + visual differentiation

Coverage: workspace ≥80% (Rust + TS); framework_loader::walker ≥95%
(pure-function logic, high coverage is cheap).

Not in this stage: capability enforcer (Stage B), sandbox subprocess
(Stage C), tier system (Stage D), audit log (Stage E), GapPanel UI
(Stage F).

https://claude.ai/code/session_019xg3cDSLdtX6JkNTLc9a6T
```

---

## Stage B — §8.security L1 + L2a Capability Enforcer (new safety primitive ≥95%)

### B.1 Problem Statement

Every tool dispatch + sub-agent spawn passes through a capability enforcer that reads the calling agent's capability declaration, looks up what the requested action declares it needs, and **rejects mismatches with a `capability_violation` event + an HITL prompt** (`on_capability_violation` trigger, already wired in M04.E). Default-deny: no declaration → reject. The enforcer also narrows on agent→agent edges — a parent can only grant a subset of its own capabilities to a child sub-agent (the L2a layer).

The §8.security spec describes five layers (L1 declarations + L2a narrowing + L2b runtime-grants + L3 sandbox validation + L4 tier gates + L5 provenance/audit). Stage B implements **L1 + L2a only** — the in-process check that fires on every tool/sub-agent invocation. L3 sandbox is Stage C's out-of-process layer. L4 tier gates is Stage D. L5 provenance is Stage E.

Concrete deliverables:
1. `schemas/capability.v1.json` — new schema for capability declarations (what a tool / skill / agent declares it needs)
2. `crates/runtime-main/src/capability/` — new module containing the enforcer, declaration parser, allow-evaluator, narrowing logic
3. Wiring of the enforcer into the SDK's tool dispatch path + sub-agent spawn path
4. graphStore.ts lights up the `capability_violation` + `capability_grant` applyEvent branches (currently no-op from M04-era prep)
5. ≥95% per-module coverage on the enforcer + narrowing evaluator (new safety primitives)

Not in this stage:
- Sandbox subprocess (Stage C)
- Tier-based capability gates (Stage D)
- Audit log (Stage E)
- UI for capability-violation modal (Stage F — reuses M04.E HITLModal)

### B.2 Files to Change

| File | Status | Change |
|---|---|---|
| `schemas/capability.v1.json` | **new** | Capability declaration shape (kind, resource, scope, side-effect class) |
| `crates/runtime-core/src/generated/capability.rs` | **new** | Regenerated from schema |
| `src/types/capability.ts` | **new** | Regenerated from schema |
| `crates/runtime-main/src/capability/mod.rs` | **new** | Module root |
| `crates/runtime-main/src/capability/enforcer.rs` | **new** | The check-before-dispatch primitive |
| `crates/runtime-main/src/capability/declaration.rs` | **new** | Declaration parser + lookup (per-tool, per-skill, per-agent) |
| `crates/runtime-main/src/capability/narrowing.rs` | **new** | L2a narrowing evaluator: parent grants ⊆ own capabilities |
| `crates/runtime-main/src/capability/error.rs` | **new** | CapabilityError enum |
| `crates/runtime-main/src/capability/tests.rs` | **new** | Unit tests (mod or `#[cfg(test)]`) |
| `crates/runtime-main/src/sdk/mod.rs` | exists | Edit: wrap tool dispatch + sub-agent spawn through enforcer |
| `crates/runtime-main/src/hitl/policy.rs` | exists | Edit: `on_capability_violation` trigger emission wired to capability enforcer's reject path (the trigger already exists from M04.E — Stage B fires it) |
| `crates/runtime-main/src/lib.rs` | exists | Edit: `pub mod capability;` |
| `schemas/event.v1.json` | exists (Stage A edited) | Edit: enrich `capability_violation` event with `agent_id`, `capability_kind`, `requested_action`, `declared_scope`; enrich `capability_grant` with `parent_agent_id`, `granted_to`, `narrowed_from` |
| `src/lib/graphStore.ts` | exists | Edit: light up `capability_violation` + `capability_grant` applyEvent branches |
| `tests/unit/lib/graphStore.test.ts` | exists | Add: applyEvent tests for the two enriched event variants |
| `crates/runtime-main/tests/capability_enforcer_smoke.rs` | **new** | Integration test exercising the enforcer end-to-end |
| `docs/build-prompts/retrospectives/M05.B-retrospective.md` | **new** | Stage B retrospective |
| `CHANGELOG.md` | exists | Edit: `[Unreleased]` notes M05.B — capability enforcer + narrowing |

Effort budget: ~4–6 hours of code execution. Narrowing evaluator is the highest-cognitive-load piece; gets a property test.

### B.3 Detailed Changes

#### B.3.1 Capability declaration schema

`schemas/capability.v1.json` defines what a tool / skill / agent declares about itself:

```jsonc
{
  "$id": "https://agent-runtime.local/schemas/capability.v1.json",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "CapabilityDeclaration",
  "description": "A declaration of what a runtime primitive needs to do its job. The enforcer compares this against the calling agent's grants at dispatch time.",
  "type": "object",
  "required": ["kind", "resource", "scope", "side_effect_class"],
  "properties": {
    "kind": { "$ref": "#/$defs/CapabilityKind" },
    "resource": { "$ref": "common.v1.json#/$defs/NonEmptyString" },
    "scope": { "$ref": "#/$defs/CapabilityScope" },
    "side_effect_class": { "$ref": "#/$defs/SideEffectClass" }
  },
  "$defs": {
    "CapabilityKind": { "type": "string", "enum": ["read", "write", "exec", "network", "process_spawn"] },
    "CapabilityScope": {
      "oneOf": [
        { "type": "object", "title": "GlobScope",     "required": ["glob"], "properties": { "glob": { "type": "string", "minLength": 1 } } },
        { "type": "object", "title": "DomainScope",   "required": ["domain"], "properties": { "domain": { "type": "string", "format": "hostname" } } },
        { "type": "object", "title": "PathScope",     "required": ["path"], "properties": { "path": { "type": "string", "minLength": 1 } } }
      ]
    },
    "SideEffectClass": { "type": "string", "enum": ["pure", "filesystem_mutate", "network_egress", "process_spawn", "irreversible"] }
  }
}
```

Per gotcha #43 (typify root-oneOf) + gotcha #57 (schema root must be concrete `type: object`) — the schema root is `type: object` not `oneOf`; the variant-scoped types live in `$defs` with `title` for typify-naming.

#### B.3.2 Enforcer state + check primitive

```rust
// crates/runtime-main/src/capability/enforcer.rs
use runtime_core::generated::capability::CapabilityDeclaration;

pub struct CapabilityEnforcer {
    grants_by_agent: HashMap<AgentId, Vec<CapabilityDeclaration>>,
}

impl CapabilityEnforcer {
    /// Check that `agent` has been granted a capability that satisfies
    /// `requested`. Returns Ok if granted; Err(CapabilityError::Denied)
    /// otherwise. Default-deny: if `agent` has no declarations at all,
    /// returns Err(Denied).
    pub fn check(
        &self,
        agent: AgentId,
        requested: &CapabilityDeclaration,
    ) -> Result<(), CapabilityError> {
        let grants = self.grants_by_agent.get(&agent).ok_or(CapabilityError::Denied {
            reason: DenyReason::NoDeclarations,
        })?;
        grants
            .iter()
            .find(|grant| grant.satisfies(requested))
            .map(|_| ())
            .ok_or(CapabilityError::Denied {
                reason: DenyReason::NoMatchingGrant,
            })
    }

    /// Grant a capability to an agent. Called by the loader (initial
    /// declaration parsing) and by request_capability resolution.
    pub fn grant(&mut self, agent: AgentId, capability: CapabilityDeclaration) {
        self.grants_by_agent.entry(agent).or_default().push(capability);
    }
}
```

The `satisfies` method on `CapabilityDeclaration` is the inner check (same kind, same side-effect class, scope-containment). It's pure-function logic — property-tested.

#### B.3.3 Narrowing (L2a) evaluator

When agent A spawns sub-agent B with capabilities, B's grants MUST be a subset of A's grants. The narrowing evaluator enforces this:

```rust
// crates/runtime-main/src/capability/narrowing.rs

/// Verify that `proposed` is a subset of `parent`'s capabilities.
/// Returns Ok with the (possibly-clamped) child grants; Err if proposed
/// includes any capability the parent doesn't have OR if scope widens.
pub fn narrow(
    parent: &[CapabilityDeclaration],
    proposed: &[CapabilityDeclaration],
) -> Result<Vec<CapabilityDeclaration>, NarrowingError> {
    for prop in proposed {
        let satisfying_parent_grant = parent
            .iter()
            .find(|p| p.subsumes(prop))
            .ok_or(NarrowingError::CapabilityNotHeldByParent { proposed: prop.clone() })?;
        // Scope clamping: child's scope may be narrower than parent's,
        // never wider. The `subsumes` check enforces this — but be
        // explicit:
        debug_assert!(satisfying_parent_grant.scope_contains(&prop.scope));
    }
    Ok(proposed.to_vec())
}
```

The narrowing invariant is the load-bearing property: child capabilities ⊆ parent capabilities, for every spawn. Property-tested with proptest.

#### B.3.4 SDK wire-up

The SDK's tool dispatch path currently calls into providers without a capability check. Stage B wraps the dispatch:

```rust
// crates/runtime-main/src/sdk/mod.rs (excerpt)
async fn dispatch_tool(
    &mut self,
    invocation: ToolInvocation,
) -> Result<ToolResult, ToolDispatchError> {
    let agent_id = invocation.agent_id;
    let needed: CapabilityDeclaration = invocation.tool.required_capability();
    match self.capability_enforcer.check(agent_id, &needed) {
        Ok(()) => {
            self.emit_capability_grant(agent_id, &needed).await;
            self.dispatch_inner(invocation).await
        }
        Err(CapabilityError::Denied { reason }) => {
            self.emit_capability_violation(agent_id, &needed, reason).await;
            // HITL on_capability_violation trigger handles the rest;
            // returns Ok(retry) or Ok(skip) or Err(abort).
            self.await_hitl_resolution(agent_id, /* ... */).await
        }
    }
}
```

Sub-agent spawn does the parallel narrowing check. Both call sites become the enforcement boundary.

#### B.3.5 graphStore lights up violation + grant branches

```typescript
// src/lib/graphStore.ts (the previously-no-op case branches get real bodies)
case 'capability_violation': {
  return updateState((s) => {
    s.capabilityViolations.set(event.agent_id, {
      capability_kind: event.capability_kind,
      requested_action: event.requested_action,
      declared_scope: event.declared_scope,
      timestamp: event.timestamp ?? Date.now(),
    });
  });
}
case 'capability_grant': {
  return updateState((s) => {
    s.capabilityGrants.append({
      parent: event.parent_agent_id ?? null,
      granted_to: event.granted_to,
      narrowed_from: event.narrowed_from ?? null,
      capability_kind: event.capability_kind,
      timestamp: event.timestamp ?? Date.now(),
    });
  });
}
```

Stage F renders these into the `CapabilityBadge` per-node visual + the capability-violation modal (HITLModal variant per ADR-0007).

### B.4 Tests

Per gotcha #66 + #69, behavior-first + multi-call invariants enforced.

| Test | Type | Catches |
|---|---|---|
| `enforcer::tests::default_deny_when_no_declarations` | Unit (Rust) | Default-deny baseline |
| `enforcer::tests::exact_match_grant_passes` | Unit (Rust) | Happy path |
| `enforcer::tests::scope_widening_is_denied` | Unit (Rust) | A glob-scope grant doesn't satisfy a request outside that glob |
| `enforcer::tests::side_effect_class_mismatch_denied` | Unit (Rust) | Read grant ≠ write request |
| `enforcer::tests::twice_in_sequence_both_succeed` | Unit (Rust) | Multi-call invariant per gotcha #69 |
| `narrowing::tests::child_subset_of_parent_ok` | Unit (Rust) | Happy path |
| `narrowing::tests::child_widening_scope_denied` | Unit (Rust) | Anti-widening |
| `narrowing::tests::child_capability_parent_lacks_denied` | Unit (Rust) | Anti-capability-addition |
| `narrowing::tests::property_narrowing_preserves_invariant` (proptest) | Property (Rust) | For any parent + proposed-child, narrowed result ⊆ parent |
| `tests/capability_enforcer_smoke.rs::tool_call_with_grant_succeeds_emits_capability_grant` | Integration | End-to-end via SDK wrap |
| `tests/capability_enforcer_smoke.rs::tool_call_without_grant_denied_emits_capability_violation` | Integration | Default-deny end-to-end |
| `graphStore.test.ts::applies_capability_violation_appends_to_state` | Unit (TS) | Renderer-side reducer |
| `graphStore.test.ts::applies_capability_grant_appends_to_log` | Unit (TS) | Renderer-side reducer |

Coverage gate: `cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs" --fail-under-lines 95` — extend the runtime-main ≥95% gate to cover the new `capability/` module. Per-module baseline: `enforcer.rs` and `narrowing.rs` aim for 100% (pure-function logic); `declaration.rs` aims for ≥95%.

### B.5 CLI Prompt

```xml
<work_stage_prompt id="M05.B">
  <context>
    Stage B of M05. Implements §8.security L1 + L2a capability enforcer
    + narrowing primitives as a NEW safety primitive at ≥95% per-module
    coverage. The enforcer wraps every tool dispatch + sub-agent spawn
    in the SDK; default-deny semantics; capability_violation event
    routes through the existing M04.E on_capability_violation HITL
    trigger. The narrowing evaluator (L2a) enforces "child capabilities
    ⊆ parent capabilities" with a proptest. New schema
    `schemas/capability.v1.json` ships the declaration shape. Builds on
    Stage A's framework_loader (which now grants initial capabilities
    via the enforcer). Stage C does NOT start until this stage's commit
    is on the milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M05-gap-capability.md (Background, Document Structure, Implementation Workflow, Pre-existing legacy file inventory, Stage B sections B.1–B.4)</file>
    <file>docs/build-prompts/retrospectives/M05.A-retrospective.md (the [END] Decisions section — apply before code)</file>
    <file>agent-runtime-spec.md §8.security (especially L1 declarations + L2a narrowing — L3 sandbox is Stage C; L4 tier is Stage D; L5 provenance is Stage E)</file>
    <file>docs/MVP-v0.1.md §M5</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #43, #51, #57, #66 tests-pass-but-contract-fails, #69 multi-call invariants)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="SDK tool dispatch path to wrap">crates/runtime-main/src/sdk/mod.rs</file>
    <file purpose="M04.E HitlSeam archetype — the on_capability_violation trigger lookup that Stage B fires">crates/runtime-main/src/hitl/policy.rs</file>
    <file purpose="M04 budget enforcer pattern — Stage B's enforcer follows the same in-process check shape">crates/runtime-main/src/budget/enforcer.rs</file>
    <file purpose="Stage A's framework_loader — Stage B integrates capability declarations parsed during loader walk">crates/runtime-main/src/framework_loader/mod.rs</file>
    <file purpose="proptest archetype from M01.C">crates/runtime-drone/src/snapshot.rs</file>
  </read_reference>

  <read_prior_stages>
    <retrospective milestone="M05" stage="A"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M05-gap-capability.md" section="B.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M05-gap-capability.md" section="B.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M05-gap-capability.md" section="Key constraints"/>

  <gates milestone="M05"/>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="stage_a_committed" gate="git log --oneline | head -1 must reference Stage M05.A commit"/>
    <check name="schema_regen_clean" gate="cargo xtask regenerate-types --check produces zero diff"/>
    <check name="rust_toolchain" gate="cargo --version must report 1.95.0"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <inventory_row path="schemas/capability.v1.json" status="new"/>
    <inventory_row path="crates/runtime-core/src/generated/capability.rs" status="new"/>
    <inventory_row path="src/types/capability.ts" status="new"/>
    <inventory_row path="crates/runtime-main/src/capability/mod.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/capability/enforcer.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/capability/declaration.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/capability/narrowing.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/capability/error.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/sdk/mod.rs" status="exists"/>
    <inventory_row path="crates/runtime-main/src/hitl/policy.rs" status="exists"/>
    <inventory_row path="crates/runtime-main/src/lib.rs" status="exists"/>
    <inventory_row path="schemas/event.v1.json" status="exists"/>
    <inventory_row path="src/lib/graphStore.ts" status="exists"/>
    <inventory_row path="crates/runtime-main/tests/capability_enforcer_smoke.rs" status="new"/>
  </phase_doc_inventory_audit>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <schema_audit>
    <survey pattern='"CapabilityKind"' purpose="confirm CapabilityKind not already $def'd elsewhere"/>
    <survey pattern='"CapabilityScope"' purpose="confirm CapabilityScope not already declared"/>
    <survey pattern='"SideEffectClass"' purpose="confirm SideEffectClass not already declared"/>
  </schema_audit>

  <schema_root_check/>

  <architecture_check>
    <claim description="Enforcer is in-process per ADR-0007 (HITL seam in-process pattern); no separate enforcer-process subprocess in Stage B (that's Stage C's L3 sandbox)" verify="grep -rn 'tokio::process::Command' crates/runtime-main/src/capability/ ; expect zero matches"/>
    <claim description="Enforcer wraps tool dispatch BEFORE LLM call, not after — failed enforcement never reaches Anthropic" verify="grep -n 'capability_enforcer.check' crates/runtime-main/src/sdk/mod.rs ; expect call site appears before any provider.invoke call"/>
    <claim description="on_capability_violation HITL trigger already exists from M04.E (M04 retrospective E.3); Stage B FIRES it, does not DECLARE it" verify="grep -n 'on_capability_violation' crates/runtime-main/src/hitl/policy.rs ; expect existing entry from M04.E"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="case 'capability_violation'|case 'capability_grant'" purpose="confirm graphStore is the only consumer of these previously-no-op cases; Stage B replaces both"/>
    <grep pattern="dispatch_tool|spawn_sub_agent" purpose="enumerate every tool-dispatch + sub-agent-spawn call site so Stage B's wrap covers all of them"/>
  </fan_out_grep>

  <runtime_environment os="windows" note="Build agent runs on Windows 11; use cargo +stable for tests if needed (rust-toolchain.toml pins 1.95.0 — Windows CI is the gate)"/>

  <gotchas>
    <trap>Default-deny is load-bearing: the no-declarations path MUST return Err, not Ok. A test asserting "no declarations passes" would be wrong — write the test to assert Err with reason=NoDeclarations.</trap>
    <trap>Narrowing is asymmetric: parent.subsumes(child) NOT child.subsumes(parent). The property test must check direction; mis-direction would pass when child WIDENS parent.</trap>
    <trap>Scope containment (glob/domain/path) requires per-variant logic. A glob-scope grant doesn't satisfy a path-scope request unless the path matches the glob. Property test the containment carefully.</trap>
    <trap>capability_violation event MUST emit BEFORE the HITL prompt (so the renderer can surface state); the HITL prompt comes after. Order matters for the renderer's responsiveness.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT add capability declarations to existing tools/skills in this stage. Stage B ships the enforcer + the schema; granting declarations to existing tools is its own (smaller) follow-up that wires `examples/aria/framework.json` to declare capabilities. Default-deny means no existing tool dispatches will pass until grants are wired — keep the smoke test focused on the enforcer's check logic, not full end-to-end.</warning>
  </execution_warnings>

  <time_box estimate_hours="5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Property-test outcome for narrowing — did proptest surface any narrowing-invariant edge cases? Coverage breakdown per file in `capability/` — which file fell below 95% and why. Decisions for Stage C (the sandbox subprocess): what L3 boundary contract does the L1+L2a enforcer expose to L3?</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="B.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log + retro listing)</item>
    <item>diff stat</item>
    <item>gate results (cargo fmt + clippy + test + doc + coverage on runtime-main::capability + frontend + validator + schema_drift_check)</item>
    <item>retrospective filled-in [END] section (three-axis scoring + verdict + decisions for Stage C)</item>
    <item>draft commit message from M05-gap-capability.md B.6</item>
    <item>explicit statement: "Stage M05.B is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### B.6 Commit Message

```
feat(runtime): M05 Stage B — §8.security L1 + L2a capability enforcer (new safety primitive)

New module crates/runtime-main/src/capability/ implements the §8.security
L1 (declarations) + L2a (narrowing) layers. The enforcer wraps every
tool dispatch + sub-agent spawn in the SDK with a check-before-call
gate. Default-deny: no declarations → reject. On reject, emits a
capability_violation event and routes through the existing M04.E
on_capability_violation HITL trigger (no new seam).

Narrowing evaluator (L2a) enforces "child capabilities ⊆ parent
capabilities" — verified by a proptest covering the asymmetric
subsumes-direction invariant.

Schema:
- schemas/capability.v1.json (new): CapabilityDeclaration with kind +
  resource + scope (glob/domain/path variants) + side_effect_class.
- schemas/event.v1.json (edit): enrich capability_violation +
  capability_grant payloads.
- runtime-core::generated::capability + src/types/capability.ts:
  regenerated.

SDK integration:
- crates/runtime-main/src/sdk/mod.rs: tool dispatch + sub-agent spawn
  wrap enforcer.check() / narrow() before invocation.

Renderer:
- src/lib/graphStore.ts: lit up capability_violation + capability_grant
  applyEvent branches (previously no-op).

Coverage: ≥95% on runtime-main::capability (per-module gate; enforcer +
narrowing aim for 100%, pure-function logic).

Tests:
- 5 enforcer unit tests (default-deny, exact match, scope widening,
  side-effect mismatch, multi-call invariant)
- 4 narrowing unit tests (subset, widening denied, capability-add
  denied) + 1 proptest (narrowing-invariant property)
- 2 integration tests (with-grant succeeds, without-grant denied)
- 2 graphStore applyEvent tests for new event branches

https://claude.ai/code/session_019xg3cDSLdtX6JkNTLc9a6T
```

---

## Stage C1 — §8.security L3 Sandbox crate + main-side IPC client + lifecycle (new safety primitive ≥95%)

> **Note on C1/C2 split:** Stage C is split into C1 (sandbox crate plumbing + IPC + lifecycle) and C2 (cross-platform OS isolation: seccomp / landlock / Job Objects). C is the only milestone stage that's split, and only because it's two coherent surfaces (subprocess plumbing vs OS-specific kernel isolation) with different dependencies, test harnesses, and failure modes. Stage A is NOT similarly split — Stage A's surface is one coherent piece per the Background section above.

### C1.1 Problem Statement

When an agent receives a generated artifact (a tool implementation or skill recipe produced by the M09 generators — NOT in v0.1; M05 ships the L3 boundary that M09 will eventually use), the runtime executes it inside a **sandbox subprocess**. The sandbox receives the generated code + the agent's capability declaration, runs the validator, and either Ok's it (cleared for execution by the main process) or rejects.

Stage C1 ships the **subprocess plumbing**: the `runtime-sandbox` crate skeleton (entry point, IPC server, validator, error types), the main-side `sandbox_ipc` client (mirrors the M04+PR-#64 drone IPC borrow-not-move pattern from the start, not retrofitted), and the Tauri-side lifecycle (spawn at startup, register Arc<SandboxClient> as managed state, graceful shutdown). **Cross-platform kernel-level isolation (seccomp / landlock / Job Objects) is Stage C2** — separated because it requires per-platform dependencies + per-platform test matrix + different failure modes (seccomp filter bug crashes the subprocess; IPC bug just hangs).

L3 is the cross-process layer parallel to drone (M01) and main: a third subprocess (`runtime-sandbox`) spawned by main per validation request, communicating via framed JSON over Unix socket / Windows named pipe. Stage C1 provides the cross-platform plumbing (entry point + IPC protocol + validator + main-side client + lifecycle) WITHOUT OS-specific isolation; Stage C2 layers seccomp / landlock / Job Objects on top.

v0.1 does NOT ship M09 generators, so the sandbox subprocess sits as a callable primitive that no production code path invokes yet. Stages C1 + C2 together provide the boundary; M09 wires the first caller.

Concrete C1 deliverables (plumbing only — no OS isolation):
1. `crates/runtime-sandbox/src/lib.rs` — lit up from M01 scaffold; module structure mirrors `runtime-drone` shape
2. `crates/runtime-sandbox/src/main.rs` — binary entry point (mirrors `runtime-drone/src/main.rs`)
3. `crates/runtime-sandbox/src/protocol.rs` — framed-JSON IPC protocol (mirrors drone IPC)
4. `crates/runtime-sandbox/src/validator.rs` — pure-function validator (artifact + declaration → Ok / Reject)
5. `crates/runtime-sandbox/src/error.rs` — SandboxError enum
6. `crates/runtime-main/src/sandbox_ipc/` — client side (mirrors `runtime-main/src/drone_ipc/`; borrow-not-move pattern from PR #64 + multi-call test FIRST per gotcha #69)
7. `src-tauri/src/sandbox_lifecycle.rs` — Tauri-side spawn/manage (mirrors `drone_lifecycle.rs`)

### C1.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-sandbox/Cargo.toml` | exists | Edit: minimal deps for plumbing (tokio + serde + thiserror — NO platform-specific yet; Stage C2 adds those) |
| `crates/runtime-sandbox/src/lib.rs` | exists (M01 stub) | Edit: real module tree |
| `crates/runtime-sandbox/src/main.rs` | **new** | Binary entry point |
| `crates/runtime-sandbox/src/protocol.rs` | **new** | IPC protocol — framed JSON |
| `crates/runtime-sandbox/src/ipc.rs` | **new** | IPC server half on sandbox side; sibling to main-side `sandbox_ipc/client.rs`. Lifted into the C2 ≥95% gate post-C1; baseline 92.58% line / 94.01% region at C1-end / ≥95% at C2-end. (M06.A truth-up per M05.V Finding #3 — added to inventory after the original C1.2 table closed.) |
| `crates/runtime-sandbox/src/validator.rs` | **new** | Pure-function validator |
| `crates/runtime-sandbox/src/error.rs` | **new** | SandboxError enum |
| `crates/runtime-sandbox/tests/integration.rs` | **new** | Round-trip integration test WITHOUT isolation (Stage C2 extends with isolation-on tests) |
| `crates/runtime-main/src/sandbox_ipc/mod.rs` | **new** | Module root (mirrors drone_ipc) |
| `crates/runtime-main/src/sandbox_ipc/client.rs` | **new** | SandboxClient |
| `crates/runtime-main/src/sandbox_ipc/connection.rs` | **new** | Connection — borrow-not-move per gotcha #72 |
| `src-tauri/src/sandbox_lifecycle.rs` | **new** | Spawn/manage sandbox subprocess at app startup |
| `src-tauri/src/main.rs` | exists | Edit: `mod sandbox_lifecycle` + spawn at setup hook |
| `docs/build-prompts/retrospectives/M05.C1-retrospective.md` | **new** | Stage C1 retrospective |
| `CHANGELOG.md` | exists | Edit: M05.C1 — sandbox crate plumbing + main-side IPC + lifecycle |

Effort budget C1: ~4–6 hours.

### C1.3 Detailed Changes

#### C1.3.1 Sandbox subprocess shape (mirrors drone)

```rust
// crates/runtime-sandbox/src/main.rs — entry point, exact archetype from runtime-drone
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    init_tracing();
    let args = Args::parse();
    info!(session_id = %args.session_id, "sandbox starting");
    if let Err(e) = runtime_sandbox::run(args.session_id, args.ipc_socket).await {
        error!(error = %e, "sandbox exited with error");
        std::process::exit(1);
    }
}
```

Subprocess takes session_id + ipc_socket path; mirrors drone subprocess. Spawned once at app startup; long-lived. No OS-specific isolation here — Stage C2 layers seccomp/landlock/Job Objects on top of this binary's startup.

#### C1.3.2 IPC protocol

Framed JSON, request-response shape. Two commands:
- `ValidateArtifact { artifact_code: String, declaration: CapabilityDeclaration }` → `ValidationResult { ok: bool, reasons: Vec<String> }`
- `Shutdown` → no response; subprocess exits

Same `next_event` borrow-not-move pattern as the M04 drone IPC fix (gotcha #72 codifies; PR #64 fixed for drone — Stage C1 follows the same shape from the start).

#### C1.3.3 Validator (the cross-platform pure-function check)

```rust
// crates/runtime-sandbox/src/validator.rs
pub fn validate(
    artifact: &Artifact,
    declaration: &CapabilityDeclaration,
) -> ValidationResult {
    // Pure-function check: scan artifact for syscall/imports that
    // exceed the declared capability. Returns Ok if consistent.
    let scan = artifact.scan_syscalls();
    let exceeded: Vec<_> = scan.into_iter()
        .filter(|s| !declaration.permits(s))
        .collect();
    if exceeded.is_empty() {
        ValidationResult::ok()
    } else {
        ValidationResult::reject(exceeded.into_iter().map(|s| format!("disallowed syscall: {s}")).collect())
    }
}
```

Cross-platform; OS-specific isolation (Stage C2) installs the kernel-level fence BEFORE this function runs.

#### C1.3.4 Main-side client + lifecycle

`sandbox_ipc/client.rs` + `sandbox_ipc/connection.rs` mirror the drone IPC client + connection EXACTLY — same borrow-not-move pattern, same multi-call invariant tests from the start. Stage C1 explicitly cites gotcha #72 and writes the multi-call test FIRST (per the gotcha #69 rule).

`src-tauri/src/sandbox_lifecycle.rs` mirrors `drone_lifecycle.rs`: spawn the subprocess at Tauri setup hook, register `Arc<SandboxClient>` as Tauri-managed state, graceful shutdown on app exit.

### C1.4 Tests

| Test | Type | Catches |
|---|---|---|
| `validator::tests::pure_artifact_passes` | Unit (Rust) | Happy path — no syscalls |
| `validator::tests::filesystem_syscall_exceeds_pure_declaration_rejects` | Unit (Rust) | Validator math |
| `validator::tests::network_syscall_in_network_declaration_passes` | Unit (Rust) | Scope honored |
| `sandbox_ipc::client::tests::validate_request_response_succeeds` | Unit (Rust) | IPC happy path |
| `sandbox_ipc::client::tests::validate_succeeds_twice_in_sequence` | Unit (Rust) | **Multi-call invariant per gotcha #69 + #72** |
| `sandbox_ipc::connection::tests::next_event_returns_consecutive_events_without_consuming_reader` | Unit (Rust) | borrow-not-move pattern from PR #64 — applied from the start, not retrofitted |
| `tests/integration.rs::sandbox_round_trip_under_real_subprocess` | Integration (Rust) | End-to-end without isolation — main spawns sandbox, validates, shuts down |
| `tests/integration.rs::sandbox_restart_after_kill_resumes` | Integration (Rust) | Sandbox subprocess lifecycle resilience (mirrors drone reconnect tests) |

Coverage gate: `cargo llvm-cov --package runtime-main` extended to cover `sandbox_ipc/` ≥95%; `cargo llvm-cov --package runtime-sandbox` on the plumbing files (validator + protocol + error) aiming ≥95% with `lib.rs` + `main.rs` excluded (OS-signal-orchestrator pattern from M01.C). Stage C2 extends the runtime-sandbox gate to cover the OS-isolation files.

### C1.5 CLI Prompt

```xml
<work_stage_prompt id="M05.C1">
  <context>
    Stage C1 of M05. Lights up `crates/runtime-sandbox/` plumbing from
    M01 scaffold — the §8.security L3 sandbox subprocess WITHOUT OS
    isolation (that's Stage C2). Cross-platform plumbing only:
    subprocess entry point, framed-JSON IPC, pure-function validator,
    main-side IPC client (borrow-not-move + multi-call invariant test
    FIRST per gotchas #69 + #72 — PR #64 lessons applied, not
    retrofitted), Tauri-side lifecycle. Stage C2 layers seccomp /
    landlock / Job Objects on top of this binary; Stage D does NOT
    start until C2's commit is on the milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M05-gap-capability.md (Background, Stage C1 sections C1.1–C1.4)</file>
    <file>docs/build-prompts/retrospectives/M05.B-retrospective.md (apply [END] Decisions — particularly the L3 boundary contract Stage B specified)</file>
    <file>agent-runtime-spec.md §8.security L3 (sandbox subprocess); §1d (IPC topology, cross-platform addressing)</file>
    <file>docs/MVP-v0.1.md §M5</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #46 unfold-stream panic, #47 Windows file lock, #56 cargo llvm-cov subprocess flake, #69 multi-call invariants, #72 tokio::io::duplex EOF)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="Drone subprocess archetype — sandbox mirrors EXACTLY this shape">crates/runtime-drone/src/main.rs</file>
    <file purpose="Drone IPC protocol archetype — sandbox uses parallel shape">crates/runtime-drone/src/ipc.rs</file>
    <file purpose="Drone IPC client archetype — sandbox client mirrors borrow-not-move pattern from PR #64">crates/runtime-main/src/drone_ipc/connection.rs</file>
    <file purpose="Drone lifecycle archetype — sandbox lifecycle mirrors EXACTLY">src-tauri/src/drone_lifecycle.rs</file>
    <file purpose="Stage B's capability declaration consumed by sandbox validator">crates/runtime-core/src/generated/capability.rs</file>
    <file purpose="Cross-platform OS-call wrapper exclusion pattern from M01.C">CLAUDE.md §5 (Coverage gate semantics)</file>
  </read_reference>

  <read_prior_stages>
    <retrospective milestone="M05" stage="A"/>
    <retrospective milestone="M05" stage="B"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M05-gap-capability.md" section="C1.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M05-gap-capability.md" section="C1.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M05-gap-capability.md" section="Key constraints"/>
  <gates milestone="M05"/>
  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="stage_b_committed" gate="git log --oneline | head -1 must reference Stage M05.B commit"/>
    <check name="schema_regen_clean" gate="cargo xtask regenerate-types --check produces zero diff"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <inventory_row path="crates/runtime-sandbox/Cargo.toml" status="exists"/>
    <inventory_row path="crates/runtime-sandbox/src/lib.rs" status="exists"/>
    <inventory_row path="crates/runtime-sandbox/src/main.rs" status="new"/>
    <inventory_row path="crates/runtime-sandbox/src/protocol.rs" status="new"/>
    <inventory_row path="crates/runtime-sandbox/src/validator.rs" status="new"/>
    <inventory_row path="crates/runtime-sandbox/src/error.rs" status="new"/>
    <inventory_row path="crates/runtime-sandbox/tests/integration.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/sandbox_ipc/mod.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/sandbox_ipc/client.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/sandbox_ipc/connection.rs" status="new"/>
    <inventory_row path="src-tauri/src/sandbox_lifecycle.rs" status="new"/>
    <inventory_row path="src-tauri/src/main.rs" status="exists"/>
  </phase_doc_inventory_audit>

  <architecture_check>
    <claim description="Sandbox is a SEPARATE subprocess spawned by main at app startup; not an in-process module. Cross-process IPC isolates failures." verify="grep -rn 'tokio::process::Command' src-tauri/src/sandbox_lifecycle.rs ; expect at least one match (the spawn)"/>
    <claim description="Sandbox IPC client uses borrow-not-move next_event pattern from PR #64, NOT take_event_stream single-use" verify="grep -n 'next_event\\|take_event_stream' crates/runtime-main/src/sandbox_ipc/connection.rs ; expect next_event matches; zero or one take_event_stream (only for events() if needed)"/>
    <claim description="C1 ships ZERO unsafe blocks; all unsafe lives in Stage C2's seccomp/Job Objects files which don't exist yet" verify="grep -rn 'unsafe' crates/runtime-sandbox/src/ crates/runtime-main/src/sandbox_ipc/ ; expect zero matches in C1 commit"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="fn spawn" purpose="enumerate every subprocess-spawn call site in src-tauri to ensure sandbox_lifecycle's spawn shape matches drone_lifecycle's pattern"/>
    <grep pattern="next_event|take_event_stream" purpose="confirm sandbox_ipc's pattern matches PR #64's drone fix shape, not the old single-use"/>
  </fan_out_grep>

  <runtime_environment os="windows" note="C1 plumbing tests pass on any platform — no per-OS isolation yet. Stage C2 brings the platform-specific test gating via #[cfg(unix)] / #[cfg(windows)]"/>

  <gotchas>
    <trap>gotcha #72 first-class application: sandbox_ipc/connection.rs MUST use next_event borrow-not-move from the START. Do NOT copy the old take_event_stream pattern from M01-era drone code; copy from connection.rs POST PR #64.</trap>
    <trap>gotcha #69 first-class application: sandbox_ipc/client.rs ships with `validate_succeeds_twice_in_sequence` test FIRST, then implementation. Multi-call invariant codified.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT add libseccomp-rs / landlock / winapi deps in C1. Those land in Stage C2's dependency_audit_check. C1 Cargo.toml stays minimal (tokio + serde + thiserror only).</warning>
    <warning>DO NOT wire sandbox into production tool-dispatch flow in Stage C1. M05 ships the L3 BOUNDARY (the sandbox subprocess + IPC client). M09 (generators) wires the first caller. Stage C1's integration test exercises the round trip via a synthetic artifact; no production code invokes the sandbox.</warning>
  </execution_warnings>

  <time_box estimate_hours="5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Did the sandbox_ipc client's borrow-not-move pattern compose cleanly with the new validator request/response shape, or was there friction relative to drone_ipc's existing shape? Decisions for Stage C2: which platform's isolation should land first (Linux seccomp/landlock OR Windows Job Objects), and does the order affect cross-platform CI matrix outcomes?</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="C1.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state</item>
    <item>diff stat</item>
    <item>gate results (cargo fmt + clippy + test + doc + per-crate coverage on runtime-main::sandbox_ipc + plumbing files in runtime-sandbox + workspace ≥80% + frontend + validator)</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from M05-gap-capability.md C1.6</item>
    <item>explicit statement: "Stage M05.C1 is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### C1.6 Commit Message

```
feat(runtime): M05 Stage C1 — §8.security L3 sandbox crate plumbing + main-side IPC + lifecycle

Lights up crates/runtime-sandbox/ plumbing from M01 scaffold WITHOUT
OS-specific isolation (Stage C2 layers seccomp / landlock / Job
Objects on top). New subprocess + cross-platform IPC + main-side
client + Tauri-side lifecycle. The validator (pure-function) scans
artifact code for syscalls/imports against the agent's capability
declaration; returns Ok or Reject with reasons. IPC mirrors the
drone shape: framed JSON over Unix socket / Windows named pipe.

Crucially: the IPC client (crates/runtime-main/src/sandbox_ipc/)
ships with the borrow-not-move next_event pattern from the START
(PR #64 lessons applied, not retrofitted) + the multi-call
invariant test ships FIRST (gotcha #69 codified).

Coverage: ≥95% on runtime-main::sandbox_ipc + plumbing files in
runtime-sandbox (validator + protocol + error). Stage C2 extends
the runtime-sandbox gate to cover the OS-isolation files when those
land.

Tests:
- 3 validator unit tests (pure passes; filesystem-exceeds rejects;
  network-in-scope passes)
- 2 sandbox_ipc::client tests (single-call; **twice-in-sequence**)
- 1 sandbox_ipc::connection test (next_event borrow-not-move)
- 2 integration tests (round trip; restart after kill)

Stage C2 follows with seccomp / landlock / Job Objects + per-platform
test matrix.

https://claude.ai/code/session_019xg3cDSLdtX6JkNTLc9a6T
```

---

## Stage C2 — §8.security L3 Cross-platform OS isolation (seccomp / landlock / Job Objects) (new safety primitive ≥95%)

### C2.1 Problem Statement

Stage C1 shipped the sandbox subprocess + IPC plumbing + validator. Stage C2 adds the kernel-level isolation that makes the L3 boundary actually safe: **seccomp filter** + **landlock filesystem fence** on Linux; **Job Objects** on Windows. The isolation installs at the sandbox subprocess's `run()` startup, before any validation request is handled — so even a maliciously-crafted artifact reaching the validator is bounded by the kernel-level fence.

This is the largest piece of C1+C2 by complexity but the smaller piece by file count. Per-platform tests gate `#[cfg(unix)]` / `#[cfg(windows)]`; cross-platform integration tests exercise the round trip WITH isolation active.

### C2.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-sandbox/Cargo.toml` | exists (C1 edited) | Edit: add platform-specific deps (libseccomp-rs + landlock on Linux; winapi on Windows) |
| `crates/runtime-sandbox/src/lib.rs` | exists (C1 lit) | Edit: `run()` calls the new isolation-install functions at startup |
| `crates/runtime-sandbox/src/seccomp.rs` | **new** | `cfg(unix)` seccomp filter installation; `// SAFETY:` comments mandatory |
| `crates/runtime-sandbox/src/landlock.rs` | **new** | `cfg(unix)` landlock ruleset (filesystem path restrictions) |
| `crates/runtime-sandbox/src/job_objects.rs` | **new** | `cfg(windows)` Job Object setup; `// SAFETY:` comments mandatory |
| `crates/runtime-sandbox/tests/integration.rs` | exists (C1 created) | Edit: add integration tests WITH isolation active (per-platform `#[cfg(...)]` gated) |
| `Cargo.toml` (workspace) | exists | Edit: workspace deps for cross-platform syscall crates |
| `docs/build-prompts/retrospectives/M05.C2-retrospective.md` | **new** | Stage C2 retrospective |
| `CHANGELOG.md` | exists | Edit: M05.C2 — cross-platform OS isolation |

Effort budget C2: ~4–6 hours.

### C2.3 Detailed Changes

#### C2.3.1 Linux: seccomp + landlock

```rust
// crates/runtime-sandbox/src/seccomp.rs
#[cfg(unix)]
pub fn install_filter() -> Result<(), SandboxError> {
    // SAFETY: seccomp::ScmpFilterContext is constructed from a documented
    // libseccomp API; default action ALLOW for testing, narrowed to DENY
    // in production. We allow a curated syscall whitelist consistent with
    // the §8.security L3 spec: read, write, brk, mmap, exit, exit_group,
    // rt_sigreturn, futex, getpid, getuid, geteuid, getgid, getegid.
    // All other syscalls return EPERM via seccomp ENOSYS.
    let filter = ScmpFilterContext::new_filter(ScmpAction::KillProcess)?;
    for syscall in ALLOWED_SYSCALLS {
        filter.add_rule(ScmpAction::Allow, *syscall)?;
    }
    filter.load()?;
    Ok(())
}
```

Each `unsafe` block carries a `// SAFETY:` comment per CLAUDE.md §4 Rule 7. Landlock parallel — `cfg(unix)` ruleset restricting filesystem access to a tightly-scoped working directory.

#### C2.3.2 Windows: Job Objects

```rust
// crates/runtime-sandbox/src/job_objects.rs
#[cfg(windows)]
pub fn install_restrictions() -> Result<(), SandboxError> {
    // SAFETY: CreateJobObjectW + SetInformationJobObject are documented
    // Win32 APIs. We construct a JOBOBJECT_BASIC_LIMIT_INFORMATION
    // with JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE +
    // JOB_OBJECT_LIMIT_BREAKAWAY_OK so the subprocess can't escape via
    // CreateProcess, and the OS terminates if the job is closed.
    unsafe {
        let job = CreateJobObjectW(ptr::null_mut(), ptr::null());
        if job.is_null() {
            return Err(SandboxError::JobObjectCreate(GetLastError()));
        }
        // ... configure + AssignProcessToJobObject(job, GetCurrentProcess())
    }
    Ok(())
}
```

#### C2.3.3 Wire into `run()` startup

`crates/runtime-sandbox/src/lib.rs::run()` calls `seccomp::install_filter()` + `landlock::install_ruleset()` on Linux, OR `job_objects::install_restrictions()` on Windows, BEFORE entering the IPC loop. Isolation installs once at startup; the validator can't escape.

### C2.4 Tests

| Test | Type | Catches |
|---|---|---|
| `seccomp::tests::filter_blocks_disallowed_syscall` | Unit (Linux, `cfg(unix)`) | Kernel-level check |
| `seccomp::tests::filter_allows_whitelisted_syscall` | Unit (Linux, `cfg(unix)`) | Happy path |
| `landlock::tests::path_outside_ruleset_denied` | Unit (Linux, `cfg(unix)`) | Filesystem fence |
| `job_objects::tests::child_process_inherits_job` | Unit (Windows, `cfg(windows)`) | Job containment |
| `tests/integration.rs::isolation_active_blocks_disallowed_syscall_under_real_subprocess` | Integration (per-platform) | End-to-end with isolation on |
| `tests/integration.rs::isolation_persists_across_validate_calls` | Integration (per-platform) | Isolation isn't reset per call |

Coverage gate: extend `cargo llvm-cov --package runtime-sandbox` to cover the new `seccomp.rs` + `landlock.rs` + `job_objects.rs` files at ≥95% with `lib.rs` + `main.rs` excluded (M01.C OS-signal-orchestrator exclusion pattern).

### C2.5 CLI Prompt

```xml
<work_stage_prompt id="M05.C2">
  <context>
    Stage C2 of M05. Layers cross-platform OS isolation (seccomp +
    landlock on Linux; Job Objects on Windows) on top of Stage C1's
    sandbox plumbing. Isolation installs at sandbox subprocess
    startup, before any validation request is handled. Per-platform
    tests gate `#[cfg(unix)]` / `#[cfg(windows)]`; cross-platform
    integration tests exercise the round trip WITH isolation active.
    Stage D does NOT start until this stage's commit is on the
    milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M05-gap-capability.md (Stage C2 sections C2.1–C2.4)</file>
    <file>docs/build-prompts/retrospectives/M05.C1-retrospective.md (apply [END] Decisions — particularly the platform-order question)</file>
    <file>agent-runtime-spec.md §8.security L3 (sandbox subprocess); §1d (IPC topology)</file>
    <file>docs/MVP-v0.1.md §M5</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #21 cross-platform syscalls + unsafe SAFETY-comment discipline, #47 Windows file lock, #56 cargo llvm-cov subprocess flake)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="Stage C1 sandbox subprocess to wire isolation into">crates/runtime-sandbox/src/lib.rs</file>
    <file purpose="M01.C drone shutdown.rs OS-call-wrapper exclusion archetype for coverage gate">CLAUDE.md §5 (Coverage gate semantics)</file>
  </read_reference>

  <read_prior_stages>
    <retrospective milestone="M05" stage="A"/>
    <retrospective milestone="M05" stage="B"/>
    <retrospective milestone="M05" stage="C1"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M05-gap-capability.md" section="C2.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M05-gap-capability.md" section="C2.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M05-gap-capability.md" section="Key constraints"/>
  <gates milestone="M05"/>
  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="stage_c1_committed" gate="git log --oneline | head -1 must reference Stage M05.C1 commit"/>
    <check name="windows_sdk_present" gate="On Windows: cargo check --target x86_64-pc-windows-msvc passes for runtime-sandbox"/>
    <check name="linux_libseccomp_present" gate="On Linux: pkg-config libseccomp returns version (build dep)"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <inventory_row path="crates/runtime-sandbox/Cargo.toml" status="exists"/>
    <inventory_row path="crates/runtime-sandbox/src/lib.rs" status="exists"/>
    <inventory_row path="crates/runtime-sandbox/src/seccomp.rs" status="new"/>
    <inventory_row path="crates/runtime-sandbox/src/landlock.rs" status="new"/>
    <inventory_row path="crates/runtime-sandbox/src/job_objects.rs" status="new"/>
    <inventory_row path="crates/runtime-sandbox/tests/integration.rs" status="exists"/>
    <inventory_row path="Cargo.toml" status="exists"/>
  </phase_doc_inventory_audit>

  <dependency_audit_check>
    <dep name="libseccomp-rs" required_features="default" min_version="0.5" audit="cargo deny check passes; license MIT/Apache-2.0 compatible"/>
    <dep name="landlock" required_features="default" min_version="0.4" audit="cargo deny check passes; Linux-only"/>
    <dep name="winapi" required_features="jobapi2,winnt,handleapi,errhandlingapi" min_version="0.3" audit="cargo deny check passes; Windows-only"/>
  </dependency_audit_check>

  <architecture_check>
    <claim description="Isolation installs ONCE at sandbox subprocess startup, BEFORE the IPC loop accepts any request — not per-request" verify="grep -B2 -A5 'install_filter\\|install_restrictions' crates/runtime-sandbox/src/lib.rs ; expect call sites BEFORE the IPC accept loop"/>
    <claim description="seccomp/landlock/Job Objects code lives ONLY in crates/runtime-sandbox/; no unsafe blocks anywhere else in M05" verify="grep -rn 'unsafe' crates/runtime-main/src/capability/ crates/runtime-main/src/sandbox_ipc/ ; expect zero matches"/>
    <claim description="Every unsafe block in seccomp.rs / job_objects.rs carries a // SAFETY: comment naming the invariant" verify="grep -B1 'unsafe' crates/runtime-sandbox/src/seccomp.rs crates/runtime-sandbox/src/job_objects.rs ; expect SAFETY comment above every unsafe block"/>
  </architecture_check>

  <runtime_environment os="windows" note="Job Objects test path requires Windows; Linux CI uses seccomp+landlock. Cross-platform: cargo test --workspace exercises both on the respective platform; per-platform tests gate via #[cfg(unix)] / #[cfg(windows)]"/>

  <gotchas>
    <trap>gotcha #21 unsafe SAFETY comments: every `unsafe` block in seccomp.rs / job_objects.rs needs a `// SAFETY:` comment naming the invariant. clippy warns; CI gates this via `cargo clippy --workspace --all-targets -- -D warnings`.</trap>
    <trap>gotcha #47 Windows file lock on subprocess SIGKILL: integration tests that kill the sandbox subprocess need to handle the brief Windows file-lock retention. Use `Stop-Process -Force` followed by 2-3 second wait OR mark tests `#[ignore]` on Windows and run only in nightly.</trap>
    <trap>gotcha #56 cargo llvm-cov subprocess flake: per-crate coverage on runtime-sandbox may flake on Windows-local; use `cargo llvm-cov ... -- --test-threads=1 --skip sandbox_restart` to serialize problematic tests.</trap>
  </gotchas>

  <execution_warnings>
    <warning>seccomp filter list MUST be conservative — only the syscalls strictly needed for the validator to run. EVERY new syscall added to the allowlist needs a comment naming why. The default of KillProcess on disallowed syscall is intentional (failures must be visible, not silent).</warning>
  </execution_warnings>

  <time_box estimate_hours="5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Cross-platform parity outcomes — did Linux + Windows tests both pass without `#[ignore]` markers? seccomp allowlist size — how many syscalls did the validator actually need? unsafe block count + SAFETY-comment compliance. Decisions for Stage D: what does the tier system need to know about the sandbox's status (running / crashed / restarting)?</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="C2.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state</item>
    <item>diff stat</item>
    <item>gate results (cargo fmt + clippy + test + doc + runtime-sandbox per-crate coverage extended to cover OS-isolation files + workspace ≥80% + cross-platform CI matrix outcomes)</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from M05-gap-capability.md C2.6</item>
    <item>explicit statement: "Stage M05.C2 is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### C2.6 Commit Message

```
feat(runtime): M05 Stage C2 — §8.security L3 cross-platform OS isolation (seccomp / landlock / Job Objects)

Layers cross-platform kernel-level isolation on top of Stage C1's
sandbox plumbing:
- Linux: seccomp filter (curated syscall allowlist; KillProcess on
  disallowed) + landlock filesystem path restrictions
- Windows: Job Objects (JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE +
  JOB_OBJECT_LIMIT_BREAKAWAY_OK)

Isolation installs once at sandbox subprocess startup, BEFORE the
IPC loop accepts any validation request. The validator from C1
operates inside this kernel-level fence.

Coverage: runtime-sandbox ≥95% extended to cover the new isolation
files. Cross-platform CI passes on Linux + Windows; per-platform
tests gate via `#[cfg(unix)]` / `#[cfg(windows)]`.

Every `unsafe` block carries a `// SAFETY:` comment per CLAUDE.md
§4 Rule 7. seccomp filter list documented per-syscall.

Tests:
- 2 seccomp tests (filter blocks; filter allows)
- 1 landlock test (path outside ruleset denied)
- 1 Job Objects test (child inherits job)
- 2 integration tests (isolation blocks disallowed syscall; isolation
  persists across validate calls)

C1 + C2 together complete the §8.security L3 boundary. v0.1 ships
the BOUNDARY only; M09 (generators) wires the first production
caller.

https://claude.ai/code/session_019xg3cDSLdtX6JkNTLc9a6T
```

---

## Stage D — §8.security L4 Tier System (Novice + Promoted)

### D.1 Problem Statement

The L4 layer of §8.security caps WHICH capabilities can be granted at all, per the user's tier. v0.1 ships two tiers per §0d:

- **Novice** — curated allowlist. Read-only filesystem (no `write`/`exec`); HTTPS-only network (no `process_spawn`); the capability_kind/scope combinations a beginner-safe persona can use.
- **Promoted** — full capability surface. Any kind, any scope. The tier the user moves to once they understand what the runtime can do.

The Full tier (which adds runtime-specific advanced surfaces) is post-v0.1 per §0d.

The L4 tier evaluator sits BEFORE the L1+L2a enforcer in the dispatch chain: tier check → enforcer check → dispatch. A Promoted user with a `write` declaration still passes through the L1 check; a Novice user requesting `write` is rejected at L4 before L1 even runs.

Tier transitions (Novice → Promoted) happen via HITL. The renderer surfaces a "Promote" affordance; the user clicks; HITL prompts confirm; tier flips. Demotion (Promoted → Novice) is also user-initiated (a "downgrade-on-uncertainty" affordance for sensitive sessions).

Concrete deliverables:
1. `crates/runtime-main/src/tier/` — new module: TierEvaluator, tier matrix, transition logic
2. `crates/runtime-main/src/tier/persistence.rs` — stores the user's current tier (drone-side `users.audit` table or simple flat file in app data dir)
3. Integration with the Stage B capability enforcer: tier check runs first
4. `tier_violation` + `tier_transition` event variants (added to `schemas/event.v1.json`)
5. First-run prompt (the §14 first-run UX) defaults to Novice; user can promote later

### D.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/tier/mod.rs` | **new** | Module root |
| `crates/runtime-main/src/tier/evaluator.rs` | **new** | TierEvaluator — tier → allowed CapabilityKind/Scope matrix |
| `crates/runtime-main/src/tier/matrix.rs` | **new** | The Novice + Promoted allowlist tables (data, not code) |
| `crates/runtime-main/src/tier/persistence.rs` | **new** | Read/write the user's current tier |
| `crates/runtime-main/src/tier/error.rs` | **new** | TierError enum |
| `crates/runtime-main/src/capability/enforcer.rs` | exists (Stage B) | Edit: enforcer's `check()` runs `TierEvaluator::allows()` first |
| `schemas/event.v1.json` | exists | Edit: add `tier_violation` + `tier_transition` event variants |
| `crates/runtime-core/src/generated/event.rs` | exists | Regenerated |
| `src/types/agent_event.ts` | exists | Regenerated |
| `src/lib/graphStore.ts` | exists | Edit: applyEvent branches for tier_violation + tier_transition |
| `src-tauri/src/commands.rs` | exists | Edit: new Tauri commands `get_current_tier` + `request_tier_transition` |
| `crates/runtime-main/tests/tier_smoke.rs` | **new** | Integration test |
| `tests/unit/lib/graphStore.test.ts` | exists | Add: tier event applyEvent tests |
| `docs/build-prompts/retrospectives/M05.D-retrospective.md` | **new** | Stage D retrospective |
| `CHANGELOG.md` | exists | Edit: M05.D — tier system |

Effort: ~3–4h.

### D.3 Detailed Changes

#### D.3.1 Tier evaluator + matrix

```rust
// crates/runtime-main/src/tier/evaluator.rs
use runtime_core::generated::capability::{CapabilityDeclaration, CapabilityKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tier { Novice, Promoted }

pub struct TierEvaluator;

impl TierEvaluator {
    /// Check whether the supplied tier permits the requested capability
    /// AT ALL. Returns Ok if the tier permits; Err if forbidden.
    /// Called BEFORE the L1 enforcer's check — tier acts as the outer
    /// gate.
    pub fn allows(tier: Tier, capability: &CapabilityDeclaration) -> Result<(), TierError> {
        match tier {
            Tier::Novice => Self::novice_allows(capability),
            Tier::Promoted => Ok(()), // Promoted: any capability permitted at the tier gate; L1 still narrows by declaration
        }
    }

    fn novice_allows(c: &CapabilityDeclaration) -> Result<(), TierError> {
        use CapabilityKind::*;
        match c.kind {
            Read => Ok(()),
            Network if Self::is_https_only_scope(&c.scope) => Ok(()),
            // Forbidden in Novice: Write, Exec, ProcessSpawn, plain Network.
            _ => Err(TierError::ForbiddenInTier { tier: Tier::Novice, capability: c.clone() }),
        }
    }

    fn is_https_only_scope(scope: &CapabilityScope) -> bool {
        match scope {
            CapabilityScope::DomainScope { .. } => true, // domain-scoped network = OK
            _ => false,
        }
    }
}
```

The matrix is data-tabular — easy to expand when v1.0 adds Full tier. Don't bake matrix rules into nested if/else; use a table-lookup pattern.

#### D.3.2 Persistence

Store tier in the app data dir (`%APPDATA%\AgentRuntime\tier.json` on Windows; `~/.config/agent-runtime/tier.json` on Linux). Simple JSON file `{ "tier": "novice", "since_unix_ms": ... }`. Read at app startup; write on tier transition.

```rust
// crates/runtime-main/src/tier/persistence.rs
pub fn load_tier(app_data_dir: &Path) -> Result<Tier, TierError> {
    let path = app_data_dir.join("tier.json");
    if !path.exists() {
        return Ok(Tier::Novice); // first-run default
    }
    let raw = fs::read_to_string(&path)?;
    let stored: StoredTier = serde_json::from_str(&raw)?;
    Ok(stored.tier)
}

pub fn save_tier(app_data_dir: &Path, tier: Tier) -> Result<(), TierError> {
    let path = app_data_dir.join("tier.json");
    fs::create_dir_all(app_data_dir)?;
    let stored = StoredTier { tier, since_unix_ms: now_unix_ms() };
    fs::write(&path, serde_json::to_string_pretty(&stored)?)?;
    Ok(())
}
```

#### D.3.3 Wiring into enforcer

```rust
// crates/runtime-main/src/capability/enforcer.rs (Stage B + Stage D edit)
impl CapabilityEnforcer {
    pub fn check(
        &self,
        agent: AgentId,
        requested: &CapabilityDeclaration,
    ) -> Result<(), CapabilityError> {
        // L4 first
        TierEvaluator::allows(self.current_tier, requested)
            .map_err(CapabilityError::TierForbidden)?;
        // Then L1
        let grants = self.grants_by_agent.get(&agent)
            .ok_or(CapabilityError::Denied { reason: DenyReason::NoDeclarations })?;
        grants.iter().find(|g| g.satisfies(requested))
            .map(|_| ())
            .ok_or(CapabilityError::Denied { reason: DenyReason::NoMatchingGrant })
    }
}
```

#### D.3.4 Tier-transition flow

Renderer affordance → `request_tier_transition(target: "promoted")` Tauri command → HITL prompt → user approves → tier flipped + persisted + `tier_transition` event emitted.

Demotion path (Promoted → Novice) skips HITL — it's always safer; user can demote freely.

### D.4 Tests

| Test | Type | Catches |
|---|---|---|
| `evaluator::tests::novice_allows_read` | Unit | Allowlist row |
| `evaluator::tests::novice_denies_write` | Unit | Denylist row |
| `evaluator::tests::novice_allows_https_network` | Unit | Scope-conditional allowlist |
| `evaluator::tests::novice_denies_plain_network` | Unit | Scope-conditional deny |
| `evaluator::tests::promoted_allows_all_kinds` | Unit | Promoted bypass |
| `persistence::tests::load_returns_novice_on_first_run` | Unit | First-run default |
| `persistence::tests::save_and_load_round_trip` | Unit | Round trip |
| `persistence::tests::save_and_load_twice_in_sequence` | Unit | Multi-call invariant (gotcha #69) |
| `tests/tier_smoke.rs::tier_check_runs_before_l1_enforcer` | Integration | Layer ordering |
| `tests/tier_smoke.rs::novice_request_for_write_returns_tier_violation` | Integration | End-to-end via SDK wrap |
| `graphStore.test.ts::applies_tier_violation_updates_state` | Unit (TS) | Renderer reducer |
| `graphStore.test.ts::applies_tier_transition_flips_current_tier` | Unit (TS) | Renderer reducer |

Coverage: per-module ≥95% on the tier evaluator (pure-function data lookup; trivial to test high-coverage).

### D.5 CLI Prompt

```xml
<work_stage_prompt id="M05.D">
  <context>
    Stage D of M05. Implements §8.security L4 tier system (Novice +
    Promoted only — Full tier is post-v0.1 per §0d). Tier evaluator sits
    BEFORE the Stage B L1+L2a capability enforcer in the dispatch chain.
    Novice tier: curated allowlist (read + HTTPS-only network);
    Promoted tier: full capability surface (L1 still narrows). First-run
    defaults to Novice; promotion requires HITL approval; demotion is
    user-initiated (no HITL). Persists in app data dir
    (`%APPDATA%\AgentRuntime\tier.json` on Windows; XDG config on
    Linux). Stage E does NOT start until this stage's commit is on the
    milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M05-gap-capability.md (Stage D sections D.1–D.4)</file>
    <file>docs/build-prompts/retrospectives/M05.C-retrospective.md (apply [END] Decisions)</file>
    <file>agent-runtime-spec.md §8.security L4; §0d (tier scope lock to two tiers); §14 (first-run UX)</file>
    <file>docs/MVP-v0.1.md §M5</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="Enforcer to wrap with tier check">crates/runtime-main/src/capability/enforcer.rs</file>
    <file purpose="HITL trigger for tier-transition flow">crates/runtime-main/src/hitl/policy.rs</file>
  </read_reference>

  <read_prior_stages>
    <retrospective milestone="M05" stage="A"/>
    <retrospective milestone="M05" stage="B"/>
    <retrospective milestone="M05" stage="C1"/>
    <retrospective milestone="M05" stage="C2"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M05-gap-capability.md" section="D.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M05-gap-capability.md" section="D.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M05-gap-capability.md" section="Key constraints"/>
  <gates milestone="M05"/>
  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="stage_c2_committed" gate="git log --oneline | head -1 must reference Stage M05.C2 commit"/>
    <check name="schema_regen_clean" gate="cargo xtask regenerate-types --check produces zero diff"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <inventory_row path="crates/runtime-main/src/tier/mod.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/tier/evaluator.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/tier/matrix.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/tier/persistence.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/tier/error.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/capability/enforcer.rs" status="exists"/>
    <inventory_row path="schemas/event.v1.json" status="exists"/>
    <inventory_row path="src/lib/graphStore.ts" status="exists"/>
    <inventory_row path="src-tauri/src/commands.rs" status="exists"/>
    <inventory_row path="crates/runtime-main/tests/tier_smoke.rs" status="new"/>
  </phase_doc_inventory_audit>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <architecture_check>
    <claim description="Tier check runs BEFORE L1 enforcer check in the dispatch chain — tier acts as outer gate" verify="grep -B2 -A2 'fn check' crates/runtime-main/src/capability/enforcer.rs | grep -E 'TierEvaluator::allows.*before.*grants_by_agent.get' ; OR inspect the function body's statement order"/>
    <claim description="Tier persisted in app data dir — Windows: %APPDATA%, Linux: XDG_CONFIG_HOME or ~/.config" verify="grep -n 'tier.json' crates/runtime-main/src/tier/persistence.rs ; expect path resolution via dirs crate or std::env::var('APPDATA')"/>
    <claim description="Tier demotion does NOT require HITL; only promotion does" verify="grep -A5 'demote\\|to_novice' crates/runtime-main/src/tier/ ; expect no HitlSeam invocation"/>
  </architecture_check>

  <runtime_environment os="windows" note="Tier persistence path uses dirs::config_dir() on both platforms; on Windows resolves to %APPDATA%"/>

  <gotchas>
    <trap>Matrix as data, not code: novice's allowlist lives in a table (matrix.rs); the evaluator looks up. If you nest if/else, expanding to Full tier later will be painful.</trap>
    <trap>Tier evaluator returns Err for novice-forbidden; the enforcer wraps as `CapabilityError::TierForbidden` distinct from `Denied`. Renderer needs both event variants.</trap>
    <trap>Persistence first-run default: tier.json absent → default to Novice. Do NOT default to Promoted. Default-safe matches §8.security spirit.</trap>
  </gotchas>

  <time_box estimate_hours="4"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Matrix expansion path — how much code change would Full tier (v1.0+) add? Tier-transition HITL coupling — does the M04.E HitlSeam compose cleanly with tier promotion, or does promotion need a new seam variant?</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="D.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state</item>
    <item>diff stat</item>
    <item>gate results (cargo + frontend + validator + per-module coverage on tier/)</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from M05-gap-capability.md D.6</item>
    <item>explicit statement: "Stage M05.D is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### D.6 Commit Message

```
feat(runtime): M05 Stage D — §8.security L4 tier system (Novice + Promoted)

New module crates/runtime-main/src/tier/ implements the L4 tier
gate. Two tiers per §0d: Novice (curated allowlist — read + HTTPS-only
network) and Promoted (full capability surface). Tier evaluator
sits BEFORE the Stage B L1+L2a enforcer; tier check is the outer
gate.

Persistence in app data dir (Windows: %APPDATA%\AgentRuntime\tier.json;
Linux: $XDG_CONFIG_HOME/agent-runtime/tier.json). First-run defaults
to Novice.

Schema:
- schemas/event.v1.json: tier_violation + tier_transition variants.

Tauri commands:
- get_current_tier (read)
- request_tier_transition (promote requires HITL; demote does not)

Tests:
- 5 evaluator tests (matrix coverage)
- 3 persistence tests (first-run, round trip, twice-in-sequence)
- 2 integration tests (layer ordering, novice-write denial)
- 2 graphStore tests for new event branches

Coverage: ≥95% on tier/ per-module gate (pure-function data lookup).

https://claude.ai/code/session_019xg3cDSLdtX6JkNTLc9a6T
```

---

## Stage E — §8.security L5 Provenance + skills.audit.jsonl Audit Log

### E.1 Problem Statement

Every gap, every capability_grant, every capability_violation, every tier_transition appends a line to `skills.audit.jsonl`. The audit log is append-only and survives across sessions; provides the trail a maintainer can review when reproducing "why did X happen?". Per spec §13.5 dev-logging discipline.

The L5 layer adds **provenance metadata** to every grant: who granted it (loader / request_capability / tier-promotion), when, what's the framework JSON path or session id, what's the justification.

v0.1 ships:
- File-based jsonl writer (no SQLite; flat file in app data dir)
- One line per event; structured JSON; UTC timestamps
- No rotation in v0.1 (file grows unbounded; users can manually delete)
- Read-only via SQL inspector? No — the audit log is OUTSIDE drone's SQLite. Could be added in M06+.

Concrete deliverables:
1. `crates/runtime-main/src/audit/` — new module: writer, entry schema
2. `crates/runtime-main/src/audit/file_path.rs` — platform-appropriate audit file path
3. Wiring: capability enforcer (Stage B) + tier evaluator (Stage D) + framework_loader (Stage A) all write to audit log on grant/violation/transition
4. Spec §13.5 dev-logging — ensure the audit format is human-grep-able

### E.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/audit/mod.rs` | **new** | Module root |
| `crates/runtime-main/src/audit/writer.rs` | **new** | AuditWriter: open-append + write line + flush |
| `crates/runtime-main/src/audit/entry.rs` | **new** | AuditEntry type — discriminated union over GrantedCapability / DeniedCapability / TierTransition / GapDetected / GapResolved |
| `crates/runtime-main/src/audit/file_path.rs` | **new** | Resolve audit file path per platform |
| `crates/runtime-main/src/audit/error.rs` | **new** | AuditError enum |
| `crates/runtime-main/src/tier/transition.rs` | **new** | Tier-transition primitive paired with audit emission; 99.24% line coverage at E-end (the 1 uncovered line is the `tracing::error!` branch on underlying audit-write failure). (M06.A truth-up per M05.V Finding #3 — added to inventory after the original E.2 table closed.) |
| `schemas/audit.v1.json` | **new** | AuditEntry schema |
| `crates/runtime-core/src/generated/audit.rs` | **new** | Regenerated |
| `crates/runtime-main/src/capability/enforcer.rs` | exists | Edit: call `audit::log_grant()` / `audit::log_denial()` |
| `crates/runtime-main/src/tier/evaluator.rs` | exists | Edit: call `audit::log_tier_transition()` |
| `crates/runtime-main/src/framework_loader/mod.rs` | exists (Stage A) | Edit: call `audit::log_gap_detected()` |
| `crates/runtime-main/tests/audit_smoke.rs` | **new** | Integration test |
| `docs/build-prompts/retrospectives/M05.E-retrospective.md` | **new** | Stage E retrospective |
| `CHANGELOG.md` | exists | Edit: M05.E — audit log + provenance |

Effort: ~3–4h.

### E.3 Detailed Changes

#### E.3.1 AuditEntry shape

```jsonc
// schemas/audit.v1.json
{
  "$id": "https://agent-runtime.local/schemas/audit.v1.json",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AuditEntry",
  "type": "object",
  "required": ["timestamp_unix_ms", "session_id", "kind"],
  "properties": {
    "timestamp_unix_ms": { "type": "integer", "minimum": 0 },
    "session_id": { "$ref": "common.v1.json#/$defs/NonEmptyString" },
    "kind": { "$ref": "#/$defs/AuditEntryKind" },
    "details": { "type": "object", "additionalProperties": true }
  },
  "$defs": {
    "AuditEntryKind": {
      "type": "string",
      "enum": [
        "capability_granted",
        "capability_denied",
        "tier_transition",
        "gap_detected",
        "gap_resolved",
        "framework_loaded"
      ]
    }
  }
}
```

`details` is open-shape per-kind (different kinds have different relevant fields); the consumer reads kind + interprets details accordingly.

#### E.3.2 Writer

```rust
// crates/runtime-main/src/audit/writer.rs
pub struct AuditWriter {
    file: Mutex<File>, // append-only mode
}

impl AuditWriter {
    pub fn open(path: &Path) -> Result<Self, AuditError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file: Mutex::new(file) })
    }

    pub async fn log(&self, entry: AuditEntry) -> Result<(), AuditError> {
        let line = serde_json::to_string(&entry)?;
        let mut file = self.file.lock().await;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        Ok(())
    }
}
```

Mutex-guarded; single writer per process; one line per entry. Newline-delimited JSON for grep-ability.

#### E.3.3 File path

Resolve via `dirs::data_dir()`:
- Windows: `%APPDATA%\AgentRuntime\skills.audit.jsonl`
- Linux: `$XDG_DATA_HOME/agent-runtime/skills.audit.jsonl` or `~/.local/share/agent-runtime/`
- macOS: `~/Library/Application Support/AgentRuntime/skills.audit.jsonl`

If the directory doesn't exist, create it.

#### E.3.4 Wiring

Capability enforcer (Stage B), tier evaluator (Stage D), framework_loader (Stage A) all hold an `Arc<AuditWriter>` reference. On grant/denial/transition/gap, call `audit_writer.log(entry).await`. Don't fail the dispatch if audit write fails — log error to tracing and continue (audit is a best-effort observability layer; v0.1 doesn't gate dispatch on audit availability).

### E.4 Tests

| Test | Type | Catches |
|---|---|---|
| `writer::tests::log_single_entry_writes_one_line` | Unit | Happy path |
| `writer::tests::two_sequential_entries_two_lines` | Unit | Multi-call invariant (gotcha #69) |
| `writer::tests::concurrent_writes_serialized_by_mutex` | Unit | Concurrency safety |
| `entry::tests::serializes_to_compact_json` | Unit | JSON shape |
| `file_path::tests::resolves_per_platform` (cfg-conditional) | Unit | Cross-platform paths |
| `tests/audit_smoke.rs::capability_grant_writes_audit_line` | Integration | End-to-end |
| `tests/audit_smoke.rs::tier_transition_writes_audit_line` | Integration | End-to-end |

Coverage: workspace ≥80%; per-module ≥95% on writer + entry (pure-function logic).

### E.5 CLI Prompt

```xml
<work_stage_prompt id="M05.E">
  <context>
    Stage E of M05. Implements §8.security L5 — provenance + audit log.
    New module crates/runtime-main/src/audit/ writes one JSONL line per
    capability grant/denial/transition/gap event to skills.audit.jsonl
    in the app data dir. Append-only; no rotation in v0.1. Wires Stage A
    framework_loader + Stage B capability enforcer + Stage D tier
    evaluator to call audit::log() on every relevant action. Stage F
    does NOT start until this stage's commit is on the milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M05-gap-capability.md (Stage E sections E.1–E.4)</file>
    <file>docs/build-prompts/retrospectives/M05.D-retrospective.md (apply [END] Decisions)</file>
    <file>agent-runtime-spec.md §8.security L5; §13.5 dev-logging discipline</file>
    <file>docs/MVP-v0.1.md §M5</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="Stage B enforcer that audit wires into">crates/runtime-main/src/capability/enforcer.rs</file>
    <file purpose="Stage D tier evaluator that audit wires into">crates/runtime-main/src/tier/evaluator.rs</file>
    <file purpose="Stage A framework_loader that audit wires into">crates/runtime-main/src/framework_loader/mod.rs</file>
  </read_reference>

  <read_prior_stages>
    <retrospective milestone="M05" stage="A"/>
    <retrospective milestone="M05" stage="B"/>
    <retrospective milestone="M05" stage="C1"/>
    <retrospective milestone="M05" stage="C2"/>
    <retrospective milestone="M05" stage="D"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M05-gap-capability.md" section="E.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M05-gap-capability.md" section="E.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M05-gap-capability.md" section="Key constraints"/>
  <gates milestone="M05"/>
  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="stage_d_committed" gate="git log --oneline | head -1 must reference Stage M05.D commit"/>
    <check name="schema_regen_clean" gate="cargo xtask regenerate-types --check produces zero diff"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <inventory_row path="crates/runtime-main/src/audit/mod.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/audit/writer.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/audit/entry.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/audit/file_path.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/audit/error.rs" status="new"/>
    <inventory_row path="schemas/audit.v1.json" status="new"/>
    <inventory_row path="crates/runtime-core/src/generated/audit.rs" status="new"/>
    <inventory_row path="crates/runtime-main/src/capability/enforcer.rs" status="exists"/>
    <inventory_row path="crates/runtime-main/src/tier/evaluator.rs" status="exists"/>
    <inventory_row path="crates/runtime-main/src/framework_loader/mod.rs" status="exists"/>
    <inventory_row path="crates/runtime-main/tests/audit_smoke.rs" status="new"/>
  </phase_doc_inventory_audit>

  <schema_drift_check gate="cargo xtask regenerate-types --check"/>

  <schema_audit>
    <survey pattern='"AuditEntryKind"' purpose="confirm AuditEntryKind $def not already declared elsewhere"/>
  </schema_audit>

  <schema_root_check/>

  <architecture_check>
    <claim description="Audit write failures DO NOT fail the calling dispatch — best-effort observability" verify="grep -A5 'audit_writer.log' crates/runtime-main/src/capability/enforcer.rs ; expect Result handled with tracing::error, not propagated"/>
    <claim description="Single writer per process via Mutex; multiple concurrent callers serialize" verify="grep -n 'Mutex<File>\\|Mutex<.*File' crates/runtime-main/src/audit/writer.rs ; expect at least one match"/>
    <claim description="Audit file is jsonl (newline-delimited JSON), NOT JSON array — append-only for grep-ability" verify="grep -A3 'fn log' crates/runtime-main/src/audit/writer.rs ; expect write_all(b'\\\\n') OR similar newline emission"/>
  </architecture_check>

  <gotchas>
    <trap>Best-effort audit: do NOT propagate audit write errors to the dispatch caller. Use tracing::error to log failures and continue. Audit availability is observability, not a hard gate. (Spec §13.5 dev-logging discipline.)</trap>
    <trap>JSONL not JSON array: each entry is its own line; consumer streams the file with `.lines()` and parses each. A JSON array would require rewriting the whole file on every append. Don't do that.</trap>
    <trap>Mutex around the file handle: multiple async callers writing simultaneously would interleave bytes without a lock. Use tokio::sync::Mutex (not std::sync::Mutex — the async context matters).</trap>
  </gotchas>

  <time_box estimate_hours="4"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Did the audit write actually fire on every grant/denial/transition path? Coverage on audit/ — what's the per-file breakdown? Decisions for Stage F (renderer UI): what audit data does GapPanel surface to the user, and what stays purely observability (not displayed)?</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="E.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state</item>
    <item>diff stat</item>
    <item>gate results</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from M05-gap-capability.md E.6</item>
    <item>explicit statement: "Stage M05.E is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### E.6 Commit Message

```
feat(runtime): M05 Stage E — §8.security L5 provenance + skills.audit.jsonl audit log

New module crates/runtime-main/src/audit/ ships an append-only JSONL
writer to skills.audit.jsonl in the app data dir (Windows: %APPDATA%;
Linux: XDG_DATA_HOME; macOS: ~/Library/Application Support).
Best-effort observability — write failures DO NOT fail the calling
dispatch.

Wires:
- Stage A framework_loader: gap_detected entries on every emitted gap.
- Stage B capability enforcer: capability_granted + capability_denied
  entries on every check outcome.
- Stage D tier evaluator: tier_transition entries on every flip.

Schema:
- schemas/audit.v1.json (new): AuditEntry shape.

JSONL format (one entry per line, newline-delimited) so the file is
greppable by humans and streamable by consumers.

v0.1 ships unbounded growth; rotation/archival is post-v0.1.

Tests:
- 3 writer tests (single, twice-in-sequence, concurrent)
- 1 entry serialization test
- 1 file_path platform-resolution test
- 2 integration tests (capability grant; tier transition)

Coverage: ≥95% on writer + entry per-module gates.

https://claude.ai/code/session_019xg3cDSLdtX6JkNTLc9a6T
```

---

## Stage F — Renderer UI (GapPanel + capability-violation modal wiring + CapabilityBadge)

### F.1 Problem Statement

The renderer surfaces M05's runtime state to the user via three components:

1. **`GapPanel`** — right-side rail listing unresolved gaps (subscribed to the graphStore's gap-event state). Each item shows kind + name + severity + suggested_action + agent_id; click expands to show the framework path that referenced the gap; resolved gaps fall out of the panel automatically when `gap_resolved` events fire.

2. **Capability-violation modal** — when a `capability_violation` event fires, the renderer shows the M04.E `HITLModal` variant (reused per ADR-0007 — no new modal component). The modal's content is parameterized: the requested action, the declared scope, the agent_id, and three response buttons (Allow once + grant capability / Deny / Abort session).

3. **`CapabilityBadge`** — per-node visual decoration on AgentNodes showing the agent's current tier (Novice / Promoted) + a count of active capability grants. Renders as a small pill on the node's corner.

Stage F is the FINAL work stage of M05 — renderer-only, no Rust changes beyond confirming the wire path. Stage V runs after Stage F; closeout follows.

Concrete deliverables:
1. `src/components/GapPanel.tsx` — new right-rail component
2. `src/components/nodes/CapabilityBadge.tsx` — new per-node decoration
3. Wiring `capability_violation` event to `HITLModal` via on_capability_violation trigger (the trigger already routes to modal variant in M04.E policy.rs)
4. `App.tsx` layout — add GapPanel alongside ApprovalPanel + InspectorPanel
5. CSS for GapPanel + CapabilityBadge in `src/styles.css`
6. Vitest + DOM-render tests per gotcha #67 (CSS rules must exist) + #68 (component reads right fields)

### F.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/components/GapPanel.tsx` | **new** | Right-rail component subscribing to graphStore.gaps |
| `src/components/nodes/CapabilityBadge.tsx` | **new** | Per-node tier + grant-count badge |
| `src/lib/graphStore.ts` | exists | Edit: ensure `gaps` and `capabilityGrants` selector hooks expose what GapPanel + CapabilityBadge consume |
| `src/components/nodes/AgentNode.tsx` | exists | Edit: render `CapabilityBadge` as child element |
| `src/App.tsx` | exists | Edit: mount `<GapPanel />` alongside `<ApprovalPanel />` + `<HITLPanel />` |
| `src/styles.css` | exists | Add: `.gap-panel`, `.gap-panel__item`, `.gap-panel__severity-*`, `.capability-badge`, `.capability-badge--novice`, `.capability-badge--promoted` rules |
| `tests/unit/components/GapPanel.test.tsx` | **new** | Behavior tests + computed-style CSS assertions per gotcha #67 |
| `tests/unit/nodes/CapabilityBadge.test.tsx` | **new** | Behavior tests |
| `tests/e2e/gap_panel.spec.ts` | **new** | Playwright spec — inject gap event, assert panel surfaces |
| `docs/build-prompts/retrospectives/M05.F-retrospective.md` | **new** | Stage F retrospective |
| `CHANGELOG.md` | exists | Edit: M05.F — renderer UI |

Effort: ~3–5h.

### F.3 Detailed Changes

#### F.3.1 GapPanel — subscribes to graphStore.gaps

```typescript
// src/components/GapPanel.tsx
import { useGraphStore } from '../lib/graphStore';

export function GapPanel(): JSX.Element | null {
  const gaps = useGraphStore((s) => s.gaps);
  if (gaps.size === 0) {
    return null; // Hidden when no unresolved gaps
  }
  return (
    <aside className="gap-panel" data-testid="gap-panel" role="region" aria-label="Unresolved capability gaps">
      <h2 className="gap-panel__title">Unresolved Gaps ({gaps.size})</h2>
      <ul className="gap-panel__list">
        {[...gaps.values()].map((gap) => (
          <li
            key={gap.id}
            className={`gap-panel__item gap-panel__item--${gap.severity}`}
            data-testid={`gap-item-${gap.id}`}
          >
            <span className="gap-panel__kind">{gap.kind}</span>
            <span className="gap-panel__name">{gap.name}</span>
            <span className="gap-panel__suggested-action" title={gap.suggested_action}>
              {gap.suggested_action}
            </span>
            <span className="gap-panel__agent" data-testid={`gap-item-${gap.id}-agent`}>
              agent: {gap.agent_id.slice(0, 8)}
            </span>
          </li>
        ))}
      </ul>
    </aside>
  );
}
```

Selector pattern: `useGraphStore((s) => s.gaps)`. graphStore exposes `gaps: Map<string, GapEntry>` keyed by `${kind}:${name}:${agent_id}`. Idempotence handled by the Stage A applyEvent branches; this component is pure-render.

#### F.3.2 CapabilityBadge — per-node decoration

```typescript
// src/components/nodes/CapabilityBadge.tsx
interface CapabilityBadgeProps {
  agentId: string;
}

export function CapabilityBadge({ agentId }: CapabilityBadgeProps): JSX.Element {
  const tier = useGraphStore((s) => s.currentTier);
  const grantCount = useGraphStore((s) => s.capabilityGrants.filter(g => g.granted_to === agentId).length);
  return (
    <span
      className={`capability-badge capability-badge--${tier}`}
      data-testid={`capability-badge-${agentId}`}
      title={`Tier: ${tier} (${grantCount} grants)`}
    >
      {tier === 'novice' ? 'N' : 'P'}
      {grantCount > 0 && <span className="capability-badge__count">{grantCount}</span>}
    </span>
  );
}
```

Rendered inside AgentNode as `<CapabilityBadge agentId={agentId} />`.

#### F.3.3 Capability-violation modal — reuses M04.E HITLModal

No new modal component. The Stage B enforcer emits `capability_violation` event AND fires the `on_capability_violation` HITL trigger (M04.E policy.rs already routes that trigger to modal variant). The HITLModal renders with the violation's question text + options ("Allow once", "Deny", "Abort"). User clicks; `respond_hitl` Tauri command resolves; enforcer continues per response. Per ADR-0007.

Stage F just confirms the wire path by adding a Playwright test that:
1. Injects a `capability_violation` event via `window.__graphStore.applyEvent`
2. Asserts the HITLModal mounts with the right question text
3. Clicks "Deny" 
4. Asserts the modal dismisses + the violation is recorded in graphStore.capabilityViolations

#### F.3.4 CSS — per gotcha #67

```css
/* src/styles.css (additions) */

.gap-panel {
  position: fixed;
  right: 1rem;
  top: 4rem;
  width: 320px;
  max-height: 60vh;
  overflow-y: auto;
  padding: 0.75rem;
  background: var(--node-bg-alt);
  color: var(--node-fg);
  border: 1px solid var(--node-base-border);
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  z-index: 50;
}
.gap-panel__title { font-size: 14px; margin: 0 0 0.5rem 0; font-weight: 600; }
.gap-panel__list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.4rem; }
.gap-panel__item { display: grid; grid-template-columns: auto 1fr auto; gap: 0.4rem; padding: 0.5rem; border-radius: 4px; border-left: 3px solid var(--node-base-border); }
.gap-panel__item--critical { border-left-color: var(--node-error); }
.gap-panel__item--important { border-left-color: var(--node-gap); }
.gap-panel__item--advisory { border-left-color: var(--node-active); }
.gap-panel__item--requested { border-left-color: var(--node-complete); }

.capability-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 0.3rem;
  font-size: 10px;
  font-weight: 700;
  color: #fff;
  border-radius: 9px;
  margin-left: 0.4rem;
}
.capability-badge--novice { background-color: #4a90e2; }
.capability-badge--promoted { background-color: #ff9800; }
.capability-badge__count { margin-left: 0.2rem; font-weight: 400; opacity: 0.9; }
```

### F.4 Tests

| Test | Type | Catches |
|---|---|---|
| `GapPanel.test.tsx::renders_nothing_when_no_gaps` | Vitest+jsdom | Empty state |
| `GapPanel.test.tsx::renders_one_item_per_gap` | Vitest+jsdom | List rendering |
| `GapPanel.test.tsx::item_shows_kind_name_severity_agent_id` | Vitest+jsdom | Per gotcha #68 (component reads right fields) |
| `GapPanel.test.tsx::every_severity_class_has_corresponding_CSS_rule_in_styles_css` | Vitest static | **Per gotcha #67 — catches the M04.F BUD-01 class** |
| `GapPanel.test.tsx::dismisses_item_on_gap_resolved` | Vitest+jsdom | applyEvent dismissal flow |
| `CapabilityBadge.test.tsx::renders_N_for_novice_tier` | Vitest+jsdom | Tier display |
| `CapabilityBadge.test.tsx::renders_P_for_promoted_tier` | Vitest+jsdom | Tier display |
| `CapabilityBadge.test.tsx::shows_grant_count_when_nonzero` | Vitest+jsdom | Conditional rendering |
| `CapabilityBadge.test.tsx::tier_classes_have_CSS_rules_in_styles_css` | Vitest static | gotcha #67 |
| `tests/e2e/gap_panel.spec.ts::injecting_gap_event_surfaces_panel` | Playwright | Renderer-level E2E |
| `tests/e2e/gap_panel.spec.ts::capability_violation_event_mounts_hitl_modal` | Playwright | Capability-violation modal wire-path |

Coverage: renderer ≥80% (vitest); no per-module ≥95% requirement (Stage F is rendering, not safety primitive).

### F.5 CLI Prompt

```xml
<work_stage_prompt id="M05.F">
  <context>
    Stage F of M05. Final work stage — renderer-only UI. Ships
    GapPanel (right-rail list of unresolved gaps), CapabilityBadge
    (per-AgentNode tier+grant-count visual), and confirms the
    capability-violation modal wire-path through reuse of M04.E
    HITLModal per ADR-0007 (no new modal component). Applies gotcha
    #67 (CSS rules must exist for every status class) and gotcha #68
    (component reads right field from projection). Stage V runs after
    Stage F.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M05-gap-capability.md (Stage F sections F.1–F.4)</file>
    <file>docs/build-prompts/retrospectives/M05.E-retrospective.md (apply [END] Decisions)</file>
    <file>agent-runtime-spec.md §3 (Visual Design Principles); §4b (gap UI surface); §8.security (capability surface)</file>
    <file>docs/MVP-v0.1.md §M5</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #54 window.__graphStore Playwright injection, #66 tests-pass-but-contract-fails, #67 CSS-rule-missing pattern, #68 wrong-field-read pattern)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="M04.E HITLModal — reused for capability-violation per ADR-0007">src/components/HITLModal.tsx</file>
    <file purpose="ApprovalPanel right-rail layout archetype">src/components/ApprovalPanel.tsx</file>
    <file purpose="AgentNode integration point for CapabilityBadge child">src/components/nodes/AgentNode.tsx</file>
    <file purpose="Stage A applyEvent gap branches — selector source">src/lib/graphStore.ts</file>
    <file purpose="every-class-has-CSS-rule pattern from PR #64">tests/unit/components/BudgetHeaderBar.test.tsx</file>
    <file purpose="ADR-0007 in-process HITL seam — capability-violation modal reuse rationale">docs/adr/0007-in-process-hitl-seam-architecture.md</file>
  </read_reference>

  <read_prior_stages>
    <retrospective milestone="M05" stage="A"/>
    <retrospective milestone="M05" stage="B"/>
    <retrospective milestone="M05" stage="C1"/>
    <retrospective milestone="M05" stage="C2"/>
    <retrospective milestone="M05" stage="D"/>
    <retrospective milestone="M05" stage="E"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M05-gap-capability.md" section="F.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M05-gap-capability.md" section="F.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M05-gap-capability.md" section="Key constraints"/>
  <gates milestone="M05"/>
  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="stage_e_committed" gate="git log --oneline | head -1 must reference Stage M05.E commit"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <inventory_row path="src/components/GapPanel.tsx" status="new"/>
    <inventory_row path="src/components/nodes/CapabilityBadge.tsx" status="new"/>
    <inventory_row path="src/lib/graphStore.ts" status="exists"/>
    <inventory_row path="src/components/nodes/AgentNode.tsx" status="exists"/>
    <inventory_row path="src/App.tsx" status="exists"/>
    <inventory_row path="src/styles.css" status="exists"/>
    <inventory_row path="tests/unit/components/GapPanel.test.tsx" status="new"/>
    <inventory_row path="tests/unit/nodes/CapabilityBadge.test.tsx" status="new"/>
    <inventory_row path="tests/e2e/gap_panel.spec.ts" status="new"/>
  </phase_doc_inventory_audit>

  <architecture_check>
    <claim description="Capability-violation modal REUSES M04.E HITLModal per ADR-0007; no new modal component" verify="find src/components -name 'CapabilityViolation*' -o -name 'CapabilityModal*' ; expect zero matches"/>
    <claim description="HITLModal's existing props shape (question: string, options: string[]) accepts capability_violation content without modification — question = formatted violation text; options = ['Allow once', 'Deny', 'Abort']. If the shape doesn't fit, Stage F adds a thin content-builder helper (still NOT a new modal component) and documents the divergence in the retrospective." verify="grep -nE 'question.*:.*string|options.*:.*string\\[\\]|interface HITLPrompt|type HITLPrompt' src/components/HITLModal.tsx ; expect at least one match showing the existing prop shape — if matches are absent or different, the content-builder helper is required (NOT a new modal)"/>
    <claim description="GapPanel subscribes to graphStore.gaps (selector pattern); does NOT have its own gap-event handler" verify="grep -n 'useGraphStore.*gaps' src/components/GapPanel.tsx ; expect at least one selector hook"/>
    <claim description="CapabilityBadge is a CHILD of AgentNode; not a top-level renderer mount" verify="grep -n 'CapabilityBadge' src/components/nodes/AgentNode.tsx ; expect element rendered within AgentNode's JSX"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="\\.gap-panel|\\.capability-badge" purpose="confirm all class names referenced in TSX have corresponding CSS rules in styles.css per gotcha #67"/>
  </fan_out_grep>

  <gotchas>
    <trap>gotcha #67 applies hard here: every `.gap-panel__item--<severity>` and `.capability-badge--<tier>` class set by the React component MUST have a matching CSS rule in styles.css. Add the every_class_has_a_corresponding_CSS_rule_in_styles_css test pattern from PR #64 (BudgetHeaderBar pattern) — read styles.css at test time and assert each class is present.</trap>
    <trap>gotcha #68 applies hard here too: GapPanel reads `gap.kind`, `gap.name`, `gap.severity`, `gap.suggested_action`, `gap.agent_id`. Confirm the Stage A graphStore applyEvent branches POPULATE all five fields on the GapEntry shape — otherwise the panel renders empty values.</trap>
    <trap>gotcha #54 window.__graphStore Playwright injection: for Playwright tests, inject the capability_violation event via `page.evaluate(() => window.__graphStore.getState().applyEvent(...))`. Module-mocking @tauri-apps/api across the ESM boundary doesn't work in Playwright (only Vitest's vi.mock).</trap>
    <trap>HITLModal reuse: do NOT add a new modal component. The capability_violation event's HITL prompt routes to HITLModal via M04.E policy.rs's on_capability_violation trigger. Stage F's modal test asserts the existing HITLModal mounts; no new component required.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT add Playwright tests that require a full Tauri-driver desktop-shell E2E. Vitest-level renderer tests + Playwright at Vite-dev-server level are sufficient for Stage F. Tauri-driver remains M03 carry-forward (deferred).</warning>
  </execution_warnings>

  <time_box estimate_hours="4"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Did every CSS class set by GapPanel + CapabilityBadge have a matching rule in styles.css? Did the Playwright capability-violation modal wire-path test work without a new modal component (ADR-0007 reuse held)? Decisions for Stage V verifier: what behavior assertions does V need to cover the renderer surface this stage shipped?</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="F.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state</item>
    <item>diff stat</item>
    <item>gate results (frontend lint/typecheck/test + Playwright + validator)</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from M05-gap-capability.md F.6</item>
    <item>explicit statement: "Stage M05.F is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### F.6 Commit Message

```
feat(renderer): M05 Stage F — GapPanel + CapabilityBadge + capability-violation modal wiring

Renderer UI for M05. Three components/wires:

1. GapPanel (new src/components/GapPanel.tsx) — right-rail list of
   unresolved gaps subscribed to graphStore.gaps. Severity drives
   color (critical/important/advisory/requested). Auto-dismisses on
   gap_resolved events.

2. CapabilityBadge (new src/components/nodes/CapabilityBadge.tsx) —
   per-AgentNode pill showing current tier (Novice "N" / Promoted "P")
   + grant count. Rendered as AgentNode child.

3. Capability-violation modal — REUSES M04.E HITLModal per ADR-0007;
   no new modal component. Stage F adds the Playwright test asserting
   the wire path holds.

CSS additions in src/styles.css for .gap-panel + .capability-badge
class families with per-severity / per-tier color variants. Every
class set by TSX has a matching CSS rule (gotcha #67) — verified by
the every_class_has_a_corresponding_CSS_rule_in_styles_css test
pattern from PR #64.

Tests:
- 5 GapPanel Vitest+jsdom tests (empty, render, fields, CSS rules,
  dismissal)
- 4 CapabilityBadge Vitest+jsdom tests (tier display, count, CSS)
- 2 Playwright specs (gap event surfaces panel; capability_violation
  mounts HITLModal)

Coverage: renderer ≥80% (workspace floor).

https://claude.ai/code/session_019xg3cDSLdtX6JkNTLc9a6T
```

---

## Stage V — Verifier (per v1.5 — first milestone shipped with V section in the Phase doc)

> Per ADR-0008 + `STAGE-PROMPT-PROTOCOL.md` §14. Runs between Stage F (last work stage) and Stage G (closeout). Fresh CLI session; clear-and-paste bias guard. Findings tagged 🔴 / 🟡 / 🟢 with merge gating per the severity model.

### V.1 Problem Statement

Run the four-pass verifier against M05's deliverables (Stages A–F) in a fresh CLI session. The agent reads only the spec, the phase doc body (V.1–V.6 included), the current code, and the verifier templates — NOT the M05.A–F retrospectives or the M05 summary or `docs/gap-analysis.md`. Bias guard is structural via clear-and-paste.

Per the M04.V calibration data (TD-001..004 + the protocol-refinement learning in PR #68), M05.V's `<scope_to_verify>` derives from THIS milestone's V.2 table below, which itself derives from each stage's X.2 files-to-change tables — NOT from spec-section pattern-matching.

### V.2 Scope to verify

Aggregated from Stages A through F's X.2 tables:

| Layer | Files / surfaces in scope for M05.V |
|---|---|
| **Inventory** | Every file path from §A.2 + §B.2 + §C1.2 + §C2.2 + §D.2 + §E.2 + §F.2 (Stage A: framework_loader/, request_capability + schema event.v1.json enriched; Stage B: capability/, schemas/capability.v1.json; Stage C1: runtime-sandbox/ plumbing + sandbox_ipc/ + sandbox_lifecycle.rs; Stage C2: runtime-sandbox/ OS-isolation files (seccomp.rs + landlock.rs + job_objects.rs); Stage D: tier/; Stage E: audit/, schemas/audit.v1.json; Stage F: GapPanel.tsx, CapabilityBadge.tsx, styles.css additions) — verifier checks `git ls-files` presence + shape match against §X.3 detailed-changes narrative. |
| **Wire** | Spec claims to trace end-to-end: §4b gap detection (loader emits gap → graphStore.gaps → GapPanel renders); §8.security L1 enforcement (capability_enforcer.check before tool dispatch); §8.security L2a narrowing (sub-agent spawn grants ⊆ parent); §8.security L3 sandbox validation (round trip via sandbox_ipc); §8.security L4 tier gate (TierEvaluator runs before L1 enforcer); §8.security L5 audit log (every grant/deny/transition appends to skills.audit.jsonl). |
| **Behavior** | Vitest+jsdom: AgentNode renders CapabilityBadge with correct tier; GapPanel renders per-severity colors; capability_violation event mounts HITLModal. Vitest static: every `.gap-panel__item--<severity>` + `.capability-badge--<tier>` class has a CSS rule. Rust runtime: framework_loader walker against examples/aria/framework.json emits zero gaps; capability enforcer rejects on default-deny; sandbox subprocess round-trips a validation request. |
| **Multi-call** | `query_session_db`/`read_signals`/`recover_session` (M04 + PR #64 — confirm regression tests still green); `respond_hitl`/`respond_uncertainty` (M04 — still green); `framework_loader::load_and_validate` two-call invariant (Stage A); `sandbox_ipc::client::validate` two-call invariant (Stage C1); `audit_writer.log` concurrent-call invariant (Stage E). |

### V.3 Verification passes (per-pass detail for M05)

#### Inventory pass

For each file path from Stages A–F's X.2 tables (listed above), run `git ls-files` and confirm presence. For each `new` file, confirm shape matches the corresponding X.3 detailed-changes narrative (module boundaries, function names, exposed types). For each `exists` file, confirm the edits described in X.3 are present. Missing → 🔴; shape-drift → 🟡.

#### Wire pass (5-step data-path tracing per gotcha #66 + the v1.5 template's authoring rule)

Traces for M05:

| Trace # | Spec claim | Source event | Projection | Consumer | Step 5 check |
|---|---|---|---|---|---|
| 1 | §4b "gap detection emits an event per unresolved reference" | framework_loader emits `tool_missing` / `skill_missing` / `mcp_missing` / `agent_missing` | graphStore.gaps Map (Stage A applyEvent branches) | GapPanel.tsx reads `gap.kind`, `gap.name`, `gap.severity`, `gap.suggested_action`, `gap.agent_id` | All five fields populated by the projection AND read by the consumer |
| 2 | §8.security L1 "every tool dispatch checked" | `tool_invoked` (M02) | (no projection; enforcer runs in-process before invocation) | SDK dispatch path calls `enforcer.check(agent, needed)` | The call site is BEFORE provider.invoke |
| 3 | §8.security L2a "sub-agent spawn grants ⊆ parent" | `agent_spawned` | (no projection; narrowing runs in-process) | SDK spawn path calls `narrow(parent_grants, proposed)` | Narrowing returns Err for any widening attempt (proptest invariant) |
| 4 | §8.security L3 "generated artifacts sandbox-validated" | (no v0.1 event — M09 wires) | (none in v0.1) | `sandbox_ipc::client::validate()` callable; integration test round trip | Sandbox subprocess round-trip succeeds under the integration test |
| 5 | §8.security L4 "tier caps capability requests" | `tier_violation` | graphStore.currentTier + capabilityViolations | CapabilityBadge.tsx reads `currentTier` | Badge displays N or P per tier |
| 6 | §8.security L5 "every grant/deny appends audit line" | `capability_granted` / `capability_denied` / `tier_transition` / `gap_detected` | (no projection; direct file write) | `skills.audit.jsonl` file in app data dir | After integration test exercise, file contains one line per event |
| 7 | §3 Visual Design "every node-status class has a CSS rule" | (no event; CSS-side check) | (no projection) | styles.css contains `.gap-panel__item--<severity>` for each of 4 severities AND `.capability-badge--<tier>` for each of 2 tiers | Static check via the every_class_has_a_corresponding_CSS_rule pattern |

Each trace breaks at step 4 with missing/multiple consumers → 🔴 ("wire incomplete" / "wire ambiguous").

#### Behavior pass

Run the live harness:

```cmd
:: Vitest renderer-level
npx vitest run tests/unit/components/GapPanel.test.tsx tests/unit/nodes/CapabilityBadge.test.tsx tests/unit/components/BudgetHeaderBar.test.tsx tests/unit/nodes/AgentNode.test.tsx tests/unit/nodes/TaskNode.test.tsx

:: Rust safety-primitive
cargo test -p runtime-main --lib capability --lib tier --lib audit
cargo test -p runtime-main --tests framework_loader_smoke capability_enforcer_smoke tier_smoke audit_smoke
cargo test -p runtime-sandbox

:: Coverage gates (the four ≥95% per-module gates from M05)
cargo llvm-cov --package runtime-main --ignore-filename-regex "..." --fail-under-lines 95
cargo llvm-cov --package runtime-sandbox --ignore-filename-regex "..." --fail-under-lines 95
```

For each failure: trace which pass would have caught it (Inventory → file missing; Wire → step-5 mismatch; Multi-call → second call broken). Findings cite the failing test name + the pass that should have caught it earlier.

Additionally: Playwright `gap_panel.spec.ts` exercising the renderer-level injection per gotcha #54.

#### Multi-call invariants pass

For each public API/IPC method/Tauri command M05 adds:

| Surface | Twice-in-sequence test | Outcome expected |
|---|---|---|
| `framework_loader::load_and_validate` | `tests/framework_loader_smoke.rs::two_consecutive_loads_succeed` (Stage A) | Both calls return Ok with same Framework |
| `enforcer.check` | `enforcer::tests::twice_in_sequence_both_succeed` (Stage B) | Both calls return Ok |
| `sandbox_ipc::client::validate` | `sandbox_ipc::client::tests::validate_succeeds_twice_in_sequence` (Stage C1) | Both calls return Ok with distinct request ids |
| `tier_persistence::save_and_load` | `persistence::tests::save_and_load_twice_in_sequence` (Stage D) | Both saves succeed; load returns the second tier |
| `audit_writer.log` | `writer::tests::two_sequential_entries_two_lines` (Stage E) | File contains 2 lines |
| `respond_hitl` (M04, regression) | M04 tests still green | (existing) |
| `query_session_db` (M04, regression) | PR #64's `query_session_db_succeeds_twice_in_sequence` still green | (existing) |

Missing per-surface test → 🟡 (carry forward to TD-NNN); both-calls-don't-pass → 🔴.

### V.4 Findings format

Per `docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md` § Findings. Numbered globally across passes (#1, #2, #3...). Each finding cites: pass, primitive, spec claim, observed-vs-expected, recommended action.

### V.5 CLI Prompt

Paste into a **fresh** Claude Code session (clear-and-paste pattern is load-bearing).

```xml
<verifier_stage_prompt id="M05.V">
  <context>
    Stage V (Verifier) of M05. Fresh-context contract-fidelity check of
    M05's deliverables (§4b gap detection + §8.security L1+L2a+L3+L4+L5
    + Renderer UI) against `agent-runtime-spec.md`. Run with empty
    session memory — you have NOT seen the M05.A/B/C/D/E/F retros, the
    M05-summary, or any prior gap-analysis entries. Four passes in
    order: Inventory → Wire → Behavior → Multi-call invariants. Findings
    tagged 🔴 (block merge → D.fix), 🟡 (carry forward), 🟢 (tech debt).
    Maximum 2 D.fix iterations before maintainer escalation.

    M05 is the first milestone shipped under v1.5 protocol with the
    Stage V section authored from the start (M04.V was retroactive per
    grandfathering; M05.V is in-band per ADR-0008).
  </context>

  <read_first>
    <file>STAGE-PROMPT-PROTOCOL.md §14 (the verifier schema)</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (design rationale + four passes + bias guard)</file>
    <file>docs/build-prompts/M05-gap-capability.md (Background, all stages A.1/A.2/A.3/A.4 through F.1/F.2/F.3/F.4, AND Stage V section V.1/V.2/V.3 — but NOT any retrospective references the doc may make)</file>
    <file>agent-runtime-spec.md §4b (gap detection); §8.security L1–L5; §3 Visual Design; §1d IPC topology</file>
    <file>docs/MVP-v0.1.md §M5 (acceptance criteria)</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #43, #57, #66, #67, #68, #69, #70, #71, #72 — the M04 IRL bug patterns now codified)</file>
    <file>docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md (output shape)</file>
    <file>docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md (parameterization guidance — especially the "Choosing <scope_to_verify>" subsection added in PR #68)</file>
    <file>docs/tech-debt.md (TD-001..004 already logged from M04.V; M05.V's 🟢 findings append here)</file>
  </read_first>

  <scope_to_verify ref="docs/build-prompts/M05-gap-capability.md" section="V.2 Scope to verify"/>

  <verification_passes>
    <pass name="inventory">
      For each file path enumerated in M05's Stage A.2 + B.2 + C.2 +
      D.2 + E.2 + F.2 "Files to Change" tables, confirm presence in
      `git ls-files` AND shape-match against the corresponding X.3
      "Detailed Changes" narrative. Missing → 🔴. Stub/empty → 🟡.
      Wrong scope/signature → 🟡. Pay attention to: the three new
      schema files (capability.v1.json, audit.v1.json) + their
      regenerated Rust/TS bindings; the four new modules under
      runtime-main (framework_loader, capability, tier, audit); the
      sandbox crate's lit-up shape; the two new renderer components.
    </pass>
    <pass name="wire">
      Run the seven Wire traces from V.3 above using the 5-step
      protocol. Trace breaks at step 4 with zero matching consumers
      OR multiple plausible consumers → 🔴. Note: per PR #68's
      TD-004 lesson, derive trace endpoints from this milestone's
      files-to-change tables, NOT from spec-section pattern-matching
      ("§4a Verify ⇒ schemas/verification.v1.json" is the failure
      mode the template's "Choosing scope_to_verify" subsection
      now codifies).
    </pass>
    <pass name="behavior">
      Run the live harness from V.3 (Vitest + cargo test + cargo
      llvm-cov + Playwright). For each failing test, trace which
      pass (Inventory / Wire / Multi-call) would have caught it
      earlier. Coverage failures on the four ≥95% per-module gates
      (capability, sandbox, tier, audit) are 🔴 if below threshold.
      Renderer-side computed-style checks per gotcha #67.
    </pass>
    <pass name="multi_call_invariants">
      Run the seven sequential-call tests from V.3. For each surface
      lacking a twice-in-sequence test → 🟡 (carry forward to TD-NNN).
      For any test where the second call fails → 🔴. Confirm M04
      regression tests (PR #64) still green.
    </pass>
  </verification_passes>

  <findings_format ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md" section="Findings"/>

  <merge_gate red_blocks="true" dfix_iteration_cap="2" waiver_path="docs/adr/NNNN-waiver-M05-finding-N.md"/>

  <gates milestone="M05"/>

  <self_correction_budget>3</self_correction_budget>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md">
    <special_log>M05.V is the SECOND real-world V run + the first in-band V (M04 was retroactive). Log explicit calibration observations: did the four-pass shape feel adequate at M05's scope (genuinely large milestone)? Were any new bug classes surfaced that M04.V didn't see (concurrency, error paths, cross-platform, performance, etc.)? Should v1.6 protocol-iteration add new passes? Decisions[END] section should include explicit protocol refinement recommendations.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="V.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (build machine `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M05.*-retrospective.md`)</item>
    <item>findings list, sorted by severity</item>
    <item>per-pass summary (counts + notable findings)</item>
    <item>retrospective filled-in [END] section per VERIFIER-RETROSPECTIVE-TEMPLATE.md (verification axes + protocol-calibration observations)</item>
    <item>merge recommendation: "Proceed to G (closeout)" | "Open D.fix for 🔴 findings: &lt;cite numbers&gt;" | "Re-tier"</item>
    <item>explicit statement: "Stage M05.V is ready. I will not commit until you approve."</item>
  </approval_surface>
</verifier_stage_prompt>
```

### V.6 Commit Message

```
verify(M05): in-band V run — findings <N🔴 N🟡 N🟢>

First in-band Stage V run (M04.V was retroactive per ADR-0008
grandfathering). Exercised M05's six work stages (A: §4b gap
detection + framework_loader; B: L1+L2a capability enforcer;
C: L3 sandbox subprocess; D: L4 tier system; E: L5 audit log;
F: renderer UI) via four passes.

Per-pass summary:
  Inventory:      <N> files / <N> shape-match / <N> findings
  Wire:           <N> traces (7 named) / <N> findings
  Behavior:       <N> primitives exercised / <N> coverage-gate findings
  Multi-call:     <N> surfaces / <N> findings

Findings: see docs/build-prompts/retrospectives/M05.V-retrospective.md

Outcome: <Sound | Sound but rough | Friction-heavy | Not ready>
Merge recommendation: <Proceed to G | Open D.fix #X,#Y | Re-tier>

Protocol calibration: second real-world V run; first in-band.
Refinement notes (if any): see retrospective [END] Decisions.

https://claude.ai/code/session_<id>
```

---

<!-- ============================================================ -->
<!-- Stage G — Phase Closeout (always FINAL, runs AFTER Stage V). -->
<!-- Per CLAUDE.md §20: append-only entry in docs/gap-analysis.md.  -->
<!-- ============================================================ -->

## Stage G — Phase Closeout: Gap Analysis + Parent-Milestone Summary

> Per CLAUDE.md §20. Runs after Stage V (and any D.fix iterations) commits. Produces THREE artifacts: M05 summary, M05 gap-analysis entry (append-only), M05 PR description draft.

### G.1 Problem Statement

Generate the M05 entry in `docs/gap-analysis.md` per the six-section template, plus `docs/build-prompts/retrospectives/M05-summary.md` per the SUMMARY template, plus the M05 PR description draft. Cumulative review of code-vs-spec across M01-M05. Append-only — never edit prior entries.

The Stage V retrospective's findings feed the closeout: 🟡 findings go into the M05 gap-analysis entry's "Carry-forward" section; 🟢 findings already log to `docs/tech-debt.md` during V.

### G.2 Files to Change

| File | Status | Change |
|---|---|---|
| `docs/gap-analysis.md` | exists | **Edited (append-only)** — new M05 entry appended at bottom per the entry template |
| `docs/build-prompts/retrospectives/M05-summary.md` | **new** | M05 milestone summary per SUMMARY-TEMPLATE.md |
| `CHANGELOG.md` | exists | Edit: `[Unreleased]` notes M05 closeout |

### G.3 Detailed Changes

Per CLAUDE.md §20 — the six-section structure is NOT optional:

1. **Codebase deep dive** — cumulative review of code shipped through M05 (200–500 words). Touch on the new gap detection + capability enforcement primitives + sandbox subprocess + tier system + audit log + renderer UI. Note shape of new module structures, integration with existing M01–M04 surfaces.

2. **Adherence to spec** — ✅ / ⚠️ / ❌ with file:line for every M05-touched spec section (§4b, §8.security L1–L5, §3 Visual Design, §1d IPC topology, §13.5 dev-logging). Stage V's Wire findings (the 7 traces) populate this section directly.

3. **Spec review forward-looking** — what spec sections need updating based on M05 implementation reality? At minimum: §8.security L4 tier matrix (the data table from Stage D); §13.5 audit log file path (cross-platform resolution); the §4b severity matrix (loader-driven vs request_capability-driven defaults).

4. **Fix backlog** — 🔴 Critical / 🟡 Important / 🟢 Nice-to-have items, severity non-elastic per §20. Stage V's findings populate; closeout adds anything V missed (cumulative-review-only items).

5. **Carry-forward from prior milestones** — disposition of every prior milestone's 🟡 + 🟢 items. Each line: `**Mxx 🟡 "Title"** — RESOLVED at Myy.<stage> | STAYS DEFERRED to Mzz | RESOLVED via PR #N`. M04.V Decisions 1+2 (TaskNode test absorbed at M05.A; §4a reconciliation surfaced for maintainer) get final disposition here. TD-001..004 are forward-tracked (resolution status updated in `docs/tech-debt.md`, not gap-analysis).

6. **Sign-off** — Stage G commit hash + DCO sign-off + AI assistance disclosure.

Plus the `<gotchas_graduation>` subsection — audit per-stage `<gotchas>` across A–F, dispositions: kept / graduated / resolved / expired.

### G.4 Tests

N/A — Stage G ships documentation only. Validation:
- `gap-analysis.md` append-only CI check still passes (no prior entries edited)
- `M05-summary.md` follows SUMMARY-TEMPLATE.md shape

### G.5 CLI Prompt

```xml
<closeout_stage_prompt id="M05.G">
  <context>
    Phase Closeout for M05. FINAL stage. Runs after Stage V commits.
    Produces the cumulative gap-analysis entry (append-only) + the
    M05 parent-milestone summary + the M05 PR description draft.
    Per CLAUDE.md §20. The gap-analysis entry's commit is the FINAL
    commit on the milestone branch claude/m05-gap-capability-phase-doc
    and gates the milestone PR push.
  </context>

  <cumulative_reads>
    <codebase>entire shipped codebase through M05 (cumulative across M01-M05 merges)</codebase>
    <spec>agent-runtime-spec.md (end-to-end, focus on §4b + §8.security + §3 + §1d + §13.5)</spec>
    <gap_analysis>docs/gap-analysis.md (ALL prior entries — M01, M02, M03, M03.5, M04)</gap_analysis>
    <retrospectives>docs/build-prompts/retrospectives/M05.*-retrospective.md (all of A, B, C, D, E, F, V — closeout reads these; verifier did NOT)</retrospectives>
    <summary>docs/build-prompts/retrospectives/M05-summary.md (authored as part of this stage)</summary>
    <tech_debt>docs/tech-debt.md (cumulative TD-NNN entries — M05.V's 🟢 findings should be present from V's run)</tech_debt>
  </cumulative_reads>

  <read_first>
    <file>CLAUDE.md (especially §20 Gap Analysis Protocol)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (especially §8 Closeout-only tags)</file>
    <file>docs/build-prompts/M05-gap-capability.md (this entire phase doc)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md</file>
    <file>docs/gap-analysis.md (the six-section template defined at top of file)</file>
  </read_first>

  <scope_locks ref="docs/build-prompts/M05-gap-capability.md" section="Key constraints"/>

  <gates milestone="M05"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>For M05 specifically: the milestone introduced THREE new safety primitives (capability enforcer, sandbox subprocess, tier evaluator) + one new file-based observability surface (audit log). Aggregate per-primitive coverage outcomes in the summary. Note the protocol-level observation: M05 is the first milestone shipped under v1.5 with V in-band — did the V→closeout handoff work cleanly, or did the closeout duplicate work V already did?</special_log>
  </retrospective_requirements>

  <deliverables>
    <milestone_summary>docs/build-prompts/retrospectives/M05-summary.md (per SUMMARY-TEMPLATE.md; aggregates per-stage retros + V results + scores axes across stages + marks verdict)</milestone_summary>
    <gap_analysis_entry>docs/gap-analysis.md (append new M05 entry; six required sections; gotchas_graduation subsection for stages A–F)</gap_analysis_entry>
    <pr_description>draft only; do not open PR until explicitly asked</pr_description>
  </deliverables>

  <gap_analysis_requirements ref="CLAUDE.md" section="20. Gap Analysis Protocol">
    <gotchas_graduation>
      <stage_review id="A"/>
      <stage_review id="B"/>
      <stage_review id="C1"/>
      <stage_review id="C2"/>
      <stage_review id="D"/>
      <stage_review id="E"/>
      <stage_review id="F"/>
      <!-- Stage V's special_log observations also feed graduation decisions -->
    </gotchas_graduation>
    <special_check>Verify the V→closeout handoff: 🟡 findings from V's retro carry into the gap-analysis Carry-forward section; 🟢 findings already in tech-debt.md; 🔴 findings (if any) were resolved by D.fix iter before closeout</special_check>
  </gap_analysis_requirements>

  <append_only_verification>
    <local_check>prior content of docs/gap-analysis.md must be a literal prefix of HEAD before commit</local_check>
    <ci_check name="gap-analysis-append-only">fails if any prior line is modified</ci_check>
  </append_only_verification>

  <three_artifact_review>
    <artifact>code diff (cumulative across M05 stages A–F + V findings absorbed)</artifact>
    <artifact>per-stage retrospectives (M05.A through M05.F) + Stage V retro + M05 summary</artifact>
    <artifact>new gap-analysis entry — flagged "IMMUTABLE once committed"</artifact>
    <pushback_blocks_pr>true</pushback_blocks_pr>
  </three_artifact_review>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M05-gap-capability.md" section="G.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (build machine git log + retro listing including M05.V + M05-summary.md)</item>
    <item>diff stat (gap-analysis.md additions + M05-summary.md + CHANGELOG.md edit)</item>
    <item>three-artifact review: code diff cumulative + per-stage retros + new gap-analysis entry</item>
    <item>PR description draft for the M05 milestone PR (do NOT open yet — surface only)</item>
    <item>explicit statement: "Stage M05.G is ready. I will not commit until you approve."</item>
  </approval_surface>
</closeout_stage_prompt>
```

### G.6 Commit Message

```
docs(closeout): M05 — gap-analysis entry + parent-milestone summary

Append-only M05 entry to docs/gap-analysis.md (cumulative product↔spec
audit across M01-M05). Six sections + gotchas_graduation across stages
A-F. Per CLAUDE.md §20.

M05 summary (docs/build-prompts/retrospectives/M05-summary.md) aggregates
per-stage retros + Stage V findings + verifier-axes scores.

Stage V findings carry-forward dispositions:
- 🟡 findings → gap-analysis Carry-forward section (next milestone Stage A absorbs)
- 🟢 findings → docs/tech-debt.md (logged during V)
- 🔴 findings (if any) → resolved by D.fix iter before this commit

This commit is the FINAL on claude/m05-gap-capability-phase-doc;
gates the M05 PR push.

https://claude.ai/code/session_019xg3cDSLdtX6JkNTLc9a6T
```



