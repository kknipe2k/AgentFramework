# runtime-core

Shared types for the agent runtime.

## Type generation pipeline

Types in `src/generated/` are emitted by `cargo xtask regenerate-types` from
`schemas/*.v1.json`. **Do not hand-edit them** — CI's drift check (`cargo
xtask regenerate-types --check`) fails if committed types differ from
regenerated.

To change a type:

1. Edit `schemas/<name>.v1.json` (with ADR per `CLAUDE.md` §11 if it's a
   primitive change).
2. Run `cargo xtask regenerate-types`.
3. Commit the schema + generated output together.

## Hand-curated types

`event.rs` (`AgentEvent`), `drone.rs` (`DroneEvent`, `DroneCommand`), and
`error.rs` (`RuntimeError`) are NOT generated. They're the contract every
later milestone evolves. Adding a variant is semver-minor; removing or
restructuring is a breaking change requiring an ADR.

## License

Apache-2.0. See repo root `LICENSE` and `NOTICE`.
