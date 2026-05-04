# Template — Per-Milestone Specification + Stage Prompts

> **Protocol version:** v1.0 grandfathered shape. The current authoring shape for M03 onward is the XML stage-prompt schema in `STAGE-PROMPT-PROTOCOL.md` v1.2 (repo root). This template stays as the canonical reference for the v1.0 markdown structure used by M01-foundation.md and M02-event-pipeline.md, and as the source for the milestone-level outline (Background, Document Structure, Implementation Workflow, per-stage X.1–X.6 sections) that the v1.2 prompts still reference via `section="..."` lookups.

This file is the **shape** of a milestone document. Copy it to `M[NN]-<short-title>.md` and fill in every section. Sections are not optional; if a section doesn't apply, write "N/A — <one-line reason>" rather than deleting.

The shape combines the milestone **specification** (rationale, design decisions) and the **stage prompts** (paste-able CLI prompts that drive Claude through implementation) in one document. Each milestone produces one PR; each stage within a milestone produces one commit on the same feature branch. Each stage produces a retrospective per `CLAUDE.md` §19.

---

## When to stage a milestone

> **Rule:** If a milestone's scope estimate exceeds **250 prompt-lines** OR **12 hours of work**, stage it.

A 540-line opening prompt is too much for a fresh Claude Code session — context window pressure, attention dilution, and acceptance-criteria fatigue compound. Staging splits the milestone into 2–4 sequential stages on one feature branch:

- **Each stage is its own commit** (sequential ordering enforced by branch history).
- **Each stage has its own X.5 CLI Prompt** — pasted into a fresh Claude Code session. The fresh session reads the milestone's Background + the specific stage's X.1–X.4 sections from this document, plus the CLI prompt itself.
- **Each stage has its own retrospective** at `docs/build-prompts/retrospectives/M[NN].<X>-retrospective.md` (where `<X>` is `A`, `B`, etc.).
- **The parent milestone has one PR** drafted at the end of the last stage. All stage commits + all stage retrospectives + the parent-milestone summary land in that PR.

### Naming

- Document file: `docs/build-prompts/M[NN]-<short-title>.md` (one per parent milestone).
- Branch: `claude/m[nn]-<short-kebab-title>` (one per parent milestone).
- Stages: `Stage A`, `Stage B`, etc. inside the document.
- Stage retrospectives: `M[NN].A-retrospective.md`, `M[NN].B-retrospective.md`, etc.
- Parent-milestone summary: `M[NN]-summary.md`.

### How to stage

Look at the milestone scope and identify natural boundaries. Common axes:

- **Layer boundaries** — e.g., M01 stages into workspace-skeleton / type-generation / drone-implementation / fuzz-and-polish.
- **Subsystem boundaries** — e.g., M04 (Plan + Verify + HITL + Budget) stages into one stage per primitive.
- **Risk boundaries** — the highest-risk piece is its own stage so failure surfaces early.
- **Test-type boundaries** — sometimes implementation is one stage and the fuzz/coverage closure is another.

Each stage should:
- Be deliverable in one session (≤12 hours; ideally 5–8).
- Have its own scope, TDD plan, acceptance criteria.
- Build on prior stages (declare prerequisite stages explicitly).
- Close cleanly — no "we'll wire this up in the next stage" hand-waves.
- End with a tight commit on the feature branch (no push until the parent-milestone PR is approved).

If a stage needs further substaging, recurse: `Stage A.1`, `Stage A.2`. Try to avoid this — three levels usually means scope was estimated wrong.

### Reference: M01 stages

M01 was originally a 540-line single prompt. It became:

- `M01-foundation.md` Stage A (~290 lines, 5–8h) — Workspace skeleton.
- `M01-foundation.md` Stage B (~270 lines, 6–10h) — Type generation pipeline.
- `M01-foundation.md` Stage C (~340 lines, 12–18h) — Drone Phase 1 implementation.
- `M01-foundation.md` Stage D (~250 lines, 4–6h) — Fuzz + polish + closeout.

Stage C is at the upper bound. If it grows, it gets sub-staged.

### Why stage rather than rely on Claude's context window

Even with 200K+ context, attention dilutes across long prompts:
- Acceptance criteria mid-prompt get less weight than ones near the end.
- Mid-flight, Claude can lose track of what was specified earlier.
- Self-correction state gets muddied across unrelated subsystems.
- A single failing test can pull focus from the broader plan.

Staging is the cheap intervention that prevents these failure modes. Per-stage retrospectives also surface friction early: a Stage A retrospective after ~5–8 hours can catch a pattern problem before Stage B starts, saving 25+ hours of compounding error.

---

## Document Shape

```
# M[NN] <Short Title> — Specification + Stage Prompts

**Date:** YYYY-MM-DD
**Status:** Design approved — implement one stage at a time, in order
**Scope:** [one-paragraph summary]

[Background and Design Decision]
[Document Structure table]
[Implementation Workflow code block]

## Stage A — <Title>
### A.1 Problem Statement
### A.2 Files to Change
### A.3 Detailed Changes
### A.4 Tests
### A.5 CLI Prompt
### A.6 Commit Message

## Stage B — <Title>
### B.1 ... B.6 ...

[... additional stages as needed ...]

## Summary Table
## Verification Checklist
```

The header sections (Date through Implementation Workflow) live once at the top. Each stage repeats X.1–X.6. The Summary Table and Verification Checklist live at the bottom.

---

<!-- ============================================================ -->
<!-- HEADER BLOCK — date, status, scope                            -->
<!-- ============================================================ -->

# M[NN] <Short Title> — Specification + Stage Prompts

**Date:** YYYY-MM-DD
**Status:** Design approved — implement one stage at a time, in order
**Scope:** [One-paragraph summary of what this milestone delivers, in user-facing terms when possible. Reference the spec sections that govern.]

---

<!-- ============================================================ -->
<!-- BACKGROUND AND DESIGN DECISION — front-load the WHY            -->
<!-- ============================================================ -->

## Background and Design Decision

**Problem:** [What real-world thing is unblocked by this milestone? In user terms.]

**Solution:** [What this milestone delivers, in 2–4 sentences. Reference the stages by name.]

**Why one PR for the parent milestone (not one PR per stage):** [If the milestone has stages, justify the parent-milestone-as-PR pattern here. Sub-milestone-as-PR was over-engineering; stages-as-commits-on-one-branch gives the same incremental discipline.]

**Why stages, not a single prompt:** [Cite the scope-split rule; note the original prompt size; explain the natural boundaries.]

**Key constraints:** [Bullet list of non-negotiables — scope from §0d, capability adherence, etc.]

**License:** Apache 2.0; DCO sign-off on every commit.

**Existing patterns to mirror:** [List spec sections, ADRs, prior milestone files that codify patterns this milestone follows.]

**Pre-existing legacy file inventory:** [Required when this milestone touches a tree that already exists from a prior milestone (e.g., M03 touches `src/` which M02 created; M07 touches `crates/runtime-main/` which M02 created). List every tracked-but-orphaned file the milestone might trip over: legacy CommonJS files conflicting with `"type": "module"`, dead imports left behind, fixture files that prettier/eslint will scan. Each entry: file path + one-line "why it exists / what it conflicts with / disposition for this milestone (delete | preserve | refactor)." Empty if this is the first milestone touching a fresh tree. Source: M02.E friction r5 — `src/counter.{js,test.js}` legacy files weren't inventoried at M02 authoring and tripped prettier/eslint mid-stage.]

---

<!-- ============================================================ -->
<!-- DOCUMENT STRUCTURE — quick stage table                         -->
<!-- ============================================================ -->

## Document Structure

| Stage | Summary | Estimated effort |
|---|---|---|
| **A** | [one-line summary] | ~Xh |
| **B** | [...] | ~Yh |
| **C** | [...] | ~Zh |
| **D** | [...] | ~Wh |

Total: [sum of estimates]. ~10–15 hours human direction.

**Estimation calibration.** When authoring this milestone, recalibrate effort
estimates against the prior milestone's actual elapsed time per the
retrospectives. M01's method overestimated by ~3× (estimated 29–46h, ran
9–14h, ratio 0.3×). For each stage, name the analogous M01/prior-milestone
stage and use that stage's actual elapsed time as the floor; add complexity
multipliers for new domain (e.g., adding LLM provider work in M02 vs purely
Rust workspace setup in M01) but base the number on observed prior-stage
time, not intuition.

---

<!-- ============================================================ -->
<!-- IMPLEMENTATION WORKFLOW — the constant cycle                   -->
<!-- ============================================================ -->

## Implementation Workflow

Each stage runs through this exact cycle:

```
1. /clear                     — fresh context (only between stages)
2. Paste CLI Prompt below     — Claude writes failing tests first, then implements
3. cargo test --workspace     — confirm new tests fail before any production code
4. implement                  — Claude makes production changes
5. cargo test --workspace     — all tests green
6. cargo clippy + fmt + audit — zero warnings
7. cargo llvm-cov             — coverage threshold met
8. fill in retrospective      — docs/build-prompts/retrospectives/M[NN].<X>-retrospective.md
9. commit (no push)           — exact commit message provided per stage
10. user reviews + approves   — Claude does NOT push without approval
11. push (final stage only)   — to PR draft + open the M[NN] PR
```

**Rule:** If a new test passes before implementation, the test is wrong — stop and fix the test.

**Rule:** Stages are sequential. Stage B does not start until Stage A's commit is on the feature branch (locally is sufficient; push is optional). The parent-milestone PR pushes only at the end of the final stage.

**Rule per `CLAUDE.md` §8:** Claude does not commit without user approval. After tests pass + retrospective filled, Claude surfaces the diff stat + retrospective + draft commit message. User approves; Claude commits.

**Rule per `CLAUDE.md` §19:** Each stage produces a retrospective; the final stage also produces an `M[NN]-summary.md` aggregating across stages.

---

<!-- ============================================================ -->
<!-- STAGE A (template; repeat for each stage)                      -->
<!-- ============================================================ -->

## Stage A — <Stage Title>

<!-- WEBCHECK header — required when the stage touches fast-moving tooling.
     Source: M02-summary.md "Decisions to apply before next parent milestone"
     (M02.E surprise event 1 — recurring pattern across stages that touch the
     npm / Tauri / esbuild / Vite ecosystem). The author web-verifies each URL
     against the prompt body BEFORE the fresh session opens, per CLAUDE.md §12
     web-first rule. Omit the WEBCHECK header for stages that touch only
     stable Rust workspace surfaces. -->

**WEBCHECK:** [List authoritative URLs to verify against this stage's prompt body before code. Example for a stage touching Tauri 2.x + Vite + ESLint:
- https://v2.tauri.app/develop/tests/webdriver/ (Tauri 2.x E2E framework)
- https://vitejs.dev/guide/ (Vite root convention + script tag references)
- https://eslint.org/docs/latest/use/configure/migration-guide (ESLint flat-config)
- https://docs.rs/keyring/latest/keyring/ (keyring crate API surface)
- https://docs.anthropic.com/en/api/messages (provider HTTP wire format)

Verify the prompt's claims about API shapes, version pins, and best practices against each URL. If a claim is stale, update the prompt body BEFORE the fresh session paste — never let a fresh session work from a stale snapshot.]

### A.1 Problem Statement

[What this stage delivers, in 1–2 paragraphs. Reference what's net-new vs what edits an existing file. End with a one-line success criterion.]

**New artifacts:**
- [List of files this stage creates]

### A.2 Files to Change

| File | Change |
|---|---|
| `path/to/file` | **New** — [one-line description] |
| `other/path` | **Edited** — [one-line description] |

### A.3 Detailed Changes

[Surgical edit instructions. For new files: full content (or template). For edits: Find / Replace blocks with exact OLD code and exact NEW code.]

#### `path/to/file` (new) or (edited)

```<lang>
<exact content or surgical Find/Replace>
```

[Implementer notes inline where the spec leaves room for judgment.]

### A.4 Tests

#### Pedantic-pass preflight (for stages adding new modules)

Before writing the test plan, the author runs through this clippy
pedantic+nursery checklist for every new module the stage introduces — the
patterns recurred in every M02 stage retrospective and consolidating here
keeps fresh sessions from re-discovering them. Per `docs/gotchas.md` #21.

- [ ] `redundant_pub_crate` — used plain `pub` in private modules, not `pub(crate)`
- [ ] `derive_partial_eq_without_eq` — `serde_json::Value` containment carries `#[allow]` with one-line rationale
- [ ] `unused_async` — cross-platform `cfg` variants with mismatched await trees suppressed per-fn
- [ ] `default_trait_access` — explicit type (`HashMap::default()`) over inferred (`Default::default()`)
- [ ] `match_wildcard_for_single_variants` — explicit binding over `_` when a single variant remains
- [ ] `cast_precision_loss` / `suboptimal_flops` — exact numeric types pinned with inline precision claim
- [ ] `struct_excessive_bools` — 3+ `bool` fields collapsed to typed flag enum/struct
- [ ] `missing_const_for_fn` — pure constructors marked `const fn`
- [ ] `unnecessary_literal_bound` / `doc_markdown` — code identifiers backticked in doc comments

#### Default test plan for stages adding a new safety primitive

Pattern proven across M01.C / M02.A / M02.C / M02.D / M02.E (per
`M02-summary.md` Decisions): `(N) unit tests for the testable seam (\`*_with\`
/ `from_streams`) + (M) integration tests for end-to-end behavior`. The
testable seam decouples OS calls (the structural-100% holdout) from
business logic; integration tests cover the whole-system path that the
seam abstracts away.

Use as the starting template for any new safety-primitive stage; specialize
N and M to the surface area.

#### Test files

[Test plan as embedded code, not just descriptions. For unit tests, full test file content. For property tests, the proptest! block. For integration tests, the test function body.]

```<lang>
<test file content>
```

#### Coverage target

- [Specific crates/modules and their thresholds]

**Safety primitive coverage gate.** When a stage delivers a safety primitive
(per CLAUDE.md §5: drone, capability enforcer, plan state machine,
snapshot/recovery), the dual-gate policy applies:

- Workspace ≥80% (general)
- Safety primitive crate ≥95% line / region (with documented OS-signal-
  orchestrator exclusions via `--ignore-filename-regex` if cross-platform
  100% is structurally infeasible)

The exclusion list goes inline in the crate's coverage configuration
(`Cargo.toml` or `coverage.toml`) with a one-line rationale per excluded
function. See M01.C codification at commit `1dec4ba` for the established
pattern.

**Doc-to-CI invariant.** When a stage adds a new exclusion to a coverage gate,
update both (a) `.github/workflows/ci.yml` `--ignore-filename-regex` (the
authoritative gate) AND (b) `CLAUDE.md` §5 documented exclusion list AND (c)
the per-stage retrospective's `[END] Coverage holdouts` subsection in the
same commit. The Stage E doc-to-CI drift bug (`key_store.rs` documented as
excluded but not actually in the workflow regex; PR #42 follow-up commit
`9b741e3` fixed) is the cautionary tale — schedule the retro check BEFORE
the surface-for-approval step, not after.

### A.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M[NN]-<title>.md Stage A (sections A.1 through A.4).

[For stages B, C, D, ...: also include this block before STEP 1.]
[For Stage A: skip — no prior retrospective exists yet.]

Read prior stage retrospectives for guidance:
  docs/build-prompts/retrospectives/M[NN].A-retrospective.md
  [add rows for each prior stage]
  Focus: [END] "Decisions for the next stage" sections + any [LIVE]
  friction events flagged as relevant to this stage. Apply decisions.

Read docs/gap-analysis.md for any Carry-forward items targeting this
stage (look at the most recent entry's Carry-forward section + any
prior milestone's Fix backlog items still open).

═══ STEP 1 — WRITE FAILING TESTS ═══

[Specific instructions: which test files to create, with which content.]

Run: <exact command>
Confirm: <exact failure mode>

If any test passes before implementation, the test is wrong — stop and fix it.

═══ STEP 2 — IMPLEMENT ═══

[Numbered steps: 1, 2, 3, ... Each step is a single change with the file path
and a description of what to put in it. For complex changes, reference back to
A.3 ("see A.3 for the exact content").]

═══ STEP 3 — VERIFY ═══

Run each gate; all must pass:
  <exact commands one per line>

If any gate fails, follow CLAUDE.md §7 self-correction. Max 3 iterations
then surface.

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M[NN].A-retrospective.md

Fill in [LIVE] sections, [END] scoring, threshold gates, decisions for Stage B.

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD
Re-run all gates one final time.

Surface: diff stat, gate results, M[NN].A retrospective, draft commit from A.6.
State: "Stage A is ready. I will NOT commit until you approve."
Wait for explicit approval. Do NOT push (push waits for final stage).

On approval (Stage A — work stage; not the final stage of a parent milestone):
1. Commit Stage A on the parent-milestone branch (do NOT push).
2. Stop. Surface the commit. The next stage (Stage B, or Phase Closeout
   if A was the last work stage) is opened in a fresh session.
3. Per CLAUDE.md §20, the Phase Closeout stage (Gap Analysis) is the
   final commit on the parent-milestone branch and gates the PR push.

The same "On approval" sequence applies verbatim to every non-final
work stage (Stage B, Stage C, Stage D, ...). Do NOT push and do NOT
open a PR from a work stage — those happen only after Phase Closeout.
```

### A.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
<type>(<scope>): M[NN] Stage A — <one-line summary>

<paragraph describing what shipped in this stage>

Refs: M[NN]-<title>.md §A
Retrospective: docs/build-prompts/retrospectives/M[NN].A-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- (Repeat Stage B / C / D using the same X.1–X.6 structure)      -->
<!-- ============================================================ -->

## Stage B — <Stage Title>

[X.1 ... X.6, same shape as Stage A]

---

<!-- ============================================================ -->
<!-- Phase Closeout — Gap Analysis (always the FINAL stage,         -->
<!-- regardless of how many work stages preceded it). Per           -->
<!-- CLAUDE.md §20, every parent milestone produces an              -->
<!-- append-only entry in docs/gap-analysis.md.                     -->
<!-- ============================================================ -->

## Stage <last letter> — Phase Closeout: Gap Analysis

> **Per CLAUDE.md §20.** This stage runs after all prior stages commit and the parent-milestone summary lands. It produces one new entry in `docs/gap-analysis.md`. The gap analysis commit is the final commit on the parent-milestone branch — it gates the PR push.

### <X>.1 Problem Statement

Generate the M[NN] entry in `docs/gap-analysis.md`. Cumulative review of code-vs-spec across all milestones to date (not just this one). Append-only — never edit prior entries.

### <X>.2 Files to Change

| File | Change |
|---|---|
| `docs/gap-analysis.md` | **Edited (append-only)** — new section appended at the bottom per the entry template in the file's header |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes that M[NN] gap analysis entry was added |

### <X>.3 Detailed Changes

The entry follows the six-section template defined at the top of `docs/gap-analysis.md`. Do NOT diverge from the template; do NOT skip sections (write "None observed." if a section truly has nothing to report).

**Process:**

1. Re-read `agent-runtime-spec.md` end-to-end (yes, all of it — at least skim with focus on sections this milestone touched).
2. Read every file produced or edited across all stages of this milestone (and prior milestones if cumulative review surfaces issues there).
3. Read the prior `docs/gap-analysis.md` entries in full to know what's outstanding.
4. Draft the new entry per the template.
5. Run the append-only check locally before surfacing: `git show origin/main:docs/gap-analysis.md > /tmp/gap-base.md && diff /tmp/gap-base.md <(head -n "$(wc -l < /tmp/gap-base.md)" docs/gap-analysis.md)` — must be empty.

### <X>.4 Tests

No new code tests. Verification is the append-only check (CI-enforced) plus user review of the entry's substance.

#### Coverage target

N/A — documentation stage.

### <X>.5 CLI Prompt

```
Read CLAUDE.md §20 (Gap Analysis Protocol) and docs/gap-analysis.md
header (entry template).
Read docs/build-prompts/M[NN]-<title>.md Stage <last letter> sections
<X>.1 through <X>.4.

═══ STEP 1 — INGEST ═══

Read in order:
  1. agent-runtime-spec.md (skim; focus on sections this milestone touched)
  2. All files produced/edited across this milestone's prior stages
     (commit list: git log --oneline main..HEAD)
  3. Prior gap-analysis.md entries in full (every one — short for early
     milestones, longer for late ones)
  4. This milestone's per-stage retrospectives + the M[NN]-summary

═══ STEP 2 — DRAFT THE ENTRY ═══

Append to docs/gap-analysis.md a new section following the six-section
template at the top of that file:

  ## M[NN] — <Title> (<YYYY-MM-DD>, commit `<sha-of-prior-stage-commit>`)
  ### Codebase deep dive
  ### Adherence to spec
  ### Spec review (forward-looking)
  ### Fix backlog
  ### Carry-forward from prior milestones
  ### Sign-off

Severity in the Fix backlog is non-elastic. If everything is "Important,"
re-prioritize. Critical = "must fix before next milestone starts." A pile
of Criticals is a signal the milestone shouldn't have shipped; surface
that honestly.

═══ STEP 3 — VERIFY APPEND-ONLY ═══

Run locally:
  git show origin/main:docs/gap-analysis.md > /tmp/gap-base.md
  base_lines=$(wc -l < /tmp/gap-base.md)
  diff /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md)

Output must be empty. If it isn't, prior content was modified; revert and
re-edit by APPENDING ONLY at the bottom.

═══ STEP 4 — SURFACE TO USER ═══

Run: git status, git diff docs/gap-analysis.md
Surface: the new entry (full text), the diff, draft commit message from
<X>.6.

State: "M[NN] Gap Analysis is ready. I will NOT commit until you approve.
Please review the entry — once committed, prior entries are immutable
forever per CLAUDE.md §20."

Wait for explicit approval.

On approval (Stage <last letter> — Phase Closeout: Gap Analysis; final stage):
1. Commit Stage <last letter> on the parent-milestone branch.
2. Push the branch (first push for the milestone — push waits until
   after Stage <last letter> per CLAUDE.md §20).
3. Draft the parent-milestone PR description. Surface for approval.
4. On approval to open: use mcp__github__create_pull_request to open
   the PR. Do NOT merge.
```

### <X>.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
docs(gap-analysis): M[NN] — append cumulative product+spec audit

Per CLAUDE.md §20. Reviews codebase to date against agent-runtime-spec.md;
records adherence findings, spec gaps, and prioritized fix backlog. This
entry is immutable — future milestones report status via Carry-forward.

Refs: M[NN]-<title>.md Stage <last letter>

https://claude.ai/code/session_<id>
EOF
)"
```

---

## Summary Table

| Stage | New Files | Edited Files | Tests Added | Effort |
|---|---|---|---|---|
| **A** [title] | [count or list] | [count or list] | [count + summary] | ~Xh |
| **B** [title] | [...] | [...] | [...] | ~Yh |
| **Total** | [sum] | [sum] | [sum + summary] | ~Total h |

---

## Verification Checklist

Before approving the M[NN] PR (final stage's surface), verify:

### Automated (gates)

- [ ] `cargo fmt --all -- --check` — zero diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- [ ] `cargo build --workspace` — succeeds on Linux/macOS/Windows × stable + MSRV
- [ ] `cargo test --workspace` — all tests pass
- [ ] [Other gates per milestone]
- [ ] CI green on all OS × toolchain cells

### Manual

- [ ] [Manual checks specific to this milestone — e.g., drone smoke, framework load, etc.]
- [ ] All stage retrospectives present and filled in
- [ ] `M[NN]-summary.md` aggregates across stages with verdict
- [ ] M[NN] PR description references all stage commits + retrospectives
- [ ] CHANGELOG `[Unreleased]` reflects what M[NN] actually delivered
- [ ] `docs/MVP-v0.1.md` §M[N] acceptance criteria all `- [x]`

### Approval gate (per CLAUDE.md §19)

- [ ] **Hard Gate G1: do-not-commit-until-approved held** — every stage commit happened only after explicit user approval
- [ ] User has reviewed each stage retrospective; scoring matches observable evidence
- [ ] M[NN]-summary verdict is "Pattern held" (sound) or "Pattern held with friction"; not "Pattern strained"

---

*End of template. Replace placeholders, repeat the stage block per stage, fill in summary + verification checklist for the milestone.*
