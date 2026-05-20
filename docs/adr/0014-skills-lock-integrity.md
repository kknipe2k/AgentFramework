# ADR-0014: skills.lock artifact-integrity primitive

**Status:** Accepted
**Date:** 2026-05-18
**Deciders:** @kknipe2k
**Tags:** schema, security, scope

## Context

M07 (Registry Import) lets a user import a skill / tool / agent /
MCP-server-config by GitHub-raw URL or local file. Spec §2181-2216
requires that every installed artifact be recorded in a `skills.lock`
at the framework root and that the runtime **validate the content hash
on every artifact load** — a mismatch blocks the load with an
`artifact_hash_mismatch` event so the user reinstalls or removes the
artifact. The lock is checked into the user's framework repo alongside
the framework JSON (spec §2216), enabling reproducible installs across
machines (spec §2204).

No such primitive exists on `main`. It is a new artifact shape consumed
by the runtime, so per the §12 Engineering Charter + CLAUDE.md §11/§14
it requires a new schema (`schemas/skills-lock.v1.json`) and an ADR.
Constraints: v0.1 scope is URL/file import only — the Anthropic-upstream
search UI and its `source_commit` trust chain (spec §2186-2192,
§2218-2243) are v1.0 (MVP §M7), so v0.1 must NOT model `source_commit`
or upstream provenance. Sigstore / SLSA / TUF artifact provenance is
also v1.0 (spec §0d) and out of scope here. The module must be
path-agnostic per the CLAUDE.md §9 `audit::file_path` archetype.

The spec sketch (§2197-2211) is illustrative, not normative on every
detail, and two of its specifics needed a decision before the schema
could be authored:

1. The sketch's top-level map key is `installed`; an early M07 phase-doc
   draft used `entries`.
2. The sketch's `content_hash` is `"sha256:<hex>"` (algorithm-`:`-hex).

## Decision

**We adopt a schema-defined `skills.lock` whose content hash is an
SRI-encoded SHA-256 digest, whose load-time mismatch is a hard block,
and whose serialization is canonical for byte-identical cross-machine
reproducibility.** Specifically:

1. **Integrity > availability.** A recomputed content hash that does not
   match the locked hash BLOCKS the artifact's use. `skills_lock::verify`
   returns `LockError::HashMismatch`; the load path maps it onto the new
   `AgentEvent::ArtifactHashMismatch` event (schema-defined in
   `event.v1.json`) and refuses to run the drifted bytes. An artifact
   with no lock entry is also blocked (`LockError::NotFound`) — an
   un-locked artifact must not load by virtue of an absent record.

2. **SRI-encoded SHA-256.** `content_hash` is `sha256-<standard-base64>`
   (the npm/Subresource-Integrity convention), not the spec sketch's
   `sha256:<hex>`. The `sha256-` algorithm prefix is self-describing, so
   a future digest change (npm itself uses SHA-512 via SRI) is a value
   change, not a schema break. The `schemas/skills-lock.v1.json`
   `SriHash` pattern (`^sha256-[A-Za-z0-9+/]+={0,2}$`) enforces the
   prefix at the schema level, so a bare hex digest fails validation.
   This is a deliberate, recorded tightening of the spec sketch.

3. **Spec-faithful `installed` key.** The top-level map key is
   `installed` (spec §2200), not `entries`. The lock is a durable
   user-VCS-committed artifact; the wire key matches the spec contract
   (CLAUDE.md §2 — the spec is the contract). Maintainer-decided
   2026-05-18.

4. **v0.1 `source` is URL | file only.** `source` is a `type`-tagged
   union of `{ url }` / `{ path }`. `source_commit` and
   `anthropic-upstream` are deliberately absent — they belong to the
   v1.0 upstream trust chain (MVP §M7), not modelled in v0.1.

5. **Canonical serialization.** `write_entry` serializes with `installed`
   keys sorted alphabetically by `name@version` and a stable field
   order, independent of `HashMap` iteration order and of whether
   `serde_json`'s `preserve_order` feature is enabled. Two installs of
   the same artifact set produce a byte-identical file (spec §2204);
   git auto-resolves concurrent adds rather than conflicting.

6. **Path-agnostic.** The module takes `&Path`; the Tauri shell resolves
   `<framework_root>/skills.lock` (CLAUDE.md §9). Unit-tested with
   `tempfile`.

Types are generated from the schema via `cargo xtask regenerate-types`
(typify); the `artifact_hash_mismatch` event variant is mirrored into
the hand-curated `runtime_core::event::AgentEvent` per the established
curated-union precedent. The schema add introduces a `base64` runtime
dependency (MIT/Apache-2.0; supply-chain-clean per `cargo deny`) for
SRI encoding, alongside the already-present `sha2`.

This ADR is `Proposed`; it flips to `Accepted` in the M07 PR before
merge (CLAUDE.md §11). It also introduces a new ≥95% per-module
coverage gate on `runtime_main::skills_lock` (safety primitive,
CLAUDE.md §5); the four-mirror coverage-policy sync
(`docs/coverage-policy.md` §B/§C + CLAUDE.md §5/§6 + `codecov.yml`)
lands at the M07 closeout stage per the v1.8
`<coverage_policy_reconciliation>`.

## Consequences

### Positive

- A tampered or drifted artifact cannot silently run; integrity is
  enforced on every load, not just at install.
- Reproducible cross-machine installs: the canonical lock is a
  meaningful, diffable, mergeable VCS artifact.
- The SRI prefix makes the digest algorithm forward-swappable without a
  schema major bump.
- Schema-as-source-of-truth: Rust types are generated, not hand-written
  (Hard Rule 5); the renderer consumes the same `event.v1.json`.

### Negative

- A legitimate out-of-band artifact edit (a user hand-fixes an installed
  skill) now blocks load until reinstall/relock. This is the intended
  trade-off (integrity > convenience); the Reinstall/Remove prompt
  (M07 Stage E) is the escape hatch.
- One new runtime dependency (`base64`).
- The spec sketch's `sha256:<hex>` and `installed`/`entries` ambiguity
  are resolved here, not in the spec; the spec text is not edited
  (immutable-contract discipline) — this ADR is the reconciliation
  record.

### Neutral / future implications

- v1.0 attaches Sigstore / SLSA / TUF provenance at the
  `share_provenance` seam (ADR-0005) and the upstream `source_commit`
  trust chain (spec §2186-2192) — additive to this lock, not a
  replacement; `source` is a union ready to gain an upstream variant.
- Digest agility is a value change (`sha256-` → `sha512-`), still within
  `skills-lock.v1.json` if the pattern is widened in a minor bump.

## Alternatives Considered

### Alternative A: bare-hex `sha256:<hex>` per the spec sketch verbatim

**Rejected because:** it hard-codes the algorithm into the format; a
future digest change would be a schema break. SRI's `sha256-` prefix is
the established ecosystem convention (npm) for exactly this agility and
costs nothing now.

### Alternative B: warn-and-continue on hash mismatch (availability > integrity)

**Rejected because:** the spec is explicit (§2214 "block load") and a
silent-continue on a tampered artifact defeats the entire point of the
L1-L5 + lock chain. Observability-class best-effort is the audit
primitive's posture (ADR-aligned), not the integrity primitive's.

### Alternative C: store the hash in the existing `audit` log / SQLite, no new schema

**Rejected because:** the lock is a user-VCS-committed, human-diffable,
reproducible artifact (spec §2216) with a distinct shape and lifecycle
from the append-only audit log; conflating them breaks reproducibility
and the schema-as-source-of-truth contract.

### Alternative D: keep the phase-doc `entries` key / cross-schema `$ref` for SriHash

**Rejected because:** (1) `installed` is the spec §2200 contract and the
spec wins (CLAUDE.md §2); (2) a cross-schema `$ref`
(`skills-lock.v1.json#/$defs/SriHash`) from `event.v1.json` breaks that
schema's `json-schema-to-typescript` target, which resolves local
`$defs` only — the established M04.D mirror pattern (`CapabilityKindRef`,
`McpServerNameRef`, `TierRef`) requires a local `SriHashRef` `$def`
instead. Pattern kept byte-identical to the `skills-lock.v1.json` source
of truth.

## Related

- Spec sections: §2181-2216 (skills.lock format + reproducibility +
  load-time validation), §2156-2180 (Phase 7 import), §0d (Sigstore /
  upstream are v1.0), §12 (schema-as-source-of-truth)
- MVP: §M7 (URL/file import only in v0.1; no upstream search)
- Prior ADRs: ADR-0005 (share-provenance seam — v1.0 provenance attaches
  here), ADR-0006 (mcp-servers schema — `mcp_server` is a lock `kind`),
  ADR-0008 (Stage V), ADR-0010/0011 (M07 D1/D2 carry-forward)
- Issues: none
- External references: W3C Subresource Integrity (SRI) format;
  npm `package-lock.json` integrity field; Cargo/uv lockfile
  reproducibility prior art

## Notes

The `installed` vs `entries` and SRI vs hex decisions were surfaced to
the maintainer before the M07.B strict-TDD red commit (per CLAUDE.md §2
"surface the contradiction, don't pick" + the M07.A precedent) and
decided 2026-05-18: `installed` (spec-faithful) + SRI `sha256-<base64>`
(this ADR's refinement). Recorded in the M07.B retrospective
`[END] Decisions`.
