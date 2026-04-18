# Agent Runtime Spec — Remediation Plan

**Companion to:** `AGENT-RUNTIME-SPEC-REVIEW.md`
**Source spec:** `agent-runtime-spec.md` (795 lines, commit `2e36f5b`)
**Created:** 2026-04-18
**Branch:** `claude/review-aria-agentic-wrapper-Xz7Y3`

---

## Purpose

This document turns the review findings into **discrete, 1:1 work items**. Each item is self-contained: problem → proposed resolution → acceptance criteria → dependencies → effort.

Work items are the unit of execution. Pick one, resolve it (spec update + if applicable code/prototype), check the acceptance criteria, move to the next.

---

## How to Use This Spec

1. **Pick by priority.** Critical items (WI-01 to WI-07) must be resolved in the spec before any Phase 1 code is written. Important items (WI-08 to WI-17) should be resolved in the spec before their related phase starts. Nice-to-haves (WI-18 to WI-25) are v2+ unless a compelling reason pulls them forward.
2. **Honor dependencies.** Some items block others (see dependency graph below). Resolving out-of-order wastes work.
3. **Update the source spec.** Each resolution should land as a PR to `agent-runtime-spec.md` (or a new `SPEC-v2.md` if rewriting). Check the acceptance criteria by rereading the updated spec.
4. **Mark status.** Use `STATUS: OPEN | IN PROGRESS | RESOLVED | DEFERRED` at the top of each item as work progresses.

---

## Priority Ordering

| Tier | IDs | Gate |
|---|---|---|
| **P0 — Blocking** | WI-01 | Nothing else starts until resolved |
| **P1 — Critical** | WI-02 to WI-07 | Must be resolved before Phase 1 code |
| **P2 — Important** | WI-08 to WI-17 | Resolve per related phase |
| **P3 — Nice-to-have** | WI-18 to WI-25 | v2+ |
| **P4 — Polish** | WI-26 | Any time |

---

## Dependency Graph

```
WI-01 (ARIA relationship)
  ├── WI-02 (Verification + rails)
  ├── WI-03 (Planning model)
  ├── WI-04 (Skill vs Tool)
  │     ├── WI-05 (Gap detection)
  │     └── WI-06 (Skill-writer security)
  ├── WI-07 (Budget enforcement)
  ├── WI-15 (Mode router import)
  └── WI-08 (Signal schema v2 inherit)
        └── WI-14 (Recovery semantics)

WI-09 (IPC) ──── independent, Phase 1 blocker
WI-10 (Multi-session) ──── independent, Phase 1 blocker
WI-11 (MCP namespace) ──── Phase 5 blocker
WI-12 (Registry trust) ──── Phase 7 blocker
WI-13 (LLMProvider abstraction) ──── Phase 2 blocker
WI-16 (HITL policies) ──── Phase 4 blocker
WI-17 (Dev-loop) ──── Phase 1 blocker
```

---

## P0 — Blocking

### WI-01: Define relationship to existing ARIA

**STATUS:** OPEN
**Priority:** P0 (blocks everything)
**Effort:** 1–2 days (decision + 1-pager)

**Problem**
The spec makes no reference to the existing 13,190-LOC ARIA codebase (shell engine, skills, Ralph loop, dashboard, offline RL, hooks). Every downstream decision depends on whether ARIA is replaced, wrapped, or run in parallel, and the spec silently assumes greenfield.

**Proposed resolution**
Write a new `agent-runtime-spec.md §0: Relationship to ARIA` section that commits to one of:

- **Option A — Replace.** Deprecate `.aria/` over N releases. Port all skills, rails, verify.sh, Ralph loop, offline-learner to TypeScript. Highest value, highest risk, ~6–9 months.
- **Option B — Wrapper (recommended for v1).** Electron runtime is the UI/drone/graph layer. Backend shells out to existing `.aria/verify.sh`, `.aria/rails-executor.sh`, `.aria/ralph/ralph.sh`, `.aria/lib/offline-learner.py`. Framework JSON references shell entry-points. ~2–3 months.
- **Option C — Parallel product.** New target user, different distribution. Existing ARIA stays shell-based. Worst option; dilutes focus.

Decision must include:
- Fate of `.aria/` directory over the next 12 months
- Migration path for existing skills (if replace)
- Which subprocess contract shell tools must expose (if wrapper)
- Named owner for the decision

**Acceptance criteria**
- [ ] `agent-runtime-spec.md §0` exists, ≤2 pages, names chosen option
- [ ] Every later phase references §0 when making a design choice dependent on it
- [ ] Fate of each top-level `.aria/` subsystem explicitly listed: inherit / port / deprecate / drop
- [ ] Owner identified

**Dependencies:** None (root)
**Blocks:** WI-02, WI-03, WI-04, WI-07, WI-08, WI-15

---

## P1 — Critical (resolve before Phase 1 code)

### WI-02: Verification and safety rails in runtime

**STATUS:** OPEN
**Priority:** P1
**Effort:** 2–3 days spec; 2–3 weeks implementation

**Problem**
The spec contains no verification pipeline, no safety rails, no rollback primitive. ARIA's central value proposition ("autonomous but safe") depends on these. Current `.aria/verify.sh` (701L, with git-stash rollback) and `.aria/rails/safety.json` have no spec analog.

**Proposed resolution**
Add `§4a: Verification & Rails` between current Phase 4 and 5, covering:

1. **`VerifyNode`** as a first-class graph node type. Emitted automatically after any agent action that writes files. Shows verify.sh level (quick/standard/full), pass/fail, failure details on click.
2. **Rails config** in framework JSON:
   ```json
   "rails": {
     "hard": [{ "id": "no_secrets", "check": "...", "message": "..." }],
     "soft": [{ "id": "no_debug", "check": "...", "message": "..." }]
   }
   ```
3. **Verify events** in `AgentEvent` union:
   - `verify_start`, `verify_pass`, `verify_fail`, `rail_triggered`
4. **Rollback primitive** — drone's existing snapshot system gains a `revert_to_snapshot` command. On `verify_fail`, HITL prompt offers rollback.
5. **Project-context zones** — framework JSON includes `don't_touch: [...]` paths. Attempting to edit a file in the list triggers `rail_triggered` with policy `hard`.

**Acceptance criteria**
- [ ] Spec §4a exists with all five sub-items
- [ ] Framework JSON schema updated with `rails` and `don't_touch` fields
- [ ] `AgentEvent` union includes new verify/rail event types
- [ ] Drone command API includes `revert_to_snapshot`
- [ ] Example framework JSON updated to show a minimal rails config

**Dependencies:** WI-01 (determines whether rails port from shell or rewrite)
**Blocks:** Phase 1 code

---

### WI-03: Planning model and task progression

**STATUS:** OPEN
**Priority:** P1
**Effort:** 2 days spec; 1 week implementation

**Problem**
Spec has no plan model. ARIA's core workflow is plan → HITL approve → execute one task at a time → verify after each → commit → next. Without this the runtime is a flat agent trace, not a plan-driven workflow.

**Proposed resolution**
Add `§3a: Plan & Task Model` between Phase 3 and 4, covering:

1. **`PlanNode`** in the graph — root of the current plan, shows total tasks, completed count, current task.
2. **`TaskNode`** children — each task shows title, status (`pending | running | done | blocked`), HITL flag, estimated time, actual time.
3. **Plan schema** (inherits from `current-plan.json`):
   ```typescript
   interface Plan {
     id: string
     title: string
     tasks: Task[]
     status: 'pending_approval' | 'approved' | 'in_progress' | 'complete' | 'aborted'
     hitl_checkpoints: string[]
     risks: string[]
   }
   ```
4. **Plan events** added to `AgentEvent`:
   - `plan_created`, `plan_approved`, `plan_revised`, `task_started`, `task_completed`, `task_failed`
5. **Approval gate** — on `plan_created`, runtime enters `awaiting_approval` state. Graph dims, approval panel shows. `a | r | c` flow same as shell ARIA.

**Acceptance criteria**
- [ ] Spec §3a exists
- [ ] Plan/Task node types defined with render semantics
- [ ] `AgentEvent` union extended
- [ ] `Plan` interface defined and matches `current-plan.json` schema
- [ ] Approval-gate UX documented
- [ ] Example end-to-end walkthrough: user types a goal → planner agent creates plan → approval → tasks execute one by one with verify gates

**Dependencies:** WI-01, WI-02 (verify gate integrates with task loop)
**Blocks:** Phase 1 code

---

### WI-04: Disambiguate "skill" vs "tool"

**STATUS:** OPEN
**Priority:** P1
**Effort:** 1 day spec

**Problem**
The spec's `skill.md` format has `input_schema`/`output_schema` frontmatter — that's a tool. ARIA's skills are context-loaded markdown prompts with semantic triggers and mode variations. Calling both "skill" will cause years of developer confusion.

**Proposed resolution**
Pick one terminology and apply globally. Recommendation:

- **Tool** = callable capability with input/output schemas. Source: MCP server, in-process function, or generated skill-as-callable. Maps to Anthropic SDK `tool_use` blocks.
- **Skill** = context-loaded instruction set (markdown with semantic triggers, mode variations, subagent orchestration hints). Loaded into the system prompt or read by agents as part of their working context. Does not have input/output schemas.

Update:
1. Spec Phase 7 renamed "Tool Finder" for registry-discovered MCPs / callables
2. Spec Phase 8 renamed "Tool Writer" for generated callables; new Phase 8b "Skill Writer" for generated instruction sets
3. `skill.md` format (L502–533) renamed `tool.md`; create separate `skill.md` format mirroring ARIA's existing skill structure
4. Framework JSON gains both:
   ```json
   {
     "tools": [ "create_component", "read_pdf" ],
     "skills": [ "planning", "tdd", "debugging" ]
   }
   ```
5. `AgentEvent.skill_invoked` → `tool_invoked`; add `skill_loaded` for context-loads

**Acceptance criteria**
- [ ] Global rename across the spec; no remaining ambiguous use
- [ ] Both `tool.md` and `skill.md` formats defined
- [ ] Framework JSON schema has both fields
- [ ] Event taxonomy separates `tool_*` from `skill_*`
- [ ] Registry (WI-12) search filterable by `type: tool | skill`

**Dependencies:** WI-01
**Blocks:** WI-05, WI-06

---

### WI-05: Gap detection mechanism

**STATUS:** OPEN
**Priority:** P1
**Effort:** 2 days spec; 3 days implementation

**Problem**
`skill_missing` is the trigger for the GapNode + clean suspension — the spec's claimed differentiator. But the spec never says **how** a missing skill/tool is detected. The model does not volunteer "I need tool X."

**Proposed resolution**
Add `§4b: Gap Detection Mechanisms`, specifying three detection layers:

1. **Static (pre-flight).** When a framework loads, validate that every `tools[]` and `skills[]` reference resolves locally. Unresolved → emit `skill_missing` at load time, session suspends before the first agent runs.
2. **Dynamic (meta-tool).** Inject a `request_capability` tool into every agent system prompt:
   ```
   name: request_capability
   description: Use when you need a capability not in your tool list.
   input_schema:
     type: object
     properties:
       capability_name: { type: string }
       reason: { type: string }
       example_input: { type: object }
   ```
   When the model calls `request_capability`, the event pipeline translates it to `skill_missing`.
3. **Heuristic (post-hoc).** If an agent fails 3+ times on similar errors OR repeatedly asks for user confirmation on the same sub-task, emit a soft `skill_missing` candidate event. User sees a suggestion ("This session keeps stumbling on PDF parsing — want to add a PDF tool?") but is not forced to suspend.

Document that v1 ships with (1) and (2); (3) is a stretch goal.

**Acceptance criteria**
- [ ] Spec §4b exists with all three layers
- [ ] `request_capability` tool schema fully defined
- [ ] System-prompt template shown with `request_capability` injected
- [ ] Pre-flight validation algorithm specified (pseudo-code)
- [ ] Heuristic layer flagged as v1.1

**Dependencies:** WI-04
**Blocks:** Phase 4 (Gap Detection) code

---

### WI-06: Skill-writer security model

**STATUS:** OPEN
**Priority:** P1 (security)
**Effort:** 3 days spec; ongoing implementation

**Problem**
Spec Phase 8 ships autonomous skill generation with "flag dangerous patterns (undeclared network calls, exec, etc.)" as the sole safety mechanism. This is regex on LLM-generated code. It is not a security boundary. If the autonomous path ships as described, the runtime is an arbitrary-code-execution pipeline with a LLM at the top.

**Proposed resolution**
Pick one of three security postures for v1 and commit in the spec:

- **Posture A — Declarative only.** Skills/tools generated by the writer contain no executable code. They are prompt + input_schema + output_schema + MCP-or-API binding. Execution happens via already-trusted MCP servers. Lowest risk; most restrictive.
- **Posture B — Gated autonomous.** Autonomous mode is renamed "drafted." After generation, a mandatory human review shows: full diff, running-test output, declared capabilities. User must explicitly approve before install. "Install anyway without review" is disabled in v1.
- **Posture C — Sandboxed execution.** Skills run in a capability-restricted runtime (Deno with explicit `--allow-*` flags declared in skill frontmatter, WASM, or Firecracker microVM). Requires sandbox selection, capability model, and escape-hatch policy.

**Recommendation:** ship v1 with **A + B combined** (declarative only, all generation gated by review). Defer C to v2.

Spec must include:
- Named security posture
- Threat model (what attack are we defending against? malicious LLM output? compromised registry? user error?)
- Non-goals explicitly stated (e.g., "not defending against a user who installs a known-bad skill")
- Audit log: every install → entry in `skills.audit.jsonl` with who/what/when/hash

**Acceptance criteria**
- [ ] Spec §8 rewritten with chosen posture
- [ ] Threat model section added
- [ ] `skills.audit.jsonl` schema defined
- [ ] "Install anyway" path documented or explicitly disabled in v1
- [ ] Installation requires a hash-based lockfile entry (see WI-22)

**Dependencies:** WI-04
**Blocks:** Phase 8 code

---

### WI-07: Budget and cost enforcement

**STATUS:** OPEN
**Priority:** P1
**Effort:** 1 day spec; 1 week implementation

**Problem**
Spec tracks token usage in a table but has no enforcement: no per-session cap, no per-framework cap, no model-downshift logic. A runaway agent loop can generate $100s in minutes.

**Proposed resolution**
Add `§2a: Budget & Cost Controls`, specifying:

1. **Three budget scopes**, stacking:
   - Per-session (hard cap, configurable default $5)
   - Per-framework (configurable in framework JSON)
   - Per-day global (optional, from settings)
2. **Enforcement actions** on threshold breach:
   - `warn_at_50_percent` — toast notification, no action
   - `downshift_at_75_percent` — switch to cheaper model (opus → sonnet → haiku)
   - `hitl_at_90_percent` — suspend session, require user approval to continue
   - `hard_stop_at_100_percent` — kill agents, mark session as `budget_exceeded`
3. **Budget events** added:
   - `budget_warning`, `budget_downshift`, `budget_suspended`, `budget_exceeded`
4. **Graph rendering** — each AgentNode shows spend as node weight (already in spec L277). Add session header bar showing `spent / budget` with color gradient.
5. **Reuse existing logic.** Port `model-selector.sh`'s budget tiers (< 20% → haiku, < 50% → avoid opus) as the downshift algorithm.

**Acceptance criteria**
- [ ] Spec §2a exists
- [ ] Framework JSON schema extends with `budget: { session_usd, per_agent_usd, actions: {...} }`
- [ ] Event taxonomy extended
- [ ] Settings UI mock describing how user configures global default
- [ ] Downshift algorithm specified (pseudo-code)

**Dependencies:** WI-01 (decides whether to shell out to model-selector.sh or port)
**Blocks:** Phase 2 code

---

## P2 — Important (resolve before related phase)

### WI-08: Inherit Signal Schema v2

**STATUS:** OPEN
**Priority:** P2
**Effort:** 1 day spec

**Problem**
Spec's VDR (L228–242) is strictly weaker than ARIA's Signal Schema v2. It loses: pre/post event separation, retry chains, parent-signal correlation, context classification (`skill|framework|code|search|verify|commit|subagent`), output previews, duration tracking, skill-load vs skill-invoke distinction.

**Proposed resolution**
Replace Phase 2's `AgentEvent` section with a direct port of `SIGNAL-SCHEMA-V2.md`'s 8 signal types. Add a new section `§2b: Event → VDR Projection` explaining that VDR is a projection of the event stream (one row per decision-producing event) and signals are the raw stream (one row per tool call, skill load, agent spawn, etc.).

Keep VDR simple for decision traceability; keep signals rich for forensics and dashboard.

**Acceptance criteria**
- [ ] All 8 signal types from `SIGNAL-SCHEMA-V2.md` present in spec §2
- [ ] VDR clearly documented as projection of signals, not parallel system
- [ ] SQLite schema extended: `signals` table alongside `vdr` table
- [ ] Existing `decisions.jsonl` / `signals.jsonl` importer specified (if shell ARIA is being migrated)

**Dependencies:** WI-01
**Blocks:** Phase 2 code

---

### WI-09: IPC channel (fork, not stdio-JSON)

**STATUS:** OPEN
**Priority:** P2
**Effort:** 2 hours spec; zero-cost implementation

**Problem**
Spec's starting prompt tells Claude to implement drone ↔ main IPC via `process.stdin` / `process.stdout` newline-delimited JSON. Any library that prints to stdout corrupts the stream.

**Proposed resolution**
Replace the stdio-JSON approach with `child_process.fork()`, which provides:
- Dedicated `process.send(msg)` / `process.on('message', handler)` channel
- Separate from stdout (which stays available for logging)
- Automatic JSON serialization
- Built-in disconnect detection

Update the starting prompt and the `DroneEvent` / `DroneCommand` section to use `process.send` semantics.

**Acceptance criteria**
- [ ] Spec §1 specifies `child_process.fork` with `process.send` / `.on('message')`
- [ ] Starting prompt (L752) updated
- [ ] Drone stdout/stderr remain usable for logging

**Dependencies:** None
**Blocks:** Phase 1 code

---

### WI-10: Multi-session concurrency model

**STATUS:** OPEN
**Priority:** P2
**Effort:** 1–2 days spec

**Problem**
Spec doesn't answer: can two frameworks run simultaneously? Is the drone singleton or per-session? How is SQLite accessed concurrently? What if two sessions want the same MCP server with different auth?

**Proposed resolution**
Commit to one model in spec §1:

- **Recommended: one drone per session**, all drones write to one shared SQLite database in WAL mode. MCP servers are singleton-per-URL at the process level, shared across sessions, locked by a ref-counted connection pool.

Spec must add:
- Session lifecycle diagram (spawn drone, connect MCPs, run, shutdown drone)
- SQLite config: WAL mode, busy-timeout, transaction scope
- MCP ref-counted pool sketch
- Rule for auth conflicts: first-wins with warning, session-level auth cannot be overridden mid-run

**Acceptance criteria**
- [ ] Spec §1 specifies session-drone relationship
- [ ] SQLite WAL configured in schema init
- [ ] MCP pool model documented
- [ ] Conflict rule for auth documented

**Dependencies:** None
**Blocks:** Phase 1 code

---

### WI-11: MCP namespace collision rule

**STATUS:** OPEN
**Priority:** P2
**Effort:** 2 hours spec

**Problem**
"Tool namespace collision detection and resolution" (L356) is listed as a capability but no algorithm is given.

**Proposed resolution**
Specify the algorithm in Phase 5:

1. **Default:** every tool is exposed as `<server_name>__<tool_name>`.
2. **Short-name alias:** if unambiguous across all connected servers, the short name also resolves. Ambiguous → short name fails with a clear error listing the full options.
3. **Explicit override in framework JSON:**
   ```json
   "mcp_aliases": {
     "create_component": "react-mcp__create_component"
   }
   ```
4. Server name cannot contain `__`. Tools containing `__` are allowed but the delimiter for `<server>__<tool>` is a double-underscore; parser is right-anchored on the first `__` from the left.

**Acceptance criteria**
- [ ] Algorithm specified in spec §5
- [ ] Example collision shown with resolution
- [ ] `mcp_aliases` field added to framework JSON schema

**Dependencies:** None
**Blocks:** Phase 5 code

---

### WI-12: Registry trust chain

**STATUS:** OPEN
**Priority:** P2 (security)
**Effort:** 2 days spec

**Problem**
Spec Phase 7 lists `agent.md registry — GitHub index of community agent definitions` (doesn't exist), `mcp.so/api/search` (real-ish), and "community registries — pluggable" (imaginary). No signature verification, no lockfile, no version pinning.

**Proposed resolution**
v1 ships with:

1. **One trusted upstream only:** Anthropic's official skills repo. Single URL, hardcoded, signed by repo HEAD commit hash.
2. **Local-only library** is the primary mode. Users install/generate skills locally; registries are advisory.
3. **`skills.lock` file** in the project. Every installed skill has an entry: `{ name, version, source_url, sha256, installed_at }`. Runtime validates hash on load.
4. **Pluggable registries** deferred to v2 with a proper design (signature verification, trust roots, user-configurable trust policy, reputation).

Update Phase 7 to reflect this scope.

**Acceptance criteria**
- [ ] Spec §7 scoped to v1: local library + one vetted upstream
- [ ] `skills.lock` format specified
- [ ] Hash validation at load time specified
- [ ] Deferred items clearly marked "v2"

**Dependencies:** WI-04, WI-06
**Blocks:** Phase 7 code

---

### WI-13: LLMProvider abstraction

**STATUS:** OPEN
**Priority:** P2
**Effort:** 1 day spec

**Problem**
Spec claims "Claude family first, frontier model agnostic long term" but wraps `@anthropic-ai/sdk` directly in `AgentSDK`. Adding a second provider later will be a large refactor.

**Proposed resolution**
Introduce `LLMProvider` interface in Phase 2:

```typescript
interface LLMProvider {
  name: string
  stream(config: AgentConfig): AsyncIterable<ProviderEvent>
  countTokens(messages: Message[]): Promise<number>
  listModels(): Promise<ModelInfo[]>
}
```

`AgentSDK` depends on `LLMProvider`, not `Anthropic`. v1 ships `AnthropicProvider` as the only implementation, but the seam exists.

**Acceptance criteria**
- [ ] `LLMProvider` interface defined in spec §2
- [ ] `AgentSDK` depends on interface, not concrete SDK
- [ ] `ProviderEvent` → `AgentEvent` mapping specified
- [ ] `AnthropicProvider` shown as the v1 implementation
- [ ] Model ID strings moved to provider config, not hardcoded

**Dependencies:** WI-08 (event shape)
**Blocks:** Phase 2 code

---

### WI-14: Recovery semantics documented

**STATUS:** OPEN
**Priority:** P2
**Effort:** 1 day spec

**Problem**
Spec L133 says "Resume rebuilds SDK message history from snapshot, reconnects MCPs, restores graph state" — but model tool results often depend on external state (a web fetch, a file read at time T). Replay is non-deterministic. Spec doesn't distinguish rebuilding *history* from *re-executing*.

**Proposed resolution**
Add `§1b: Recovery Semantics` clarifying:

1. **Resume rebuilds history, does not re-execute.** Prior agent messages, tool calls, tool results are loaded from the snapshot into the SDK message history as if they had already happened. The model starts generating the next turn fresh, with full prior context.
2. **Tool calls in flight at crash time** are marked `tool_call_uncertain` in the VDR and surfaced to the user on resume with a prompt: "This tool call did not complete. [r]etry / [s]kip / [a]bort."
3. **MCPs reconnect on resume.** If an MCP fails to reconnect, its node goes offline; tool calls to that server will fail and emit `skill_missing` (gap flow takes over).
4. **Plan state is restored.** Current task index, completion status per task, HITL checkpoint state.
5. **Non-deterministic tool replay** is an explicit non-goal in v1. Document it.

**Acceptance criteria**
- [ ] Spec §1b exists
- [ ] `tool_call_uncertain` state + recovery prompt specified
- [ ] Plan-state restore steps listed
- [ ] Non-determinism acknowledged in spec

**Dependencies:** WI-03, WI-08
**Blocks:** Recovery UI work

---

### WI-15: Mode router import (LITE/STANDARD/FULL/FULL+)

**STATUS:** OPEN
**Priority:** P2
**Effort:** 1–2 days spec

**Problem**
ARIA's sizing matrix (tasks × LOC × files × deps × auth scope → LITE/STANDARD/FULL/FULL+) drives planning, verification rigor, HITL density, subagent isolation. Spec has one mode.

**Proposed resolution**
Add `§3b: Mode Router` as a framework-level concept:

1. Framework JSON gets:
   ```json
   "modes": {
     "default": "STANDARD",
     "allow": ["LITE", "STANDARD", "FULL", "FULL+"],
     "per_mode": {
       "LITE":     { "verification": "if_tests_exist", "hitl_policy": "destructive_only", "subagents": false },
       "STANDARD": { "verification": "every_task",     "hitl_policy": "risky_actions",   "subagents": true  },
       "FULL":     { "verification": "mandatory",      "hitl_policy": "all_risky",       "subagents": true, "design_notes": true },
       "FULL+":    { "verification": "mandatory",      "hitl_policy": "per_epic",        "subagents": true, "design_notes": true, "design_doc": true }
     }
   }
   ```
2. **Router agent** runs first on session start, sizing the request, proposing a mode. User confirms (or the framework forces a mode).
3. Mode becomes a session property; all subsequent behavior keys off it.
4. Dashboard / graph show active mode in the session header.

**Acceptance criteria**
- [ ] Spec §3b exists
- [ ] Framework JSON extended
- [ ] Sizing matrix from `CLAUDE.md` included verbatim or referenced
- [ ] Default ARIA framework JSON shows all four modes configured

**Dependencies:** WI-01, WI-03
**Blocks:** Default ARIA framework bundling

---

### WI-16: HITL policy modes

**STATUS:** OPEN
**Priority:** P2
**Effort:** 1 day spec

**Problem**
Spec has one HITL policy value: `"on_gap"`. ARIA's HITL fires on: destructive ops, risky tool use, per-epic gates, explicit user choice prompts, failure escalation.

**Proposed resolution**
Extend framework JSON `hitl_policy` to an object:

```json
"hitl_policy": {
  "on_gap": true,
  "on_risky_tool": true,
  "risky_tools": ["Bash:rm", "Bash:git push", "WebFetch:*"],
  "on_dont_touch_edit": true,
  "on_failure_threshold": 3,
  "per_task": false,
  "per_epic": false,
  "on_budget_90_percent": true
}
```

Each triggers a different `HITLNode` variant with appropriate UI (approve/deny for destructive, approve/revise/cancel for plans, etc.).

**Acceptance criteria**
- [ ] Framework JSON `hitl_policy` schema extended
- [ ] HITL event variants defined in `AgentEvent`
- [ ] UI variants documented for each HITL type
- [ ] Default ARIA framework populates policy appropriately per mode

**Dependencies:** WI-03, WI-07, WI-15
**Blocks:** Phase 4 code

---

### WI-17: Dev-loop story

**STATUS:** OPEN
**Priority:** P2
**Effort:** 2 hours spec

**Problem**
Spec has no story for iterating on a framework without packaging Electron. Developers need hot reload across main/renderer/drone.

**Proposed resolution**
Add `§0b: Development Setup`:

- `npm run dev` → Vite dev server for renderer + `electron .` for main + forked drone with nodemon
- Drone reload does NOT lose session state (state lives in SQLite)
- Main-process reload WILL lose connections; user is warned
- `npm run test` → Vitest unit tests; `npm run test:e2e` → Playwright tests against a running dev build
- `npm run build` → electron-builder for current platform
- `npm run pack` → unsigned build for local testing

**Acceptance criteria**
- [ ] Spec §0b exists
- [ ] `package.json` scripts list specified
- [ ] Hot-reload behavior documented per process

**Dependencies:** None
**Blocks:** Phase 1 code

---

## P3 — Nice-to-Have (v2+)

Each item is a single self-contained spec addition, ordered for impact.

### WI-18: Graph replay

**STATUS:** DEFERRED (v2)
**Effort:** 3 days spec; 2 weeks implementation

Rewatch a completed session from `snapshots` + `signals`. Timeline scrubber in the graph, speed control (1x, 2x, 5x, instant). Useful for postmortems, onboarding, and regression triage.

**Acceptance criteria**
- [ ] Playback state machine defined
- [ ] Event sequencing from DB documented (one frame per snapshot OR one frame per event)
- [ ] Scrubber UI specified

**Dependencies:** WI-08

---

### WI-19: Plugin node types

**STATUS:** DEFERRED (v2)
**Effort:** 2 days spec; 1 week implementation

Framework can register custom graph node renderers via a `plugins` field. Use cases: domain-specific visualizations, custom status overlays, per-framework node styling.

**Acceptance criteria**
- [ ] Plugin interface defined
- [ ] Trust/sandboxing rules (plugins run in renderer — XSS risk)
- [ ] Example plugin shown

**Dependencies:** WI-04, WI-06 (shared security model)

---

### WI-20: Team / collaboration mode

**STATUS:** DEFERRED (v2)
**Effort:** 5 days spec; 4–6 weeks implementation

Export session as `.aria-session` bundle (snapshots + VDR + graph state + redacted secrets). Import on another machine for review or pair-debugging. Real-time collab is a stretch goal (needs CRDTs or authoritative server).

**Acceptance criteria**
- [ ] Bundle format specified (likely tarball with manifest)
- [ ] Secret-redaction rules before export
- [ ] Import UX (as read-only by default)

**Dependencies:** WI-08, WI-14

---

### WI-21: Remote / CI execution

**STATUS:** DEFERRED (v2)
**Effort:** 5 days spec; 6 weeks implementation

Run a framework headlessly on a server, stream events to a connected Electron client over WebSocket. Enables CI-driven agentic workflows and remote GPU nodes.

**Acceptance criteria**
- [ ] Event stream protocol over WS specified
- [ ] Auth / session-token flow
- [ ] Headless drone mode with HTTP control API

**Dependencies:** WI-09, WI-10

---

### WI-22: Skill/tool signing and lockfile

**STATUS:** PARTIAL (lockfile in WI-12; signing deferred)
**Effort:** 2 days spec

Beyond the `skills.lock` hash check from WI-12: every installed skill/tool carries a signature from its source. Trust roots are user-configurable. Revocation lists supported.

**Acceptance criteria**
- [ ] Signature format (PGP? Sigstore? simple X25519?)
- [ ] Trust root config
- [ ] Revocation mechanism

**Dependencies:** WI-12

---

### WI-23: Telemetry export (OpenTelemetry)

**STATUS:** DEFERRED (v2)
**Effort:** 1 day spec; 3 days implementation

OTel span export for enterprise users centralizing agent traces in Datadog/Honeycomb/Grafana. Signals → OTel spans mapping is straightforward (context = span parent, duration = span duration, tool_name = span name).

**Acceptance criteria**
- [ ] Signal → span mapping table
- [ ] OTel exporter config in settings
- [ ] Opt-in, off by default

**Dependencies:** WI-08

---

### WI-24: Diff-view for autonomous skill writer

**STATUS:** OPEN (bundles with WI-06)
**Effort:** 1 day spec; 3 days implementation

When the generator produces a skill, show a side-by-side diff of any existing version, test output, declared capabilities. Already recommended as part of WI-06 Posture B — this item just names the UI.

**Acceptance criteria**
- [ ] Diff-view mock
- [ ] Test-run panel shown below diff
- [ ] Capabilities panel (input schema, output schema, flagged patterns)

**Dependencies:** WI-06

---

### WI-25: Graph export to PNG/SVG

**STATUS:** DEFERRED (v2)
**Effort:** 2 hours spec; 1 day implementation

React Flow supports PNG export out of the box. Add an "Export graph" button. Useful for postmortems, docs, and slideshows.

**Acceptance criteria**
- [ ] Export button in graph toolbar
- [ ] Resolution options
- [ ] Secret redaction before export (see WI-20)

**Dependencies:** None

---

## P4 — Polish

### WI-26: Spec-level quality fixes

**STATUS:** OPEN
**Effort:** 1 hour

Grab-bag of small issues:

- Fix L743 typo: `AGENT_RUNTIME_SPEC.md` → `agent-runtime-spec.md`
- Update L385 model ID (`claude-sonnet-4-20250514`) to current (`claude-sonnet-4-6` or `claude-opus-4-7` as of 2026-04)
- Remove `sessions.snapshot_count` from SQLite schema (derivable via `COUNT(*)`)
- Add a sequence diagram for events → graph state → VDR → dashboard projection
- Add a "Degraded Modes" matrix: what the UI shows when API is down, MCP is down, drone is down

**Acceptance criteria**
- [ ] Typo fixed
- [ ] Model IDs current as of spec update date
- [ ] Schema trimmed
- [ ] Sequence diagram present
- [ ] Degraded-mode matrix present

**Dependencies:** None

---

## Execution Sequencing (Suggested)

Two-week sprints, spec-only until Sprint 4:

| Sprint | Items | Output |
|---|---|---|
| **S1** | WI-01 | ARIA relationship decision, 2-pager |
| **S2** | WI-02, WI-03, WI-04 | Verification + Planning + terminology fixed in spec |
| **S3** | WI-05, WI-06, WI-07, WI-08 | Gap detection, security, budget, signal schema in spec |
| **S4** | WI-09..WI-17 | All P2 items resolved in spec |
| **S5** | Phase 1 code begins (drone) | Code matches v2 spec |
| **S6+** | Phases 2–9 per build order | Iterate |
| **v2** | WI-18..WI-25 | After v1 ships |
| **Anytime** | WI-26 | Polish pass |

---

## Tracking

Each WI can be:
- Filed as a GitHub issue with ID as title
- Linked from the spec sections they modify
- Resolved by a PR that updates `agent-runtime-spec.md` and checks all acceptance criteria

**Suggested workflow per item:**
1. Open issue with full WI text
2. Branch: `spec/WI-NN-short-title`
3. PR updates spec + appends to this file with `STATUS: RESOLVED` and a link to the PR
4. Merge once every acceptance criterion is checked

---

*End of remediation plan.*

