# Retrospectives

This directory holds **per-session retrospectives** that Claude fills in for every milestone session. They're how the project tests its build pattern (per `docs/build-prompts/PROCESS-VALIDATION.md`).

## What's here

| File | Status | Authored by |
|---|---|---|
| `README.md` (this file) | Stable | hand-written |
| `RETROSPECTIVE-TEMPLATE.md` | Stable | hand-written; per-session shape |
| `SUMMARY-TEMPLATE.md` | Stable | hand-written; per-parent-milestone roll-up shape |
| `M[NN].[N]-retrospective.md` | Created per session | **Claude**, during/after the milestone session |
| `M[NN]-summary.md` | Created per parent milestone | **Claude**, after the last sub-milestone of that parent merges |
| `TRENDS.md` | Optional | **Claude**, when patterns emerge across multiple parent milestones |

## File-naming convention

- Per-sub-milestone retrospective: `M01.1-retrospective.md`, `M01.2-retrospective.md`, ..., `M11.4-retrospective.md`
- Per-parent-milestone summary: `M01-summary.md`, `M02-summary.md`, ..., `M11-summary.md`
- Trend log (cross-milestone): `TRENDS.md`

If a milestone is not split into sub-milestones (e.g., `M07-registry-import.md` doesn't split), the retrospective is `M07-retrospective.md` and there's no separate summary.

## How retrospectives get authored

Per `CLAUDE.md` §19 Retrospective Protocol:

1. **At session start**, Claude creates a draft retrospective at `M[NN].[N]-retrospective.md` from `RETROSPECTIVE-TEMPLATE.md`.
2. **During the session**, Claude appends to the live observation log AS friction events surface. Don't summarize at the end — details fade.
3. **At session end**, Claude scores the three-axis retrospective, evaluates threshold gates, and proposes decisions in the Decisions section.
4. **Surface to the user alongside the PR description.** Per `CLAUDE.md` §8, Claude doesn't commit; the retrospective is part of the surfaced PR draft.
5. **User reviews the retrospective alongside the code.** User validates Claude's self-assessment against observable evidence (especially the do-not-commit hard gate G1).
6. **On approval**, the retrospective is committed alongside the milestone code on the same PR.
7. **At end of last sub-milestone of a parent milestone**, Claude creates the `M[NN]-summary.md` aggregating findings.

## What user reviews

The user does **not** fill in retrospectives. The user reviews:

1. **The PR code diff** — does the milestone deliver?
2. **The retrospective Claude filled in** — does Claude's self-assessment match what the user observed?

If user sees a discrepancy (e.g., Claude scored G1 "passed" but a commit is in the git log without prior approval), the user pushes back and Claude revises.

## Why per-session, Claude-driven

Claude has the live context (friction events, ambiguities, self-correction iterations); the user only sees the final PR. Asking the user to score what they didn't observe is asking them to reconstruct context they never had. Claude self-assessing is more honest about who has the information; user review applies to claims, not first-hand reconstruction.

The framework lives in `docs/build-prompts/PROCESS-VALIDATION.md`. This directory is where the actual filled-in retrospectives accumulate as the project progresses.

## After M11

The chain of M[NN].[N] retrospectives + M[NN] summaries + TRENDS.md is part of the project's quality history. A future contributor reading the project a year from now can see exactly where the build pattern hit friction, what was fixed, and what the experience was actually like — friction included.
