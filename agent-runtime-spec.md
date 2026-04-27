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
  | { type: 'skill_invoked'; skill_id: string; agent_id: string; input: unknown }
  | { type: 'skill_complete'; skill_id: string; output: unknown; duration_ms: number }
  | { type: 'skill_missing'; skill_name: string; agent_id: string; context: string }
  | { type: 'mcp_tool_called'; tool_name: string; server: string; input: unknown }
  | { type: 'mcp_tool_result'; tool_name: string; output: unknown; duration_ms: number }
  | { type: 'hitl_requested'; agent_id: string; question: string; options: string[] | null }
  | { type: 'hitl_response'; agent_id: string; response: string }
  | { type: 'token_usage'; input: number; output: number; model: string; cost_usd: number }
  | { type: 'stream_text'; agent_id: string; text: string }
  | { type: 'decision_record'; agent_id: string; decision: string; rationale: string; tool_used: string }
```

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
    // On tool use: emit skill_invoked or mcp_tool_called
    // On tool result: emit skill_complete or mcp_tool_result
    // On missing tool: emit skill_missing → drone snapshots → session suspends cleanly
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
SkillNode      — invoked skill, shows input/output summary on hover
MCPNode        — connected MCP server, shows tool calls flowing through it
GapNode        — missing skill or agent, pulsing amber, suspends session cleanly
HITLNode       — blocked on human input, highlighted, awaiting response
FrameworkNode  — root node, the active framework (Aria, custom, etc.)
```

### Graph Behavior

- Nodes spawn in real time as events arrive from the pipeline
- Edges animate as tool calls flow: agent → skill, agent → MCP
- GapNode appears immediately on `skill_missing` event — session suspends cleanly
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

## Phase 4: Gap Detection and Clean Suspension

The runtime’s job on a gap is to stop cleanly and tell the user exactly what is missing. Nothing more.

### Gap Flow (Runtime)

```
skill_missing event received
  │
  ├── Emit GapNode to graph immediately
  │     - Shows: agent that needed the skill
  │     - Shows: skill name and the context it was called in
  │     - Shows: last known agent output before the gap
  │
  ├── Drone: snapshot_now (reason: 'gap_detected')
  │
  ├── Session moves to 'suspended' state
  │
  └── Gap Panel opens in UI
        - "Session suspended. Missing skill: X"
        - "Go to Agent Builder to find or create it"
        - "Resume" button — active once skill is installed
        - Full VDR trace up to suspension point available
```

### Gap Panel UX

```
Gap Panel (replaces graph chrome, graph visible underneath dimmed)
  - Skill name and description of what was needed
  - Agent context — what the agent was trying to accomplish
  - Link: "Open Agent Builder" (switches to build mode)
  - Session state preserved — graph will reconstruct on resume
  - Option: "Resume anyway without this skill" (marks session as degraded)
```

### GapNode in Graph

```
GapNode
  - Amber pulsing ring
  - Shows skill name
  - Shows which agent hit the gap
  - Edge from agent to GapNode animated in red
  - On resume: GapNode replaced by newly installed SkillNode
```

-----

## Phase 5: MCP Manager

Visible in the graph. Every connected MCP server is a node. Tool calls route through it as animated edges.

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

### Phase 8: Skill Writer

When nothing exists in registries the builder writes it.

**Collaborative Mode**

```
User describes what they need in natural language
Platform asks clarifying questions (HITL)
SDK generates skill.md iteratively
User reviews each iteration
Approved → runs through validator → added to skill library
```

**Autonomous Mode**

```
User provides: intent + any examples or constraints
SDK generates complete skill.md without back-and-forth
Validator runs immediately
Result presented for user approval before install
User can request revisions or approve as-is
```

### skill.md Standard Format

```markdown
---
name: skill_name
version: 1.0.0
description: What this skill does
author: source or generated
input_schema:
  type: object
  properties:
    param_name:
      type: string
      description: what this param does
  required: [param_name]
output_schema:
  type: object
tags: [tag1, tag2]
tested: true
---

## Description
Full description of what this skill does.

## Implementation
How the skill accomplishes its goal.

## Examples
Input/output examples.

## Error Handling
Known failure modes and how they are handled.
```

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