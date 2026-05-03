Stage Prompt Protocol
> XML schema for stage CLI prompts. Defines required and optional slots, the two schemas (work-stage and closeout-stage), where prompts live, and how they're extracted and validated. Companion to `BUILD-PLAYBOOK.md`.
---
1. Purpose
Stage CLI prompts are the structured input pasted into a fresh agent session at the start of each stage. They orient the agent, constrain scope, name the gates, and reference the protocols (retrospective, commit, gate matrix) the stage must follow.
This document defines the schema. It is the canonical reference for how stage prompts are written; the bare templates derived from it live at `prompts/WORK-STAGE-TEMPLATE.md` and `prompts/CLOSEOUT-STAGE-TEMPLATE.md`.
2. Why XML inside markdown
The Phase doc has two distinct audiences with different needs.
The human reads it for planning, scope review, and navigation. Markdown is what humans read — headers, tables, links, prose narrative for the milestone.
The agent in a fresh session consumes the structured prompt portion. It benefits from explicit slots (`<context>`, `<deliverable>`, `<gates>`) so nothing required gets dropped, parsing is unambiguous, and stage prompts can be diffed cleanly across milestones to see exactly what evolved.
Pure-markdown prompts lose the slot discipline that makes them parseable and diffable. Pure-XML phase docs become unreadable for the planning purpose. The hybrid — markdown wrapper, XML inside fenced code blocks — gives both audiences what they need.
A second-order benefit: every prompt across all milestones can be extracted programmatically with a single regex over fenced ```xml blocks. This is the bridge to ARIA (or any orchestrator) running phases later without rewriting the prompts.
3. Where prompts live
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
4. Programmatic extraction
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
5. The two schemas
There are exactly two stage prompt schemas. They share most tags but the closeout adds requirements that don't apply to work stages.
Schema	Used for	Distinct requirements
`<work_stage_prompt>`	Work stages (A, B, C, D, …)	Concrete deliverable, test plan required, acceptance criteria
`<closeout_stage_prompt>`	Closeout stage (E, the final stage of every milestone)	Cumulative reads, gap-analysis entry, append-only verification, three-artifact review
The closeout is genuinely a different ceremony — it does cumulative review and writes the immutable ledger entry that gates the milestone PR. Forcing it into the work-stage schema with optional tags would lose enforcement: a closeout missing `<cumulative_reads>` is broken; a work stage missing `<cumulative_reads>` is fine. Two schemas make this difference enforceable.
6. Common tags (used by both schemas)
Root attribute: `id`
Required. Format `M[NN].<X>`. Examples: `M01.A`, `M01.E`, `M11.D`. Used for retrospective filenames, session register entries, and cross-references.
`<context>`
Required. Two to four sentences. Why this stage exists, what it builds on, what's about to happen. The orientation paragraph the agent reads first after expanding the prompt.
```xml
<context>
  Stage A of M01 (Foundation). Establish the workspace skeleton — Cargo workspace,
  crate boundaries, lefthook hook, CI scaffold. No business logic yet; this stage
  exists so subsequent stages have a place to land. Builds on nothing; this is the
  first stage of the project.
</context>
```
`<read_first>`
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
`<read_prior_milestones>`
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
`<scope_locks>`
Required. Constraints from spec or ADRs that apply across this stage. These are the things the agent must not do even if locally tempting. Contrasts with `<acceptance_criteria>` (what must be done) — `<scope_locks>` is what must not.
Inline form (use when locks are stage-specific or short):
```xml
<scope_locks>
  <lock>v0.1 is single-session; no multi-session code paths</lock>
  <lock>STANDARD mode hardcoded; no mode router</lock>
  <lock>Anthropic-only; no provider abstraction layer in v0.1</lock>
  <lock>Windows-only test target; CI runs on all three OSes for drift detection</lock>
</scope_locks>
```
Reference form (use when the milestone's Phase doc has a "Key constraints" or equivalent section that applies to all stages — see Authoring Rules §10 reference-first pattern):
```xml
<scope_locks ref="docs/build-prompts/M02-event-pipeline.md#key-constraints"/>
```
Use one form or the other, never both.
`<gates milestone="M[NN]"/>`
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
`<self_correction_budget>`
Optional; defaults to 3 per `BUILD-PLAYBOOK.md` §4.3. Override only when the work genuinely warrants it (e.g., a debugging stage where iteration is the deliverable).
```xml
<self_correction_budget>3</self_correction_budget>
```
`<retrospective_requirements>`
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
`<commit_protocol>`
Required. Reference to the playbook section. Self-closing. The agent re-reads §4.7 to refresh on the do-not-commit rule.
```xml
<commit_protocol ref="BUILD-PLAYBOOK.md#4.7"/>
```
`<commit_message>`
Required. Reference to the pre-authored commit message in the Phase doc (each stage's `X.6 Commit Message` section). Self-closing.
```xml
<commit_message ref="docs/build-prompts/M02-event-pipeline.md#A.6"/>
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
`<approval_surface>`
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
7. Work-stage-only tags
These tags are valid only inside `<work_stage_prompt>`.
`<deliverable>`
Required. What this stage produces. Concrete: files, modules, capabilities. Not aspirational. If you can't enumerate it, the stage isn't ready to start.
Inline form (use when deliverable is short — fewer than ~10 items, no detailed code content):
```xml
<deliverable>
  <item>Cargo workspace at repository root with crates: runtime-core, runtime-drone, runtime-main, runtime-sandbox</item>
  <item>Top-level Cargo.toml with workspace members and shared lints</item>
  <item>lefthook.yml with pre-commit hook running fmt + clippy + test (fast subset)</item>
  <item>.github/workflows/ci.yml with the M01 gate suite</item>
  <item>docs/adr/0005-lefthook-over-husky.md (the dependency-justification ADR)</item>
</deliverable>
```
Reference form (use when the Phase doc has a detailed `X.2 Files to Change` + `X.3 Detailed Changes` section — the typical case for non-trivial stages):
```xml
<deliverable ref="docs/build-prompts/M02-event-pipeline.md#A.3"/>
```
Use one form or the other, never both. Items in inline form are implicitly ordered (top-to-bottom = implementation order); items in the referenced section are ordered by their position in that section.
`<test_plan_required>`
Required. Almost always `true`. The agent must state the test plan before writing code per `BUILD-PLAYBOOK.md` §3.3. Setting `false` means the stage produces no testable code (rare; usually only the very first scaffolding stage of a project).
```xml
<test_plan_required>true</test_plan_required>
```
`<acceptance_criteria>`
Required. The stage's exit conditions as a checklist. The agent verifies each before surfacing for approval. Distinct from gates (which are CI-style automated checks) and from `<deliverable>` (which is what files exist) — acceptance criteria are behavioral checks the deliverable must satisfy.
Inline form (use when criteria are short and stable):
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
Reference form (use when the Phase doc has a `X.4 Tests` or equivalent section enumerating behavioral checks):
```xml
<acceptance_criteria ref="docs/build-prompts/M02-event-pipeline.md#A.4"/>
```
Use one form or the other, never both.
`<read_prior_stages>`
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
8. Closeout-only tags
These tags are valid only inside `<closeout_stage_prompt>`.
`<cumulative_reads>`
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
`<deliverables>`
Required. The closeout produces three artifacts. Plural form distinguishes from work-stage `<deliverable>`.
```xml
<deliverables>
  <milestone_summary>retrospectives/M01-summary.md (aggregates per-stage retrospectives, scores axes across stages, marks verdict)</milestone_summary>
  <gap_analysis_entry>docs/gap-analysis.md (append new entry; six required sections, none optional)</gap_analysis_entry>
  <pr_description>draft only; do not open PR until explicitly asked</pr_description>
</deliverables>
```
`<gap_analysis_requirements>`
Required. Reference to the playbook section that defines the six-section structure. Self-closing.
```xml
<gap_analysis_requirements ref="BUILD-PLAYBOOK.md#3.4"/>
```
If the closeout has special items to flag in the gap-analysis entry (e.g., a known divergence to resolve), add them inline:
```xml
<gap_analysis_requirements ref="BUILD-PLAYBOOK.md#3.4">
  <special_check>Verify lefthook.yml matches ADR-0005's named gate set; flag any drift</special_check>
</gap_analysis_requirements>
```
`<append_only_verification>`
Required. Names the two append-only checks: local diff and CI job.
```xml
<append_only_verification>
  <local_check>prior content of docs/gap-analysis.md must be a literal prefix of HEAD before commit</local_check>
  <ci_check name="gap-analysis-append-only">fails if any prior line is modified</ci_check>
</append_only_verification>
```
`<three_artifact_review>`
Required. Names the three artifacts the human reviews at PR time and the immutability flag for the ledger entry.
```xml
<three_artifact_review>
  <artifact>code diff (cumulative across milestone)</artifact>
  <artifact>per-stage retrospectives + milestone summary</artifact>
  <artifact>new gap-analysis entry — flagged "IMMUTABLE once committed"</artifact>
  <pushback_blocks_pr>true</pushback_blocks_pr>
</three_artifact_review>
```
9. Optional tags (valid in both schemas)
`<adr_triggers>`
Use when the stage's planned work might trip ADR requirements (per `BUILD-PLAYBOOK.md` §4.8). Pre-flagging keeps the agent from discovering the requirement mid-stage.
```xml
<adr_triggers>
  <trigger>If pre-commit hook tool is changed (e.g., husky over lefthook), file ADR per §4.8</trigger>
  <trigger>If any new core dependency is added beyond those named in spec, file ADR</trigger>
</adr_triggers>
```
`<gotchas>`
Stage-specific traps. Project-wide gotchas live in `docs/gotchas.md` and are read via `<read_first>`. Use this tag only for traps unique to this stage that don't generalize.
```xml
<gotchas>
  <trap>lefthook v1.x changed the YAML format from v0.x — use the v1 syntax explicitly</trap>
  <trap>cargo workspace inheritance for lints requires Cargo.toml [workspace.lints], not [lints]</trap>
</gotchas>
```
`<dependencies>`
Use when a stage depends on artifacts outside the obvious prior-stage chain (e.g., depends on an external review, an upstream branch, an ADR not yet accepted).
```xml
<dependencies>
  <dependency>ADR-0004 (code-signing deferral) must be Accepted before Stage D</dependency>
</dependencies>
```
`<time_box>`
Estimated wall-clock duration. Informs staging boundaries, not deliverable size (per `BUILD-PLAYBOOK.md` §1.2). Reviewed at retrospective for soft gate S4 (within 2× of actual).
```xml
<time_box estimate_hours="6"/>
```
10. Authoring rules
One stage per fenced block. Don't combine stages. The Phase doc may have many fenced blocks but each contains exactly one root element.
No foreign tags. Every tag inside a stage prompt must be in this protocol. Adding a new tag means updating this doc first (and bumping the protocol version per Part 13). Drift is a bug.
No HTML escaping inside `<context>` or prose tags unless required. XML inside fenced markdown blocks parses cleanly with literal angle brackets in attribute values via `&lt;` and `&gt;`. Use them only when the text contains XML-meaningful characters (e.g., "<30 min").
Self-closing for reference tags. When a tag points at an external file with no inline body, use the self-closing form: `<gates milestone="M01"/>` not `<gates milestone="M01"></gates>`.
Stable child element names. Within `<deliverable>`, every child is `<item>`. Within `<scope_locks>`, every child is `<lock>`. Within `<acceptance_criteria>`, every child is `<criterion>`. This consistency makes validation and aggregation simple.
Order tags consistently across milestones. The recommended order:
For work stages: `<context>` → `<read_first>` → `<read_prior_milestones>` (Stage A only when applicable) → `<read_prior_stages>` (B+) → `<deliverable>` → `<test_plan_required>` → `<acceptance_criteria>` → `<scope_locks>` → `<gates>` → `<self_correction_budget>` → `<adr_triggers>` (opt) → `<gotchas>` (opt) → `<time_box>` (opt) → `<dependencies>` (opt) → `<retrospective_requirements>` → `<commit_protocol>` → `<commit_message>` → `<approval_surface>`.
For closeout stages: `<context>` → `<read_first>` → `<read_prior_milestones>` (rare for closeout; included only if absorbing additional carry-forward) → `<cumulative_reads>` → `<deliverables>` → `<gap_analysis_requirements>` → `<append_only_verification>` → `<three_artifact_review>` → `<scope_locks>` → `<gates>` → `<self_correction_budget>` → `<adr_triggers>` (opt) → `<gotchas>` (opt) → `<time_box>` (opt) → `<retrospective_requirements>` → `<commit_protocol>` → `<commit_message>` → `<approval_surface>`.
Consistent ordering makes diffs across milestones immediately scannable.
Reference-first for content-heavy tags. Tags that support both inline and reference forms — currently `<deliverable>`, `<acceptance_criteria>`, `<scope_locks>` — should default to the reference form when the corresponding Phase doc section exists. The Phase doc's `X.2 Files to Change`, `X.3 Detailed Changes`, `X.4 Tests`, and milestone-level `Key constraints` sections are the canonical locations for content; the prompt references rather than restates them.
Use the reference form when:
The content is more than ~10 lines or includes code blocks
The same content already exists in the Phase doc (avoid duplication)
The content is generated/maintained at milestone authoring time, not stage authoring time
Use the inline form when:
The content is short (a handful of items, no code)
The content is stage-specific and doesn't warrant a Phase doc section
The stage is small enough that a self-contained prompt is more readable
Never use both forms in the same tag. A tag with `ref="..."` must be self-closing; a tag with inline content must not have a `ref` attribute. Validation enforces this.
11. Validation
A validation script lives at `scripts/validate-stage-prompts.py` (or your preferred language). It runs in CI on every PR that touches `docs/build-prompts/M[NN]-*.md`.
The script:
Extracts every fenced ```xml block from the Phase doc
Confirms each block contains exactly one root element
Confirms the root tag is `work_stage_prompt` or `closeout_stage_prompt`
Confirms `id` attribute matches `M[0-9]{2}\.[A-Z]`
Confirms all required tags for the schema are present (including `<commit_message>`)
Confirms no foreign tags appear
Confirms ordering matches the recommended order (warning, not error)
Confirms reference-first tags (`<deliverable>`, `<acceptance_criteria>`, `<scope_locks>`) use either inline form OR self-closing `ref="..."` form, never both
Cross-checks: every retrospective referenced in `<read_prior_stages>` exists; every milestone in `<read_prior_milestones>` has the named gap-analysis section + summary section; every file in `<read_first>` exists; every `ref="..."` URI on a content tag resolves to a real Phase doc anchor; the milestone in `<gates milestone="...">` matches the Phase doc's milestone
CI fails on any error; warnings are surfaced in the PR check output.
12. Worked examples
12.1 Work-stage prompt — M01.A
```xml
<work_stage_prompt id="M01.A">
  <context>
    Stage A of M01 (Foundation). Establish the workspace skeleton — Cargo workspace,
    crate boundaries, lefthook hook, CI scaffold. No business logic yet; this stage
    exists so subsequent stages have a place to land. First stage of the project;
    nothing to inherit from prior stages.
  </context>

  <read_first>
    <file>BUILD-PLAYBOOK.md</file>
    <file>docs/identity.md</file>
    <file>docs/gates.md</file>
    <file>spec/agent-runtime-spec.md §0–§0d</file>
    <file>docs/MVP-v0.1.md §M01</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md</file>
  </read_first>

  <deliverable>
    <item>Cargo workspace at repository root with crates: runtime-core, runtime-drone, runtime-main, runtime-sandbox</item>
    <item>Top-level Cargo.toml with workspace members, shared lints, MSRV pinned</item>
    <item>lefthook.yml with pre-commit hook running fmt + clippy + test (fast subset)</item>
    <item>.github/workflows/ci.yml with the M01 gate suite (per docs/gates.md)</item>
    <item>docs/adr/0005-lefthook-over-husky.md (dependency-justification ADR; status: Proposed)</item>
  </deliverable>

  <test_plan_required>true</test_plan_required>

  <acceptance_criteria>
    <criterion>cargo build --workspace succeeds</criterion>
    <criterion>cargo fmt --all -- --check passes (no diff)</criterion>
    <criterion>cargo clippy --workspace --all-targets -- -D warnings passes</criterion>
    <criterion>lefthook install succeeds; pre-commit hook fires on a test commit and blocks a deliberately malformed commit</criterion>
    <criterion>CI workflow file validates against GitHub Actions schema (act --dry-run or yamllint + jsonschema)</criterion>
    <criterion>ADR-0005 filed, status: Proposed, PR-ready</criterion>
  </acceptance_criteria>

  <scope_locks>
    <lock>v0.1 is single-session; no multi-session code paths</lock>
    <lock>STANDARD mode hardcoded; no mode router yet</lock>
    <lock>Anthropic-only; no provider abstraction layer</lock>
    <lock>No business logic in any crate this stage; types and trait stubs only if needed for the workspace to build</lock>
  </scope_locks>

  <gates milestone="M01"/>

  <self_correction_budget>3</self_correction_budget>

  <adr_triggers>
    <trigger>If lefthook is rejected for any reason and a different pre-commit tool is chosen, file ADR-0005 with the alternative</trigger>
  </adr_triggers>

  <gotchas>
    <trap>cargo workspace inheritance for lints requires Cargo.toml [workspace.lints.rust] and [workspace.lints.clippy], not [lints]</trap>
    <trap>lefthook v1.x changed YAML format from v0.x — use v1 syntax explicitly; pin version</trap>
  </gotchas>

  <time_box estimate_hours="6"/>

  <retrospective_requirements ref="prompts/RETROSPECTIVE-TEMPLATE.md"/>
  <commit_protocol ref="BUILD-PLAYBOOK.md#4.7"/>
  <commit_message ref="docs/build-prompts/M01-foundation.md#A.6"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers)</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for next stage)</item>
    <item>draft commit message (Conventional Commits + DCO + session URL footer)</item>
    <item>explicit statement: "Stage M01.A is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```
12.2 Closeout-stage prompt — M01.E
```xml
<closeout_stage_prompt id="M01.E">
  <context>
    Closeout stage of M01 (Foundation). Stages A–D have committed on the milestone branch.
    This stage produces the cumulative artifacts: M01 summary aggregating retrospectives,
    the new (and first) docs/gap-analysis.md entry, and the draft PR description.
    The gap-analysis commit is the final commit on this branch and gates the PR push.
  </context>

  <read_first>
    <file>BUILD-PLAYBOOK.md (especially §3.4, §4.6)</file>
    <file>docs/identity.md</file>
    <file>docs/gates.md</file>
    <file>spec/agent-runtime-spec.md §0–§0d</file>
    <file>docs/MVP-v0.1.md §M01</file>
  </read_first>

  <cumulative_reads>
    <codebase>entire shipped codebase to date (M01.A through M01.D commits on this branch)</codebase>
    <spec>spec/agent-runtime-spec.md (end-to-end, focus on M01-touched sections)</spec>
    <gap_analysis>docs/gap-analysis.md (no prior entries; M01.E writes the first)</gap_analysis>
    <retrospectives>retrospectives/M01.A-retrospective.md, M01.B-, M01.C-, M01.D-retrospective.md (all stages)</retrospectives>
  </cumulative_reads>

  <deliverables>
    <milestone_summary>retrospectives/M01-summary.md (aggregates per-stage retrospectives; scores axes across stages; marks verdict)</milestone_summary>
    <gap_analysis_entry>docs/gap-analysis.md (append first entry; six required sections, none optional; "Carry-forward" section will be empty for M01 — write "None observed.")</gap_analysis_entry>
    <pr_description>draft only; PR opens only on explicit human ask after approval</pr_description>
  </deliverables>

  <gap_analysis_requirements ref="BUILD-PLAYBOOK.md#3.4"/>

  <append_only_verification>
    <local_check>prior content of docs/gap-analysis.md must be a literal prefix of HEAD (trivially true for first entry, but verify the check itself works for future milestones)</local_check>
    <ci_check name="gap-analysis-append-only">added in this stage; verify it fails when given a deliberately mutated prior entry, then passes on the real append</ci_check>
  </append_only_verification>

  <three_artifact_review>
    <artifact>code diff (cumulative M01.A through M01.E)</artifact>
    <artifact>per-stage retrospectives + M01 milestone summary</artifact>
    <artifact>new docs/gap-analysis.md entry — flagged "IMMUTABLE once committed"</artifact>
    <pushback_blocks_pr>true</pushback_blocks_pr>
  </three_artifact_review>

  <scope_locks>
    <lock>Append-only is a hard rule (BUILD-PLAYBOOK.md §4.1, §4.6) — no editing prior entries, ever. M01 has none, but the discipline starts here.</lock>
    <lock>The gap-analysis CI check is part of this stage's deliverable; don't defer it</lock>
  </scope_locks>

  <gates milestone="M01"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>The "Carry-forward" section is required even when empty — write "None observed." rather than omit (BUILD-PLAYBOOK.md §3.4)</trap>
    <trap>Severity is non-elastic — if M01 has a pile of 🔴 Criticals in the fix backlog, the milestone shouldn't ship; surface this rather than rationalize</trap>
  </gotchas>

  <time_box estimate_hours="4"/>

  <retrospective_requirements ref="prompts/RETROSPECTIVE-TEMPLATE.md"/>
  <commit_protocol ref="BUILD-PLAYBOOK.md#4.7"/>
  <commit_message ref="docs/build-prompts/M01-foundation.md#E.6"/>

  <approval_surface>
    <item>new gap-analysis entry text (full)</item>
    <item>diff of docs/gap-analysis.md</item>
    <item>M01-summary.md (full)</item>
    <item>draft PR description (per .github/PULL_REQUEST_TEMPLATE.md)</item>
    <item>draft commit message for the gap-analysis commit</item>
    <item>explicit flag: "This gap-analysis entry is IMMUTABLE once committed. Please review carefully."</item>
    <item>explicit statement: "M01 closeout is ready. I will not commit until you approve."</item>
  </approval_surface>
</closeout_stage_prompt>
```
12.3 Work-stage prompt — M02.A (reference-first style)
Stage A of a non-first milestone. Demonstrates `<read_prior_milestones>` (absorbing M01 carry-forward), reference-first `<deliverable>` and `<acceptance_criteria>` (pointing at the rich Phase doc sections), and reference-first `<scope_locks>` (pointing at the milestone-level Key constraints section).
Compare line count to M01.A above: this prompt is roughly half the size because the substantive content lives in the Phase doc, not duplicated inline.
```xml
<work_stage_prompt id="M02.A">
  <context>
    Stage A of M02 (Event Pipeline). Build hygiene + scaffolds. Absorbs all M01 carry-forward
    🟡 Important items so Stages B–E focus on the real M02 deliverables (LLMProvider trait,
    AnthropicProvider, AgentSdk, Tauri shell). Stage B does not start until Stage A's commit
    is on the milestone branch.
  </context>

  <read_first>
    <file>BUILD-PLAYBOOK.md</file>
    <file>docs/identity.md</file>
    <file>docs/gates.md</file>
    <file>spec/agent-runtime-spec.md §0–§0d, §2, §2b, §11</file>
    <file>docs/MVP-v0.1.md §M02</file>
    <file>docs/build-prompts/M02-event-pipeline.md (Background, Document Structure, Implementation Workflow, Stage A sections A.1–A.4)</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md</file>
  </read_first>

  <read_prior_milestones>
    <gap_analysis_carry_forward milestone="M01"/>
    <milestone_summary milestone="M01" section="Decisions to apply before next parent milestone"/>
  </read_prior_milestones>

  <deliverable ref="docs/build-prompts/M02-event-pipeline.md#A.3"/>

  <test_plan_required>true</test_plan_required>

  <acceptance_criteria ref="docs/build-prompts/M02-event-pipeline.md#A.4"/>

  <scope_locks ref="docs/build-prompts/M02-event-pipeline.md#key-constraints"/>

  <gates milestone="M02"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>Stage A's job is to close M01-carry-forward Important items, not to start Stage B's LLMProvider work — resist scope creep into provider abstractions even if locally tempting</trap>
    <trap>Coverage delta gating script (.github/workflows/scripts/coverage-delta.sh) is a new artifact this stage; verify it triggers correctly on a synthetic regression before relying on it</trap>
  </gotchas>

  <time_box estimate_hours="1.5"/>

  <retrospective_requirements ref="prompts/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Decisions for Stage B: which scaffolds Stage B will import from runtime-core; whether the *_with pattern doc landed in the right form; whether the coverage delta gate ran cleanly on first try</special_log>
  </retrospective_requirements>

  <commit_protocol ref="BUILD-PLAYBOOK.md#4.7"/>
  <commit_message ref="docs/build-prompts/M02-event-pipeline.md#A.6"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers)</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage B)</item>
    <item>draft commit message from M02-event-pipeline.md#A.6 (filled with session URL)</item>
    <item>explicit statement: "Stage M02.A is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```
13. Anti-patterns
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
Mixing inline and reference forms in the same tag. `<deliverable ref="..."><item>...</item></deliverable>` is a schema violation. Pick one form. The validator rejects the mix because the precedence rule (which wins?) is genuinely ambiguous and the right answer is to make the choice explicit at authoring time.
Reference form pointing at a non-existent Phase doc anchor. `<deliverable ref="docs/build-prompts/M02-event-pipeline.md#A.3"/>` requires `M02-event-pipeline.md` to have a heading that resolves to `A.3` (typically `### A.3 Detailed Changes`). The validator cross-checks every `ref="..."` URI against the actual Phase doc structure.
Inline content in a stage that has rich Phase doc sections. The reference form exists because content drift between prompt and Phase doc is a real failure mode — the prompt restates what's in `A.3`, then `A.3` gets edited and the prompt doesn't, and the agent works from a stale snapshot. If the Phase doc has detailed `X.2`/`X.3`/`X.4` sections, the prompt should reference them, not duplicate them.
14. Versioning this protocol
This protocol changes when:
A new tag is needed across all stages (additive)
A tag's semantics change (breaking; requires migration of in-flight Phase docs)
The two-schema split needs revision (e.g., a third schema for a new stage type — unlikely but possible)
Validation rules change (e.g., a previously-warning becomes an error)
Substantive changes get clear `docs(stage-prompt-protocol): ...` commit messages and a CHANGELOG entry. The commit history of this file is itself an audit of how stage prompts evolved.
If this protocol disagrees with `BUILD-PLAYBOOK.md`, the playbook wins. This protocol is the schema; the playbook is the authority on what stages are and how they run.
Changelog
v1.1 — Three additive changes informed by M02 Phase doc audit:
New common tag `<read_prior_milestones>` for Stage A of non-first milestones absorbing prior-milestone carry-forward
New common tag `<commit_message ref="..."/>` (required) referencing the pre-authored commit message in the Phase doc's `X.6` section
Reference-first pattern formalized for content-heavy tags (`<deliverable>`, `<acceptance_criteria>`, `<scope_locks>`): each may use either inline form OR self-closing `ref="..."` form pointing at the corresponding Phase doc section, never both. Validator enforces.
Existing v1.0 prompts remain valid; the additions are backward-compatible (the new tags are required from v1.1 forward, but existing Phase docs can be updated incrementally as they're touched).
v1.0 — Initial protocol. Two-schema split (`<work_stage_prompt>` and `<closeout_stage_prompt>`); common, work-only, closeout-only, and optional tag sets; authoring rules; validation contract; worked examples for M01.A and M01.E; anti-patterns.
---
End of Stage Prompt Protocol.
