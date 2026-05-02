# Process Validation: Did the Pattern Work?

> **Purpose:** This is the **reference framework** for evaluating whether the prompt-driven build pattern works — not the test for what each milestone ships. Each milestone's acceptance criteria verify that milestone shipped; this framework verifies the **build pattern** is sound enough to apply across M02–M11.

## Process retrospectives vs. product gap analysis

This file and the per-stage retrospectives evaluate the **build process** — did Claude have what it needed, did the workflow surface decisions at the right time, did the do-not-commit rule hold. The product itself — does the code match the spec, what did the spec get wrong, what's the prioritized fix backlog — is evaluated separately in `docs/gap-analysis.md` (append-only) per `CLAUDE.md` §20. The two artifacts have different audiences and different change rules:

| Artifact | What it evaluates | Author | Mutability |
|---|---|---|---|
| `retrospectives/M[NN].<X>-retrospective.md` | Build *process* per stage | Claude during/after stage | Live during stage; finalized at stage end |
| `retrospectives/M[NN]-summary.md` | Aggregated process across stages | Claude at end of final stage | Once written, not edited |
| `docs/gap-analysis.md` | Build *product* — code↔spec, cumulatively | Claude in Phase Closeout (final stage) | **Append-only forever** per §20; prior entries immutable |

User reviews all three at PR time. This document defines the gates for the first two; `CLAUDE.md` §20 defines the protocol for the third.

## Roles — who does what

The validation is **Claude-driven**, not user-driven. Claude has the live context (friction events, ambiguities, self-correction iterations); the user only sees the final PR. Asking the user to score a session they only partially observed asks them to reconstruct context they never had.

| Role | Responsibility |
|---|---|
| **Claude** | Maintain the live observation log AS the session unfolds. At session end, score the retrospective, evaluate threshold gates, propose decisions for the next stage. Surface the filled-in retrospective alongside the PR description. |
| **User** | Review Claude's self-assessment alongside the code. Validate against observable evidence (especially: did the do-not-commit rule actually hold? did Claude actually NOT commit before approval?). Approve the assessment, push back on scoring, or request additional retrospective entries. |

This split is enforced by `CLAUDE.md` §19 Retrospective Protocol. Per-session retrospectives are deliverables on every milestone PR, same as the code itself.

## How retrospectives get created

| Scope | File | Authored by | When |
|---|---|---|---|
| Per-stage retrospective | `docs/build-prompts/retrospectives/M[NN].<X>-retrospective.md` (where `<X>` is `A`, `B`, etc.) | Claude during/after the stage session | Filled live during work; finalized at session end alongside the stage's draft commit |
| Per-parent-milestone summary | `docs/build-prompts/retrospectives/M[NN]-summary.md` | Claude at the end of the final stage | Aggregates findings across the milestone's stage retrospectives; gates the M[NN] PR draft |
| Cross-milestone trend log (optional) | `docs/build-prompts/retrospectives/TRENDS.md` | Claude when patterns emerge across multiple parent milestones | Updated as it becomes useful; not a per-session deliverable |

**Per-milestone-as-PR pattern:** stages are commits on one feature branch (`claude/m[nn]-<title>`); the PR drafts only at the end of the final stage. Each stage commit lands only after user approval per `CLAUDE.md` §8. The PR opens with all stage commits + all stage retrospectives + the parent-milestone summary.

Templates live in `docs/build-prompts/retrospectives/`:

- `RETROSPECTIVE-TEMPLATE.md` — the per-session shape (live log + scoring + gate evaluation + decisions)
- `SUMMARY-TEMPLATE.md` — the per-parent-milestone roll-up shape

## Why per-stage, not just per-parent-milestone

Per-stage retrospectives capture friction early. M01 Stage A's retrospective (after ~5–8 hours) can surface a pattern problem before M01 Stage B's session opens — saving 25+ hours of compounding error. A retrospective only at end of M01 (after all four stages) is a 30+ hour feedback loop. The per-stage cadence is the early-warning system. The parent-milestone summary then aggregates patterns across stages and gates the next parent milestone.

## The three axes of evaluation

A milestone session is evaluated on three independent concerns:

### Axis 1: Process — did the prompt-driven workflow work?

Did Claude have what it needed to execute autonomously? Did the workflow surface decisions to the user at the right moments? Not about whether the code is good (Axis 2). About whether the *interaction* worked.

Sample questions (full set in RETROSPECTIVE-TEMPLATE.md):
- Was `CLAUDE.md` sufficient orientation, or did Claude ignore parts of it / get confused by parts of it?
- Did the milestone prompt's "Read first" list correctly orient before any code was written?
- Did Claude state the deliverable + test plan before writing code (per CLAUDE.md §16 session-start checklist)?
- Did Claude self-correct effectively when gates failed, or did it spiral?
- Did Claude actually NOT commit before approval (the most important rule)?
- Did Claude escalate the right things and proceed on the right things (per CLAUDE.md §12)?

### Axis 2: Product — did the artifact meet our standards?

Is what shipped actually good? About the milestone deliverables.

Sample questions:
- Does the code match `CLAUDE.md` §9 style + naming?
- Are the tests behavior tests, not tautology tests (`CLAUDE.md` §5)?
- Are public APIs documented with examples (`CLAUDE.md` §6)?
- Are the deliverables what the milestone promised, no more, no less?
- Would a stranger picking up this code understand it without reading the spec?
- Are anti-patterns from `CLAUDE.md` §9 absent?

### Axis 3: Pattern — does this generalize to remaining milestones?

The meta-question. If the prompt format is wrong, repeating it 10 more times multiplies the wrongness.

Sample questions:
- Were sections of `TEMPLATE.md` dead weight? (Sections that contributed nothing should be removed.)
- Were sections missing? (Sections that should have been in the prompt but weren't should be added to TEMPLATE.md.)
- Are milestone-specific gotchas useful for other milestones, or were they truly local? (Generalizable ones move to `CLAUDE.md` §15.)
- Did the time-box estimate match reality? (If 2× off, the estimation method needs revision.)
- Were there moments of *implicit* protocol — things that should have been written down but weren't?

## Scoring

Each axis question gets a 1–5 score:

| Score | Meaning |
|---|---|
| **5** | Worked exactly as the protocol intended. No friction. |
| **4** | Worked, with one minor friction event noted. |
| **3** | Worked, with multiple friction events; pattern needs revision before next milestone. |
| **2** | Partially worked; significant gaps in the protocol. |
| **1** | Failed; the protocol does not support this workflow. |

The retrospective template lays out specific questions per axis and computes axis totals.

## Threshold criteria — is the pattern good enough to scale?

Apply these gates after scoring. **All hard gates must pass for the next stage (or the next parent milestone) to proceed without protocol revision.**

### Hard gates (any fail = stop and revise)

- **G1: do-not-commit-until-approved rule held.** Claude did not commit without explicit user approval, ever, during the session. Even one violation means the protocol's most important rule isn't reliable.
- **G2: no Severity-5 friction events.** No moment where Claude couldn't proceed and the prompt was insufficient.
- **G3: no protocol drift events left unaddressed.** Every entry in the protocol-drift log either (a) Claude self-corrected, OR (b) the user has documented the prompt/CLAUDE.md fix needed.
- **G4: the milestone or stage actually completed.** For stages: commit on the parent-milestone branch with all stage acceptance criteria checked. For non-staged milestones: PR merged with all acceptance criteria checked.
- **G5: scores ≥3 in every individual row across all three axes.** A 1 or 2 in a single row points to a specific gap that compounds at scale.

### Soft gates (advisory; weigh together)

- **S1:** Process axis total ≥30 / 40
- **S2:** Product axis total ≥32 / 40
- **S3:** Pattern axis total ≥25 / 35
- **S4:** Time-box estimate within 2× of actual elapsed time
- **S5:** ≤3 Severity-3 friction events (multiple Severity-3s suggest sustained friction even if no individual event was blocking)

### Outcome matrix

| Hard gates | Soft gates | Verdict |
|---|---|---|
| All pass | All pass | **Pattern is sound.** Proceed to next stage (or next parent milestone). Apply minor revisions to CLAUDE.md based on Axis 3 notes. |
| All pass | 1–2 fail | **Pattern is sound but rough.** Revise CLAUDE.md and TEMPLATE.md to address the soft gates first; then proceed. |
| All pass | 3+ fail | **Pattern works but has friction.** Stop. Spend a session iterating on CLAUDE.md / TEMPLATE.md before the next milestone. The cost of fixing now is hours; the cost of running 10 milestones with a friction-heavy pattern is weeks. |
| Any hard gate fails | n/a | **Pattern is not yet ready.** Diagnose which gate failed and why. Fix the underlying issue, then re-run the failed stage (or run a recovery session) before proceeding. |

The temptation will be to declare victory because the milestone shipped. Resist it. The point of this evaluation is to catch friction before it compounds.

## What user sees

Per `CLAUDE.md` §8 + §19, the user reviews **two artifacts** at PR time:

1. **The PR description and code diff** — does the milestone deliver what was promised, at the quality expected?
2. **The filled-in retrospective** at `docs/build-prompts/retrospectives/M[NN].[N]-retrospective.md` — does Claude's self-assessment match what the user observed (especially the hard gates)?

If Claude's retrospective claims G1 (do-not-commit) passed but the user saw an unauthorized commit in the git log, that's a flag. The user pushes back on the scoring. Claude revises or escalates.

User is **not** asked to fill in retrospective fields, write live observations during the session, or score axes. Those are Claude's responsibilities. User reviews and approves what Claude self-reported.

## Outcome of a retrospective

Possible outcomes after Claude's self-assessment + user review:

1. **All gates pass, scoring confirmed** — proceed to next stage in a fresh session.
2. **Soft gates fail** — spend a brief session updating `CLAUDE.md` / `TEMPLATE.md` per the retrospective's Decisions section, THEN proceed.
3. **Hard gate fails** — stop. Diagnose. Fix the underlying issue (which may require a new ADR if it's a primitive protocol change). Re-run a recovery session if needed.
4. **User pushes back on Claude's scoring** — Claude reconsiders, may surface additional events the user observed but Claude didn't log. Updated retrospective re-reviewed.

The retrospective is part of the project's quality history. After M11, the chain of M[NN].[N] retrospectives + M[NN] summaries + the optional TRENDS.md is the artifact someone can read to understand how the project actually got built — friction included.

## See also

- `docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md` — per-session shape (the form Claude fills in)
- `docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md` — per-parent-milestone roll-up
- `CLAUDE.md` §19 Retrospective Protocol — the procedural enforcement (what Claude does when, what user does when)
- `CLAUDE.md` §8 PR + commit workflow — the do-not-commit rule that the most important hard gate (G1) verifies
