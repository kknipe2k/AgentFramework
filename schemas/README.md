# Schemas

JSON Schema Draft 2020-12 documents that define the shapes of artifacts the runtime consumes.

These schemas are the **source of truth**. Per spec §12 Engineering Charter, Rust types and TypeScript types are generated from these schemas (via `typify` and `json-schema-to-typescript`) — hand-written types are forbidden to prevent drift.

## Files

| Schema | Defines | Spec section |
|---|---|---|
| `common.v1.json`    | Shared types: `Capabilities`, `Provenance`, `HookRef`, `Hook`, `JSONLogicExpression`, `ModelRef`, `SemVer`, `FileGlobList`, `Tier`, `AlertLevel` | §0b, §4a, §8.security |
| `skill.v1.json`     | Canonical `skill.md` frontmatter | §0b |
| `tool.v1.json`      | Canonical `tool.md` frontmatter | §0b, §8a |
| `agent.v1.json`     | Agent definition (in `agents[]` or standalone `agent.md`) | §0b |
| `framework.v1.json` | Top-level `framework.json` | §6 |

## Versioning

Schemas use independent SemVer per artifact:

- **Major bump** (`v1` → `v2`) — breaking change to required fields, removed properties, or stricter constraints. Old framework JSON files referencing `v1` continue to validate against `v1`; `v2` is opt-in.
- **Minor bump** within a major version is in-place (the `$id` does not change). New optional properties can be added without bumping the major.
- **Patch bump** is for clarification or example fixes that do not change validation behavior.

Each major version is a separate file (`framework.v1.json`, `framework.v2.json`, etc.) so consumers can pin. The runtime ships with all supported major versions and dispatches by the `$schema` URL in the consumed document.

## Generated Types

```
schemas/framework.v1.json  ──┬──> crates/runtime-core/src/framework.rs   (typify)
                             └──> src/types/framework.ts                 (json-schema-to-typescript)

schemas/common.v1.json     ──┬──> crates/runtime-core/src/common.rs
                             └──> src/types/common.ts
```

CI runs the generators and fails if the committed types differ from what would be generated. To regenerate:

```bash
just regenerate-types       # or: cargo xtask regenerate-types
```

## Validation in the Runtime

The Phase 6 framework loader validates every loaded `framework.json` against `framework.v1.json` before instantiating. The `$schema` field in the document selects which major version to validate against. Validation failures are surfaced as user-facing errors with the JSON Pointer of the offending field.

Skill / tool / agent files are validated similarly when imported — by the registry, by the generator (Phase 8 §8.security L3), and lazily on first load.

## URLs

The `$id` fields use `https://schemas.aria-runtime.dev/` URLs. Until that domain is provisioned, the runtime resolves all `$id` references against the local `schemas/` directory. Once the project ships, the domain serves these files for use by external tooling (editor schema completion, third-party validators).

## Adding a New Schema

1. Create `schemas/<name>.v1.json` following the same structural conventions as the existing files.
2. Add a row to the table above.
3. Add type generation entries to `xtask` / `Cargo.toml` / `package.json`.
4. Bump the entry in the spec's §0a Capability Matrix or §6 if the schema unlocks new capabilities.
5. Open an ADR per §12 — adding a runtime-consumed schema is a primitive change.

## Validating a File Locally

```bash
# With ajv (npm):
npx ajv validate -s schemas/framework.v1.json -d examples/aria/framework.json --spec=draft2020 --all-errors

# With jsonschema (Python):
pip install jsonschema && python -c "
import json, jsonschema
schema = json.load(open('schemas/framework.v1.json'))
data = json.load(open('examples/aria/framework.json'))
jsonschema.Draft202012Validator(schema).validate(data)
print('valid')
"
```

CI runs both validators against every `examples/*/framework.json` on every PR.
