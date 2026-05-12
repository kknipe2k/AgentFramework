# Technical Debt — Append-Only Ledger

> 🟢 findings from Stage V verifier runs (per ADR-0008). Distinct from `docs/gap-analysis.md` (product↔spec drift) and `docs/gotchas.md` (don't-do-this patterns). Tech debt is "noted, not blocking" — code that works correctly per the spec but is structurally fragile, awkward to extend, or known-future-rework. Append-only: existing entries never edited or reordered.

---

## Format

Each entry is one section. Required fields:

```markdown
## TD-NNN — <short title>

**Date logged:** YYYY-MM-DD
**Found by:** Stage V verifier run M[NN].V (or "manual review", "post-merge IRL test", etc.)
**Pass that surfaced it:** Inventory | Wire | Behavior | Multi-call | (N/A if manual)
**Category:** structural | cosmetic | scalability | extensibility | observability | other
**Resolution status:** open | in-flight (PR #N) | resolved (commit hash) | superseded (TD-MMM)

### Description

One paragraph. Concrete: file paths, line numbers, the structural shape of the debt.

### Why it's debt not bug

What works correctly today. The spec is satisfied; code is functional. The debt is structural — what's hard to do next, not what's broken now.

### Recommended approach (when addressed)

One paragraph. Concrete: which files would change, estimated complexity, dependencies (if any).
```

## Numbering

`TD-001`, `TD-002`, … sequentially. Never reuse. Resolved entries stay; their `Resolution status` field is updated to `resolved (<commit hash>)` in a NEW entry that supersedes — the original entry text never changes.

## Append-only enforcement

CI gate (planned): a diff check parallel to `docs/gap-analysis.md` append-only verification. Until that gate ships, append-only is a code-review discipline.

---

## Entries

<!--
Initial seed: this ledger ships with ADR-0008. First entries land when M05.V runs (the first milestone shipped under v1.5 protocol with Stage V active). Until then, "None observed." is honest.
-->

None observed. (This ledger begins receiving entries at M05.V or any earlier Stage V iteration of M04 that surfaces 🟢 findings.)
