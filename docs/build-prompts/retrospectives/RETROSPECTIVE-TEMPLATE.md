<!--
Per-session retrospective template. Claude copies this to
M[NN].[N]-retrospective.md at session start, fills it in during
and after the session, and surfaces it alongside the PR per
CLAUDE.md §19.

The user reviews the filled-in retrospective alongside the code
diff. The user does NOT fill in retrospective fields.

Sections marked [LIVE] are filled in DURING the session as events
surface. Sections marked [END] are filled in at session end.
Do NOT save filling for the end — details fade.
-->

# M[NN].<X> — Stage Retrospective

> **Stage:** M[NN] Stage `<X>` (where `<X>` is `A`, `B`, etc.) — see `docs/build-prompts/M[NN]-<title>.md`
> **Parent milestone:** M[NN] of M11 in `docs/MVP-v0.1.md`
> **Authored by:** Claude (Claude-driven per `CLAUDE.md` §19)
> **Session start:** YYYY-MM-DD HH:MM TZ
> **Session end:** YYYY-MM-DD HH:MM TZ
> **Branch:** `claude/m[nn]-<short-title>` (parent-milestone feature branch; stages commit on the same branch)
> **Starting commit:** `<sha at session start>`
> **Ending commit:** `<sha of stage commit at user-approval time, OR "uncommitted — awaiting approval">`
> **Estimated effort (per prompt):** [X hours]
> **Actual elapsed:** [Y hours]

---

## Pre-flight (claimed before session start)

These were satisfied before code was written:

- [ ] CLAUDE.md was loaded (auto-loaded by Claude Code) and the §4 Hard Rules were stated at the top of the first response
- [ ] All "Read first" files in the milestone prompt were read in the order listed
- [ ] The deliverable was stated in 1–3 sentences and the test plan in 3–5 bullets BEFORE writing code
- [ ] User confirmed the deliverable + test plan before code began
- [ ] The branch was correct per the milestone prompt
- [ ] No uncommitted changes carried over from a prior session (`git status` was clean)

---

## [LIVE] Friction events

Things that slowed Claude down during the session. Filled in AS friction surfaces, not at the end.

| When (commit / step) | What happened | What the prompt should have said | Severity (1–5) |
|---|---|---|---|

Severity scale:
- **1** — Trivial: paused for ≤30 seconds. Cosmetic.
- **2** — Minor: one clarifying question that the prompt could have answered.
- **3** — Notable: multiple questions, mid-flight scope revision, or 2+ self-correction rounds where 1 should have sufficed.
- **4** — Significant: a protocol gap caused real rework or wrong direction.
- **5** — Blocking: couldn't proceed without human intervention beyond approval. The prompt was insufficient.

If no friction events occurred during this session, write **"None observed"** below the table header.

---

## [LIVE] Ambiguity events

Places where the spec, MVP doc, ADRs, or milestone prompt gave conflicting or unclear guidance.

| Source files in conflict | What was unclear | How resolved | Should be fixed where? |
|---|---|---|---|

If none, write **"None observed"** below the table header.

---

## [LIVE] Surface events

Moments when Claude surfaced a decision to the user (per `CLAUDE.md` §8 + §12).

| When | What was surfaced | Was the surface actionable? | Was the timing right? |
|---|---|---|---|

If none, write **"None observed"** below the table header.

Look for: surfaces that arrived too early (before self-correction was attempted), too late (after going off-track), or with insufficient detail.

---

## [LIVE] Protocol drift events

Moments when Claude almost broke a `CLAUDE.md` Hard Rule (§4), OR when the protocol said one thing and Claude did another.

| Hard rule | What happened | Did Claude self-correct? | Notes |
|---|---|---|---|

If none, write **"None observed"** below the table header.

If any drift event occurred, **the user is alerted immediately during the session**, not at the end.

---

## [LIVE] Surprise events

Things that worked better than expected, or worse, in ways that suggest the protocol assumptions are off.

| What was surprising | Why it surprised | What this implies for the protocol |
|---|---|---|

If none, write **"None observed"** below the table header.

---

## [LIVE] Stage 0 real-app discovery routing

For any stage touching the real app (a `tests/e2e-tauri/` surface), record how the Stage-0 real-app discovery walk was handled: **run** (walked the flow on the build machine; log adjacent findings here) / **deferred to Stage V** (the assembled regression + V's assembled-execution pass cover it — name the regression) / **replaced by an assembled test** (the mechanical regression IS the discovery). The cluster-gate close still requires the assembled run + IRL observation regardless (`docs/cluster-pattern.md` §1 step 4). Backend-only stage with no real-app surface → write **"n/a — no real-app surface."**

| Stage-0 disposition | Evidence / regression file | Adjacent findings (triaged in-place — zero propagation) |
|---|---|---|

---

## [END] Three-axis scoring

Filled in at session end. 1–5 per row per `PROCESS-VALIDATION.md` scoring rubric.

### Axis 1: Process — did the prompt-driven workflow work?

| # | Question | Score | Evidence / notes |
|---|---|---|---|
| 1 | Was `CLAUDE.md` sufficient orientation for a fresh session? | | |
| 2 | Did the milestone prompt's "Read first" list correctly orient Claude? | | |
| 3 | Did Claude state deliverable + test plan before code per §16? | | |
| 4 | Did Claude self-correct in ≤3 rounds per §7? | | |
| 5 | Did the do-not-commit-until-approved rule hold? | | |
| 6 | Did Claude escalate the right things, proceed on the right things? | | |
| 7 | Was the human-direction load reasonable? | | |
| 8 | Were the surfaces (PR drafts, escalations) actionable? | | |
| **Process axis total** | | **/40** | |

### Axis 2: Product — did the artifact meet our standards?

| # | Question | Score | Evidence / notes |
|---|---|---|---|
| 1 | Does the code match `CLAUDE.md` §9 style + naming? | | |
| 2 | Are tests behavior tests, not tautology tests (§5)? | | |
| 3 | Are public APIs documented with compile-checked examples (§6)? | | |
| 4 | Are deliverables exactly what was promised — no scope creep, no scope cuts? | | |
| 5 | Would a stranger picking up this code understand it without reading the spec? | | |
| 6 | Are `CLAUDE.md` §9 anti-patterns absent? | | |
| 7 | Is coverage ≥80% / 100% on safety primitives — measured, not estimated? | | |
| 8 | Did CI green hold across Linux/macOS/Windows × stable/MSRV? | | |
| **Product axis total** | | **/40** | |

### Axis 3: Pattern — does this generalize to remaining milestones?

| # | Question | Score | Evidence / notes |
|---|---|---|---|
| 1 | Were all `TEMPLATE.md` sections useful, or were any dead weight? | | |
| 2 | Were any sections needed but missing from `TEMPLATE.md`? | | |
| 3 | Are milestone-specific gotchas truly local, or do they generalize to `CLAUDE.md` §15? | | |
| 4 | Did the time-box estimate match reality (within 2×)? | | |
| 5 | Were there *implicit* protocol moments — things that should have been written down but weren't? | | |
| 6 | Could this prompt be re-run by a different fresh session and produce comparable results? | | |
| 7 | If we copy this prompt format for remaining milestones, does anything need to change first? | | |
| **Pattern axis total** | | **/35** | |

---

## [END] Threshold evaluation

Per `PROCESS-VALIDATION.md` threshold criteria.

### Hard gates (any fail = stop and revise)

- [ ] **G1: do-not-commit-until-approved rule held** — *evidence:* [git log shows no commits before user approval message at HH:MM]
- [ ] **G2: no Severity-5 friction events** — *evidence:* [none in friction-events table above, OR specific entry & resolution]
- [ ] **G3: no protocol drift events left unaddressed** — *evidence:* [protocol-drift-events table is empty OR every row has Claude-self-corrected = yes]
- [ ] **G4: the milestone or stage actually completed** — *evidence:* [for stages: commit `<sha>` on the parent-milestone branch with all stage acceptance criteria checked; for non-staged milestones: PR #N opened, CI green, all acceptance criteria checked]
- [ ] **G5: scores ≥3 in every individual row across all three axes** — *evidence:* [check each axis row above for any score <3; list any below-3 rows here for follow-up]
- [ ] **G6: CI-parity confirmation** (per `CLAUDE.md` §6 CI-parity hard rule) — *evidence:* [paste the exact command(s) run locally for each gate, then confirm they match the corresponding job step in `.github/workflows/ci.yml`. If any flag differs (`--skip <test>`, `--test-threads=N`, `--features`, `--target`, env-var override), list every divergence and the reason it was necessary; if the divergence is a recurring pattern across stages, add it (or update the existing entry) in `docs/gotchas.md` so future stages encode the rationale up-front, and add a follow-up to the `[END] Decisions` section for the CI workflow change that closes the divergence structurally. A surface that asserts "all gates green locally" without this confirmation is structurally untrusted.]

### Soft gates

- [ ] **S1:** Process axis total ≥30 / 40 — *actual:* [N]
- [ ] **S2:** Product axis total ≥32 / 40 — *actual:* [N]
- [ ] **S3:** Pattern axis total ≥25 / 35 — *actual:* [N]
- [ ] **S4:** Time-box within 2× of actual — *actual:* [estimated X hours, took Y hours, ratio Y/X]
- [ ] **S5:** ≤3 Severity-3 friction events — *actual count:* [N]

### Outcome

Mark one:

- [ ] **Pattern is sound.** All hard gates pass; all soft gates pass. Proceed to next stage with minor `CLAUDE.md` updates per Axis 3 notes.
- [ ] **Pattern is sound but rough.** All hard gates pass; 1–2 soft gates fail. Address soft-gate findings in `CLAUDE.md` / `TEMPLATE.md` updates before next stage.
- [ ] **Pattern works but has friction.** All hard gates pass; 3+ soft gates fail. Stop. Spend a session iterating on protocol before next stage.
- [ ] **Pattern is not yet ready.** A hard gate failed. Diagnose underlying issue; may require new ADR. Recovery session may be needed.

---

## [END] Coverage holdouts

Per-stage record of coverage-gate exclusions and per-module baselines. Single source of truth for what's excluded from `cargo llvm-cov --fail-under-lines` gates and why — replaces the historical pattern where exclusion rationale was scattered across `CLAUDE.md` §5 + per-stage `[END] Decisions`. Stage F gap-analysis aggregates across the milestone; this subsection is the per-stage source data.

### Workspace gate (≥80% line)

- **Coverage actual this stage:** [N.NN%]
- **Exclusions added this stage:** [path/to/file — one-line rationale | "None added this stage"]
- **Exclusion list as of stage end:** [enumerate every regex term currently in `.github/workflows/ci.yml` workspace `--ignore-filename-regex`]

### Per-package gates (≥95% line on safety primitives)

For each safety-primitive crate gated this stage (`runtime-drone`, `runtime-main`, future: `runtime-sandbox`, capability enforcer, plan state machine, snapshot/recovery), record:

- **Crate:** [name]
- **Coverage actual this stage:** [N.NN%]
- **Exclusions added this stage:** [path/to/file — one-line rationale | "None added this stage"]
- **Exclusion list as of stage end:** [enumerate every regex term currently in `.github/workflows/ci.yml` for this package's `--ignore-filename-regex`]

### Per-module baselines (preserved-or-improved invariant)

Per `CLAUDE.md` §5, "Subsequent milestones must not regress any module below its baseline without a retro entry recording the reason." Record any module whose coverage moved this stage:

| Module | Prior baseline | This stage | Direction | Reason if down |
|---|---|---|---|---|
| `path/to/module.rs` | NN.NN% (M0X.Y) | NN.NN% | ↑↓→ | [if ↓, one-line rationale] |

### CI workflow drift check

- [ ] The `--ignore-filename-regex` patterns in `.github/workflows/ci.yml` (gate steps + lcov export steps) match what's documented above. Confirm by inspecting the workflow file at end of stage; mismatches between this retro and the workflow are a Stage E-class drift bug — fix in this stage's commit, not "the next stage's housekeeping."

---

## [END] Decisions for the next stage (or next parent milestone if this is the final stage)

Specific changes to apply BEFORE the next stage session opens. Empty list = no changes recommended.

### `CLAUDE.md` updates

- [ ] [e.g., "Add gotcha #21: <specific trap discovered this session>"]

### `TEMPLATE.md` updates

- [ ] [e.g., "Add new section X" / "Remove section Y because it was dead weight"]

### Milestone document updates (this milestone's `M[NN]-<title>.md` if a stage prompt was off)

- [ ] [e.g., "Stage <X+1> Read first list should add reference to <file>"]

### Protocol additions

- [ ] [e.g., "Add to PR workflow: state Rust version pinned in PR description"]

### Open issues to file

- [ ] [e.g., "File issue: <thing that needs follow-up but doesn't block next stage>"]

---

## [END] User-review notes

> The user reviews this retrospective alongside the PR. If the user sees a discrepancy between Claude's self-assessment and observable evidence, they push back here and Claude revises.

User-review notes:

- [Empty until user reviews. User adds notes here on review; Claude revises retrospective if needed.]

---

## [END] Sign-off

> This retrospective is Claude's self-assessment of M[NN] Stage `<X>`, authored per `CLAUDE.md` §19 Retrospective Protocol. User review and approval pending. Per the per-milestone-as-PR pattern, this stage's commit lands on the parent-milestone feature branch; the PR drafts and pushes only at the end of the final stage.

**Claude:** I have completed M[NN] Stage `<X>` per the stage prompt. The retrospective above reflects my honest self-assessment. I have NOT committed; I am surfacing this retrospective + the diff stat + the draft commit message for user approval. The PR remains undrafted until the final stage.

**Surfaced at:** [timestamp]
