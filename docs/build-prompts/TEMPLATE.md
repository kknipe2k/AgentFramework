# Template — Per-Milestone Prompt

This file is the **shape** of a milestone prompt. Copy it to `M[NN]-<short-title>.md` and fill in every section. Sections are not optional; if a section doesn't apply, write "N/A — <one-line reason>" rather than deleting.

The template is annotated. Annotations live as HTML comments and are stripped from the rendered milestone prompt. Comments explain the *why* of each section so future authors understand the intent and don't accidentally hollow out the structure.

---

## When to split a milestone

> **Rule:** If a milestone's scope estimate exceeds **250 lines of prompt** OR **12 hours of work**, **split it** into sub-milestones.

A 540-line opening prompt is too much for a fresh Claude Code session — context window pressure, attention dilution, and acceptance-criteria fatigue all compound. The right move is to split the milestone into sequential sub-milestones, each with its own prompt, branch, and PR.

### Naming convention for sub-milestones

- `M[NN].1-<short-title>.md`, `M[NN].2-<short-title>.md`, etc.
- Sequential — M[NN].1's PR must merge to `main` before M[NN].2's session opens
- Each sub-milestone is its own branch (`claude/m[nn]-<n>-<short-title>`) and its own PR
- Each sub-milestone has its own retrospective entry in `PROCESS-VALIDATION.md`

### How to split

Look at the milestone scope and identify natural boundaries. Common axes:

- **Layer boundaries** — e.g., M01 splits into workspace-skeleton / type-generation / drone-implementation / fuzz-and-polish
- **Subsystem boundaries** — e.g., M04 (Plan + Verify + HITL + Budget) splits into one sub-milestone per primitive
- **Risk boundaries** — e.g., the highest-risk piece is its own sub-milestone so failure surfaces early
- **Test-type boundaries** — sometimes implementation is one sub-milestone and the fuzz/coverage closure is another

Each sub-milestone should:
- Be deliverable in one session (≤12 hours; ideally 5–8)
- Have its own scope, TDD plan, acceptance criteria
- Build on prior sub-milestones (via dependencies declared in "Prerequisite sub-milestones")
- Close cleanly — no "we'll wire this up in the next sub-milestone" hand-waves

If you find a sub-milestone needs further splitting, recurse: M[NN].2.a, M[NN].2.b. Try to avoid this — three levels of sub-milestone is usually a sign the scope was estimated wrong.

### Why split rather than rely on Claude's context window

Even with 200K+ context windows, attention dilutes across long prompts:
- Acceptance criteria mid-prompt get less weight than ones near the end
- Mid-flight, Claude can lose track of what was specified earlier
- Self-correction state gets muddied across unrelated subsystems
- A single failing test can pull focus from the broader plan

Splitting is the cheap intervention that prevents these failure modes.

### Reference: M01 split

M01 was originally written as one 540-line prompt. It was split into four sub-milestones:

- `M01.1-workspace-skeleton.md` (~180 lines, 5–8h)
- `M01.2-type-generation.md` (~200 lines, 6–10h)
- `M01.3-drone-implementation.md` (~280 lines, 12–18h)
- `M01.4-fuzz-and-polish.md` (~150 lines, 4–6h)

M01.3 is the largest — it's at the upper bound of the rule. If it grows, split further.

---

<!-- ============================================================ -->
<!-- IDENTITY — who Claude is for this milestone, what they're doing -->
<!-- ============================================================ -->

# M[NN] — <Milestone title>

> **Milestone:** M[NN] of M11 in `docs/MVP-v0.1.md`
> **Estimated effort:** [X] hours Claude execution + [Y] hours human direction; [N] weeks elapsed at sustained pace
> **Branch:** `claude/m[nn]-<short-kebab-title>` (off `main`)
> **Prerequisite milestones:** [list, or "none — root milestone"]

---

<!-- ============================================================ -->
<!-- READ FIRST — explicit list, with one-line "why" per file       -->
<!-- This is what Claude reads before writing any code. Keep it tight; -->
<!-- if a file isn't strictly needed for THIS milestone, leave it out. -->
<!-- CLAUDE.md is implicit (auto-loaded) but list it anyway for clarity. -->
<!-- ============================================================ -->

## Read first

Before writing any code, read in this order:

1. **`CLAUDE.md`** (repo root) — protocol, hard rules, quality gates, PR workflow, anti-patterns. You should already have this auto-loaded; confirm by stating the §4 Hard Rules at the top of your first response.
2. **`agent-runtime-spec.md`** — read these sections:
   - §0 Project Positioning, §0a Capability Matrix, §0b Three Concepts, §0c Dev Loop, §0d Release Scope (always)
   - **[milestone-specific spec sections, named explicitly with anchors]**
3. **`docs/MVP-v0.1.md`** — read the §M[NN] section in full + the milestone overview table at the top
4. **`docs/adr/`** — read these ADRs:
   - **[list ADRs by number with one-line "why each"]**
5. **`schemas/*.v1.json`** — read these schemas if applicable:
   - **[list, or "N/A for this milestone"]**
6. **`examples/aria/`** — read these reference artifacts if applicable:
   - **[list paths]**

After reading, state in 1-3 sentences what this milestone delivers and the test plan in 3-5 bullets. Wait for confirmation before writing code.

---

<!-- ============================================================ -->
<!-- PROBLEM STATEMENT — what real-world thing is unblocked by this  -->
<!-- milestone, in user terms not engineering terms                  -->
<!-- ============================================================ -->

## Problem statement

[One paragraph, user-facing language. What can a user do after this milestone that they couldn't before? What real problem is unblocked?]

[For early milestones (M1, M2) where there's no user yet, frame as "what Claude / contributors / the build can do that they couldn't before."]

---

<!-- ============================================================ -->
<!-- SCOPE — what's IN, what's OUT. Mirror MVP-v0.1.md but more       -->
<!-- granular. The "out" list is as important as the "in" list —     -->
<!-- it prevents scope creep mid-milestone.                          -->
<!-- ============================================================ -->

## Scope

### In scope (deliver these)

- [Specific, testable deliverable 1]
- [Specific, testable deliverable 2]
- [...]

### Out of scope (do NOT deliver these)

- [Things that look related but belong to a later milestone — name the milestone]
- [Things that look like obvious next steps but are explicitly v1.0 or v2.0+ — reference §0d row]
- [Refactors / improvements that aren't required by acceptance criteria]

If you find yourself wanting to deliver something on the "Out of scope" list, **stop and ask** — never silently expand.

---

<!-- ============================================================ -->
<!-- TDD PLAN — write tests first, in this order. Specifies WHAT     -->
<!-- tests, not what implementation. The implementation falls out of -->
<!-- making the tests pass.                                          -->
<!-- ============================================================ -->

## TDD plan

Before writing any production code, write these tests in this order. Each test should fail when first written; the production code that makes it pass is the implementation.

### Unit tests (Rust — `cargo test`)

1. **[Test name]** — [what it asserts; what production code it drives]
2. **[Test name]** — [...]

### Property tests (Rust — `proptest`)

1. **[Property]** — [the invariant; the input space]

### Fuzz harnesses (Rust — `cargo-fuzz`)

[List, or "N/A — no parsers introduced this milestone"]

### Integration tests (Rust — `cargo test --features integration`)

[List, or "N/A — milestone is unit-test scope"]

### Frontend unit tests (TypeScript — Vitest)

[List, or "N/A — frontend doesn't exist yet"]

### E2E tests (Playwright)

[List, or "N/A — milestone is below the E2E threshold; renderer doesn't run a session yet"]

### Doc tests

[List public API additions that need doc-comment examples; or "N/A — no public API added"]

### Coverage target

- ≥80% line coverage on all new code
- 100% on safety primitives if any are touched ([list applicable safety primitive paths])

---

<!-- ============================================================ -->
<!-- ACCEPTANCE CRITERIA — checkbox format, testable, ties back to   -->
<!-- MVP-v0.1.md M[N] acceptance + adds milestone-prompt detail.    -->
<!-- Every criterion must be verifiable by running a command or      -->
<!-- inspecting an artifact. Vague criteria ("works correctly") are -->
<!-- bugs in the criteria themselves.                                -->
<!-- ============================================================ -->

## Acceptance criteria

The milestone is "done" only when every criterion below is checked:

- [ ] **[Criterion 1]** — verifiable by [exact command or inspection]
- [ ] **[Criterion 2]** — verifiable by [...]
- [ ] [...]

### Quality gates (the must-pass list per CLAUDE.md §6)

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo test --workspace --doc` passes
- [ ] `cargo doc --workspace --no-deps -- -D rustdoc::missing_docs` passes
- [ ] `cargo audit` clean (no high/critical)
- [ ] `cargo deny check` passing
- [ ] `cargo llvm-cov --workspace` shows coverage ≥80% on lines added/modified
- [ ] [Frontend gates if applicable]
- [ ] [E2E gates if applicable]
- [ ] CI green on Linux/macOS/Windows (manually verify by inspecting CI run after push)

---

<!-- ============================================================ -->
<!-- CODE EXPECTATIONS — patterns to match, anti-patterns to avoid,  -->
<!-- file/module layout, anything specific to THIS milestone that's  -->
<!-- not already covered by CLAUDE.md's project-wide rules.          -->
<!-- ============================================================ -->

## Code expectations

### File / module layout

```
[show the directory tree this milestone produces]
```

### Patterns to match

- [Specific pattern other crates/components in the project use that this milestone should follow]

### Naming for this milestone

- [Anything milestone-specific beyond CLAUDE.md §9]

### What NOT to write

- [Things that look like reasonable additions but aren't appropriate for this milestone]

---

<!-- ============================================================ -->
<!-- VERIFICATION COMMANDS — exact commands to run, in order, with   -->
<!-- pass/fail criteria. Copy-pasteable.                             -->
<!-- ============================================================ -->

## Verification commands

Run these in order. All must pass before the milestone is considered done.

```bash
[exact command 1]
# Expected: [pass criterion]
# On failure: [where to look first]

[exact command 2]
# Expected: [pass criterion]
# On failure: [where to look first]
```

---

<!-- ============================================================ -->
<!-- SELF-CORRECTION — milestone-specific guidance that augments      -->
<!-- CLAUDE.md §7. Common failure modes for THIS milestone and how   -->
<!-- to triage them.                                                 -->
<!-- ============================================================ -->

## Self-correction guidance

### Likely failure modes for this milestone

| Failure | Likely cause | First thing to check |
|---|---|---|
| [Specific test failure] | [Hypothesis] | [Diagnostic command or file] |
| [Specific build failure] | [...] | [...] |
| [Lint or coverage failure] | [...] | [...] |

### Escalate if

- After 3 self-correction iterations, any gate is still failing
- The work requires a dependency or schema change not in scope
- A decision is needed that touches §0a primitives or capability enforcement

Per CLAUDE.md §12, escalation surfaces:
- What you tried (1 line per attempt)
- Current failures (full output, not summarized)
- Best current hypothesis
- What you would try next, if anything

---

<!-- ============================================================ -->
<!-- DELIVERABLES — concrete list of files/artifacts produced. The   -->
<!-- PR description's "What this PR does" maps directly to this list. -->
<!-- ============================================================ -->

## Deliverables

After the milestone, these files exist and are committed:

- [Specific file 1] — [purpose]
- [Specific file 2] — [...]
- [...]

These files are updated:

- [`CHANGELOG.md`] — entry under `[Unreleased]`
- [`docs/MVP-v0.1.md`] — milestone status updated (if format calls for it)
- [Other docs that reference what this milestone delivers]

---

<!-- ============================================================ -->
<!-- PR + COMMIT RULE — explicit reminder. CLAUDE.md §8 covers the   -->
<!-- workflow; this section pulls out the do-not-commit rule and any -->
<!-- milestone-specific PR notes (e.g., expected reviewer count).    -->
<!-- ============================================================ -->

## PR + commit rule

Per **`CLAUDE.md` §8 PR + commit workflow** — Claude does not commit until the user explicitly approves.

When all acceptance criteria are checked and all gates pass:

1. Run a final `git status` and `git diff --stat HEAD`.
2. Re-run all quality gates and capture exact results.
3. Draft the PR description following `.github/PULL_REQUEST_TEMPLATE.md`. Include all required sections.
4. **Surface to the user** — PR title, PR description (markdown), diff stat, gate results.
5. State explicitly: *"I will not commit until you approve."*
6. Wait for explicit approval before any `git commit` or `git push`.

PR notes specific to this milestone:

- [Anything unusual about the merge — e.g., "this milestone has 5 logical commits worth preserving as merge-commit, not squash"]
- [Reviewer expectations — e.g., "CODEOWNERS-flagged paths touched: capability/, sandbox/"]
- [Anything else]

---

<!-- ============================================================ -->
<!-- COMMON GOTCHAS — milestone-specific traps. CLAUDE.md §15 has     -->
<!-- project-wide ones; this section adds what's specific to THIS     -->
<!-- milestone (e.g., M1's WAL pragma order matters; M5's L2a vs L2b -->
<!-- distinction; M9's tier-gated review).                           -->
<!-- ============================================================ -->

## Milestone-specific gotchas

1. [Gotcha 1 — what trips people up at this stage]
2. [Gotcha 2 — ...]
3. [...]

---

<!-- ============================================================ -->
<!-- ANTI-PATTERNS — milestone-specific "do not do this." Augments    -->
<!-- CLAUDE.md §9 anti-patterns with milestone-scoped traps.         -->
<!-- ============================================================ -->

## Milestone-specific anti-patterns

- [Specific thing that looks reasonable but breaks the milestone's intent]
- [...]

---

<!-- ============================================================ -->
<!-- ESTIMATED TIME-BOX — soft guidance for the human; not binding   -->
<!-- on the work, but useful for "is this off the rails?" check.     -->
<!-- ============================================================ -->

## Time-box (soft)

- **Reading + planning:** [N] minutes
- **TDD red phase (write all failing tests):** [N] hours
- **TDD green phase (make tests pass):** [N] hours
- **Refactor + polish:** [N] hours
- **Gate verification + PR drafting:** [N] minutes

If actual time exceeds 2× the estimate, surface it. Estimate may be wrong, scope may have grown, or there's a blocker that needs to be named.

---

<!-- ============================================================ -->
<!-- END OF TEMPLATE                                                 -->
<!-- ============================================================ -->
