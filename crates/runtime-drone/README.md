# runtime-drone

The drone process for the agent runtime — the survival layer per
`agent-runtime-spec.md` §1. One drone per session; spawned by the main
process; owns the SQLite database; takes append-only snapshots; replies to
commands over a Unix domain socket (Linux/macOS) or Windows named pipe.

This crate is a **safety primitive** per `CLAUDE.md` §5: drone changes
are gated on the per-module coverage baseline below and survive `cargo
deny` / `cargo audit`.

## CLI

```
runtime-drone --session-id <id> --db-path <path> --ipc-socket <path>
```

- `--session-id` — opaque session identifier, also used to scope the
  `sessions` row.
- `--db-path` — path to the SQLite file. Created if missing. Schema (7
  tables) is initialized on first open; pragmas set per spec §1c
  (`journal_mode=WAL`, `synchronous=NORMAL`, `busy_timeout=5000`,
  `foreign_keys=ON`).
- `--ipc-socket` — path to the IPC channel. On Unix this is a Unix
  domain socket file; on Windows the file-name component is used as the
  named-pipe name (`\\.\pipe\<name>`). See `src/ipc.rs` module docs for
  the Windows configuration details (security descriptor, ServerOptions).

Heartbeats fire every 5 seconds while the loop runs. SIGTERM / SIGINT
(Unix) and CTRL_BREAK / CTRL_C (Windows) trigger a best-effort emergency
snapshot before exit per spec §1.

## IPC protocol

Newline-delimited JSON via `tokio_util::codec::Framed` with `LinesCodec`.
The drone reads `DroneCommand` frames from the read half and writes
`DroneEvent` frames on the write half. Both types are defined in
`runtime_core::drone`. Malformed JSON does not kill the server — it
emits a `DroneEvent::Alert` and continues on the next frame.

### Commands (main → drone)

| Command | Behavior |
|---|---|
| `SnapshotNow` | Insert a row in `snapshots` with SHA-256 `state_hash`; emit `SnapshotWritten` |
| `GracefulShutdown` | Exit the command loop within `timeout_ms`; main aborts the IPC server next |
| `RevertToSnapshot` | Look up the snapshot id; emit `SnapshotWritten` if present, `Alert{Critical}` otherwise |
| `SpawnProcess`, `StopProcess`, `SetActivityTimeout` | Emit `Alert{Warn}` "not yet implemented" — these land in M05+ |

### Example frames

```json
{"type":"snapshot_now","reason":"task_boundary","state_json":{}}
{"type":"graceful_shutdown","timeout_ms":5000}
{"type":"revert_to_snapshot","snapshot_id":"abc","reason":"hook_rollback"}
```

```json
{"type":"heartbeat","status":"active","timestamp":1714665600}
{"type":"snapshot_written","snapshot_id":"01J...","session_id":"smoke","reason":"task_boundary","timestamp":1714665600}
{"type":"alert","level":"warn","message":"spawn_process not yet implemented"}
```

## SQLite schema

7 tables initialized on first DB open per spec §1c + §11:

- `sessions` — session lifecycle (`active | suspended | complete |
  crashed | recovered | budget_exceeded`).
- `snapshots` — append-only; SHA-256 `state_hash`; reason; timestamp.
- `signals` — typed signal stream (§2a) + 3 indexes for replay.
- `heartbeats` — drone liveness rows + 1 index on `session_id`.
- `vdr` — Verifiable Data Records (§2b); `signal_ids` and
  `context_type` are inlined here, not via a separate ALTER.
- `token_usage` — per-call token accounting.
- `skills` — installed skill manifest cache.

Pragmas applied in order on `db::init`:

```
PRAGMA journal_mode = WAL;       -- enable WAL
PRAGMA synchronous = NORMAL;     -- NORMAL safe + fast under WAL
PRAGMA busy_timeout = 5000;      -- 5s before SQLITE_BUSY
PRAGMA foreign_keys = ON;        -- enforce FK constraints
```

The journal-mode pragma must be first; reordering is an FAQ trap (see
`CLAUDE.md` §15 gotchas).

## Manual smoke (Linux/macOS)

```bash
mkdir -p /tmp/drone-smoke
cargo run --bin runtime-drone -- \
    --session-id smoke \
    --db-path /tmp/drone-smoke/d.sqlite \
    --ipc-socket /tmp/drone-smoke/d.sock &
DRONE_PID=$!; sleep 6; kill -TERM $DRONE_PID; wait $DRONE_PID
sqlite3 /tmp/drone-smoke/d.sqlite "SELECT event_type FROM snapshots;"
# Expected: includes 'sigterm'
```

The integration test at `tests/integration.rs` automates this lifecycle
on Unix; Windows CI exercises the in-process unit tests only (the
SIGTERM lifecycle is `#[cfg(unix)]`).

## Platform-specific notes

- **Unix:** IPC backed by `tokio::net::UnixListener`. The socket file is
  created at the path passed via `--ipc-socket` and unlinked on graceful
  shutdown.
- **Windows:** IPC backed by `tokio::net::windows::named_pipe::ServerOptions`.
  The name is taken from the `--ipc-socket` argument's file name (the
  parent directory portion is dropped) and prefixed with `\\.\pipe\`.
  The first instance must be created with `.first_pipe_instance(true)`;
  subsequent instances reuse the same name. Default security descriptor
  is process-owner only (no ACL widening at v0.1).
- **Signals:** `tokio::signal::ctrl_c` on all platforms; on Unix the
  drone additionally listens for SIGTERM via
  `tokio::signal::unix::signal(SignalKind::terminate())`.

## Coverage requirement

The drone is a safety primitive per `CLAUDE.md` §5. Two gates apply:

```bash
# Gate 1: workspace ≥80%, generated code + binary stubs excluded.
cargo llvm-cov --workspace \
    --ignore-filename-regex "src.main\.rs|generated" \
    --fail-under-lines 80

# Gate 2: drone safety primitive ≥95%, additionally excluding the OS-
# signal orchestrators (lib.rs and shutdown.rs are thin wrappers around
# testable `_inner`/`_with` variants and are exercised end-to-end by
# the Unix subprocess integration test in tests/integration.rs).
cargo llvm-cov --package runtime-drone \
    --ignore-filename-regex "src.main\.rs|generated|src.lib\.rs|src.shutdown\.rs" \
    --fail-under-lines 95
```

**Per-module baseline** (M01 Stage C measured; subsequent milestones
must not regress without a retro entry):

| Module | Line | Region |
|---|---|---|
| `snapshot.rs` | 100.00% | 97.14% |
| `db.rs` | 98.82% | 96.08% |
| `heartbeat.rs` | 98.59% | 96.45% |
| `command_handler.rs` | 97.94% | 98.01% |
| `ipc.rs` | 84.70% | 87.23% |

`db.rs`, `snapshot.rs`, `heartbeat.rs`, and `command_handler.rs` are
expected to hit ~100% individually. `ipc.rs`'s 84.70% reflects the
platform-cfg accept-loop variants (only the active OS's variant is
exercised per run) plus the broadcast-lagged path; cross-OS CI lifts
it for Linux/macOS but Windows-only runs see the lower number.

Rationale for excluding `lib.rs` + `shutdown.rs`: both are OS-signal
entry points (real `SIGTERM`/`SIGINT`/`CTRL_BREAK`/`CTRL_C` listeners)
that cannot be unit-tested cross-platform without firing real OS
signals. The subprocess integration test on Linux/macOS exercises them
end-to-end. See
`docs/build-prompts/retrospectives/M01.C-retrospective.md` and
`docs/build-prompts/M01-foundation.md` Stage D §D.3 "Coverage gate
semantics" for the decision history.

## License

Apache-2.0. See repo root `LICENSE` and `NOTICE`.
