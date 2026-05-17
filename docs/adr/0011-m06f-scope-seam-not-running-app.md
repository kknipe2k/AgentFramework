# ADR-0011: M06 Stage F scope — SDK run-loop interception + src-tauri injection seam (mock-verified), NOT a live running-app end-to-end wire

**Status:** Accepted
**Date:** 2026-05-15
**Deciders:** @kknipe2k (maintainer)
**Tags:** scope, mcp, capability, sdk, m06, m07

## Context

Stage F was maintainer-inserted post-M06.E to close the ADR-0010
composition-root MCP-dispatch wire in-milestone (the headline M06
deliverable) rather than carry it to M07. The Stage F prompt (F.5) and
F.1 Problem Statement frame the mandate as: "MCP tool calls dispatching
end-to-end through the L1 capability gate **in the running app**" — and
the Stage V section (V.1, V.2 Wire row, V.3 trace #11) declares
M06.D `<scope_change>` #1+#2 "CLOSED by F, so this is trace #11 expected
DELIVERED, 🔴 if missing/regressed."

Reading the actual code against that mandate before the Stage F red
phase (per the ADR-0010 precedent — surface architectural/scope
contradictions before sinking TDD effort) surfaced four facts that
make the literal "live in the running app end-to-end" framing
unsatisfiable within Stage F's own scope locks:

1. **No `impl ConnectionResolver for McpClient` exists.** The M06.D
   retrospective special-log + `crates/runtime-mcp/src/dispatch.rs:18-21`
   doc-comment both state "`McpClient` impls it for production; tests
   inject a mock." Grep is decisive:
   `grep -rn "ConnectionResolver" crates/runtime-mcp/src/` returns only
   the trait definition (`dispatch.rs:47`), its struct field/ctor uses
   (`dispatch.rs:95,108,181`), the `lib.rs:41` re-export, and the
   module doc — **zero `impl ConnectionResolver for McpClient`**. The
   concrete `McpDispatcher` is therefore not constructible in
   `src-tauri` today: `McpDispatcher::new` requires
   `Arc<dyn ConnectionResolver>` and no production impl exists. The
   M06.D claim is grep-verified false (a M06.D documentation drift, not
   a Stage F regression).
2. **Stage F is forbidden from adding it.** F's `<execution_warnings>`
   #1: "Do NOT modify runtime-mcp's dispatcher or namespace logic — D
   shipped + tested it. F only wires it." F.1: "Not in this stage:
   anything new in `runtime-mcp` (D shipped the dispatcher)." Writing
   `impl ConnectionResolver for McpClient` is new runtime-mcp code,
   explicitly out of F scope.
3. **No `CapabilityEnforcer` / `NamespaceResolver` is constructed in
   the shell.** `grep -rn "CapabilityEnforcer\|NamespaceResolver" src-tauri/src/`
   returns zero hits. `McpDispatcher::new`'s other arguments
   (`Arc<RwLock<NamespaceResolver>>`, `Arc<CapabilityEnforcer>`) have
   no construction site in `src-tauri` to wire from.
4. **The only `AgentSdk` construction in the shell is the no-tools
   smoke path.** `src-tauri/src/commands.rs:149` constructs
   `AgentSdk::new(...)` (no capability wiring) for `run_smoke_session`,
   which runs a fixed "Say only the word: hello" prompt with
   `tools: vec![]`. It never emits `ProviderEvent::ToolUse`. The real
   agent-with-tools loop is M07 — a pre-existing scope lock recorded in
   ADR-0009 ("the multi-turn agent loop … is M07-scope"), the M06
   Stage A `<scope_change>` ("M07 multi-turn loop … stays M07"), and
   restated in F.1 itself ("Not in this stage: M07 multi-turn loop").

The contradiction is the same architectural class as the M06.D Cargo
cycle resolved by ADR-0010: a maintainer-inserted phase-doc mandate
whose literal wording over-reaches the code reality + the milestone's
own scope locks. Per `CLAUDE.md` §12 ("Ask first: any spec ambiguity
or contradiction") and the ADR-0010 precedent (surfaced via
`AskUserQuestion` before the red phase), this was surfaced to the
maintainer before the Stage F red phase. The maintainer selected the
"SDK seam + injection seam" option and directed this ADR + a coupled
phase-doc forward-correction be surfaced together for review before F
red begins.

If this is not decided, Stage F would either (a) silently re-create
the gotcha #66 / ADR-0009-recurrence pattern (ship a seam the running
app never reaches and call it "closed"), or (b) overrun scope by
pulling M07's agent loop + forbidden runtime-mcp glue into M06.

## Decision

**We scope M06 Stage F to the SDK run-loop interception seam plus the
src-tauri composition-root injection seam, both mock-verified per the
ADR-0010 / `Arc<dyn _>` shell-injected-seam archetype. The concrete
`McpDispatcher` construction, the `impl ConnectionResolver for
McpClient` production glue, and the live end-to-end exercise are an
explicit M07 carry-forward — not a Stage F miss and not an M06.V 🔴.**

Concretely, Stage F delivers:

- **runtime-main (the headline architectural wire, fully delivered +
  tested).** `AgentSdk<P>` gains an
  `Option<Arc<dyn McpToolDispatch>>` field + a `with_mcp_dispatch`
  builder seam. `run_agent_with_provider_stream` intercepts
  `ProviderEvent::ToolUse` before the existing Stage A pipeline call:
  `dispatch_if_mcp` returns `None` → the event falls through to Stage
  A's non-MCP L1 path **unchanged** (asserted no-regression);
  `Some(Ok(Invoked))` → the run loop emits **agent_id-correct**
  `ToolInvoked` + `ToolResult` directly with the `agent_id` it holds
  (the gotcha #68 fix — `apply_mcp_dispatch` + `McpDispatchOutcome`
  are left untouched so the D-frozen `mcp_dispatch_wire.rs` integration
  test stays intact); `Some(Ok(Blocked))` → `apply_mcp_dispatch` events
  + the existing `on_capability_violation` HITL trigger (ADR-0007, no
  new seam); `Some(Err)` → `mcp_dispatch_error_event`. Verified by
  `crates/runtime-main/tests/mcp_dispatch_runloop.rs` against a **mock**
  `Arc<dyn McpToolDispatch>` (agent_id-correct, non-MCP fall-through,
  blocked→HITL, twice-in-sequence per gotcha #69).
- **src-tauri (the injection seam, mock-verified holdout pattern).**
  The `*_with` seam (`run_smoke_session_with`) accepts an
  `Option<Arc<dyn McpToolDispatch>>` and applies `.with_mcp_dispatch`
  when present — unit-tested with a mock dispatch. The production
  `run_smoke_session` wrapper passes `None` for now (the concrete
  `McpDispatcher` is not constructible per Context #1-#3); this is the
  same OS-call-holdout pattern as `providers/anthropic.rs` /
  `key_store.rs` / `open_mcp_client` (CLAUDE.md §5 — the seam gets the
  unit test, the wrapper is the excluded holdout).

Stage F does **not** deliver: `impl ConnectionResolver for McpClient`
(forbidden runtime-mcp work — Context #2); a concrete `McpDispatcher`
constructed in `src-tauri/src/main.rs` (blocked by Context #1-#3); the
agent-with-tools loop that would exercise the wire in a real session
(M07 — Context #4).

The ADR-0010 archetype is the precedent for "delivered + verified":
every `Arc<dyn _>` shell-injected seam in this codebase
(`Arc<dyn Connection>`, `Arc<dyn SecretStore>`, `Arc<dyn Transport>`,
`Arc<AuditWriter>`) is verified via a mock at the seam, with the
concrete OS-call construction as the excluded holdout. The MCP-dispatch
seam is verified the same way. "Delivered" means the seam exists, the
run loop reaches it, agent_id is correct, and the injection point is
tested — exactly what the mock-verified tests prove.

M06.D `<scope_change>` #1+#2 are therefore **CLOSED at the
seam + injection-seam level** (the SDK *can* reach an injected
dispatcher; agent_id is correct; the composition-root injection point
exists and is tested). The residual — concrete-dispatcher construction
+ ConnectionResolver-for-McpClient glue + live exercise — is an
explicit, named M07 carry-forward.

## Consequences

### Positive

- **The headline architectural wire ships in M06, fully tested.** The
  SDK run-loop interception (the part that was genuinely missing and is
  buildable) lands with agent_id-correct events, non-MCP no-regression,
  blocked→HITL reuse, and the multi-call invariant — the substantive
  gotcha #68 + ADR-0010 closure.
- **Honest "closed."** The M06.V trace #11 endpoint is the seam + the
  injection seam (grep-findable + mock-tested), consistent with how
  every other ADR-0010-class seam is verified. No misleading
  "running-app end-to-end" assertion against code the running app can't
  reach.
- **No scope creep, no forbidden edits.** runtime-mcp is untouched
  (honors F `<execution_warnings>` #1). M07's agent loop is not pulled
  forward (honors the Stage A `<scope_change>` + ADR-0009).
- **Surfaces a real M06.D documentation drift** (the false "McpClient
  impls ConnectionResolver" claim) with grep evidence, so M07's
  carry-forward starts from accurate state — the same value ADR-0010
  delivered for the Cargo-cycle drift.
- **Re-uses an established adjudication lane.** This is the ADR-0010
  pattern (phase-doc over-reach reconciled via ADR + phase-doc
  forward-correction before red), not a new mechanism.

### Negative

- **MCP dispatch is not exercisable in the running v0.1 app until
  M07.** A subtle bug at the concrete-dispatcher construction boundary
  (NamespaceResolver population from connected servers, enforcer grant
  keying, ConnectionResolver-for-McpClient adapter) will not surface
  until M07's first real-session run. The mock-verified seam tests +
  the D-frozen `mcp_dispatch_integration.rs` (concrete dispatcher vs
  mock transport + real enforcer) reduce but do not eliminate this —
  the seam↔concrete boundary is the unexercised gap.
- **The "running app end-to-end" phrasing in F.1/F.5 was wrong on
  arrival.** Stage F was inserted specifically to avoid the
  gotcha #66 pattern; this ADR concludes a chunk of that mandate was
  itself unsatisfiable in-milestone given the pre-existing M07 lock.
  The pattern argues for phase-doc-authoring vigilance (a
  `<dependency_cycle_check>`-class slot that also verifies
  construction-graph reachability, per the M06.D carry-forward).
- **Adds an ADR artifact + a phase-doc forward-correction to the
  milestone.** Minor process cost; the ADR-0010 precedent already
  established this is the right cost to pay for a surfaced
  contradiction.

### Neutral / future implications

- **M07's phase doc inherits a concrete, small carry-forward:**
  (a) `impl ConnectionResolver for McpClient`; (b) construct
  `CapabilityEnforcer` + `NamespaceResolver` (populated from connected
  servers) in the shell; (c) build the concrete `McpDispatcher` in
  `src-tauri` and pass `Some(dispatch)` through `run_smoke_session_with`
  (or its M07 agent-loop successor); (d) the first agent-with-tools
  loop that emits a real `ProviderEvent::ToolUse` an MCP server
  resolves. The Stage F seam tests become M07's wire-trace endpoints —
  same structure as ADR-0009's M05→M06 carry-forward.
- **M06.V Wire trace #11 is split** (see the coupled phase-doc
  forward-correction): the seam + injection-seam portion is
  DELIVERED/mock-verified (🔴 if *that* is missing or regressed);
  the concrete-dispatcher + live-exercise portion is this ADR's M07
  carry-forward and is **NOT** 🔴 at M06.V. V reads this ADR + the
  forward-corrected F.1/V.* (per the v1.6 `<scope_change>`-into-V
  read-list mechanism) and adjusts the trace #11 expectation
  accordingly — exactly the ADR-0009 "if M06 does not wire it, that's
  the verifier's expected trace endpoint" mechanism, one milestone
  later.
- **If M07 does not wire the concrete dispatcher + agent-loop
  exercise, that is itself a 🔴 under M07's Wire pass** — the
  carry-forward becomes the next verifier's expected endpoint
  (mirrors ADR-0009's closing clause).

## Alternatives Considered

### Alternative A: Include the runtime-mcp `ConnectionResolver` adapter + a concrete `McpDispatcher` in `main.rs`

Add `impl ConnectionResolver for McpClient` in runtime-mcp (closing
the false M06.D claim) + a `build_mcp_dispatch` holdout in
`src-tauri/src/main.rs` constructing the concrete dispatcher from the
managed `McpClient` + a shell-constructed `CapabilityEnforcer` /
`NamespaceResolver`.

**Rejected because:** it overrides F's explicit `<execution_warnings>`
#1 + F.1 "nothing new in runtime-mcp," and still cannot be *exercised*
by the no-tools smoke path until M07 adds the agent-with-tools loop —
so it adds forbidden surface + an unexercised construction path for no
in-milestone observable benefit. The construction is better placed in
M07 next to the loop that actually drives it, where it can be
integration-tested end-to-end rather than constructed-but-dead.

### Alternative B: Minimal — SDK seam only, no src-tauri `*_with` injection seam

Deliver only the runtime-main run-loop interception + tests; do not
touch `src-tauri` at all.

**Rejected because:** it leaves the composition-root injection point
unbuilt, so M06.V trace #11 has no src-tauri injection endpoint to
grep — re-creating the "seam exists but nothing injects it" half of
the gotcha #66 pattern Stage F exists to kill. The `*_with` injection
seam (mock-tested) is the honest, in-scope composition-root endpoint;
omitting it under-delivers the closeable part of the wire.

### Alternative C: Full — switch the smoke/production path to a capability-wired, tool-emitting agent loop

Make MCP dispatches truly end-to-end live now by replacing the fixed
"hello" smoke path with a capability-wired agent-with-tools loop.

**Rejected because:** that *is* M07. It contradicts the Stage A
`<scope_change>` (multi-turn loop stays M07), ADR-0009 (the v0.1 SDK
has no synchronous tool-dispatch surface; the agent loop is M07-scope),
and F.1's own "Not in this stage: M07 multi-turn loop." §0d locks the
milestone boundary; pulling M07 forward bloats M06 and compresses the
plan_loop design work ADR-0009 already protected.

## Related

- **ADR-0010** — *MCP dispatch via dependency inversion* — defines the
  `McpToolDispatch` seam + the shell-injected-seam archetype this ADR
  scopes Stage F's delivery of. Same lineage: phase-doc over-reach
  surfaced before red, reconciled via ADR + phase-doc
  forward-correction.
- **ADR-0009** — *Waiver — M05.V Findings #1+#2 (L1+L2a SDK wire
  deferral)* — establishes (a) the v0.1 SDK has no synchronous
  tool-dispatch surface and the agent loop is M07-scope (Context #4),
  and (b) the "if the next milestone doesn't wire it, that's the
  verifier's expected endpoint" carry-forward mechanism this ADR
  re-uses one milestone later.
- **ADR-0007** — *In-process HITL seam architecture* — the
  `on_capability_violation` trigger the Stage F Blocked path reuses
  (no new seam).
- **ADR-0008** — *Milestone Stage V Verifier* — the four-pass
  protocol + the v1.6 `<scope_change>`-into-V-read-list mechanism that
  lets V read this ADR + the forward-corrected F.1/V.* and split
  trace #11 rather than emit a blanket 🔴.
- **Spec sections:** §5 (MCP Manager), §5a (Tool Namespace
  Resolution), §8.security L1 (the gate the seam routes through),
  §0d (the v0.1/M07 scope boundary this ADR honors).
- **Phase doc:** `docs/build-prompts/M06-mcp-basic.md` — F.1 Problem
  Statement + §V one-liner + V.1 + V.2 Wire row + V.3 trace #11 + the
  M06.D `<scope_change>` `carry_forward_to` + **F.5 `<context>`/
  `<read_first>` + F.6 commit message** (the coupled forward-correction
  filed with this ADR). F.5/F.6 ARE corrected because Stage F is
  unexecuted — the grandfathered-not-edited precedent applies only to
  *executed* stages' prompts (per maintainer direction at ADR-0011
  acceptance), not an un-run one.
- **M06.D retrospective:** the special-log "Capability declaration
  shape" + the `<scope_change>` #1+#2 carry-forwards + the
  grep-verified-false "McpClient impls ConnectionResolver for
  production" claim this ADR records.
- **Carry-forward target:** M07 phase doc (forthcoming) — explicit
  deliverables (a)-(d) in *Neutral / future implications*.

## Notes

This is the second ADR in the M06.D/ADR-0010 lineage of "a
maintainer-inserted phase-doc mandate over-reaches the code reality;
surface before the red phase rather than sink TDD effort, reconcile via
ADR + phase-doc forward-correction." ADR-0010 handled an *uncompilable*
instruction (the Cargo cycle); this handles an *unsatisfiable-in-scope*
instruction (running-app end-to-end given the pre-existing M07
agent-loop lock + a M06.D documentation drift).

The adjudication burden mirrors ADR-0009's:

- **The build agent's burden** — name (a) the prior scope lock that
  bounds the deliverable (Stage A `<scope_change>` + ADR-0009: agent
  loop is M07), (b) the in-stage authorization for the reduction (F's
  own `<execution_warnings>` #1 + F.1 "nothing new in runtime-mcp"),
  and (c) the concrete next-milestone deliverable that closes the loop
  (M07 carry-forward (a)-(d)). All three are cited.
- **The maintainer's burden** — confirm the four Context facts are
  grep-verifiable (no `impl ConnectionResolver for McpClient`; no
  shell `CapabilityEnforcer`/`NamespaceResolver`; smoke path emits no
  `ToolUse`; the M07 lock pre-dates Stage F), not build-agent
  invention. The grep commands are inlined in Context for direct
  re-execution.
- **The protocol's burden** — a phase-doc-authoring check that
  verifies *construction-graph reachability* (not just file/symbol
  existence — the M06.D `<dependency_cycle_check>` carry-forward,
  generalized) would have caught "the concrete dispatcher has no
  constructor inputs in the shell" at authoring time. Tracked as a
  carry-forward to the STAGE-PROMPT-PROTOCOL iteration.

Surfaced to the maintainer before the Stage F red phase via the
`AskUserQuestion` decision gate (ADR-0010 precedent); maintainer
selected "SDK seam + injection seam (Recommended)" and directed this
ADR + the coupled phase-doc forward-correction be surfaced together
for review before F red begins. Status flips to Accepted in the same
PR per `CLAUDE.md` §11 step 4.
