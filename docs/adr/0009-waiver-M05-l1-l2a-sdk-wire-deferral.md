# ADR-0009: Waiver — M05.V Findings #1 + #2 (L1 + L2a SDK wire deferral to M06)

**Status:** Proposed
**Date:** 2026-05-13
**Deciders:** @kknipe2k (maintainer)
**Tags:** waiver, capability, security, sdk, scope, m05, m06

## Context

M05 Stage V's Wire pass (per ADR-0008's four-pass protocol) surfaced two 🔴 findings against the M05.B deliverables, both recorded in `docs/build-prompts/retrospectives/M05.V-retrospective.md`:

- **Finding #1** — L1 capability enforcer never invoked from production SDK. `grep -rn '.check(' crates/runtime-main/src/ --include='*.rs'` returns 14 matches, all inside `crates/runtime-main/src/capability/enforcer.rs` `#[cfg(test)] mod tests` (lines 306–519). Zero production call sites. `crates/runtime-main/src/sdk/event_pipeline.rs:57–66` translates `ProviderEvent::ToolUse` → `AgentEvent::ToolInvoked` without an enforcer check.
- **Finding #2** — L2a `narrow()` never invoked from sub-agent spawn path. `grep -rn 'narrow' crates/runtime-main/src/ --include='*.rs'` returns matches only inside `crates/runtime-main/src/capability/narrowing.rs` (unit tests + proptest). `AgentEvent::AgentSpawned` is emitted unconditionally at `crates/runtime-main/src/sdk/agent_sdk.rs:124` with no narrowing call.

Both findings are the same architectural class: the capability primitive ships complete (100% line + region coverage, 33 unit tests + proptest + 6 integration tests, schema-typed and round-trip-stable) but is not wired into a production call site. Per ADR-0008, 🔴 findings block merge unless the build agent files a waiver-as-ADR documenting an interpretation dispute.

The build agent disputes the "block merge" interpretation on architectural grounds:

1. **v0.1 SDK is streaming-only per M02** (PR #42 + #45). `crates/runtime-main/src/sdk/` ships `AgentSdk` generic over `LLMProvider`; `run_agent_with_provider_stream` consumes a single SSE stream from the provider and translates `ProviderEvent` variants to `AgentEvent` variants. **There is no synchronous tool-dispatch surface in v0.1.** Anthropic's Messages API dispatches tools server-side — the SDK observes `ProviderEvent::ToolUse` as a *report* that a tool was invoked, not as a *request* the runtime gets to gate.
2. **No sub-agent spawn loop exists in v0.1.** `AgentSpawned` is emitted from the framework-loader walk-and-instantiate path, not from a runtime spawn driver. The multi-turn agent loop (plan_loop driver) is M07-scope per `docs/MVP-v0.1.md`.
3. **The M05 phase doc's own `<execution_warnings>` at Stage B authorizes the scope reduction**: *"keep the smoke test focused on the enforcer's check logic, not full end-to-end"* (`docs/build-prompts/M05-gap-capability.md:924`). The phase doc anticipated that v0.1's SDK shape would not support end-to-end wire-up at M05 and explicitly bounded the deliverable to the primitive.
4. **The M05.B retrospective's Decision D1** (`docs/build-prompts/retrospectives/M05.B-retrospective.md:194`) surfaced the structural finding before any code landed and resolved it as a scope reduction citing the `<execution_warnings>` block: *"The enforcer ships complete-as-primitive; M06+ wires it to the live dispatch path when multi-turn tool loops land."* Maintainer review of the M05.B surface did not intercept this decision.

The M05.V finding is therefore not "drift the build agent missed" — it is "intentional descope the build agent surfaced for review, surface-approved at Stage B, that V's fresh-context read deliberately could not see." Per ADR-0008's *Negative consequence* note: *"Bug classes V may still miss (until the protocol iterates): … intentional descopes documented only in retros V can't read."* M05.V's own Decision 3 (protocol refinement) calls this out and proposes a v1.6 `<scope_change>` slot on the work-stage prompt schema to surface intentional descopes into the V agent's read-list.

Findings #1 and #2 are linked — same architectural class, same root cause (no synchronous dispatch surface), same fix (wire when the dispatch path lands). M05.V Decision 1 recommends a **single** waiver covering both. This ADR is that waiver.

## Decision

**We waive M05.V Findings #1 and #2 as intentional v0.1 scope deferrals to M06.**

The L1 capability enforcer and the L2a narrowing primitive ship in M05 as complete, schema-typed, fully tested safety primitives (100% line + region coverage on `crates/runtime-main/src/capability/{declaration,enforcer,narrowing}.rs`). Their integration into the production tool-dispatch and sub-agent-spawn call paths is deferred to M06 Stage A, when the multi-turn agent loop (`plan_loop` driver per `docs/MVP-v0.1.md` §M6) introduces the first synchronous dispatch surface that the enforcer can gate.

Concretely:

- **No D.fix iteration runs** for Findings #1 + #2 against the M05 branch. The waiver closes both findings as documented architectural deferral.
- **M06 Stage A's phase doc carries forward** explicit deliverables for the wire-up: `enforcer.check(agent_id, &needed)` before `provider.invoke` in the dispatch path, and `narrow(parent_grants, proposed_child_grants)` before `AgentSpawned` emission in the spawn path. The wire-up tests planned in M05.B (e.g., `tool_call_with_grant_succeeds_and_emits_capability_grant`) become the M06 acceptance criteria.
- **The smoke test `crates/runtime-main/tests/capability_enforcer_smoke.rs`** stands as the canonical "what the SDK will call" surface until M06 wires the real call site. The smoke test's header comment already records this rationale.
- **The 🟡 finding #3** (phase-doc-vs-implementation file drift on `crates/runtime-sandbox/src/ipc.rs` + `crates/runtime-main/src/tier/transition.rs`) is unaffected by this waiver and carries forward via the M05 closeout gap-analysis Carry-forward section per M05.V Decision 2.

## Consequences

### Positive

- **M05 merges on schedule** with the L1 + L2a primitives shipped complete. The capability surface is testable, reviewable, and ready to wire — only the wire is deferred.
- **Aligns the M05 deliverable with v0.1's locked scope** per `docs/MVP-v0.1.md` §M5 (gap detection + capability primitives) vs §M6 (multi-turn tool loop). Forcing the wire-up into M05 would require pulling forward M06 scope, which would either bloat M05 or short-circuit the plan_loop design work.
- **Validates ADR-0008's waiver-as-ADR mechanism on its first real-world test.** The mechanism exists precisely for this case: V flags 🔴, build agent disputes on architectural grounds, maintainer adjudicates via the ADR review surface. No new artifact class needed.
- **The carry-forward to M06 Stage A is concrete and small** — wire two call sites, port the existing M05 smoke-test assertions into real call-path integration tests. Estimated 2–3h per M05.V Decision 1's D.fix-iter-1 alternative scope.

### Negative

- **The 🔴 finding pattern is genuinely the M04-class "primitive ships green, production path missing" bug.** Waiving it on architectural grounds is correct for this specific case, but the pattern argues for protocol vigilance — a future waiver under thinner reasoning would erode V's blocking power.
- **The L1 + L2a primitives are unexercised end-to-end until M06 lands.** A subtle bug in the integration boundary (e.g., agent-id passing, grant-map construction, narrowing-direction symmetry under real provider events) would not surface until M06's first test run. The primitive-level proptest + unit tests reduce but do not eliminate the risk.
- **M05's `capability_violation` + `capability_grant` event variants are renderer-side reachable** (the graphStore branches exist per M05.B deliverable 4, and the GapPanel + CapabilityBadge wire them per M05.F) but are never emitted in v0.1 production runs. Stage F tests inject the events synthetically; no real provider session produces them until M06.
- **Adds an ADR artifact to the milestone** (one more file in `docs/adr/`). Minor process cost.

### Neutral / future implications

- **M06 Stage A's read-first must include this waiver ADR**, the M05.V retrospective (`M05.V-retrospective.md`), and the M05.B Decision D1 (`M05.B-retrospective.md:194`). The M06 phase doc's `<read_prior_milestones>` slot should reference all three.
- **The v1.6 protocol iteration's `<scope_change>` slot** (per M05.V Decision 3) would have surfaced this intentional descope into the V agent's read-list, avoiding the waiver round-trip. The waiver path remains the correct fallback when descope decisions land mid-stage; the protocol slot covers the planned-descope case. Both belong in v1.6.
- **The M04.V + M05.V combined record establishes the waiver-as-ADR pattern as the "interpretation dispute" lane** distinct from D.fix (the "drift correction" lane). Future verifier runs should default to D.fix unless the build agent can name (a) the prior stage where the decision was surfaced, (b) the phase-doc scope-warning that authorized it, and (c) the concrete next-milestone deliverable that closes the loop. This waiver satisfies all three.
- **If M06 Stage A does not wire the enforcer**, that is itself a 🔴 finding under M06.V's Wire pass — the carry-forward becomes the verifier's expected trace endpoint.

## Alternatives Considered

### Alternative A: D.fix iter 1 — wire the enforcer in M05

Add the dispatch wrap to `crates/runtime-main/src/sdk/agent_sdk.rs::run_agent_with_provider_stream` so that processing a `ProviderEvent::ToolUse` first calls `enforcer.check(agent_id, &translate_tool_to_capability(name))`; emit `capability_grant` / `capability_violation` per outcome. Wire `CapabilityEnforcer` into `AgentSdk` construction with grant population from `framework_loader`. Estimated 1–2h per M05.V Finding #1's Action option 1.

**Rejected because:** the wrap point is semantically wrong for v0.1's SDK shape. `ProviderEvent::ToolUse` is a *report* from Anthropic's server-side dispatch — the runtime cannot gate the dispatch (it already happened server-side); it can only observe the outcome and emit a post-hoc `capability_violation` if the agent should not have had access. That post-hoc emission is honest defensive observability, but it is not L1 enforcement in the spec §8.security sense (which requires *blocking* the dispatch). Implementing it would create a misleading test surface that passes a "check before dispatch" assertion against code that actually checks after. The honest semantics arrive with M06's multi-turn loop, which the runtime drives directly.

For Finding #2 specifically: v0.1 has no sub-agent spawn loop. `AgentSpawned` at `agent_sdk.rs:124` is emitted from the framework-loader walk, not from a spawn driver. There is no parent grant + proposed child grant pair to narrow at the v0.1 emission site. Wiring would require fabricating a no-op narrowing pass, which would pass the wire-trace test but assert no useful invariant.

### Alternative B: Two separate waivers (one per finding)

File `docs/adr/0009-waiver-M05-finding-1.md` for L1 and `docs/adr/0010-waiver-M05-finding-2.md` for L2a, per ADR-0008's `docs/adr/NNNN-waiver-M[NN]-finding-N.md` naming pattern.

**Rejected because:** both findings share a single root cause (no synchronous dispatch surface in v0.1), a single resolution (M06 Stage A wires both), and a single phase-doc authorization (Stage B `<execution_warnings>`). Splitting into two waivers duplicates the reasoning and fragments the M06 carry-forward. The single combined waiver matches M05.V Decision 1's explicit recommendation: *"Recommended path: build agent files one waiver ADR (`docs/adr/NNNN-waiver-M05-l1-l2a-sdk-wire-deferral.md`) covering both findings together."* The filename slug reflects the combined scope.

### Alternative C: Reject the waiver; pull M06 scope forward into M05

Declare the L1 + L2a wire-up an M05-blocking requirement and pull the M06 plan_loop driver work forward to make the wire-up testable end-to-end at M05.

**Rejected because:** §0d Release Scope Matrix locks v0.1 milestone boundaries. M06 (multi-turn tool loop + plan_loop driver) is a distinct scope unit with its own ADR-class design decisions still ahead (provider-level vs runtime-level tool dispatch routing, retry semantics, partial-failure handling). Pulling it into M05 would either compress that design work or expand M05 indefinitely. The §0d boundary exists to prevent exactly this drift.

## Related

- **ADR-0008** — *Milestone Stage V Verifier* — defines the waiver-as-ADR mechanism this waiver invokes. This waiver is the first invocation; ADR-0008 §"Waiver path" + §"Neutral / future implications" anticipate this exact case.
- **Spec sections:** §8.security L1 (capability enforcer) + L2a (sub-agent narrowing) — the contract the M05 primitives satisfy; the wire-up to production call sites is the deferred portion.
- **Phase doc:** `docs/build-prompts/M05-gap-capability.md` Stage B (`<execution_warnings>` at line 924; `<retrospective_requirements>` mentioning L3 boundary contract at line 929) — authorizes the scope reduction.
- **Stage B retrospective:** `docs/build-prompts/retrospectives/M05.B-retrospective.md` Decision D1 (line 194) — surfaces the structural finding before code landed.
- **Stage V retrospective:** `docs/build-prompts/retrospectives/M05.V-retrospective.md` Findings #1 + #2 + Decision 1 — the verifier output this waiver answers.
- **Carry-forward target:** M06 Stage A phase doc (forthcoming) — explicit wire-up deliverables for `sdk/agent_sdk.rs::run_agent_with_provider_stream` + the spawn-narrowing call site.
- **External references:** Anthropic Messages API tool-use semantics (server-side dispatch; SSE events as post-dispatch reports) — the architectural fact that drives the v0.1 deferral.

## Notes

This is the first waiver-as-ADR filed under ADR-0008's mechanism. Its adjudication establishes the precedent for how future waivers are reviewed:

- **The build agent's burden** is to name the (a) prior surface where the descope was raised, (b) phase-doc warning that authorized it, and (c) concrete next-milestone deliverable that closes the loop. All three are cited above.
- **The maintainer's burden** is to confirm the architectural reasoning holds — specifically, that the v0.1 SDK genuinely lacks the dispatch surface (not that the build agent missed it). The grep evidence in M05.V Finding #1 + #2 + the `event_pipeline.rs:57–66` code structure are the verifiable artifacts.
- **The protocol's burden** is to learn from each waiver: M05.V Decision 3's v1.6 `<scope_change>` slot is the protocol-level response to this case. The waiver is the right tool when descope decisions land mid-stage; the protocol slot is the right tool when they are planned at phase-doc authoring time.

Calibration: M05.V's bias-guard worked exactly as ADR-0008 designed — the fresh-context verifier surfaced the wire-incomplete finding the build agent's surface had retro-time visibility into but the maintainer review at M05.B surface time did not intercept. The waiver lane lets the architectural truth (the descope is correct for v0.1) override the literal contract-trace failure, with the M06 carry-forward as the structural assurance that the loop closes.
