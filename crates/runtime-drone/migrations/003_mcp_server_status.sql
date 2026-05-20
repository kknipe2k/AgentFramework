-- Migration 003 — mcp_servers.status vocabulary aligned to
-- mcp.v1.json::McpServerStatus (M07.D1 CQ-6).
--
-- 000_initial.sql pinned the status CHECK to the pre-schema vocabulary
-- ('configured','connected','errored','disabled','failed'). The
-- M06.B-shipped JSON-Schema enum McpServerStatus is
-- ('connected','disconnected','health_pending','error'). CLAUDE.md §14
-- (schemas are source of truth): CQ-6 makes the Rust side use the
-- generated enum, so the DB CHECK mirror must be realigned or every
-- write of a schema-valid value ('disconnected', 'health_pending',
-- 'error') fails the stale CHECK. This is a DB-mirror realignment to an
-- already-accepted schema — no schemas/*.json change, no ADR trigger.
--
-- SQLite cannot ALTER/DROP a CHECK constraint, so the table is rebuilt
-- per the documented procedure (sqlite.org/lang_altertable.html
-- #otheralter). No table references mcp_servers via FK (grep-verified),
-- so the DROP is safe under `PRAGMA foreign_keys = ON`. Idempotent via
-- the `_migrations` version-skip (the runner wraps this in BEGIN/COMMIT).
--
-- Column set mirrors the live shape AFTER migration 002 (auth_token_ref
-- renamed → auth_secret_ref; cwd appended). Pre-existing rows' status is
-- remapped: connected→connected, errored/failed→error, everything else
-- (configured/disabled/unknown) → disconnected.

CREATE TABLE mcp_servers_new (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,

    name                        TEXT NOT NULL UNIQUE,
    transport                   TEXT NOT NULL
                                CHECK (transport IN ('stdio', 'http', 'sse', 'streamable_http')),

    command                     TEXT,
    args_json                   TEXT,
    env_json                    TEXT,

    url                         TEXT,
    headers_json                TEXT,

    auth_kind                   TEXT
                                CHECK (auth_kind IN ('none', 'bearer', 'oauth', 'custom') OR auth_kind IS NULL),
    auth_secret_ref             TEXT,
    oauth_state_json            TEXT,

    status                      TEXT NOT NULL DEFAULT 'disconnected'
                                CHECK (status IN ('connected', 'disconnected', 'health_pending', 'error')),
    last_error                  TEXT,
    last_connected_at           INTEGER,
    retry_count                 INTEGER NOT NULL DEFAULT 0,

    startup_timeout_ms          INTEGER NOT NULL DEFAULT 10000,
    tool_timeout_ms             INTEGER NOT NULL DEFAULT 60000,

    enabled                     BOOLEAN NOT NULL DEFAULT 1,
    scope                       TEXT NOT NULL DEFAULT 'user'
                                CHECK (scope IN ('user', 'project', 'plugin', 'local')),
    plugin_id                   TEXT,

    discovered_tool_count       INTEGER,
    last_capabilities_refresh   INTEGER,

    added_at                    INTEGER NOT NULL,
    updated_at                  INTEGER NOT NULL,

    cwd                         TEXT,

    CHECK (
        (transport = 'stdio' AND command IS NOT NULL AND url IS NULL)
        OR
        (transport IN ('http', 'sse', 'streamable_http') AND url IS NOT NULL AND command IS NULL)
    )
);

INSERT INTO mcp_servers_new (
    id, name, transport, command, args_json, env_json, url, headers_json,
    auth_kind, auth_secret_ref, oauth_state_json, status, last_error,
    last_connected_at, retry_count, startup_timeout_ms, tool_timeout_ms,
    enabled, scope, plugin_id, discovered_tool_count,
    last_capabilities_refresh, added_at, updated_at, cwd
)
SELECT
    id, name, transport, command, args_json, env_json, url, headers_json,
    auth_kind, auth_secret_ref, oauth_state_json,
    CASE status
        WHEN 'connected' THEN 'connected'
        WHEN 'errored'   THEN 'error'
        WHEN 'failed'    THEN 'error'
        ELSE 'disconnected'
    END,
    last_error, last_connected_at, retry_count, startup_timeout_ms,
    tool_timeout_ms, enabled, scope, plugin_id, discovered_tool_count,
    last_capabilities_refresh, added_at, updated_at, cwd
FROM mcp_servers;

DROP TABLE mcp_servers;
ALTER TABLE mcp_servers_new RENAME TO mcp_servers;

CREATE INDEX IF NOT EXISTS idx_mcp_servers_status  ON mcp_servers(status);
CREATE INDEX IF NOT EXISTS idx_mcp_servers_enabled ON mcp_servers(enabled);
CREATE INDEX IF NOT EXISTS idx_mcp_servers_scope   ON mcp_servers(scope);
