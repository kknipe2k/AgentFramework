# Agentic Runtime Platform — Product Specification

## What This Is

A local Electron desktop runtime for agentic AI workflows. Not a chatbot wrapper. Not a framework. A **runtime** — the way Node.js is to JavaScript — that frameworks, agents, and skills execute inside.

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

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Electron Shell                        │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │              Renderer Process (UI)               │   │
│  │                                                  │   │
│  │   Live Graph    │  Agent Builder  │  MCP Manager │   │
│  │   Gap Detector  │  Registry Search│  Session UI  │   │
│  │   (runtime)     │  Skill Writer   │  Framework   │   │
│  │                 │  (build time)   │  Manager     │   │
│  └──────────────────────────────────────────────────┘   │
│                         │ IPC                            │
│  ┌──────────────────────────────────────────────────┐   │
│  │              Main Process (Node)                 │   │
│  │                                                  │   │
│  │   SDK Event Pipeline   │   MCP Client Layer      │   │
│  │   Framework Loader     │   Gap Suspender         │   │
│  │   Builder: Registry    │   Builder: Skill Writer │   │
│  │   Builder: Test Harness│                         │   │
│  └──────────────────────────────────────────────────┘   │
│                         │                                │
│  ┌──────────────────────────────────────────────────┐   │
│  │           Drone Process (Survival Layer)         │   │
│  │                                                  │   │
│  │   Heartbeat  │  Snapshots  │  Recovery  │  Spawn │   │
│  └──────────────────────────────────────────────────┘   │
│                         │                                │
│  ┌──────────────────────────────────────────────────┐   │
│  │           Persistence Layer (SQLite)             │   │
│  │   Sessions  │  VDR Traces  │  Artifacts  │  Logs │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

-----

## Tech Stack

|Layer         |Technology               |Reason                                            |
|--------------|-------------------------|--------------------------------------------------|
|Shell         |Electron (latest stable) |Process ownership, desktop integration            |
|UI Framework  |React 18 + TypeScript    |Component model, type safety                      |
|Graph Renderer|React Flow               |Production-grade, extensible, live updates        |
|Styling       |Tailwind CSS             |Utility-first, consistent design system           |
|SDK           |@anthropic-ai/sdk        |Native streaming, tool use, type safety           |
|Persistence   |SQLite via better-sqlite3|Local, zero server, fast                          |
|PTY           |node-pty                 |CLI bridge fallback, bidirectional process control|
|IPC           |Electron contextBridge   |Secure renderer ↔ main communication              |
|Build         |Vite + electron-builder  |Fast dev loop, cross-platform packaging           |
|Test          |Vitest + Playwright      |Unit and E2E coverage                             |

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

```typescript
// Drone spawned as: node drone.js --session-id <id> --db-path <path>

// Messages drone sends to main process via IPC
type DroneEvent =
  | { type: 'heartbeat'; status: HeartbeatStatus; timestamp: number }
  | { type: 'snapshot_written'; snapshot_id: string; session_id: string }
  | { type: 'activity_state_change'; from: ActivityState; to: ActivityState }
  | { type: 'process_spawned'; pid: number; process_type: ProcessType }
  | { type: 'process_stopped'; pid: number; reason: StopReason }
  | { type: 'recovery_available'; session_id: string; snapshot_id: string }
  | { type: 'alert'; level: 'warn' | 'critical'; message: string }

// Messages main process sends to drone
type DroneCommand =
  | { type: 'snapshot_now'; reason: string }
  | { type: 'graceful_shutdown'; timeout_ms: number }
  | { type: 'spawn_process'; process_type: ProcessType; config: ProcessConfig }
  | { type: 'stop_process'; pid: number; force: boolean }
  | { type: 'set_activity_timeout'; ms: number }

type ActivityState = 'active' | 'idle' | 'stalled' | 'timed_out' | 'user_aborted' | 'recovering'
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

```typescript
// src/main/sdk/AgentSDK.ts

class AgentSDK {
  private client: Anthropic
  private eventBus: EventEmitter
  private sessionId: string

  async runAgent(config: AgentConfig): Promise<void> {
    // Emit agent_spawned
    // Stream with SDK, emit stream_text per chunk
    // On tool use: emit tool_invoked (source: mcp | builtin | generated)
    // On tool result: emit tool_result
    // On LoadSkill tool result: also emit skill_loaded with mode/trigger_kind
    // On request_capability: emit capability_requested → tool_missing or skill_missing per kind (see §4b)
    // On missing tool (static): emit tool_missing → gap flow (see Phase 4, §4b)
    // On text block: extract decision records, emit decision_record
    // On complete: emit agent_complete
    // On error: emit agent_error → drone notified
  }
}
```

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
  - Tool namespace collision detection and resolution
  - Per-server auth config (stored in secrets vault, not in session state)
```

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
  "model": "claude-sonnet-4-20250514",
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
  "hitl_policy": "on_gap",
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

## Phases 7–9: Agent Builder (Build Time)

The Agent Builder is a distinct mode — not part of the runtime. The user switches to it deliberately, either from the gap panel or from the main nav. Nothing built here affects a running session until the user explicitly resumes.

### Phase 7: Registry Search and Skill Finder

```
Search Sources (searched in order, results ranked by relevance)
  ├── agent.md registry — GitHub index of community agent definitions
  ├── skill.md registry — curated and community skill index
  └── MCP server registry — mcp.so, Anthropic directory, community sources

Search Interface
  - Natural language query: "I need a skill that can read PDF files"
  - Results show: name, description, author, verified badge, source
  - Preview: full skill.md or agent.md before installing
  - Install: validated locally before becoming available to frameworks

Validation Before Install
  1. Parse skill.md / agent.md — valid format?
  2. Run against mock inputs in isolated sandbox (drone manages lifecycle)
  3. Check output schema matches declared schema
  4. Flag dangerous patterns (undeclared network calls, exec, etc.)
  5. Pass → added to local skill library
  6. Fail → report shown, user decides whether to install anyway
```

```typescript
interface RegistrySearchResult {
  name: string
  description: string
  type: 'skill' | 'agent' | 'mcp'
  source_url: string
  author: string
  verified: boolean
  install_method: 'json' | 'mcp_url' | 'npm' | 'github'
  skill_md_url?: string
  agent_md_url?: string
}

const REGISTRIES = [
  'https://raw.githubusercontent.com/anthropics/skills/main/index.json',
  'https://mcp.so/api/search',
  // community registries — pluggable
]
```

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

##### L2: Capability Enforcement (runtime interception)

The runtime maintains a per-artifact capability set loaded from L1. Every operation initiated by the artifact passes through a capability check:

- Tool call → check `tools_called` includes the target.
- Skill load → check `skills_loaded` includes the target.
- File read/write → check glob patterns in `file_access`.
- Network access (via WebFetch / generated tool that calls out) → check host in `network`.
- Shell access (via Bash) → check `shell == true`.
- Agent spawn → check `spawn_agents` includes the target.

Capability violation:
- Emits `capability_violation` event with `{ artifact, attempted, declared }`
- Blocks the operation
- Surfaces a HITL prompt: "Skill `<name>` attempted `<operation>` not in its declared capabilities. Allow once / Block / Open Builder to update."
- VDR records the attempt regardless of user choice.

> **Implication:** L2 is the layer that makes "auto-accept tested" safe. Without L2, "tested" only means the sandbox didn't catch anything; with L2, the artifact literally cannot exceed what it declared, no matter what the model wrote.

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
  status TEXT,  -- active | suspended | complete | crashed | recovered
  snapshot_count INTEGER
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
Storage: OS keychain via keytar (Electron compatible)
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
├── electron/
│   ├── main.ts              # Electron main process entry
│   ├── preload.ts           # contextBridge IPC definitions
│   └── drone/
│       ├── drone.ts         # Drone process entry (spawned separately)
│       ├── heartbeat.ts
│       ├── snapshot.ts
│       ├── recovery.ts
│       └── process-manager.ts
│
├── src/
│   ├── main/                # Main process modules
│   │   ├── sdk/
│   │   │   ├── AgentSDK.ts
│   │   │   ├── EventPipeline.ts
│   │   │   └── VDRLogger.ts
│   │   ├── mcp/
│   │   │   ├── MCPManager.ts
│   │   │   └── MCPClient.ts
│   │   ├── framework/
│   │   │   └── FrameworkLoader.ts
│   │   ├── builder/         # Build-time only — not loaded during runtime
│   │   │   ├── RegistrySearch.ts
│   │   │   ├── SkillValidator.ts
│   │   │   └── SkillWriter.ts
│   │   └── db/
│   │       ├── schema.ts
│   │       └── SessionStore.ts
│   │
│   └── renderer/            # React UI
│       ├── runtime/         # Runtime mode
│       │   ├── graph/
│       │   │   ├── LiveGraph.tsx
│       │   │   ├── nodes/
│       │   │   │   ├── AgentNode.tsx
│       │   │   │   ├── SkillNode.tsx
│       │   │   │   ├── MCPNode.tsx
│       │   │   │   ├── GapNode.tsx
│       │   │   │   └── HITLNode.tsx
│       │   │   └── edges/
│       │   │       └── AnimatedEdge.tsx
│       │   └── panels/
│       │       ├── GapPanel.tsx
│       │       └── HITLPrompt.tsx
│       ├── builder/         # Build-time mode (Agent Builder)
│       │   ├── Canvas.tsx
│       │   ├── SkillSearch.tsx
│       │   ├── SkillWriter.tsx
│       │   └── Tester.tsx
│       └── shared/
│           ├── MCPManager.tsx
│           ├── FrameworkManager.tsx
│           ├── SessionManager.tsx
│           ├── CostTracker.tsx
│           └── RecoveryDialog.tsx
│
├── examples/
│   └── aria/                # Reference framework reconstructing ARIA via primitives (see §0)
│       ├── framework.json
│       ├── skills/
│       └── tools/
│
├── AGENT_RUNTIME_SPEC.md    # This document
└── package.json
```

-----

## Starting Prompt for Claude Code

Use this to begin the build:

```
Read AGENT_RUNTIME_SPEC.md fully before writing any code.

We are building a local Electron desktop runtime for agentic AI workflows.
Start with Phase 1: The Drone.

Build the drone as a standalone Node.js process in electron/drone/drone.ts.
It should:
1. Accept --session-id and --db-path as CLI args
2. Initialize SQLite using better-sqlite3 at db-path
3. Start a heartbeat loop (5 second interval) and write heartbeat records to SQLite
4. Accept commands from main process via process.stdin (newline-delimited JSON)
5. Emit events to main process via process.stdout (newline-delimited JSON)
6. Implement snapshot_now command: serialize provided state to snapshots table
7. Implement graceful_shutdown command: flush pending writes, exit cleanly
8. Catch SIGTERM and SIGINT for emergency snapshot before exit

Use the exact DroneEvent and DroneCommand types from the spec.
Use the exact SQLite schema from the spec for sessions and snapshots tables.

Write tests in Vitest for:
- Heartbeat fires at correct interval
- Snapshot is written correctly to SQLite
- Graceful shutdown flushes before exit
- SIGTERM triggers emergency snapshot

Do not build anything beyond the drone in this session.
```

-----

## What This Is Not

- Not a chatbot interface
- Not a Claude Desktop replacement
- Not a general-purpose terminal emulator
- Not a low-code platform for non-technical users (first version)

## What Success Looks Like

A developer opens the platform, loads a framework JSON, runs a task, watches the live graph render agents and skills spawning in real time. A gap is detected — the session suspends cleanly, a GapNode shows exactly what is missing. The developer switches to the Agent Builder, finds the skill in the registry, installs and validates it, switches back, resumes — the graph picks up where it left off. If they close the laptop mid-run, they reopen and resume from the last snapshot.

That is the product.