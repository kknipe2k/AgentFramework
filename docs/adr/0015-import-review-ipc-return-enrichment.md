# ADR-0015: `import_artifact` IPC return enrichment for the §M7 review screen

**Status:** Proposed
**Date:** 2026-05-19
**Deciders:** @kknipe2k
**Tags:** ipc, scope, security, capability

## Context

MVP §M7 / M07 Stage E specify a Builder Import panel whose tier-gate
review screen surfaces, before a Novice import is confirmed: the
artifact's **declared capabilities** (plain-English disclosure), the
**L3 sandbox report**, and the **`share_provenance`** trust block.

Stage E was authored under a "NO new backend" scope lock on the
assumption that this review data was already renderer-reachable. The
v1.8 `<wire_signature_audit>` falsified that assumption against the
**shipped Stage C wire**:

- The shipped `import_artifact` Tauri command returns only
  `ImportOutcome { lock_key, review_required, requires_secrets }`
  (`src-tauri/src/commands.rs`). It is a hand-written serde bridge
  struct — **not** schema-generated (no `import*.json`; absent from
  `crates/runtime-core/src/generated/`; the established
  `McpTool`/`ResumePlan` hand-mirror precedent).
- `import_artifact_with` (`crates/runtime-main/src/import/mod.rs`)
  **already computes** all three pieces during the import —
  `capability_summary(&art.raw)`, the `L3Report`, and
  `art.meta.share_provenance` — then **discards** them; only
  `Installed { lock_key, report, requires_secrets }` is returned and
  the command narrows it further.
- `skills.lock` (`schemas/skills-lock.v1.json`) is a **closed** 6-field
  integrity/provenance ledger with **no `capabilities` field**, and **no
  Tauri command reads the lock**. So the "render from the lock/manifest"
  (browser-extension) pattern is not available without a
  `skills-lock.v2.json` + ADR-0014-class bump to a user-VCS-committed
  artifact.
- The review data is **not persisted keyed by `lock_key`**
  (`install_with` writes only the lock entry). A post-hoc read command
  (`get_import_review(lock_key)`) would therefore have to re-fetch +
  re-validate + **re-run the L3 sandbox** — expensive, and unsound for
  `file:` sources whose path may be gone.

If we don't decide, Stage E either ships an impoverished review screen
that does not meet the §M7 spec, or it re-runs the import pipeline
post-hoc (a worse change than the one below).

## Decision

We **additively enrich the existing `import_artifact` return** with the
review data the import pipeline already computes and currently discards.

Concretely: `crates/runtime-main/src/import/mod.rs` computes
`capability_summary(&art.raw)` unconditionally inside
`import_artifact_with` and carries it, the already-built `L3Report`, and
`art.meta.share_provenance` into the `Installed` struct;
`src-tauri/src/commands.rs` additively widens `ImportOutcome` with the
corresponding fields and maps them through. The command **signature
(params) is unchanged**; there is **no new fetch**, **no new IPC
command**, and **no schema change** (both structs are hand-written serde
bridge types, not `schemas/*.v*.json`-generated — verified before this
ADR per the §14 hand-mirror precedent). The Stage E renderer mirrors the
widened `ImportOutcome` in `src/lib/ipc.ts` (the `McpTool`/`ResumePlan`
hand-mirror pattern) and renders the §M7 disclosure from it.

This is filed as an ADR because it changes the renderer↔main Tauri IPC
return contract of a §5-shell command on a CODEOWNERS-flagged path
(`import/`), per CLAUDE.md §11 + Hard Rule 8 (plan-first; the plan was
surfaced and approved before this ADR).

## Consequences

### Positive
- The §M7 review screen renders the spec'd capability disclosure + L3
  report + `share_provenance` from real backend data.
- No re-fetch / no re-run of the L3 sandbox; the data is the exact
  artifact that was installed (no TOCTOU between review and install).
- No new schema, no new IPC command, no new dependency; the change is
  ~one struct enrichment + the command mapping + contract tests.
- The renderer↔main contract is pinned by a Rust integration test that
  serializes a *real* `ImportOutcome` from a *real* import, so the
  renderer's fixture cannot drift into a fabricated-shape false-green.

### Negative
- The Stage-E "NO new backend" scope lock is broken (this is a real
  backend touch). Recorded as a grandfathered M07-phase-doc defect + a
  `docs/gap-analysis.md` entry; the phase doc itself is not edited
  mid-flight per CLAUDE.md §8 (the M07.A–D grandfathering precedent).
- `ImportOutcome` / `Installed` grow; existing call sites are
  unaffected (Stage E is the first consumer) but the structs are now
  load-bearing for the renderer review contract.

### Neutral / future implications
- A v1.0 SLSA/TUF provenance layer (noted in `import/mod.rs`) attaches
  at the same `share_provenance` seam this exposes.
- If `import_artifact`'s shape later needs cross-language schema
  guarantees, a future ADR may promote these bridge structs to a
  `schemas/import.v1.json` source-of-truth (explicitly out of scope
  here — they remain hand-mirrored per the `McpTool`/`ResumePlan`
  precedent).

## Alternatives Considered

### Alternative A: Impoverished render (faithful-to-shipped-wire only)
Render only `review_required` + `requires_secrets` + a static trust
line; carry capabilities/L3/provenance as a backlog item.
**Rejected because:** it does not meet the §M7 spec (no capability
disclosure, no L3 report) and defers the core review primitive
indefinitely.

### Alternative B: `get_import_review(lock_key)` read-only command
A separate additive command the renderer calls after import.
**Rejected because:** the review data is not persisted keyed by
`lock_key`; the command would re-fetch + re-validate + re-run the L3
sandbox (expensive; unsound for `file:` sources). Strictly worse than
enriching the return the pipeline already produces.

### Alternative C: Add `capabilities` to `skills.lock`
Make the lock a permission manifest the renderer reads.
**Rejected because:** `skills-lock.v1.json` is a closed schema; this is
a `skills-lock.v2.json` + ADR-0014-class bump to a user-VCS-committed
artifact, and still needs a new command to expose the lock to the
renderer. Far heavier than enriching an in-process serde bridge struct.

### Alternative D: Split E1 (backend) / E2 (renderer)
Isolate the backend enrichment as its own stage per the D1/D2
precedent.
**Rejected because:** the touch is ~30–50 lines (one struct enrichment
+ mapping + contract tests), not D1/D2-scale; the ceremony exceeds the
change. Plan-first + this ADR provide the CODEOWNERS-path safeguard
without a stage split.

## Related
- Spec sections: §M7 (MVP), §15c/§15d, §8.security L3/L4, ADR-0005
  (`share_provenance`), ADR-0014 (skills.lock integrity)
- Prior ADRs: ADR-0005, ADR-0014; CLAUDE.md §11 (IPC-protocol ADR
  trigger), §8 (phase-doc grandfathering), Hard Rule 8
- Issues: None
- External references: None

## Notes

Surfaced as a Hard Rule 8 plan-first decision before any red test;
approved in principle by the maintainer with two binding conditions:
(1) `ImportOutcome` verified hand-mirrored, not schema-generated (else
this becomes a §14 schema v1.1 + ADR) — **verified PASS** before this
ADR was written; (2) the regression must drive a real import → enriched
return → rendered disclosure (no mocking the bridge/`ImportOutcome`) —
encoded as the Rust-integration-test serialization anchor described
under Decision/Consequences.
