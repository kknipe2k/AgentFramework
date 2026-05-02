# runtime-drone

The drone process for the agent runtime — the survival layer per
`agent-runtime-spec.md` §1. One drone per session; spawned by the main
process; owns the SQLite database; takes append-only snapshots; replies to
commands over a Unix domain socket (Linux/macOS) or Windows named pipe.

This crate is a **safety primitive** per `CLAUDE.md` §5: drone changes are
gated on high line coverage and survive `cargo deny` / `cargo audit`.

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
- `--ipc-socket` — path to the IPC channel. On Unix this is a Unix domain
  socket file; on Windows the file-name component is used as the named-pipe
  name (`\\.\pipe\<name>`). See `src/ipc.rs` module docs for the Windows
  configuration details.

## IPC protocol

Newline-delimited JSON. The drone reads `DroneCommand` from the read half
and writes `DroneEvent` on the write half. Both types are defined in
`runtime-core::drone`. Malformed JSON does not kill the server — it emits a
`DroneEvent::Alert` and continues.

Five command variants are dispatched by the drone today:

| Command | Behavior |
|---|---|
| `SnapshotNow` | Insert a row in `snapshots` with SHA-256 `state_hash`; emit `SnapshotWritten` |
| `GracefulShutdown` | Exit the command loop within `timeout_ms`; main aborts the IPC server next |
| `RevertToSnapshot` | Look up the snapshot id; emit `SnapshotWritten` if present, `Alert{Critical}` otherwise |
| `SpawnProcess`, `StopProcess`, `SetActivityTimeout` | Emit `Alert{Warn}` "not yet implemented" — these land in M05+ |

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

## Coverage

Run with:

```bash
cargo llvm-cov --package runtime-drone \
    --ignore-filename-regex "src.main\.rs|generated"
```

The OS-signal entry points (`lib::run`, `lib::shutdown_signal_future`, the
platform-specific `ipc::accept_loop`) are exercised by the subprocess-spawn
integration test in `tests/integration.rs` (Unix only at v0.1) — coverage
of those lines on Windows runs depends on the test binary being able to
reach the production paths, which is structurally limited. See M01.C
retrospective for the detailed holdout discussion.

## License

Apache-2.0. See repo root.
