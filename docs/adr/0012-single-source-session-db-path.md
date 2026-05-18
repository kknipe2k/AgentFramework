# ADR-0012: Single source-of-truth session DB path (MCP registry shares the drone session DB)

**Status:** Accepted
**Date:** 2026-05-17
**Deciders:** @kknipe2k
**Tags:** persistence, ipc-adjacent, scope

## Context

M06 IRL testing (`docs/M06-irl-findings.md` finding 🔴-1) surfaced a
persistence-path divergence invisible to every green automated gate
because Stage-V verified `runtime_mcp::client::Registry` in isolation
against a single path — it never asserted that the *assembled Tauri
shell* hands the registry and the drone the **same** file.

Verified root cause (citations on `main` @ `71107c7`):

- `src-tauri/src/main.rs:262` — `open_mcp_client` independently builds
  `app_local_data_dir().join("mcp.sqlite")` and passes it to
  `Registry::open`.
- `src-tauri/src/main.rs:298-301` — `resolve_db_path` builds
  `app_local_data_dir().join("session.sqlite")` and passes it to
  `DroneLifecycle::spawn` (the drone's DB).
- `Registry::open` → `runtime_drone::db::init(path)` runs **all**
  migrations (the `mcp_servers` table is migration `002`; `db.rs:71-75`)
  against whatever path it is given. Both files therefore receive the
  full schema; the only defect is that the registry writes to a
  different file than the drone reads.

The runtime reads the registry from the live drone session DB
(`session.sqlite`); the UI's `Add MCP Server` wrote it to the stray
`mcp.sqlite`. Result: an added server is audited (`mcp_installed`) yet
invisible and (downstream, at M07) undispatchable. A 0-byte
`mcp_servers.sqlite` also observed in the field — a stale artifact from
a removed code path; no current code constructs that name (the build
confirms via grep in Stage A.fix).

The `Registry` type is correctly path-agnostic (CLAUDE.md §9
path-agnostic-persistence archetype). The bug is purely that the
**composition layer** never decided which file is canonical. ADR-0007
already establishes the drone as the system-of-record; spec §1c + the
`db.rs` module doc comment explicitly design the `mcp_servers` table to
live in this database. The decision was implicit and therefore drifted.

If we do not decide this explicitly, M07 (the agent-with-tools loop
that dispatches MCP tools by reading the registry) inherits a registry
the runtime cannot see, and the next persistence module added (M07+
skill/framework caches) repeats the same divergence.

## Decision

We adopt a **single source-of-truth session database**: the drone
session SQLite database (`<app_local_data_dir>/session.sqlite`) is the
canonical store for all persisted runtime state in v0.1, **including the
MCP server registry**.

Concretely: the Tauri shell resolves the session DB path through **one**
resolver (`resolve_db_path`, or a path-agnostic helper it and
`open_mcp_client` both call); `open_mcp_client` opens the `Registry`
against that same resolved path rather than independently constructing
`"mcp.sqlite"`. The registry is accessed by a second `rusqlite`
connection from the Tauri main process, distinct from the drone
subprocess's connection to the same file.

This is safe because `runtime_drone::db::init` (the path both the drone
and the registry open through) configures `journal_mode=WAL`,
`synchronous=NORMAL`, `busy_timeout=5000`, `foreign_keys=ON`
(`db.rs:5-7`) and runs an **idempotent**, `_migrations`-tracked
migration runner with `CREATE TABLE IF NOT EXISTS` bodies (`db.rs:13`;
gotcha #80). SQLite WAL supports concurrent multi-connection /
multi-process access (many readers + one writer, serialized by
`busy_timeout`). The drone writes the high-frequency
heartbeat/snapshot/signal stream; the Tauri main process writes the
low-frequency registry (a user adds a server occasionally) — contention
is negligible and serialized.

The drone process and the Tauri main process writing different tables in
one WAL file is a supported, documented SQLite pattern. We do **not**
route the registry through new drone IPC commands in v0.1 (see
Alternatives).

## Consequences

### Positive

- Read path == write path: the server the UI adds is the server the
  runtime (and M07's dispatch loop) reads. 🔴-1 closed at its root.
- No new IPC surface, no `runtime-mcp` change, no new dependency — the
  fix is a composition-layer path unification plus an
  assembled-app regression test. Tight, fix-cycle-scoped.
- The cross-process single-file invariant is now explicit and
  reviewable, preventing the next persistence module from re-diverging.
- Consistent with ADR-0007 (drone system-of-record), spec §1c/§11, and
  the `db.rs` design intent that `mcp_servers` lives in this DB.

### Negative

- Two connections to one SQLite file across two processes is a real (if
  well-supported) concurrency surface. Mitigated by WAL + `busy_timeout`
  + idempotent migrations + low registry write frequency; the
  assembled-app regression test pins the round-trip.
- Both processes run `db::init` (idempotent) on the same file at
  startup; correctness depends on the migration runner staying
  idempotent (gotcha #80). The Stage A.fix regression test asserts a
  second connection at the same path sees a freshly-added row.

### Neutral / future implications

- If a future milestone needs strict single-writer ownership of the
  registry, the route-through-drone-IPC alternative (below) is the
  forward path and would itself be ADR-gated (§11 IPC-protocol change).
  v0.1 explicitly does not need it.
- The path-agnostic-persistence archetype (CLAUDE.md §9) is reaffirmed:
  modules take `path: &Path`; the shell resolves the **one** path.

## Alternatives Considered

### Alternative A: Route the MCP registry through new drone IPC commands (`DroneCommand::McpRegistry{Upsert,List,Remove}`), making the drone the only process that opens the file

**Rejected because:** it is an IPC-protocol change (§11 ADR-gated in its
own right), expands a focused fix cycle into a registry-ownership
refactor, and couples to M07's dispatch work. The findings doc mandates
the fix before M07.A, not a heavyweight refactor. WAL multi-connection
already gives the correctness the single-source-of-truth requirement
needs. This remains the documented forward path if a later milestone
requires strict single-writer ownership.

### Alternative B: Keep `mcp.sqlite` separate; make the runtime read the registry from `mcp.sqlite` instead of `session.sqlite`

**Rejected because:** it preserves two databases (registry vs
signals/snapshots) and contradicts ADR-0007 (single system-of-record)
and the `db.rs` design intent. Recovery/replay/audit correlate registry
state with the signal stream by session; splitting the files
re-introduces the divergence class elsewhere.

### Alternative C: In-memory registry

**Rejected because:** it loses persistence across launches (the IRL
"restart-persist" requirement, B-series scenarios) and the §0d scope
expects installed MCP servers to survive a relaunch.

## Related

- Spec sections: §1c (SQLite setup), §11 (reconciliation / signal log),
  §0d (release scope — persistence survives relaunch)
- Prior ADRs: ADR-0007 (drone as in-process system-of-record — this
  reaffirms it for the registry), ADR-0010 (MCP dispatch dependency
  inversion — M07 reads the registry this ADR makes canonical)
- Findings: `docs/M06-irl-findings.md` 🔴-1
- Code: `src-tauri/src/main.rs:262,298-301`;
  `crates/runtime-drone/src/db.rs:5-7,13,71-75`;
  `crates/runtime-mcp/src/client/registry.rs`
- External: SQLite WAL concurrency model
  (<https://www.sqlite.org/wal.html>)

## Notes

Status flips `Proposed → Accepted` in the M06.5 fix-cycle PR before
merge, per CLAUDE.md §11. This ADR records a decision whose *absence*
was the root cause of 🔴-1; making the cross-process single-file
invariant explicit is the structural regression guard, complementary to
the Stage A.fix assembled-app round-trip test.
