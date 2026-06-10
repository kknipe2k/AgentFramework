# M09 — Workbench Vertical Slice: author one real agent, run it on real data

> **Protocol version:** v1.11 (per `STAGE-PROMPT-PROTOCOL.md` — `<tdd_discipline strict="true">` two-commit on every code stage, now **validator-enforced**: each code stage's `<execution_steps>` carries the explicit `red_phase_commit → surface_for_red_approval → green_phase_commit → surface_for_final_approval` sequence — `bin/validate-stage-prompts.mjs::checkStrictTddTwoCommit` fails CI otherwise; v1.8 `<construction_reachability_check>` + `<wire_signature_audit>`; the **assembled-app-regression mandate** is the close bar for the integration stage).
>
> **Re-cut alignment:** this is the **first vertical slice of the ADR-0032 re-cut** (Accepted 2026-06-07). M09 stays the thinnest end-to-end thread — author one agent + `file_access` + one real MCP tool → run → write a real file, at the enforced tier — and ships with **"suspends cleanly"** (E-04) only. Gap **resolve→resume** + plan-approval move to **M10**; sub-agents to **M11**; the verify-loop + shell-exec to **M12**; the data-source catalog + save/import-export to **M13**. The forward pointers below are remapped to ADR-0032's M10–M13 (ADR-0031's horizontal M10–M12 author-anything / real-data / execution layers are withdrawn).
>
> **The milestone where the workbench stops *composing* and starts *authoring + running*.** Today the canvas can only rearrange agents that already exist in JSON, and a built workflow runs read-only at Novice. M09 makes the smallest **complete, real** loop work end-to-end *in the app*: drag a **new** agent onto an empty canvas, grant it a **file_access** scope, attach a **real MCP server's tool**, hit Run, and watch it **pull real data and write a real file** — at the tracked tier, enforced. It is the **first vertical slice** of the `docs/workbench-delivery-plan.md` re-cut (ADR-0032), and it is deliberately small because the substrate it rides — single-agent streaming, built-in Read/Write, MCP dispatch, capability + tier enforcement — **already executes** (`docs/execution-status.md`).

---

## Background — the grounded state at entry

**What runs (rule 11, IRL-confirmed — `docs/execution-status.md`).** A single agent that streams multi-turn, runs built-in **Read/Write**, loads a skill, suspends on a gap, tracks budget, dispatches an **MCP tool**, and is gated by **capability (L1/L2) + tier (L4)** — all execute in the assembled app today. MCP dispatch is wired into the run loop (`crates/runtime-main/src/sdk/agent_sdk.rs:884` `try_mcp_dispatch` → `crates/runtime-mcp/src/dispatch.rs:91` `McpDispatcher`, capability-enforced). The L2 file-scope enforcement is proven by the E-02 eval (`crates/runtime-main/tests/capability_live_tool.rs`: an in-scope Write lands; an out-of-scope Write is blocked with no file on disk).

**What does NOT exist (grounded, file:line) — the authoring gap this milestone closes:**

- **You cannot create an agent from scratch.** The Palette's Agents tab lists only installed artifacts + a loaded framework's agents (`src/components/builder/Palette.tsx:173-184`); there is no built-in/blank agent. A fresh project opens with `emptyFramework()` (`src/lib/builderStore.ts:124`, `agents: []`), so the Agents tab is **empty** and nothing can be authored on the canvas. The store *already supports* minting one — `addNode` (`builderStore.ts:506`) → `applyDrop` (`:183`) → `builderAgent` (`:145`), reached by the canvas drop handler `BuilderCanvas.onDrop` (`:86-96`) — the **Palette simply never offers a blank item**. This is a UI gap, not an architecture gap.
- **An authored agent cannot be made *capable*.** `builderAgent` (`builderStore.ts:145-154`) constructs `{ id, role, model, allowed_tools, allowed_skills, spawns }` and **omits `capabilities`** — which `agent.v1.json:9` lists as **required** (`common.v1.json#/$defs/Capabilities`: `{tools_called, skills_loaded, file_access:{read,write}, network, shell, spawn_agents}`). So a canvas agent is *schema-invalid* and, more importantly, carries **no `file_access`** — the read/write glob scope the L2 enforcer checks. `NodeConfigPanel` (`src/components/builder/NodeConfigPanel.tsx:105-146`) edits only `role`/`model`/`allowed_tools`/`allowed_skills` — there is no capability surface. **Without `file_access.write`, the agent's Write is denied — nothing lands.**
- **A real MCP server's tools cannot be reached on the canvas.** `mcp_list_servers` (`src-tauri/src/commands.rs:1083`) lists installed servers and `mcp_test_connection` (`:1049`) enumerates a server's tools via `McpClient::test_connection`→`list_tools` (`crates/runtime-mcp/src/client/mod.rs:246-259`), but **there is no command to list an *already-installed* server's tools** for the palette, and the Tools tab (`Palette.tsx:143-160`) offers only built-ins (`Read`/`Write`/`Bash` — and `Bash` doesn't even execute) + installed artifacts + framework tools. So the data-bearing tools an agent needs are invisible to the author.

**The load-bearing fact.** Every runtime piece the vertical slice needs **already executes** — the gap is entirely in *authoring* (three small surfaces) plus one small backend command (list an installed server's tools). That is why M09 is four tight stages, not a quarter.

**This milestone = the first vertical slice of `docs/workbench-delivery-plan.md` (ADR-0032).** M09 ships **pure** — author → run → write a real file — and ships with **"suspends cleanly"** (E-04) only; the prior M08.8 stage-trim items re-home into the re-cut's slices: **gap resolve→resume → M10** (with plan-approval), **save-polish → M13** (industrialize), **budget-visible → M10** (HITL steers). The **M08.8.C / TD-036** tier-display fix already shipped — the Tester run now reads the tracked tier at invocation and the Settings display no longer desyncs from the enforced tier (`commands.rs:1730-1736`, #19 resolved) — so the **read+set-Promoted precondition for M09.D's IRL is already met**, not a carry-in.

---

## Scope

**In scope (the vertical slice, end-to-end, in the real app):**
- **M09.A** — blank-create an **agent** on the canvas (the Palette "New agent" affordance + a fresh-id helper; the store path already exists).
- **M09.B** — a **file_access editor** in `NodeConfigPanel` (the agent's `capabilities.file_access.{read,write}` glob lists) + `builderAgent` initializing a valid `capabilities`.
- **M09.C** — a new `mcp_list_server_tools(name)` command + surfacing an installed MCP server's tools in the Palette Tools tab + attaching one to the agent (`allowed_tools` + `capabilities.tools_called`).
- **M09.D** — the **assembled real-app IRL**: build the agent fresh, scope it, attach the MCP tool, Run via the Tester, observe real data pulled + a real file written; flip the execution-status "observed in app" row for *canvas-authored single-agent + MCP + file_access*.

**Out of scope (later vertical slices of the ADR-0032 re-cut — authoring widens per slice, with its concept's execution):**
- **Gap resolve→resume + plan-approval + plan task execution** (M09 ships "suspends cleanly" / E-04 only) and the **budget-visible** surface → **M10** (HITL steers the run). The Plan palette node + its config land here, with plan execution.
- **Sub-agents executing** (orchestrator spawns a narrowed child, sequential `max_concurrent:1`) + the spawn/`spawn_agents` authoring surface → **M11**.
- **Hooks / rails firing + controlled shell-exec** (the verify loop — `bash verify.sh` gate, `dont_touch` rail) + the hook/rail authoring surface + the `shell` capability UI + the `Bash`-advertised-but-unbuilt integrity fix (Bash becomes real shell-exec here) → **M12**.
- **MCP servers as first-class canvas citizens + a data-source catalog** (GitHub/Postgres/Slack/Drive/Notion) + credentials UX + **validated whole-workflow import/export + save-path** + first-run → **M13** (industrialize + ship).
- The remaining generic authoring polish — config for every node kind, node delete, id-rename, the rest of the `Capabilities` surface (`network`) — lands with the slice that consumes it (ADR-0032 verticals), not a separate author-anything layer.

**Locks:** no schema change (the `Capabilities`/`Agent`/`McpServerConfig` shapes are the source of truth — author *to* them); **declaration-only capability authoring** — M09.B writes the agent's grant, it never widens the enforcer or the user's tier; the new M09.C command **reuses** the existing `test_connection`/`list_tools` path (no new MCP machinery); real-app `tauri-driver` IRL is the close bar (ADR-0021); strict v1.11 two-commit TDD on every code stage.

---

## Staging (each stage builds the next; M09.D is the assembled close)

| Stage | Deliverable (forensic) | Key real seams |
|---|---|---|
| **M09.A** | **Blank-create an agent.** A "New agent" Palette affordance (Agents tab) carrying `{kind:'agent', ref:<fresh-id>}`; the existing `BuilderCanvas.onDrop:96 → addNode` mints `builderAgent`. A `nextAgentRef(framework)` helper generates an id matching `^[a-z][a-z0-9-]*$`. | `Palette.tsx:173-184` (empty agents tab) · `builderStore.ts:506/183/145` · `BuilderCanvas.tsx:86-96` |
| **M09.B** | **file_access editor.** `builderAgent` initializes a valid `capabilities`; `NodeConfigPanel` gains a read/write glob-list editor over `agent.capabilities.file_access`; edits flow through `updateNode`. | `builderStore.ts:145` · `NodeConfigPanel.tsx:105-146` · `common.v1.json#/$defs/Capabilities` · enforced by E-02 |
| **M09.C** | **Attach a real MCP tool.** New `mcp_list_server_tools(name)` (registry lookup + `list_tools`, reusing `test_connection`'s path); the Palette Tools tab surfaces an installed server's tools (`source:'mcp'`); attaching adds to `allowed_tools` + `capabilities.tools_called`. **`test_framework` already wires `build_test_mcp_dispatcher` + the tracked tier** (`commands.rs:1769-1775` / `:1758`) — C adds **no** execution-wiring; it is the read-only command + palette only. | new command ∼ `commands.rs:1049/1083` · `client/mod.rs:246` · `Palette.tsx:143-160` · `agent_sdk.rs:884` dispatch |
| **M09.D** | **The vertical-slice IRL.** Assembled `tauri-driver` e2e: a fresh canvas-authored agent + `file_access.write` + an installed MCP tool runs in the Tester and writes a real file from real MCP data; the maintainer IRL-watches it on the live app. Flip execution-status. | `tests/e2e-tauri/` · `test_framework` (`commands.rs`) · `docs/execution-status.md` |

**The falsifiable hypothesis M09.D must disprove (v1.8 assembled mandate):** "a **canvas-authored** agent — `capabilities.file_access` + an MCP tool authored through the M09.A–C UI, serialized across the Tauri IPC boundary into `framework_doc` — runs in the assembled Tester *exactly as a hand-written-JSON framework does*: the authored `file_access.write` gates the run and a real file lands." The individual wires are already present and confirmed (the MCP dispatcher + the tracked tier — `commands.rs:1769-1775` / `:1758`; the enforcer is built from the framework's agents via `grant_framework_capabilities` → `AgentSdk::with_capability_wiring` — `tester.rs:36/445`, the same construction E-02 proves). The genuine unknown is the **composed authored→serialized→run path end-to-end** (structure ≠ behavior — rule 11): no canvas-authored framework has ever run. If the assembled test fails, the gap is a serialization/threading defect in that path, closed in M09.D — never a new execution primitive.

---

## Process (per stage)

1. **Authored at entry** against the live renderer + the real schemas (§8 phase-doc pre-flight — cross-machine state checked; the build `git pull`s the milestone branch before dropping any stage prompt).
2. **Cluster-gate close** (`docs/cluster-pattern.md`): acceptance-first (BDD) → strict v1.11 two-commit TDD → machine gates + **both** e2e gates (`test:e2e` + `test:e2e:tauri`) → **the real-app IRL** → triage-in-place (zero-propagation).
3. **Mutation gate** — advisory on the renderer stages (A/B); **on M09.C** the new Rust command is non-critical (a read-only `list_tools` wrapper) → advisory. The dispatcher + tier + capabilities-granting are **already wired** in `test_framework` (`commands.rs:1769-1775` / `:1758`; `tester.rs:445`), so M09.C/D add **no** execution-wiring under expectation. The standing rule still holds (`cluster-pattern.md` §5): *if* any stage unexpectedly touches an executor/dispatch branch or the enforcer wiring, the mutation gate is **blocking** on that diff — surfaced first (Hard Rule 8).
4. **The close flips execution-status's "observed in app" status** for the surface — on the maintainer IRL watch, never tests-green (rule 11). Only **M09.D** flips the row (A–C build toward it).

---

## Stage M09.A — Blank-create an agent on the canvas

### A.1 Problem statement
A fresh project cannot author anything. `emptyFramework()` (`builderStore.ts:124-135`) opens with `agents: []`; the Palette's Agents tab is built from `installed` + the loaded `framework` only (`Palette.tsx:173-184`) — no built-in/blank agent — so the tab renders its empty state (`Palette.tsx:256-259` "No agents.") and there is nothing to drag. The store already mints an agent on a drop: `BuilderCanvas.onDrop` (`:86-96`) parses the `application/x-builder-node` payload `{kind, ref}` and calls `addNode(kind, ref, position)` (`builderStore.ts:506`), which applies `applyDrop` (`:183` → `builderAgent(ref)` `:145`) and records the position. **The only missing piece is a Palette item that offers a *fresh* agent.**

### A.2 Files to change
**`<wire_signature_audit>` — pin before pseudocode:** `BuilderNodeKind = 'agent'|'tool'|'skill'|'hitl'|'hook'` (`builderStore.ts:24`); `addNode(kind, ref, position)` (`:506`, idempotent on `${kind}:${ref}` in `nodePositions` `:509`); `builderAgent(id)` (`:145`); the drag payload contract `{kind, ref}` on `application/x-builder-node` (`Palette.tsx:274-277` / `BuilderCanvas.tsx:88-96`); the agent id pattern `^[a-z][a-z0-9-]*$` (`agent.v1.json:14`).
- `src/components/builder/Palette.tsx` — the Agents tab gains a leading **"+ New agent"** item whose `ref` is a fresh unique id (`nextAgentRef(framework)`); it is a normal draggable carrying `{kind:'agent', ref}`. (The blank-create affordance for tools/skills widens in a later ADR-0032 slice — M09 scopes to the agent, the authoring blocker.)
- `src/lib/builderStore.ts` — export a pure `nextAgentRef(framework): string` helper (`agent-1`, `agent-2`, … skipping any existing id), matching the schema pattern.
- `tests/e2e-tauri/builder_create_agent.e2e.ts` (new) + a vitest unit on `nextAgentRef` + the Palette item.

### A.3 Detailed changes
1. **`nextAgentRef` (pure, in `builderStore.ts`).** Existing ids come from `framework.agents` (each `entry.id`). Return the first `agent-N` (N from 1) not already present:
   ```ts
   /** The next free `agent-N` id for a blank-created agent — matches the
    *  agent.v1.json id pattern `^[a-z][a-z0-9-]*$` and skips ids already
    *  in the framework so a re-create never collides with addNode's
    *  `${kind}:${ref}` idempotence guard (builderStore.ts:509). */
   export function nextAgentRef(framework: Framework): string {
     const taken = new Set(framework.agents.map((a) => a.id));
     for (let n = 1; ; n += 1) {
       const ref = `agent-${n}`;
       if (!taken.has(ref)) return ref;
     }
   }
   ```
2. **The Palette "New agent" item.** In `paletteItemsForTab`'s `'agents'` case (`Palette.tsx:173`), prepend a `source:'builtin'` item `{ kind:'agent', ref: nextAgentRef(framework), label:'+ New agent' }` ahead of the installed/framework items. It drags through the *same* `onDragStart` contract (`:269-279`) — no drop-handler change. Because `addNode` mutates `framework` and the Palette reads `framework` (`:212`), the next render recomputes `nextAgentRef` → `agent-2`, so repeated creates never collide.
3. **No drop-handler / store-core change.** `onDrop:96 → addNode → applyDrop:185 (agent → builderAgent)` already does the work; M09.A only *offers* the item. The new agent appears, is selectable (`BuilderCanvas onNodeClick → selectNode`, `:111`), and `NodeConfigPanel` already renders for it (`findAgent`, `NodeConfigPanel.tsx:12`). Continuous validation (`scheduleValidation`, `builderStore.ts:454`) surfaces the still-missing `role`/`capabilities`/`session_root_agent`.

### A.4 Acceptance (BDD)
```gherkin
Feature: Author an agent from scratch
  Scenario: a fresh project creates its first agent on the canvas
    Given a brand-new project with an empty canvas
    When I open the Agents palette tab
    Then I see a "+ New agent" item
    When I drag it onto the canvas
    Then an agent node appears, selectable, and framework.agents contains a new agent
    And dragging "+ New agent" again creates a distinct second agent (agent-2)
```
Asserted on the **real Tauri app via `tauri-driver`** (`builder_create_agent.e2e.ts`) + the maintainer IRL — not Playwright-mock-green (rule 11 / ADR-0021).

### A.5 construction_reachability_check
`inputs_reachable="false"`: the Agents tab is empty on a fresh project (`Palette.tsx:173-184` — installed+framework only); nothing is draggable; the canvas cannot author. M09.A inverts it (`"true"`): the "New agent" item + the existing `addNode` mint. Verify by grep: the `'+ New agent'` branch in `paletteItemsForTab`; `nextAgentRef` exported; the e2e creates an agent on an empty project.

### A.6 Close gate
Strict v1.11 two-commit TDD (red: the `builder_create_agent.e2e.ts` + `nextAgentRef` unit fail right-reason — no New-agent item, empty tab; impl untouching tests, diff over test paths EMPTY). Frontend gates + **both** e2e gates (`test:e2e` renderer + `test:e2e:tauri` real-app). Mutation **advisory** (renderer). Stage-D design review (the "New" affordance matches `DESIGN.md`). **No execution-status flip** (M09.A authors but runs nothing — the flip is M09.D).

### A.7 CLI prompt
```xml
<work_stage_prompt id="M09.A">
  <context>
    M09 Stage A (the vertical slice's first stage) — let a fresh project create an
    agent from scratch on the canvas. Today the Palette Agents tab lists only
    installed + loaded-framework agents (Palette.tsx:173-184), so a new project
    (emptyFramework, agents:[]) shows an empty tab and nothing is draggable. The
    store ALREADY mints an agent on drop — BuilderCanvas.onDrop:96 → addNode
    (builderStore.ts:506) → applyDrop:183 → builderAgent:145 — so this is a
    Palette-only change: offer a "+ New agent" item carrying {kind:'agent',
    ref:<fresh-id>} via the existing application/x-builder-node drag contract. Add
    a pure nextAgentRef(framework) helper (agent-N, matches agent.v1.json:14
    ^[a-z][a-z0-9-]*$, skips existing). No drop-handler / store-core change. The
    new agent is schema-INVALID until M09.B adds capabilities — continuous
    validation surfaces that; do NOT add capabilities here (that's B).
    Zero-propagation; the tools/skills "New" affordance widens in a later ADR-0032 slice.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.A — A.1-A.6; Background)</file>
    <file>docs/workbench-delivery-plan.md (the first vertical slice per ADR-0032; this is its first stage)</file>
    <file>src/components/builder/Palette.tsx (paletteItemsForTab @137; the agents case @173-184; the onDragStart contract @269-279; the empty state @256)</file>
    <file>src/lib/builderStore.ts (addNode @506 + its idempotence guard @509; applyDrop @183; builderAgent @145; emptyFramework @124) + src/components/builder/BuilderCanvas.tsx (onDrop @86-96 → addNode)</file>
    <file>schemas/agent.v1.json (the id pattern @14; required fields @9) + docs/adr/0020-* (document-as-source-of-truth) + docs/adr/0021-* + CLAUDE.md §5/§6/§8</file>
  </read_first>
  <deliverable>A "+ New agent" Palette item (Agents tab) that drags a fresh-id agent onto the canvas via the existing addNode path, plus an exported pure nextAgentRef(framework) helper. A fresh project can author its first agent; repeated creates yield distinct ids. No drop-handler or store-core change; capabilities are M09.B.</deliverable>
  <tdd_discipline strict="true">Two commits: red (tests/e2e-tauri/builder_create_agent.e2e.ts + a nextAgentRef vitest unit + a Palette "+ New agent" render test — fail right-reason: empty agents tab, no helper) → impl untouching tests (diff over test paths EMPTY). Net-new additive tests in a separate labelled follow-up commit.</tdd_discipline>
  <wire_signature_audit>Pin: addNode(kind,ref,position) (builderStore.ts:506) + the ${kind}:${ref} idempotence guard (:509); builderAgent(id) (:145); the {kind,ref} payload on application/x-builder-node (Palette.tsx:274 / BuilderCanvas.tsx:88-96); the agent id pattern ^[a-z][a-z0-9-]*$ (agent.v1.json:14).</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="A.5"/>
  <execution_steps>
    <step name="ground_at_red" budget="1"/>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3">Frontend gates (prettier/eslint/tsc/vitest≥80/npm audit) + BOTH e2e gates (test:e2e renderer + test:e2e:tauri). No backend change; no schema change.</step>
    <step name="mutation_gate" blocking="false"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="assembled_run_irl"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <close_gate>
    <real_app_irl>Maintainer: fresh project → Agents tab shows "+ New agent" → drag onto the canvas → an agent node appears and is selectable; a second drag yields agent-2. (Authoring only — no run; no execution-status flip.)</real_app_irl>
    <mutation_gate blocking="false">Advisory — renderer + a pure helper.</mutation_gate>
    <design_review>The "+ New" affordance matches DESIGN.md (palette item style; principle 1).</design_review>
    <cumulative_regression>builder_create_agent.e2e.ts joins the e2e-tauri suite (key-independent — runs in CI).</cumulative_regression>
  </close_gate>
  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + retros present) + the red→impl diff over test paths (EMPTY)</item>
    <item>frontend gate results + test:e2e:tauri; the mutation note (advisory)</item>
    <item>the create-agent-from-scratch real-app walkthrough</item>
    <item>Stage-D design-review note</item>
    <item>explicit: "M09.A is ready. I will not commit until you approve."</item>
  </approval_surface>
  <scope_locks>
    <lock>Agents tab only — the "+ New" affordance for tools/skills + the missing primitives land with their ADR-0032 slice (Plan→M10, rails→M12, budget→M10, MCP-as-node→M13). No drop-handler or store-core change.</lock>
    <lock>Do NOT add capabilities here — the new agent is intentionally schema-invalid until M09.B; continuous validation surfaces the gap.</lock>
    <lock>No schema change; no backend change.</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>That the store already supported create-from-scratch (the gap was Palette-only); that the new agent is intentionally invalid-until-B; rule 11 (authoring observed, not run).</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Stage M09.A"/>
</work_stage_prompt>
```

### A.8 Commit message
```
feat(M09.A): author an agent from scratch — the "New agent" palette affordance

The canvas could only rearrange agents that already existed in JSON: the
Palette Agents tab listed installed + loaded-framework agents only
(Palette.tsx:173-184), so a fresh project's tab was empty. The store already
minted an agent on drop (addNode→applyDrop→builderAgent); add a "+ New agent"
palette item carrying a fresh nextAgentRef id through the existing drag
contract. Palette-only — no drop-handler or store-core change. The new agent
is schema-invalid until M09.B adds capabilities (continuous validation flags it).
```

---

## Stage M09.B — file_access editor (make the agent capable)

### B.1 Problem statement
An authored agent cannot read or write real data, and is schema-invalid. `builderAgent` (`builderStore.ts:145-154`) omits `capabilities`, which `agent.v1.json:9` marks **required** (`common.v1.json#/$defs/Capabilities`). The enforced read/write scope lives at `capabilities.file_access.{read, write}` (glob lists — `common.v1.json:56-64`): the L2 enforcer checks a Write target against `file_access.write`, so an empty/absent `write` list means **every Write is denied** (E-02 `capability_live_tool.rs` proves the in-scope/out-of-scope split). `NodeConfigPanel` (`NodeConfigPanel.tsx:105-146`) edits only `role`/`model`/`allowed_tools`/`allowed_skills` — there is no capability surface. So a canvas agent can never be granted the scope that makes its Write land.

### B.2 Files to change
**`<wire_signature_audit>` — pin before pseudocode:** the `Capabilities` shape (`common.v1.json#/$defs/Capabilities` — required `{tools_called, skills_loaded, file_access:{read,write}, network, shell, spawn_agents}`; `file_access.{read,write}: FileGlobList`); `updateNode(nodeId, patch)` merges `{...entry, ...patch}` (`builderStore.ts:519-526`) — a `{capabilities}` patch lands directly; `findAgent` resolves the selected inline agent (`NodeConfigPanel.tsx:12-18`); the existing `AllowedList` editable-list pattern (`:32-79`).
- `src/lib/builderStore.ts` — `builderAgent` initializes a **minimal-valid** `capabilities`: `{ tools_called: [], skills_loaded: [], file_access: { read: [], write: [] }, network: [], shell: false, spawn_agents: [] }` (so a created agent is schema-valid once it has a `role`).
- `src/components/builder/NodeConfigPanel.tsx` — a **file_access** sub-editor: two glob-list editors (Read globs, Write globs) reusing the `AllowedList` shape, writing `agent.capabilities.file_access.{read,write}` via `updateNode(selectedNodeId, { capabilities: nextCaps })`.
- `tests/e2e-tauri/builder_file_access.e2e.ts` (new) + a vitest unit (the editor mutates `capabilities.file_access`; `builderAgent` is schema-valid-shaped).

### B.3 Detailed changes
1. **`builderAgent` gains a valid `capabilities`** (closes the validity gap; the comment at `builderStore.ts:138-143` already names this omission):
   ```ts
   function builderAgent(id: string): Agent {
     return {
       id,
       role: '',
       model: { provider: 'anthropic', id: DEFAULT_MODEL_ID },
       allowed_tools: [],
       allowed_skills: [],
       spawns: [],
       capabilities: {
         tools_called: [],
         skills_loaded: [],
         file_access: { read: [], write: [] },
         network: [],
         shell: false,
         spawn_agents: [],
       },
     } as unknown as Agent;
   }
   ```
2. **The file_access editor** in `NodeConfigPanel`. After the Tools/Skills `AllowedList`s, render a "File access" group with a **Read globs** list and a **Write globs** list (the `AllowedList` component already gives add/remove). Each change recomputes the agent's `capabilities` immutably and calls `updateNode`:
   ```tsx
   const caps = agent.capabilities; // Capabilities (common.v1.json)
   <AllowedList
     label="File read (globs)" testId="node-config-fa-read"
     /* …remove/add testids… */ items={caps.file_access.read}
     onChange={(read) =>
       updateNode(selectedNodeId, {
         capabilities: { ...caps, file_access: { ...caps.file_access, read } },
       })}
   />
   /* …and the symmetric "File write (globs)" editor over caps.file_access.write… */
   ```
   `updateNode` merges the `{capabilities}` patch onto the agent entry (`builderStore.ts:522-523`) and re-schedules validation. The canvas + JSON view re-derive (ADR-0020).
3. **Scope is declaration-only.** The editor writes the agent's *grant* in the document; the L2 enforcer (unchanged) consumes it at run time. Granting `write: ["C:/Users/.../out/**"]` lets the agent's in-scope Write land and leaves an out-of-scope Write denied — the existing E-02 behavior, now *authorable*. The editor never touches the enforcer or the user's tier.

### B.4 Acceptance (BDD)
```gherkin
Feature: Grant an agent file access on the canvas
  Scenario: a written-scope agent can write within scope and only within scope
    Given a canvas agent selected in the Inspector
    When I add a Write glob "out/**" and a Read glob "data/**" under File access
    Then framework.agents[].capabilities.file_access = { read:["data/**"], write:["out/**"] }
    And the framework validates (a created agent with a role + this scope is valid)
    And (M09.D) a Promoted run writes a file under out/ and is denied writing elsewhere
```
Asserted on the real Tauri app via `tauri-driver` (`builder_file_access.e2e.ts`) + the maintainer IRL.

### B.5 construction_reachability_check
`inputs_reachable="false"`: `builderAgent` omits `capabilities` (`builderStore.ts:145`) → a canvas agent is invalid + scope-less; `NodeConfigPanel` has no capability field (`:105-146`) → file_access is unauthorable → every Write is denied. M09.B inverts both: `builderAgent` initializes `capabilities`; the editor writes `file_access.{read,write}`. Verify by grep: `capabilities:` in `builderAgent`; `node-config-fa-read`/`-write` in `NodeConfigPanel`; the e2e asserts the `file_access` mutation.

### B.6 Close gate
Strict v1.11 two-commit TDD (red: the e2e + the unit fail right-reason — no capability editor, `builderAgent` lacks `capabilities`; impl untouching tests). Frontend gates + **both** e2e gates (`test:e2e` renderer + `test:e2e:tauri` real-app). Mutation **advisory** (renderer + a store constructor; no enforcer change — the enforcer already consumes `file_access`, declaration-only). Stage-D design review (the file_access group matches `DESIGN.md`). **No execution-status flip** (the *enforced* write lands at M09.D's run).

### B.7 CLI prompt
```xml
<work_stage_prompt id="M09.B">
  <context>
    M09 Stage B — make an authored agent capable. builderAgent (builderStore.ts:145)
    omits capabilities, which agent.v1.json:9 marks REQUIRED
    (common.v1.json#/$defs/Capabilities); the enforced read/write scope is
    capabilities.file_access.{read,write} (glob lists). NodeConfigPanel
    (NodeConfigPanel.tsx:105-146) edits role/model/allowed_tools/allowed_skills
    only — no capability surface — so a canvas agent has no file_access and EVERY
    Write is denied (E-02 capability_live_tool.rs proves the in-scope/out-of-scope
    split). Add (1) a valid capabilities to builderAgent (all-empty, file_access
    {read:[],write:[]}); (2) a file_access editor in NodeConfigPanel (Read/Write
    glob lists) writing via updateNode(id,{capabilities:next}) — updateNode merges
    {...entry,...patch} (:522), so the patch lands. DECLARATION-ONLY: write the
    agent's grant; never touch the enforcer or the user's tier. The enforced write
    is observed at M09.D. Scope M09 to file_access; the rest of the Capabilities
    surface (network/shell/spawn_agents UI) widens with the ADR-0032 slice that
    executes it (spawn_agents→M11, shell→M12, network→M13); tools_called is handled
    at M09.C (attaching an MCP tool mirrors into it).
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.B — B.1-B.6)</file>
    <file>schemas/common.v1.json (#/$defs/Capabilities @39 — required fields + file_access:{read,write}:FileGlobList) + schemas/agent.v1.json (capabilities @41, required @9)</file>
    <file>src/lib/builderStore.ts (builderAgent @145 — add capabilities; updateNode @519-526 merges the patch) </file>
    <file>src/components/builder/NodeConfigPanel.tsx (findAgent @12; the fields @105-146; the AllowedList pattern @32-79 to reuse)</file>
    <file>crates/runtime-main/tests/capability_live_tool.rs (E-02 — file_access.write enforcement: in-scope lands, out-of-scope blocked) + docs/adr/0020-* + CLAUDE.md §5/§6/§8</file>
  </read_first>
  <deliverable>builderAgent initializes a valid Capabilities (all-empty, file_access {read:[],write:[]}); NodeConfigPanel gains a file_access editor (Read/Write glob lists) writing agent.capabilities.file_access via updateNode. A created agent is schema-valid (given a role) and can be granted the write scope that makes its Write land. Declaration-only — no enforcer/tier change. M09 scopes to file_access; the rest of Capabilities widens per ADR-0032 slice (spawn_agents→M11, shell→M12, network→M13; tools_called via M09.C).</deliverable>
  <tdd_discipline strict="true">Two commits: red (tests/e2e-tauri/builder_file_access.e2e.ts + a vitest unit asserting the file_access mutation + builderAgent's capabilities shape — fail right-reason: no editor, no capabilities) → impl untouching tests (diff over test paths EMPTY).</tdd_discipline>
  <wire_signature_audit>Pin: Capabilities (common.v1.json:39 — required {tools_called,skills_loaded,file_access,network,shell,spawn_agents}; file_access.{read,write}:FileGlobList); updateNode merges {...entry,...patch} (builderStore.ts:522-523); AllowedList (NodeConfigPanel.tsx:32-79); enforcement E-02 (capability_live_tool.rs).</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="B.5"/>
  <execution_steps>
    <step name="ground_at_red" budget="1"/>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3">Frontend gates + BOTH e2e gates (test:e2e renderer + test:e2e:tauri). No Rust/enforcer change (declaration-only — the enforcer already consumes file_access). No schema change.</step>
    <step name="mutation_gate" blocking="false"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="assembled_run_irl"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <close_gate>
    <real_app_irl>Maintainer: select a canvas agent → add a Write glob + a Read glob under File access → the JSON view shows capabilities.file_access updated; the framework validates. (The enforced write lands at M09.D — no flip here.)</real_app_irl>
    <mutation_gate blocking="false">Advisory — renderer + a store constructor; no enforcer change.</mutation_gate>
    <design_review>The file_access group matches DESIGN.md (principle 2 state-visible; the AllowedList register).</design_review>
    <cumulative_regression>builder_file_access.e2e.ts joins the e2e-tauri suite.</cumulative_regression>
  </close_gate>
  <approval_surface>
    <item>cross-machine state + the red→impl diff over test paths (EMPTY)</item>
    <item>frontend gate results + test:e2e:tauri; mutation note (advisory)</item>
    <item>the file_access authoring walkthrough + the validates-now result</item>
    <item>Stage-D design-review note</item>
    <item>explicit: "M09.B is ready. I will not commit until you approve."</item>
  </approval_surface>
  <scope_locks>
    <lock>file_access (read/write globs) is the M09 surface; the rest of Capabilities widens per ADR-0032 slice (spawn_agents→M11, shell→M12, network→M13; tools_called via M09.C).</lock>
    <lock>Declaration-only — write the agent's grant in the document; NEVER touch the enforcer, the L-layers, or the user's tier.</lock>
    <lock>No schema change (author to the Capabilities/Agent shapes).</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>That capabilities was required + omitted (builderAgent minted invalid agents); that file_access.write is what makes a Write land (E-02); declaration-only (no enforcer touch); rule 11 (the enforced write is M09.D's observation).</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Stage M09.B"/>
</work_stage_prompt>
```

### B.8 Commit message
```
feat(M09.B): file_access editor — grant a canvas agent real read/write scope

builderAgent omitted capabilities (required per agent.v1.json:9), so a canvas
agent was schema-invalid and, with no file_access.write, every Write was denied
(E-02). Initialize a valid Capabilities in builderAgent and add a Read/Write
glob editor to NodeConfigPanel writing capabilities.file_access via updateNode.
Declaration-only — the L2 enforcer (unchanged) consumes the grant at run time;
the enforced write is observed at M09.D. file_access is the M09 surface;
the rest of Capabilities widens per ADR-0032 slice (spawn_agents M11, shell M12).
```

---

## Stage M09.C — Attach a real MCP server's tool

### C.1 Problem statement
The data-bearing tools an agent needs are unreachable on the canvas. `mcp_list_servers` (`commands.rs:1083`) lists installed servers and `mcp_test_connection` (`:1049`) enumerates a server's tools (`McpClient::test_connection`→`conn.list_tools()`, `client/mod.rs:246-259`, returning `Vec<McpTool>`), but **no command lists an *already-installed* server's tools by name** — `test_connection` takes an inline `config`, not a registered server name. And the Palette Tools tab (`Palette.tsx:143-160`) offers only built-ins + installed-artifacts + framework tools, never a connected MCP server's tools. Dispatch itself is solved: `try_mcp_dispatch` (`agent_sdk.rs:884`) → `McpDispatcher` (`dispatch.rs:91`) resolves a tool name to `server__tool`, enforces L1/L4, and invokes it — **an `allowed_tools` entry that names an MCP tool already dispatches in a real run.** And the **Tester run path already wires it**: `test_framework` builds `build_test_mcp_dispatcher` + `connect_test_session_mcp` and threads the dispatcher + the tracked tier into `test_framework_with` (`commands.rs:1769-1775` / `:1758` / `:1782-1792`). So the missing links are only *enumerate an installed server's tools by name* + *surface them in the palette* — **no Tester-wiring change**.

### C.2 Files to change
**`<wire_signature_audit>` — pin before pseudocode:** `McpClient::test_connection(transport) -> Vec<McpTool>` (`client/mod.rs:246`); `McpClient::list_servers` (`:270`) + `registry.get(name)` (the registry lookup); `McpClient::transport_from_config(&McpServerConfig)` (used at `commands.rs:1069`); `mcp_list_servers` command shape (`commands.rs:1083-1100`, `*_with` seam); the `McpTool` shape `{name, description?, input_schema}` (Rust `runtime-core`/`runtime-mcp` — **the build pins the exact path**; the Explore-cited `types/mcp.ts` location was wrong); `paletteItemsForTab('tools', …)` (`Palette.tsx:143`). (No Tester-wiring pin needed — `test_framework` already injects `build_test_mcp_dispatcher` + `connect_test_session_mcp`, `commands.rs:1769-1775`.)
- `src-tauri/src/commands.rs` — a new `mcp_list_server_tools(name: String) -> Vec<McpTool>` Tauri command (+ a `*_with` test seam): look the server up in the registry, rebuild its transport, and reuse the `test_connection` connect→`list_tools`→disconnect path (or the cached `get_connection` + `list_tools`). A read-only enumeration; no new MCP machinery.
- `src/lib/ipc.ts` — `mcpListServerTools(name): Promise<McpTool[]>`.
- `src/components/builder/Palette.tsx` — the Tools tab gains the installed servers' tools as `source:'mcp'` items (labelled `server · tool`), fetched once on mount like `listInstalledArtifacts` (`:214-222`); a drop adds the tool to `framework.tools` (`applyDrop` 'tool' case, `builderStore.ts:190-197`) and an Agent→Tool edge wires `allowed_tools` (`connectEdgeReducer:419`); B's editor mirrors it into `capabilities.tools_called`.
- **No Tester-wiring change (precondition already met):** `test_framework` (`commands.rs:1769-1775`) already builds `build_test_mcp_dispatcher` + drives `connect_test_session_mcp` for the candidate framework's servers and threads the dispatcher (and the tracked tier, `:1758`) into `test_framework_with` (`:1782-1792`). The authored MCP tool dispatches in a Tester run without any change here — C touches only the new read-only enumeration command + the palette.
- `tests/e2e-tauri/builder_mcp_tool.e2e.ts` (new) + a Rust unit on `mcp_list_server_tools_with` (a stub/registered server's tools enumerate) + a vitest on the palette items.

### C.3 Detailed changes
1. **`mcp_list_server_tools` (Rust).** Mirror `mcp_test_connection_with` (`commands.rs:1061-1075`) but key on a registered name — `registry.get(name)` → `transport_from_config` → `test_connection(transport)` (or `get_connection(name, …)` + `list_tools`). Returns `Vec<McpTool>`. Read-only; no persistence.
2. **`mcpListServerTools` (ipc.ts).** A thin `invoke('mcp_list_server_tools', { name })` wrapper, mirroring `mcpTestConnection` (the existing `Vec<McpTool>` bridge).
3. **Palette MCP tools.** On mount, for each `mcp_list_servers()` entry, fetch its tools and add `{ kind:'tool', ref: '<server>__<tool>' (or the short tool name the resolver accepts), label:'<server> · <tool>', source:'mcp' }` to the Tools tab, de-duped against built-ins/installed/framework (the existing `dedupeByKindRef`, `Palette.tsx:120`). The drag contract is unchanged (`{kind:'tool', ref}`).
4. **Wire to the agent.** Dropping the MCP tool node + an Agent→Tool edge records it in `allowed_tools` (`connectEdgeReducer:419`); the M09.B file_access editor's sibling pattern adds it to `capabilities.tools_called`. At run time, `try_mcp_dispatch` resolves the name to the server's tool and dispatches it (already executing).
5. **No Tester dispatcher wiring.** `test_framework` already calls `build_test_mcp_dispatcher` + `connect_test_session_mcp` (`commands.rs:1769-1775`), so a Tester run dispatches MCP tools as-is. C adds no execution-wiring; the mutation gate is **advisory** (the new command is a read-only `list_tools` wrapper).

### C.4 Acceptance (BDD)
```gherkin
Feature: Attach a real MCP server's tool to an agent
  Scenario: an installed server's tools are draggable and dispatch in a run
    Given an MCP server installed (e.g. a filesystem or GitHub MCP server)
    When I open the Tools palette tab
    Then I see that server's tools (labelled "server · tool", source mcp)
    When I drag one onto the canvas and connect my agent to it
    Then framework.agents[].allowed_tools contains the tool
    And (M09.D) running the framework dispatches the MCP tool and feeds its result back
```
Asserted on the real Tauri app via `tauri-driver` + the maintainer IRL (a real installed server).

### C.5 construction_reachability_check
`inputs_reachable="false"`: no command lists an installed server's tools by name (only `test_connection` by inline config); the Palette Tools tab never surfaces MCP tools (`Palette.tsx:143-160`). (The Tester dispatcher is *already* reachable — `build_test_mcp_dispatcher` is called from `test_framework` at `commands.rs:1769-1775` — so it is **not** a missing input.) M09.C inverts the two real gaps: the new enumeration command + the `source:'mcp'` palette source. Verify by grep: `mcp_list_server_tools` in `commands.rs` + `ipc.ts`; `source:'mcp'` items in `Palette.tsx`.

### C.6 Close gate
Strict v1.11 two-commit TDD (red: the e2e + the Rust `*_with` unit + the palette vitest fail right-reason — no command, no MCP palette items; impl untouching tests). Rust gates (runtime-main/`src-tauri`) + frontend gates + **both** e2e gates (`test:e2e` renderer + `test:e2e:tauri` real-app). **Mutation gate: advisory** — the new command is a read-only `list_tools` wrapper and C touches no `test_framework` wiring (the dispatcher is already injected, `commands.rs:1769-1775`); the standing blocking rule applies only if an executor/dispatch branch is unexpectedly touched (`cluster-pattern.md` §5). Stage-D design review (the `source:'mcp'` palette items match `DESIGN.md`). **No execution-status flip** (the dispatched-in-a-run observation is M09.D).

### C.7 CLI prompt
```xml
<work_stage_prompt id="M09.C">
  <context>
    M09 Stage C — make a real MCP server's tools attachable on the canvas. MCP
    dispatch ALREADY executes in the run loop (agent_sdk.rs:884 try_mcp_dispatch →
    dispatch.rs:91, capability-enforced) — an allowed_tools entry naming an MCP
    tool dispatches — AND the Tester run path ALREADY wires it: test_framework
    builds build_test_mcp_dispatcher + connect_test_session_mcp and threads the
    dispatcher + the tracked tier into test_framework_with (commands.rs:1769-1775 /
    :1758 / :1782-1792). So C adds NO execution-wiring; only two real gaps remain:
    (1) no command lists an INSTALLED server's tools by name (mcp_test_connection:1049
    takes an inline config; McpClient::test_connection →list_tools, client/mod.rs:246,
    returns Vec<McpTool>); (2) the Palette Tools tab (Palette.tsx:143-160) never
    surfaces MCP tools. Add mcp_list_server_tools(name)→Vec<McpTool> (reuse the
    registry.get + transport + list_tools path), an ipc wrapper, and source:'mcp'
    palette items; a drop + Agent→Tool edge records allowed_tools (connectEdge:419)
    + capabilities.tools_called. Reuse existing MCP machinery; no new transport; do
    NOT touch test_framework (the dispatcher is already injected). Mutation
    ADVISORY (read-only list_tools wrapper). NOTE: the McpTool type location the
    prior survey cited (types/mcp.ts) was WRONG — pin the real Rust/TS McpTool
    {name, description?, input_schema} in the audit.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.C — C.1-C.6; the test_framework dispatcher precondition)</file>
    <file>src-tauri/src/commands.rs (mcp_test_connection @1049 + _with @1061; mcp_list_servers @1083; test_framework @1745 — ALREADY wires build_test_mcp_dispatcher + connect_test_session_mcp + the tracked tier @1758/1769-1775/1782-1792, confirm not change)</file>
    <file>crates/runtime-mcp/src/client/mod.rs (test_connection @246 → list_tools; list_servers @270; get_connection @289) + crates/runtime-mcp/src/dispatch.rs (McpDispatcher @91) + crates/runtime-main/src/sdk/agent_sdk.rs (try_mcp_dispatch @884)</file>
    <file>src/components/builder/Palette.tsx (the tools case @143-160; listInstalledArtifacts-on-mount @214-222; dedupeByKindRef @120; the drag contract @269-279) + src/lib/ipc.ts (mcpTestConnection / mcpListServers — the Vec<McpTool> bridge) + src/lib/builderStore.ts (applyDrop 'tool' @190; connectEdgeReducer agent->tool @419)</file>
    <file>schemas/mcp.v1.json (McpServerConfig/McpTool) + docs/adr/0021-* + docs/cluster-pattern.md (§5 mutation gate) + CLAUDE.md §5/§6/§8</file>
  </read_first>
  <deliverable>A new mcp_list_server_tools(name)→Vec<McpTool> command (+ _with seam) reusing the registry+list_tools path; an mcpListServerTools ipc wrapper; the Palette Tools tab surfaces an installed server's tools (source:'mcp'); a drop + Agent→Tool edge records allowed_tools + capabilities.tools_called. No test_framework change — the dispatcher + tier are already wired (commands.rs:1769-1775/1758). Reuses existing MCP machinery; no new transport.</deliverable>
  <tdd_discipline strict="true">Two commits: red (tests/e2e-tauri/builder_mcp_tool.e2e.ts + a Rust mcp_list_server_tools_with unit + a Palette source:'mcp' vitest — fail right-reason: no command, no MCP palette items) → impl untouching tests (diff over test paths EMPTY).</tdd_discipline>
  <wire_signature_audit>Pin: McpClient::test_connection(transport)->Vec<McpTool> (client/mod.rs:246); registry.get(name) + transport_from_config (commands.rs:1069); the McpTool shape {name,description?,input_schema} (pin the REAL Rust+TS path — the prior types/mcp.ts cite was wrong); test_framework's existing dispatcher wiring (build_test_mcp_dispatcher + connect_test_session_mcp, commands.rs:1769-1775 — CONFIRM present, do not change); connectEdgeReducer agent->tool (builderStore.ts:419).</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="C.5"/>
  <execution_steps>
    <step name="ground_at_red" budget="1"/>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3">Rust gates (runtime-main ≥95 / src-tauri) + frontend gates + BOTH e2e gates (test:e2e renderer + test:e2e:tauri). No test_framework change; no schema change.</step>
    <step name="mutation_gate" blocking="false"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="assembled_run_irl"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <close_gate>
    <real_app_irl>Maintainer: with an MCP server installed, the Tools tab shows its tools (source mcp); drag one + connect the agent → allowed_tools records it. (The dispatched-in-a-run observation is M09.D.)</real_app_irl>
    <mutation_gate blocking="false">Advisory — the new command is a read-only list_tools wrapper and C touches no test_framework wiring (the dispatcher is already injected, commands.rs:1769-1775). The standing blocking rule applies only if an executor/dispatch branch is unexpectedly touched.</mutation_gate>
    <design_review>The source:'mcp' palette items match DESIGN.md (the palette item style + source badge).</design_review>
    <cumulative_regression>builder_mcp_tool.e2e.ts joins the e2e-tauri suite; the mcp_list_server_tools unit joins the Rust suite.</cumulative_regression>
  </close_gate>
  <approval_surface>
    <item>cross-machine state + the red→impl diff over test paths (EMPTY)</item>
    <item>the confirmation that test_framework already wires the dispatcher + tier (commands.rs:1769-1775/1758 — no change) + the advisory mutation note</item>
    <item>Rust + frontend gate results + both e2e gates (test:e2e + test:e2e:tauri)</item>
    <item>the attach-MCP-tool walkthrough + Stage-D design-review note</item>
    <item>explicit: "M09.C is ready. I will not commit until you approve."</item>
  </approval_surface>
  <scope_locks>
    <lock>Reuse existing MCP machinery — the new command only enumerates (registry + list_tools); no new transport, no dispatch change.</lock>
    <lock>MCP-server-as-canvas-node + the data-source catalog + credentials UX are M13 (industrialize) — M09 only attaches an installed server's tool to an agent.</lock>
    <lock>Do NOT touch test_framework — the dispatcher + tier are already injected (commands.rs:1769-1775/1758); C is reuse-only, mutation advisory.</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>That test_framework already wired the dispatcher + tier (commands.rs:1769-1775/1758) so C added no execution-wiring; that dispatch already executed (the gap was enumeration + palette); the McpTool-type-location correction (prior survey wrong); rule 11 (dispatch-in-run observed at M09.D).</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Stage M09.C"/>
</work_stage_prompt>
```

### C.8 Commit message
```
feat(M09.C): attach a real MCP server's tool on the canvas

MCP dispatch already executed in the run loop (agent_sdk.rs:884) and the
Tester already wires it (test_framework builds build_test_mcp_dispatcher +
connect_test_session_mcp, commands.rs:1769-1775), but no command listed an
installed server's tools by name and the palette never surfaced them. Add
mcp_list_server_tools(name) (reusing the registry + list_tools path), an ipc
wrapper, and source:'mcp' Tools-tab items; a drop + Agent->Tool edge records
allowed_tools + capabilities.tools_called. No test_framework change (the
dispatcher + tier are already injected); mutation advisory. Reuses existing
MCP machinery; the data-source catalog + MCP-as-node are M13.
```

---

## Stage M09.D — The vertical-slice IRL (assembled, end-to-end)

### D.1 Problem statement
M09.A–C are unit/e2e-green in isolation, but **a canvas-authored framework has never run in the assembled app**: build a fresh agent → grant `file_access.write` → attach an MCP tool → Run → real data pulled + a real file written, at the tracked tier. Per the assembled-app-regression mandate (CLAUDE.md v1.8), the close is an assembled test that exercises the **real Tester run path** (`test_framework` against the real `AnthropicProvider`, real in-process Read/Write, real MCP dispatch, the real L2/L4 enforcer) — not the isolated pieces. The individual wires are already present and confirmed: the MCP dispatcher + the tracked tier (`commands.rs:1769-1775` / `:1758`) and the enforcer built from the framework's agents via `grant_framework_capabilities` → `AgentSdk::with_capability_wiring` (`tester.rs:36/445`, the construction E-02 proves). The falsifiable hypothesis is therefore about the **composition, not a known-missing wire**: *a canvas-authored agent (capabilities + MCP tool authored through M09.A–C, serialized across the Tauri IPC boundary into `framework_doc`) runs exactly as a hand-written-JSON framework — the authored `file_access.write` gates the run and a real file lands* (structure ≠ behavior — rule 11). If it fails, the gap is a serialization/threading defect in that composed path, closed here.

### D.2 Files to change
**`<wire_signature_audit>` — pin before pseudocode (CONFIRM already-wired, do not rebuild):** `test_framework` (`commands.rs:1745`) — the tracked tier read from `CurrentTierState` (`:1758`), `build_test_mcp_dispatcher` + `connect_test_session_mcp` threaded into `test_framework_with` (`:1769-1775` / `:1782-1792`), the enforcer built from the framework's agents via `grant_framework_capabilities` → `AgentSdk::with_capability_wiring` (`tester.rs:36/445`); the Tester UI run trigger (`openTester`/`testerOpen`, `builderStore.ts:541`); the execution-status row to flip.
- `tests/e2e-tauri/vertical_slice.e2e.ts` (new — the assembled close): drive the built app through `tauri-driver` to author the agent + scope + MCP tool and Run, asserting a real file is written within scope (and an out-of-scope target is denied). Key-gated on a live model + an installed MCP server (the maintainer's IRL machine; CI runs the key-independent subset).
- Any **serialization/threading glue** the composed authored→run path surfaces (the dispatcher/tier/capabilities-granting are already wired) — closed here; mutation advisory unless a `test_framework` executor/dispatch branch must be touched (then blocking, surfaced first).
- `docs/execution-status.md` — flip a new row: *canvas-authored single-agent + file_access + MCP-tool, observed end-to-end in the app, eval `vertical_slice.e2e.ts` + the IRL date.*

### D.3 Detailed changes
1. **The assembled e2e** authors the framework through the real UI (reusing M09.A–C surfaces) and Runs it via the Tester, asserting the on-disk side effect (a file written under the granted `write` glob from MCP-sourced data) + the out-of-scope denial — the observable behavior, not an emitted event (rule 11).
2. **Close any serialization/threading glue** the composed authored path exposes — the dispatcher/tier/capabilities-granting are already wired (`commands.rs:1769-1775` / `:1758`; `tester.rs:445`), so this is glue (e.g. an authored field not surviving the IPC round-trip), not new wiring. Minimal, gated.
3. **Flip the execution-status row** on the maintainer's real-app IRL — the authoritative close (rule 11 / ADR-0021), not the CI e2e alone.

### D.4 Acceptance (BDD)
```gherkin
Feature: Build and run a real workflow end-to-end in the app
  Scenario: a from-scratch agent pulls real MCP data and writes a real file
    Given a fresh project, Promoted tier
    And I author an agent, grant file_access.write "out/**", and attach an installed MCP server's tool
    When I Run it in the Tester
    Then the agent calls the MCP tool, receives real data, and writes a file under out/
    And a write outside out/ is denied (no file on disk)
    And the run is observable in the app (the Output rail / nodes), not only RUST_LOG
```
The maintainer IRL on the real Tauri app is the authoritative close (rule 11 / ADR-0021); `vertical_slice.e2e.ts` is the regression.

### D.5 construction_reachability_check
`inputs_reachable="false"` (the composition hypothesis): the dispatcher, the tracked tier, and capabilities-granting are each individually wired (`commands.rs:1769-1775`/`:1758`; `tester.rs:445`), but **no canvas-authored framework has ever run end-to-end** — the authored→serialized→run path is unproven (structure ≠ behavior). M09.D inverts it by running the real composition and asserting the on-disk effect; any serialization/threading gap is closed here. Verify: `vertical_slice.e2e.ts` writes a real file from MCP data within scope and is denied outside it, on the built app.

### D.6 Close gate
Strict v1.11 two-commit TDD on any glue (red: the assembled e2e fails right-reason; impl untouching tests). Rust + frontend gates + **both** e2e gates (`test:e2e` renderer + `test:e2e:tauri` real-app). **Mutation gate** advisory by default — the dispatcher/tier/capabilities-granting are already wired, so D's glue (if any) is serialization, not execution-wiring; the gate is **BLOCKING** only *if* D must touch a `test_framework` executor/dispatch/enforcer-wiring branch (surfaced first, Hard Rule 8). **The maintainer real-app IRL is the authoritative close** — author fresh → scope → attach MCP tool → Run → real file written within scope, denied outside, observable in-app. **Flip the execution-status row** citing `vertical_slice.e2e.ts` + the IRL date. This closes M09 — the vertical slice is real.

### D.7 CLI prompt
```xml
<work_stage_prompt id="M09.D">
  <context>
    M09 Stage D — the assembled vertical-slice IRL. A-C are green in isolation; a
    CANVAS-AUTHORED framework has never run in the real app. Per the v1.8
    assembled-app mandate, the close is an assembled test on the REAL Tester run
    path (test_framework vs the real AnthropicProvider, real in-process Read/Write,
    real MCP dispatch, the real L2/L4 enforcer): author a fresh agent → grant
    file_access.write → attach an installed MCP server's tool → Run → assert a real
    file is written within scope from real MCP data, and an out-of-scope write is
    denied (the on-disk effect, not an event — rule 11). The individual wires are
    ALREADY present and confirmed — the MCP dispatcher + the tracked tier
    (commands.rs:1769-1775/:1758) and the enforcer built from the framework's agents
    via grant_framework_capabilities → AgentSdk::with_capability_wiring
    (tester.rs:36/445, the E-02 construction). So the falsifiable hypothesis is about
    the COMPOSITION, not a known-missing wire: a canvas-authored agent (capabilities
    + MCP tool authored through M09.A–C, serialized across the Tauri IPC into
    framework_doc) runs exactly as hand-written JSON — the authored file_access.write
    gates the run and a real file lands (structure ≠ behavior). If it fails, the gap
    is a serialization/threading defect, closed here — never a new execution
    primitive. Mutation ADVISORY unless D must touch a test_framework
    executor/dispatch branch (surface first, Hard Rule 8). The maintainer real-app
    IRL is the authoritative close; flip the execution-status row. This closes M09.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.D — D.1-D.6; the Background falsifiable hypothesis)</file>
    <file>src-tauri/src/commands.rs (test_framework @1745 — the run path: tier from CurrentTierState @1758, build_test_mcp_dispatcher + connect_test_session_mcp @1769-1775, threaded into test_framework_with @1782-1792 — ALREADY wired; confirm, do not rebuild) + crates/runtime-main/src/builder/tester.rs (grant_framework_capabilities + AgentSdk::with_capability_wiring @36/445 — the enforcer is built from the framework's agents)</file>
    <file>crates/runtime-main/tests/capability_live_tool.rs (E-02 file_access enforcement) + crates/runtime-main/src/sdk/agent_sdk.rs (the run loop + try_mcp_dispatch @884)</file>
    <file>src/lib/builderStore.ts (openTester/testerOpen @541; useTestGraphStore @444) + the M09.A-C surfaces (the authored framework)</file>
    <file>docs/execution-status.md (the row to flip + the maintenance protocol) + docs/cluster-pattern.md (§1 IRL close, §5 mutation) + docs/adr/0021-* + CLAUDE.md §4 rule 11 / §5/§6/§8</file>
  </read_first>
  <deliverable>An assembled tests/e2e-tauri/vertical_slice.e2e.ts that drives the built app to author a fresh agent + file_access.write + an installed MCP tool and Run it, asserting a real file is written within scope from real MCP data and an out-of-scope write is denied; any serialization/threading glue the test surfaces in the composed authored→run path, closed (the dispatcher/tier/capabilities-granting are already wired); the execution-status row flipped on the maintainer IRL. Closes M09.</deliverable>
  <tdd_discipline strict="true">Two commits: red (vertical_slice.e2e.ts + any glue's assembled regression fail right-reason: the canvas-authored run does not write the file / does not dispatch MCP / the authored file_access does not gate) → impl untouching tests (diff over test paths EMPTY).</tdd_discipline>
  <wire_signature_audit>Pin (CONFIRM already-wired, do not rebuild): test_framework run path — tier from CurrentTierState (commands.rs:1758), build_test_mcp_dispatcher + connect_test_session_mcp (:1769-1775) threaded into test_framework_with (:1782-1792); the enforcer built from the framework's agents via grant_framework_capabilities → AgentSdk::with_capability_wiring (tester.rs:36/445); openTester/testerOpen (builderStore.ts:541); the execution-status row + maintenance protocol.</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="D.5"/>
  <execution_steps>
    <step name="ground_at_red" budget="1"/>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3">Rust + frontend gates + BOTH e2e gates (test:e2e renderer + test:e2e:tauri). Surface any test_framework executor/dispatch touch BEFORE coding (Hard Rule 8 → mutation BLOCKING on that diff); none expected — the wiring exists. No schema change.</step>
    <step name="mutation_gate" blocking="false"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="assembled_run_irl"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <close_gate>
    <real_app_irl>Maintainer (Promoted), on the real Tauri app: fresh project → author an agent → grant file_access.write "out/**" → attach an installed MCP server's tool → Run in the Tester → a real file is written under out/ from real MCP data; a write outside out/ is denied; the run is observable in-app. THE authoritative close (rule 11 / ADR-0021).</real_app_irl>
    <mutation_gate blocking="false">Advisory by default — the dispatcher/tier/capabilities-granting are already wired, so D's glue (if any) is serialization, not execution-wiring. BLOCKING only if D must touch a test_framework executor/dispatch/enforcer-wiring branch (surfaced first, Hard Rule 8).</mutation_gate>
    <design_review>The end-to-end run surfaces (Tester, Output rail, nodes) match DESIGN.md.</design_review>
    <cumulative_regression>vertical_slice.e2e.ts joins the e2e-tauri suite; the execution-status row joins the cumulative execution-regression eval.</cumulative_regression>
  </close_gate>
  <approval_surface>
    <item>cross-machine state + the red→impl diff over test paths (EMPTY)</item>
    <item>the composition finding (the canvas-authored→serialized→run path confirmed, or the serialization glue closed here) + the mutation result (advisory unless a test_framework executor branch was touched)</item>
    <item>Rust + frontend gates + both e2e gates (test:e2e + test:e2e:tauri); the execution-status row flip (citing vertical_slice.e2e.ts + the IRL date)</item>
    <item>the full real-app IRL walkthrough (author → scope → attach MCP → Run → real file, denied out-of-scope, observable)</item>
    <item>explicit: "M09.D is ready. I will not commit until you approve."</item>
  </approval_surface>
  <scope_locks>
    <lock>The vertical slice only — single agent, one MCP tool, file_access write, "suspends cleanly" (E-04). Per ADR-0032: gap resolve→resume + plans → M10; sub-agents → M11; hooks/rails + shell-exec → M12; the data-source catalog → M13.</lock>
    <lock>Any test_framework touch is serialization/threading glue to make the AUTHORED framework run as a hand-written one already does — never a new execution primitive, never an enforcer-widening; the dispatcher/tier/capabilities-granting are already wired.</lock>
    <lock>The flip requires the maintainer real-app IRL observation (rule 11) — CI e2e-green alone does NOT flip the row.</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Whether the assembled test disproved or confirmed the composed authored→serialized→run hypothesis (the individual wires were already present); the on-disk-effect assertion (not an event — rule 11); the execution-status flip + IRL date; that this is the first canvas-authored real run.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Stage M09.D"/>
</work_stage_prompt>
```

### D.8 Commit message
```
feat(M09.D): the vertical slice runs — author one real agent, write a real file

Close the whole loop in the assembled app: a from-scratch canvas agent +
file_access.write + an installed MCP server's tool runs via the Tester
(test_framework, real provider, real Read/Write, real MCP dispatch, real
L2/L4 enforcer) and writes a real file from real MCP data within scope, denied
outside it. The dispatcher/tier/capabilities-granting are already wired
(commands.rs:1769-1775/1758; tester.rs:445); close any serialization/threading
glue in the composed authored->run path the assembled test surfaces. Flip the
execution-status row on the maintainer real-app IRL. The workbench now builds
AND runs a real single-agent, MCP-data workflow from scratch.
```

---

## Stage M09.D.fix — Surface the canvas-authored MCP tool to the model (+ the DESIGN.md disclosure pass)

> **Reopened by the M09.D maintainer IRL — the slice did not run.** An agent authored entirely correctly — `allowed_tools: ["fs__read_text_file","Write"]`, `capabilities.tools_called` matching, `file_access` set, `session_root_agent: "agent-1"`, validates clean, both tools wired on the canvas — ran in the Tester with **only the built-in `Write`** in the tool list the model saw. It reported *"I don't have a filesystem read tool, only Write,"* requested a `filesystem_read` capability, wrote no file (`result: null`), and the verdict still read **PASS**. The canvas-authored MCP tool is recorded but **never surfaced to the model at run time**: `connect_test_session_mcp` + `build_test_mcp_dispatcher` (M09.C, `commands.rs:1769-1775`) wire *dispatch*, but the tool's *definition* is never injected into the model's tool list, so the model never emits the call and `try_mcp_dispatch` is never reached. The one layer no test hit — `vertical_slice.e2e.ts` ran tool-free, `builder_mcp_tool.e2e.ts` was store-driven; the maintainer IRL is the first real-model-meets-real-MCP-tool run, and it found the hole (rule 11 — exactly why the flip waited on the IRL, not the green e2e). **Bundled (maintainer-directed):** a DESIGN.md-grounded UI pass — Tester expand/resize + settings progressive disclosure — required to re-verify in a usable window. The 🔴 wiring is its own mutation-blocked commit; the two UI changes ride beside it as separate commits. Closes the 🔴 and re-licenses the execution-status flip on the maintainer's re-run IRL.

### D.fix.1 Problem statement
The run path builds the tool list handed to the model by resolving the session-root agent's `allowed_tools` to tool definitions: the built-ins (`Read`/`Write`/`Bash`) resolve to their hand-written defs and `request_capability_tool_def()` is auto-injected (the `run_test_session_inner` construction), but an `allowed_tools` entry that names an **MCP tool** (`server__tool`, e.g. `fs__read_text_file`) has **no resolver** — it is silently dropped, so the model is never told the tool exists. `connect_test_session_mcp` + `build_test_mcp_dispatcher` (M09.C, `commands.rs:1769-1775`) establish *dispatch* (a tool call *would* route to the server), but nothing fetches the connected server's `list_tools` schema and injects the authored MCP tool's definition into the model-facing list. Net: the agent runs tool-blind to its own MCP tool, `try_mcp_dispatch` (`agent_sdk.rs`) is never reached because the model never emits the call, no file is written, and the gap-suspend (the agent requesting the missing capability) reads as a green PASS.

### D.fix.2 Files to change
**`<wire_signature_audit>` — the build pins the exact file:line at red** (the M09.C run-path diff is local-only at authoring time; this section is authored against the IRL evidence + the architecture, not the unpushed diff):
- The **model tool-list builder** on the `test_framework` → `run_test_session_inner` path — where the built-in tool defs + `request_capability_tool_def()` are pushed onto the agent's tool list. This is where MCP tool defs must also be injected. *Build pins the fn + file:line.*
- The **connected MCP tool source** — the `McpDispatcher` / the test session's connected servers, which hold (or fetch) each server's `list_tools` `Vec<McpTool> { name, description?, input_schema }` (`crates/runtime-mcp/src/transport/mod.rs:61`; dispatch `crates/runtime-mcp/src/dispatch.rs`). The injector reads from here and maps `server__tool` → its definition.
- The **`server__tool` naming** — the same canonical id M09.C records in `allowed_tools` + `try_mcp_dispatch` resolves; the injected def's `name` must match it so the model's call routes.
- **UI:** `src/components/builder/TesterModal.tsx` + the `Modal` primitive (M08.8.B — z-index/scroll); `src/components/SettingsPanel.tsx`; `DESIGN.md` (the progressive-disclosure / panel-sizing principles to cite).

Files: the run-path tool-list builder (Rust — the 🔴, its own commit); the assembled stub-MCP regression + the two UI tests; `src/components/builder/TesterModal.tsx`; `src/components/SettingsPanel.tsx`; `docs/execution-status.md` (the MCP-dispatch row reconcile + the M09 slice row, flipped on the re-IRL).

### D.fix.3 Detailed changes
1. **Inject the authored MCP tool definitions into the model's tool list (the 🔴).** In the run-path tool-list construction, after the built-ins + `request_capability`, for each `allowed_tools` entry that is an MCP tool (`server__tool`), resolve its definition from the connected MCP source (`list_tools` schema) and push it, named with the same `server__tool` id so the model's call routes through `try_mcp_dispatch`. Read-through only; no new transport, no dispatch change. Mutation gate **BLOCKING** (this branch decides whether the tool runs — execution-wiring, `cluster-pattern.md` §5).
2. **The assembled regression** (v1.8 mandate — reproduces the IRL): through the real `run_test_session_inner` path, a stub MCP source exposing one tool (`read_text_file` → known content) + a stub `LLMProvider` that **captures the `AgentConfig.tools` it receives** and scripts a call to that tool (mirror the E-02 `WriteToolStub` pattern). Assert (a) the captured tool list **includes** the authored MCP tool — the direct assertion that fails today; and (b) end-to-end, the agent calls it, the stub dispatch returns the content, the built-in `Write` lands the file. Red must first fail with the MCP tool **absent** from the provider's tools.
3. **The UI pass (separate commits, DESIGN.md-grounded).** `TesterModal` gains expand/fullscreen + a resizable, scrollable output/disclosure pane; `SettingsPanel` gains progressive-disclosure (collapsible) sections. Each cites the DESIGN.md principle it implements; each gets its own render/e2e test.
4. **Reconcile the ledger.** `docs/execution-status.md` claims "MCP dispatch executes." This IRL shows the model was never offered the tool, so that claim was not true through the assembled model path. Correct/qualify the row to what is actually proven, and re-assert it on the green of #2 + the maintainer IRL (rule 11).

### D.fix.4 Tests
The assembled stub-MCP regression (#2) is the core — it pins the model-facing tool-list injection and the end-to-end dispatch→`Write`→file. Plus the two UI render/e2e tests. No weakening of any M09.A–D test.

### D.fix.5 construction_reachability_check
`inputs_reachable="false"`: the authored MCP tool is in `allowed_tools` (+ `tools_called`, validated, canvas-wired) but **unreachable to the model** — the tool-list builder drops it, so the model can't call it and `try_mcp_dispatch` is never reached. M09.D.fix inverts it: the builder injects the MCP tool def from the connected source. Verify by the assembled test — the provider's captured tool list contains `fs__read_text_file` and the run writes the file; and by the maintainer re-IRL writing a real `result.txt` in-scope, denied out-of-scope.

### D.fix.6 Close gate
Strict v1.11 two-commit TDD: write all failing tests (the assembled stub-MCP regression + the two UI tests), confirm right-reason red, commit `test(M09.D.fix): …`, **surface red for approval**; then implement as **three distinct commits** untouching tests — (i) the MCP-tool injection [mutation **BLOCKING**], (ii) Tester resize, (iii) settings disclosure — `git diff <red>..<impl> -- '**/tests/**'` EMPTY across them (binary-crate variant via a scoped `#[cfg(test)]` diff if the Rust test is in-source). Full Rust (runtime-main/runtime-mcp ≥95) + frontend gates + **both** e2e legs. **The maintainer real-app re-IRL is the authoritative close** — re-run steps 4–5 in the resizable Tester: a real `C:/…/out/result.txt` lands in-scope, an out-of-scope write is denied, observable in-app. **Flip the execution-status row** on that observation (and the reconciled MCP-dispatch row), citing `vertical_slice.e2e.ts` + the stub-MCP regression + the IRL date. This closes M09.D.

### D.fix.7 CLI prompt
```xml
<work_stage_prompt id="M09.D.fix">
  <context>
    M09.D.fix — the M09.D maintainer IRL disproved the slice. An agent authored
    correctly (allowed_tools ["fs__read_text_file","Write"], tools_called matching,
    file_access set, session_root_agent "agent-1", validates, canvas-wired) ran in
    the Tester with ONLY built-in Write in the model's tool list — it reported "no
    read tool", requested a filesystem_read capability, wrote nothing (result null),
    verdict PASS. The canvas-authored MCP tool is recorded but never surfaced to the
    model: connect_test_session_mcp + build_test_mcp_dispatcher (M09.C,
    commands.rs:1769-1775) wire DISPATCH, but the tool DEFINITION is never injected
    into the model's tool list, so the model never emits the call and try_mcp_dispatch
    is never reached. Painted, not wired, at the one layer no test hit
    (vertical_slice.e2e.ts ran tool-free; builder_mcp_tool.e2e.ts store-driven).
    PRIMARY (the red): inject the authored MCP tool's definition (server__tool →
    list_tools schema from the connected source) into the model tool-list builder so
    the model can call it; mutation BLOCKING on that branch (execution-wiring).
    BUNDLED (maintainer-directed) UI pass, DESIGN.md-grounded, SEPARATE commits:
    TesterModal expand/resize + SettingsPanel progressive disclosure. Reconcile the
    execution-status "MCP dispatch executes" claim (never true through the model).
    Surface red first; no flip until the maintainer re-runs the IRL green.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.D.fix — D.fix.1–D.fix.6; Stage D + the Background falsifiable hypothesis)</file>
    <file>the M09.C run-path diff (local): the run_test_session_inner tool-list construction (where built-ins + request_capability_tool_def() are pushed — PIN the fn + file:line); connect_test_session_mcp + build_test_mcp_dispatcher (commands.rs:1769-1775)</file>
    <file>crates/runtime-mcp/src/transport/mod.rs:61 (McpTool {name,description?,input_schema}) + crates/runtime-mcp/src/dispatch.rs (McpDispatcher — the connected tools source) + crates/runtime-main/src/sdk/agent_sdk.rs (try_mcp_dispatch)</file>
    <file>crates/runtime-main/tests/capability_live_tool.rs (E-02 — the stub-provider + assembled-run pattern to mirror) + docs/cluster-pattern.md (§4 assert-the-side-effect, §5 mutation gate)</file>
    <file>DESIGN.md (progressive-disclosure / panel-sizing principles to cite) + src/components/builder/TesterModal.tsx + the Modal primitive + src/components/SettingsPanel.tsx + docs/adr/0021-* + CLAUDE.md §4 rule 11 / §5/§6/§8</file>
  </read_first>
  <deliverable>The run-path tool-list builder injects each authored MCP tool's definition (server__tool → the connected server's list_tools schema) into the model's tool list, so a canvas-authored MCP tool is callable and try_mcp_dispatch executes it; an assembled stub-MCP regression proves the tool reaches the provider's tool list AND the end-to-end dispatch→Write lands the file; the execution-status MCP-dispatch claim reconciled. Bundled UI (separate commits, DESIGN.md-grounded): TesterModal expand/resize + SettingsPanel progressive disclosure. The red wiring is its own mutation-blocked commit. Closes M09.D.</deliverable>
  <tdd_discipline strict="true">Write all failing tests first (the assembled stub-MCP regression — the provider's captured AgentConfig.tools lacks the MCP tool and the file isn't written; the Tester-resize e2e; the settings-disclosure render) → commit test(M09.D.fix): … (red) → SURFACE RED. Then implement as THREE distinct commits untouching tests (MCP injection [mutation BLOCKING]; Tester resize; settings disclosure); git diff &lt;red&gt;..&lt;impl&gt; over test paths EMPTY (binary-crate variant via a scoped #[cfg(test)] diff if the Rust test is in-source). Net-new/mechanical test fixups in separate labelled follow-ups.</tdd_discipline>
  <wire_signature_audit>PIN at red (the M09.C run-path diff is local — author confirms symbols, build pins file:line): the model tool-list builder on test_framework → run_test_session_inner (where built-in defs + request_capability_tool_def() are pushed); the connected MCP tools source (McpDispatcher / the test session's servers, holding list_tools Vec&lt;McpTool&gt;); the server__tool naming (must match allowed_tools + try_mcp_dispatch). UI: TesterModal + Modal primitive; SettingsPanel; the DESIGN.md principle ids.</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="D.fix.5"/>
  <execution_steps>
    <step name="ground_at_red" budget="1"/>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="3"/>
    <step name="verify_gates" budget_iterations="3">Rust ≥95 (runtime-main/runtime-mcp) + frontend gates + BOTH e2e legs (test:e2e + test:e2e:tauri). No schema change.</step>
    <step name="mutation_gate" blocking="true"/>
    <step name="green_phase_commit" budget="3"/>
    <step name="assembled_run_irl"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <close_gate>
    <real_app_irl>Maintainer (Promoted), resizable Tester: re-run steps 4–5 — the in-scope task writes a real C:/…/out/result.txt from real MCP data; the out-of-scope write is denied (no file); both observable in-app. THE authoritative close (rule 11 / ADR-0021); licenses the flip.</real_app_irl>
    <mutation_gate blocking="true">BLOCKING on the MCP-tool-injection branch (execution-wiring — decides whether the tool runs). Advisory on the UI commits.</mutation_gate>
    <design_review>The Tester resize + settings disclosure cite + match the DESIGN.md progressive-disclosure / sizing principles.</design_review>
    <cumulative_regression>the stub-MCP assembled regression + the two UI tests join the suites; the execution-status row + reconcile recorded.</cumulative_regression>
  </close_gate>
  <approval_surface>
    <item>cross-machine state + the red→impl diff over test paths (EMPTY), three distinct impl commits</item>
    <item>the MCP-tool-injection root cause confirmed (file:line) + the blocking mutation result on that diff</item>
    <item>the assembled stub-MCP result (the tool present in the provider's list; the file written) + Rust/frontend gates + both e2e legs</item>
    <item>the DESIGN.md design-review (Tester resize + settings disclosure) + the execution-status MCP-dispatch reconcile</item>
    <item>explicit: "M09.D.fix is ready. I will not commit until you approve; the flip waits on your re-IRL."</item>
  </approval_surface>
  <scope_locks>
    <lock>The red is run-path tool-list injection only — resolve authored MCP tool names to their connected-server definitions; no new transport, no dispatch change, no enforcer/tier change, no schema change.</lock>
    <lock>The UI pass is DESIGN.md-grounded, separate commits behind the red; do NOT fold UI into the wiring commit.</lock>
    <lock>The verdict-shows-PASS-on-gap-suspend truthfulness gap is OUT of scope here (tech-debt / M10) — note it; do not change verdict logic.</lock>
    <lock>No execution-status flip on green alone — the maintainer re-IRL is the authoritative close (rule 11).</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>That the authoring was correct and the gap was model-facing tool-list injection (dispatch wired, definition not injected); that no test hit it (tool-free e2e / store-driven); the execution-status MCP-dispatch reconcile (rule 11); the maintainer re-IRL result; that the red stayed its own mutation-blocked commit beside the UI.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Stage M09.D.fix"/>
</work_stage_prompt>
```

### D.fix.8 Commit messages
```
test(M09.D.fix): failing assembled stub-MCP + UI-pass tests (red)

fix(M09.D.fix): surface the canvas-authored MCP tool to the model

The M09.D IRL ran a correctly-authored agent (fs__read_text_file in
allowed_tools, validates, session_root set) in the Tester and the model saw
ONLY built-in Write — it asked for a read capability and wrote nothing, yet
the verdict read PASS. Dispatch was wired (M09.C) but the MCP tool's
definition was never injected into the model's tool list, so the model never
emitted the call. Resolve each authored MCP allowed_tools entry (server__tool)
to its connected-server list_tools definition and inject it into the run-path
tool list, named to match try_mcp_dispatch. Assembled stub-MCP regression
proves the tool reaches the provider's tool list and the end-to-end
dispatch->Write lands the file. Mutation-blocking (execution-wiring).

feat(M09.D.fix): Tester expand/resize per DESIGN.md disclosure

feat(M09.D.fix): settings progressive disclosure per DESIGN.md
```

---

## Stage M09.D.fix — Iteration 2 (wire the MCP dispatcher's enforcer; DESIGN.md UI v2)

> **The iteration-1 re-IRL closed the original 🔴 but exposed a deeper, misdiagnosed root cause — corrected here after a Hard-Rule-8 escalation (maintainer-approved).** Maintainer, real app: the run showed `fs__read_text_file` **injected, called by the model, the server connected, `try_mcp_dispatch` reached** — conditions 1+2 proven end-to-end. But it **FAILed at Novice** (`Exec forbidden in tier Novice`) although the maintainer is **Promoted** (Settings + the `request_tier_transition → target=Promoted` log). The original D.fix.9 diagnosis ("the new seam drops the tracked tier") was **wrong** — the tier threads correctly to the run-session enforcer. The real denial is the **MCP dispatcher's own, separate enforcer**: `build_test_mcp_dispatcher` (`commands.rs:1709`) and production `build_mcp_dispatcher` (`:282`) construct a **bare `CapabilityEnforcer::new()`** — default-Novice, no grants — never framework-/tier-wired (the docstring at `commands.rs:255-266` says exactly this: "the L1 CapabilityEnforcer is the empty default … [the loop] builds the framework-/tier-wired enforcer," CODEOWNERS / Hard-Rule-8). A known stub, harmless while only the no-tools smoke ran; M09 is the first real MCP tool dispatched through it. So `try_mcp_dispatch`'s `check()` (`dispatch.rs:192`) denies on **L4** (Novice, any user tier) and, even tier-fixed, on **L1** (`mcp_tool_capability` is Exec/**Irreversible** `dispatch.rs:74/87`; the framework grant is Exec/**Pure** `capability_map.rs:148`; `subsumes` needs exact `side_effect_class` equality). **Maintainer-approved fix (Hard-Rule-8):** a `build_session_mcp_enforcer(framework, tier)` helper — `set_tier(tier)` + `grant(agent_id, mcp_tool_capability(server, tool))` per authored MCP tool per agent — wired into **both** dispatchers, replacing the bare `new()`. Corrects the execution-status "MCP dispatch executes" claim (never true end-to-end). + the DESIGN.md UI v2. **ADR-0008 iteration 2 — the last;** the misdiagnosis was caught pre-impl (the cap isn't burned).

### D.fix2.1 Problem statement
The iteration-1 re-IRL surfaced a denial the phase doc misdiagnosed:
1. **The MCP dispatcher's enforcer is never wired (🔴).** `try_mcp_dispatch` (`agent_sdk.rs:884`) checks the **`McpDispatcher`'s own** `CapabilityEnforcer` (`dispatch.rs:192`), not the run-session enforcer. Both `build_test_mcp_dispatcher` (`commands.rs:1709`) and production `build_mcp_dispatcher` (`commands.rs:282`) build it as a bare `CapabilityEnforcer::new()` — default-Novice (`enforcer.rs:63/84`), no grants — a docstring-acknowledged stub (`commands.rs:255-266`; CODEOWNERS / Hard-Rule-8). `check()` runs **L4 first** (`enforcer.rs:220`): default-Novice → `Exec forbidden in tier Novice` at *any* user tier — the maintainer's exact error. Even tier-fixed, **L1** denies: `mcp_tool_capability` needs Exec/**Irreversible** (`dispatch.rs:74/87`) while `grant_framework_capabilities` produces Exec/**Pure** (`capability_map.rs:148`), and `subsumes` requires exact `side_effect_class` equality. The tier *already* threads to the run-session enforcer — the original "seam drops the tier" diagnosis was wrong (escalated, re-scoped). This gap is in **both** the Tester and production dispatcher construction (production is latent in v0.1 — only the no-tools smoke runs there — but real).
2. **UI v2 (DESIGN.md).** The Tester `Expand` grows the modal/canvas whitespace not the watch frame; the results section has no progressive disclosure; settings disclosure covers only budget.

### D.fix2.2 Files to change
**`<wire_signature_audit>` — pin at red:**
- `build_test_mcp_dispatcher` (`commands.rs:1699`, bare `CapabilityEnforcer::new()` @`:1709`) + production `build_mcp_dispatcher` (`:271`, @`:282`) — both replace the bare enforcer with `build_session_mcp_enforcer(framework, tier)`; thread `framework` + `tier` to each call site (`test_framework` holds the tracked tier; the production caller is `run_smoke_session`).
- The new `build_session_mcp_enforcer(framework, tier) -> CapabilityEnforcer`.
- `CapabilityEnforcer::grant(&mut self, agent, CapabilityDeclaration)` (`enforcer.rs:102`) + `set_tier` — build the enforcer **mut**, grant + set_tier, then `Arc::new` (grant takes `&mut`).
- `mcp_tool_capability(server, tool) -> CapabilityDeclaration` (Exec/Irreversible, `dispatch.rs:74/87`) — the grant that L1-subsumes the dispatch requirement; the `server__tool` → (server, tool) split.
- The dispatcher `check()` (`dispatch.rs:192`; L4-first `enforcer.rs:220`); `capability_map.rs:148` (the framework grant's Pure class — the contrast that explains why the generic grant doesn't subsume).
- **UI:** `TesterModal.tsx` (Expand handler + watch-frame layout + results section); `SettingsPanel.tsx` (the sections); `DESIGN.md`.

Files: `build_session_mcp_enforcer` + both dispatcher call sites (Rust — the 🔴 commit); its unit test; the assembled regression update; `TesterModal.tsx`; `SettingsPanel.tsx`; their tests; `docs/execution-status.md` (the ledger correction + the slice row, flipped on the re-IRL).

### D.fix2.3 Detailed changes
1. **`build_session_mcp_enforcer(framework, tier)` (the 🔴).** Build a `CapabilityEnforcer`, `set_tier(tier)`, then for each agent, for each `allowed_tools` entry that is an MCP name (`server__tool`), `grant(agent.id, mcp_tool_capability(server, tool))`; return it (the caller `Arc`-wraps). Wire it into **both** `build_test_mcp_dispatcher` and `build_mcp_dispatcher`, replacing the bare `new()`. Granting `mcp_tool_capability` (the exact Exec/Irreversible declaration the dispatch requires) resolves the L1 subsume; `set_tier(Promoted)` resolves L4. No new transport, no dispatch-logic change, no schema change. Mutation **BLOCKING** (capability-enforcement construction).
2. **Authored-only boundary (mutant-killer).** Grant only each agent's *own* `allowed_tools` MCP entries (per-agent `grant`) — an unauthored / other-agent tool stays denied. Pinned by a unit test.
3. **UI v2 (DESIGN.md, separate commits).** `TesterModal` Expand grows the watch frame (canvas + OUTPUT + run-trace); the results section (PASS/FAIL card + trace) gets progressive disclosure; `SettingsPanel` makes **every** section collapsible. Each cites its DESIGN.md principle; each its own render/e2e test.
4. **Ledger correction.** `docs/execution-status.md` "MCP dispatch executes" → qualify: dispatch was wired but the dispatcher enforcer was never framework-/tier-wired, so a real MCP tool is enforced-and-dispatched for the first time here; re-assert on the re-IRL.

### D.fix2.4 Tests
The `build_session_mcp_enforcer` unit is the core: at **Promoted** an authored MCP tool's `mcp_tool_capability` is granted + tier-allowed → `check()` passes; at **Novice** → L4-denied; an **unauthored** tool → not granted → L1-denied (the authored-only boundary). Plus the assembled regression extended to run through the **real dispatcher enforcer** (Promoted ⇒ the MCP tool runs + the file lands; Novice ⇒ denied). Plus the three UI tests. No weakening of any prior test.

### D.fix2.5 construction_reachability_check
`inputs_reachable="false"`: the dispatcher's enforcer is bare (default-Novice, no grants) → `try_mcp_dispatch`'s `check()` denies the authored MCP tool on L4 (Novice) and L1 (wrong grant class) at any user tier → the slice can't complete. Iteration 2 inverts it: `build_session_mcp_enforcer` sets the tier + grants `mcp_tool_capability` per authored tool, in both dispatchers. Verify by the unit (Promoted⇒allowed / Novice⇒denied / unauthored⇒denied), the assembled run, and the maintainer re-IRL.

### D.fix2.6 Close gate
Strict v1.11 two-commit: write all failing tests (the `build_session_mcp_enforcer` unit + the assembled-through-the-real-enforcer regression + the three UI tests), right-reason red, commit `test(M09.D.fix): …`, **surface red**; then implement as distinct commits untouching tests — `build_session_mcp_enforcer` + both dispatcher wirings [mutation **BLOCKING**] + the three UI fixes — red→impl test diff EMPTY (binary-crate variant via a scoped `#[cfg(test)]` diff if in-source). Full Rust (runtime-main/runtime-mcp ≥95) + frontend gates + both e2e legs. **The maintainer re-IRL is the authoritative close** — Promoted, in a Tester whose content frame grows: a real `out/result.txt` lands in-scope, denied out-of-scope, observable in-app. **Flip the execution-status row** + apply the ledger correction. **ADR-0008 iteration 2 — if the re-IRL still fails, escalate (do not iterate a third time).**

### D.fix2.7 CLI prompt
```xml
<work_stage_prompt id="M09.D.fix">
  <context>
    M09.D.fix ITERATION 2 (ADR-0008 last; maintainer-approved Hard-Rule-8 plan).
    Iteration-1 closed the original wiring red (the MCP tool injected + called +
    dispatch reached). The re-IRL then FAILed at Novice though the maintainer is
    Promoted — NOT because the seam drops the tier (it threads correctly). The real
    denial is the MCP dispatcher's OWN enforcer: build_test_mcp_dispatcher
    (commands.rs:1709) and production build_mcp_dispatcher (:282) build a bare
    CapabilityEnforcer::new() (default-Novice, no grants) — a docstring-flagged stub
    (commands.rs:255-266, "framework-/tier-wired by the loop that drives it";
    CODEOWNERS/Hard-Rule-8), never wired because no real MCP tool dispatched until
    M09. try_mcp_dispatch's check (dispatch.rs:192) denies on L4 (Novice, any user
    tier) and, even tier-fixed, on L1 (mcp_tool_capability is Exec/Irreversible
    dispatch.rs:74/87; the framework grant is Exec/Pure capability_map.rs:148;
    subsumes needs exact side_effect_class equality). FIX (the red): a
    build_session_mcp_enforcer(framework, tier) helper — set_tier(tier) +
    grant(agent_id, mcp_tool_capability(server, tool)) per authored MCP tool per
    agent (build mut, then Arc::new; grant is &mut self, enforcer.rs:102) — wired into
    BOTH dispatchers, replacing the bare new(). Grant ONLY each agent's own
    allowed_tools MCP entries (authored-only boundary; mutant-killer). Correct the
    execution-status "MCP dispatch executes" claim. BUNDLED UI v2 (DESIGN.md, separate
    commits): Expand grows the watch frame; Tester results disclosure; settings
    disclosure on EVERY section. Surface red first; no flip until the maintainer
    re-IRL. If the re-IRL still fails, ESCALATE — do not iterate a third time.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.D.fix — Iteration 2, D.fix2.1–D.fix2.6; and the iteration-1 section)</file>
    <file>src-tauri/src/commands.rs (build_test_mcp_dispatcher @1699 + the bare CapabilityEnforcer::new() @1709; build_mcp_dispatcher @271 + @282 + the docstring @255-266; the test_framework tracked-tier read; run_smoke_session as the build_mcp_dispatcher caller)</file>
    <file>crates/runtime-mcp/src/dispatch.rs (mcp_tool_capability @74 / Irreversible @87; the check() path @192) + crates/runtime-main/src/capability/enforcer.rs (grant @102; set_tier; L4-first @220; new() default-Novice @63/84) + crates/runtime-main/src/capability/capability_map.rs:148 (the framework grant's Pure class — contrast)</file>
    <file>crates/runtime-main/tests/capability_live_tool.rs (the assembled tier-path archetype) + crates/runtime-main/src/sdk/agent_sdk.rs (try_mcp_dispatch @884)</file>
    <file>DESIGN.md (progressive-disclosure / panel-sizing) + src/components/builder/TesterModal.tsx (Expand handler + watch-frame layout + results section) + src/components/SettingsPanel.tsx + docs/execution-status.md (the ledger row) + docs/adr/0021-* + docs/adr/0008-* + CLAUDE.md §4 rule 8 + rule 11 / §5/§6/§8</file>
  </read_first>
  <deliverable>A build_session_mcp_enforcer(framework, tier) helper (set_tier + per-authored-tool, per-agent mcp_tool_capability grants) wired into BOTH build_test_mcp_dispatcher and build_mcp_dispatcher, replacing the bare CapabilityEnforcer::new() — so a canvas-authored MCP tool passes the dispatcher's L4 (tier) + L1 (grant) checks at Promoted and is denied at Novice / when unauthored. Unit-tested (Promoted⇒allowed / Novice⇒denied / unauthored⇒denied); the assembled regression runs through the real dispatcher enforcer (Promoted ⇒ the tool runs + the file lands). The execution-status "MCP dispatch executes" claim corrected. Plus the DESIGN.md UI v2 (Expand watch-frame; Tester results disclosure; settings all-section disclosure). The enforcer fix is its own mutation-blocked commit. Closes M09.D on the maintainer re-IRL.</deliverable>
  <tdd_discipline strict="true">Write all failing tests first (the build_session_mcp_enforcer unit — Promoted⇒allowed/Novice⇒denied/unauthored⇒denied; the assembled-through-the-real-enforcer regression; the three UI render/e2e tests) → commit test(M09.D.fix): … (red) → SURFACE RED. Then implement as distinct commits untouching tests (build_session_mcp_enforcer + both dispatcher wirings [mutation BLOCKING]; Expand-frame + results disclosure; settings all-section disclosure); git diff &lt;red&gt;..&lt;impl&gt; over test paths EMPTY (binary-crate variant via a scoped #[cfg(test)] diff if the Rust test is in-source). Net-new/mechanical test fixups in separate labelled follow-ups.</tdd_discipline>
  <wire_signature_audit>PIN at red: build_test_mcp_dispatcher (commands.rs:1709 bare new()) + build_mcp_dispatcher (:282) + their call sites (thread framework + tier); build_session_mcp_enforcer (new); CapabilityEnforcer::grant (enforcer.rs:102, &mut) + set_tier; mcp_tool_capability (dispatch.rs:74/87, Exec/Irreversible); the dispatcher check() (dispatch.rs:192) + L4-first (enforcer.rs:220); the server__tool split.</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="D.fix2.5"/>
  <execution_steps>
    <step name="ground_at_red" budget="1"/>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="3"/>
    <step name="verify_gates" budget_iterations="3">Rust ≥95 (runtime-main/runtime-mcp) + frontend gates + BOTH e2e legs (test:e2e + test:e2e:tauri). No schema change.</step>
    <step name="mutation_gate" blocking="true"/>
    <step name="green_phase_commit" budget="3"/>
    <step name="assembled_run_irl"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <close_gate>
    <real_app_irl>Maintainer (Promoted), in a Tester whose content frame grows: re-run the in-scope task — a real C:/…/out/result.txt lands from real MCP data; the out-of-scope write is denied (no file); the run + results are readable in-app. THE authoritative close (rule 11 / ADR-0021); licenses the flip.</real_app_irl>
    <mutation_gate blocking="true">BLOCKING on build_session_mcp_enforcer + the dispatcher enforcer wiring (capability-enforcement construction — decides whether the MCP tool runs). Advisory on the UI commits.</mutation_gate>
    <design_review>Expand grows the watch frame (not the chrome); the Tester results + every settings section follow the DESIGN.md disclosure principle.</design_review>
    <cumulative_regression>the build_session_mcp_enforcer unit + the assembled-through-the-real-enforcer regression + the three UI tests join the suites; the execution-status row flipped + the ledger correction applied.</cumulative_regression>
  </close_gate>
  <approval_surface>
    <item>cross-machine state + the red→impl diff over test paths (EMPTY), distinct commits</item>
    <item>the dispatcher-enforcer root cause confirmed (bare new() in both; file:line) + the blocking mutation result + the unit (Promoted⇒allowed/Novice⇒denied/unauthored⇒denied)</item>
    <item>Rust + frontend gates + both e2e legs; the Expand-frame + disclosure design-review; the ledger correction</item>
    <item>explicit: "M09.D.fix iteration 2 is ready. I will not commit until you approve; the flip waits on your re-IRL. If the re-IRL fails I escalate, not iterate."</item>
  </approval_surface>
  <scope_locks>
    <lock>The red is the dispatcher-enforcer wiring only — build_session_mcp_enforcer (set_tier + per-authored-tool mcp_tool_capability grants) in both dispatchers; no dispatch-logic/transport/schema change; grant only each agent's OWN allowed_tools MCP entries.</lock>
    <lock>UI v2 is DESIGN.md-grounded, separate commits behind the red; do not fold UI into the enforcer commit.</lock>
    <lock>The verdict-on-gap-suspend truthfulness gap stays OUT of scope (tech-debt / M10).</lock>
    <lock>ADR-0008 iteration 2 — the last; if the re-IRL still fails, escalate, do not iterate a third time. No flip on green alone.</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>That the original D.fix.9 diagnosis (seam drops tier) was wrong (the tier threads); the real cause was the dispatcher's bare enforcer (both Tester + prod) — escalated per Hard Rule 8 before burning the iteration; the L4 + L1 layers; the build_session_mcp_enforcer fix + the authored-only boundary; the ledger correction; the re-IRL result; that this was iteration 2 (the ADR-0008 cap).</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Stage M09.D.fix — Iteration 2"/>
</work_stage_prompt>
```

### D.fix2.8 Commit messages
```
test(M09.D.fix): failing dispatcher-enforcer + UI-v2 tests (red)

fix(M09.D.fix): wire the MCP dispatcher's enforcer (tier + per-tool grants)

The iteration-1 re-IRL FAILed at Novice though the maintainer was Promoted.
Root cause (escalated per Hard Rule 8, the seam-drops-tier diagnosis was
wrong): try_mcp_dispatch checks the McpDispatcher's OWN enforcer, which
build_test_mcp_dispatcher / build_mcp_dispatcher built as a bare
CapabilityEnforcer::new() (default-Novice, no grants) — never framework-/tier-
wired. It denied L4 (Novice, any tier) and, even tier-fixed, L1
(mcp_tool_capability is Exec/Irreversible; the framework grant is Exec/Pure;
subsumes needs exact equality). Add build_session_mcp_enforcer(framework, tier)
(set_tier + per-authored-tool, per-agent mcp_tool_capability grants) and wire
it into both dispatchers, replacing the bare new(). Grant only each agent's own
allowed_tools MCP entries (authored-only boundary). Mutation-blocking.

feat(M09.D.fix): Tester Expand grows the watch frame + results disclosure

feat(M09.D.fix): settings progressive disclosure across all sections
```

---

## Stage M09.V — Five-pass real-app verifier

### V.1 Problem statement
Fresh-context five-pass verifier against the built surfaces (A blank-create, B file_access editor, C MCP-tool attach, D the assembled vertical slice). M09's whole point is that a from-scratch agent **builds and runs** — so the 5th pass **drives the real Tauri app** and observes the authored agent pull real MCP data and write a real file within the granted scope (denied outside it), not the unit tests. The bias guard: the verifier runs with empty session memory and a read-list that deliberately omits the M09 retrospectives / summary / gap-analysis (ADR-0008 / STAGE-PROMPT-PROTOCOL.md §14).

### V.2 Scope to verify
| Surface | Observable-in-app check (5th pass drives the real app) |
|---|---|
| A | a fresh project's Agents tab shows "+ New agent"; dragging it mints an agent (a second drag → `agent-2`); the node is selectable |
| B | a selected agent gains a File-access editor; adding a Write glob writes `capabilities.file_access.write`; the framework validates |
| C | with a server installed, the Tools tab surfaces its tools (`source:'mcp'`); dragging one + an Agent→Tool edge records `allowed_tools` + `capabilities.tools_called` |
| D | a Promoted run of the canvas-authored agent **writes a real file within the granted write glob from real MCP data**, and an out-of-scope write is **denied (no file)** — the on-disk effect, not an event |

### V.5 CLI prompt
```xml
<verifier_stage_prompt id="M09.V">
  <context>
    M09 Stage V — the fresh-context FIVE-pass verifier (ADR-0008 + the v1.9 5th
    assembled_execution pass). M09 is the first vertical slice (ADR-0032): author one
    agent (A) + grant file_access (B) + attach a real MCP tool (C) → run → write a
    real file at the enforced tier (D). Run with empty session memory: you have NOT
    seen the M09 retrospectives or summary. The clear-and-paste session is the bias
    guard. CENTRAL DUTY: DRIVE THE REAL TAURI APP (npm run test:e2e:tauri, ADR-0021)
    and OBSERVE the canvas-authored agent write a real file WITHIN the granted scope
    and be DENIED outside it — a "Sound" that did not drive the real app is FORBIDDEN
    (rule 11). The individual wires (MCP dispatcher + tier + capabilities-granting)
    are already present (commands.rs:1769-1775/1758; tester.rs:445) — your job is to
    verify the COMPOSED authored→serialized→run path, not re-confirm the wires.
    Findings: 🔴 block → D.fix (max 2 iter); 🟡 zero-propagation; 🟢 docs/tech-debt.md.
  </context>
  <read_first>
    <file>STAGE-PROMPT-PROTOCOL.md §14 + docs/cluster-pattern.md (§3 the 5th pass, §4 assert-the-side-effect) + docs/adr/0008-* + docs/adr/0021-* (tauri-driver) + docs/adr/0032-* (the vertical re-cut)</file>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Background + Stages A/B/C/D + V.2 — but NOT any *-retrospective.md it references)</file>
    <file>src/components/builder/Palette.tsx (the "+ New agent" item + nextAgentRef; source:'mcp' tools) + src/lib/builderStore.ts (builderAgent capabilities; nextAgentRef; updateNode) + src/components/builder/NodeConfigPanel.tsx (the file_access editor)</file>
    <file>src-tauri/src/commands.rs (mcp_list_server_tools; test_framework @1745 — dispatcher+tier already wired @1769-1775/1758) + crates/runtime-main/src/builder/tester.rs (grant_framework_capabilities + with_capability_wiring) + src/lib/ipc.ts</file>
    <file>tests/e2e-tauri/ (builder_create_agent + builder_file_access + builder_mcp_tool + vertical_slice) + docs/execution-status.md (the row M09.D flips) + docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md + VERIFIER-RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>
  <scope_to_verify ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="V.2 Scope to verify"/>
  <verification_passes>
    <pass name="inventory">A (the "+ New agent" item + nextAgentRef) + B (the file_access editor + builderAgent.capabilities) + C (mcp_list_server_tools + source:'mcp' palette items) + D (vertical_slice.e2e.ts) exist + match A.3/B.3/C.3/D.3. Missing→🔴; stub→🟡.</pass>
    <pass name="wire">The authoring→serialize→enforce trace: a canvas-authored agent's capabilities.file_access (NodeConfigPanel → updateNode → framework) reaches framework_doc → test_framework → grant_framework_capabilities → the L2 enforcer; the MCP tool name in allowed_tools resolves through try_mcp_dispatch. Break at any step → 🔴.</pass>
    <pass name="behavior">Assertions on the observable side effect (§4): a Promoted run of the authored agent WRITES a real file within the granted write glob; an out-of-scope write is DENIED (no file on disk); a second "New agent" drag mints agent-2. Event/structure-only → 🔴.</pass>
    <pass name="multi_call_invariants">The authored file_access round-trips through updateNode + the JSON view (ADR-0020 — canvas and JSON re-derive); re-creating after a delete never collides on id (the ${kind}:${ref} guard); attaching a second MCP tool appends, not replaces.</pass>
    <pass name="assembled_execution">THE 5th PASS — DRIVE THE REAL TAURI APP and OBSERVE per V.2: author from scratch → grant file_access.write → attach an installed MCP server's tool → Run → a real file lands within scope from real MCP data, denied outside. A surface you did NOT drive → HYPOTHESIS, labeled (rule 11), pass INCOMPLETE. Report N/M (no key/no server → state it; never 0/0-as-pass).</pass>
  </verification_passes>
  <design_review>The authoring surfaces (the "+ New" affordance, the file_access editor, the source:'mcp' palette items) + the end-to-end run (Tester / Output rail / nodes) conform to DESIGN.md (palette item style; state-visible; progressive disclosure). Divergences → findings.</design_review>
  <findings_format ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md" section="Findings"/>
  <merge_gate red_blocks="true" dfix_iteration_cap="2" waiver_path="docs/adr/NNNN-waiver-M09-finding-N.md"/>
  <gates milestone="M08"/>
  <approval_surface>
    <item>the five-pass findings table (🔴/🟡/🟢 + file:line + the rule each tests)</item>
    <item>the 5th-pass result per surface — RAN-real-app vs read; observed; N/M (no key/server → stated)</item>
    <item>which verdicts are grounded-by-real-app-execution vs hypothesis (rule 11)</item>
    <item>the design-review (the authoring surfaces + the run)</item>
    <item>the merge-gate disposition + the verifier retrospective</item>
    <item>explicit: "M09.V is complete. Findings surfaced for routing; I will not commit until you approve."</item>
  </approval_surface>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md">
    <special_log>Per surface (A, B, C, D), whether you DROVE THE REAL TAURI APP (the 5th pass) or only read its test + the observed result; N/M for the e2e:tauri run (or "not run — no key/server"); which verdicts are grounded-by-real-app vs hypothesis (rule 11); the design-review divergences; that the canvas-authored agent's write is observably scope-gated (lands in-scope, denied out-of-scope) in the running app.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Closeout — V commits its retrospective per ADR-0008"/>
</verifier_stage_prompt>
```

---

## Closeout

### Closeout CLI prompt (M09.G)
```xml
<closeout_stage_prompt id="M09.G">
  <context>
    M09 Stage G — the phase closeout. FINAL stage of M09 (the first ADR-0032 vertical
    slice), runs after Stage V (+ any D.fix). Built surfaces: A (blank-create an
    agent) + B (file_access editor) + C (attach a real MCP tool) + D (the assembled
    vertical-slice IRL — the execution-status row flipped). Produces: M09-summary.md,
    the immutable gap-analysis entry, simplify_pass, coverage_policy_reconciliation,
    and the PR draft. No ADR flips (M09 files none — no schema change, no new
    primitive; declaration-only authoring + a read-only command; ADR-0032 was already
    accepted in the re-cut PR). Per CLAUDE.md §19/§20. Next milestone: M10 (HITL steers
    — gap resolve→resume + plan-approval + plan task execution).
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Background + Stages A/B/C/D + this closeout)</file>
    <file>CLAUDE.md (§5/§6 coverage; §19 summary; §20 the Gap Analysis Protocol)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (§8 closeout-only tags; simplify_pass + coverage_policy_reconciliation children)</file>
    <file>docs/build-prompts/retrospectives/{RETROSPECTIVE-TEMPLATE.md, SUMMARY-TEMPLATE.md, VERIFIER-RETROSPECTIVE-TEMPLATE.md} + docs/gap-analysis.md (the six-section template)</file>
    <file>docs/adr/0032-* (the vertical re-cut M09 delivers) + docs/adr/0008-* (the 🟢 tech-debt ledger) + docs/execution-status.md (the row M09.D flipped)</file>
  </read_first>
  <cumulative_reads>
    <codebase>the shipped codebase through M09 (cumulative across M01–M08.9 + the M09 stages A, B, C, D)</codebase>
    <spec>agent-runtime-spec.md (focus §9 the Canvas/Tester + §8.security L2 file_access + L4 tier; §5 MCP) + docs/adr/0032-* (the §0d re-amendment)</spec>
    <gap_analysis>docs/gap-analysis.md (ALL prior entries — M01 … M08.9)</gap_analysis>
    <retrospectives>docs/build-prompts/retrospectives/M09.*-retrospective.md (A, B, C, D, V)</retrospectives>
    <summary>docs/build-prompts/retrospectives/M09-summary.md (authored as part of this stage)</summary>
    <tech_debt>docs/tech-debt.md (any V 🟢 added)</tech_debt>
    <coverage_policy>docs/coverage-policy.md (§A/§B/§C)</coverage_policy>
  </cumulative_reads>
  <scope_locks ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Scope">
    <lock>Built surfaces only (A blank-create + B file_access + C MCP-tool attach + D the assembled run). No new execution primitive, no schema change; declaration-only capability authoring.</lock>
    <lock>Do NOT open the PR — draft the PR description; the orchestrator opens it + merges.</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md">
    <special_log>M09 made the workbench BUILD AND RUN a real single-agent, MCP-data workflow from scratch (the first ADR-0032 vertical slice). Record: that the substrate already executed (the gap was authoring + one read-only command); that the dispatcher/tier/capabilities-granting were already wired (commands.rs:1769-1775/1758; tester.rs:445) so M09 added no execution-wiring; declaration-only authoring; the execution-status flip on the real-app IRL; the M10 hand-off (HITL steers — gap resolve→resume + plans).</special_log>
  </retrospective_requirements>
  <deliverables>
    <milestone_summary>docs/build-prompts/retrospectives/M09-summary.md (per SUMMARY-TEMPLATE.md; aggregate A/B/C/D + V; 3-axis scoring; time-box; verdict; tick the coverage-policy reconciliation).</milestone_summary>
    <gap_analysis_entry>docs/gap-analysis.md (append the immutable M09 entry; the six required sections). Carry-forward: M09 delivers the ADR-0032 walking skeleton (the first canvas-authored real run); the execution-status row flipped (cite vertical_slice.e2e.ts + the IRL date); record that the dispatcher/tier wiring was confirmed-present (commands.rs:1769-1775/1758), not added.</gap_analysis_entry>
    <coverage_policy_reconciliation>mcp_list_server_tools lands in src-tauri (the tauri-shell patch gate) + a runtime-mcp/`src-tauri` seam; the renderer additions in the vitest ≥80 gate. Confirm no exclusion change; if the new command is a thin OS/network wrapper paired with a `*_with` seam, re-confirm the seam-vs-wrapper split; sync the four mirrors; append a §C M09 entry.</coverage_policy_reconciliation>
    <simplify_pass>
      <invoke skill="simplify" against="the milestone cumulative diff (M09.A..HEAD)"/>
      <surface kind="refactor_proposals" examples="nextAgentRef / the file_access editor's immutable capabilities recompute / the palette MCP-tool fetch-on-mount / mcp_list_server_tools"/>
      <approval_required>true</approval_required>
      <commit_on_approval>a focused refactor commit on the same branch before the PR opens</commit_on_approval>
      <defer_unapproved_to>docs/tech-debt.md (per the ADR-0008 🟢 ledger)</defer_unapproved_to>
    </simplify_pass>
    <pr_description>draft only; do not open the PR until explicitly asked</pr_description>
  </deliverables>
  <gap_analysis_requirements ref="CLAUDE.md" section="20. Gap Analysis Protocol">
    <gotchas_graduation>
      <stage_review id="A"/>
      <stage_review id="B"/>
      <stage_review id="C"/>
      <stage_review id="D"/>
    </gotchas_graduation>
    <special_check>V→closeout handoff: 🔴 resolved by a D.fix before this commit; 🟡 fixed-in-cluster or recorded; 🟢 in docs/tech-debt.md. Record the execution-status flip (cite vertical_slice.e2e.ts + the real-app IRL date).</special_check>
    <special_check>Confirm schemas/* unchanged (declaration-only authoring; mcp_list_server_tools is read-only). No ADR filed/flipped by M09 (ADR-0032 was accepted in the re-cut PR; M09 implements it).</special_check>
  </gap_analysis_requirements>
  <append_only_verification>
    <local_check>the prior content of docs/gap-analysis.md must be a literal prefix of HEAD before commit</local_check>
    <ci_check name="gap-analysis-append-only">fails if any prior line is modified</ci_check>
  </append_only_verification>
  <three_artifact_review>
    <artifact>the code diff (M09 A + B + C + D + the V-finding fixes + any simplify refactor)</artifact>
    <artifact>the per-stage retrospectives (A, B, C, D) + the Stage V retro + the M09 summary</artifact>
    <artifact>the new gap-analysis M09 entry — flagged "IMMUTABLE once committed"</artifact>
    <pushback_blocks_pr>true</pushback_blocks_pr>
  </three_artifact_review>
  <self_correction_budget>3</self_correction_budget>
  <approval_surface>
    <item>the three-artifact bundle (summary + gap-analysis + the cumulative diff)</item>
    <item>the execution-status row flip (the first canvas-authored real run, real-app-observed) + the assembled vertical slice</item>
    <item>the simplify_pass proposal + the coverage_policy_reconciliation result</item>
    <item>explicit: "M09.G closeout is ready. I will not commit until you approve; I will NOT open the PR."</item>
  </approval_surface>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Closeout"/>
</closeout_stage_prompt>
```

---

## Verification checklist (before the M09 PR pushes)

- [ ] Every UI stage closed on the **real Tauri app via `tauri-driver`** + the maintainer IRL — not Playwright-mock-green, not code-read (rule 11 / ADR-0021).
- [ ] **M09.D** ran the **assembled Tester path** (real provider + real MCP dispatch + real enforcer) and asserted the **on-disk file** within scope + the out-of-scope denial — the assembled-app-regression mandate, not the isolated pieces.
- [ ] The composed **authored→serialized→run** hypothesis was explicitly **confirmed, or any serialization glue closed** (M09.D); the dispatcher/tier/capabilities-granting were confirmed already-wired (`commands.rs:1769-1775`/`:1758`; `tester.rs:445`), so M09 added no execution-wiring.
- [ ] The execution-status ledger gained the **canvas-authored single-agent + MCP + file_access** row, citing `vertical_slice.e2e.ts` + the IRL date.
- [ ] Strict **v1.11** two-commit TDD held on every code stage (validator-enforced `<execution_steps>`: `red_phase_commit → surface_for_red_approval → green_phase_commit → surface_for_final_approval`; red→impl diff over test paths EMPTY).
- [ ] **Stage M09.V** (the five-pass real-app verifier) ran fresh-context and **drove the real Tauri app** (5th pass); its findings were routed (🔴→D.fix / 🟡→zero-propagation / 🟢→`docs/tech-debt.md`).
- [ ] **Stage M09.G** (closeout) produced the M09 summary + the immutable gap-analysis entry + the simplify_pass + the coverage-policy reconciliation; the PR is drafted, not opened.
- [ ] Scope locks held — M09 shipped **pure** (suspends-cleanly / E-04 only — resume is M10); no new execution primitive (plans M10 / sub-agents M11 / hooks+shell M12), no MCP-as-node/data-source catalog (M13), no schema change; capability authoring stayed declaration-only.
- [ ] The closeout states the next milestone: **M10** (HITL steers the run — gap resolve→resume + plan-approval + plan task execution, per ADR-0032).

## Summary table

| Stage | Goal | Close bar |
|---|---|---|
| M09.A | Blank-create an agent (the "New agent" affordance) | real-app IRL: a fresh project authors its first agent on the canvas |
| M09.B | file_access editor (the agent's `capabilities.file_access`) | real-app IRL: a write glob is authorable; the framework validates |
| M09.C | Attach a real MCP server's tool (enumerate + palette) | real-app IRL: an installed server's tools are draggable + recorded in allowed_tools; the Tester dispatcher already wired (`commands.rs:1769-1775`) |
| M09.D | The vertical-slice IRL (assembled, end-to-end) | **maintainer real-app IRL: author → scope → attach MCP → Run → real file written within scope, denied outside, observable**; execution-status flipped |
| M09.V | Five-pass real-app verifier | the canvas-authored agent writes a real file within scope + is denied outside, observed in the running app (5th pass); findings routed |
| M09.G | Closeout | summary + gap-analysis (the execution-status flip) + simplify + coverage; M10 hand-off |

**Ending:** the workbench **builds and runs a real single-agent, MCP-data workflow from scratch** — author an agent on an empty canvas, scope it, attach a real data tool, Run, watch it write a real file at the enforced tier. The first turn of paint into a real, authored, executing product, and the first ADR-0032 vertical slice. **M10** lets a human steer the run (gap resolve→resume + plans); **M11** adds sub-agents; **M12** the verify-loop + controlled shell-exec; **M13** industrializes + ships (the data-source catalog + save/import-export).
