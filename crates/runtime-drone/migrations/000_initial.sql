-- Migration 000 — initial schema (preserves M01 baseline).
--
-- Verbatim move of the M01 init_schema content + the 8th mcp_servers
-- table added in M02. Idempotent: every CREATE statement uses IF NOT
-- EXISTS so the migration runner's `_migrations` version-tracking row
-- is the authoritative "applied" signal, not statement success.
--
-- Spec §1c (SQLite Concurrency) + §11 (Persistence Layer DDL) +
-- spec §11:2435-2444 + ADR-0006 (mcp_servers schema).

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  framework_name TEXT,
  framework_version TEXT,
  model TEXT,
  started_at INTEGER,
  last_active INTEGER,
  status TEXT,
  mode TEXT
);

CREATE TABLE IF NOT EXISTS snapshots (
  id TEXT PRIMARY KEY,
  session_id TEXT,
  timestamp INTEGER,
  event_type TEXT,
  state_json TEXT,
  state_hash TEXT,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS signals (
  id TEXT PRIMARY KEY,
  session_id TEXT,
  type TEXT,
  event TEXT,
  timestamp TEXT,
  duration_ms INTEGER,
  payload_json TEXT,
  pre_signal_id TEXT,
  parent_signal_id TEXT,
  retry_of TEXT,
  context_type TEXT,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);
CREATE INDEX IF NOT EXISTS idx_signals_session_time ON signals(session_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_signals_type ON signals(type);
CREATE INDEX IF NOT EXISTS idx_signals_correlation ON signals(pre_signal_id, parent_signal_id, retry_of);

CREATE TABLE IF NOT EXISTS heartbeats (
  id TEXT PRIMARY KEY,
  session_id TEXT,
  timestamp INTEGER,
  status TEXT,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);
CREATE INDEX IF NOT EXISTS idx_heartbeats_session_time ON heartbeats(session_id, timestamp);

CREATE TABLE IF NOT EXISTS vdr (
  id TEXT PRIMARY KEY,
  session_id TEXT,
  agent_id TEXT,
  timestamp INTEGER,
  decision TEXT,
  rationale TEXT,
  tool_invoked TEXT,
  tool_input_json TEXT,
  tool_output_json TEXT,
  token_cost_usd REAL,
  outcome TEXT,
  snapshot_id TEXT,
  signal_ids TEXT,
  context_type TEXT,
  contributing_signal_id TEXT,
  FOREIGN KEY (session_id) REFERENCES sessions(id),
  FOREIGN KEY (snapshot_id) REFERENCES snapshots(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_vdr_contributing_signal
  ON vdr(contributing_signal_id);

CREATE TABLE IF NOT EXISTS token_usage (
  id TEXT PRIMARY KEY,
  session_id TEXT,
  agent_id TEXT,
  timestamp INTEGER,
  model TEXT,
  input_tokens INTEGER,
  output_tokens INTEGER,
  cost_usd REAL
);

CREATE TABLE IF NOT EXISTS skills (
  id TEXT PRIMARY KEY,
  name TEXT,
  version TEXT,
  source_url TEXT,
  installed_at INTEGER,
  validated INTEGER,
  skill_md TEXT
);

CREATE TABLE IF NOT EXISTS mcp_servers (
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
    auth_token_ref              TEXT,
    oauth_state_json            TEXT,

    status                      TEXT NOT NULL DEFAULT 'configured'
                                CHECK (status IN ('configured', 'connected', 'errored', 'disabled', 'failed')),
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

    CHECK (
        (transport = 'stdio' AND command IS NOT NULL AND url IS NULL)
        OR
        (transport IN ('http', 'sse', 'streamable_http') AND url IS NOT NULL AND command IS NULL)
    )
);
CREATE INDEX IF NOT EXISTS idx_mcp_servers_status  ON mcp_servers(status);
CREATE INDEX IF NOT EXISTS idx_mcp_servers_enabled ON mcp_servers(enabled);
CREATE INDEX IF NOT EXISTS idx_mcp_servers_scope   ON mcp_servers(scope);
