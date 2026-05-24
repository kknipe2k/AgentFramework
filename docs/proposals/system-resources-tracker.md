# System Resources Tracker — proposal

**Status:** Proposed (idea-stage; M9-and-beyond scope)
**Date:** 2026-05-24
**Author:** @kknipe2k (mid-M08.5.5 capture)
**Tags:** product, observability, ux, multi-agent

## The idea

A built-in system resources tracker visible in the runtime that surfaces:

- **App's current RAM usage** (the Tauri shell + drone + sandbox + MCP
  subprocesses + their LLM in-flight buffers, broken down by process).
- **Total system RAM** (installed + currently used by all OS processes).
- **What remains** (free + cached vs strictly free per the OS's
  definition of "available").
- **Critical thresholds** — visual + (optional) audible alert when the
  app or the host system hits a configurable warning / critical band
  (e.g., 75% warning, 90% critical).

Why: the runtime can spawn N agents concurrently (per the
laptop-vs-cloud question discussed 2026-05-23 — ~10-100 concurrent
agents on a modern laptop, bounded by MCP subprocess RAM more than
CPU). Without a resource lens, the user has no way to know they're
about to OOM until the OS kills something. A built-in tracker turns
"why did my agents stop?" into "I can see the budget."

## What it surfaces (minimum viable)

- **Per-process memory** broken out: agent-runtime (Tauri shell + main),
  runtime-drone, runtime-sandbox, each MCP server subprocess. Both RSS
  (resident) + private working set + (Windows) commit charge.
- **System totals**: physical installed, currently used, free,
  cached/buffered, swap used.
- **Pressure indicators**: OS-level "memory pressure" event count
  (Windows: low-memory notification; Linux: PSI; macOS: vm_pressure
  notify). Color-coded badge in the runtime chrome.
- **Trend line**: 60-second rolling RAM-by-process chart so the user
  sees whether usage is spiking, stable, or leaking.
- **Per-agent attribution** (when M9+ multi-agent runs are first-class):
  which agent owns which MCP subprocess; which agent's LLM call is
  currently in flight + its approximate buffer cost.

## When to design + implement

**Not v0.1.** v0.1 is single-session, single-framework — resource
scaling isn't a user concern yet. The tracker becomes valuable when:

- M9 Generators ship + users start running generated frameworks with
  3-5+ agents (Sept 2026 timeframe).
- Multi-session runtime arrives (v1.0+) — multiple frameworks
  concurrently makes resource visibility necessary, not optional.
- Runtime Companion overlay mode (`docs/proposals/runtime-companion.md`)
  — the companion uses the resource tracker as one of its diagnostic
  inputs ("agent X is in retry-storm + RAM is at 87% — want to halt?").

Most natural milestone home: **M9 Stage F or M10** (the "first-run +
polish" milestone where observability + UX-feedback gates land). Could
also live as its own M9.5 if it's small enough scope.

## Web research to do at implementation time

**Do NOT pre-research now.** When the implementing X.5 cycle or
milestone is dispatched, the orchestration session web-verifies (per
CLAUDE.md §12 + gotcha #32 cross-stack verbatim-quote rule):

- **Rust system-info crate**: `sysinfo` (de facto standard; check
  latest version + caveats); alternatives like `procfs` (Linux-only),
  `windows-rs` for Job Objects accounting.
- **Per-process subprocess attribution**: how to track a child process's
  RAM without polling-by-PID-list (PSI on Linux; Job Objects on
  Windows; mach task ports on macOS).
- **Memory pressure event subscription**: Windows
  `CreateMemoryResourceNotification`; Linux PSI (`/proc/pressure/memory`);
  macOS `notify_register_dispatch("com.apple.system.memory_pressure")`.
- **Tauri/Electron prior art**: Electron's `process.getProcessMemoryInfo()`
  pattern; Tauri's IPC surface for periodic memory polls; ActivityMonitor
  / Task Manager / htop UX conventions to mirror.
- **Threshold defaults**: industry-standard warn/critical bands (50%
  caution, 75% warn, 90% critical are common; verify against current
  OS guidance for the host platform).
- **Performance overhead of polling**: a 1-second poll has cost; verify
  acceptable + offer user-configurable poll rate.

## Related (existing)

- Spec §0 single-session scope (v0.1) — context for why this isn't
  shipped earlier.
- Spec §4 plan + budget — budget covers token cost; this proposal covers
  RAM cost (complementary, not overlapping).
- `docs/proposals/runtime-companion.md` — the companion consumes the
  resource tracker as a diagnostic input + can take action on pressure.
- The laptop-vs-cloud discussion 2026-05-24 (chat-only) — the rough
  numbers (5-10 agents trivial / 10-20 watch RAM / 100+ cloud) inform
  the tracker's defaults.

## Status + tracking

Forward-design proposal. Lives in `docs/proposals/` (starter-kit
convention adopted post-M08.5.5). Re-evaluate at M9 dispatch to scope
into the right milestone home.
