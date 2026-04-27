# Process Validation: Did the Pattern Work?

> **Purpose:** This doc is the **test for the build pattern itself**, not the test for what each milestone ships. M01's acceptance criteria (in `M01-foundation.md`) verify M01 shipped. This doc verifies that the **prompt-driven, fresh-session, TDD-with-claude approach is a good pattern** — sufficient to confidently apply across M02–M11.
>
> If this doc says "the pattern works," generating M02–M11 from `TEMPLATE.md` is safe. If this doc surfaces enough friction to fail any of the threshold criteria below, **stop and revise** `CLAUDE.md` / `TEMPLATE.md` / the milestone-prompt format **before** generating M02–M11.

## Why this exists

There are three things that could be true after M01 ships:

1. **M01 shipped AND the pattern is sound.** Generate M02–M11 from `TEMPLATE.md` with confidence. Update `CLAUDE.md` only for minor learnings.
2. **M01 shipped, but the pattern was painful.** The deliverable is fine; the *experience* was friction-heavy. Surface what was painful, fix it before M02. Pattern revisions take 1–2 hours; running 11 milestones with a bad pattern costs weeks.
3. **M01 didn't ship cleanly, AND the pattern is unclear.** This is the worst case — both the artifact and the approach are suspect. Resist the temptation to declare victory. Diagnose what failed (CLAUDE.md? prompt? Claude's execution? something deeper?) before any further generation.

You can't tell which case you're in without a deliberate evaluation. That's this doc.

## How to use this doc

There are **three phases** to the evaluation:

| Phase | When | Action |
|---|---|---|
| **Pre-flight** | Before opening the M01 fresh session | Run the [Pre-flight checklist](#pre-flight-checklist) — verify environment, prompt is current, branch is right |
| **Live observation** | During M01 execution | Fill in the [Live observation log](#live-observation-log-fill-during-m01) as friction surfaces — don't wait until the end; details fade |
| **Post-milestone retrospective** | After M01 PR merges to main | Run the three-axis evaluation, score against thresholds, decide what to update |

The doc is structured so you can move through it linearly during the M01 run.

---

## The three axes of evaluation

The build pattern has three concerns. Each has its own questions.

### Axis 1: Process — did the prompt-driven workflow work?

Asks: *Did Claude have what it needed to execute autonomously? Did the workflow surface decisions to the user at the right moments?*

This isn't about whether the code is good (that's Axis 2). It's about whether the *interaction* worked:

- Was `CLAUDE.md` sufficient orientation, or did Claude ignore parts of it / get confused by parts of it?
- Did the M01 prompt's "Read first" list correctly orient Claude before any code was written?
- Did Claude state the deliverable + test plan before writing code (per CLAUDE.md §16 session-start checklist)?
- Did Claude self-correct effectively when gates failed, or did it spiral?
- Did Claude actually NOT commit before approval (the most important rule)?
- Did Claude escalate the right things and proceed on the right things (per CLAUDE.md §12)?

### Axis 2: Product — did the artifact meet our standards?

Asks: *Is what shipped actually good?*

This is about the M01 deliverables:

- Does the code match CLAUDE.md §9 style + naming?
- Are the tests behavior tests, not tautology tests (CLAUDE.md §5)?
- Are public APIs documented with examples (CLAUDE.md §6)?
- Are the deliverables what M01 promised, no more, no less?
- Would a stranger picking up this code understand it without reading the spec?
- Are anti-patterns from CLAUDE.md §9 absent?

### Axis 3: Pattern — does this generalize to M02–M11?

Asks: *Is the prompt structure sound enough to copy 10 more times?*

This is the meta-question. M01 is the proof-of-concept; if its prompt format is wrong, repeating it 10 times multiplies the wrongness.

- Were sections of `TEMPLATE.md` dead weight in the M01 prompt? (Sections that contributed nothing should be removed before M02.)
- Were sections missing? (Sections that should have been in the prompt but weren't should be added to TEMPLATE.md.)
- Are M01-specific gotchas useful for M02+, or were they truly milestone-local? (Generalizable ones move to CLAUDE.md §15.)
- Did the time-box estimate match reality? (If 2× off, the estimation method needs revision.)
- Were there moments of *implicit* protocol — things you knew but didn't write down — that need to become explicit?

---

## Pre-flight checklist

**Run this before opening the M01 fresh Claude Code session.**

Environment:
- [ ] You're on a clean working tree (`git status` shows nothing pending)
- [ ] You're on `main` and `main` is up to date with origin
- [ ] PRs #22 and #23 (or whatever PRs land CLAUDE.md + build-prompts/ + the code-signing pivot) are merged
- [ ] You've reviewed `agent-runtime-spec.md` §0d to confirm M01 scope matches the release scope matrix
- [ ] You've created the new branch `claude/m01-foundation` off the freshly-merged `main`

Prompt:
- [ ] You have `docs/build-prompts/M01-foundation.md` open in another window or tab
- [ ] You have `CLAUDE.md` (root) open or readily accessible
- [ ] You have `agent-runtime-spec.md` §1 (Phase 1 Drone), §1c (Multi-session), §1d (IPC) open or accessible
- [ ] You're going to paste the **entire contents of M01-foundation.md** as the opening message — not a summary

Tooling availability for M01 (verify present on your dev machine):
- [ ] `cargo` (rustc/cargo via rustup)
- [ ] `git` 2.x
- [ ] (later — for fuzzing) `cargo +nightly fuzz` available

Mindset:
- [ ] You commit to **not** intervening in Claude's TDD loop unless asked. The prompt is designed for autonomy; resist the urge to course-correct mid-flight.
- [ ] You commit to filling in this doc's [Live observation log](#live-observation-log-fill-during-m01) as friction surfaces, not at the end.
- [ ] You commit to **reading the PR description and gate results carefully** before approving the commit, per CLAUDE.md §8 do-not-commit-until-approved rule.
- [ ] If something feels wrong during M01, write it in the live log immediately. Don't wait until the end.

---

## Live observation log (fill during M01)

**Fill this in as friction surfaces during the M01 run.** Don't summarize at the end — the details that matter most are the ones you'd forget.

### Friction events

Things that slowed Claude down or that Claude had to ask about.

| When (commit / step) | What happened | What the prompt should have said | Severity (1–5) |
|---|---|---|---|
| (e.g., "before first commit") | (e.g., "Claude asked which Rust version to pin to") | (e.g., "TEMPLATE.md section 'Read first' should require choosing latest stable; M01 prompt should specify '1.80 or current latest, whichever is newer'") | (e.g., 2) |
| | | | |
| | | | |

Severity scale:
- **1** — Trivial: Claude paused for ≤30 seconds. Cosmetic.
- **2** — Minor: Claude asked one clarifying question that the prompt could have answered.
- **3** — Notable: Claude asked multiple questions, or had to revise scope mid-flight, or self-corrected through 2+ rounds where 1 should have sufficed.
- **4** — Significant: A protocol gap caused real rework or wrong direction. Claude went off-track until corrected.
- **5** — Blocking: Claude couldn't proceed without human intervention beyond approval. The prompt was insufficient.

### Ambiguity events

Places where the spec, MVP doc, ADRs, or M01 prompt gave conflicting or unclear guidance.

| Source files in conflict | What was unclear | How resolved | Should be fixed where? |
|---|---|---|---|
| (e.g., "spec §1d vs CLAUDE.md §15 gotcha 12") | (e.g., "Whether main↔drone uses Tauri IPC or Unix socket") | (e.g., "User confirmed Unix socket per spec §1d") | (e.g., "Add cross-reference in CLAUDE.md gotcha 12 → spec §1d") |
| | | | |

### Surface events

Moments when Claude surfaced a decision to the user (per CLAUDE.md §8 + §12).

| When | What was surfaced | Was the surface actionable? | Was the timing right? |
|---|---|---|---|
| (e.g., "before committing") | (e.g., "PR description draft + diff stat + gate results") | (e.g., "Yes — could approve in 2 min") | (e.g., "Yes — at end after gates green") |
| | | | |

Look for: surfaces that arrived too early (before Claude tried to self-correct), too late (after Claude went off-track), or with insufficient detail (forced the user to investigate before deciding).

### Protocol drift events

Moments when Claude almost broke a CLAUDE.md hard rule, OR when the protocol said one thing and Claude did another.

| Hard rule | What happened | Did Claude self-correct? | Notes |
|---|---|---|---|
| (e.g., "§4.1 don't commit without approval") | (e.g., "Claude staged + committed without surfacing PR") | (e.g., "No — user had to revert and remind") | (e.g., "Strengthen §8 wording or add to M01 prompt") |
| | | | |

If any drift event occurs, **stop and surface immediately**. Don't wait until the retrospective.

### Surprise events

Things that worked better than expected, or worse, in ways that suggest the protocol assumptions are off.

| What was surprising | Why it surprised you | What this implies for the protocol |
|---|---|---|
| | | |

---

## Post-milestone retrospective (fill after M01 PR merges)

**Run this only after M01's PR is approved and merged to main.** Filling it in earlier biases toward "we're so close, ship it"; let the merge happen first, then evaluate honestly.

### Axis 1: Process — score 1–5

| Question | Score | Notes |
|---|---|---|
| Was `CLAUDE.md` sufficient orientation for a fresh session? | | |
| Did the M01 prompt's "Read first" list correctly orient Claude? | | |
| Did Claude state deliverable + test plan before code per §16? | | |
| Did Claude self-correct in ≤3 rounds per §7? | | |
| Did the do-not-commit-until-approved rule hold? | | |
| Did Claude escalate the right things, proceed on the right things? | | |
| Was the human-direction load reasonable? | | |
| Were the surfaces (PR drafts, escalations) actionable? | | |

Score scale (apply per row):
- **5** — Worked exactly as the protocol intended. No friction.
- **4** — Worked, with one minor friction event noted.
- **3** — Worked, with multiple friction events; pattern needs revision before M02.
- **2** — Partially worked; significant gaps in the protocol.
- **1** — Failed; the protocol does not support this workflow.

**Process score:** sum of row scores. Out of 40.

### Axis 2: Product — score 1–5

| Question | Score | Notes |
|---|---|---|
| Does the M01 code match CLAUDE.md §9 style + naming? | | |
| Are tests behavior tests, not tautology tests (§5)? | | |
| Are public APIs documented with compile-checked examples (§6)? | | |
| Are M01 deliverables exactly what was promised — no scope creep, no scope cuts? | | |
| Would a stranger picking up this code understand it without reading the spec? | | |
| Are CLAUDE.md §9 anti-patterns absent? | | |
| Is coverage actually ≥80% / 100% on safety primitives — measured, not estimated? | | |
| Did CI green hold across Linux/macOS/Windows × stable/MSRV? | | |

**Product score:** sum of row scores. Out of 40.

### Axis 3: Pattern — score 1–5

| Question | Score | Notes |
|---|---|---|
| Were all `TEMPLATE.md` sections useful for M01, or were any dead weight? | | |
| Were any sections needed but missing from `TEMPLATE.md`? | | |
| Are M01-specific gotchas truly milestone-local, or do they generalize to CLAUDE.md §15? | | |
| Did the time-box estimate match reality (within 2×)? | | |
| Were there *implicit* protocol moments — things the user knew but didn't write down? | | |
| Could M01-foundation.md be re-run by a different fresh session and produce comparable results? | | |
| If we copy this prompt format 10 more times for M02–M11, does anything need to change first? | | |

**Pattern score:** sum of row scores. Out of 35.

---

## Threshold criteria — is the pattern good enough to scale?

Apply these gates after scoring. **All must pass to proceed to generating M02–M11.** If any gate fails, the pattern needs revision.

### Hard gates (any fail = stop)

- [ ] **G1: do-not-commit-until-approved rule held.** Claude did not commit without explicit user approval, ever, during M01. Even once would mean the protocol's most important rule isn't reliable, and the pattern fails.
- [ ] **G2: no Severity-5 friction events.** No moment where Claude couldn't proceed and the prompt was insufficient. Severity-5 means the protocol is materially incomplete.
- [ ] **G3: no protocol drift events left unaddressed.** Every entry in the protocol-drift table either (a) Claude self-corrected, OR (b) the user has documented the prompt/CLAUDE.md fix.
- [ ] **G4: M01 actually shipped.** PR merged to main; CI green; M01 acceptance criteria all checked. If M01 didn't ship, the pattern question is moot.
- [ ] **G5: scores ≥3 in every individual row across all three axes.** Average isn't enough — a 1 or 2 in a single row points to a specific gap that compounds at scale.

### Soft gates (advisory; weigh together)

- **S1:** Process axis total ≥30 / 40
- **S2:** Product axis total ≥32 / 40
- **S3:** Pattern axis total ≥25 / 35
- **S4:** Time-box estimate within 2× of actual elapsed time
- **S5:** ≤3 Severity-3 friction events (multiple Severity-3s suggest sustained friction even if no individual event was blocking)

### Outcome

| Hard gates | Soft gates | Verdict |
|---|---|---|
| All pass | All pass | **Pattern is sound. Generate M02–M11 from TEMPLATE.md.** Apply minor revisions to CLAUDE.md based on Axis 3 notes. |
| All pass | 1–2 fail | **Pattern is sound but rough.** Revise CLAUDE.md and TEMPLATE.md to address the soft gates first; then generate. |
| All pass | 3+ fail | **Pattern works but has friction.** Stop. Spend a session iterating on CLAUDE.md / TEMPLATE.md before generating M02. The cost of fixing now is hours; the cost of running 10 milestones with a friction-heavy pattern is weeks. |
| Any hard gate fails | n/a | **Pattern is not yet ready.** Diagnose which gate failed and why. Fix the underlying issue, then re-run M01 (or M01-equivalent) before proceeding. |

The temptation will be to declare victory because M01 shipped. Resist it. The point of this evaluation is to catch friction before it compounds.

---

## Decisions for M02–M11 (fill after retrospective)

Based on the retrospective, list specific changes to apply before generating M02–M11.

### CLAUDE.md updates

- [ ] (e.g., "Add gotcha #21: typify edge case for `$ref` to external schemas")
- [ ] ...

### TEMPLATE.md updates

- [ ] (e.g., "Add new section 'Pre-flight tooling check' after 'Read first'")
- [ ] (e.g., "Remove 'Time-box (soft)' section — wasn't useful at M01")
- [ ] ...

### Protocol additions

- [ ] (e.g., "Add to PR workflow: state Rust version pinned in PR description")
- [ ] ...

### M02-specific carry-overs

- [ ] (e.g., "M02 must read M01-retrospective in addition to its own prompt")
- [ ] ...

After applying these updates, **regenerate `TEMPLATE.md`** and use it to author M02 through M11 in one batch. Then iterate this validation doc — keep this M01 section as historical record; add new sections per milestone retrospective.

---

## Future milestones — copy this shape

This doc is generic-shaped. After M01, copy the [Live observation log](#live-observation-log-fill-during-m01) and [Post-milestone retrospective](#post-milestone-retrospective-fill-after-m01-pr-merges) sections into per-milestone retrospectives:

```
docs/build-prompts/PROCESS-VALIDATION.md       # this file (M01 sections inline)
docs/build-prompts/retrospectives/
    M01-retrospective.md                       # archived after M02 starts
    M02-retrospective.md
    ...
    M11-retrospective.md
```

Or keep them all inline here as one growing document. Whichever you prefer; consistency over time matters more than format.

A useful pattern after each milestone retrospective:

- **Trend log** — surface-level patterns that emerge across milestones (e.g., "Claude consistently asks about X — needs to be in CLAUDE.md")
- **Anti-pattern log** — things that worked at M01 but stopped working at M05 — protocol that's brittle at scale
- **Time-tracking log** — actual vs estimate per milestone — calibrates future estimates

By M11, you have 11 retrospectives + a trend analysis showing how the pattern evolved. That's the artifact someone reading this project a year from now will use to learn how it was built.

---

*End of process validation doc.*
