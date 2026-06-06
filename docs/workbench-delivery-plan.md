# Workbench Delivery Plan — author-anything + real-data execution

> **Status:** proposed (2026-06-05). Re-cuts the remaining roadmap around the
> product the maintainer actually wants: an app where you **build any agentic
> workflow** — drag every primitive from a complete palette, or import JSON —
> **configure it for real** (capabilities, models, MCP/API data), and **run it
> for real**, industrial-strength. This document supersedes the M08.8 stage-trim
> (tier-display / budget-bar / gap-resume polish) as the sequencing spine. It is
> grounded in the actual code (file:line) and the executes-vs-paints ledger
> (`docs/execution-status.md`), and researched against the 2026 industrial bar.

---

## 1. Ground truth (what runs vs what is painted)

Per `docs/execution-status.md` (rule 11 — IRL-confirmed, not inferred):

| Executes today (real) | Paints only (drawn, no work) |
|---|---|
| Single-agent provider streaming (multi-turn) | **Sub-agents** — spawn emits a node, no child runs (`agent_sdk.rs:467`) |
| **MCP tool dispatch** — capability-enforced, in the run loop (`agent_sdk.rs:884`, `dispatch.rs:91`) | **Plans** — `drive_plan` runs zero tasks, no production caller (`plan_loop.rs`) |
| Built-in **Read / Write** (`Bash`/`Glob`/`Grep`/`WebFetch` NOT built) | **Hooks / rails** — no firing engine |
| Skills (`LoadSkill`), gap **suspend**, budget engine, **tier enforcement** | Gap **resume** (suspend works; resume not built) |

**The load-bearing fact:** a *single agent* doing Read/Write + an **MCP tool** +
a skill, gated by capability + tier, **runs end-to-end today**. The vertical
slice below rides exactly that path — so it is buildable now, not a research
project. Everything multi-agent / plan / hook is paint → the back half of this
plan turns it into execution.

---

## 2. The industrial bar (researched)

Every serious node-based builder (n8n, Flowise, LangFlow, Dify, Copilot Studio,
Microsoft Agent Framework) converges on the same shape:

1. **Visual graph over an exportable JSON document.** Drag nodes onto a canvas,
   connect them, and the canvas *is* a view over a portable JSON workflow you can
   export/import. **We already have this** (ADR-0020: `framework.json` is the
   source of truth; the canvas + JSON view are two editors over it —
   `builderStore.ts:8`). This is the correct architecture.
2. **A complete, categorized palette you build *from scratch*.** Drag a fresh
   Agent / Tool / data-source node, not only pre-existing ones.
3. **Deep per-node configuration + credentials.** Every node type configurable;
   secrets handled (keychain).
4. **100+ data integrations — in 2026 that means MCP-native.** MCP is "the USB-C
   for AI": 97M+ monthly SDK downloads, 200+ servers (GitHub, Postgres, Slack,
   Google Drive, Notion, Jira, Salesforce), backed by every major vendor. The
   pattern is: connect a server → it advertises its tools (`tools/list`) → attach
   tools to an agent → the agent calls them mid-run. **Our MCP layer already does
   the connect + advertise + dispatch** (`mcp_test_connection` → `list_tools`,
   `client/mod.rs:246`; `try_mcp_dispatch`, `agent_sdk.rs:884`).

**Verdict:** our architecture matches the industrial shape. The gaps are
**breadth of authoring** (the palette/config) and **execution completeness**
(the painted primitives) — not the design. This is a finish, not a rewrite.

---

## 3. Architecture verdict — what's right, what's missing

**Right (extensible, keep):**

- **Document-as-source-of-truth + projection** (`builderStore.ts`): the canvas is
  derived from `framework` via `projectCanvasNodes`/`projectCanvasEdges`
  (`:223`/`:308`); edits mutate the document. Adding a primitive = extend the
  `BuilderNodeKind` union (`:24`) + `applyDrop` (`:183`) + the projection + a node
  component. A clean, repeatable seam.
- **Create-from-scratch already works at the store layer.** `addNode(kind, ref,
  pos)` (`:506`) mints `builderAgent(ref)` (`:145`) into `framework.agents`. The
  blocker is only that the **Palette never offers a blank item**.
- **MCP layer is strong:** registry (`registry.rs`), capability-enforced dispatch
  wired into the run loop (`dispatch.rs:91`, `agent_sdk.rs:884`), health-pinging,
  tool enumeration (`McpClient::test_connection`/`list_tools`), an `MCPNode`
  canvas component already exists (`nodes/MCPNode.tsx`).
- **Capability model is well-formed + two-layered.** The per-action *declaration*
  (`capability.v1.json`: `{kind, resource, scope, side_effect_class}`) is the
  enforcer's L1/L2 shape; the **agent** carries the `Capabilities` *aggregate*
  (`common.v1.json#/$defs/Capabilities`: `file_access.{read,write}` globs +
  `tools_called`/`network`/`shell`/`spawn_agents`), required per `agent.v1.json:9`.
  The live-tool eval (E-02, `capability_live_tool.rs`) proves an in-scope Write
  lands and an out-of-scope Write is blocked with no file on disk. **α.B edits the
  agent's `file_access`.**
- **Continuous schema validation** on canvas edits (`validate_framework`,
  debounced — `builderStore.ts:454`).

**Missing / broken (the work):**

| # | Gap | Evidence |
|---|---|---|
| G1 | Palette can't create from scratch (Agents/Skills tabs empty on a fresh project) | `Palette.tsx:173-184` — agents = installed + framework only, no built-in/blank |
| G2 | Half the vocabulary has no palette: **Plan, MCP-server, rails, budget** | `BuilderNodeKind = agent\|tool\|skill\|hitl\|hook` (`builderStore.ts:24`); tabs `:24` |
| G3 | Config is agent-only + shallow — **no capabilities/file_access**; tool/skill/hook/hitl have no config panel | `NodeConfigPanel.tsx:98-100,129-146`; `builderAgent` omits `capabilities` (`:145`) |
| G4 | No way to attach a real MCP server's tools on the canvas | no `mcp_list_server_tools` command; `MCPNode` not in the builder projection |
| G5 | Whole-workflow JSON import is an unvalidated textarea (`as Framework`) | `JsonView.tsx:45`; `replaceFramework` bypasses `validate_framework` (`builderStore.ts:479`) |
| G6 | Palette advertises unbuilt capability (`Bash` draggable, doesn't execute) | `Palette.tsx:28` vs `execution-status.md` |
| G7 | Can't delete a node | `removeNode` is a no-op stub (`builderStore.ts:535`) |
| G8 | Past one agent nothing runs (sub-agents/plans/hooks paint) | `execution-status.md` rows 2/5/6 |

---

## 4. The plan — five milestones

Sequenced so a real, end-to-end, industrial-feeling result lands **first**, on the
substrate that already executes; breadth and execution-completeness follow.

### M-α — The Vertical Slice: *one real agent, real data, real file*

**Why first:** proves the entire thesis end-to-end on the executing substrate, and
de-risks every later milestone. It is small because the substrate exists.

**Closing IRL (rule 11 / ADR-0021):** on a fresh project, the maintainer drags a
**new** agent onto the canvas, gives it a write `file_access` capability + attaches
a **real** MCP tool (e.g. a Postgres or GitHub MCP server), hits Run, and watches
it **pull real data and write a real file to disk** — at the tracked tier; an
out-of-scope write is blocked.

| Stage | Deliverable | Real seams (grounded) |
|---|---|---|
| **α.A** | **Blank-create on the canvas.** A "+ New agent / tool / skill" affordance per Palette tab that mints a fresh unique ref and calls `addNode` → a configurable node appears on a fresh project. | `Palette.tsx` (add the "new" item); `builderStore.addNode` (`:506`) + `builderAgent` (`:145`) already support it; auto-id now (full id-rename → M-β, `updateNode` re-keys `:519`). |
| **α.B** | **file_access editor.** `builderAgent` initializes a valid `capabilities` (the `Capabilities` aggregate — `common.v1.json#/$defs/Capabilities`, **required** per `agent.v1.json:9`, today omitted); a Read/Write glob-list editor in `NodeConfigPanel` over `agent.capabilities.file_access.{read,write}`. | `builderAgent` (`:145`); `NodeConfigPanel.tsx`; shape from `common.v1.json#/$defs/Capabilities` + `agent.v1.json:41`; enforcement already real (E-02 `capability_live_tool.rs`). |
| **α.C** | **Attach a real MCP tool.** New `mcp_list_server_tools(name)` Tauri command (reuse `McpClient::test_connection`→`list_tools`); surface an installed server's tools in the Tools palette (`source:'mcp'`); attaching adds to the agent's `allowed_tools`. | new command wraps `client/mod.rs:246`; `mcp_list_servers` (`commands.rs:1083`); agent→tool edge already writes `allowed_tools` (`builderStore.ts:419`); dispatch already executes (`agent_sdk.rs:884`). |
| **α.D** | **Assembled IRL + execution-status flip.** Build fresh → Run → real MCP data → real file. Flip the ledger: "canvas-authored single-agent + MCP + capability path observed end-to-end in the app." | `tests/e2e-tauri/` real-app regression; `docs/execution-status.md`. |

Strict v1.8 TDD; the α.C MCP command + α.B capability wiring are the higher-risk
seams (real backend). Mutation advisory on the renderer, **blocking** on any
capability-enforcer touch (none expected — α.B authors a declaration the existing
L2 enforcer already consumes).

### M-β — Author-anything (complete the palette + config)

Make the canvas a real authoring tool for the **whole** vocabulary.

- **β.1 Missing primitives as first-class canvas citizens:** extend
  `BuilderNodeKind` + `applyDrop` + `projectCanvasNodes` + node components for
  **Plan, MCP-server, rails, budget** (each lands in its `framework` home).
- **β.2 Config for every node kind** (the "D2 widens to other kinds" that never
  shipped — `NodeConfigPanel.tsx:11`): tool, skill, hook, hitl, plan, mcp.
- **β.3 Node delete + agent id-rename** (`removeNode` stub `:535`; `updateNode`
  re-key `:519`).
- **β.4 Palette integrity (G6):** the palette only offers what executes, or marks
  the unbuilt (`Bash`/`Glob`/`Grep`/`WebFetch`) explicitly as "not yet wired."
- **Close:** build a non-trivial multi-node framework entirely on the canvas, no
  JSON, and it validates.

### M-γ — Real data, industrialized

The "pull significant real data into the flow" milestone.

- **γ.1 MCP server as a canvas node** (`MCPNode` exists — wire it into the builder
  projection) + an **agent ↔ MCP-tool** edge so data wiring is visual.
- **γ.2 Data-source catalog:** install MCP servers from *within* the builder
  (GitHub/Postgres/Slack/Drive/Notion…), reusing the validated import pipeline
  (`ImportPanel.tsx`) + the keychain secret path (`auth_secret_ref`).
- **γ.3 MCP resources** (not just tools) + credential UX.
- **Close:** author a flow that reads from a live external system (a real DB / API
  via MCP) and acts on it, watched in the app.

### M-δ — Execution breadth (turn paint into run) — *the old M08.9*

Make authored multi-step workflows actually execute.

- **δ.1 Sub-agents run** — a spawned child runs its own loop and returns a result
  (`agent_sdk.rs:467` paints today; rung 6).
- **δ.2 Plans drive real tasks** — `drive_plan` gets a production caller and runs
  `TaskStarted/TaskCompleted → PlanComplete` (`plan_loop.rs`; ADR-0026; rung 7).
- **δ.3 Hooks/rails fire** on their triggers (rung 8; + the TD-046 `vdr` fix
  before a verify producer wires).
- **Close:** a two-agent and a plan-driven framework, authored on the canvas, run
  to completion in the app (each an IRL flip in `execution-status.md`).

### M-ε — Industrial hardening

- **ε.1 Validated whole-workflow import/export (G5):** route JSON/file/clipboard
  import through `validate_framework`; export to file; never `as Framework` a raw
  paste into the store.
- **ε.2 Save-path robustness** (#32 save-companions, #22 budget-persist, #17
  template) — your built work persists correctly across restart.
- **ε.3 Error/validation UX** + the execution-status integrity audit (nothing in
  the UI claims a capability the runtime lacks).

---

## 5. Sequencing & scope reconciliation (decision needed)

```
M-α  vertical slice ──▶ M-β author-anything ──▶ M-γ real data
   (proves the loop)        (breadth)              (industrial)
                                 └──────────────▶ M-δ execution breadth (backend, parallelizable)
                                                       └──▶ M-ε hardening
```

M-α is the gate: until a single real workflow runs end-to-end **in the app**, the
later milestones are building on an unproven loop. M-δ is backend-heavy and can run
in parallel with M-β/γ once M-α lands.

**This re-cut exceeds the locked v0.1 scope** (§0d: single-session, Novice +
Promoted, Anthropic-only, STANDARD mode). It does not break those locks — it adds
authoring breadth + execution completeness on top. **Recommended scope re-line:**

- **New v0.1 = M-α + M-β + M-γ** — "a real workbench that builds and runs
  single-agent, MCP-data workflows from scratch." Shippable, demoable, honest.
- **v1.0 = M-δ + M-ε** — multi-agent/plan/hook execution + hardening.

That boundary is the maintainer's call (product scope). Everything else here is
technical sequencing I own.

**Milestone numbering.** M-α = **M09** (redefines the deprioritized
generators/mentor milestone), M-β = **M10**, M-γ = **M11**, M-δ = **M12** (the
execution-breadth milestone the handoff called "M08.9"), M-ε = **M13**. Phase
docs use these numeric ids — the stage-prompt validator requires `M\d\d.<X>`
(`bin/validate-stage-prompts.mjs`). `docs/MVP-v0.1.md` and the in-flight M08.8
(stages D/E/F now superseded by this re-cut) are reconciled to this spine at the
M08.8 closeout.

---

## 6. Risks

- **MCP server availability for the α IRL.** Need one real, installable MCP server
  (Postgres or GitHub are the cleanest) with credentials on the build/IRL machine.
  Mitigation: the GitHub MCP server is stdio + a PAT; Postgres MCP is stdio + a
  connection string — both already supported (`stdio` transport, `auth_secret_ref`).
- **α.C tool-enumeration command** is net-new backend (small) — reuses
  `test_connection`'s `list_tools`; the risk is connection lifecycle (the health
  pinger + cache already exist — `client/lifecycle.rs`, `get_connection` `:289`).
- **Paint-vs-execute integrity (G6).** While M-δ is pending, the palette must not
  advertise multi-agent/plan/hook as runnable. β.4 enforces this so the app never
  lies (rule 11).
- **Capability authoring safety.** α.B authors capability *declarations* the
  existing enforcer consumes; it must never widen beyond the user's tier — no
  enforcer change, declaration-only.

---

## 7. Research sources

- n8n / Flowise / LangFlow / Dify comparison (architecture, palette, JSON,
  integrations): <https://huggingface.co/blog/daya-shankar/n8n-vs-flowise-vs-langflow-enterprises>,
  <https://www.zenml.io/blog/langflow-vs-n8n>
- LangFlow agent-node + tool-connection model:
  <https://cohorte.co/blog/langflow-a-visual-guide-to-building-llm-apps-with-langchain>
- MCP (the 2026 data-integration standard): <https://www.anthropic.com/news/model-context-protocol>,
  <https://modelcontextprotocol.io/docs/getting-started/intro>,
  <https://www.essamamdani.com/blog/complete-guide-model-context-protocol-mcp-2026>
- MCP tools in agent builders (enumerate + attach): Microsoft Agent Framework
  <https://learn.microsoft.com/en-us/agent-framework/agents/tools/local-mcp-tools>,
  Copilot Studio <https://learn.microsoft.com/en-us/microsoft-copilot-studio/mcp-add-components-to-agent>
