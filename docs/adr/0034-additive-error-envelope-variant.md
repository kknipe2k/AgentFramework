# ADR-0034: Additive `CmdError` envelope variant (`PathNotPermitted`)

**Status:** Proposed (flips to Accepted at M09.5.G)
**Date:** 2026-06-10
**Deciders:** @kknipe2k
**Tags:** schema, security, scope

## Context

M09.5.A (TD-051 / external review C2) confines renderer-supplied filesystem
paths: `save_framework` / `load_framework` / `import_artifact(file)` now refuse
a path that resolves outside the dialog-registered roots ∪ `app_local_data_dir`.
The phase doc (A.3 step 3, A.5, A.8) directs that refusal to surface as a
**typed** `CmdError` of kind `path_not_permitted` — the renderer's
`unwrapCmdError` renders it, and the adversarial e2e asserts
`errorType === 'path_not_permitted'`. The existing `CmdError` variants
(`SetupRequired`/`Provider`/`Drone`/`KeyStore`/`Internal`) carry no such kind,
so a new variant is required.

Two project rules bear on this:

- **CLAUDE.md §11** requires an ADR for any change to a `schemas/*.json` file.
- **The M09.5 milestone "Locks"** say "no schema change (nothing here touches
  artifact shapes)."

`schemas/error.v1.json` is the renderer↔main IPC **error envelope**, not an
**artifact** schema (framework / skill / tool / agent / capability). The lock's
parenthetical scopes it to artifact shapes; the error envelope falls outside it.
Per CLAUDE.md §14, adding an enum variant is a minor, in-place `v1` change (the
`$id` URL does not change). What remained was the §11 ADR obligation — recorded
here.

## Decision

We add the additive `PathNotPermitted` variant to `schemas/error.v1.json` as a
§14 minor bump, in place: the `$id` is unchanged, the typify-generated
`crates/runtime-core/src/generated/error.rs` and `src/types/error.ts` are
regenerated via `cargo xtask regenerate-types` (the CI drift gate stays green),
and `runtime_core::CmdError` gains a `path_not_permitted(msg)` constructor plus
`Display` / `message()` arms. The renderer's `isCmdError` guard and
`unwrapCmdError` recognize the new kind.

Additive error-envelope variants remain ADR-recorded per §11 — this ADR is the
record for this one, and sets the precedent that the error envelope is **not**
covered by the milestone's artifact-shape lock.

## Consequences

### Positive
- The path refusal is a typed contract the renderer renders distinctly (a
  plain-language "location not permitted" message), not an opaque `Internal`.
- Old documents/clients are unaffected — additive variants don't break the
  existing five.

### Negative
- The wire enum grows; every exhaustive `match` on `CmdError` in Rust must
  handle the new arm (the compiler enforces this — caught at build).

### Neutral / future implications
- `codecov.yml` and the §6 `cargo llvm-cov` invocations are unchanged (no new
  exclusion). Future additive error-envelope variants follow this same minor-bump
  + ADR-record path.

## Alternatives Considered

### Alternative A: a new `error.v2.json` major-version file
**Rejected because:** the change is purely additive (a new variant, no removal or
restriction), which §14 classifies as a minor in-place bump — a v2 file is for
breaking changes.

### Alternative B: no schema change — map the refusal to `CmdError::Internal`
**Rejected because:** the typed `path_not_permitted` kind IS the contract the
renderer consumes (the e2e asserts on it; `unwrapCmdError` renders it
distinctly). Folding it into `Internal` would make a security refusal
indistinguishable from an unexpected runtime error.
