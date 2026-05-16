# M06 — MCP Basic

> **Protocol version:** v1.6 (per `STAGE-PROMPT-PROTOCOL.md` v1.6 — first milestone authored under v1.6; `<simplify_pass>` required at G; nine new + two extended optional work-stage slots available).
> **Branch:** `claude/m06-mcp-basic`
> **Builds on:** M05 (Gap Detection + Capability Enforcement) merged at `65c4c86`; M05.6 (v1.5→v1.6 protocol iteration) merged at `05318fb`; ADR-0009 waiver of M05.V Findings #1+#2 deferred the L1+L2a SDK wire-up to M06 Stage A.
> **MVP scope:** `docs/MVP-v0.1.md` §M6.

---

## Background and Design Decision

### What this milestone produces

M06 lights up the **§5 MCP Manager + §5a Tool Namespace Resolution** primitives that the spec names as the runtime's external-tooling surface, AND closes the **ADR-0009 carry-forward** (L1+L2a SDK wire-up that M05 shipped complete-as-primitive but unwired to a production call site). End-to-end:

- **MCP transport** — a new `runtime-mcp` workspace crate with `rmcp` 1.7.0 as the protocol library. Two transports: stdio (subprocess JSON-RPC for local MCP servers like filesystem-mcp, git-mcp) and streamable HTTP (for remote MCP servers per MCP spec 2025-11-25). Pluggable transport layer behind a small trait.
- **MCP client lifecycle** — Add/Remove/Test/List operations against MCP servers. Per-server auth via the M02 `key_store` keychain surface. Connect/disconnect/health-ping. SQLite-persisted `mcp_servers` registry (the table already exists from M02; M06 lights it up).
- **§5a Tool Namespace Resolution** — canonical `<server>__<tool>` names; short-name aliasing when unambiguous; explicit `mcp_aliases` framework JSON field override; `tool_alias_ambiguous` warning event when short-name resolution fails post connect/disconnect.
- **MCP dispatch through the L1+L2a gates** — agent invokes MCP tool → SDK calls `enforcer.check(agent_id, &needed)` → if Ok, dispatch to `runtime-mcp` client → tool result streams back. The ADR-0009 wire-up lands in Stage A; Stage D adds MCP-specific dispatch on top.
- **L1+L2a SDK wire-up (ADR-0009 closure)** — Stage A wires `enforcer.check` before `provider.invoke` in `crates/runtime-main/src/sdk/agent_sdk.rs::run_agent_with_provider_stream` (M05.V Finding #1 trace endpoint); and wires `narrow(parent_grants, proposed_child_grants)` before `AgentSpawned` emission in the spawn path (M05.V Finding #2 trace endpoint). The M06.V Wire pass re-runs these traces; 🔴 if the wire-up is missing.
- **Audit log integration** — every MCP lifecycle event (`mcp_installed`, `mcp_uninstalled`, `mcp_auth_granted`, `mcp_request_blocked`) appends to `skills.audit.jsonl` via the M05.E writer surface (no new audit primitive; consumes the existing one).
- **Renderer wiring** — `MCPNode` extended with live connection status + tool list display; Settings panel → MCP Servers → Add/Remove/Test UI; per-server status surfaces; capability-violation modal already shipped at M05.F handles MCP-dispatch-blocked cases via the existing M04.E HITLModal per ADR-0007.

### What's not in scope

- **MCP discovery / browsing** — v1.0. M06 accepts URL or local path as input; the user supplies the server identity.
- **Multi-server collision UI for `tool_alias_ambiguous` resolution** — v1.0. M06 emits the warning event and the user resolves via framework JSON `mcp_aliases` edit; in-UI conflict resolution UI is deferred.
- **MCP server crash auto-recovery beyond default `rmcp` reconnect** — v1.0. M06 wires the basic `mcp_missing` gap event when a previously-connected server goes offline mid-session; sophisticated auto-reconnect with exponential backoff stays at rmcp-defaults.
- **MCP-published Skills or Agents** — per §0b Scope: MCP exposes Tools only. Skills and Agents are owned by the framework / local library / generators (M07+).
- **OAuth flows for MCP server auth** — v1.0. rmcp 1.7.0 supports OAuth per `docs/OAUTH_SUPPORT.md` but v0.1 ships with simple per-server credential strings stored in keychain; no OAuth flow surface.
- **Registry import (M07) + Generators (M09) + Workbench (M08)** — distinct milestones; M06 only adds + connects MCP servers.

### Why six work stages + V + closeout (no sub-stage splits)

> Authored as five (A–E); **Stage F was inserted post-M06.E by maintainer scope call** to close the ADR-0010 composition-root production wire (the headline MCP-dispatch deliverable) in-milestone rather than carry it to M07 — the gotcha #66 / ADR-0009-recurrence pattern the v1.6 `<scope_change>` slot exists to surface, here resolved by a focused in-milestone wire stage. The original A–E rationale below stands; F is the wire-completion addendum.

M06 splits along the natural seams of MCP integration: the protocol-layer crate creation (B) is genuinely separate from lifecycle management (C), which is separate from the §5a namespace algorithm + dispatch wiring (D), which is separate from the renderer surface (E). Stage A is a single coherent surface combining the ADR-0009 wire-up + the M05.V Finding #3 truth-up pre-flight + small M04 carry-forwards — same shape as M05.A's "wire §4b detection + framework_loader + request_capability + M04 carry-forwards" combined surface (per the M05 design-review push-back against A1/A2 splits when the surface is coherent).

- **A** — ADR-0009 SDK wire-up (L1 `enforcer.check` before `provider.invoke` + L2a `narrow()` before `AgentSpawned`) + M05.V Finding #3 phase-doc X.2 truth-up pre-flight + M04 carry-forward closure. The L1+L2a primitives ship from M05 at 100% coverage; Stage A wires them to production call sites and adds the integration tests M05.B's smoke test stood in for.
- **B** — `crates/runtime-mcp` workspace crate creation + `rmcp 1.7.0` transport layer (stdio + http) + `schemas/mcp.v1.json` schema + mock-transport unit tests (M05.C1 `sandbox_ipc` archetype) + wiremock for HTTP path (M02.C `anthropic_sse` archetype) + feature-gated reference-MCP-server integration test. NEW safety primitive: ≥95% per-crate coverage on `runtime-mcp`.
- **C** — `runtime-mcp::client` lifecycle: server install (add + uninstall + test-connection); per-server auth via M02 `key_store` (callback or trait); connection management (connect/disconnect/health-ping); SQLite `mcp_servers` registry lit up; audit emissions for `mcp_installed` / `mcp_uninstalled` / `mcp_auth_granted` via the M05.E writer. Continues runtime-mcp ≥95% gate.
- **D** — §5a namespace resolver (canonical `<server>__<tool>` + short-name + `mcp_aliases`); MCP dispatch wired through Stage A's L1+L2a gates; `tool_alias_ambiguous` + `mcp_request_blocked` event variants; `mcp_request_blocked` audit emission. Per-module ≥95% on namespace + dispatch.
- **E** — Renderer: `MCPNode` extended with connection status + tool list; Settings panel → MCP Servers Add/Remove/Test UI; live indicator on the graph node; Playwright behavior test for Add flow. Renderer ≥80% (vitest).
- **F** — *(maintainer-inserted post-E)* src-tauri `Arc<dyn McpToolDispatch>` injection + live `ProviderEvent::ToolUse` interception in the SDK run loop + `apply_mcp_dispatch` empty-`agent_id` (gotcha #68) test-first fix. Closes the M06.D `<scope_change>` #1+#2 in-milestone. runtime-main ≥95% on the interception logic; src-tauri injection is the `*_with`-seam holdout (M02.C/M05 precedent). Strict v1.7 two-commit TDD.
- **V** — Stage V Verifier (in-band; fresh CLI session; four passes). Wire pass traces the ADR-0009 closure + MCP dispatch path + audit emissions + the Stage F run-loop dispatch SEAM (trace #11, SPLIT per ADR-0011: 11a seam+injection-seam expected DELIVERED/mock-verified; 11b concrete-construction + live exercise = ADR-0011 M07 carry-forward, NOT 🔴).
- **G** — Closeout (gap-analysis entry + parent-milestone summary + **v1.6 required `<simplify_pass>`** child of `<deliverables>`).

Stage B is the only stage that ships a new safety primitive (the `runtime-mcp` crate) at ≥95% coverage — comparable to M05.B (capability enforcer) + M05.C1 (sandbox plumbing). C extends the same crate; D extends with namespace + dispatch wiring; E is renderer-only (≥80%). The L1+L2a primitives already exist and ship from M05 at 100%; Stage A wires them — the wire-up tests cover the integration boundary.

### M05.V Findings carry-forward absorbed

Per the v1.5+ protocol's gap-analysis → next-milestone carry-forward chain + ADR-0009:

- **🔴 #1 + #2 (resolved via ADR-0009 waiver)** — L1 + L2a SDK wire-up deferral from M05 to M06 Stage A. M06.V Wire pass traces `enforcer.check` before `provider.invoke` (Finding #1 trace endpoint) AND `narrow(parent_grants, proposed)` before `AgentSpawned` (Finding #2 trace endpoint). If either trace breaks at step 4, M06.V emits 🔴 and M06 cannot merge without D.fix iter.
- **🟡 #3 (carry-forward to M06 Stage A pre-flight)** — phase-doc-vs-implementation X.2 file drift on `crates/runtime-sandbox/src/ipc.rs` (added at M05.C1 outside C1.2 table) + `crates/runtime-main/src/tier/transition.rs` (added at M05.E outside E.2 table). Stage A pre-flights an X.2 truth-up sweep: confirm both files are listed in their respective stages' X.2 tables via a focused `docs:` edit, OR document via the v1.6 `<scope_change>` slot why they remain undocumented. The structural close moves the M05 phase doc into editable scope ONLY for the truth-up; no other M05 edits.

### M04 carry-forwards still open at M06 entry

Per the M05 gap-analysis Carry-forward section + M05-summary:

- **M04.V Decision 2 (🟡)** — spec §4a `hook_*` vs codebase `verify_*` naming reconcile. Maintainer-decision item. Stage A absorbs by either filing the spec edit OR surfacing for explicit maintainer adjudication in the Stage A retrospective.
- **TD-001..004** (tech-debt; logged in `docs/tech-debt.md`) — forward-applicable; resolution timing per the natural-incorporation windows. TD-002 in particular (read_signals/recover_session per-method tests) fits M06's IPC work; absorb if natural.

### Key constraints

- **v0.1 single-session** per §0d. No multi-session MCP server registry; per-installation registry only.
- **Anthropic-only LLM provider** per §0d. MCP tools dispatch through the existing `AnthropicProvider`; no provider-specific MCP path.
- **STANDARD mode hardcoded** per §0d. M06 doesn't ship mode-aware MCP gates.
- **`fresh_context_per_task` loop policy** per §0d. MCP tool calls fit within a task's fresh-context budget; no cross-task MCP state.
- **Safety-primitive coverage gates ≥95%** for the new `runtime-mcp` crate (Stages B/C/D extend it). Per CLAUDE.md §5 + Codecov gates.
- **Schema-as-source-of-truth** per CLAUDE.md §14. The new `schemas/mcp.v1.json` is authored hand-first then `cargo xtask regenerate-types` lifts to Rust + TS. New event variants extend the existing `schemas/event.v1.json` union.
- **In-process seam architecture (ADR-0007)** for any MCP user-prompt flow (e.g., the MCP `elicitation` protocol). Reuse the M04.E `HitlSeam` per ADR-0007's "M06 MCP user prompts follow this pattern" forward-applicability note. Do NOT introduce a new IPC variant.
- **Path-agnostic persistence + Tauri-shell-resolves-directory archetype** (CLAUDE.md §9, M05.D + M05.E pattern) — any new persistence (MCP server config, per-server settings) accepts `path: &Path` at the public API; the Tauri shell layer resolves `AppHandle::path().app_local_data_dir().join("<file>")` and passes it in.
- **CI-parity hard rule** (CLAUDE.md §6 + retrospective G6) — every stage's `<execution_steps>::implement` runs gates in the canonical v1.6 order (`cargo fmt --all` → `cargo clippy --fix --allow-dirty -p <crate>` → `cargo clippy --workspace -- -D warnings` → remaining); no CI-divergent flags without a gotcha citation.
- **`unsafe_code` forbid everywhere except `runtime-sandbox`** (CLAUDE.md §4 Rule 7). `runtime-mcp` does NOT add `unsafe`; rmcp + reqwest + tower are all safe Rust at the surfaces M06 touches.
- **e2e-tauri-driver job stays DISABLED** unless explicitly re-enabled in a focused infrastructure session (M03 carry-forward).
- **npm overrides for serialize-javascript ≥7.0.5 continues** (M03.F gotcha #39).

---

## Document Structure

| Stage | Scope | Effort estimate | Coverage gate |
|---|---|---|---|
| **A** | ADR-0009 SDK wire-up (`enforcer.check` + `narrow()`) + M05.V #3 X.2 truth-up + M04 carry-forwards | 5–7 h | workspace ≥80%; maintain runtime-main ≥95% |
| **B** | `crates/runtime-mcp` crate + rmcp 1.7.0 transport (stdio + http) + `schemas/mcp.v1.json` + mock-transport tests | 8–10 h | per-crate ≥95% on `runtime-mcp::transport` |
| **C** | `runtime-mcp::client` lifecycle: install + per-server auth + connection mgmt + audit emissions | 6–8 h | per-crate ≥95% on `runtime-mcp::client` (continues `runtime-mcp` gate) |
| **D** | §5a namespace + MCP dispatch through L1+L2a gates + 2 new event variants + audit | 5–6 h | per-module ≥95% on `runtime-mcp::namespace` + `runtime-main::sdk::mcp_dispatch` |
| **E** | Renderer: MCPNode live wiring + Settings → MCP Servers Add/Remove/Test UI + Playwright | 4–5 h | renderer ≥80% (vitest) |
| **F** | Close the M06.D-deferred production wire: src-tauri `Arc<dyn McpToolDispatch>` injection + live `ProviderEvent::ToolUse` interception in the SDK run loop + `apply_mcp_dispatch` empty-`agent_id` (gotcha #68) test-first fix | 3–4 h | runtime-main ≥95% (run-loop interception + wire test); src-tauri shell-injection is the `*_with`-seam holdout per the M02.C / M05 precedent |
| **V** | Verifier — four-pass contract-fidelity check (Inventory + Wire + Behavior + Multi-call invariants) | 2–4 h | N/A (verification stage; no code shipped) |
| **G** | Closeout — gap-analysis entry + M06 summary + **v1.6 required `<simplify_pass>` against M06.A..HEAD diff** | 3–4 h | N/A |

Total: ~36–48 h estimated; ~24–33 h actual at M05's 0.64× calibration baseline with Stage B's novel-protocol bump to ~1× (M02.C anthropic_sse precedent). MCP is a new third-party protocol; the first transport stage historically runs closer to estimate than locked-archetype stages. Stage F was inserted post-M06.E (maintainer scope call) to close the ADR-0010 composition-root wire in-milestone rather than carry the headline deliverable to M07 — the gotcha #66 / ADR-0009-recurrence pattern the v1.6 `<scope_change>` slot exists to surface, here resolved by an in-milestone wire stage instead of a cross-milestone waiver.

---

## Implementation Workflow

Apply these rules consistently across every stage. Don't restate per-stage; they're the project-wide protocol (CLAUDE.md §3, §4, §5, §6, §8, §16, §19).

1. **Read first.** Each stage's `<read_first>` declares what to read before code. Read those files. Stage B+ also reads the prior stage's retrospective `[END] Decisions` section and applies decisions before writing code (CLAUDE.md §19 rule 1).
2. **TDD.** Write a failing behavior test FIRST. Run it; confirm it fails for the right reason. Then implement. Then refactor. Each red-green-refactor cycle is 5–15 min; if longer, the test is too big — split it (CLAUDE.md §5).
3. **Schema-as-source-of-truth.** Every new event variant or shared type goes in `schemas/*.v1.json`. Don't hand-author Rust types in `runtime-core` or TS types in `src/types/`. After schema edits, run `cargo xtask regenerate-types` and commit the generated changes alongside the schema change (CLAUDE.md §14).
4. **Safety-primitive ≥95% coverage** — Stages B/C/D extend the `runtime-mcp` crate. Per-module coverage gates apply via the new v1.6 `<coverage_gate>` slot (named regex per stage); long-form rationale lands in CLAUDE.md §5 at closeout.
5. **No `unsafe` in `runtime-mcp`.** Workspace `forbid(unsafe_code)` applies; rmcp + reqwest + tower are all safe Rust at the surfaces M06 touches. If a transport implementation appears to need `unsafe`, stop and surface — the answer is almost certainly a different rmcp API.
6. **In-process seam (ADR-0007).** Any user-prompt flow (e.g., MCP elicitation prompts) reuses `HitlSeam` from M04.E; do NOT introduce a new IPC variant. Per CLAUDE.md §10.
7. **Path-agnostic persistence (CLAUDE.md §9 + docs/style.md).** New persistence modules accept `path: &Path`; the Tauri shell resolves `AppHandle::path().app_local_data_dir().join("<file>")` and passes it in.
8. **Pre-flight checks every stage.** Stage prompts include `<pre_flight_check>` (env, branch, prior commits), `<phase_doc_inventory_audit>` with v1.6 method/struct_field/read_first_target extension, `<coverage_gate>` naming the exact regex, `<schema_ref_audit>` for any cross-schema `$ref`, `<scope_change>` for any in-stage descope, `<interpretation_declarations>` for any adopted reading of an ambiguous spec section.
9. **Quality-gate execution ordering (v1.6 canonical per CLAUDE.md §6).** At every stage's `<execution_steps>::implement` step: `cargo fmt --all` → `cargo clippy --fix --allow-dirty -p <touched-crate>` → `cargo clippy --workspace --all-targets -- -D warnings` → remaining (test, doc, audit, deny, llvm-cov, frontend). CI-parity per retrospective G6: local commands match `.github/workflows/ci.yml` verbatim; no `--skip` / `--test-threads=N` / env tweaks unless a gotcha citation backs the divergence.
10. **Surface, don't commit, until approved.** Per CLAUDE.md §4 Hard Rule 1. Each stage's `<approval_surface>` declares what the agent shows the human; the human approves before commit lands.
11. **Cross-machine state on every surface.** Every stage end MUST surface `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M06.*-retrospective.md` so any downstream session has real state, not origin's partial view (CLAUDE.md §19 rule 7).
12. **Stage V runs in fresh CLI session.** The user clears the session and pastes the V prompt fresh (the bias guard). V deliberately doesn't read prior retrospectives — the discipline is structurally enforced by the validator's bias-guard rule. V's `<read_first>` includes the phase doc's `<scope_change>` blocks per the STAGE-V-VERIFIER-PROMPT-TEMPLATE.md update from M05.6.
13. **Stage G runs the v1.6 `<simplify_pass>`.** After milestone summary + gap-analysis entry are drafted, before PR opens, invoke the `simplify` skill against `M06.A..HEAD` cumulative diff; surface refactor proposals; apply approved as a focused commit on the same branch; defer non-approved to `docs/tech-debt.md` per ADR-0008 🟢 ledger.

---

## Pre-existing legacy file inventory

Grep-verified at authoring time against `origin/main` at `05318fb` (post-M05.6 merge). Files M06 stages CONSUME or REFERENCE (not create); shape claims are factual as of the authoring snapshot.

| File | Purpose | M06 stage that touches it |
|---|---|---|
| `crates/runtime-main/src/sdk/agent_sdk.rs` | SDK loop with `run_agent_with_provider_stream` (M02-shipped streaming); `AgentSpawned` emitted at the framework-loader walk path (M05.A) | A (ADR-0009 wire-up: `enforcer.check` before provider.invoke; `narrow()` before `AgentSpawned`) + D (MCP-dispatch routing) |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | Translates `ProviderEvent::ToolUse` → `AgentEvent::ToolInvoked` (M02-shipped) — the M05.V Finding #1 wrap-point claim | A (wire enforcer.check at the dispatch surface; emit `capability_grant` / `capability_violation` per outcome) |
| `crates/runtime-main/src/capability/enforcer.rs` | L1 enforcer: `check` + `grant` + `audit_grant` (M05.B + M05.E); 100% line coverage at B-end, 94.24% at E-end with audit branches | A (called from production SDK; integration tests added) |
| `crates/runtime-main/src/capability/narrowing.rs` | L2a narrowing: `narrow(parent, proposed)` (M05.B); 100% line coverage | A (called from production spawn path; integration tests added) |
| `crates/runtime-main/src/audit/writer.rs` | Audit log writer with `log` method (M05.E); ≥99% line coverage | C (mcp_installed/mcp_uninstalled/mcp_auth_granted) + D (mcp_request_blocked) |
| `crates/runtime-main/src/audit/entry.rs` | Audit entry builders (M05.E); 99.39% line coverage | C + D (new entry-builder calls for mcp_* variants) |
| `crates/runtime-main/src/key_store.rs` | OS-keychain integration (M02 Stage E); `set_api_key` / `get_api_key` surface | C (per-server MCP auth secrets via the same `keyring` surface) |
| `crates/runtime-main/src/hitl/seam.rs` | `HitlSeam` (M04.E) — per ADR-0007's "M06 MCP user prompts follow this pattern" forward-applicability | C (reused for any MCP elicitation flow; no new seam variant) |
| `crates/runtime-drone/src/db.rs` | Drone-side SQLite schema + the existing `mcp_servers` table (scaffolded in M01) | C (lit up: insert/select/update MCP server rows) |
| `crates/runtime-core/src/generated/event.rs` | typify-generated event types from `schemas/event.v1.json` | A (regenerated after Stage A doesn't touch the schema; check `cargo xtask regenerate-types --check` passes) + D (regenerated after adding 5 new mcp_* + tool_alias_ambiguous variants) |
| `src/types/agent_event.ts` | TS event types from `schemas/event.v1.json` | D (regenerated alongside Rust types) |
| `src/lib/graphStore.ts` | Zustand store + `applyEvent` reducer | D (new branches for `mcp_installed` / `mcp_uninstalled` / `mcp_auth_granted` / `mcp_request_blocked` / `tool_alias_ambiguous` events) + E (extend `currentMcpServers` slot) |
| `src/components/nodes/MCPNode.tsx` | M03-shipped 11th node type (per spec §3); ships as a stub renderer before MCP events exist | E (lit up: connection status indicator + tool list + active call animation) |
| `src/components/HITLModal.tsx` | M04.E modal variant; M05.F reuses for capability_violation per ADR-0007 | D (reused for `mcp_request_blocked` modal trigger via existing `on_capability_violation` HITL trigger — no new trigger declaration) |
| `examples/aria/framework.json` | Reference framework — should not declare MCP servers in v0.1 (ARIA doesn't depend on MCP) | A pre-flight (`mcp_aliases` field handling at the loader is exercised by example-edit test fixtures, not the canonical example) |
| `schemas/framework.v1.json` | Framework JSON schema | D (extend with optional `mcp_aliases` field per §5a — minor bump within v1; document the addition + run `cargo xtask regenerate-types`) |
| `docs/MVP-v0.1.md` §M6 | M06 acceptance criteria | All stages (consult continuously; verifier later checks against it) |
| `agent-runtime-spec.md` §5 + §5a | Spec sections this milestone implements | All stages |
| `docs/adr/0009-waiver-M05-l1-l2a-sdk-wire-deferral.md` | The waiver that names M06 Stage A as the structural close | A (read first; Stage A's hard deliverable derives from §"Decision" + §"Carry-forward target") |
| `docs/build-prompts/retrospectives/M05.V-retrospective.md` | M05.V verifier output — Findings #1 + #2 absorbed via ADR-0009; Finding #3 truth-up at M06.A pre-flight | A (read in `<read_prior_milestones>` for the carry-forward shape) |
| `docs/build-prompts/M05.6-protocol-v1-6.md` | v1.6 protocol definition | All stages — Stage prompts adopt v1.6 slots inline |

---

## Stage A — ADR-0009 SDK wire-up + M05.V #3 X.2 truth-up + M04 carry-forwards

### A.1 Problem Statement

Wire the L1+L2a capability primitives into the production SDK's call paths, closing the ADR-0009 carry-forward (M05.V Findings #1 + #2). The primitives ship from M05 at 100% coverage (`crates/runtime-main/src/capability/enforcer.rs::check` + `narrowing.rs::narrow`); only the wire is missing. Also pre-flight M05.V Finding #3 (phase-doc X.2 truth-up for `runtime-sandbox/src/ipc.rs` + `runtime-main/src/tier/transition.rs`); absorb the M04.V Decision 2 spec §4a hook_*/verify_* reconcile; reconcile any small M04 carry-forwards still open per M05 closeout.

Concrete deliverables:
1. **L1 wire-up.** Insert `enforcer.check(agent_id, &needed)` call BEFORE `provider.invoke(...)` in `crates/runtime-main/src/sdk/event_pipeline.rs` (the `ProviderEvent::ToolUse → AgentEvent::ToolInvoked` translation path). The translation of `tool_name` to `Vec<CapabilityDeclaration>` uses the framework_loader's resolved capability map (loaded at agent-spawn time, indexed by tool name). On `Ok(())`: continue to existing translation + emit `capability_grant` event. On `Err(CapabilityError::Denied { .. })`: emit `capability_violation` event + route through `HitlSeam` (existing M04.E `on_capability_violation` trigger). On `Err(CapabilityError::TierForbidden { .. })`: emit `tier_violation` event (existing M05.D variant) + route through HitlSeam.
2. **L2a wire-up.** Insert `narrow(parent_grants, proposed_child_grants)` call BEFORE `AgentEvent::AgentSpawned` emission. v0.1's spawn path is the framework-loader walk at `crates/runtime-main/src/sdk/agent_sdk.rs:124` (per M05.V Finding #2 trace); for each declared sub-agent in `Framework.agents`, the loader knows the parent (current walk frame) and the proposed child grants (sub-agent's declared `capabilities` block). Insert the narrowing call so widening attempts emit `capability_violation` and block the spawn.
3. **Integration tests for the L1 wire-up.** Port M05.B's `capability_enforcer_smoke.rs` scenarios into REAL call-path integration tests in `crates/runtime-main/tests/sdk_capability_integration.rs`: (a) tool dispatch with valid grant → `tool_invoked` + `capability_grant`; (b) tool dispatch missing grant → `capability_violation` + HITL fires; (c) tool dispatch denied at tier level → `tier_violation` + HITL fires. The smoke test header comment (line ~1) gets updated to indicate the real integration is now in `sdk_capability_integration.rs`; smoke stays as the per-method unit fixture.
4. **Integration tests for the L2a wire-up.** New `crates/runtime-main/tests/sdk_narrowing_integration.rs`: (a) sub-agent spawn with narrowed grants → `agent_spawned`; (b) sub-agent spawn with widening attempt → `capability_violation` + no `agent_spawned`; (c) sub-agent spawn with empty-grants child → `agent_spawned` with empty grants.
5. **M05.V Finding #3 X.2 truth-up.** Edit `docs/build-prompts/M05-gap-capability.md` C1.2 table to add a row for `crates/runtime-sandbox/src/ipc.rs` (status: new); edit E.2 table to add a row for `crates/runtime-main/src/tier/transition.rs` (status: new). This is a focused `docs:` correction; it does NOT modify any other M05 content. The append-only nature of `docs/gap-analysis.md` is unaffected (gap-analysis already documented the discrepancy at M05 closeout). Alternatively (if maintainer prefers preserving M05 phase doc immutability): surface a `<scope_change>` block in the Stage A prompt's XML documenting the in-stage scope expansion at M05 retroactively; the truth-up lands as a `<scope_change>` `<expand>` entry referencing M05.C1 + M05.E. Pick option A (phase-doc edit) per the v1.6 principle that immutability is convention not law, and the M05 truth-up is a single-PR `docs:` correction. Maintainer decision recorded in retrospective.
6. **M04.V Decision 2 (🟡) spec §4a reconcile.** Surface in Stage A retrospective for maintainer adjudication. Default recommendation per M05.A absorption: update spec §4a text to `verify_*` (code is internally consistent across all 5 milestones now; renaming would cascade). Maintainer decision recorded.
7. **Stage A retrospective surface** includes ADR-0009 closure confirmation: cite the four integration tests by name + the file:line of the wire-up insertion points + the M06.V Wire pass expected-pass disposition.

Not in this stage:
- The MCP protocol layer (Stage B's `runtime-mcp` crate)
- The MCP client lifecycle (Stage C)
- The §5a namespace + dispatch wiring (Stage D)
- The renderer UI (Stage E)

### A.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/sdk/event_pipeline.rs` | exists | Edit: insert `enforcer.check(agent_id, &needed)` call before `ProviderEvent::ToolUse → AgentEvent::ToolInvoked` translation; route Ok/Err per A.1.1 |
| `crates/runtime-main/src/sdk/agent_sdk.rs` | exists | Edit: insert `narrow(parent_grants, proposed)` call before `AgentSpawned` emission in the framework-loader walk path (line ~124); route Err per A.1.2 |
| `crates/runtime-main/src/sdk/mod.rs` | exists | Edit: re-export `CapabilityEnforcer` + `narrow` if not already; add construction of `Arc<CapabilityEnforcer>` in `AgentSdk::new_with_provider` (or equivalent constructor); pass into the event pipeline + spawn path |
| `crates/runtime-main/src/framework_loader/mod.rs` | exists | Edit: populate the agent-id → tool-capability map AND parent → child-grants map at framework-load time; expose getters on the loaded `Framework` |
| `crates/runtime-main/tests/sdk_capability_integration.rs` | **new** | Three end-to-end integration tests per A.1.3 |
| `crates/runtime-main/tests/sdk_narrowing_integration.rs` | **new** | Three end-to-end integration tests per A.1.4 |
| `crates/runtime-main/tests/capability_enforcer_smoke.rs` | exists | Edit: update the header comment to point at `sdk_capability_integration.rs` as the canonical wire-up test surface; keep the smoke as the per-method unit fixture |
| `docs/build-prompts/M05-gap-capability.md` | exists | Edit: C1.2 table adds `ipc.rs` row; E.2 table adds `transition.rs` row — focused `docs:` correction per A.1.5 |
| `CHANGELOG.md` | exists | Edit: `[Unreleased]` notes M06.A — ADR-0009 closure + L1+L2a SDK wire-up + M05.V #3 truth-up |
| `docs/build-prompts/retrospectives/M06.A-retrospective.md` | **new** | Stage A retrospective per `RETROSPECTIVE-TEMPLATE.md` |

Effort budget: ~5–7 hours of code execution. Wire-up itself is small (~30 lines of code per integration point); test surface is the bulk (~6 integration tests + smoke header edit).

### A.3 Detailed Changes

#### A.3.1 L1 wire-up — `event_pipeline.rs`

Before A, the pipeline translates `ProviderEvent::ToolUse(payload)` → `AgentEvent::ToolInvoked { tool_name, ... }` without any capability check. After A, the pipeline first looks up the calling agent's capability declaration via the framework-loader, translates the tool name to a `Vec<CapabilityDeclaration>` needed shape, calls `enforcer.check(agent_id, &needed)`, and routes:

```rust
// In event_pipeline.rs translation logic — illustrative shape:
ProviderEvent::ToolUse(payload) => {
    let needed = framework_loader.capabilities_for_tool(&payload.tool_name)?;
    let agent_id = current_agent_id;  // from the SDK's per-agent context
    match enforcer.check(&agent_id, &needed) {
        Ok(()) => {
            emit(AgentEvent::CapabilityGrant {
                agent_id: agent_id.to_owned(),
                grant_id: payload.tool_use_id.clone(),
                capabilities: needed.clone(),
            }).await?;
            emit(AgentEvent::ToolInvoked { /* existing fields */ }).await?;
        }
        Err(CapabilityError::Denied { capability, .. }) => {
            emit(AgentEvent::CapabilityViolation {
                agent_id: agent_id.to_owned(),
                capability,
                /* existing fields */
            }).await?;
            hitl_seam.await_decision(prompt_id).await?;
            // continue per decision: retry (re-issue translation) | block (drop event) | abort
        }
        Err(CapabilityError::TierForbidden { tier, .. }) => {
            emit(AgentEvent::TierViolation { /* existing M05.D variant */ }).await?;
            hitl_seam.await_decision(prompt_id).await?;
        }
    }
}
```

The exact field shapes match the existing event variants in `schemas/event.v1.json` (M05.B + D shipped them). The `tool_use_id` correlation field is reused as the grant_id per M05.B's design.

#### A.3.2 L2a wire-up — `agent_sdk.rs`

Before A, the framework-loader walk at line ~124 unconditionally emits `AgentEvent::AgentSpawned` for each declared sub-agent. After A, the walk constructs the proposed child grants (from the sub-agent's `capabilities` block in framework JSON), calls `narrow(parent_grants, proposed)`, and routes:

```rust
// In agent_sdk.rs framework_loader walk — illustrative:
for child_decl in framework.agents.iter().filter(|a| a.parent == Some(current_id.clone())) {
    let proposed = child_decl.capabilities.clone();  // from framework.v1.json
    match narrow(&parent_grants, &proposed) {
        Ok(narrowed) => {
            emit(AgentEvent::AgentSpawned {
                agent_id: child_decl.id.clone(),
                grants: narrowed.clone(),
                narrowed_from: Some(proposed.clone()),
                /* existing fields */
            }).await?;
        }
        Err(NarrowingError::Widening { capability, .. }) => {
            emit(AgentEvent::CapabilityViolation { /* widening details */ }).await?;
            // sub-agent does NOT spawn; framework load continues with the next sub-agent
        }
    }
}
```

`narrowed_from` is a new optional field on `AgentSpawned` — minor schema bump within `event.v1.json` v1; document the addition in A.3.4. The `narrow` function's `Err` only fires on widening attempts; identical or narrowed grants succeed.

#### A.3.3 Capability map construction — `framework_loader/mod.rs`

The framework-loader needs to expose two getters used at runtime:
- `capabilities_for_tool(name: &str) -> Result<Vec<CapabilityDeclaration>, FrameworkError>` — for the L1 wire's `needed` lookup at tool-dispatch time.
- `parent_grants_for_agent(agent_id: &str) -> Option<Vec<CapabilityDeclaration>>` — for the L2a wire's `parent_grants` lookup at spawn time.

Both maps are computed once at `load_and_validate` time, stored on the `Framework` struct, and exposed via accessor methods. The maps are immutable post-load (no runtime mutation; framework swap = new Framework).

#### A.3.4 Schema addition — `narrowed_from` field on `AgentSpawned`

Minor bump within `schemas/event.v1.json` v1: add `narrowed_from` as an optional `Vec<CapabilityDeclaration>` on the `agent_spawned` variant. Use the existing `CapabilityDeclaration` type from `schemas/capability.v1.json`. Regenerate Rust + TS types via `cargo xtask regenerate-types`.

The new field's presence signals "this spawn passed L2a narrowing"; absence (e.g., for top-level agents with no parent) signals "no narrowing applied". Renderer ignores absent field; AgentNode's debug inspector shows narrowed-from when present.

#### A.3.5 M05.V #3 X.2 truth-up — `M05-gap-capability.md` edits

Two focused edits:

**C1.2 table** — add row after the existing `protocol.rs` row:
```markdown
| `crates/runtime-sandbox/src/ipc.rs` | **new** | IPC server half on sandbox side; sibling to main-side `sandbox_ipc/client.rs`. Lifted into the C2 ≥95% gate post-C1; baseline 92.58% at C1-end / ≥95% at C2-end. |
```

**E.2 table** — add row after the existing `audit/writer.rs` row:
```markdown
| `crates/runtime-main/src/tier/transition.rs` | **new** | Tier-transition primitive paired with audit emission; 99.24% line coverage at E-end (the 1 uncovered line is the tracing::error! branch on underlying audit-write failure) |
```

The edits are scoped — every other M05 phase-doc line stays unchanged. The `docs/gap-analysis.md` M05 entry's Carry-forward section already documents the discrepancy at M05 closeout per CLAUDE.md §20 append-only; this truth-up moves the canonical X.2 record to match what shipped, closing the M05.V Finding #3 🟡 carry-forward.

#### A.3.6 M04.V Decision 2 spec §4a reconcile — maintainer-decision surface

In Stage A's retrospective `[END] Decisions` section: recommend updating spec §4a text from `hook_*` to `verify_*` to match codebase reality (M04 + M05 shipped `verify_*` consistently; spec drift is unilateral). Cite file:line per the M05.A retrospective's identical recommendation. Defer to maintainer; the code stays as-is in both directions.

### A.4 Tests

#### A.4.1 Wire-up integration tests (new)

`crates/runtime-main/tests/sdk_capability_integration.rs`:

```rust
// Test 1: tool_dispatch_with_valid_grant_emits_capability_grant_and_tool_invoked
// Test 2: tool_dispatch_missing_grant_emits_capability_violation_and_triggers_hitl
// Test 3: tool_dispatch_denied_at_tier_level_emits_tier_violation_and_triggers_hitl
```

`crates/runtime-main/tests/sdk_narrowing_integration.rs`:

```rust
// Test 1: spawn_with_narrowed_grants_succeeds
// Test 2: spawn_with_widening_attempt_emits_capability_violation_and_blocks_spawn
// Test 3: spawn_with_empty_child_grants_succeeds_with_empty_narrowed_set
```

Each test sets up an `AgentSdk` with a wiremock-backed `AnthropicProvider`, a synthetic `Framework` loaded from inline JSON (no fixture file dependency), and a `HitlSeam` whose responses are scripted via `oneshot::Sender`. The mocked provider responds with a scripted `ProviderEvent::ToolUse` payload; the test asserts the emitted `AgentEvent` sequence matches the expected wire-up outcome.

#### A.4.2 Multi-call invariant tests (new)

Each new test file includes a `<scenario>_twice_in_sequence_both_succeed` test per CLAUDE.md §5 + gotcha #69 (IPC primitives need multi-call invariant tests). For the wire-up tests:

```rust
// In sdk_capability_integration.rs
#[tokio::test]
async fn tool_dispatch_with_valid_grant_twice_in_sequence_both_succeed() { /* ... */ }
```

#### A.4.3 Smoke test header update

`crates/runtime-main/tests/capability_enforcer_smoke.rs` line ~1–10 (header comment): replace the M05.B-era "v0.1 SDK has no dispatch path to wrap; smoke stands in" text with a pointer to the new `sdk_capability_integration.rs` as the canonical wire surface. The smoke test itself remains; it's now the per-method unit fixture for the L1 primitive.

#### A.4.4 Acceptance criteria

- [ ] `cargo test -p runtime-main --tests sdk_capability_integration` — all 4 tests pass
- [ ] `cargo test -p runtime-main --tests sdk_narrowing_integration` — all 4 tests pass
- [ ] `cargo test -p runtime-main --tests capability_enforcer_smoke` — all existing tests still pass (no regression)
- [ ] `cargo test -p runtime-main --lib capability` — all unit tests still pass; `enforcer.rs` line coverage ≥95% (M05.E baseline preserved or improved)
- [ ] `cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.key_store\.rs" --fail-under-lines 95` — runtime-main gate holds
- [ ] `cargo xtask regenerate-types --check` — no schema drift after `narrowed_from` addition
- [ ] `npm run test` + `npx tsc --noEmit` — no regression in TS-side
- [ ] M05.V Finding #3 truth-up: `docs/build-prompts/M05-gap-capability.md` C1.2 + E.2 tables updated per A.3.5
- [ ] M04.V Decision 2 reconcile surfaced in retrospective for maintainer adjudication
- [ ] M06.V Wire pass's expected end-to-end traces are now satisfiable (traces 2 + 3 from M05.V trace-table — L1 enforcer.check + L2a narrow have production consumers)
- [ ] CI-parity per G6: paste exact local commands run + confirm match `.github/workflows/ci.yml`

### A.5 CLI Prompt

Paste the XML block below into a fresh Claude Code session as the opening message. Per `STAGE-PROMPT-PROTOCOL.md` v1.6 — Stage A is a work-stage prompt; uses v1.4 + v1.5 + v1.6 protocol tags.

```xml
<work_stage_prompt id="M06.A">
  <context>
    Stage A of M06 (MCP Basic). Closes the ADR-0009 carry-forward by wiring
    the L1 capability enforcer (`enforcer.check`) into the production SDK's
    tool-dispatch path AND the L2a narrowing primitive (`narrow`) into the
    production sub-agent spawn path. Both primitives ship from M05 at 100%
    line coverage; M05.B's smoke test stood in for the wire-up. Stage A
    ports the smoke scenarios into real call-path integration tests in two
    new test files (`sdk_capability_integration.rs` + `sdk_narrowing_integration.rs`).
    Also pre-flights the M05.V Finding #3 X.2 truth-up (focused `docs:` edit
    on the M05 phase doc's C1.2 + E.2 tables) and surfaces the M04.V
    Decision 2 spec §4a reconcile for maintainer adjudication. First
    milestone authored under v1.6 protocol with the new optional slots
    available; G's `<simplify_pass>` deliverable child is mandatory.
    M06.V Wire pass will trace `enforcer.check before provider.invoke`
    + `narrow before AgentSpawned`; 🔴 if either wire is missing.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M06-mcp-basic.md (Background, Document Structure, Implementation Workflow, Pre-existing legacy file inventory, Stage A sections A.1–A.4)</file>
    <file>docs/adr/0007-in-process-hitl-seam-architecture.md (in-process seam pattern; reused at C if MCP elicitation prompts land)</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (verifier protocol — informs Stage V design)</file>
    <file>docs/adr/0009-waiver-M05-l1-l2a-sdk-wire-deferral.md (defines this stage's hard deliverable + the M06.V Wire-pass expected trace endpoints)</file>
    <file>docs/build-prompts/retrospectives/M05.V-retrospective.md (Findings #1 + #2 absorbed via ADR-0009; Finding #3 X.2 truth-up is this stage's pre-flight)</file>
    <file>docs/build-prompts/retrospectives/M05-summary.md (cumulative trends; M06 calibration anchor 0.64×; M04 carry-forwards still open)</file>
    <file>agent-runtime-spec.md §8.security L1 + L2a (the contract the wire-up satisfies); §3a (the spawn path L2a wraps); §6a HITL on_capability_violation trigger (existing; reused)</file>
    <file>docs/MVP-v0.1.md §M6 (acceptance criteria)</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #43 typify root-oneOf, #66 tests-pass-but-contract-fails, #68 wrong-field-read, #69 IPC multi-call invariants, #74 cfg-target-os, #66 contract tests)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="L1 wire-up insertion point — ProviderEvent::ToolUse → AgentEvent::ToolInvoked translation">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="L2a wire-up insertion point — framework-loader walk at line ~124 emits AgentSpawned unconditionally">crates/runtime-main/src/sdk/agent_sdk.rs</file>
    <file purpose="L1 primitive (ships 100%); call signature for the wire">crates/runtime-main/src/capability/enforcer.rs</file>
    <file purpose="L2a primitive (ships 100%); call signature for the wire">crates/runtime-main/src/capability/narrowing.rs</file>
    <file purpose="M05.B smoke test — header comment update + integration test scenarios to port">crates/runtime-main/tests/capability_enforcer_smoke.rs</file>
    <file purpose="framework loader — capability map construction (Stage A extends with two getters)">crates/runtime-main/src/framework_loader/mod.rs</file>
    <file purpose="HitlSeam — reused for capability_violation + tier_violation HITL routing">crates/runtime-main/src/hitl/seam.rs</file>
    <file purpose="wiremock pattern for AnthropicProvider — copy for integration test setup">crates/runtime-main/tests/anthropic_wiremock.rs</file>
    <file purpose="event schema — add narrowed_from optional field on agent_spawned variant">schemas/event.v1.json</file>
    <file purpose="capability schema — type used in new narrowed_from field">schemas/capability.v1.json</file>
    <file purpose="M05 phase doc — focused edit to C1.2 + E.2 tables per M05.V Finding #3 truth-up">docs/build-prompts/M05-gap-capability.md</file>
  </read_reference>

  <read_prior_milestones>
    <gap_analysis_carry_forward milestone="M05"/>
    <milestone_summary milestone="M05" section="Decisions to apply before the next parent milestone"/>
    <verifier_retrospective milestone="M05">docs/build-prompts/retrospectives/M05.V-retrospective.md — Findings #1 + #2 absorbed via ADR-0009 (this stage closes); Finding #3 X.2 truth-up is this stage's pre-flight; Decision 3 is the v1.6 `<scope_change>` slot now available to this stage</verifier_retrospective>
  </read_prior_milestones>

  <deliverable ref="docs/build-prompts/M06-mcp-basic.md" section="A.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the test plan's buckets. Stub the
      production surfaces just enough to make the test files compile
      (todo!() / unimplemented!() bodies are fine; the goal is link-time
      test discovery, not behavior). Confirm tests fail with right-reason
      errors per CLAUDE.md §5 (assertion failed / cannot find function /
      unresolved import / not-yet-implemented panic — NOT a test-file
      compile error and NOT a tautological pass). Commit as a STANDALONE
      `test(M06.&lt;stage&gt;): failing tests for ...` commit on
      claude/m06-mcp-basic BEFORE green-phase impl; the commit body
      pastes the first ~40 lines of cargo test output proving the
      expected-failure class. Surface the red-phase commit to the user;
      user approves before green phase begins.
    </red_phase>
    <green_phase>
      Implement until ALL failing tests pass. Do NOT modify the test
      files during implementation — if a test is wrong, fix it in a
      SEPARATE labelled follow-up commit with explanation, never silently
      in the impl commit. The impl commit body MUST state the verifiable
      audit-surface invariant: `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      rustfmt/clippy fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message (M06.B Decision;
      gotcha-candidate territory on third recurrence).
    </green_phase>
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

  <acceptance_criteria ref="docs/build-prompts/M06-mcp-basic.md" section="A.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M06-mcp-basic.md" section="Key constraints"/>

  <gates milestone="M06"/>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch_state" gate="git rev-parse --abbrev-ref HEAD must equal claude/m06-mcp-basic"/>
    <check name="prior_milestone_merged" gate="git log origin/main --oneline | head -10 must include the M05 closeout merge (65c4c86) AND the M05.6 protocol merge (05318fb)"/>
    <check name="rust_toolchain" gate="cargo --version must report the version pinned in rust-toolchain.toml"/>
    <check name="no_uncommitted_changes" gate="git status --porcelain must be empty"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="crates/runtime-main/src/sdk/event_pipeline.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/sdk/agent_sdk.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/capability/enforcer.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/capability/narrowing.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/framework_loader/mod.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/tests/capability_enforcer_smoke.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/tests/anthropic_wiremock.rs" verified="true"/>
    <claim type="file" path="schemas/event.v1.json" verified="true"/>
    <claim type="file" path="schemas/capability.v1.json" verified="true"/>
    <claim type="file" path="docs/build-prompts/M05-gap-capability.md" verified="true"/>
    <claim type="method" path="crates/runtime-main/src/capability/enforcer.rs" symbol="check" verified="true" note="ships from M05.B at 100% line coverage; called from in-stage wire-up at event_pipeline.rs"/>
    <claim type="method" path="crates/runtime-main/src/capability/narrowing.rs" symbol="narrow" verified="true" note="ships from M05.B at 100% line coverage; called from in-stage wire-up at agent_sdk.rs:124"/>
    <claim type="method" path="crates/runtime-main/src/framework_loader/mod.rs" symbol="capabilities_for_tool" verified="false" note="Stage A adds this getter; not yet present"/>
    <claim type="method" path="crates/runtime-main/src/framework_loader/mod.rs" symbol="parent_grants_for_agent" verified="false" note="Stage A adds this getter; not yet present"/>
    <claim type="struct_field" path="schemas/event.v1.json" symbol="agent_spawned.narrowed_from" verified="false" note="Stage A adds this optional field; minor bump within event.v1.json v1"/>
    <claim type="read_first_target" path="docs/adr/0009-waiver-M05-l1-l2a-sdk-wire-deferral.md" verified="true"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M05.V-retrospective.md" verified="true"/>
  </phase_doc_inventory_audit>

  <schema_drift_check gate="cargo xtask regenerate-types --check (before Stage A's schema edit; rerun after the narrowed_from addition to confirm clean regen)"/>

  <schema_ref_audit>
    <ref schema="schemas/event.v1.json" path="#/$defs/CapabilityDeclaration" verified="true" note="referenced by new narrowed_from field; existing $def from M05.B"/>
    <ref schema="schemas/capability.v1.json" path="#/$defs/ResourceName" verified="true" note="existing $def; transitively reached via CapabilityDeclaration"/>
  </schema_ref_audit>

  <schema_audit>
    <survey pattern='"narrowed_from"' purpose="confirm narrowed_from not already declared in any schemas/*.v1.json before adding to agent_spawned variant"/>
    <survey pattern='"capabilities_for_tool"' purpose="confirm no existing getter with this name in framework_loader before authoring"/>
  </schema_audit>

  <schema_root_check/>

  <architecture_check>
    <claim description="L1 enforcer.check is called from event_pipeline.rs BEFORE the existing tool_invoked translation — ordering matters per spec §8.security L1 'every dispatch passes through the enforcer'" verify="grep -B5 'AgentEvent::ToolInvoked' crates/runtime-main/src/sdk/event_pipeline.rs ; expect enforcer.check call to precede the translation after Stage A lands"/>
    <claim description="L2a narrow is called at the framework-loader walk in agent_sdk.rs around line 124 (per M05.V Finding #2 trace) — NOT at a separate spawn-driver (there is no spawn-driver in v0.1)" verify="grep -n 'AgentSpawned' crates/runtime-main/src/sdk/agent_sdk.rs ; expect narrow call to precede the emission at the loader walk site after Stage A lands"/>
    <claim description="HitlSeam reused for on_capability_violation routing — no new seam variant per ADR-0007" verify="grep -n 'on_capability_violation' crates/runtime-main/src/hitl/policy.rs ; expect existing trigger entry from M04.E"/>
    <claim description="framework_loader exposes capabilities_for_tool + parent_grants_for_agent — getters used by the wire-up" verify="grep -n 'pub fn capabilities_for_tool\\|pub fn parent_grants_for_agent' crates/runtime-main/src/framework_loader/mod.rs ; expect both getters present after Stage A lands"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern='enforcer\.check' purpose="confirm zero pre-Stage-A production call sites; after Stage A, expect at least one in event_pipeline.rs (the wire-up); current test-only sites stay in capability/enforcer.rs #[cfg(test)] mod tests + capability_enforcer_smoke.rs"/>
    <grep pattern='narrow\(' purpose="confirm zero pre-Stage-A production call sites; after Stage A, expect at least one in agent_sdk.rs around line 124"/>
    <grep pattern='AgentSpawned' purpose="enumerate all emission sites; Stage A guards the one at agent_sdk.rs:124 (the framework-loader walk); confirm no other unguarded emissions surface"/>
    <grep pattern='ProviderEvent::ToolUse' purpose="enumerate all translation sites; Stage A guards the one at event_pipeline.rs:~57"/>
  </fan_out_grep>

  <api_breaking_change_audit>
    <change api="framework_loader::Framework" before_signature="struct without capabilities_for_tool getter" after_signature="struct with new pub fn capabilities_for_tool(name: &amp;str) -> Result&lt;Vec&lt;CapabilityDeclaration&gt;, FrameworkError&gt;" call_sites="0" test_sites="0" recommendation="purely additive — no breaking change; existing consumers unaffected"/>
    <change api="framework_loader::Framework" before_signature="struct without parent_grants_for_agent getter" after_signature="struct with new pub fn parent_grants_for_agent(agent_id: &amp;str) -> Option&lt;Vec&lt;CapabilityDeclaration&gt;&gt;" call_sites="0" test_sites="0" recommendation="purely additive — no breaking change"/>
    <change api="schemas/event.v1.json agent_spawned variant" before_signature="without narrowed_from field" after_signature="with optional narrowed_from: Vec&lt;CapabilityDeclaration&gt; field" call_sites="3" test_sites="6" recommendation="optional field — existing consumers continue to work; new field surfaces narrowing outcome for renderer + audit"/>
  </api_breaking_change_audit>

  <existing_pattern_audit>
    <pattern grep_for="AgentSpawned {" rationale="adding narrowed_from to AgentSpawned changes struct literal shape; all existing emit sites need the new field (default None) at minimum" affected_files="grep result" remediation="add `narrowed_from: None` to existing emit sites OR derive from Default::default()"/>
  </existing_pattern_audit>

  <interpretation_declarations>
    <adopt spec_section="§8.security L1" interpretation="L1 wraps at the SDK's ProviderEvent::ToolUse translation site (event_pipeline.rs); the runtime cannot gate Anthropic's server-side dispatch, but it CAN gate the runtime's subsequent emission of AgentEvent::ToolInvoked + downstream observers + sub-tool chain" alternative_interpretation="L1 wraps at a hypothetical pre-dispatch site that doesn't exist in v0.1's streaming-only SDK" rationale="per ADR-0009 §Alternative A 'rejected because the wrap point is semantically wrong for v0.1's SDK shape'; the post-translation gate is the correct v0.1 site and produces honest defensive observability rather than misleading test surface"/>
    <adopt spec_section="§8.security L2a" interpretation="L2a wraps at the framework-loader walk that emits AgentSpawned (agent_sdk.rs:124); v0.1 has no separate spawn-driver" alternative_interpretation="L2a wraps at a hypothetical spawn-driver that doesn't exist in v0.1" rationale="per ADR-0009 §Decision 'M06 Stage A is the structural close for both findings via the spawn-narrowing call site at the existing emission point'; the framework-loader walk IS the v0.1 spawn site"/>
  </interpretation_declarations>

  <scope_change>
    <descope deliverable="multi-turn agent loop (plan_loop driver)" reason="v0.1 SDK is streaming-only; multi-turn is M07-scope per docs/MVP-v0.1.md" carry_forward_to="M07" authorized_by="docs/MVP-v0.1.md §M6 Out-of-scope + ADR-0009 §Alternative C 'rejected because §0d Release Scope Matrix locks v0.1 milestone boundaries'"/>
  </scope_change>

  <dependency_audit_check>
    <dep name="no new crates" required_features="N/A" min_version="N/A" audit="cargo deny check still passes; Stage A introduces no new third-party dependencies (wire-up reuses existing tokio + serde + thiserror + reqwest + wiremock — all already in workspace)"/>
  </dependency_audit_check>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-drone" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.lib\.rs|src.shutdown\.rs"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.key_store\.rs"/>
    <gate scope="package" name="runtime-sandbox" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.lib\.rs|src.seccomp\.rs|src.landlock\.rs"/>
  </coverage_gate>

  <runtime_environment os="windows" note="Build agent runs on Windows 11 per M01-M05 pattern; gotcha #56 windows-local cargo llvm-cov subprocess flake mitigation still applies per the v1.6 CI-parity rule — if --test-threads=N is needed, cite gotcha #56 inline AND file the CI-workflow fix as the structural close"/>

  <gotchas>
    <trap>Tests-pass-but-contract-fails (gotcha #66). The L1+L2a primitives ship at 100% unit-test coverage but were unwired; this stage's failure mode is shipping tests that exercise the new integration code without proving it's the SDK call site. Mitigation: every integration test asserts BOTH the emitted AgentEvent sequence AND the absence of the previously-emitted unguarded events (e.g., assert no ToolInvoked emission without a preceding CapabilityGrant).</trap>
    <trap>Wrong-field reads (gotcha #68). When asserting on emitted events, destructure ALL fields the projection writes — not just the field-of-interest. Stage A's new narrowed_from field is the canonical "field that exists but isn't read" trap; the AgentNode debug-inspector should read it post-Stage E.</trap>
    <trap>IPC multi-call invariants (gotcha #69). Every new integration test file includes a *_twice_in_sequence_both_succeed variant per CLAUDE.md §5 + gotcha #69 + the v1.5 multi-call pass.</trap>
    <trap>typify root-oneOf (gotcha #43). The narrowed_from addition is inside an existing oneOf variant, not at root — typify accepts inline-validated strings inside variants. If typify panics anyway, extract to $defs.</trap>
    <trap>Schema regeneration discipline. After editing event.v1.json, run `cargo xtask regenerate-types` and commit the generated changes in the SAME commit as the schema edit; never let generated/event.rs drift from event.v1.json. CI's regenerate-types --check enforces.</trap>
    <trap>Framework-load-time capability map construction. The map is computed once at load_and_validate; it must include every declared tool's capabilities AND every declared sub-agent's parent linkage. Test fixture frameworks must include these.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT touch crates/runtime-mcp — that crate does NOT exist yet; Stage B creates it. Stage A wires existing M05 primitives into the existing M02 SDK.</warning>
    <warning>DO NOT add a new HITL trigger for capability_violation OR tier_violation — both exist from M04.E + M05.D respectively. Reuse `on_capability_violation` + `on_tier_violation` (or whichever trigger M05.D introduced for tier_violation; grep policy.rs to confirm naming).</warning>
    <warning>The M05 phase-doc truth-up edit is the ONLY edit to a prior milestone's phase doc allowed in M06.A — it's a focused `docs:` correction per A.1.5 + A.3.5. Do NOT modify any other M05 content. The append-only nature of `docs/gap-analysis.md` is unaffected.</warning>
    <warning>The M04.V Decision 2 reconcile is a maintainer-decision item; surface in retrospective `[END] Decisions`, do not unilaterally edit `agent-runtime-spec.md` §4a without maintainer approval. The default recommendation (spec text → `verify_*`) goes in the retrospective; the spec edit lands in a separate `docs(spec):` PR after maintainer approves.</warning>
    <warning>The M05.V Finding #3 truth-up is in scope; the M05.V Findings #1 + #2 are NOT a re-litigation — ADR-0009 closes them via this stage's wire-up. Don't reopen the waiver; do close the wire.</warning>
  </execution_warnings>

  <time_box estimate_hours="7"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>ADR-0009 closure confirmation: cite the four wire-up integration tests by name + the file:line of each insertion point + the M06.V Wire pass expected-pass disposition for traces 2 + 3. M04.V Decision 2 maintainer-decision surface (spec §4a `hook_*` vs `verify_*` reconcile). M05.V Finding #3 truth-up outcome (phase-doc edit applied OR `<scope_change>` route taken). Any v1.6 slot that surfaced new value in this stage (likely `<scope_change>` per the multi-turn-loop deferral; possibly `<api_breaking_change_audit>` per the narrowed_from addition).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M06-mcp-basic.md" section="A.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M06.*-retrospective.md)</item>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results per the v1.6 canonical ordering (fmt → clippy --fix → clippy -D warnings → test → doc → audit → deny → llvm-cov per gate + frontend lint/typecheck/test + validator + schema_drift_check + xtask regenerate-types --check, each pass/fail with key numbers; CI-parity confirmation per G6)</item>
    <item>ADR-0009 closure summary: four integration test names + file:line of L1 wire-up insertion + file:line of L2a wire-up insertion + M06.V Wire-pass trace endpoints satisfied</item>
    <item>M05.V Finding #3 truth-up outcome (phase-doc edit applied OR scope_change route taken)</item>
    <item>M04.V Decision 2 maintainer-decision surface in retrospective `[END] Decisions`</item>
    <item>retrospective filled-in [END] section (three-axis scoring + verdict + decisions for Stage B)</item>
    <item>draft commit message from M06-mcp-basic.md A.6 Commit Message section (filled with session URL)</item>
    <item>explicit statement: "Stage M06.A is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### A.6 Commit Message

```
feat(runtime): M06 Stage A — ADR-0009 closure (L1 + L2a SDK wire-up) + M05.V #3 X.2 truth-up

Closes the ADR-0009 carry-forward by wiring the L1 capability enforcer
(`enforcer.check`) into the production SDK's tool-dispatch path (the
ProviderEvent::ToolUse → AgentEvent::ToolInvoked translation in
crates/runtime-main/src/sdk/event_pipeline.rs) AND the L2a narrowing
primitive (`narrow`) into the production sub-agent spawn path (the
framework-loader walk emitting AgentSpawned at
crates/runtime-main/src/sdk/agent_sdk.rs:~124). Both primitives shipped
from M05 at 100% line coverage; M05.B's smoke test stood in for the
wire-up. M05.V Findings #1 + #2 (waived via ADR-0009) are now structurally
closed — M06.V Wire pass will trace `enforcer.check before provider.invoke`
+ `narrow before AgentSpawned` and emit 🟢 (or surface concrete drift if
the wire-up regressed).

Wire-up details:
- event_pipeline.rs: enforcer.check(agent_id, &needed) before translation;
  on Ok emit CapabilityGrant + ToolInvoked; on Err route through HitlSeam
  (existing M04.E on_capability_violation trigger).
- agent_sdk.rs: narrow(parent_grants, proposed) before AgentSpawned;
  on Err emit CapabilityViolation + skip the spawn (framework load
  continues with next sub-agent).

Schema:
- schemas/event.v1.json: agent_spawned variant gains optional narrowed_from
  field (Vec<CapabilityDeclaration>). Minor bump within v1 — purely
  additive.
- crates/runtime-core/src/generated/event.rs + src/types/agent_event.ts:
  regenerated via cargo xtask regenerate-types.

framework_loader extensions:
- capabilities_for_tool(name: &str) -> Result<Vec<CapabilityDeclaration>,
  FrameworkError> — used by L1 wire's `needed` lookup at tool-dispatch
  time.
- parent_grants_for_agent(agent_id: &str) -> Option<Vec<CapabilityDeclaration>>
  — used by L2a wire's `parent_grants` lookup at spawn time.

Tests:
- crates/runtime-main/tests/sdk_capability_integration.rs (new): 4 tests
  covering grant-succeeds-emits-grant-and-invoked, missing-grant-emits-
  violation-and-triggers-hitl, denied-at-tier-emits-tier-violation-and-
  hitl, twice-in-sequence-both-succeed.
- crates/runtime-main/tests/sdk_narrowing_integration.rs (new): 4 tests
  covering spawn-with-narrowed-succeeds, widening-attempt-emits-violation-
  blocks-spawn, empty-child-grants-succeeds-with-empty, twice-in-sequence.
- crates/runtime-main/tests/capability_enforcer_smoke.rs: header comment
  updated to point at sdk_capability_integration.rs as canonical wire
  surface; smoke retained as per-method unit fixture.

Coverage: workspace ≥80%; runtime-main per-package ≥95% preserved (M05.E
baseline 94.24% on enforcer.rs lifts back toward 100% with the new
integration paths exercising the audit-grant + audit-check-result
branches).

M05.V Finding #3 X.2 truth-up:
- docs/build-prompts/M05-gap-capability.md C1.2 table: add row for
  crates/runtime-sandbox/src/ipc.rs.
- docs/build-prompts/M05-gap-capability.md E.2 table: add row for
  crates/runtime-main/src/tier/transition.rs.
Focused `docs:` correction; no other M05 content modified. The
docs/gap-analysis.md M05 entry's Carry-forward section already documents
the discrepancy per CLAUDE.md §20 append-only.

M04.V Decision 2 (spec §4a hook_*/verify_* reconcile) surfaced in this
stage's retrospective for maintainer adjudication. Default recommendation:
update spec §4a text to `verify_*` (code is internally consistent across
M04 + M05). Maintainer decision pending; no spec edit in this PR.

Not in this stage: MCP transport (Stage B), MCP client lifecycle (Stage
C), §5a namespace + MCP dispatch (Stage D), renderer UI (Stage E).

https://claude.ai/code/session_<id>
```

---

## Stage B — `runtime-mcp` crate + rmcp 1.7.0 transport (NEW safety primitive ≥95%)

### B.1 Problem Statement

Create the `runtime-mcp` workspace crate as the protocol-layer dependency boundary for MCP. The crate contains the transport abstraction (stdio + streamable HTTP via `rmcp` 1.7.0), the `mcp.v1.json` schema for MCP server config, transport-level error mapping, and full unit-test coverage via rmcp's mock-transport abstraction (M05.C1 `sandbox_ipc` archetype for stdio in-memory testing; M02.C `anthropic_sse` archetype for HTTP via `wiremock`). Lifecycle (server install + auth + connection mgmt) lands in Stage C; namespace + dispatch in Stage D. Stage B ships the protocol foundation.

The `runtime-mcp` crate is the third standalone crate beyond `runtime-main` for an external resource the runtime manages — `runtime-drone` (persistence subprocess), `runtime-sandbox` (isolation subprocess), `runtime-mcp` (protocol-layer client for MCP servers, which are also subprocesses for stdio + HTTP endpoints for remote). The pattern: external resource = its own crate. Dep containment, independent coverage gate, clean swap path (rmcp 1.7.0 is mature with 4.7M downloads but if it ever blocks, a direct-JSON-RPC fallback stays viable per the B.5 `<interpretation_declarations>` slot).

Concrete deliverables:
1. **Workspace member.** New `crates/runtime-mcp/` directory with `Cargo.toml` declaring `rmcp = { version = "1.7.0", features = ["client", "transport-io", "transport-streamable-http-client-reqwest"] }` + tokio + serde + thiserror + (for tests) wiremock + rmcp test helpers. `Cargo.toml` (root) workspace `members` includes `crates/runtime-mcp`.
2. **Schema.** `schemas/mcp.v1.json` defines `McpServer` config (name, transport (stdio | http), command/url, args, env, auth_secret_ref) + `McpServerStatus` (connected | disconnected | health_pending | error) + `$defs/McpServerName` validated string. Run `cargo xtask regenerate-types` to populate `crates/runtime-core/src/generated/mcp.rs` + `src/types/mcp.ts`.
3. **Transport trait.** `crates/runtime-mcp/src/transport/mod.rs` declares a small async trait: `async fn connect(&self) -> Result<Connection, TransportError>` + `async fn disconnect(self) -> Result<(), TransportError>` + the connection type's `async fn invoke_tool(...)` + `async fn list_tools(...)` + `async fn health_check(...)`. Two implementations: `StdioTransport` (wraps `rmcp::transport::TokioChildProcess`) + `HttpTransport` (wraps `rmcp::transport::StreamableHttpClient`).
4. **Error mapping.** `crates/runtime-mcp/src/error.rs` declares `McpError` with variants `ConnectFailed`, `Transport`, `Protocol`, `Timeout`, `ToolNotFound`, `Cancelled` (mapping rmcp's error variants to runtime-mcp's stable surface). Implements `thiserror::Error`; schema-aligned for the future `error.v1.json` extension (M07+).
5. **Mock transport for unit tests.** `crates/runtime-mcp/src/transport/mock.rs` (`#[cfg(test)]` or `cfg(any(test, feature = "test-helpers"))`) implements the same trait with `tokio::io::duplex` for stdio path (M05.C1 archetype; per gotcha #72 + #77, buffer-vs-payload sizing must surface error paths) and an `httpmock`-backed fake for HTTP path (`wiremock` per M02.C archetype). Test helpers expose preset scripted responses (tool_list, tool_call_success, tool_call_failure, server_timeout).
6. **Unit tests at ≥95% line on `transport/` files.** Stdio path: connection + tool invocation happy path + tool not found + transport error + reconnect after peer drop + multi-call invariant. HTTP path: same surface via wiremock. Mock transport: trait conformance tests.
7. **Feature-gated integration smoke** (`--features integration`). `crates/runtime-mcp/tests/integration.rs` spawns the official reference `@modelcontextprotocol/server-filesystem` (npm package; assumed installed on the build machine, OR skipped if unavailable with a clear log message). Tests: connect → list_tools → invoke `read_file` against a tempfile → disconnect. M02.C precedent (the anthropic_smoke.rs gated integration test).

Not in this stage:
- Server install / uninstall (Stage C)
- Per-server auth via key_store (Stage C)
- Connection registry + health-ping loop (Stage C)
- §5a namespace resolution (Stage D)
- MCP dispatch through capability gates (Stage D)
- Renderer wiring (Stage E)

### B.2 Files to Change

| File | Status | Change |
|---|---|---|
| `Cargo.toml` (workspace root) | exists | Edit: add `crates/runtime-mcp` to `workspace.members`; add rmcp to `workspace.dependencies` with the 3 features |
| `crates/runtime-mcp/Cargo.toml` | **new** | Crate manifest; rmcp 1.7.0 + tokio + serde + thiserror; dev-dependencies wiremock + tempfile + tokio (with test-util) |
| `crates/runtime-mcp/src/lib.rs` | **new** | Public surface: `pub mod transport; pub mod error;` |
| `crates/runtime-mcp/src/transport/mod.rs` | **new** | Transport trait + Connection type + tool-call request/response shapes |
| `crates/runtime-mcp/src/transport/stdio.rs` | **new** | `StdioTransport` wrapping `rmcp::transport::TokioChildProcess` |
| `crates/runtime-mcp/src/transport/http.rs` | **new** | `HttpTransport` wrapping `rmcp::transport::StreamableHttpClient` |
| `crates/runtime-mcp/src/transport/mock.rs` | **new** | Mock transport (`tokio::io::duplex` for stdio + `wiremock` setup for HTTP); gated `#[cfg(any(test, feature = "test-helpers"))]` |
| `crates/runtime-mcp/src/error.rs` | **new** | `McpError` enum + thiserror impls + From conversions from rmcp errors |
| `crates/runtime-mcp/src/transport/tests.rs` OR `#[cfg(test)] mod tests` inline | **new** | Unit tests for each transport (≥95% line) |
| `crates/runtime-mcp/tests/integration.rs` | **new** | Feature-gated `--features integration` smoke against reference MCP server |
| `schemas/mcp.v1.json` | **new** | MCP server config + status schema |
| `crates/runtime-core/src/generated/mcp.rs` | **new** | typify-generated from `schemas/mcp.v1.json` |
| `src/types/mcp.ts` | **new** | json-schema-to-typescript-generated from `schemas/mcp.v1.json` |
| `crates/runtime-core/src/lib.rs` | exists | Edit: re-export `generated::mcp::*` if convention applies; check existing re-exports of generated/event.rs etc. |
| `Cargo.lock` | exists | Regenerated after `Cargo.toml` updates |
| `CHANGELOG.md` | exists | Edit: `[Unreleased]` notes M06.B — runtime-mcp crate + rmcp 1.7.0 transport |
| `docs/build-prompts/retrospectives/M06.B-retrospective.md` | **new** | Stage B retrospective |

Effort budget: ~8–10 hours of code execution (rmcp 1.7.0 is new third-party protocol; novel-protocol stages historically run at 1× per M02.C precedent). Largest piece is the transport trait + the two implementations + mock-transport authoring + the ≥95% unit-test surface.

### B.3 Detailed Changes

#### B.3.1 Cargo.toml workspace edit

```toml
# At workspace root Cargo.toml:
[workspace]
members = [
    # ... existing ...
    "crates/runtime-mcp",
]

[workspace.dependencies]
# ... existing ...
rmcp = { version = "1.7.0", default-features = false, features = ["client", "transport-io", "transport-streamable-http-client-reqwest"] }
```

`default-features = false` keeps the dependency tree minimal — rmcp's `server` + `transport-streamable-http-server` + tower-adapter features aren't needed for client-only use.

#### B.3.2 `crates/runtime-mcp/Cargo.toml`

```toml
[package]
name = "runtime-mcp"
version.workspace = true
edition.workspace = true
license.workspace = true

[lints]
workspace = true

[dependencies]
rmcp.workspace = true
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "process", "io-util", "sync", "time"] }
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
runtime-core = { path = "../runtime-core" }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
wiremock.workspace = true
tempfile.workspace = true

[features]
test-helpers = []  # gate the mock transport for downstream test consumers (Stage C+)
integration = []   # gate the reference-MCP-server smoke
```

The `test-helpers` feature lets `runtime-main`'s Stage C/D tests use the mock transport without exposing it in production builds.

#### B.3.3 Schema — `schemas/mcp.v1.json`

```jsonc
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://agent-runtime.local/schemas/mcp.v1.json",
  "title": "McpServerConfig",
  "type": "object",
  "required": ["name", "transport"],
  "properties": {
    "name": { "$ref": "#/$defs/McpServerName" },
    "transport": { "$ref": "#/$defs/McpTransport" },
    "auth_secret_ref": { "type": ["string", "null"], "description": "OS-keychain key for per-server auth (set by Stage C); null for unauthenticated servers" }
  },
  "$defs": {
    "McpServerName": { "type": "string", "minLength": 1, "maxLength": 64, "pattern": "^[a-z0-9][a-z0-9-]*$" },
    "McpTransport": {
      "oneOf": [
        {
          "type": "object",
          "required": ["type", "command"],
          "properties": {
            "type": { "const": "stdio" },
            "command": { "type": "string", "minLength": 1 },
            "args": { "type": "array", "items": { "type": "string" }, "default": [] },
            "env": { "type": "object", "additionalProperties": { "type": "string" }, "default": {} },
            "cwd": { "type": ["string", "null"] }
          }
        },
        {
          "type": "object",
          "required": ["type", "url"],
          "properties": {
            "type": { "const": "http" },
            "url": { "type": "string", "format": "uri" }
          }
        }
      ]
    },
    "McpServerStatus": { "type": "string", "enum": ["connected", "disconnected", "health_pending", "error"] }
  }
}
```

The discriminated union for `McpTransport` uses the established `type` field pattern (gotcha #26 — struct-shape variants required for serde(tag = "type")). Both variants are struct-shaped, not newtype.

#### B.3.4 Transport trait shape — `transport/mod.rs`

```rust
//! Transport abstraction for MCP clients.
//!
//! Two implementations: `StdioTransport` for local subprocess MCP servers and
//! `HttpTransport` for remote streamable-HTTP MCP servers. A mock transport is
//! available behind the `test-helpers` feature for in-process testing.

use crate::error::McpError;
use serde_json::Value;
use std::collections::BTreeMap;

/// Connection to a running MCP server. Drops cleanly on `disconnect` or panic.
#[async_trait::async_trait]
pub trait Connection: Send + Sync {
    /// Lists tools the server exposes. Cached after first call until disconnect.
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError>;

    /// Invokes a tool. Returns the result Value or an error.
    async fn invoke_tool(&self, name: &str, args: Value) -> Result<Value, McpError>;

    /// Health-check ping. Returns Ok(()) if the server responds within rmcp's default timeout.
    async fn health_check(&self) -> Result<(), McpError>;
}

/// Transport factory. Calling `connect` produces a live `Connection`.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&self) -> Result<Box<dyn Connection>, McpError>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}
```

Concrete impls (StdioTransport + HttpTransport) live in `transport/stdio.rs` + `transport/http.rs`; both delegate to rmcp's underlying transport types and translate rmcp errors to `McpError` via `From` impls.

#### B.3.5 `StdioTransport` shape — `transport/stdio.rs`

```rust
use rmcp::transport::TokioChildProcess;
use tokio::process::Command;

pub struct StdioTransport {
    command: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: Option<PathBuf>,
}

#[async_trait::async_trait]
impl Transport for StdioTransport {
    async fn connect(&self) -> Result<Box<dyn Connection>, McpError> {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);
        for (k, v) in &self.env { cmd.env(k, v); }
        if let Some(c) = &self.cwd { cmd.current_dir(c); }
        let transport = TokioChildProcess::new(cmd).await.map_err(McpError::from)?;
        let client = rmcp::ClientHandler::new(transport).serve().await.map_err(McpError::from)?;
        Ok(Box::new(StdioConnection { client }))
    }
}
```

Real rmcp 1.7.0 API may differ; the shape above is illustrative — Stage B's implementation reads `docs.rs/rmcp/1.7.0` per gotcha #74 (cross-platform cfg-gated code / new-API derivation) and adopts the verbatim shape. WEBCHECK discipline applies per gotcha #32 — any rmcp method/type referenced must match upstream docs at authoring time.

#### B.3.6 `HttpTransport` shape — `transport/http.rs`

Same trait surface; underlying `rmcp::transport::StreamableHttpClient` with a configurable base URL. The rmcp 1.7.0 streamable-http-client supports server-sent events for tool result streaming; transport translates to `McpError::Transport` on connection failure / `McpError::Protocol` on malformed JSON-RPC / `McpError::Timeout` on rmcp's default timeout. M02.C anthropic_sse archetype — wiremock-backed unit tests cover happy + auth + timeout + malformed-bytes-skipped + partial-chunk-reassembly + server-emitted-error paths.

#### B.3.7 Mock transport — `transport/mock.rs`

```rust
//! Mock transport for in-process testing. Substitutes a `tokio::io::duplex`-backed
//! transport for stdio paths and a `wiremock::MockServer`-backed transport for HTTP.

use crate::error::McpError;
use crate::transport::{Connection, McpTool, Transport};

#[cfg(any(test, feature = "test-helpers"))]
pub struct MockTransport {
    pub scripted_tools: Vec<McpTool>,
    pub scripted_responses: BTreeMap<String, Value>,
    pub scripted_errors: BTreeMap<String, McpError>,
}

// impl Transport + Connection delegate to scripted state.
```

Per gotcha #77, when testing peer-write-failure branches in a duplex-backed mock, reduce the buffer to ≤8 bytes (smaller than the smallest valid JSON-RPC frame) so the write blocks and the peer-drop error propagates. Per gotcha #72, dropping the peer's WRITER half (not reader) propagates EOF; tests asserting EOF behavior must drop the writer explicitly.

#### B.3.8 Error mapping — `error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("connection failed: {0}")]
    ConnectFailed(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("operation cancelled")]
    Cancelled,
}

impl From<rmcp::Error> for McpError {
    fn from(e: rmcp::Error) -> Self {
        // Map rmcp's error variants. Refer to docs.rs/rmcp/1.7.0/rmcp/enum.Error.html.
        match e {
            // ... per upstream variants
        }
    }
}
```

WEBCHECK at authoring time: the exact rmcp::Error variant set lives in `docs.rs/rmcp/1.7.0/rmcp/`; the From impl reads the docs and produces a verbatim match. Per gotcha #74, the match arm coverage must include every rmcp variant — if a new rmcp version adds a variant, `cargo check` catches the missing arm.

### B.4 Tests

#### B.4.1 Stdio path unit tests (`transport/stdio_tests.rs` or inline)

- `stdio_connect_succeeds_with_valid_command_path`
- `stdio_connect_fails_with_nonexistent_command_returns_connect_failed`
- `stdio_list_tools_returns_scripted_tools_via_mock_transport`
- `stdio_invoke_tool_succeeds_via_mock_transport`
- `stdio_invoke_tool_returns_tool_not_found_for_unknown_name`
- `stdio_health_check_passes_against_responsive_mock`
- `stdio_health_check_times_out_after_default_window` (use `tokio::test(start_paused = true)` + paused-time machinery per gotcha #72)
- `stdio_invoke_tool_twice_in_sequence_both_succeed` (multi-call invariant per gotcha #69)
- `stdio_invoke_returns_transport_error_when_peer_drops_writer` (per gotcha #72 + #77 — drop peer writer + small buffer)
- `stdio_disconnect_drops_subprocess_cleanly_on_normal_close`

#### B.4.2 HTTP path unit tests (`transport/http_tests.rs` or inline)

Wiremock-backed (M02.C `anthropic_wiremock.rs` archetype):

- `http_connect_succeeds_with_valid_url_returning_200`
- `http_connect_fails_with_404_returns_connect_failed`
- `http_connect_fails_with_timeout_returns_timeout`
- `http_invoke_tool_succeeds_with_scripted_sse_response`
- `http_invoke_tool_returns_protocol_error_on_malformed_jsonrpc`
- `http_invoke_tool_handles_partial_chunk_reassembly` (wire-format SSE; same archetype as anthropic_sse)
- `http_invoke_tool_skips_malformed_bytes_continues_stream`
- `http_health_check_passes`
- `http_invoke_tool_twice_in_sequence_both_succeed` (multi-call invariant)
- `http_invoke_returns_transport_error_when_server_drops_connection_mid_stream`

#### B.4.3 Mock transport conformance tests

The `MockTransport` must satisfy the same `Transport` trait contract as the real implementations:

- `mock_connect_returns_connection_with_scripted_tools`
- `mock_invoke_tool_returns_scripted_response`
- `mock_invoke_tool_returns_scripted_error`
- `mock_implements_send_plus_sync` (compile-time check via `static_assertions::assert_impl_all!`)

#### B.4.4 Schema regen check

- `cargo xtask regenerate-types --check` passes after `mcp.v1.json` lands + `generated/mcp.rs` + `src/types/mcp.ts` committed.

#### B.4.5 Integration smoke (feature-gated)

`crates/runtime-mcp/tests/integration.rs` (`#[cfg(feature = "integration")]`):

```rust
#[tokio::test]
async fn stdio_against_filesystem_mcp_server() {
    // Skip if npx @modelcontextprotocol/server-filesystem not available
    if !test_helpers::has_npx_command() { eprintln!("SKIP: npx not found"); return; }
    let temp = tempfile::TempDir::new().unwrap();
    let transport = StdioTransport::new(
        "npx",
        vec!["@modelcontextprotocol/server-filesystem".into(), temp.path().to_str().unwrap().into()],
    );
    let conn = transport.connect().await.expect("connect");
    let tools = conn.list_tools().await.expect("list_tools");
    assert!(tools.iter().any(|t| t.name == "read_file"));
    // ... invoke read_file against a tempfile
}
```

Same disposition as M02's `anthropic_smoke.rs` — manual gate; CI doesn't run by default; documentation as the spec of "what the SDK will call." Run manually via `cargo test -p runtime-mcp --features integration`.

#### B.4.6 Acceptance criteria

- [ ] `crates/runtime-mcp/` exists; cargo metadata returns it as a workspace member
- [ ] `cargo build -p runtime-mcp` passes (no `unsafe`; no `forbid(unsafe_code)` violation)
- [ ] `cargo test -p runtime-mcp --lib` — all unit tests pass
- [ ] `cargo llvm-cov --package runtime-mcp --ignore-filename-regex "src.main\.rs|generated|src.lib\.rs" --fail-under-lines 95` — runtime-mcp per-crate gate
- [ ] `cargo xtask regenerate-types --check` — mcp.v1.json round-trips cleanly to Rust + TS
- [ ] `cargo deny check` — rmcp 1.7.0 + transitive tree license-compatible (Apache-2.0/MIT) + no unmaintained warnings
- [ ] `cargo clippy -p runtime-mcp --all-targets -- -D warnings` — clean
- [ ] `cargo doc -p runtime-mcp --no-deps` — clean; all `pub` items doc-commented
- [ ] No new high-severity npm audit findings (transport is Rust-only; TS-side gets just the generated mcp.ts)
- [ ] CI-parity per G6: paste exact local commands run

### B.5 CLI Prompt

```xml
<work_stage_prompt id="M06.B">
  <context>
    Stage B of M06 (MCP Basic). Creates the `runtime-mcp` workspace crate
    as the protocol-layer dependency boundary for MCP. Adds rmcp 1.7.0
    with the `client` + `transport-io` (stdio) + `transport-streamable-
    http-client-reqwest` features. Ships the Transport trait + two
    implementations (stdio via TokioChildProcess; http via
    StreamableHttpClient) + a mock transport for unit tests + the
    `schemas/mcp.v1.json` schema. NEW safety primitive at ≥95% per-crate
    line coverage on transport files. Lifecycle (Stage C) + namespace/
    dispatch (Stage D) consume this crate's surface; do not blur the
    boundary. Novel-protocol stage; calibration anchor 1× per M02.C
    anthropic_sse precedent.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M06-mcp-basic.md (Background, Document Structure, Implementation Workflow, Pre-existing legacy file inventory, Stage A as immediate predecessor, Stage B sections B.1–B.4)</file>
    <file>docs/build-prompts/retrospectives/M06.A-retrospective.md (immediate predecessor; apply Decisions for next stage)</file>
    <file>agent-runtime-spec.md §5 (MCP Manager); §5a (Tool Namespace Resolution — informs Stage D; this stage's tool shape must accommodate it)</file>
    <file>docs/MVP-v0.1.md §M6 (acceptance criteria)</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #26 serde(tag) struct-variants, #32 cross-stack examples WEBCHECK, #43 typify root-oneOf, #57 top-level $ref breaks json-schema-to-typescript, #69 multi-call invariants, #72 tokio duplex EOF propagation, #73 typify oneOf non-Copy, #74 cfg-target-os new-API docs derivation, #77 duplex buffer-vs-payload)</file>
    <file>docs/adr/0002-tauri-rust-vs-electron.md (stack rationale — no third-party SDK if avoidable; rmcp is the exception because it IS the protocol library, not an SDK shim)</file>
    <file>https://docs.rs/rmcp/1.7.0/rmcp/ (WEBCHECK at authoring time; verbatim cite the trait + type names + version)</file>
    <file>https://modelcontextprotocol.io/specification/2025-11-25 (MCP wire format — JSON-RPC 2.0 over stdio + HTTP)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="M02.C wire-format precedent — wiremock-backed SSE unit tests; copy archetype for http transport tests">crates/runtime-main/src/providers/anthropic_sse.rs</file>
    <file purpose="M02.C wiremock setup pattern">crates/runtime-main/tests/anthropic_wiremock.rs</file>
    <file purpose="M05.C1 sandbox_ipc — tokio::io::duplex archetype for in-memory stdio transport tests">crates/runtime-main/src/sandbox_ipc/client.rs</file>
    <file purpose="M02.C feature-gated integration smoke pattern">crates/runtime-main/tests/anthropic_smoke.rs</file>
    <file purpose="schema regen workflow + xtask shape">crates/xtask/src/main.rs</file>
    <file purpose="schema authoring pattern — capability.v1.json is the most recent net-new schema (M05.B)">schemas/capability.v1.json</file>
    <file purpose="root Cargo.toml — workspace.members + workspace.dependencies setup pattern; mirrors how runtime-sandbox was added in M05.C1">Cargo.toml</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M06.A" decisions_file="docs/build-prompts/retrospectives/M06.A-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M06-mcp-basic.md" section="B.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the test plan's buckets. Stub the
      production surfaces just enough to make the test files compile
      (todo!() / unimplemented!() bodies are fine; the goal is link-time
      test discovery, not behavior). Confirm tests fail with right-reason
      errors per CLAUDE.md §5 (assertion failed / cannot find function /
      unresolved import / not-yet-implemented panic — NOT a test-file
      compile error and NOT a tautological pass). Commit as a STANDALONE
      `test(M06.&lt;stage&gt;): failing tests for ...` commit on
      claude/m06-mcp-basic BEFORE green-phase impl; the commit body
      pastes the first ~40 lines of cargo test output proving the
      expected-failure class. Surface the red-phase commit to the user;
      user approves before green phase begins.
    </red_phase>
    <green_phase>
      Implement until ALL failing tests pass. Do NOT modify the test
      files during implementation — if a test is wrong, fix it in a
      SEPARATE labelled follow-up commit with explanation, never silently
      in the impl commit. The impl commit body MUST state the verifiable
      audit-surface invariant: `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      rustfmt/clippy fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message (M06.B Decision;
      gotcha-candidate territory on third recurrence).
    </green_phase>
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

  <acceptance_criteria ref="docs/build-prompts/M06-mcp-basic.md" section="B.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M06-mcp-basic.md" section="Key constraints"/>

  <gates milestone="M06"/>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch_state" gate="git rev-parse --abbrev-ref HEAD must equal claude/m06-mcp-basic"/>
    <check name="m06_a_committed" gate="git log --oneline main..HEAD | head -2 must include the M06.A commit"/>
    <check name="rust_toolchain" gate="cargo --version must report the pinned version"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="Cargo.toml" verified="true"/>
    <claim type="file" path="crates/runtime-mcp/Cargo.toml" verified="false" note="Stage B creates this"/>
    <claim type="file" path="crates/runtime-mcp/src/lib.rs" verified="false" note="Stage B creates"/>
    <claim type="file" path="crates/runtime-mcp/src/transport/mod.rs" verified="false" note="Stage B creates"/>
    <claim type="file" path="crates/runtime-mcp/src/transport/stdio.rs" verified="false" note="Stage B creates"/>
    <claim type="file" path="crates/runtime-mcp/src/transport/http.rs" verified="false" note="Stage B creates"/>
    <claim type="file" path="crates/runtime-mcp/src/transport/mock.rs" verified="false" note="Stage B creates"/>
    <claim type="file" path="crates/runtime-mcp/src/error.rs" verified="false" note="Stage B creates"/>
    <claim type="file" path="schemas/mcp.v1.json" verified="false" note="Stage B creates"/>
    <claim type="file" path="crates/runtime-core/src/generated/mcp.rs" verified="false" note="Stage B regenerates via xtask"/>
    <claim type="file" path="src/types/mcp.ts" verified="false" note="Stage B regenerates"/>
    <claim type="file" path="crates/runtime-core/src/lib.rs" verified="true"/>
    <claim type="method" path="docs.rs/rmcp/1.7.0/rmcp/transport" symbol="TokioChildProcess" verified="WEBCHECK" note="confirm at authoring time per gotcha #74 — exact API may have shifted since 1.7.0 release"/>
    <claim type="method" path="docs.rs/rmcp/1.7.0/rmcp/transport" symbol="StreamableHttpClient" verified="WEBCHECK" note="confirm at authoring time per gotcha #74"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M06.A-retrospective.md" verified="true" note="Stage A committed before Stage B starts"/>
  </phase_doc_inventory_audit>

  <schema_drift_check gate="cargo xtask regenerate-types --check (must succeed after mcp.v1.json + generated/mcp.rs land together)"/>

  <schema_ref_audit>
    <ref schema="schemas/mcp.v1.json" path="#/$defs/McpServerName" verified="declared inline in this stage"/>
    <ref schema="schemas/mcp.v1.json" path="#/$defs/McpTransport" verified="declared inline in this stage"/>
    <ref schema="schemas/mcp.v1.json" path="#/$defs/McpServerStatus" verified="declared inline in this stage"/>
  </schema_ref_audit>

  <schema_audit>
    <survey pattern='"McpServerName"' purpose="confirm no existing $def with this name in any schemas/*.v1.json (other than this stage's new one)"/>
    <survey pattern='"McpTransport"' purpose="same"/>
    <survey pattern="runtime-mcp" purpose="confirm no existing crate with this name in workspace.members before adding"/>
  </schema_audit>

  <schema_root_check/>

  <architecture_check>
    <claim description="runtime-mcp crate is in workspace.members AND in workspace.dependencies' transitive tree only via runtime-main/runtime-drone/src-tauri using `runtime-mcp = { path = '../runtime-mcp' }` (when Stage C adds the dep); NO direct dep from runtime-core or other crates" verify="cargo metadata --format-version 1 ; expect runtime-mcp dependency_graph leaves clean"/>
    <claim description="rmcp is a direct dep of runtime-mcp ONLY; runtime-main does NOT depend on rmcp directly (runtime-mcp abstracts the protocol layer)" verify="grep 'rmcp' crates/runtime-main/Cargo.toml ; expect zero matches"/>
    <claim description="no unsafe code in runtime-mcp; workspace.lints applies forbid(unsafe_code) which the crate inherits per workspace.lints = true convention" verify="grep -n 'unsafe' crates/runtime-mcp/src/ -r ; expect zero matches except in // SAFETY: comment-only references (none expected for Stage B)"/>
    <claim description="Mock transport is test-helpers-feature-gated; production builds don't link the mock" verify="grep -B2 'pub struct MockTransport' crates/runtime-mcp/src/transport/mock.rs ; expect #[cfg(any(test, feature = test-helpers))] gate above"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="rmcp::" purpose="enumerate all rmcp API surfaces used in runtime-mcp; cross-reference against docs.rs/rmcp/1.7.0/rmcp/ at authoring time per WEBCHECK discipline"/>
    <grep pattern="serde_json::Value" purpose="confirm Value usage in tool args + responses is consistent with rmcp's API and with framework JSON's existing Value-typed payloads"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="rmcp" version="1.7.0" prefer_crates_io_name="true" source_authority="WEBCHECK docs.rs/rmcp/latest = 1.7.0 (released 2026-05-13) per modelcontextprotocol/rust-sdk GitHub" required_features="client,transport-io,transport-streamable-http-client-reqwest" audit="cargo deny check must pass — rmcp + transitive (tower, reqwest, tokio variants) all license-compatible (Apache-2.0/MIT); no unmaintained warnings"/>
    <dep name="wiremock" version="workspace" prefer_crates_io_name="true" source_authority="already in workspace.dev-dependencies from M02.C" required_features="N/A" audit="no change"/>
    <dep name="tempfile" version="workspace" prefer_crates_io_name="true" source_authority="already in workspace" required_features="N/A" audit="no change"/>
    <feature_interdependency crate="rmcp" function="N/A — top-level features only" home_feature="client" requires_features="transport-io for stdio AND transport-streamable-http-client-reqwest for http" reason="rmcp's client transports are independently feature-gated; both needed for v0.1 dual-transport scope"/>
  </dependency_audit_check>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-mcp" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.lib\.rs"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.key_store\.rs"/>
  </coverage_gate>

  <runtime_environment os="windows" note="Build agent on Windows 11; rmcp's stdio transport uses tokio::process which works on Windows (named-pipes-under-the-hood); http transport uses reqwest which is platform-agnostic. No platform-specific cfg expected in Stage B; if any appears, cite gotcha #74."/>

  <interpretation_declarations>
    <adopt spec_section="§5 MCP Manager" interpretation="rmcp 1.7.0 as the protocol library — official Rust SDK, 4.7M+ downloads, mature API per docs.rs/rmcp/1.7.0" alternative_interpretation="direct JSON-RPC 2.0 over tokio (hand-rolled wire format)" rationale="per MVP §M6 'uses rmcp if feature-complete enough; fallback is direct JSON-RPC over stdio' — rmcp 1.7.0 IS feature-complete for v0.1's stdio + streamable-http scope; using the upstream crate produces protocol correctness from a maintained reference rather than re-implementing the wire format"/>
    <adopt spec_section="§5 MCP Manager — transport scope" interpretation="stdio + streamable HTTP per MCP spec 2025-11-25; both transports ship in v0.1" alternative_interpretation="stdio-only for v0.1; HTTP deferred to v1.0" rationale="MVP §M6 acceptance criteria 'add server by URL or local path' — URL implies HTTP, local path implies stdio; the spec wants both; rmcp abstracts transport setup so the additional cost is small"/>
  </interpretation_declarations>

  <api_breaking_change_audit>
    <change api="runtime-mcp crate (new)" before_signature="N/A — crate did not exist" after_signature="public crate at workspace level with Transport + Connection + McpTool + McpError" call_sites="0 — Stage C is the first consumer" test_sites="0 — Stage B's own unit tests + Stage B's feature-gated integration test" recommendation="purely additive — new crate; existing crates unaffected"/>
  </api_breaking_change_audit>

  <gotchas>
    <trap>typify root-oneOf (gotcha #43) — `mcp.v1.json` uses oneOf for `McpTransport`; if typify panics, extract any validated inline strings to `$defs` with titles. The discriminated `type` field pattern (gotcha #26 — struct variants required for serde(tag = "type")) is correct for both stdio + http variants.</trap>
    <trap>top-level $ref breaks json-schema-to-typescript (gotcha #57) — `mcp.v1.json` keeps a concrete `type: object` at root; the McpServerConfig top-level is inline, only `$defs/<Name>` are reused.</trap>
    <trap>typify oneOf non-Copy variants (gotcha #73) — `McpTransport` variants nest String + arrays; if typify-generated PartialEq/Eq fails, compare via serde round-trip rather than `==`.</trap>
    <trap>cfg-target-os new-API derivation (gotcha #74) — rmcp's API may have shifted since 1.7.0 release; verbatim cite docs.rs/rmcp/1.7.0/rmcp/ at authoring time. Watch for method renames (e.g., `new` vs `new_with`), return-type wrappers (`Connection` vs `Box&lt;dyn Connection&gt;`), and trait bounds.</trap>
    <trap>tokio::io::duplex EOF propagation (gotcha #72) — mock stdio transport tests asserting EOF must drop the peer's WRITER half, not reader.</trap>
    <trap>duplex buffer-vs-payload (gotcha #77) — mock stdio transport tests asserting peer-write-failure branches need buffer ≤8 bytes so writes block immediately.</trap>
    <trap>Multi-call invariants (gotcha #69) — every public transport method has a *_twice_in_sequence_both_succeed test.</trap>
    <trap>Hand-authored cross-stack examples in build prompts (gotcha #32) — the rmcp API examples in this phase doc are illustrative shapes derived from the latest docs.rs snapshot but may need verbatim correction at authoring time per WEBCHECK discipline.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT add `rmcp` to `runtime-main`'s Cargo.toml directly. The runtime-mcp crate is the protocol boundary; runtime-main gains a dep on `runtime-mcp` in Stage C, not rmcp directly. Re-litigating this in Stage C is a signal that runtime-mcp's API surface is wrong.</warning>
    <warning>DO NOT implement the MCP server install / uninstall / health-ping loop in Stage B. Those are Stage C deliverables. Stage B's surface is the protocol primitive (Transport + Connection); Stage C wraps it with lifecycle management.</warning>
    <warning>DO NOT add `unsafe` to runtime-mcp. The workspace `forbid(unsafe_code)` lint applies; if a rmcp API requires unsafe, the answer is almost certainly a different rmcp API — surface and stop.</warning>
    <warning>The reference-MCP-server integration test (`tests/integration.rs`) is FEATURE-GATED; CI does NOT run it. It's the M02.C anthropic_smoke.rs analogue — manual smoke. The unit tests via mock transport are the canonical CI surface.</warning>
    <warning>Run `cargo xtask regenerate-types` AFTER editing `mcp.v1.json` and commit the generated files in the SAME commit as the schema edit. CI's regenerate-types --check enforces.</warning>
  </execution_warnings>

  <time_box estimate_hours="10"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>rmcp 1.7.0 API surface vs the illustrative shapes in this phase doc — note any drift discovered at authoring time per WEBCHECK discipline. Transport-trait shape suitability for Stage C consumption — surface any API tweak needed to make lifecycle wiring clean. Coverage outcome on the two transport files (stdio + http) — any holdouts mirror the M05.C2 OS-isolation pattern (excluded with rationale) vs the M02.C wire-format pattern (in-gate). Novel-protocol-stage calibration data — actual hours vs the 10h estimate; informs whether the 1× anchor holds for future novel-protocol work.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M06-mcp-basic.md" section="B.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log + retro listing)</item>
    <item>diff stat</item>
    <item>gate results per v1.6 canonical ordering (fmt → clippy --fix → clippy -D warnings → test → doc → audit → deny → llvm-cov on runtime-mcp gate at ≥95% + workspace gate + frontend lint/typecheck/test + validator + schema_drift_check + xtask regenerate-types --check; CI-parity per G6)</item>
    <item>runtime-mcp crate added to workspace; rmcp 1.7.0 + 3 features confirmed via cargo deny check + cargo tree -p rmcp</item>
    <item>WEBCHECK confirmations from docs.rs/rmcp/1.7.0/rmcp/ — paste verbatim signatures for TokioChildProcess + StreamableHttpClient as used</item>
    <item>per-module coverage on transport/stdio.rs + transport/http.rs + transport/mock.rs + error.rs (≥95% line each; document holdouts if any)</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from B.6</item>
    <item>explicit statement: "Stage M06.B is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### B.6 Commit Message

```
feat(runtime): M06 Stage B — runtime-mcp crate + rmcp 1.7.0 transport (stdio + http)

Creates the runtime-mcp workspace crate as the protocol-layer dependency
boundary for MCP. rmcp 1.7.0 (the official Rust SDK; 4.7M+ downloads;
released 2026-05-13) with the `client` + `transport-io` (stdio) +
`transport-streamable-http-client-reqwest` (http) features. Transport
trait + two implementations + mock transport for unit tests + the
`schemas/mcp.v1.json` schema for MCP server config + the McpError
mapping from rmcp's variants to runtime-mcp's stable surface.

NEW safety primitive at ≥95% per-crate line coverage on transport
files. Lifecycle (Stage C) + namespace/dispatch (Stage D) consume this
crate's surface; this stage establishes the boundary.

Cargo.toml workspace:
- workspace.members += "crates/runtime-mcp"
- workspace.dependencies += rmcp = { version = "1.7.0", default-features
  = false, features = ["client", "transport-io",
  "transport-streamable-http-client-reqwest"] }

Crate layout:
- crates/runtime-mcp/src/lib.rs: pub mod transport; pub mod error;
- crates/runtime-mcp/src/transport/mod.rs: Transport + Connection traits
  + McpTool data shape.
- crates/runtime-mcp/src/transport/stdio.rs: StdioTransport wrapping
  rmcp::transport::TokioChildProcess.
- crates/runtime-mcp/src/transport/http.rs: HttpTransport wrapping
  rmcp::transport::StreamableHttpClient (M02.C anthropic_sse archetype
  for wiremock-backed unit tests).
- crates/runtime-mcp/src/transport/mock.rs: MockTransport gated behind
  `#[cfg(any(test, feature = "test-helpers"))]`; tokio::io::duplex for
  stdio path (M05.C1 sandbox_ipc archetype) + scripted-response setup
  for http path.
- crates/runtime-mcp/src/error.rs: McpError + thiserror impls + From
  conversions from rmcp::Error variants.

Schema:
- schemas/mcp.v1.json: McpServerConfig + McpTransport (oneOf stdio/http)
  + McpServerStatus + McpServerName validated string.
- crates/runtime-core/src/generated/mcp.rs +
  src/types/mcp.ts: regenerated via cargo xtask regenerate-types.

Tests:
- Stdio path: 10 unit tests covering connect happy + connect failure +
  list_tools + invoke_tool happy + tool not found + health_check happy
  + health_check timeout (paused-time) + multi-call invariant + transport
  error via peer-drop (per gotchas #72 + #77) + disconnect cleanly.
- HTTP path: 10 unit tests via wiremock covering connect + 404 +
  timeout + invoke + protocol error + partial chunk reassembly + malformed
  bytes skipped + health_check + multi-call invariant + stream-drop.
- Mock transport: 4 conformance tests + Send+Sync compile-time check.
- Schema regen check: cargo xtask regenerate-types --check passes.
- Feature-gated integration smoke (--features integration) against
  @modelcontextprotocol/server-filesystem; skipped in CI per M02.C
  precedent.

Coverage: runtime-mcp per-crate ≥95% line on transport/* + error.rs;
mock.rs counted (cfg-gated but exercised by tests). lib.rs excluded as
pub-mod-declarations boilerplate per the runtime-sandbox precedent.

No `unsafe` in runtime-mcp; workspace forbid(unsafe_code) holds.

cargo deny check passes — rmcp + transitive (tower, reqwest, tokio
variants) all Apache-2.0/MIT-licensed; no unmaintained warnings.

Not in this stage: server install / uninstall (Stage C), per-server
auth (Stage C), connection lifecycle / health-ping loop (Stage C),
§5a namespace resolution (Stage D), MCP dispatch through capability
gates (Stage D), renderer UI (Stage E).

https://claude.ai/code/session_<id>
```

---

## Stage C — `runtime-mcp::client` lifecycle (server install + auth + connection mgmt)

### C.1 Problem Statement

Wrap the Stage B transport primitive with **server lifecycle management** — Add/Remove/Test operations against MCP servers, per-server auth secrets via the M02 `key_store` keychain surface, connection management (connect/disconnect/health-ping with default-reconnect via rmcp), and SQLite-persisted `mcp_servers` registry (the table already exists from M02 — Stage C lights it up). Audit emissions for `mcp_installed` / `mcp_uninstalled` / `mcp_auth_granted` via the M05.E writer surface.

The L4 tier system (M05.D) gates MCP server installation per the existing capability surface — installing an MCP server with `shell: true` capabilities from a Promoted-tier user falls back to Novice review per the existing tier matrix; M06.C does NOT introduce new tier shapes, just calls the existing `tier::evaluator::allows` at install time.

Concrete deliverables:
1. **`crates/runtime-mcp/src/client/mod.rs`** — `McpClient` struct managing N active connections. Public methods: `add_server(config: McpServerConfig, auth: Option<String>) -> Result<(), McpError>`, `remove_server(name: &str) -> Result<(), McpError>`, `test_connection(name: &str) -> Result<Vec<McpTool>, McpError>` (connect + list_tools + disconnect; for the Settings panel's "Test" button), `list_servers() -> Vec<McpServerSummary>`, `get_connection(name: &str) -> Option<Arc<dyn Connection>>` (for Stage D's dispatch).
2. **`crates/runtime-mcp/src/client/lifecycle.rs`** — connect / disconnect / health-ping loop. Health-pings every 30s (default; configurable). Server going from `connected` → `error` emits a `mcp_missing` event (the existing M05.A variant) and routes through the existing `on_gap` HITL trigger.
3. **`crates/runtime-mcp/src/client/registry.rs`** — SQLite read/write against the `mcp_servers` table. Path-agnostic: `Registry::open(path: &Path)`. Tauri shell resolves `AppHandle::path().app_local_data_dir().join("mcp_servers.sqlite")` and passes it in (M05.D + M05.E archetype). Schema migration if needed (the table exists from M01; Stage C confirms shape + adds any missing columns).
4. **`crates/runtime-mcp/src/client/auth.rs`** — per-server auth via a trait `SecretStore`. The trait is implemented by `runtime-main::key_store::KeyStore` (the M02 `keyring`-backed implementation). The auth_secret_ref string is the keychain key; `auth.rs` exposes `store_secret(ref: &str, secret: &str)` + `fetch_secret(ref: &str) -> Result<String>` against the trait.
5. **Audit emissions.** `mcp_installed` on successful `add_server` (records name + transport type + presence-of-auth). `mcp_uninstalled` on successful `remove_server`. `mcp_auth_granted` on successful `store_secret` (records name; secret is never logged). Audit writes via a `Box<dyn AuditWriter>` injected at `McpClient::new_with_audit(writer)` — the M05.E `AuditWriter` trait is the contract.
6. **New event variants**: `mcp_installed`, `mcp_uninstalled`, `mcp_auth_granted` — added to `schemas/event.v1.json`. Regenerate types. graphStore branches added in Stage E.
7. **SQLite `mcp_servers` table** — M01 scaffolded the table; Stage C lights up the insert/select/update queries with proper PRAGMA + indexes. Schema migration file if column adds are needed (likely yes: `auth_secret_ref` + `created_at` + `last_health_check`).
8. **≥95% per-crate coverage on `runtime-mcp::client`** — same crate as Stage B; the gate continues. Audit emissions and SQLite paths get path-agnostic + injection-based tests.

Not in this stage:
- §5a tool namespace resolution (Stage D)
- MCP dispatch through L1+L2a gates (Stage D)
- Renderer UI (Stage E)

### C.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-mcp/Cargo.toml` | exists (from B) | Edit: add `rusqlite` (workspace dep) + `runtime-main` (path = "../runtime-main") for AuditWriter trait + tracing for telemetry; no new third-party crates |
| `crates/runtime-mcp/src/lib.rs` | exists | Edit: `pub mod client;` |
| `crates/runtime-mcp/src/client/mod.rs` | **new** | McpClient struct + public API surface |
| `crates/runtime-mcp/src/client/lifecycle.rs` | **new** | connect/disconnect/health-ping loop |
| `crates/runtime-mcp/src/client/registry.rs` | **new** | SQLite-backed mcp_servers registry; path-agnostic |
| `crates/runtime-mcp/src/client/auth.rs` | **new** | SecretStore trait + thin wrappers |
| `crates/runtime-mcp/src/client/error.rs` | **new** | LifecycleError; thiserror; From conversions from McpError + rusqlite::Error |
| `crates/runtime-mcp/tests/client_lifecycle.rs` | **new** | Integration tests for add/remove/test/health flow (mock transport + tempfile SQLite) |
| `crates/runtime-mcp/tests/registry.rs` | **new** | SQLite registry tests against tempfile path |
| `crates/runtime-mcp/tests/auth.rs` | **new** | SecretStore trait tests against in-memory fake |
| `crates/runtime-drone/migrations/002_mcp_servers.sql` | **new** | Migration that adds missing columns to the existing mcp_servers table (per M01 scaffold) — auth_secret_ref + created_at + last_health_check |
| `crates/runtime-drone/src/db.rs` | exists | Edit: extend the migration runner to pick up `002_mcp_servers.sql`; no schema-shape changes beyond the migration |
| `schemas/event.v1.json` | exists | Edit: add `mcp_installed`, `mcp_uninstalled`, `mcp_auth_granted` event variants |
| `crates/runtime-core/src/generated/event.rs` | exists | Regenerated |
| `src/types/agent_event.ts` | exists | Regenerated |
| `src-tauri/src/main.rs` | exists | Edit: wire `McpClient::new_with_audit(...)` at app startup; resolve `AppHandle::path().app_local_data_dir().join("mcp_servers.sqlite")` (M05.D + M05.E archetype) |
| `src-tauri/src/commands.rs` | exists | Edit: add Tauri commands `mcp_add_server(config) -> Result<()>`, `mcp_remove_server(name)`, `mcp_test_connection(name) -> Result<Vec<McpTool>>`, `mcp_list_servers() -> Vec<McpServerSummary>`. Renderer wires in Stage E |
| `CHANGELOG.md` | exists | Edit |
| `docs/build-prompts/retrospectives/M06.C-retrospective.md` | **new** | Stage C retrospective |

Effort budget: ~6–8 hours. Largest piece is the lifecycle + registry surface; auth is a thin trait wrap of M02's key_store.

### C.3 Detailed Changes

#### C.3.1 `McpClient` shape

```rust
pub struct McpClient {
    transports: BTreeMap<String, Arc<dyn Transport>>,
    connections: RwLock<BTreeMap<String, Arc<dyn Connection>>>,
    registry: Arc<Registry>,
    secret_store: Arc<dyn SecretStore>,
    audit: Option<Arc<dyn AuditWriter>>,
}

impl McpClient {
    pub fn new_with_audit(
        registry: Arc<Registry>,
        secret_store: Arc<dyn SecretStore>,
        audit: Arc<dyn AuditWriter>,
    ) -> Self { /* ... */ }

    pub async fn add_server(&self, config: McpServerConfig, auth: Option<String>) -> Result<(), LifecycleError> {
        // 1. Persist to registry (SQLite)
        // 2. Persist auth (if Some) via secret_store
        // 3. Construct Transport from config
        // 4. Test connection (connect + disconnect)
        // 5. Audit: mcp_installed (+ mcp_auth_granted if auth was set)
        // Note: actual connection is established on next get_connection() call OR on app startup's "reconnect-known-servers" sweep
    }

    pub async fn remove_server(&self, name: &str) -> Result<(), LifecycleError> {
        // 1. Disconnect (if connected)
        // 2. Remove from registry
        // 3. Remove auth secret (if present)
        // 4. Audit: mcp_uninstalled
    }

    pub async fn test_connection(&self, name: &str) -> Result<Vec<McpTool>, LifecycleError> { /* connect + list_tools + disconnect; no persistence */ }

    pub async fn get_connection(&self, name: &str) -> Result<Arc<dyn Connection>, LifecycleError> {
        // Return cached connection if active, otherwise connect + cache + return
    }
}
```

#### C.3.2 Lifecycle loop — health pings

```rust
// In lifecycle.rs:
pub fn spawn_health_pinger(client: Arc<McpClient>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            for (name, conn) in client.snapshot_connections().iter() {
                if let Err(e) = conn.health_check().await {
                    // emit mcp_missing event + disconnect
                    client.handle_health_failure(name, e).await;
                }
            }
        }
    })
}
```

The `handle_health_failure` emits the existing `mcp_missing` event variant (M05.A) with `severity: critical` (existing field per M05's enrichment) + the existing `on_gap` HITL trigger fires — no new event variant or trigger for the offline case. The `mcp_request_blocked` event variant (Stage D) is distinct: blocked at dispatch time by capability check, not by transport failure.

#### C.3.3 Registry shape — `client/registry.rs`

```rust
pub struct Registry {
    conn: Mutex<rusqlite::Connection>,
}

impl Registry {
    pub fn open(path: &Path) -> Result<Self, LifecycleError> {
        let conn = rusqlite::Connection::open(path)?;
        // PRAGMA in order per gotcha #13:
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            PRAGMA foreign_keys = ON;
        ")?;
        // Run migrations (the existing migration runner from M04 plan/task work picks up 002_mcp_servers.sql)
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn insert(&self, config: &McpServerConfig) -> Result<(), LifecycleError> { /* ... */ }
    pub fn remove(&self, name: &str) -> Result<(), LifecycleError> { /* ... */ }
    pub fn list(&self) -> Result<Vec<McpServerConfig>, LifecycleError> { /* ... */ }
    pub fn get(&self, name: &str) -> Result<Option<McpServerConfig>, LifecycleError> { /* ... */ }
    pub fn update_last_health(&self, name: &str, ts: i64) -> Result<(), LifecycleError> { /* ... */ }
}
```

Path-agnostic per CLAUDE.md §9 + docs/style.md archetype. Tests use tempfile-backed paths.

#### C.3.4 Migration — `002_mcp_servers.sql`

```sql
-- Add columns to the existing mcp_servers table (M01 scaffold).
-- The table currently has: id, name, command. Stage C adds:
ALTER TABLE mcp_servers ADD COLUMN transport_type TEXT NOT NULL DEFAULT 'stdio';
ALTER TABLE mcp_servers ADD COLUMN url TEXT;  -- for http transport
ALTER TABLE mcp_servers ADD COLUMN args TEXT NOT NULL DEFAULT '[]';  -- JSON array
ALTER TABLE mcp_servers ADD COLUMN env TEXT NOT NULL DEFAULT '{}';  -- JSON object
ALTER TABLE mcp_servers ADD COLUMN cwd TEXT;
ALTER TABLE mcp_servers ADD COLUMN auth_secret_ref TEXT;
ALTER TABLE mcp_servers ADD COLUMN created_at INTEGER NOT NULL DEFAULT (strftime('%s','now') * 1000);
ALTER TABLE mcp_servers ADD COLUMN last_health_check INTEGER;

CREATE INDEX IF NOT EXISTS idx_mcp_servers_name ON mcp_servers(name);
```

The exact existing column set must be verified at authoring time via `grep -A20 "CREATE TABLE.*mcp_servers" crates/runtime-drone/migrations/`. Adjust the ALTERs accordingly.

#### C.3.5 Auth — `SecretStore` trait

```rust
#[async_trait::async_trait]
pub trait SecretStore: Send + Sync {
    async fn store_secret(&self, ref_: &str, secret: &str) -> Result<(), LifecycleError>;
    async fn fetch_secret(&self, ref_: &str) -> Result<String, LifecycleError>;
    async fn remove_secret(&self, ref_: &str) -> Result<(), LifecycleError>;
}
```

`runtime-main::key_store::KeyStore` adds an impl (thin delegation to the existing `keyring`-backed M02 surface). Stage C's tests use an `InMemorySecretStore` fake.

#### C.3.6 New event variants — schema additions

```jsonc
{
  "type": "object",
  "title": "McpInstalled",
  "required": ["type", "name", "transport_type", "has_auth"],
  "properties": {
    "type": { "const": "mcp_installed" },
    "name": { "$ref": "mcp.v1.json#/$defs/McpServerName" },
    "transport_type": { "type": "string", "enum": ["stdio", "http"] },
    "has_auth": { "type": "boolean" }
  }
}
// + parallel mcp_uninstalled (type + name)
// + mcp_auth_granted (type + name)
```

Cross-schema $ref per the v1.6 `<schema_ref_audit>` slot. Stage A's `narrowed_from` addition didn't cross schemas; Stage C's variants reference `mcp.v1.json#/$defs/McpServerName` which Stage B created.

#### C.3.7 Audit emissions

```rust
// In add_server:
if let Some(audit) = &self.audit {
    audit.log(AuditEntry::mcp_installed(name.clone(), config.transport_type(), auth.is_some())).await?;
    if auth.is_some() {
        audit.log(AuditEntry::mcp_auth_granted(name.clone())).await?;
    }
}
```

Uses the existing M05.E `AuditWriter::log` trait method; no new method addition. New `AuditEntry::mcp_*` constructors in `crates/runtime-main/src/audit/entry.rs` — minor extensions following the M05.E builder pattern.

### C.4 Tests

#### C.4.1 Client lifecycle integration tests

`crates/runtime-mcp/tests/client_lifecycle.rs`:

- `add_server_persists_to_registry_and_audits_mcp_installed`
- `add_server_with_auth_persists_secret_and_audits_mcp_auth_granted`
- `add_server_failing_test_connection_does_not_persist`
- `remove_server_disconnects_and_removes_from_registry_and_audits_mcp_uninstalled`
- `remove_server_removes_auth_secret_when_present`
- `test_connection_returns_tools_list_without_persistence`
- `test_connection_returns_error_on_unreachable_server`
- `get_connection_returns_cached_connection_on_second_call`
- `health_check_failure_emits_mcp_missing_event`
- `add_server_twice_in_sequence_with_distinct_names_both_succeed` (multi-call)

#### C.4.2 Registry tests

`crates/runtime-mcp/tests/registry.rs`:

- `registry_open_initializes_schema_via_migration`
- `registry_insert_persists_config`
- `registry_get_returns_persisted_config`
- `registry_list_returns_all_configs`
- `registry_remove_deletes_row`
- `registry_update_last_health_persists_timestamp`
- `registry_open_twice_in_sequence_does_not_re_run_migrations` (idempotent migration check; multi-call)

#### C.4.3 Auth tests

`crates/runtime-mcp/tests/auth.rs`:

- `secret_store_round_trip_via_in_memory_fake`
- `secret_store_fetch_returns_error_for_missing_ref`
- `secret_store_remove_then_fetch_returns_error`
- `secret_store_store_then_fetch_twice_in_sequence_returns_same_secret` (multi-call)

#### C.4.4 Schema regen check

`cargo xtask regenerate-types --check` passes after event.v1.json adds.

#### C.4.5 Acceptance criteria

- [ ] `cargo test -p runtime-mcp --tests client_lifecycle registry auth` all pass
- [ ] `cargo llvm-cov --package runtime-mcp --ignore-filename-regex "src.main\.rs|generated|src.lib\.rs" --fail-under-lines 95` runtime-mcp gate holds
- [ ] `cargo xtask regenerate-types --check` passes
- [ ] `cargo deny check` — rusqlite added (already in workspace) — no new license issues
- [ ] Tauri commands compile (`cargo check -p src-tauri`); commands wired into `main.rs` invoke_handler
- [ ] CI-parity per G6

### C.5 CLI Prompt

```xml
<work_stage_prompt id="M06.C">
  <context>
    Stage C of M06 (MCP Basic). Wraps the Stage B transport primitive
    with server lifecycle management: Add/Remove/Test operations,
    per-server auth via M02 key_store, connection management with
    health-ping loop, and SQLite-persisted mcp_servers registry. Audit
    emissions (mcp_installed / mcp_uninstalled / mcp_auth_granted) via
    the M05.E writer surface. Adds 3 new event variants to event.v1.json
    + extends the M01-scaffolded mcp_servers table via migration
    002_mcp_servers.sql. Path-agnostic persistence per CLAUDE.md §9 +
    docs/style.md archetype. Tauri commands wired so Stage E renderer
    can invoke. NEW safety primitive coverage continues on the
    runtime-mcp crate (≥95%).
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M06-mcp-basic.md (Stage C sections C.1–C.4 + Stage B as immediate predecessor)</file>
    <file>docs/build-prompts/retrospectives/M06.B-retrospective.md</file>
    <file>agent-runtime-spec.md §5 MCP Manager (lifecycle); §13.5 dev logging (audit emissions); §8.security L5 (audit surface)</file>
    <file>docs/MVP-v0.1.md §M6</file>
    <file>docs/style.md (path-agnostic persistence archetype)</file>
    <file>docs/adr/0007-in-process-hitl-seam-architecture.md (forward applicability — MCP elicitation prompts reuse HitlSeam)</file>
    <file>docs/gotchas.md (#13 SQLite WAL pragmas, #29 keyring backend feature flags, #43 typify root-oneOf, #69 multi-call, #73 typify oneOf non-Copy)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="M02 key_store — Stage C adds a SecretStore trait impl">crates/runtime-main/src/key_store.rs</file>
    <file purpose="M05.E AuditWriter — Stage C calls log() with new mcp_* entries">crates/runtime-main/src/audit/writer.rs</file>
    <file purpose="M05.E audit entry builder pattern — Stage C extends with mcp_* constructors">crates/runtime-main/src/audit/entry.rs</file>
    <file purpose="M01 SQLite scaffold — mcp_servers table exists; Stage C extends via migration">crates/runtime-drone/src/db.rs</file>
    <file purpose="M04 plan_projector migration pattern — Stage C follows for 002_mcp_servers.sql">crates/runtime-drone/migrations/001_plans_tasks.sql</file>
    <file purpose="M05.D persistence archetype — Stage C mirrors the path-agnostic open(path: &amp;Path) pattern">crates/runtime-main/src/tier/persistence.rs</file>
    <file purpose="M05.E audit file_path archetype — Tauri shell resolves directory">crates/runtime-main/src/audit/file_path.rs</file>
    <file purpose="src-tauri Tauri command wiring pattern">src-tauri/src/commands.rs</file>
    <file purpose="src-tauri main.rs — Tauri builder + invoke_handler setup">src-tauri/src/main.rs</file>
    <file purpose="Stage B transport surface — Stage C consumes via Arc&lt;dyn Transport&gt; + Connection">crates/runtime-mcp/src/transport/mod.rs</file>
    <file purpose="Stage B mock transport — Stage C uses for lifecycle integration tests">crates/runtime-mcp/src/transport/mock.rs</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M06.A" decisions_file="docs/build-prompts/retrospectives/M06.A-retrospective.md"/>
    <stage id="M06.B" decisions_file="docs/build-prompts/retrospectives/M06.B-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M06-mcp-basic.md" section="C.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the test plan's buckets. Stub the
      production surfaces just enough to make the test files compile
      (todo!() / unimplemented!() bodies are fine; the goal is link-time
      test discovery, not behavior). Confirm tests fail with right-reason
      errors per CLAUDE.md §5 (assertion failed / cannot find function /
      unresolved import / not-yet-implemented panic — NOT a test-file
      compile error and NOT a tautological pass). Commit as a STANDALONE
      `test(M06.&lt;stage&gt;): failing tests for ...` commit on
      claude/m06-mcp-basic BEFORE green-phase impl; the commit body
      pastes the first ~40 lines of cargo test output proving the
      expected-failure class. Surface the red-phase commit to the user;
      user approves before green phase begins.
    </red_phase>
    <green_phase>
      Implement until ALL failing tests pass. Do NOT modify the test
      files during implementation — if a test is wrong, fix it in a
      SEPARATE labelled follow-up commit with explanation, never silently
      in the impl commit. The impl commit body MUST state the verifiable
      audit-surface invariant: `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      rustfmt/clippy fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message (M06.B Decision;
      gotcha-candidate territory on third recurrence).
    </green_phase>
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

  <acceptance_criteria ref="docs/build-prompts/M06-mcp-basic.md" section="C.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M06-mcp-basic.md" section="Key constraints"/>

  <gates milestone="M06"/>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch_state" gate="git rev-parse --abbrev-ref HEAD must equal claude/m06-mcp-basic"/>
    <check name="m06_a_b_committed" gate="git log --oneline main..HEAD | head -3 must include M06.A + M06.B commits"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="crates/runtime-mcp/Cargo.toml" verified="true" note="Stage B created"/>
    <claim type="file" path="crates/runtime-mcp/src/transport/mod.rs" verified="true" note="Stage B created"/>
    <claim type="file" path="crates/runtime-mcp/src/transport/mock.rs" verified="true" note="Stage B created"/>
    <claim type="file" path="crates/runtime-main/src/key_store.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/audit/writer.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/audit/entry.rs" verified="true"/>
    <claim type="file" path="crates/runtime-drone/src/db.rs" verified="true"/>
    <claim type="file" path="crates/runtime-drone/migrations/001_plans_tasks.sql" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/tier/persistence.rs" verified="true" note="archetype for path-agnostic persistence"/>
    <claim type="file" path="crates/runtime-main/src/audit/file_path.rs" verified="true" note="archetype for Tauri-shell-resolves-directory"/>
    <claim type="method" path="crates/runtime-main/src/audit/writer.rs" symbol="log" verified="true" note="existing M05.E method; Stage C calls with mcp_* entries"/>
    <claim type="method" path="crates/runtime-main/src/audit/entry.rs" symbol="mcp_installed" verified="false" note="Stage C adds this constructor"/>
    <claim type="method" path="crates/runtime-main/src/audit/entry.rs" symbol="mcp_uninstalled" verified="false" note="Stage C adds"/>
    <claim type="method" path="crates/runtime-main/src/audit/entry.rs" symbol="mcp_auth_granted" verified="false" note="Stage C adds"/>
    <claim type="struct_field" path="crates/runtime-drone/migrations/000_initial.sql" symbol="mcp_servers" verified="WEBCHECK-NEEDED" note="confirm M01 scaffolded the table name + initial columns; Stage C's migration adds columns relative to that baseline"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M06.B-retrospective.md" verified="true"/>
  </phase_doc_inventory_audit>

  <schema_drift_check gate="cargo xtask regenerate-types --check after event.v1.json mcp_* additions"/>

  <schema_ref_audit>
    <ref schema="schemas/event.v1.json" path="mcp.v1.json#/$defs/McpServerName" verified="true" note="Stage B defined; cross-schema reference"/>
  </schema_ref_audit>

  <schema_root_check/>

  <architecture_check>
    <claim description="McpClient is constructed via new_with_audit; the audit writer is injected (not constructed inside); same pattern as M05.E injection points" verify="grep 'new_with_audit' crates/runtime-mcp/src/client/mod.rs ; expect constructor signature taking Arc&lt;dyn AuditWriter&gt;"/>
    <claim description="Registry::open accepts path: &amp;Path (path-agnostic); Tauri shell resolves directory" verify="grep 'pub fn open' crates/runtime-mcp/src/client/registry.rs ; expect signature pub fn open(path: &amp;Path)"/>
    <claim description="No new IPC variant for MCP elicitation prompts — reuse HitlSeam per ADR-0007" verify="grep -n 'McpPromptSeam\\|mcp_seam' crates/runtime-mcp/src/ -r ; expect zero matches"/>
    <claim description="mcp_missing event reused for health-check failure routing; NO new variant for offline servers" verify="grep -B5 'mcp_missing' crates/runtime-mcp/src/client/lifecycle.rs ; expect existing variant reuse"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="mcp_servers" purpose="enumerate all references to the table name; confirm migration adds columns rather than creating a new table"/>
    <grep pattern="key_store::" purpose="confirm Stage C's SecretStore trait impl in runtime-main/src/key_store.rs delegates to existing keyring surface"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="rusqlite" version="workspace" prefer_crates_io_name="true" source_authority="already in workspace from M01" required_features="bundled" audit="no change"/>
    <dep name="runtime-main" version="path" prefer_crates_io_name="N/A" source_authority="workspace member" required_features="N/A" audit="adds intra-workspace dep — runtime-mcp now depends on runtime-main for AuditWriter trait and SecretStore impl"/>
  </dependency_audit_check>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-mcp" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.lib\.rs"/>
  </coverage_gate>

  <runtime_environment os="windows" note="Build on Windows; rusqlite bundled feature builds; M01 SQLite WAL pattern carries forward"/>

  <interpretation_declarations>
    <adopt spec_section="§5 MCP Manager — health monitoring" interpretation="30s default health-ping interval per server; rmcp's default-reconnect handles transient failures; sustained failure (3 consecutive ping failures) emits mcp_missing event + routes through on_gap HITL" alternative_interpretation="rmcp's transport-level reconnect alone with no runtime-mcp polling" rationale="rmcp's reconnect is transport-level; the runtime needs an application-level observer so the user-visible graph reflects offline state, and the existing on_gap HITL trigger handles the user prompt — reusing rather than introducing a new failure pathway"/>
  </interpretation_declarations>

  <api_breaking_change_audit>
    <change api="crates/runtime-main/src/audit/entry.rs AuditEntry" before_signature="builders for capability_granted, capability_denied, tier_transition, framework_loaded, gap_detected" after_signature="adds mcp_installed, mcp_uninstalled, mcp_auth_granted builders" call_sites="0 — Stage C is first caller" test_sites="0 — Stage C's tests" recommendation="purely additive — new constructors; existing builders unchanged"/>
    <change api="crates/runtime-main/src/key_store.rs KeyStore" before_signature="set_api_key + get_api_key (M02)" after_signature="impl SecretStore for KeyStore" call_sites="2 (set_api_key + get_api_key from existing Tauri commands)" test_sites="existing key_store_with tests" recommendation="purely additive — trait impl; existing methods unchanged"/>
  </api_breaking_change_audit>

  <existing_pattern_audit>
    <pattern grep_for="DroneCommand::" rationale="if Stage C surfaces a NEW DroneCommand variant for MCP operations, every existing match arm + irrefutable bindings of DroneCommand variants would break — verify zero need before authoring" affected_files="0 expected" remediation="MCP operations should NOT add DroneCommand variants; drone is audit, not orchestrator per ADR-0007; if a variant seems necessary, surface in retrospective"/>
  </existing_pattern_audit>

  <gotchas>
    <trap>SQLite WAL pragmas (gotcha #13) — apply in order: journal_mode = WAL → synchronous = NORMAL → busy_timeout = 5000 → foreign_keys = ON. Stage C's Registry::open MUST follow this order.</trap>
    <trap>keyring backend feature flags (gotcha #29) — Stage C's SecretStore impl delegates to M02's KeyStore which already has the proper feature flags. Do not re-instantiate keyring with default features; reuse the M02 surface.</trap>
    <trap>Multi-call invariants (gotcha #69) — every public method has a *_twice_in_sequence test.</trap>
    <trap>Tests-pass-but-contract-fails (gotcha #66) — assert audit emissions are correlated (e.g., a successful add_server with auth emits BOTH mcp_installed AND mcp_auth_granted in order, not just one).</trap>
    <trap>Path-agnostic persistence — Registry::open accepts path: &amp;Path; tests use tempfile-backed paths. Do NOT couple Registry to AppHandle / Tauri APIs internally.</trap>
    <trap>Migration idempotency — Stage C's 002_mcp_servers.sql must be idempotent (CREATE INDEX IF NOT EXISTS, ALTER TABLE only if column doesn't exist). The migration runner from M04 plan/task work handles re-runs gracefully.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT introduce a new HITL trigger for MCP server offline — reuse `on_gap` (existing M04.E trigger) routing the `mcp_missing` event variant (existing M05.A). The user-visible behavior of "MCP server went offline" is structurally identical to "MCP server reference is missing at framework load" — same trigger, same modal variant.</warning>
    <warning>DO NOT add an audit method to the AuditWriter trait — the existing `log` method takes any `AuditEntry`. Stage C adds new `AuditEntry::mcp_*` constructors; the trait surface stays unchanged.</warning>
    <warning>DO NOT manage MCP-server connections from the drone process. The drone is audit + projection + persistence; MCP connections live in the main process (runtime-mcp crate is a runtime-main dependency). Same architectural principle as ADR-0007.</warning>
    <warning>DO NOT introduce per-server reconnect backoff beyond rmcp's defaults — out-of-scope per MVP §M6.</warning>
  </execution_warnings>

  <time_box estimate_hours="8"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Health-ping cadence calibration: 30s default — note any signal that it's too aggressive (battery on laptops) or too lax (user-visible offline lag). SecretStore trait shape — note any awkwardness in delegating to M02 key_store; surface for Stage D consumption.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M06-mcp-basic.md" section="C.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state</item>
    <item>diff stat</item>
    <item>gate results (v1.6 canonical order; runtime-mcp ≥95% holds with client/* in gate; workspace ≥80%; CI-parity per G6)</item>
    <item>migration 002_mcp_servers.sql idempotency confirmation (run migration runner against existing DB; re-run; both succeed)</item>
    <item>audit emission test results — mcp_installed / mcp_uninstalled / mcp_auth_granted each tested with file inspection</item>
    <item>Tauri commands compile + invoke_handler wiring confirmed</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from C.6</item>
    <item>explicit statement: "Stage M06.C is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### C.6 Commit Message

```
feat(runtime): M06 Stage C — runtime-mcp::client lifecycle (install + auth + connection mgmt + audit)

Wraps the Stage B transport primitive with server lifecycle management:
Add/Remove/Test operations, per-server auth via M02 key_store, connection
management with 30s health-ping loop, SQLite-persisted mcp_servers
registry, and audit emissions (mcp_installed / mcp_uninstalled /
mcp_auth_granted) via the M05.E writer.

Crate additions (runtime-mcp):
- src/client/mod.rs: McpClient struct + public API (add_server,
  remove_server, test_connection, list_servers, get_connection).
- src/client/lifecycle.rs: connect/disconnect/health-ping loop;
  failure routes through mcp_missing event + on_gap HITL trigger
  (no new event/trigger).
- src/client/registry.rs: SQLite-backed mcp_servers registry; path-
  agnostic Registry::open(path: &Path); Tauri shell resolves the
  directory per CLAUDE.md §9 archetype.
- src/client/auth.rs: SecretStore trait + InMemorySecretStore fake
  for tests. runtime-main::key_store::KeyStore adds the impl.
- src/client/error.rs: LifecycleError; thiserror; From conversions.

Migration:
- crates/runtime-drone/migrations/002_mcp_servers.sql: extends the
  M01-scaffolded mcp_servers table with transport_type / url / args /
  env / cwd / auth_secret_ref / created_at / last_health_check
  columns. Idempotent (uses ALTER TABLE / CREATE INDEX IF NOT EXISTS).

Schema:
- schemas/event.v1.json: adds mcp_installed, mcp_uninstalled,
  mcp_auth_granted event variants. Cross-schema refs to
  mcp.v1.json#/$defs/McpServerName (Stage B introduced).
- generated/event.rs + src/types/agent_event.ts: regenerated.

Audit:
- crates/runtime-main/src/audit/entry.rs: adds mcp_installed,
  mcp_uninstalled, mcp_auth_granted entry constructors per the M05.E
  builder pattern. AuditWriter trait surface unchanged.

Tauri commands:
- src-tauri/src/commands.rs: mcp_add_server, mcp_remove_server,
  mcp_test_connection, mcp_list_servers. Renderer wires in Stage E.
- src-tauri/src/main.rs: McpClient constructed at app startup with
  AppHandle::path().app_local_data_dir().join("mcp_servers.sqlite")
  resolved by the Tauri shell.

Tests:
- crates/runtime-mcp/tests/client_lifecycle.rs: 10 integration tests
  covering add+audit, remove+audit, test_connection, get_connection
  caching, health-failure routing, multi-call invariant.
- crates/runtime-mcp/tests/registry.rs: 7 tests covering open,
  insert/get/list/remove, last-health update, migration idempotency.
- crates/runtime-mcp/tests/auth.rs: 4 tests covering trait round-trip
  + multi-call.

Coverage: runtime-mcp ≥95% per-crate line holds across transport/* +
client/* + error.rs. workspace ≥80% holds.

cargo deny check passes; no new license issues (rusqlite already in
workspace from M01).

Not in this stage: §5a tool namespace resolution (Stage D), MCP
dispatch through L1+L2a gates (Stage D), renderer UI (Stage E).

https://claude.ai/code/session_<id>
```

---

## Stage D — §5a Tool Namespace Resolution + MCP Dispatch through L1+L2a + audit

### D.1 Problem Statement

Implement spec §5a Tool Namespace Resolution + wire MCP tool dispatch through the L1+L2a capability gates (Stage A's wire-up). Add `tool_alias_ambiguous` warning event + `mcp_request_blocked` event variant; extend `schemas/framework.v1.json` with the optional `mcp_aliases` field per §5a; emit `mcp_request_blocked` audit entries via the M05.E writer when capability check denies an MCP tool call.

This stage is where MCP tool calls actually flow end-to-end: agent invokes a tool → SDK looks up the tool name (canonical `<server>__<tool>` OR short name OR alias) → enforcer.check (Stage A's wire) → if Ok, runtime-mcp dispatches to the appropriate `McpClient` connection → tool result streams back → emit `tool_invoked` + `tool_result_received` (existing M02+ variants).

Concrete deliverables:
1. **`crates/runtime-mcp/src/namespace/mod.rs`** — `NamespaceResolver` struct. `resolve(name: &str, aliases: &BTreeMap<String, String>) -> Result<ResolvedTool, NamespaceError>`. ResolvedTool is `{ server: McpServerName, tool: String }`. Resolution rules per §5a: canonical `<server>__<tool>` (split on first `__`); short name if unambiguous across all currently-connected servers; explicit `mcp_aliases` framework field override.
2. **`crates/runtime-mcp/src/namespace/aliases.rs`** — `Aliases` struct wrapping the `BTreeMap<String, String>` from framework JSON's `mcp_aliases` field. Validates alias values point at canonical names; collision detection.
3. **`crates/runtime-main/src/sdk/mcp_dispatch.rs`** — MCP tool dispatch routing. Reads the tool name from `ProviderEvent::ToolUse`, calls `NamespaceResolver::resolve`, calls `enforcer.check` (Stage A's primitive), on Ok dispatches to `McpClient::get_connection(server_name)?.invoke_tool(tool_name, args)`, on Err emits `capability_violation` + `mcp_request_blocked` + audit emission.
4. **Schema additions.**
   - `schemas/framework.v1.json`: optional `mcp_aliases` field on Framework root (`{ "type": "object", "additionalProperties": { "type": "string" } }`). Minor bump within v1.
   - `schemas/event.v1.json`: `tool_alias_ambiguous` variant (warning event; fires on re-resolution when connect/disconnect makes a short name newly ambiguous, per §5a step 5) + `mcp_request_blocked` variant (fires when capability check denies an MCP tool call; records server + tool + capability violated).
5. **Audit emission for `mcp_request_blocked`** via the M05.E writer; new `AuditEntry::mcp_request_blocked` constructor.
6. **Re-resolution loop.** When an MCP server connects or disconnects, `NamespaceResolver` re-evaluates short-name uniqueness across all currently-connected servers. Newly-ambiguous short names emit `tool_alias_ambiguous` warning events so frameworks can pin via `mcp_aliases`. This is integrated into `McpClient`'s `add_server` + `remove_server` post-success paths.
7. **graphStore.ts branches** for `mcp_installed`, `mcp_uninstalled`, `mcp_auth_granted`, `mcp_request_blocked`, `tool_alias_ambiguous` events. Renderer wires in Stage E; Stage D ships the store branches so they're testable independently.
8. **Per-module ≥95% coverage** on `runtime-mcp::namespace` (new) + `runtime-main::sdk::mcp_dispatch` (new). The runtime-mcp crate gate continues at ≥95%; the runtime-main crate gate continues at ≥95%.

Not in this stage:
- Renderer UI (Stage E — uses Stage D's store branches + Tauri commands)
- Multi-server collision resolution UI (deferred to v1.0)

### D.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-mcp/src/lib.rs` | exists | Edit: `pub mod namespace;` |
| `crates/runtime-mcp/src/namespace/mod.rs` | **new** | NamespaceResolver + ResolvedTool + NamespaceError |
| `crates/runtime-mcp/src/namespace/aliases.rs` | **new** | Aliases wrapper + validation |
| `crates/runtime-mcp/src/namespace/tests.rs` OR inline | **new** | Unit tests for resolution (canonical / short / alias / ambiguous) |
| `crates/runtime-main/src/sdk/mcp_dispatch.rs` | **new** | MCP tool dispatch routing through L1+L2a gates |
| `crates/runtime-main/src/sdk/mod.rs` | exists | Edit: `pub mod mcp_dispatch;` + wire dispatch into the SDK event pipeline |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | exists | Edit: when ProviderEvent::ToolUse names a tool resolvable via NamespaceResolver, route through mcp_dispatch instead of the existing default path |
| `crates/runtime-main/src/audit/entry.rs` | exists | Edit: add `AuditEntry::mcp_request_blocked` constructor |
| `crates/runtime-main/Cargo.toml` | exists | Edit: add `runtime-mcp = { path = "../runtime-mcp" }` if not added in Stage C; confirm transitive deps |
| `schemas/framework.v1.json` | exists | Edit: add optional `mcp_aliases: { type: "object", additionalProperties: { type: "string" } }` field |
| `schemas/event.v1.json` | exists | Edit: add `tool_alias_ambiguous` + `mcp_request_blocked` variants |
| `crates/runtime-core/src/generated/framework.rs` | exists | Regenerated |
| `crates/runtime-core/src/generated/event.rs` | exists | Regenerated |
| `src/types/agent_event.ts` | exists | Regenerated |
| `src/types/framework.ts` (if exists) OR equivalent | exists/check | Regenerated |
| `src/lib/graphStore.ts` | exists | Edit: applyEvent branches for `mcp_installed` / `mcp_uninstalled` / `mcp_auth_granted` / `mcp_request_blocked` / `tool_alias_ambiguous`; new `currentMcpServers` store slot (map of name → ServerStatus) |
| `tests/unit/graphStore.test.ts` | exists | Edit: tests for the 5 new applyEvent branches + idempotence |
| `crates/runtime-main/tests/mcp_dispatch_integration.rs` | **new** | End-to-end MCP dispatch through L1+L2a gates + audit |
| `crates/runtime-mcp/tests/namespace_resolution.rs` | **new** | Resolution edge cases (ambiguous + reconfigure post-connect/disconnect) |
| `CHANGELOG.md` | exists | Edit |
| `docs/build-prompts/retrospectives/M06.D-retrospective.md` | **new** | Stage D retrospective |

Effort budget: ~5–6 hours. Namespace algorithm is small + pure; dispatch wiring is mechanical against Stage A's primitives.

### D.3 Detailed Changes

#### D.3.1 Namespace algorithm — `namespace/mod.rs`

```rust
pub struct NamespaceResolver {
    /// (server_name, [tool_name1, tool_name2, ...])  -- snapshotted at the latest connect/disconnect
    connected_servers: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTool {
    pub server: String,
    pub tool: String,
}

#[derive(Debug, thiserror::Error)]
pub enum NamespaceError {
    #[error("tool '{0}' not found in any connected MCP server")]
    NotFound(String),
    #[error("tool '{name}' is ambiguous; candidates: {candidates:?}")]
    Ambiguous { name: String, candidates: Vec<String> },
    #[error("alias '{0}' points at unknown canonical '{1}'")]
    UnknownAlias(String, String),
}

impl NamespaceResolver {
    pub fn new(connected: BTreeMap<String, Vec<String>>) -> Self { /* ... */ }

    pub fn resolve(&self, name: &str, aliases: &BTreeMap<String, String>) -> Result<ResolvedTool, NamespaceError> {
        // 1. alias override
        if let Some(canonical) = aliases.get(name) {
            return self.resolve_canonical(canonical);
        }
        // 2. canonical name (contains "__" — split on first)
        if let Some((server, tool)) = name.split_once("__") {
            if self.connected_servers.get(server).map_or(false, |tools| tools.iter().any(|t| t == tool)) {
                return Ok(ResolvedTool { server: server.into(), tool: tool.into() });
            }
            return Err(NamespaceError::NotFound(name.into()));
        }
        // 3. short name; check unambiguity
        let mut matches: Vec<String> = self.connected_servers.iter()
            .filter(|(_, tools)| tools.iter().any(|t| t == name))
            .map(|(server, _)| format!("{server}__{name}"))
            .collect();
        match matches.len() {
            0 => Err(NamespaceError::NotFound(name.into())),
            1 => {
                let canonical = matches.remove(0);
                self.resolve_canonical(&canonical)
            }
            _ => Err(NamespaceError::Ambiguous { name: name.into(), candidates: matches }),
        }
    }

    pub fn re_evaluate_short_names(&self) -> Vec<NewAmbiguity> {
        // For each previously-unambiguous short name now ambiguous across the current connected set,
        // emit a NewAmbiguity record. Caller (McpClient) translates to tool_alias_ambiguous events.
    }
}
```

#### D.3.2 MCP dispatch — `sdk/mcp_dispatch.rs`

```rust
pub struct McpDispatcher {
    client: Arc<McpClient>,
    resolver: Arc<NamespaceResolver>,
    enforcer: Arc<CapabilityEnforcer>,
    audit: Option<Arc<dyn AuditWriter>>,
}

impl McpDispatcher {
    pub async fn dispatch_if_mcp(
        &self,
        agent_id: &str,
        tool_name: &str,
        args: serde_json::Value,
        aliases: &BTreeMap<String, String>,
    ) -> Option<Result<McpDispatchOutcome, McpError>> {
        // 1. Try namespace resolution; if NotFound, return None (not an MCP tool — let default dispatch handle)
        let resolved = match self.resolver.resolve(tool_name, aliases) {
            Ok(r) => r,
            Err(NamespaceError::NotFound(_)) => return None,
            Err(NamespaceError::Ambiguous { candidates, .. }) => {
                return Some(Err(McpError::Protocol(format!("ambiguous: {:?}", candidates))));
            }
            Err(NamespaceError::UnknownAlias(_, _)) => return Some(Err(McpError::Protocol("unknown alias".into()))),
        };

        // 2. L1 capability check (Stage A's wire)
        let needed = vec![CapabilityDeclaration::for_mcp_tool(&resolved.server, &resolved.tool)];
        if let Err(e) = self.enforcer.check(agent_id, &needed) {
            // Emit capability_violation (existing M05.B variant) AND mcp_request_blocked (new D variant)
            // Audit: mcp_request_blocked
            if let Some(audit) = &self.audit {
                audit.log(AuditEntry::mcp_request_blocked(
                    agent_id.into(),
                    resolved.server.clone(),
                    resolved.tool.clone(),
                    format!("{:?}", e),
                )).await?;
            }
            return Some(Err(McpError::Protocol(format!("capability denied: {:?}", e))));
        }

        // 3. Dispatch to MCP server via the runtime-mcp client
        let connection = self.client.get_connection(&resolved.server).await?;
        let result = connection.invoke_tool(&resolved.tool, args).await;
        Some(result.map(|value| McpDispatchOutcome { server: resolved.server, tool: resolved.tool, value }))
    }
}
```

The `dispatch_if_mcp` returns `Option<Result<...>>` so the caller (event pipeline) can fall through to the default tool-dispatch path when the tool isn't an MCP tool. This separates "MCP tool successfully resolved + dispatched" from "not an MCP tool, fall through" cleanly.

#### D.3.3 Event pipeline integration — `sdk/event_pipeline.rs`

After Stage A's L1 wire-up, Stage D extends the translation:

```rust
ProviderEvent::ToolUse(payload) => {
    // Try MCP dispatch first
    if let Some(result) = mcp_dispatcher.dispatch_if_mcp(&agent_id, &payload.tool_name, payload.input.clone(), &framework.mcp_aliases).await {
        match result {
            Ok(outcome) => {
                emit(AgentEvent::ToolInvoked { /* with server context */ }).await?;
                emit(AgentEvent::ToolResultReceived { result: outcome.value, /* ... */ }).await?;
                return Ok(());
            }
            Err(McpError::Protocol(msg)) if msg.starts_with("capability denied") => {
                emit(AgentEvent::CapabilityViolation { /* ... */ }).await?;
                emit(AgentEvent::McpRequestBlocked { /* ... */ }).await?;
                // Route through HitlSeam (existing on_capability_violation trigger)
                return Ok(());
            }
            Err(e) => {
                emit(AgentEvent::ToolResultReceived { error: Some(format!("{e}")), /* ... */ }).await?;
                return Ok(());
            }
        }
    }
    // Not an MCP tool — fall through to Stage A's existing L1 wire + default dispatch
    // ... existing Stage A code ...
}
```

#### D.3.4 Schema additions — `framework.v1.json` + `event.v1.json`

```jsonc
// framework.v1.json — add to root properties:
"mcp_aliases": {
  "type": "object",
  "additionalProperties": { "type": "string" },
  "description": "Optional short-name → canonical-name mapping per §5a"
}

// event.v1.json — new variants:
{
  "type": "object",
  "title": "ToolAliasAmbiguous",
  "required": ["type", "name", "candidates"],
  "properties": {
    "type": { "const": "tool_alias_ambiguous" },
    "name": { "type": "string", "minLength": 1 },
    "candidates": { "type": "array", "items": { "type": "string" }, "minItems": 2 }
  }
}
{
  "type": "object",
  "title": "McpRequestBlocked",
  "required": ["type", "agent_id", "server", "tool", "reason"],
  "properties": {
    "type": { "const": "mcp_request_blocked" },
    "agent_id": { "$ref": "common.v1.json#/$defs/AgentId" },
    "server": { "$ref": "mcp.v1.json#/$defs/McpServerName" },
    "tool": { "type": "string", "minLength": 1 },
    "reason": { "type": "string", "minLength": 1 }
  }
}
```

Cross-schema $refs to `mcp.v1.json#/$defs/McpServerName` (Stage B introduced) — the v1.6 `<schema_ref_audit>` slot verifies at authoring time.

### D.4 Tests

#### D.4.1 Namespace resolution tests (`namespace_resolution.rs`)

- `resolve_canonical_succeeds_when_server_and_tool_match`
- `resolve_canonical_fails_when_server_not_connected`
- `resolve_short_name_succeeds_when_unambiguous`
- `resolve_short_name_fails_when_ambiguous_returns_candidate_list`
- `resolve_short_name_fails_when_not_found`
- `resolve_alias_succeeds_when_alias_maps_to_valid_canonical`
- `resolve_alias_fails_when_alias_maps_to_unknown_canonical`
- `re_evaluate_short_names_emits_new_ambiguity_when_server_connects_with_overlapping_tool`
- `re_evaluate_short_names_returns_empty_when_no_new_ambiguity`
- `resolve_twice_in_sequence_both_succeed` (multi-call)

#### D.4.2 MCP dispatch integration tests (`mcp_dispatch_integration.rs`)

- `mcp_tool_dispatch_with_valid_grant_succeeds_and_emits_tool_invoked`
- `mcp_tool_dispatch_missing_grant_emits_capability_violation_and_mcp_request_blocked`
- `mcp_tool_dispatch_ambiguous_short_name_emits_tool_alias_ambiguous`
- `mcp_tool_dispatch_with_alias_succeeds`
- `non_mcp_tool_falls_through_to_default_dispatch_path` (negative — MCP dispatcher returns None)
- `mcp_tool_dispatch_audits_mcp_request_blocked_on_capability_deny`
- `mcp_tool_dispatch_twice_in_sequence_both_succeed` (multi-call)

#### D.4.3 graphStore branch tests

- `applyEvent_mcp_installed_adds_server_to_currentMcpServers`
- `applyEvent_mcp_uninstalled_removes_server_from_currentMcpServers`
- `applyEvent_mcp_auth_granted_updates_server_has_auth_flag`
- `applyEvent_mcp_request_blocked_appends_to_capabilityViolations_list_with_mcp_context`
- `applyEvent_tool_alias_ambiguous_emits_warning_toast` (or whatever the existing warning surface is)
- All applyEvent branches are idempotent under repeated identical events

#### D.4.4 Schema regen check

- `cargo xtask regenerate-types --check` after `framework.v1.json` + `event.v1.json` adds

#### D.4.5 Acceptance criteria

- [ ] `cargo test -p runtime-mcp --tests namespace_resolution` all pass
- [ ] `cargo test -p runtime-main --tests mcp_dispatch_integration` all pass
- [ ] `npm run test -- graphStore.test.ts` — 5 new tests pass
- [ ] `cargo llvm-cov` runtime-mcp + runtime-main per-crate gates hold (≥95%)
- [ ] `cargo xtask regenerate-types --check` passes
- [ ] CI-parity per G6

### D.5 CLI Prompt

```xml
<work_stage_prompt id="M06.D">
  <context>
    Stage D of M06 (MCP Basic). Implements §5a Tool Namespace Resolution
    (canonical `<server>__<tool>` + short-name unambiguous + framework
    `mcp_aliases` override) + wires MCP tool dispatch through Stage A's
    L1+L2a capability gates. Adds 2 new event variants (tool_alias_
    ambiguous + mcp_request_blocked) to event.v1.json + optional
    mcp_aliases field to framework.v1.json. Audit emissions for
    mcp_request_blocked via M05.E writer. graphStore branches for the 5
    new mcp_* event variants (3 from Stage C + 2 from this stage).
    Per-module ≥95% coverage on namespace + mcp_dispatch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M06-mcp-basic.md (Stage D sections D.1–D.4 + Stages A/B/C as predecessors)</file>
    <file>docs/build-prompts/retrospectives/M06.C-retrospective.md</file>
    <file>agent-runtime-spec.md §5a Tool Namespace Resolution (the canonical algorithm Stage D implements verbatim)</file>
    <file>docs/MVP-v0.1.md §M6</file>
    <file>docs/gotchas.md (#26 serde tag struct-variants, #43 typify root-oneOf, #66 contract tests, #68 wrong-field reads, #69 multi-call, #73 typify oneOf non-Copy)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="Stage A L1 wire-up — Stage D consumes via Arc&lt;CapabilityEnforcer&gt;">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="Stage A capabilities map getters — Stage D's mcp_dispatch lookups">crates/runtime-main/src/framework_loader/mod.rs</file>
    <file purpose="Stage B transport surface — Stage D dispatches via Connection::invoke_tool">crates/runtime-mcp/src/transport/mod.rs</file>
    <file purpose="Stage C client — Stage D's dispatcher gets connections via McpClient::get_connection">crates/runtime-mcp/src/client/mod.rs</file>
    <file purpose="M05.E AuditWriter — Stage D emits mcp_request_blocked entries">crates/runtime-main/src/audit/writer.rs</file>
    <file purpose="M05.E AuditEntry builder pattern — Stage D adds mcp_request_blocked constructor">crates/runtime-main/src/audit/entry.rs</file>
    <file purpose="graphStore — Stage D extends applyEvent with 5 new mcp_* branches + currentMcpServers slot">src/lib/graphStore.ts</file>
    <file purpose="capability declaration shape — used by mcp_dispatch's `needed` construction">crates/runtime-main/src/capability/declaration.rs</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M06.A" decisions_file="docs/build-prompts/retrospectives/M06.A-retrospective.md"/>
    <stage id="M06.B" decisions_file="docs/build-prompts/retrospectives/M06.B-retrospective.md"/>
    <stage id="M06.C" decisions_file="docs/build-prompts/retrospectives/M06.C-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M06-mcp-basic.md" section="D.3 Detailed Changes"/>

  <!-- Added post-execution (2026-05-15) per maintainer direction so
       M06.V's bias-guarded <read_first> treats these as KNOWN-DEFERRED
       carry-forwards, not red findings. Both flow from ADR-0010
       (dependency inversion) + strict-TDD "no untested production in
       the green commit". Resolution is Stage E, test-first. -->
  <scope_change>
    <descope deliverable="SDK run-loop MCP wiring — AgentSdk field + ProviderEvent::ToolUse interception calling dispatch_if_mcp (None→Stage A fall-through; Some(Ok)→apply_mcp_dispatch + outcome_needs_hitl HITL await; Some(Err)→mcp_dispatch_error_event)" reason="ADR-0010 composition-root: the concrete Arc<dyn McpToolDispatch> is constructed + injected by src-tauri (same shell-injected-seam archetype as Arc<dyn Connection> / Arc<AuditWriter>); no red test covers the run-loop interception, and strict-TDD <tdd_discipline> forbids untested production in the green commit. Stage D ships the seam + outcome→event mapping + concrete dispatcher (all tested)." carry_forward_to="M06 Stage F (run-loop interception SEAM + src-tauri *_with injection SEAM, test-first — maintainer-inserted wire stage). FORWARD-CORRECTED BY ADR-0011 (surfaced pre-F-red): trace #11 is SPLIT — 11a (the SDK run-loop interception seam + the src-tauri *_with injection seam, mock-verified) is closed in-milestone, expected DELIVERED, 🔴 if THAT regressed/missing; 11b (concrete McpDispatcher in main.rs + impl ConnectionResolver for McpClient + a live agent-loop exercise) is the explicit ADR-0011 M07 carry-forward, expected ABSENT at M06.V, NOT a 🔴. V reads ADR-0011 and applies the split." authorized_by="ADR-0010 + M06.D <tdd_discipline> strict green-phase constraint; Stage F insertion authorized by maintainer post-M06.E scope call; F-scope split authorized by ADR-0011 (maintainer-selected pre-F-red)"/>
    <descope deliverable="agent_id-correct ToolInvoked/ToolResult for the apply_mcp_dispatch Invoked success path" reason="McpDispatchOutcome::Invoked carries {server,tool,value} only (the integration test pins that pattern; adding agent_id would break the frozen red test). apply_mcp_dispatch's Invoked branch therefore emits empty agent_id; the SDK run loop (which holds agent_id) must emit the agent_id-correct success events directly when the run-loop wiring lands. gotcha #68 class — must NOT ship empty agent_id to the renderer." carry_forward_to="M06 Stage F (test-first fix: run-loop emits Invoked events directly with the agent_id it holds, leaving apply_mcp_dispatch + the D-frozen McpDispatchOutcome test untouched — maintainer-inserted wire stage; closed in-milestone). FORWARD-CORRECTED BY ADR-0011: this is part of trace #11's 11a (mock-verified) — expected DELIVERED, 🔴 if F ships empty/wrong agent_id on the run-loop Invoked path; the live agent-loop exercise of this path is 11b (ADR-0011 M07 carry-forward, NOT a 🔴)." authorized_by="ADR-0010 note + M06.D-retrospective.md [END] special-log + Decisions; Stage F insertion authorized by maintainer post-M06.E scope call; F-scope split authorized by ADR-0011 (maintainer-selected pre-F-red)"/>
  </scope_change>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the test plan's buckets. Stub the
      production surfaces just enough to make the test files compile
      (todo!() / unimplemented!() bodies are fine; the goal is link-time
      test discovery, not behavior). Confirm tests fail with right-reason
      errors per CLAUDE.md §5 (assertion failed / cannot find function /
      unresolved import / not-yet-implemented panic — NOT a test-file
      compile error and NOT a tautological pass). Commit as a STANDALONE
      `test(M06.&lt;stage&gt;): failing tests for ...` commit on
      claude/m06-mcp-basic BEFORE green-phase impl; the commit body
      pastes the first ~40 lines of cargo test output proving the
      expected-failure class. Surface the red-phase commit to the user;
      user approves before green phase begins.
    </red_phase>
    <green_phase>
      Implement until ALL failing tests pass. Do NOT modify the test
      files during implementation — if a test is wrong, fix it in a
      SEPARATE labelled follow-up commit with explanation, never silently
      in the impl commit. The impl commit body MUST state the verifiable
      audit-surface invariant: `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      rustfmt/clippy fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message (M06.B Decision;
      gotcha-candidate territory on third recurrence).
    </green_phase>
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

  <acceptance_criteria ref="docs/build-prompts/M06-mcp-basic.md" section="D.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M06-mcp-basic.md" section="Key constraints"/>

  <gates milestone="M06"/>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch_state" gate="claude/m06-mcp-basic"/>
    <check name="prior_stages" gate="git log --oneline main..HEAD | head -4 must include M06.A + M06.B + M06.C"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="crates/runtime-mcp/src/client/mod.rs" verified="true" note="Stage C created"/>
    <claim type="file" path="crates/runtime-mcp/src/transport/mod.rs" verified="true" note="Stage B created"/>
    <claim type="file" path="crates/runtime-main/src/sdk/event_pipeline.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/capability/enforcer.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/framework_loader/mod.rs" verified="true"/>
    <claim type="file" path="src/lib/graphStore.ts" verified="true"/>
    <claim type="file" path="schemas/framework.v1.json" verified="true"/>
    <claim type="file" path="schemas/event.v1.json" verified="true"/>
    <claim type="file" path="schemas/mcp.v1.json" verified="true" note="Stage B created"/>
    <claim type="method" path="crates/runtime-mcp/src/client/mod.rs" symbol="get_connection" verified="true" note="Stage C exposed"/>
    <claim type="method" path="crates/runtime-main/src/capability/enforcer.rs" symbol="check" verified="true" note="M05.B + Stage A wired"/>
    <claim type="method" path="crates/runtime-main/src/audit/entry.rs" symbol="mcp_request_blocked" verified="false" note="Stage D adds"/>
    <claim type="struct_field" path="schemas/framework.v1.json" symbol="root.mcp_aliases" verified="false" note="Stage D adds"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M06.C-retrospective.md" verified="true"/>
  </phase_doc_inventory_audit>

  <schema_drift_check gate="cargo xtask regenerate-types --check after framework.v1.json + event.v1.json adds"/>

  <schema_ref_audit>
    <ref schema="schemas/event.v1.json" path="common.v1.json#/$defs/AgentId" verified="true" note="existing cross-schema ref"/>
    <ref schema="schemas/event.v1.json" path="mcp.v1.json#/$defs/McpServerName" verified="true" note="Stage B introduced; Stage C consumed"/>
  </schema_ref_audit>

  <schema_audit>
    <survey pattern='"mcp_aliases"' purpose="confirm no existing mcp_aliases field in framework.v1.json before adding"/>
    <survey pattern='"tool_alias_ambiguous"' purpose="confirm no existing variant"/>
    <survey pattern='"mcp_request_blocked"' purpose="confirm no existing variant"/>
  </schema_audit>

  <schema_root_check/>

  <architecture_check>
    <claim description="MCP dispatch routes through enforcer.check BEFORE Connection::invoke_tool — same ordering principle as Stage A's L1 wire" verify="grep -B5 'invoke_tool' crates/runtime-main/src/sdk/mcp_dispatch.rs ; expect enforcer.check call precedes"/>
    <claim description="dispatch_if_mcp returns None when tool name isn't an MCP tool (caller falls through) — separates 'is MCP' from 'is MCP and succeeded'" verify="grep 'dispatch_if_mcp' crates/runtime-main/src/sdk/mcp_dispatch.rs ; expect Option&lt;Result&gt; return type"/>
    <claim description="Re-evaluation of short-name uniqueness fires on connect/disconnect — McpClient.add_server + remove_server call re_evaluate_short_names post-success" verify="grep 're_evaluate_short_names' crates/runtime-mcp/src/client/ -r ; expect calls in lifecycle.rs"/>
    <claim description="capability_violation event reused for MCP deny (existing M05.B variant); mcp_request_blocked is a SEPARATE event recording the MCP context (server + tool); both emit on a single deny" verify="grep -B5 'McpRequestBlocked' crates/runtime-main/src/sdk/mcp_dispatch.rs ; expect both events emitted in sequence"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="ProviderEvent::ToolUse" purpose="confirm Stage D extends the existing translation site (Stage A's wire-up); single site"/>
    <grep pattern="mcp_aliases" purpose="enumerate consumers; expect Framework struct (regenerated), NamespaceResolver.resolve, mcp_dispatch"/>
  </fan_out_grep>

  <dependency_audit_check>
    <dep name="runtime-mcp" version="path" prefer_crates_io_name="N/A" source_authority="workspace member" required_features="N/A" audit="runtime-main adds intra-workspace dep on runtime-mcp if Stage C didn't already; confirm path-dep declared exactly once"/>
  </dependency_audit_check>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-mcp" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.lib\.rs"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.key_store\.rs"/>
  </coverage_gate>

  <existing_pattern_audit>
    <pattern grep_for="case 'tool_invoked'" rationale="Stage D extends event_pipeline.rs's ProviderEvent::ToolUse handling; existing match arms / case statements in graphStore for tool_invoked must continue to work" affected_files="src/lib/graphStore.ts" remediation="extend rather than replace the existing tool_invoked case"/>
    <pattern grep_for="AgentEvent::" rationale="adding new event variants must preserve existing irrefutable bindings; default-case handlers in graphStore must still default-no-op the unknown variants" affected_files="0 expected" remediation="N/A — Stage A established narrowed_from optional field pattern"/>
  </existing_pattern_audit>

  <interpretation_declarations>
    <adopt spec_section="§5a step 4 — server name constraints" interpretation="server names cannot contain `__`; validated at McpServerName regex `^[a-z0-9][a-z0-9-]*$` (no underscores at all)" alternative_interpretation="allow single underscores; ban only `__`" rationale="kebab-case naming is industry-standard for service names; banning underscores entirely is simpler and more conservative; aligns with MCP spec's canonical naming"/>
    <adopt spec_section="§5a step 5 — re-resolution on connect/disconnect" interpretation="re-evaluation fires on both add_server (post-test-connection success) and remove_server (post-disconnect); not at every health-ping" alternative_interpretation="re-evaluate on every health-ping cycle (every 30s)" rationale="connect/disconnect are the only events that change the connected-server set; health-pings don't change uniqueness; re-evaluation cost is negligible but unnecessary on health-pings"/>
  </interpretation_declarations>

  <api_breaking_change_audit>
    <change api="schemas/framework.v1.json Framework" before_signature="without mcp_aliases" after_signature="with optional mcp_aliases: BTreeMap&lt;String, String&gt;" call_sites="3 (framework_loader + Builder Canvas (M08) + Tester)" test_sites="6 (framework_loader_smoke + variants)" recommendation="optional field — existing frameworks continue to work; new field surfaces alias override"/>
    <change api="schemas/event.v1.json" before_signature="without tool_alias_ambiguous + mcp_request_blocked" after_signature="with two new variants" call_sites="0 — Stage D emits" test_sites="6 — graphStore branch tests" recommendation="purely additive event variants"/>
  </api_breaking_change_audit>

  <gotchas>
    <trap>serde tag struct-variants (gotcha #26) — both new event variants use struct-shape fields per the established pattern.</trap>
    <trap>typify oneOf non-Copy (gotcha #73) — the new event variants nest String + Vec&lt;String&gt;; if typify-generated PartialEq fails on the discriminated union, compare via serde round-trip.</trap>
    <trap>Tests-pass-but-contract-fails (gotcha #66) — dispatch tests must assert BOTH the emitted event sequence AND the audit log entry — file-inspection check, not just method-call assertion.</trap>
    <trap>Wrong-field reads (gotcha #68) — mcp_request_blocked has agent_id + server + tool + reason; graphStore branches must read every field.</trap>
    <trap>Multi-call invariants (gotcha #69) — dispatch + namespace resolve + re-evaluation all need *_twice_in_sequence tests.</trap>
    <trap>Schema regeneration discipline — both framework.v1.json and event.v1.json edits land in the SAME commit as the regenerated Rust + TS files. CI's regenerate-types --check enforces.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT introduce a new HITL trigger for capability-deny on MCP tools — reuse `on_capability_violation` (existing M04.E + M05.B). The user-visible behavior is the existing capability-violation modal per ADR-0007.</warning>
    <warning>DO NOT replace Stage A's L1 wire-up in event_pipeline.rs — Stage D EXTENDS by routing MCP tool dispatch BEFORE the default Stage A path. The Stage A wire continues to handle non-MCP tool dispatch.</warning>
    <warning>The capability check for MCP tools uses `CapabilityDeclaration::for_mcp_tool(server, tool)` — a new helper that constructs the proper Capability shape. If the M05.B-shipped CapabilityDeclaration doesn't have a Mcp variant, Stage D adds it via a minor capability.v1.json bump. Verify at authoring time.</warning>
    <warning>graphStore's currentMcpServers slot follows the M05.D currentTier pattern — persistent across `clear()` calls, requires explicit `beforeEach` reset in test files (per v1.6 &lt;test_isolation_audit&gt; slot — apply at Stage E since renderer tests start there).</warning>
  </execution_warnings>

  <time_box estimate_hours="6"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>§5a algorithm correctness — note any edge case from the spec that wasn't obvious until implementation (especially re-evaluation timing on rapid connect/disconnect/connect cycles). Capability declaration shape for MCP tools — surface for Stage E renderer wiring + M06.V Wire pass trace.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M06-mcp-basic.md" section="D.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state</item>
    <item>diff stat</item>
    <item>gate results (v1.6 canonical order; both runtime-mcp + runtime-main ≥95% gates; CI-parity)</item>
    <item>§5a algorithm conformance: each of the 5 spec rules (canonical, short, alias, re-resolution, server-name constraint) cited as a test by name</item>
    <item>MCP dispatch end-to-end test results — wire trace from ProviderEvent::ToolUse through resolve + check + dispatch</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from D.6</item>
    <item>explicit statement: "Stage M06.D is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### D.6 Commit Message

```
feat(runtime): M06 Stage D — §5a Tool Namespace Resolution + MCP dispatch through L1+L2a + audit

Implements spec §5a Tool Namespace Resolution (canonical
`<server>__<tool>` + short-name unambiguous + framework `mcp_aliases`
override + re-resolution on connect/disconnect) + wires MCP tool
dispatch through Stage A's L1+L2a capability gates. Adds 2 new event
variants (tool_alias_ambiguous + mcp_request_blocked) + optional
mcp_aliases field on Framework root. Audit emissions for
mcp_request_blocked via the M05.E writer. graphStore branches for the
5 new mcp_* event variants (3 from Stage C + 2 from this stage) +
currentMcpServers store slot.

runtime-mcp::namespace (new module):
- src/namespace/mod.rs: NamespaceResolver + ResolvedTool + NamespaceError.
  resolve(name, aliases) implements §5a step 1 (alias override) + step 2
  (canonical split on first `__`) + step 3 (short-name unambiguity check).
- src/namespace/aliases.rs: Aliases wrapper + validation.
- re_evaluate_short_names: fires on connect/disconnect; emits
  NewAmbiguity records translated to tool_alias_ambiguous events.

runtime-main::sdk::mcp_dispatch (new module):
- src/sdk/mcp_dispatch.rs: McpDispatcher routes ProviderEvent::ToolUse
  through NamespaceResolver → enforcer.check → Connection::invoke_tool.
  On capability deny: emits capability_violation + mcp_request_blocked
  + audits mcp_request_blocked + routes through existing
  on_capability_violation HITL trigger (no new trigger).

Schema:
- schemas/framework.v1.json: optional mcp_aliases field (Map<String,String>).
- schemas/event.v1.json: tool_alias_ambiguous + mcp_request_blocked
  variants. Cross-schema refs to mcp.v1.json#/$defs/McpServerName.
- generated/event.rs + generated/framework.rs + src/types/agent_event.ts
  + src/types/framework.ts: regenerated.

Audit:
- crates/runtime-main/src/audit/entry.rs: mcp_request_blocked constructor.

Renderer (graphStore branches; Stage E wires components):
- src/lib/graphStore.ts: applyEvent branches for mcp_installed,
  mcp_uninstalled, mcp_auth_granted, mcp_request_blocked,
  tool_alias_ambiguous; new currentMcpServers store slot
  (Map<name, ServerStatus>); persistent across clear() (test files
  need beforeEach reset per v1.6 <test_isolation_audit>).

Tests:
- crates/runtime-mcp/tests/namespace_resolution.rs: 10 tests covering
  canonical / short / alias / ambiguous / re-evaluation / multi-call.
- crates/runtime-main/tests/mcp_dispatch_integration.rs: 7 end-to-end
  tests covering valid-grant + missing-grant + ambiguous + alias +
  fall-through + audit + multi-call.
- tests/unit/graphStore.test.ts: 5 new applyEvent branch tests +
  idempotence tests.

Coverage: runtime-mcp + runtime-main per-crate ≥95% gates hold.
namespace + mcp_dispatch per-module ≥95% each.

cargo xtask regenerate-types --check passes; cargo deny check clean.

Not in this stage: renderer UI (Stage E uses store branches + Tauri
commands), multi-server collision resolution UI (deferred to v1.0).

https://claude.ai/code/session_<id>
```

---

## Stage E — Renderer: `MCPNode` live wiring + Settings → MCP Servers Add/Remove/Test UI

### E.1 Problem Statement

Light up the renderer surface for MCP. M03 shipped `MCPNode` as a stub renderer (the 11th node type per spec §3); Stage E extends it with live connection status + tool list + active-call animation. Settings panel gains a MCP Servers section with Add / Remove / Test affordances driving the Stage C Tauri commands. graphStore branches added in Stage D now have visible consumers.

Renderer-only stage. No Rust changes; no schema changes. Vitest + Playwright behavior coverage.

Concrete deliverables:
1. **`src/components/nodes/MCPNode.tsx` extension** — read `currentMcpServers` slot for live status; render connection indicator (connected / disconnected / health_pending / error); tool list panel on click; active-call animation when a `tool_invoked` event lands with a server reference (Stage D's MCP dispatch added server context).
2. **`src/components/MCPServerSettings.tsx`** — new component. List of installed servers; per-server status indicator; Add button → modal with form for name + transport (stdio / http) + command-or-url + args + env + auth_secret; Test button per row → calls `mcp_test_connection` Tauri command → displays tool list; Remove button → confirm modal → calls `mcp_remove_server`.
3. **`src/lib/ipc.ts` extensions** — typed wrappers around the Stage C Tauri commands (`mcp_add_server`, `mcp_remove_server`, `mcp_test_connection`, `mcp_list_servers`).
4. **Tier-gate display.** Per spec §8.security L4, MCP server install is tier-gated — `shell: true`-equivalent for stdio transports gets the Promoted→Novice fallback per the existing tier matrix. The Settings panel's Add modal surfaces the tier-eval outcome before confirming install; uses the existing M05.F `CapabilityBadge` pattern for display.
5. **Playwright behavior test for the Add flow.** `tests/e2e/mcp_server_add.spec.ts` exercises the renderer Add path against a state-injection affordance (per gotcha #54 — `window.__graphStore`).
6. **Vitest unit tests** for `MCPNode` + `MCPServerSettings`. Coverage ≥80% on renderer code.

Not in this stage:
- Multi-server collision resolution UI (deferred to v1.0)
- MCP server discovery / browsing (deferred to v1.0)
- In-graph drag-drop wiring of MCP tools to agents (deferred to M08 Builder Canvas)

### E.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/components/nodes/MCPNode.tsx` | exists | Edit: extend with currentMcpServers reads + status indicator + tool list panel + active-call animation |
| `src/components/MCPServerSettings.tsx` | **new** | Settings panel section + Add modal + Test button per row + Remove confirmation |
| `src/components/MCPServerAddModal.tsx` | **new** | Form for name + transport + command-or-url + args + env + auth_secret; submits to mcp_add_server Tauri command |
| `src/lib/ipc.ts` | exists | Edit: typed wrappers for mcp_add_server, mcp_remove_server, mcp_test_connection, mcp_list_servers |
| `src/App.tsx` | exists | Edit: wire MCPServerSettings into Settings panel routing |
| `src/styles.css` | exists | Edit: classes for `.mcp-server-row`, `.mcp-server-row--connected`, `.mcp-server-row--error`, `.mcp-server-add-modal`, `.mcp-node--connected`, `.mcp-node--health-pending`, `.mcp-node--error`, `.mcp-tool-list` |
| `tests/unit/nodes/MCPNode.test.tsx` | exists/new | Edit/create: tests for status indicator, tool list, active-call animation |
| `tests/unit/components/MCPServerSettings.test.tsx` | **new** | Tests for the Settings panel — list rendering, status indicators, button-action wiring |
| `tests/unit/components/MCPServerAddModal.test.tsx` | **new** | Tests for the Add modal — field validation, submit-on-valid, error display |
| `tests/e2e/mcp_server_add.spec.ts` | **new** | Playwright behavior test for the renderer Add flow via window.__graphStore |
| `CHANGELOG.md` | exists | Edit |
| `docs/build-prompts/retrospectives/M06.E-retrospective.md` | **new** | Stage E retrospective |

Effort budget: ~4–5 hours. Renderer-only; pattern locks from M04.E (HITL modals/panels) + M05.F (CapabilityBadge + GapPanel) carry forward cleanly.

### E.3 Detailed Changes

#### E.3.1 MCPNode extension

```tsx
// src/components/nodes/MCPNode.tsx
import { useGraphStore } from '../../lib/graphStore';
import { useShallow } from 'zustand/react/shallow';
import { type NodeProps } from '@xyflow/react';

interface MCPNodeData {
  serverName: string;
}

export function MCPNode({ data }: NodeProps<MCPNodeData>) {
  // Pair with v1.6 <zustand_selector_audit>: useShallow for derived selectors
  const server = useGraphStore(useShallow((s) => s.currentMcpServers.get(data.serverName)));
  const status = server?.status ?? 'disconnected';
  const tools = server?.tools ?? [];
  const activeCallId = useGraphStore((s) => s.activeMcpCalls.get(data.serverName));

  return (
    <div className={`mcp-node mcp-node--${status} ${activeCallId ? 'mcp-node--active' : ''}`}>
      <header>{data.serverName}</header>
      <div className="mcp-node__status-indicator" aria-label={`status: ${status}`} />
      <ul className="mcp-tool-list">
        {tools.map((t) => <li key={t.name}>{t.name}</li>)}
      </ul>
    </div>
  );
}
```

Per gotcha #75 + v1.6 `<zustand_selector_audit>`: derived selectors use `useShallow`.

#### E.3.2 MCPServerSettings layout

Three-region layout matching the existing Settings panel pattern (M04.E):

```tsx
export function MCPServerSettings() {
  const servers = useMcpServers();
  const [showAdd, setShowAdd] = useState(false);
  return (
    <section className="settings-section settings-section--mcp">
      <header>
        <h2>MCP Servers</h2>
        <button onClick={() => setShowAdd(true)}>Add Server</button>
      </header>
      <ul className="mcp-server-list">
        {servers.map((s) => (
          <li key={s.name} className={`mcp-server-row mcp-server-row--${s.status}`}>
            <span className="mcp-server-row__name">{s.name}</span>
            <span className="mcp-server-row__transport">{s.transport.type}</span>
            <span className="mcp-server-row__status">{s.status}</span>
            <button onClick={() => handleTest(s.name)}>Test</button>
            <button onClick={() => handleRemove(s.name)}>Remove</button>
          </li>
        ))}
      </ul>
      {showAdd && <MCPServerAddModal onClose={() => setShowAdd(false)} />}
    </section>
  );
}
```

#### E.3.3 MCPServerAddModal — form

Fields: `name` (validated against `^[a-z0-9][a-z0-9-]*$`), `transport.type` (stdio | http), `transport.command` or `transport.url`, optional `transport.args` (CSV → `string[]`), optional `transport.env` (key=value lines → `Record<string, string>`), optional `transport.cwd`, optional `auth_secret` (textarea; opt-out for no-auth servers).

Per the in-process seam (ADR-0007), the Add modal stays renderer-side; the Tauri command does the heavy lift. No new HITL seam variant.

#### E.3.4 ipc.ts wrappers

```ts
// src/lib/ipc.ts (additions)
import { invoke } from '@tauri-apps/api/core';

export async function mcpAddServer(config: McpServerConfig, auth: string | null): Promise<void> {
  return invoke('mcp_add_server', { config, auth });
}

export async function mcpRemoveServer(name: string): Promise<void> {
  return invoke('mcp_remove_server', { name });
}

export async function mcpTestConnection(name: string): Promise<McpTool[]> {
  return invoke('mcp_test_connection', { name });
}

export async function mcpListServers(): Promise<McpServerSummary[]> {
  return invoke('mcp_list_servers');
}
```

Per gotcha #30, errors from Tauri are unwrapped via the existing `unwrapCmdError` helper.

#### E.3.5 Tier-gate display in Add modal

The Add modal calls a tier-evaluation Tauri command (existing from M05.D — or surface a new one if needed; verify at authoring time) to display "this install will require Novice review because the stdio transport runs an arbitrary command" or "this install will auto-accept because you're Promoted and the server has no shell capabilities" BEFORE the user clicks Confirm. Same pattern as M05.F CapabilityBadge.

#### E.3.6 Styles

`.mcp-server-row--connected` (green dot), `.mcp-server-row--health_pending` (amber), `.mcp-server-row--disconnected` (gray), `.mcp-server-row--error` (red). Same color tokens as M05.F `.capability-badge--<tier>` for visual consistency.

Per gotcha #67 (component rendered in DOM ≠ CSS exists), every new className gets a corresponding CSS rule; styles tests assert via the existing `every_class_has_a_corresponding_CSS_rule` pattern from M04.F.

### E.4 Tests

#### E.4.1 MCPNode tests

- `renders_server_name`
- `renders_status_indicator_per_status_value` (4 statuses × 4 class assertions)
- `renders_tool_list_when_server_is_connected`
- `renders_empty_tool_list_when_server_is_disconnected`
- `renders_active_call_class_when_activeMcpCalls_has_serverName`
- `uses_useShallow_for_derived_selector_no_infinite_loop` (per gotcha #75)

#### E.4.2 MCPServerSettings tests

- `renders_empty_state_when_no_servers_installed`
- `renders_row_per_installed_server`
- `clicking_add_opens_modal`
- `clicking_test_calls_mcpTestConnection_and_displays_tool_list`
- `clicking_remove_shows_confirmation_then_calls_mcpRemoveServer`
- `status_indicator_classes_match_server_status`
- `survives_repeated_renders_with_currentMcpServers_mutation` (per gotcha #66 — contract test)

#### E.4.3 MCPServerAddModal tests

- `validates_name_against_regex_disabling_submit_on_invalid`
- `submit_with_stdio_transport_calls_mcpAddServer_with_correct_config`
- `submit_with_http_transport_calls_mcpAddServer_with_url`
- `parses_args_csv_into_array`
- `parses_env_lines_into_record`
- `displays_tier_eval_outcome_before_confirming` (per spec §8.security L4)

#### E.4.4 Playwright behavior test — `tests/e2e/mcp_server_add.spec.ts`

```ts
test.describe.configure({ timeout: 90_000 }); // per gotcha #53 (Vite cold-start)

test('adding MCP server via Settings updates graphStore and surfaces row', async ({ page }) => {
  // Per v1.6 <playwright_warmup_recipe> + gotcha #53:
  // (warmup probe handled by webServer config; explicit curl probe also runs in CI before the test)

  await page.goto('/');
  await page.evaluate(() => window.__graphStore.getState().applyEvent({
    type: 'mcp_installed',
    name: 'filesystem',
    transport_type: 'stdio',
    has_auth: false,
  }));

  // Open Settings panel
  await page.click('[data-test=open-settings]');
  await page.click('[data-test=settings-mcp-tab]');

  // Confirm row rendered
  await expect(page.locator('.mcp-server-row--connected')).toContainText('filesystem');
});
```

#### E.4.5 Schema regen check

- N/A — Stage E ships no schema changes.

#### E.4.6 Acceptance criteria

- [ ] `npm run test` — all vitest tests pass (≥80% coverage on `src/components/MCPServerSettings*` + `src/components/nodes/MCPNode.tsx`)
- [ ] `npx tsc --noEmit` — clean
- [ ] `npx eslint .` — clean
- [ ] `npx prettier --check` — clean
- [ ] `npm run test:e2e -- mcp_server_add.spec.ts` — passes against Vite dev server with the curl warmup probe (per v1.6 `<playwright_warmup_recipe>`)
- [ ] Every new CSS class has a corresponding rule in `src/styles.css` (per gotcha #67 + the `every_class_has_a_corresponding_CSS_rule` pattern)
- [ ] `tests/unit/nodes/MCPNode.test.tsx` — explicit `beforeEach` reset of `currentMcpServers` slot (per v1.6 `<test_isolation_audit>`)
- [ ] CI-parity per G6

### E.5 CLI Prompt

```xml
<work_stage_prompt id="M06.E">
  <context>
    Stage E of M06 (MCP Basic). Lights up the renderer surface for MCP.
    Extends MCPNode (M03 stub) with live connection status + tool list
    + active-call animation. Adds Settings → MCP Servers section with
    Add / Remove / Test affordances driving the Stage C Tauri commands.
    Pairs tier-gate display per spec §8.security L4 with the existing
    M05.F CapabilityBadge pattern. Playwright behavior test for the
    Add flow via window.__graphStore state injection (per gotcha #54).
    Renderer ≥80% (vitest); pattern locks from M04.E + M05.F.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M06-mcp-basic.md (Stage E sections E.1–E.4 + Stages A–D as predecessors; Stage D's graphStore branches are this stage's data source)</file>
    <file>docs/build-prompts/retrospectives/M06.D-retrospective.md</file>
    <file>agent-runtime-spec.md §3 Visual Design (node types — MCPNode); §5 MCP Manager (UI affordances); §8.security L4 (tier-gate display)</file>
    <file>docs/MVP-v0.1.md §M6</file>
    <file>docs/adr/0007-in-process-hitl-seam-architecture.md (MCP user prompts reuse HitlSeam; Add modal stays renderer-side)</file>
    <file>docs/gotchas.md (especially #27 vitest re-query after await, #30 unwrapCmdError, #54 window.__graphStore, #67 component+CSS contract, #66 contract tests, #75 useShallow, #53 Vite cold-start)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="M03-shipped stub — Stage E extends">src/components/nodes/MCPNode.tsx</file>
    <file purpose="M05.F CapabilityBadge — pattern for tier-eval display">src/components/nodes/CapabilityBadge.tsx</file>
    <file purpose="M04.E Settings panel pattern — Stage E mirrors layout">src/components/HITLPanel.tsx</file>
    <file purpose="M04.E modal pattern — Stage E's Add modal mirrors">src/components/HITLModal.tsx</file>
    <file purpose="ipc.ts existing pattern + unwrapCmdError helper">src/lib/ipc.ts</file>
    <file purpose="graphStore currentMcpServers slot + applyEvent branches (Stage D added)">src/lib/graphStore.ts</file>
    <file purpose="styles.css existing class conventions">src/styles.css</file>
    <file purpose="every_class_has_a_corresponding_CSS_rule pattern from M04.F">tests/unit/components/BudgetHeaderBar.test.tsx</file>
    <file purpose="Playwright spec pattern + window.__graphStore injection from M04.C + M05.F">tests/e2e/gap_panel.spec.ts</file>
    <file purpose="Tauri command wiring — Stage C added the mcp_* commands">src-tauri/src/commands.rs</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M06.A" decisions_file="docs/build-prompts/retrospectives/M06.A-retrospective.md"/>
    <stage id="M06.B" decisions_file="docs/build-prompts/retrospectives/M06.B-retrospective.md"/>
    <stage id="M06.C" decisions_file="docs/build-prompts/retrospectives/M06.C-retrospective.md"/>
    <stage id="M06.D" decisions_file="docs/build-prompts/retrospectives/M06.D-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M06-mcp-basic.md" section="E.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the test plan's buckets. Stub the
      production surfaces just enough to make the test files compile
      (todo!() / unimplemented!() bodies are fine; the goal is link-time
      test discovery, not behavior). Confirm tests fail with right-reason
      errors per CLAUDE.md §5 (assertion failed / cannot find function /
      unresolved import / not-yet-implemented panic — NOT a test-file
      compile error and NOT a tautological pass). Commit as a STANDALONE
      `test(M06.&lt;stage&gt;): failing tests for ...` commit on
      claude/m06-mcp-basic BEFORE green-phase impl; the commit body
      pastes the first ~40 lines of cargo test output proving the
      expected-failure class. Surface the red-phase commit to the user;
      user approves before green phase begins.
    </red_phase>
    <green_phase>
      Implement until ALL failing tests pass. Do NOT modify the test
      files during implementation — if a test is wrong, fix it in a
      SEPARATE labelled follow-up commit with explanation, never silently
      in the impl commit. The impl commit body MUST state the verifiable
      audit-surface invariant: `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      rustfmt/clippy fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message (M06.B Decision;
      gotcha-candidate territory on third recurrence).
    </green_phase>
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

  <acceptance_criteria ref="docs/build-prompts/M06-mcp-basic.md" section="E.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M06-mcp-basic.md" section="Key constraints"/>

  <gates milestone="M06"/>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch_state" gate="claude/m06-mcp-basic"/>
    <check name="prior_stages" gate="git log --oneline main..HEAD | head -5 must include M06.A through M06.D"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="src/components/nodes/MCPNode.tsx" verified="true" note="M03-shipped stub"/>
    <claim type="file" path="src/components/MCPServerSettings.tsx" verified="false" note="Stage E creates"/>
    <claim type="file" path="src/components/MCPServerAddModal.tsx" verified="false" note="Stage E creates"/>
    <claim type="file" path="src/lib/ipc.ts" verified="true"/>
    <claim type="file" path="src/App.tsx" verified="true"/>
    <claim type="file" path="src/lib/graphStore.ts" verified="true" note="Stage D extended"/>
    <claim type="file" path="src-tauri/src/commands.rs" verified="true" note="Stage C added mcp_* commands"/>
    <claim type="struct_field" path="src/lib/graphStore.ts" symbol="currentMcpServers" verified="true" note="Stage D added"/>
    <claim type="struct_field" path="src/lib/graphStore.ts" symbol="activeMcpCalls" verified="false" note="Stage E adds this slot for active-call animation"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M06.D-retrospective.md" verified="true"/>
  </phase_doc_inventory_audit>

  <architecture_check>
    <claim description="MCPNode reads currentMcpServers via useShallow per gotcha #75 + v1.6 zustand_selector_audit; no naive filter/map/find without useShallow" verify="grep -B2 'currentMcpServers' src/components/nodes/MCPNode.tsx ; expect useShallow wrap"/>
    <claim description="MCPServerAddModal stays renderer-side; Tauri command does the heavy lift; no new HITL seam variant per ADR-0007" verify="grep 'McpAddSeam\\|McpSeam' src/components/ -r ; expect zero matches"/>
    <claim description="every new className has a corresponding CSS rule per gotcha #67" verify="for each className listed in E.3.6, grep src/styles.css for the rule; expect every class found"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="MCPNode" purpose="confirm MCPNode is referenced from GraphCanvas.tsx (nodeTypes map); existing M03 wiring"/>
    <grep pattern="mcp_add_server\\|mcp_remove_server\\|mcp_test_connection\\|mcp_list_servers" purpose="confirm Stage C wired the Tauri commands"/>
  </fan_out_grep>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="renderer-vitest" target_lines="80" note="vitest.config.ts gate"/>
  </coverage_gate>

  <zustand_selector_audit>
    <selector pattern="filter|map|find" requires_use_shallow="true" import_path="zustand/react/shallow"/>
  </zustand_selector_audit>

  <playwright_warmup_recipe url="http://localhost:1420" timeout_seconds="16" before_first_spec="true"/>

  <test_isolation_audit>
    <persistent_slot store="useGraphStore" field="currentMcpServers" preserved_across_clear="true" required_reset="beforeEach(() => useGraphStore.setState({ currentMcpServers: new Map() }))"/>
    <persistent_slot store="useGraphStore" field="activeMcpCalls" preserved_across_clear="true" required_reset="beforeEach(() => useGraphStore.setState({ activeMcpCalls: new Map() }))"/>
  </test_isolation_audit>

  <existing_pattern_audit>
    <pattern grep_for="nodeTypes" rationale="MCPNode is registered in nodeTypes map per M03; Stage E extends rather than re-registers" affected_files="src/components/GraphCanvas.tsx" remediation="confirm existing registration; do not redeclare"/>
  </existing_pattern_audit>

  <runtime_environment os="windows" note="Vitest + Playwright run on the Tauri dev server; Vite cold-start mitigated per gotcha #53"/>

  <gotchas>
    <trap>vitest re-query after await (gotcha #27) — when asserting after `userEvent.click`, re-query the DOM rather than reusing the captured handle.</trap>
    <trap>unwrapCmdError (gotcha #30) — Tauri errors come as objects; use the existing helper.</trap>
    <trap>window.__graphStore (gotcha #54) — renderer-level Playwright tests inject state via the existing affordance.</trap>
    <trap>component+CSS contract (gotcha #67) — every className gets a CSS rule + a static test confirming it.</trap>
    <trap>contract tests (gotcha #66) — assert visual state (computed-style on status indicator) not just className strings.</trap>
    <trap>useShallow for derived selectors (gotcha #75) — wrap every derived array/object selector.</trap>
    <trap>Vite cold-start (gotcha #53) — Playwright first-spec timeout configured per v1.6 playwright_warmup_recipe.</trap>
    <trap>persistent store slots survive clear() — beforeEach reset required (per v1.6 test_isolation_audit).</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT introduce a new HITLModal variant for MCP install confirmation — Add modal is a regular renderer modal (not an HITL seam). The MCP capability_violation modal (Stage D's deny-path) DOES reuse HITLModal per ADR-0007 — that's Stage D's concern, not Stage E's.</warning>
    <warning>DO NOT add Tauri commands in Stage E — Stage C added the 4 mcp_* commands. If a new command appears necessary, surface in retrospective; default is "Stage E renders only".</warning>
    <warning>DO NOT call `window.__graphStore` from production code — it's a Playwright-only test affordance per gotcha #54. Production renderer reads useGraphStore() directly.</warning>
    <warning>The Add modal's tier-eval display reuses M05.D's tier evaluator surface — if a new Tauri command is needed to surface the eval, file as a Stage D retroactive add OR surface in retrospective; do not introduce a new tier primitive at Stage E.</warning>
  </execution_warnings>

  <time_box estimate_hours="5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Tier-eval display surface — was a Stage D retroactive Tauri command needed, OR did Stage D ship one already? Document the path. CapabilityBadge reuse for MCP server tier surface — note any awkwardness in the pattern as it scales. Settings panel routing — the existing M04.E pattern is the source-of-truth; surface any drift.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M06-mcp-basic.md" section="E.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state</item>
    <item>diff stat</item>
    <item>gate results (v1.6 canonical order; vitest renderer ≥80%; Playwright + warmup; CI-parity)</item>
    <item>every-class-has-CSS-rule confirmation (the static check from M04.F)</item>
    <item>retrospective filled-in [END] section</item>
    <item>draft commit message from E.6</item>
    <item>explicit statement: "Stage M06.E is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### E.6 Commit Message

```
feat(renderer): M06 Stage E — MCPNode live wiring + Settings → MCP Servers UI

Lights up the renderer surface for MCP. Extends MCPNode (M03 stub) with
live connection status + tool list + active-call animation. Adds
Settings → MCP Servers section with Add / Remove / Test affordances
driving the Stage C Tauri commands. Tier-gate display in the Add modal
per spec §8.security L4. Playwright behavior test for the Add flow via
window.__graphStore state injection.

Components:
- src/components/nodes/MCPNode.tsx: extended with useShallow-wrapped
  currentMcpServers + activeMcpCalls selectors; status indicator;
  tool list panel; active-call animation class.
- src/components/MCPServerSettings.tsx (new): list of installed
  servers + per-row status + Add/Test/Remove buttons.
- src/components/MCPServerAddModal.tsx (new): form for name + transport
  + command/url + args + env + auth_secret; tier-eval display
  pre-confirmation.

IPC:
- src/lib/ipc.ts: typed wrappers for mcp_add_server, mcp_remove_server,
  mcp_test_connection, mcp_list_servers (Stage C-shipped Tauri commands).

Store extensions:
- src/lib/graphStore.ts: activeMcpCalls slot (Map<server, callId>) for
  the active-call animation; populated from tool_invoked events with
  MCP server context.

Styles:
- src/styles.css: .mcp-server-row + variants per status (connected /
  health_pending / disconnected / error); .mcp-server-add-modal;
  .mcp-node + variants for status; .mcp-tool-list. Every className
  paired with a CSS rule per gotcha #67.

Tests:
- tests/unit/nodes/MCPNode.test.tsx: 6 tests covering status indicator
  variants + tool list + active-call class + useShallow no-loop check.
  beforeEach reset of currentMcpServers + activeMcpCalls (per v1.6
  test_isolation_audit).
- tests/unit/components/MCPServerSettings.test.tsx (new): 7 tests
  covering empty state + row rendering + Add/Test/Remove wiring +
  status indicators + contract tests for repeated renders.
- tests/unit/components/MCPServerAddModal.test.tsx (new): 6 tests
  covering field validation + transport-specific submission + CSV/env
  parsing + tier-eval display.
- tests/e2e/mcp_server_add.spec.ts (new): Playwright behavior test
  for the renderer Add flow against the Vite dev server. timeout 90s
  per gotcha #53 (Vite cold-start). curl warmup probe per v1.6
  playwright_warmup_recipe.

Coverage: renderer vitest ≥80% on src/components/MCPServer*.tsx +
src/components/nodes/MCPNode.tsx.

cargo gates unchanged (renderer-only stage).

Not in this stage: multi-server collision resolution UI (v1.0), MCP
server discovery / browsing (v1.0), in-graph drag-drop wiring of MCP
tools to agents (M08 Builder Canvas).

https://claude.ai/code/session_<id>
```

---

<!-- ============================================================ -->
<!-- Stage F — Production wire (maintainer-inserted post-M06.E).   -->
<!-- Closes the ADR-0010 composition-root wire IN-milestone        -->
<!-- rather than carry the headline deliverable to M07.            -->
<!-- ============================================================ -->

## Stage F — src-tauri MCP-dispatch injection + live run-loop interception + gotcha #68 fix

### F.1 Problem Statement

> **Forward-correction (ADR-0011, surfaced before the F red phase — same lineage as ADR-0010's pre-red reconciliation).** The original F.1 framing below ("dispatching end-to-end … in the *running app*") over-reached the code reality + Stage F's own scope locks. Grep-verified before red: (1) **no `impl ConnectionResolver for McpClient` exists** (the M06.D retro special-log + `dispatch.rs` doc-comment claim "McpClient impls it for production" is **false** — only the trait + the test mock exist), so the concrete `McpDispatcher` is **not constructible** in `src-tauri`; (2) F's own `<execution_warnings>` #1 + F.1 "nothing new in `runtime-mcp`" **forbid** adding that adapter; (3) no `CapabilityEnforcer`/`NamespaceResolver` is constructed in the shell; (4) the only `AgentSdk` construction is the fixed no-tools `run_smoke_session` "hello" path — it emits no `ProviderEvent::ToolUse`; the agent-with-tools loop is M07 (pre-existing Stage A `<scope_change>` + ADR-0009, restated in F.1's own "Not in this stage"). **Honest, in-scope F mandate (per ADR-0011, maintainer-selected):** deliver the SDK run-loop interception seam **+** the src-tauri `*_with` composition-root injection seam, both **mock-verified** per the ADR-0010 / `Arc<dyn _>` shell-injected-seam archetype (the same way `Arc<dyn Connection>` / `Arc<AuditWriter>` are verified — mock at the seam, concrete OS-call construction is the excluded holdout). M06.D `<scope_change>` #1+#2 are **CLOSED at the seam + injection-seam level**. The concrete-`McpDispatcher` construction + `impl ConnectionResolver for McpClient` glue + the live agent-loop exercise are an **explicit M07 carry-forward** (ADR-0011 *Neutral / future implications* (a)-(d)) — NOT a Stage F miss and NOT an M06.V 🔴. Read the prose below through this correction. F.5/F.6 ARE forward-corrected to match (F is unexecuted — the grandfathered-not-edited precedent applies only to *executed* stages' prompts, not an un-run one).

M06's headline deliverable — the SDK run loop reaching an injected `Arc<dyn McpToolDispatch>` so MCP tool calls route through the L1 capability gate with agent_id-correct events — is not wired. Stage A built the `ProviderEvent::ToolUse` L1 interception point in `event_pipeline.rs`; Stage D built the `McpToolDispatch` trait + concrete `McpDispatcher` + integration tests (per ADR-0010 dependency inversion); Stage E was renderer-only. The SDK-side interception + the composition-root injection *seam* that connect them exist nowhere. Without them the architectural wire is absent: units green, the run loop has no path to a dispatcher. This is the gotcha #66 / ADR-0009-recurrence pattern; the maintainer's post-M06.E scope call closes the **buildable, in-scope** portion in-milestone via this focused wire stage (the residual concrete-construction + live exercise is the ADR-0011 M07 carry-forward, not a cross-milestone waiver of the seam).

Concrete deliverables (as scoped by ADR-0011):
1. **src-tauri composition-root injection *seam* (mock-verified holdout pattern).** The `*_with` seam (`run_smoke_session_with`) accepts an `Option<Arc<dyn McpToolDispatch>>` and applies `.with_mcp_dispatch` when present — unit-tested with a **mock** dispatch. The production `run_smoke_session` wrapper passes `None` for now (the concrete `McpDispatcher` is not constructible — see the forward-correction); same OS-call-holdout pattern as `providers/anthropic.rs` / `key_store.rs` / `open_mcp_client` (CLAUDE.md §5 — the seam gets the unit test, the wrapper is the excluded holdout). Constructing the concrete `McpDispatcher` in `main.rs` is the **M07 carry-forward** (ADR-0011), not this stage.
2. **Live run-loop interception.** The `AgentSdk` run loop gains an `Option<Arc<dyn McpToolDispatch>>` field + a `with_mcp_dispatch` builder seam; at the Stage A `ProviderEvent::ToolUse` site (the run loop in `agent_sdk.rs`, which is async — `event_pipeline.rs::next_event` is sync and cannot host the async dispatch; reconcile-not-escalate per the M06.D/E grandfathered-doc precedent), it calls `dispatch_if_mcp` FIRST: `None` → fall through to Stage A's existing non-MCP L1 path (unchanged); `Some(Ok(Invoked))` → the run loop emits **agent_id-correct** `ToolInvoked` + `ToolResult` directly; `Some(Ok(Blocked))` → `apply_mcp_dispatch` events + `outcome_needs_hitl` HITL await (existing `on_capability_violation` trigger, ADR-0007); `Some(Err)` → `mcp_dispatch_error_event`.
3. **gotcha #68 fix (the empty-`agent_id`).** `apply_mcp_dispatch`'s Invoked branch currently emits an empty `agent_id` (the M06.D `<scope_change>` #2). Fixed via **run-loop-emits-Invoked-events-directly** (the F.3-preferred approach): the run loop holds `agent_id` and emits the success `ToolInvoked`/`ToolResult` itself, leaving `apply_mcp_dispatch` + `McpDispatchOutcome` untouched so the D-frozen `mcp_dispatch_wire.rs` integration test stays intact. The renderer MUST receive a correct non-empty `agent_id`.
4. **≥95% runtime-main** on the run-loop interception logic + a new wire test against a **mock** `Arc<dyn McpToolDispatch>`. The src-tauri injection seam's wrapper is the shell-injection holdout (the `commands.rs` / `main.rs` wrappers are excluded per the M02.C / M05 `*_with`-seam precedent — CLAUDE.md §5); the `*_with` testable seam IS unit-tested.

Not in this stage: anything renderer (E shipped it); anything new in `runtime-mcp` incl. `impl ConnectionResolver for McpClient` (D shipped the dispatcher; the adapter is the ADR-0011 M07 carry-forward); the concrete `McpDispatcher` constructed in `main.rs` (ADR-0011 M07 carry-forward — not constructible in-shell yet); M07 multi-turn / agent-with-tools loop (Stage A `<scope_change>` + ADR-0009, stays M07 — it is what exercises this wire live).

### F.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/sdk/agent_sdk.rs` | exists | `Option<Arc<dyn McpToolDispatch>>` field + constructor seam (`with_mcp_dispatch`) |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | exists | At the Stage A `ProviderEvent::ToolUse` site, call `dispatch_if_mcp` first; route None/Some(Ok)/Some(Err) per F.1.2 |
| `crates/runtime-main/src/sdk/mcp_dispatch.rs` | exists | `apply_mcp_dispatch` agent_id fix (signature or run-loop-emits-directly) per F.1.3 |
| `crates/runtime-mcp/src/dispatch.rs` | exists | Only if the chosen agent_id approach needs `McpDispatchOutcome` plumbing (prefer the run-loop-emits-directly approach to keep the D-frozen `{server,tool,value}` test intact) |
| `src-tauri/src/main.rs` | exists | Construct concrete `McpDispatcher`; inject `Arc<dyn McpToolDispatch>` into `AgentSdk` (mirrors `open_mcp_client`) |
| `src-tauri/src/commands.rs` | exists | `*_with` seam only if needed for the injection-wiring unit test |
| `crates/runtime-main/tests/mcp_dispatch_runloop.rs` | **new** | Wire test: MCP tool → injected dispatch → agent_id-correct events; non-MCP → fall-through; blocked → HITL; twice-in-sequence |
| `CHANGELOG.md` | exists | `[Unreleased]` notes M06.F |
| `docs/build-prompts/retrospectives/M06.F-retrospective.md` | **new** | Stage F retrospective |

### F.3 Detailed Changes

The run loop holds `agent_id`; the dispatcher does not (ADR-0010 keeps the trait minimal). Preferred approach for F.1.3: the run-loop interception, on `Some(Ok(McpDispatchOutcome::Invoked { server, tool, value }))`, emits `ToolInvoked { agent_id, .. }` + `ToolResult { agent_id, .. }` itself with the `agent_id` it holds — leaving `McpDispatchOutcome` (and the D-frozen integration test) untouched. `apply_mcp_dispatch` becomes a thin mapper the run loop calls with `agent_id` in scope, or is inlined at the interception site. Blocked/Err path reuses the existing `on_capability_violation` HITL trigger (no new seam — ADR-0007). The src-tauri injection mirrors the existing best-effort `open_mcp_client` pattern (parallel to the M05.E audit-writer open): construct, inject, log-on-failure, never panic the shell.

### F.4 Tests

`crates/runtime-main/tests/mcp_dispatch_runloop.rs` (new):
- `mcp_tool_use_routes_through_injected_dispatch_and_emits_agent_id_correct_events` — asserts `agent_id` is non-empty AND equals the run-loop agent (gotcha #68 — the load-bearing assertion)
- `non_mcp_tool_use_falls_through_to_stage_a_l1_path_unchanged` — `dispatch_if_mcp` None → existing Stage A behavior, no regression
- `blocked_mcp_tool_use_awaits_hitl_and_emits_dispatch_error_event` — Some(Err)/violation path + existing trigger
- `mcp_tool_use_twice_in_sequence_both_emit_correct_events` (gotcha #69)
- src-tauri injection-wiring unit test via the `*_with` seam (the shell wrapper itself is the excluded holdout; the seam is tested)

Acceptance: all four run-loop tests + the seam test pass; `cargo llvm-cov -p runtime-main` ≥95% holds (exact CI cmd; `cargo llvm-cov clean` first per gotcha #81); full v1.6 canonical gate suite green; no Co-Authored-By; v1.7 invariant `git diff <red>..<impl> -- '**/tests/**'` EMPTY stated in the impl commit body.

### F.5 CLI Prompt

```xml
<work_stage_prompt id="M06.F">
  <context>
    Stage F of M06 (maintainer-inserted post-M06.E, pre-V). Closes the
    ADR-0010 composition-root wire SEAM in-milestone, scoped per ADR-0011
    (forward-correction surfaced + accepted pre-F-red): the AgentSdk run
    loop gains an Option&lt;Arc&lt;dyn McpToolDispatch&gt;&gt; field + a
    with_mcp_dispatch builder seam, and at the Stage A
    ProviderEvent::ToolUse site calls dispatch_if_mcp first (None→Stage A
    fall-through unchanged; Some(Ok(Invoked))→agent_id-correct
    ToolInvoked+ToolResult emitted directly by the run loop;
    Some(Ok(Blocked))→on_capability_violation HITL; Some(Err)→
    mcp_dispatch_error_event); src-tauri exposes the *_with injection
    seam (run_smoke_session_with accepts + applies Option&lt;Arc&lt;dyn
    McpToolDispatch&gt;&gt;), mock-verified per the ADR-0010/Arc&lt;dyn _&gt;
    archetype. Fixes the apply_mcp_dispatch empty-agent_id (gotcha #68,
    M06.D &lt;scope_change&gt; #2) test-first via run-loop-emits-directly
    (apply_mcp_dispatch + the D-frozen McpDispatchOutcome test left
    untouched). Per ADR-0011 the CONCRETE McpDispatcher construction +
    impl ConnectionResolver for McpClient + the live agent-loop exercise
    are the explicit M07 carry-forward (NOT this stage; not constructible
    in-shell and forbidden by F's own runtime-mcp scope lock). This is
    the headline M06 deliverable's architectural wire seam. Strict v1.7
    two-commit TDD.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M06-mcp-basic.md (Stage F sections F.1–F.4 incl. the ADR-0011 forward-correction banner; the M06.D &lt;scope_change&gt; block this stage closes at the seam level; ADR-0010)</file>
    <file>docs/adr/0011-m06f-scope-seam-not-running-app.md (the F-scope split this stage executes: 11a seam delivered, 11b M07 carry-forward)</file>
    <file>docs/adr/0010-mcp-dispatch-dependency-inversion.md (the seam this stage injects)</file>
    <file>docs/adr/0007-in-process-hitl-seam-architecture.md (shell-injected-seam archetype + on_capability_violation reuse)</file>
    <file>docs/build-prompts/retrospectives/M06.D-retrospective.md ([END] special-log + Decisions — the empty-agent_id + run-loop-injection carry-forwards)</file>
    <file>docs/build-prompts/retrospectives/M06.E-retrospective.md (immediate prior stage Decisions)</file>
    <file>docs/gotchas.md (#66 tests-pass-contract-fails, #68 empty-field-to-renderer, #69 multi-call, #81 llvm-cov clean)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="Stage A L1 ProviderEvent::ToolUse interception site this stage extends">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="ADR-0010 seam + apply_mcp_dispatch (agent_id fix target)">crates/runtime-main/src/sdk/mcp_dispatch.rs</file>
    <file purpose="concrete McpDispatcher to inject">crates/runtime-mcp/src/dispatch.rs</file>
    <file purpose="composition-root injection archetype (best-effort open, log-on-failure)">src-tauri/src/main.rs</file>
    <file purpose="AgentSdk run loop + constructor-seam pattern">crates/runtime-main/src/sdk/agent_sdk.rs</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M06.E" retro="docs/build-prompts/retrospectives/M06.E-retrospective.md"/>
    <stage id="M06.D" retro="docs/build-prompts/retrospectives/M06.D-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M06-mcp-basic.md" section="F.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests in crates/runtime-main/tests/mcp_dispatch_runloop.rs
      + the src-tauri *_with seam test. Stub run-loop wiring just enough
      to compile (todo!()/unimplemented!()). Confirm right-reason
      failure per CLAUDE.md §5. Commit STANDALONE
      `test(M06.F): failing tests for run-loop MCP dispatch wire` on
      claude/m06-mcp-basic; body pastes ~40 lines proving expected
      failure. Surface for red approval before green.
    </red_phase>
    <green_phase>
      Implement until all pass. Do NOT modify test files in the impl
      commit — fix wrong tests in a separate labelled follow-up. Impl
      commit body states the verifiable invariant
      `git diff &lt;red-sha&gt;..&lt;impl-sha&gt; -- '**/tests/**'` is EMPTY.
      Mechanical fmt + net-new tests in a separate follow-up commit.
      No Co-Authored-By (M06.B Decision).
    </red_phase>
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

  <acceptance_criteria ref="docs/build-prompts/M06-mcp-basic.md" section="F.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M06-mcp-basic.md" section="Key constraints"/>

  <gates milestone="M06"/>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.key_store\.rs"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch" gate="git branch --show-current == claude/m06-mcp-basic"/>
    <check name="prior_pushed" gate="origin/claude/m06-mcp-basic at f31faf7 or later (M06.E pushed); M06.F builds on it"/>
    <check name="adr_0010_present" gate="docs/adr/0010-mcp-dispatch-dependency-inversion.md exists"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="crates/runtime-main/src/sdk/event_pipeline.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/sdk/mcp_dispatch.rs" verified="true"/>
    <claim type="file" path="crates/runtime-mcp/src/dispatch.rs" verified="true"/>
    <claim type="file" path="src-tauri/src/main.rs" verified="true"/>
    <claim type="method" path="crates/runtime-mcp/src/dispatch.rs" symbol="McpDispatcher" verified="true"/>
    <claim type="method" path="crates/runtime-main/src/sdk/mcp_dispatch.rs" symbol="apply_mcp_dispatch" verified="true"/>
  </phase_doc_inventory_audit>

  <runtime_environment os="windows"/>

  <gotchas>
    <trap>#68 — apply_mcp_dispatch must NOT emit empty agent_id to the renderer. The load-bearing test asserts agent_id is non-empty AND equals the run-loop agent. Prefer run-loop-emits-Invoked-events-directly so the D-frozen McpDispatchOutcome {server,tool,value} integration test stays intact.</trap>
    <trap>#66 — assert the OBSERVABLE contract (agent_id-correct events reach the renderer-facing event stream), not just that apply_mcp_dispatch returns Ok.</trap>
    <trap>#69 — twice-in-sequence run-loop dispatch test.</trap>
    <trap>#81 — `cargo llvm-cov clean` before the ≥95% measurement (prior-run .profraw merge).</trap>
    <trap>ADR-0007 — blocked/Err path reuses on_capability_violation HITL trigger; do NOT add a new seam.</trap>
  </gotchas>

  <execution_warnings>
    <warning>Do NOT modify runtime-mcp's dispatcher or namespace logic — D shipped + tested it. F only wires it.</warning>
    <warning>Do NOT touch renderer — E shipped it.</warning>
    <warning>src-tauri main.rs/commands.rs wrappers are the coverage holdout (M02.C/M05 precedent); the `*_with` seam is what gets the unit test, not the wrapper.</warning>
    <warning>Non-MCP ProviderEvent::ToolUse MUST still take Stage A's existing L1 path unchanged — the dispatch_if_mcp None branch is a pure fall-through; assert no regression.</warning>
  </execution_warnings>

  <time_box hours="3-4"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Confirm the M06.D &lt;scope_change&gt; #1 + #2 are now CLOSED (run-loop wire delivered + agent_id-correct), so M06.V Wire trace #11 verifies a real wire (🔴 if regressed) rather than reading a deferred descope. State which agent_id approach was taken (signature change vs run-loop-emits-directly) + why. Note whether the src-tauri injection needed a new `*_with` seam or reused an existing one.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M06-mcp-basic.md" section="F.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + retro listing incl. M06.F)</item>
    <item>strict-TDD invariant: git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**' EMPTY</item>
    <item>gate results (v1.6 canonical order; runtime-main ≥95%; CI-parity, cite any divergence)</item>
    <item>gotcha #68 closure proof: the agent_id-correct assertion test name + that it asserts non-empty AND equals run-loop agent</item>
    <item>confirmation M06.D &lt;scope_change&gt; #1+#2 are closed (so M06.V trace #11 expects DELIVERED)</item>
    <item>M06.F retrospective [END]</item>
    <item>explicit: "Stage M06.F is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### F.6 Commit Message

```
feat(runtime): M06 Stage F — MCP-dispatch run-loop interception + src-tauri injection seam + gotcha #68 fix

Closes the ADR-0010 composition-root wire SEAM in-milestone
(maintainer scope call post-M06.E), scoped per ADR-0011
(forward-correction accepted pre-F-red). The AgentSdk run loop
gains an Option<Arc<dyn McpToolDispatch>> field + a with_mcp_dispatch
builder seam and intercepts ProviderEvent::ToolUse at the Stage A
L1 site: None → Stage A non-MCP fall-through (unchanged);
Some(Ok(Invoked)) → agent_id-correct ToolInvoked + ToolResult
emitted directly by the run loop; Some(Ok(Blocked)) →
on_capability_violation HITL await; Some(Err) →
mcp_dispatch_error_event. src-tauri exposes the *_with injection
seam (run_smoke_session_with accepts + applies the optional
dispatch), mock-verified per the ADR-0010 / Arc<dyn _> archetype.

Fixes the apply_mcp_dispatch empty-agent_id (gotcha #68, M06.D
<scope_change> #2): the run loop emits Invoked success events
directly with the agent_id it holds, leaving the D-frozen
McpDispatchOutcome {server,tool,value} integration test intact.

Per ADR-0011 the concrete McpDispatcher construction +
impl ConnectionResolver for McpClient + the live agent-loop
exercise are the explicit M07 carry-forward (not constructible
in-shell; forbidden by F's runtime-mcp scope lock). M06.D
<scope_change> #1 + #2 are CLOSED at the seam + injection-seam
level — M06.V Wire trace #11 SPLIT: 11a delivered/mock-verified,
11b is the ADR-0011 M07 carry-forward (NOT a 🔴).

Coverage: runtime-main ≥95% on the run-loop interception + new
crates/runtime-main/tests/mcp_dispatch_runloop.rs wire test
(agent_id-correct, non-MCP fall-through, blocked→HITL, twice-in-
sequence) against a mock Arc<dyn McpToolDispatch>. The src-tauri
*_with injection seam is mock-tested; the production wrapper
(passes None until M07) is the holdout per the M02.C/M05
precedent (CLAUDE.md §5).

Strict v1.7 two-commit TDD: git diff <red>..<impl> -- '**/tests/**'
EMPTY (stated verbatim in the impl commit body).

https://claude.ai/code/session_<id>
```

---

<!-- ============================================================ -->
<!-- Stage V — Verifier (per v1.5/v1.6 — fresh CLI session, four passes). -->
<!-- Runs between Stage E (last work stage) and Stage G (closeout).         -->
<!-- ============================================================ -->

## Stage V — Verifier (per v1.5/v1.6 — first M06 V run in-band)

> Per ADR-0008 + `STAGE-PROMPT-PROTOCOL.md` §14. Runs between Stage E (last work stage) and Stage G (closeout). Fresh CLI session; clear-and-paste bias guard. Four passes (Inventory + Wire + Behavior + Multi-call invariants). Findings tagged 🔴 / 🟡 / 🟢 with merge gating per the severity model. M06.V is the SECOND in-band V run (M05.V was the first; M04.V was retroactive).

### V.1 Problem Statement

Run the four-pass verifier against M06's deliverables (Stages A–F) in a fresh CLI session. The agent reads only the spec, the phase doc body (V.1–V.6 included), the current code, the verifier templates, the phase doc's `<scope_change>` blocks (per v1.6 STAGE-V-VERIFIER-PROMPT-TEMPLATE.md update), **AND ADR-0011** (the F-scope forward-correction, per the same `<scope_change>`-into-V-read-list mechanism) — NOT the M06.A–F retrospectives or M06-summary or `docs/gap-analysis.md`. Bias guard is structural via clear-and-paste. Note: Stage F was maintainer-inserted post-M06.E to close the M06.D `<scope_change>` #1+#2 in-milestone; **per ADR-0011, trace #11 is SPLIT**: the SDK run-loop interception seam **+** the src-tauri `*_with` injection seam are verified DELIVERED/**mock-verified** (🔴 if *that* seam/injection-seam is missing or regressed); the concrete-`McpDispatcher` construction + `impl ConnectionResolver for McpClient` glue + live agent-loop exercise are the ADR-0011 **M07 carry-forward** and are **NOT** an M06.V 🔴 (V reads ADR-0011 and adjusts the trace #11 expectation accordingly — the ADR-0009 "if the next milestone doesn't wire it, that's the verifier's expected endpoint" mechanism, one milestone later).

M06.V is the FIRST V run to consume v1.6's `<scope_change>` slot (M05.V's Decision 3 + ADR-0009's "future implications"); if any descope appears in a per-stage `<scope_change>` block, V reads it and adjusts expectations rather than emitting 🔴 for the documented carry-forward.

The two structural endpoints V must trace are the **ADR-0009 closure** — `enforcer.check` before `provider.invoke` in `event_pipeline.rs` (M05.V Finding #1 trace) AND `narrow` before `AgentSpawned` in `agent_sdk.rs:~124` (M05.V Finding #2 trace). If either trace breaks at step 4 (zero matching consumers), V emits 🔴 and M06 cannot merge without D.fix iter OR a new waiver-as-ADR.

### V.2 Scope to verify

Aggregated from Stages A through F's X.2 tables:

| Layer | Files / surfaces in scope for M06.V |
|---|---|
| **Inventory** | Every file path from §A.2 + §B.2 + §C.2 + §D.2 + §E.2 + §F.2 (Stage A: SDK wire-ups + integration tests + `narrowed_from` schema + M05 phase-doc edit; Stage B: `runtime-mcp` crate creation + transport files + `mcp.v1.json`; Stage C: client lifecycle + registry + auth + migration + Tauri commands; Stage D: namespace + mcp_dispatch + 2 schema-event additions + framework `mcp_aliases`; Stage E: 3 new + 1 extended renderer components + Playwright + styles; Stage F: run-loop interception in `event_pipeline.rs` + `agent_sdk.rs` `with_mcp_dispatch` seam + `apply_mcp_dispatch` agent_id fix + src-tauri injection + `mcp_dispatch_runloop.rs`) — verifier checks `git ls-files` presence + shape match against §X.3 detailed-changes narrative. |
| **Wire** | Spec claims to trace end-to-end: ADR-0009 closure (L1 enforcer.check before provider.invoke; L2a narrow before AgentSpawned) + §5a tool namespace resolution (canonical / short / alias / re-resolution) + §5 MCP lifecycle (add → connect → invoke → disconnect) + §5 audit emissions (mcp_installed / mcp_uninstalled / mcp_auth_granted / mcp_request_blocked) + §8.security L1 MCP-tool gate (mcp_dispatch calls enforcer.check before Connection::invoke_tool) + **§5 MCP dispatch SEAM in the live run loop (ProviderEvent::ToolUse → injected `Arc<dyn McpToolDispatch>` → agent_id-correct ToolInvoked/ToolResult) — Stage F, per ADR-0011. trace #11 is SPLIT: the SDK run-loop interception seam + the src-tauri `*_with` injection seam are mock-verified DELIVERED (🔴 if THAT is missing/regressed); the concrete-`McpDispatcher` construction + ConnectionResolver-for-McpClient glue + live agent-loop exercise are the ADR-0011 M07 carry-forward, NOT an M06.V 🔴** + renderer wiring (currentMcpServers → MCPNode + MCPServerSettings). |
| **Behavior** | Vitest+jsdom: MCPNode renders status indicator variants; MCPServerSettings renders rows + handles Add/Test/Remove; MCPServerAddModal validates + submits. Static: every `.mcp-server-row--<status>` + `.mcp-node--<status>` class has a CSS rule. Rust: namespace resolver against fixture-server-set produces expected resolutions; mcp_dispatch end-to-end against mock transport + real CapabilityEnforcer emits expected events; client lifecycle adds + removes against tempfile SQLite + mock transport; **Stage F `mcp_dispatch_runloop.rs` — MCP ToolUse → injected dispatch → agent_id-correct events, non-MCP fall-through, blocked→HITL, twice-in-sequence**. Playwright: mcp_server_add.spec.ts passes against Vite dev server (with warmup). |
| **Multi-call** | `NamespaceResolver::resolve` (Stage D); `McpClient::add_server` + `remove_server` + `test_connection` + `get_connection` (Stage C); `mcp_dispatch::dispatch_if_mcp` (Stage D); run-loop MCP dispatch twice-in-sequence (Stage F); `StdioTransport::connect` + `HttpTransport::connect` (Stage B); regression: M04 + M05 multi-call tests still green. |

### V.3 Verification passes (per-pass detail for M06)

#### Inventory pass

For each file path from Stages A–F's X.2 tables (listed above), run `git ls-files` and confirm presence. For each `new` file, confirm shape matches the corresponding X.3 detailed-changes narrative (module boundaries, function names, exposed types). For each `exists` file, confirm the edits described in X.3 are present. Pay attention to: the new `runtime-mcp/` crate (workspace member + lib.rs + transport/* + client/* + namespace/* + error.rs); the new `schemas/mcp.v1.json` + regenerated `generated/mcp.rs` + `src/types/mcp.ts`; the new event variants in `event.v1.json` (3 from C + 2 from D); the `mcp_aliases` field on Framework; the 3 new renderer components + 2 new test files + Playwright spec; **Stage F: the `with_mcp_dispatch` seam on `agent_sdk.rs`, the run-loop interception in `event_pipeline.rs`, the `apply_mcp_dispatch` agent_id fix, the src-tauri injection in `main.rs`, and `crates/runtime-main/tests/mcp_dispatch_runloop.rs`**. Missing → 🔴; shape-drift → 🟡.

Per v1.6 STAGE-V-VERIFIER-PROMPT-TEMPLATE.md update, the Inventory pass also reads every per-stage `<scope_change>` block in the phase doc; intentional descopes (e.g., the multi-turn agent loop deferral in Stage A) are NOT flagged as inventory gaps.

#### Wire pass (5-step data-path tracing per gotcha #66 + the v1.5 template's authoring rule)

Traces for M06:

| Trace # | Spec claim | Source event | Projection | Consumer | Step 5 check |
|---|---|---|---|---|---|
| 1 | §8.security L1 "every tool dispatch checked" (ADR-0009 closure) | `ProviderEvent::ToolUse` (M02) | (no projection; enforcer runs in-process) | `event_pipeline.rs` calls `enforcer.check(agent_id, &needed)` BEFORE the ToolInvoked translation | grep finds production call site at event_pipeline.rs; M05.V Finding #1 endpoint satisfied |
| 2 | §8.security L2a "sub-agent spawn grants ⊆ parent" (ADR-0009 closure) | (no event; runtime check during framework-loader walk) | (no projection) | `agent_sdk.rs:~124` calls `narrow(parent_grants, proposed)` BEFORE `AgentSpawned` emission | grep finds production call site at agent_sdk.rs; M05.V Finding #2 endpoint satisfied |
| 3 | §5a "canonical `<server>__<tool>` resolution" | (no event; runtime resolution at dispatch) | (no projection) | `NamespaceResolver::resolve` called from `mcp_dispatch::dispatch_if_mcp` | grep finds call site in mcp_dispatch.rs; resolver returns ResolvedTool for valid canonical |
| 4 | §5a "short-name aliasing when unambiguous" | (no event) | (no projection) | Same resolver call returns ResolvedTool for unambiguous short name OR NamespaceError::Ambiguous for ambiguous | resolver implementation + test fixture validates both branches |
| 5 | §5a "explicit mcp_aliases framework override" | (no event) | (no projection) | Resolver reads framework.mcp_aliases passed in; alias overrides short-name ambiguity | resolver call site passes framework.mcp_aliases; test validates |
| 6 | §5a step 5 "re-resolution on connect/disconnect" | `mcp_installed` / `mcp_uninstalled` (M06.C + D variants) | (no projection — runtime call) | `McpClient::add_server` + `remove_server` call `NamespaceResolver::re_evaluate_short_names`; new ambiguities emit `tool_alias_ambiguous` event | client lifecycle has the call site; graphStore branch handles event |
| 7 | §5 "MCP lifecycle: add → connect → invoke → disconnect" | mcp_installed event | mcp_servers SQLite table + currentMcpServers store slot | MCPServerSettings renders rows; MCPNode renders status | trace from add_server through Registry to graphStore to MCPNode |
| 8 | §5 "audit emissions for MCP lifecycle" | mcp_installed / mcp_uninstalled / mcp_auth_granted / mcp_request_blocked | skills.audit.jsonl (M05.E writer) | file inspection | Audit log contains the entry per event |
| 9 | §3 Visual Design + §5 "MCPNode shows connection status" | mcp_installed → graphStore.currentMcpServers | MCPNode reads currentMcpServers via useShallow selector | rendered DOM shows status indicator with correct class | computed-style + class assertion |
| 10 | §3 "every node-status class has a CSS rule" | (no event; CSS-side check) | (no projection) | styles.css contains `.mcp-server-row--<status>` for each of 4 statuses AND `.mcp-node--<status>` for each | Static check via every_class_has_a_corresponding_CSS_rule pattern |
| 11 | §5 "MCP dispatch SEAM in the live run loop" (Stage F — SPLIT per ADR-0011; closes M06.D `<scope_change>` #1+#2 at the seam level) | `ProviderEvent::ToolUse` (M02) | (no projection; in-process dispatch) | **(11a, DELIVERED)** the `agent_sdk.rs` run loop holds an `Option<Arc<dyn McpToolDispatch>>` + a `with_mcp_dispatch` seam; at the `ProviderEvent::ToolUse` site it calls injected `dispatch_if_mcp` → `Some(Ok(Invoked))` emits **agent_id-correct** `ToolInvoked`+`ToolResult` directly; `None` falls through to Stage A's non-MCP L1 path unchanged; `Some(Ok(Blocked))`→HITL; `Some(Err)`→`mcp_dispatch_error_event`. The src-tauri `*_with` injection seam (`run_smoke_session_with`) accepts + applies `Option<Arc<dyn McpToolDispatch>>`. **(11b, M07 carry-forward — NOT 🔴)** the concrete `McpDispatcher` constructed in `main.rs` + `impl ConnectionResolver for McpClient` + a real agent-loop that emits a resolvable MCP `ToolUse`. | **11a:** grep finds the `with_mcp_dispatch` seam + the `dispatch_if_mcp` call at the run-loop `ToolUse` site + the `*_with` injection seam; `mcp_dispatch_runloop.rs` asserts (against a **mock** `Arc<dyn McpToolDispatch>`) agent_id is non-empty AND equals the run-loop agent (gotcha #68) + non-MCP fall-through + blocked→HITL + twice-in-sequence. **11a is expected DELIVERED/mock-verified — 🔴 only if the seam, the run-loop call site, the injection seam, OR the agent_id-correctness is missing/regressed.** **11b:** per ADR-0011 the concrete construction has no in-shell constructor inputs (no `impl ConnectionResolver for McpClient`; no shell enforcer; no-tools smoke path) and the agent-with-tools loop is M07 (Stage A `<scope_change>` + ADR-0009) — **11b is the documented M07 carry-forward, expected ABSENT at M06.V, NOT a 🔴.** |

Each trace breaks at step 4 with missing/multiple consumers → 🔴 ("wire incomplete" / "wire ambiguous"). Trace #11 specifically (per ADR-0011, which V reads): **11a** is 🔴 only if the `with_mcp_dispatch` seam, the run-loop `dispatch_if_mcp` call site, the src-tauri `*_with` injection seam, OR the non-empty/run-loop-correct `agent_id` is missing or regressed (Stage F's in-scope mandate, mock-verified). **11b** (concrete `McpDispatcher` in `main.rs` + `impl ConnectionResolver for McpClient` + a live agent-loop exercise) is the **documented ADR-0011 M07 carry-forward** — V must read it as expected-ABSENT-at-M06.V, **NOT** 🔴 (same mechanism as ADR-0009's M05→M06 deferral: if M07 doesn't wire 11b, that becomes M07.V's expected endpoint). V must NOT read 11a as still-deferred (the seam IS closed in-milestone) and must NOT read 11b as a Stage F miss (it is the ADR-0011 carry-forward).

#### Behavior pass

Run the live harness:

```cmd
:: Vitest renderer-level
npx vitest run tests/unit/nodes/MCPNode.test.tsx tests/unit/components/MCPServerSettings.test.tsx tests/unit/components/MCPServerAddModal.test.tsx tests/unit/graphStore.test.ts

:: Rust safety-primitive (new runtime-mcp crate)
cargo test -p runtime-mcp --lib
cargo test -p runtime-mcp --tests
cargo test -p runtime-main --lib capability --lib sdk
cargo test -p runtime-main --tests sdk_capability_integration sdk_narrowing_integration mcp_dispatch_integration mcp_dispatch_runloop

:: Coverage gates (the per-crate ≥95% gates for runtime-mcp + runtime-main)
cargo llvm-cov --package runtime-mcp --ignore-filename-regex "src.main\.rs|generated|src.lib\.rs" --fail-under-lines 95
cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.key_store\.rs" --fail-under-lines 95

:: Playwright
npm run test:e2e -- mcp_server_add.spec.ts
```

For each failure: trace which pass would have caught it (Inventory → file missing; Wire → step-5 mismatch; Multi-call → second call broken). Findings cite the failing test name + the pass that should have caught it earlier.

Additionally: Playwright `mcp_server_add.spec.ts` exercising the renderer-level injection per gotcha #54.

#### Multi-call invariants pass

For each public API/IPC method/Tauri command M06 adds:

| Surface | Twice-in-sequence test | Outcome expected |
|---|---|---|
| `NamespaceResolver::resolve` | `namespace::tests::resolve_twice_in_sequence_both_succeed` (Stage D) | Both calls return Ok with same ResolvedTool |
| `McpClient::add_server` | `client_lifecycle::add_server_twice_in_sequence_with_distinct_names_both_succeed` (Stage C) | Both add_server calls succeed with distinct names |
| `mcp_dispatch::dispatch_if_mcp` | `mcp_dispatch_integration::dispatch_twice_in_sequence_both_succeed` (Stage D) | Both dispatches succeed with distinct call IDs |
| `StdioTransport::invoke_tool` | `stdio_invoke_tool_twice_in_sequence_both_succeed` (Stage B) | Both invocations succeed |
| `HttpTransport::invoke_tool` | `http_invoke_tool_twice_in_sequence_both_succeed` (Stage B) | Both invocations succeed |
| `Registry::open` | `registry::tests::open_twice_in_sequence_does_not_re_run_migrations` (Stage C) | Migration idempotent |
| `enforcer.check` (regression from M05.B + Stage A) | `enforcer::tests::twice_in_sequence_both_succeed` + Stage A's integration tests | Both calls return Ok |
| `narrow` (regression from M05.B + Stage A) | `narrowing::tests` proptest + Stage A's integration tests | Both calls behave per invariant |
| `query_session_db` (M04 regression) | PR #64's test still green | (existing) |

Missing per-surface test → 🟡 (carry forward to TD-NNN); both-calls-don't-pass → 🔴.

### V.4 Findings format

Per `docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md` § Findings. Numbered globally across passes (#1, #2, #3...). Each finding cites: pass, primitive, spec claim, observed-vs-expected, recommended action.

### V.5 CLI Prompt

Paste into a **fresh** Claude Code session (clear-and-paste pattern is load-bearing).

```xml
<verifier_stage_prompt id="M06.V">
  <context>
    Stage V (Verifier) of M06. Fresh-context contract-fidelity check of
    M06's deliverables (ADR-0009 closure + §5 MCP Manager + §5a Tool
    Namespace Resolution + §8.security L1 MCP-tool gate + audit
    emissions + renderer UI) against `agent-runtime-spec.md`. Run with
    empty session memory — you have NOT seen the M06.A/B/C/D/E retros,
    the M06-summary (does not yet exist), or any prior gap-analysis
    entries beyond M05's. Four passes in order: Inventory → Wire →
    Behavior → Multi-call invariants. Findings tagged 🔴 (block merge →
    D.fix), 🟡 (carry forward), 🟢 (tech debt). Maximum 2 D.fix
    iterations before maintainer escalation.

    M06.V is the SECOND in-band V run (M05.V was the first; M04.V was
    retroactive per grandfathering). M06 is the FIRST milestone shipped
    under v1.6 protocol — read the per-stage `<scope_change>` blocks in
    the phase doc per the v1.6 STAGE-V-VERIFIER-PROMPT-TEMPLATE.md
    update; intentional descopes documented in those blocks are NOT
    flagged as inventory gaps.

    The ADR-0009 closure is M06's hard deliverable: M05.V Findings #1
    + #2 expected to be satisfied by Stage A's wire-up. If either
    trace (`enforcer.check before provider.invoke` OR `narrow before
    AgentSpawned`) breaks at step 4 of the Wire pass, emit 🔴 and
    surface for D.fix iter OR a new waiver-as-ADR.
  </context>

  <read_first>
    <file>STAGE-PROMPT-PROTOCOL.md §14 (the verifier schema)</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (design rationale + four passes + bias guard)</file>
    <file>docs/adr/0009-waiver-M05-l1-l2a-sdk-wire-deferral.md (the carry-forward this M06 V run validates the closure of)</file>
    <file>docs/build-prompts/M06-mcp-basic.md (Background, all stages A.1/A.2/A.3/A.4 through E.1/E.2/E.3/E.4, AND Stage V section V.1/V.2/V.3 — but NOT any retrospective references the doc may make)</file>
    <file>docs/build-prompts/M06-mcp-basic.md per-stage `<scope_change>` blocks (per v1.6 STAGE-V-VERIFIER-PROMPT-TEMPLATE.md update — intentional descopes are load-bearing for V's read-list)</file>
    <file>agent-runtime-spec.md §5 (MCP Manager); §5a (Tool Namespace Resolution); §8.security L1 + L2a (the closure this stage validates); §3 Visual Design (MCPNode); §13.5 audit log</file>
    <file>docs/MVP-v0.1.md §M6 (acceptance criteria)</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #43, #57, #66, #67, #68, #69, #72, #73, #74, #75, #76, #77 — the M04/M05 IRL bug patterns now codified)</file>
    <file>docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md (output shape)</file>
    <file>docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md (parameterization guidance — especially the v1.6 update flagging `<scope_change>` blocks as load-bearing)</file>
    <file>docs/tech-debt.md (TD-001..NNN already logged; M06.V's 🟢 findings append here)</file>
  </read_first>

  <scope_to_verify ref="docs/build-prompts/M06-mcp-basic.md" section="V.2 Scope to verify"/>

  <verification_passes>
    <pass name="inventory">
      For each file path enumerated in M06's Stage A.2 + B.2 + C.2 +
      D.2 + E.2 + F.2 "Files to Change" tables, confirm presence in
      `git ls-files` AND shape-match against the corresponding X.3
      "Detailed Changes" narrative. Missing → 🔴. Stub/empty → 🟡.
      Wrong scope/signature → 🟡. Pay attention to: the new
      runtime-mcp crate (workspace member + transport/* + client/* +
      namespace/*); the new schemas/mcp.v1.json + regenerated
      generated/mcp.rs + src/types/mcp.ts; the 5 new event variants
      in event.v1.json (3 from C + 2 from D); the mcp_aliases field
      on framework.v1.json; the 3 new renderer components + 2 new
      test files + Playwright spec. ALSO read per-stage `<scope_change>`
      blocks in the phase doc; intentional descopes are not flagged
      as gaps. Stage A's `<scope_change>` block documents the
      multi-turn agent loop deferral to M07.
    </pass>
    <pass name="wire">
      Run the ten Wire traces from V.3 above using the 5-step
      protocol. Trace breaks at step 4 with zero matching consumers
      OR multiple plausible consumers → 🔴. Note: traces #1 + #2 are
      the ADR-0009 closure endpoints (M05.V Findings #1 + #2);
      missing trace evidence here means the closure didn't land →
      🔴 with D.fix or new waiver-as-ADR.
    </pass>
    <pass name="behavior">
      Run the live harness from V.3 (Vitest + cargo test + cargo
      llvm-cov + Playwright). For each failing test, trace which
      pass (Inventory / Wire / Multi-call) would have caught it
      earlier. Coverage failures on the runtime-mcp + runtime-main
      per-crate ≥95% gates are 🔴 if below threshold. Renderer-side
      computed-style checks per gotcha #67. Playwright + warmup
      per v1.6 playwright_warmup_recipe.
    </pass>
    <pass name="multi_call_invariants">
      Run the nine sequential-call tests from V.3. For each surface
      lacking a twice-in-sequence test → 🟡 (carry forward to TD-NNN).
      For any test where the second call fails → 🔴. Confirm M04 + M05
      regression tests still green (drone IPC, respond_hitl,
      framework_loader, capability_enforcer smoke, sandbox round-trip,
      tier evaluator, audit writer).
    </pass>
  </verification_passes>

  <findings_format ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md" section="Findings"/>

  <merge_gate red_blocks="true" dfix_iteration_cap="2" waiver_path="docs/adr/NNNN-waiver-M06-finding-N.md"/>

  <gates milestone="M06"/>

  <self_correction_budget>3</self_correction_budget>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md">
    <special_log>M06.V is the SECOND in-band V run + the first under v1.6 protocol. Log explicit calibration observations: did the v1.6 `<scope_change>` slot consumption work cleanly (did V read the descope blocks; did any 🔴 finding turn out to be a missed-scope-change)? Did the four-pass shape feel adequate at M06's scope (mixed novel-protocol + lifecycle + dispatch + renderer)? Were any new bug classes surfaced that M05.V didn't see (e.g., transport-protocol-correctness bugs the mock-transport missed)? Should v1.7 protocol-iteration add new passes? Decisions[END] section should include explicit protocol refinement recommendations.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M06-mcp-basic.md" section="V.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (build machine `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M06.*-retrospective.md`)</item>
    <item>findings list, sorted by severity</item>
    <item>per-pass summary (counts + notable findings)</item>
    <item>ADR-0009 closure confirmation: traces #1 + #2 endpoint disposition (satisfied / unsatisfied / deferred via new waiver)</item>
    <item>retrospective filled-in [END] section per VERIFIER-RETROSPECTIVE-TEMPLATE.md (verification axes + v1.6 scope_change slot consumption observation + protocol-calibration observations)</item>
    <item>merge recommendation: "Proceed to G (closeout)" | "Open D.fix for 🔴 findings: &lt;cite numbers&gt;" | "Re-tier"</item>
    <item>explicit statement: "Stage M06.V is ready. I will not commit until you approve."</item>
  </approval_surface>
</verifier_stage_prompt>
```

### V.6 Commit Message

```
verify(M06): in-band V run — findings <N🔴 N🟡 N🟢>

Second in-band Stage V run + first under v1.6 protocol. Exercised
M06's six work stages (A: ADR-0009 closure (L1+L2a SDK wire-up); B:
runtime-mcp crate + rmcp 1.7.0 transport; C: client lifecycle; D:
§5a namespace + MCP dispatch through L1+L2a; E: renderer UI; F:
maintainer-inserted production wire — src-tauri injection + live
run-loop interception + gotcha #68 fix, closing M06.D <scope_change>
#1+#2 in-milestone) via four passes.

Per-pass summary:
  Inventory:      <N> files / <N> shape-match / <N> findings
  Wire:           <N> traces (11 named) / <N> findings
  Behavior:       <N> primitives exercised / <N> coverage-gate findings
  Multi-call:     <N> surfaces / <N> findings

ADR-0009 closure status: <satisfied | unsatisfied | deferred via
waiver>. Traces #1 (enforcer.check before provider.invoke) + #2
(narrow before AgentSpawned) endpoint disposition cited in
retrospective.

Findings: see docs/build-prompts/retrospectives/M06.V-retrospective.md

Outcome: <Sound | Sound but rough | Friction-heavy | Not ready>
Merge recommendation: <Proceed to G | Open D.fix #X,#Y | Re-tier>

v1.6 calibration: first V run consuming `<scope_change>` blocks per
the v1.6 STAGE-V-VERIFIER-PROMPT-TEMPLATE.md update. Refinement notes
(if any): see retrospective [END] Decisions.

https://claude.ai/code/session_<id>
```

---

<!-- ============================================================ -->
<!-- Stage G — Phase Closeout (always FINAL, runs AFTER Stage V). -->
<!-- Per CLAUDE.md §20: append-only entry in docs/gap-analysis.md.  -->
<!-- v1.6: <simplify_pass> required child of <deliverables>.       -->
<!-- ============================================================ -->

## Stage G — Phase Closeout: Gap Analysis + Parent-Milestone Summary + Simplify Pass

> Per CLAUDE.md §20. Runs after Stage V (and any D.fix iterations) commits. Produces FOUR artifacts: M06 summary, M06 gap-analysis entry (append-only), the v1.6-required `<simplify_pass>` outcome, M06 PR description draft.

### G.1 Problem Statement

Generate the M06 entry in `docs/gap-analysis.md` per the six-section template, plus `docs/build-prompts/retrospectives/M06-summary.md` per the SUMMARY template, plus run the v1.6 `<simplify_pass>` (`simplify` skill against M06.A..HEAD cumulative diff; approved refactors land as a focused commit on the same branch BEFORE the PR opens; non-approved → `docs/tech-debt.md`), plus the M06 PR description draft. Cumulative review of code-vs-spec across M01-M06. Append-only — never edit prior entries.

The Stage V retrospective's findings feed the closeout: 🟡 findings go into the M06 gap-analysis entry's "Carry-forward" section; 🟢 findings already log to `docs/tech-debt.md` during V. ADR-0009 closure status (from V's Wire pass on traces #1 + #2) is recorded in the gap-analysis entry's "Adherence to spec" section AND in the M06-summary's "Surprises" or "Pattern-level wins" section.

### G.2 Files to Change

| File | Status | Change |
|---|---|---|
| `docs/gap-analysis.md` | exists | **Edited (append-only)** — new M06 entry appended at bottom per the entry template |
| `docs/build-prompts/retrospectives/M06-summary.md` | **new** | M06 milestone summary per SUMMARY-TEMPLATE.md |
| `docs/tech-debt.md` | exists | Append: M06.V 🟢 findings (if any) + Simplify pass non-approved items |
| `CHANGELOG.md` | exists | Edit: `[Unreleased]` notes M06 closeout |

Plus the (optional) focused-refactor commit produced by Simplify pass approval, on the same branch BEFORE PR opens.

### G.3 Detailed Changes

Per CLAUDE.md §20 — the six-section structure is NOT optional:

1. **Codebase deep dive** — cumulative review of code shipped through M06 (200–500 words). Touch on the new runtime-mcp crate + ADR-0009 closure + §5a tool namespace resolution + MCP dispatch through L1+L2a + renderer wiring. Note shape of new module structures, integration with existing M01–M05 surfaces, the M05.V Finding #3 X.2 truth-up outcome.

2. **Adherence to spec** — ✅ / ⚠️ / ❌ with file:line for every M06-touched spec section (§5, §5a, §8.security L1+L2a closure, §3 Visual Design MCPNode, §13.5 audit emissions, §6a `on_capability_violation` reuse for MCP-deny path). Stage V's Wire findings (the 10 traces) populate this section directly. ADR-0009 closure status pinned here — traces #1 + #2 confirmed satisfied (or surfaced as carry-forward if any 🔴 lingered).

3. **Spec review forward-looking** — what spec sections need updating based on M06 implementation reality? At minimum: §5a server name constraints (kebab-case-only? underscore-banned?); §5 health-monitoring cadence default (30s); §5 MCP-server-offline → mcp_missing reuse pattern (current spec implies a separate event variant; M06 reuses).

4. **Fix backlog** — 🔴 Critical / 🟡 Important / 🟢 Nice-to-have items, severity non-elastic per §20. Stage V's findings populate; closeout adds anything V missed (cumulative-review-only items).

5. **Carry-forward from prior milestones** — disposition of every prior milestone's 🟡 + 🟢 items. Each line: `**Mxx 🟡 "Title"** — RESOLVED at M06.<stage> | STAYS DEFERRED to M07+ | RESOLVED via PR #N`. M05.V Findings #1 + #2 (via ADR-0009) get FINAL DISPOSITION here — RESOLVED at M06.A. M05.V Finding #3 (X.2 truth-up) gets FINAL DISPOSITION — RESOLVED at M06.A via the phase-doc edit. M04.V Decision 2 (spec §4a reconcile) — STAYS DEFERRED to M07 unless maintainer approved the spec edit in M06.A. TD-001..NNN forward-tracked in tech-debt.md.

6. **Sign-off** — Stage G commit hash + DCO sign-off + AI assistance disclosure.

Plus the `<gotchas_graduation>` subsection — audit per-stage `<gotchas>` across A–E, dispositions: kept / graduated / resolved / expired.

Plus the **Simplify pass outcome** subsection — what the `simplify` skill flagged, what the maintainer approved, what landed as the focused-refactor commit (commit hash), what deferred to tech-debt.

### G.4 Tests

N/A — Stage G ships documentation only. Validation:
- `gap-analysis.md` append-only CI check still passes (no prior entries edited)
- `M06-summary.md` follows SUMMARY-TEMPLATE.md shape
- (If Simplify pass produces a refactor commit) all M06 gates re-run after the commit and stay green

### G.5 CLI Prompt

```xml
<closeout_stage_prompt id="M06.G">
  <context>
    Phase Closeout for M06. FINAL stage. Runs after Stage V commits.
    Produces FOUR artifacts: the cumulative gap-analysis entry
    (append-only) + the M06 parent-milestone summary + the v1.6-required
    Simplify pass outcome + the M06 PR description draft. Per CLAUDE.md
    §20 + STAGE-PROMPT-PROTOCOL.md §8 (v1.6 simplify_pass). The
    gap-analysis entry's commit is the FINAL commit on the milestone
    branch claude/m06-mcp-basic and gates the milestone PR push, UNLESS
    the Simplify pass produces a focused-refactor commit — in which case
    that's the final commit and the gap-analysis lands one commit prior.
  </context>

  <cumulative_reads>
    <codebase>entire shipped codebase through M06 (cumulative across M01-M06 merges)</codebase>
    <spec>agent-runtime-spec.md (end-to-end, focus on §5 + §5a + §8.security L1+L2a + §3 + §13.5)</spec>
    <gap_analysis>docs/gap-analysis.md (ALL prior entries — M01, M02, M03, M03.5, M04, M05)</gap_analysis>
    <retrospectives>docs/build-prompts/retrospectives/M06.*-retrospective.md (all of A, B, C, D, E, V — closeout reads these; verifier did NOT)</retrospectives>
    <summary>docs/build-prompts/retrospectives/M06-summary.md (authored as part of this stage)</summary>
    <tech_debt>docs/tech-debt.md (cumulative TD-NNN entries — M06.V's 🟢 findings should be present from V's run)</tech_debt>
  </cumulative_reads>

  <read_first>
    <file>CLAUDE.md (especially §20 Gap Analysis Protocol)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (especially §8 Closeout-only tags + v1.6 simplify_pass)</file>
    <file>docs/build-prompts/M06-mcp-basic.md (this entire phase doc)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md</file>
    <file>docs/gap-analysis.md (the six-section template defined at top of file)</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (the 🟢 ledger convention per v1.5+)</file>
  </read_first>

  <scope_locks ref="docs/build-prompts/M06-mcp-basic.md" section="Key constraints"/>

  <gates milestone="M06"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>For M06 specifically: the milestone introduced ONE new safety primitive (runtime-mcp crate) + closed the M05.V ADR-0009 carry-forward (L1+L2a SDK wire-up) + added 5 new event variants + extended the renderer + ran the FIRST V-pass under v1.6 protocol. Aggregate per-primitive coverage outcomes in the summary. Note the protocol-level observation: M06 is the first milestone with `<simplify_pass>` at closeout — did the pass produce actionable refactor proposals, or surface that the cumulative diff was already in clean shape? Cite the simplify outcome.</special_log>
  </retrospective_requirements>

  <deliverables>
    <milestone_summary>docs/build-prompts/retrospectives/M06-summary.md (per SUMMARY-TEMPLATE.md; aggregates per-stage retros + V results + scores axes across stages + marks verdict)</milestone_summary>
    <gap_analysis_entry>docs/gap-analysis.md (append new M06 entry; six required sections; gotchas_graduation subsection for stages A–E)</gap_analysis_entry>
    <simplify_pass>
      <invoke skill="simplify" against="milestone cumulative diff (M06.A..HEAD)"/>
      <surface kind="refactor_proposals" examples="duplication / dead code / parallel API surfaces / modules grown across stages / premature abstractions"/>
      <approval_required>true</approval_required>
      <commit_on_approval>focused refactor commit on same branch before PR opens</commit_on_approval>
      <defer_unapproved_to>docs/tech-debt.md (per ADR-0008 🟢 ledger)</defer_unapproved_to>
    </simplify_pass>
    <pr_description>draft only; do not open PR until explicitly asked</pr_description>
  </deliverables>

  <gap_analysis_requirements ref="CLAUDE.md" section="20. Gap Analysis Protocol">
    <gotchas_graduation>
      <stage_review id="A"/>
      <stage_review id="B"/>
      <stage_review id="C"/>
      <stage_review id="D"/>
      <stage_review id="E"/>
      <!-- Stage V's special_log observations also feed graduation decisions -->
    </gotchas_graduation>
    <special_check>Verify the V→closeout handoff: 🟡 findings from V's retro carry into the gap-analysis Carry-forward section; 🟢 findings already in tech-debt.md; 🔴 findings (if any) were resolved by D.fix iter OR new waiver-as-ADR before closeout. ADR-0009 closure status (Stage V Wire traces #1 + #2) explicitly cited in Adherence to spec section.</special_check>
    <special_check>Run the v1.6 `<simplify_pass>` skill against M06.A..HEAD cumulative diff. Surface refactor proposals. Apply maintainer-approved subset as focused refactor commit BEFORE PR opens. Defer unapproved to docs/tech-debt.md.</special_check>
  </gap_analysis_requirements>

  <append_only_verification>
    <local_check>prior content of docs/gap-analysis.md must be a literal prefix of HEAD before commit</local_check>
    <ci_check name="gap-analysis-append-only">fails if any prior line is modified</ci_check>
  </append_only_verification>

  <three_artifact_review>
    <artifact>code diff (cumulative across M06 stages A–E + V findings absorbed + any Simplify-pass refactor commit)</artifact>
    <artifact>per-stage retrospectives (M06.A through M06.E) + Stage V retro + M06 summary</artifact>
    <artifact>new gap-analysis entry — flagged "IMMUTABLE once committed"</artifact>
    <pushback_blocks_pr>true</pushback_blocks_pr>
  </three_artifact_review>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M06-mcp-basic.md" section="G.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (build machine git log + retro listing including M06.V + M06-summary.md)</item>
    <item>diff stat (gap-analysis.md additions + M06-summary.md + CHANGELOG.md edit + any Simplify-pass refactor commit)</item>
    <item>three-artifact review: code diff cumulative + per-stage retros + new gap-analysis entry</item>
    <item>Simplify pass outcome: refactor proposals surfaced + maintainer-approved subset + focused refactor commit (if any) + non-approved deferred to tech-debt.md</item>
    <item>ADR-0009 closure final disposition (cited from V's Wire pass)</item>
    <item>PR description draft for the M06 milestone PR (do NOT open yet — surface only)</item>
    <item>explicit statement: "Stage M06.G is ready. I will not commit until you approve."</item>
  </approval_surface>
</closeout_stage_prompt>
```

### G.6 Commit Message

```
docs(closeout): M06 — gap-analysis entry + parent-milestone summary + Simplify pass

Append-only M06 entry to docs/gap-analysis.md (cumulative product↔spec
audit across M01-M06). Six sections + gotchas_graduation across stages
A-E. Per CLAUDE.md §20.

M06 summary (docs/build-prompts/retrospectives/M06-summary.md) aggregates
per-stage retros + Stage V findings + verifier-axes scores. First
milestone shipped under v1.6 protocol; first to consume `<scope_change>`
blocks at Stage V; first to run `<simplify_pass>` at Stage G.

ADR-0009 closure: <SATISFIED | UNSATISFIED — see waiver NNNN>. Stage V
Wire pass traces #1 (enforcer.check before provider.invoke) + #2 (narrow
before AgentSpawned) endpoint disposition documented in the
"Adherence to spec" section of the gap-analysis entry.

Stage V findings carry-forward dispositions:
- 🟡 findings → gap-analysis Carry-forward section (next milestone Stage A absorbs)
- 🟢 findings → docs/tech-debt.md (logged during V)
- 🔴 findings (if any) → resolved by D.fix iter OR new waiver-as-ADR before this commit

Simplify pass outcome:
- Refactor proposals surfaced: <N>
- Maintainer-approved subset: <N> applied as commit <hash> on same branch
- Non-approved deferred to docs/tech-debt.md: <N>

This commit is the FINAL on claude/m06-mcp-basic (modulo Simplify pass
refactor commit if one was approved); gates the M06 PR push.

https://claude.ai/code/session_<id>
```

---
