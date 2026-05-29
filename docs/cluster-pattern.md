# Cluster-gate process — the canonical work cycle (CLAUDE.md §4 rule 11's enforcement)

> **What this is.** The single canonical description of how every unit of
> work closes, from M08.7 onward. It does **not** replace the major-phase
> structure (staged build → Stage V → closeout → gap-analysis); it
> **folds a close-gate discipline into it** so the assembled thing is run
> before anything is called done. It is the operational enforcement of
> CLAUDE.md §4 rule 11 (grounded-claims / no-gaslighting).
>
> **Why it exists.** M08.6 was a full major milestone — A–F stages, Stage
> V, closeout — authored with the existing discipline, and it shipped
> **7 🔴** because nothing in the machinery ever ran the assembled app.
> Writing discipline alone cannot catch painted-not-running behavior
> (built-in tools that emit `ToolInvoked` but never execute; `save_framework`
> that drops companion files). **Only running it catches those.** This doc
> makes "run it" the close condition, everywhere.

---

## 1. The unit of work: a cluster

A **cluster** is the unit of work — a small, related group of behaviors
(3–7 findings, or one engine function, or one feature surface) under
**one user-observable acceptance contract**. A major-milestone *stage*
is a cluster. An M08.7 ladder *rung* is a cluster. An X.5 *fix* is a
cluster. Same shape, one machine — there is no separate "fix-only" track.

Every cluster closes through the same five steps, in order:

```
1. ACCEPTANCE FIRST   Write the "what working looks like" test before any code,
                      in user-observable terms (BDD: Given / When / Then).
                      Dual-role: it is BOTH the maintainer's IRL script
                      AND the spec the machine turns into an assembled test.

2. BUILD              Strict v1.8 two-commit TDD: failing tests committed
                      standalone (red) → surface for red approval →
                      implement without touching the test files (impl) →
                      `git diff <red>..<impl>` over test paths is EMPTY.

3. MACHINE GATES      Lint / coverage / unit + the MUTATION GATE (prove the
                      tests actually bite — a surviving mutant means the
                      tests pass without verifying). CI-parity (CLAUDE.md §6).

4. RUN THE ASSEMBLED THING ← THE CLOSE GATE
                      The assembled path runs and is observed. For any
                      user-observable surface, the maintainer IRL-watches it.
                      "Tests green" is NOT close. A cluster is done only when
                      the real behavior has been run and seen (rule 11).

5. TRIAGE IN PLACE    New findings from the run are dispositioned right here:
                      fix-in-cluster / open a new cluster / explicit
                      ADR-class scope-out. NEVER "→ next milestone." (§2 below)
```

A cluster whose acceptance contract has not been **run and observed** is
not closed — it is, in rule-11 terms, a **hypothesis**, and is labeled
one on its surface.

---

## 2. The zero-propagation rule (hard)

**No finding moves forward except by explicit ADR-class scope-out.**
"Route to the next milestone's Stage A" / "M9 intake will absorb it" is
**banned** — it is the mechanism by which 32 findings were about to land
on M9. A finding has exactly three legal dispositions:

1. **Fixed in this cluster** — observed resolved in the running system.
2. **A new cluster** — opened now, scheduled in this phase.
3. **Explicit scope-out** — an ADR (or a one-line waiver-ADR) stating it
   is v1.0+ and *why*, with the observable v0.1 line documented.

"Resolved" requires observed resolution (rule 11) — a code change that
*should* fix it is "fix written, unverified," not resolved.

---

## 3. How clusters compose into a major milestone

The major-phase structure is **kept** — clusters are how its stages
close:

```
MAJOR MILESTONE (e.g. M08.7)
  = a sequence of clusters (its stages / rungs)
      each closes per §1 (acceptance-first → build → gates → RUN → triage)
  → STAGE V (fresh-context verifier)
      now with a 5th pass: ASSEMBLED-EXECUTION — V RUNS the assembled app /
      assembled integration tests and OBSERVES each cluster execute. A
      "Sound" that did not run the assembled path is forbidden (rule 11;
      the M08.6.V escape this closes).
  → CLOSEOUT
      summary + immutable gap-analysis (dispositions are OBSERVED, not
      inferred) + simplify_pass + coverage reconciliation + PR draft.
  → MILESTONE IRL CONFIRM
      the maintainer runs the assembled milestone end-to-end (the gate that
      would have caught M08.6's 7 🔴).
```

A **fix cycle** (X.5) is the same machine, scoped to findings — a
milestone made of fix-clusters. No special-case machinery.

---

## 4. Acceptance authoring (eval-first, dual-role)

Each cluster's acceptance is written **before** the build and serves two
masters at once:

- **The maintainer's IRL script** — small and tight (a cluster is small,
  so its IRL is minutes, not a 3-hour walk). The maintainer both
  **confirms** the contract and **discovers** the unknowns no script
  predicts (UX, contrast, "this feels wrong" — the design-quality lens).
- **The machine's assembled test** — the same Given/When/Then, authored
  as a `tests/e2e-tauri/` (renderer/shell) or assembled Rust integration
  test that drives the **real** path. It runs in CI first; the IRL pass
  *confirms* an already-green test, it doesn't *rediscover*. The
  discovery happens once; the finding becomes a permanent encoded test
  so it can never silently regress.

**Assertion rule (the trap this closes):** acceptance assertions are on
**user-observable behavior / observable side effects**, never on internal
events alone. "A `ToolInvoked` event is emitted" licenses "the event is
emitted" — NOT "the tool ran." "The file exists on disk with the written
content" is the grounded assertion. Injecting a `ProviderEvent::ToolUse`
and asserting the emitted event is the exact pattern that hid the
built-in-tools-don't-execute gap.

---

## 5. The mutation gate

A test suite that passes is not evidence the tests *bite*. The mutation
gate breaks the production code and confirms a test fails.

- **Rust:** `cargo-mutants` (already wired nightly, CLAUDE.md §5) — run
  on the cluster diff; a surviving mutant on the cluster's logic blocks
  close (or is justified inline).
- **TypeScript/renderer:** **Stryker** (`@stryker-mutator/core` + the
  vitest runner). **Phase-0a setup task (build-side — install + verify is
  the build machine's; authoring the config blind risks gotcha #32, so it
  lands WITH a verify-against-upstream check):** add the dev-deps; a
  `stryker.conf.json` targeting `src/` with the vitest test runner + a
  `mutate` glob scoped to the cluster's touched files; an `npm run mutate`
  script; a CI job (or a documented manual run) gated like the other
  frontend jobs. Verify with one smoke run on a small module — confirm a
  deliberately-broken mutant is caught. Until it lands, renderer clusters
  lean on the BDD assembled test + the IRL gate, and the cluster surface
  states "TS mutation gate: Stryker not yet installed — BDD+IRL only"
  (the gap is recorded, not silently ignored — rule 11). Once installed,
  the mutation gate is REQUIRED for a renderer cluster's close.

---

## 6. What changed vs the pre-M08.7 process

| | Before (shipped 7🔴) | Now (cluster-gate) |
|---|---|---|
| A stage/cluster closes on | tests green | **the assembled thing runs + is observed** (rule 11) |
| Stage V | reads tests | **runs the app** (5th assembled-execution pass) |
| IRL | once, late, post-merge — 32 findings at once | **per-cluster** — small, frequent, diagnosable |
| Findings | "→ next milestone Stage A" | **triaged in-cluster; zero propagation** (§2) |
| Acceptance | sometimes implementation-detail | **user-observable, authored first, dual-role** |
| Test quality | coverage % | **+ mutation gate (tests must bite)** |
| Claims | "Sound / FINAL DISPOSITION" | **grounded — observed evidence at the claim's granularity** (rule 11) |

The structure is unchanged. The close gate is the change — applied to
**every** unit of work, major or fix, not a fix-only side-process and not
writing discipline alone.

---

## 7. Authoring checklist (per cluster)

- [ ] Acceptance written first, user-observable, Given/When/Then, dual-role (IRL script + assembled test).
- [ ] Assembled test drives the real path; assertions on observable behavior/side-effects, not events alone.
- [ ] Strict two-commit TDD; impl↔red test diff EMPTY.
- [ ] Machine gates green at CI-parity + mutation gate (Rust now; TS once Stryker lands).
- [ ] **Assembled thing run and observed** (IRL for user-observable surfaces) — the close gate.
- [ ] New findings triaged in-place (fix-now / new cluster / ADR scope-out). Zero propagation.
- [ ] Surface states what was run AND what was not (rule 11). No "done" without observed evidence at its granularity.

---

## Status of this doc's enforcement (rule 11 — honest)

- **Codified here:** the cluster cycle, the zero-propagation rule, the
  acceptance/mutation/close-gate disciplines, the Stage V 5th pass.
- **Wired into the decision index:** ORCHESTRATOR.md §3 (zero-propagation
  replaces the "carry forward" rows).
- **Not yet done (Phase 0a remainder):** Stryker install (build-side);
  the STAGE-PROMPT-PROTOCOL.md fold-in (adding the close-gate +
  acceptance-first slots to the existing `<work_stage_prompt>` rather
  than a parallel schema, per "keep the major phases"); the Stage V
  template's 5th-pass codification. These are authoring tasks, not done
  by this doc's existence.
