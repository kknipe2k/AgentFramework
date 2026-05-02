<!--
Per-parent-milestone summary template. Claude creates this at the
end of the FINAL stage of a parent milestone (e.g., after M01 Stage D
is committed on the parent-milestone branch, Claude creates
M01-summary.md aggregating findings across M01.A, M01.B, M01.C, M01.D).

The user reviews the summary alongside the M[NN] PR description.
Summary verdict gates whether the M[NN] PR is ready to merge AND
whether the next parent milestone can start.

If a parent milestone is not staged (small enough per the TEMPLATE.md
scope-split rule), this file isn't needed — the single retrospective
IS the summary.
-->

# M[NN] — Parent-Milestone Summary

> **Parent milestone:** M[NN] of M11 in `docs/MVP-v0.1.md`
> **Authored by:** Claude (per `CLAUDE.md` §19)
> **Aggregates:** M[NN].A, M[NN].B, ..., M[NN].`<X>` stage retrospectives
> **Created at:** YYYY-MM-DD HH:MM TZ
> **Total elapsed:** [sum of stage session times]
> **Estimated:** [original parent-milestone estimate from MVP-v0.1.md]

---

## Stage trail

| Stage | Status | Stage commit | Retrospective | Outcome |
|---|---|---|---|---|
| Stage A | Committed | `<sha>` | `M[NN].A-retrospective.md` | [Sound / Sound-but-rough / Friction-heavy / Not-ready] |
| Stage B | Committed | `<sha>` | `M[NN].B-retrospective.md` | [...] |
| Stage C | Committed | `<sha>` | `M[NN].C-retrospective.md` | [...] |
| Stage D | Committed | `<sha>` | `M[NN].D-retrospective.md` | [...] |

All stages on parent-milestone feature branch `claude/m[nn]-<title>`. The M[NN] PR drafts after this summary lands and surfaces all stage commits + retrospectives + this summary together.

---

## Aggregate scoring (sum across stages)

### Process axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | | /40 |
| Stage B | | /40 |
| Stage C | | /40 |
| Stage D | | /40 |
| **Mean** | | /40 |

### Product axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | | /40 |
| Stage B | | /40 |
| Stage C | | /40 |
| Stage D | | /40 |
| **Mean** | | /40 |

### Pattern axis

| Stage | Total | /35 |
|---|---|---|
| Stage A | | /35 |
| Stage B | | /35 |
| Stage C | | /35 |
| Stage D | | /35 |
| **Mean** | | /35 |

---

## Cross-stage trends

### Friction patterns that recurred

- [List friction events that appeared in multiple stages — these point to gaps in `CLAUDE.md` or `TEMPLATE.md`, not just one prompt's wording]

### Pattern-level wins

- [Things the protocol got right consistently — keep doing these]

### Surprises across the parent milestone

- [Things that surprised across multiple stages — adjust `CLAUDE.md` §15 gotchas accordingly]

### Hard gate violations across the milestone

- [If any hard gate failed in any stage, list it here with how it was resolved — these inform whether the next parent milestone is safe to start]

---

## Time-box accuracy

| Stage | Estimated | Actual | Ratio |
|---|---|---|---|
| Stage A | [X] h | [Y] h | [Y/X] |
| Stage B | [X] h | [Y] h | [Y/X] |
| Stage C | [X] h | [Y] h | [Y/X] |
| Stage D | [X] h | [Y] h | [Y/X] |
| **Total** | [sum X] h | [sum Y] h | [sum ratio] |

If total ratio >2.0, the parent-milestone estimation method is off. Note correction for next parent milestone.

---

## Decisions to apply before the next parent milestone

Drives `CLAUDE.md` / `TEMPLATE.md` / per-milestone-prompt updates that landed (or should land) before M[NN+1].1's session opens.

### `CLAUDE.md` updates carrying forward

- [Specific change applied or pending]

### `TEMPLATE.md` updates carrying forward

- [Specific change applied or pending]

### M[NN+1] stage prompts — known constraints to encode

- [If trends surfaced something that the next parent milestone's prompts should explicitly address]

### Open issues filed

- [Issues opened during M[NN] that don't block but should be tracked]

---

## Verdict

Mark one:

- [ ] **Pattern held across M[NN].** Proceed to M[NN+1].1 with the protocol updates above applied. Confidence in the prompt-driven approach: high.
- [ ] **Pattern held but with friction.** Apply soft-gate fixes from stage retrospectives before M[NN+1].1. Confidence: medium.
- [ ] **Pattern strained.** A hard gate failed in one or more stages; or aggregate scores indicate sustained pattern-level friction. Spend a session iterating on `CLAUDE.md` / `TEMPLATE.md` BEFORE M[NN+1].1. Confidence: low until protocol is updated.

---

## User-review notes

> User reviews this summary as part of the final stage's PR. Approval here gates the next parent milestone.

User-review notes:

- [Empty until user reviews]

---

## Sign-off

**Claude:** This summary aggregates the per-stage retrospectives for M[NN]. It is my honest assessment of how the parent milestone went and what the protocol should carry forward. User review and approval pending. The next parent milestone (M[NN+1]) does not begin until this summary is approved.

**Surfaced at:** [timestamp]
