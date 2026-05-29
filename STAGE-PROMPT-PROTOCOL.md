# Stage Prompt Protocol
> XML schema for stage CLI prompts. Defines required and optional slots, the two schemas (work-stage and closeout-stage), where prompts live, and how they're extracted and validated. Companion to `BUILD-PLAYBOOK.md`.
---
## 1. Purpose
Stage CLI prompts are the structured input pasted into a fresh agent session at the start of each stage. They orient the agent, constrain scope, name the gates, and reference the protocols (retrospective, commit, gate matrix) the stage must follow.
This document defines the schema. It is the canonical reference for how stage prompts are written; the bare templates derived from it live at `prompts/WORK-STAGE-TEMPLATE.md` and `prompts/CLOSEOUT-STAGE-TEMPLATE.md`.
## 2. Why XML inside markdown
The Phase doc has two distinct audiences with different needs.
The human reads it for planning, scope review, and navigation. Markdown is what humans read — headers, tables, links, prose narrative for the milestone.
The agent in a fresh session consumes the structured prompt portion. It benefits from explicit slots (`<context>`, `<deliverable>`, `<gates>`) so nothing required gets dropped, parsing is unambiguous, and stage prompts can be diffed cleanly across milestones to see exactly what evolved.
Pure-markdown prompts lose the slot discipline that makes them parseable and diffable. Pure-XML phase docs become unreadable for the planning purpose. The hybrid — markdown wrapper, XML inside fenced code blocks — gives both audiences what they need.
A second-order benefit: every prompt across all milestones can be extracted programmatically with a single regex over fenced ```xml blocks. This is the bridge to ARIA (or any orchestrator) running phases later without rewriting the prompts.
## 3. Where prompts live
Inside fenced ````xml` code blocks within `docs/build-prompts/M[NN]-<title>.md`. One fenced block per stage. The Phase doc's markdown wrapper is for human planning and review; the XML inside the fenced blocks is what gets pasted into a fresh agent session.
The Phase doc looks roughly like this:
```markdown
# M01: Foundation

## Overview
[prose: what this milestone is, why now, what depends on it]

## Scope
**In scope:** ...
**Out of scope:** ...

## References
| File | Read for |
|---|---|
| ... | ... |

## Stage A — Workspace skeleton
[prose: what Stage A is, what it produces, time-box estimate]

### CLI prompt
\`\`\`xml
<work_stage_prompt id="M01.A">
  ...
</work_stage_prompt>
\`\`\`

## Stage B — ...
[same pattern]

## Stage E — Closeout
[prose: closeout responsibilities]

### CLI prompt
\`\`\`xml
<closeout_stage_prompt id="M01.E">
  ...
</closeout_stage_prompt>
\`\`\`
```
## 4. Programmatic extraction
The contract for extraction is simple and stable:
Every stage prompt is inside a fenced ````xml` block.
Exactly one root element per block: `<work_stage_prompt>`, `<closeout_stage_prompt>`, or `<verifier_stage_prompt>` (v1.5+).
The root element has an `id` attribute formatted `M[NN].<X>` (e.g., `M01.A`, `M01.E`, `M05.V`).
A regex extractor: ````xml\n(<(?:work_stage_prompt|closeout_stage_prompt|verifier_stage_prompt)[\s\S]*?</(?:work_stage_prompt|closeout_stage_prompt|verifier_stage_prompt)>)\n````
A validator should:
Confirm one and only one root element per block
Confirm the root tag is one of the three valid schemas
Confirm `id` attribute matches the format
Confirm all required tags for the schema are present
Confirm no foreign tags appear (every tag must be in the protocol)
## 5. The three schemas
There are exactly three stage prompt schemas. The work and closeout schemas share most tags; the verifier schema deliberately diverges to enforce its fresh-context contract-fidelity role.
Schema	Used for	Distinct requirements
`<work_stage_prompt>`	Work stages (A, B, C, D, …)	Concrete deliverable, test plan required, acceptance criteria
`<closeout_stage_prompt>`	Closeout stage (E, the final stage of every milestone)	Cumulative reads, gap-analysis entry, append-only verification, three-artifact review
`<verifier_stage_prompt>` (v1.5+)	Verifier stage (V, between last work stage and closeout)	Four verification passes (inventory + wire + behavior + multi-call), severity-tagged findings, deliberately omits prior retros from `<read_first>` (bias guard), iteration cap on D.fix loop
Each schema is a different ceremony with distinct cognitive mode. The closeout is cumulative review + immutable ledger entry. The verifier is fresh-context contract fidelity — same milestone, but the agent shows up knowing only what the spec said, not what the work-stage agents narrated. Forcing any pair into one schema would lose enforcement: a closeout missing `<cumulative_reads>` is broken; a verifier reading prior retros is structurally untrusted; a work stage doing either is overkill. Three schemas make these differences enforceable. See §14 (Verifier-only tags) for the v1.5 schema definition and ADR-0008 for the design rationale.
## 6. Common tags (used by both schemas)
> **Note:** "Both schemas" in this section refers to `<work_stage_prompt>` and `<closeout_stage_prompt>`. The v1.5 verifier schema deliberately diverges from these common tags — see §14 for which tags it uses and which it omits.
### Root attribute: `id`
Required. Format `M[NN].<X>`. Examples: `M01.A`, `M01.E`, `M11.D`, `M05.V` (verifier — see §14).
### `<context>`
Required. Two to four sentences. Why this stage exists, what it builds on, what's about to happen. The orientation paragraph the agent reads first after expanding the prompt.
```xml
<context>
  Stage A of M01 (Foundation). Establish the workspace skeleton — Cargo workspace,
  crate boundaries, lefthook hook, CI scaffold. No business logic yet; this stage
  exists so subsequent stages have a place to land. Builds on nothing; this is the
  first stage of the project.
</context>
```
### `<read_first>`
Required. Ordered list of files to read before any code. Cardinality: usually 4–8 files. Always includes the playbook, project identity, relevant spec sections, and the gate matrix.
```xml
<read_first>
  <file>BUILD-PLAYBOOK.md</file>
  <file>docs/identity.md</file>
  <file>docs/gates.md</file>
  <file>spec/agent-runtime-spec.md §0–§0d</file>
  <file>docs/MVP-v0.1.md §M01</file>
  <file>docs/style.md</file>
  <file>docs/gotchas.md</file>
</read_first>
```
### `<read_prior_milestones>`
Optional. Used by Stage A of any non-first milestone that absorbs carry-forward work from prior milestones (the canonical pattern: Stage A closes 🟡 Important items from the prior milestone's gap-analysis entry before opening the milestone's real deliverables). Distinct from `<read_first>` (general orientation) and from `<read_prior_stages>` (within-milestone retrospectives, used by Stage B+).
```xml
<read_prior_milestones>
  <gap_analysis_carry_forward milestone="M01"/>
  <milestone_summary milestone="M01" section="Decisions to apply before next parent milestone"/>
</read_prior_milestones>
```
Multiple prior milestones can be referenced:
```xml
<read_prior_milestones>
  <gap_analysis_carry_forward milestone="M01"/>
  <gap_analysis_carry_forward milestone="M02"/>
  <milestone_summary milestone="M02" section="Decisions"/>
</read_prior_milestones>
```
Omit this tag for Stage A of the first milestone (no prior to absorb) and for Stage B+ (which uses `<read_prior_stages>` for within-milestone reads).
### `<scope_locks>`
Required. Constraints from spec or ADRs that apply across this stage. These are the things the agent must not do even if locally tempting. Contrasts with `<acceptance_criteria>` (what must be done) — `<scope_locks>` is what must not.
Inline form (use when no Phase doc section exists for stage scope — e.g., a single-stage milestone or a stage whose locks are uniquely stage-specific):
```xml
<scope_locks>
  <lock>v0.1 is single-session; no multi-session code paths</lock>
  <lock>STANDARD mode hardcoded; no mode router</lock>
  <lock>Anthropic-only; no provider abstraction layer in v0.1</lock>
  <lock>Windows-only test target; CI runs on all three OSes for drift detection</lock>
</scope_locks>
```
Reference form (required when the Phase doc has a "Key constraints" or equivalent section — strict reference-first per Authoring Rules §10). Section names are resolved by markdown-AST heading lookup, not URI fragments:
```xml
<scope_locks ref="docs/build-prompts/M02-event-pipeline.md" section="Key constraints"/>
```
Use one form or the other, never both. Validator rejects inline content if the named section exists in the Phase doc (strict reference-first; v1.2 hardening).
### `<gates milestone="M[NN]"/>`
Required. Reference to the gate matrix. The agent looks up the milestone row in `docs/gates.md` to see which gates are live. Self-closing tag with `milestone` attribute.
```xml
<gates milestone="M01"/>
```
If a stage temporarily relaxes a gate (rare; requires ADR), use the override form:
```xml
<gates milestone="M01">
  <override gate="coverage_threshold" reason="ADR-0007: M01.A produces no testable code; coverage gate activates at M01.B"/>
</gates>
```
### `<self_correction_budget>`
Optional; defaults to 3 per `BUILD-PLAYBOOK.md` §4.3. Override only when the work genuinely warrants it (e.g., a debugging stage where iteration is the deliverable).
```xml
<self_correction_budget>3</self_correction_budget>
```
### `<retrospective_requirements>`
Required. Reference to the per-stage retrospective template. Self-closing.
```xml
<retrospective_requirements ref="prompts/RETROSPECTIVE-TEMPLATE.md"/>
```
If the stage has retrospective items beyond the template (e.g., specific friction patterns to watch for), add them inline:
```xml
<retrospective_requirements ref="prompts/RETROSPECTIVE-TEMPLATE.md">
  <special_log>Time spent on Cargo.toml feature flag debugging — flag if &gt;30 min</special_log>
</retrospective_requirements>
```
### `<commit_protocol>`
Required. Reference to the playbook section. Self-closing. The agent re-reads §4.7 to refresh on the do-not-commit rule.
```xml
<commit_protocol ref="BUILD-PLAYBOOK.md#4.7"/>
```
### `<commit_message>`
Required. Reference to the pre-authored commit message in the Phase doc (each stage's `X.6 Commit Message` section). Self-closing. Section names are resolved by markdown-AST heading lookup, not URI fragments — drop the URL anchor form entirely (renderer-dependent slugification: `### A.6 Commit Message` → `#a6-commit-message` on GitHub, `#a-6-commit-message` on GitLab/mdBook, etc.).
```xml
<commit_message ref="docs/build-prompts/M02-event-pipeline.md" section="A.6 Commit Message"/>
```
The agent uses the referenced commit message verbatim (filling in only the `session_<id>` placeholder) when surfacing for approval. Pre-authored commit messages keep audit-trail consistency across stages and let the human review the message as part of the Phase doc rather than re-evaluating each one ad-hoc.
If a stage genuinely cannot have a pre-authored commit message (rare; usually only experimental or recovery stages), inline form is permitted:
```xml
<commit_message inline="true">
  <type>feat</type>
  <scope>workspace</scope>
  <subject>...</subject>
  <body_template>...</body_template>
</commit_message>
```
Default to the reference form. Inline is the exception.

### `<read_reference>`
Optional. Files the agent should read for **archetypal pattern reference** (not orientation, not decisions). Distinct from `<read_first>` (general orientation files) and `<read_prior_stages>` (within-milestone retrospectives' Decisions sections). Use when the stage's implementation should mirror a pattern that already exists in the codebase.
```xml
<read_reference>
  <file purpose="*_with seam archetype">crates/runtime-providers/src/anthropic_sse.rs</file>
  <file purpose="codec pattern">crates/runtime-drone/src/ipc.rs</file>
</read_reference>
```
The `purpose` attribute is required — names *why* the file is being referenced. Without it, the slot degrades into "miscellaneous reads" and loses its discriminator value. Validator warns if `purpose` is missing (warning, not error, in v1.2; promote to error in v1.3 once usage stabilizes).

### `<execution_warnings>`
Optional. Inline operational warnings — workflow-time guardrails that apply during stage execution (cost concerns, side-effecting commands, environment-dependent behavior). Distinct from `<gotchas>` (pre-flight implementation traps) and `<scope_locks>` (deliverable-shape constraints).
```xml
<execution_warnings>
  <warning>DO NOT run `cargo test --features integration` in normal flow — hits the live Anthropic API and incurs cost. Reserve for explicit smoke-test sessions with API key in keychain.</warning>
  <warning>Coverage runs (`cargo llvm-cov`) take 3–5 min on a clean target dir — budget accordingly</warning>
</execution_warnings>
```
The distinction matters: a `<gotchas>` entry warns about a code-shape trap the agent might write into a file; an `<execution_warnings>` entry warns about a *command* the agent might run during the stage. Mixing them in `<gotchas>` (the v1.0/v1.1 pattern) loses the action-vs-artifact discriminator.
### `<approval_surface>`
Required. Enumerates what the agent surfaces to the human at stage end and in what order. The order matters — the human reads top-down.

**Surface audience (v1.9; M08.6.C remediation).** Surfaces are paste-bridged from the build machine to a separate orchestration session for verification, NOT read by a human for narrative. Each `<item>` must include the **verbatim data** the orchestration session needs to verify the work, not a pre-summarized prose description. Specifically:

- "cross-machine state" item: the actual `git log --oneline main..HEAD` output + the actual `ls docs/build-prompts/retrospectives/M[NN].*-retrospective.md` output, not "A/B/C committed, retros present"
- "strict-TDD invariant" item: the verbatim diff command + its output (e.g., `$ git diff <red>..<impl> -- 'crates/<x>/tests/**'` followed by `(empty — zero lines)` or the actual diff), not "invariant proven"
- "closure proof" item: a pre/post evidence table or verbatim before/after artifact dumps, not "the regression now passes"
- "gate results" item: each gate's command + verbatim result (and any CI-divergent flag cited inline per CLAUDE.md §6), not "all gates green"
- "retrospective summary" item: the three-axis scoring table with evidence column, not the outcome verdict alone
- "forward decisions" item: enumerated, numbered, file:line-cited

Anti-pattern: surfacing a paragraph that says "X is proven, Y passes, Z is filed" without the verbatim data behind each claim — defeats the orchestration session's ability to verify and produces a longer back-and-forth as it asks for the data.

The M08.6.B surface is the reference shape: cross-machine SHAs + strict-TDD diff command output + ADR closure table + per-gate verbatim results + retro three-axis table + numbered forward decisions. The M08.6.C surface was the negative case (pre-summarized verdict-only items) and triggered this codification.

For work stages, default item order: cross-machine state → strict-TDD invariant proof (if `<tdd_discipline strict="true">`) → closure proof → gate results → retrospective summary → forward decisions → draft commit message → "I will not push until you approve."
```xml
<approval_surface>
  <item>cross-machine state (verbatim git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M[NN].*-retrospective.md)</item>
  <item>strict-TDD invariant proof (verbatim diff command + output) — only if &lt;tdd_discipline strict="true"&gt;</item>
  <item>closure proof (pre/post evidence table; verbatim before/after artifact dumps)</item>
  <item>gate results (each gate: command + verbatim result; CI-divergent flags cited inline per CLAUDE.md §6)</item>
  <item>retrospective summary (three-axis table with score + evidence column)</item>
  <item>forward decisions (numbered; file:line-cited)</item>
  <item>draft commit message (Conventional Commits + DCO + session URL)</item>
  <item>explicit statement: "I will not push until you approve."</item>
</approval_surface>
```
## 7. Work-stage-only tags
These tags are valid only inside `<work_stage_prompt>`.

v1.3 adds five additive optional tags — `<pre_flight_check>`, `<schema_drift_check>`, `<fan_out_grep>`, `<dependency_audit_check>`, `<runtime_environment>` — informed by M01–M03 friction. v1.4 adds four more — `<architecture_check>`, `<schema_audit>`, `<schema_root_check>`, `<phase_doc_inventory_audit>` — informed by M04 friction. v1.6 adds nine more — `<coverage_gate>`, `<schema_ref_audit>`, `<api_breaking_change_audit>`, `<existing_pattern_audit>`, `<interpretation_declarations>`, `<scope_change>`, `<zustand_selector_audit>`, `<playwright_warmup_recipe>`, `<test_isolation_audit>` — plus extensions to `<phase_doc_inventory_audit>` (method/struct/read-first claims) and `<dependency_audit_check>` (feature-flag interdependencies), informed by M05 friction. v1.7 adds one more — `<tdd_discipline>` — making the strict red-phase/green-phase two-commit pattern a first-class protocol element, informed by M06.A's TDD-discipline lapse (hard-gate G5 failure, maintainer override) + M06.C's empirical validation of the strict pattern + web evidence (Nagappan et al. 2009 industrial TDD 60–90% defect reduction; TDAD arXiv:2603.17973 showing TDD prompting WITHOUT structural enforcement INCREASES regressions 9.94%; Anthropic Claude Code TDD docs explicitly recommending "commit tests before impl"). See sections below. v1.9 adds one more — `<close_gate>` — making the cluster-gate close discipline (assembled-run + IRL observed; `docs/cluster-pattern.md` + CLAUDE.md §4 rule 11) a first-class element, plus three cluster-cycle step names + a fifth verifier pass (`assembled_execution`), informed by the M08.6 escape (shipped "Sound, 0🔴"; the post-M08.6 IRL found 7🔴 — the assembled app was never run).
### `<deliverable>`
Required. What this stage produces. Concrete: files, modules, capabilities. Not aspirational. If you can't enumerate it, the stage isn't ready to start.
Inline form (use only when no Phase doc section enumerates the deliverable — e.g., a stage with a one-or-two-item produce-list that doesn't warrant a Phase doc section):
```xml
<deliverable>
  <item>Cargo workspace at repository root with crates: runtime-core, runtime-drone, runtime-main, runtime-sandbox</item>
  <item>Top-level Cargo.toml with workspace members and shared lints</item>
  <item>lefthook.yml with pre-commit hook running fmt + clippy + test (fast subset)</item>
  <item>.github/workflows/ci.yml with the M01 gate suite</item>
  <item>docs/adr/0005-lefthook-over-husky.md (the dependency-justification ADR)</item>
</deliverable>
```
Reference form (required when the Phase doc has a detailed `X.2 Files to Change` + `X.3 Detailed Changes` section — strict reference-first per Authoring Rules §10). Section names are resolved by markdown-AST heading lookup, not URI fragments:
```xml
<deliverable ref="docs/build-prompts/M02-event-pipeline.md" section="A.3 Detailed Changes"/>
```
Use one form or the other, never both. Items in inline form are implicitly ordered (top-to-bottom = implementation order); items in the referenced section are ordered by their position in that section. Validator rejects inline content if the named section exists in the Phase doc.

### `<execution_steps>`
Required. Procedural anchor: the named sequence the agent walks during the stage. Provides the work-cycle skeleton without restating the playbook's internal rules. Each `<step>` names a phase the agent moves through; the playbook (`BUILD-PLAYBOOK.md`) defines what each named step entails.
```xml
<execution_steps>
  <step name="write_failing_tests" budget="1"/>
  <step name="implement" budget="1"/>
  <step name="verify_gates" budget_iterations="3"/>
  <step name="fill_retrospective"/>
  <step name="surface"/>
</execution_steps>
```
Standard step names (validator warns on unrecognized names): `write_failing_tests`, `implement`, `verify_gates`, `fill_retrospective`, `surface`. v1.7 adds four more recognized names for the strict two-commit TDD shape governed by `<tdd_discipline>`: `red_phase_commit`, `surface_for_red_approval`, `green_phase_commit`, `surface_for_final_approval` (see the `<tdd_discipline>` slot in this section for the canonical two-commit sequence). v1.9 adds three more for the cluster-gate cycle governed by `<close_gate>`: `ground_at_red`, `mutation_gate`, `assembled_run_irl` (see the `<close_gate>` slot). Stages with non-standard cycles (e.g., a debugging stage where iteration is the deliverable) may add custom steps with explicit `name` attributes; document them in the Phase doc's stage section.
The `budget` / `budget_iterations` attributes are advisory — if a stage budgets `verify_gates` at 3 iterations and the agent hits 4, the agent surfaces per `BUILD-PLAYBOOK.md` §4.3 escalation rule rather than silently continuing.
Why this slot exists: in v1.0/v1.1, the procedural sequence lived in inline STEP 1–5 prose inside each prompt. That worked but duplicated playbook content into every prompt and risked drift. The slot replaces the prose with named anchors that resolve to playbook sections.
### `<test_plan_required>`
Required. Almost always `true`. The agent must state the test plan before writing code per `BUILD-PLAYBOOK.md` §3.3. Setting `false` means the stage produces no testable code (rare; usually only the very first scaffolding stage of a project).
```xml
<test_plan_required>true</test_plan_required>
```
### `<acceptance_criteria>`
Required. The stage's exit conditions as a checklist. The agent verifies each before surfacing for approval. Distinct from gates (which are CI-style automated checks) and from `<deliverable>` (which is what files exist) — acceptance criteria are behavioral checks the deliverable must satisfy.
Inline form (use only when no Phase doc section enumerates the criteria):
```xml
<acceptance_criteria>
  <criterion>cargo build --workspace succeeds</criterion>
  <criterion>cargo fmt --all -- --check passes (no diff)</criterion>
  <criterion>cargo clippy --workspace --all-targets -- -D warnings passes</criterion>
  <criterion>lefthook install succeeds; pre-commit hook fires on a test commit</criterion>
  <criterion>CI workflow file validates against GitHub Actions schema</criterion>
  <criterion>ADR-0005 filed and PR-ready</criterion>
</acceptance_criteria>
```
Reference form (required when the Phase doc has a `X.4 Tests` or equivalent section enumerating behavioral checks — strict reference-first per Authoring Rules §10):
```xml
<acceptance_criteria ref="docs/build-prompts/M02-event-pipeline.md" section="A.4 Tests"/>
```
Use one form or the other, never both. Validator rejects inline content if the named section exists in the Phase doc.
### `<read_prior_stages>`
Required for Stage B onward; omitted for Stage A. References to prior stage retrospectives' "Decisions for next stage" sections. The agent reads these as the first action in the stage and applies the decisions.
```xml
<read_prior_stages>
  <retrospective section="decisions">retrospectives/M01.A-retrospective.md</retrospective>
</read_prior_stages>
```
For Stage C+, list all prior stages of the same milestone:
```xml
<read_prior_stages>
  <retrospective section="decisions">retrospectives/M01.A-retrospective.md</retrospective>
  <retrospective section="decisions">retrospectives/M01.B-retrospective.md</retrospective>
</read_prior_stages>
```
### `<pre_flight_check>`

Optional. Pre-stage sanity checks the agent runs BEFORE any code is written or test plan executed. Distinct from `<read_first>` (orientation reads) and `<execution_steps>` (procedural sequence): pre-flight checks are environmental verifications — branch state, prior-stage commit presence, dependency installation, environment variables — that gate the stage from starting if violated.

Children: `<check>` elements with `name="..."` and inline body describing the check + expected outcome. Each `<check>` is a single shell command or condition; the agent runs them in order before STEP 1 of `<execution_steps>`.

Validator behavior (v1.3 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<check>` children lack a `name` attribute.

Schema: work-stage only.

Example:

```xml
<pre_flight_check>
  <check name="branch_correct">git rev-parse --abbrev-ref HEAD must equal claude/m04-plan-verify-hitl-budget</check>
  <check name="prior_stage_committed">git log --oneline -1 must show "M04 Stage A" subject (Stage A.2 is current)</check>
  <check name="anthropic_key_set">Test-Path env:ANTHROPIC_API_KEY must succeed (Stage B verify gates need live API)</check>
</pre_flight_check>
```

If any check fails, the agent surfaces the failure and stops — does not proceed to STEP 1. Authoring rule §10 cross-stack-integration-must-be-verified pairs naturally with this tag for cross-stack stages.

### `<schema_drift_check>`

Optional. Verify `schemas/*.v1.json` matches the generated Rust + TS types committed to the repo. Wraps `cargo xtask regenerate-types --check` as a stage-level gate so the failure surfaces at pre-flight rather than mid-implementation. Specifically addresses the M03.D + M03 gap-analysis pattern where hand-maintained `event.rs` drifted from the schema source-of-truth (per CLAUDE.md §14).

Self-closing form supported. Optional `gate="..."` attribute names a specific gate command to run; defaults to `cargo xtask regenerate-types --check`.

Validator behavior (v1.3 lean): structural — error if the tag appears outside a work-stage prompt.

Schema: work-stage only.

Example:

```xml
<schema_drift_check/>
```

Or with explicit gate:

```xml
<schema_drift_check gate="cargo xtask regenerate-types --check"/>
```

When this tag appears in a stage prompt, the agent runs the gate command after STEP 2 (implement) and BEFORE STEP 3 (verify_gates); a non-zero exit fails the stage immediately and the agent surfaces "schema drift detected — regenerate types or update the schema."

### `<fan_out_grep>`

Optional. Explicit grep searches the agent must run before changing a name, type signature, or schema field — to find all call-sites that need coordinated updates. Addresses rename/move surprise bugs where a stage changes a type and the agent misses one consumer (M02 + M03 retros recurring pattern).

Children: `<grep>` elements with `pattern="..."` and `purpose="..."` attributes. Each `<grep>` is a literal pattern (not regex unless purpose names regex); the agent runs them and lists matched files BEFORE making the rename or signature change.

Validator behavior (v1.3 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<grep>` children lack `pattern` or `purpose` attributes.

Schema: work-stage only.

Example (from a hypothetical M04 stage renaming a type):

```xml
<fan_out_grep>
  <grep pattern="ContextType" purpose="all callsites of the Rust enum being renamed; expect runtime-core, runtime-main, drone, and any Tauri commands referencing it"/>
  <grep pattern="context_type" purpose="snake_case TS field name corresponding to the Rust enum; renderer-side"/>
  <grep pattern='"context"' purpose="serde-renamed JSON field name on the wire; check schemas/ and any test fixtures"/>
</fan_out_grep>
```

The agent runs each grep and surfaces matched-file count per pattern. If any pattern matches files outside the stage's `<deliverable>` scope, the agent surfaces "rename fan-out exceeds stage scope" and asks for direction before proceeding.

A second use is value-consistency verification — confirming a hard-coded value (URL, version pin, identifier) in the stage's `<deliverable>` ref-content matches the convention already established in the codebase. Worked example: when authoring a new `schemas/*.v1.json` file, grep for `"$id"` across `schemas/` to confirm the base URL pattern matches existing schemas before writing the value into the new file. M03.5 Stage A retroactively validated this pattern when the Phase doc's verbatim `$id` value diverged from the existing `schemas/` convention; a `<fan_out_grep pattern='"$id"' purpose="verify schema $id base URL pattern is consistent across schemas/*"/>` would have caught the discrepancy at pre-flight.

### `<dependency_audit_check>`

Optional. Explicit verification of dependency tree state — version pins, feature flags, transitive audit findings — before code that depends on those deps is written. Addresses gotcha #29 (`keyring` 3.x stub backend silently passing writes; M02 PR #45 hotfix) and gotcha #39 (`npm audit` transitive overrides; M03.F).

Children: `<dep>` elements with `name="..."` (crate or npm package), optional `required_features="..."` (comma-separated), optional `min_version="..."`, optional `audit="..."` (e.g., `audit="high"`). Each `<dep>` is a fact the agent verifies via the appropriate dep-tree inspection (`cargo tree -p <name> -e features`; `npm ls <name>`; `npm audit --audit-level=<audit>`) before writing dependent code.

Validator behavior (v1.3 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<dep>` children lack a `name` attribute.

Schema: work-stage only.

Example:

```xml
<dependency_audit_check>
  <dep name="keyring" required_features="apple-native,windows-native,sync-secret-service" min_version="3.6"/>
  <dep name="reqwest" required_features="rustls,rustls-native-certs,json,stream"/>
  <dep name="serialize-javascript" min_version="7.0.5" audit="high"/>
</dependency_audit_check>
```

When this tag appears in a stage prompt, the agent runs the verification commands during STEP 2 (implement) before adding code that depends on the verified deps. A failed verification surfaces as a blocker — the agent stops and asks for direction before proceeding.

### `<runtime_environment>`

Optional. Explicit declaration of the OS the build agent is expected to run on for this stage, plus platform-specific command variants where the prompt body contains commands that differ across OS (e.g., `Select-String` vs `grep`, `Test-Path` vs `test -f`). Addresses CRLF warnings (PR #48), PowerShell-vs-bash command differences in stage prompts, and the macOS-unsupported caveats (gotcha #23 tauri-driver, gotchas #25-#27 various Vite/test patterns).

Self-closing form with `os="..."` attribute, OR child `<command>` elements with `os="..."` and `cmd="..."` attributes for platform-specific commands.

Attributes:
- `os` — one of `windows | linux | macos | any`. Default `any`.
- `note` — optional inline rationale for the OS pin (e.g., "Tauri-driver requires Linux or Windows; macOS unsupported per gotcha #23").

Validator behavior (v1.3 lean): structural — error if the tag appears outside a work-stage prompt; warning if `os` attribute is missing on the root tag.

Schema: work-stage only.

Examples:

Single-OS:

```xml
<runtime_environment os="windows" note="Build agent runs on Windows 11 per the established M01-M03 pattern; Select-String is the assumed grep equivalent throughout the prompt"/>
```

Multi-OS with command variants:

```xml
<runtime_environment os="any">
  <command os="windows" cmd="Select-String -Path schemas/error.v1.json -Pattern 'CmdError'"/>
  <command os="linux" cmd="grep -n 'CmdError' schemas/error.v1.json"/>
  <command os="macos" cmd="grep -n 'CmdError' schemas/error.v1.json"/>
</runtime_environment>
```

When this tag appears in a stage prompt, the agent runs only the commands matching the current `$RUNNER_OS` (or local equivalent). Authors using inline OS-specific commands (e.g., a `Select-String` command in the prompt body) should pair them with this tag so the validator can flag missing variants in v1.4+.

A common realistic case for the build agent's local environment: PowerShell shell with the Bash tool available as a fallback. Authors pinning `os="windows"` should note in the `note` attribute whether the Phase doc's verification commands assume native PowerShell invocation (avoiding bash variable expansion of `$_`, `$checks`, etc. in heredoc-wrapped scripts) or bash-tool-wrapped invocation (which requires the commands to be safe under bash's variable interpolation).

### `<architecture_check>`

Optional. Verify HOW-claims about cross-process flow, IPC topology, or in-process-vs-out-of-process state ownership BEFORE writing code that depends on them. Companion to `docs/gotchas.md` #41 (grep-verify codebase): that gotcha catches WHAT-claims (file paths, symbol names, struct fields); `<architecture_check>` catches HOW-claims (which process owns which state, which IPC path carries which data, which seam suspension is in-process vs cross-process). Bit M04.C and M04.E (approval/HITL seam architecture — phase doc text implied drone-mediated; reality is in-process per ADR-0007).

Children: `<claim>` elements with `description="..."` and `verify="..."` attributes. Each `<claim>` states the architectural assumption + the command or check that confirms it. The agent runs each `verify` and asserts the result matches the `description` before proceeding to STEP 1 implementation.

Validator behavior (v1.4 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<claim>` children lack `description` or `verify` attributes.

Schema: work-stage only.

Example:

```xml
<architecture_check>
  <claim description="ApprovalSeam resolves in-process via Tauri command (not via drone IPC roundtrip)" verify="grep -r 'ApprovalSeam' src-tauri/src/ | grep -v 'DroneClient'"/>
  <claim description="Plan/task events flow through WriteSignal IPC (not a separate ApproveDecision variant)" verify="grep -nE 'DroneCommand::(ApproveDecision|HitlDecision)' crates/runtime-core/src/drone.rs; expect zero matches"/>
  <claim description="Drone is audit log, not orchestrator: vdr.rs + plan_projector.rs project at signal-write; drone holds no in-flight HITL state" verify="grep -n 'HashMap<String, oneshot' crates/runtime-drone/src/; expect zero matches"/>
</architecture_check>
```

Different from `<fan_out_grep>` (which finds *consumers* of a name): `<architecture_check>` confirms *invariants about the codebase's structural shape*.

### `<schema_audit>`

Optional. Before proposing a new `schemas/*.v1.json` file, audit the relevant spec section's existing `$defs` + neighboring schema files to confirm the proposed type isn't already declared elsewhere. Phase-doc claims of "new schema X" may collide with an existing `$defs/X` in an already-shipped schema (gotcha #41 sibling failure mode). Bit M04.D (`hooks.v1.json` proposal — turned out `Hook` + `HookRef` + `HookCategory` + `JsonLogicExpression` already in `common.v1.json` and `Rail` already in `framework.v1.json`).

Children: `<survey>` elements with `pattern="..."` and `purpose="..."` attributes. Each `<survey>` enumerates existing `$defs` or root types matching the pattern across `schemas/*.v1.json`. The agent runs each survey and surfaces matched-file count before authoring the new schema.

Validator behavior (v1.4 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<survey>` children lack `pattern` or `purpose` attributes.

Schema: work-stage only.

Example:

```xml
<schema_audit>
  <survey pattern='"HookCategory"' purpose="confirm HookCategory $def not already in common.v1.json or framework.v1.json before authoring hook.v1.json"/>
  <survey pattern='"Rail"' purpose="confirm Rail $def not already in framework.v1.json"/>
  <survey pattern='"JsonLogicExpression"' purpose="confirm JsonLogicExpression $def not already in common.v1.json"/>
</schema_audit>
```

If a survey matches existing schemas, the agent surfaces "schema collision: pattern X already declared in `schemas/Y.v1.json` line N" and asks for direction (extend existing schema vs author new namespaced type) before proceeding.

### `<schema_root_check>`

Optional. Verify a new or edited `schemas/*.v1.json` file has a concrete `type` at the root (not a top-level `$ref`) before running `cargo xtask regenerate-types`. `json-schema-to-typescript` errors at `parseNonLiteral` on top-level `$ref` even when typify (Rust side) accepts it (gotcha #57). Bit M04.E `hitl.v1.json` initial draft.

Self-closing form supported. Optional `gate="..."` attribute names a specific check command; defaults to a jq inline that asserts presence of `type` OR `oneOf` OR `anyOf` at the schema root.

Validator behavior (v1.4 lean): structural — error if the tag appears outside a work-stage prompt.

Schema: work-stage only.

Example:

```xml
<schema_root_check/>
```

Or with explicit gate per-file:

```xml
<schema_root_check gate="jq -e '.type or .oneOf or .anyOf' schemas/budget.v1.json"/>
```

When this tag appears, the agent runs the gate command after each new or edited schema file is authored AND before invoking `cargo xtask regenerate-types`. A failure surfaces "schema root must be concrete (`type` / `oneOf` / `anyOf` required); remove top-level `$ref` and inline the root definition's fields."

### `<phase_doc_inventory_audit>`

Optional. Pre-flight verification that every file path in the phase doc's `X.2 Files to Change` table exists in `git ls-files` (or is explicitly marked `new` / `deleted` / `renamed`). Codifies the discipline that produced gotcha #41 ("Phase doc claims about codebase reality must be grep-verified"). Bit M04.A2 (phase doc claimed `vdr.rs` in `runtime-main`, `event_translation.rs`, `prompt_template.rs`, `RevertToSnapshot` as new — none matched reality); M04 Stage B inherited a 1476-line wrong rewrite from PR #53.

Children: `<inventory_row>` elements with `path="..."` and `status="..."` attributes. Status enum: `exists` | `new` | `deleted` | `renamed`. The agent runs `git ls-files <path>` for each `<inventory_row>` and asserts the file's actual presence/absence matches `status`. Mismatches surface as "phase doc inventory drift" and the agent stops before code work begins.

Validator behavior (v1.4 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<inventory_row>` children lack `path` or `status`; error if `status` value is not in the enum.

Schema: work-stage only.

Example:

```xml
<phase_doc_inventory_audit>
  <inventory_row path="crates/runtime-main/src/plan/state_machine.rs" status="new"/>
  <inventory_row path="crates/runtime-drone/src/db.rs" status="exists"/>
  <inventory_row path="crates/runtime-main/src/sdk/decision_extractor.rs" status="deleted"/>
  <inventory_row path="crates/runtime-drone/migrations/" status="new"/>
</phase_doc_inventory_audit>
```

Different from `<pre_flight_check>` (which runs environmental checks like branch state, commit presence, env vars): `<phase_doc_inventory_audit>` checks the phase doc's own truth claims against the codebase before the stage starts authoring against them.

#### v1.6 extension — method / struct-field / read-first claim audit

v1.5's `<phase_doc_inventory_audit>` accepts `<claim type="file">` children only ("file path exists" check). v1.6 extends the slot to accept three additional `type` values, each verified at pre-flight time:

- `type="method"` — verify a function / method symbol exists at a specified `path`. Bit M05.B (phase doc B.3.4 claimed `sdk/mod.rs::dispatch_tool` exists; it doesn't — the structural seed for ADR-0009).
- `type="struct_field"` — verify a struct field exists with a specified name at `path`. Bit M05.F (phase doc F.3.1 sample referenced `gap.title` when the actual field name was `missingName` / `agentTitle`).
- `type="read_first_target"` — verify a file referenced from another stage's `<read_first>` block exists at the literal path written. Bit M05.D (D's `<read_first>` referenced `M05.C-retrospective.md`; the actual files are `M05.C1-retrospective.md` + `M05.C2-retrospective.md` after the C-stage split).

Backward-compatible: existing `<claim type="file">` shape continues to work. The validator (v1.6 lean) does not enforce these new type values structurally — it accepts any string in `type=` and warns on unrecognized values; promote to enum-rejection in v1.7+ once usage stabilizes.

Example (M05.B + D shape — would have surfaced D1 + the C-retrospective filename drift at authoring time):

```xml
<phase_doc_inventory_audit>
  <claim type="file" path="crates/runtime-main/src/sdk/mod.rs" verified="true"/>
  <claim type="method" path="crates/runtime-main/src/sdk/agent_sdk.rs" symbol="dispatch_tool" verified="false" note="phase doc B.3.4 claimed exists; does not; surface as descope via <scope_change>"/>
  <claim type="struct_field" path="crates/runtime-main/src/budget/cost.rs" symbol="BudgetEntry.session_id" verified="true"/>
  <claim type="read_first_target" path="docs/build-prompts/retrospectives/M05.C1-retrospective.md" verified="true"/>
  <claim type="read_first_target" path="docs/build-prompts/retrospectives/M05.C2-retrospective.md" verified="true"/>
</phase_doc_inventory_audit>
```

Pairs naturally with v1.6's new `<scope_change>` slot — when an authoring-time claim verifies false, the resolution is often to descope and surface via `<scope_change>` so the next stage and Stage V's bias-guarded read can see the intentional carry-forward.

#### v1.8 extension — store-slot / wrapper-param shape audit

The v1.6 `<claim type="...">` children verify a symbol *exists*. M06.E proved existence is insufficient at the renderer/IPC wire: phase-doc pseudocode modeled `currentMcpServers` as a `Map` with a `.tools` member while the shipped store slot was `Record<string, McpServerStatusRecord>`, and modeled `mcp_test_connection {config}` while the actual Tauri command took `{name}` — five such shape drifts at one stage, none catchable by an existence check. v1.8 adds an optional `shape="..."` attribute to `<claim>` for store-slot (and analogous typed-surface) claims: the author pins the **actual** TS/Rust type at authoring time, not just the symbol name.

- `<claim type="store_slot" path="..." symbol="..." shape="<actual TS type>" verified="true"/>` — verify the store slot exists AND its declared type matches `shape`. `shape` is the literal type string (e.g. `Record<string, McpServerStatusRecord>`); a mismatch surfaces as "phase-doc shape drift" at pre-flight, before any renderer pseudocode is authored against the wrong shape.

Backward-compatible: `shape=` is optional; existing `<claim>` children without it are unchanged. The validator (v1.8 lean) does not enforce `shape` structurally — it is honor-system attestation, same treatment as `verified=`; promote to a generated-type cross-check at v1.9+ once usage stabilizes. Pairs with the new `<wire_signature_audit>` slot below (which pins the IPC wrapper params the same way `shape=` pins the store slot).

Example (M06.E shape — would have surfaced the `Map`-vs-`Record` drift at authoring time):

```xml
<phase_doc_inventory_audit>
  <claim type="store_slot" path="src/state/graphStore.ts" symbol="currentMcpServers" shape="Record&lt;string, McpServerStatusRecord&gt;" verified="true"/>
</phase_doc_inventory_audit>
```

### `<coverage_gate>` (v1.6)

Optional. Names the exact `--ignore-filename-regex` argument the stage's `cargo llvm-cov` invocation will use, replacing prose enumeration of "plumbing files" / "OS-signal holdouts" / "the new module". The slot eliminates the prose-vs-regex translation step that recurred at 7 stages of M05 (A/B/C1/C2/D/E/F) — M05.C1's "plumbing files" phrase took 4 attempts to land the correct argument; M05.C2 hit the same when lifting `ipc.rs` into the gate.

Children: `<gate>` elements with `scope="workspace" | "package"`, optional `name="..."` for the package, `target_lines="<N>"`, and `ignore_filename_regex="<regex>"`. The regex string is the literal argument value (escape `.` as `\\.` per llvm-cov conventions).

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<gate>` children lack `scope` or `target_lines`.

Schema: work-stage only.

Example (M05.C2 final state):

```xml
<coverage_gate>
  <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
  <gate scope="package" name="runtime-drone" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.lib\.rs|src.shutdown\.rs"/>
  <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs"/>
  <gate scope="package" name="runtime-sandbox" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.lib\.rs|src.seccomp\.rs|src.landlock\.rs"/>
</coverage_gate>
```

The agent uses the regex strings verbatim in its `cargo llvm-cov` invocation at the `verify_gates` step. Stage-end retrospective records the exclusion list in the `[END] Coverage holdouts` section; CLAUDE.md §5 carries the long-form rationale.

### `<schema_ref_audit>` (v1.6)

Optional. Verifies that every `$defs/<Name>` reference the phase doc names actually exists in the sibling schema file. Authoring-time check. Bit M05.A (phase doc A.3 sample referenced `mcp_missing` as one of "three existing variants" when only two existed) and M05.E (phase doc E.3.1 referenced `common.v1.json#/$defs/NonEmptyString` — a `$def` that doesn't exist in `common.v1.json`).

Children: `<ref>` elements with `schema="<path>"`, `path="#/$defs/<Name>"`, and `verified="true|false"` (honor-system attestation by the phase-doc author).

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<ref>` children lack `schema` or `path`. The `verified` attribute is honor-system; promote to validator cross-check at v1.7 if the pattern repeats.

Schema: work-stage only.

Example:

```xml
<schema_ref_audit>
  <ref schema="schemas/event.v1.json" path="#/$defs/GapSeverity" verified="true"/>
  <ref schema="schemas/capability.v1.json" path="#/$defs/ResourceName" verified="true"/>
  <ref schema="schemas/common.v1.json" path="#/$defs/NonEmptyString" verified="false" note="bit M05.E — $def does not exist; use local $defs/AuditSessionId instead"/>
</schema_ref_audit>
```

Different from `<schema_audit>` (v1.4: surveys neighboring schemas for collision before authoring a NEW schema) and `<schema_drift_check>` (v1.3: runs `cargo xtask regenerate-types --check` to ensure generated code matches schemas). `<schema_ref_audit>` verifies that *cross-schema `$ref` strings* point at real `$defs` — different failure mode than collision or drift.

### `<api_breaking_change_audit>` (v1.6)

Optional. Surfaces the migration cost (call-site count + test count) of a sync→async (or other signature-breaking) change BEFORE implementation starts. Bit M05.E — phase doc implied making `CapabilityEnforcer::grant` async, which would have required `.await` at every call site; the eventual solution was a sync `grant()` + async `audit_grant()` split, settled after ~10 minutes of design iteration.

Children: `<change>` elements with `api="<fq-path>"`, `before_signature="..."`, `after_signature="..."`, `call_sites="<N>"`, `test_sites="<M>"`, and `recommendation="<one-line guidance>"`.

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<change>` children lack `api`, `before_signature`, or `after_signature`.

Schema: work-stage only.

Example:

```xml
<api_breaking_change_audit>
  <change api="CapabilityEnforcer::grant" before_signature="fn grant(&mut self, agent_id: &str, caps: Vec&lt;CapabilityDeclaration&gt;)" after_signature="async fn grant(&amp;mut self, agent_id: &amp;str, caps: Vec&lt;CapabilityDeclaration&gt;)" call_sites="6" test_sites="14" recommendation="split into sync grant() + async audit_grant(); production callers chain grant(); audit_grant().await;"/>
</api_breaking_change_audit>
```

When this tag appears, the agent surfaces the audit content during `<execution_steps>::implement` pre-flight before any signature change. If the proposed change has >0 call sites and the recommendation is "split", the agent applies the split rather than the breaking change.

### `<existing_pattern_audit>` (v1.6)

Optional. Greps for existing irrefutable bindings of an enum variant or struct field that an in-progress change is about to break. Bit M05.D — adding `TierForbidden` to `CapabilityError` immediately broke 4 call sites using `let CapabilityError::Denied { .. } = err;` patterns (irrefutable bindings of the now-multi-variant enum). Recurred conceptually at M05.E + M05.F (3 stages total).

Children: `<pattern>` elements with `grep_for="<literal pattern>"`, `rationale="<why this matters>"`, `affected_files="<N>"`, and `remediation="<one-line fix guidance>"`.

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<pattern>` children lack `grep_for` or `rationale`.

Schema: work-stage only.

Example (M05.D shape):

```xml
<existing_pattern_audit>
  <pattern grep_for="let CapabilityError::Denied" rationale="adding TierForbidden variant breaks irrefutable bindings of the existing single-variant pattern" affected_files="4" remediation="convert to refutable match arms covering both Denied and TierForbidden"/>
</existing_pattern_audit>
```

Different from `<fan_out_grep>` (v1.3: finds CONSUMERS of a name about to be renamed). `<existing_pattern_audit>` finds CODE SHAPES that an enum-variant addition (or analogous breaking change) will break — not call sites of the type itself, but uses that exhaustiveness-check would have flagged before the change landed.

### `<interpretation_declarations>` (v1.6)

Optional. Surfaces the phase-doc-author's adopted interpretation of an ambiguous spec section, distinguishing it from a plausible alternative. Bit M05.D — the runtime-vs-install-time interpretation of §8.security L4 (tier filter): phase doc / spec / MVP-v0.1.md gave three different framings; the implementer had to diagnose the discrepancy at runtime. Authoring-time declaration eliminates the diagnostic cost.

Children: `<adopt>` elements with `spec_section="<§N.M>"`, `interpretation="<what the phase doc adopts>"`, `alternative_interpretation="<what was rejected>"`, and `rationale="<why this one>"`.

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<adopt>` children lack `spec_section` or `interpretation`.

Schema: work-stage only.

Example (M05.D shape):

```xml
<interpretation_declarations>
  <adopt spec_section="§8.security L4" interpretation="runtime gate filtering every dispatch" alternative_interpretation="install-time capability filter that strips disallowed entries from the loaded framework" rationale="v0.1 has no install flow distinct from runtime load; runtime gate is strictly stronger than install-time filter"/>
</interpretation_declarations>
```

Different from `<architecture_check>` (v1.4: verifies HOW-claims about codebase structure via grep). `<interpretation_declarations>` declares the spec interpretation the stage adopts, BEFORE writing code — no verify command, just a phase-doc-author commitment.

### `<scope_change>` (v1.6 — highest priority)

Optional. Surfaces intentional in-stage descopes (or scope expansions) of phase-doc deliverables so they are visible to (a) the next stage's `<read_prior_stages>` reads AND (b) Stage V's bias-guarded `<read_first>` — without requiring V to read the per-stage retrospective where descopes were previously documented. Bit M05.V Decision 3, surfaced via Findings #1 + #2 + ADR-0009: Stage B's authorized SDK wire-up descope was documented only in M05.B-retrospective.md — exactly the file V is forbidden to read.

The waiver-as-ADR machinery (ADR-0008/ADR-0009) absorbed the M05 finding correctly, but `<scope_change>` is the structural fix that lets V's read-list pick up the descope without weakening the fresh-context bias guard.

Children: `<descope>` (intentional removal of a phase-doc deliverable) or `<expand>` (intentional addition not in the phase-doc X.2 table — e.g., M05.C1's `crates/runtime-sandbox/src/ipc.rs` and M05.E's `crates/runtime-main/src/tier/transition.rs`). Each child has:

- `deliverable="<one-line description>"`
- `reason="<why scoped this way>"`
- `carry_forward_to="<next-stage / next-milestone target>"` (descope only)
- `authorized_by="<phase doc section that authorized the change>"` (descope only)
- `documented_in="<commit-message / retro section>"` (expand only — points to where the expansion was recorded)

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<descope>` lacks `deliverable` or `reason`. **V protocol implication:** Stage V's `<read_first>` is expected to consume the phase doc's `<scope_change>` block when present (the verifier prompt template at `docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` carries the inline note).

Schema: work-stage only.

Example (M05.B + C1 + E shape — would have prevented M05.V's two 🔴 findings from triggering ADR-0009 if authored at phase-doc time):

```xml
<scope_change>
  <descope deliverable="L1 enforcer.check SDK wire-up in run_agent_with_provider_stream" reason="v0.1 SDK is streaming-only; no synchronous dispatch surface exists to wrap (Anthropic API dispatches tools server-side)" carry_forward_to="M06 Stage A (multi-turn tool loops)" authorized_by="M05 Stage B execution_warnings"/>
  <descope deliverable="L2a narrow() SDK spawn-path wire-up" reason="v0.1 has no sub-agent spawn loop" carry_forward_to="M06 Stage A" authorized_by="M05 Stage B execution_warnings"/>
  <expand deliverable="crates/runtime-sandbox/src/ipc.rs (sandbox-side IPC server)" reason="C1 plumbing requires a server half not enumerated in C1.2; sibling to C2's main-side sandbox_ipc client" documented_in="M05.C1 commit body + M05.C1-retrospective.md surface log"/>
</scope_change>
```

Pairs naturally with `<phase_doc_inventory_audit>` extensions (v1.6) — when an authoring-time `<claim>` verifies false, the typical resolution is to surface the underlying descope via `<scope_change>`. Pairs with — does not replace — the ADR-0008 waiver-as-ADR lane: `<scope_change>` is for descopes known at phase-doc authoring time; waiver-as-ADR is for descopes discovered at Stage V verification time when the build agent disputes a 🔴 finding on interpretation grounds.

### `<zustand_selector_audit>` (v1.6)

Optional. Renderer-stage-only. Surfaces the Zustand v5 `useShallow` requirement for derived-array selectors (`filter` / `map` / `find` over a slice of store state). Bit M05.F — phase doc F.3.1 sample didn't name `useShallow`; the naive selector triggered `Maximum update depth exceeded` infinite-loops in CapabilityBadge tests.

Children: `<selector>` elements with `pattern="<which operations>"`, `requires_use_shallow="true|false"`, `import_path="<module path>"`.

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt.

Schema: work-stage only (renderer-stage convention; the validator does not yet distinguish renderer from backend stages).

Example:

```xml
<zustand_selector_audit>
  <selector pattern="filter|map|find" requires_use_shallow="true" import_path="zustand/react/shallow"/>
</zustand_selector_audit>
```

When this tag appears, the agent applies `useShallow` to every derived-array selector it writes in the stage's deliverable, and confirms via grep at `verify_gates` that no naked `.filter` / `.map` / `.find` exists in the new renderer code at a `useStore(...)` call site.

### `<playwright_warmup_recipe>` (v1.6)

Optional. Renderer-stage-only. Names the curl warmup probe to run before the first Playwright spec invocation against a cold Vite dev server. Bit M04.C ApprovalPanel + M05.F GapPanel (2 stages now) — Vite cold-start dep-optimizer pass on top of the smoke baseline (~30–60s) exceeds Playwright's default per-spec timeout.

Self-closing form with `url`, `timeout_seconds`, and `before_first_spec` attributes.

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt.

Schema: work-stage only.

Example:

```xml
<playwright_warmup_recipe url="http://localhost:1420" timeout_seconds="16" before_first_spec="true"/>
```

When this tag appears, the agent runs the warmup probe (e.g., `curl -fs http://localhost:1420 -o /dev/null --max-time 16`) before invoking Playwright on the first new spec file in the stage. Pairs with gotcha #53 (Vite cold-start dep-optimizer pass).

### `<test_isolation_audit>` (v1.6)

Optional. Renderer-stage-only. Names persistent store slots (Zustand slices preserved across `clear()` calls) that test files mutating those slots must reset via explicit `beforeEach`. Bit M05.F — CapabilityBadge tests inherited the `currentTier` slot Stage D added to `useGraphStore`; Stage D's contract preserves user-preference slots across `clear()`, so test files need explicit reset to avoid cross-test bleed.

Children: `<persistent_slot>` elements with `store="<store name>"`, `field="<field name>"`, `preserved_across_clear="true|false"`, `required_reset="<reset shape>"`.

Validator behavior (v1.6 lean): structural — error if the tag appears outside a work-stage prompt.

Schema: work-stage only.

Example:

```xml
<test_isolation_audit>
  <persistent_slot store="useGraphStore" field="currentTier" preserved_across_clear="true" required_reset="beforeEach(() => useGraphStore.setState({ currentTier: 'novice' }))"/>
</test_isolation_audit>
```

### `<dependency_audit_check>` (v1.6 extension)

v1.3's `<dependency_audit_check>` accepts `<dep>` children for crate / npm-package version + feature-list verification. v1.6 extends the slot to accept an additional child kind plus new attributes on `<dep>`:

- `<feature_interdependency>` — declare that one FFI function binding requires features beyond its home module. Bit M05.C2: `CreateJobObjectW` lives in `Win32_System_JobObjects` but its `SECURITY_ATTRIBUTES` parameter type drags in `Win32_Security` — `windows-sys` feature-gates function bindings by ALL parameter type modules, not just the function's home module. Forward-applicable to any future Windows FFI.
- `<dep>` (existing) with new optional `prefer_crates_io_name="true"` + `source_authority="<reference>"` attributes — distinguish crates.io canonical names from GitHub-org names (`libseccomp` on crates.io vs `libseccomp-rs` as GitHub org). Bit M05.C2 (`windows-sys 0.59` vs `winapi 0.3` choice took web research to settle).

Backward-compatible: existing `<dep>` shape continues to work without the new attributes.

Validator behavior (v1.6 lean): structural — warning if `<feature_interdependency>` children lack `crate`, `function`, `home_feature`, or `requires_features`.

Example (M05.C2 final form):

```xml
<dependency_audit_check>
  <dep name="windows-sys" version="0.59" prefer_crates_io_name="true" source_authority="WEBCHECK Microsoft Rust-for-Windows guide" required_features="Win32_Foundation,Win32_Security,Win32_System_JobObjects,Win32_System_Threading"/>
  <feature_interdependency crate="windows-sys" function="CreateJobObjectW" home_feature="Win32_System_JobObjects" requires_features="Win32_Security" reason="SECURITY_ATTRIBUTES parameter type lives in Win32_Security"/>
  <dep name="libseccomp" version="0.3" prefer_crates_io_name="true" source_authority="docs.rs/libseccomp/0.3"/>
</dependency_audit_check>
```

When this tag appears, the agent runs `cargo tree -p windows-sys -e features` and confirms the listed `<feature_interdependency>` rows are satisfied before invoking the FFI function.

### `<tdd_discipline>` (v1.7)

Optional. Makes the strict red-phase/green-phase two-commit TDD pattern a first-class, structurally-auditable protocol element. When present with `strict="true"`, the stage's `<execution_steps>` MUST be the two-commit shape (below) and the agent commits failing tests as a standalone commit BEFORE any implementation, surfaces for red-phase approval, then implements without modifying the test files.

**Evidence basis (why this is a slot, not a per-stage user override):**

- Industrial TDD: 60–90% defect-rate reduction (Microsoft + IBM case studies, Nagappan et al. 2009).
- LLM-specific: TDAD paper (arXiv:2603.17973) found TDD *prompting* WITHOUT structural enforcement INCREASES regressions 9.94%; structural enforcement drops regressions 70%. The discriminator is structural enforcement, not exhortation — which is exactly why a `<tdd_discipline>` slot (structural) beats a `<gotchas>` note (exhortation).
- Anthropic Claude Code docs: "TDD is the single strongest pattern for working with agentic coding tools"; explicitly warns "Claude naturally writes implementation first, then tests — TDD requires the inverse"; explicitly recommends "commit tests before impl" as the safety net against the agent silently modifying tests to make them pass.
- M06.A empirical: TDD-discipline lapse (implementation before tests) failed hard-gate G5; required a documented maintainer override. M06.C empirical: the strict two-commit pattern worked as designed at ~10% time overhead and provided STRUCTURAL defense against gotcha #66 ("tests-pass-but-contract-fails") — the green-phase commit's diff against the red-phase commit is the load-bearing audit surface, verifiable in one command.

Children: `<red_phase>` and `<green_phase>` (prose bodies; honor-system at the v1.7 lean validator level).

Validator behavior (v1.7 lean): structural — the tag is optional and passes through the regex parser without allowlisting (same lean treatment as all v1.6 optional slots per the v1.6 changelog item #13). The `<tdd_discipline>` → two-commit `<execution_steps>` coupling is an authoring-discipline rule documented in §11, NOT code-enforced at v1.7; promote to a validator cross-check at v1.8+ once the pattern has 2+ milestones of clean signal (lean-validator pattern continued from v1.3/v1.4/v1.5/v1.6).

Schema: work-stage only.

The two-commit `<execution_steps>` shape this slot governs (the v1.7 recommended default for any work stage that ships testable code):

```xml
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
```

The four new step names (`red_phase_commit`, `surface_for_red_approval`, `green_phase_commit`, `surface_for_final_approval`) join the recognized set from v1.7 onward (the validator does not enforce step names; they are documented here so future stages don't re-derive them).

Example (M06.C final form — the empirical reference implementation):

```xml
<tdd_discipline strict="true">
  <red_phase>
    Write all failing tests across the test plan's buckets. Stub the
    production code surfaces just enough to make the test files
    compile (todo!() / unimplemented!() bodies are fine; the goal is
    link-time test discovery, not behavior). Confirm tests fail with
    right-reason errors per CLAUDE.md §5 (assertion failed / cannot
    find function / unresolved import / not-yet-implemented panic —
    NOT a test-file compile error and NOT a tautological pass).
    Commit as a STANDALONE `test(M0X.Y): failing tests for ...`
    commit on the active branch BEFORE green-phase impl; the commit
    body pastes the first ~40 lines of cargo test output proving the
    expected-failure class. Surface the red-phase commit to the user;
    user approves before green phase begins.
  </red_phase>
  <green_phase>
    Implement until ALL failing tests pass. Do NOT modify the test
    files during implementation — if a test is wrong, fix it in a
    SEPARATE follow-up commit with explanation, never silently in the
    impl commit. The impl commit body MUST state the verifiable
    audit-surface invariant: `git diff <red-sha>..<impl-sha> --
    '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
    rustfmt/clippy fixes to test files go in the separate follow-up
    commit, keeping the impl↔red diff a pure implementation delta.
  </green_phase>
</tdd_discipline>
```

The audit-surface invariant is the load-bearing property: any reviewer can run one `git diff` command and confirm the implementation did not tamper with the contract the red phase pinned. This is the structural defense the TDAD research identifies as the difference between TDD-that-works and TDD-prompting-that-backfires. Different from `<execution_steps>` alone (which names the sequence): `<tdd_discipline>` declares the *contract* the sequence must satisfy + the verifiable invariant the green-phase commit must state.

### `<close_gate>` (v1.9)

Optional; the **cluster-gate close discipline** made first-class (parallel to `<tdd_discipline>`). Governs the condition under which a work stage / cluster is **done**: not "tests green" but the **assembled thing run and observed**. Per `docs/cluster-pattern.md` + CLAUDE.md §4 rule 11 (grounded-claims): a stage that ships user-observable behavior closes only when (a) the assembled path (CI e2e / assembled integration test) has RUN and the observable behavior was seen, and (b) for user-facing surfaces, the maintainer IRL-watched it. Mirrors why `<tdd_discipline>` is structural, not exhortation (the TDAD finding): the close-on-run discipline must be a declared contract, not a hope. Bit M08.6 — it shipped "Sound, 0🔴" through the full A–F + Stage V + closeout machinery while the assembled app was never run; the post-M08.6 IRL found 7🔴 (built-in tools emitted `ToolInvoked` but never executed; `save_framework` dropped companion files).

Children: `<assembled_run>` (the assembled test/path that proves the behavior — the dual-role BDD acceptance), `<irl_required>` (`true|false` — true for user-observable surfaces), `<mutation_gate>` (`cargo-mutants` on Rust clusters / Stryker on TS — a surviving mutant on the cluster's logic blocks close or is justified inline), `<triage>` (zero-propagation — new findings from the run are fix-in-cluster / new-cluster / explicit-ADR-scope-out, never routed forward; cluster-pattern.md §2).

The matching `<execution_steps>` shape (v1.9 recognized step names): `ground_at_red` (resolve grounding sub-steps before writing tests) → the strict two-commit steps → `verify_gates` → `mutation_gate` → `assembled_run_irl` (the close gate — run the assembled thing + IRL-watch) → `fill_retrospective`. The `<approval_surface>` carries the **closure proof** (the assembled-run output observed + the IRL result — NOT "tests green").

Validator behavior (v1.9 lean): structural pass-through (same lean treatment as `<tdd_discipline>` + the v1.8 audit slots); the close-gate contract is authoring-discipline governed by `docs/cluster-pattern.md`, not code-enforced at v1.9 — promote to a cross-check once 2+ milestones show clean signal.

Schema: work-stage only.

```xml
<close_gate>
  <assembled_run>run_test_session_with against a tempfile fixture; assert the agent quotes the file (observable), not the ToolInvoked event</assembled_run>
  <irl_required>true</irl_required>
  <mutation_gate>cargo-mutants on the executor + the drive_stream branch</mutation_gate>
  <triage>new findings: fix-in-cluster / new cluster / ADR scope-out — zero propagation (cluster-pattern.md §2)</triage>
</close_gate>
```

### `<construction_reachability_check>` (v1.8)

Optional. For every phase-doc deliverable phrased as "wire / inject / construct X into/through Y", the author traces the **construction graph** at authoring time: does Y's constructor (in the assembled composition the phase doc targets) actually receive X, and are X's constructor inputs reachable at that call site? The discriminator from `<phase_doc_inventory_audit>`: that slot proves the symbol *exists*; this slot proves the symbol is *constructible where the deliverable claims it is wired*. Bit M06.D (the construction-reachability carry-forward generalizing the M06.D `<dependency_cycle_check>`) + M06.F (the "MCP dispatch end-to-end in the running app" mandate over-reached — no `impl ConnectionResolver for McpClient`, no shell ctor inputs for the concrete dispatcher, the only `AgentSdk` construction is the no-tools smoke path; `M06.F-retrospective.md:31,77,116,121`).

Children: `<wire>` elements with `claim="<inject X into Y>"`, `constructor="<Y::new at file:line>"`, `inputs_reachable="<true|false — narrative>"`, `resolution="<scope to M0N / descope via <scope_change>>"`.

Validator behavior (v1.8 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<wire>` children lack `claim` or `constructor`. Honor-system at v1.8 (same lean treatment as the nine v1.6 optional slots + v1.7 `<tdd_discipline>`); promote to a validator cross-check at v1.9+ once 2+ milestones show clean signal.

Schema: work-stage only.

Example (M06.F shape — would have caught "no shell ctor inputs for the concrete dispatcher" at authoring):

```xml
<construction_reachability_check>
  <wire claim="inject Arc&lt;dyn McpToolDispatch&gt; into AgentSdk run loop"
        constructor="AgentSdk::new (src-tauri/src/commands.rs:NNN)"
        inputs_reachable="false — no shell McpDispatcher ctor; only the no-tools smoke path constructs AgentSdk"
        resolution="scope to M07 / descope via &lt;scope_change&gt;"/>
</construction_reachability_check>
```

Pairs with `<scope_change>` (v1.6) — when `inputs_reachable="false"`, the resolution is typically a descope surfaced via `<scope_change>` so the next stage and Stage V's bias-guarded read see the authorized carry-forward. Different from `<architecture_check>` (v1.4: verifies a HOW-claim about existing structure via grep): `<construction_reachability_check>` traces whether a *planned* wire's constructor inputs are reachable in the assembled composition the deliverable targets.

### `<wire_signature_audit>` (v1.8)

Optional. Renderer/IPC-stage convention. Before authoring any renderer pseudocode or IPC-wrapper call, the author pins the wrapper to the **actual** Tauri command params. Bit M06.E — phase-doc pseudocode modeled `mcp_test_connection {config}` while the shipped Tauri command took `{name}` (`M06.E-retrospective.md:118,183`; five such drifts at E). Companion to the v1.8 `<phase_doc_inventory_audit shape=...>` extension: that pins the store slot's type; this pins the IPC wrapper's params.

Children: `<wrapper>` elements with `ipc_command="<command name>"`, `actual_params="<the real param object shape>"`, `phase_doc_assumed="<what the phase doc pseudocode assumed>"`.

Validator behavior (v1.8 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<wrapper>` children lack `ipc_command` or `actual_params`. Honor-system at v1.8; promote at v1.9+ per the lean-validator pattern.

Schema: work-stage only.

Example:

```xml
<wire_signature_audit>
  <wrapper ipc_command="mcp_test_connection" actual_params="{ name: string }" phase_doc_assumed="{ config }"/>
</wire_signature_audit>
```

### `<wire_trace_vs_adr_reconcile>` (v1.8)

Optional. Work-stage AND verifier-relevant (Stage V's bias-guarded `<read_first>` consumes it, same mechanism as `<scope_change>`). At phase-doc-authoring time, every Wire trace is reconciled against accepted ADRs: does any accepted ADR supersede the architecture the trace assumes? Bit M06.V Decision 6 — V.3 Wire trace #6 expected `McpClient` to drive namespace re-resolution; ADR-0010 (accepted, read after the trace was authored) had moved the resolver into `McpDispatcher`. The inconsistency should have been caught at phase-doc-authoring, not at V (`M06.V-retrospective.md:76`).

Children: `<trace>` elements with `id="<trace number>"`, `assumes="<architecture the trace assumes>"`, `adr_checked="<ADR-NNNN>"`, `superseded="<true|false — what moved>"`, `resolution="<rewrite trace N against the current architecture before authoring>"`.

Validator behavior (v1.8 lean): structural — error if the tag appears outside a work-stage prompt; warning if `<trace>` children lack `id` or `adr_checked`. Honor-system at v1.8; promote at v1.9+ per the lean-validator pattern.

Schema: work-stage only (consumed by Stage V's read-list, like `<scope_change>`).

Example (M06.V Decision 6 shape — would have caught trace #6 at authoring):

```xml
<wire_trace_vs_adr_reconcile>
  <trace id="6" assumes="McpClient drives namespace re-resolution"
         adr_checked="ADR-0010" superseded="true — resolver moved to McpDispatcher"
         resolution="rewrite trace #6 against McpDispatcher before authoring"/>
</wire_trace_vs_adr_reconcile>
```

Pairs with `<scope_change>` (v1.6) for the V-read mechanism: both are work-stage slots whose content Stage V's `<read_first>` deliberately consumes (the verifier prompt template at `docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` carries the inline note). Different from `<interpretation_declarations>` (v1.6: declares a spec-section reading): this slot reconciles a *Wire trace's assumed architecture* against *accepted ADRs*.

## 8. Closeout-only tags
These tags are valid only inside `<closeout_stage_prompt>`.
### `<cumulative_reads>`
Required. Enumerates what must be read before drafting the closeout artifacts. Distinct from `<read_first>` (orientation files for any stage) — cumulative reads are the body of work the closeout reviews.
```xml
<cumulative_reads>
  <codebase>entire shipped codebase to date (cumulative across all merged milestones)</codebase>
  <spec>spec/agent-runtime-spec.md (end-to-end, focus on M01-touched sections)</spec>
  <gap_analysis>docs/gap-analysis.md (all prior entries)</gap_analysis>
  <retrospectives>retrospectives/M01.*-retrospective.md (all stages of this milestone)</retrospectives>
  <summary>retrospectives/M01-summary.md (will be authored as part of this stage)</summary>
</cumulative_reads>
```
### `<deliverables>`
Required. The closeout produces three artifacts (four from v1.6 onward — the `<simplify_pass>` child below is now a required child of `<deliverables>`). Plural form distinguishes from work-stage `<deliverable>`.
```xml
<deliverables>
  <milestone_summary>retrospectives/M01-summary.md (aggregates per-stage retrospectives, scores axes across stages, marks verdict)</milestone_summary>
  <gap_analysis_entry>docs/gap-analysis.md (append new entry; six required sections, none optional)</gap_analysis_entry>
  <coverage_policy_reconciliation>If any coverage threshold/exclusion changed this milestone: docs/coverage-policy.md §C milestone entry appended (+ §B baseline if a module entered a gate), and the CLAUDE.md §5 category list, §6 llvm-cov commands, and codecov.yml verified byte-consistent. If nothing changed, state "no coverage change this milestone". Per CLAUDE.md §6 "Coverage policy: source of truth & change protocol".</coverage_policy_reconciliation>
  <simplify_pass>...</simplify_pass>
  <pr_description>draft only; do not open PR until explicitly asked</pr_description>
</deliverables>
```

`<coverage_policy_reconciliation>` is a required child of closeout
`<deliverables>` from this version onward: the closeout is the
single point that verifies the four coverage mirrors
(`docs/coverage-policy.md`, CLAUDE.md §5 categories, §6 commands,
`codecov.yml`) did not drift during the milestone. A "no coverage
change this milestone" statement satisfies it when nothing changed.

### `<simplify_pass>` (v1.6, required child of closeout `<deliverables>`)

Required child of `<deliverables>` in `<closeout_stage_prompt>` from v1.6 onward. Recurring structural refactor checkpoint at every Stage G closeout, addressing maintainer feedback that `simplify`-class consolidations (duplicated patterns across stages, modules that grew organically across the milestone, parallel API surfaces, dead code, premature abstractions) have historically been requested out-of-band per milestone.

The pass runs AFTER the milestone summary + gap-analysis entry have been drafted and BEFORE the PR opens — so refactor proposals consume the cumulative diff of the milestone and the maintainer reviews them as part of three-artifact review.

Required children:

- `<invoke>` — name the skill and the diff scope. Self-closing with `skill="simplify"` and `against="<diff-spec>"` (typically `milestone cumulative diff (M[NN].A..HEAD)`).
- `<surface>` — name the kinds of refactor proposals the pass produces. Self-closing with `kind="refactor_proposals"` and `examples="<categories>"`.
- `<approval_required>` — `true` (a hard contract — applied refactors require explicit maintainer approval before they land on the milestone branch).
- `<commit_on_approval>` — text describing the commit shape (typically "focused refactor commit on same branch before PR opens").
- `<defer_unapproved_to>` — name the destination for non-approved proposals (typically `docs/tech-debt.md` — the ADR-0008 🟢 ledger).

Validator behavior (v1.6): the `<simplify_pass>` child is required to be present somewhere in the closeout prompt's XML; child elements within it are structurally checked but the bodies (e.g., the `<invoke skill="simplify"...>` attributes) are honor-system at v1.6 — promote to attribute-level enforcement in v1.7 once usage stabilizes.

Schema: closeout-only.

Example:

```xml
<simplify_pass>
  <invoke skill="simplify" against="milestone cumulative diff (M[NN].A..HEAD)"/>
  <surface kind="refactor_proposals" examples="duplication / dead code / parallel API surfaces / modules grown across stages / premature abstractions"/>
  <approval_required>true</approval_required>
  <commit_on_approval>focused refactor commit on same branch before PR opens</commit_on_approval>
  <defer_unapproved_to>docs/tech-debt.md (per ADR-0008 🟢 ledger)</defer_unapproved_to>
</simplify_pass>
```

Workflow at Stage G:
1. Closeout agent drafts the milestone summary + gap-analysis entry first (the work that informs the simplify pass — knowing what shipped, knowing what's already on the carry-forward).
2. Agent runs the `simplify` skill against the cumulative diff.
3. Agent surfaces refactor proposals as part of the approval surface (alongside summary + gap-analysis).
4. Maintainer approves a subset.
5. Agent applies the approved subset as a focused refactor commit ON THE SAME MILESTONE BRANCH, BEFORE the PR opens.
6. Agent logs non-approved items to `docs/tech-debt.md` (per ADR-0008 🟢 severity convention).
7. PR opens with the simplify-commit included.

The pass is intentionally tied to the closeout, not to a separate cadence, because (a) cumulative diff is the natural surface for finding stage-spanning duplication; (b) maintainer review at the milestone PR is the natural surface for approving refactors; (c) embedding it in closeout makes it a structural ceremony rather than an ad-hoc maintainer ask.

### `<gap_analysis_requirements>`
Required. Reference to the playbook section that defines the six-section structure, plus a required `<gotchas_graduation>` subsection that audits per-stage `<gotchas>` entries across the milestone (v1.2 enforcement; see Authoring Rules §10 graduation rule).
```xml
<gap_analysis_requirements ref="BUILD-PLAYBOOK.md" section="3.4 Gap Analysis Entry">
  <gotchas_graduation>
    <stage_review id="A">
      <gotcha>brief description of the trap as it appeared in Stage A</gotcha>
      <disposition>kept | graduated | resolved | expired</disposition>
      <target>
        for graduated: docs/gotchas.md §N (heading);
        for resolved: commit-hash that fixed it;
        for expired: "n/a" + 1-line rationale (why it doesn't apply forward);
        for kept: "stays in per-stage <gotchas> of stages X, Y" (next-milestone forward references)
      </target>
    </stage_review>
    <!-- one stage_review per prior stage in the milestone; required even when empty -->
    <!-- if a stage had no gotchas, write: <gotcha>None observed.</gotcha><disposition>n/a</disposition> -->
  </gotchas_graduation>
</gap_analysis_requirements>
```
Disposition enum (exhaustive, validator rejects unknown values):
- **kept** — trap still applies forward; stays in the per-stage `<gotchas>` of future stages that hit the same surface
- **graduated** — recurred in 2+ stages; promoted to `docs/gotchas.md` and removed from per-stage tags; `<target>` cites the gotchas.md section
- **resolved** — fixed by code change so the trap is no longer reachable; `<target>` cites the commit hash
- **expired** — stage-local trap with no forward applicability (e.g., "lefthook v1 syntax" in a one-time scaffold stage); `<target>` is `n/a` + 1-line rationale. The rationale is the safety valve — forces the author to articulate *why* it doesn't apply forward, catching the case where someone marks `expired` to avoid evaluating `kept`/`graduated`.

If the closeout has additional special items to flag in the gap-analysis entry (e.g., a known divergence to resolve), add them inline alongside `<gotchas_graduation>`:
```xml
<gap_analysis_requirements ref="BUILD-PLAYBOOK.md" section="3.4 Gap Analysis Entry">
  <gotchas_graduation>...</gotchas_graduation>
  <special_check>Verify lefthook.yml matches ADR-0005's named gate set; flag any drift</special_check>
</gap_analysis_requirements>
```
Validator rules for `<gotchas_graduation>`:
- Every prior stage in the milestone must appear as a `<stage_review id="...">` element (counted by parsing the milestone's Phase doc for stage headings)
- Each `<stage_review>` must contain at least one `<gotcha>` + `<disposition>` pair
- `<disposition>` must be one of the four enum values
- The validator does **not** semantically check correctness of the disposition (author judgment); it only checks the structural shape
### `<append_only_verification>`
Required. Names the two append-only checks: local diff and CI job.
```xml
<append_only_verification>
  <local_check>prior content of docs/gap-analysis.md must be a literal prefix of HEAD before commit</local_check>
  <ci_check name="gap-analysis-append-only">fails if any prior line is modified</ci_check>
</append_only_verification>
```
### `<three_artifact_review>`
Required. Names the three artifacts the human reviews at PR time and the immutability flag for the ledger entry.
```xml
<three_artifact_review>
  <artifact>code diff (cumulative across milestone)</artifact>
  <artifact>per-stage retrospectives + milestone summary</artifact>
  <artifact>new gap-analysis entry — flagged "IMMUTABLE once committed"</artifact>
  <pushback_blocks_pr>true</pushback_blocks_pr>
</three_artifact_review>
```
## 9. Optional tags (valid in both schemas)
### `<adr_triggers>`
Use when the stage's planned work might trip ADR requirements (per `BUILD-PLAYBOOK.md` §4.8). Pre-flagging keeps the agent from discovering the requirement mid-stage.
```xml
<adr_triggers>
  <trigger>If pre-commit hook tool is changed (e.g., husky over lefthook), file ADR per §4.8</trigger>
  <trigger>If any new core dependency is added beyond those named in spec, file ADR</trigger>
</adr_triggers>
```
### `<gotchas>`
Stage-specific traps. Project-wide gotchas live in `docs/gotchas.md` and are read via `<read_first>`. Use this tag only for traps unique to this stage that don't generalize.
```xml
<gotchas>
  <trap>lefthook v1.x changed the YAML format from v0.x — use the v1 syntax explicitly</trap>
  <trap>cargo workspace inheritance for lints requires Cargo.toml [workspace.lints], not [lints]</trap>
</gotchas>
```
### `<dependencies>`
Use when a stage depends on artifacts outside the obvious prior-stage chain (e.g., depends on an external review, an upstream branch, an ADR not yet accepted).
```xml
<dependencies>
  <dependency>ADR-0004 (code-signing deferral) must be Accepted before Stage D</dependency>
</dependencies>
```
### `<time_box>`
Estimated wall-clock duration. Informs staging boundaries, not deliverable size (per `BUILD-PLAYBOOK.md` §1.2). Reviewed at retrospective for soft gate S4 (within 2× of actual).
```xml
<time_box estimate_hours="6"/>
```
## 10. Authoring rules
One stage per fenced block. Don't combine stages. The Phase doc may have many fenced blocks but each contains exactly one root element.
No foreign tags. Every tag inside a stage prompt must be in this protocol. Adding a new tag means updating this doc first (and bumping the protocol version per Part 13). Drift is a bug.
No HTML escaping inside `<context>` or prose tags unless required. XML inside fenced markdown blocks parses cleanly with literal angle brackets in attribute values via `&lt;` and `&gt;`. Use them only when the text contains XML-meaningful characters (e.g., "<30 min").
Self-closing for reference tags. When a tag points at an external file with no inline body, use the self-closing form: `<gates milestone="M01"/>` not `<gates milestone="M01"></gates>`.
Stable child element names. Within `<deliverable>`, every child is `<item>`. Within `<scope_locks>`, every child is `<lock>`. Within `<acceptance_criteria>`, every child is `<criterion>`. Within `<execution_steps>`, every child is `<step>`. Within `<read_reference>`, every child is `<file>`. Within `<execution_warnings>`, every child is `<warning>`. Within `<gotchas_graduation>`, every child is `<stage_review>`. Within v1.6 additions: `<coverage_gate>` children are `<gate>`; `<schema_ref_audit>` children are `<ref>`; `<api_breaking_change_audit>` children are `<change>`; `<existing_pattern_audit>` children are `<pattern>`; `<interpretation_declarations>` children are `<adopt>`; `<scope_change>` children are `<descope>` or `<expand>`; `<zustand_selector_audit>` children are `<selector>`; `<test_isolation_audit>` children are `<persistent_slot>`; `<phase_doc_inventory_audit>` v1.6 extensions stay `<claim>` (new `type=` values); `<dependency_audit_check>` v1.6 extension adds a `<feature_interdependency>` sibling to the existing `<dep>` children. Within v1.7 additions: `<tdd_discipline>` children are `<red_phase>` and `<green_phase>`. Within v1.8 additions: `<construction_reachability_check>` children are `<wire>`; `<wire_signature_audit>` children are `<wrapper>`; `<wire_trace_vs_adr_reconcile>` children are `<trace>`; `<phase_doc_inventory_audit>` v1.8 extension stays `<claim>` (new optional `shape=` attribute on `type="store_slot"` claims). This consistency makes validation and aggregation simple.
Order tags consistently across milestones. The recommended order:
For work stages: `<context>` → `<read_first>` → `<read_reference>` (opt) → `<read_prior_milestones>` (Stage A only when applicable) → `<read_prior_stages>` (B+) → `<interpretation_declarations>` (opt, v1.6) → `<deliverable>` → `<test_plan_required>` → `<tdd_discipline>` (opt, v1.7; immediately before `<execution_steps>` because it governs the execution shape) → `<execution_steps>` → `<acceptance_criteria>` → `<scope_locks>` → `<scope_change>` (opt, v1.6) → `<gates>` → `<coverage_gate>` (opt, v1.6) → `<self_correction_budget>` → `<adr_triggers>` (opt) → `<gotchas>` (opt) → `<execution_warnings>` (opt) → `<pre_flight_check>` / `<schema_drift_check>` / `<fan_out_grep>` / `<dependency_audit_check>` / `<runtime_environment>` / `<architecture_check>` / `<schema_audit>` / `<schema_root_check>` / `<phase_doc_inventory_audit>` (opt; cluster these v1.3 + v1.4 + v1.6 authoring-time audit tags together) → `<schema_ref_audit>` / `<api_breaking_change_audit>` / `<existing_pattern_audit>` / `<zustand_selector_audit>` / `<playwright_warmup_recipe>` / `<test_isolation_audit>` (opt, v1.6 authoring-time audit cluster) → `<construction_reachability_check>` / `<wire_signature_audit>` / `<wire_trace_vs_adr_reconcile>` (opt, v1.8 authoring-time audit cluster; cluster these with the v1.3/v1.4/v1.6 audit tags above) → `<time_box>` (opt) → `<dependencies>` (opt) → `<retrospective_requirements>` → `<commit_protocol>` → `<commit_message>` → `<approval_surface>`.
For closeout stages: `<context>` → `<read_first>` → `<read_reference>` (opt) → `<read_prior_milestones>` (rare for closeout; included only if absorbing additional carry-forward) → `<cumulative_reads>` → `<deliverables>` (now includes required `<simplify_pass>` child from v1.6) → `<gap_analysis_requirements>` (with required `<gotchas_graduation>`) → `<append_only_verification>` → `<three_artifact_review>` → `<scope_locks>` → `<gates>` → `<self_correction_budget>` → `<adr_triggers>` (opt) → `<gotchas>` (opt) → `<execution_warnings>` (opt) → `<time_box>` (opt) → `<retrospective_requirements>` → `<commit_protocol>` → `<commit_message>` → `<approval_surface>`.
Consistent ordering makes diffs across milestones immediately scannable.
Reference-first **strict** for content-heavy tags (v1.2 hardening). Tags that support both inline and reference forms — currently `<deliverable>`, `<acceptance_criteria>`, `<scope_locks>`, `<commit_message>`, `<gap_analysis_requirements>` — **must use the reference form when the corresponding Phase doc section exists**. The Phase doc's `X.2 Files to Change`, `X.3 Detailed Changes`, `X.4 Tests`, `X.6 Commit Message`, and milestone-level `Key constraints` sections are the canonical locations for content; the prompt references rather than restates them.
Validator behavior:
- If the Phase doc has a section matching the tag's expected anchor (e.g., `### A.3 Detailed Changes` for the Stage A `<deliverable>`), inline content in that tag is **rejected** (error). Authors must use the reference form.
- If no matching section exists, inline form is permitted (e.g., a single-stage milestone with no `X.3` section, or a stage whose content is genuinely too small to warrant a Phase doc section).
- The validator finds Phase doc sections by markdown-AST heading lookup against the `section="..."` attribute string — renderer-agnostic, slugifier-agnostic.
Section-name resolution (drops URI fragments — v1.2 anchor-stability fix). Reference tags use the `section="..."` attribute, not URL fragment notation. The validator parses the referenced markdown file's AST, finds the heading whose text matches the `section` attribute (case-sensitive, exact match), and confirms the heading exists. Renderer-dependent slugification (`### A.6 Commit Message` → `#a6-commit-message` on GitHub vs `#a-6-commit-message` on GitLab/mdBook vs different again on VS Code preview) becomes irrelevant.
Never use both forms in the same tag. A tag with `ref="..."` must be self-closing (or contain only nested allowed children like `<gotchas_graduation>` for `<gap_analysis_requirements>`); a tag with inline list-content must not have a `ref` attribute. Validation enforces this.
Gotchas graduation rule (v1.2 enforcement). Stage-specific `<gotchas>` are **per-stage scratch space**. Across a milestone, every per-stage `<gotchas>` entry must be evaluated at closeout via `<gotchas_graduation>` (see Section 8) and assigned a disposition: `kept | graduated | resolved | expired`. If a trap recurs in 2+ stages of the same milestone (or across milestones), promote it to `docs/gotchas.md` and remove it from per-stage tags. The closeout `<gotchas_graduation>` slot is the forcing function — without it, per-stage `<gotchas>` would accumulate as discipline decay sets in.
**Cross-stack integration examples must be verified before shipping (v1.2 hardening).** When a stage prompt contains an inline code example for a cross-stack integration — Rust ↔ TS glue, Rust ↔ OS-platform integration (keychain, sandbox, IPC), runtime ↔ third-party protocol (MCP, OAuth, WebDriver), build-tool config (tauri.conf.json, eslint.config.js, vite.config.ts) — the example must either: (a) be quoted verbatim from a known-working upstream example, with the source repo + commit SHA referenced in a comment above the example, OR (b) carry an explicit "verify against upstream reference X before shipping" instruction inside `<execution_warnings>` naming the upstream file or issue to check. Hand-authoring examples from docs/memory is a named failure mode — see `docs/gotchas.md` #32. Pattern bit M02 (PR #45 keyring stub; gotcha #29) and M03 (PR #47 tauri-driver capabilities, three iterations). Pure-Rust and pure-React/TS examples don't trigger this rule; the discriminator is whether the example crosses a stack boundary. Validator does not enforce this rule mechanically (impractical to detect "this is a cross-stack example"); it's an authoring discipline backed by §13 anti-pattern and the gotcha entry.

**v1.3 hardening — pre-flight + drift + fan-out + dep + environment slots are additive and lean.** The five new optional tags introduced in v1.3 are structural-only at the validator level (lean validator pattern continued from v1.2). Authors use them where the corresponding failure mode applies; absence is not flagged. Promotion to required-on-conditions (e.g., "schema_drift_check required if the stage edits `schemas/*.v1.json` or `crates/runtime-core/src/`") is a v1.4+ decision after observing v1.3 in 2+ milestones.

**Phase doc claims about codebase reality must be grep-verified at authoring time (v1.3 hardening).** When a milestone Phase doc names file paths in X.2 "Files to Change" tables OR symbol/function/struct/IPC variant/integration-point claims in X.3 "Detailed Changes" narrative, every claim must be verified via `grep` / `Test-Path` / file inspection at authoring time — BEFORE the Phase doc ships. Trusting `agent-runtime-spec.md` text or `docs/gap-analysis.md` carry-forward chains as proxies for codebase state is a named failure mode: spec describes intent (not necessarily reality); gap-analysis is append-only and items "carry forward" but don't auto-close when work happens incidentally; both can drift from current code reality between milestones. Bit M04 (Phase doc claimed `event_translation.rs`, `prompt_template.rs`, `WriteSignal` IPC variant, `vdr.rs` in `runtime-main`, `RevertToSnapshot` as new — none matched reality; 8 of 11 plan/task events already existed; structurally misaligned across Stages B/D/F; build agent halted at execution time per CLAUDE.md §12). Different discriminator from the cross-stack rule above: that rule is about *third-party* examples; this rule is about claims about *your own* codebase. Mitigation: at authoring time, for every X.2 file path and every X.3 symbol claim, run the verification command and confirm reality before shipping. The "Pre-existing legacy file inventory" section in `TEMPLATE.md` is **verified**, not **descriptive** — see `TEMPLATE.md` addition + `docs/gotchas.md` #41 for the discipline rule. Validator does not enforce this rule mechanically (impractical to detect "this claim is about codebase reality"); it's an authoring discipline backed by the gotcha entry + TEMPLATE.md verification rule.

**Quality-gate execution ordering is canonical at every `<execution_steps>::implement` step (v1.6 hardening).** Run gates in this order at every work stage: (1) `cargo fmt --all`; (2) `cargo clippy --fix --allow-dirty -p <touched-crate>` (mechanical first-pass autofix); (3) `cargo clippy --workspace --all-targets -- -D warnings` (final verification); (4) remaining gates (test, doc, audit, deny, llvm-cov). The autofix step eliminates 6–15 mechanical lints per stage (`too_long_first_doc_paragraph`, `doc_markdown` unbackticked, `missing_const_for_fn`, `manual_let_else`, `items_after_statements`, `option_if_let_else → unnecessary_map_or` cascade). M03 graduated gotcha #34; M05 confirmed at 6 stages of recurrence (B + C1 + C2 + D + E + light at A). v1.6 makes the ordering canonical via a CLAUDE.md §6 paragraph; this protocol rule is the surface in stage prompts. Validator does not enforce mechanically (impractical to detect "gate ordering in implement step"); authoring discipline backed by CLAUDE.md §6.

**Build-machine state must be confirmed before phase-doc edits (v1.4 hardening).** When an orchestration session edits a Phase doc that may affect stages whose execution status is uncertain, the session MUST surface a request for cross-machine state and the user MUST paste output of `git log --oneline main..HEAD` from the build machine on the active milestone branch. The pasted output is authoritative; `origin` is not. Origin is a partial view when §8 forbids per-stage pushes (CLAUDE.md §8 "DO NOT push between stages"); a build machine that has shipped Stages A1 + A2 locally without pushing leaves origin silent on those stages. Inferring "stage X unexecuted" from origin's silence is a banned failure mode. Companion to v1.3 grep-verify rule: that rule covers WHAT codebase claims need verification; this rule covers WHICH codebase to verify against when origin and build-machine diverge. Stronger backstop: every stage end surface includes git log + retrospective file listing as a standard item (CLAUDE.md §19 rule 7) — when the user pastes the surface to any downstream orchestration session, that session sees actual project state, not origin's partial view. Bit M04 (PR #53 rewrote the M04 phase doc against "A1+A2 unexecuted per origin" while the build machine had `c5fe035` + `2b0e8d2` with full retros; 4+ hours of work merged on a false premise; reverted via PR #54). Validator does not enforce this rule mechanically (impractical to detect "this Phase doc edit changes stage status"); it's an authoring discipline backed by `docs/gotchas.md` #42 + the §19 rule-7 surface requirement.

## 11. Validation
A validation script lives at `scripts/validate-stage-prompts.py` (or your preferred language). It runs in CI on every PR that touches `docs/build-prompts/M[NN]-*.md`.
**v1.2 ships lean.** Structural checks are errors (block CI); cross-file resolution checks are warnings (surface in PR check output, do not block). Cross-checks promote to errors in v1.3 once the cross-check logic survives 3+ milestones without false positives.
**Errors (block CI):**
- Extracts every fenced ```xml block from the Phase doc
- Confirms each block contains exactly one root element
- Confirms the root tag is `work_stage_prompt` or `closeout_stage_prompt`
- Confirms `id` attribute matches `M[0-9]{2}\.[A-Z]`
- Confirms all required tags for the schema are present (including `<commit_message>`, `<execution_steps>`, and — for closeout — `<gap_analysis_requirements>` containing `<gotchas_graduation>`)
- Confirms no foreign tags appear
- Confirms reference-first tags use either inline form OR `section="..."` reference form, never both (the v1.0/v1.1 mixing-rule, retained as error)
- **Strict reference-first (v1.2):** if the Phase doc has a section matching the expected anchor for a content-heavy tag (`<deliverable>`, `<acceptance_criteria>`, `<scope_locks>`, `<commit_message>`), inline content in that tag is rejected — author must use reference form
- `<disposition>` values inside `<gotchas_graduation>` must be one of: `kept`, `graduated`, `resolved`, `expired`
- Every prior stage in the milestone must have a `<stage_review id="...">` entry inside the closeout's `<gotchas_graduation>` (counted by parsing the milestone's Phase doc for stage headings)
- For `expired` disposition, `<target>` must include rationale beyond bare `n/a` (a single line minimum; validator checks length > "n/a" alone)
- v1.3 tags `<pre_flight_check>`, `<schema_drift_check>`, `<fan_out_grep>`, `<dependency_audit_check>`, `<runtime_environment>` must appear inside a work-stage prompt only — appearing in a closeout-stage prompt is a structural error
- v1.6: `<simplify_pass>` must appear inside a closeout-stage prompt (the validator's `REQUIRED_BY_ROOT.closeout_stage_prompt` list includes `simplify_pass` from v1.6 onward; phase docs landing under v1.6 must include it)
- v1.6 tags `<coverage_gate>`, `<schema_ref_audit>`, `<api_breaking_change_audit>`, `<existing_pattern_audit>`, `<interpretation_declarations>`, `<scope_change>`, `<zustand_selector_audit>`, `<playwright_warmup_recipe>`, `<test_isolation_audit>` must appear inside a work-stage prompt only — appearing in a closeout-stage prompt is a structural error
- v1.7 tag `<tdd_discipline>` must appear inside a work-stage prompt only — appearing in a closeout- or verifier-stage prompt is a structural error. The `<tdd_discipline strict="true">` → two-commit `<execution_steps>` coupling (the stage's `<execution_steps>` must contain `red_phase_commit` + `surface_for_red_approval` + `green_phase_commit` + `surface_for_final_approval` steps when `<tdd_discipline strict="true">` is present) is an authoring-discipline rule, NOT code-enforced at the v1.7 lean validator level — promote to a validator cross-check at v1.8+ once 2+ milestones show clean signal (lean-validator pattern continued from v1.3/v1.4/v1.5/v1.6; the validator passes the optional tag through structurally without allowlisting, exactly as it does for the nine v1.6 optional slots)
- v1.8 tags `<construction_reachability_check>`, `<wire_signature_audit>`, `<wire_trace_vs_adr_reconcile>` must appear inside a work-stage prompt only — appearing in a closeout-stage prompt is a structural error. All three are structural pass-through + honor-system at v1.8 (the validator's regex parser passes them through without allowlisting, identical lean treatment to the nine v1.6 optional slots + v1.7 `<tdd_discipline>`); their child-body content (`<wire>` / `<wrapper>` / `<trace>` attributes) and the v1.8 `<phase_doc_inventory_audit shape=...>` extension are NOT code-enforced — promote to a validator cross-check at v1.9+ once 2+ milestones show clean signal. **Per the §15 v1.8 changelog §G maintainer decision, the deferred v1.7 `<tdd_discipline strict="true">` → two-commit coupling promotion stays deferred to v1.9 as well** (M06 + the M06.5 fix-cycle is ~1 milestone of clean signal, short of the stated "2+ milestones" bar; M07 supplies the second).
**Warnings (surface in PR output, don't block):**
- Confirms ordering matches the recommended order
- Cross-checks: every retrospective referenced in `<read_prior_stages>` exists; every milestone in `<read_prior_milestones>` has the named gap-analysis section + summary section; every file in `<read_first>` and `<read_reference>` exists; every `section="..."` value on a reference tag resolves to a real Phase doc heading via markdown-AST lookup; the milestone in `<gates milestone="...">` matches the Phase doc's milestone
- `<read_reference>` entries without a `purpose` attribute (warning in v1.2; promotes to error in v1.3)
- Recognized `<execution_steps>` step names (`write_failing_tests`, `implement`, `verify_gates`, `fill_retrospective`, `surface`); custom step names emit a warning encouraging Phase doc documentation
- v1.3 tags' child elements with required-but-missing attributes (`<check>` without `name`, `<grep>` without `pattern`/`purpose`, `<dep>` without `name`, `<runtime_environment>` without `os`) emit warnings; promote to errors in v1.4 once the cross-check logic has 2+ milestones of clean signal
- v1.6 tags' child elements with required-but-missing attributes (`<gate>` without `scope`/`target_lines`, `<ref>` without `schema`/`path`, `<change>` without `api`/`before_signature`/`after_signature`, `<pattern>` without `grep_for`/`rationale`, `<adopt>` without `spec_section`/`interpretation`, `<descope>` without `deliverable`/`reason`, `<feature_interdependency>` without `crate`/`function`/`home_feature`/`requires_features`) emit warnings; promote to errors in v1.8+ once the cross-check logic has 2+ milestones of clean signal
**Section-name resolution (replaces URI-fragment lookup — v1.2 anchor-stability fix).** The validator parses the referenced markdown file's AST, finds the heading whose text matches the `section="..."` attribute (case-sensitive, exact match), and confirms the heading exists. Renderer-agnostic. The fragment notation (e.g., `ref="...md#A.6"`) is no longer recognized; v1.2 prompts must use `ref="...md" section="A.6 Commit Message"`. v1.0-grandfathered prompts (M01-M02) skip this check via the version banner in the Phase doc header (see Authoring Rules §10 grandfathering).
CI fails on any error; warnings are surfaced in the PR check output.
## 12. Worked examples
**Note:** these examples illustrate v1.2 syntax (section-name refs, `<execution_steps>`, `<read_reference>`, `<execution_warnings>`, closeout `<gotchas_graduation>`). The actual M01-foundation.md and M02-event-pipeline.md prompts in the repo are v1.0-grandfathered and use the older syntax — see Authoring Rules §10 grandfathering. v1.2 applies to M03 forward.
### 12.1 Work-stage prompt — M03.A (hypothetical, Live Graph milestone)
A non-first milestone, Stage A — absorbs M02 carry-forward and references the Phase doc's `A.3 Detailed Changes` + `A.4 Tests` + milestone-level `Key constraints` sections via the strict reference-first pattern.
```xml
<work_stage_prompt id="M03.A">
  <context>
    Stage A of M03 (Live Graph). Build hygiene + scaffolds. Absorbs M02 carry-forward
    🟡 Important items so Stages B–E focus on the real M03 deliverables (React Flow integration,
    node types, VDR projection). Stage B does not start until Stage A's commit is on the
    milestone branch.
  </context>

  <read_first>
    <file>BUILD-PLAYBOOK.md</file>
    <file>docs/identity.md</file>
    <file>docs/gates.md</file>
    <file>spec/agent-runtime-spec.md §0–§0d, §3</file>
    <file>docs/MVP-v0.1.md §M03</file>
    <file>docs/build-prompts/M03-live-graph.md (Background, Document Structure, Implementation Workflow, Stage A sections A.1–A.4)</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md</file>
  </read_first>

  <read_reference>
    <file purpose="event taxonomy archetype">crates/runtime-core/src/events.rs</file>
    <file purpose="renderer IPC pattern">src/ipc/events.ts</file>
  </read_reference>

  <read_prior_milestones>
    <gap_analysis_carry_forward milestone="M02"/>
    <milestone_summary milestone="M02" section="Decisions to apply before next parent milestone"/>
  </read_prior_milestones>

  <deliverable ref="docs/build-prompts/M03-live-graph.md" section="A.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M03-live-graph.md" section="A.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M03-live-graph.md" section="Key constraints"/>

  <scope_change>
    <descope deliverable="ApprovalPanel renderer wiring" reason="M02 Important items consume Stage A's budget; ApprovalPanel is Stage C scope per phase doc" carry_forward_to="M03 Stage C" authorized_by="phase doc Stage A scope_locks"/>
  </scope_change>

  <gates milestone="M03"/>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>Stage A's job is to close M02-carry-forward Important items, not to start Stage B's React Flow work — resist scope creep into graph rendering even if locally tempting</trap>
    <trap>React Flow v12 changed the edge-update API from v11 — pin version explicitly in package.json</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT run `npm run dev` in CI flow — it spawns a long-running server. Use `npm run build` for build-time validation instead.</warning>
    <warning>Frontend coverage runs (`npm run test -- --coverage`) take 1–2 min on cold cache — budget accordingly</warning>
  </execution_warnings>

  <time_box estimate_hours="2"/>

  <retrospective_requirements ref="prompts/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Decisions for Stage B: which event-to-node mapping conventions Stage B will inherit; whether the React Flow v12 pinning held; whether the carry-forward sweep closed all M02 Important items</special_log>
  </retrospective_requirements>

  <commit_protocol ref="BUILD-PLAYBOOK.md" section="4.7 Do-Not-Commit Rule"/>
  <commit_message ref="docs/build-prompts/M03-live-graph.md" section="A.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers)</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage B)</item>
    <item>draft commit message from M03-live-graph.md A.6 Commit Message section (filled with session URL)</item>
    <item>explicit statement: "Stage M03.A is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```
### 12.2 Closeout-stage prompt — M03.F (hypothetical)
Demonstrates the v1.2 closeout shape, including the required `<gotchas_graduation>` subsection inside `<gap_analysis_requirements>`.
```xml
<closeout_stage_prompt id="M03.F">
  <context>
    Closeout stage of M03 (Live Graph). Stages A–E have committed on the milestone branch.
    This stage produces the cumulative artifacts: M03 summary aggregating retrospectives,
    the new docs/gap-analysis.md entry (third in the ledger after M01 and M02), and the
    draft PR description. The gap-analysis commit is the final commit on this branch and
    gates the PR push.
  </context>

  <read_first>
    <file>BUILD-PLAYBOOK.md (especially §3.4, §4.6)</file>
    <file>docs/identity.md</file>
    <file>docs/gates.md</file>
    <file>spec/agent-runtime-spec.md §0–§0d, §3</file>
    <file>docs/MVP-v0.1.md §M03</file>
  </read_first>

  <cumulative_reads>
    <codebase>entire shipped codebase to date (cumulative across M01 + M02 + M03.A–M03.E commits)</codebase>
    <spec>spec/agent-runtime-spec.md (end-to-end, focus on M03-touched sections)</spec>
    <gap_analysis>docs/gap-analysis.md (M01 and M02 prior entries; M03.F appends the third)</gap_analysis>
    <retrospectives>retrospectives/M03.A-retrospective.md, M03.B-, M03.C-, M03.D-, M03.E-retrospective.md (all work stages)</retrospectives>
  </cumulative_reads>

  <deliverables>
    <milestone_summary>retrospectives/M03-summary.md (aggregates per-stage retrospectives; scores axes across stages; marks verdict)</milestone_summary>
    <gap_analysis_entry>docs/gap-analysis.md (append third entry; six required sections, none optional; Carry-forward section addresses M01+M02 open items by status)</gap_analysis_entry>
    <simplify_pass>
      <invoke skill="simplify" against="milestone cumulative diff (M03.A..HEAD)"/>
      <surface kind="refactor_proposals" examples="duplication / dead code / parallel API surfaces / modules grown across stages / premature abstractions"/>
      <approval_required>true</approval_required>
      <commit_on_approval>focused refactor commit on same branch before PR opens</commit_on_approval>
      <defer_unapproved_to>docs/tech-debt.md (per ADR-0008 🟢 ledger)</defer_unapproved_to>
    </simplify_pass>
    <pr_description>draft only; PR opens only on explicit human ask after approval</pr_description>
  </deliverables>

  <gap_analysis_requirements ref="BUILD-PLAYBOOK.md" section="3.4 Gap Analysis Entry">
    <gotchas_graduation>
      <stage_review id="A">
        <gotcha>React Flow v12 edge-update API breakage from v11</gotcha>
        <disposition>graduated</disposition>
        <target>docs/gotchas.md §21 (frontend version pinning)</target>
      </stage_review>
      <stage_review id="B">
        <gotcha>Node-type registry collision when two stages register same key</gotcha>
        <disposition>resolved</disposition>
        <target>commit a7c2f4e (added duplicate-key check)</target>
      </stage_review>
      <stage_review id="C">
        <gotcha>VDR projection state initialized before any events arrive</gotcha>
        <disposition>kept</disposition>
        <target>stays in per-stage gotchas of M04.A (plan integration touches same surface)</target>
      </stage_review>
      <stage_review id="D">
        <gotcha>None observed.</gotcha>
        <disposition>n/a</disposition>
        <target>n/a — stage produced no per-stage gotchas</target>
      </stage_review>
      <stage_review id="E">
        <gotcha>Playwright headless mode interferes with React Flow viewport calculation</gotcha>
        <disposition>expired</disposition>
        <target>n/a — Stage E was the only stage using Playwright with React Flow; no forward stage uses both</target>
      </stage_review>
    </gotchas_graduation>
  </gap_analysis_requirements>

  <append_only_verification>
    <local_check>prior content of docs/gap-analysis.md (M01 + M02 entries) must be a literal prefix of HEAD before commit</local_check>
    <ci_check name="gap-analysis-append-only">verify the M01 + M02 entries are byte-identical to their committed state; fail otherwise</ci_check>
  </append_only_verification>

  <three_artifact_review>
    <artifact>code diff (cumulative M03.A through M03.F)</artifact>
    <artifact>per-stage retrospectives + M03 milestone summary</artifact>
    <artifact>new docs/gap-analysis.md entry — flagged "IMMUTABLE once committed"</artifact>
    <pushback_blocks_pr>true</pushback_blocks_pr>
  </three_artifact_review>

  <scope_locks>
    <lock>Append-only is a hard rule (BUILD-PLAYBOOK.md §4.1, §4.6) — no editing M01 or M02 prior entries, ever</lock>
    <lock>The `<gotchas_graduation>` subsection must list every prior stage of M03, even those with no gotchas (write "None observed.")</lock>
  </scope_locks>

  <gates milestone="M03"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>The "Carry-forward" section in the gap-analysis entry is required even when empty — write "None observed." rather than omit (BUILD-PLAYBOOK.md §3.4)</trap>
    <trap>Severity is non-elastic — if M03 has a pile of 🔴 Criticals in the fix backlog, the milestone shouldn't ship; surface this rather than rationalize</trap>
  </gotchas>

  <time_box estimate_hours="4"/>

  <retrospective_requirements ref="prompts/RETROSPECTIVE-TEMPLATE.md"/>
  <commit_protocol ref="BUILD-PLAYBOOK.md" section="4.7 Do-Not-Commit Rule"/>
  <commit_message ref="docs/build-prompts/M03-live-graph.md" section="F.6 Commit Message"/>

  <approval_surface>
    <item>new gap-analysis entry text (full)</item>
    <item>diff of docs/gap-analysis.md (proves append-only — only new lines at bottom)</item>
    <item>M03-summary.md (full)</item>
    <item>draft PR description (per .github/PULL_REQUEST_TEMPLATE.md)</item>
    <item>draft commit message from M03-live-graph.md F.6 Commit Message section</item>
    <item>explicit flag: "This gap-analysis entry is IMMUTABLE once committed. Please review carefully."</item>
    <item>explicit statement: "M03 closeout is ready. I will not commit until you approve."</item>
  </approval_surface>
</closeout_stage_prompt>
```
## 13. Anti-patterns
Stage prompts that look right but aren't. These are the failure modes worth naming.
Vague `<context>`. "Build foundation stuff." Prompt is unusable; agent has to invent the framing. Two to four sentences naming the milestone, the stage, what builds on it.
Aspirational `<deliverable>`. "A great Cargo workspace." If you can't enumerate the items, the stage isn't ready to start. Either decompose the work first or split into more stages.
`<acceptance_criteria>` that restate `<gates>`. Acceptance criteria are behavioral checks beyond the gate suite. "cargo test passes" belongs in `<gates milestone="M01"/>` (which references the gate matrix). "lefthook hook fires on a test commit and blocks malformed input" is a behavioral acceptance check — that's what belongs here.
Missing `<read_prior_stages>` on Stage B+. Single most common protocol drift. Per `BUILD-PLAYBOOK.md` §4.5, Stage B+ must read prior retrospectives' Decisions sections before any code. Omitting the tag from the prompt makes this rule invisible.
`<scope_locks>` repeated from spec verbatim. The locks should be the active constraints for this stage, not a copy of the entire spec scope section. Reference broad scope via `<read_first>`; use `<scope_locks>` for the specific things this stage might tempt the agent to violate.
Closeout missing `<append_only_verification>`. The whole point of the closeout is the immutable ledger entry. Omitting the verification tag means the ledger could land mutated and the next milestone inherits a corrupted history.
Closeout with a `<deliverable>` (singular) tag. That's the work-stage tag. Closeouts use `<deliverables>` (plural) because they always produce three: summary, ledger entry, draft PR description. If your closeout has only one, it's not a closeout.
Foreign tags introduced silently. Adding `<priority>` or `<owner>` or `<estimate>` ad-hoc means future stages won't have them and validation breaks across milestones. New tags require updating this protocol first (and bumping its version).
`<gates>` with the `milestone` attribute pointing at the wrong milestone. Copy-paste error from a prior milestone's prompt. The CI validator catches this; humans miss it.
`<approval_surface>` reordered without reason. Order matters because the human reads top-down. The recommended order has diff first (because that's what the human cares about most) and the "I will not commit" statement last (because it's the verbal anchor of the do-not-commit rule). Reorder only with a stated reason.
Missing `<commit_message>`. Every stage prompt requires a `<commit_message>` slot — almost always referencing the pre-authored commit in the Phase doc's `X.6 Commit Message` section. Omitting it means the agent drafts a commit ad-hoc, which produces inconsistent commit-message style across milestones and forces the human to evaluate each one as a separate review item.
`<read_prior_milestones>` on Stage B+ within the same milestone. Stage B+ uses `<read_prior_stages>` for within-milestone retrospective reads. `<read_prior_milestones>` is for absorbing prior milestone carry-forward — overwhelmingly Stage A of a non-first milestone. Putting it on Stage B+ is a sign of confusing the two read patterns; the validator catches this.
`<tdd_discipline strict="true">` with single-commit `<execution_steps>` (v1.7). The slot declares the strict two-commit contract; if the `<execution_steps>` still reads `write_failing_tests → implement → verify_gates → surface` (single commit), the slot is decorative and the audit-surface invariant cannot hold. When `<tdd_discipline strict="true">` is present, `<execution_steps>` MUST contain `red_phase_commit` + `surface_for_red_approval` + `green_phase_commit` + `surface_for_final_approval`. The whole point of the slot (per TDAD arXiv:2603.17973) is that *structural* enforcement — not exhortation — is what makes TDD work for LLM agents; a slot without the matching execution shape is exhortation wearing a slot's clothes.
Green-phase commit that modifies the red-phase test files (v1.7). The load-bearing audit property is `git diff <red-sha>..<impl-sha> -- '**/tests/**'` being EMPTY. A green-phase commit that "tidies" or "fixes" the red-phase tests destroys the property — and is precisely the failure mode Anthropic's TDD docs warn about ("Claude will sometimes change tests to make them pass rather than fixing the implementation"). Net-new additive tests + mechanical formatting belong in a SEPARATE labelled follow-up commit; the impl commit touches zero existing test files. An impl commit body that omits the explicit empty-diff invariant statement is a softer instance of the same anti-pattern (the invariant is unverifiable-by-inspection without the stated command).
`<read_prior_milestones>` on Stage A of the first milestone. M01.A has no prior milestone to absorb. The tag is omitted entirely; not "empty" but absent. (Same rule as `<read_prior_stages>` being absent on Stage A.)
Mixing inline and reference forms in the same tag. `<deliverable ref="...md" section="..."><item>...</item></deliverable>` is a schema violation. Pick one form. The validator rejects the mix because the precedence rule (which wins?) is genuinely ambiguous and the right answer is to make the choice explicit at authoring time.
**v1.2 anti-patterns (new):**
Inline content in a tag whose Phase doc section exists (v1.2 strict reference-first). If `M03-live-graph.md` has a heading `### A.3 Detailed Changes`, the Stage A `<deliverable>` must use `ref="docs/build-prompts/M03-live-graph.md" section="A.3 Detailed Changes"` — inline `<item>` lists are rejected by the validator. The drift failure mode (prompt and Phase doc diverging) is the reason; v1.2 makes the rule strict instead of advisory.
URI-fragment ref form (e.g., `ref="...md#A.3"`) instead of section-name form (`ref="...md" section="A.3 Detailed Changes"`). The fragment form is renderer-dependent (GitHub vs GitLab vs mdBook vs VS Code preview each slugify differently) and brittle — v1.2 drops it entirely. Use `section="..."` and let the validator resolve the heading via markdown-AST lookup. v1.0-grandfathered prompts (M01-M02) are exempt via the version banner in their headers.
Missing `<execution_steps>`. v1.2 requires this slot in every work stage prompt. Omitting it means the procedural sequence (write_failing_tests → implement → verify_gates → fill_retrospective → surface) is invisible to the agent — it has to derive the cycle from the playbook each time. The slot is the procedural anchor.
Missing `<gotchas_graduation>` in closeout. v1.2 requires the subsection inside `<gap_analysis_requirements>`. Without it, per-stage `<gotchas>` accumulate across milestones with no forcing function for graduation to `docs/gotchas.md` — discipline decay sets in by M05 or M06.
`<gotchas_graduation>` missing a stage. Every prior stage in the milestone must have a `<stage_review id="...">` entry, even if the stage had no gotchas (use `<gotcha>None observed.</gotcha><disposition>n/a</disposition>` in that case). Validator catches missing stages by counting stage headings in the Phase doc.
Foreign `<disposition>` value. The enum is exhaustive: `kept | graduated | resolved | expired`. Anything else (`promoted`, `archived`, `closed`, `wontfix`) is a schema violation. Validator rejects.
`<disposition>` of `expired` without rationale in `<target>`. The `expired` disposition is the safety valve for stage-local traps with no forward applicability — but it's also the easiest disposition to abuse ("expire" everything to skip the work of evaluating `kept`/`graduated`). The validator requires `<target>` to contain text beyond bare `n/a` (a single line of rationale minimum). Authors who can't articulate why a trap doesn't apply forward probably haven't actually evaluated it.
`<read_reference>` without `purpose` attribute. Each `<file>` inside `<read_reference>` must have a `purpose="..."` attribute naming *why* the agent reads it. Without `purpose`, the slot degrades into "miscellaneous reads" and loses its discriminator value vs `<read_first>`. Validator warns in v1.2; promotes to error in v1.3.
`<execution_warnings>` used for `<gotchas>` content (or vice versa). The distinction matters: `<gotchas>` warns about code-shape traps the agent might write into a file; `<execution_warnings>` warns about *commands* the agent might run during the stage. "Use `[workspace.lints.rust]` not `[lints]`" is a `<gotchas>` entry (artifact-shape trap). "Don't run `cargo test --features integration` — hits live API" is an `<execution_warnings>` entry (command-time guardrail). Mixing them loses the action-vs-artifact discriminator.
Inline cross-stack integration example without upstream-verification guard (v1.2). Symptom: build agent ships the prompt's example verbatim, CI fails with a setup-shaped error (capability mismatch, "no binary at...", silent stub backend, "Failed to match capabilities"). Pattern bit M02 PR #45 (keyring 3.x stub backend) and M03 PR #47 (tauri-driver capabilities, three iterations). Authoring rule §10 ("Cross-stack integration examples must be verified before shipping") covers the prevention; this anti-pattern names the failure shape so future authors recognize it on inspection. The fix is at the prompt-authoring layer, not the execution layer — the build agent is doing exactly what it's told.

**v1.3 anti-patterns (new):**

`<pre_flight_check>` used as a substitute for `<read_first>`. Pre-flight checks are environmental verifications (branch, prior-commit, env vars, deps); they are NOT for orientation reads. If your "check" is "agent reads CLAUDE.md", that's `<read_first>`. If your "check" is "git rev-parse HEAD matches expected branch", that's `<pre_flight_check>`. The discriminator is whether the check fails imperatively (command exit non-zero) vs informationally (read the file).

`<fan_out_grep>` enumerating the change set instead of finding consumers. Fan-out grep is for finding consumers of a name BEFORE the rename — to verify nothing is missed. If your `<grep>` patterns are the same as the files in `<deliverable>`, you've conflated "files I'll edit" with "files that reference the thing I'm renaming." The grep should find files OUTSIDE `<deliverable>` that need coordinated updates (or confirm none exist).

`<runtime_environment>` declared without command variants when the prompt body contains OS-specific commands. If the prompt body has `Select-String` commands, declare `os="windows"` (single-OS pin) or provide `<command os="...">` variants for the supported OSes. Authors who pin to one OS without rationale in the `note` attribute are setting a trap for future authors who fork the prompt to a different platform.

**v1.4 anti-patterns (new):**

`<architecture_check>` enumerating WHAT-claims instead of HOW-claims. Architecture checks verify *invariants about codebase structure* (which process owns state, which IPC path carries data, which seam is in-process). They are NOT a substitute for `<fan_out_grep>` or grep-verify-codebase discipline (gotcha #41). If your `<claim>` reads "the file `X.rs` exists", that's a `<phase_doc_inventory_audit>` row OR a `<pre_flight_check>` check — not an architecture claim. Architecture claims start with "X resolves Y in-process via Z" or "DroneCommand has no W variant" or "main holds the HITL seam state, not drone."

`<schema_audit>` surveying only the proposed-type-name without surveying neighboring schemas. The point of the audit is to catch *collisions* with existing `$defs` declarations in already-shipped schemas. If your `<survey>` patterns only match the new schema's own root type, you've surveyed the work product not the existing surface. Survey patterns must match `"<TypeName>"` literals across `schemas/*.v1.json` to enumerate prior declarations.

`<schema_root_check>` skipped for "obviously concrete" schemas. The check is cheap (one `jq` invocation per new schema) and the failure mode (top-level `$ref` breaks `json-schema-to-typescript`) is opaque when it bites. Include the slot for every new `schemas/*.v1.json` authoring stage — there's no upside to skipping.

`<phase_doc_inventory_audit>` treating `status="exists"` as informational. The whole point of the audit is to fail the stage if a file claimed as "exists" doesn't, or a file claimed as "new" already does. If your stage runs without the validator asserting on `git ls-files` mismatches, the audit slot is decoration. The `status` enum is load-bearing: `exists` MUST verify file presence, `new` MUST verify file absence (the file is created during the stage), `deleted` MUST verify file presence (the file is removed during the stage), `renamed` MUST verify both old-path absence and new-path presence at stage end.

**v1.6 anti-patterns (new):**

`<scope_change>` used as the only descope-disclosure mechanism (without `<execution_warnings>` or commit-message disclosure). `<scope_change>` exists so Stage V's bias-guarded read-list can pick up the descope without crossing the V protocol's fresh-context line. It is NOT a replacement for noting the descope in the stage's commit message + retrospective + (for waiver-dispute cases) ADR-0008 waiver-as-ADR artifact. The slot is the artifact-level structural fix; the build-agent discipline of noting descopes in commit + retro is still required. Authors who treat `<scope_change>` as "the only place I need to write this" are reintroducing the discoverability gap V's bias guard was designed to expose.

`<coverage_gate>` content drifting from `.github/workflows/ci.yml`. The slot names the exact `--ignore-filename-regex` argument; CI ALSO names the same argument. Authors who update the prompt without updating the workflow (or vice versa) reintroduce the prose-vs-regex translation drift the slot was added to eliminate. Stage-end retrospective's `[END] Coverage holdouts` section's "CI workflow drift check" subsection is the forcing function — confirm both sources agree at every stage commit.

`<interpretation_declarations>` used to declare an interpretation that's actually unambiguous. The slot exists for spec sections with two plausible readings (M05.D §8.security L4 runtime-vs-install-time). If the spec is clear, you don't need the slot — using it anyway pads the prompt and reduces the slot's discriminator value. Reserve for genuinely ambiguous sections; otherwise apply the obvious reading without declaration.

`<existing_pattern_audit>` used to grep for callers of a name (`<fan_out_grep>`'s job). Enum-variant additions break SHAPES that mention an existing variant in an irrefutable position — that's what `<existing_pattern_audit>` finds. Renames break CONSUMERS of the renamed item — that's what `<fan_out_grep>` finds. Conflating them produces grep results that miss the actual breakage class. Watch for: a `<pattern grep_for="<TypeName>">` is a `<fan_out_grep>` not an `<existing_pattern_audit>`.

`<simplify_pass>` skipped on the milestone branch with "no refactoring needed this milestone". The pass is the recurring structural refactor checkpoint; it runs every milestone whether refactor proposals surface or not. Empty pass output ("no proposals surfaced") is a valid outcome — but the pass must run and the outcome must be recorded. Skipping the pass entirely is the anti-pattern; running it and producing zero proposals is fine.

`<simplify_pass>` proposals applied without maintainer approval. The `<approval_required>true</approval_required>` child is a hard contract. Refactors that change observable behavior at all (renames in public APIs, removed dead code that was actually still referenced, premature-abstraction collapses that break dependents) are exactly the class that require human review. Applying without approval before PR open defeats the pass's purpose.

## 14. Verifier-only tags (v1.5+)

These tags are valid only inside `<verifier_stage_prompt>`. The verifier schema deliberately diverges from the work + closeout schemas — same root-attribute conventions (`id`, format `M[NN].V`), but the body shape is structured around the four verification passes rather than build deliverables. See ADR-0008 for the design rationale.

The verifier schema requires these tags:
- `<context>` (same as common §6)
- `<read_first>` (same as common §6 — but with the bias-guard rule below)
- `<scope_to_verify>` (new — replaces `<deliverable>` for V's role)
- `<verification_passes>` (new — wraps the four pass declarations)
- `<findings_format>` (new — declares the structured-output shape V produces)
- `<merge_gate>` (new — declares 🔴/🟡/🟢 severity model + D.fix iteration cap)
- `<gates milestone="M[NN]"/>` (same as common §6)
- `<retrospective_requirements>` (same as common §6 — references the verifier-retro template)
- `<commit_protocol>` (same as common §6)
- `<commit_message>` (same as common §6 — typically `verify(MNN): findings — N🔴 N🟡 N🟢`)
- `<approval_surface>` (same as common §6 — verifier-flavored item ordering)

The verifier schema forbids:
- `<read_prior_stages>` — the per-stage retrospectives are bias-loaded; reading them defeats the fresh-context guard
- `<deliverable>` — V produces findings, not build artifacts (use `<scope_to_verify>` instead)
- `<test_plan_required>` — V is the test plan, not a stage that adds tests

### `<read_first>` (verifier-specific authoring rule)

Required, but with one rule that distinguishes the verifier schema from work + closeout. The verifier's `<read_first>` MUST include:
- The phase doc (what was promised)
- The spec sections it implements (what was contracted)
- Pointers to the current code (what shipped — typically resolved by the agent at runtime via `git ls-files` rather than enumerated)

The verifier's `<read_first>` MUST omit:
- Per-stage retrospectives (`docs/build-prompts/retrospectives/M[NN].*-retrospective.md`)
- The milestone summary (`docs/build-prompts/retrospectives/M[NN]-summary.md`)
- Prior gap-analysis entries (the verifier reads the spec, not what prior milestones decided about the spec)

This is the bias guard: the verifier agent shows up knowing what was supposed to ship, not the work narrative that justified what did ship. The clear-and-paste session pattern (the user clears the CLI session and pastes the V prompt fresh) is the structural enforcement; the `<read_first>` omission is the artifact-level discipline.

Validator behavior (v1.5 lean): structural — error if any of the forbidden retros/summary/gap-analysis paths appear in `<file>` children.

Example:

```xml
<read_first>
  <file>STAGE-PROMPT-PROTOCOL.md (v1.5 — for the verifier schema you are running under)</file>
  <file>docs/build-prompts/M[NN]-<short-title>.md (the phase doc — every X.1 problem statement, X.2 files, X.3 changes, X.4 tests)</file>
  <file>agent-runtime-spec.md (the spec sections the milestone implements)</file>
  <file>docs/adr/0008-milestone-stage-v-verifier.md (this stage's design rationale)</file>
  <file>docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md</file>
  <file>docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md (the parameterized prompt this stage instantiates)</file>
  <!-- NO prior retros. NO milestone summary. NO gap-analysis entries. -->
</read_first>
```

### `<scope_to_verify>`

Required. Replaces `<deliverable>` for the verifier role. Enumerates the files + spec sections the four passes operate on. Inline form lists files + sections explicitly; reference form points at a Phase doc section that does the same.

Reference form (preferred — keeps source-of-truth in the Phase doc):

```xml
<scope_to_verify ref="docs/build-prompts/M[NN]-<short-title>.md" section="V.2 Scope to verify"/>
```

Inline form (one-or-two-item scopes — rare for V; V usually covers a whole milestone):

```xml
<scope_to_verify>
  <files>crates/runtime-main/src/drone_ipc/*.rs</files>
  <files>src/components/nodes/*.tsx</files>
  <spec_sections>§1d (drone IPC), §3 (visual design), §2a (budget)</spec_sections>
</scope_to_verify>
```

### `<verification_passes>`

Required. Wraps the five pass declarations. Each pass is a `<pass>` child with a `name="..."` attribute and inline detail. Passes run in order; later passes may consume findings from earlier passes (e.g., Pass 3 Behavior tests primitives surfaced by Pass 1 Inventory).

```xml
<verification_passes>
  <pass name="inventory">
    Every file the phase doc said would be created / modified — does it exist in `git ls-files` and match the shape claimed in X.2 + X.3? Missing files → 🔴. Files present but empty/stub → 🟡. Files present with wrong scope → 🟡.
  </pass>
  <pass name="wire">
    Every spec claim must have a verifiable code path. Use 5-step data-path tracing:
    (1) Pick a spec claim (e.g., "node size scales with token spend").
    (2) Identify the source event / API surface (`agent_complete.tokens_total`).
    (3) Identify the projection / store path (`graphStore.ts` writes `tokensTotal`).
    (4) Identify the consumer (`AgentNode.tsx` should read `tokensTotal`).
    (5) Verify the consumer reads what the projector writes.
    Trace breaks at step 4 with zero matching consumers OR multiple plausible consumers → 🔴 ("wire incomplete" or "ambiguous"). Forces the build agent to either fix the wire or file an ADR-class waiver explaining why the projection is unused.
  </pass>
  <pass name="behavior">
    Runtime-render check. For each user-observable primitive, exercise it with a synthetic harness and observe the output. Renderer: Vitest + jsdom DOM-render with computed-style inspection. IPC: integration test with real subprocess + duplex pair. Static analysis is insufficient for this class — bugs where every static check passes but the running thing is broken (M04 BudgetHeaderBar-CSS) are caught here, not in Pass 1 or 2.
  </pass>
  <pass name="multi_call_invariants">
    Every public API / IPC method / Tauri command survives "called twice in sequence." Build agent declares the public surface in the Phase doc's V.3 section; verifier asserts a sequential-call test exists OR runs one inline. M04 IRL drone IPC bug (`take_event_stream` single-use) is the canonical case.
  </pass>
  <pass name="assembled_execution">
    RUN the assembled app / assembled integration tests and OBSERVE each milestone primitive EXECUTE — not just confirm a test exists or reads green. Static + unit + wire + multi-call green is insufficient: M08.6 shipped "Sound, 0🔴" through full A–F + Stage V + closeout while the assembled app was never run, and IRL found 7🔴 (built-in tools emitted `ToolInvoked` but never executed; `save_framework` dropped companion files). For each user-observable primitive, drive the REAL path (e2e-tauri / assembled integration) and confirm the behavior, not the event. A "Sound" verdict that did not run the assembled path is FORBIDDEN (CLAUDE.md §4 rule 11); the verdict states explicitly what was NOT exercised.
  </pass>
</verification_passes>
```

Validator behavior (v1.5 lean): structural — error if `<verification_passes>` is missing or empty; error if any `<pass>` child lacks the `name` attribute. The pass-name enum is documented (v1.9: five — `inventory` | `wire` | `behavior` | `multi_call_invariants` | `assembled_execution`) but NOT code-enforced (the validator never carried the four-name enum in code — the fifth pass is additive, no validator change).

### `<findings_format>`

Required. Declares the structured-output shape the verifier produces. Inline form lists the sections and severity model; reference form points at the verifier-retro template (the canonical source).

Reference form (preferred — keeps the format definition in the template):

```xml
<findings_format ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md" section="Findings"/>
```

Inline form:

```xml
<findings_format>
  <section name="per_pass_summary">N files inventoried; N hook claims data-path-traced; N behavior tests exercised; N multi-call invariants verified.</section>
  <section name="findings_list">Per finding: severity 🔴/🟡/🟢, pass that surfaced it, primitive affected, observed-vs-expected, recommended action.</section>
  <section name="merge_recommendation">"Proceed to E (closeout)" | "Open D.fix for 🔴 findings (cite finding numbers)" | "Re-tier — fix is broader than the bug."</section>
</findings_format>
```

### `<merge_gate>`

Required. Declares how findings interact with the milestone PR merge gate. Self-closing with three attributes:
- `red_blocks="true"` — 🔴 findings block the milestone PR merge; require a D.fix iteration to resolve. Always true in v1.5.
- `dfix_iteration_cap="2"` — maximum D.fix iterations before escalation to maintainer. The cap is the structural signal that a fix needs design, not patching.
- `waiver_path="docs/adr/NNNN-waiver-M[NN]-finding-N.md"` — the build agent's escape valve. If V flags 🔴 but the build agent disputes on interpretation grounds, the build agent files an ADR-class waiver with one-paragraph reasoning; maintainer adjudicates.

```xml
<merge_gate red_blocks="true" dfix_iteration_cap="2" waiver_path="docs/adr/NNNN-waiver-M[NN]-finding-N.md"/>
```

🟡 findings carry forward to the next milestone's Stage A (per `CLAUDE.md` §20 gap-analysis carry-forward). 🟢 findings land in `docs/tech-debt.md` (an append-only ledger distinct from gap-analysis and gotchas).

Validator behavior (v1.5 lean): structural — error if any of the three required attributes are missing; warning if `dfix_iteration_cap` is not a positive integer.

### `<commit_message>` (verifier-specific format)

Same tag as common §6 (typically reference form `ref="docs/build-prompts/M[NN]-<short-title>.md" section="V.6 Commit Message"`). The convention for the verifier's commit message:

```
verify(MNN): findings — N🔴 N🟡 N🟢

<one-paragraph summary of what V exercised and the headline finding count>

[per-finding detail, mirroring the verifier-retrospective body]

https://claude.ai/code/session_<id>
```

### `<approval_surface>` (verifier-specific item ordering)

Same tag as common §6, but the item ordering differs from work/closeout:

```xml
<approval_surface>
  <item>cross-machine state (build machine git log + retrospective file listing)</item>
  <item>findings list, sorted by severity (🔴 first, then 🟡, then 🟢)</item>
  <item>per-pass summary (N files inventoried; N wires traced; N behaviors exercised; N multi-call invariants checked)</item>
  <item>retrospective filled-in [END] section (briefer than work-stage retros — no work axes, just verification axes per VERIFIER-RETROSPECTIVE-TEMPLATE.md)</item>
  <item>merge recommendation (proceed to E / open D.fix / re-tier)</item>
  <item>explicit statement: "Stage M[NN].V is ready. I will not commit until you approve."</item>
</approval_surface>
```

The "I will not commit" surface preserves CLAUDE.md §4 Hard Rule 1 — V findings, like work stages, ship via user-approved commits.

## 15. Versioning this protocol
This protocol changes when:
A new tag is needed across all stages (additive)
A tag's semantics change (breaking; requires migration of in-flight Phase docs)
The two-schema split needs revision (e.g., a third schema for a new stage type — unlikely but possible)
Validation rules change (e.g., a previously-warning becomes an error)
Substantive changes get clear `docs(stage-prompt-protocol): ...` commit messages and a CHANGELOG entry. The commit history of this file is itself an audit of how stage prompts evolved.
If this protocol disagrees with `BUILD-PLAYBOOK.md`, the playbook wins. This protocol is the schema; the playbook is the authority on what stages are and how they run.
### Changelog

v1.9 — One additive optional tag in `<work_stage_prompt>` (`<close_gate>`) + three new recognized `<execution_steps>` step names (`ground_at_red`, `mutation_gate`, `assembled_run_irl`) + a fifth verifier pass (`assembled_execution`) in `<verification_passes>` + the `STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` five-pass update, enacting the cluster-gate close discipline (`docs/cluster-pattern.md`) + CLAUDE.md §4 rule 11 (grounded-claims / no-gaslighting). The through-line: a stage/cluster is done only when the **assembled thing has run and been observed** — not "tests green." The M08.6 escape (shipped "Sound, 0🔴" through full A–F + Stage V + closeout; the post-M08.6 IRL found 7🔴 because the assembled app was never run) is exactly what the `<close_gate>` slot + the fifth pass exist to prevent. Lean-validator pattern continued from v1.3–v1.8 (the new tag passes through without allowlisting; the fifth pass name is additive — the validator never enforced the pass-name enum in code; cross-checks deferred):
1. **`<close_gate>` (work-stage, optional, lean).** Declares the assembled-run + IRL close condition + mutation gate + zero-propagation triage; governed by `docs/cluster-pattern.md`. Parallel to v1.7 `<tdd_discipline>` — a structural contract, not exhortation (the TDAD finding: structural enforcement, not hope, is what makes the discipline hold for LLM agents).
2. **Three recognized `<execution_steps>` step names** for the cluster-gate cycle: `ground_at_red`, `mutation_gate`, `assembled_run_irl`. The cluster cycle: `ground_at_red → write_failing_tests → red_phase_commit → surface_for_red_approval → implement → verify_gates → mutation_gate → green_phase_commit → assembled_run_irl → surface_for_final_approval → fill_retrospective`.
3. **Fifth verifier pass `assembled_execution`** — V RUNS the assembled app + observes each primitive execute; a "Sound" without running is forbidden (rule 11). The verdict states what was NOT exercised.
4. **Validator extension: none (lean continued).** `<close_gate>` passes through the regex parser without allowlisting (identical to the v1.3–v1.8 optional slots + v1.7 `<tdd_discipline>`); the fifth pass name is additive (the pass-name enum was documented-only, never code-enforced). A documentation comment records the v1.9 pass-through; no enforcement logic added.

v1.8 — Three additive optional tags in `<work_stage_prompt>` (`<construction_reachability_check>`, `<wire_signature_audit>`, `<wire_trace_vs_adr_reconcile>`) + a `shape=` extension to `<phase_doc_inventory_audit>` + two `STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` codifications + a CLAUDE.md §6 assembled-app-regression mandate, enacting the 5 M06 graduated mechanisms the M06 gap-analysis routed here (`docs/gap-analysis.md` lines 1897/1901; the other 3 of 8 graduations landed mid-M06 via PR #76 + CLAUDE.md §6 — not re-landed) plus the M06.5-summary "To Cycle 2 (M06.6)" recorded input. The through-line: `<phase_doc_inventory_audit verified="true">` proves a symbol *exists*, not that it is *reachable / correctly-shaped / ADR-current / exercised in the assembled app*. Lean-validator pattern continued from v1.3/v1.4/v1.5/v1.6/v1.7 (structural-only; the three optional tags + the `shape=` extension pass through without allowlisting; cross-checks deferred to v1.9+):

1. **New optional slot `<construction_reachability_check>`** in work stages. Children: `<wire claim="..." constructor="..." inputs_reachable="..." resolution="..."/>`. For every "wire / inject / construct X into Y" deliverable, traces the construction graph at authoring time — Y's constructor receives X AND X's ctor inputs are reachable at that call site. Generalizes the M06.D `<dependency_cycle_check>`/construction-reachability carry-forward; would have caught M06.F's "MCP dispatch end-to-end" over-reach (no shell ctor inputs for the concrete dispatcher; only the no-tools `AgentSdk` smoke path; `M06.F-retrospective.md:31,77,116,121`). Discriminator from `<phase_doc_inventory_audit>`: that proves existence, this proves constructible-where-wired.

2. **New optional slot `<wire_signature_audit>`** in work stages (renderer/IPC convention). Children: `<wrapper ipc_command="..." actual_params="..." phase_doc_assumed="..."/>`. Pins IPC-wrapper params to the actual Tauri command before authoring renderer pseudocode. Bit M06.E `mcp_test_connection {config}`-vs-`{name}` drift (`M06.E-retrospective.md:118,183`; five such drifts at E).

3. **Extension to existing `<phase_doc_inventory_audit>`** (v1.4 tag, v1.6-extended). v1.8 adds an optional `shape="<actual TS/Rust type>"` attribute to `<claim>` for `type="store_slot"` claims — verifies the slot's *type*, not just its symbol. Bit M06.E `currentMcpServers` `Map`+`.tools`-vs-`Record<string, McpServerStatusRecord>` drift. Backward-compatible; `shape=` is honor-system (same as `verified=`).

4. **New optional slot `<wire_trace_vs_adr_reconcile>`** in work stages (work-stage AND verifier-relevant — Stage V's bias-guarded `<read_first>` consumes it, same mechanism as `<scope_change>`). Children: `<trace id="..." assumes="..." adr_checked="..." superseded="..." resolution="..."/>`. Reconciles every Wire trace against accepted ADRs at phase-doc-authoring time. Bit M06.V Decision 6 — V.3 trace #6 assumed `McpClient` drives re-resolution; ADR-0010 had moved the resolver into `McpDispatcher` (`M06.V-retrospective.md:76`).

5. **Verifier-template codification (M06.V Decision 6 second half).** `STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` gains a standing rule: "primitive delivered + unit-tested, but the production driver is absent, and the root cause is an already-accepted ADR's named carry-forward → classify 🟡 with mandatory carry-forward enumeration (name the ADR + the exact carry-forward clause + the next milestone/stage that owns it); NOT 🔴, NOT silent." Stops V re-deriving this each milestone (it reasoned correctly from ADR-0011 at M06.V, but unaided). Matching record line added to `VERIFIER-RETROSPECTIVE-TEMPLATE.md`.

6. **Verifier-template codification (M06.V Decision 7).** From M07.V onward the Behavior pass MUST run the `--features integration` reference-MCP-server smoke (a real dispatch path exists by M07; mock-only Behavior cannot rule out rmcp wire-format correctness — the `transport/stdio.rs`+`http.rs` excluded holdout, attributed to the `tests/integration.rs` smoke that ran 0/0 at M06.V; `M06.V-retrospective.md:77`). Documented as the explicit M06.V→M07.V protocol carry-forward; matching `VERIFIER-RETROSPECTIVE-TEMPLATE.md` record line ("integration smoke executed: N/M, not 0/0").

7. **CLAUDE.md §6 assembled-app-regression mandate (M06.5-summary "To Cycle 2" recorded input).** A work stage's regression test must exercise the assembled running-app path (real composition / real subprocess), not the isolated component that already passes its unit test; the phase-doc root cause is a falsifiable hypothesis the assembled regression test must disprove, not a premise. Plus the binary-crate variant: when the target crate is binary-only and `git diff <red>..<impl> -- '**/tests/**'` is vacuously empty, the strict-TDD invariant is satisfied instead by proving the in-source `#[cfg(test)]` block byte-identical red→impl (the M06.5.A.fix precedent). Long-form in CLAUDE.md §6; this protocol's `<construction_reachability_check>` + the verifier-template codifications are the authoring-time and verification-time surfaces of the same through-line.

8. **Validator extension: none (lean continued).** The three new optional tags + the `shape=` extension pass through the regex parser without allowlisting — identical lean treatment to the nine v1.6 optional slots (v1.6 changelog #13) and v1.7 `<tdd_discipline>` (v1.7 changelog #4). A documentation-only comment block in `bin/validate-stage-prompts.mjs` records the v1.8 pass-through set; no enforcement logic added.

9. **§G validator-promotion decision (maintainer call).** The v1.7 changelog deferred promoting the `<tdd_discipline strict="true">`→two-commit coupling to a validator cross-check "at v1.8+ once 2+ milestones show clean signal." Signal so far: v1.7 shipped at M06 (PR #76, before M06.D); M06.D/E/F + M06.5 A.fix/B.fix ran strict two-commit with the `**/tests/**`-empty invariant held cleanly — ~1 milestone + 1 fix-cycle, short of the "2+ milestones" bar. Recorded maintainer decision: keep the three new v1.8 slots lean (pass-through, honor-system) AND defer the `<tdd_discipline>` coupling promotion to v1.9 (M07 supplies the second milestone of signal). §11 carries the deferral.

10. **No source changes; no gap-analysis entry; M01–M06.5 phase docs grandfathered.** Protocol iteration is process, not product (CLAUDE.md §20 — not a parent milestone). The v1.8 slots apply M07+ only (v1.6-established grandfathering, M06.D/E-confirmed). Validator PASS pre+post (the change is additive/recognizing, not breaking).

v1.7 — One additive optional tag in `<work_stage_prompt>` (`<tdd_discipline>`) + four new recognized `<execution_steps>` step names + CLAUDE.md §6 `cargo llvm-cov clean` note + the two-commit pattern documented as the v1.7 recommended default + three graduated gotchas, informed by M06 friction (M06.A's TDD-discipline lapse failing hard-gate G5 + maintainer override; M06.C's empirical validation of the strict pattern) + web evidence (Nagappan et al. 2009; TDAD arXiv:2603.17973; Anthropic Claude Code TDD docs). Lean-validator pattern continued from v1.3/v1.4/v1.5/v1.6 (structural-only; the optional tag passes through without allowlisting; cross-checks deferred to v1.8+):

1. **New optional slot `<tdd_discipline>`** in work stages. `strict="true"` attribute + `<red_phase>` / `<green_phase>` children. Makes the strict red-phase/green-phase two-commit TDD pattern a first-class, structurally-auditable protocol element. Addresses M06.A's TDD-discipline lapse (implementation before tests → hard-gate G5 failure → documented maintainer override) by giving the discipline a structural home rather than a per-stage user override. Evidence basis in the §7 slot definition: Nagappan et al. 2009 (industrial TDD 60–90% defect reduction); TDAD arXiv:2603.17973 (TDD *prompting* WITHOUT structural enforcement INCREASES regressions 9.94%; structural enforcement drops them 70% — the discriminator is structure, not exhortation); Anthropic Claude Code TDD docs (commit-tests-before-impl as the safety net). M06.C is the empirical reference implementation (~10% time overhead; structural defense against gotcha #66).

2. **Four new recognized `<execution_steps>` step names:** `red_phase_commit`, `surface_for_red_approval`, `green_phase_commit`, `surface_for_final_approval`. The two-commit `<execution_steps>` shape (`write_failing_tests → red_phase_commit → surface_for_red_approval → implement → verify_gates → green_phase_commit → surface_for_final_approval → fill_retrospective`) is the v1.7 recommended default for any work stage that ships testable code. The validator does not enforce step names (lean); these are documented in §7 so future stages don't re-derive them.

3. **Load-bearing audit-surface invariant.** When `<tdd_discipline strict="true">` is present, the green-phase commit body MUST state the verifiable invariant `git diff <red-sha>..<impl-sha> -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical rustfmt/clippy fixes to test files go in a SEPARATE labelled follow-up commit, keeping the impl↔red diff a pure implementation delta. This is the structural property the TDAD research identifies as the difference between TDD-that-works and TDD-prompting-that-backfires.

4. **Validator extension: none (lean continued).** `<tdd_discipline>` is optional; the validator's regex parser passes it through structurally without allowlisting — identical lean treatment to the nine v1.6 optional slots (v1.6 changelog item #13). The `<tdd_discipline strict="true">` → two-commit `<execution_steps>` coupling is an authoring-discipline rule documented in §11, NOT code-enforced at v1.7; promote to a validator cross-check at v1.8+ once 2+ milestones show clean signal.

5. **Two new anti-patterns** in §13: (a) `<tdd_discipline strict="true">` with single-commit `<execution_steps>` (slot decorative; audit invariant unholdable); (b) green-phase commit that modifies the red-phase test files (destroys the empty-diff property; the exact failure mode Anthropic's TDD docs warn about).

6. **Three graduated gotchas** in `docs/gotchas.md`: self-deactivating `#[expect(clippy::unused_async)]` for TDD-stub async methods (M06.C; #79 — prefer `expect` over `allow` so green-phase impl naturally removes it via `cargo clippy --fix`); non-idempotent migrations (ALTER/RENAME/DROP) make `init_idempotent` tests meaningful for the first time (M06.C; #80 — verify the migration runner's version-skip is the sole idempotency guarantor before landing a non-idempotent migration); `cargo llvm-cov` merges `.profraw` across runs — run `cargo llvm-cov clean` before a gate measurement if a prior (especially failing) run happened in the same session (M06.C; #81 — wasted ~10 min chasing a phantom coverage drop).

7. **CLAUDE.md §6 update.** The canonical gate-ordering paragraph gains a `cargo llvm-cov clean` note (gotcha #81) + a reference to the v1.7 two-commit `<tdd_discipline>` pattern.

8. **M01–M06 grandfathered per their existing banners.** M06 was authored on v1.6; the v1.7 `<tdd_discipline>` slot applies forward from M06.D (the next unexecuted stage) — M06's remaining stages (D, E) adopt the two-commit pattern via a focused phase-doc edit on the milestone branch; M07+ author it natively. The slot is optional, so no grandfathered phase doc breaks.

v1.6 — Eleven additive optional tags / extensions in `<work_stage_prompt>` + one new required child of `<closeout_stage_prompt>` `<deliverables>` + CLAUDE.md §6 quality-gate ordering paragraph + four graduated gotchas, informed by M05 friction (seven work stages of phase-doc-vs-codebase drift recurrence; first in-band Stage V verifier run; first waiver-as-ADR cycle per ADR-0009). Lean-validator pattern continued from v1.3/v1.4/v1.5 (structural checks only; cross-checks deferred to v1.7+):

1. **New optional slot `<coverage_gate>`** in work stages. Children: `<gate scope="..." target_lines="..." ignore_filename_regex="..." />` elements. Names the exact `--ignore-filename-regex` argument the stage's `cargo llvm-cov` invocation will use, replacing prose enumeration. Addresses M05.C1 + C2 "plumbing files" prose-vs-regex 4-attempt friction; recurred at 7 stages of M05 (A/B/C1/C2/D/E/F).

2. **New optional slot `<schema_ref_audit>`** in work stages. Children: `<ref schema="..." path="#/$defs/<Name>" verified="..." />` elements. Verifies that `$defs/<Name>` references the phase doc names exist in the sibling schema file. Addresses M05.A `mcp_missing` factual error + M05.E `common.v1.json#/$defs/NonEmptyString` non-existent reference.

3. **New optional slot `<api_breaking_change_audit>`** in work stages. Children: `<change api="..." before_signature="..." after_signature="..." call_sites="..." test_sites="..." recommendation="..." />` elements. Surfaces sync→async migration cost (call-sites + test-sites) BEFORE implementation starts. Addresses M05.E's implied-async-grant ~10-minute design-iteration cost.

4. **New optional slot `<existing_pattern_audit>`** in work stages. Children: `<pattern grep_for="..." rationale="..." affected_files="..." remediation="..." />` elements. Greps for existing irrefutable bindings of an enum variant about to be broken by a new variant addition. Addresses M05.D `TierForbidden` 4-site immediate breakage.

5. **New optional slot `<interpretation_declarations>`** in work stages. Children: `<adopt spec_section="..." interpretation="..." alternative_interpretation="..." rationale="..." />` elements. Declares the phase doc's adopted interpretation of an ambiguous spec section. Addresses M05.D runtime-vs-install-time L4 interpretation gap across phase doc / spec / MVP-v0.1.

6. **New optional slot `<scope_change>` (highest priority)** in work stages. Children: `<descope deliverable="..." reason="..." carry_forward_to="..." authorized_by="..." />` or `<expand deliverable="..." reason="..." documented_in="..." />` elements. Surfaces intentional in-stage descopes (and undocumented expansions) so Stage V's bias-guarded `<read_first>` can pick them up without crossing the fresh-context discipline line. Pairs with — does not replace — the v1.5 waiver-as-ADR lane (ADR-0008/ADR-0009). Addresses M05.V Decision 3 + the canonical M05.V Findings #1 + #2 case study (Stage B's authorized SDK wire-up descope was documented only in M05.B-retrospective.md — the file V is forbidden to read; ADR-0009 absorbed the finding correctly but `<scope_change>` is the structural fix).

7. **New optional slot `<zustand_selector_audit>`** in work stages (renderer-stage convention). Children: `<selector pattern="..." requires_use_shallow="..." import_path="..." />` elements. Surfaces the Zustand v5 `useShallow` requirement for derived-array selectors. Addresses M05.F `Maximum update depth exceeded` infinite-loop friction.

8. **New optional slot `<playwright_warmup_recipe>`** in work stages (renderer-stage convention). Self-closing with `url`, `timeout_seconds`, `before_first_spec` attributes. Names the curl warmup probe to run before the first Playwright spec invocation against a cold Vite dev server. Addresses M04.C ApprovalPanel + M05.F GapPanel (2 stages) recurrence; pairs with gotcha #53.

9. **New optional slot `<test_isolation_audit>`** in work stages (renderer-stage convention). Children: `<persistent_slot store="..." field="..." preserved_across_clear="..." required_reset="..." />` elements. Surfaces persistent store slots (Zustand slices preserved across `clear()`) that test files mutating those slots must reset via `beforeEach`. Addresses M05.F `currentTier` cross-test bleed pattern.

10. **Extension to existing `<phase_doc_inventory_audit>`** (v1.4 tag). v1.6 adds three new `type=` values for `<claim>` children: `method` (verify function/method symbol exists at path; bit M05.B `dispatch_tool`), `struct_field` (verify struct field exists at path; bit M05.F field-name divergence), `read_first_target` (verify a file referenced from another stage's `<read_first>` block exists; bit M05.D `M05.C-retrospective.md` reference when the C stage split into C1 + C2). Backward-compatible — existing `<claim type="file">` shape continues to work.

11. **Extension to existing `<dependency_audit_check>`** (v1.3 tag). v1.6 adds (a) `<feature_interdependency crate="..." function="..." home_feature="..." requires_features="..." reason="..." />` sibling child to declare that one FFI function binding requires features beyond its home module (M05.C2 `CreateJobObjectW` needing `Win32_Security` for `SECURITY_ATTRIBUTES`); (b) new optional `prefer_crates_io_name="true"` + `source_authority="..."` attributes on `<dep>` to distinguish crates.io canonical names from GitHub-org names (M05.C2 `windows-sys 0.59` vs `winapi 0.3` web-research). Backward-compatible.

12. **New required child of `<closeout_stage_prompt>` `<deliverables>`: `<simplify_pass>`.** Recurring structural refactor checkpoint at every Stage G closeout. Children: `<invoke skill="simplify" against="..."/>`, `<surface kind="refactor_proposals" examples="..."/>`, `<approval_required>true</approval_required>`, `<commit_on_approval>...</commit_on_approval>`, `<defer_unapproved_to>docs/tech-debt.md (per ADR-0008 🟢 ledger)</defer_unapproved_to>`. Runs AFTER summary + gap-analysis drafted, BEFORE PR opens. Maintainer request: `simplify`-class consolidations (duplicated patterns, dead code, parallel API surfaces, modules grown across stages) should be a standing ceremony, not an out-of-band per-milestone ask.

13. **Validator extension.** The only required-tag change in v1.6: `bin/validate-stage-prompts.mjs`'s `REQUIRED_BY_ROOT.closeout_stage_prompt` list gains `simplify_pass`. All other v1.6 tags are optional — the validator's regex parser passes them through structurally without explicit allowlisting.

14. **Authoring rule: quality-gate execution ordering is canonical** (§10 hardening). Every work stage's `<execution_steps>::implement` step runs `cargo fmt --all` → `cargo clippy --fix --allow-dirty -p <crate>` → `cargo clippy --workspace -- -D warnings` → remaining gates. Graduates gotcha #34 + gotcha #64 from per-stage citation to per-stage execution discipline. Recurring 6 stages of M05 (B + C1 + C2 + D + E + light at A). CLAUDE.md §6 carries the long-form paragraph.

15. **Lean validator continued.** All ten new optional tags + two extensions are structural-only at the v1.6 validator level. Cross-checks (e.g., "schema_ref_audit is required if stage's `<deliverable>` mentions a sibling schema's `$defs`") deferred to v1.7+ once v1.6 has 2+ milestones of clean signal.

16. **Six new anti-patterns** in §13 covering the v1.6-introduced failure shapes (scope_change as the only descope-disclosure mechanism; coverage_gate content drifting from CI workflow; interpretation_declarations over-use for unambiguous spec sections; existing_pattern_audit confused with fan_out_grep; simplify_pass skipped with "no refactoring needed"; simplify_pass proposals applied without maintainer approval).

17. **Four graduated gotchas** in `docs/gotchas.md` (the canonical project-wide trap ledger): typify oneOf-enums with non-`Copy` variant payloads don't derive `PartialEq`/`Eq` (M05.B + C1; gotcha #73 graduates from "reserved"); Zustand v5 derived-array selectors require `useShallow` (M05.F; gotcha #75); windows-sys feature flags gate function bindings by ALL parameter type modules, not just function's home module (M05.C2; gotcha #76); `tokio::io::duplex` buffer must be smaller than payload to surface write-failure branches (M05.C2; gotcha #77).

18. **M01–M05 grandfathered as v1.0/v1.2/v1.3/v1.4/v1.5 per their existing banners.** M05 was authored on v1.5; M06+ are the first milestones authored on v1.6. M05's Stage G closeout prompt is v1.5 — the `<simplify_pass>` required-child requirement applies forward from M06 only. (The validator's enforcement of `simplify_pass` in closeout prompts is exempt for any phase doc carrying a `**Protocol version:** v1.5 (grandfathered).` banner.)

v1.5 — New third schema variant `<verifier_stage_prompt>` for Stage V (Milestone Verifier), informed by M04 IRL bug-finding (five contract-fidelity failures the existing protocol missed). Companion ADR-0008 documents the design rationale:
1. **New schema variant `<verifier_stage_prompt>`** parallel to `<work_stage_prompt>` and `<closeout_stage_prompt>`. Used for the verifier stage (V) between the last work stage and closeout. Documented in new §14 (Verifier-only tags).
2. **Five new required tags** for the verifier schema: `<scope_to_verify>` (replaces `<deliverable>` for V's role), `<verification_passes>` (wraps the four pass declarations), `<findings_format>` (declares the structured output shape), `<merge_gate>` (severity model + D.fix iteration cap + waiver path), and the standard common tags adapted for V. Three tags are forbidden in V: `<read_prior_stages>` (bias guard), `<deliverable>` (V produces findings not artifacts), `<test_plan_required>` (V is the test plan).
3. **Four verification passes** (Inventory + Wire + Behavior + Multi-call invariants), each declared as a `<pass name="...">` child of `<verification_passes>`. Pass 2 (Wire) ships with the explicit 5-step data-path tracing structure to replace bare grep. Pass 3 (Behavior) adds runtime-render coverage that closes the M04 BudgetHeaderBar-CSS class of bug (static checks insufficient).
4. **Fresh-context bias guard** via the `<read_first>` discipline: V's read list MUST omit prior retrospectives, the milestone summary, and prior gap-analysis entries. Structural enforcement is the clear-and-paste session pattern (the user clears the CLI session and pastes the V prompt fresh); the `<read_first>` omission is the artifact-level discipline.
5. **Severity model + merge gate** aligned with gap-analysis: 🔴 blocks merge (triggers D.fix); 🟡 carries forward to next milestone's Stage A; 🟢 lands in `docs/tech-debt.md` (new append-only ledger distinct from gap-analysis and gotchas). Maximum 2 D.fix iterations before escalation. Waiver path uses ADR-class artifacts (no new artifact class introduced).
6. **Lean validator continued.** All verifier-specific tags are structural-only at the v1.5 validator level. Cross-checks (e.g., "verifier prompt's `<read_first>` references prior retros → reject") deferred to v1.6 once v1.5 has 2+ milestones of clean signal.
7. **Validator extended** to recognize the new root element (`<verifier_stage_prompt>`) alongside the existing two. Same regex pattern, third schema variant. See `bin/validate-stage-prompts.mjs` for the implementation.
8. **M01–M04 + M04.5 grandfathered as v1.0/v1.2/v1.3/v1.4 per their existing banners.** M05 is the first milestone authored on v1.5 — the first to include a Stage V section in its Phase doc. M04 retroactively receives a V run (no Phase doc edit; results land in `M04.V-retrospective.md`) as the protocol's first real-world test.

v1.4 — Four additive optional tags + four anti-patterns informed by M04 friction. Lean-validator pattern continued from v1.3 (structural checks only; cross-checks deferred to v1.5+):
1. **New optional slot `<architecture_check>`** in work stages. Children: `<claim description="..." verify="..." />` elements. Verifies HOW-claims about cross-process flow, IPC topology, in-process-vs-out-of-process state ownership. Companion to gotcha #41 (WHAT-claims grep verification) for *invariants about codebase structure*. Addresses M04.C + M04.E approval/HITL seam architecture surprise (phase doc text implied drone-mediated; reality is in-process per ADR-0007).
2. **New optional slot `<schema_audit>`** in work stages. Children: `<survey pattern="..." purpose="..." />` elements. Pre-flight enumeration of existing `$defs` + root types in `schemas/*.v1.json` before authoring a new schema file — phase-doc claims of "new schema X" may collide with an existing `$defs/X` elsewhere. Addresses M04.D (`hooks.v1.json` proposal — `Hook` + `HookRef` + `HookCategory` + `JsonLogicExpression` already in `common.v1.json`; `Rail` already in `framework.v1.json`).
3. **New optional slot `<schema_root_check>`** in work stages. Self-closing with optional `gate="..."` attribute (defaults to a jq check). Asserts new/edited `schemas/*.v1.json` has a concrete `type` / `oneOf` / `anyOf` at the root, not a top-level `$ref` — `json-schema-to-typescript` errors on top-level `$ref` even when typify accepts it. Addresses M04.E `hitl.v1.json` initial draft (graduated gotcha #57).
4. **New optional slot `<phase_doc_inventory_audit>`** in work stages. Children: `<inventory_row path="..." status="..." />` elements (status enum: `exists` | `new` | `deleted` | `renamed`). Pre-flight verification that every file path in the phase doc's `X.2 Files to Change` table matches `git ls-files` reality before code work begins. Codifies the discipline that produced gotcha #41. Addresses M04.A2 phase-doc-vs-reality scope-mismatch friction (Sev 4) + M04 PR #53 false-premise rewrite.
5. **Lean validator continued.** All four new tags are structural-only at the v1.4 validator level. Cross-checks (e.g., "schema_audit is required if stage adds a new file to `schemas/`") deferred to v1.5+ once v1.4 has 2+ milestones of clean signal.
6. **Four new anti-patterns** in §13 covering the v1.4-introduced failure shapes (architecture_check enumerating WHAT-claims; schema_audit enumerating proposed-types-only without surveying neighbors; schema_root_check skipping for "obviously concrete" schemas; phase_doc_inventory_audit treating status=`exists` as informational not load-bearing).
7. **M01–M03 + M03.5 grandfathered as v1.0/v1.2/v1.3 per their existing banners.** M04 was authored on v1.3; M05+ are the first milestones authored on v1.4. Prior grandfathering remains in effect.

v1.3 — Five additive optional tags + three anti-patterns informed by M01–M03 friction. Lean-validator pattern continued from v1.2 (structural checks only; cross-checks deferred to v1.4):
1. **New optional slot `<pre_flight_check>`** in work stages. Children: `<check name="..." />` elements. Pre-stage environmental verifications (branch, prior-commit, env vars, deps) that gate the stage from starting if violated. Addresses M03.F Stage E sequencing slip.
2. **New optional slot `<schema_drift_check>`** in work stages. Self-closing with optional `gate="..."` attribute (defaults to `cargo xtask regenerate-types --check`). Forces schema-as-source-of-truth invariant (CLAUDE.md §14) at stage level. Addresses M03.D hand-maintained event.rs brittleness.
3. **New optional slot `<fan_out_grep>`** in work stages. Children: `<grep pattern="..." purpose="..." />` elements. Pre-rename consumer enumeration AND value-consistency verification (e.g., schema `$id` URL conventions). Addresses M02–M03 recurring rename/move surprise pattern + M03.5.A `$id` discrepancy catch.
4. **New optional slot `<dependency_audit_check>`** in work stages. Children: `<dep name="..." required_features="..." min_version="..." audit="..." />` elements. Pre-code dependency-tree verification. Addresses gotcha #29 (keyring stub) + gotcha #39 (npm audit overrides).
5. **New optional slot `<runtime_environment>`** in work stages. Self-closing with `os="..."` attribute, OR child `<command os="..." cmd="..." />` elements. Explicit OS pin + platform-specific command variants. Addresses CRLF warnings (PR #48) + PowerShell-vs-bash differences (M03.5.A friction event).
6. **Lean validator continued.** All five new tags are structural-only at the v1.3 validator level. Cross-checks (e.g., "schema_drift_check is required if stage edits schemas/*.v1.json") deferred to v1.4 once v1.3 has 2+ milestones of clean signal.
7. **Three new anti-patterns** in §13 covering the v1.3-introduced failure shapes (pre-flight as orientation; fan-out enumerating change set; runtime_environment without command variants).
8. **M01-M03 grandfathered as v1.0/v1.2.** M03.5 (this milestone) was authored on v1.2; M04+ are the first milestones authored on v1.3. M01-M03 prompts retain their version banners and are exempt from v1.3 validator rules.
v1.2 — Eight additive/hardening changes informed by M01 + M02 retrospective + opinion review (M02 Phase Closeout). Anchor stability, procedural slot, two new content slots, strict reference-first, lean validator, gotchas-graduation enforcement, grandfathering of M01 + M02:
1. **Anchor stability fix.** Reference tags use `section="..."` attribute instead of URI fragment notation (e.g., `ref="...md" section="A.6 Commit Message"` not `ref="...md#A.6"`). Renderer-agnostic, slugifier-agnostic. Validator resolves headings by markdown-AST lookup. Applies to `<commit_message>`, `<deliverable>`, `<acceptance_criteria>`, `<scope_locks>`, `<gap_analysis_requirements>`, `<commit_protocol>`. Old fragment form no longer recognized by the validator (v1.0-grandfathered prompts exempt via header banner).
2. **New required slot `<execution_steps>`** in work stages. Named procedural anchor — `write_failing_tests → implement → verify_gates → fill_retrospective → surface`. Replaces inline STEP 1–5 prose that previously lived in each prompt; the slot resolves to playbook sections rather than restating them.
3. **New optional slot `<read_reference>`** (both schemas). Files for archetypal pattern reference (e.g., "see `crates/runtime-providers/src/anthropic_sse.rs` as `*_with` archetype"). Distinct from `<read_first>` (orientation) and `<read_prior_stages>` (within-milestone retrospectives). `purpose` attribute required (warning in v1.2, promotes to error in v1.3).
4. **New optional slot `<execution_warnings>`** (both schemas). Inline operational warnings — workflow-time guardrails that apply during stage execution (cost concerns, side-effecting commands). Distinct from `<gotchas>` (pre-flight implementation traps) and `<scope_locks>` (deliverable-shape constraints).
5. **Reference-first STRICT.** v1.0/v1.1 had reference-first as the default-but-not-required pattern. v1.2 makes it strict: if the Phase doc has a section matching the expected anchor for a content-heavy tag, inline content in that tag is rejected by the validator (error). Forces authors to commit to one source of truth and prevents prompt-vs-Phase-doc drift.
6. **Gotchas graduation rule + `<gotchas_graduation>` enforced in closeout.** New required subsection inside `<gap_analysis_requirements>`. Audits per-stage `<gotchas>` entries across the milestone with disposition enum: `kept | graduated | resolved | expired`. The `expired` disposition requires rationale in `<target>` beyond bare `n/a`. Validator: every prior stage must have a `<stage_review>` entry; disposition values must match the enum; `expired` rationale length is checked.
7. **Lean validator.** v1.2 ships with structural checks as errors (block CI) and cross-file resolution checks as warnings (surface in PR output, do not block). Cross-checks promote to errors in v1.3 once the cross-check logic survives 3+ milestones without false positives. Reduces brittleness during the v1.2 → v1.3 transition.
8. **M01 + M02 grandfathered as v1.0.** The M01-foundation.md and M02-event-pipeline.md Phase docs predate v1.2 and use URI-fragment refs / inline content / no `<execution_steps>` etc. Both files carry a `**Protocol version:** v1.0 (pre-XML-schema; grandfathered).` header banner that exempts them from v1.2 validator rules. v1.2 applies to M03 forward.
v1.1 — Three additive changes informed by M02 Phase doc audit:
New common tag `<read_prior_milestones>` for Stage A of non-first milestones absorbing prior-milestone carry-forward
New common tag `<commit_message ref="..."/>` (required) referencing the pre-authored commit message in the Phase doc's `X.6` section
Reference-first pattern formalized for content-heavy tags (`<deliverable>`, `<acceptance_criteria>`, `<scope_locks>`): each may use either inline form OR self-closing `ref="..."` form pointing at the corresponding Phase doc section, never both. Validator enforces.
Existing v1.0 prompts remain valid; the additions are backward-compatible (the new tags are required from v1.1 forward, but existing Phase docs can be updated incrementally as they're touched).
v1.0 — Initial protocol. Two-schema split (`<work_stage_prompt>` and `<closeout_stage_prompt>`); common, work-only, closeout-only, and optional tag sets; authoring rules; validation contract; worked examples for M01.A and M01.E; anti-patterns.
---
End of Stage Prompt Protocol.
