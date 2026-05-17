-- Migration 002 — mcp_servers schema alignment with mcp.v1.json (M06 Stage C).
--
-- Two schema-driven changes per CLAUDE.md §14 (schemas as source of truth):
--   1. RENAME `auth_token_ref` → `auth_secret_ref` so the SQL column matches
--      the JSON-Schema-defined McpServerConfig.auth_secret_ref field name.
--      Registry insert/select queries (Stage C `client/registry.rs`) carry
--      zero translation layer.
--   2. ADD `cwd TEXT` to round-trip the McpTransport stdio variant's
--      optional working directory. Existing schema (000_initial.sql) lacked
--      this column; reused last_connected_at as the "last known alive"
--      signal so no last_health_check column is required.
--
-- Idempotency: this migration is registered in the `_migrations` table by
-- the M04 migration runner (crates/runtime-drone/src/db.rs::run_migrations).
-- The runner skips already-applied versions; SQLite's RENAME COLUMN +
-- ADD COLUMN are NOT idempotent in their raw form, but the runner's
-- version-skip semantics provide the idempotency guarantee.

ALTER TABLE mcp_servers RENAME COLUMN auth_token_ref TO auth_secret_ref;
ALTER TABLE mcp_servers ADD COLUMN cwd TEXT;
