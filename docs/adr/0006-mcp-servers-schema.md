# ADR-0006: `mcp_servers` table schema (richer than spec §10)

**Status:** Accepted
**Date:** 2026-05-04
**Deciders:** @kknipe2k
**Tags:** schema, mcp, persistence, capability, M02-M06

## Context

Spec §10 (Persistence Layer) lists a 7-field shape for the `mcp_servers` SQLite table — `id`, `name`, `url`, `auth_key_ref`, `added_at`, `last_connected`, `status`. That shape was authored before any MCP client implementation existed; it captures the spec-level intent ("MCP servers stored with id, name, URL, auth ref, lifecycle timestamps, status") but does not enumerate the operational state an MCP client needs to manage at runtime.

M02 Stage A landed the table as a carry-forward closure from M01's gap-analysis. At authoring time we had two paths:

1. **Match the 7-field spec shape** — minimal schema, ALTER TABLE migrations as M06 (MCP basic) and M06+ features need fields. Each migration is a schema version bump and a code-side DB version check.
2. **Ship a richer shape now** — design for the M06–M11 MCP feature set (multiple transports, OAuth refresh state, capability discovery cache, scoped servers from §M0d, retry/timeout policy) and migrate fields once at v0.1 schema.

We chose (2). The shipped table at `crates/runtime-drone/src/db.rs::init_schema` has 22 fields. The decision is more architectural than the field count suggests — it locks the MCP shape (which transports the runtime supports, how OAuth refresh state persists, how capability discovery caches) ahead of the M06 client implementation. CLAUDE.md §11 requires an ADR for schema changes; the M02 gap-analysis flagged this as the divergence-from-spec to reconcile via dedicated ADR rather than docs(spec) PR elaboration.

## Decision

We adopt the 22-field `mcp_servers` schema as shipped in M02 Stage A and document the divergence-from-spec rationale here.

### The 22 fields

```sql
CREATE TABLE IF NOT EXISTS mcp_servers (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Identity
    name                        TEXT NOT NULL UNIQUE,

    -- Transport (locks the v0.1 + v1.0 transport set)
    transport                   TEXT NOT NULL
                                CHECK (transport IN ('stdio', 'http', 'sse', 'streamable_http')),

    -- stdio-transport columns (mutually exclusive with url+headers_json)
    command                     TEXT,
    args_json                   TEXT,
    env_json                    TEXT,

    -- Remote-transport columns (mutually exclusive with command+args_json+env_json)
    url                         TEXT,
    headers_json                TEXT,

    -- Auth (keychain-ref only; never literal secrets)
    auth_kind                   TEXT
                                CHECK (auth_kind IN ('none', 'bearer', 'oauth', 'custom') OR auth_kind IS NULL),
    auth_token_ref              TEXT,
    oauth_state_json            TEXT,

    -- Connection lifecycle
    status                      TEXT NOT NULL DEFAULT 'configured'
                                CHECK (status IN ('configured', 'connected', 'errored', 'disabled', 'failed')),
    last_error                  TEXT,
    last_connected_at           INTEGER,
    retry_count                 INTEGER NOT NULL DEFAULT 0,

    -- Timeout policy
    startup_timeout_ms          INTEGER NOT NULL DEFAULT 10000,
    tool_timeout_ms             INTEGER NOT NULL DEFAULT 60000,

    -- Scope (per §M0d release scope — user / project / plugin / local)
    enabled                     BOOLEAN NOT NULL DEFAULT 1,
    scope                       TEXT NOT NULL DEFAULT 'user'
                                CHECK (scope IN ('user', 'project', 'plugin', 'local')),
    plugin_id                   TEXT,

    -- Capability discovery cache
    discovered_tool_count       INTEGER,
    last_capabilities_refresh   INTEGER,

    -- Audit timestamps
    added_at                    INTEGER NOT NULL,
    updated_at                  INTEGER NOT NULL
);
```

Plus a stdio-vs-remote mutual-exclusion CHECK constraint (in the table-level CHECK) that ensures `command`/`args_json`/`env_json` are populated for stdio transports and `url`/`headers_json` for remote transports — forbidding the half-configured server.

### Per-field rationale (delta from spec §10)

| Field | Spec §10 | Why richer |
|---|---|---|
| `transport` | not present | Locks v0.1 transport set: stdio (local subprocess, MCP standard), http (REST per MCP spec), sse (server-sent events per MCP spec), streamable_http (chunked responses; stretch). `CHECK` constraint prevents typo'd transports from silently entering rotation. |
| `command` / `args_json` / `env_json` | not present | stdio-transport server config: subprocess command, arg vector (JSON-encoded for ordered preservation), environment overrides. JSON columns avoid a separate child table for variadic args/env. |
| `url` / `headers_json` | `url` present; headers not | Remote-transport server config: target URL + per-server headers (auth, content-type overrides). Headers are JSON to avoid a separate child table. |
| `auth_kind` | not present | Discriminator for the auth flow: `none` for local subprocess, `bearer` for static API tokens, `oauth` for OAuth flows with refresh, `custom` for non-standard MCP server auth. CLAUDE.md §13 zero-telemetry compatible — no auth values stored, only the kind + a keychain ref. |
| `auth_token_ref` | `auth_key_ref` (renamed) | Same purpose; renamed for clarity (it's a token reference, not a key reference for symmetric crypto). |
| `oauth_state_json` | not present | Persists OAuth refresh-token state across sessions (refresh_token + expires_at + scopes). Critical for long-lived MCP servers; without it, every session relogins. JSON column lets the field hold provider-specific state without locking the shape. |
| `status` | present (less granular) | Spec status was opaque string; richer enum: `configured` (added but not connected), `connected` (live), `errored` (transient — retry pending), `disabled` (user-disabled), `failed` (terminal — manual reset). Per §11 reconciliation. |
| `last_error` | not present | Surfaces the most-recent transport / handshake / auth error in the Settings UI without joining against an audit log table. Truncated to last 1024 chars. |
| `retry_count` | not present | Backoff state — main process consumes to compute next retry interval. Resets on `connected`. |
| `startup_timeout_ms` / `tool_timeout_ms` | not present | Per-server policy: how long to wait for the initial handshake vs how long any tool invocation can take. Defaults are the §11 reconciliation defaults; per-server override allows slow MCP servers (large model loaders) to coexist with fast ones. |
| `enabled` | not present | User-controlled toggle (Settings panel). Disabled servers don't auto-connect on session start. Distinct from `status='disabled'` which is the runtime-driven version. |
| `scope` | not present | Per spec §M0d release scope: `user` (all sessions), `project` (this framework only), `plugin` (auto-installed by a plugin), `local` (this session only). The Settings UI filter uses this; the M07 registry import path defaults to `user` scope. |
| `plugin_id` | not present | When `scope='plugin'`, names the installing plugin. Foreign key to a future `plugins` table (M07+). NULL otherwise. |
| `discovered_tool_count` | not present | Capability discovery cache — number of tools the server exposed at last handshake. Used by the Settings UI to display "MCP server: N tools" without re-querying. |
| `last_capabilities_refresh` | not present | Timestamp of the last capability discovery handshake. Capability cache TTL (default 24h, framework-overridable) drives re-discovery. |
| `updated_at` | not present (only `added_at`) | Audit field for any column-modifying operation. Pairs with `added_at`. |

### Mutual-exclusion CHECK

```sql
CHECK (
    (transport = 'stdio'           AND command IS NOT NULL AND url IS NULL) OR
    (transport IN ('http', 'sse', 'streamable_http') AND url IS NOT NULL AND command IS NULL)
)
```

Schema-level enforcement of the stdio-vs-remote disjoint shape — runtime can't accidentally store a half-configured server.

### Migration policy

The 22-field shape is `mcp_servers.v1`. Future field additions follow `schemas/README.md` versioning policy: additive optional fields are minor in-place bumps, breaking changes (rename, remove, narrow constraint) trigger a new ADR + new schema major version. v0.1 ships at `mcp_servers.v1`; M06 (MCP basic) consumes this shape directly.

## Consequences

### Positive

- **No mid-implementation migration.** M06 reads the table as-shipped; no `ALTER TABLE ADD COLUMN` storm during MCP feature work. The schema is the contract M06 implements against.
- **Transport-set locked.** `stdio | http | sse | streamable_http` is the v0.1 + v1.0 supported set. Adding a fifth transport (e.g., gRPC) is an explicit ADR + CHECK-constraint update, not an implicit string field.
- **OAuth refresh state persists across sessions.** Without `oauth_state_json`, every session would relogin to OAuth-protected MCP servers. This was identified during M02 design review as a v0.1 user-facing requirement (§14 first-run UX implies "set up MCP server once, use it across sessions").
- **Capability cache is first-class.** `discovered_tool_count` + `last_capabilities_refresh` enable the Settings UI to render "MCP server: 12 tools (refreshed 2h ago)" without per-render handshakes. Cache invalidation is a wall-clock check, not a per-query one.
- **Stdio-vs-remote mutual exclusion is enforced at the SQL layer.** Half-configured servers can't enter the table, full stop. Runtime code paths don't need to repeat the validation.

### Negative

- **Spec §10 disagrees with the shipped schema** (7 fields vs 22). This ADR is the bridge — readers find §10 first, see the 7-field shape, and follow the ADR cross-reference for the 22-field reality. The drift is documented; it's not unbounded.
- **Schema migration path is heavier.** Renaming `auth_key_ref` → `auth_token_ref` is a v1 → v2 migration in this scheme rather than a transparent column rename. Mitigated by the v0.1 timing — no v0.1 user has data in the table yet.
- **22-field tables are visually larger** than 7-field. Code review of any change touching this table is heavier. Mitigated by the per-field rationale table above; reviewers can scope changes to the affected concern (auth, transport, lifecycle, scope, cache).

### Neutral / future implications

- **A `mcp_servers.v2` is foreseeable** if M06 implementation discovers we need fields not enumerated here (e.g., per-server feature flags, capability subsets, per-server cache TTL overrides). The v1 ADR + v2 ADR pair is the pattern.
- **The `plugin_id` foreign key is dangling** in v0.1 — no `plugins` table exists yet. M07 (registry import) will land it. NULL is the v0.1 default; the foreign key is unenforced until the parent table exists.
- **The capability cache TTL** (default 24h) is currently hardcoded; M03+ frameworks may want per-framework override. That's a framework.v1.x bump, not an mcp_servers schema change.

## Alternatives Considered

### Alternative A: Match the 7-field spec §10 shape exactly

**Rejected because:** every M06 feature (transport set, OAuth refresh, capability cache, scoped servers, timeout policy, retry state) would require an `ALTER TABLE ADD COLUMN` migration. By M06 Stage A start the table would have ~12 migrations queued; each requires a DB version bump and a code-side compatibility check. v0.1 ships before any user has data in this table, so the once-at-v0.1 design wins.

### Alternative B: Split into multiple tables (one per concern: `mcp_servers`, `mcp_auth`, `mcp_capabilities`, `mcp_lifecycle`)

**Rejected because:** every MCP-server query joins across at least two of these (rendering the Settings UI requires `mcp_servers` + `mcp_auth` + `mcp_capabilities`). The table is small (≤100 rows expected per user), and SQLite doesn't penalize wide tables. Splitting introduces three foreign keys and three indexes for negligible normalization win.

### Alternative C: Use a single JSON column for the entire server config

**Rejected because:** the Settings UI filter (`WHERE enabled = 1 AND scope = 'user'`) needs SQL-queryable columns. Putting transport, status, scope in a JSON blob means full-table-scan filters or SQLite's slow JSON1 extension. Spec-level fields stay columnar; per-transport variadic fields (env, args, headers, oauth_state) stay JSON.

### Alternative D: Defer the schema design to M06 Stage A

**Rejected because:** M02 gap-analysis carry-forward required closing the M01 🟡 "mcp_servers — add now or document deferral" item. Documenting deferral was option (b) in the M01 retro; we explicitly chose option (a) (add now) at M02 authoring because the schema crystallized the M06 design constraints early. The ADR locks that choice.

## Related

- Spec sections: **§10 Persistence Layer** (the 7-field shape this ADR diverges from); **§11 Reconciliation & Degraded Modes** (uses `status` for MCP-server-down detection); **§14 First-run** (Settings panel surfaces this table); **§M0d Release Scope** (`scope` column maps to the user/project/plugin/local tiers); future **Phase 5: MCP Manager** (M06 — primary consumer)
- Schemas: `schemas/README.md` versioning policy (additive in-place vs new-major-version); SQLite schema in `crates/runtime-drone/src/db.rs::init_schema`
- Prior ADRs: ADR-0001 (ARIA as Archetype — frameworks don't dictate MCP-server config, that's runtime state); ADR-0003 (Engineering Charter — schemas are source of truth, ADR required for schema bumps); ADR-0005 (Headless-share schema groundwork — `mcp_servers.scope` interacts with the share-it module's per-OS bundle scoping)
- M02 references: `docs/build-prompts/M02-event-pipeline.md` Stage A (table authored); `docs/gap-analysis.md` M02 entry (this ADR's flagged-for-creation item); `crates/runtime-drone/src/db.rs::init_schema` (the shipped schema this ADR locks)
- Issues: none yet

## Notes

The 22-field count is not the design goal — the goal is a complete operational shape for the M06 MCP client. Future MCP servers (e.g., a local Ollama-compatible MCP server, a corporate-proxied MCP gateway) should fit in this shape without ALTER TABLE; if they don't, that's an `mcp_servers.v2` ADR.

**Target ship:** This ADR was filed during the post-M02 docs(spec) PR (post-M02 housekeeping), per the M02 gap-analysis Fix Backlog deadline ("Target: file before M06 Stage A"). The schema itself shipped in M02 Stage A (`crates/runtime-drone/src/db.rs::init_schema`); this ADR's role is the audit-trail justification for the divergence from spec §10.
