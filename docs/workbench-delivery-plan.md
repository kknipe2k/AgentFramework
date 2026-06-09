# Workbench Delivery Plan — author-anything + real-data execution

> **Status:** Accepted — ADR-0031 (2026-06-05, author-and-run) + the **ADR-0032
> vertical re-cut** (2026-06-07): re-verticalized M10–M13 and pulled execution
> breadth + shell-exec into v0.1. The authoritative detailed roadmap for the
> product the maintainer wants: an app where you **build any agentic workflow** —
> drag every primitive, or import JSON — **configure it for real** (capabilities,
> models, MCP/API data), and **run it for real**, industrial-strength. Grounded in
> the actual code (file:line) and the executes-vs-paints ledger
> (`docs/execution-status.md`), researched against the 2026 industrial bar.

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

### M09 — The Vertical Slice: *one real agent, real data, real file* (the walking skeleton)

**Why first:** proves the entire thesis end-to-end on the executing substrate, and
de-risks every later milestone. It is small because the substrate exists. Kept **pure**
(ADR-0032): author + run one agent; gap *resume* moves to M10 so the skeleton stays the
thinnest "does it run" thread (it ships with **suspends cleanly**, E-04).

**Closing IRL (rule 11 / ADR-0021):** on a fresh project, the maintainer drags a
**new** agent onto the canvas, gives it a write `file_access` capability + attaches
a **real** MCP tool (e.g. a Postgres or GitHub MCP server), hits Run, and watches
it **pull real data and write a real file to disk** — at the tracked tier; an
out-of-scope write is blocked.

| Stage | Deliverable | Real seams (grounded) |
|---|---|---|
| **M09.A** | **Blank-create on the canvas.** A "+ New agent" affordance that mints a fresh unique ref and calls `addNode` → a configurable node appears on a fresh project. | `Palette.tsx`; `builderStore.addNode` (`:506`) + `builderAgent` (`:145`) already support it (full id-rename → M13, `updateNode` re-keys `:519`). |
| **M09.B** | **file_access editor.** `builderAgent` initializes a valid `capabilities` (`common.v1.json#/$defs/Capabilities`, **required** per `agent.v1.json:9`, today omitted); a Read/Write glob-list editor in `NodeConfigPanel` over `agent.capabilities.file_access.{read,write}`. | `builderAgent` (`:145`); `NodeConfigPanel.tsx`; shape from `common.v1.json#/$defs/Capabilities`; enforcement already real (E-02 `capability_live_tool.rs`). |
| **M09.C** | **Attach a real MCP tool.** New `mcp_list_server_tools(name)` Tauri command (reuse `McpClient::test_connection`→`list_tools`); surface an installed server's tools (`source:'mcp'`); attaching writes `allowed_tools`. | new command wraps `client/mod.rs:246`; `mcp_list_servers` (`commands.rs:1083`); agent→tool edge writes `allowed_tools` (`builderStore.ts:419`); dispatch executes (`agent_sdk.rs:884`). |
| **M09.D** | **Assembled IRL + execution-status flip.** Build fresh → Run → real MCP data → real file. Flip the ledger: "canvas-authored single-agent + MCP + capability path observed end-to-end in the app." | `tests/e2e-tauri/` real-app regression; `docs/execution-status.md`. |

Strict v1.11 two-commit TDD; M09.C/B are the higher-risk seams (real backend). Mutation
advisory on the renderer, **blocking** on any capability-enforcer touch (none expected —
M09.B authors a declaration the existing L2 enforcer already consumes). **No gap-resume
here** (→ M10).

### M10 — HITL steers the run (gap resolve→resume + plans)

The slice where a **human approves/grants to let execution proceed** — the shared theme
of gap-resume, plan-approval, and task execution. Each is authored on the canvas and **runs**.

- **M10.1 Gap resolve→resume** (ADR-0029): M09 ships suspend (E-04); M10 adds **resume** —
  once the user grants/installs/declines, restore session state + resume the loop, with the
  grant→resume affordance in the UI. (Suspend = E-04; resume is the unwired half.)
- **M10.2 Plan task execution** (ADR-0026; rung 7): `drive_plan` gets its **production
  caller** + a task loop — each task runs on the **single-agent** loop (`AgentSdk::run_agent`,
  **not** sub-agents — `plan_loop.rs:7-8,128`), one at a time, behind the **plan-approval
  HITL** gate (`drive_plan` already emits `plan_approval_requested` → awaits the seam,
  `plan_loop.rs:94-126`).
- **Author + run.** Author a plan (tasks) on the canvas + approve it → tasks run
  one-at-a-time in the live graph; a run that hits a gap suspends, you grant, it resumes.
  **Close (IRL):** the maintainer authors-approves-runs a plan and resumes a suspended gap;
  flip the **plans** + **gap-resume** execution-status rows.

### M11 — Sub-agents (sequential)

The **multi-agent entry phase**: an orchestrator spawns a child with narrowed grants; the
child runs its own loop and returns a summary; the parent continues.

- **Sequential only** (`spawn_constraints.max_concurrent: 1`) — the dev loop is inherently
  sequential (research → PRD → plan → implement). Parallel fan-out is v1.0.
- **The wire** (rung 6): `spawn_framework_subagents` (`agent_sdk.rs:467`, paints today —
  emits `AgentSpawned` then stops) gets a real child execution context + narrowed grants +
  the summary returned to the parent.
- **Author + run.** Author a two-agent framework on the canvas (orchestrator → child, e.g.
  **research-agent + PRD-writer**) → the child runs, returns, the parent uses its result.
  **Close (IRL):** the two-agent run; flip the **sub-agents** execution-status row.

### M12 — The verify loop (hooks/rails + shell exec — H)

The objective-verify capability that makes the dev loop self-correcting — and the milestone
where **shell execution comes in scope** (ADR-0032; the §12 correction). **One vertical
capability, staged heavily.**

- **M12.1 Hooks/rails firing engine** (rung 8): a **post-task `verify` hook** fires on its
  trigger and a **`dont_touch` rail** blocks a forbidden edit (defined in framework JSON
  today; **no firing engine** — `M08.7-execution-engine.md` §Background).
- **M12.2 H — controlled shell exec (its own Hard-Rule-8 sub-ladder + ADR):** finish
  `runtime-sandbox`'s exec path — a **controlled-exec isolation profile** (the validation
  profile denies exec today, `landlock.rs:198`), `SandboxRequest::Execute` (`protocol.rs:29`
  is `ValidateArtifact|Shutdown`), the command-spawn — on the **existing** `seccomp` /
  `landlock` / `job_objects` fences. **Threat model: semi-trusted** (your own framework +
  own `verify.sh`, local, single-user, no-telemetry); OS-native is the correct fit,
  **explicitly weaker than microVM** (the v1.0+ upgrade) — recorded, not over-claimed.
  `// SAFETY:` discipline + security-review posture.
- **The loop.** Post-task → `bash verify.sh` runs the tests → **green → next task / red →
  rollback + retry**; a `dont_touch`-violating edit is blocked. **Close (IRL):** the
  maintainer watches a verify gate run the tests (green-advances, red-rolls-back) + a
  forbidden edit blocked; flip the **hooks/rails** + **shell-exec** execution-status rows.

### M13 — Industrialize + ship

- **M13.1 Data-source catalog:** install MCP servers from *within* the app
  (GitHub/Postgres/Slack/Drive/Notion…), reusing the validated import pipeline
  (`ImportPanel.tsx`) + the keychain secret path (`auth_secret_ref`); MCP server as a canvas
  node (`MCPNode` → the builder projection) + MCP **resources** + credentials UX.
- **M13.2 Validated whole-workflow import/export (G5):** route JSON/file/clipboard import
  through `validate_framework`; export to file; never `as Framework` a raw paste.
- **M13.3 Save-path + first-run:** save-companions (#32), budget-persist (#22), template
  (#17); first-run polish; node delete + agent id-rename; the execution-status integrity
  audit (nothing in the UI claims a capability the runtime lacks).
- **Close:** the full **research→PRD→plan→implement→verify** loop builds, runs, and ships;
  the v0.1 success criterion (the author-and-run + verify-loop IRL).

---

## 5. Sequencing & scope (ADR-0032 — decided)

```
M09 ──▶ M10 ──▶ M11 ──▶ M12 ──▶ M13
slice   HITL    sub-    verify   industrialize
        steers  agents  loop     + ship
```

**Vertical, not horizontal** (ADR-0032). Each milestone cuts canvas→engine→run and ships
one capability the maintainer can **author AND run AND IRL-watch** — proving the author→run
integration continuously, never bolting authoring onto a separately-built engine. M09 is the
walking skeleton; M10–M13 each add one runnable capability on top. This **supersedes** the
prior horizontal split (a whole author-anything layer → a whole real-data layer → a whole
execution-breadth layer) **and** the withdrawn **M08.9.1** (wire-all-execution-first against
hand-written JSON) — both were horizontal slicing (the ADR-0032 Alternatives A + B).

**Scope (ADR-0032):**

- **v0.1 = M09 + M10 + M11 + M12 + M13** — the software-development loop
  (research→PRD→plan→implement→verify) **builds, runs, and ships**, industrial-strength:
  multi-agent (sequential), objective verify gates (`bash verify.sh` via the controlled-exec
  sandbox), HITL approval, rails, and gap suspend→resume. Execution breadth is **in** v0.1, not
  deferred.
- **v1.0 = concurrent/parallel multi-agent** (fan-out / agent-pool / teams — the P2–P4
  orchestration model) **+ the ML/data framework** (a structurally identical pipeline, mostly
  tool/skill swaps) **+ the microVM/gVisor sandbox upgrade** (if the threat model ever becomes
  arbitrary untrusted code at scale) **+ Generators** (the LLM build-assist, old M9) + the
  remainder of §0d's v1.0 column.

The shell-exec sandbox (M12.2) is **OS-native** under a **semi-trusted threat model** (your
own framework + own `verify.sh`, local, single-user, no-telemetry) — the correct fit, and
explicitly weaker than microVM for arbitrary untrusted code (ADR-0032; rule 11 — do not
over-claim the isolation).

**Milestone numbering.** M09 (vertical slice) · M10 (HITL-steers) · M11 (sub-agents) · M12
(verify loop + shell exec) · M13 (industrialize + ship). The stage-prompt validator requires
`M\d\d.<X>` (`bin/validate-stage-prompts.mjs`); phase docs authored at Protocol **v1.11+**
carry the explicit two-commit `<execution_steps>` (the v1.11 strict-TDD gate). `docs/MVP-v0.1.md`
is the milestone index; `docs/execution-status.md` tracks the paint→execute flips, now
**per-slice**.

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
