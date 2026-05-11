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
Exactly one root element per block: `<work_stage_prompt>` or `<closeout_stage_prompt>`.
The root element has an `id` attribute formatted `M[NN].<X>` (e.g., `M01.A`, `M01.E`).
A regex extractor: ````xml\n(<(?:work_stage_prompt|closeout_stage_prompt)[\s\S]*?</(?:work_stage_prompt|closeout_stage_prompt)>)\n````
A validator should:
Confirm one and only one root element per block
Confirm the root tag is one of the two valid schemas
Confirm `id` attribute matches the format
Confirm all required tags for the schema are present
Confirm no foreign tags appear (every tag must be in the protocol)
## 5. The two schemas
There are exactly two stage prompt schemas. They share most tags but the closeout adds requirements that don't apply to work stages.
Schema	Used for	Distinct requirements
`<work_stage_prompt>`	Work stages (A, B, C, D, …)	Concrete deliverable, test plan required, acceptance criteria
`<closeout_stage_prompt>`	Closeout stage (E, the final stage of every milestone)	Cumulative reads, gap-analysis entry, append-only verification, three-artifact review
The closeout is genuinely a different ceremony — it does cumulative review and writes the immutable ledger entry that gates the milestone PR. Forcing it into the work-stage schema with optional tags would lose enforcement: a closeout missing `<cumulative_reads>` is broken; a work stage missing `<cumulative_reads>` is fine. Two schemas make this difference enforceable.
## 6. Common tags (used by both schemas)
### Root attribute: `id`
Required. Format `M[NN].<X>`. Examples: `M01.A`, `M01.E`, `M11.D`. Used for retrospective filenames, session register entries, and cross-references.
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
For work stages, default order: diff stat → gate results → retrospective → draft commit message → "I will not commit until you approve."
```xml
<approval_surface>
  <item>diff stat (git diff --stat HEAD)</item>
  <item>gate results (each gate, pass/fail, key numbers)</item>
  <item>retrospective (filled-in [END] section)</item>
  <item>draft commit message (Conventional Commits + DCO + session URL)</item>
  <item>explicit statement: "I will not commit until you approve."</item>
</approval_surface>
```
## 7. Work-stage-only tags
These tags are valid only inside `<work_stage_prompt>`.

v1.3 adds five additive optional tags — `<pre_flight_check>`, `<schema_drift_check>`, `<fan_out_grep>`, `<dependency_audit_check>`, `<runtime_environment>` — informed by M01–M03 friction. v1.4 adds four more — `<architecture_check>`, `<schema_audit>`, `<schema_root_check>`, `<phase_doc_inventory_audit>` — informed by M04 friction. See sections below.
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
Standard step names (validator warns on unrecognized names): `write_failing_tests`, `implement`, `verify_gates`, `fill_retrospective`, `surface`. Stages with non-standard cycles (e.g., a debugging stage where iteration is the deliverable) may add custom steps with explicit `name` attributes; document them in the Phase doc's stage section.
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
Required. The closeout produces three artifacts. Plural form distinguishes from work-stage `<deliverable>`.
```xml
<deliverables>
  <milestone_summary>retrospectives/M01-summary.md (aggregates per-stage retrospectives, scores axes across stages, marks verdict)</milestone_summary>
  <gap_analysis_entry>docs/gap-analysis.md (append new entry; six required sections, none optional)</gap_analysis_entry>
  <pr_description>draft only; do not open PR until explicitly asked</pr_description>
</deliverables>
```
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
Stable child element names. Within `<deliverable>`, every child is `<item>`. Within `<scope_locks>`, every child is `<lock>`. Within `<acceptance_criteria>`, every child is `<criterion>`. Within `<execution_steps>`, every child is `<step>`. Within `<read_reference>`, every child is `<file>`. Within `<execution_warnings>`, every child is `<warning>`. Within `<gotchas_graduation>`, every child is `<stage_review>`. This consistency makes validation and aggregation simple.
Order tags consistently across milestones. The recommended order:
For work stages: `<context>` → `<read_first>` → `<read_reference>` (opt) → `<read_prior_milestones>` (Stage A only when applicable) → `<read_prior_stages>` (B+) → `<deliverable>` → `<test_plan_required>` → `<execution_steps>` → `<acceptance_criteria>` → `<scope_locks>` → `<gates>` → `<self_correction_budget>` → `<adr_triggers>` (opt) → `<gotchas>` (opt) → `<execution_warnings>` (opt) → `<time_box>` (opt) → `<dependencies>` (opt) → `<retrospective_requirements>` → `<commit_protocol>` → `<commit_message>` → `<approval_surface>`.
For closeout stages: `<context>` → `<read_first>` → `<read_reference>` (opt) → `<read_prior_milestones>` (rare for closeout; included only if absorbing additional carry-forward) → `<cumulative_reads>` → `<deliverables>` → `<gap_analysis_requirements>` (with required `<gotchas_graduation>`) → `<append_only_verification>` → `<three_artifact_review>` → `<scope_locks>` → `<gates>` → `<self_correction_budget>` → `<adr_triggers>` (opt) → `<gotchas>` (opt) → `<execution_warnings>` (opt) → `<time_box>` (opt) → `<retrospective_requirements>` → `<commit_protocol>` → `<commit_message>` → `<approval_surface>`.
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
**Warnings (surface in PR output, don't block):**
- Confirms ordering matches the recommended order
- Cross-checks: every retrospective referenced in `<read_prior_stages>` exists; every milestone in `<read_prior_milestones>` has the named gap-analysis section + summary section; every file in `<read_first>` and `<read_reference>` exists; every `section="..."` value on a reference tag resolves to a real Phase doc heading via markdown-AST lookup; the milestone in `<gates milestone="...">` matches the Phase doc's milestone
- `<read_reference>` entries without a `purpose` attribute (warning in v1.2; promotes to error in v1.3)
- Recognized `<execution_steps>` step names (`write_failing_tests`, `implement`, `verify_gates`, `fill_retrospective`, `surface`); custom step names emit a warning encouraging Phase doc documentation
- v1.3 tags' child elements with required-but-missing attributes (`<check>` without `name`, `<grep>` without `pattern`/`purpose`, `<dep>` without `name`, `<runtime_environment>` without `os`) emit warnings; promote to errors in v1.4 once the cross-check logic has 2+ milestones of clean signal
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

  <gates milestone="M03"/>

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

## 14. Versioning this protocol
This protocol changes when:
A new tag is needed across all stages (additive)
A tag's semantics change (breaking; requires migration of in-flight Phase docs)
The two-schema split needs revision (e.g., a third schema for a new stage type — unlikely but possible)
Validation rules change (e.g., a previously-warning becomes an error)
Substantive changes get clear `docs(stage-prompt-protocol): ...` commit messages and a CHANGELOG entry. The commit history of this file is itself an audit of how stage prompts evolved.
If this protocol disagrees with `BUILD-PLAYBOOK.md`, the playbook wins. This protocol is the schema; the playbook is the authority on what stages are and how they run.
### Changelog
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
