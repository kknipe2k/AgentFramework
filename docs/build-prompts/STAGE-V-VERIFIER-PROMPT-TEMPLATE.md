# M0n Stage V — Verifier prompt template

> Parameterized template for the milestone-V CLI prompt. Copy + parameterize per milestone (M05, M06, …). The agent-runtime instance of the Stage V Verifier protocol per ADR-0008. Companion to `STAGE-PROMPT-PROTOCOL.md` §14 (the schema definition) and `docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md` (the per-V retrospective shape).

---

## How to use this template

1. **Copy** the XML block below into the parent milestone's Phase doc at section `V.5 CLI prompt`.
2. **Parameterize** all `{{placeholder}}` values:
   - `{{MNN}}` — milestone identifier without dot (e.g. `M05`)
   - `{{milestone-short-title}}` — milestone kebab-title (e.g. `gap-capability`)
   - `{{spec-sections}}` — comma-separated spec section refs (e.g. `§4b, §8.security L1–L5, §6a`)
   - `{{primitives-list}}` — semicolon-separated list of public primitives V will exercise (e.g. `framework_loader; request_capability meta-tool; capability enforcer; tier system`)
   - `{{multi-call-surface}}` — semicolon-separated list of methods/commands that must survive sequential-call (e.g. `query_session_db; read_signals; recover_session; respond_hitl; respond_uncertainty`)
   - `{{behavior-targets}}` — semicolon-separated list of user-observable primitives that need runtime-render or DOM/state inspection (e.g. `GapPanel renders; CapabilityBadge shows tier; audit-log file written`)
   - `{{adversarial-targets}}` — semicolon-separated list of trust-boundary surfaces the milestone added/touched, each with its hostile case (e.g. `save_framework dir containment vs ../ traversal; IPC frame cap vs 5MiB no-newline frame; SSE stream vs mid-stream stall`)
3. **Strip** any pass-specific items that don't apply to the milestone (e.g. an all-backend milestone may drop the Behavior pass's DOM targets — but should NOT drop the pass entirely; replace with IPC/state-observation targets).
4. **Surface** the parameterized prompt for user review before the V session runs (per CLAUDE.md §4 Hard Rule 1).

The parameterized prompt is what the user pastes into a fresh CLI session for the V run.

---

## The clear-and-paste session pattern (bias guard)

Before pasting the V prompt:

1. **End** the prior stage's session. Close the CLI window or `/clear` it.
2. **Open** a fresh CLI session. Empty context. No memory of the milestone's work narrative.
3. **Paste** the V prompt as the opening message. The XML `<read_first>` declares what to read; the prompt enforces the fresh-context mandate explicitly.

This is the load-bearing discipline. The build agent + per-stage retro agent + closeout agent ALL share confirmation bias from having shipped the milestone. A fresh-context V agent shows up knowing only the spec — and asks the "naïve" questions that catch contract drift.

---

## The XML prompt template

```xml
<verifier_stage_prompt id="{{MNN}}.V">
  <context>
    Stage V (Verifier) of {{MNN}}. Fresh-context contract-fidelity check of
    {{MNN}}'s deliverables against `agent-runtime-spec.md` ({{spec-sections}}).
    Run with empty session memory — you have NOT seen the work-stage retros,
    the milestone summary, or any prior gap-analysis entries. Your job is to
    ask whether the code does what the spec said, when actually exercised.
    Six passes in order: Inventory → Wire → Behavior → Multi-call invariants → Assembled-execution → Adversarial.
    Findings are tagged 🔴 (block merge), 🟡 (carry forward), 🟢 (tech debt).
    Maximum 2 D.fix iterations before maintainer escalation.
  </context>

  <read_first>
    <file>STAGE-PROMPT-PROTOCOL.md §14 (the verifier schema you are running under — v1.12: six passes)</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (design rationale + the original four passes + the bias guard)</file>
    <file>docs/cluster-pattern.md (the cluster-gate close discipline — basis of the v1.9 fifth pass, assembled_execution) + CLAUDE.md §4 rule 11 (grounded-claims: a "Sound" verdict that did NOT run the assembled path is forbidden)</file>
    <file>docs/build-prompts/{{MNN}}-{{milestone-short-title}}.md (the phase doc — Background, every X.1 problem statement, every X.2 files-to-change table, every X.3 detailed changes, every X.4 tests, section V (this stage's parameters), AND every per-stage `<scope_change>` block in the embedded `<work_stage_prompt>` XML — `<descope>` and `<expand>` children declare authorized in-stage carry-forwards visible to V's bias-guarded read but not to the per-stage retros V is forbidden to read; treat them as source-of-truth for "what V should NOT flag as missing")</file>
    <file>agent-runtime-spec.md ({{spec-sections}})</file>
    <file>docs/MVP-v0.1.md §{{MNN}} (the milestone's scope + acceptance criteria)</file>
    <file>docs/style.md (project conventions — apply when interpreting code shape)</file>
    <file>docs/gotchas.md (project-wide traps — apply when interpreting "is this a bug")</file>
    <file>docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md (your output shape)</file>
    <!-- BIAS GUARD: do NOT load — the validator enforces structural absence: -->
    <!--   - docs/build-prompts/retrospectives/{{MNN}}.*-retrospective.md (per-stage retros) -->
    <!--   - docs/build-prompts/retrospectives/{{MNN}}-summary.md (milestone summary) -->
    <!--   - docs/gap-analysis.md (prior milestone gap-analysis entries) -->
    <!-- Reading these reintroduces the bias the V stage is structured to eliminate. -->
  </read_first>

  <scope_to_verify ref="docs/build-prompts/{{MNN}}-{{milestone-short-title}}.md" section="V.2 Scope to verify"/>

  <verification_passes>
    <pass name="inventory">
      For each file path in {{MNN}}'s X.2 "Files to Change" tables across all
      stages, confirm it exists in `git ls-files` AND matches the shape
      claimed in X.3 "Detailed Changes" (function/struct/type/CSS rule
      named exists at the file). Missing files → 🔴 (build agent claimed
      shipped, didn't). Files present but stub/empty → 🟡 (work in flight).
      Files present with wrong scope (e.g., function exists but wrong
      signature) → 🟡.
    </pass>
    <pass name="wire">
      For each spec claim about user-observable behavior, follow the
      data-path end-to-end using the 5-step protocol:
      (1) Pick a spec claim (e.g., "node size scales with token spend").
      (2) Identify the source event / API surface (`agent_complete.tokens_total`).
      (3) Identify the projection / store path (`graphStore.ts` writes which field?).
      (4) Identify the consumer (which component reads what the projector wrote?).
      (5) Verify the consumer reads what the projector writes.
      Trace breaks at step 4 with zero matching consumers OR multiple
      plausible consumers → 🔴 ("wire incomplete: <which step broke>") OR
      🔴 ("wire ambiguous: <which interpretation is right?>"). Forces the
      build agent to either fix the wire or file an ADR-class waiver per
      `<merge_gate waiver_path="...">` (see below).
      STANDING RULE (codified from M06.V Decision 6 — apply every
      milestone, do NOT re-derive): when the primitive IS delivered and
      unit-tested but its PRODUCTION DRIVER is absent, AND the root cause
      is an already-accepted ADR's NAMED carry-forward, classify 🟡
      (NOT 🔴 — the primitive is correct; NOT silent) WITH mandatory
      carry-forward enumeration: name the ADR, quote the exact
      carry-forward clause, and name the next milestone/stage that owns
      the wire-up. (M06.V handled trace #11/#6 correctly by reasoning
      from ADR-0011 — this rule makes it template-codified, not
      per-milestone re-derived.) Cross-check the milestone's
      `<wire_trace_vs_adr_reconcile>` and `<scope_change>` blocks
      (STAGE-PROMPT-PROTOCOL.md v1.8 / v1.6) before applying — if a
      `<trace>` already flags the trace as ADR-superseded, the 🟡 +
      enumeration is the expected, not surprising, outcome.
    </pass>
    <pass name="behavior">
      Runtime-render check. For each primitive in {{behavior-targets}}, run
      an actual harness call and observe the output — NOT just static read
      the code. Renderer: Vitest + jsdom DOM render with computed-style
      inspection (e.g., assert `.budget-bar__bar--warn` has a `background-color`
      that's not transparent). IPC / backend: integration test with real
      duplex pair / subprocess, exercise the wire and observe the
      response. Static analysis is INSUFFICIENT for this pass — the M04
      BudgetHeaderBar bug (component shipped without its CSS rules) is the
      canonical case static analysis missed.
      STANDING RULE (codified from M06.V Decision 7 — the explicit
      M06.V→M07.V protocol carry-forward): from M07.V onward the
      Behavior pass MUST run the `--features integration`
      reference-MCP-server smoke (e.g.
      `cargo test -p runtime-mcp --features integration`) and record
      "integration smoke executed: N/M, not 0/0" in the verifier retro.
      Rationale: by M07 a real MCP dispatch path exists; a mock-only
      Behavior pass CANNOT rule out rmcp wire-format correctness — that
      is the `transport/stdio.rs`+`http.rs` excluded-holdout latent
      risk, attributed to the feature-gated `tests/integration.rs`
      smoke which ran 0/0 at M06.V (not exercised). If the smoke is
      genuinely unrunnable on the V environment, that is itself a 🟡
      finding (not a silent skip) naming what blocked it.
    </pass>
    <pass name="multi_call_invariants">
      For each method / IPC command / Tauri command in {{multi-call-surface}},
      verify "called twice in sequence works." Run a sequential-call test
      OR confirm one exists in the test suite. M04 IRL drone IPC bug
      (`take_event_stream` single-use) is the canonical case. Verify the
      test PROVES the second call works — not just "the code is shaped
      such that it should work."
    </pass>
    <pass name="assembled_execution">
      RUN the assembled app / assembled integration tests and OBSERVE each
      {{MNN}} user-observable primitive EXECUTE — not just confirm a test
      exists or reads green. Static + unit + wire + multi-call green is
      INSUFFICIENT: M08.6 shipped "Sound, 0🔴" through full A–F + Stage V +
      closeout while the assembled app was never run, and the post-M08.6
      IRL found 7🔴 (built-in tools emitted `ToolInvoked` but never
      executed; `save_framework` dropped companion files). For each
      primitive, drive the REAL path ({{assembled-harness}} — e2e-tauri /
      assembled Rust integration) and confirm the BEHAVIOR, not the event.
      If the assembled harness is genuinely unrunnable on the V
      environment, that is a 🟡 finding naming what blocked it — NOT a
      silent skip, and NOT grounds for a "Sound" verdict. Per CLAUDE.md §4
      rule 11: a "Sound" that did not run the assembled path is forbidden;
      the verdict states explicitly what was NOT exercised.
    </pass>
    <pass name="adversarial">
      THE 6th PASS (v1.12 — standing; origin: the 2026-06-09 external
      review's meta-finding that an IRL-driven culture verifies the happy
      path really runs and exercises nothing hostile — TD-050…057 all
      lived in that blind spot). Scope: the trust-boundary surfaces this
      milestone ADDED or TOUCHED — {{adversarial-targets}} — not a
      whole-app pentest. For each target, actively ATTEMPT the hostile
      case and OBSERVE the rejection/containment:
      (a) boundary escape — `..` traversal, symlink escape, scope-glob
          bypass, renderer-supplied params no UI sends (direct invoke);
      (b) malformed input — oversized / no-delimiter frames, truncated
          streams, invalid JSON where a schema is assumed;
      (c) failure injection — stalls and timeouts (does it hang or
          suspend cleanly?), mid-stream disconnects, every error arm;
      (d) granted-authority abuse (the injection lens) — can
          attacker-shaped CONTENT steer a granted capability beyond
          intent (docs/SECURITY.md "Prompt injection — posture")?
      Assertions are on the OBSERVED rejection + the ABSENT side effect
      (no file outside scope, no unbounded buffer growth, no hang) —
      never on an error event alone. A hostile case not actually run →
      HYPOTHESIS, labeled (rule 11). An unrunnable case → 🟡 naming the
      blocker, never a silent skip. Severity: a reachable escape/hang on
      a shipped surface → 🔴; reachable only behind a precondition the
      milestone doesn't ship → 🟡 (precondition named); hardening
      polish → 🟢.
    </pass>
  </verification_passes>

  <findings_format ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md" section="Findings"/>

  <merge_gate red_blocks="true" dfix_iteration_cap="2" waiver_path="docs/adr/NNNN-waiver-{{MNN}}-finding-N.md"/>

  <gates milestone="{{MNN}}"/>

  <self_correction_budget>3</self_correction_budget>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md">
    <special_log>Time spent per pass; passes that surfaced 🔴 vs 🟡 vs 🟢 findings; whether any pass produced zero findings (potential signal that the pass is too narrow OR the milestone is genuinely clean — distinguish in the retrospective)</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/{{MNN}}-{{milestone-short-title}}.md" section="V.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (build machine `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/{{MNN}}.*-retrospective.md`) — required so any downstream session has the real state, not origin's partial view (CLAUDE.md §19 rule 7)</item>
    <item>findings list, sorted by severity (🔴 first, then 🟡, then 🟢; per-finding: pass that surfaced it, primitive affected, observed-vs-expected, recommended action)</item>
    <item>per-pass summary (N files inventoried; N wires data-path-traced; N behaviors exercised; N multi-call invariants checked)</item>
    <item>retrospective filled-in [END] section (per VERIFIER-RETROSPECTIVE-TEMPLATE.md — verification axes, not build axes)</item>
    <item>merge recommendation: "Proceed to E (closeout)" | "Open D.fix for 🔴 findings: <cite finding numbers>" | "Re-tier — finding scope exceeds D.fix budget"</item>
    <item>explicit statement: "Stage {{MNN}}.V is ready. I will not commit until you approve."</item>
  </approval_surface>
</verifier_stage_prompt>
```

---

## Authoring guidance for the per-milestone parameterization

### Choosing `<scope_to_verify>` — the file lists for all four passes

**Derive from the phase doc, NOT from spec-section pattern-matching.** This is the lesson from M04.V finding #6 (logged at `docs/tech-debt.md` TD-004): pattern-matching "§4a Verify ⇒ `schemas/verification.v1.json`" produced a file path that doesn't exist in the codebase. The phase doc's reality wins; spec sections describe intent, not necessarily what shipped.

The derivation chain:

```
per-stage X.2 "Files to Change" tables (authored when each stage was planned)
    │
    ▼
milestone V.2 "Scope to verify" table (aggregates every X.2 row across stages A → last-work-stage)
    │
    ▼
V prompt's `<scope_to_verify>` (reference form pointing at V.2; inline form only for grandfathered milestones)
```

For M05+ phase docs (which include the Stage V section per `docs/build-prompts/TEMPLATE.md`), use reference form:

```xml
<scope_to_verify ref="docs/build-prompts/{{MNN}}-{{milestone-short-title}}.md" section="V.2 Scope to verify"/>
```

The V.2 table itself is authored at phase-doc-writing time by aggregating every X.2 row across stages A → last-work-stage. **Do not add files that aren't in any X.2 table.** If a spec section seems to imply a file the phase doc didn't claim shipped, that's spec/code drift — flag it during V's Wire pass as a 🟡 finding (the analogous M04.V finding #2 surfaced exactly this pattern: spec §4a names `hook_*` events; code ships `verify_*`). Do NOT add the imagined file to `<scope_to_verify>`.

For grandfathered milestones (M04 only, per ADR-0008) where no V.2 section exists, the V prompt uses inline form with explicit `<files>` and `<spec_sections>` children — and the inline list is still derived from the milestone's X.2 tables, not from spec pattern-matching. `docs/build-prompts/M04-V-prompt.md` shows the inline shape; M04.V finding #6 documents the calibration data from that first real-world run.

### Reading `<scope_change>` blocks (v1.6+)

`STAGE-PROMPT-PROTOCOL.md` v1.6 adds the `<scope_change>` optional slot to `<work_stage_prompt>` — children `<descope>` and `<expand>` declare authorized in-stage carry-forwards. V's `<read_first>` explicitly consumes these blocks (the line above flags them) because they are the structural fix for the M05.V findings #1 + #2 class of bug: descopes documented only in per-stage retros V is forbidden to read.

When V's Inventory pass observes a file or symbol the phase doc X.2 / X.3 named as a deliverable but the codebase doesn't have, check the milestone's `<scope_change>` blocks before flagging 🔴. If a `<descope>` child covers the missing deliverable with a concrete `carry_forward_to=` target, V's correct action is:

- Treat the missing deliverable as **not a 🔴 finding** for the current milestone.
- Log a 🟡 finding referencing the `carry_forward_to=` target so the next milestone's Stage A absorbs the wire-up.
- Note in the verifier retro that the `<scope_change>` block was load-bearing — preserves the bias-guard while documenting the authorized deferral.

Conversely, `<expand>` children declare files / symbols / capabilities that landed without being in the phase doc X.2 / X.3 tables. V's Inventory pass should NOT flag these as drift; the expand block is the authoritative record that the addition was intentional and documented at stage time.

If the milestone has NO `<scope_change>` block and a deliverable is missing, V proceeds as in M05.V — 🔴 finding; build agent must address via D.fix iter 1 OR file an ADR-class waiver per ADR-0008's interpretation-dispute lane.

### Standing rules codified from M06.V (v1.8)

Two rules the verifier previously re-derived every milestone are now codified inline in the pass prose above. Authors parameterizing this template do NOT strip them:

1. **Delivered + tested / driver-absent / root = accepted-ADR carry-forward → 🟡 + mandatory enumeration** (M06.V Decision 6). In the Wire pass, when a primitive is correct and unit-tested but its production driver is absent because an already-accepted ADR explicitly carries the wire-up forward, the finding is 🟡 (not 🔴, not silent), and the finding text MUST name the ADR + quote the carry-forward clause + name the owning next milestone/stage. Cross-reference the milestone's `<wire_trace_vs_adr_reconcile>` (STAGE-PROMPT-PROTOCOL.md v1.8) and `<scope_change>` (v1.6) blocks — when a `<trace>` already declares the trace ADR-superseded, the 🟡 + enumeration is the expected outcome, not a surprise finding. The matching record line is in `VERIFIER-RETROSPECTIVE-TEMPLATE.md`.

2. **M07.V+ Behavior pass MUST run the `--features integration` reference-MCP-server smoke** (M06.V Decision 7 — the explicit M06.V→M07.V protocol carry-forward). Mock-only Behavior cannot rule out rmcp wire-format correctness (the `transport/stdio.rs`+`http.rs` excluded holdout). From M07.V the smoke is a required Behavior input; the retro records `integration smoke executed: N/M, not 0/0`. An unrunnable smoke is a 🟡 finding naming the blocker, never a silent skip. For M06.V (already run) this is informational only; it binds from M07.V onward.

### Choosing `{{behavior-targets}}` — the Pass 3 inputs

This is the pass that catches the M04 BudgetHeaderBar-CSS class of bug. Each behavior target needs:
1. A specific assertion the test asserts (e.g., "computed `background-color` is not `rgba(0,0,0,0)`")
2. A specific harness (Vitest+jsdom for renderer; integration test with subprocess for IPC)
3. A specific failure mode the assertion catches (e.g., "missing CSS rule")

Don't pad the list with generic items. Each entry should have a clear test-fail-mode.

Examples:

| Behavior target | Harness | Failure caught |
|---|---|---|
| `GapPanel renders when `gap_resolved` event applied` | Vitest+jsdom | Component never mounts (event handler missing) |
| `CapabilityBadge background color changes per tier` | Vitest+jsdom + computed-style | Missing CSS rule per tier |
| `audit-log file written when capability check fires` | Tokio integration test + tmpfile read | Wire missing between enforcer + file appender |
| `respond_hitl Tauri command resolves the seam when called twice` | Vitest mock of Tauri invoke | Seam state cleared between calls (single-use bug) |

### Choosing `{{multi-call-surface}}` — the Pass 4 inputs

For each public surface, write the assertion as "call the method twice; both must succeed and return distinct expected results." If the method is one-shot by design (e.g., `take_event_stream` for the long-lived subscription pattern), document that explicitly — the assertion becomes "first call succeeds; second call returns empty stream" (proving the intended one-shot semantics).

If a surface has known concurrency requirements (e.g., two callers hitting the same Tauri command from different renderer tabs), add a concurrent variant — but only after the sequential variant is in place.

### Choosing `{{adversarial-targets}}` — the Pass 6 inputs (v1.12)

Scope to the trust-boundary surfaces the milestone ADDED or TOUCHED — not a
whole-app pentest. Derive from the milestone's X.2 tables filtered to boundary
code (IPC framing, path handling, renderer-supplied parameters,
network/provider failure paths, sandbox policy), cross-referenced against
`docs/SECURITY.md`'s trust-boundary table. Each target needs (1) the hostile
case to attempt, (2) the observed rejection that counts as a pass — which is
the ABSENT side effect (no file outside the root, no hang, no unbounded
buffer), and (3) the harness that drives it. Don't pad: a milestone that adds
no boundary surface declares that explicitly (`{{adversarial-targets}}:
none added — pass scoped to regression of prior-hardened surfaces`) rather
than inventing targets.

| Adversarial target | Hostile case | Pass = observed |
|---|---|---|
| `save_framework` dir containment | direct `invoke` with `dir: "../../x"` | typed rejection; no write outside the root |
| IPC frame length cap | 5 MiB frame with no newline | `MaxLineLengthExceeded` surfaces; process alive |
| provider SSE stream | wiremock stalls mid-stream | typed idle-timeout; session suspends cleanly, no hang |

### Choosing `{{spec-sections}}` — the Pass 2 inputs

Pull the spec sections the milestone's Phase doc claims to implement. Cross-reference against `docs/MVP-v0.1.md` §{{MNN}} (the acceptance criteria). Each section becomes one or more wire-trace inputs for Pass 2.

### What V does NOT do

- **Does NOT** re-author the work-stage tests. V exercises the existing test suite (or adds tests via `D.fix` if 🔴 findings surface).
- **Does NOT** re-run gap-analysis. The closeout (Stage E) writes the gap-analysis entry; V findings feed into closeout's "Carry forward" section as 🟡/🟢 inputs.
- **Does NOT** re-run per-stage retros. V's own retrospective is brief (verification axes only — `VERIFIER-RETROSPECTIVE-TEMPLATE.md`).
- **Does NOT** rewrite the milestone scope. If a 🔴 finding requires a scope change, V escalates ("Re-tier"); maintainer adjudicates.

### Surface-flow at stage end

After V runs, the agent surfaces (per `<approval_surface>` above):

```
Stage M{{NN}}.V — Verifier results

cross-machine state:
  git log --oneline main..HEAD:
    <agent pastes output>
  ls docs/build-prompts/retrospectives/M{{NN}}.*-retrospective.md:
    <agent pastes output>

per-pass summary:
  Inventory:      N files / N matching shape / N missing / N stub
  Wire:           N spec claims traced / N broke / N ambiguous
  Behavior:       N primitives exercised / N passed / N failed at runtime
  Multi-call:     N surfaces tested / N two-call-sequential-pass / N broke on second call
  Assembled:      N primitives driven in the real app / N observed / N not exercised (named)
  Adversarial:    N hostile cases attempted / N rejected-as-expected / N escapes-or-hangs

findings:
  🔴 #1 — <pass>: <primitive> — observed: <X>; expected: <Y>; action: open D.fix
  🟡 #2 — <pass>: <primitive> — <one-line>; action: carry forward to M{{NN+1}}.A
  🟢 #3 — <pass>: <primitive> — <one-line>; action: log to docs/tech-debt.md
  ...

[END] Stage V retrospective:
  Inventory pass took <X> min, surfaced <N> findings
  Wire pass took <X> min, surfaced <N> findings (notable: <one-liner>)
  Behavior pass took <X> min, surfaced <N> findings (notable: <one-liner>)
  Multi-call pass took <X> min, surfaced <N> findings (notable: <one-liner>)
  Verifier axes scoring: see VERIFIER-RETROSPECTIVE-TEMPLATE.md
  Decisions for D.fix or next milestone: <list>

merge recommendation: <Proceed | Open D.fix #1, #4 | Re-tier #2 (scope exceeds D.fix budget)>

Stage M{{NN}}.V is ready. I will not commit until you approve.
```

The user reviews. If 🔴 absent: approve V's findings commit, proceed to Stage E (closeout). If 🔴 present: approve V's findings commit, then user pastes the D.fix prompt (authored on demand by the orchestration session, scoped to the specific 🔴 findings).

### D.fix authoring (when needed)

D.fix is a normal `<work_stage_prompt>` with:
- `id="M{{NN}}.D.fix"` (or D.fix2 for the second iteration)
- `<read_first>` includes V's findings (the verifier retrospective)
- `<deliverable>` cites the specific 🔴 finding numbers to address
- `<scope_locks>` includes "DO NOT introduce new scope outside the cited findings — flagging out-of-scope findings is V's job, not D.fix's"
- Standard work-stage tags otherwise

The D.fix commit message format: `fix({{MNN}}): D.fix iter <N> — finding #X, #Y`.

After D.fix lands, the user clears the session again and re-pastes the V prompt — V re-runs all four passes. If 🔴 surfaces a NEW finding (i.e., not one D.fix addressed), the structural-signal escape applies: stop and re-tier. Otherwise proceed.

Maximum 2 D.fix iterations. After the second, escalate to maintainer regardless.
