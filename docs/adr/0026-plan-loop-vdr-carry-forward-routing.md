# ADR-0026: plan_loop / TestOutcome.vdr carry-forward routing

**Status:** Accepted
**Date:** 2026-05-31
**Deciders:** @kknipe2k
**Tags:** scope, process

## Context

Two M08.V verifier findings were routed forward without a durable record,
and two project docs subsequently disagreed about where they land:

- **🟡 #2 — `plan_loop` / `drive_plan` shipped but has no production
  caller.** Delivered + tested at M08.A (7 tests, 100% line coverage) but
  `drive_plan` "runs no tasks" and has no production call site
  (`plan_loop.rs:79`,`:128-129`).
- **🟡 #1 — `TestOutcome.vdr` is structurally dead** (`fold_outcome`
  hardcodes `vdr: Value::Null`; `tester.rs:153`), so MVP §M8 criterion 5's
  VDR surface always renders `null`.

M08.V Decisions 1 + 2 routed **both** to **M9 Stage A intake**, and
Decision 2 explicitly noted the `plan_loop` deferral "files the ADR-class
carry-forward record the deferral lacks" — which was never filed.
ORCHESTRATOR.md §9 later **re-routed both to M08.7** under the
zero-propagation rule (`docs/cluster-pattern.md` §2 — "route to the next
milestone's Stage A" is banned), recording "`plan_loop` rung 7,
`TestOutcome.vdr` rung 6 → M08.7." Two problems with that note:

1. It **contradicts** the still-standing M08.V Decisions (→ M9.A).
2. Its "`vdr` rung 6" label is a **mislabel** — the M08.7 phase-doc rung
   table's **rung 6 is "sequential spawn ⭐"** and the ladder contains no
   `vdr` rung at all (`drive_plan` IS at rung 7).

`plan_loop` and `vdr` are different in kind: `plan_loop` is one of the
six paints-not-executes **execution primitives** the M08.7 ladder exists
to make run; `vdr` is a **Tester output projection** (fold the run's
`decision_record` events in `fold_outcome`), not an engine primitive.
This ADR is the one-time routing reconciliation directed at M08.7.X.

## Decision

We **split** the routing:

- **`plan_loop` production wiring → M08.7 rung 7** (zero-propagation
  re-route, superseding M08.V Decision 2's → M9.A). Plans are an
  execution primitive; M08.7 is the execution milestone; routing the
  wiring there keeps the engine's "does it run" scope intact and honors
  zero-propagation (legal disposition = a new cluster scheduled this
  phase, not a carry-forward).
- **`TestOutcome.vdr` population → M9 Stage A** (per the original M08.V
  Decision 1). `vdr` is a Tester output field with no M08.7 ladder rung;
  ORCHESTRATOR §9's "rung 6" was a mislabel.

This ADR **is** the ADR-class carry-forward record M08.V Decision 2
flagged as missing. ORCHESTRATOR §9 is corrected to cite this ADR; the
`docs/execution-status.md` row-5 owner is set to rung 7. The M08.V V.2
"Scope to verify" table over-claim (it expected a `plan_loop` production
caller Stage A never contracted) is reconciled at M08.7 phase-doc
authoring time.

## Consequences

### Positive
- The missing ADR-class carry-forward record now exists (M08.V Dec-2
  satisfied); the plan_loop deferral is rooted in an accepted ADR.
- Plan execution gets a grounded home in the eval-first ladder (rung 7),
  with an assembled regression on close (`docs/cluster-pattern.md` §9).
- Zero-propagation honored: nothing carried forward to M9 by default.
- The §9 `vdr` mislabel is corrected; the docs agree.

### Negative
- M08.7b scope carries rung 7's plan task-loop build (dependent on
  rung 6's child-execution primitive — already structured in the phase
  doc).

### Neutral / future implications
- `vdr` remains a small Tester-surface field; its M9.A home is natural
  (the mentor / Builder-adjacent work). `docs/execution-status.md` has no
  `vdr` row (it is not one of the six execution primitives).
- This is a **one-time** routing decision, not a standing rule.

## Alternatives Considered

### Alternative A: both → M9 Stage A (keep M08.V as-is)
**Rejected because:** plans are one of the six paints-not-executes
execution primitives, and M08.7 is the execution milestone; routing the
plan wiring to M9 violates zero-propagation and leaves the engine
incomplete at the milestone whose ending is "ARIA runs the v0.1 subset."

### Alternative B: both → M08.7 (add a `vdr` rung)
**Rejected because:** `vdr` is a Tester output projection (fold
`decision_record` in `fold_outcome`), not an execution primitive the
ladder builds. It has no natural ladder rung and would dilute the
ladder's "does the engine run" focus; it is correctly Tester/M9 work.

## Related

- M08.V verifier retro: `docs/build-prompts/retrospectives/M08.V-retrospective.md` (Decisions 1 + 2; special-log item 5)
- `ORCHESTRATOR.md` §9 (corrected to cite this ADR)
- `docs/build-prompts/M08.7-execution-engine.md` (rung table; rung 7 = plan-driven tasks)
- `docs/cluster-pattern.md` §2 (zero-propagation), §9 (append-on-close cumulative regression)
- `docs/execution-status.md` (ledger row 5)
- MVP §M8 criterion 5 (the `vdr` surface)

## Notes

This record was directed at the M08.7.X process-hardening session: the
maintainer chose "Split: plan_loop → M08.7, vdr → M9.A" on the session's
decision surface. The M08 `docs/gap-analysis.md` entry is append-only
(`CLAUDE.md` §20) and is **not** edited; M08.7's gap-analysis
carry-forward will cite this ADR.
