# Retrospectives

This directory holds **per-stage retrospectives** that Claude fills in for every milestone-stage session. They're how the project tests its build pattern (per `docs/build-prompts/PROCESS-VALIDATION.md`).

## What's here

| File | Status | Authored by |
|---|---|---|
| `README.md` (this file) | Stable | hand-written |
| `RETROSPECTIVE-TEMPLATE.md` | Stable | hand-written; per-stage shape |
| `SUMMARY-TEMPLATE.md` | Stable | hand-written; per-parent-milestone roll-up shape |
| `M[NN].<X>-retrospective.md` | Created per stage | **Claude**, during/after the stage session |
| `M[NN]-summary.md` | Created per parent milestone | **Claude**, after the final stage's commit, before the M[NN] PR opens |
| `TRENDS.md` | Optional | **Claude**, when patterns emerge across multiple parent milestones |

## File-naming convention

- Per-stage retrospective: `M01.A-retrospective.md`, `M01.B-retrospective.md`, ..., `M11.D-retrospective.md`. Stages use `A`/`B`/`C`/... within a parent milestone.
- Per-parent-milestone summary: `M01-summary.md`, `M02-summary.md`, ..., `M11-summary.md`.
- Trend log (cross-milestone): `TRENDS.md`.

If a milestone is not staged (small enough to fit one prompt under the 250-line / 12-hour rule in `TEMPLATE.md`), the retrospective is `M[NN]-retrospective.md` and there's no separate summary.

## How retrospectives get authored

Per `CLAUDE.md` §19 Retrospective Protocol:

1. **At stage-session start**, Claude creates a draft retrospective at `M[NN].<X>-retrospective.md` from `RETROSPECTIVE-TEMPLATE.md`.
2. **During the session**, Claude appends to the live observation log AS friction events surface. Don't summarize at the end — details fade.
3. **At session end**, Claude scores the three-axis retrospective, evaluates threshold gates, and proposes decisions in the Decisions section.
4. **Surface alongside the stage's draft commit message.** Per `CLAUDE.md` §8, Claude doesn't commit; the retrospective is part of the surfaced draft. User reviews retrospective + diff stat + commit message together.
5. **User reviews the retrospective.** User validates Claude's self-assessment against observable evidence (especially the do-not-commit hard gate G1).
6. **On approval**, Claude commits the stage on the parent-milestone feature branch (does NOT push). Stage retrospectives accumulate; the PR doesn't draft until the final stage.
7. **At end of the final stage**, Claude creates the `M[NN]-summary.md` aggregating findings across all stages, then drafts the M[NN] PR with all stage commits + all stage retrospectives + the summary.

## What user reviews

The user does **not** fill in retrospectives. The user reviews:

1. **The PR code diff** — does the milestone deliver?
2. **The retrospective Claude filled in** — does Claude's self-assessment match what the user observed?

If user sees a discrepancy (e.g., Claude scored G1 "passed" but a commit is in the git log without prior approval), the user pushes back and Claude revises.

## Why per-stage, Claude-driven

Claude has the live context (friction events, ambiguities, self-correction iterations); the user only sees the diffs and commits. Asking the user to score what they didn't observe is asking them to reconstruct context they never had. Claude self-assessing is more honest about who has the information; user review applies to claims, not first-hand reconstruction.

Per-stage cadence is the early-warning system: Stage A's retrospective (after ~5–8 hours) can catch a pattern problem before Stage B starts, saving compounding error in later stages.

The framework lives in `docs/build-prompts/PROCESS-VALIDATION.md`. This directory is where the actual filled-in retrospectives accumulate as the project progresses.

## After M11

The chain of M[NN].[N] retrospectives + M[NN] summaries + TRENDS.md is part of the project's quality history. A future contributor reading the project a year from now can see exactly where the build pattern hit friction, what was fixed, and what the experience was actually like — friction included.
