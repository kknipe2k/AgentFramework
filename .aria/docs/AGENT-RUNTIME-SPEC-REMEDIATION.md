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
| **P0 — Blocking** | WI-00, WI-01 | Nothing else starts until resolved |
| **P1 — Critical** | WI-02 to WI-07 | Must be resolved before Phase 1 code |
| **P2 — Important** | WI-08 to WI-17 | Resolve per related phase |
| **P3 — Nice-to-have** | WI-18 to WI-25 | v2+ |
| **P4 — Polish** | WI-26 | Any time |

**Project framing (locked 2026-04-18):** The runtime is a **generic agent-building, maintenance, and runtime-management platform**. ARIA is the **reference archetype** — the canonical agentic system used to validate that the runtime's primitives are sufficient. v1 MVP is complete when a user can reconstruct every ARIA capability inside the runtime using only primitives, without modifying runtime source. The existing `.aria/` shell codebase is reference material, not a port target.

---

## Dependency Graph

```
WI-00 (Archetype Capability Matrix) — MVP done-criterion
  └── informs every P1 item (each must trace back to one or more matrix rows)

WI-01 (ARIA relationship: Archetype)
  ├── WI-02 (Verify-as-primitive)
  ├── WI-03 (Plan-as-primitive)
  ├── WI-04 (Skill vs Tool)
  │     ├── WI-05 (Gap detection)
  │     └── WI-06 (Skill-writer security)
  ├── WI-07 (Budget primitive)
  ├── WI-15 (Mode router primitive)
  └── WI-08 (Signal schema)
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

### WI-00: ARIA Archetype Capability Matrix (MVP done-criterion)

**STATUS:** OPEN
**Priority:** P0 (defines MVP scope)
**Effort:** 2–3 days

**Problem**
With the runtime positioned as a generic agent platform and ARIA as the reference archetype, "MVP done" is ambiguous unless we have a concrete checklist. Without it, every P1 work item risks being scoped by intuition rather than by what's actually needed to reconstruct ARIA.

**Proposed resolution**
Build a capability matrix table in the spec (`§0a: Archetype Capability Matrix`) that lists every ARIA capability and the runtime primitive that must exist to express it. Each row becomes an MVP acceptance row. v1 ships when every row's primitive is present and a worked example reconstructs the row using only framework JSON + shipped primitives.

Initial rows (extend during WI-00 work):

| ARIA Capability | Runtime Primitive Required | Driving WI |
|---|---|---|
| LITE / STANDARD / FULL / FULL+ mode router | `modes` field in framework JSON + sizing-agent role | WI-15 |
| Sizing matrix (tasks × LOC × files × deps × auth) | Framework-defined sizing function (declarative table or agent prompt) | WI-15 |
| `verify.sh` after every task | Per-task post-hook field (shell command or tool ref) | WI-02 |
| Hard / soft rails (`rails/safety.json`) | `rails` section in framework JSON + rails-evaluator hook | WI-02 |
| Plan → HITL approve → execute | PlanNode + TaskNode + approval-gate event | WI-03 |
| Subagent isolation (analyzer, implementer, verify-app, simplifier) | Agent type definitions in framework JSON + spawn rules | WI-03, WI-04 |
| Decision trace (`decisions.jsonl`) | Built-in VDR projection from event stream | WI-08 |
| Signal Schema v2 (8 signal types) | Native event taxonomy in runtime | WI-08 |
| Ralph autonomous loop | Loop-policy primitive + PRD-style goal store | WI-03 |
| Model selection (budget + learning) | Model-selector hook + budget primitive | WI-07, WI-13 |
| Offline RL (Thompson Sampling) | External-process plugin reading signals, writing policy | WI-08, WI-23 |
| Dashboard (`:8420`) | Built-in (replaces ARIA dashboard) | core |
| Git ops (checkpoint/rollback/PR) | Tool wrappers + drone snapshot integration | WI-02 |
| HITL notifications (terminal/desktop/sound) | Notifier plugin interface | WI-16 |
| Slash commands (`/aria-start`, etc.) | Command-palette registration from framework JSON | core |
| Hooks (PreToolUse, PostToolUse, Stop) | Hook event types in runtime + framework subscription | WI-08 |
| Project-context "don't touch" zones | `dont_touch` field in framework JSON | WI-02 |
| Failure escalation (3 failures → HITL) | Failure-counter primitive + HITL trigger policy | WI-16 |
| Skills as context-loaded markdown | Skill-as-instruction-set type (distinct from tool) | WI-04 |
| Mode-variant skill behavior | Mode-aware skill loader | WI-04, WI-15 |

**Acceptance criteria**
- [ ] Spec §0a exists with the table
- [ ] Every row maps to at least one downstream WI
- [ ] Every P1 WI references the matrix rows it unlocks
- [ ] A "ARIA Reconstruction Walkthrough" example is added to the spec showing how the bundled `examples/aria/` framework reconstructs a representative row using only primitives
- [ ] Matrix is review-locked before any Phase 1 code is written

**Dependencies:** None (root)
**Blocks:** Every P1 WI scope

---

### WI-01: ARIA relationship — Archetype model

**STATUS:** OPEN
**Priority:** P0
**Effort:** 1 day spec

**Problem**
The spec makes no reference to the existing 13,190-LOC ARIA codebase. Without an explicit relationship statement, every downstream design decision is ambiguous about whether it should reuse, port, or ignore existing work.

**Proposed resolution**
Write `agent-runtime-spec.md §0: Project Positioning & Relationship to ARIA` committing to the **Archetype** model:

- The runtime is a generic agent-building / maintenance / runtime-management platform.
- ARIA is the canonical reference framework — recreated inside the runtime to validate the runtime's primitives.
- Existing `.aria/` shell codebase remains as **reference material**. It is not ported, replaced, or wrapped. It stays usable as-is for users who prefer shell.
- The runtime ships an `examples/aria/` directory containing the framework JSON + bundled tools/skills that reconstruct ARIA's capabilities. This is not a "built-in default" — it is a worked example users can copy or study.
- v1 MVP success criterion = the WI-00 capability matrix is fully checked.

Per-subsystem fate (referenced from §0):

| `.aria/` subsystem | Fate |
|---|---|
| `verify.sh`, `verify-executor.sh` | Reference only. `examples/aria/` reimplements via runtime post-task hooks. |
| `rails-executor.sh`, `rails/safety.json` | Reference only. Reimplemented via runtime rails primitive. |
| `ralph/ralph.sh` | Reference only. Reimplemented via runtime loop policy. |
| `planner/planner.sh` | Reference only. Reimplemented via planning agent + plan primitive. |
| `model-selector.sh` | Reference only. Reimplemented via budget primitive + model-selector hook. |
| `lib/offline-learner.py` | Stays as external Python process, consumes runtime signal export. Optional. |
| `lib/meta-reasoning.sh` | Reference only. Reimplemented as in-runtime decision skill. |
| `hitl.sh` | Reference only. Reimplemented via HITL primitive + notifier plugins. |
| `git-ops.sh` | Reference only. Reimplemented via tool wrappers around git. |
| `hooks/` (Git hooks) | Stays. Independent of runtime. |
| `.claude/agents/` (subagents) | Reference only. Reimplemented as runtime agent definitions. |
| `dashboard/`, `serve-dashboard.py` | Replaced. Runtime ships its own dashboard. |
| `docs/` (CONCEPT-*, WORKFLOW-MAP, etc.) | Stays. Source of truth for what the archetype must do. |

**Acceptance criteria**
- [ ] Spec §0 exists, ≤2 pages
- [ ] Archetype model named explicitly
- [ ] Per-subsystem fate table present
- [ ] Spec L426 ("Aria ships as the built-in default framework") rewritten to reflect `examples/aria/`
- [ ] Owner identified

**Dependencies:** WI-00 (the matrix references this positioning)
**Blocks:** WI-02, WI-03, WI-04, WI-07, WI-08, WI-15

---

## P1 — Critical (resolve before Phase 1 code)

### WI-02: Verify-and-rails primitive (so users can build ARIA's verify pipeline themselves)

**STATUS:** RESOLVED (2026-04-18, locked in spec §4a)
**Priority:** P1
**Effort:** 2–3 days spec; 2–3 weeks implementation
**Unlocks matrix rows:** verify.sh hook (3), rails (4), dont_touch zones (17), git checkpoint/rollback (13)

**Problem**
The runtime needs primitives that let a framework author *express* a verification pipeline and safety rails — not built-in implementations of ARIA's specific verify.sh and rails/safety.json. Without these primitives, the WI-00 matrix rows for verify and rails cannot be reconstructed.

**Proposed resolution**
Add `§4a: Verify & Rails Primitives` between current Phase 4 and 5. The runtime ships:

1. **Post-task hook field** in framework JSON:
   ```json
   "hooks": {
     "post_task": [{ "type": "shell", "command": "bash .aria/verify.sh", "level": "standard" }],
     "post_file_edit": [{ "type": "tool", "tool": "lint_changed_files" }]
   }
   ```
   Hooks accept `type: shell | tool | agent` so authors can plug in any executor (shell script, MCP tool, or another agent).
2. **`VerifyNode`** as a graph node type rendered when a `post_task` hook of category `verify` fires. Pass/fail visible. Click for output. *The runtime knows nothing about test frameworks; it just renders what the hook emits.*
3. **Rails primitive** in framework JSON:
   ```json
   "rails": {
     "hard": [{ "id": "no_secrets", "check": "shell:scripts/check-secrets.sh", "message": "..." }],
     "soft": [{ "id": "no_debug",   "check": "tool:scan_for_debug",           "message": "..." }]
   }
   ```
   Rails are evaluated by a runtime-provided rails-evaluator that shells out / calls tools as declared. The runtime provides the *framework* for rails; the *checks* come from the author.
4. **Hook & rail events** in the `AgentEvent` union:
   `hook_started`, `hook_passed`, `hook_failed`, `rail_triggered{ id, policy, message }`.
5. **`revert_to_snapshot` drone command** + automatic HITL rollback prompt on `hook_failed` (policy configurable per hook).
6. **`dont_touch` field** in framework JSON — a list of glob patterns. Built-in pre-edit rail blocks writes to matching paths with `policy: hard`.

The `examples/aria/` framework wires (1)–(6) up to reproduce ARIA's `verify.sh`, `rails/safety.json`, and `project-context.md` zones using only these primitives.

**Acceptance criteria**
- [ ] Spec §4a exists with all six sub-items, all framed as **primitives**
- [ ] Framework JSON schema gains `hooks`, `rails`, `dont_touch` fields
- [ ] `AgentEvent` union extended with hook/rail events
- [ ] Drone command API includes `revert_to_snapshot`
- [ ] `examples/aria/framework.json` shows a working reconstruction of verify + rails + dont_touch
- [ ] WI-00 matrix rows for verify, rails, dont_touch, git ops are checkable from this WI

**Dependencies:** WI-00, WI-01
**Blocks:** Phase 1 code

---

### WI-03: Plan / task primitive (so frameworks can express plan-driven workflows)

**STATUS:** RESOLVED (2026-04-18, locked in spec §3a)
**Priority:** P1
**Effort:** 2 days spec; 1 week implementation
**Unlocks matrix rows:** plan→approve→execute, ralph loop, subagent isolation, failure escalation

**Problem**
The runtime needs a generic plan/task primitive that frameworks can use to express *any* multi-step workflow with approval gates and per-task lifecycle. ARIA's plan-then-execute is one realization; Ralph's PRD-driven loop is another. Without a plan primitive, the runtime cannot express either.

**Proposed resolution**
Add `§3a: Plan & Task Primitive`. The runtime provides:

1. **`Plan` data type** (durable, in SQLite, snapshot-aware):
   ```typescript
   interface Plan {
     id: string
     title: string
     tasks: Task[]
     status: 'pending_approval' | 'approved' | 'in_progress' | 'complete' | 'aborted'
     approval_required: boolean      // false = auto-approve (Ralph-style)
     loop_policy: 'one_shot' | 'fresh_context_per_task' | 'continuous'
   }

   interface Task {
     id: string
     title: string
     status: 'pending' | 'running' | 'done' | 'blocked' | 'failed'
     hitl: boolean
     hitl_reason?: string
     post_hooks?: HookRef[]          // override framework defaults
     failure_count: number
     max_failures: number             // triggers escalation
   }
   ```
2. **Graph nodes:** `PlanNode` (root), `TaskNode` (children). Render driven by data type, not framework-specific logic.
3. **Plan/task events** in `AgentEvent`:
   `plan_created`, `plan_approval_requested`, `plan_approved`, `plan_revised`, `task_started`, `task_completed`, `task_failed`, `task_escalated`.
4. **Approval-gate primitive** — when `approval_required: true` and `plan_created` fires, runtime enters `awaiting_approval`, graph dims, approval panel shows. `a | r | c` actions emit corresponding events. Framework decides UX copy.
5. **Loop policy primitive** — `loop_policy: fresh_context_per_task` is what ARIA's planning skill needs. `loop_policy: continuous` is what Ralph needs (one persistent agent, multi-iteration). Framework picks per plan.
6. **Failure escalation primitive** — when `task.failure_count >= max_failures`, emit `task_escalated` and route to whatever HITL handler the framework has registered (see WI-16).

The `examples/aria/` framework uses `approval_required: true` + `loop_policy: fresh_context_per_task`. An `examples/ralph/` framework uses `approval_required: false` + `loop_policy: continuous` + a PRD goal store.

**Acceptance criteria**
- [ ] Spec §3a exists, framed as primitive
- [ ] `Plan` and `Task` data types defined
- [ ] Graph node types and event taxonomy specified
- [ ] Approval-gate, loop-policy, failure-escalation are independent primitives the framework composes
- [ ] `examples/aria/` reconstructs ARIA-style plan workflow
- [ ] `examples/ralph/` reconstructs Ralph-style loop using the same primitive
- [ ] WI-00 matrix rows for plan, ralph loop, failure escalation are checkable

**Dependencies:** WI-00, WI-01, WI-02 (verify hook lives at task boundary)
**Blocks:** Phase 1 code

---

### WI-04: Three concepts — Tool, Skill, Agent

**STATUS:** RESOLVED (2026-04-18, locked in spec §0b)
**Priority:** P1
**Effort:** 1 day spec
**Unlocks matrix rows:** 6 (subagent isolation), 19 (skills as instruction sets), 20 (mode-variant skills)

**Locked decisions**

1. **Three concepts, not two:**
   - **Tool** — callable, has input/output schema. `tool_use` block. Sources: MCP server, generated `tool.md`, built-in.
   - **Skill** — context-loaded instruction set in canonical `skill.md` (frontmatter + free-form body). Loaded via the runtime-injected `LoadSkill` tool.
   - **Agent** — composable LLM role: system prompt + allowed tools + allowed skills + model. Spawned via `SpawnAgent` tool.
2. **Dedicated `LoadSkill` runtime tool** — clean `skill_loaded` event boundary; framework auto-injects.
3. **Skill triggers — semantic AND programmatic, both in v1.**
   - Semantic: agent self-loads from "Available skills" block in system prompt.
   - Programmatic: JSONLogic-style trigger expressions evaluated against the event stream; runtime emits `skill_load_requested` on match.
4. **Strict frontmatter, free-form body.** Existing ARIA skills get ported to canonical format in `examples/aria/skills/`.
5. **Skill writer output scoped to non-executable** (cross-ref WI-06): Skills (instructions), Tool-bindings (MCP wrappers, no new code), Agent compositions. Never executable code.

**Spec changes**
- Added `§0b Three Concepts: Tool, Skill, Agent` (canonical definitions + LoadSkill / SpawnAgent specs + trigger expression language).
- Renamed `skill_invoked` / `skill_complete` → `tool_invoked` / `tool_result` in Phase 2 event union.
- Added `skill_loaded`, `skill_load_requested`, `tool_missing` events.
- Phase 3 node types updated: `ToolNode` (callable invocation), `SkillNode` (context load — distinct visual).
- Phase 5 MCP: explicit that MCP exposes Tools, never Skills.
- Marked Phase 8 split (Tool Writer / Skill Writer / Agent Composer) as a follow-up under WI-06.

**Acceptance criteria** — all met
- [x] Three concepts defined and enforced consistently
- [x] `tool.md` and `skill.md` formats both specified
- [x] Framework JSON has `tools`, `skills`, `agents` distinct fields
- [x] Event taxonomy split tool / skill / agent
- [x] Programmatic trigger language defined
- [x] Existing spec contradictions resolved (Phase 2/3/5)

**Dependencies:** WI-01 ✓
**Blocks:** WI-05 (gap detection now has tool_missing vs skill_missing), WI-06 (writer scope locked)

---

### WI-05: Gap detection mechanism

**STATUS:** RESOLVED (2026-04-18, locked in spec Phase 4 / §4b)
**Priority:** P1
**Effort:** 2 days spec; 3 days implementation
**Unlocks matrix rows:** clean-suspension UX (Phase 4 was a UX-only stub before this)

**Locked decisions**

1. **Three-layer detection, two ship in v1:**
   - **Layer 1 (Static):** load-time and spawn-time reference validation. Skill missing → warn (recoverable); tool missing → suspend; agent_missing → schema-block at load.
   - **Layer 2 (`request_capability` meta-tool):** runtime auto-injects into every agent's tool list. Translates to `tool_missing` or `skill_missing` per `capability_kind`.
   - **Layer 3 (Heuristic):** deferred to v1.1. Repeated similar failures → suggest a missing capability. v1 routes repeated failures to HITL via WI-16 instead.
2. **Severity locked:** `tool_missing` suspends; `skill_missing` warns and continues. Recovery via `gap_resolved` event after Builder install.
3. **Distinct events:** `tool_missing`, `skill_missing`, `agent_missing`, `capability_requested`, `gap_resolved` all in the canonical Phase 2 union.

**Spec changes**
- Phase 4 expanded with §4b "Detection Mechanisms" (three layers + severity matrix + gap flow + resolution flow).
- `request_capability` tool fully specified with input schema and system-prompt addition.
- Severity matrix table covers all four event-source combinations.
- Gap flow now branches on tool-vs-skill kind (suspend vs warn).
- Phase 2 union extended: `tool_missing.reason` includes `'request_capability'` and `suspends_session: boolean`; `skill_missing.source` distinguishes static vs request; new events `agent_missing`, `gap_resolved`, `capability_requested`.
- SDK wrapper comments updated to call out `request_capability` translation.

**Acceptance criteria** — all met
- [x] Spec Phase 4 / §4b exists with all three layers (Layer 3 deferred and documented)
- [x] `request_capability` tool schema fully defined
- [x] System-prompt addition specified
- [x] Static-validation reference table covers all 5 framework JSON reference types
- [x] Severity matrix distinguishes tool/skill, static/request, suspend/warn
- [x] Gap-resolved flow specified (snapshot resume vs warning clear)
- [x] Phase 2 event union updated for consistency

**Dependencies:** WI-04 ✓
**Blocks:** Phase 4 code (now unblocked), WI-12 registry trust (gap-resolved depends on registry install path)

---

### WI-06: Generator security model — five layers

**STATUS:** RESOLVED (2026-04-18, locked in spec Phase 8 §8.security)
**Priority:** P1 (security)
**Effort:** 3 days spec; 4–6 weeks implementation
**Unlocks matrix rows:** prerequisite for WI-12 registry trust; enables auto-install path safely

**Locked decisions**

After review of established patterns (browser permissions, Deno `--allow-*`, npm provenance, Sigstore, Dependabot auto-merge), the spec ships **all five layers**, not the originally-proposed three tiers alone:

1. **L1 — Capability Disclosure.** Every generated artifact has a mandatory `capabilities` block in frontmatter declaring tools_called, skills_loaded, file_access globs, network hosts, shell flag, spawn_agents. Builder UI translates to plain English at install.
2. **L2 — Capability Enforcement at runtime.** Runtime intercepts every operation initiated by an artifact and checks against its declared `capabilities`. Violations emit `capability_violation` and route to HITL grant prompt. *This is the layer that makes "auto-accept tested" actually safe — without it, "tested" is only as good as the sandbox.*
3. **L3 — Sandboxed Validation.** Drone-managed sandbox runs schema check, declared-examples, capability-bound execution, adversarial inputs, static red-flag scan. Output attached as `validation_report`.
4. **L4 — Tiered Human Gate.** Three tiers — Novice (manual review every install), Promoted (auto-accept validated artifacts within bounds; toggle requires one-time warning), Operator (auto-accept anything that passes L3; toggle requires stronger warning). Promotion sticky but reversible; tier changes audit-logged. Forbidden at all tiers: auto-install of an artifact that fails L3 or whose declared capabilities exceed validator scope.
5. **L5 — Provenance & Audit.** Every artifact carries a `provenance` block (generator, model, prompt_hash, generated_at, validated_at, content_hash, signature). Every install/reject/uninstall/violation/tier-change appended to `skills.audit.jsonl` (append-only, hash-chained, secret-redacted).

**Threat model (locked)**
- Defends against (a) malicious model output, (b) compromised registry, (c) user error.
- Out of scope: Operator-tier user knowingly installing known-bad; runtime-binary attacks.

**Phase 8 split (locked)**
- §8a Tool Writer — outputs `tool.md` MCP-binding only (no executable code).
- §8b Skill Writer — outputs canonical `skill.md` instruction-set markdown.
- §8c Agent Composer — outputs framework JSON entry composing existing tools/skills/child agents. Composer enforces capability *narrowing* (child cannot exceed parent).

**Spec changes**
- Added §8.security with all five layers (~120 lines covering enforcement details, three-tier table, threat model, generator-specific surface).
- Updated §0b canonical `skill.md` schema to include mandatory `capabilities` and optional-but-recommended `provenance` blocks.
- Phase 2 union extended with `capability_violation`, `capability_grant`, `tier_changed`, `artifact_installed`, `artifact_validation`.
- Phase impact summary in §0b updated to reflect 5-layer model.
- §8a/§8b/§8c each documented with generator-specific surface and validator extensions.
- Builder UI flow specified (capability disclosure plain-English render; auto-accept toast pattern).

**Acceptance criteria** — all met
- [x] Spec §8 rewritten with all five security layers
- [x] Threat model section added covering a/b/c
- [x] `skills.audit.jsonl` schema defined (append-only, hash-chained)
- [x] Auto-accept paths gated by L4 tier + L2 enforcement + L3 validation; novice users protected by default
- [x] Capability disclosure mandatory and rendered in plain English
- [x] Provenance block specified
- [x] Generator split (8a/8b/8c) with per-generator validator behavior
- [x] Phase 2 union extended for consistency

**Best-practice alignment**
| Layer | Pattern source |
|---|---|
| L1 | Browser permissions, Deno `--allow-*`, Chrome extension manifest |
| L2 | Deno permission model, WASM sandbox, capability-based security |
| L3 | npm prepublish, Vercel preview deploys |
| L4 | Dependabot auto-merge, browser permission "remember this decision" |
| L5 | npm provenance, Sigstore, SLSA |

**Dependencies:** WI-04 ✓, WI-05 ✓
**Blocks:** Phase 8 code, WI-12 (registry trust composes with this), WI-24 (diff-view subsumed by Builder UI flow)

---

### WI-07: Budget and cost enforcement

**STATUS:** RESOLVED (2026-04-18, locked in spec §2a)
**Priority:** P1
**Effort:** 1 day spec; 1 week implementation
**Unlocks matrix row:** 10 (model selection budget + learning) — primitive only; learning stays external (row 11, WI-23)

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

**STATUS:** RESOLVED (2026-04-18, locked in spec §2b)
**Priority:** P2
**Effort:** 1 day spec
**Unlocks matrix rows:** 7 (decision trace), 8 (signal schema v2), 11 (offline RL via export), 16 (hooks)

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

**STATUS:** RESOLVED (2026-04-18, locked in spec §2c)
**Priority:** P2
**Effort:** 1 day spec
**Unlocks matrix row:** 10 (model selection budget+learning) — primitive substrate for downshift hook in §2a

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

**STATUS:** RESOLVED (2026-04-18, locked in spec §1b)
**Priority:** P2
**Effort:** 1 day spec
**Notes:** Replay (deterministic) is explicit non-goal in v1; deferred to WI-18 (v2)

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

### WI-15: Mode-and-sizing primitive (so frameworks can express LITE/STANDARD/FULL/FULL+)

**STATUS:** OPEN
**Priority:** P2
**Effort:** 1–2 days spec
**Unlocks matrix rows:** mode router, sizing matrix, mode-variant skill behavior

**Problem**
The runtime needs a generic mode primitive: a session-scoped enum value the framework defines and that downstream primitives (hooks, plan policy, HITL policy, subagent rules) can branch on. ARIA's LITE/STANDARD/FULL/FULL+ is one realization; another framework might define `dev | staging | prod` or `learning | production`. Without a mode primitive, none of these can be expressed.

**Proposed resolution**
Add `§3b: Mode & Sizing Primitive`. The runtime provides:

1. **Mode field** in framework JSON — author-defined enum:
   ```json
   "modes": {
     "values": ["LITE", "STANDARD", "FULL", "FULL+"],
     "default": "STANDARD",
     "per_mode_overrides": {
       "LITE":  { "hooks.post_task": [], "plan.approval_required": false },
       "FULL+": { "design_doc_required": true }
     }
   }
   ```
   Any other primitive (hooks, plan, rails, HITL policy) can reference `${mode}` and apply overrides per mode.
2. **Sizing-agent role** — special agent type whose job is to read the user's request and emit a recommended mode value via a `propose_mode` tool call. Output is human-confirmed (or auto-accepted, framework's choice).
3. **Sizing function (declarative alternative)** — instead of an LLM agent, framework can declare a JSON sizing table:
   ```json
   "sizing": {
     "mode": "declarative",
     "rules": [
       { "if": { "tasks_estimated": "<=5", "loc_estimated": "<2000" }, "then": "LITE" },
       { "if": { "auth_or_payments": true }, "then": "FULL" }
     ]
   }
   ```
   Runtime evaluates the rules against a user-supplied or framework-collected fact set.
4. **Session-scoped mode value** — once set, immutable for the session. Available to every event handler, hook, plan, agent prompt as `session.mode`.
5. **Mode change events** — `mode_proposed`, `mode_confirmed`, `mode_locked`. Graph header shows active mode.

The `examples/aria/` framework declares ARIA's four modes and sizing matrix verbatim using these primitives.

**Acceptance criteria**
- [ ] Spec §3b exists, framed as primitive
- [ ] Mode field schema defined (author-defined enum, not built-in)
- [ ] Both sizing-agent and declarative-sizing supported
- [ ] `${mode}` interpolation rules across other primitives specified
- [ ] `examples/aria/framework.json` reproduces ARIA's mode router using only the primitive
- [ ] WI-00 matrix rows for mode router, sizing matrix, mode-variant skill behavior are checkable

**Dependencies:** WI-00, WI-01, WI-03 (plan policy is mode-overridable)
**Blocks:** `examples/aria/` framework bundling

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

