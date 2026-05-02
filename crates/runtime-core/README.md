# runtime-core

Shared types for the agent runtime — the wire-format contract every other
crate depends on. Contains the `AgentEvent` taxonomy, drone IPC types
(`DroneEvent`, `DroneCommand`), the `RuntimeError` enum, and Rust
representations of all `schemas/*.v1.json` artifacts.

Per `agent-runtime-spec.md` §12 and `CLAUDE.md` §14, schemas in `schemas/`
are the **single source of truth**. The bulk of this crate's surface area
is generated from those schemas; only the event/error/IPC enums are
hand-curated.

## Type generation pipeline

Files in `src/generated/` (`framework.rs`, `skill.rs`, `tool.rs`,
`agent.rs`, `common.rs`) are emitted by `cargo xtask regenerate-types`
from `schemas/*.v1.json` via the [`typify`](https://crates.io/crates/typify)
crate. The xtask binary lives at `crates/xtask/`; see
`crates/xtask/README.md` for the subcommand reference.

**Do not hand-edit generated files.** CI's drift check (`cargo xtask
regenerate-types --check`) compares the committed output against a
fresh regeneration and fails on any divergence.

To change a generated type:

1. Edit the relevant `schemas/<name>.v1.json` (with an ADR per
   `CLAUDE.md` §11 if it's a §0a Capability Matrix primitive).
2. Run `cargo xtask regenerate-types`.
3. Commit the schema and the regenerated file together in one commit.

The xtask resolves cross-file `$ref` references (typify only handles
internal refs) by inlining the referenced file's `$defs` before passing
the schema to typify. Output is rustfmt-formatted and headed with an
auto-generated marker plus an `#[allow(...)]` block scoped to the
quirks of typify's emitted code.

## Hand-curated types

The remaining files are NOT generated. They form the project's stable
contract and changes follow semver discipline:

- `event.rs` — `AgentEvent` (full variant list per spec §2 + §2a + §2b
  + §3a + §3b + §4a + §4b + §6a + §8.security). Adding a variant is
  semver-minor; removing or restructuring is a breaking change requiring
  an ADR.
- `drone.rs` — `DroneEvent`, `DroneCommand`, and supporting enums
  (`ActivityState`, `StopReason`, `ProcessType`, `AlertLevel`,
  `RevertReason`, `ProcessConfig`). The wire format for main↔drone IPC
  per spec §1d.
- `error.rs` — `RuntimeError` (via `thiserror`). The error type
  surfaced from runtime crates to higher layers.

Both `AgentEvent` and `DroneCommand` use serde's tagged-enum encoding
(`#[serde(tag = "type", rename_all = "snake_case")]`) so JSON wire
frames carry the variant discriminator explicitly. Property tests in
each module verify serialize → deserialize → serialize stability.

## Cross-references

- Event taxonomy: `agent-runtime-spec.md` §2 (`AgentEvent` union), §2a
  (signals), §2b (VDR), §3a (plan), §3b (mode), §4a (verify+rails),
  §4b (gap), §6a (HITL), §8.security (capability events).
- Drone IPC: spec §1 (drone overview), §1c (snapshots/heartbeats),
  §1d (commands and events). Wire-format details:
  `crates/runtime-drone/README.md` → IPC protocol section.
- Schema versioning: `schemas/README.md`. ADRs that touched a schema
  bump its version per the policy there.
- Generated allow-list rationale: `CLAUDE.md` §15 gotcha #24
  (typify-emitted code triggers several pedantic/nursery lints that
  must be tolerated en bloc rather than spot-allowed).

## License

Apache-2.0. See repo root `LICENSE` and `NOTICE`.
