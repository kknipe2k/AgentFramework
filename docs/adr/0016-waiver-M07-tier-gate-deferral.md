# ADR-0016: Waiver — M07.V Finding #1 (tier-gate enforcement deferral to the M07.5 fix-cycle)

**Status:** Accepted
**Date:** 2026-05-19
**Deciders:** @kknipe2k (maintainer)
**Tags:** waiver, capability, security, import, tier, scope, m07, m07.5

## Context

M07 Stage V's Wire pass (per ADR-0008's four-pass protocol) surfaced one 🔴 finding, recorded in `docs/build-prompts/retrospectives/M07.V-retrospective.md` Finding #1:

**`tier_gate` is defined but never invoked in the import pipeline; a Novice "Reject" does not reject.** The drift has three steps:

1. `tier_gate` (`crates/runtime-main/src/import/mod.rs:491`) returns `ImportError::TierReviewRequired(TierReview { capabilities, l3_report, requires_secrets })` for `Tier::Novice`. The function has **zero call sites** — one definition-site hit, no callers.
2. `import_artifact_with` (`crates/runtime-main/src/import/mod.rs:592-660`) skips `tier_gate` entirely and always calls `install_with` → `skills_lock::write_entry`. For `ArtifactKind::McpServer` it also unconditionally calls `reg.upsert(...)` before the install assertion, so a Novice rejection still leaves a registry row.
3. The Tauri command (`src-tauri/src/commands.rs`) sets `review_required` *post-install*; the renderer `dismissImport` only `delete`s the record from the store — no Tauri command, no backend rollback. There is **no `uninstall_artifact` / `skills_lock::remove`** anywhere in the codebase.

Net: a Novice "Reject" in the Builder Import panel dismisses the panel record but leaves the `skills.lock` entry (and, for `mcp_server`, the MCP registry row) in place. Spec §8.security L4 ("Novice — explicit 'Install' click; every install is reviewed") and MVP §M7 ("tier-gate review screen … Install/Reject") are not satisfied — the Reject click does not gate persistence.

The finding originated as an M07.C owned technical decision: the M07.C retrospective `[END]` records "removing the in-seam `tier_gate?` propagation … the renderer drives confirm-before-use." The M07 phase doc C.3.1 in fact instructed `tier_gate(&art, tier)?;` — the build agent deviated from the phase doc; **there is no phase-doc authorization for the deviation.** V's fresh-context Wire pass correctly identified that the deviation produced a non-spec-faithful contract because no rollback path was ever built.

Per ADR-0008, a 🔴 finding blocks merge unless a D.fix iteration resolves it or the build agent files a waiver-as-ADR. A D.fix was attempted — red `518f6cf` + WIP impl `bb71dbf` (the validate-vs-commit import-lifecycle split) — and is preserved on the `m07.5-salvage` branch, **not merged into M07**.

The maintainer adjudicated (decision recorded in commit `8a861cd`): the fix is not a small, scoped, same-branch D.fix. It is a multi-crate import-lifecycle restructure — a validate-and-review vs commit-install split, a new `complete_import_artifact` / `cancel_pending_import` Tauri command pair, a `PendingImportState`, an `ImportOutcome` discriminated union, and a renderer rewire (~742 LOC across 9 files per the `m07.5-salvage` WIP diff). It warrants a dedicated post-M07 fix-cycle (M07.5), the same machinery as the M06.5 IRL fix-cycle. M07 ships with the post-V deadlock fix (`8a861cd`) and finding #1 carried forward.

This waiver is unlike ADR-0009: V and the build agent **do not dispute** the finding. The drift is a genuine defect. The waiver scopes only *where and when* the fix lands, not whether it is needed.

## Decision

**We waive M07.V Finding #1 as a deferral to a dedicated post-M07 M07.5 fix-cycle.**

The waiver does not dispute the finding — V is correct that the v0.1 import path does not honor the spec §8.security L4 "Reject ⇒ no install" contract. The waiver substitutes, for the ADR-0008 "block merge until a same-branch D.fix resolves the 🔴" requirement, a dedicated **M07.5 fix-cycle milestone** executed before M08 Stage A (the M06.5 fix-cycle precedent).

Concretely:

- **M07 merges with finding #1 open and explicitly carried forward.** The post-V deadlock fix `8a861cd` (the only V-surfaced gate blocker in scope for M07 itself) ships; it is unrelated to finding #1.
- **The D.fix design + WIP implementation is preserved on `m07.5-salvage`** (red `518f6cf` + impl `bb71dbf`). M07.5 picks up from there.
- **M07.5 runs as a fix-cycle** (work-stage-class per ADR-0008; no closeout stage and no `docs/gap-analysis.md` entry of its own per `CLAUDE.md` §20 — resolution flows into M08's gap-analysis Carry-forward), **before M08 Stage A.**
- **M07.5's deliverable** is V's recommended **Option A**: move the `tier_gate(&art, tier, &report)?` call into `import_artifact_with` between L3 and `install_with` (and before `reg.upsert` for `McpServer`); add `complete_import_artifact(pending_review_id)` + `cancel_pending_import(pending_review_id)` Tauri commands; wire the renderer's `confirmImport` / `dismissImport` to them. ADR-0014's "lock-on-first-install" is preserved (the lock is written exactly once, at the point of true Install). The fix-cycle ships an **assembled regression** `reject_rolls_back_lock_and_registry` asserting a Novice + Reject leaves no `skills.lock` entry and no MCP registry row.
- **ADR number 0016 on the `claude/m07-registry-import` branch is this waiver.** The M07.5 design ADR (the validate-vs-commit lifecycle split), currently numbered `0016-import-validate-vs-commit-lifecycle.md` on the unmerged `m07.5-salvage` branch, **renumbers to ADR-0017** when M07.5 is authored — maintainer housekeeping on that unmerged branch.

## Consequences

### Positive

- **M07 merges on schedule** with the registry-import feature, the `skills.lock` integrity primitive, and the ADR-0011 (a)–(d) concrete-construction discharge (the largest open architectural carry-forward in the ledger) all shipped and V-verified.
- **The fix is sized honestly.** A ~742-LOC multi-crate import-lifecycle restructure that introduces a new IPC command pair, a `PendingImportState`, and an `ImportOutcome` discriminated union is not a "scoped D.fix iteration." It wants its own red→green stage discipline and a Stage-V-class check — which a dedicated fix-cycle gives it and a closeout-adjacent patch does not.
- **The M07.5 fix-cycle inherits proven machinery** — the M06.5 IRL fix-cycle pattern, including the assembled-app-regression mandate (`CLAUDE.md` §6 v1.8) that the `reject_rolls_back_lock_and_registry` test satisfies.
- **Validates ADR-0008's waiver-as-ADR mechanism for the fix-cycle-scheduling case** — distinct from ADR-0009's interpretation-dispute case. The mechanism now covers both "the finding is wrong for v0.1" (ADR-0009) and "the finding is right but the fix is its own milestone" (this ADR).

### Negative

- **M07 ships a known spec-contract gap.** In the v0.1 build, a Novice "Reject" in the Builder Import panel does not roll back the `skills.lock` entry (or, for `mcp_server`, the registry row). The user-visible behavior: "Reject" dismisses the panel record, but the artifact stays locked/installed. This is honest debt, recorded as a 🔴 Critical in the M07 gap-analysis Fix backlog.
- **The import-panel renderer ships (Stage E)**, so an end user *can* reach this gap in v0.1 — it is not latent behind an unbuilt surface.
- **A 🔴 shipping open is a precedent that argues for protocol vigilance.** A future waiver under thinner reasoning would erode V's blocking power. The same caution ADR-0009 raised applies here with more force, precisely because this is a genuine drift rather than an interpretation dispute.

### Bounding the blast radius (why shipping the gap is acceptable for the M07→M07.5 interval)

- **Safety primitives are not bypassed.** Every imported artifact still passes schema-validation and the L3 sandbox before `install_with`. The gap is "Reject doesn't un-install," not "unsafe artifacts install." §15c `compatible_os` mismatch still blocks before L3.
- **No production code path loads or executes an imported artifact in v0.1.** M07.V findings #2 + #5 establish exactly this: `skills_lock::verify` has no production caller, `McpDispatcher::on_server_connected` has no production caller, and the agent-with-tools loop has no production trigger (the smoke session is no-tools) — all three carried to M08.A. A wrongly-retained artifact therefore cannot be silently executed before M08.
- **M07.5 runs before M08 Stage A** — the gap is closed before the milestone that introduces the production artifact-load path.

### Neutral / future implications

- **M07.5's Stage-V-equivalent (or M08.V) is the verification endpoint.** The `reject_rolls_back_lock_and_registry` assembled regression is the expected trace; if M07.5 does not restore the contract, that is itself a 🔴 under M08.V's Wire pass.
- **M08.A's `<read_prior_milestones>` must reference this waiver**, the M07.V retrospective, and the M07.5 fix-cycle outcome — alongside M07.V's three Dec-6 🟡 carry-forwards (#2/#3/#5), which also converge on M08.A.
- **The waiver-as-ADR lane now has two shapes on record.** ADR-0009 = interpretation dispute (the finding is wrong for v0.1's architecture). ADR-0016 = fix-cycle scheduling (the finding is right; the fix is a dedicated milestone). Future verifier runs should still default to D.fix; a fix-cycle waiver requires the maintainer to confirm the fix genuinely exceeds D.fix scope (here: a multi-crate lifecycle restructure with a new IPC surface).

## Alternatives Considered

### Alternative A: D.fix iteration on the M07 branch

Run the ADR-0008 scoped D.fix (max 2 iterations) on `claude/m07-registry-import` — land the gate-then-install split, the `complete_` / `cancel_` commands, and the renderer rewire before the M07 PR opens.

**Rejected because:** the fix is a multi-crate import-lifecycle restructure (~742 LOC / 9 files per the `m07.5-salvage` WIP), not a scoped one-or-two-iteration correction. It introduces a new IPC command pair, a `PendingImportState`, and an `ImportOutcome` discriminated union — design surface that wants its own red→green stage discipline and a Stage-V-class check, not a closeout-adjacent patch. Landing it as a D.fix would either bloat the M07 PR past coherent review or rush the lifecycle-split design.

### Alternative B: Hold M07 — re-open Stages C + E

Hold the M07 milestone until the import lifecycle is correct end-to-end, effectively re-opening Stage C (the pipeline seam) and Stage E (the renderer).

**Rejected because:** M07's headline deliverables — the `skills.lock` primitive, the import-pipeline backend, and the ADR-0011 (a)–(d) discharge (the largest open architectural carry-forward in the ledger) — are all complete, V-verified, and independent of the Install/Reject contract. Holding them hostage to the lifecycle-split redesign delays the ADR-0011 discharge with no safety benefit, since the blast radius is bounded (above).

### Alternative C: Gap-analysis record only, no waiver ADR

Record the deferral solely in the M07 gap-analysis 🔴 entry and the `8a861cd` commit-message body.

**Rejected because:** ADR-0008 routes a 🔴 to "a D.fix or a waiver-as-ADR"; the M05 / ADR-0009 precedent established the waiver ADR as the durable, reviewable adjudication artifact for "a verifier found a 🔴 and the milestone shipped anyway." A commit-message-body deferral is not a durable record — it is not surfaced in the three-artifact PR review, not linked from M08.A's read-list, and not immutable. (Maintainer-selected at the M07.G closeout: file the waiver ADR.)

## Related

- **ADR-0008** — *Milestone Stage V Verifier* — defines the waiver-as-ADR mechanism this waiver invokes.
- **ADR-0009** — the M05 waiver precedent (same mechanism; that one an interpretation dispute, this one a fix-cycle scheduling waiver).
- **ADR-0014** — *skills.lock integrity* — the "lock-on-first-install" model the M07.5 fix must preserve.
- **ADR-0015** — *import-review IPC return enrichment* — the Stage E wire the M07.5 fix extends with the `pending_review_id` / `complete_` / `cancel_` surface.
- **Spec sections:** §8.security L4 (tier-gate review — "every install is reviewed; explicit Install click"); MVP §M7 ("tier-gate review screen … Install/Reject").
- **Stage V retrospective:** `docs/build-prompts/retrospectives/M07.V-retrospective.md` Finding #1 + Decision 1 (V's recommended Option A) — the verifier output this waiver answers.
- **Phase doc:** `docs/build-prompts/M07-registry-import.md` C.3.1 (`tier_gate(&art, tier)?;` — the instruction the build agent deviated from) and the M07.C retrospective `[END]` (the deviation, owned as a §12 technical decision).
- **Commit `8a861cd`** — the maintainer's M07.5-deferral decision; **`m07.5-salvage` branch** (`518f6cf` red + `bb71dbf` WIP impl, design ADR `0016` → renumber to `0017`).
- **M06.5 IRL fix-cycle** (`docs/build-prompts/M06.5-irl-fix.md`, `M06.5-summary.md`) — the fix-cycle-milestone precedent.

## Notes

The waiver burden per ADR-0008 — a waiver must name (a) the prior surface where the decision was raised, (b) the phase-doc context, and (c) the concrete loop-closing deliverable:

- **(a)** The M07.C retrospective `[END]` recorded the "remove the in-seam `tier_gate?` propagation; the renderer drives confirm-before-use" decision as an owned `CLAUDE.md` §12 technical decision.
- **(b)** M07 phase doc C.3.1 instructed `tier_gate(&art, tier)?;`. The build agent deviated from it. There is **no phase-doc authorization** for the deviation — so, unlike ADR-0009 (which cited an `<execution_warnings>` block that authorized the descope), this waiver is explicit that finding #1 is a **genuine defect**, not an authorized descope.
- **(c)** The M07.5 fix-cycle, V's Option A, with the `reject_rolls_back_lock_and_registry` assembled regression as the falsifiable proof the contract is restored.

Distinction from ADR-0009: ADR-0009 waived its findings because the wire was *architecturally correct as deferred* (no synchronous dispatch surface existed in v0.1). ADR-0016 waives nothing about correctness — finding #1 is a real bug — it waives only the *block-merge-until-a-same-branch-D.fix* requirement, substituting a dedicated fix-cycle. ADR-0009's protocol-vigilance caution therefore applies here with more force: this waiver must be the rare case (the fix genuinely exceeds D.fix scope), not a routine lane.
