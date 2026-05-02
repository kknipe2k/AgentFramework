# xtask

Project-wide build and maintenance subcommands. Invoked via the
`cargo xtask` alias defined in `.cargo/config.toml`.

```
cargo xtask <subcommand> [flags]
```

This crate is the bridge between `schemas/*.v1.json` (the source of
truth, per `CLAUDE.md` §14) and the `runtime-core` Rust types that
consume them. Adding new project-wide automation (lint helpers, schema
audits, code generators) belongs here rather than in ad-hoc shell
scripts.

## Subcommands

### `regenerate-types`

Regenerates `crates/runtime-core/src/generated/` from the JSON schemas.

| Flag | Effect |
|---|---|
| *(none)* | Regenerate and write to `crates/runtime-core/src/generated/`. |
| `--check` | Regenerate to memory and diff against committed files. Exits non-zero on any drift; lists drifted file basenames. |

**Inputs:** `schemas/{common,framework,skill,tool,agent}.v1.json`.

**Outputs:** `crates/runtime-core/src/generated/{common,framework,skill,tool,agent}.rs`,
each formatted via an inline `rustfmt` subprocess and headed with an
auto-generated marker plus an `#[allow(...)]` block scoped to the lint
quirks of typify-emitted code.

**External `$ref` resolution:** typify does not handle cross-file
references, so the xtask first inlines all `$defs` from any externally
referenced schema before passing the value to typify. Bare file refs
(e.g., `"$ref": "agent.v1.json"`) are rewritten to internal refs
against a derived `$defs` entry.

**When to run:**

- After editing any `schemas/*.v1.json` (with the schema-change ADR
  per `CLAUDE.md` §11). Commit the schema and regenerated `.rs` files
  together.
- CI runs `cargo xtask regenerate-types --check` on every PR to catch
  drift between committed types and current schemas.

## Adding a subcommand

1. Add a variant to `Cmd` in `src/main.rs`.
2. Implement the dispatch arm in `main()`.
3. Document the subcommand in this README under its own `###` heading.
4. Add a doc-test or unit test if the subcommand has non-trivial logic.

## License

Apache-2.0. See repo root `LICENSE` and `NOTICE`.
