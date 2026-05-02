# Template — Per-Milestone Specification + Stage Prompts

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

[Test plan as embedded code, not just descriptions. For unit tests, full test file content. For property tests, the proptest! block. For integration tests, the test function body.]

```<lang>
<test file content>
```

#### Coverage target

- [Specific crates/modules and their thresholds]

### A.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M[NN]-<title>.md Stage A (sections A.1 through A.4).

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
