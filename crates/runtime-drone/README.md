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

The drone is a safety primitive per `CLAUDE.md` §5. Two gates apply:

```bash
# Gate 1: workspace ≥80%, generated code + binary stubs excluded.
cargo llvm-cov --workspace \
    --ignore-filename-regex "src.main\.rs|generated" \
    --fail-under-lines 80

# Gate 2: drone safety primitive ≥95%, additionally excluding the OS-
# signal orchestrators (lib.rs and shutdown.rs are thin wrappers around
# testable `_inner`/`_with` variants and are exercised end-to-end by the
# Unix subprocess integration test in tests/integration.rs).
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

`ipc.rs`'s 84.70% reflects the platform-cfg accept-loop variants (only
the active OS's variant is exercised per run) plus the broadcast-lagged
path; cross-OS CI lifts it for Linux/macOS but Windows-only runs see
the lower number.

Rationale for excluding `lib.rs` + `shutdown.rs`: both are OS-signal
entry points (real `SIGTERM`/`SIGINT`/`CTRL_BREAK`/`CTRL_C` listeners)
that cannot be unit-tested cross-platform without firing real OS signals.
The subprocess integration test on Linux/macOS exercises them
end-to-end. See `docs/build-prompts/retrospectives/M01.C-retrospective.md`
and `docs/build-prompts/M01-foundation.md` Stage D §D.3 "Coverage gate
semantics" for the decision history.

## License

Apache-2.0. See repo root.
