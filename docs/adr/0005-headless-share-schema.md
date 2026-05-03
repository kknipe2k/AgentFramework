# ADR-0005: Headless-share metadata in `framework.v1.json`

**Status:** Accepted
**Date:** 2026-05-03
**Deciders:** @kknipe2k
**Tags:** schema, scope, sharing, distribution, capability

## Context

The runtime's product identity treats agentic frameworks as **portable artifacts** — built, tested, and signed once, then sharable to recipients who may or may not have the runtime installed. Spec §7 (Registry import) covers runtime-to-runtime sharing in v0.1. The new requirement is to support **runtimeless sharing** — recipients running the framework without the desktop runtime.

Three sharing tiers are now declared in spec §15:

1. **Runtime-to-runtime** (v0.1, M07) — recipient has the full Tauri runtime installed
2. **Headless CLI** (v1.0) — recipient runs `agent-runtime-cli` (single static binary, ~10 MB)
3. **WASM runtime** (v2.0+ stretch) — embeddable in browsers, Node, any WASM host

The headless CLI tier is the v1.0 commitment. It requires the framework artifact to declare four pieces of metadata so the CLI can run it without ambiguity:

- Which secrets the recipient must provide
- Whether the framework is headless-compatible at all (vs requiring the live graph / workbench)
- Which OSes the framework supports (some tools are OS-specific)
- An audit trail of any rebaking the export-time "Share It" module (spec §15e) applied

Without these fields in v0.1, M03–M07 ship frameworks that can't be inspected by the v1.0 Share It module without breaking-schema-change migration. With them, the v0.1 runtime reads them but doesn't yet act on them — the runtime carries the metadata forward, and v1.0 lights up the consumers.

The constraint: this can't be a v0.1 *feature* — no UI, no CLI binary, no Share It module yet. It can only be schema groundwork. Per CLAUDE.md §11, schema changes require an ADR; per `schemas/README.md` versioning policy, additive optional fields are a minor in-place bump (the `$id` doesn't change).

## Decision

We add four optional properties to `schemas/framework.v1.json` as an additive in-place minor bump:

1. **`requires_secrets`** — array of strings (env-var-style names matching `^[A-Z][A-Z0-9_]*$`); defaults to `[]`. Names only, never values.
2. **`runtime_dependency_class`** — enum `"desktop_runtime"` | `"headless_compatible"`; defaults to `"desktop_runtime"` (safe default — explicit opt-in to headless).
3. **`compatible_os`** — array of enum `"windows"` | `"macos"` | `"linux"`; defaults to `["windows", "macos", "linux"]` (assume portable).
4. **`share_provenance`** — object with `exported_at`, `exported_by`, `for_runtime_class`, `for_os`, `rebake_changes`; absent unless populated by the Share It module at export time.

The fields are documented in schema descriptions and in spec §15d. The runtime reads them at framework-load time but does not yet enforce them in v0.1 user-facing surfaces (no headless CLI exists; no Share It module exists). M03–M07 frameworks ship with the right artifact shape so the v1.0 consumers (headless CLI, Share It module) have data to work with — no schema migration required at v1.0 cutover.

The `$id` URL of `framework.v1.json` does not change. Existing v0.1 frameworks remain valid (all four properties are optional with defaults). Generated Rust types (`crates/runtime-core/src/generated/framework.rs`) and TS types (`src/types/framework.ts`) regenerate via `cargo xtask regenerate-types`; CI fails until they match the new schema.

## Consequences

### Positive

- **Forward compatibility.** Frameworks built in M03–M07 ship sharing-ready. When the v1.0 headless CLI lands, it runs M03–M07-era frameworks without re-export.
- **Zero v0.1 user-facing scope.** No UI changes, no CLI binary, no Share It module yet. The runtime quietly carries the metadata forward.
- **Defaults are safe.** `runtime_dependency_class` defaults to `desktop_runtime` (safer — Share It explicitly opts into headless after inspection). `compatible_os` defaults to all three (the Share It module narrows automatically; manual override possible).
- **Audit trail in `share_provenance`.** Recipients see when/by-what the bundle was prepared and any substitutions the Share It module made. Trust signal without telemetry.
- **OS portability becomes enforced.** The `compatible_os` field plus the spec §15c "POSIX-style relative paths only" rule mean the schema is the gate for cross-OS shareability — a Windows author who hand-codes `C:\Users\...` paths gets a schema error before share, not after.

### Negative

- **Generated types regenerate.** `crates/runtime-core/src/generated/framework.rs` and `src/types/framework.ts` change on next `cargo xtask regenerate-types` run. CI fails on the schema-change PR until types are committed alongside.
- **Doc surface widens.** Spec §15 (~160 lines) is a new top-level section; future spec readers must orient on it even though no v0.1 code lives there yet.
- **Slight risk of metadata staleness.** Authors might hand-set `compatible_os: ["linux"]` without realizing they're locking out Windows users. Mitigation: M08 Workbench surfaces a warning when narrowing reduces target audience; the Share It module recomputes on export.

### Neutral / future implications

- **Schema becomes the canonical source for share-time inspection.** The Share It module reads it; the headless CLI reads it; the desktop runtime reads it. Single source of truth.
- **WASM tier (spec §15f, v2.0+) reuses the same metadata.** A `wasm_compatible` runtime class can be added as a new enum value when WASM lands, without renaming or restructuring.
- **Cross-version migration (v1.0 → v2.0)** will need additional `share_provenance` fields. Easy to extend additively without breaking v1.0.

## Alternatives Considered

### Alternative A: Defer all share metadata to v1.0 along with the headless CLI

**Rejected because:** every framework built in M03–M07 (potentially many real workflows by the time v1.0 lands) would need a schema migration at v1.0 cutover. The schema bump itself is small (~50 lines of JSON Schema); deferring saves ~half-day of v0.1 work but creates a much larger v1.0 migration ceremony. Worse: any framework shared between v0.1 users at runtime-to-runtime tier wouldn't carry the metadata, so a recipient on v1.0 importing a v0.1-shared framework hits a "metadata missing" prompt every time. Land the schema groundwork now.

### Alternative B: Put share metadata in a separate `share-manifest.json` companion file

**Rejected because:** the metadata describes intrinsic properties of the framework (what secrets it needs, what OSes it supports, what runtime class it requires). These are framework attributes, not share-event attributes. Putting them in a separate file decouples them from the framework — a recipient who unzips and runs `agent-runtime-cli run .` shouldn't have to remember to keep `share-manifest.json` next to `framework.json`. The `share_provenance` sub-object captures the share-event-specific data (when exported, by what); intrinsic metadata stays in `framework.json`.

### Alternative C: Make `runtime_dependency_class` default to `headless_compatible` (more permissive)

**Rejected because:** a framework that uses workbench-only hooks would silently mark itself headless-compatible by default and fail at headless run time with a confusing error. The safer default is `desktop_runtime`; the Share It module flips it to `headless_compatible` only after static analysis confirms no UI-coupled dependencies. Authors who want headless from day one set the field explicitly.

### Alternative D: Encode `compatible_os` as a single enum (`"windows" | "macos" | "linux" | "all"`) instead of an array

**Rejected because:** the array form generalizes to `["windows", "linux"]` (skip macOS, e.g., for tools that need Linux container features but also work on Windows WSL). Single-enum form would force `"all"` or one OS — no way to express "two of three." Array form costs nothing schema-wise and accepts every realistic combination.

## Related

- Spec sections: **§15 Sharing & Distribution** (entire new section); §7 Registry import (runtime-to-runtime tier baseline); §8.security (capability declarations the Share It module also inspects); §13 Privacy & Telemetry (sharing must remain offline-installable, no phone-home)
- Schemas: `schemas/framework.v1.json` (this ADR's target); `schemas/README.md` (versioning policy for in-place minor bumps); `schemas/common.v1.json` (existing `ModelRef`, `Capabilities`, `SemVer` types)
- Prior ADRs: ADR-0001 (ARIA as Archetype — frameworks are the unit of share); ADR-0002 (Tauri + Rust — single static-binary CLI is feasible); ADR-0003 (Engineering Charter — schemas as source of truth, ADR required for schema bumps); ADR-0004 (defer paid code-signing — Sigstore is the share-time signing mechanism)
- MVP doc: `docs/MVP-v0.1.md` §M07 (export emits manifest); §M08 (Share It module forward declaration); CHANGELOG `[Unreleased]` entry
- Issues: none yet

## Notes

The four fields' names use snake_case to match the rest of `framework.v1.json`. The `share_provenance` sub-object's `exported_by` field uses `share-it@1.0.0` SemVer tagging so multiple Share It module versions over the runtime's lifetime can be distinguished in the audit trail.

The "Share It" module itself (spec §15e) is forward-declared in this ADR but not built. It lands in M08 Workbench or as a slim follow-up milestone (M08.5) — sequencing decided when M08 begins. The metadata fields in this ADR are the prerequisite for that module to inspect; they ship in v0.1 even though the module is v1.0+.
