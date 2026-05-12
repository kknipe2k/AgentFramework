# M04 Stage V — Verifier prompt (retroactive, one-off)

> **Standalone V prompt for M04.** Per ADR-0008 grandfathering, M04 receives a retroactive V run without editing the M04 phase doc or `docs/gap-analysis.md`. This file holds the parameterized prompt the build agent pastes into a fresh CLI session; findings land at `docs/build-prompts/retrospectives/M04.V-retrospective.md`.

---

## Why this file exists outside the phase doc

The standard Stage V flow has the prompt at section V.5 of the milestone Phase doc (template at `docs/build-prompts/TEMPLATE.md` § Stage V). M04 was authored on v1.3 protocol — before V existed — and its Phase doc (`docs/build-prompts/M04-plan-verify-hitl-budget.md`) is grandfathered: no V section, no edit. The retroactive run is the protocol's first real-world test; the artifact landing point is the verifier retrospective only.

This file is the V prompt's home for M04 specifically. No future milestone needs this — M05+ phase docs include the Stage V section directly, parameterized from `docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md`. M04 is the one-off because of grandfathering.

## What the V run validates

The five M04 IRL bugs (drone IPC single-use, AgentNode wrong-field, BudgetHeaderBar CSS missing, "untitled" nodes, viewport sizing) are **already fixed** in PR #64. The retroactive V run is NOT looking to re-find those bugs — they're resolved. The run validates:

1. **The protocol's structural soundness.** Does V's four-pass shape produce useful findings on a real milestone? Or does it produce noise?
2. **The regression-test coverage holds.** PR #64 added unit tests that catch each of the five bug classes. V's Multi-call + Behavior passes should confirm those tests exist and exercise the right surfaces.
3. **Any other M04 issues V surfaces.** Bugs the IRL test didn't catch, contract drift the work-stage agents missed, etc. Expected: mostly 🟡 (carry-forwards) and 🟢 (tech-debt), few or zero 🔴 (since the major bugs are fixed).

## How to run

1. **End** the current CLI session — close the window or `/clear`. The clear-and-paste pattern is the bias guard; do not run V in a session that has already read M04's retrospectives or summary.
2. **Open** a fresh Claude Code session on the build machine (the one with `C:\agent-runtime` and the test harness).
3. **Paste** the XML block below as the opening message.
4. The agent reads only the spec, phase doc body (NOT the per-stage retros), and code; runs four passes; surfaces findings + retrospective per `VERIFIER-RETROSPECTIVE-TEMPLATE.md`.
5. Surface back for review per `<approval_surface>`. The retrospective commits to `docs/build-prompts/retrospectives/M04.V-retrospective.md`.

---

## The prompt

```xml
<verifier_stage_prompt id="M04.V">
  <context>
    Stage V (Verifier) of M04 — retroactive. Fresh-context contract-fidelity
    check of M04's deliverables (Plan / Verify+Rails / HITL / Budget /
    Recovery primitives, spec §3a + §4a + §6a + §2a + §1b) against
    `agent-runtime-spec.md`. Run with empty session memory — you have NOT
    seen the M04 work-stage retros, the M04-summary, or any prior
    gap-analysis entries. M04 is grandfathered — no Phase doc edit, no
    gap-analysis edit; this is the protocol's first real-world test.

    Five known M04 IRL bugs were fixed in PR #64 (drone IPC single-use,
    AgentNode wrong-field, BudgetHeaderBar CSS missing, "untitled" nodes,
    viewport sizing). Your job is NOT to re-find them — they are resolved.
    Your job is to verify (a) the regression tests added in PR #64 catch
    those bug classes when re-run, (b) M04's remaining surfaces hold up
    under the four-pass check, (c) surface any new findings. Expected
    outcome: mostly 🟡 (carry-forwards) and 🟢 (tech-debt); few or zero 🔴.

    Four passes in order: Inventory → Wire → Behavior → Multi-call
    invariants. Maximum 2 D.fix iterations before maintainer escalation.
  </context>

  <read_first>
    <file>STAGE-PROMPT-PROTOCOL.md §14 (the verifier schema this prompt is structured under)</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (the design rationale + four passes + bias guard)</file>
    <file>docs/build-prompts/M04-plan-verify-hitl-budget.md (Background, Document Structure, Pre-existing legacy file inventory, all stages A1/A2/B/C/D/E/F sections X.1/X.2/X.3/X.4 — but NOT any retrospective references the doc may make)</file>
    <file>agent-runtime-spec.md §3a (Plan + Task), §4a (Verify + Rails), §6a (HITL primitive), §2a (Budget), §1b (Recovery semantics), §1d (drone IPC topology)</file>
    <file>docs/MVP-v0.1.md §M4 (acceptance criteria)</file>
    <file>docs/style.md (project conventions)</file>
    <file>docs/gotchas.md (project-wide traps — entries #66–#72 specifically codify the M04 IRL bug patterns; treat as reference for "what bugs look like in this codebase")</file>
    <file>docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md (your output shape)</file>
    <file>docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md (the parameterized template this M04 prompt instantiates; reference for the four-pass authoring guidance)</file>
  </read_first>

  <scope_to_verify>
    <files>crates/runtime-core/src/plan.rs (Plan + Task schemas + FSM)</files>
    <files>crates/runtime-core/src/verify.rs (Verify hook + Rail schemas)</files>
    <files>crates/runtime-core/src/hitl.rs (HITL prompt schemas + trigger enum)</files>
    <files>crates/runtime-core/src/budget.rs (Budget threshold schemas)</files>
    <files>crates/runtime-main/src/plan/ (Plan FSM + ApprovalSeam — M04 Stage B)</files>
    <files>crates/runtime-main/src/hitl/ (HitlSeam + 9-trigger policy — M04 Stage E)</files>
    <files>crates/runtime-main/src/budget/ (Budget enforcer — M04 Stage F)</files>
    <files>crates/runtime-main/src/recovery/ (Recovery + uncertainty flow — M04 Stage F)</files>
    <files>crates/runtime-main/src/drone_ipc/ (Connection + Client — M04 Stage A2 + PR #64 next_event refactor)</files>
    <files>crates/runtime-drone/src/plan_projector.rs (Plan + Task projection — M04 Stage B)</files>
    <files>src-tauri/src/commands.rs (Tauri commands: approve_plan / revise_plan / abort_plan / respond_hitl / respond_uncertainty / set_global_budget / query_session_db / read_signals / request_resume — all M04-added or M04-modified)</files>
    <files>src/components/ApprovalPanel.tsx (M04 Stage C)</files>
    <files>src/components/HITLPanel.tsx, HITLModal.tsx, HITLToast.tsx (M04 Stage E — 3 UI variants)</files>
    <files>src/components/BudgetHeaderBar.tsx (M04 Stage F — including PR #64 CSS rules)</files>
    <files>src/components/RecoveryDialog.tsx + UncertaintyPrompt.tsx (M04 Stage F)</files>
    <files>src/components/nodes/PlanNode.tsx, TaskNode.tsx, VerifyNode.tsx, HookNode.tsx (M04 Stage C + D)</files>
    <files>src/components/nodes/AgentNode.tsx (PR #64 tokensTotal fix — re-verify the wire holds)</files>
    <files>src/lib/graphStore.ts (plan_projector + verify projection + HITL + budget event applyEvent branches)</files>
    <files>src/styles.css (M04 Stage F CSS + PR #64 budget-bar color rules)</files>
    <files>schemas/plan.v1.json, task.v1.json, hitl.v1.json, budget.v1.json, verification.v1.json (M04 Stage A1 / B / D / E / F schemas)</files>
    <spec_sections>§3a Plan + Task; §4a Verify + Rails; §6a HITL primitive (9 triggers, 3 UI variants); §2a Budget (4 thresholds + downshift hook); §1b Recovery semantics; §1d drone IPC topology</spec_sections>
  </scope_to_verify>

  <verification_passes>
    <pass name="inventory">
      For each file path enumerated in M04's stages A1/A2/B/C/D/E/F under
      "X.2 Files to Change", confirm it exists in `git ls-files` AND
      matches the shape claimed in the corresponding X.3 "Detailed
      Changes" narrative. Missing files → 🔴 ("build agent claimed
      shipped; didn't"). Files present but stub/empty (no implementation
      where the phase doc said one would land) → 🟡. Files present with
      wrong scope (e.g., function name exists but signature differs from
      X.3) → 🟡.

      Pay attention to: schemas (`schemas/*.v1.json` files added in A1
      should exist + have concrete `type` at root per gotcha #57);
      generated TS bindings (`src/types/*.ts` derived from schemas should
      exist alongside their `.json` sources); test files (per X.4 every
      stage should have new tests landed); CSS files (M04 Stage F should
      have budget-bar rules — graduated to gotcha #67 — confirm PR #64
      landed them).
    </pass>
    <pass name="wire">
      For each spec claim about user-observable behavior in M04's scope,
      follow the data path end-to-end via the 5-step protocol:
      (1) Pick a spec claim from §3a, §4a, §6a, §2a, or §1b.
      (2) Identify the source event / API surface (event variant name
          from `schemas/event.v1.json` or Tauri command from
          `src-tauri/src/commands.rs`).
      (3) Identify the projection / store path (which file writes which
          field on which node-data shape in `graphStore.ts`).
      (4) Identify the consumer (which component reads the projected
          field; which Rust code reads the IPC response).
      (5) Verify the consumer reads what the projector writes — same
          field name, same shape.

      Specific traces to perform (at minimum):
      - §3a "Plan approval gate" → `plan_approval_requested` event →
        graphStore `pendingApproval` map → `ApprovalPanel` → `approve_plan`
        Tauri command → `ApprovalSeam::resolve`.
      - §4a "Verify result" → `verify_passed` / `verify_failed` event →
        graphStore `VerifyNodeData` → `VerifyNode.tsx` → DOM status class.
      - §6a "HITL UI variant" → `hitl_requested` event with `ui_variant`
        → graphStore `pendingHitl` map → ONE OF `HITLPanel` / `HITLModal`
        / `HITLToast` based on variant → `respond_hitl` command →
        `HitlSeam::resolve`.
      - §2a "Token spend visualized" → `agent_complete.tokens_total` →
        graphStore writes `tokensTotal` → `AgentNode.tsx` reads
        `tokensTotal` (verify PR #64 fix held — should not be
        `tokensIn + tokensOut`) → `tokenScale()` → DOM transform-scale.
      - §1b "Recovery semantics" → cold-start `localStorage.lastSessionId`
        → `request_resume` Tauri command → `recover_session` drone IPC →
        `RecoveredSession` → `RecoveryDialog` + `UncertaintyPrompt`.

      Trace breaks at step 4 with zero matching consumers OR multiple
      plausible consumers → 🔴 ("wire incomplete" or "ambiguous"). Forces
      a fix or a waiver ADR per `<merge_gate>`.
    </pass>
    <pass name="behavior">
      Runtime-render / runtime-exercise check. For each user-observable
      primitive in M04's scope, run an actual harness call and observe
      the output — NOT just static read the code. Specific behavior
      targets to exercise:

      - `AgentNode` rendered with `tokensTotal: 10` vs `tokensTotal: 50000`
        → assert DOM `style.transform` differs by ≥1.5× scale ratio
        (Vitest + jsdom). Catches the M04 LG-03 class.
      - `BudgetHeaderBar` rendered with each of `status: 'ok' | 'warn' |
        'downshift' | 'suspended' | 'exceeded'` → assert the
        `.budget-bar__bar--<status>` class is in `styles.css` (static
        check) AND the computed background-color is non-transparent for
        each (Vitest + jsdom). Catches the M04 BUD-01 class.
      - `TaskNode` rendered with `title: ''` → assert displays
        `task <id-prefix>` fallback, not blank. Catches the M04 LG-02
        class.
      - `HITLPanel` / `HITLModal` / `HITLToast` rendered with `hitl_requested`
        of each `ui_variant` → assert ONE of the three mounts, two don't,
        each renders the question text. (Vitest + jsdom + testid lookup.)
      - `ApprovalPanel` rendered with `plan_approval_requested` → assert
        Approve / Revise / Abort buttons visible; click Approve → assert
        `approve_plan` Tauri command was invoked. (Vitest mock of Tauri.)
      - Drone IPC `query_session_db` called twice in sequence → assert
        both succeed with distinct expected rows. Confirms PR #64's
        regression test exists + passes. (Rust integration test —
        `cargo test -p runtime-main --lib drone_ipc::client::tests::
        query_session_db_succeeds_twice_in_sequence`.)
      - `<main>` element rendered in a Tauri-window-typical viewport
        (≥1280px) → assert graph canvas fills available width. (Manual
        or Playwright if available; static check of `styles.css` confirms
        max-width is not 720px.)
    </pass>
    <pass name="multi_call_invariants">
      Sequential-call check on M04's public surface. For each, run a
      two-call test (or confirm one exists) and assert both calls
      succeed with distinct expected outcomes:

      - `query_session_db` (Rust integration test in `drone_ipc/client.rs`
        — `query_session_db_succeeds_twice_in_sequence`)
      - `read_signals` (analogous test; check it exists post-PR-#64)
      - `recover_session` (analogous; the next_event refactor in PR #64
        addressed all three — confirm the regression coverage)
      - `respond_hitl` (Tauri command — second call after first response
        should soft-Ok per `respond_hitl_with`)
      - `respond_uncertainty` (Tauri command — same shape)
      - `approve_plan` / `revise_plan` / `abort_plan` (Tauri commands —
        soft-Ok on no-pending-awaiter per `approve_plan_with`)
      - `set_global_budget` (Tauri command — second call replaces first
        value; verify the GlobalBudgetState mutex is per-app not per-call)
    </pass>
  </verification_passes>

  <findings_format ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md" section="Findings"/>

  <merge_gate red_blocks="true" dfix_iteration_cap="2" waiver_path="docs/adr/NNNN-waiver-M04-finding-N.md"/>

  <gates milestone="M04"/>

  <self_correction_budget>3</self_correction_budget>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md">
    <special_log>This is the protocol's first real-world V run. Log explicit observations on (a) whether each pass produced useful signal or noise; (b) whether the 5-step Wire trace was practical at the scope of M04 or felt over-specified; (c) whether the Behavior pass harness coverage was adequate or needed gaps; (d) whether the bias-guard discipline held (did you find yourself wanting to peek at retros?). Decisions[END] section should include explicit protocol refinement recommendations for v1.6 if the run surfaces them.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>

  <commit_message inline="true">
    <type>verify</type>
    <scope>M04</scope>
    <subject>retroactive V run — findings &lt;N🔴/N🟡/N🟢&gt;</subject>
    <body_template>
First real-world test of the Stage V Verifier protocol (ADR-0008,
STAGE-PROMPT-PROTOCOL.md v1.5 §14). Retroactive run against M04 — the
five known M04 IRL bugs (PR #64 fixed) are NOT in scope to re-find;
the run validates protocol structural soundness, regression-test
coverage, and surfaces any other M04 findings.

Per-pass summary:
  Inventory:      &lt;N&gt; files / &lt;N&gt; matching shape / &lt;N&gt; findings
  Wire:           &lt;N&gt; traces / &lt;N&gt; findings
  Behavior:       &lt;N&gt; primitives exercised / &lt;N&gt; findings
  Multi-call:     &lt;N&gt; surfaces tested / &lt;N&gt; findings

Findings: see docs/build-prompts/retrospectives/M04.V-retrospective.md

Outcome: &lt;Sound | Sound but rough | Friction-heavy | Not ready&gt;
Merge recommendation: &lt;Proceed | Open D.fix #X,#Y | Re-tier&gt;

Protocol refinement notes (if any): see retrospective's [END]
Decisions section.

https://claude.ai/code/session_&lt;id&gt;
    </body_template>
  </commit_message>

  <approval_surface>
    <item>cross-machine state at start (build machine `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M04.*-retrospective.md`)</item>
    <item>findings list, sorted by severity (🔴 first, then 🟡, then 🟢; per-finding: pass that surfaced it, primitive affected, observed-vs-expected, recommended action)</item>
    <item>per-pass summary (counts as in commit_message body)</item>
    <item>retrospective filled-in [END] section per VERIFIER-RETROSPECTIVE-TEMPLATE.md — especially the protocol refinement observations (this run is the protocol's calibration data)</item>
    <item>merge recommendation: "Proceed" | "Open D.fix for cited 🔴 findings" | "Re-tier"</item>
    <item>cross-machine state at end (post-V-commit git log + retrospective file listing)</item>
    <item>explicit statement: "Stage M04.V is ready. I will not commit until you approve."</item>
  </approval_surface>
</verifier_stage_prompt>
```

---

## After the run

Per ADR-0008 grandfathering:

- The retrospective lands at `docs/build-prompts/retrospectives/M04.V-retrospective.md` — that's the only artifact created.
- M04 phase doc stays untouched (no V.5 section added retroactively).
- `docs/gap-analysis.md` stays untouched (M04's entry is already merged + immutable; any 🟡 findings carry forward into M05's gap-analysis entry instead).
- 🟢 findings (if any) append to `docs/tech-debt.md`.
- 🔴 findings (unlikely given PR #64) → D.fix iteration scoped to the cited findings, then re-run V.

After M04.V completes Sound (or Sound-but-rough with successful D.fix iter 1), the protocol is validated and M05 starts authoring under v1.5.

If the V run surfaces protocol refinements (a pass that missed a bug class, a pass that produced noise, the 5-step Wire trace over-specified for the scope, etc.), capture them in the retrospective's `[END] Decisions` section. A small protocol-iteration PR (v1.5 → v1.6) lands before M05 starts.
