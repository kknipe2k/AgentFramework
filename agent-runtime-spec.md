# Agentic Runtime Platform — Product Specification

## What This Is

A local Tauri desktop runtime for agentic AI workflows. Not a chatbot wrapper. Not a framework. A **runtime** — the way the JVM is to Java, or Deno is to TypeScript — that frameworks, agents, and skills execute inside.

The core differentiator is a live visual graph that renders agent and skill spawning in real time, detects capability gaps, and suspends cleanly when something is missing — directing the user to the Agent Builder to resolve it. Underneath everything is a dedicated drone process that owns session survival, recovery, and process lifecycle.

Skill finding, writing, and testing is a **build-time activity** in the Agent Builder. The runtime executes what exists. It does not modify itself mid-run.

Built on the Anthropic SDK natively. MCP manager built in. Claude family first, frontier model agnostic long term.

-----

## §0 Project Positioning & Relationship to ARIA

> **Locked decision (2026-04-18).** Supersedes any contrary statement elsewhere in this spec.

### What this runtime is

A **generic agent-building, maintenance, and runtime-management platform**. It ships **primitives** (drone, event pipeline, live graph, plan/task model, hooks, rails, mode field, HITL primitive, registry, builder) — not opinionated agent workflows.

Frameworks are how you compose those primitives into a product. The runtime ships zero built-in frameworks.

### What ARIA is, in this project

**ARIA is the reference archetype**, not a built-in. The existing `.aria/` shell codebase (engine, skills, Ralph loop, dashboard, offline RL, hooks, ~13K LOC) is reference material that defines what an agentic system *should* be capable of. We do not port it, wrap it, or replace it.

The runtime ships an `examples/aria/` directory containing a framework JSON + bundled tools/skills that **reconstruct ARIA's capabilities using only the runtime's primitives**. This is a worked example, not a default.

### MVP success criterion

v1 ships when a user can reconstruct every row of the **ARIA Archetype Capability Matrix** (§0a) inside the runtime using only framework JSON + primitives, **without modifying runtime source**. If a row cannot be reconstructed, either the matrix row is wrong or a primitive is missing — both block v1.

`examples/aria/` is the executable proof of this criterion.

### Per-subsystem fate of `.aria/`

| `.aria/` subsystem | Fate |
|---|---|
| `verify.sh`, `verify-executor.sh` | **Reference only.** `examples/aria/` reimplements via runtime post-task hooks. |
| `rails-executor.sh`, `rails/safety.json` | **Reference only.** Reimplemented via runtime rails primitive. |
| `ralph/ralph.sh` | **Reference only.** Reimplemented via runtime loop policy + plan primitive. |
| `planner/planner.sh` | **Reference only.** Reimplemented via planning agent + plan primitive. |
| `model-selector.sh` | **Reference only.** Reimplemented via budget primitive + model-selector hook. |
| `lib/offline-learner.py` | **Stays as external Python process.** Consumes runtime signal export. Optional. |
| `lib/meta-reasoning.sh` | **Reference only.** Reimplemented as in-runtime decision skill. |
| `hitl.sh` | **Reference only.** Reimplemented via HITL primitive + notifier plugins. |
| `git-ops.sh` | **Reference only.** Reimplemented via tool wrappers around git. |
| `hooks/` (Git hooks) | **Stays untouched.** Independent of runtime. |
| `.claude/agents/` (subagents) | **Reference only.** Reimplemented as runtime agent definitions. |
| `dashboard/`, `serve-dashboard.py` | **Replaced.** Runtime ships its own dashboard (the live graph + panels). |
| `docs/` (CONCEPT-*, WORKFLOW-MAP, etc.) | **Stays.** Source of truth for what the archetype must do. |

### Existing shell ARIA stays usable

Users who prefer the shell experience can keep using `.aria/` as-is. The runtime does not deprecate or break it. They are independent products that share a problem statement.

### Cross-references

- §0a — **ARIA Archetype Capability Matrix** (MVP done-criterion)
- Phase 6 — Framework JSON Loader: the schema below is extended by every primitive (hooks, rails, plan, mode, HITL, budget). Phase 6's example must show `examples/aria/framework.json` end-to-end.
- The phrase "Aria ships as the built-in default framework" elsewhere in this spec is **superseded** by this section. ARIA ships in `examples/`, not as a built-in default.

-----

## §0a ARIA Archetype Capability Matrix

> **MVP done-criterion.** Every row must be reconstructible inside the runtime using framework JSON + primitives. If a row's primitive is not in the runtime, v1 is not done.

| # | ARIA Capability | Runtime Primitive Required | Driving WI |
|---|---|---|---|
| 1 | LITE / STANDARD / FULL / FULL+ mode router | `modes` field in framework JSON + sizing-agent role | WI-15 |
| 2 | Sizing matrix (tasks × LOC × files × deps × auth) | Declarative sizing rules OR sizing agent | WI-15 |
| 3 | `verify.sh` after every task | Per-task `post_task` hook (shell / tool / agent) | WI-02 |
| 4 | Hard / soft rails (`rails/safety.json`) | `rails` section + rails-evaluator | WI-02 |
| 5 | Plan → HITL approve → execute one-by-one | `Plan` primitive + approval-gate event | WI-03 |
| 6 | Subagent isolation (analyzer, implementer, verify-app, simplifier) | Agent type defs + spawn rules in framework JSON | WI-03, WI-04 |
| 7 | Decision trace (`decisions.jsonl`) | Built-in VDR projection from event stream | WI-08 |
| 8 | Signal Schema v2 (8 signal types) | Native event taxonomy in runtime | WI-08 |
| 9 | Ralph autonomous loop | `loop_policy: continuous` + PRD-style goal store | WI-03 |
| 10 | Model selection (budget + learning) | Model-selector hook + budget primitive | WI-07, WI-13 |
| 11 | Offline RL (Thompson Sampling) | External-process plugin reading exported signals | WI-08, WI-23 |
| 12 | Dashboard (`:8420`) | Built-in (replaces ARIA dashboard) | core |
| 13 | Git ops (checkpoint / rollback / PR) | Tool wrappers + drone snapshot integration | WI-02 |
| 14 | HITL notifications (terminal / desktop / sound) | Notifier plugin interface | WI-16 |
| 15 | Slash commands (`/aria-start`, etc.) | Command-palette registration from framework JSON | core |
| 16 | Hooks (PreToolUse, PostToolUse, Stop) | Hook event types + framework subscription | WI-08 |
| 17 | Project-context "don't touch" zones | `dont_touch` field + pre-edit rail | WI-02 |
| 18 | Failure escalation (3 failures → HITL) | Failure-counter primitive + HITL trigger policy | WI-16 |
| 19 | Skills as context-loaded markdown | `Skill` type distinct from `Tool` | WI-04 |
| 20 | Mode-variant skill behavior | Mode-aware skill loader | WI-04, WI-15 |

This matrix is the spec's contract with itself. Every P1 work item must justify its scope by which row(s) it unlocks. Rows that turn out not to need a primitive (because they fall out of an existing one) get marked `subsumed-by: <primitive>` rather than dropped.

-----

## §0b Three Concepts: Tool, Skill, Agent

> **Locked terminology (2026-04-18).** Skills are read, tools are called, agents are spawned. Anywhere this spec used "skill" to mean "callable thing," that has been corrected to "tool."

### Definitions

| Concept | What it is | How declared | How invoked | Sources |
|---|---|---|---|---|
| **Tool** | Callable capability with input/output schema | MCP server JSON; generated `tool.md`; built-in TS registration | Model emits `tool_use` block | MCP, built-in, generator |
| **Skill** | Context-loaded instruction set; markdown read into agent context | Canonical `skill.md` (frontmatter + free-form body) | Runtime-injected `LoadSkill` tool | Local library, registry, generator |
| **Agent** | Composable LLM role: system prompt + allowed tools + allowed skills + model | Framework JSON `agents[]` entry; standalone `agent.md` | Runtime-injected `SpawnAgent` tool; or root agent at session start | Framework JSON |

### Tool

Already covered by Phase 5 (MCP) and Phase 8a (Tool Writer). The previously-named `skill.md` format (callables with `input_schema` / `output_schema`) is renamed `tool.md`. Tool calls emit `tool_invoked` and `tool_result` events.

### Skill

**Canonical `skill.md` schema** (strict frontmatter, free-form body):

```markdown
---
name: planning
version: 1.0.0
description: Create implementation plans with HITL approval gates
triggers:
  semantic:
    - "create a plan"
    - "/plan"
    - "mode_start"
  programmatic:
    - event: session_start
      when: { "!=": [{ "var": "session.mode" }, "LITE"] }
    - event: task_failed
      when: { ">=": [{ "var": "task.failure_count" }, 2] }
mode_variants:
  LITE:     { include_sections: ["quick"] }
  STANDARD: { include_sections: ["full"] }
  FULL:     { include_sections: ["full", "risks"] }
  FULL+:    { include_sections: ["full", "risks", "design_doc"] }
required_tools: ["Read", "Write"]
required_skills: []

# Security & lineage (see Phase 8 / WI-06)
capabilities:
  tools_called:    ["Read", "Write"]
  skills_loaded:   []
  file_access:     { read: [".aria/state/**"], write: [".aria/state/current-plan.json"] }
  network:         []
  shell:           false
  spawn_agents:    []
provenance:        # only present for generated skills; absent for hand-authored
  generator:       "skill_writer"
  model:           "claude-opus-4-7"
  prompt_hash:     "sha256:..."
  generated_at:    "2026-04-18T14:23:00Z"
  validated_at:    "2026-04-18T14:23:42Z"
  content_hash:    "sha256:..."
  signature:       "ed25519:..."
---

# Planning Skill

(free-form body — markdown sections, optionally tagged for mode_variants.include_sections)
```

The `capabilities` block is mandatory for generated skills (Phase 8 validator rejects artifacts missing it). For hand-authored skills it is strongly recommended; if absent, the runtime treats the skill as Operator-tier-only (cannot be loaded under Novice/Promoted enforcement).

**LoadSkill runtime tool** (auto-injected into every agent's tool list):

```typescript
{
  name: "LoadSkill",
  description: "Load instructional context for a named skill before performing related work.",
  input_schema: {
    type: "object",
    properties: {
      skill_name: { type: "string", description: "Name from the available-skills block in your system prompt." },
      reason:     { type: "string", description: "Why you're loading this skill now." }
    },
    required: ["skill_name", "reason"]
  }
}
```

Tool result returns the skill body, with sections filtered by `${session.mode}` per `mode_variants`. Emits `skill_loaded` event with skill name, version, mode, parent agent.

**Triggers — both semantic and programmatic, both ship in v1.**

- **Semantic.** Runtime injects an "Available skills" block in every agent's system prompt:
  ```
  ## Available skills (use the LoadSkill tool to read one before related work)
  - planning — Create implementation plans with HITL approval gates. Triggers: "create a plan", "/plan", mode_start.
  - debugging — Diagnose test failures and errors. Triggers: "debug", "test failure", "error in".
  - tdd — Test-driven development workflow. Triggers: "tdd", "test first".
  ...
  ```
  Agent decides when to call `LoadSkill`.

- **Programmatic.** Skills declare `triggers.programmatic` as a list of event-matchers with optional JSONLogic `when` clauses. The runtime registers a small evaluator that subscribes to the event stream; on match, it emits `skill_load_requested` to the appropriate agent. The agent typically complies (calls `LoadSkill`) but may decline with a brief rationale (recorded as a decision).

  Trigger expression language:
  ```yaml
  - event: <event_type>          # any AgentEvent.type or '*'
    agent: <agent_id_pattern>    # optional, defaults to '*'
    when:                        # optional JSONLogic against the event payload
      ">=": [{ "var": "task.failure_count" }, 2]
  ```

  v1 evaluator supports JSONLogic operators: `var`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `and`, `or`, `not`, `in`. New operators added on demand.

  Why both: semantic alone is brittle (agent forgets to load `debugging` after a failure); programmatic alone is rigid (can't load on a hunch). Together: programmatic safety net + semantic flexibility.

### Agent

**Framework JSON entry:**

```json
{
  "id": "analyzer",
  "role": "Read-only code analysis",
  "system_prompt_template": "You are the analyzer. Read code, understand patterns, propose plans. You do not write files.",
  "allowed_tools": ["Read", "Glob", "Grep"],
  "allowed_skills": ["discovery"],
  "model": "haiku",
  "spawns": [],
  "spawn_constraints": { "max_concurrent": 1, "timeout_ms": 60000 }
}
```

**SpawnAgent runtime tool** — auto-injected for any parent agent whose definition lists `spawns: [...]`. Spawning a child enforces the child's `allowed_tools` / `allowed_skills` constraints. Spawn emits `agent_spawned` with parent_id chain.

Agent definitions can also live as standalone `agent.md` files for distribution (frontmatter mirrors the JSON entry; body is the system prompt template).

### Event taxonomy implications

Phase 2's `AgentEvent` union is updated:

- Renamed: `skill_invoked` → `tool_invoked`, `skill_complete` → `tool_result`, `mcp_tool_called` → folded into `tool_invoked` (with `source: 'mcp' | 'builtin' | 'generated'`), `mcp_tool_result` → folded into `tool_result`.
- Added: `skill_loaded`, `skill_load_requested`, `tool_missing` (vs existing `skill_missing`).
- `skill_missing` retained but now means "framework declares a skill that isn't installed" (load-time gap). `tool_missing` means "agent tried to use a tool that isn't in its allowed list or doesn't exist" (runtime gap).

### Phase impact summary

| Phase | Change |
|---|---|
| Phase 2 | Event union renamed (see above). |
| Phase 3 | `SkillNode` and `ToolNode` are distinct: ToolNode has flowing-edge animation during call; SkillNode has dashed outline indicating in-context load (no in/out flow). |
| Phase 4 | Gap flow distinguishes tool-missing (suspend, builder needed) from skill-missing (warn + suggest install — usually less severe). Detail in WI-05. |
| Phase 5 | MCP exposes Tools only. MCP cannot publish Skills or Agents. |
| Phase 7 | Registry search filterable by `type: tool | skill | agent`. |
| Phase 8 | Splits into 8a Tool Writer, 8b Skill Writer, 8c Agent Composer. All three share the §8.security 5-layer model (L1 Capability Disclosure, L2 Capability Enforcement, L3 Sandboxed Validation, L4 Tiered Human Gate, L5 Provenance & Audit). |
| Phase 9 | Builder canvas palette has three sections: Tools / Skills / Agents. |

-----

## §0c Development Loop

> **Locked (2026-04-18, WI-17 + Tauri migration).** How a developer iterates on the runtime + frameworks.

### Repo layout (workspace)

```
.
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── runtime-core/           # Domain types, AgentEvent, framework schema
│   ├── runtime-main/           # Tauri main process (orchestration, MCP, SDK)
│   ├── runtime-drone/          # Drone binary (heartbeat, snapshots, IPC)
│   └── runtime-sandbox/        # Per-artifact sandbox host (L2 enforcement)
├── src-tauri/                  # Tauri wrapper (commands, allowlist config)
├── src/                        # Frontend (TypeScript + React)
├── examples/                   # Framework artifacts (aria/, ralph/)
└── package.json                # Frontend deps only
```

### Dev commands

```bash
# Frontend HMR + Tauri main + drone, all hot-reloading
cargo tauri dev

# Watch + restart drone independently when iterating on it
cargo watch -x "run -p runtime-drone -- --session-id dev --db-path .dev/runtime.db"

# Frontend-only iteration (mock backend)
npm run dev

# Tests
cargo test --workspace                    # Rust unit + integration
cargo test --workspace --features fuzz   # Property + fuzz suites
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all -- --check               # Format check
npm run test                              # Frontend unit (Vitest)
npm run test:e2e                          # E2E (Playwright against built app)

# Production build (signed, reproducible)
cargo tauri build --target universal-apple-darwin    # macOS
cargo tauri build --target x86_64-pc-windows-msvc    # Windows
cargo tauri build --target x86_64-unknown-linux-gnu  # Linux
```

### Hot-reload behavior per process

| Process | Hot reload | State preserved? |
|---|---|---|
| Renderer (Vite HMR via Tauri dev) | Yes, instant | Local React state lost; SQLite-backed graph state preserved |
| Main (Rust) | On rebuild (~2-5s for incremental) | All in-memory state lost; sessions resume from drone snapshots |
| Drone (Rust, via `cargo watch`) | On rebuild | Drone state lives in SQLite — fully preserved across restarts |
| Sandbox (per-artifact) | Spawned fresh per validation; no hot-reload concept | N/A |

**Drone reloads do not lose session state** — state lives in SQLite. Useful for iterating drone logic against a live session.

**Main reloads drop MCP connections and active streams.** User sees a "main reloading" toast; sessions resume on reconnect (drone keeps running, snapshots ensure no data loss).

### Working on a framework

Framework JSON files live in `examples/`. Edit a file, hit "Reload framework" in the Builder — runtime re-validates against the JSON Schema (see Phase 6) and swaps the active framework for new sessions. Existing sessions continue with the old framework version (snapshot keeps the JSON it loaded with).

### Working on the runtime

- Rust code triggers `cargo watch` rebuild; clippy + rustfmt run on save (configured in editor / pre-commit).
- TypeScript errors surface in renderer HMR; `tsc --noEmit` runs in CI.
- Tests run via `cargo test` + `npm run test`; coverage thresholds enforced in CI (see §12 Engineering Charter).
- Pre-commit hook runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `prettier --check`, `tsc --noEmit`. Hooks blocked from `--no-verify` in CI mirror.

-----

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                       Tauri Shell                         │
│                                                          │
│  ┌───────────────────────────────────────────────────┐   │
│  │       OS WebView (WebKit / WebView2 / GTK)        │   │
│  │       Renderer = TypeScript + React + React Flow  │   │
│  │                                                   │   │
│  │  Live Graph │ Agent Builder │ MCP Manager         │   │
│  │  Gap Panel  │ Registry/Generators │ Session UI    │   │
│  │  HITL UI    │ (runtime + build modes)             │   │
│  │                                                   │   │
│  │  No Node API. Tauri IPC only. Capability-checked. │   │
│  └───────────────────────────────────────────────────┘   │
│                       │ Tauri typed IPC (allowlisted)    │
│  ┌───────────────────────────────────────────────────┐   │
│  │                Main Process (Rust + tokio)        │   │
│  │                                                   │   │
│  │  SDK Event Pipeline (HTTP+SSE → AgentEvent)       │   │
│  │  MCP Client Layer (rmcp / JSON-RPC stdio)         │   │
│  │  Framework Loader + JSON Schema validator         │   │
│  │  Gap Suspender + capability enforcer (§8 L2)      │   │
│  │  Builder: Registry / Generators / Test Harness    │   │
│  │  Notifier plugin host                             │   │
│  └───────────────────────────────────────────────────┘   │
│                       │ Unix socket / Windows named pipe │
│                       │ (framed JSON, dead-process detect)│
│  ┌───────────────────────────────────────────────────┐   │
│  │           Drone Process (Rust + tokio)            │   │
│  │                                                   │   │
│  │  Heartbeat │ Snapshots │ Recovery │ Process spawn │   │
│  │  Per-session; survives main crash; SQLite owner   │   │
│  └───────────────────────────────────────────────────┘   │
│                       │                                  │
│  ┌───────────────────────────────────────────────────┐   │
│  │      Persistence Layer (SQLite, WAL, rusqlite)    │   │
│  │  Sessions │ Snapshots │ Signals │ VDR │ Artifacts │   │
│  │  skills.lock │ skills.audit.jsonl │ token_usage   │   │
│  └───────────────────────────────────────────────────┘   │
│                                                          │
│  ┌───────────────────────────────────────────────────┐   │
│  │   Sandboxes (per-skill capability enforcement)    │   │
│  │  Spawned by drone for L3 validation + L2 runtime  │   │
│  │  OS process boundary; Tauri allowlist; no Node    │   │
│  └───────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

-----

## Tech Stack

> **Locked (2026-04-18, OSS-driven decision).** Tauri + Rust backend chosen over Electron for: ~10 MB binaries vs 150 MB, ~50–80 MB RAM vs 400–600 MB, real OS-level process sandboxing for §8.security L2 capability enforcement, smaller attack surface for OSS scrutiny, faster startup, better battery life on laptops where the runtime stays open all day. Frontend stack stays TypeScript/React — Tauri uses the OS webview (WebKit / WebView2 / WebKitGTK).

|Layer         |Technology                            |Reason                                                     |
|--------------|--------------------------------------|-----------------------------------------------------------|
|Shell         |Tauri 2.x                             |Small footprint, real sandboxing, signed reproducible builds|
|Backend       |Rust 1.80+                            |Memory safety, zero-cost abstractions, fits drone/IPC strengths|
|Async runtime |tokio                                 |Production-grade async I/O, channels, process supervision  |
|UI Framework  |React 18 + TypeScript                 |Component model, mature React Flow ecosystem               |
|Graph Renderer|React Flow                            |Production-grade, extensible, live updates                 |
|Styling       |Tailwind CSS                          |Utility-first, consistent design system                    |
|LLM client    |Direct HTTP + SSE via reqwest + eventsource-stream | Anthropic API is small/stable; direct HTTP avoids SDK churn and a maintenance shim |
|MCP client    |rmcp (official Rust MCP) or direct JSON-RPC over stdio | Official crate when feature-complete; fallback is straightforward |
|Persistence   |SQLite via rusqlite (WAL mode)        |Local, zero server, fast, embedded                         |
|PTY           |portable-pty                          |Cross-platform PTY, CLI bridge fallback                    |
|IPC (renderer↔main)|Tauri typed IPC commands + events |Secure, allowlist-enforced, capability-checked             |
|IPC (main↔drone)|tokio Unix socket / Windows named pipe with framed JSON|Stdout-clean, binary-safe, dead-process detection |
|Frontend build|Vite                                  |Fast dev loop, HMR for renderer                            |
|App build     |Tauri CLI + cargo                     |Reproducible builds, signed releases (Sigstore)            |
|Test (Rust)   |cargo test + proptest + cargo-fuzz    |Unit + property + fuzz                                     |
|Test (TS)     |Vitest + Playwright                   |Renderer unit + E2E                                        |
|Lint/format   |rustfmt + clippy::pedantic + eslint + prettier | Opinionated, consistent across contributors      |

-----

## Build Order

**Phase 1 — Drone (foundation, nothing else starts without this)**
**Phase 2 — SDK Event Pipeline**
**Phase 3 — Live Graph Renderer**
**Phase 4 — Gap Detection + Clean Suspension**
**Phase 5 — MCP Manager**
**Phase 6 — Framework JSON Loader**
**Phase 7 — Agent Builder: Registry Search + Skill Finder**
**Phase 8 — Agent Builder: Skill Writer (collaborative and autonomous)**
**Phase 9 — Agent Builder: Visual Canvas + Tester**

-----

## Two Modes: Runtime vs Build Time

**Runtime** — executes what exists. The live graph renders agents and skills. When a gap is detected the runtime suspends cleanly and directs the user to the Agent Builder. It does not search, install, or write anything mid-run.

**Build Time (Agent Builder)** — a separate mode where the user searches registries, installs skills, writes new ones collaboratively or autonomously, and tests them before activating. Output is a validated skill or updated framework JSON. Once built, return to runtime and resume.

-----

## Phase 1: The Drone

The drone is a dedicated child process spawned before anything else. It owns process survival. It has no UI responsibility. It never dies if the main process crashes.

### Responsibilities

```
Heartbeat
  - Ping main process every 5 seconds
  - Ping Anthropic API connectivity every 30 seconds
  - Ping active MCP servers every 15 seconds
  - Log all heartbeat state to SQLite

Session Snapshots
  - Snapshot full session state every 30 seconds during active run
  - Snapshot on every significant agent event (tool call, skill spawn, agent handoff)
  - Snapshots are immutable — append only, never overwrite
  - Each snapshot tagged: timestamp, session_id, event_type, state_hash

Activity Detection
  - Track last meaningful agent event timestamp
  - Distinguish between: active, idle, stalled, timed_out, user_aborted
  - Long process alive check — is API still streaming? SDK connection still open?
  - Configurable timeout thresholds per session type

Graceful Shutdown
  - On user abort: snapshot → flush logs → kill agent processes in order → confirm clean
  - On crash detection: snapshot what exists → flag session as interrupted → prepare recovery
  - On timeout: warn UI → wait configurable grace period → snapshot → suspend

Process Spawn and Stop
  - Own the lifecycle of all child processes: agents, MCP servers, skill test sandboxes
  - Spawn with registered PID tracking
  - Stop is always graceful first, force kill only on confirmed hang
  - Restart policy per process type: never, on-crash, always

Session Recovery
  - On platform start, check for interrupted sessions
  - Load last valid snapshot
  - Present recovery options to user: resume, discard, inspect
  - Resume rebuilds SDK message history from snapshot, reconnects MCPs, restores graph state
```

### Drone Process Interface

```rust
// crates/runtime-drone/src/protocol.rs
//
// Drone spawned by main as a tokio child process:
//   runtime-drone --session-id <id> --db-path <path> --ipc-socket <path>
//
// IPC: framed JSON-newline over Unix domain socket (Linux/macOS) or
// Windows named pipe. Stdout/stderr reserved for logs (captured to file).

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DroneEvent {
    Heartbeat            { status: HeartbeatStatus, timestamp: i64 },
    SnapshotWritten      { snapshot_id: String,    session_id: String },
    ActivityStateChange  { from: ActivityState,    to: ActivityState },
    ProcessSpawned       { pid: u32,               process_type: ProcessType },
    ProcessStopped       { pid: u32,               reason: StopReason },
    RecoveryAvailable    { session_id: String,     snapshot_id: String },
    Alert                { level: AlertLevel,      message: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DroneCommand {
    SnapshotNow         { reason: String },
    GracefulShutdown    { timeout_ms: u64 },
    SpawnProcess        { process_type: ProcessType, config: ProcessConfig },
    StopProcess         { pid: u32, force: bool },
    SetActivityTimeout  { ms: u64 },
    RevertToSnapshot    { snapshot_id: String, reason: RevertReason },  // §1b / WI-14
}

#[derive(Serialize, Deserialize)]
pub enum ActivityState { Active, Idle, Stalled, TimedOut, UserAborted, Recovering }
type StopReason = 'graceful' | 'crash' | 'timeout' | 'user_abort' | 'force_kill'
```

### Drone Error Matrix

|Scenario                        |Drone Behavior                                        |User Experience                                |
|--------------------------------|------------------------------------------------------|-----------------------------------------------|
|Main process crash              |Drone survives, continues snapshots, flags session    |On relaunch: recovery offered                  |
|API connectivity lost           |Alert to main, enter stalled state, retry with backoff|Warning banner, auto-resume when reconnected   |
|MCP server dies                 |Alert, attempt restart, snapshot before restart       |Graph shows MCP node as offline, auto-recovers |
|User force-closes app           |OS SIGTERM caught, emergency snapshot, flush          |On next open: recovery offered                 |
|Agent hangs (no output >timeout)|Warn user, wait grace period, snapshot, suspend       |“Agent stalled” prompt with options            |
|Gap detected (skill missing)    |Snapshot, suspend session cleanly                     |Gap panel opens, user directed to Agent Builder|

### §1b Recovery Semantics

> **Locked (2026-04-18, WI-14).** Resume rebuilds **history**, not execution. Document tool-call uncertainty handling. Make non-determinism an explicit non-goal in v1.

#### Resume rebuilds history (does not re-execute)

When a user resumes a suspended/crashed session from a drone snapshot:

1. Prior agent messages, tool calls, and tool results are loaded from the snapshot into the SDK message history **as if they had already happened**.
2. The model starts generating the **next** turn fresh, with full prior context.
3. The runtime does NOT replay tool calls. Tools that depended on external state (web fetches, time-of-day, file contents at time T) are not re-invoked.

This is intentional. Re-invoking tool calls would be non-deterministic (web responses change, files change) and could make irreversible operations (writes, commits, API calls) happen twice.

#### Tool calls in flight at crash time

A snapshot taken *between* a `tool_invoked` and the corresponding `tool_result` represents an uncertain operation. The runtime detects this on resume by looking for `tool_invoked` signals without a paired `tool_result`.

For each such tool call:
1. VDR row marked `tool_call_uncertain: true`.
2. Resume UI surfaces a prompt:

   > Tool `<name>` was invoked but did not complete before crash. What happened?
   > [r]etry — re-invoke from scratch
   > [s]kip — treat as if it returned nothing; agent continues with that gap
   > [m]ark complete — assume it completed (provide output if known)
   > [a]bort — cancel resume, archive session

3. User decision is recorded as a `tool_call_uncertainty_resolved` decision signal.

#### MCP reconnection

On resume:
1. Each MCP server in the snapshot's connection list is reconnected.
2. Failed reconnections leave the MCP node offline; tools from that server become unavailable.
3. Tools-from-offline-MCP that the agent attempts emit `tool_missing` and route through the gap flow (Phase 4 / §4b).
4. User can cancel reconnect and continue with the MCP offline (degraded mode).

#### Plan state restoration

If the suspended session had an active plan (§3a):
1. Plan + task statuses restored from snapshot.
2. Currently-running task is set to `pending` (unless its `task_completed` event is in the snapshot, in which case it's already `done`).
3. Tasks that completed before suspension stay `done`.
4. Loop policy resumes from the restored task.

#### Capability state

Per-artifact capability sets (Phase 8 §8.security L2) are restored. Pending capability grants (`scope: 'session'`) carry over; `scope: 'once'` grants are cleared.

#### Non-determinism is explicit

The runtime does NOT attempt deterministic replay. If a user wants to *replay* a session (rerun decisions to see if the model would behave differently), that's a separate "replay" mode (WI-18, deferred to v2) that runs against frozen tool inputs/outputs from the original VDR rather than re-invoking tools.

For v1: resume continues; replay does not exist.

### §1c Multi-Session & SQLite Concurrency

> **Locked (2026-04-18, WI-10).** One drone per session; one shared SQLite database in WAL mode; ref-counted MCP connection pool.

#### Drone-per-session

Each session spawns its own drone process (`child_process.fork('drone.ts', ['--session-id', s])`). Drones do not share state in memory; coordination happens through the shared SQLite database.

Rationale:
- Crash isolation — one drone dying cannot corrupt another session.
- Resource accounting — easy to attribute snapshots, signals, and budget to the drone owning the session.
- Clean shutdown — graceful_shutdown applies per-drone without ordering hazards.

Tradeoff: more processes. With Tauri's lighter process model (~10 MB resident per drone in Rust) this is fine for tens of concurrent sessions; v1 caps at 8 concurrent active sessions and queues additional requests as a conservative starting point.

#### SQLite WAL mode

Database opened with:
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;
```

WAL allows concurrent readers + one writer; busy_timeout retries on writer contention. Snapshot writes are the highest-volume path — kept under a single transaction per snapshot.

#### MCP connection pool

MCP servers identified by URL are singleton-per-URL at the runtime level, ref-counted across sessions:

1. Session A connects to `mcp://localhost:3000` → connection created, refcount 1.
2. Session B connects to same URL → existing connection reused, refcount 2.
3. Session A ends → refcount 1, connection stays.
4. Session B ends → refcount 0, connection torn down.

Auth conflicts resolved by **first-connection-wins**: the first session to connect with a given URL sets the auth config. Subsequent sessions attempting to connect with different auth get a warning (`mcp_auth_conflict` event) and either accept the existing config or fail their connection. v2 may add isolated-pools-per-auth.

#### Cross-session UI

The webview renderer can display multiple sessions simultaneously (tabs or split view). Each session has its own graph, panels, and gap state. Switching tabs does not pause sessions — only the active tab is rendered.

### §1d IPC Channels

> **Locked (2026-04-18, WI-09 + Tauri migration).** Two IPC layers, both typed and stdout-clean.

#### Layer 1: Renderer ↔ Main (Tauri IPC)

Tauri provides typed commands + events between the webview and the Rust main process. Renderer cannot bypass these — there is no Node API in the webview. All commands are allowlist-enforced via `tauri.conf.json`.

```typescript
// Renderer (TS)
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

await invoke('load_framework', { path: 'examples/aria/framework.json' })
const unlisten = await listen<AgentEvent>('agent_event', (e) => graph.handle(e.payload))
```

```rust
// Main (Rust)
#[tauri::command]
async fn load_framework(path: String, app: tauri::AppHandle) -> Result<FrameworkInfo, String> {
    // ... validation, schema check, load ...
    app.emit("agent_event", AgentEvent::SessionStart { /* ... */ }).ok();
    Ok(info)
}
```

Tauri's `tauri.conf.json` allowlist enumerates which commands and which file/network operations the renderer can request. Anything not on the allowlist is hard-denied at the bridge — capability enforcement starts here.

#### Layer 2: Main ↔ Drone (framed JSON over Unix socket / named pipe)

Drone is spawned by main as a tokio child process. IPC uses framed JSON-newline over a Unix domain socket (macOS/Linux) or Windows named pipe — chosen over stdio for the same reasons as before, plus stronger isolation.

```rust
// Main spawns drone
let socket_path = session_socket_path(&session_id);
let drone = tokio::process::Command::new("runtime-drone")
    .arg("--session-id").arg(&session_id)
    .arg("--db-path").arg(&db_path)
    .arg("--ipc-socket").arg(&socket_path)
    .stdout(Stdio::piped())                      // captured to log file
    .stderr(Stdio::piped())                      // captured to log file
    .spawn()?;

let listener = UnixListener::bind(&socket_path)?;
let (sock, _) = listener.accept().await?;
let (read, write) = sock.into_split();
let mut events = FramedRead::new(read, JsonCodec::<DroneEvent>::new());
let mut commands = FramedWrite::new(write, JsonCodec::<DroneCommand>::new());

// Drone heartbeats arrive on `events`; main sends commands via `commands`.
```

```rust
// Drone connects back to main's socket
let stream = UnixStream::connect(&args.ipc_socket).await?;
let (read, write) = stream.into_split();
// Mirror codec on the drone side; receive DroneCommand, emit DroneEvent.
```

Why not stdio:
- Library warnings to stdout corrupt JSON streams.
- Socket-framed JSON is binary-safe and gives clean dead-process detection (socket close = drone gone).
- Stdout/stderr stay available for log capture (`runtime-drone` logs to a structured file via `tracing` crate, not to the IPC stream).

Why a socket per session:
- Trivially supports multi-session (`§1c`) — each session pair (main, drone) has its own socket path under `$RUNTIME_DATA_DIR/sockets/{session_id}.sock`.
- Permission-restricted: socket created with mode 0600, owner-only.
- Easy to mock in tests (point at a `tempfile`-backed socket).

-----

## Phase 2: SDK Event Pipeline

Every agent action surfaces as a typed event. The graph renders from these events. Decision traces log from these events. Nothing in the UI touches the SDK directly.

### Event Types

```typescript
type AgentEvent =
  | { type: 'session_start'; session_id: string; framework: string; model: string }
  | { type: 'agent_spawned'; agent_id: string; agent_name: string; parent_id: string | null }
  | { type: 'agent_complete'; agent_id: string; result: string }
  | { type: 'agent_error'; agent_id: string; error: string }

  // Tools (callables — see §0b)
  | { type: 'tool_invoked'; tool_name: string; agent_id: string; source: 'mcp' | 'builtin' | 'generated'; server?: string; input: unknown }
  | { type: 'tool_result'; tool_name: string; agent_id: string; output: unknown; duration_ms: number }
  | { type: 'tool_missing'; tool_name: string; agent_id: string; reason: 'not_in_allowed_list' | 'not_installed' | 'request_capability'; suspends_session: boolean }

  // Skills (instruction sets — see §0b)
  | { type: 'skill_loaded'; skill_name: string; skill_version: string; agent_id: string; mode: string; trigger_kind: 'semantic' | 'programmatic' }
  | { type: 'skill_load_requested'; skill_name: string; agent_id: string; trigger_event: string }
  | { type: 'skill_missing'; skill_name: string; agent_id: string; context: string; source: 'static' | 'request_capability' } // recoverable

  // Gaps (see Phase 4 / WI-05)
  | { type: 'agent_missing'; agent_id: string; referenced_by: string } // schema error at load
  | { type: 'gap_resolved'; capability_name: string; capability_kind: 'tool' | 'skill' | 'agent' }
  | { type: 'capability_requested'; agent_id: string; capability_name: string; capability_kind: 'tool' | 'skill'; reason: string } // request_capability call

  // Budget & cost (see §2a / WI-07)
  | { type: 'budget_warning';     scope: 'session' | 'framework' | 'global'; spent_usd: number; cap_usd: number; percent: number }
  | { type: 'budget_downshift';   from_model: string; to_model: string; reason: 'budget_threshold'; spent_usd: number; cap_usd: number }
  | { type: 'budget_suspended';   scope: 'session' | 'framework' | 'global'; spent_usd: number; cap_usd: number; percent: number }
  | { type: 'budget_exceeded';    scope: 'session' | 'framework' | 'global'; spent_usd: number; cap_usd: number }

  // Hooks & Rails (see §4a / WI-02)
  | { type: 'hook_started'; hook_id: string; category: 'verify' | 'lint' | 'build' | 'test' | 'custom'; firing_point: string }
  | { type: 'hook_passed';  hook_id: string; duration_ms: number; output_preview?: string }
  | { type: 'hook_failed';  hook_id: string; duration_ms: number; error: string; on_failure: 'block' | 'warn' | 'rollback' }
  | { type: 'rail_triggered'; rail_id: string; policy: 'hard' | 'soft'; firing_point: string; message: string; agent_id?: string }

  // Mode (see §3b / WI-15)
  | { type: 'mode_proposed';  proposed_mode: string; rationale?: string; agent_id?: string }
  | { type: 'mode_confirmed'; mode: string; confirmed_by: 'user' | 'auto' | 'declarative' }
  | { type: 'mode_locked';    mode: string }

  // Tool alias warnings (see §5a / WI-11)
  | { type: 'tool_alias_ambiguous'; short_name: string; candidates: string[] }

  // MCP auth (see §1c / WI-10)
  | { type: 'mcp_auth_conflict'; server_url: string; existing_session_id: string; requesting_session_id: string }

  // HITL extras (see §6a / WI-16)
  | { type: 'hitl_timeout'; trigger: string; session_id: string; default_action: string }
  | { type: 'notifier_dispatched'; notifier_type: string; trigger: string; success: boolean }
  | { type: 'notifier_failed'; notifier_type: string; trigger: string; error: string }

  // Registry / artifact integrity (see Phase 7 / WI-12)
  | { type: 'artifact_hash_mismatch'; artifact_name: string; expected: string; got: string }

  // Plan & Task lifecycle (see §3a / WI-03)
  | { type: 'plan_created'; plan_id: string; title: string; task_count: number; approval_required: boolean }
  | { type: 'plan_approval_requested'; plan_id: string }
  | { type: 'plan_approved'; plan_id: string; approved_by: 'user' | 'auto' }
  | { type: 'plan_revised'; plan_id: string; revision_reason: string }
  | { type: 'plan_aborted'; plan_id: string; reason: string }
  | { type: 'plan_complete'; plan_id: string; duration_ms: number }
  | { type: 'task_started'; task_id: string; agent_id: string }
  | { type: 'task_completed'; task_id: string; duration_ms: number }
  | { type: 'task_failed'; task_id: string; error: string; failure_count: number }
  | { type: 'task_skipped'; task_id: string; reason: string }
  | { type: 'task_escalated'; task_id: string; failure_count: number; max_failures: number }

  // Capability enforcement (see Phase 8 §8.security L2 / WI-06)
  | { type: 'capability_violation'; artifact_kind: 'tool' | 'skill' | 'agent'; artifact_name: string; attempted: string; declared: string[]; agent_id: string }
  | { type: 'capability_grant'; artifact_name: string; granted_capability: string; scope: 'once' | 'session' | 'forever' } // user explicitly allowed a violation
  | { type: 'tier_changed'; from: 'novice' | 'promoted' | 'operator'; to: 'novice' | 'promoted' | 'operator' }
  | { type: 'artifact_installed'; kind: 'tool' | 'skill' | 'agent'; name: string; version: string; tier: string; gate: 'manual' | 'auto_accepted'; provenance_id: string }
  | { type: 'artifact_validation'; kind: 'tool' | 'skill' | 'agent'; name: string; passed: boolean; report_id: string }

  // HITL, plan, hooks (see WI-02, WI-03, WI-16)
  | { type: 'hitl_requested'; agent_id: string; question: string; options: string[] | null }
  | { type: 'hitl_response'; agent_id: string; response: string }

  // Cost & streaming
  | { type: 'token_usage'; input: number; output: number; model: string; cost_usd: number }
  | { type: 'stream_text'; agent_id: string; text: string }
  | { type: 'decision_record'; agent_id: string; decision: string; rationale: string; tool_used: string }
```

> Plan/task, hook, rail, and budget events are added by WI-03, WI-02, and WI-07 respectively. This union grows; Phase 2 owns the canonical list.

### SDK Wrapper

```rust
// crates/runtime-main/src/sdk/agent_sdk.rs

pub struct AgentSdk<P: LLMProvider> {
    provider:   P,                         // see §2c — generic over provider
    event_tx:   mpsc::Sender<AgentEvent>,
    session_id: SessionId,
}

impl<P: LLMProvider> AgentSdk<P> {
    pub async fn run_agent(&self, config: AgentConfig) -> Result<(), SdkError> {
        // Emit agent_spawned
        // Stream provider events; translate ProviderEvent → AgentEvent
        // On tool use:    emit tool_invoked   (source: mcp | builtin | generated)
        // On tool result: emit tool_result
        // On LoadSkill result: emit skill_loaded with mode + trigger_kind
        // On request_capability: emit capability_requested → tool_missing or
        //                        skill_missing per kind (see §4b)
        // On missing tool (static): emit tool_missing → gap flow
        // On text block:  extract decision records, emit decision_record
        // On complete:    emit agent_complete
        // On error:       emit agent_error → drone notified
        Ok(())
    }
}
```

Provider implementation hits the Anthropic HTTP+SSE API directly:

```rust
// crates/runtime-main/src/providers/anthropic.rs

pub struct AnthropicProvider {
    http: reqwest::Client,
    api_key: SecretString,                 // from keychain via secrets vault
    base_url: Url,
}

impl LLMProvider for AnthropicProvider {
    async fn stream(&self, config: AgentConfig)
        -> Result<impl Stream<Item = ProviderEvent>, ProviderError>
    {
        // POST /v1/messages with stream=true
        // Parse SSE via eventsource-stream
        // Yield ProviderEvent { kind: TextDelta | ToolUse | ToolResult | MessageStop | Error }
    }
}
```

Direct HTTP keeps the dependency surface small: `reqwest`, `eventsource-stream`, `serde`, `tokio`. No third-party SDK to track for breaking changes.

### Verified Decision Records (VDR)

Every agent decision is logged with full lineage:

```typescript
interface VerifiedDecisionRecord {
  id: string
  session_id: string
  agent_id: string
  timestamp: number
  decision: string
  rationale: string
  tool_invoked: string | null
  tool_input: unknown
  tool_output: unknown
  token_cost: number
  outcome: 'success' | 'failure' | 'pending'
  snapshot_id: string  // links to drone snapshot at time of decision
}
```

### §2c LLMProvider Abstraction

> **Locked (2026-04-18, WI-13 + Tauri migration).** The runtime ships a single provider implementation in v1 (Anthropic, hitting the HTTP API directly with reqwest + eventsource-stream — no third-party SDK). `AgentSdk` is generic over an `LLMProvider` trait so a second provider (OpenAI, Google, local-Ollama) is a new impl, not a refactor.

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;                               // "anthropic" | "openai" | ...
    fn supports(&self) -> ProviderSupport;

    async fn stream(&self, config: AgentConfig)
        -> Result<BoxStream<'_, ProviderEvent>, ProviderError>;
    async fn count_tokens(&self, messages: &[Message]) -> Result<u64, ProviderError>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;
    fn estimate_cost(&self, input_tokens: u64, output_tokens: u64, model: &str) -> f64;
}

pub struct ProviderSupport {
    pub tool_use:  bool,
    pub streaming: bool,
    pub thinking:  bool,
}

pub enum ProviderEvent {
    TextDelta     { text: String },
    ToolUse       { id: String, name: String, input: serde_json::Value },
    ToolResult    { id: String, output: serde_json::Value },
    ThinkingDelta { text: String },
    MessageStop   { stop_reason: String },
    Error         { code: String, message: String },
}

pub struct ModelInfo {
    pub id:             String,
    pub display_name:   String,
    pub context_window: u32,
    pub pricing:        Pricing,
    pub capabilities:   ModelCapabilities,
}
```

`AgentSdk` consumes `ProviderEvent` and translates into `AgentEvent`. Provider-specific concerns (Anthropic's `content_block_delta`, OpenAI's `delta`, etc.) stay inside the provider impl. The translation layer is what adds source/agent_id/session context.

v1 ships `AnthropicProvider`; provider selection lives in framework JSON:

```json
{
  "model": { "provider": "anthropic", "id": "claude-opus-4-7" },
  "fallback_models": [
    { "provider": "anthropic", "id": "claude-sonnet-4-6" }
  ]
}
```

Model IDs move out of hardcoded constants; provider's `listModels()` populates the Builder UI's selector dropdown.

-----

### §2b Signals & VDR Projection

> **Locked (2026-04-18, WI-08).** The `AgentEvent` union above is the live event stream the runtime emits. Persistence layers two views on top: **signals** (rich per-operation forensic records) and **VDR** (decision-level projection). This inherits `.aria/docs/SIGNAL-SCHEMA-V2.md` directly rather than re-deriving a weaker model.

#### Signals (rich, forensic)

A signal is a persisted, schema-validated record of one operation. Eight signal types (1:1 with Signal Schema v2):

| # | Signal type | Source events | Purpose |
|---|---|---|---|
| 1 | `tool` | `tool_invoked` (pre) + `tool_result` (post), correlated | Per-tool-call forensic record with input/output preview, duration, retry chain |
| 2 | `skill` | `skill_loaded`, `skill_load_requested` | Skill-load record with mode, trigger_kind, parent agent |
| 3 | `agent` | `agent_spawned`, `agent_complete`, `agent_error` | Subagent record with tools_used summary, files_touched, parent chain |
| 4 | `decision` | `decision_record`, `capability_grant`, `tier_changed` | Discrete decision points with rationale and confidence |
| 5 | `verify` | `hook_*` where category=verify; `rail_triggered` | Verification outcome with test results, coverage, failing items |
| 6 | `error` | `agent_error`, `hook_failed`, `tool_missing`, `capability_violation` | Error chain with retry_of correlation |
| 7 | `hitl` | `hitl_requested`, `hitl_response`, `plan_approval_*`, `task_escalated` | Human intervention record with decision time, response |
| 8 | `session` | `session_start`, `plan_complete`, `session_end` | Session-level summary boundaries |

Pre/post events are correlated via a `pre_signal_id` field. Retry chains via `retry_of`. Parent-child via `parent_signal_id`. Context classification via `context.type ∈ {skill, framework, code, search, verify, commit, subagent}`.

Full schema in `.aria/docs/SIGNAL-SCHEMA-V2.md`. The runtime ports this verbatim — adding new fields only when justified.

#### VDR (Verified Decision Records — projection)

VDR is a **projection** of signals 4 (decision) + 5 (verify), narrowed to decisions that affected outcomes. It is not a parallel system. One row per decision-producing event with:

- `signal_ids: string[]` — pointers back to the contributing signals (preserves forensic depth)
- Decision text, rationale, alternatives considered, confidence
- Tool invoked + input/output (denormalized from the contributing tool signal for fast read)
- Outcome (success / failure / pending), token cost
- Snapshot ID linking to drone snapshot at decision time

**Why two layers:** signals are write-heavy and forensic (every Read, every Bash); VDR is read-heavy and decision-focused (dashboard, query-decisions, postmortems). Splitting keeps signal writes fast and VDR queries cheap.

#### SQLite schema additions

```sql
-- Signals (write-heavy, append-only, indexed for forensic query)
CREATE TABLE signals (
  id TEXT PRIMARY KEY,
  session_id TEXT,
  type TEXT,                  -- tool | skill | agent | decision | verify | error | hitl | session
  event TEXT,                 -- pre | post | started | completed | failed | etc.
  timestamp TEXT,
  duration_ms INTEGER,
  payload_json TEXT,          -- full signal record per Signal Schema v2
  pre_signal_id TEXT,         -- correlation: post -> pre
  parent_signal_id TEXT,      -- correlation: child -> parent
  retry_of TEXT,              -- correlation: retry chain
  context_type TEXT,          -- skill | framework | code | search | verify | commit | subagent
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);
CREATE INDEX idx_signals_session_time ON signals(session_id, timestamp);
CREATE INDEX idx_signals_type ON signals(type);
CREATE INDEX idx_signals_correlation ON signals(pre_signal_id, parent_signal_id, retry_of);

-- VDR remains as defined (Phase 2) but adds:
ALTER TABLE vdr ADD COLUMN signal_ids TEXT;     -- JSON array of contributing signal IDs
ALTER TABLE vdr ADD COLUMN context_type TEXT;
```

#### Importer for existing ARIA traces

For users transitioning from shell ARIA: the runtime ships a one-shot importer that reads `.aria/state/signals.jsonl` and `.aria/state/decisions.jsonl` into the new schema. Non-destructive (does not modify shell ARIA's files). Per §0 archetype model, this enables the runtime to load historical context for users reconstructing ARIA in `examples/aria/`.

#### Export

Signals can stream out via the OpenTelemetry exporter (WI-23) or to a flat-file mirror (`.aria-runtime/signals.jsonl`) for external consumers like the offline RL learner (matrix row 11).

-----

## §2a Budget & Cost Controls

> **Locked (2026-04-18, WI-07).** Three budget scopes that stack. Four enforcement actions on threshold breach. Reuses the §3a HITL flow for suspend/resume on budget triggers.

A runtime without budget controls produces $500 surprise bills. ARIA's `model-selector.sh` and `offline-learner.py` capture the spend-aware logic; the runtime ports the *enforcement primitives*, not the specific algorithms.

### Budget scopes

Three scopes evaluated in order; the tightest applicable cap wins.

| Scope | Where defined | Default | Use |
|---|---|---|---|
| **Per-session** | Settings or framework JSON | $5.00 | Bound any single session run |
| **Per-framework** | `framework.budget` | None | Cap total spend for a long-running framework |
| **Per-day global** | Settings | None | Total all-sessions cap (defense in depth) |

### Threshold actions

```typescript
interface BudgetActions {
  warn_at_percent?:        number   // default 50 — toast + graph header color shift
  downshift_at_percent?:   number   // default 75 — switch to cheaper model tier
  hitl_at_percent?:        number   // default 90 — suspend, require approval to continue
  hard_stop_at_percent?:   number   // default 100 — kill agents, mark session budget_exceeded
}
```

Each action is independently configurable per scope; setting any to `null` disables it for that scope.

### Downshift policy

When `downshift_at_percent` triggers, the runtime invokes the model-selector hook (see WI-13 LLMProvider abstraction). Hook receives current model + remaining budget, returns the model to switch to. Default built-in policy mirrors ARIA's tiers:

```
opus    → sonnet  (any time downshift fires)
sonnet  → haiku   (only if remaining budget < 10% AND avg-task-cost > remaining/3)
haiku   → haiku   (no further downshift; once at haiku, only HITL/hard-stop remain)
```

Frameworks can replace the hook with their own selector (e.g., a port of `model-selector.sh`'s learning-based selection).

### Budget events (added to Phase 2 union)

```typescript
| { type: 'budget_warning';     scope: 'session' | 'framework' | 'global'; spent_usd: number; cap_usd: number; percent: number }
| { type: 'budget_downshift';   from_model: string; to_model: string; reason: 'budget_threshold'; spent_usd: number; cap_usd: number }
| { type: 'budget_suspended';   scope: 'session' | 'framework' | 'global'; spent_usd: number; cap_usd: number; percent: number }
| { type: 'budget_exceeded';    scope: 'session' | 'framework' | 'global'; spent_usd: number; cap_usd: number }
```

`budget_exceeded` triggers immediate agent kill via the drone's `stop_process` command for all session-spawned agents.

### Graph integration

Session header bar shows `spent / cap` with color gradient:
- `< warn_at_percent` — green
- `≥ warn_at_percent` — amber
- `≥ downshift_at_percent` — orange
- `≥ hitl_at_percent` — red, suspended badge
- `≥ hard_stop_at_percent` — red, exceeded badge, agents killed

AgentNode size already reflects per-agent spend (Phase 3 L277); §2a adds the session-level header bar.

### Framework JSON

```json
{
  "budget": {
    "session_usd_cap": 5.00,
    "framework_usd_cap": 100.00,
    "actions": {
      "warn_at_percent":      50,
      "downshift_at_percent": 75,
      "hitl_at_percent":      90,
      "hard_stop_at_percent": 100
    },
    "downshift_hook": { "type": "tool", "tool_name": "select_cheaper_model" }
  }
}
```

Global per-day cap lives in user settings, not framework JSON.

This is the §0a matrix proof for row 10 (model selection budget + learning) — the *primitive* required is the budget scope/action model; the *learning* part is row 11 (offline RL) which stays as an external Python process consuming exported signals.

-----

## Phase 3: Live Graph Renderer

The graph is the product’s face. It renders the full agentic runtime as it happens. Every spawned agent is a node. Every skill invocation is an edge. Every gap is visible.

### Node Types

```
AgentNode      — spawned agent, shows status, current action, token spend
ToolNode       — callable invocation (MCP / built-in / generated); animated edge during call
SkillNode      — context-loaded instruction set (LoadSkill); dashed outline, no flow animation
MCPNode        — connected MCP server, hosts ToolNodes for its tools
GapNode        — missing tool (suspends) or missing skill (warns); see Phase 4
HITLNode       — blocked on human input, highlighted, awaiting response
PlanNode       — current plan root, shows progress (added by WI-03)
TaskNode       — task within plan, shows status and HITL flag (added by WI-03)
VerifyNode     — post-task hook firing, pass/fail (added by WI-02)
FrameworkNode  — root node, the active framework (e.g., examples/aria)
```

### Graph Behavior

- Nodes spawn in real time as events arrive from the pipeline
- Edges animate as tool calls flow: agent → ToolNode (or agent → MCPNode → ToolNode for MCP-hosted tools)
- Skill loads render a brief dashed line from agent → SkillNode (no in-flight animation; loaded skill stays in context)
- GapNode appears immediately on `tool_missing` (suspends) or `skill_missing` (warn — see Phase 4 / WI-05)
- HITLNode blocks the graph visually, dims non-relevant nodes, prompts user
- Completed agents and skills collapse to summary state, remain inspectable
- Full graph is zoomable, pannable, selectable
- Click any node for full VDR trace, input/output, timing
- Graph state is persisted per session — reopen a session and the graph reconstructs

### Visual Design Principles

- Dark background, high contrast node labels
- Color encodes state: active (blue), complete (green), error (red), gap (amber), hitl (white/bright)
- Edges use animated dashes during active calls, solid when complete
- Token spend shown as node weight — larger spend = visually larger node
- No clutter — only show detail on hover or selection

-----

## §3a Plan & Task Primitive

> **Locked (2026-04-18, WI-03).** A generic plan/task primitive the framework composes. ARIA's "plan → HITL approve → execute one task → verify → commit → next" is one realization. Ralph's "PRD-driven continuous loop" is another. Both reconstruct using the same primitive.

### Data types

```typescript
interface Plan {
  id: string
  session_id: string
  title: string
  description?: string
  tasks: Task[]
  status: 'pending_approval' | 'approved' | 'in_progress' | 'complete' | 'aborted' | 'awaiting_replan'
  approval_required: boolean              // false = auto-approve (Ralph-style)
  loop_policy: LoopPolicy
  hitl_checkpoints: string[]              // free-form list of checkpoint names referenced by tasks
  risks: string[]                         // free-form list, surfaced in approval UI
  created_at: number                      // unix ms
  approved_at?: number
  completed_at?: number
}

interface Task {
  id: string
  plan_id: string
  title: string
  description?: string
  status: 'pending' | 'running' | 'done' | 'blocked' | 'failed' | 'skipped'
  hitl: boolean
  hitl_reason?: string
  estimated_minutes?: number
  actual_minutes?: number
  post_hooks?: HookRef[]                  // override framework defaults; see §4a
  failure_count: number                    // increments on task_failed; resets on retry
  max_failures: number                     // default 3; overridable per task; triggers escalation
  files_affected?: string[]                // optional, for HITL UX
  acceptance_criteria?: string[]           // optional, for verify integration
}

type LoopPolicy =
  | { kind: 'one_shot' }                            // run once, exit
  | { kind: 'fresh_context_per_task' }              // ARIA pattern: each task gets a new agent with clean context
  | { kind: 'continuous'; goal_store: string }       // Ralph pattern: one persistent agent; goal_store points to PRD-style file
```

### Events (added to Phase 2 union)

```typescript
| { type: 'plan_created'; plan_id: string; title: string; task_count: number; approval_required: boolean }
| { type: 'plan_approval_requested'; plan_id: string }
| { type: 'plan_approved'; plan_id: string; approved_by: 'user' | 'auto' }
| { type: 'plan_revised'; plan_id: string; revision_reason: string }
| { type: 'plan_aborted'; plan_id: string; reason: string }
| { type: 'plan_complete'; plan_id: string; duration_ms: number }
| { type: 'task_started'; task_id: string; agent_id: string }
| { type: 'task_completed'; task_id: string; duration_ms: number }
| { type: 'task_failed'; task_id: string; error: string; failure_count: number }
| { type: 'task_skipped'; task_id: string; reason: string }
| { type: 'task_escalated'; task_id: string; failure_count: number; max_failures: number }   // failure_count >= max_failures
```

### Approval-gate primitive

When a `plan_created` event fires with `approval_required: true`:
1. Runtime emits `plan_approval_requested` immediately after.
2. Session state moves to `awaiting_approval`. Graph dims; approval panel surfaces.
3. UI presents plan title, task list, risks, HITL checkpoints, estimated total time.
4. User chooses:
   - **Approve** → emits `plan_approved { approved_by: 'user' }`; status → `approved`; execution starts.
   - **Revise** → user edits plan inline (or sends back to planner agent with feedback); emits `plan_revised`; status stays `pending_approval`.
   - **Cancel** → emits `plan_aborted`; session continues without a plan.

When `approval_required: false`:
- Runtime emits `plan_approved { approved_by: 'auto' }` immediately. No UI gate. Used by Ralph-style loops.

### Loop policy primitive

The loop policy controls how the runtime executes the task list.

| Policy | Behavior | Used by |
|---|---|---|
| `one_shot` | Plan runs once start-to-finish. No retries beyond `max_failures`. Exit on completion. | Simple scripted workflows |
| `fresh_context_per_task` | Each task spawns a new agent (per `framework.agents[]` definition) with clean context. Prior task summaries passed as input. **ARIA-archetype default.** | examples/aria/ |
| `continuous` | One persistent agent runs. Plan tasks become iteration targets in a `goal_store` file (PRD-style JSON). Agent reads the goal store each iteration, picks the next incomplete item, works on it, updates the store. Loop until store is fully complete. **Ralph-archetype.** | examples/ralph/ |

The runtime ships all three as built-ins. `goal_store` for `continuous` is referenced by path; framework decides format.

### Failure escalation primitive

Per-task failure counter + max threshold:

1. On `task_failed` → `failure_count++`; emit `task_failed`.
2. If `failure_count >= max_failures` → emit `task_escalated`.
3. Runtime invokes the framework's HITL handler (see WI-16) with the task context, last error, and prior attempts.
4. HITL outcomes route back as `task_started` (retry with guidance), `task_skipped`, or `plan_aborted`.

Frameworks can override `max_failures` per task or set a session-wide default (`framework.task_defaults.max_failures`, default 3).

### Graph integration

`PlanNode` (root):
- Renders title, total/completed task count, approval state, current task pointer.
- Click expands to show full task list.

`TaskNode` (children):
- Status (pending/running/done/blocked/failed/skipped) drives color.
- HITL flag rendered as a badge.
- Clicking a task surfaces its hooks, agent, and any failure history.
- Animated edge from PlanNode → currently-running TaskNode.

When `loop_policy: continuous`, `PlanNode` shows the goal-store progress instead of discrete TaskNode children (since tasks are dynamic).

### Framework JSON

```json
{
  "task_defaults": {
    "max_failures": 3,
    "hitl_default": false,
    "post_hooks": [{ "type": "shell", "command": "bash .aria/verify.sh" }]
  },
  "plan_creation": {
    "agent": "planner",
    "approval_required_per_mode": {
      "LITE":     false,
      "STANDARD": true,
      "FULL":     true,
      "FULL+":    true
    },
    "loop_policy_per_mode": {
      "LITE":     { "kind": "one_shot" },
      "STANDARD": { "kind": "fresh_context_per_task" },
      "FULL":     { "kind": "fresh_context_per_task" },
      "FULL+":    { "kind": "fresh_context_per_task" }
    }
  }
}
```

(Mode-keyed overrides above use the §3b mode primitive — added by WI-15.)

`examples/aria/framework.json` uses the structure above. `examples/ralph/framework.json` overrides with `loop_policy: { kind: 'continuous', goal_store: '.ralph/prd.json' }` and `approval_required: false`.

-----

## §3b Mode & Sizing Primitive

> **Locked (2026-04-18, WI-15).** A generic mode primitive: an author-defined enum value, scoped per session, that other primitives reference for overrides. ARIA's LITE/STANDARD/FULL/FULL+ is one realization. Frameworks can define their own.

### Mode field in framework JSON

```json
{
  "modes": {
    "values":  ["LITE", "STANDARD", "FULL", "FULL+"],
    "default": "STANDARD",
    "per_mode_overrides": {
      "LITE": {
        "task_defaults.post_hooks": [],
        "plan_creation.approval_required": false,
        "plan_creation.loop_policy": { "kind": "one_shot" }
      },
      "FULL+": {
        "design_doc_required": true,
        "task_defaults.max_failures": 5
      }
    }
  }
}
```

`per_mode_overrides` uses dotted-path keys; the runtime walks the framework JSON and applies the override for the active mode at session start. Any primitive (hooks, plan, rails, HITL policy, budget) can be mode-overridden.

### Sizing — two paths

Frameworks pick how the mode is determined per session.

#### Path A: Sizing agent

```json
{
  "sizing": {
    "mode":  "agent",
    "agent": "router",                  // an agent in framework.agents[]
    "auto_confirm": false                // if true, skip user confirmation
  }
}
```

The router agent receives the user's request, emits a `propose_mode` tool call with one of `modes.values`, optionally with rationale. Runtime emits `mode_proposed`; user confirms (`mode_confirmed`) unless `auto_confirm: true`.

#### Path B: Declarative sizing rules

```json
{
  "sizing": {
    "mode":  "declarative",
    "rules": [
      { "if": { "tasks_estimated":  { "<=": 5 },  "loc_estimated": { "<": 2000 } }, "then": "LITE" },
      { "if": { "auth_or_payments": true },                                          "then": "FULL" },
      { "if": { "tasks_estimated":  { ">":  40 } },                                  "then": "FULL+" },
      { "default": "STANDARD" }
    ]
  }
}
```

Rules evaluated against a fact set the framework collects (asks user, infers from request, reads project context). Runtime evaluates JSONLogic-style operators. First matching rule wins; `default` catches the rest.

### Session-scoped mode value

Once `mode_confirmed` fires, the value is immutable for the session. Available as `session.mode` to:
- Hook conditions (`when: { "==": [{ "var": "session.mode" }, "FULL"] }`)
- Skill mode_variants (canonical `skill.md` already references `${session.mode}`)
- Plan policy (`plan_creation.approval_required_per_mode`)
- HITL policy (WI-16)
- Budget caps (per-mode caps allowed in §2a)

### Mode events (added to Phase 2 union)

```typescript
| { type: 'mode_proposed';  proposed_mode: string; rationale?: string; agent_id?: string }
| { type: 'mode_confirmed'; mode: string; confirmed_by: 'user' | 'auto' | 'declarative' }
| { type: 'mode_locked';    mode: string }   // emitted once at session start, after confirm
```

### Graph integration

Session header bar shows active mode badge (color-coded per framework's choice). Mode is also rendered on the FrameworkNode as a sub-label.

This is the §0a matrix proof for rows 1 (mode router), 2 (sizing matrix), and 20 (mode-variant skill behavior — combined with §0b skill `mode_variants`).

-----

## Phase 4: Gap Detection and Clean Suspension

The runtime’s job on a gap is to stop cleanly and tell the user exactly what is missing. Nothing more.

### §4b Detection Mechanisms

Three layers, two ship in v1. Each layer emits `tool_missing` or `skill_missing` events; Phase 4's flow below handles both.

#### Layer 1: Static (load-time and spawn-time)

When a framework loads or an agent is spawned, the runtime validates every reference.

| Reference | When checked | Mismatch action |
|---|---|---|
| `framework.skills[]` | Load time | `skill_missing` (warn) — load continues without that skill |
| `framework.tools[]` (built-in or generated) | Load time | `tool_missing` (block) — framework fails to load |
| `framework.agents[].allowed_tools[]` | Spawn time | `tool_missing` (suspend) — that agent cannot start |
| `framework.agents[].allowed_skills[]` | Load time | `skill_missing` (warn) — load continues without that skill |
| `framework.agents[].spawns[]` | Load time | `agent_missing` (block) — schema error, framework fails to load |

Skill-missing is recoverable (continue without, or fetch from registry). Tool-missing is not (the agent that needs it cannot proceed).

#### Layer 2: `request_capability` meta-tool

The runtime auto-injects `request_capability` into every agent's tool list. The model uses it when it realizes mid-task that it needs something not in its toolset.

```typescript
{
  name: "request_capability",
  description:
    "Use when you need a capability (tool or skill) that is not in your toolset. " +
    "Pause your current work and call this rather than improvising.",
  input_schema: {
    type: "object",
    properties: {
      capability_name: { type: "string", description: "Best guess at the name of the missing capability." },
      capability_kind: { type: "string", enum: ["tool", "skill"], description: "tool = callable, skill = instructional context" },
      reason:          { type: "string", description: "Why you need it — what task it would unlock." },
      example_input:   { type: "object", description: "If a tool, an example input you would provide.", nullable: true }
    },
    required: ["capability_name", "capability_kind", "reason"]
  }
}
```

A `request_capability` call translates to `tool_missing` or `skill_missing` based on `capability_kind`. The agent's tool result stays unresolved until the gap is closed (or the user dismisses it). GapNode appears with the agent's stated `reason`.

System prompt addition (added to every agent): *"If you need a capability you don't have, call `request_capability` rather than improvising."*

#### Layer 3: Heuristic (v1.1, deferred)

Pattern: 3+ similar failures on the same sub-task → suggest a missing capability. Requires a classifier (clustering similar errors, mapping to known capability gaps). Useful but error-prone, and depends on registry metadata to make suggestions concrete.

For v1: repeatedly-failing agents hit the failure-escalation primitive (WI-16) and route to HITL. Heuristic gap detection adds on top in v1.1.

### Severity Matrix

| Event | Trigger | Session state | UX |
|---|---|---|---|
| `tool_missing` (static, spawn time) | Agent's `allowed_tools` references unresolved name | Suspended | GapNode on agent edge; "Open Builder to install tool" |
| `tool_missing` (request_capability) | Agent calls request_capability with kind=tool | Suspended | GapNode at agent's current position; agent's reason shown |
| `skill_missing` (static, load time) | Framework references unresolved skill | Loaded with warning | GapNode in graph margin; "Install or continue without" |
| `skill_missing` (request_capability) | Agent calls request_capability with kind=skill | Active, warning toast | GapNode appears, agent continues; user can install async |
| `agent_missing` (static, load time) | `spawns[]` references unresolved agent_id | Cannot start | Error dialog at framework load; user fixes JSON |

### Gap Flow (Runtime)

```
tool_missing or skill_missing event received
  │
  ├── Emit GapNode to graph immediately
  │     - Shows: agent that needed the capability
  │     - Shows: capability name, kind (tool/skill), and the context it was called in
  │     - Shows: last known agent output before the gap
  │
  ├── If kind=tool OR static-load-blocking:
  │     ├── Drone: snapshot_now (reason: 'gap_tool_missing')
  │     └── Session moves to 'suspended' state
  │
  ├── If kind=skill (recoverable):
  │     ├── No snapshot needed
  │     ├── Session stays 'active' with warning toast
  │     └── VDR records the gap event
  │
  └── Gap Panel opens in UI
        - "Missing <kind>: <name>"
        - "Go to Agent Builder to find or create it"
        - "Resume" button — active once installed (suspend case)
        - "Continue without" button (recoverable case)
        - Full VDR trace up to gap available
```

### Gap Resolution

When the user installs the missing capability via the Builder:
1. Runtime re-validates the framework references.
2. Emits `gap_resolved { capability_name, capability_kind }`.
3. If session was suspended: drone reloads from snapshot, agent's pending tool result is delivered (for `request_capability`) or the agent is respawned (for static `tool_missing`).
4. If session was active: GapNode collapses to a normal SkillNode/ToolNode and the warning clears.

### Gap Panel UX

```
Gap Panel (replaces graph chrome, graph visible underneath dimmed)
  - Capability name, kind (tool|skill), and description of what was needed
  - Agent context — what the agent was trying to accomplish (request_capability.reason)
  - Link: "Open Agent Builder" (switches to build mode)
  - Session state preserved — graph will reconstruct on resume
  - Option: "Resume anyway without this capability" (marks session as degraded; tool case only allowed if framework explicitly opts in)
```

### GapNode in Graph

```
GapNode
  - Amber pulsing ring (skill kind) or red pulsing ring (tool kind)
  - Shows capability name and kind
  - Shows which agent hit the gap
  - Edge from agent to GapNode animated in red (tool) or amber (skill)
  - On resolution: GapNode replaced by newly installed ToolNode or SkillNode
```

-----

## §4a Verify & Rails Primitives

> **Locked (2026-04-18, WI-02).** The runtime ships hook and rail primitives. Frameworks compose them into pipelines like ARIA's `verify.sh` (5-layer verification) and `rails/safety.json` (hard/soft blocks). The runtime does not bundle test runners, linters, or specific rule sets — those come from the framework via hooks.

### Hook primitive

Hooks fire on lifecycle events. Each hook is a typed reference the runtime knows how to invoke.

```typescript
type HookRef =
  | { type: 'shell'; command: string; timeout_ms?: number; cwd?: string }
  | { type: 'tool';  tool_name: string; input?: Record<string, unknown> }
  | { type: 'agent'; agent_id: string; prompt?: string }

type HookCategory =
  | 'verify'      // post-task or post-edit verification
  | 'lint'        // style / static analysis
  | 'build'       // compilation / packaging
  | 'test'        // unit / integration / e2e
  | 'custom'      // anything else; UI shows generic outcome

interface Hook {
  id: string
  category: HookCategory
  level?: 'quick' | 'standard' | 'full'   // optional grouping
  ref: HookRef
  on_failure: 'block' | 'warn' | 'rollback'
}
```

Hook firing points (in `framework.hooks`):

| Field | Fires when | Typical use |
|---|---|---|
| `pre_task` | Before each task starts | Pre-flight checks, setup |
| `post_task` | After each task completes (success or fail) | **Verify pipeline (ARIA `verify.sh`)** |
| `post_file_edit` | After any agent writes a file | Lint, format on save |
| `pre_commit` | Before any git commit | Secret scan, hook chain |
| `pre_agent_spawn` | Before a child agent spawns | Capability narrowing check, env prep |
| `session_end` | When a session terminates | Report generation, cleanup |

```json
"hooks": {
  "post_task": [
    { "id": "verify", "category": "verify", "level": "standard",
      "ref": { "type": "shell", "command": "bash .aria/verify.sh", "timeout_ms": 300000 },
      "on_failure": "rollback" }
  ],
  "post_file_edit": [
    { "id": "lint", "category": "lint",
      "ref": { "type": "tool", "tool_name": "lint_changed_files" },
      "on_failure": "warn" }
  ]
}
```

### Hook events (added to Phase 2 union)

```typescript
| { type: 'hook_started'; hook_id: string; category: HookCategory; firing_point: string; ref: HookRef }
| { type: 'hook_passed';  hook_id: string; duration_ms: number; output_preview?: string }
| { type: 'hook_failed';  hook_id: string; duration_ms: number; error: string; on_failure: 'block' | 'warn' | 'rollback' }
```

### Rails primitive

Rails are policy checks declared in framework JSON. The runtime ships a rails-evaluator that runs them at the appropriate firing points.

```json
"rails": {
  "hard": [
    { "id": "no_secrets",
      "fires_on": ["pre_commit", "post_file_edit"],
      "check": { "type": "shell", "command": "scripts/check-secrets.sh" },
      "message": "Secrets detected in staged changes; commit blocked." },
    { "id": "no_env_files",
      "fires_on": ["pre_commit"],
      "check": { "type": "tool", "tool_name": "scan_for_env_files" },
      "message": "Cannot commit .env files." }
  ],
  "soft": [
    { "id": "no_debug",
      "fires_on": ["post_file_edit"],
      "check": { "type": "shell", "command": "scripts/scan-debug.sh" },
      "message": "Debug statements found; consider removing before commit." }
  ]
}
```

`hard` rails block; `soft` rails warn. Rails evaluator emits `rail_triggered` events per evaluation.

```typescript
| { type: 'rail_triggered'; rail_id: string; policy: 'hard' | 'soft'; firing_point: string; message: string; agent_id?: string }
```

### Don't-touch primitive

Pre-edit rail built into the runtime. Framework JSON declares glob patterns; any agent attempting to write a matching path triggers a hard rail.

```json
"dont_touch": [
  ".aria/state/**",
  "package-lock.json",
  ".env*"
]
```

Implemented as a built-in hook on `pre_file_edit` (new firing point) with `on_failure: block`. Emits `rail_triggered { rail_id: 'dont_touch', policy: 'hard' }`.

### Rollback integration

When a hook with `on_failure: rollback` fails, the runtime invokes the drone's `revert_to_snapshot` command targeting the snapshot taken at the most recent `task_started` boundary. Snapshots are already taken there per Phase 1.

Drone command surface added:

```typescript
| { type: 'revert_to_snapshot'; snapshot_id: string; reason: 'hook_rollback' | 'user_rollback' | 'gap_recovery' }
```

After rollback, runtime emits `task_failed` with `error: 'rolled_back_after_hook_<hook_id>'` and the failure-escalation path (§3a) takes over.

### Graph integration

`VerifyNode` (specialization for `category: verify`):
- Renders inline at the relevant TaskNode boundary.
- Pass = green; warn = amber; fail = red; rollback = red with rollback icon.
- Click for full output and exit code.

Other categories (`lint`, `build`, `test`, `custom`) render as generic `HookNode` with category badge.

### Framework JSON example (assembling the primitives)

`examples/aria/framework.json` excerpt reconstructing ARIA's verify + rails + dont_touch:

```json
{
  "task_defaults": {
    "max_failures": 3,
    "post_hooks": [
      { "id": "verify_standard", "category": "verify", "level": "standard",
        "ref": { "type": "shell", "command": "bash .aria/verify.sh" },
        "on_failure": "rollback" }
    ]
  },
  "hooks": {
    "post_file_edit": [
      { "id": "lint", "category": "lint",
        "ref": { "type": "shell", "command": "npm run lint" },
        "on_failure": "warn" }
    ]
  },
  "rails": {
    "hard": [
      { "id": "no_secrets", "fires_on": ["pre_commit"],
        "check": { "type": "shell", "command": "scripts/check-secrets.sh" },
        "message": "Secrets detected." }
    ],
    "soft": [
      { "id": "no_debug", "fires_on": ["post_file_edit"],
        "check": { "type": "shell", "command": "scripts/scan-debug.sh" },
        "message": "Debug statements found." }
    ]
  },
  "dont_touch": [".aria/state/**", "package-lock.json", ".env*"]
}
```

This is the §0a matrix proof for rows: `verify.sh after every task` (3), `Hard/soft rails` (4), `Project-context don't touch zones` (17), `Git checkpoint/rollback` (13).

-----

## Phase 5: MCP Manager

Visible in the graph. Every connected MCP server is a node. Tool calls route through it as animated edges.

> **Scope (per §0b):** MCP exposes **Tools only**. MCP servers cannot publish Skills or Agents. Skills and Agents are owned by the framework / local library / generators.

### Manager Capabilities

```
Connect / Disconnect
  - Add MCP server by URL or local path
  - Test connection before activating
  - Show connection status in graph node

Tool Discovery
  - On connect, enumerate available tools
  - Surface tool list in framework loader and agent builder
  - Available tools visible in Agent Builder palette

Health Monitoring
  - Drone pings MCP servers on heartbeat cycle
  - Dead server → node goes offline in graph → auto-retry with backoff
  - Configurable retry policy per server

Multi-Server
  - Multiple MCP servers active simultaneously
  - Tool namespace resolution per §5a (below)
  - Per-server auth config (stored in secrets vault, not in session state)
```

### §5a Tool Namespace Resolution

> **Locked (2026-04-18, WI-11).** Algorithm for resolving tool names across multiple connected MCP servers.

1. **Canonical name:** every tool is exposed to agents as `<server_name>__<tool_name>` (double-underscore delimiter).
2. **Short-name alias:** if a tool name is unambiguous across all currently-connected servers, the short name also resolves. Ambiguous → short name fails with a clear error listing the canonical options.
3. **Explicit override** in framework JSON via `mcp_aliases`:
   ```json
   "mcp_aliases": {
     "create_component": "react-mcp__create_component",
     "extract_text":     "pdf-mcp__extract_text"
   }
   ```
   Aliases override short-name ambiguity errors.
4. **Server name constraints:** server names cannot contain `__`. Tools may contain `__`; the parser splits on the first `__` from the left.
5. **Re-resolution on connect/disconnect:** when an MCP connects/disconnects, runtime re-evaluates short-name uniqueness. Newly-ambiguous short names emit a `tool_alias_ambiguous` warning event so frameworks can pin via `mcp_aliases`.

Example:
- Connected: `pdf-mcp` (exposes `extract_text`), `image-mcp` (exposes `extract_text`)
- Agent calls `extract_text` → ambiguous → fails with: *"Tool `extract_text` is ambiguous. Candidates: `pdf-mcp__extract_text`, `image-mcp__extract_text`. Use the canonical name or set an alias in `mcp_aliases`."*
- Framework adds `"extract_text": "pdf-mcp__extract_text"` → short name resolves to PDF.

### MCP Node in Graph

```
MCPNode shows:
  - Server name and URL
  - Connection status (live indicator)
  - Active tool calls (animated edges from agent nodes)
  - Tool call count and avg latency (hover detail)
  - Error rate (color shifts amber/red if degraded)
```

-----

## Phase 6: Framework JSON Loader

Frameworks are portable JSON files that define agent behavior. They load into the runtime without code changes.

### Framework Schema

```json
{
  "name": "aria",
  "version": "1.0.0",
  "description": "UI prototyping and component generation framework",
  "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
  "system_prompt": "You are Aria...",
  "tools": [
    {
      "name": "create_component",
      "description": "Creates a React component",
      "input_schema": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "description": { "type": "string" }
        },
        "required": ["name", "description"]
      }
    }
  ],
  "agents": [
    {
      "id": "planner",
      "name": "Planning Agent",
      "role": "Breaks tasks into component steps",
      "spawns": ["builder", "reviewer"]
    }
  ],
  "skills": ["create_component", "write_tests", "review_code"],
  "hitl_policy": { /* see §6a — structured object, not a single string */ },
  "decision_trace": true,
  "session": {
    "snapshot_interval_seconds": 30,
    "idle_timeout_seconds": 300,
    "stall_timeout_seconds": 60
  }
}
```

### Framework Manager

```
Load — upload JSON, validate schema, preview in UI before activating
Switch — swap active framework between sessions (snapshot before switch)
Version — frameworks are versioned, rollback available
Share — export framework JSON for distribution
Examples — `examples/aria/` ships as the reference framework that reconstructs ARIA's capabilities (see §0). The runtime has no built-in default framework
```

-----

## §6a HITL Policy Primitive

> **Locked (2026-04-18, WI-16).** HITL is a structured object with multiple trigger types and a notifier plugin interface — not a single string value.

ARIA's HITL fires for: gaps, destructive operations, risky tools, per-epic gates, failure escalation, budget thresholds, capability violations. The runtime needs primitives for each.

### Framework JSON

```json
"hitl_policy": {
  "on_gap":                  { "enabled": true,  "ui": "panel" },
  "on_risky_tool":           { "enabled": true,  "tools": ["Bash:rm", "Bash:git push", "WebFetch:*"], "ui": "modal" },
  "on_dont_touch_edit":      { "enabled": true,  "ui": "modal" },
  "on_failure_threshold":    { "enabled": true,  "threshold": 3, "ui": "panel" },
  "on_capability_violation": { "enabled": true,  "ui": "modal" },
  "on_budget_threshold":     { "enabled": true,  "percent": 90, "ui": "modal" },
  "on_plan_approval":        { "enabled": true,  "ui": "panel" },
  "per_task":                { "enabled": false },
  "per_epic":                { "enabled": false },

  "notifiers": [
    { "type": "terminal_bell", "enabled": true },
    { "type": "desktop",       "enabled": true },
    { "type": "sound",         "enabled": false },
    { "type": "plugin",        "name": "slack-webhook", "config": { "url_secret_ref": "secret://slack_webhook" } }
  ],

  "timeout_seconds": 3600,
  "default_action_on_timeout": "abort"
}
```

### HITL trigger types (locked)

| Trigger | Fires when | Per-mode default | UI variant |
|---|---|---|---|
| `on_gap` | `tool_missing` or `skill_missing` (Phase 4 / §4b) | All modes: enabled | Panel (full takeover) |
| `on_risky_tool` | Agent attempts a tool listed in `tools` | All modes: enabled | Modal (approve/deny) |
| `on_dont_touch_edit` | Agent attempts to edit a `dont_touch` path (§4a) | All modes: enabled | Modal (approve/deny once or always) |
| `on_failure_threshold` | `task_escalated` (§3a, after `failure_count >= max_failures`) | All modes: enabled | Panel (retry with guidance / skip / abort) |
| `on_capability_violation` | Phase 8 §8.security L2 violation | All modes: enabled | Modal (allow once / session / forever / block) |
| `on_budget_threshold` | `budget_suspended` (§2a, configurable %) | All modes: enabled | Modal (approve continue / reduce cap / abort) |
| `on_plan_approval` | `plan_approval_requested` (§3a) | LITE: disabled; STANDARD/FULL/FULL+: enabled | Panel (approve/revise/cancel) |
| `per_task` | Before each task | LITE/STANDARD/FULL: disabled; FULL+: optional | Modal |
| `per_epic` | Between epics | FULL+: optional | Panel |

### UI variants

- **Panel:** takes over the main view, dims graph, requires explicit interaction. Used for substantial decisions.
- **Modal:** floating dialog, blocks adjacent interaction, dismissed by approve/deny. Used for quick yes/no.
- **Toast:** non-blocking notification, auto-dismisses with default action after timeout. Used for soft warnings (not currently any HITL trigger uses toast).

### Notifier plugin interface

Notifiers are plugins called when any enabled HITL trigger fires.

```typescript
interface HitlNotifier {
  readonly type: string
  notify(event: HitlNotifyEvent): Promise<void>
}

interface HitlNotifyEvent {
  trigger: string                    // e.g., 'on_failure_threshold'
  session_id: string
  question: string                    // human-readable summary
  options: string[] | null            // expected user choices, if any
  context: Record<string, unknown>
  timeout_at: number                  // unix ms
}
```

Built-in notifiers in v1:
- `terminal_bell` — emits BEL to active terminal (works in all OSes)
- `desktop` — Tauri's notification plugin (cross-platform: native macOS/Windows/Linux notifications)
- `sound` — play a wav from settings; off by default

Plugin notifiers (Slack, email, custom) loaded from `notifiers/` directory. Plugins follow the same Phase 8 §8.security model: declared capabilities, sandboxed validation, tier-gated install.

### Failure escalation flow (cross-ref §3a)

When `task_escalated` fires:

1. HITL trigger `on_failure_threshold` evaluated. If enabled, runtime emits `hitl_requested` with task context, last error, attempts.
2. All enabled notifiers called in parallel.
3. Session waits for response (default 1h timeout, then `default_action_on_timeout`).
4. User response routes back as one of:
   - `task_started` (retry with guidance)
   - `task_skipped`
   - `plan_aborted`
5. Decision is recorded in VDR with full context.

### HITL events (already in Phase 2 union, unchanged structurally)

`hitl_requested`, `hitl_response` already exist; this WI adds:

```typescript
| { type: 'hitl_timeout'; trigger: string; session_id: string; default_action: string }
| { type: 'notifier_dispatched'; notifier_type: string; trigger: string; success: boolean }
| { type: 'notifier_failed'; notifier_type: string; trigger: string; error: string }
```

### Mode-keyed defaults

Frameworks set per-mode HITL policy overrides via §3b's `per_mode_overrides`:

```json
"per_mode_overrides": {
  "LITE":  { "hitl_policy.on_plan_approval.enabled": false },
  "FULL+": { "hitl_policy.per_epic.enabled": true }
}
```

This is the §0a matrix proof for rows 14 (HITL notifications) and 18 (failure escalation).

-----

## Phases 7–9: Agent Builder (Build Time)

The Agent Builder is a distinct mode — not part of the runtime. The user switches to it deliberately, either from the gap panel or from the main nav. Nothing built here affects a running session until the user explicitly resumes.

### Phase 7: Registry Search and Capability Finder

> **v1 scope (locked 2026-04-18, WI-12):** one trusted upstream + local library only. Pluggable community registries deferred to v2.

#### v1 Sources

```
Search Sources (searched in order)
  ├── Local library — installed artifacts on this machine (always searched first)
  └── Anthropic Skills upstream — https://github.com/anthropics/skills
       (single trusted, hash-pinned, signed-by-repo-HEAD)

NOT in v1:
  ✗ Pluggable community registries (deferred to v2)
  ✗ mcp.so/api/search (deferred to v2 — no signature/trust chain yet)
  ✗ Arbitrary GitHub URLs (deferred to v2 — no provenance)
```

#### Trust chain for v1 upstream

1. Anthropic skills repo URL is hardcoded as a constant.
2. On first connect, runtime fetches the repo HEAD commit hash via the GitHub API. That hash becomes the trust root for this install.
3. Each artifact retrieved from the upstream includes its content hash. Runtime verifies hash on download.
4. Trust root rotates on user-initiated "Update upstream" action; user sees the old/new HEAD hashes and confirms.

#### `skills.lock` file

Every installed artifact records to `.aria-runtime/skills.lock`:

```json
{
  "version": 1,
  "installed": {
    "pdf_summarizer@1.0.0": {
      "kind": "skill",
      "source": "anthropic-upstream",
      "source_commit": "abc123def456...",
      "content_hash": "sha256:def456...",
      "installed_at": "2026-04-18T14:23:00Z",
      "validation_report_id": "vr-789xyz",
      "tier_at_install": "promoted"
    }
  }
}
```

Runtime validates `content_hash` on every load. Mismatch → block load with `artifact_hash_mismatch` event; user reinstalls or removes.

`skills.lock` is checked into version control alongside the framework JSON, enabling reproducible installs across machines.

#### Search interface

- Natural language query → upstream's pre-built index (also fetched and hash-verified).
- Results show: name, description, type (`tool | skill | agent`), source, content_hash, declared capabilities (from L1 — see Phase 8).
- Preview shows full artifact + capabilities + author from the upstream commit.
- Install runs full Phase 8 §8.security validation (L1–L5) and updates `skills.lock`.

#### Validation before install

Per Phase 8 §8.security — same five layers apply to registry installs as to generated artifacts. Hash check (above) is in addition.

#### Result type

```typescript
interface RegistrySearchResult {
  name: string
  description: string
  type: 'tool' | 'skill' | 'agent'
  source: 'local' | 'anthropic-upstream'
  source_commit: string
  content_hash: string
  capabilities: CapabilityBlock        // from L1 disclosure
  preview_url: string
  author: string
}
```

#### Deferred to v2

- Pluggable registry config (user-configurable trust roots)
- Sigstore-style cryptographic signatures (currently rely on Git commit signing)
- Reputation / community ratings
- Revocation lists for known-bad artifacts
- Multi-registry namespace resolution

These are tracked under WI-22.

### Phase 8: Generators (Tool Writer / Skill Writer / Agent Composer)

When nothing exists in registries, the builder generates it. Per §0b, three distinct generators correspond to the three concepts.

> **Output scope (locked, see §0b and WI-06):** generators emit declarative artifacts only — never executable code. Tool Writer outputs MCP-binding configurations. Skill Writer outputs instruction-set markdown. Agent Composer outputs framework JSON entries composing existing tools + skills.

#### §8.security Five-Layer Security Model

All generators share one security model. Every layer applies to every artifact, regardless of generator type or user tier.

| Layer | Purpose | Enforcement | Pattern source |
|---|---|---|---|
| **L1: Capability Disclosure** | User sees declared capabilities at install in plain English | Mandatory in artifact frontmatter; validator rejects artifacts missing it | Browser permissions, Deno `--allow-*`, Chrome extension manifest |
| **L2: Capability Enforcement** | Artifact cannot exceed declared capabilities at runtime | Runtime intercepts every tool call / skill load / file access from the artifact and checks against its declared `capabilities` block | Deno permission model, WASM sandbox, capability-based security |
| **L3: Sandboxed Validation** | Artifact is exercised against mock inputs before install | Dedicated sandbox process spawned by drone; validator runs declared examples + adversarial inputs; result attached to artifact metadata | npm prepublish, Vercel preview deploys |
| **L4: Tiered Human Gate** | Default-safe install path with promotion | Three install tiers (Novice / Promoted / Operator); see below | Dependabot auto-merge, browser permission "remember this decision" |
| **L5: Provenance & Audit** | Immutable record of generation lineage and install decisions | Every artifact carries `provenance` block; every install/reject/uninstall logged to `skills.audit.jsonl` | npm provenance, Sigstore, SLSA |

##### L1: Capability Disclosure (mandatory frontmatter block)

Every generated `tool.md`, `skill.md`, and agent JSON entry must include a `capabilities` block. The validator rejects artifacts missing it.

```yaml
capabilities:
  tools_called:    ["WebFetch", "Read"]              # other tools this artifact will invoke
  skills_loaded:   ["debugging"]                     # other skills this artifact will load
  file_access:     { read: ["src/**"], write: [] }   # glob patterns
  network:         ["api.example.com"]               # allowed hosts; "*" requires Operator tier
  shell:           false                              # true requires Operator tier
  spawn_agents:    []                                 # which child agents this can spawn
```

Builder UI translates this to plain English at install time:

> *This skill will: read files matching `src/**`; call the `WebFetch` tool against `api.example.com`; load the `debugging` skill. It will NOT: write files, run shell commands, or access the network beyond `api.example.com`.*

##### L2: Capability Enforcement (defense in depth)

The runtime maintains a per-artifact capability set loaded from L1. Every operation initiated by the artifact passes through **two enforcement layers**:

**L2a — Application-level check (Rust main process)**

Before any operation is dispatched to a tool or sandbox:

- Tool call → check `tools_called` includes the target.
- Skill load → check `skills_loaded` includes the target.
- File read/write → check glob patterns in `file_access`.
- Network access (via WebFetch / generated tool that calls out) → check host in `network`.
- Shell access (via Bash) → check `shell == true`.
- Agent spawn → check `spawn_agents` includes the target.

**L2b — OS-level enforcement (Tauri allowlist + sandbox process)**

The runtime exploits the Tauri + Rust + per-artifact-sandbox stack to enforce capabilities at the OS boundary, not just in application code:

1. **Tauri allowlist** — the renderer cannot reach any backend command not explicitly allowlisted in `tauri.conf.json`. There is no Node API in the renderer, so prompt-injection-driven `eval` or shell-out is structurally impossible.
2. **Per-artifact sandbox process** — when an artifact with `shell: true` or `network: [...]` runs, the drone spawns a dedicated `runtime-sandbox` child process. The sandbox process is launched with OS-level restrictions:
   - **Linux:** seccomp-bpf syscall allowlist + landlock filesystem restrictions + namespaces (mount, network, PID).
   - **macOS:** sandbox-exec profile derived from declared capabilities.
   - **Windows:** Job Objects + AppContainer + restricted token.
3. **File access enforcement** — even at L2a, file paths are normalized and glob-matched; symlink escape attempts are detected and blocked.
4. **Network enforcement** — `WebFetch` and generated network-bound tools route through a single Rust HTTP client that consults the artifact's `network` allowlist before issuing any request. DNS pinning prevents allowlist-bypass via DNS rebinding.

**Capability violation handling**

- Emits `capability_violation` event with `{ artifact, attempted, declared, layer: 'l2a' | 'l2b' }`.
- Blocks the operation at whichever layer caught it.
- Surfaces a HITL prompt: "Skill `<name>` attempted `<operation>` not in its declared capabilities. Allow once / Block / Open Builder to update."
- VDR records the attempt regardless of user choice.
- L2b violations (the artifact bypassed L2a and was caught by the OS) are flagged at higher severity — the artifact attempted something it declared it would not. Triggers `tier_changed` audit entry; the artifact is automatically demoted to "review-required" status regardless of user tier.

> **Why this matters for OSS scrutiny.** L2 in Electron is best-effort — V8 isolates and worker_threads aren't real sandboxes. L2 in Tauri/Rust delivers what it promises: an artifact that declares `shell: false` literally cannot invoke a shell, because the sandbox process has no shell binary in its filesystem view and the seccomp filter blocks `execve`. This is what makes "auto-accept tested" actually safe.

##### L3: Sandboxed Validation (always-on)

Drone spawns a dedicated sandbox process for validation. Validator runs:

1. **Schema check** — frontmatter parses, required fields present, capability block present.
2. **Declared-examples run** — every example in the artifact is exercised. Outputs must match `output_schema`.
3. **Capability-bound execution** — same L2 check applied to the sandbox. If the artifact attempts anything outside its declared capabilities during validation, validation fails (artifact is lying about what it does).
4. **Adversarial inputs (skills/tools)** — empty input, oversize input, inputs with prompt-injection patterns. Outputs logged but not blocking unless they cause crashes.
5. **Static red flags** — known-bad patterns in skill body or tool config (literal API keys, shell-out instructions, `eval`-style patterns). Hard block.

Validator output is attached to the artifact as `validation_report` and surfaced in the Builder UI.

##### L4: Tiered Human Gate

Three tiers. User starts at Novice; promotion requires explicit opt-in with warning.

| Tier | Default for | Install gate | Auto-accept criteria |
|---|---|---|---|
| **Novice** | New users; first 5 installs | Manual review of capabilities + diff + validation report; explicit "Install" click | None — every install is reviewed |
| **Promoted** | Users who toggled "auto-accept tested artifacts" in settings (with one-time warning explaining risks) | Auto-install if **all** of: validation passed; capabilities don't include `shell:true` or `network:["*"]`; not from an untrusted registry | Generated and validated artifact within Promoted-allowed capability bounds |
| **Operator** | Power users who explicitly enabled (with stronger warning) | Auto-install permitted for any capability set; only L2 enforcement and L3 validation gate | Anything that passes L3 |

Promotion is sticky but reversible. Tier changes are audit-logged (L5).

**Forbidden in all tiers:**
- Auto-install of an artifact that fails L3 validation.
- Auto-install of an artifact whose declared capabilities exceed what the validator could verify.
- Bypassing L2 enforcement at runtime.

##### L5: Provenance & Audit

Every generated artifact carries a `provenance` block in frontmatter:

```yaml
provenance:
  generator: "skill_writer"                          # tool_writer | skill_writer | agent_composer
  model: "claude-opus-4-7"
  prompt_hash: "sha256:abc123..."                    # hash of generation prompt; full prompt in audit log
  generated_at: "2026-04-18T14:23:00Z"
  validated_at: "2026-04-18T14:23:42Z"
  validation_report_id: "vr-789xyz"
  content_hash: "sha256:def456..."                   # hash of post-validation artifact
  signature: "ed25519:..."                            # runtime-signed; key per-installation
```

Every install / reject / uninstall / capability-violation / tier-change is appended to `.aria-runtime/skills.audit.jsonl`:

```json
{
  "id": "audit-1714512345",
  "timestamp": "2026-04-18T14:23:50Z",
  "event": "install",
  "artifact_kind": "skill",
  "artifact_name": "pdf_summarizer",
  "artifact_version": "1.0.0",
  "content_hash": "sha256:def456...",
  "tier_at_install": "promoted",
  "tier_gate": "auto_accept_tested",
  "validation_report_id": "vr-789xyz",
  "user_decision": "auto_accepted",
  "provenance": { ... }
}
```

Audit log is append-only, hash-chained (each entry includes the hash of the previous entry). Redaction rule: prompts and tool inputs containing detected secrets are replaced with `[REDACTED:<reason>]` before logging; original kept in encrypted local-only store accessible from Builder.

#### Threat Model

What v1 defends against:

- **(a) Malicious model output** — prompt injection or model hallucination producing a trojan skill. Defenses: L2 (declared capabilities are enforced), L3 (validator catches red flags), L4 Novice tier (mandatory review). A trojan skill can declare capabilities matching its trojan behavior, but a Novice user reviewing the capability block will see, in plain English, what it will do. Auto-accept tiers refuse network:* and shell:true.
- **(b) Compromised registry** — upstream serves a poisoned skill. Defenses: hash-locked installs (WI-12), L3 sandbox runs every install regardless of source, L5 provenance flags non-runtime-signed artifacts.
- **(c) User error** — user installs a known-bad skill. Defenses: Novice tier review + capability disclosure forces user to read what they're installing; Promoted tier blocks dangerous capability sets; deny-list of known-bad patterns hard-blocks regardless of tier.

What v1 does NOT defend against (out of scope):

- Operator tier user knowingly installing a known-bad artifact.
- Skills attempting prompt injection on the next agent (mitigation: skill bodies are loaded as user-content blocks, not system prompts; hardening is Phase 4 / runtime-wide concern, not generator concern).
- Attacks on the runtime binary itself (signed releases, OS-level concern).

#### Generator-specific surface

##### §8a Tool Writer

Outputs `tool.md` declaring an MCP-binding, never executable code.

```markdown
---
name: read_pdf
version: 1.0.0
description: Extract text from PDF files
provenance: { ... }
mcp_binding:
  server: "pdf-mcp@1.0"
  tool: "extract_text"
  argument_mapping:
    file_path: "$.input.path"
input_schema: { ... }
output_schema: { ... }
capabilities:
  tools_called: []
  skills_loaded: []
  file_access: { read: ["**/*.pdf"], write: [] }
  network: []
  shell: false
---

## Description
...
## Examples
...
```

Validator additionally checks: MCP server is reachable, declared tool exists on server, argument mapping is valid against MCP tool schema.

##### §8b Skill Writer

Outputs canonical `skill.md` per §0b. Never executable code; the body is markdown instructional context loaded into agent prompts via `LoadSkill`.

Modes:
- **Collaborative** — iterative HITL: user describes need, model proposes, user reviews each iteration, approved version validated and installed per L4 tier.
- **Autonomous** — single-shot: user provides intent + constraints; model generates; validator runs; per L4 tier either auto-installs (Promoted/Operator within bounds) or queues for review (Novice).

Skill writer cannot emit a skill that declares `tools_called` or `skills_loaded` for items not in the local library or registry — references must resolve at generation time. Else: validation fails.

##### §8c Agent Composer

Outputs framework JSON entries composing existing tools + skills + child agents. No new code, only composition.

Composer enforces capability *narrowing*: a child agent's declared capabilities cannot exceed the parent's. Parent that lacks `network` cannot spawn a child that has `network`.

#### Builder UI integration

Each generator runs inside the Builder. Generator UI flow:

```
1. User describes intent (natural language)
2. Generator produces draft + capability block
3. Validator runs (L3) — drone-managed sandbox, isolated
4. Builder displays:
     - Plain-English capability disclosure (from L1)
     - Validation report (L3) — pass/fail per check
     - Diff view (vs existing version if any)
     - Provenance block (L5)
5. Tier-appropriate gate (L4):
     - Novice → "Install" button + review checklist
     - Promoted → auto-installs if within bounds; surfaces toast with link to artifact
     - Operator → auto-installs; surfaces toast
6. On install: artifact + provenance + validation report committed; audit entry written
```

Auto-accept toast example (Promoted tier):

> ⚡ Installed `pdf_summarizer` v1.0.0 (auto-accepted: validation passed; declared capabilities within Promoted bounds). [Review] [Uninstall]

### Phase 9: Visual Canvas and Tester

**Builder Canvas**

```
Palette (left sidebar)
  - Agent types
  - Installed skills (local library)
  - Available MCP tools
  - HITL nodes

Canvas (center)
  - Drag palette items onto canvas
  - Connect nodes with edges (defines spawn and tool relationships)
  - Configure each node inline
  - Live preview of generated framework JSON (right sidebar)
  - Export as framework JSON

Tester
  - Load a framework
  - Define a test task (natural language)
  - Run in sandboxed session (drone managed, isolated)
  - Watch graph render
  - Review VDR output
  - Check token spend and timing
  - Pass / fail with full trace
  - Does not affect any real session or data
```

-----

## Persistence Layer

All state lives in SQLite. Schema:

```sql
-- Sessions
CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  framework_name TEXT,
  framework_version TEXT,
  model TEXT,
  started_at INTEGER,
  last_active INTEGER,
  status TEXT,  -- active | suspended | complete | crashed | recovered | budget_exceeded
  mode TEXT     -- active mode value (see §3b)
  -- snapshot_count derivable via COUNT(*) FROM snapshots WHERE session_id = ?
);

-- Snapshots (drone written)
CREATE TABLE snapshots (
  id TEXT PRIMARY KEY,
  session_id TEXT,
  timestamp INTEGER,
  event_type TEXT,
  state_json TEXT,  -- full session state
  state_hash TEXT,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Verified Decision Records
CREATE TABLE vdr (
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
  FOREIGN KEY (session_id) REFERENCES sessions(id),
  FOREIGN KEY (snapshot_id) REFERENCES snapshots(id)
);

-- Token/Cost Tracking
CREATE TABLE token_usage (
  id TEXT PRIMARY KEY,
  session_id TEXT,
  agent_id TEXT,
  timestamp INTEGER,
  model TEXT,
  input_tokens INTEGER,
  output_tokens INTEGER,
  cost_usd REAL
);

-- Installed Skills
CREATE TABLE skills (
  id TEXT PRIMARY KEY,
  name TEXT,
  version TEXT,
  source_url TEXT,
  installed_at INTEGER,
  validated INTEGER,
  skill_md TEXT
);

-- MCP Servers
CREATE TABLE mcp_servers (
  id TEXT PRIMARY KEY,
  name TEXT,
  url TEXT,
  auth_key_ref TEXT,  -- reference to secrets vault, never store key here
  added_at INTEGER,
  last_connected INTEGER,
  status TEXT
);
```

-----

## Secrets Vault

Keys, tokens, credentials stored separately from session state. Never written to snapshots. Never logged in VDR.

```
Storage: OS keychain via the `keyring` crate (cross-platform: macOS Keychain, Windows Credential Manager, GNOME Keyring / KWallet via secret-service)
Access: Main process only, never renderer
API keys: Per model provider
MCP auth: Per server
Git credentials: For repo-based skill sources
```

-----

## Session Recovery UX

On platform launch with an interrupted session:

```
Recovery Dialog (not a modal — full panel)
  - Session name, framework, when it was interrupted, reason
  - Last stable snapshot timestamp
  - "Resume from snapshot" — rebuilds graph, reconnects MCPs, restores message history
  - "Inspect only" — read-only view of session state and VDR trace
  - "Discard" — archive and start fresh
```

Recovery is never destructive. Discarded sessions are archived, not deleted.

-----

## Project Structure

```
/
├── Cargo.toml                  # Workspace root
├── Cargo.lock                  # Committed
├── rust-toolchain.toml         # Pin Rust version for reproducibility
│
├── crates/
│   ├── runtime-core/           # Shared domain types
│   │   ├── src/lib.rs
│   │   ├── src/event.rs        # AgentEvent enum (canonical)
│   │   ├── src/framework.rs    # Framework JSON schema types
│   │   ├── src/capability.rs   # Capability declaration + enforcement types
│   │   └── src/signal.rs       # Signal Schema v2 types
│   │
│   ├── runtime-main/           # Tauri main process
│   │   ├── src/main.rs
│   │   ├── src/sdk/agent_sdk.rs
│   │   ├── src/sdk/event_pipeline.rs
│   │   ├── src/sdk/vdr_logger.rs
│   │   ├── src/providers/anthropic.rs       # Direct HTTP+SSE
│   │   ├── src/providers/mod.rs              # LLMProvider trait
│   │   ├── src/mcp/manager.rs
│   │   ├── src/mcp/client.rs
│   │   ├── src/framework/loader.rs
│   │   ├── src/framework/validator.rs
│   │   ├── src/builder/registry.rs           # Build-time
│   │   ├── src/builder/skill_writer.rs
│   │   ├── src/builder/validator.rs
│   │   ├── src/db/schema.rs
│   │   ├── src/db/session_store.rs
│   │   └── src/capability/enforcer.rs        # L2a application-level
│   │
│   ├── runtime-drone/          # Drone binary
│   │   ├── src/main.rs         # Entry: --session-id --db-path --ipc-socket
│   │   ├── src/protocol.rs     # DroneEvent / DroneCommand
│   │   ├── src/heartbeat.rs
│   │   ├── src/snapshot.rs
│   │   ├── src/recovery.rs
│   │   └── src/process_manager.rs
│   │
│   └── runtime-sandbox/        # Per-artifact sandbox host (L2b OS-level)
│       ├── src/main.rs
│       ├── src/linux.rs        # seccomp + landlock + namespaces
│       ├── src/macos.rs        # sandbox-exec
│       └── src/windows.rs      # Job Objects + AppContainer
│
├── src-tauri/                  # Tauri wrapper
│   ├── tauri.conf.json         # Allowlist, signing keys, build config
│   ├── src/lib.rs              # tauri::Builder + commands
│   └── icons/
│
├── src/                        # Frontend (TypeScript + React)
│   ├── runtime/                # Runtime-mode UI
│   │   ├── graph/
│   │   │   ├── LiveGraph.tsx
│   │   │   ├── nodes/
│   │   │   │   ├── AgentNode.tsx
│   │   │   │   ├── ToolNode.tsx
│   │   │   │   ├── SkillNode.tsx
│   │   │   │   ├── MCPNode.tsx
│   │   │   │   ├── GapNode.tsx
│   │   │   │   ├── HITLNode.tsx
│   │   │   │   ├── PlanNode.tsx
│   │   │   │   ├── TaskNode.tsx
│   │   │   │   ├── VerifyNode.tsx
│   │   │   │   └── HookNode.tsx
│   │   │   └── edges/AnimatedEdge.tsx
│   │   └── panels/{GapPanel,HITLPrompt,ApprovalPanel}.tsx
│   │
│   ├── builder/                # Build-time UI
│   │   ├── Canvas.tsx
│   │   ├── RegistrySearch.tsx
│   │   ├── SkillWriter.tsx
│   │   ├── Tester.tsx
│   │   └── CapabilityDisclosure.tsx          # L1 plain-English render
│   │
│   └── shared/{MCPManager,FrameworkManager,SessionManager,CostTracker,RecoveryDialog}.tsx
│
├── examples/                   # Reference frameworks
│   ├── aria/                   # ARIA archetype proof (see §0)
│   │   ├── framework.json
│   │   ├── skills/  agents/  tools/
│   │   └── README.md
│   └── ralph/                  # Sibling continuous-loop framework
│
├── docs/                       # Public documentation
│   ├── adr/                    # Architecture Decision Records
│   ├── SECURITY.md             # Threat model
│   ├── CONTRIBUTING-DEEPDIVE.md
│   └── PROVIDERS.md            # How to add a new LLMProvider
│
├── schemas/                    # JSON Schemas (source of truth)
│   ├── framework.v1.json
│   ├── skill.v1.json
│   ├── tool.v1.json
│   └── agent.v1.json
│
├── .github/                    # OSS scaffolding
│   ├── workflows/{ci.yml, release.yml, security.yml}
│   ├── ISSUE_TEMPLATE/
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── CODEOWNERS
│
├── LICENSE                     # Apache 2.0
├── CONTRIBUTING.md
├── SECURITY.md                 # Disclosure flow
├── CODE_OF_CONDUCT.md
├── README.md
├── agent-runtime-spec.md       # This document
└── package.json                # Frontend deps only
```

-----

## Starting Prompt for Claude Code

Use this to begin the build:

```
Read agent-runtime-spec.md fully before writing any code. Pay special
attention to §0 (positioning), §0a (capability matrix — MVP done-criterion),
§0b (Tool/Skill/Agent terminology), §1 (drone), §1c (multi-session),
§1d (IPC — Unix socket / Windows named pipe with framed JSON),
§8.security (5-layer model, especially L2 OS-level enforcement),
and §12 (Engineering Charter — coverage thresholds and CI gates).

We are building a local Tauri desktop runtime for agentic AI workflows
in Rust + TypeScript. Start with Phase 1: The Drone.

Pre-flight (do this before code):
1. Verify the workspace layout in §"Project Structure" exists or create
   the empty Cargo workspace skeleton (Cargo.toml, crates/runtime-core,
   crates/runtime-drone). Empty crates with `lib.rs` / `main.rs` stubs.
2. Set up rust-toolchain.toml pinning Rust to a specific stable version.
3. Add CI scaffolding (.github/workflows/ci.yml) with cargo fmt --check,
   cargo clippy --workspace -- -D warnings, cargo test --workspace.
   CI must be green before any logic lands.

Phase 1 implementation in crates/runtime-drone:
1. CLI args via clap: --session-id, --db-path, --ipc-socket
2. Initialize SQLite using rusqlite at db-path with WAL mode + busy_timeout
   per §1c (PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
   PRAGMA busy_timeout=5000; PRAGMA foreign_keys=ON)
3. Open a Unix domain socket (or Windows named pipe) at --ipc-socket;
   accept main's connection; framed JSON-newline using tokio_util Codec
4. Heartbeat task (tokio::spawn) firing every 5 seconds; emits
   DroneEvent::Heartbeat and writes to a heartbeats table in SQLite
5. Command handler task receives DroneCommand from socket; implements:
   - SnapshotNow { reason }: serialize provided state to snapshots table
   - GracefulShutdown { timeout_ms }: flush pending writes, exit 0
   - RevertToSnapshot { snapshot_id, reason }: load + return snapshot blob
6. SIGTERM/SIGINT handler (tokio::signal): emergency snapshot before exit
7. stdout/stderr go to a structured tracing log file, not the IPC channel

Use the exact DroneEvent and DroneCommand types from the spec. Use the
exact SQLite schema (Persistence Layer section) for sessions, snapshots,
signals, vdr tables. Implement all five tables now; logic that uses
signals/vdr comes in Phase 2.

Write tests:
- cargo test for heartbeat interval (use tokio::time::pause + advance)
- cargo test for snapshot_now writes correct row to SQLite
- cargo test for graceful_shutdown flushes within timeout_ms
- cargo test for SIGTERM-triggered emergency snapshot
- proptest for DroneEvent / DroneCommand JSON round-trip stability
- Coverage must be ≥80% for crates/runtime-drone before merge (§12)

Quality gates that must pass before any commit:
- cargo fmt --all -- --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- cargo audit (no known vulns)

Do not build anything beyond the drone in this session. Phase 2 (SDK
event pipeline) and beyond come in subsequent sessions.

If any spec section seems unclear, stop and surface the ambiguity rather
than improvise. The spec is the contract.
```

-----

## §11 Reconciliation & Degraded Modes

### Event → Graph → Signals → VDR → Dashboard sequence

```
┌──────────────┐
│   Anthropic  │
│   SDK stream │
└──────┬───────┘
       │ ProviderEvent
       ▼
┌──────────────────────┐
│  AgentSdk (Rust)     │  translates ProviderEvent → AgentEvent
│  (per session)       │  enriches with agent_id, session_id, ts
└──────┬───────────────┘
       │ AgentEvent
       ├──────────────┬──────────────┬──────────────┐
       ▼              ▼              ▼              ▼
┌──────────┐  ┌──────────────┐  ┌──────────┐  ┌─────────────┐
│ EventBus │  │ Drone        │  │ Signals  │  │ VDR         │
│ (in-mem) │  │ snapshot_now │  │ writer   │  │ projector   │
└────┬─────┘  │ on key events│  │ (SQLite) │  │ (decisions) │
     │        └──────┬───────┘  └────┬─────┘  └──────┬──────┘
     │               │                │               │
     ▼               ▼                ▼               ▼
┌──────────┐  ┌────────────┐  ┌──────────────────────────┐
│ Renderer │  │ snapshots  │  │ signals + vdr tables     │
│ live     │  │ table      │  │ (forensic + decision    │
│ graph    │  └────────────┘  │  layers, see §2b)        │
└────┬─────┘                  └──────────────┬───────────┘
     │                                       │
     │              ┌────────────────────────┘
     ▼              ▼
┌─────────────────────────┐
│ Dashboard (in-app)       │  reads snapshots + signals + VDR
│ + OTel exporter (WI-23) │  + token_usage + plans
└─────────────────────────┘
```

Key invariants:
- Every `AgentEvent` is delivered to all four sinks (in-mem bus, drone, signals, VDR projector). No sink can drop events without an entry in `error` signals.
- Drone snapshots are taken on `task_started`, `task_completed`, gap events, HITL events, and on a 30s timer fallback. Other events are signal-only.
- VDR is downstream of signals; signal writes never block on VDR.
- Renderer can be offline (closed window) — events queue in EventBus and replay when renderer reattaches.

### Degraded modes matrix

What the UI shows when a critical subsystem is unavailable.

| Subsystem down | Detection | UI behavior | Session behavior |
|---|---|---|---|
| **Anthropic API** (provider unavailable) | Heartbeat fails OR stream errors with auth/quota/network | Top banner: "Provider offline — sessions paused"; agent nodes show stalled state | Drone keeps running; snapshots continue; on reconnect, sessions resume from `stalled` state |
| **MCP server (one)** | Heartbeat fails for that server | MCPNode goes offline (red); affected ToolNodes pulse warning | Agents needing that server's tools route through gap flow (Phase 4 / §4b); other agents unaffected |
| **MCP server (all)** | All servers down | All MCPNodes offline; toast: "All MCP servers unavailable" | Frameworks with no MCP-dependent tools continue; others suspend |
| **Drone process** | Main loses IPC channel (socket EOF / pipe closure) | Top banner: "Drone process crashed — recovering"; graph disabled | Main spawns replacement drone via `tokio::process::Command`, reconnects socket, loads from last snapshot, replays events from in-memory tail; if replacement fails twice, session marked `crashed`, recovery offered on next launch |
| **SQLite database** (lock contention or corruption) | WAL write fails | Top banner: "Persistence layer degraded"; signals queued in memory | Session continues for `degraded_session_window_seconds` (default 60s); after window, session marked `crashed` and graceful_shutdown invoked |
| **Renderer (window closed)** | Main detects renderer disconnect | N/A (no UI) | Drone + main continue; on relaunch, renderer reattaches to running session via session_id; full graph reconstructed from signals |
| **Hook command unavailable** (e.g., `npm` not installed) | Hook exec fails with ENOENT | HookNode shows red with "command not found"; rail violation if hook is `block` | Per `on_failure` policy: warn (continue) or block (suspend) or rollback |
| **Registry upstream** (Anthropic skills repo unreachable) | Fetch fails | Builder search shows "Upstream unavailable; local-only results"; install button disabled for upstream items | Local artifacts unaffected; existing installs continue working |
| **Secrets vault** (keychain access denied) | keytar throws | Settings shows "Cannot access keychain"; affected MCPs cannot connect | Sessions using those MCPs route through gap flow |
| **Budget exceeded** (cap hit) | `budget_exceeded` event | Session header red badge "Budget exceeded"; agents killed | Session marked `budget_exceeded`; user must reset cap or end session |

### Reconciliation rule

Once per minute, the runtime reconciles in-memory state with SQLite:
- Compare unflushed signals queue against signals table → flush any missing.
- Compare current plan in memory against plans table → write back any drift.
- Detect orphaned tool calls (`tool_invoked` without `tool_result` for >5min and no error) → mark `tool_call_uncertain`, surface to user.
- Detect orphaned agents (last activity >timeout) → mark stalled, alert user.

This is the safety net for transient subsystem failures. Hard crashes are handled by the snapshot/recovery path (§1b).

-----

## §12 Engineering Charter

> **Locked (2026-04-18, OSS quality gate).** Process > language. These are the contracts that keep junk code out of the runtime regardless of contributor skill level.

### Test Rigor

| Layer | Tool | Threshold |
|---|---|---|
| Unit (Rust) | `cargo test` | ≥80% line coverage; **100% on safety primitives** (drone, capability enforcer, plan state machine, snapshot/recovery) |
| Property (Rust) | `proptest` | All public state machines have property tests for invariants; all serde types have round-trip property tests |
| Fuzz (Rust) | `cargo-fuzz` | Framework JSON parser, capability declaration parser, signal codec, IPC frame codec — all have fuzz harnesses; CI runs short fuzz on every PR, long fuzz nightly |
| Unit (TS) | Vitest | ≥80% line coverage on renderer logic |
| E2E | Playwright | Smoke test for every Phase deliverable: load framework → start session → trigger gap → resume → end session passes against built app |
| Integration | `cargo test --features integration` | Real SQLite, real socket IPC, mocked Anthropic API via wiremock; covers drone↔main and main↔sandbox boundaries |

Coverage is enforced by `cargo-llvm-cov` (Rust) and `vitest --coverage` (TS) in CI. PRs that drop coverage below threshold fail the gate.

### Type Strictness

| Concern | Rule |
|---|---|
| Rust warnings | `#![deny(warnings)]` at workspace root in CI builds |
| Clippy | `#![warn(clippy::pedantic, clippy::nursery)]`; CI runs `cargo clippy --workspace --all-targets -- -D warnings` |
| Unsafe | `#![forbid(unsafe_code)]` everywhere except `runtime-sandbox` (where seccomp/landlock require it); each `unsafe` block requires a `// SAFETY:` comment explaining invariants |
| TS strict | `"strict": true` in `tsconfig.json` — no `any` (use `unknown`); no implicit any; no untyped catch |
| TS escape hatches | `// @ts-ignore` and `// @ts-expect-error` require an issue link explaining why |

### Lint & Format

- `cargo fmt --all -- --check` blocks PRs.
- `cargo clippy --workspace -- -D warnings` blocks PRs.
- `prettier --check` and `eslint .` block PRs.
- `cargo deny check` blocks PRs (license + vuln + duplicate-version policy).
- Pre-commit hook (`lefthook` or `pre-commit`) runs all of the above locally; CI mirror prevents `--no-verify` bypass.

### Code Review

- **Branch protection** on `main`: required statuses (CI green), required reviewers (2 maintainers for core; CODEOWNERS for `crates/runtime-drone`, `runtime-sandbox`, `capability/`, `§8.security` paths), no force push, no direct push.
- **CODEOWNERS** mandates security-track reviewers on `runtime-sandbox/`, `capability/`, and `crates/runtime-main/src/providers/` (LLM client surface).
- **Squash-merge only.** Linear history; revert via revert PR, never via force push.
- **PRs must link to an issue or ADR** for any change touching a §0a matrix-row primitive.

### Dependency Hygiene

- `cargo audit` and `npm audit` run in CI; high/critical findings block release.
- `renovate` opens upgrade PRs weekly with grouped semver-minor.
- `cargo deny` policy in repo: deny GPL/AGPL deps (Apache-2.0 incompatibility); deny duplicate major versions; deny unmaintained crates per RustSec advisories.
- All Cargo + npm deps pin at minimum to `~major.minor` in manifests; `Cargo.lock` and `package-lock.json` committed.
- **SBOM (CycloneDX) generated per release** via `cargo-cyclonedx` + `cyclonedx-bom` (npm). Attached to release artifacts.

### Documentation

- **Public API** (`pub` items in Rust, exported types in TS) requires doc comments. CI runs `cargo doc --workspace --no-deps -- -D rustdoc::missing_docs` and `typedoc` in strict mode.
- **Doc tests must compile.** `cargo test --doc` runs in CI; broken examples fail the gate. Same for runnable Markdown blocks in `examples/` validated by a small custom checker.
- Every `pub` API has at least one example in its doc comment.
- **ADRs** in `docs/adr/`: every primitive change (anything affecting §0a matrix rows) requires an ADR using the template; PR description links to the ADR.

### Versioning & Release

- **SemVer** strict. Breaking changes to the framework JSON schema, AgentEvent union, or any `pub` Rust API require a major bump.
- **`schemas/` directory is the source of truth** for framework JSON / skill / tool / agent shapes. Versioned as `framework.v1.json`, `framework.v2.json`, etc. Rust types and TS types both generated from these schemas via `typify` (Rust) and `json-schema-to-typescript` (TS) — no hand-written drift.
- **Conventional Commits** enforced via `commitlint`; `release-please` automates changelog + tag + GitHub Release.
- **Releases are signed** (Sigstore) and reproducible (Tauri supports this; build via documented `cargo tauri build` invocation in CI). SLSA Level 3 provenance attached.
- **LTS policy:** the most recent two minor releases on the current major receive security patches.

### Security Disclosure

- `SECURITY.md` at repo root documents the disclosure flow: encrypted email + 90-day embargo before public disclosure unless actively exploited.
- `docs/SECURITY.md` carries the threat model — currently §8 §8.security threat model is the seed; expanded over time to cover runtime-binary attacks (Phase 1+ once code lands), supply-chain (Phase 2), and provider-side attacks (e.g., malicious tool result that injects a follow-up prompt).
- CVE numbers requested via GitHub Security Advisories.
- Disclosed CVEs reflected in the release changelog under a "Security" header with severity (CVSS v3.1) and remediation version.

### License & Contributor Agreement

- **Apache 2.0.** Patent grant matters for AI tooling. Compatible with most downstream uses; not infectious like AGPL.
- **DCO sign-off** instead of CLA. Contributors append `Signed-off-by: Name <email>` to commits; lower friction than CLA + adequate IP hygiene.
- `LICENSE` at repo root contains Apache 2.0 verbatim. `NOTICE` file lists third-party Apache-licensed deps' attributions.

### Contributor Experience

- `CONTRIBUTING.md` covers: clone → setup → first build → first test → first PR.
- `.devcontainer/` config so contributors can `code .` and have all toolchains (Rust, Node, Tauri deps) ready in a container.
- Issue templates: bug, feature, security (private channel link), proposal (pre-ADR).
- PR template asks: linked issue/ADR? tests added? coverage delta? breaking change?
- Maintainer onboarding doc covers: how to triage, how to release, how to handle CVE, how to update CODEOWNERS.

### Architecture Decision Records (ADRs)

ADRs are durable rationale; tribal knowledge dies with maintainers. Required for:

- Adding/changing/removing any §0a matrix-row primitive
- Changing the framework JSON schema (any version)
- Adding a new `LLMProvider` impl
- Changing capability enforcement behavior (any L1–L5 layer)
- Changing the IPC protocol between main, drone, or sandbox
- Adopting a new core dependency (anything that becomes a runtime dependency, not dev-only)

Template at `docs/adr/0000-template.md`. ADRs are immutable once merged; superseded ADRs link to their successor.

### CI Matrix

| OS | Rust | Node | Tauri |
|---|---|---|---|
| ubuntu-latest | stable, 1.80, MSRV | 20 | latest stable |
| macos-latest | stable | 20 | latest stable |
| windows-latest | stable | 20 | latest stable |

Each cell runs: fmt-check, clippy, test, doc-test, build, e2e (smoke). Nightly cell adds: extended fuzz, cargo-audit, renovate refresh, dependency review.

### Observability of Quality

`docs/QUALITY.md` is auto-generated weekly by a CI job that reads:
- Latest coverage report (per crate)
- Open security advisories
- Test count, fuzz hours, CVE-fix-time SLO compliance
- Release cadence and changelog churn

Public read of project health. Quality regressions visible to anyone evaluating adoption.

-----

## What This Is Not

- Not a chatbot interface
- Not a Claude Desktop replacement
- Not a general-purpose terminal emulator
- Not a low-code platform for non-technical users (first version)

## What Success Looks Like

A developer opens the platform, loads a framework JSON, runs a task, watches the live graph render agents and skills spawning in real time. A gap is detected — the session suspends cleanly, a GapNode shows exactly what is missing. The developer switches to the Agent Builder, finds the skill in the registry, installs and validates it, switches back, resumes — the graph picks up where it left off. If they close the laptop mid-run, they reopen and resume from the last snapshot.

That is the product.