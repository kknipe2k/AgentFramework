# M09 — Workbench Vertical Slice: author one real agent, run it on real data

> **Protocol version:** v1.8 (per `STAGE-PROMPT-PROTOCOL.md` — `<tdd_discipline strict="true">` two-commit on every code stage; v1.8 `<construction_reachability_check>` + `<wire_signature_audit>`; the **assembled-app-regression mandate** is the close bar for the integration stage).
>
> **The milestone where the workbench stops *composing* and starts *authoring + running*.** Today the canvas can only rearrange agents that already exist in JSON, and a built workflow runs read-only at Novice. M09 makes the smallest **complete, real** loop work end-to-end *in the app*: drag a **new** agent onto an empty canvas, grant it a **file_access** scope, attach a **real MCP server's tool**, hit Run, and watch it **pull real data and write a real file** — at the tracked tier, enforced. It is the first milestone of the `docs/workbench-delivery-plan.md` re-cut (**M-α, the vertical slice**), and it is deliberately small because the substrate it rides — single-agent streaming, built-in Read/Write, MCP dispatch, capability + tier enforcement — **already executes** (`docs/execution-status.md`).

---

## Background — the grounded state at entry

**What runs (rule 11, IRL-confirmed — `docs/execution-status.md`).** A single agent that streams multi-turn, runs built-in **Read/Write**, loads a skill, suspends on a gap, tracks budget, dispatches an **MCP tool**, and is gated by **capability (L1/L2) + tier (L4)** — all execute in the assembled app today. MCP dispatch is wired into the run loop (`crates/runtime-main/src/sdk/agent_sdk.rs:884` `try_mcp_dispatch` → `crates/runtime-mcp/src/dispatch.rs:91` `McpDispatcher`, capability-enforced). The L2 file-scope enforcement is proven by the E-02 eval (`crates/runtime-main/tests/capability_live_tool.rs`: an in-scope Write lands; an out-of-scope Write is blocked with no file on disk).

**What does NOT exist (grounded, file:line) — the authoring gap this milestone closes:**

- **You cannot create an agent from scratch.** The Palette's Agents tab lists only installed artifacts + a loaded framework's agents (`src/components/builder/Palette.tsx:173-184`); there is no built-in/blank agent. A fresh project opens with `emptyFramework()` (`src/lib/builderStore.ts:124`, `agents: []`), so the Agents tab is **empty** and nothing can be authored on the canvas. The store *already supports* minting one — `addNode` (`builderStore.ts:506`) → `applyDrop` (`:183`) → `builderAgent` (`:145`), reached by the canvas drop handler `BuilderCanvas.onDrop` (`:86-96`) — the **Palette simply never offers a blank item**. This is a UI gap, not an architecture gap.
- **An authored agent cannot be made *capable*.** `builderAgent` (`builderStore.ts:145-154`) constructs `{ id, role, model, allowed_tools, allowed_skills, spawns }` and **omits `capabilities`** — which `agent.v1.json:9` lists as **required** (`common.v1.json#/$defs/Capabilities`: `{tools_called, skills_loaded, file_access:{read,write}, network, shell, spawn_agents}`). So a canvas agent is *schema-invalid* and, more importantly, carries **no `file_access`** — the read/write glob scope the L2 enforcer checks. `NodeConfigPanel` (`src/components/builder/NodeConfigPanel.tsx:105-146`) edits only `role`/`model`/`allowed_tools`/`allowed_skills` — there is no capability surface. **Without `file_access.write`, the agent's Write is denied — nothing lands.**
- **A real MCP server's tools cannot be reached on the canvas.** `mcp_list_servers` (`src-tauri/src/commands.rs:1083`) lists installed servers and `mcp_test_connection` (`:1049`) enumerates a server's tools via `McpClient::test_connection`→`list_tools` (`crates/runtime-mcp/src/client/mod.rs:246-259`), but **there is no command to list an *already-installed* server's tools** for the palette, and the Tools tab (`Palette.tsx:143-160`) offers only built-ins (`Read`/`Write`/`Bash` — and `Bash` doesn't even execute) + installed artifacts + framework tools. So the data-bearing tools an agent needs are invisible to the author.

**The load-bearing fact.** Every runtime piece the vertical slice needs **already executes** — the gap is entirely in *authoring* (three small surfaces) plus one small backend command (list an installed server's tools). That is why M09 is four tight stages, not a quarter.

**This milestone = M-α of `docs/workbench-delivery-plan.md`.** It supersedes the M08.8 stage-trim (D budget-visible / E gap-resume / F save-polish) as the next thing built; those fold into M11–M13 of the re-cut. M08.8 A/B/B.fix/C shipped; the **M08.8.C.fix** tier-display fix (seed `currentTier` on mount) is a precondition for M09.D's IRL (the maintainer must be able to read+set Promoted), and should land with M08.8's close.

---

## Scope

**In scope (the vertical slice, end-to-end, in the real app):**
- **M09.A** — blank-create an **agent** on the canvas (the Palette "New agent" affordance + a fresh-id helper; the store path already exists).
- **M09.B** — a **file_access editor** in `NodeConfigPanel` (the agent's `capabilities.file_access.{read,write}` glob lists) + `builderAgent` initializing a valid `capabilities`.
- **M09.C** — a new `mcp_list_server_tools(name)` command + surfacing an installed MCP server's tools in the Palette Tools tab + attaching one to the agent (`allowed_tools` + `capabilities.tools_called`).
- **M09.D** — the **assembled real-app IRL**: build the agent fresh, scope it, attach the MCP tool, Run via the Tester, observe real data pulled + a real file written; flip the execution-status "observed in app" row for *canvas-authored single-agent + MCP + file_access*.

**Out of scope (later milestones of the re-cut):**
- The rest of the palette vocabulary — Plan / MCP-server-node / rails / budget — and config for every node kind, node delete, id-rename, the `Bash`-advertised-but-unbuilt integrity fix → **M10** (author-anything).
- MCP servers as first-class canvas citizens + a data-source catalog (GitHub/Postgres/Slack/Drive/Notion) + credentials UX → **M11**.
- Sub-agents / plans / hooks *executing* → **M12**. Validated whole-workflow import/export + save-path → **M13**.

**Locks:** no schema change (the `Capabilities`/`Agent`/`McpServerConfig` shapes are the source of truth — author *to* them); **declaration-only capability authoring** — α.B writes the agent's grant, it never widens the enforcer or the user's tier; the new α.C command **reuses** the existing `test_connection`/`list_tools` path (no new MCP machinery); real-app `tauri-driver` IRL is the close bar (ADR-0021); strict v1.8 two-commit TDD on every code stage.

---

## Staging (each stage builds the next; M09.D is the assembled close)

| Stage | Deliverable (forensic) | Key real seams |
|---|---|---|
| **M09.A** | **Blank-create an agent.** A "New agent" Palette affordance (Agents tab) carrying `{kind:'agent', ref:<fresh-id>}`; the existing `BuilderCanvas.onDrop:96 → addNode` mints `builderAgent`. A `nextAgentRef(framework)` helper generates an id matching `^[a-z][a-z0-9-]*$`. | `Palette.tsx:173-184` (empty agents tab) · `builderStore.ts:506/183/145` · `BuilderCanvas.tsx:86-96` |
| **M09.B** | **file_access editor.** `builderAgent` initializes a valid `capabilities`; `NodeConfigPanel` gains a read/write glob-list editor over `agent.capabilities.file_access`; edits flow through `updateNode`. | `builderStore.ts:145` · `NodeConfigPanel.tsx:105-146` · `common.v1.json#/$defs/Capabilities` · enforced by E-02 |
| **M09.C** | **Attach a real MCP tool.** New `mcp_list_server_tools(name)` (registry lookup + `list_tools`, reusing `test_connection`'s path); the Palette Tools tab surfaces an installed server's tools (`source:'mcp'`); attaching adds to `allowed_tools` + `capabilities.tools_called`. **Verify `test_framework` wires `build_mcp_dispatcher`** (else the Tester won't dispatch the MCP tool — α.C/D precondition). | new command ∼ `commands.rs:1049/1083` · `client/mod.rs:246` · `Palette.tsx:143-160` · `agent_sdk.rs:884` dispatch |
| **M09.D** | **The vertical-slice IRL.** Assembled `tauri-driver` e2e: a fresh canvas-authored agent + `file_access.write` + an installed MCP tool runs in the Tester and writes a real file from real MCP data; the maintainer IRL-watches it on the live app. Flip execution-status. | `tests/e2e-tauri/` · `test_framework` (`commands.rs`) · `docs/execution-status.md` |

**The falsifiable hypothesis M09.D must disprove (v1.8 assembled mandate):** "the Tester run path (`test_framework`) wires the MCP dispatcher + the authored agent's `capabilities` into the enforcer exactly as `run_smoke_session` does." If the assembled test fails, the gap is *integration wiring in `test_framework`*, not the authoring UI — and that wiring is part of M09.C/D, not a separate milestone.

---

## Process (per stage)

1. **Authored at entry** against the live renderer + the real schemas (§8 phase-doc pre-flight — cross-machine state checked; the build `git pull`s the milestone branch before dropping any stage prompt).
2. **Cluster-gate close** (`docs/cluster-pattern.md`): acceptance-first (BDD) → strict v1.8 two-commit TDD → machine gates + `test:e2e:tauri` → **the real-app IRL** → triage-in-place (zero-propagation).
3. **Mutation gate** — advisory on the renderer stages (A/B); **on M09.C** the new Rust command is non-critical (a read-only `list_tools` wrapper) → advisory, BUT any touch to `test_framework`'s enforcer/dispatcher wiring is execution-wiring → **blocking** on that diff (`cluster-pattern.md` §5).
4. **The close flips execution-status's "observed in app" status** for the surface — on the maintainer IRL watch, never tests-green (rule 11). Only **M09.D** flips the row (A–C build toward it).

---

## Stage M09.A — Blank-create an agent on the canvas

### A.1 Problem statement
A fresh project cannot author anything. `emptyFramework()` (`builderStore.ts:124-135`) opens with `agents: []`; the Palette's Agents tab is built from `installed` + the loaded `framework` only (`Palette.tsx:173-184`) — no built-in/blank agent — so the tab renders its empty state (`Palette.tsx:256-259` "No agents.") and there is nothing to drag. The store already mints an agent on a drop: `BuilderCanvas.onDrop` (`:86-96`) parses the `application/x-builder-node` payload `{kind, ref}` and calls `addNode(kind, ref, position)` (`builderStore.ts:506`), which applies `applyDrop` (`:183` → `builderAgent(ref)` `:145`) and records the position. **The only missing piece is a Palette item that offers a *fresh* agent.**

### A.2 Files to change
**`<wire_signature_audit>` — pin before pseudocode:** `BuilderNodeKind = 'agent'|'tool'|'skill'|'hitl'|'hook'` (`builderStore.ts:24`); `addNode(kind, ref, position)` (`:506`, idempotent on `${kind}:${ref}` in `nodePositions` `:509`); `builderAgent(id)` (`:145`); the drag payload contract `{kind, ref}` on `application/x-builder-node` (`Palette.tsx:274-277` / `BuilderCanvas.tsx:88-96`); the agent id pattern `^[a-z][a-z0-9-]*$` (`agent.v1.json:14`).
- `src/components/builder/Palette.tsx` — the Agents tab gains a leading **"+ New agent"** item whose `ref` is a fresh unique id (`nextAgentRef(framework)`); it is a normal draggable carrying `{kind:'agent', ref}`. (Tools/Skills get the same affordance in **M10** — α scopes to the agent, the authoring blocker.)
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
3. **No drop-handler / store-core change.** `onDrop:96 → addNode → applyDrop:185 (agent → builderAgent)` already does the work; α.A only *offers* the item. The new agent appears, is selectable (`BuilderCanvas onNodeClick → selectNode`, `:111`), and `NodeConfigPanel` already renders for it (`findAgent`, `NodeConfigPanel.tsx:12`). Continuous validation (`scheduleValidation`, `builderStore.ts:454`) surfaces the still-missing `role`/`capabilities`/`session_root_agent`.

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
Strict v1.8 two-commit TDD (red: the `builder_create_agent.e2e.ts` + `nextAgentRef` unit fail right-reason — no New-agent item, empty tab; impl untouching tests, diff over test paths EMPTY). Frontend gates + `test:e2e:tauri`. Mutation **advisory** (renderer). Stage-D design review (the "New" affordance matches `DESIGN.md`). **No execution-status flip** (α.A authors but runs nothing — the flip is M09.D).

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
    Zero-propagation; tools/skills "New" affordance is M10.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.A — A.1-A.6; Background)</file>
    <file>docs/workbench-delivery-plan.md (M-α the vertical slice; this is its first stage)</file>
    <file>src/components/builder/Palette.tsx (paletteItemsForTab @137; the agents case @173-184; the onDragStart contract @269-279; the empty state @256)</file>
    <file>src/lib/builderStore.ts (addNode @506 + its idempotence guard @509; applyDrop @183; builderAgent @145; emptyFramework @124) + src/components/builder/BuilderCanvas.tsx (onDrop @86-96 → addNode)</file>
    <file>schemas/agent.v1.json (the id pattern @14; required fields @9) + docs/adr/0020-* (document-as-source-of-truth) + docs/adr/0021-* + CLAUDE.md §5/§6/§8</file>
  </read_first>
  <deliverable>A "+ New agent" Palette item (Agents tab) that drags a fresh-id agent onto the canvas via the existing addNode path, plus an exported pure nextAgentRef(framework) helper. A fresh project can author its first agent; repeated creates yield distinct ids. No drop-handler or store-core change; capabilities are M09.B.</deliverable>
  <tdd_discipline strict="true">Two commits: red (tests/e2e-tauri/builder_create_agent.e2e.ts + a nextAgentRef vitest unit + a Palette "+ New agent" render test — fail right-reason: empty agents tab, no helper) → impl untouching tests (diff over test paths EMPTY). Net-new additive tests in a separate labelled follow-up commit.</tdd_discipline>
  <wire_signature_audit>Pin: addNode(kind,ref,position) (builderStore.ts:506) + the ${kind}:${ref} idempotence guard (:509); builderAgent(id) (:145); the {kind,ref} payload on application/x-builder-node (Palette.tsx:274 / BuilderCanvas.tsx:88-96); the agent id pattern ^[a-z][a-z0-9-]*$ (agent.v1.json:14).</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="A.5"/>
  <execution_steps><implement>Frontend gates (prettier/eslint/tsc/vitest≥80/npm audit) + test:e2e:tauri. No backend change. No schema change.</implement></execution_steps>
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
    <lock>Agents tab only — the "+ New" affordance for tools/skills + the missing primitives (Plan/MCP/rails/budget) are M10. No drop-handler or store-core change.</lock>
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
Strict v1.8 two-commit TDD (red: the e2e + the unit fail right-reason — no capability editor, `builderAgent` lacks `capabilities`; impl untouching tests). Frontend gates + `test:e2e:tauri`. Mutation **advisory** (renderer + a store constructor; no enforcer change — the enforcer already consumes `file_access`, declaration-only). Stage-D design review (the file_access group matches `DESIGN.md`). **No execution-status flip** (the *enforced* write lands at M09.D's run).

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
    is observed at M09.D. Scope α to file_access; the full Capabilities surface
    (network/shell/tools_called UI) is M10.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.B — B.1-B.6)</file>
    <file>schemas/common.v1.json (#/$defs/Capabilities @39 — required fields + file_access:{read,write}:FileGlobList) + schemas/agent.v1.json (capabilities @41, required @9)</file>
    <file>src/lib/builderStore.ts (builderAgent @145 — add capabilities; updateNode @519-526 merges the patch) </file>
    <file>src/components/builder/NodeConfigPanel.tsx (findAgent @12; the fields @105-146; the AllowedList pattern @32-79 to reuse)</file>
    <file>crates/runtime-main/tests/capability_live_tool.rs (E-02 — file_access.write enforcement: in-scope lands, out-of-scope blocked) + docs/adr/0020-* + CLAUDE.md §5/§6/§8</file>
  </read_first>
  <deliverable>builderAgent initializes a valid Capabilities (all-empty, file_access {read:[],write:[]}); NodeConfigPanel gains a file_access editor (Read/Write glob lists) writing agent.capabilities.file_access via updateNode. A created agent is schema-valid (given a role) and can be granted the write scope that makes its Write land. Declaration-only — no enforcer/tier change. α scopes to file_access; network/shell/tools_called UI is M10.</deliverable>
  <tdd_discipline strict="true">Two commits: red (tests/e2e-tauri/builder_file_access.e2e.ts + a vitest unit asserting the file_access mutation + builderAgent's capabilities shape — fail right-reason: no editor, no capabilities) → impl untouching tests (diff over test paths EMPTY).</tdd_discipline>
  <wire_signature_audit>Pin: Capabilities (common.v1.json:39 — required {tools_called,skills_loaded,file_access,network,shell,spawn_agents}; file_access.{read,write}:FileGlobList); updateNode merges {...entry,...patch} (builderStore.ts:522-523); AllowedList (NodeConfigPanel.tsx:32-79); enforcement E-02 (capability_live_tool.rs).</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="B.5"/>
  <execution_steps><implement>Frontend gates + test:e2e:tauri. No Rust/enforcer change (declaration-only — the enforcer already consumes file_access). No schema change.</implement></execution_steps>
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
    <lock>file_access (read/write globs) is the α surface; the rest of Capabilities (network/shell/tools_called/spawn_agents UI) is M10.</lock>
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
the enforced write is observed at M09.D. file_access is the α surface;
network/shell/tools_called UI is M10.
```

---

## Stage M09.C — Attach a real MCP server's tool

### C.1 Problem statement
The data-bearing tools an agent needs are unreachable on the canvas. `mcp_list_servers` (`commands.rs:1083`) lists installed servers and `mcp_test_connection` (`:1049`) enumerates a server's tools (`McpClient::test_connection`→`conn.list_tools()`, `client/mod.rs:246-259`, returning `Vec<McpTool>`), but **no command lists an *already-installed* server's tools by name** — `test_connection` takes an inline `config`, not a registered server name. And the Palette Tools tab (`Palette.tsx:143-160`) offers only built-ins + installed-artifacts + framework tools, never a connected MCP server's tools. Dispatch itself is solved: `try_mcp_dispatch` (`agent_sdk.rs:884`) → `McpDispatcher` (`dispatch.rs:91`) resolves a tool name to `server__tool`, enforces L1/L4, and invokes it — **an `allowed_tools` entry that names an MCP tool already dispatches in a real run.** The missing links are *enumerate by server name* + *surface in the palette* + the **precondition that the Tester run path wires the dispatcher**.

### C.2 Files to change
**`<wire_signature_audit>` — pin before pseudocode:** `McpClient::test_connection(transport) -> Vec<McpTool>` (`client/mod.rs:246`); `McpClient::list_servers` (`:270`) + `registry.get(name)` (the registry lookup); `McpClient::transport_from_config(&McpServerConfig)` (used at `commands.rs:1069`); `mcp_list_servers` command shape (`commands.rs:1083-1100`, `*_with` seam); the `McpTool` shape `{name, description?, input_schema}` (Rust `runtime-core`/`runtime-mcp` — **the build pins the exact path**; the Explore-cited `types/mcp.ts` location was wrong); `paletteItemsForTab('tools', …)` (`Palette.tsx:143`); `build_mcp_dispatcher` (`commands.rs:271`) + whether `test_framework` calls it.
- `src-tauri/src/commands.rs` — a new `mcp_list_server_tools(name: String) -> Vec<McpTool>` Tauri command (+ a `*_with` test seam): look the server up in the registry, rebuild its transport, and reuse the `test_connection` connect→`list_tools`→disconnect path (or the cached `get_connection` + `list_tools`). A read-only enumeration; no new MCP machinery.
- `src/lib/ipc.ts` — `mcpListServerTools(name): Promise<McpTool[]>`.
- `src/components/builder/Palette.tsx` — the Tools tab gains the installed servers' tools as `source:'mcp'` items (labelled `server · tool`), fetched once on mount like `listInstalledArtifacts` (`:214-222`); a drop adds the tool to `framework.tools` (`applyDrop` 'tool' case, `builderStore.ts:190-197`) and an Agent→Tool edge wires `allowed_tools` (`connectEdgeReducer:419`); B's editor mirrors it into `capabilities.tools_called`.
- **Precondition check (the α.C/D integration risk):** confirm `test_framework` (the Tester run path, `commands.rs`) builds + injects the MCP dispatcher (`build_mcp_dispatcher`, `:271`, as `run_smoke_session` does at `:178`). If it does not, wiring it is part of this stage — without it the authored MCP tool will not dispatch in a Tester run (`<construction_reachability_check>` C.5).
- `tests/e2e-tauri/builder_mcp_tool.e2e.ts` (new) + a Rust unit on `mcp_list_server_tools_with` (a stub/registered server's tools enumerate) + a vitest on the palette items.

### C.3 Detailed changes
1. **`mcp_list_server_tools` (Rust).** Mirror `mcp_test_connection_with` (`commands.rs:1061-1075`) but key on a registered name — `registry.get(name)` → `transport_from_config` → `test_connection(transport)` (or `get_connection(name, …)` + `list_tools`). Returns `Vec<McpTool>`. Read-only; no persistence.
2. **`mcpListServerTools` (ipc.ts).** A thin `invoke('mcp_list_server_tools', { name })` wrapper, mirroring `mcpTestConnection` (the existing `Vec<McpTool>` bridge).
3. **Palette MCP tools.** On mount, for each `mcp_list_servers()` entry, fetch its tools and add `{ kind:'tool', ref: '<server>__<tool>' (or the short tool name the resolver accepts), label:'<server> · <tool>', source:'mcp' }` to the Tools tab, de-duped against built-ins/installed/framework (the existing `dedupeByKindRef`, `Palette.tsx:120`). The drag contract is unchanged (`{kind:'tool', ref}`).
4. **Wire to the agent.** Dropping the MCP tool node + an Agent→Tool edge records it in `allowed_tools` (`connectEdgeReducer:419`); the M09.B file_access editor's sibling pattern adds it to `capabilities.tools_called`. At run time, `try_mcp_dispatch` resolves the name to the server's tool and dispatches it (already executing).
5. **Tester dispatcher wiring (if absent).** If `test_framework` does not call `build_mcp_dispatcher`, add it (the same construction `run_smoke_session` uses) so a Tester run dispatches MCP tools. This is execution-wiring → the **mutation gate is blocking on that diff**.

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
`inputs_reachable="false"`: no command lists an installed server's tools by name (only `test_connection` by inline config); the Palette Tools tab never surfaces MCP tools (`Palette.tsx:143-160`); **and** `test_framework` may not wire the MCP dispatcher (the build pins this — if absent, an authored MCP tool cannot dispatch in the Tester). M09.C inverts: the new command + the palette source + (if needed) the Tester dispatcher wiring. Verify by grep: `mcp_list_server_tools` in `commands.rs` + `ipc.ts`; `source:'mcp'` items in `Palette.tsx`; `build_mcp_dispatcher` reachable from `test_framework`.

### C.6 Close gate
Strict v1.8 two-commit TDD (red: the e2e + the Rust `*_with` unit + the palette vitest fail right-reason — no command, no MCP palette items; impl untouching tests). Rust gates (runtime-main/`src-tauri`) + frontend gates + `test:e2e:tauri`. **Mutation gate: advisory on the read-only `list_tools` command; BLOCKING on any `test_framework` dispatcher/enforcer-wiring diff** (execution-wiring, `cluster-pattern.md` §5). Stage-D design review (the `source:'mcp'` palette items match `DESIGN.md`). **No execution-status flip** (the dispatched-in-a-run observation is M09.D).

### C.7 CLI prompt
```xml
<work_stage_prompt id="M09.C">
  <context>
    M09 Stage C — make a real MCP server's tools attachable on the canvas. MCP
    dispatch ALREADY executes in the run loop (agent_sdk.rs:884 try_mcp_dispatch →
    dispatch.rs:91, capability-enforced) — an allowed_tools entry naming an MCP
    tool dispatches. The gaps: (1) no command lists an INSTALLED server's tools by
    name (mcp_test_connection:1049 takes an inline config; McpClient::test_connection
    →list_tools, client/mod.rs:246, returns Vec<McpTool>); (2) the Palette Tools tab
    (Palette.tsx:143-160) never surfaces MCP tools; (3) PRECONDITION RISK — verify
    test_framework (the Tester run path) wires build_mcp_dispatcher (commands.rs:271,
    as run_smoke_session does @178); if not, an authored MCP tool won't dispatch in
    a Tester run and wiring it is part of THIS stage (execution-wiring → mutation
    BLOCKING on that diff). Add mcp_list_server_tools(name)→Vec<McpTool> (reuse the
    registry.get + transport + list_tools path), an ipc wrapper, and source:'mcp'
    palette items; a drop + Agent→Tool edge records allowed_tools (connectEdge:419)
    + capabilities.tools_called. Reuse existing MCP machinery; no new transport.
    NOTE: the McpTool type location the prior survey cited (types/mcp.ts) was WRONG
    — pin the real Rust/TS McpTool {name, description?, input_schema} in the audit.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.C — C.1-C.6; the test_framework dispatcher precondition)</file>
    <file>src-tauri/src/commands.rs (mcp_test_connection @1049 + _with @1061; mcp_list_servers @1083; build_mcp_dispatcher @271 + its run_smoke_session call @178; the test_framework run path — VERIFY it wires build_mcp_dispatcher)</file>
    <file>crates/runtime-mcp/src/client/mod.rs (test_connection @246 → list_tools; list_servers @270; get_connection @289) + crates/runtime-mcp/src/dispatch.rs (McpDispatcher @91) + crates/runtime-main/src/sdk/agent_sdk.rs (try_mcp_dispatch @884)</file>
    <file>src/components/builder/Palette.tsx (the tools case @143-160; listInstalledArtifacts-on-mount @214-222; dedupeByKindRef @120; the drag contract @269-279) + src/lib/ipc.ts (mcpTestConnection / mcpListServers — the Vec<McpTool> bridge) + src/lib/builderStore.ts (applyDrop 'tool' @190; connectEdgeReducer agent->tool @419)</file>
    <file>schemas/mcp.v1.json (McpServerConfig/McpTool) + docs/adr/0021-* + docs/cluster-pattern.md (§5 mutation gate) + CLAUDE.md §5/§6/§8</file>
  </read_first>
  <deliverable>A new mcp_list_server_tools(name)→Vec<McpTool> command (+ _with seam) reusing the registry+list_tools path; an mcpListServerTools ipc wrapper; the Palette Tools tab surfaces an installed server's tools (source:'mcp'); a drop + Agent→Tool edge records allowed_tools + capabilities.tools_called. If test_framework does not wire build_mcp_dispatcher, wire it (so the Tester dispatches MCP tools). Reuses existing MCP machinery; no new transport.</deliverable>
  <tdd_discipline strict="true">Two commits: red (tests/e2e-tauri/builder_mcp_tool.e2e.ts + a Rust mcp_list_server_tools_with unit + a Palette source:'mcp' vitest — fail right-reason: no command, no MCP palette items) → impl untouching tests (diff over test paths EMPTY).</tdd_discipline>
  <wire_signature_audit>Pin: McpClient::test_connection(transport)->Vec<McpTool> (client/mod.rs:246); registry.get(name) + transport_from_config (commands.rs:1069); the McpTool shape {name,description?,input_schema} (pin the REAL Rust+TS path — the prior types/mcp.ts cite was wrong); build_mcp_dispatcher (commands.rs:271) + its presence/absence in test_framework; connectEdgeReducer agent->tool (builderStore.ts:419).</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="C.5"/>
  <execution_steps><implement>Surface the test_framework-dispatcher finding BEFORE coding if it needs wiring (it is execution-wiring — mutation BLOCKING on that diff). Rust gates (runtime-main ≥95 / src-tauri) + frontend gates + test:e2e:tauri. No schema change.</implement></execution_steps>
  <close_gate>
    <real_app_irl>Maintainer: with an MCP server installed, the Tools tab shows its tools (source mcp); drag one + connect the agent → allowed_tools records it. (The dispatched-in-a-run observation is M09.D.)</real_app_irl>
    <mutation_gate blocking="true">BLOCKING on any test_framework dispatcher/enforcer-wiring diff (execution-wiring); advisory on the read-only list_tools command.</mutation_gate>
    <design_review>The source:'mcp' palette items match DESIGN.md (the palette item style + source badge).</design_review>
    <cumulative_regression>builder_mcp_tool.e2e.ts joins the e2e-tauri suite; the mcp_list_server_tools unit joins the Rust suite.</cumulative_regression>
  </close_gate>
  <approval_surface>
    <item>cross-machine state + the red→impl diff over test paths (EMPTY)</item>
    <item>the test_framework-dispatcher finding (wired already, or wired here) + the mutation-gate result on that diff</item>
    <item>Rust + frontend gate results + test:e2e:tauri</item>
    <item>the attach-MCP-tool walkthrough + Stage-D design-review note</item>
    <item>explicit: "M09.C is ready. I will not commit until you approve."</item>
  </approval_surface>
  <scope_locks>
    <lock>Reuse existing MCP machinery — the new command only enumerates (registry + list_tools); no new transport, no dispatch change.</lock>
    <lock>MCP-server-as-canvas-node + the data-source catalog + credentials UX are M11 — α only attaches an installed server's tool to an agent.</lock>
    <lock>If test_framework needs the dispatcher wired, that is the only execution-wiring change; surface it first (Hard Rule 8) and gate it blocking-mutation.</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Whether test_framework already wired the dispatcher (the precondition finding); that dispatch already executed (the gap was enumeration + palette); the McpTool-type-location correction (prior survey wrong); rule 11 (dispatch-in-run observed at M09.D).</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="Stage M09.C"/>
</work_stage_prompt>
```

### C.8 Commit message
```
feat(M09.C): attach a real MCP server's tool on the canvas

MCP dispatch already executed in the run loop (agent_sdk.rs:884), but no
command listed an installed server's tools by name and the palette never
surfaced them. Add mcp_list_server_tools(name) (reusing the registry +
list_tools path), an ipc wrapper, and source:'mcp' Tools-tab items; a drop +
Agent->Tool edge records allowed_tools + capabilities.tools_called. Wire
build_mcp_dispatcher into test_framework if absent (so the Tester dispatches
MCP tools) — gated blocking-mutation as execution-wiring. Reuses existing MCP
machinery; the data-source catalog + MCP-as-node are M11.
```

---

## Stage M09.D — The vertical-slice IRL (assembled, end-to-end)

### D.1 Problem statement
M09.A–C are unit/e2e-green in isolation, but **the whole loop has never run in the assembled app**: build a fresh agent → grant `file_access.write` → attach an MCP tool → Run → real data pulled + a real file written, at the tracked tier. Per the assembled-app-regression mandate (CLAUDE.md v1.8), the close is an assembled test that exercises the **real Tester run path** (`test_framework` against the real `AnthropicProvider`, real in-process Read/Write, real MCP dispatch, the real L2/L4 enforcer) — not the isolated pieces. The falsifiable hypothesis: *`test_framework` wires the MCP dispatcher + the authored agent's `capabilities` into the enforcer exactly as `run_smoke_session` does.* If it fails, the gap is integration wiring in the Tester run path, closed here.

### D.2 Files to change
**`<wire_signature_audit>` — pin before pseudocode:** `test_framework` (`commands.rs`) — the run path, the enforcer construction (does it `set_tier` from `CurrentTierState` + consume the agent's `capabilities`?), the dispatcher injection (`build_mcp_dispatcher`); the Tester UI run trigger (`openTester`/`testerOpen`, `builderStore.ts:541`); the execution-status row to flip.
- `tests/e2e-tauri/vertical_slice.e2e.ts` (new — the assembled close): drive the built app through `tauri-driver` to author the agent + scope + MCP tool and Run, asserting a real file is written within scope (and an out-of-scope target is denied). Key-gated on a live model + an installed MCP server (the maintainer's IRL machine; CI runs the key-independent subset).
- Any **integration glue** the assembled test surfaces (e.g., `test_framework` not threading `capabilities` or the dispatcher) — closed here, gated blocking-mutation.
- `docs/execution-status.md` — flip a new row: *canvas-authored single-agent + file_access + MCP-tool, observed end-to-end in the app, eval `vertical_slice.e2e.ts` + the IRL date.*

### D.3 Detailed changes
1. **The assembled e2e** authors the framework through the real UI (reusing M09.A–C surfaces) and Runs it via the Tester, asserting the on-disk side effect (a file written under the granted `write` glob from MCP-sourced data) + the out-of-scope denial — the observable behavior, not an emitted event (rule 11).
2. **Close any integration gap** the test exposes in `test_framework` (tier from `CurrentTierState`; the agent `capabilities` reaching the enforcer; `build_mcp_dispatcher` injected). Minimal, gated.
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
`inputs_reachable="false"` (the integration hypothesis): the assembled Tester path may not thread the dispatcher + the authored `capabilities` + the tracked tier together (each proven only in isolation). M09.D inverts it by running the real composition and asserting the on-disk effect; any missing wire is closed here. Verify: `vertical_slice.e2e.ts` writes a real file from MCP data within scope and is denied outside it, on the built app.

### D.6 Close gate
Strict v1.8 two-commit TDD on any glue (red: the assembled e2e fails right-reason; impl untouching tests). Rust + frontend gates + **`test:e2e:tauri`**. **Mutation BLOCKING** on any `test_framework` enforcer/dispatcher/tier wiring touched (execution-wiring). **The maintainer real-app IRL is the authoritative close** — author fresh → scope → attach MCP tool → Run → real file written within scope, denied outside, observable in-app. **Flip the execution-status row** citing `vertical_slice.e2e.ts` + the IRL date. This closes M09 — the vertical slice is real.

### D.7 CLI prompt
```xml
<work_stage_prompt id="M09.D">
  <context>
    M09 Stage D — the assembled vertical-slice IRL. A-C are green in isolation; the
    whole loop has never run in the real app. Per the v1.8 assembled-app mandate,
    the close is an assembled test on the REAL Tester run path (test_framework vs
    the real AnthropicProvider, real in-process Read/Write, real MCP dispatch, the
    real L2/L4 enforcer): author a fresh agent → grant file_access.write → attach an
    installed MCP server's tool → Run → assert a real file is written within scope
    from real MCP data, and an out-of-scope write is denied (the on-disk effect, not
    an event — rule 11). Falsifiable hypothesis the test must disprove:
    test_framework wires build_mcp_dispatcher + the authored agent's capabilities +
    the tracked tier exactly as run_smoke_session does. If it fails, close the
    integration gap here (execution-wiring → mutation BLOCKING). The maintainer
    real-app IRL is the authoritative close; flip the execution-status row. This
    closes M09.
  </context>
  <read_first>
    <file>docs/build-prompts/M09-workbench-vertical-slice.md (Stage M09.D — D.1-D.6; the Background falsifiable hypothesis)</file>
    <file>src-tauri/src/commands.rs (test_framework — the run path: enforcer construction + set_tier from CurrentTierState + the agent capabilities + build_mcp_dispatcher @271 / @178 run_smoke_session as the reference wiring)</file>
    <file>crates/runtime-main/tests/capability_live_tool.rs (E-02 file_access enforcement) + crates/runtime-main/src/sdk/agent_sdk.rs (the run loop + try_mcp_dispatch @884)</file>
    <file>src/lib/builderStore.ts (openTester/testerOpen @541; useTestGraphStore @444) + the M09.A-C surfaces (the authored framework)</file>
    <file>docs/execution-status.md (the row to flip + the maintenance protocol) + docs/cluster-pattern.md (§1 IRL close, §5 mutation) + docs/adr/0021-* + CLAUDE.md §4 rule 11 / §5/§6/§8</file>
  </read_first>
  <deliverable>An assembled tests/e2e-tauri/vertical_slice.e2e.ts that drives the built app to author a fresh agent + file_access.write + an installed MCP tool and Run it, asserting a real file is written within scope from real MCP data and an out-of-scope write is denied; any integration glue in test_framework (dispatcher/capabilities/tier) the test surfaces, closed; the execution-status row flipped on the maintainer IRL. Closes M09.</deliverable>
  <tdd_discipline strict="true">Two commits: red (vertical_slice.e2e.ts + any glue's assembled regression fail right-reason: the run does not write the file / does not dispatch MCP / ignores the authored tier) → impl untouching tests (diff over test paths EMPTY).</tdd_discipline>
  <wire_signature_audit>Pin: test_framework run path — enforcer construction + set_tier(CurrentTierState) + the agent capabilities into the enforcer + build_mcp_dispatcher injection (vs run_smoke_session @178); openTester/testerOpen (builderStore.ts:541); the execution-status row + maintenance protocol.</wire_signature_audit>
  <construction_reachability_check ref="docs/build-prompts/M09-workbench-vertical-slice.md" section="D.5"/>
  <execution_steps><implement>Surface any test_framework wiring finding BEFORE coding (Hard Rule 8; execution-wiring → mutation BLOCKING). Rust + frontend gates + test:e2e:tauri. No schema change.</implement></execution_steps>
  <close_gate>
    <real_app_irl>Maintainer (Promoted), on the real Tauri app: fresh project → author an agent → grant file_access.write "out/**" → attach an installed MCP server's tool → Run in the Tester → a real file is written under out/ from real MCP data; a write outside out/ is denied; the run is observable in-app. THE authoritative close (rule 11 / ADR-0021).</real_app_irl>
    <mutation_gate blocking="true">BLOCKING on any test_framework enforcer/dispatcher/tier wiring change (execution-wiring).</mutation_gate>
    <design_review>The end-to-end run surfaces (Tester, Output rail, nodes) match DESIGN.md.</design_review>
    <cumulative_regression>vertical_slice.e2e.ts joins the e2e-tauri suite; the execution-status row joins the cumulative execution-regression eval.</cumulative_regression>
  </close_gate>
  <approval_surface>
    <item>cross-machine state + the red→impl diff over test paths (EMPTY)</item>
    <item>the test_framework integration finding (already-wired or closed here) + the blocking mutation result on that diff</item>
    <item>Rust + frontend gates + test:e2e:tauri; the execution-status row flip (citing vertical_slice.e2e.ts + the IRL date)</item>
    <item>the full real-app IRL walkthrough (author → scope → attach MCP → Run → real file, denied out-of-scope, observable)</item>
    <item>explicit: "M09.D is ready. I will not commit until you approve."</item>
  </approval_surface>
  <scope_locks>
    <lock>The vertical slice only — single agent, one MCP tool, file_access write. Multi-agent/plan/hook execution is M12; the data-source catalog is M11.</lock>
    <lock>Any test_framework change is integration wiring to make the AUTHORED framework run as run_smoke_session already does — never a new execution primitive, never an enforcer-widening.</lock>
    <lock>The flip requires the maintainer real-app IRL observation (rule 11) — CI e2e-green alone does NOT flip the row.</lock>
  </scope_locks>
  <gates milestone="M08"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Whether the assembled test disproved or confirmed the test_framework-wiring hypothesis; the on-disk-effect assertion (not an event — rule 11); the execution-status flip + IRL date; that this is the first canvas-authored real run.</special_log>
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
outside it. Close any integration gap in test_framework (dispatcher/
capabilities/tier wiring) the assembled test surfaces. Flip the execution-status
row on the maintainer real-app IRL. The workbench now builds AND runs a real
single-agent, MCP-data workflow from scratch.
```

---

## Verification checklist (before the M09 PR pushes)

- [ ] Every UI stage closed on the **real Tauri app via `tauri-driver`** + the maintainer IRL — not Playwright-mock-green, not code-read (rule 11 / ADR-0021).
- [ ] **M09.D** ran the **assembled Tester path** (real provider + real MCP dispatch + real enforcer) and asserted the **on-disk file** within scope + the out-of-scope denial — the assembled-app-regression mandate, not the isolated pieces.
- [ ] The `test_framework` dispatcher/`capabilities`/tier wiring hypothesis was explicitly **confirmed or closed** (M09.C/D), gated blocking-mutation on any execution-wiring diff.
- [ ] The execution-status ledger gained the **canvas-authored single-agent + MCP + file_access** row, citing `vertical_slice.e2e.ts` + the IRL date.
- [ ] Strict v1.8 two-commit TDD held on every code stage (red→impl diff over test paths EMPTY).
- [ ] Scope locks held — no new execution primitive (M12), no MCP-as-node/catalog (M11), no schema change; capability authoring stayed declaration-only.
- [ ] The closeout states the next milestone: **M10** (author-anything — the rest of the palette + config for every node kind + delete/rename + the `Bash` integrity fix).

## Summary table

| Stage | Goal | Close bar |
|---|---|---|
| M09.A | Blank-create an agent (the "New agent" affordance) | real-app IRL: a fresh project authors its first agent on the canvas |
| M09.B | file_access editor (the agent's `capabilities.file_access`) | real-app IRL: a write glob is authorable; the framework validates |
| M09.C | Attach a real MCP server's tool (enumerate + palette + wire) | real-app IRL: an installed server's tools are draggable + recorded in allowed_tools; the Tester dispatcher wiring confirmed |
| M09.D | The vertical-slice IRL (assembled, end-to-end) | **maintainer real-app IRL: author → scope → attach MCP → Run → real file written within scope, denied outside, observable**; execution-status flipped |

**Ending:** the workbench **builds and runs a real single-agent, MCP-data workflow from scratch** — author an agent on an empty canvas, scope it, attach a real data tool, Run, watch it write a real file at the enforced tier. The first turn of paint into a real, authored, executing product. **M10** widens authoring to the whole palette; **M11** the data-source catalog; **M12** multi-agent/plan/hook execution.
