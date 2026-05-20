# ADR-0017: Import validate-vs-commit lifecycle split (install-after-confirm)

**Status:** Accepted
**Date:** 2026-05-20
**Deciders:** @kknipe2k (maintainer)
**Tags:** import, capability, security, ipc, tier, lifecycle, m07.5

## Context

M07 Stage V's Wire pass surfaced 🔴 Finding #1
(`docs/build-prompts/retrospectives/M07.V-retrospective.md`): the import
pipeline installs and hash-locks every artifact **before** the Novice
tier-gate review can refuse it, so a Novice "Reject" has no backend
effect.

The drift, verified on `main` @ `ddf2a69`:

- `tier_gate` (`crates/runtime-main/src/import/mod.rs:491`) returns
  `Err(ImportError::TierReviewRequired(TierReview { … }))` for
  `Tier::Novice`. It has **zero call sites**.
- `import_artifact_with` (`import/mod.rs:592-660`) runs fetch → validate
  → §15c → L3, then — skipping `tier_gate` (the M07.C deviation, the
  `import/mod.rs:629-635` comment) — unconditionally runs `reg.upsert`
  (`McpServer`) and `install_with` (the `skills.lock` write), returning
  `Ok(Installed { … })`.
- The Tauri `import_artifact` command sets `review_required` **after**
  the install; the renderer's Reject button only deletes a local store
  record (`src/lib/graphStore.ts` `dismissImport`). There is no
  `uninstall_artifact` / `skills_lock::remove` anywhere.

The maintainer adjudicated (ADR-0016) that the fix exceeds scoped-D.fix
scope and waived it to the dedicated **M07.5 fix-cycle**, which must run
before M08 Stage A introduces the production artifact-load path. M07.5
needs a design ADR for the lifecycle change because it adds a
**renderer↔main Tauri IPC command pair** — the same IPC class ADR-0015
governs, and a `§11`-class decision surface.

Constraints:

- **ADR-0014 lock-on-first-install** must be preserved — the
  `skills.lock` entry is written exactly once, at the point of true
  install; M07.5 may change *when* that point is reached but not the
  lock's content, schema, or SRI hash.
- **No framed-JSON IPC change** — the drone / sandbox IPC is untouched;
  the new commands are Tauri renderer↔main.
- **No schema change** — `schemas/skills-lock.v1.json` is unchanged.
- **v0.1 is single-session** (§0d) — a sub-second window between
  "import returns a review" and "user clicks Install/Reject" lives
  entirely within one process lifetime.
- The M07.V retrospective Finding #1 offered two spec-faithful options:
  **Option A** (install-after-confirm — gate then install) and
  **Option B** (lock-on-import then add an `uninstall_artifact`
  command). V recommended Option A.

Without a decision, M07 ships a known spec §8.security L4 contract gap
(a Novice "Reject" does not reject) into M08, where the
production artifact-load path makes the gap reachable.

## Decision

**We adopt Option A: the import pipeline splits into a *validate* phase
and a *commit* phase, and a Novice import is held — uninstalled,
unlocked — between them until the renderer confirms the tier-gate
review.**

Concretely, in `crates/runtime-main/src/import/mod.rs`:

- `import_artifact_with` **calls `tier_gate`** between L3 and the
  install half, and returns a new `ImportOutcome` enum instead of a
  bare `Installed`:
  - `ImportOutcome::Installed(Installed)` — the tier-gate passed
    (`Tier::Promoted`, L4 auto-accept); the artifact is installed +
    hash-locked inline.
  - `ImportOutcome::Pending { review: TierReview, pending: PendingImport }`
    — the tier-gate returned `TierReviewRequired` (`Tier::Novice`);
    **nothing is installed, nothing is locked, the MCP registry is not
    upserted.**
- The install half — MCP-registry upsert (`mcp_server` only) +
  `install_with` (the `skills.lock` write) — is extracted into a
  private `commit_import` shared by the inline (Promoted) path and the
  deferred path, so both run byte-identical install logic.
- `PendingImport` carries the held import (the `ValidatedArtifact`, the
  `ImportSource`, the `L3Report`, the `Tier`) — everything
  `complete_import_with` needs.
- `complete_import_with(&PendingImport, …)` is the public entry that
  runs `commit_import` on a renderer confirm.
- `TierReview` gains a `share_provenance` field, making it the complete
  review primitive the `Pending` arm carries to the renderer.

In `src-tauri/src/`:

- The wire `ImportOutcome` becomes a `#[serde(tag = "status")]`
  discriminated enum (`Pending` / `Installed`); the renderer
  discriminates on `status`.
- Two new Tauri commands: `complete_import_artifact(pending_review_id)`
  runs the held install half and returns the terminal `Installed`
  outcome; `cancel_pending_import(pending_review_id)` drops the held
  state (idempotent — nothing to roll back because the install half
  never ran).
- A Tauri-managed `PendingImportState`
  (`Mutex<HashMap<String, PendingImport>>`) holds Novice imports
  between `import_artifact` returning `Pending` and the renderer's
  `complete_`/`cancel_` call, keyed by a freshly minted
  `pending_review_id`.

The renderer's tier-gate review modal's Install button calls
`complete_import_artifact`; its Reject button calls
`cancel_pending_import`.

ADR-0014's lock-on-first-install is preserved: `install_with` (hence
`skills_lock::write_entry`) runs exactly once, inside `commit_import`,
at the point of true install — for Promoted that is inline, for Novice
that is on confirm. A Novice "Reject" now genuinely rejects: the
pipeline never reached `commit_import`, so there is no entry to remove.

The import-fetch egress path — the tautological `EnforcerGate`
(simplify finding CQ-M07-1) and the unhardened `HttpFetcher` — is a
sibling M07.5 concern, decided separately in **ADR-0018** (import-fetch
SSRF egress hardening). ADR-0017 governs *when* the install commits
(the tier-gate lifecycle); ADR-0018 governs *how* the artifact bytes
are fetched (the SSRF-hardened egress gate). They are independent code
paths landed in separate M07.5 stages.

## Consequences

### Positive

- **The spec §8.security L4 contract holds.** A Novice "Reject" leaves
  no `skills.lock` entry and no MCP registry row — "every install is
  reviewed; explicit Install click" (spec §8.security L4; MVP §M7
  "Install/Reject") is satisfied. M07.V 🔴 #1 is closed.
- **No rollback path is needed.** Because the install half never runs
  for an unconfirmed Novice import, there is nothing to un-install — no
  `uninstall_artifact`, no `skills_lock::remove`, no
  `Registry::remove`. The fix is structurally simpler than Option B and
  cannot leave a partially-rolled-back state.
- **ADR-0014 lock-on-first-install is preserved exactly.** The lock is
  written once, at true install; M07.5 changes only *when* that point
  is reached. The `skills.lock` content, schema, and SRI hash are
  unchanged.
- **The L3 sandbox still runs before the user sees the review.** The
  validate phase runs fetch → validate → §15c → L3 in full — the Novice
  review screen shows a *real* L3 report; "sandbox-before-trust"
  (ADR-0014 threat model) is unchanged.
- **The IPC enrichment is small and additive.** Two new Tauri commands
  + a discriminated return; the ADR-0015 `import_artifact` return shape
  is extended, not replaced.

### Negative

- **A held `PendingImport` lives in process memory.** Between
  `import_artifact` returning `Pending` and the renderer's
  `complete_`/`cancel_` call, the validated artifact bytes sit in
  `PendingImportState`. If the app is killed in that window the held
  import is lost and the user re-imports — acceptable for v0.1
  single-session scope (§0d), but it means a Novice review is not
  durable across a restart. A v1.0 multi-session runtime that wants
  durable pending-reviews would persist `PendingImportState`.
- **The L3 sandbox runs before the user has accepted.** A Novice
  importing a hostile artifact still triggers an L3 sandbox run before
  they can reject — but L3 *is* the sandboxed evaluation, and its whole
  purpose is to run untrusted code safely so the review can show a real
  report. This is the intended posture, not a regression.
- **`import_artifact_with`'s return type is a breaking change** for its
  one caller (`src-tauri/src/commands.rs::import_artifact`). M07.5
  Stage A.fix lands both the runtime-main change and the `src-tauri`
  caller in one stage precisely because the workspace would not compile
  with the change split.

### Neutral / future implications

- **The `PendingImportState` is the natural seam for a future durable
  review queue.** A v1.0 multi-session runtime persists it; v0.1 holds
  it in memory.
- **`complete_import_with` is the v1.0 attach point for a
  cryptographic-provenance gate.** Sigstore / SLSA verification
  (ADR-0014 threat model, deferred to v1.0) would run inside the commit
  phase, before `install_with`.
- **The honest `EnforcerGate` makes the v1.0 import-domain-policy work
  explicit.** The `NetworkGate` seam is where a user-configured import
  allowlist attaches; the gate's doc comment now says so plainly rather
  than implying a policy v0.1 does not have.

## Alternatives Considered

### Alternative A: Option B — lock-on-import + an `uninstall_artifact` command

Keep the current "always install + lock," and add an
`uninstall_artifact(lock_key)` Tauri command (calling a new
`skills_lock::remove_entry` and, for `mcp_server`, `Registry::remove`)
that the renderer's Reject button invokes.

**Rejected because:** it preserves a window in which a rejected
artifact is locked-and-registered before the user clicks Reject, and it
introduces a rollback path — `skills_lock::remove_entry` +
`Registry::remove` — that can itself fail or partially complete,
leaving an inconsistent state. Option A has **no rollback** because it
has nothing to roll back; the install half simply never runs. Option A
also matches the spec phrasing most literally ("every install is
reviewed" — under Option B the install precedes the review). The M07.V
retrospective recommended Option A for the same reasons.

### Alternative B: Route the MCP registry through drone IPC for single-writer ownership

Make the import pipeline write the MCP registry via a drone IPC command
so the drone is the single writer (the ADR-0012 "Alternative A" forward
path).

**Rejected because:** it is orthogonal to 🔴 #1 (the finding is "the
review does not gate persistence," not "the registry has two writers")
and it is out of v0.1 scope — ADR-0012 already records the
second-WAL-connection model as the v0.1 decision. M07.5 must stay
scoped to the waived finding.

### Alternative C: Keep the M07.C "always install, renderer-only acknowledgment" model

Treat the install-before-review behavior as a deliberate v0.1 design
and amend the spec to match it.

**Rejected because:** M07.V established — and the build agent did not
dispute (ADR-0016) — that this is a genuine spec §8.security L4 contract
drift, not an interpretation dispute. The maintainer adjudicated the
finding as a real defect requiring a fix-cycle, not a spec amendment.

## Related

- **ADR-0016** — *Waiver: M07.V Finding #1 (tier-gate enforcement
  deferral to the M07.5 fix-cycle)* — the waiver this ADR's M07.5
  fix-cycle discharges. ADR-0016 §Decision reserves ADR number 0017 for
  this design ADR (0016 being the waiver).
- **ADR-0018** — *Import-fetch SSRF egress hardening* — the sibling
  M07.5 decision; ADR-0017 governs the install lifecycle, ADR-0018 the
  fetch-egress security.
- **ADR-0014** — *skills.lock integrity* — the lock-on-first-install
  model this decision preserves.
- **ADR-0015** — *import-review IPC return enrichment* — the
  `import_artifact` return shape this decision extends with the
  discriminated `Pending`/`Installed` enum + the `complete_`/`cancel_`
  command pair.
- **ADR-0010** — *MCP dispatch dependency inversion* — why the
  `McpRegistry` seam stays dependency-inverted (the registry adapter is
  constructed in the Tauri shell); unchanged by this ADR.
- **Spec sections:** §8.security L4 (tier-gate review — "every install
  is reviewed; explicit Install click"); MVP §M7 ("tier-gate review
  screen … Install/Reject"); §15c/§15d (the metadata the validate phase
  still parses).
- **Stage V retrospective:** `docs/build-prompts/retrospectives/M07.V-retrospective.md`
  Finding #1 + Decision 1 (the recommended Option A).
- **Phase doc:** `docs/build-prompts/M07.5-tier-gate-fix.md` — the
  fix-cycle that implements this ADR.

## Notes

**The ADR-number renumber (0016 → 0017).** A WIP design draft for this
decision was authored during M07 closeout on the build-machine-local
`m07.5-salvage` branch as `0016-import-validate-vs-commit-lifecycle.md`
(alongside the abandoned D.fix WIP — red `518f6cf` + impl `bb71dbf`).
That number collides with the merged waiver ADR-0016. This file —
authored fresh from ADR-0016's fully-specified Option A and the M07.V
Finding #1 recommendation — is the canonical design ADR at **0017**.
The M07.5 branch carries this 0017; it does not carry the unmerged
salvage `0016` draft (which was never reviewed or merged). This
satisfies the "renumber the salvage design ADR 0016→0017" the ADR-0016
waiver §Decision and `ORCHESTRATOR.md` §9 call for.

**Status flip.** Per `CLAUDE.md` §11 this ADR is filed `Proposed`. M07.5
Stage A.fix flips it to `Accepted` in the impl commit that lands the
lifecycle split (the M06.5.A.fix / ADR-0012 precedent — the stage that
implements an ADR flips it).
