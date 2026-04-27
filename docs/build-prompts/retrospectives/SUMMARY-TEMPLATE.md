<!--
Per-parent-milestone summary template. Claude creates this at the
end of the LAST sub-milestone of a parent milestone (e.g., after
M01.4 ships, Claude creates M01-summary.md aggregating findings
across M01.1, M01.2, M01.3, M01.4).

The user reviews the summary as part of the final sub-milestone's
PR. Acts as the gate for proceeding to the next parent milestone.

If a parent milestone is not split into sub-milestones, this file
isn't needed — the single retrospective IS the summary.
-->

# M[NN] — Summary Retrospective

> **Parent milestone:** M[NN] of M11 in `docs/MVP-v0.1.md`
> **Authored by:** Claude (per `CLAUDE.md` §19)
> **Aggregates:** M[NN].1, M[NN].2, ..., M[NN].N retrospectives
> **Created at:** YYYY-MM-DD HH:MM TZ
> **Total elapsed:** [sum of session times across sub-milestones]
> **Estimated:** [original parent-milestone estimate from MVP-v0.1.md]

---

## Sub-milestone trail

| Sub-milestone | Status | PR | Retrospective | Outcome |
|---|---|---|---|---|
| M[NN].1 | Merged | #N | `M[NN].1-retrospective.md` | [Sound / Sound-but-rough / Friction-heavy / Not-ready] |
| M[NN].2 | Merged | #N | `M[NN].2-retrospective.md` | [...] |
| M[NN].3 | Merged | #N | `M[NN].3-retrospective.md` | [...] |
| M[NN].4 | Merged | #N | `M[NN].4-retrospective.md` | [...] |

---

## Aggregate scoring (sum across sub-milestones)

### Process axis

| Sub-milestone | Total | /40 |
|---|---|---|
| M[NN].1 | | /40 |
| M[NN].2 | | /40 |
| M[NN].3 | | /40 |
| M[NN].4 | | /40 |
| **Mean** | | /40 |

### Product axis

| Sub-milestone | Total | /40 |
|---|---|---|
| M[NN].1 | | /40 |
| M[NN].2 | | /40 |
| M[NN].3 | | /40 |
| M[NN].4 | | /40 |
| **Mean** | | /40 |

### Pattern axis

| Sub-milestone | Total | /35 |
|---|---|---|
| M[NN].1 | | /35 |
| M[NN].2 | | /35 |
| M[NN].3 | | /35 |
| M[NN].4 | | /35 |
| **Mean** | | /35 |

---

## Cross-sub-milestone trends

### Friction patterns that recurred

- [List friction events that appeared in multiple sub-milestones — these point to gaps in `CLAUDE.md` or `TEMPLATE.md`, not just one prompt's wording]

### Pattern-level wins

- [Things the protocol got right consistently — keep doing these]

### Surprises across the parent milestone

- [Things that surprised across multiple sub-milestones — adjust `CLAUDE.md` §15 gotchas accordingly]

### Hard gate violations across the milestone

- [If any hard gate failed in any sub-milestone, list it here with how it was resolved — these inform whether the next parent milestone is safe to start]

---

## Time-box accuracy

| Sub-milestone | Estimated | Actual | Ratio |
|---|---|---|---|
| M[NN].1 | [X] h | [Y] h | [Y/X] |
| M[NN].2 | [X] h | [Y] h | [Y/X] |
| M[NN].3 | [X] h | [Y] h | [Y/X] |
| M[NN].4 | [X] h | [Y] h | [Y/X] |
| **Total** | [sum X] h | [sum Y] h | [sum ratio] |

If total ratio >2.0, the parent-milestone estimation method is off. Note correction for next parent milestone.

---

## Decisions to apply before the next parent milestone

Drives `CLAUDE.md` / `TEMPLATE.md` / per-milestone-prompt updates that landed (or should land) before M[NN+1].1's session opens.

### `CLAUDE.md` updates carrying forward

- [Specific change applied or pending]

### `TEMPLATE.md` updates carrying forward

- [Specific change applied or pending]

### M[NN+1] sub-milestone prompts — known constraints to encode

- [If trends surfaced something that the next parent milestone's prompts should explicitly address]

### Open issues filed

- [Issues opened during M[NN] that don't block but should be tracked]

---

## Verdict

Mark one:

- [ ] **Pattern held across M[NN].** Proceed to M[NN+1].1 with the protocol updates above applied. Confidence in the prompt-driven approach: high.
- [ ] **Pattern held but with friction.** Apply soft-gate fixes from sub-milestone retrospectives before M[NN+1].1. Confidence: medium.
- [ ] **Pattern strained.** A hard gate failed in one or more sub-milestones; or aggregate scores indicate sustained pattern-level friction. Spend a session iterating on `CLAUDE.md` / `TEMPLATE.md` BEFORE M[NN+1].1. Confidence: low until protocol is updated.

---

## User-review notes

> User reviews this summary as part of the final sub-milestone's PR. Approval here gates the next parent milestone.

User-review notes:

- [Empty until user reviews]

---

## Sign-off

**Claude:** This summary aggregates the per-sub-milestone retrospectives for M[NN]. It is my honest assessment of how the parent milestone went and what the protocol should carry forward. User review and approval pending. The next parent milestone (M[NN+1]) does not begin until this summary is approved.

**Surfaced at:** [timestamp]
