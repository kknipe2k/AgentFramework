# M08 — Workbench (Builder Canvas)

> **Protocol version:** v1.8 (per `STAGE-PROMPT-PROTOCOL.md` v1.8 — the protocol M07 + the M07.5 fix-cycle shipped on; the three authoring-time audit slots `<construction_reachability_check>` / `<wire_signature_audit>` / `<wire_trace_vs_adr_reconcile>` + the `<phase_doc_inventory_audit shape=…>` extension are used where they bite; strict `<tdd_discipline strict="true">` two-commit on every code stage; closeout carries the `<simplify_pass>` (v1.6) + `<coverage_policy_reconciliation>` (v1.8) children; Stage V runs the **mandatory** `--features integration` reference-MCP-server smoke. The v1.9 protocol-iteration candidates the M07 gap-analysis routed forward — `<schema_mirror_ripple_check>`, `<pre_red_mechanical_sweep>`, `<data_flow_audit>`, the environment-precondition check, the carry-forward-convergence line — are **not** adopted here: v1.9 has not landed, M08 is authored under v1.8 like M07.)

## Background and Design Decision

M08 delivers the **Workbench / Builder Canvas** (MVP §M8, weeks 14–17; spec Phase 9 "Visual Canvas and Tester", §9 region ~lines 2495–2543). It is the milestone that makes v0.1 a *workbench* rather than a runtime spectator: a three-panel build-time tool — Palette / React-Flow Canvas / Inspector — that lets a user compose runtime primitives (Tools / Skills / Agents per §0b) visually, validate them continuously against `schemas/*.v1.json`, save a `framework.json` to disk, reload it, and run a sandboxed Tester against it without leaving the app. Per §0d the MVP framing is explicit: *"A novice must be able to build an agentic process from scratch using the Builder Canvas … An experienced user must be able to build something complex and useful (canvas + JSON view + Tester + import-by-URL). Both share one workbench."* M08 is that workbench shell; M09 wires the three Generators into it.

M08 is also the milestone that **absorbs the entire post-M07 carry-forward backlog** — the M07.V verifier's coupled Dec-6 set (🟡 #2/#3/#4/#5), the eight findings of the post-M07.5 gate-7 IRL walk-through (`docs/M07-irl-findings.md`), the M06.5 IRL 🟡-1..4 findings still open two milestones on, and the M04 `plan_loop` driver. The headline reason every one of those routes here: M08 introduces the **production code paths** (a Builder that loads imported artifacts; a Tester that runs a real tool-driving session) that the M07 primitives were built for but had no driver to exercise. M08 is where the construction graph the M07.V Dec-6 findings mapped finally completes.

### What this milestone produces

1. **Builder backend** (Stage B) — `validate_framework` (continuous schema + capability validation in the Rust main process, reusing `crates/runtime-core/src/capability.rs` + the `framework_loader` — spec §9 "no duplication of validation logic between TS and Rust"); framework **save/load** (write `framework.json` + companion `skill.md`/`tool.md`/`agent.md` to a directory; reload — path-agnostic `&Path` persistence per CLAUDE.md §9); a whole-framework **capability summary**; a **`skills.lock` production reader** (the first production consumer — closes M07 IRL #6 + half of M07.V 🟡 #2); capability-**narrowing** exposure for Agent→Agent edges (reuse `capability/narrowing.rs`).
2. **Builder shell + Palette** (Stage C) — the three-panel build-mode view (a new top-level view alongside the live-graph runtime view); the Palette (Tools / Skills / Agents / HITL / Hooks tabs; filterable; drag source); the **local-file picker** (`@tauri-apps/plugin-dialog` — closes M07.V 🟡 #4) wired into both the Builder import affordance and the M07 Import panel.
3. **Builder Canvas — node editor** (Stage D1) — the interactive React-Flow editor: drag a Palette item onto the canvas → instantiate a node; inline node configuration (role / model / `allowed_*`); per-node plain-English capability disclosure (reuse the M05 §8.security L1 disclosure component).
4. **Builder Canvas — edges, narrowing, validation** (Stage D2) — the four edge types (Agent→Skill = `allowed_skills`; Agent→Tool = `allowed_tools`; Agent→Agent = `spawns`; Hook→Task = `post_hooks`); **automatic capability narrowing** on Agent→Agent edges (child `allowed_*` intersected with parent — spec §9 + MVP §M8 criterion 3); continuous schema validation surfaced as **red badges** on offending nodes.
5. **Inspector + canvas↔JSON two-way binding** (Stage E) — the right-panel Inspector: live `framework.json` preview, diff against the file on disk, whole-framework capability summary, **Validate** + **Test** buttons; the Canvas | JSON tab toggle with two-way binding (canvas edits update the JSON; JSON edits re-render the canvas — MVP §M8 criterion 6).
6. **Tester backend** (Stage F1) — `test_framework`: an **isolated test session** with a separate throwaway SQLite database; capability violations surface as **test failures with defaults applied**, not live HITL prompts; no writes to any user data directory; torn down on close. This is the production tool-driving session that **discharges M07.V 🟡 #5** (agent-with-tools production driver), **wires `skills_lock::verify` into the artifact-load path** (🟡 #2), and **wires `McpDispatcher::on_server_connected` into the test session's connect handler** (🟡 #3).
7. **Tester modal** (Stage F2) — the renderer modal opened from the Inspector's Test button: a task-description input, a smaller live-graph pane for the test session, VDR + signals review, token spend + timing, pass/fail with full trace.
8. **Settings panel + tier promotion** (Stage G) — a Settings panel hosting **Novice → Promoted tier promotion** (`request_tier_transition` — the backend command already exists + is unit-tested; M08 ships the missing renderer surface), **closing the M07-IRL #5 🔴-candidate**: in v0.1 today the Promoted tier — a §0d v0.1-scope capability — is **unreachable**, which leaves MCP-server management unreachable. The panel also absorbs M06.5 IRL 🟡-4 (budget settings not state-wired).
9. **Stage V** four-pass verifier (mandatory `--features integration` reference-MCP-server smoke in the Behavior pass).
10. **Closeout** — M08 summary + immutable gap-analysis entry + `<simplify_pass>` + `<coverage_policy_reconciliation>`.

### What's not in scope

- **The three Generators (Phase 8a/b/c — Tool Writer / Skill Writer / Agent Composer).** M09. M08 builds the workbench *shell*; M09 wires the "Generate Tool / Skill / Agent" buttons into the Palette + Canvas. The M08 Palette shows **installed / imported** artifacts only.
- **The Share It module** (rebake / per-OS bundle generation) — v1.0, paired with the headless CLI (MVP §M8 forward declaration). v0.1 export is runtime-to-runtime only; `share_provenance` is already populated + surfaced (M07). The Share It module's natural home is M08's Workbench, but it is explicitly v1.0.
- **Multi-framework comparison view** — v1.0 (MVP §M8 "Out of scope"; spec Phase 9 "v1.0 adds multi-framework comparison").
- **Plugin node types** — v2.0.
- **Operator tier** — v1.0. M08's Settings panel ships Novice ↔ Promoted only (§0d: v0.1 tier system is "Novice + Promoted").
- **Anthropic upstream search UI** — v1.0.
- **§1c multi-session** (concurrent live sessions, drone pool) — ❌ for v0.1 per §0d. The Tester's "isolated session" is a *sequential, throwaway, build-time* test session, **not** the §1c concurrent-multi-session feature — see Key constraints and Stage A's intake (A.3.7).

### Why nine work stages (D split D1/D2, F split F1/F2) + V + closeout

M08 is the largest milestone in the MVP (4 weeks vs the 1–2 of M01–M07). Nine work stages keep each a clean red→green unit at roughly the M06/M07 cadence (~2–3 days each):

- **A** is the canonical carry-forward-absorption + intake stage — it clears the debt backlog and authors the `<construction_reachability_check>` for the three M07.V Dec-6 wires Stage F1 discharges. It ships **no Builder feature code** (the M07.A precedent: A maps, later stages discharge).
- **B** is a pure backend stage (validate / persist / capability-summary / `skills.lock` reader) — the renderer stages C–F consume it; it must land first. Reuses `runtime-core` capability logic + `framework_loader`; CODEOWNERS-adjacent (capability surface) → plan-first.
- **C / D1 / D2 / E** are the Canvas itself: shell+palette, node editor, edge editor + narrowing, inspector + JSON binding. The Canvas is the M08 headline and genuinely large; **D splits D1/D2** (the M05.C1/C2 precedent) at the node/edge boundary — D1 makes nodes reachable on the canvas, D2 connects them and applies narrowing. One D stage would span drag-drop instantiation + inline config + four edge types + the spec-critical capability-narrowing rule + continuous validation — too wide for one coherent red→green unit.
- **F1 / F2** are the Tester. **F splits F1/F2** at the backend/renderer boundary (the M07 C/E precedent). F1 — the isolated-session backend — is novel cross-stack work (a drone-managed throwaway session) **and** discharges three M07.V carry-forwards in one backend surface; F2 is the modal. Bundling them puts two crates + the renderer in one red→green unit. The cross-stack-integration risk (CLAUDE.md §7 — escalate at iteration 2) concentrates in F1; isolating it is deliberate.
- **G** (Settings + tier promotion) is independent of the Canvas — it closes the M07-IRL #5 🔴-candidate (a v0.1-scope unblock) and touches the capability/tier surface (CODEOWNERS-adjacent → plan-first). It is its own stage because #5 is severity-🔴-candidate and the tier surface deserves focused treatment, not a fold into the shell stage.

Bundling any pair would muddy a gate boundary (B's backend ≥95 vs the renderer ≥80; F1's CODEOWNERS sandbox surface vs F2's modal) or conflate "compose a framework" with "execute a test session". The split gives each stage a single coherent contract.

### Carry-forward absorbed

M08 absorbs the post-M07 backlog. Routing is from `docs/M07-irl-findings.md` (Disposition section), `docs/build-prompts/retrospectives/M07.V-retrospective.md` (`[END] Decisions`), and the `docs/gap-analysis.md` M07 entry + M07.V carry-forward section.

| Carry-forward | Source | M08 stage |
|---|---|---|
| **M07.V 🟡 #2** — `skills_lock::verify` has no production load-path caller | M07.V Dec-2 | **A** maps (`<construction_reachability_check>`); **F1** discharges (verify in the Tester's artifact-load path); **B** ships the `skills.lock` reader |
| **M07.V 🟡 #3** — `McpDispatcher::on_server_connected` has no production connect-handler caller | M07.V Dec-3 | **A** maps; **F1** discharges (the Tester's MCP connect handler) |
| **M07.V 🟡 #4** — local-file picker UI not shipped (MVP §M7 acceptance criterion) | M07.V Dec-4 | **C** (`@tauri-apps/plugin-dialog` — Builder + M07 Import panel) |
| **M07.V 🟡 #5** — agent-with-tools production driver absent | M07.V Dec-5 | **A** maps; **F1** discharges (the Tester *is* the production tool-driving session) |
| **M07-IRL #5 🔴-candidate** — no tier-promotion UI; Promoted tier + MCP management unreachable | `M07-irl-findings.md` | **A** dispositions (intake); **G** discharges (Settings panel) |
| **M07-IRL #2 🟡** — token in/out breakdown not populated (`tokensIn:0, tokensOut:0, tokensTotal:34`) | `M07-irl-findings.md` | **A** |
| **M07-IRL #3 🟡** — import UI text near-invisible (contrast vs theme variable) | `M07-irl-findings.md` | **A** |
| **M07-IRL #6 🟡** — Import panel does not reload installed artifacts after restart (no `skills.lock` reader) | `M07-irl-findings.md` | **B** (the reader) + **C** (Import panel + Palette consume it) — converges with 🟡 #2 |
| **M07-IRL #7 🟡** — API key does not persist across an app restart | `M07-irl-findings.md` | **A** |
| **M06.5 IRL 🟡-1** — HITL `ui_variant` not honored | M07 gap-analysis Carry-forward | **A** |
| **M06.5 IRL 🟡-2** — `npx` Windows `.cmd` shim | M07 gap-analysis Carry-forward | **A** |
| **M06.5 IRL 🟡-3** — stale Test error banner | M07 gap-analysis Carry-forward | **A** |
| **M06.5 IRL 🟡-4** — budget settings not state-wired | M07 gap-analysis Carry-forward | **G** (the Settings panel is the natural home for budget config) |
| **M04 🟡** — `plan_loop.rs` driver unwired (`commands.rs` documents the deferral) | M07 gap-analysis Carry-forward | **A** |

The M07-IRL 🟢 findings (#1 smoke too fast, #4 no bundled example artifact, #8 minimap blank) routed to `docs/tech-debt.md`, **not** M08 — they are not stages here. M06.5 IRL 🔴-1 (MCP-registry real-app re-confirm) "remains deferred, gated on #5"; once Stage G makes Promoted reachable, MCP-add becomes reachable and 🔴-1 re-confirms in the post-M08 IRL pass — the closeout records this.

### Key constraints

- **v1.8 protocol.** Stage prompts use the v1.8 audit slots where they bite. `<construction_reachability_check>`: Stage A maps the three M07.V Dec-6 wires (`skills_lock::verify`, `McpDispatcher::on_server_connected`, the agent-with-tools loop driver) `inputs_reachable="false"`; Stage F1 inverts each to `true` with file:line — the A→F1 chain documents the construction graph completing, exactly as M07's A→D1→D2 chain did for ADR-0011. `<wire_signature_audit>` + `<phase_doc_inventory_audit shape=…>`: every renderer stage (C, D1, D2, E, F2, G) pins the actual Tauri command params + store-slot TS types **before** authoring component pseudocode (the M06.E / M07.E lesson).
- **Schema-as-source-of-truth (CLAUDE.md §14, Hard Rule 5).** M08 consumes `schemas/{framework,skill,tool,agent,common}.v1.json`; the Canvas's output validates against them with the same checker CI runs. M08 is **not expected to add a schema** — if a stage needs a new event variant (e.g. for test-session scoping), it is authored in `schemas/event.v1.json`, types regenerated via `cargo xtask regenerate-types`, ADR filed. Each schema-touching stage carries a `<schema_audit>` / `<schema_ref_audit>` so this is decided at pre-flight, not mid-implementation.
- **Reuse, don't rebuild.** Validation = `runtime-core` capability + the schema validator + `framework_loader` (do not write a second validator in TS — spec §9). Capability narrowing = `capability/narrowing.rs` (M05.B L2a). Capability disclosure = the M05 §8.security L1 plain-English component (M07.E reused it; M08 reuses it again). The Tester's session = the existing smoke-session infra (`run_smoke_session` / `drone_lifecycle.rs` / `sdk/agent_sdk.rs`) pointed at a throwaway DB — not a new session engine. The Tester's smaller graph pane = the existing React-Flow live-graph rendering. Tier promotion = the existing `request_tier_transition` command. **The MVP §M8 Tester runs the *same* `AgentSdk::run_agent` multi-turn loop M07.D2 shipped — that loop is ready; M08 supplies the missing production trigger.**
- **The Tester is a build-time, throwaway, sequential session — not §1c multi-session.** Spec Phase 9 says "drone-managed sandbox per §1c", but §0d marks §1c multi-session ❌ for v0.1. These are reconcilable: the Tester needs *an* isolated session (its own throwaway SQLite; capability violations → test failures; results discarded on close), run from build mode where no live runtime session is executing — it does **not** need the §1c concurrent-session pool. Stage A's intake (A.3.7) surfaces this reading for the maintainer to confirm, and the closeout's gap-analysis records the spec-clarification.
- **No `unsafe` outside `runtime-sandbox`** (Hard Rule 7). **No telemetry** (Hard Rule 4) — the Builder reads/writes only local files the user chooses; the Tester hits only the Anthropic endpoint the live runtime already uses; no phone-home.
- **CODEOWNERS-flagged paths** (Hard Rule 8, CLAUDE.md §10): Stage B reads the capability surface; Stage F1 touches the sandbox/isolation boundary + capability-enforcement behavior (test-defaults for violations); Stage G touches the tier surface. Each surfaces its plan first — the `<construction_reachability_check>` / pre-flight plan **is** that plan.
- **No Co-Authored-By; DCO `-s`; session-URL footer.** Strict v1.8 two-commit `<tdd_discipline strict="true">` on every code stage (A–G; renderer stages per the v1.7 default).
- **Windows is the v0.1 target; CI runs all three OSes.** Renderer E2E is Playwright against the Vite dev server with `@tauri-apps/api` module-mocked (gotcha #23 — Playwright cannot drive the Tauri window; full desktop-shell E2E stays the documented carry-forward). React-Flow drag-drop + edge-creation are exercised through Playwright's drag API against the dev server.

## Document Structure

| Stage | Scope | Strict TDD | Effort | Coverage gate |
|---|---|---|---|---|
| **A** | Carry-forward absorption + intake: M04 `plan_loop` driver; M07-IRL #2/#3/#7; M06.5 IRL 🟡-1/-2/-3; #5 disposition; the §1c-Tester scope clarification; the `<construction_reachability_check>` for the three M07.V Dec-6 wires | yes | 6–9 h | workspace ≥80; maintain runtime-main ≥95 |
| **B** | Builder backend: `validate_framework` + framework save/load + capability summary + `skills.lock` production reader + narrowing exposure | yes (v1.8 two-commit) | 8–11 h | runtime-main ≥95 on the new `builder` module; workspace ≥80 |
| **C** | Builder shell (3-panel build mode + view switch) + Palette (5 tabs, filterable, drag source) + local-file picker (`@tauri-apps/plugin-dialog`) | yes (v1.7 default) | 9–12 h | renderer ≥80 (vitest) |
| **D1** | Builder Canvas — node editor: drag-drop instantiation + inline node config + per-node capability disclosure | yes (v1.7 default) | 10–13 h | renderer ≥80 |
| **D2** | Builder Canvas — edges (4 types) + Agent→Agent capability narrowing + continuous validation red badges | yes (v1.7 default) | 10–13 h | renderer ≥80 |
| **E** | Inspector (preview + disk-diff + capability summary + Validate/Test buttons) + canvas↔JSON two-way binding | yes (v1.7 default) | 9–12 h | renderer ≥80 |
| **F1** | Tester backend: isolated test session (throwaway SQLite) + discharge M07.V 🟡 #2 (`skills_lock::verify` on load) + #3 (`on_server_connected`) + #5 (agent-with-tools production driver) | yes (v1.8 two-commit) | 9–12 h | runtime-main ≥95 on the Tester module |
| **F2** | Tester modal renderer: task input + smaller graph pane + VDR/signals + token/timing + pass/fail trace | yes (v1.7 default) | 8–11 h | renderer ≥80 |
| **G** | Settings panel + Novice↔Promoted tier promotion (M07-IRL #5) + budget settings state-wiring (M06.5 IRL 🟡-4) | yes (v1.7 default) | 6–9 h | renderer ≥80 |
| **V** | Verifier — four passes; **mandatory `--features integration` reference-MCP-server smoke** | n/a | 3–5 h | n/a |
| **Closeout** | gap-analysis entry + M08 summary + `<simplify_pass>` + `<coverage_policy_reconciliation>` | n/a | 4–6 h | n/a |

Total ~82–113 h estimated; at M06/M07's converging ~0.7–0.75× calibration, ~58–85 h actual. Stage V is "Stage V"; the closeout is **Stage H** (M08 uses work-stage letters A–G with D and F split; the closeout takes the next free letter — there is no rule fixing the closeout to "G").

## Implementation Workflow

Project-wide protocol (CLAUDE.md §3–§6, §8, §16, §19; not restated per stage):

1. **Read first** — each stage's `<read_first>`; Stage B+ reads the prior stage's retrospective `[END] Decisions` and applies them (CLAUDE.md §19 rule 1).
2. **Strict v1.8 two-commit TDD** on every code stage: failing tests → standalone `test(M08.X): …` commit → red-phase surface → impl WITHOUT touching test files → gate ordering → impl commit whose body proves `git diff <red>..<impl> -- '**/tests/**'` EMPTY → final surface. Net-new / mechanical test-file changes go in a separate labelled follow-up commit. The binary-crate scoped-diff variant (in-source `#[cfg(test)]` block byte-identical red→impl) applies to `src-tauri/src/` work (the M06.5.A.fix precedent).
3. **Schema-as-source-of-truth** — any schema edit → `cargo xtask regenerate-types` → commit generated types with the schema (CLAUDE.md §14).
4. **v1.6 canonical gate ordering** (CLAUDE.md §6): `cargo fmt --all` → `cargo clippy --fix --allow-dirty -p <crate>` → `cargo clippy --workspace --all-targets -- -D warnings` → test/doc/audit/deny → `cargo llvm-cov clean --workspace` (gotcha #81) → the llvm-cov gates → frontend gates. CI-parity is a hard rule; cite any divergence inline with a gotcha reference.
5. **Surface, don't commit, until approved** (Hard Rule 1). Every stage surface includes cross-machine state: `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M08.*-retrospective.md` (CLAUDE.md §19 rule 7). Do not push between stages.
6. **Stage V in a fresh CLI session** (the bias guard); V's `<read_first>` deliberately omits prior retros / summary / gap-analysis but DOES consume the phase doc's `<construction_reachability_check>` / `<wire_trace_vs_adr_reconcile>` / `<scope_change>` blocks (v1.8 STAGE-V template).
7. **Closeout** runs `<simplify_pass>` (M08.A..HEAD diff) + the v1.8 `<coverage_policy_reconciliation>` (sync `docs/coverage-policy.md` §B/§C + CLAUDE.md §5/§6 + `codecov.yml` for any gate change — esp. the new `builder` / Tester module gates + any new OS-call-holdout exclusion).
8. **ADRs.** Two are expected (CLAUDE.md §11) — **ADR-0019** (Tester isolated-session model — Stage F1; sandbox/isolation boundary + capability-enforcement behavior) and **ADR-0020** (Builder canvas↔`framework.json` state model — Stage C/D1; `framework.json` as source of truth, the canvas as a projection). Both filed at their stage, `Proposed → Accepted` in the M08 PR before merge. Additional ADRs per §11 if a stage surfaces a trigger (a new schema, an IPC-protocol change).

---

## Pre-existing legacy file inventory

Grep-verified at authoring time against `origin/main` at `f5eac40` (post-M07.5 PR #90 + the M07-IRL PR #91 merges). Files M08 stages CONSUME or REFERENCE; shape claims are factual as of this snapshot. Renderer stages MUST re-pin every shape via the v1.8 `<wire_signature_audit>` / `<phase_doc_inventory_audit shape=…>` slots against the code shipped by the *immediately prior* M08 stage.

| File | Purpose | M08 stage that touches it |
|---|---|---|
| `src/App.tsx` | App root; flat layout (`graph-layout` div: `GraphCanvas` + panels); `subscribeAgentEvents` wiring; `localStorage` `lastSessionId` | C (adds the build-mode view switch; the Builder is a new top-level view, the live graph is unchanged) |
| `src/components/GraphCanvas.tsx` | Read-only live-execution graph; module-level `nodeTypes` (11 entries); dagre layout; `MiniMap`/`Controls` | C/D1 (the Builder Canvas is a NEW interactive component; it reuses React Flow + the §3 node visual conventions, not this read-only component) |
| `src/components/nodes/*.tsx` | 11 live-graph node components + `CapabilityBadge.tsx` | D1 (the Builder nodes are interactive/inline-editable — reuse the visual CSS; the build pins reuse-vs-new per node via `<existing_pattern_audit>`) |
| `src/lib/graphStore.ts` | Zustand store for **live-execution** graph state (`applyEvent` reducer over ~34 `AgentEvent` variants; `currentTier`, `currentMcpServers`, `imports` persistent slots) | A (M07-IRL #2 token in/out; M06.5 🟡-1/-3); C+ (the Builder store is SEPARATE — `src/lib/builderStore.ts`, new — do not overload `graphStore`) |
| `src/lib/ipc.ts` | Tauri IPC wrappers + `subscribeAgentEvents` + `unwrapCmdError`; current commands incl. `importArtifact` / `completeImportArtifact` / `cancelPendingImport` (M07.5) | B–G (new wrappers: `validateFramework`, `saveFramework`, `loadFramework`, `testFramework`, `requestTierTransition`, the file picker; params PINNED via `<wire_signature_audit>`) |
| `src/lib/layout.ts` | dagre layout (`layoutGraph`, pure) | D1 (the Builder Canvas may reuse it for an auto-layout affordance; user-placed nodes keep manual positions) |
| `src/components/ImportPanel.tsx` | M07.5 Import panel (URL → tier-gate review → install; `ImportOutcome` discriminated on `status`) | C (the file picker is wired in here too — M07.V 🟡 #4; the panel reads `skills.lock` on startup — M07-IRL #6) |
| `src/components/MCPServerSettings.tsx` + `MCPServerAddModal.tsx` | M06/M07 MCP panels; Add is correctly tier-blocked for Novice | G (Promoted tier becomes reachable → MCP-add becomes reachable; G does not rebuild these) |
| `src/styles.css` | Theme CSS variables (`--node-fg`, `--node-fg-muted`, `--node-bg`, …) + all node styles | A (M07-IRL #3 contrast fix — one CSS root cause); C–G (Builder panel styles) |
| `src/types/*.ts` | Schema-generated TS types (`agent_event.ts`, `mcp.ts`, `capability.ts`, …) | All renderer stages (consume; regenerated if a schema changes) |
| `crates/runtime-main/src/framework_loader/` | `mod.rs` + `walker.rs` + `capability_map.rs` + `error.rs` — Phase 6 framework loader + the per-framework capability map | B (reuse for `validate_framework` + the capability summary — `capability_map.rs` is the summary's basis); F1 (the Tester loads the framework through it) |
| `crates/runtime-core/src/capability.rs` | Capability primitive — the validation logic spec §9 forbids duplicating in TS | B (reuse for `validate_framework`) |
| `crates/runtime-main/src/capability/narrowing.rs` | L2a `narrow()` (M05.B) — child `allowed_*` ∩ parent | B (expose for the Builder); D2 (Agent→Agent edge narrowing surfaces it) |
| `crates/runtime-main/src/skills_lock/` | `skills.lock` integrity primitive (M07.B; `write_entry` / `verify` / canonical serialization) | B (the production reader — `verify` gets its first non-test caller path here + at F1) |
| `crates/runtime-main/src/import/` | M07/M07.5 import pipeline (`import_artifact_with` → `ImportOutcome::{Installed,Pending}`; `commit_import`; `complete_import_with`; `PendingImportState`) | F1 (the Tester's artifact-load path calls `skills_lock::verify` — M07.V 🟡 #2) |
| `crates/runtime-mcp/src/dispatch.rs` | `McpDispatcher` incl. `on_server_connected` / `on_server_disconnected` (M07.D1; spec §5a re-resolution) — no production connect-handler caller | F1 (the Tester's MCP connect handler is the production caller — M07.V 🟡 #3) |
| `crates/runtime-main/src/sdk/agent_sdk.rs` | `AgentSdk::run_agent` — the multi-turn agent-with-tools loop (M07.D2); `with_mcp_dispatch` | F1 (the Tester runs a tool-bearing framework through it — the production driver, M07.V 🟡 #5) |
| `src-tauri/src/commands.rs` | Tauri command surface; `run_smoke_session` / `import_artifact` / `complete_import_artifact` / `request_tier_transition` (`+ _with` seam) | B/F1 (new `validate_framework`/`save_framework`/`load_framework`/`test_framework` commands); G (renderer wires the existing `request_tier_transition`) |
| `src-tauri/src/drone_lifecycle.rs` | Drone construction + the M06.5.B `sdk_session_id` sharing | F1 (the Tester spawns a drone against a throwaway DB path) |
| `crates/runtime-main/src/plan/` (`plan_loop.rs`) | M04 🟡 driver unwired (`commands.rs` documents the deferral) | A (wire the driver into the session entrypoint) |
| `crates/runtime-main/src/key_store.rs` | OS-keychain key storage | A (M07-IRL #7 — key does not persist across restart; diagnose write-fail vs read-fail) |
| `schemas/{framework,skill,tool,agent,common}.v1.json` | The artifact schemas the Canvas validates against + generates | B (validation targets); D1/D2/E (the canvas↔JSON model is the generated `framework.ts` shape) |
| `docs/M07-irl-findings.md` | The post-M07.5 gate-7 IRL walk-through — the carry-forward routing | A (intake — the disposition source) |
| `docs/build-prompts/retrospectives/M07.V-retrospective.md` | M07.V `[END] Decisions` 2/3/4/5 — the coupled Dec-6 carry-forward set | A (`<read_prior_milestones>`) |
| `docs/MVP-v0.1.md` §M8 + `agent-runtime-spec.md` Phase 9 (~2495–2543) + §0–§0d / §3 / §3a / §8.security | Acceptance criteria + the spec sections M08 implements | All stages (V later checks against them) |
| `docs/coverage-policy.md` (+ CLAUDE.md §5/§6, `codecov.yml`) | The four-mirror coverage source-of-truth | B/F1 (new module gates) + closeout (`<coverage_policy_reconciliation>`) |

---

## Stage A — Carry-forward absorption + intake + construction-graph groundwork

### A.1 Problem Statement

M08 absorbs the entire post-M07 carry-forward backlog (see Background — "Carry-forward absorbed"). Stage A is the canonical **absorption + intake** stage: it clears the **debt** items (localized fixes, not Builder features), **dispositions** the items that belong in a feature stage, and **maps** the construction graph the Tester (Stage F1) discharges. This is the M07.A precedent — A absorbs and maps; later stages discharge. **A ships no Builder Canvas feature code** (B–G build the workbench shell).

The headline reason this stage exists as a discrete unit: the post-M07 backlog has accumulated three milestones deep (M04 `plan_loop`, M06.5 IRL 🟡-1..4, M07-IRL #2/#3/#5/#6/#7, M07.V Dec-6 🟡 #2/#3/#4/#5) and folding it into a feature stage would muddy that stage's red→green contract. Absorbing it first means C–G author cleanly against a debt-free `main`. The construction-graph half — the `<construction_reachability_check>` for the three M07.V Dec-6 wires — is load-bearing input to Stage F1's design: F1 cannot be authored until A has stated which production drivers are absent on `main` and confirmed F1 owns each discharge.

**Concrete deliverables:**

1. **`crates/runtime-main/src/plan/plan_loop.rs`** (new) — `drive_plan(plan: &mut PlanState, hitl: &HitlSeam, emit: &impl Fn(AgentEvent)) -> Result<(), PlanLoopError>`: the M04 🟡 plan-driver shell. Walks the `PlanStateMachine` (M04's `state_machine.rs`) through `PendingApproval → Approved → InProgress → Complete`, routing approval through the existing `HitlSeam`. `plan/mod.rs` adds `pub mod plan_loop;`. Task *execution* stays `AgentSdk::run_agent` (M07.D2) — A wires the driver shell only.
2. **`src/lib/graphStore.ts`** — the M07-IRL #2 fix: the `token_usage` reducer case (currently a graph-no-op at the `case 'token_usage':` arm) populates the running agent node's `tokensIn`/`tokensOut`; `agent_complete` keeps setting `tokensTotal`. The agent-node inspector reads all three.
3. **`crates/runtime-main/src/key_store.rs` + `src-tauri/src/commands.rs`** — the M07-IRL #7 fix: a `has_api_key()` seam + a `has_api_key` Tauri command so `src/App.tsx` pre-populates `hasKey` on launch (the diagnosed root cause — `App.tsx:51` hardcodes `useState(false)`, no startup read exists). If the diagnosis instead surfaces a Windows Credential Manager write-fail, the fix moves to the write path; A pins which.
4. **`src/styles.css`** — the M07-IRL #3 fix: the `.import-panel` / `.import-row` / `.import-review-modal` rules set `background` but never `color` (root cause confirmed at `styles.css:1130`); add `color: var(--node-fg)` / `var(--node-fg-muted)` to the import-component selectors.
5. **`src/lib/graphStore.ts` (HITL reducer) / `src/components/HITL*.tsx`** — the M06.5 IRL 🟡-1 disposition: the three HITL components already branch on `uiVariant` and the reducer already maps `event.ui_variant`; A pins whether the finding is a still-live drop or already-closed, and adds the regression test that proves the variant is honored end-to-end.
6. **`crates/runtime-mcp/src/transport/stdio.rs`** — the M06.5 IRL 🟡-2 fix: `build_command()` (the existing pure seam at `stdio.rs:80`) resolves `npx` → `npx.cmd` under `cfg(target_os = "windows")`.
7. **`src/components/SmokeButton.tsx` / `src/App.tsx`** — the M06.5 IRL 🟡-3 fix: clear the stale Test error banner at run start (`App.tsx:95` already calls `setError(null)` in `handleSmoke` — A pins whether the stale render is that path or a separate one and fixes the real binding).
8. **The `<construction_reachability_check>` map** — A.5's prompt carries the three M07.V Dec-6 wires (`skills_lock::verify`, `McpDispatcher::on_server_connected`, the agent-with-tools loop driver) `inputs_reachable="false"`; Stage F1 inverts each to `true` with file:line. Plus two non-code intake items: the M07-IRL #5 → Stage G disposition (A.3.6) and the §1c-vs-§0d Tester-scope clarification (A.3.7).

**Not in this stage:**

- Any Builder Canvas / Palette / Inspector / Tester code (Stages B–G).
- The Settings panel or tier-promotion UI — M07-IRL #5 is *dispositioned* to Stage G here, not built (A.3.6).
- `skills_lock::verify` wired into a production load path, or `McpDispatcher::on_server_connected` given a production caller — A *maps* these; **Stage F1 discharges** them.
- M06.5 IRL 🟡-4 (budget settings state-wiring) — *routed* to Stage G (the Settings panel is its natural home).
- Any schema change. If the token-split diagnosis (A.3.2) is found to need an `event.v1.json` edit, that triggers the §14 schema-regenerate-ADR flow — surface before the red commit.

### A.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/plan/plan_loop.rs` | **new** | M04 🟡: the `drive_plan` driver shell — walks `PlanStateMachine` through approval + ready-task iteration, routes approval through `HitlSeam`. |
| `crates/runtime-main/src/plan/mod.rs` | exists | Edit: `pub mod plan_loop;` + re-export `drive_plan` / `PlanLoopError`. |
| `src-tauri/src/commands.rs` | exists | M07-IRL #7: add `has_api_key` + `has_api_key_with` (the `*_with` seam); wire the M04 `plan_loop` driver reference at the documented deferral site (`commands.rs:403`/`2128`). |
| `crates/runtime-main/src/key_store.rs` | exists | M07-IRL #7: add `has_api_key() -> bool` (a `read_api_key().is_ok()` wrapper, NotFound → false). |
| `src/lib/graphStore.ts` | exists | M07-IRL #2: the `token_usage` reducer case populates the running agent node's `tokensIn`/`tokensOut`. M06.5 🟡-1: confirm/repair the `hitl_requested` `uiVariant` mapping. |
| `src/App.tsx` | exists | M07-IRL #7: call `invokeHasApiKey()` on mount, seed `hasKey`. M06.5 🟡-3: confirm the stale-banner binding. |
| `src/lib/ipc.ts` | exists | M07-IRL #7: add the `invokeHasApiKey()` wrapper. |
| `src/styles.css` | exists | M07-IRL #3: add explicit `color` tokens to the `.import-*` selectors (one CSS root cause). |
| `src/components/HITLModal.tsx` / `HITLPanel.tsx` / `HITLToast.tsx` | exists | M06.5 🟡-1: repair only if the diagnosis finds a live drop (all three already branch on `uiVariant`). |
| `src/components/SmokeButton.tsx` | exists | M06.5 🟡-3: repair only if the stale-banner binding is here rather than `App.tsx`. |
| `crates/runtime-mcp/src/transport/stdio.rs` | exists | M06.5 🟡-2: `build_command()` resolves `npx` → `npx.cmd` under `cfg(target_os = "windows")`. |
| `CHANGELOG.md` | exists | `[Unreleased]` Stage A entries. |
| `docs/build-prompts/retrospectives/M08.A-retrospective.md` | **new** | Stage A retrospective. |

Effort budget: ~6–9 hours. The largest single piece is the `plan_loop` driver shell + its behavior tests; the remaining items are localized fixes whose cost is concentrated in *diagnosis* (pinning the drop point) rather than code volume. A is itemcount-dense by nature (an absorption stage; M07.A was comparable) — if the red-phase commit grows past one coherent unit, surface for an A1/A2 split (the M04.A1/A2 precedent: backend items — `plan_loop`, `has_api_key`, `npx` — in one; renderer items — token split, contrast, HITL `uiVariant`, stale banner — in the other). Default: one stage.

### A.3 Detailed Changes

#### A.3.1 M04 🟡 — `plan_loop.rs` driver shell

`crates/runtime-main/src/plan/` ships the plan state machine (`state_machine.rs` — `PlanStateMachine` / `TaskStateMachine`, the M04 Stage B safety primitive at ≥95%). No driver loop consumes it in the production session path — M04 🟡, deferred again at M07; `src-tauri/src/commands.rs` documents the deferral at two sites (`commands.rs:403` in the `approve_plan` doc comment, `commands.rs:2128` in the `approve_plan_with_no_pending_awaiter` test rationale). After Stage A, a `plan_loop` module exists and drives the FSM. The driver shell is small — task *execution* is the existing `AgentSdk::run_agent` loop (M07.D2); A wires the shell that advances the FSM and routes plan approval through the in-process `HitlSeam` (ADR-0007):

```rust
// crates/runtime-main/src/plan/plan_loop.rs — the M04 🟡 driver shell.
//! Plan-driver loop — spec §3a. Walks a `PlanStateMachine` from
//! `PendingApproval` through `Complete`, routing the approval gate
//! through the in-process `HitlSeam` (ADR-0007). Task *execution* is
//! `AgentSdk::run_agent` (M07.D2); this module is the driver shell that
//! advances the FSM and emits the plan-lifecycle `AgentEvent`s.

use runtime_core::event::AgentEvent;
use thiserror::Error;

use crate::hitl::{HitlDecision, HitlSeam};   // pin the real HitlSeam path via <phase_doc_inventory_audit>
use crate::plan::state_machine::{PlanEvent, PlanState, PlanStateMachine, TransitionError};

/// Failure modes raised by [`drive_plan`].
#[derive(Debug, Error)]
pub enum PlanLoopError {
    /// The FSM rejected a transition the driver attempted — a driver
    /// bug (the driver must only emit legal (status, event) pairs).
    #[error(transparent)]
    Transition(#[from] TransitionError),
}

/// Drive `plan` through approval + execution.
///
/// 1. If the plan is `PendingApproval`, await the HITL approval gate;
///    `Reject` aborts the plan (FSM `Aborted`), `Approve` advances it.
/// 2. Transition `Approved → InProgress` (FSM `Started`).
/// 3. Emit `PlanStarted`; task execution is `AgentSdk::run_agent`.
/// 4. Transition `InProgress → Complete` when execution returns.
///
/// # Errors
///
/// [`PlanLoopError::Transition`] if the FSM rejects a driver-issued
/// transition (indicates a driver bug, not a user-facing condition).
pub async fn drive_plan(
    plan: &mut PlanState,
    hitl: &HitlSeam,
    emit: &impl Fn(AgentEvent),
) -> Result<(), PlanLoopError> {
    // 1. Approval gate — only when the plan was created approval-required.
    if matches!(plan.status, /* PlanStatus::PendingApproval */ _) {
        match hitl.await_decision(/* approval prompt for plan.plan_id */).await {
            HitlDecision::Approve => PlanStateMachine::transition(plan, PlanEvent::Approved)?,
            HitlDecision::Reject => {
                PlanStateMachine::transition(plan, PlanEvent::Aborted)?;
                return Ok(());
            }
        }
    }
    // 2. Approved → InProgress.
    PlanStateMachine::transition(plan, PlanEvent::Started)?;
    emit(AgentEvent::PlanStarted { plan_id: plan.plan_id.clone() /* … */ });
    // 3. Task execution is AgentSdk::run_agent (M07.D2) — driver shell only.
    // 4. InProgress → Complete.
    PlanStateMachine::transition(plan, PlanEvent::Complete)?;
    Ok(())
}
```

`PlanState` is `{ plan_id: String, status: PlanStatus }` (`state_machine.rs:74`); `PlanStateMachine::transition(state, event)` is the static, pure-logic transition validator (`state_machine.rs:110`) — the driver issues only legal pairs, so a `TransitionError` here is a driver bug, surfaced as `PlanLoopError::Transition`. **The M04 phase-doc premise that `HitlContext::BudgetThreshold` needed renaming to `BudgetWarn` was found factually false at M07.A** (it is already a distinct, correct variant) — do not re-touch it. The exact `HitlSeam` / `HitlDecision` paths + the `AgentEvent::PlanStarted` field set must be pinned via `<phase_doc_inventory_audit>` before authoring — the shape above is illustrative.

#### A.3.2 M07-IRL #2 — token in/out breakdown not populated

The agent-node inspector after a smoke run shows `tokensIn: 0, tokensOut: 0, tokensTotal: 34` — a total with no components, internally inconsistent. The drop point is **pinned** (this stage's diagnosis is complete; the build confirms it):

- The SDK **correctly** emits `AgentEvent::TokenUsage { input, output, model, cost_usd }` — `event_pipeline.rs:157` maps `ProviderEvent::Usage` to it verbatim.
- The drone **correctly** projects it — `token_usage.rs:80` `project_signal` reads `input` / `output` from the payload into the `token_usage` table.
- The renderer **drops it**: `graphStore.ts:1680` lists `token_usage` among the graph-no-op variants (`case 'token_usage': return state;`), and `agent_complete` (`graphStore.ts:829`) only sets `tokensTotal` — `tokensIn`/`tokensOut` on the agent node are never written from the wire.

The fix is renderer-side: the `token_usage` reducer case attributes the input/output to the running agent node. `AgentEvent::TokenUsage` carries **no `agent_id`** (confirmed against `schemas/event.v1.json:850` — `required: [type, input, output, model, cost_usd]`), so the reducer attributes the usage to the single running agent (mirroring how `tool_result` attributes by the active call):

```typescript
// src/lib/graphStore.ts — replace the `token_usage` no-op arm.
case 'token_usage': {
  // TokenUsage carries no agent_id (event.v1.json) — attribute to the
  // running agent, the same single-active-agent assumption tool_result
  // already relies on. Accumulates: a multi-turn loop emits one
  // token_usage per turn.
  const runningAgent = state.nodes.find(
    (n) => n.type === 'agent' && n.data.status === 'active',
  );
  if (runningAgent === undefined) {
    return state;
  }
  return {
    ...state,
    nodes: state.nodes.map((n) =>
      n.id === runningAgent.id && n.type === 'agent'
        ? {
            ...n,
            data: {
              ...n.data,
              tokensIn: n.data.tokensIn + event.input,
              tokensOut: n.data.tokensOut + event.output,
            },
          }
        : n,
    ),
  };
}
```

The test lands at the pinned drop boundary — a **vitest** reducer test (the drop is the renderer read, not the Rust mapping/projection, both of which the M07.D2 assembled regression already pins). Acceptance: a run that produces a non-zero total produces a consistent non-zero `tokensIn + tokensOut` split, and `tokensIn + tokensOut` does not contradict `tokensTotal`.

#### A.3.3 M07-IRL #7 — API key does not persist across an app restart

A key entered in a session works ("✓ stored in OS keychain"; smoke run succeeds) but after an app restart the app comes up first-start (empty key field, smoke button disabled). The session DB and `skills.lock` persist across the same restart — only the key field does not. **Root cause is pinned: there is no startup key-presence read.** `key_store.rs` exposes `read_api_key` / `write_api_key` / `delete_api_key` only — no `has_*`. `src-tauri/src/commands.rs` has no `has_api_key` command. `src/App.tsx:51` hardcodes `const [hasKey, setHasKey] = useState(false)` and only flips it to `true` inside `handleSetKey` — nothing reads the keychain at launch. The keychain *write* is fine (the key is genuinely stored); the renderer simply never asks.

The fix adds the missing read path:

```rust
// crates/runtime-main/src/key_store.rs — a presence check.
/// Whether an Anthropic API key is present in the OS keychain.
///
/// `read_api_key().is_ok()`: `NotFound` → `false`, a backend error
/// → `false` (the renderer treats "can't tell" the same as "absent" —
/// the user can re-enter the key; a launch must not 500 on a locked
/// keychain).
#[must_use]
pub fn has_api_key() -> bool {
    read_api_key().is_ok()
}
```

```rust
// src-tauri/src/commands.rs — the thin command + its *_with seam.
/// Whether a key is in the keychain — drives the renderer's `hasKey`
/// at launch so a previously-entered key survives an app restart.
#[tauri::command]
pub async fn has_api_key() -> Result<bool, CmdError> {
    has_api_key_with(runtime_main::key_store::has_api_key)
}

/// Test-seam for [`has_api_key`] (CLAUDE.md §5 `*_with` archetype).
pub fn has_api_key_with(probe: impl Fn() -> bool) -> Result<bool, CmdError> {
    Ok(probe())
}
```

`src/App.tsx` calls `invokeHasApiKey()` in the mount `useEffect` and seeds `hasKey`. Stage A runs on Windows (the v0.1 target); if a Windows Credential Manager check during the build instead reveals a *write*-side failure, the build re-pins the root cause to the write path and fixes there — but the diagnosis as authored points squarely at the absent startup read. The regression test lands at the pinned boundary (`has_api_key_with` unit test for the seam; a vitest test that `App` calls `invokeHasApiKey` on mount and disables the smoke button accordingly). Acceptance: a key entered once survives an app restart — the next launch comes up key-present.

#### A.3.4 M07-IRL #3 — import-panel contrast

The Import-panel header, the tier-gate review-modal header, and the installed-artifact row name render with text colour ≈ the dark background. Root cause is **pinned**: `styles.css:1130` `.import-panel` sets `background: var(--node-bg, #1c1c1c)` but no `color`; `.import-panel__title` (`styles.css:1142`), `.import-row__ref` (`styles.css:1189`), and the `.import-review-modal__*` heading selectors set no `color` either — so the text inherits whatever ambient colour the cascade supplies, which against the dark panel background is near-invisible. The M06 MCP Add modal renders fine because its container sets an explicit `color`. This is one CSS root cause across the M07.E import components.

The fix sets the theme tokens explicitly:

```css
/* src/styles.css — the M07.E import components inherit no foreground;
   pin them to the theme variables (one CSS root cause — M07-IRL #3). */
.import-panel,
.import-review-modal {
  color: var(--node-fg);            /* primary text — #e6e6e6 */
}
.import-panel__title,
.import-row__ref,
.import-review-modal__title {
  color: var(--node-fg);
}
.import-row__status,
.import-review-modal__intro,
.import-panel__label {
  color: var(--node-fg-muted);      /* secondary text — #a8a8a8 */
}
```

`--node-fg` / `--node-fg-muted` are the established theme variables (`styles.css:18`–`19`). A vitest assertion on the computed/className colour pins the regression. This is the smallest item — bundle it into the red commit.

#### A.3.5 M06.5 IRL 🟡-1 — HITL `ui_variant` not honored

The M06.5 IRL finding flagged the renderer ignoring the HITL prompt's `ui_variant`. **The diagnosis surface as of the M08 authoring snapshot is mixed**: all three HITL components already branch on `uiVariant` — `HITLModal.tsx:106` (`p.uiVariant === 'modal'`), `HITLPanel.tsx:141` (`'panel'`), `HITLToast.tsx:102` (`'toast'`) — and the `graphStore` reducer already maps `event.ui_variant` into `PendingHitl.uiVariant` (`graphStore.ts:1295`). The schema chain is intact: `event.v1.json:629` `hitl_requested` requires `ui_variant`; `agent_event.ts:133` `HitlUiVariantRef = "panel" | "modal" | "toast"`.

Stage A's job here is to **pin whether the finding is still live**. Two outcomes:

- **If a live drop is found** (e.g. the M06.5 emitter hardcoded one variant, or a renderer path bypasses the variant gate) — fix it at the pinned point.
- **If the surface is already correct** — the finding closed between M06.5 and M08 (likely incidental to an intervening stage). A records this as a *documented structural close* and ships the regression test that **proves** all three variants route correctly (`hitl_requested` with `ui_variant: 'modal'` surfaces `HITLModal` and not `HITLPanel`/`HITLToast`, and symmetrically for the other two) — so the close is pinned by a test, not an assertion.

Pin the variant set from `schemas/hitl.v1.json` (`HitlUiVariant` at `hitl.v1.json:25`) and `schemas/event.v1.json` (`HitlUiVariantRef`) — they must agree.

#### A.3.6 M06.5 IRL 🟡-2 — `npx` Windows `.cmd` shim

An MCP stdio server launched as a bare `npx` fails on Windows — the executable on Windows is `npx.cmd`, and `tokio::process::Command::new("npx")` does not resolve the `.cmd` shim. The fix sits on the `runtime-mcp` stdio transport's pure command-building seam:

```rust
// crates/runtime-mcp/src/transport/stdio.rs — build_command() resolves
// the platform-correct npm shim. The npm CLI ships `npx` as `npx.cmd`
// on Windows; tokio's Command does not auto-resolve the .cmd extension.
pub(crate) fn build_command(&self) -> Command {
    let program = resolve_program(&self.command);
    let mut cmd = Command::new(program);
    for arg in &self.args {
        cmd.arg(arg);
    }
    for (k, v) in &self.env {
        cmd.env(k, v);
    }
    if let Some(c) = &self.cwd {
        cmd.current_dir(c);
    }
    cmd
}

/// Map an npm-shipped CLI name to its Windows `.cmd` shim. `npx`/`npm`
/// are batch shims on Windows; bare names work on Linux/macOS.
fn resolve_program(command: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        match command {
            "npx" => return "npx.cmd".to_string(),
            "npm" => return "npm.cmd".to_string(),
            _ => {}
        }
    }
    command.to_string()
}
```

`build_command()` is the existing pure seam (`stdio.rs:80`, already tested by the `build_command_*` family); the change is contained to it. Keep `resolve_program` minimal and `cfg`-guarded. **Gotcha #74 applies**: a non-`cfg`-Linux branch compiles locally but its errors surface only on CI — write `resolve_program` so the non-Windows branch (`command.to_string()`) compiles cleanly and the `#[cfg(target_os = "windows")]` block is independently well-formed. The test for the Windows branch is itself `cfg`-guarded.

#### A.3.7 M06.5 IRL 🟡-3 — stale Test error banner

The smoke-test error banner persists after a subsequent run starts. `App.tsx:95` already calls `setError(null)` at the top of `handleSmoke`, and `App.tsx:118` renders `{error && <p className="error">{error}</p>}` — so the binding *looks* correct. Stage A **pins whether the stale render is that path**: the likely culprits are (a) the banner is bound to a different state slot than `handleSmoke` clears, or (b) `handleSmoke`'s `setError(null)` runs but a racing event-handler path (`agent_error` in the `subscribeAgentEvents` callback) re-sets it from the prior run, or (c) the error string is surfaced by a child component holding its own state. The build greps the banner's actual data source, pins the real path, and clears it at run start. A stale-banner-cleared vitest test pins the regression (start a run after an error → the banner is gone before the new run's first event).

#### A.3.8 M07-IRL #5 🔴-candidate — tier-promotion-UI disposition (non-code intake)

The M07-IRL Disposition routes #5 to "M08 Stage A intake — decide pre-M08 fix vs M08.A-absorbs after checking the M05/M06 gap-analysis for a known deferral." **The maintainer has directed that #5 is M08 scope** (overriding the MVP doc's later tier-toggle slotting — see Background "Carry-forward absorbed"). Stage A therefore does **not** re-litigate whether #5 is in M08 — it **records the disposition**: #5 is discharged at **Stage G** (the Settings panel).

Stage A's intake note confirms the ground truth from the M07-IRL finding so Stage G is correctly sized: the backend `request_tier_transition` command exists and is unit-tested (`src-tauri/src/commands.rs:542` the command, `:568` the `request_tier_transition_with` seam, `:2322` the test), and the renderer's own type comments + CSS already reference a "Settings panel" that was never built — so #5 is a **renderer-surface gap over a working backend**, correctly sized as a single Stage G. A also checks the M05/M06 gap-analysis + M05.V for a prior "Settings panel" deferral and records whether #5 is a documented-gap re-confirmation or a fresh gap (a 2-minute read, not a blocker). **A ships no Settings code.**

#### A.3.9 §1c-vs-§0d Tester-scope intake clarification (non-code intake)

Spec Phase 9 says the Tester runs "in an isolated session with a separate SQLite database (drone-managed sandbox per §1c)", but §0d marks §1c multi-session ❌ for v0.1. These are reconcilable, and Stage A surfaces the reconciling reading for the maintainer to confirm at the A surface: **the v0.1 Tester is a sequential, throwaway, build-time test session** — its own throwaway SQLite, capability violations → test failures (defaults applied), results discarded on close, run from build mode where no live runtime session is executing — **not** the §1c concurrent-session pool. The Tester needs *an* isolated session; it does not need the §1c multi-session feature.

This is a one-line intake item (recommended reading + "confirm?"), **not code**. It is load-bearing: Stage F1's `ADR-0019` (Tester isolated-session model) is authored against this reading, and the closeout's gap-analysis records the spec-clarification. Surfacing it at A means F1 is designed against a confirmed reading rather than an unresolved spec tension.

#### A.3.10 The `<construction_reachability_check>` map (load-bearing)

The v1.8 mechanism that documents a construction graph completing across stages. For each of the three M07.V Dec-6 wires, A states the production driver, that its inputs are not reachable on `main` today, and that Stage F1 makes them reachable:

- **`skills_lock::verify` on the artifact-load path** — M07.V 🟡 #2. On `main`, `skills_lock::verify` (`skills_lock/mod.rs:107`) has only test callers — there is no production code path that byte-loads an imported skill/tool/agent and verifies it. Stage F1's Tester loads the framework's imported artifacts → the production caller. (Stage B ships the `list_installed` *reader*, the read half of #2; F1 ships the *verify-on-load*, the integrity half.)
- **`McpDispatcher::on_server_connected` connect-handler** — M07.V 🟡 #3. On `main`, no production code calls `McpDispatcher::on_server_connected` / `on_server_disconnected` (`dispatch.rs:141`/`:155`). Stage F1's Tester connects MCP servers for the test session → the production caller; each returned `NewAmbiguity` → `AgentEvent::ToolAliasAmbiguous` (spec §5a step 5).
- **The agent-with-tools production driver** — M07.V 🟡 #5. `AgentSdk::run_agent` is the multi-turn loop (M07.D2), exercised only by the assembled integration test; the only production caller (`run_smoke_session`, `commands.rs:120`) builds a no-tools config and so emits no `ProviderEvent::ToolUse`. Stage F1's Tester runs a tool-bearing framework → the production trigger.

A authors this as the `<construction_reachability_check>` block in the A.5 prompt with each wire `inputs_reachable="false"`; Stage F1 inverts each to `true` with file:line and Stage V verifies the discharge — the M07 A→D1→D2 pattern, here A→F1.

### A.4 Tests

Each absorbed item gets the test its 🟡/finding demanded. Strict v1.8 two-commit TDD: the red commit carries every test below; the impl commit touches no test file.

#### A.4.1 `plan_loop` driver tests

`crates/runtime-main/src/plan/plan_loop.rs` (in-source `#[cfg(test)]`, or `crates/runtime-main/tests/plan_loop.rs`):

- `drive_plan_approval_required_advances_pending_to_inprogress_on_approve`
- `drive_plan_approval_required_aborts_plan_on_reject`
- `drive_plan_no_approval_required_runs_through_to_complete`
- `drive_plan_emits_plan_started_after_transition_to_inprogress`
- `drive_plan_transition_to_complete_when_execution_returns`
- `drive_plan_illegal_transition_surfaces_plan_loop_error` (driver-bug path)

#### A.4.2 M07-IRL #2 — token in/out split (vitest)

`src/lib/graphStore.test.ts`:

- `token_usage_event_populates_running_agent_tokens_in_and_out`
- `token_usage_event_accumulates_across_multiple_turns`
- `token_usage_event_with_no_running_agent_is_a_noop`
- `agent_complete_still_sets_tokens_total_and_does_not_zero_in_out`

#### A.4.3 M07-IRL #7 — key persistence

`src-tauri/src/commands.rs` (in-source `#[cfg(test)]`) + `src/App.test.tsx`:

- `has_api_key_with_returns_true_when_probe_reports_present`
- `has_api_key_with_returns_false_when_probe_reports_absent`
- `app_calls_has_api_key_on_mount_and_enables_smoke_button_when_present` (vitest)
- `app_keeps_smoke_button_disabled_when_has_api_key_returns_false` (vitest)

#### A.4.4 M07-IRL #3 — import-panel contrast (vitest)

`src/components/ImportPanel.test.tsx`:

- `import_panel_title_uses_node_fg_theme_token`
- `import_row_ref_uses_node_fg_theme_token`
- `import_review_modal_heading_is_not_background_coloured`

#### A.4.5 M06.5 IRL 🟡-1/-2/-3 — HITL variant / npx / stale banner

`src/components/HITL*.test.tsx`, `crates/runtime-mcp/src/transport/stdio.rs`, `src/App.test.tsx`:

- `hitl_requested_modal_variant_surfaces_modal_only`
- `hitl_requested_panel_variant_surfaces_panel_only`
- `hitl_requested_toast_variant_surfaces_toast_only`
- `build_command_resolves_npx_to_npx_cmd_on_windows` (`#[cfg(target_os = "windows")]`-guarded)
- `build_command_keeps_bare_npx_on_non_windows` (`#[cfg(not(target_os = "windows"))]`-guarded)
- `smoke_error_banner_cleared_when_new_run_starts` (vitest)

#### A.4.6 Acceptance criteria

- [ ] Every cited 🟡/finding has a named green test OR a documented structural close pinned by a test (A.3.5 / A.3.7 may resolve as documented-close-with-test).
- [ ] `cargo test --workspace` + the full v1.6 canonical gate suite green.
- [ ] `cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs" --fail-under-lines 95` — runtime-main ≥95 maintained (the new `plan_loop.rs` is pure-logic, tempfile-free; no new exclusion).
- [ ] `npm run test` + vitest ≥80 on `src/`.
- [ ] The `<construction_reachability_check>` enumerates the three M07.V Dec-6 wires `inputs_reachable="false" resolution="Stage F1"`.
- [ ] The #5 → Stage G disposition + the §1c-Tester intake reading are surfaced for the maintainer (non-code items).
- [ ] Strict v1.8 two-commit invariant proven: `git diff <red>..<impl> -- '**/tests/**'` EMPTY (binary-crate scoped-diff variant for any `src-tauri/src/` in-source `#[cfg(test)]` block).
- [ ] CI-parity per G6.

### A.5 CLI Prompt

```xml
<work_stage_prompt id="M08.A">
  <context>
    M08 Stage A — clear the post-M07 carry-forward debt backlog and author
    the construction-graph groundwork. NOT a Builder Canvas feature stage
    (B–G build the workbench). A absorbs debt (M04 plan_loop driver shell;
    M07-IRL #2 token in/out split, #3 import-panel contrast, #7 API-key
    persistence; M06.5 IRL 🟡-1 HITL ui_variant, 🟡-2 npx Windows .cmd
    shim, 🟡-3 stale Test banner), dispositions M07-IRL #5 (→ Stage G)
    and M06.5 IRL 🟡-4 (→ Stage G), surfaces the §1c-vs-§0d Tester-scope
    reading for the maintainer, and maps the
    <construction_reachability_check> for the three M07.V Dec-6 wires
    Stage F1 discharges (skills_lock::verify on load,
    McpDispatcher::on_server_connected, the agent-with-tools driver). A
    ships NO Builder feature code. A is itemcount-dense (absorption-stage
    nature, M07.A precedent) — if the red-phase commit exceeds one
    coherent unit, surface for an A1/A2 split (backend items in one,
    renderer items in the other; M04.A1/A2 precedent).
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — the audit slots; tag ordering)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Implementation Workflow, Stage A A.1–A.4)</file>
    <file>docs/M07-irl-findings.md (the post-M07.5 IRL walk-through — the routing source for #2/#3/#5/#6/#7)</file>
    <file>docs/build-prompts/retrospectives/M07.V-retrospective.md ([END] Decisions 2/3/4/5 — the coupled Dec-6 set)</file>
    <file>docs/gap-analysis.md (M07 entry + M07.V carry-forward: the M06.5 IRL 🟡-1..4 + M04 plan_loop exact text/cites)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543) + §0d (the §1c-vs-Tester scope tension) + §3a (plan/task) + §5a (re-resolution) + §6a (HITL ui_variant)</file>
    <file>docs/gotchas.md (#23 Playwright/Tauri; #56/#81 coverage; #74 cfg-Linux first-compile gap — relevant to the npx .cmd cfg-guard)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="M04 plan FSM — drive_plan walks PlanStateMachine; pin PlanState/PlanEvent/transition signatures">crates/runtime-main/src/plan/state_machine.rs</file>
    <file purpose="plan module root — A adds `pub mod plan_loop;`">crates/runtime-main/src/plan/mod.rs</file>
    <file purpose="ADR-0007 HitlSeam — drive_plan routes the approval gate through it; pin HitlSeam/HitlDecision paths">crates/runtime-main/src/hitl/mod.rs</file>
    <file purpose="M02 key_store — A adds has_api_key() (read_api_key().is_ok())">crates/runtime-main/src/key_store.rs</file>
    <file purpose="Tauri command surface — the `*_with` seam pattern (approve_plan/approve_plan_with at :409/:421); the documented plan_loop deferral at :403/:2128; add has_api_key">src-tauri/src/commands.rs</file>
    <file purpose="App root — hasKey hardcoded useState(false) at :51; handleSmoke setError(null) at :95; the mount useEffect">src/App.tsx</file>
    <file purpose="IPC wrappers — A adds invokeHasApiKey()">src/lib/ipc.ts</file>
    <file purpose="live-graph store — `token_usage` is a no-op arm at the no-op switch group; agent_complete sets only tokensTotal; hitl_requested maps ui_variant">src/lib/graphStore.ts</file>
    <file purpose="SDK event pipeline — ProviderEvent::Usage → AgentEvent::TokenUsage is correct; proves the drop is renderer-side">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="drone token_usage projector — projects input/output correctly; proves the drop is renderer-side">crates/runtime-drone/src/token_usage.rs</file>
    <file purpose="theme CSS variables (--node-fg / --node-fg-muted at :18-19) + the .import-* selectors that set background but no color (root cause ~:1130)">src/styles.css</file>
    <file purpose="MCP stdio transport — build_command() is the pure seam where the npx .cmd resolution lands (~:80)">crates/runtime-mcp/src/transport/stdio.rs</file>
    <file purpose="the three HITL components — each already branches on uiVariant; pin whether 🟡-1 is a live drop">src/components/HITLModal.tsx</file>
    <file purpose="MCP dispatcher — on_server_connected/on_server_disconnected (~:141/:155) have no production caller (the A→F1 wire)">crates/runtime-mcp/src/dispatch.rs</file>
    <file purpose="skills.lock primitive — verify() (~:107) has only test callers (the A→F1 wire)">crates/runtime-main/src/skills_lock/mod.rs</file>
  </read_reference>

  <read_prior_milestones>
    <gap_analysis_carry_forward milestone="M07"/>
  </read_prior_milestones>

  <deliverable ref="docs/build-prompts/M08-workbench.md" section="A.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      One standalone `test(M08.A): …` commit with the failing tests for
      every code-bearing item (plan_loop driver; token in/out split;
      has_api_key seam + App mount read; import-panel contrast; HITL
      ui_variant routing; npx .cmd resolution; stale banner). Stub the
      production surfaces just enough to compile the test files
      (todo!() / unimplemented!() bodies fine). Confirm right-reason
      failure per CLAUDE.md §5 (assertion failed / cannot find function /
      unresolved import / not-yet-implemented panic — NOT a test-file
      compile error, NOT a tautological pass). The #5 → Stage G
      disposition + the §1c-Tester intake reading are non-test items —
      note them as such in the commit body. Surface the red-phase commit
      for approval before green phase.
    </red_phase>
    <green_phase>
      Implement until ALL failing tests pass. Do NOT modify the test
      files during implementation — if a test is wrong, fix it in a
      SEPARATE labelled follow-up commit. The impl commit body MUST state
      the verifiable invariant `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. The binary-crate scoped-diff variant
      (in-source `#[cfg(test)]` block byte-identical red→impl via a
      scoped `git diff … src-tauri/src/commands.rs`) applies to any
      src-tauri/src/ in-source test change (the M06.5.A.fix precedent).
      No Co-Authored-By in any commit message.
    </green_phase>
  </tdd_discipline>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="A.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>

  <gates milestone="M08"/>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <adr_triggers>
    <trigger>None new in A. ADR-0019 (Tester isolation) is filed at F1; ADR-0020 (Builder state model) at C. If the token-split diagnosis (A.3.2) is found to require a schema change to event.v1.json — e.g. adding agent_id to TokenUsage — that triggers §14 (schema + cargo xtask regenerate-types + ADR); surface before the red commit rather than authoring it silently.</trigger>
  </adr_triggers>

  <pre_flight_check>
    <check name="branch">HEAD is the M08 parent-milestone branch, cut from main at f5eac40 (post-M07.5 PR #90 + M07-IRL PR #91 merges)</check>
    <check name="m07_5_merged">git log on main confirms PR #90 (M07.5) + PR #91 (M07-IRL) merged — A builds on the post-M07.5 import lifecycle</check>
    <check name="plan_loop_deferral">grep the documented plan_loop deferral sites in src-tauri/src/commands.rs (the approve_plan doc comment + the approve_plan_with_no_pending_awaiter test rationale) before wiring the driver</check>
    <check name="llvm_cov_clean">run `cargo llvm-cov clean --workspace` before the coverage gates if any prior llvm-cov ran this session (gotcha #81)</check>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="crates/runtime-main/src/plan/plan_loop.rs" verified="false" note="A creates this file"/>
    <claim type="file" path="crates/runtime-main/src/plan/state_machine.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/plan/mod.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/key_store.rs" verified="true"/>
    <claim type="file" path="crates/runtime-mcp/src/transport/stdio.rs" verified="true"/>
    <claim type="file" path="src/styles.css" verified="true"/>
    <claim type="struct_field" path="crates/runtime-main/src/plan/state_machine.rs" symbol="PlanState{plan_id,status}" verified="true" note="PlanState is {plan_id: String, status: PlanStatus}"/>
    <claim type="method" path="crates/runtime-main/src/plan/state_machine.rs" symbol="PlanStateMachine::transition" verified="true" note="static transition(state: &amp;mut PlanState, event: PlanEvent) -> Result&lt;(), TransitionError&gt;"/>
    <claim type="method" path="crates/runtime-main/src/key_store.rs" symbol="read_api_key" verified="true" note="A adds has_api_key() wrapping read_api_key().is_ok()"/>
    <claim type="method" path="crates/runtime-mcp/src/transport/stdio.rs" symbol="build_command" verified="true" note="pure pub(crate) seam — the npx .cmd resolution lands here"/>
    <claim type="method" path="crates/runtime-main/src/skills_lock/mod.rs" symbol="verify" verified="true" note="exists; A maps that it has no production caller (the F1 wire)"/>
    <claim type="method" path="crates/runtime-mcp/src/dispatch.rs" symbol="on_server_connected" verified="true" note="exists; A maps that it has no production caller (the F1 wire)"/>
    <claim type="read_first_target" path="docs/M07-irl-findings.md" verified="true"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M07.V-retrospective.md" verified="true"/>
  </phase_doc_inventory_audit>

  <schema_ref_audit>
    <ref schema="schemas/event.v1.json" path="#/$defs/HitlUiVariantRef" verified="true" note="hitl_requested.ui_variant — pin the variant set agrees with hitl.v1.json HitlUiVariant"/>
    <ref schema="schemas/event.v1.json" path="TokenUsage" verified="true" note="required: [type,input,output,model,cost_usd] — NO agent_id; the token_usage reducer attributes to the running agent"/>
  </schema_ref_audit>

  <architecture_check>
    <claim description="The token in/out drop is renderer-side, not Rust — the SDK emits AgentEvent::TokenUsage correctly and the drone projects it correctly" verify="grep -n 'AgentEvent::TokenUsage' crates/runtime-main/src/sdk/event_pipeline.rs ; grep -n 'input_tokens\|output_tokens' crates/runtime-drone/src/token_usage.rs ; expect both populated — the fix is in src/lib/graphStore.ts only"/>
    <claim description="The key-persistence root cause is the absent startup read, not a write failure — no has_api_key command exists and App.tsx hardcodes hasKey false" verify="grep -rn 'has_api_key\|hasKey' src-tauri/src/commands.rs src/App.tsx ; expect zero has_api_key command + a hardcoded useState(false)"/>
    <claim description="plan_loop drives the FSM but routes approval through the in-process HitlSeam — NOT a drone-mediated path (ADR-0007)" verify="grep -rn 'HitlSeam\|drone' crates/runtime-main/src/plan/ ; expect the driver to use the in-process seam, no DroneCommand"/>
    <claim description="The import-contrast root cause is one CSS rule set — .import-* selectors set background but no color" verify="grep -n 'import-panel\|import-row\|color:' src/styles.css ; expect .import-panel to set background and no color"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="token_usage" purpose="enumerate every consumer — confirm the renderer no-op arm is the only drop and the drone projector / SDK emitter are correct"/>
    <grep pattern="uiVariant" purpose="confirm all three HITL components branch on it + the graphStore reducer maps event.ui_variant — pins whether 🟡-1 is a live drop"/>
    <grep pattern="plan_loop" purpose="enumerate the documented deferral sites in commands.rs before wiring the driver"/>
  </fan_out_grep>

  <construction_reachability_check>
    <wire claim="skills_lock::verify called on the artifact-load path" constructor="(no production byte-load path on main — verify() has only test callers; M07.V 🟡 #2)" inputs_reachable="false" resolution="Stage F1 — the Tester loads the framework's imported artifacts and verifies them"/>
    <wire claim="McpDispatcher::on_server_connected production connect-handler" constructor="(no production caller on main — dispatch.rs on_server_connected/on_server_disconnected are test-only; M07.V 🟡 #3)" inputs_reachable="false" resolution="Stage F1 — the Tester's MCP connect handler; each NewAmbiguity → AgentEvent::ToolAliasAmbiguous"/>
    <wire claim="agent-with-tools production driver (tool-emitting ProviderEvent::ToolUse)" constructor="AgentSdk::run_agent (the only production caller, run_smoke_session, builds a no-tools config; M07.V 🟡 #5)" inputs_reachable="false" resolution="Stage F1 — the Tester runs a tool-bearing framework through run_agent"/>
  </construction_reachability_check>

  <existing_pattern_audit>
    <pattern grep_for="case 'token_usage'" rationale="the graphStore applyEvent switch is exhaustive (a `never` default arm) — changing token_usage from a no-op arm to a real reducer case must not break the exhaustiveness check; confirm the no-op group's other variants stay grouped or split cleanly" affected_files="src/lib/graphStore.ts" remediation="split token_usage out of the no-op `case` fall-through into its own block; leave session_end/tool_error/mode_changed/stream_text/decision_record as the no-op group"/>
  </existing_pattern_audit>

  <gotchas>
    <trap>#74 — a non-cfg-Linux branch compiles locally but errors only on CI; resolve_program's #[cfg(target_os = "windows")] block AND its non-Windows fallthrough must each be independently well-formed, and the Windows test is itself cfg-guarded.</trap>
    <trap>#81 — `cargo llvm-cov clean --workspace` before the coverage gates if any prior llvm-cov ran this session.</trap>
    <trap>A ships NO Builder Canvas code — that is B–G. A absorbs debt + maps the construction graph.</trap>
    <trap>The BudgetThreshold→BudgetWarn rename was found factually false at M07.A — do not re-touch it.</trap>
    <trap>AgentEvent::TokenUsage carries no agent_id — the token_usage reducer must attribute to the running agent (the single-active-agent assumption tool_result already relies on), not look up a non-existent field.</trap>
    <trap>Tests-pass-but-contract-fails (gotcha #66) — for the token split, assert the in/out values are CONSISTENT with the total a real run produces, not merely non-zero; for the HITL variants, assert each variant surfaces ITS component and NOT the other two.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT widen the token-split fix into the Rust SDK / drone — both are already correct; touching them is scope creep and risks the M07.D2 assembled regression. The fix is src/lib/graphStore.ts (+ the inspector read) ONLY.</warning>
    <warning>DO NOT add a HITL trigger or schema field for any item — A.3.5 is a renderer-routing pin, not a protocol change. If the diagnosis finds the components already correct, A.3.5 resolves as a documented structural close pinned by a test, not a code edit.</warning>
    <warning>DO NOT build any Settings-panel code for M07-IRL #5 — A records the disposition (→ Stage G) only. Building it here violates the stage contract.</warning>
    <warning>DO NOT introduce a DroneCommand variant for the plan_loop driver — the drone is audit + projection + persistence (ADR-0007); the plan driver + its HITL gate are in-process. If a variant seems necessary, surface in the retrospective rather than authoring it.</warning>
    <warning>The §1c-Tester scope reading (A.3.9) is surfaced for the maintainer to CONFIRM — do not author Stage F1 design assumptions into Stage A beyond stating the reading.</warning>
  </execution_warnings>

  <runtime_environment os="windows" note="Build on Windows (the v0.1 target); the npx .cmd resolution + the Windows Credential Manager key-persistence check both depend on the Windows environment. CI runs all three OSes."/>

  <time_box hours="6-9"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>List each absorbed item with its green test or documented-structural-close-pinned-by-test. Record the #5 → Stage G disposition + the M05/M06 prior-deferral check result (documented-gap re-confirmation vs fresh gap). Record the §1c-Tester intake reading + the maintainer's confirmation. State the &lt;construction_reachability_check&gt; map and that Stage F1 owns all three resolutions. Record the pinned root cause for each diagnosis-bearing item (token split = renderer no-op arm; key persistence = absent startup read OR Windows write-fail; HITL 🟡-1 = live drop OR already-closed; stale banner = the real binding path). If A grew past one coherent red unit, note whether an A1/A2 split was surfaced.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="A.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M08.*-retrospective.md)</item>
    <item>strict-TDD invariant: git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**' EMPTY (+ the binary-crate scoped-diff for any src-tauri/src/ in-source test change)</item>
    <item>per-item carry-forward closure table (each 🟡/finding → named test / documented-close-pinned-by-test, with the pinned root cause)</item>
    <item>the #5 → Stage G disposition + the §1c-Tester scope reading (maintainer confirmation requested)</item>
    <item>the &lt;construction_reachability_check&gt; map (three M07.V Dec-6 wires inputs_reachable="false" → Stage F1)</item>
    <item>gate results (v1.6 canonical order; runtime-main ≥95 maintained; workspace ≥80; vitest ≥80; CI-parity per G6)</item>
    <item>M08.A retrospective filled-in [END] section</item>
    <item>draft commit message from A.6</item>
    <item>explicit statement: "Stage M08.A is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### A.6 Commit Message

```
feat(runtime): M08 Stage A — clear post-M07 carry-forward backlog + Tester construction-graph groundwork

Absorbs the post-M07 debt backlog:
- M04 🟡: a new plan/plan_loop.rs driver shell that walks the
  PlanStateMachine through approval + execution, routing the approval
  gate through the in-process HitlSeam (ADR-0007).
- M07-IRL #2: the token in/out split — the renderer dropped it (the
  graphStore `token_usage` no-op arm); the SDK + drone were already
  correct. The reducer now attributes input/output to the running
  agent node.
- M07-IRL #7: API-key persistence — the root cause was the absent
  startup read (no has_api_key command, App.tsx hardcoded hasKey
  false). Adds key_store::has_api_key + the has_api_key command + the
  App mount read.
- M07-IRL #3: import-panel contrast — the .import-* CSS set background
  but no color; pinned to the --node-fg / --node-fg-muted theme tokens.
- M06.5 IRL 🟡-1 HITL ui_variant routing, 🟡-2 npx Windows .cmd shim
  (build_command resolves npx → npx.cmd under cfg(windows)), 🟡-3
  stale Test error banner.

Dispositions M07-IRL #5 (tier-promotion UI → Stage G) + M06.5 IRL
🟡-4 (budget settings → Stage G). Surfaces the §1c-vs-§0d Tester-scope
reading for the maintainer. Authors the v1.8
<construction_reachability_check> mapping the three M07.V Dec-6 wires
(skills_lock::verify on load, McpDispatcher::on_server_connected, the
agent-with-tools driver) — inputs not reachable on main; Stage F1
discharges all three.

No new ADR. Strict v1.8 two-commit TDD: git diff <red>..<impl> --
'**/tests/**' EMPTY. runtime-main ≥95 maintained; vitest ≥80.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage B — Builder backend: validate · persist · capability summary · `skills.lock` reader

### B.1 Problem Statement

Spec Phase 9: *"Capability validation runs in the Rust main process (re-using `crates/runtime-core` capability logic) … No duplication of validation logic between TS and Rust."* and *"Generated artifacts conform to schemas — anything the canvas exports is valid against `schemas/framework.v1.json`, `skill.v1.json`, `tool.v1.json`, `agent.v1.json`."* The Canvas (Stages D1/D2), the Inspector (E), and the Tester (F1) all need a backend that they share — there must be **one** validator, **one** capability summary, **one** save/load path. None of that backend exists on `main`.

Stage B builds it as a new `crates/runtime-main/src/builder/` module: pure / seam-testable, path-agnostic, and **reuse-heavy by mandate**. It validates an in-progress framework against the schema-derived types + the capability primitive (no second validator in TS — spec §9); it computes a whole-framework capability summary by aggregating the existing `framework_loader/capability_map.rs`; it writes a `framework.json` + companion `.md` files to a directory and reads them back via the existing `framework_loader`; and it ships the **first production `skills.lock` reader** so the Palette and Import panel can list what is installed. B is a pure backend stage — the renderer stages C–F consume it, so it must land first. **B ships no renderer code.**

B is CODEOWNERS-adjacent: it *reads* the capability surface (`runtime-core` capability types, `capability/narrowing.rs`). Per Hard Rule 8 it surfaces its plan first — the pre-flight `<phase_doc_inventory_audit>` + this section **are** that plan. B reuses but does **not modify** capability enforcement: if a change would touch enforcement behavior, that is a separate surfaced plan.

**Concrete deliverables:**

1. **`crates/runtime-main/src/builder/mod.rs`** (new) — the module root: `pub mod validate; pub mod persist; pub mod summary; pub mod error;` + re-exports. `crates/runtime-main/src/lib.rs` adds `pub mod builder;`.
2. **`crates/runtime-main/src/builder/validate.rs`** (new) — `validate_framework(doc: &serde_json::Value) -> FrameworkValidationReport`: pure, bytes-in/report-out, composing schema-shape validation (serde deserialization into the generated `Framework` type — which carries the schema constraints) + `framework_loader::walk` (the existing Layer-1 gap walker) + `runtime-core` capability checking. Returns `FrameworkValidationReport { schema_errors, capability_errors, ok, capability_summary }` keyed to the offending node / JSON-path; `capability_summary` (the B.3.4 whole-framework summary) rides on the report so the renderer reads one report, not a second command — `None` when schema validation fails.
3. **`crates/runtime-main/src/builder/persist.rs`** (new) — `save_framework(dir: &Path, fw: &Framework, companions: &[Companion]) -> Result<(), BuilderError>` + `load_framework(dir: &Path) -> Result<LoadedFramework, BuilderError>`: path-agnostic `&Path` persistence (CLAUDE.md §9 — the `audit::file_path` / `skills_lock` archetype); writes `framework.json` + companion `skill.md`/`tool.md`/`agent.md`; reload reuses `framework_loader`.
4. **`crates/runtime-main/src/builder/summary.rs`** (new) — `framework_capability_summary(fw: &Framework) -> FrameworkCapabilitySummary`: aggregates `framework_loader/capability_map.rs` into whole-framework totals + carries, per Agent→Agent spawn edge, the narrowing triple `{ parent_caps, child_declared_caps, narrowed_caps }` via the reused `capability/narrowing.rs::narrow()`. `validate_framework` (deliverable 2) calls this and embeds the result as the report's `capability_summary` field — there is **no separate `framework_capability_summary` Tauri command**.
5. **`crates/runtime-main/src/builder/validate.rs` (`list_installed`)** — `list_installed(lock_path: &Path) -> Result<Vec<InstalledArtifact>, BuilderError>`: the first production `skills.lock` reader (closes M07-IRL #6 + the read half of M07.V 🟡 #2). Reads via `skills_lock::read` (`skills_lock/mod.rs:61`); an absent lock returns an empty vec, not an error.
6. **`crates/runtime-main/src/builder/error.rs`** (new) — `BuilderError` (thiserror; `From` conversions from `std::io::Error`, `serde_json::Error`, `FrameworkLoadError`, `LockError`).
7. **`src-tauri/src/commands.rs`** — four thin Tauri commands (`validate_framework`, `save_framework`, `load_framework`, `list_installed_artifacts`) wrapping the `builder` seams; the `*_with` seam is the unit-tested core, the `#[tauri::command]` wrapper is the §5 tauri-shell holdout.
8. **≥95% line on the `builder` module** within `runtime-main` — the module is pure / seam / `tempfile`-tested, so B adds **no new `--ignore-filename-regex` exclusion** and **no four-mirror coverage change** (the `skills_lock` ≥95 precedent — its FS calls are `&Path`-parameterised and tempfile-reachable).

**Not in this stage:**

- Any renderer code — the Builder shell, Palette, Canvas, Inspector, Tester modal, Settings panel (Stages C–G consume the `builder` backend).
- A new schema. B *consumes* `schemas/{framework,skill,tool,agent,common}.v1.json`; if a stage need surfaces an `event.v1.json` variant (it should not in B), that is the §14 schema-regenerate-ADR flow — surface before red.
- The `skills_lock::verify`-on-load path (the *integrity* half of M07.V 🟡 #2) — that is Stage F1's Tester artifact-load path. B ships the *reader*; F1 ships the *verify*.
- Any modification to capability enforcement (`capability/enforcer.rs`), the drone, or the sandbox boundary — B reuses the capability + narrowing surfaces read-only.
- The `builderStore.ts` Zustand store — that is Stage C (the Builder store is separate from `graphStore.ts`; do not overload `graphStore`).

### B.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/builder/mod.rs` | **new** | Module root: `pub mod validate; pub mod persist; pub mod summary; pub mod error;` + re-exports of the public surface. |
| `crates/runtime-main/src/builder/validate.rs` | **new** | `validate_framework` (schema-shape + gap-walk + capability validation, errors keyed to the offending node/JSON-path) + `list_installed` (the `skills.lock` production reader). |
| `crates/runtime-main/src/builder/persist.rs` | **new** | `save_framework` / `load_framework` (path-agnostic `&Path`; `framework.json` + companion `skill.md`/`tool.md`/`agent.md`; reload reuses `framework_loader`). |
| `crates/runtime-main/src/builder/summary.rs` | **new** | `framework_capability_summary` (whole-framework totals + per-spawn-edge narrowing triple via the reused `narrowing.rs`). |
| `crates/runtime-main/src/builder/error.rs` | **new** | `BuilderError`; thiserror; `From` for `io::Error` / `serde_json::Error` / `FrameworkLoadError` / `LockError`. |
| `crates/runtime-main/src/lib.rs` | exists | Edit: `pub mod builder;`. |
| `src-tauri/src/commands.rs` | exists | Edit: `validate_framework` / `save_framework` / `load_framework` / `list_installed_artifacts` commands — thin shell wrappers over the `builder` module (`*_with` seam is the unit-tested core); wire into the `invoke_handler` in `main.rs`. |
| `src-tauri/src/main.rs` | exists | Edit: register the four new commands in the `invoke_handler` macro. |
| `crates/runtime-main/tests/builder.rs` | **new** (optional) | Integration tests for `save_framework` / `load_framework` round-trip against `tempfile` paths (or in-source `#[cfg(test)]`). |
| `CHANGELOG.md` | exists | `[Unreleased]` Stage B entries. |
| `docs/build-prompts/retrospectives/M08.B-retrospective.md` | **new** | Stage B retrospective. |

Effort budget: ~8–11 hours. The largest piece is `validate_framework` + the `framework_capability_summary` aggregation; `save`/`load` is a thin companion-`.md` writer over the existing `framework_loader` read, and `list_installed` is a thin `skills_lock::read` wrap. The cost is concentrated in getting the **report shapes** right (the renderer turns them into red badges in D2 and the Inspector's Validate result in E — they must be node-keyed and stable) rather than in raw logic volume.

### B.3 Detailed Changes

#### B.3.1 `BuilderError` — `crates/runtime-main/src/builder/error.rs`

```rust
//! Error surface for the Builder backend (M08 Stage B).

use thiserror::Error;

use crate::framework_loader::FrameworkLoadError;
use crate::skills_lock::LockError;

/// Failure modes raised by the `builder` module.
#[derive(Debug, Error)]
pub enum BuilderError {
    /// Filesystem error writing/reading a framework directory.
    #[error("builder I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// `framework.json` serialization/deserialization failed.
    #[error("framework JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// The framework_loader rejected a load (gaps / parse).
    #[error(transparent)]
    Load(#[from] FrameworkLoadError),
    /// The skills.lock reader rejected the lock (corrupt / parse).
    #[error(transparent)]
    Lock(#[from] LockError),
    /// `save_framework` was given a path that exists and is not a directory.
    #[error("save target is not a directory: {0}")]
    NotADirectory(String),
}
```

`validate_framework` itself does **not** return `Result` — validation *failures are the report*, not errors. `BuilderError` covers only `persist` (FS) and `list_installed` (lock-corruption) operational failures. `LockError` (`skills_lock/error.rs`) is `From`-converted so `list_installed` can surface a corrupt-lock distinctly from an absent one. Mirrors the M07 import-pipeline error layering.

#### B.3.2 `validate_framework` — `crates/runtime-main/src/builder/validate.rs`

`validate_framework` takes the in-progress framework document (the canvas's serialized `framework.json` candidate — it may be incomplete or invalid; that is the point — continuous validation runs as the user edits) and returns a structured report the renderer turns into red badges (D2) and the Inspector's Validate result (E). It **composes existing validators — it reimplements neither**:

- **Schema-shape validation.** v0.1 has no Rust JSON-Schema validator — the CI schema check is a Python `jsonschema` script (`.github/workflows/ci.yml` `schema-validate` job). The Rust source of schema truth is the **typify-generated `Framework` type**: `serde_json::from_value::<Framework>(doc)` succeeds iff the document matches `framework.v1.json`'s shape + the generated newtype pattern constraints (`name` pattern, `SemVer`, validated string newtypes). A serde error is a schema error, and its `path()` is the JSON-path key.
- **Reference + gap validation.** `framework_loader::walk` (`framework_loader/walker.rs` — the M04 Layer-1 walker, re-exported at `framework_loader/mod.rs:35`) returns one `Gap` per unresolved `tool`/`skill`/`agent` reference; each `Gap` carries `agent_id` + `missing_name` + `kind`.
- **Capability validation.** `framework_loader::capabilities_to_declarations` + `parent_grants_for_agent` (re-exported at `framework_loader/mod.rs:30`) translate the coarse `Capabilities` block into the per-action `CapabilityDeclaration` set; a malformed or self-inconsistent capability block surfaces as a `capability_error`.

```rust
//! Continuous framework validation (M08 Stage B) — the single validator
//! the Canvas, Inspector, and Tester share. Composes the typify-generated
//! Framework shape check + framework_loader::walk + the runtime-core
//! capability translation. NO second validator in TS (spec §9).

use serde_json::Value;

use crate::builder::summary::{framework_capability_summary, FrameworkCapabilitySummary};
use crate::framework_loader::{self, capability_map, walk, Gap};
use runtime_core::generated::framework::Framework;

/// One validation problem, keyed to the offending node / JSON-path so
/// the renderer (D2 red badges, E Validate result) can attribute it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NodeError {
    /// JSON-path or node id the error attaches to (e.g.
    /// `agents[0].allowed_tools` or the agent id `worker`).
    pub node_path: String,
    /// Human-readable problem description.
    pub message: String,
}

/// Structured validation report. `ok` iff both error lists are empty.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FrameworkValidationReport {
    /// Schema-shape problems (failed deserialization into `Framework`,
    /// pattern-constraint violations).
    pub schema_errors: Vec<NodeError>,
    /// Capability / reference problems (unresolved refs, malformed
    /// capability blocks). L1-disclosure-class problems.
    pub capability_errors: Vec<NodeError>,
    /// `schema_errors.is_empty() && capability_errors.is_empty()`.
    pub ok: bool,
    /// The whole-framework capability summary (B.3.4) — carries the
    /// per-Agent→Agent-edge narrowing triple. `None` when schema
    /// validation fails (no parsed `Framework` to summarize). Rides on
    /// the report so the Inspector (E) and the canvas (D2) render one
    /// capability picture from one backend computation — there is no
    /// separate `framework_capability_summary` command.
    pub capability_summary: Option<FrameworkCapabilitySummary>,
}

/// Validate an in-progress framework document.
///
/// Pure: bytes in, report out. No FS, no network. The seam the
/// `validate_framework` Tauri command wraps.
#[must_use]
pub fn validate_framework(doc: &Value) -> FrameworkValidationReport {
    // 1. Schema-shape — deserialize into the generated Framework type.
    let framework: Framework = match serde_json::from_value(doc.clone()) {
        Ok(fw) => fw,
        Err(e) => {
            // A shape failure short-circuits — reference + capability
            // checks need a parsed Framework. The serde error's path
            // is the offending JSON key. No `capability_summary`:
            // there is no parsed Framework to summarize.
            return FrameworkValidationReport {
                schema_errors: vec![NodeError {
                    node_path: e.path().to_string(),
                    message: e.to_string(),
                }],
                capability_errors: Vec::new(),
                ok: false,
                capability_summary: None,
            };
        }
    };
    // 2. Reference + gap validation via the M04 Layer-1 walker.
    let gaps: Vec<Gap> = walk(&framework);
    let mut capability_errors = gaps
        .iter()
        .map(|g| NodeError {
            node_path: g.agent_id.clone(),
            message: format!("unresolved {:?} reference: {}", g.kind, g.missing_name),
        })
        .collect::<Vec<_>>();
    // 3. Whole-framework capability summary (B.3.4) — also the source of
    //    the per-Agent→Agent-edge narrowing decisions. It rides on the
    //    report so the renderer reads one report, not a second command.
    let summary = framework_capability_summary(&framework);
    // 4. A failed Agent→Agent narrowing — the child declares a capability
    //    the parent does not hold (L2a, all-or-nothing) — is a capability
    //    error keyed to the child agent, so D2.3.5 badges that node and
    //    `ok` is false. The narrowing itself is `narrow()`, never redone.
    for edge in &summary.spawn_edges {
        if let Err(msg) = &edge.narrowed_caps {
            capability_errors.push(NodeError {
                node_path: edge.child_id.clone(),
                message: format!("capability narrowing failed: {msg}"),
            });
        }
    }
    let ok = capability_errors.is_empty();
    FrameworkValidationReport {
        schema_errors: Vec::new(),
        capability_errors,
        ok,
        capability_summary: Some(summary),
    }
}
```

**Wire shape — a §12-owned technical decision the build surfaces.** Spec Phase 9 says validation "posts results back as events". `validate_framework` is shipped as a **Tauri command returning the report synchronously**, not a fire-and-forget event: continuous validation as the user edits the canvas is a request/response interaction, and the project's IPC has matured to synchronous command returns since the spec was written (M07's `import_artifact` returns `ImportOutcome` synchronously; `request_tier_transition` returns its outcome synchronously). The build owns this as a §12 technical decision, states it in the retro, and the closeout gap-analysis records the spec-phrasing refinement. (A future *push* validation event — e.g. background validation — would be a schema + ADR at that point, not now.) `FrameworkValidationReport`, `NodeError`, and the embedded `FrameworkCapabilitySummary` / `SpawnEdgeNarrowing` (B.3.4) all derive `serde::Serialize` because they cross the Tauri IPC boundary as one report.

#### B.3.3 `list_installed` — the `skills.lock` production reader

`skills_lock` (M07.B) shipped `read` / `write_entry` / `verify` + canonical serialization but **no production reader caller** — M07.V 🟡 #2 flagged `verify` has no production caller, and M07-IRL #6 flagged the Import panel does not reload installed artifacts after a restart because nothing reads `skills.lock` on startup. B ships `list_installed` — the first production `skills.lock` reader:

```rust
// crates/runtime-main/src/builder/validate.rs (or a lock.rs submodule).
use std::path::Path;

use runtime_core::generated::skills_lock::{ArtifactKind, Source};
use crate::builder::error::BuilderError;
use crate::skills_lock;

/// One installed artifact, flattened from a `skills.lock` entry for the
/// Palette / Import panel (Stage C consumes via `list_installed_artifacts`).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct InstalledArtifact {
    /// The `name@version` lock key.
    pub key: String,
    /// `skill` / `tool` / `agent` / `mcp_server`.
    pub kind: ArtifactKind,
    /// Where it was imported from (URL or local file).
    pub source: Source,
    /// RFC-3339 install timestamp (passthrough from the lock entry).
    pub installed_at: String,
}

/// Read every installed artifact from the `skills.lock` at `lock_path`.
///
/// An ABSENT lock returns `Ok(vec![])` — a framework with nothing
/// installed is valid, not an error (this is the M07-IRL #6 fix: the
/// Import panel calls this on startup and gets an empty list rather
/// than a failure). A CORRUPT lock returns `Err(BuilderError::Lock)`.
///
/// # Errors
///
/// [`BuilderError::Lock`] when the lock file exists but is corrupt /
/// not schema-valid.
pub fn list_installed(lock_path: &Path) -> Result<Vec<InstalledArtifact>, BuilderError> {
    let lock = match skills_lock::read(lock_path) {
        Ok(lock) => lock,
        // skills_lock::read returns LockError::Io(NotFound) for an
        // absent lock — treat as "nothing installed", not an error.
        Err(skills_lock::LockError::Io(e))
            if e.kind() == std::io::ErrorKind::NotFound =>
        {
            return Ok(Vec::new());
        }
        Err(other) => return Err(other.into()),
    };
    let mut out: Vec<InstalledArtifact> = lock
        .installed
        .into_iter()
        .map(|(key, entry)| InstalledArtifact {
            key,
            kind: entry.kind,
            source: entry.source,
            installed_at: entry.installed_at.to_rfc3339(),
        })
        .collect();
    // skills.lock's `installed` is a HashMap — sort for a stable
    // Palette ordering (the lock file itself is canonically sorted;
    // the in-memory map is not).
    out.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(out)
}
```

`SkillsLock.installed` is `HashMap<String, LockEntry>` (`skills-lock.rs:222`); `LockEntry` is `{ content_hash: SriHash, installed_at: DateTime<Utc>, kind: ArtifactKind, source: Source, tier_at_install: TierAtInstall, validation_report_id: ValidationReportId }` (`skills-lock.rs:173`). The absent-lock-is-empty contract is the explicit M07-IRL #6 fix: `skills_lock::read` (`skills_lock/mod.rs:61`) returns `LockError::Io` of kind `NotFound` for a missing file (its own doc comment notes "callers that *read* a lock expect it to exist" — `list_installed` is the caller that deliberately tolerates absence). The `verify`-on-load half of 🟡 #2 — recomputing the hash when an artifact is actually byte-loaded for execution — is Stage F1's Tester load path; **B ships the read, F1 ships the verify**.

#### B.3.4 `framework_capability_summary` — `crates/runtime-main/src/builder/summary.rs`

Spec Phase 9 Inspector: *"Capability summary across the entire framework (totals: file paths read/written, network hosts allowed, etc.)."* B computes it by aggregating `framework_loader/capability_map.rs` (the existing per-framework capability translation) — **reuse, do not rebuild**:

```rust
//! Whole-framework capability summary (M08 Stage B). Aggregates
//! framework_loader::capability_map; carries the per-Agent→Agent-edge
//! narrowing triple so the Inspector (E) and the canvas (D2) render one
//! consistent capability picture from one backend computation.

use runtime_core::generated::capability::CapabilityDeclaration;
use runtime_core::generated::framework::Framework;

use crate::capability::narrowing::narrow;
use crate::framework_loader::capability_map;

/// The narrowing decision for one Agent→Agent spawn edge.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SpawnEdgeNarrowing {
    /// Parent agent id.
    pub parent_id: String,
    /// Child (spawned) agent id.
    pub child_id: String,
    /// The parent's grant set.
    pub parent_caps: Vec<CapabilityDeclaration>,
    /// The child's declared grant set (pre-narrowing).
    pub child_declared_caps: Vec<CapabilityDeclaration>,
    /// The narrowed result — `narrow(parent, child_declared)`. `Err`
    /// when the child declares a capability the parent does not hold
    /// (D2 renders this edge as a red badge).
    pub narrowed_caps: Result<Vec<CapabilityDeclaration>, String>,
}

/// Whole-framework capability picture.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FrameworkCapabilitySummary {
    /// Distinct file-read globs across every agent.
    pub files_read: Vec<String>,
    /// Distinct file-write globs across every agent.
    pub files_written: Vec<String>,
    /// Distinct network hosts across every agent.
    pub network_hosts: Vec<String>,
    /// Whether any agent declares `shell: true`.
    pub any_shell: bool,
    /// The narrowing decision for every Agent→Agent spawn edge.
    pub spawn_edges: Vec<SpawnEdgeNarrowing>,
}

/// Compute the whole-framework capability summary.
#[must_use]
pub fn framework_capability_summary(fw: &Framework) -> FrameworkCapabilitySummary {
    // 1. Aggregate per-agent capability blocks (reuse capability_map).
    // 2. For every agent's `spawns[]`, build a SpawnEdgeNarrowing:
    //    parent_caps  = parent_grants_for_agent(fw, parent_id)
    //    child_caps   = parent_grants_for_agent(fw, child_id)
    //    narrowed     = narrow(&parent_caps, &child_caps)  // L2a, M05.B
    // 3. Collect distinct file/network/shell totals.
    /* … */
    todo!("aggregate; reuse capability_map + narrowing — no reimplementation")
}
```

The summary carries, **per Agent→Agent spawn edge**, the narrowing result (B.3.5) so the Inspector (E) and the canvas (D2) render **one consistent capability picture from one backend computation** — the renderer never aggregates or intersects itself. `validate_framework` (B.3.2) calls `framework_capability_summary` on its success path and embeds the result as the report's `capability_summary` field; the renderer reads it off the one `FrameworkValidationReport`, never calling a second command. `framework_capability_summary` stays `pub` so B.4.2 can unit-test it directly, but no Tauri command wraps it.

#### B.3.5 Capability narrowing in the report (Agent→Agent edges)

MVP §M8 criterion 3 + spec Phase 9: an Agent→Agent edge intersects the child's `allowed_*` with the parent's. The intersection is `capability/narrowing.rs::narrow()` (M05.B L2a) — **reuse it; do not duplicate the intersection in TS** (spec §9). `narrow(parent, proposed)` (`narrowing.rs:38`) returns `Ok(proposed.to_vec())` when every proposed declaration is subsumed by a parent grant, and `Err(NarrowingError::CapabilityNotHeldByParent { proposed })` on the first uncovered one — all-or-nothing per the spec's v0.1 semantics.

`framework_capability_summary` calls `narrow` per `spawns[]` edge and stores the `{ parent_caps, child_declared_caps, narrowed_caps }` triple on `SpawnEdgeNarrowing` (B.3.4); `narrowed_caps` is the `Result` `narrow` returned. `validate_framework` (B.3.2 step 4) then **folds every `Err` triple into `capability_errors`**, keyed to the child agent — so an over-declaring Agent→Agent edge makes the report `ok: false` and D2.3.5 badges the child node, exactly as for an unresolved-reference gap. An `Ok` triple carries `proposed` verbatim (all-or-nothing — there is no partial clamp in v0.1), so D2's `NarrowingNotice` (D2.3.6) renders the surviving set; an `Err` triple renders the rejection. Continuous validation means drawing an Agent→Agent edge re-runs `validate_framework`, which embeds `framework_capability_summary` in the report's `capability_summary` field, so the narrowing surfaces live with no separate command. The `NarrowingError` is stringified into `narrowed_caps: Err(String)` because the report crosses the IPC boundary (`NarrowingError` itself is not `Serialize`); `validate_framework` re-uses that same string in the folded `capability_error` message.

#### B.3.6 `save_framework` / `load_framework` — `crates/runtime-main/src/builder/persist.rs`

Path-agnostic persistence per CLAUDE.md §9 (the `audit::file_path` / `skills_lock` archetype): the `builder::persist` functions take `dir: &Path`; the Tauri shell resolves the directory (from the save dialog / file picker, wired in Stage C) and passes it in. **No workspace dep on `dirs`** (already transitive via Tauri). Tested with `tempfile`-backed paths — so B adds **no new coverage exclusion** (unlike M07.C's `import/fetch.rs`; the FS calls here are `&Path`-parameterised and tempfile-reachable, the `skills_lock` ≥95 precedent):

```rust
//! Framework save/load (M08 Stage B). Path-agnostic per CLAUDE.md §9 —
//! the Tauri shell resolves the directory; persist takes `&Path`. The
//! load side reuses framework_loader (Phase 6) — no second loader.

use std::path::Path;

use runtime_core::generated::framework::Framework;
use crate::builder::error::BuilderError;

/// One inline-defined artifact's companion markdown file.
#[derive(Debug, Clone)]
pub struct Companion {
    /// File name relative to `dir` — e.g. `summarize.skill.md`.
    pub file_name: String,
    /// Full markdown body (frontmatter + content).
    pub body: String,
}

/// A framework reloaded from disk — the canvas reconstructs from this.
#[derive(Debug, Clone)]
pub struct LoadedFramework {
    /// The parsed `framework.json`.
    pub framework: Framework,
    /// The companion markdown files found alongside it.
    pub companions: Vec<Companion>,
}

/// Write `framework.json` + a companion `.md` for every inline artifact.
///
/// Writes `dir/framework.json` (pretty-printed, stable field order) plus
/// one file per `companions` entry. `dir` must exist and be a directory.
///
/// # Errors
///
/// - [`BuilderError::NotADirectory`] if `dir` exists and is a file.
/// - [`BuilderError::Io`] on any write failure.
/// - [`BuilderError::Json`] if the framework cannot serialize.
pub fn save_framework(
    dir: &Path,
    fw: &Framework,
    companions: &[Companion],
) -> Result<(), BuilderError> {
    if dir.exists() && !dir.is_dir() {
        return Err(BuilderError::NotADirectory(dir.display().to_string()));
    }
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(fw)?;
    std::fs::write(dir.join("framework.json"), format!("{json}\n"))?;
    for c in companions {
        std::fs::write(dir.join(&c.file_name), &c.body)?;
    }
    Ok(())
}

/// Read `framework.json` + its companion `.md` files back from `dir`.
///
/// Reuses `framework_loader` for the `framework.json` parse — the canvas
/// reconstructs **identical** to save state (MVP §M8 criteria 7+8).
///
/// # Errors
///
/// - [`BuilderError::Io`] if `dir/framework.json` is missing/unreadable.
/// - [`BuilderError::Load`] / [`BuilderError::Json`] on a parse failure.
pub fn load_framework(dir: &Path) -> Result<LoadedFramework, BuilderError> {
    let raw = std::fs::read_to_string(dir.join("framework.json"))?;
    let framework: Framework = serde_json::from_str(&raw)?;
    let companions = read_companions(dir)?;   // scans dir for *.skill.md / *.tool.md / *.agent.md
    Ok(LoadedFramework { framework, companions })
}
```

MVP §M8 criteria 7 + 8: Save → `framework.json` + companion `.md` at the chosen path; Reload → the canvas reconstructs **identical** to save state — a save→load→save cycle is byte-stable (`to_string_pretty` + a trailing newline gives a deterministic serialization). The load side parses `framework.json` into the same `Framework` type `framework_loader` uses — **do not write a second loader**. The companion-`.md` writer is the genuinely new surface; the `LoadedFramework` shape must round-trip the canvas projection (ADR-0020, authored at Stage C) exactly.

#### B.3.7 Tauri command surface + coverage

Four thin shell commands wrap the `builder` seams. The `*_with` seam is the unit-tested core; the `#[tauri::command]` wrapper is the §5 tauri-shell holdout (`codecov.yml`'s `tauri-shell` patch gate covers `src-tauri/**`):

```rust
// src-tauri/src/commands.rs — the validate_framework command + its seam.
/// Validate an in-progress framework document — the Canvas (D2 red
/// badges) + Inspector (E) call this continuously as the user edits.
#[tauri::command]
pub fn validate_framework(doc: serde_json::Value) -> FrameworkValidationReport {
    validate_framework_with(&doc)
}

/// Test-seam for [`validate_framework`] (CLAUDE.md §5 `*_with`).
pub fn validate_framework_with(doc: &serde_json::Value) -> FrameworkValidationReport {
    runtime_main::builder::validate::validate_framework(doc)
}

// save_framework / load_framework / list_installed_artifacts follow the
// same thin-wrapper-over-*_with-seam shape; save/load resolve the &Path
// from the renderer-supplied directory string; list_installed_artifacts
// resolves <framework_root>/skills.lock.
```

The `builder` module itself is in `runtime-main` → the **≥95 gate**; it is pure / seam / `tempfile`-tested, so B adds **no new `--ignore-filename-regex` exclusion and no four-mirror coverage change** — confirm this in the retro. (If a stage surfaces a genuine OS-call holdout, the v1.8 four-mirror sync applies and the closeout `<coverage_policy_reconciliation>` verifies it — but `builder` is designed to avoid one: every FS call is `&Path`-parameterised and tempfile-reachable, exactly the `skills_lock` ≥95 precedent.) The four new commands must be registered in `src-tauri/src/main.rs`'s `invoke_handler` macro.

### B.4 Tests

Strict v1.8 two-commit TDD: the red commit carries every test below; the impl commit touches no test file. The `builder` module's tests are in-source `#[cfg(test)]` (pure-logic — `validate`, `summary`) + `crates/runtime-main/tests/builder.rs` or in-source `tempfile` tests (`persist`, `list_installed`).

#### B.4.1 `validate_framework` tests

`crates/runtime-main/src/builder/validate.rs` (`#[cfg(test)]`):

- `validate_framework_valid_framework_reports_ok`
- `validate_framework_schema_invalid_reports_schema_error_keyed_to_json_path`
- `validate_framework_unresolved_tool_ref_reports_capability_error_keyed_to_agent`
- `validate_framework_unresolved_skill_ref_reports_capability_error`
- `validate_framework_unresolved_agent_ref_reports_capability_error`
- `validate_framework_agent_to_agent_narrowing_violation_reports_capability_error_keyed_to_child` (the B.3.2 step-4 fold — an over-declaring child makes the framework invalid)
- `validate_framework_valid_agent_to_agent_edge_reports_ok_with_an_ok_narrowing_triple`
- `validate_framework_report_serializes_to_json` (crosses IPC — must `Serialize`)
- `validate_framework_valid_framework_report_carries_capability_summary` (the B.3.4 summary rides on the report — `Some`)
- `validate_framework_schema_invalid_report_has_no_capability_summary` (the early-return path — `None`, no parsed `Framework`)
- `validate_framework_called_twice_on_same_doc_returns_identical_report` (multi-call, gotcha #69)

#### B.4.2 `framework_capability_summary` + narrowing tests

`crates/runtime-main/src/builder/summary.rs` (`#[cfg(test)]`):

- `summary_aggregates_file_read_globs_across_agents`
- `summary_aggregates_network_hosts_across_agents`
- `summary_any_shell_true_when_any_agent_declares_shell`
- `summary_spawn_edge_narrowing_ok_when_child_subset_of_parent`
- `summary_spawn_edge_narrowing_err_when_child_exceeds_parent` (the red-badge case)
- `summary_spawn_edge_narrowing_triple_carries_parent_child_declared_and_narrowed`
- `summary_no_spawn_edges_produces_empty_spawn_edges_list`

#### B.4.3 `save_framework` / `load_framework` tests

`crates/runtime-main/tests/builder.rs` (or in-source `#[cfg(test)]` with `tempfile`):

- `save_framework_writes_framework_json_to_dir`
- `save_framework_writes_one_companion_md_per_inline_artifact`
- `save_framework_to_a_path_that_is_a_file_returns_not_a_directory`
- `load_framework_round_trips_a_saved_framework`
- `load_framework_recovers_companion_md_files`
- `save_load_save_cycle_is_byte_stable` (MVP §M8 criterion 8)
- `load_framework_missing_framework_json_returns_io_error`

#### B.4.4 `list_installed` tests

`crates/runtime-main/src/builder/validate.rs` (`#[cfg(test)]` with `tempfile`):

- `list_installed_reads_entries_from_a_skills_lock_fixture`
- `list_installed_absent_lock_returns_empty_not_error` (the M07-IRL #6 contract)
- `list_installed_corrupt_lock_returns_lock_error`
- `list_installed_returns_entries_sorted_by_key` (stable Palette ordering)
- `list_installed_flattens_kind_source_and_installed_at_from_lock_entry`
- `list_installed_called_twice_returns_identical_list` (multi-call, gotcha #69)

#### B.4.5 Acceptance criteria

- [ ] `cargo test --workspace` + the full v1.6 canonical gate suite green.
- [ ] `cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs" --fail-under-lines 95` — runtime-main ≥95, with the new `builder` module inside the gate; **no new exclusion**.
- [ ] `validate_framework` reuses `framework_loader::walk` + the typify-generated `Framework` shape; `framework_capability_summary` reuses `capability_map` + `narrowing::narrow` — confirmed not reimplemented (grep-verified, recorded in the retro).
- [ ] The four Tauri commands compile (`cargo check -p src-tauri`) and are registered in `main.rs`'s `invoke_handler`.
- [ ] `cargo deny check` — no new dependency (B is reuse-only; no new crate).
- [ ] No schema change; no `cargo xtask regenerate-types` needed.
- [ ] Strict v1.8 two-commit invariant proven: `git diff <red>..<impl> -- '**/tests/**'` EMPTY.
- [ ] CI-parity per G6.

### B.5 CLI Prompt

```xml
<work_stage_prompt id="M08.B">
  <context>
    M08 Stage B — the Builder backend: a new crates/runtime-main/src/builder/
    module shipping validate_framework (schema-shape + reference + capability
    validation, reusing the typify-generated Framework type + framework_loader
    + runtime-core capability — NO TS duplication, spec §9), framework
    save/load (path-agnostic &Path; framework.json + companion .md; reuses
    framework_loader for the read), the whole-framework capability summary
    (reuse framework_loader/capability_map.rs) incl. per-Agent→Agent-edge
    narrowing (reuse capability/narrowing.rs::narrow — M05.B L2a), and
    list_installed (the FIRST production skills.lock reader — M07-IRL #6 +
    the read half of M07.V 🟡 #2). Four thin Tauri command wrappers over
    the seams. B is a pure backend stage — the renderer stages C–F consume
    it; it must land first. NO renderer code, NO new schema, NO new
    coverage exclusion expected. B reads the capability surface
    (CODEOWNERS-adjacent) — it reuses but does NOT modify capability
    enforcement.
  </context>

  <read_first>
    <file>CLAUDE.md (§9 path-agnostic persistence archetype; §14 schema source-of-truth; Hard Rule 8 capability surface)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Implementation Workflow, Stage B B.1–B.4)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543 — the Canvas/Inspector validation + capability-summary requirements) + §0b (Tool/Skill/Agent) + §8.security L1/L2a</file>
    <file>docs/MVP-v0.1.md §M8 (criteria 3 narrowing, 7 save, 8 byte-stable reload)</file>
    <file>docs/style.md (path-agnostic persistence + Tauri-shell-resolves-directory archetype)</file>
    <file>docs/coverage-policy.md (§A current exclusion set — B adds none; §B baselines)</file>
    <file>docs/build-prompts/retrospectives/M08.A-retrospective.md ([END] Decisions — apply them) + RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/gotchas.md (#41 grep-verify codebase; #66 tests-pass-contract-fails; #69 multi-call; #81 llvm-cov clean)</file>
  </read_first>

  <read_reference>
    <file purpose="framework_loader module — reuse `walk` + `capabilities_to_declarations` + `parent_grants_for_agent` (re-exported here); validate_framework + summary build on these">crates/runtime-main/src/framework_loader/mod.rs</file>
    <file purpose="the M04 Layer-1 gap walker — validate_framework calls walk() for reference validation; pin the Gap struct (agent_id / missing_name / kind)">crates/runtime-main/src/framework_loader/walker.rs</file>
    <file purpose="capability_map — framework_capability_summary aggregates this; pin capabilities_to_declarations + parent_grants_for_agent signatures">crates/runtime-main/src/framework_loader/capability_map.rs</file>
    <file purpose="L2a narrowing — framework_capability_summary calls narrow(parent, proposed) per spawn edge; pin the signature + NarrowingError">crates/runtime-main/src/capability/narrowing.rs</file>
    <file purpose="skills_lock primitive — list_installed calls read(); pin read()'s LockError::Io(NotFound)-for-absent-lock behavior + the SkillsLock.installed HashMap shape">crates/runtime-main/src/skills_lock/mod.rs</file>
    <file purpose="LockError variants — BuilderError From-converts LockError; pin the Io/Parse/NotFound/HashMismatch set">crates/runtime-main/src/skills_lock/error.rs</file>
    <file purpose="generated SkillsLock + LockEntry + ArtifactKind + Source types — InstalledArtifact flattens LockEntry; pin the field set">crates/runtime-core/src/generated/skills-lock.rs</file>
    <file purpose="generated Framework type — validate_framework deserializes into it; the schema-shape check IS this deserialization">crates/runtime-core/src/generated/framework.rs</file>
    <file purpose="generated CapabilityDeclaration — the summary's narrowing triple carries Vec&lt;CapabilityDeclaration&gt;">crates/runtime-core/src/generated/capability.rs</file>
    <file purpose="audit::file_path — the path-agnostic + Tauri-shell-resolves-directory archetype builder::persist mirrors">crates/runtime-main/src/audit/file_path.rs</file>
    <file purpose="Tauri command surface — the `*_with` seam pattern (approve_plan/approve_plan_with); add the four builder commands">src-tauri/src/commands.rs</file>
    <file purpose="src-tauri main.rs — the invoke_handler macro; register the four new commands">src-tauri/src/main.rs</file>
    <file purpose="CI schema-validation job — confirm there is no Rust JSON-Schema validator (the check is a Python jsonschema script); validate_framework's schema check is serde-into-Framework">.github/workflows/ci.yml</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M08.A" decisions_file="docs/build-prompts/retrospectives/M08.A-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M08-workbench.md" section="B.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      One standalone `test(M08.B): …` commit with the failing tests across
      every B.4 bucket: validate_framework (valid / schema-invalid /
      unresolved-ref / narrowing-violation-folds-to-capability-error /
      capability-summary-rides-on-report / serializes / multi-call),
      framework_capability_summary
      + Agent→Agent narrowing triple (Ok subset / Err exceeds / no edges),
      save/load round-trip + byte-stable cycle, list_installed (present lock
      / absent-lock-empty / corrupt-lock-error / sorted / multi-call). Stub
      the production surfaces just enough to compile the test files
      (todo!() / unimplemented!() bodies fine). Confirm right-reason failure
      per CLAUDE.md §5 (assertion failed / cannot find function / unresolved
      import — NOT a test-file compile error, NOT a tautological pass — the
      builder module is wholly absent on entry). Surface the red-phase
      commit for approval before green phase.
    </red_phase>
    <green_phase>
      Seam-first implementation until ALL failing tests pass. Do NOT modify
      the test files during implementation — if a test is wrong, fix it in
      a SEPARATE labelled follow-up commit. The impl commit body MUST state
      the verifiable invariant `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      rustfmt/clippy fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message.
    </green_phase>
  </tdd_discipline>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="B.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>

  <gates milestone="M08"/>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <adr_triggers>
    <trigger>No new ADR in B (reuse-only of the capability + schema-shape + narrowing + framework_loader surfaces). If validate_framework is found to NEED a new event variant rather than a synchronous command return, that triggers §14 (schema + cargo xtask regenerate-types + ADR) — surface before red. The command-return-vs-spec-"events" decision is a §12-owned technical call, recorded in the retro, NOT an ADR. ADR-0020 (Builder canvas↔framework.json state model) is filed at Stage C, not B — but B's LoadedFramework shape must anticipate it (the round-trip is byte-stable).</trigger>
  </adr_triggers>

  <pre_flight_check>
    <check name="branch">HEAD is the M08 parent-milestone branch; the M08.A impl commit is present (git log --oneline main..HEAD includes the M08.A commits)</check>
    <check name="reuse_surfaces">grep-confirm framework_loader::walk + capabilities_to_declarations + parent_grants_for_agent + capability/narrowing.rs::narrow + skills_lock::read all exist with the signatures the phase doc cites, BEFORE wiring</check>
    <check name="no_rust_schema_validator">confirm there is no pre-existing Rust JSON-Schema validator crate (grep Cargo.toml + crates/*/Cargo.toml for jsonschema) — validate_framework's schema check is serde-into-Framework, not a runtime schema library</check>
    <check name="llvm_cov_clean">run `cargo llvm-cov clean --workspace` before the coverage gates if any prior llvm-cov ran this session (gotcha #81)</check>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="crates/runtime-main/src/builder/mod.rs" verified="false" note="B creates the builder module"/>
    <claim type="file" path="crates/runtime-main/src/framework_loader/mod.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/framework_loader/walker.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/framework_loader/capability_map.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/capability/narrowing.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/skills_lock/mod.rs" verified="true"/>
    <claim type="method" path="crates/runtime-main/src/framework_loader/walker.rs" symbol="walk" verified="true" note="walk(&amp;Framework) -> Vec&lt;Gap&gt; — re-exported at framework_loader/mod.rs"/>
    <claim type="method" path="crates/runtime-main/src/framework_loader/capability_map.rs" symbol="parent_grants_for_agent" verified="true" note="parent_grants_for_agent(&amp;Framework, &amp;str) -> Option&lt;Vec&lt;CapabilityDeclaration&gt;&gt;"/>
    <claim type="method" path="crates/runtime-main/src/capability/narrowing.rs" symbol="narrow" verified="true" note="narrow(parent: &amp;[CapabilityDeclaration], proposed: &amp;[CapabilityDeclaration]) -> Result&lt;Vec&lt;CapabilityDeclaration&gt;, NarrowingError&gt;"/>
    <claim type="method" path="crates/runtime-main/src/skills_lock/mod.rs" symbol="read" verified="true" note="read(path: &amp;Path) -> Result&lt;SkillsLock, LockError&gt;; returns LockError::Io(NotFound) for an absent lock — list_installed maps that to Ok(vec![])"/>
    <claim type="struct_field" path="crates/runtime-core/src/generated/skills-lock.rs" symbol="SkillsLock{installed,version}" verified="true" note="installed: HashMap&lt;String, LockEntry&gt;; LockEntry has content_hash/installed_at/kind/source/tier_at_install/validation_report_id"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M08.A-retrospective.md" verified="false" note="created by Stage A; present on the build machine after A executes"/>
  </phase_doc_inventory_audit>

  <schema_ref_audit>
    <ref schema="schemas/framework.v1.json" path="(root)" verified="true" note="validate_framework deserializes into the typify-generated Framework — required: [name,version,description,model,tools,skills,agents,session_root_agent]"/>
    <ref schema="schemas/skills-lock.v1.json" path="installed" verified="true" note="list_installed reads the `installed` map — spec §2200 contract key; HashMap&lt;name@version, LockEntry&gt;"/>
  </schema_ref_audit>

  <architecture_check>
    <claim description="There is no Rust JSON-Schema validator — validate_framework's schema-shape check IS serde deserialization into the typify-generated Framework type, which carries the schema's pattern + newtype constraints" verify="grep -rn 'jsonschema\|Draft202012\|schemars' Cargo.toml crates/*/Cargo.toml crates/runtime-main/src/ ; expect zero runtime schema-validator dependency"/>
    <claim description="validate_framework reuses framework_loader::walk for reference validation rather than re-walking the framework" verify="grep -n 'framework_loader\|::walk' crates/runtime-main/src/builder/validate.rs ; expect a walk() call, not a hand-rolled reference scanner"/>
    <claim description="framework_capability_summary reuses capability/narrowing.rs::narrow for the Agent→Agent intersection — the intersection is NOT duplicated" verify="grep -n 'narrowing\|narrow(' crates/runtime-main/src/builder/summary.rs ; expect a narrow() call"/>
    <claim description="builder::persist is path-agnostic — takes &amp;Path, no AppHandle / dirs dependency inside the module" verify="grep -rn 'AppHandle\|dirs::\|app_local_data_dir' crates/runtime-main/src/builder/ ; expect zero matches"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="skills_lock::" purpose="confirm list_installed is the FIRST production reader caller — prior callers are tests only (the M07.V 🟡 #2 read-half premise)"/>
    <grep pattern="capability_map::" purpose="confirm framework_capability_summary aggregates the existing capability_map rather than re-deriving the per-agent translation"/>
    <grep pattern="invoke_handler" purpose="locate the src-tauri/src/main.rs invoke_handler macro so the four new builder commands are registered"/>
  </fan_out_grep>

  <existing_pattern_audit>
    <pattern grep_for="LockError" rationale="BuilderError From-converts LockError — confirm the LockError variant set (Io / Parse / NotFound / HashMismatch) so the From impl + the list_installed absent-lock match (LockError::Io of kind NotFound) are exhaustive against the real enum" affected_files="crates/runtime-main/src/skills_lock/error.rs" remediation="pin the variant set at pre-flight; if skills_lock::read's absent-lock error is a different variant than LockError::Io(NotFound), adjust list_installed's match accordingly"/>
  </existing_pattern_audit>

  <api_breaking_change_audit>
    <change api="crates/runtime-main/src/lib.rs module surface" before_signature="no `builder` module" after_signature="adds `pub mod builder;`" call_sites="0 — B is the first" test_sites="0 — B's tests" recommendation="purely additive — new module; no existing surface changes"/>
    <change api="src-tauri/src/commands.rs command surface" before_signature="no validate_framework/save_framework/load_framework/list_installed_artifacts commands" after_signature="adds four commands + their *_with seams" call_sites="0 — renderer wires in Stages C–F" test_sites="0 — B's tests" recommendation="purely additive — register in main.rs invoke_handler; renderer consumes later"/>
  </api_breaking_change_audit>

  <interpretation_declarations>
    <adopt spec_section="§9 — validation 'posts results back as events'" interpretation="validate_framework is a Tauri command returning FrameworkValidationReport synchronously; continuous validation as the user edits the canvas is a request/response interaction" alternative_interpretation="a fire-and-forget validation event the renderer subscribes to" rationale="the project's IPC has matured to synchronous command returns since the spec was written (M07 import_artifact returns ImportOutcome synchronously; request_tier_transition returns its outcome synchronously); a request/response shape is the correct fit for continuous editor validation. §12-owned technical decision, recorded in the retro; the closeout gap-analysis records the spec-phrasing refinement. A future push validation event would be a schema + ADR at that point."/>
  </interpretation_declarations>

  <runtime_environment os="windows" note="Build on Windows (the v0.1 target). builder::persist is path-agnostic + tempfile-tested — no OS-specific path handling beyond std::path; CI runs all three OSes."/>

  <gotchas>
    <trap>Spec §9 — NO duplication of validation logic between TS and Rust. validate_framework reuses the typify-generated Framework shape + framework_loader::walk; framework_capability_summary reuses capability_map + narrowing::narrow. Reuse, do not rebuild — there is no Rust JSON-Schema library; the schema-shape check is serde-into-Framework.</trap>
    <trap>Path-agnostic persistence (CLAUDE.md §9) — builder::persist takes &amp;Path; the Tauri shell resolves the directory. No workspace dep on `dirs`. tempfile-test it → NO new coverage exclusion (the skills_lock ≥95 precedent).</trap>
    <trap>B reuses but does NOT modify capability enforcement (Hard Rule 8, CODEOWNERS-adjacent). If a change would modify capability/enforcer.rs behavior, STOP and surface the plan first — B reads the capability surface, it does not change it.</trap>
    <trap>#66 tests-pass-contract-fails — for the narrowing triple, assert it carries ALL THREE of parent / child_declared / narrowed, and that narrowed is the genuine narrow() result (Err when the child exceeds the parent), not merely a non-empty vec.</trap>
    <trap>#69 multi-call — validate_framework + list_installed each get a *_twice / *_called_twice test proving idempotent output.</trap>
    <trap>#81 — `cargo llvm-cov clean --workspace` before the coverage gates if any prior llvm-cov ran this session.</trap>
    <trap>list_installed's absent-lock contract — skills_lock::read returns LockError::Io of kind NotFound for a missing file; list_installed MUST map that single case to Ok(vec![]) and propagate every other LockError. An over-broad catch (treating a corrupt lock as empty) is a silent-failure bug.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT write a second framework loader — load_framework parses framework.json into the same typify-generated Framework type framework_loader uses. DO NOT write a second capability validator or a second narrowing intersection — reuse runtime-core capability + capability/narrowing.rs (spec §9).</warning>
    <warning>DO NOT add a runtime JSON-Schema validation crate (jsonschema, schemars, valico). The schema-shape check is serde deserialization into the typify-generated Framework type — that is the project's established schema-as-source-of-truth mechanism (CLAUDE.md §14). Adding a schema library would be a new core dependency requiring an ADR + cargo deny.</warning>
    <warning>DO NOT build any renderer code — no builderStore.ts, no Builder components. C–G consume the builder backend; B ships the backend + the four Tauri commands only. The Builder Zustand store is Stage C and is SEPARATE from graphStore.ts.</warning>
    <warning>DO NOT ship the skills_lock::verify-on-load path here — B ships list_installed (the READ half of M07.V 🟡 #2 + M07-IRL #6). The verify-on-load (integrity) half is Stage F1's Tester artifact-load path.</warning>
    <warning>DO NOT introduce a new schema or event variant. If validate_framework is found to need one, surface before the red commit — it triggers the §14 schema-regenerate-ADR flow, not a silent addition.</warning>
  </execution_warnings>

  <time_box hours="8-11"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the seam/wrapper split for each of the four commands. Confirm (grep-verified) that framework_loader::walk + capability_map + narrowing::narrow + skills_lock::read were REUSED, not reimplemented. Confirm B added NO new coverage exclusion (path-agnostic + tempfile-tested) — if a genuine OS-call holdout surfaced, note it for the closeout &lt;coverage_policy_reconciliation&gt;. Record the command-return-vs-spec-"events" §12 decision and that the closeout gap-analysis owns the spec-phrasing refinement. Note whether the LoadedFramework shape round-trips byte-stably (the ADR-0020 canvas-projection anticipation). Record list_installed as the first production skills.lock reader (closing the read half of M07.V 🟡 #2 + M07-IRL #6) and that F1 owns the verify half.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="B.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M08.*-retrospective.md)</item>
    <item>strict-TDD invariant: git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**' EMPTY</item>
    <item>the builder module seam/wrapper split + the reuse confirmation (framework_loader::walk / capability_map / narrowing::narrow / skills_lock::read — grep-verified not rebuilt)</item>
    <item>the command-return-vs-spec-"events" §12 technical decision</item>
    <item>list_installed = the first production skills.lock reader (M07-IRL #6 + the read half of M07.V 🟡 #2); F1 owns the verify half</item>
    <item>gate results (v1.6 canonical order; runtime-main ≥95 incl. the builder module; workspace ≥80; NO new coverage exclusion; the four commands compile + registered in invoke_handler; CI-parity per G6)</item>
    <item>M08.B retrospective filled-in [END] section</item>
    <item>draft commit message from B.6</item>
    <item>explicit statement: "Stage M08.B is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### B.6 Commit Message

```
feat(runtime): M08 Stage B — Builder backend (validate · persist · capability summary · skills.lock reader)

New crates/runtime-main/src/builder/ module — the single backend the
Canvas (D1/D2), Inspector (E), and Tester (F1) share:

- validate_framework: schema-shape validation (serde into the
  typify-generated Framework type — there is no Rust JSON-Schema
  library; the generated type IS the schema-as-source-of-truth check),
  reference validation (reuses framework_loader::walk), and capability
  validation. Returns FrameworkValidationReport keyed to the offending
  node / JSON-path. The whole-framework capability_summary rides on
  the report as a field (no separate command); an over-declaring
  Agent→Agent edge folds into capability_errors. No TS-side validator
  duplication (spec §9).
- save_framework / load_framework: path-agnostic &Path persistence —
  framework.json + companion skill.md/tool.md/agent.md; reload reuses
  framework_loader; a save→load→save cycle is byte-stable (MVP §M8
  criterion 8).
- framework_capability_summary: whole-framework totals aggregated from
  framework_loader/capability_map.rs, carrying per-Agent→Agent-edge
  the narrowing triple {parent, child_declared, narrowed} via the
  reused capability/narrowing.rs::narrow (M05.B L2a). Embedded in the
  validate_framework report's capability_summary field — no separate
  Tauri command.
- list_installed: the FIRST production skills.lock reader (closes
  M07-IRL #6 + the read half of M07.V 🟡 #2). An absent lock returns
  an empty list, not an error; a corrupt lock returns a Lock error.

Four thin Tauri command wrappers (validate_framework / save_framework
/ load_framework / list_installed_artifacts) over the *_with seams,
registered in the invoke_handler.

The command-return-vs-spec-"events" call is a §12 technical decision
(synchronous request/response fits continuous editor validation;
recorded in the retro). No renderer code, no new schema, no new
coverage exclusion (path-agnostic + tempfile-tested — the skills_lock
≥95 precedent).

Strict v1.8 two-commit TDD: git diff <red>..<impl> -- '**/tests/**'
EMPTY. runtime-main ≥95 on the builder module.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---
## Stage C — Builder shell + Palette + local-file picker

### C.1 Problem Statement

MVP §M8 frames v0.1 as a *workbench*: *"A novice must be able to build an agentic process from scratch using the Builder Canvas."* Spec Phase 9 specifies a three-panel build-time tool — Palette (left) / React-Flow Canvas (center) / Inspector (right) — and the Palette as Tools / Skills / Agents / HITL / Hooks tabs, filterable, drag-drop. The app today (`src/App.tsx`) renders only the live-execution layout: a flat `graph-layout` div with `GraphCanvas` + panels, subscribed to `agent_event`. There is **no build mode** — no second top-level view, no place to compose a framework.

Stage C ships the **workbench shell**: the top-level Runtime ↔ Builder view switch (the §0d architecture's "runtime + build modes"), the three-panel `BuilderShell` with a working Palette and *placeholder* Canvas and Inspector regions that later stages fill (D1/D2 the Canvas, E the Inspector), the **Builder Zustand store** (`src/lib/builderStore.ts` — new; SEPARATE from `graphStore.ts`; ADR-0020 records the `framework.json`-as-source-of-truth model), and the **native local-file picker** (`@tauri-apps/plugin-dialog`). The picker closes M07.V 🟡 #4 (the MVP §M7 acceptance-criterion gap — `ImportSource::File` was wired in the backend but had no renderer surface) and, with Stage B's `list_installed`, M07-IRL #6 (the Import panel was empty after an app restart because nothing read `skills.lock` on startup). Per the v1.8 `<wire_signature_audit>` + `<phase_doc_inventory_audit shape=…>`, the Stage B Tauri-command params and the returned TS shapes are pinned to the **shipped** Stage B wire before any component pseudocode (the M06.E / M07.E drift lesson: the phase-doc-assumed signature drifted from `commands.rs` twice).

Concrete deliverables:

1. **`src/App.tsx` — the build-mode view switch.** A `view: 'runtime' | 'builder'` state (with a `ViewSwitch` chrome control); the existing `graph-layout` JSX is extracted into a `RuntimeLayout` and conditionally rendered; `<BuilderShell/>` renders for `'builder'`. The live-graph layout, `subscribeAgentEvents` wiring, and `localStorage` `lastSessionId` replay are **unchanged** — the Builder is an additional view, not a rewrite.
2. **`src/lib/builderStore.ts` — the Builder store (ADR-0020).** A new Zustand store holding the in-progress `framework.json` document (the generated `Framework` type) as the **single source of truth**; `diskFramework` (the last saved/loaded snapshot — for E's disk-diff); `selectedNodeId`; `validation` (the `FrameworkValidationReport` from B, populated D2). Actions: `addNode` / `updateNode` / `removeNode` / `connectEdge` / `selectNode` / `replaceFramework` / `setDiskFramework` / `setValidation`. C ships the store shape + the `selectNode` / `replaceFramework` / `setDiskFramework` actions and stubs the canvas-mutation actions D1/D2 implement.
3. **`src/components/builder/BuilderShell.tsx` — the three-panel layout.** A CSS grid: `Palette` left, an empty Canvas region (a valid React-Flow drop target that D1 fills), an empty Inspector region (a stub E fills).
4. **`src/components/builder/Palette.tsx` — the five-tab Palette.** Tools / Skills / Agents / HITL / Hooks tabs; per-tab text filter; every item a native-HTML drag source (`draggable` + `dataTransfer`). Tools/Skills/Agents list built-ins + whatever `listInstalledArtifacts` returns; HITL lists the §6a trigger types; Hooks lists the §4a firing points.
5. **`src/lib/ipc.ts` — two new wrappers.** `listInstalledArtifacts(): Promise<InstalledArtifact[]>` (Stage B's `list_installed_artifacts` command) and `pickLocalArtifactFile(): Promise<string | null>` (a thin wrapper over `@tauri-apps/plugin-dialog`'s `open`). Params + return shapes PINNED via `<wire_signature_audit>` / `shape=`.
6. **`src/components/ImportPanel.tsx` — file-picker companion + `skills.lock`-on-mount reload.** A "Browse…" button beside the URL field that calls `pickLocalArtifactFile()` and imports via the existing `importArtifact('file', path, kind)` wrapper (M07.V 🟡 #4); a `useEffect` that calls `listInstalledArtifacts` on mount so installed artifacts survive a restart in the panel (M07-IRL #6).
7. **`@tauri-apps/plugin-dialog` registration.** `package.json` dependency; `src-tauri/Cargo.toml` plugin crate; the `.plugin(...)` call in the Tauri builder; the Tauri capability/allowlist entry permitting `dialog:allow-open`.
8. **ADR-0020** — Builder canvas↔`framework.json` state model. Filed this stage, `Proposed → Accepted` in the M08 PR.
9. **Renderer ≥80 (vitest)** + Playwright behavior coverage for the shell / Palette / view switch.

Not in this stage:

- Drag-**drop instantiation** — Stage C makes Palette items drag *sources*; D1 makes the canvas the drop *target* that instantiates nodes.
- The interactive **Builder Canvas** node/edge editor (D1/D2) and the **Inspector** content (E) — C ships their regions as placeholders.
- "Generate Tool / Skill / Agent" Palette buttons — M09; the M08 Palette shows installed/imported artifacts only.
- Any backend change — C consumes the Stage B commands; it adds no Rust logic beyond the `plugin-dialog` registration.

### C.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/App.tsx` | exists | Top-level `view` state + `ViewSwitch`; extract the existing `graph-layout` JSX into `RuntimeLayout`; conditionally render `RuntimeLayout` / `BuilderShell`. The live-graph layout, `subscribeAgentEvents`, and `localStorage` replay are unchanged. |
| `src/lib/builderStore.ts` | **new** | The Builder Zustand store — `framework.json` as the source of truth, the canvas a projection (ADR-0020). SEPARATE from `graphStore.ts`. |
| `src/components/builder/BuilderShell.tsx` | **new** | The three-panel layout (Palette / empty Canvas region / empty Inspector region). |
| `src/components/builder/Palette.tsx` | **new** | Tools / Skills / Agents / HITL / Hooks tabs; per-tab filter; native drag source. |
| `src/components/builder/ViewSwitch.tsx` | **new** | The Runtime ↔ Builder chrome toggle (small; could inline in `App.tsx`, kept separate for test isolation). |
| `src/lib/ipc.ts` | exists | `listInstalledArtifacts` + `pickLocalArtifactFile` wrappers; the `InstalledArtifact` interface (hand-mirrored from B's command return — the `McpServerSummary` precedent). Params PINNED via `<wire_signature_audit>`. |
| `src/components/ImportPanel.tsx` | exists | "Browse…" file-picker companion to the URL field (M07.V 🟡 #4); `listInstalledArtifacts`-on-mount reload (M07-IRL #6). |
| `src-tauri/Cargo.toml` | exists | `tauri-plugin-dialog` crate dependency. |
| `src-tauri/src/main.rs` (or `lib.rs`) | exists | `.plugin(tauri_plugin_dialog::init())` in the Tauri builder. |
| `src-tauri/capabilities/*.json` | exists | `dialog:allow-open` capability entry. |
| `package.json` | exists | `@tauri-apps/plugin-dialog` dependency. |
| `src/styles.css` | exists | Builder shell + Palette + view-switch classes (theme variables — do **not** reintroduce the M07-IRL #3 contrast bug Stage A fixed). |
| `tests/e2e/builder_shell.spec.ts` | **new** | Playwright: view switch → three-panel shell; Palette tabs / filter / draggable item. |
| `tests/unit/lib/builderStore.test.ts` | **new** | Vitest: store actions + initial-state shape. |
| `tests/unit/components/builder/Palette.test.tsx` | **new** | Vitest: tab switching, per-tab filter logic, drag-start payload. |
| `tests/unit/components/builder/BuilderShell.test.tsx` | **new** | Vitest: three regions render. |
| `tests/unit/components/builder/ViewSwitch.test.tsx` | **new** | Vitest: toggle calls `onChange`. |
| `tests/unit/components/ImportPanel.test.tsx` | exists | Vitest: extend — `skills.lock`-on-mount reload + Browse-button wiring. |
| `CHANGELOG.md` / retro | exist/new | Stage C entries. |

Effort budget: ~9–12 hours. Largest pieces are `builderStore` (the ADR-0020 shape that D1/D2/E build on — get it right once) and the `plugin-dialog` registration (a new Tauri plugin: Rust crate + builder call + capability entry, all three required or the picker silently no-ops). The shell + Palette are conventional renderer work; pattern locks from M04.E (panels) + M07.5 (`ImportPanel`) carry forward.

### C.3 Detailed Changes

#### C.3.1 Wire pinning (v1.8 discipline — BEFORE pseudocode)

Pin to the **shipped** Stage B wire, not the phase-doc assumption (the M06.E `mcp_test_connection` and M07.E `import_artifact` drift — both caught by `<wire_signature_audit>`):

- `<wire_signature_audit>` (C.5): read `src-tauri/src/commands.rs` for the committed `list_installed_artifacts` signature; record `actual_params` verbatim. B.3.3 says it takes `lock_path: &Path` resolved by the Tauri shell — confirm whether the *command* takes zero JS args (shell resolves the path internally, the likely shape) or a path arg.
- `<phase_doc_inventory_audit shape=…>` (C.5): the `InstalledArtifact` shape Stage B's command returns. B.3.3 names `Vec<InstalledArtifact>` — pin the actual field set (`key` / `kind` / `source` / `installed_at`, per B.3.3's `InstalledArtifact`) from the shipped Rust struct; it is hand-mirrored into `ipc.ts` (not schema-generated — the `McpServerSummary` / `McpTool` precedent, both documented as serde-shape mirrors in `ipc.ts`).
- `<dependency_audit_check>` (C.5): `@tauri-apps/plugin-dialog` version + the exact Rust-side registration steps (crate name, `init()` call, capability permission identifier), confirmed against the current Tauri 2.x plugin docs before wiring (CLAUDE.md §12 — web-check externally-knowable facts; plugin APIs and permission identifiers have changed across Tauri 2.x point releases).

#### C.3.2 The build-mode view switch — `src/App.tsx`

The app gains a top-level view: Runtime (the existing live-graph layout) ↔ Builder (the new `BuilderShell`). The switch is a small piece of app chrome; the existing `graph-layout` JSX is extracted **verbatim** into a `RuntimeLayout` component and conditionally rendered — the live graph's behavior, the `subscribeAgentEvents` effect, the replay-on-mount, and the `__graphStore` Playwright affordance are untouched. v0.1 is a workbench (§0d) — the Builder is a first-class view, not a hidden panel.

```tsx
// src/App.tsx — illustrative; the existing graph-layout JSX moves verbatim
// into RuntimeLayout. The useEffect / handlers / __graphStore affordance
// are UNCHANGED — only the return wraps a view switch around them.
import { ViewSwitch } from './components/builder/ViewSwitch';
import { BuilderShell } from './components/builder/BuilderShell';

export type AppView = 'runtime' | 'builder';

export function App(): JSX.Element {
  const [view, setView] = useState<AppView>('runtime');
  // … existing hasKey / running / error state + the unchanged useEffect …

  return (
    <main>
      <BudgetHeaderBar />
      <h1>Agent Runtime</h1>
      <ViewSwitch value={view} onChange={setView} />
      {view === 'runtime' ? (
        <RuntimeLayout
          hasKey={hasKey}
          running={running}
          error={error}
          onSetKey={handleSetKey}
          onSmoke={handleSmoke}
          lastSessionId={lastSessionId}
        />
      ) : (
        <BuilderShell />
      )}
    </main>
  );
}

// RuntimeLayout — the existing SetupPanel + SmokeButton + graph-layout div
// + HITLModal/Toast/RecoveryDialog/UncertaintyPrompt/SqlInspector, lifted
// out of App's return verbatim. No behavior change.
```

Extracting `RuntimeLayout` rather than threading `view` deep into the existing tree keeps the diff to App reviewable and keeps each view a clean unit. `BudgetHeaderBar` stays above the switch — budget is session-global chrome, visible in both modes (M06.5 IRL 🟡-4's budget settings live in G's Settings panel, also cross-mode).

#### C.3.3 The Builder store — `src/lib/builderStore.ts` (ADR-0020)

A **new** Zustand store, separate from `graphStore.ts` (which stays the live-execution store — overloading it would entangle build-time and run-time state, two lifecycles with nothing in common). The Builder store holds the **`framework.json` document as the single source of truth**; the canvas nodes/edges (D1/D2) are a *projection* derived from `framework`; canvas edits mutate `framework`; the JSON-tab editor (E) calls `replaceFramework` and the canvas re-derives. This realises spec §9's *"the output is the source of truth, the canvas is the editor"* and is the model M09's Generators write into. **ADR-0020** records it.

```ts
// src/lib/builderStore.ts — illustrative shape; PIN `Framework` to the
// generated src/types/framework.ts (CLAUDE.md §14 — schema-generated).
import { create } from 'zustand';
import type { Framework } from '../types/framework';
import type { FrameworkValidationReport } from './ipc'; // pinned to B's report

/** One Palette item dropped on the canvas (D1). C only emits these from
 *  drag-start; D1 consumes them in `addNode`. */
export type BuilderNodeKind = 'agent' | 'tool' | 'skill' | 'hitl' | 'hook';

export interface BuilderState {
  /** THE source of truth — the in-progress framework.json (ADR-0020). */
  framework: Framework;
  /** Last saved/loaded snapshot — E diffs `framework` against this. */
  diskFramework: Framework | null;
  /** Selected canvas node id — drives D1's inline config + E's Inspector. */
  selectedNodeId: string | null;
  /** Latest validate_framework report (B) — D2 populates; null until first run. */
  validation: FrameworkValidationReport | null;

  // ── C ships these three ──
  /** Replace the whole document (E's JSON-tab edit; load_framework). */
  replaceFramework: (fw: Framework) => void;
  /** Record the on-disk snapshot after a save/load (E's diff baseline). */
  setDiskFramework: (fw: Framework | null) => void;
  selectNode: (id: string | null) => void;

  // ── D1/D2 implement these; C ships them as typed stubs ──
  /** D1: instantiate a node from a dropped Palette item into `framework`. */
  addNode: (kind: BuilderNodeKind, ref: string, position: { x: number; y: number }) => void;
  /** D1: inline-config edit (role / model / allowed_*). */
  updateNode: (nodeId: string, patch: Record<string, unknown>) => void;
  /** D2: the four edge types → the right `framework` field. */
  connectEdge: (sourceId: string, targetId: string) => void;
  /** D1/D2: drop a node + its edges. */
  removeNode: (nodeId: string) => void;
  /** D2: store a fresh report (continuous + explicit validation). */
  setValidation: (report: FrameworkValidationReport) => void;
}

/** An empty framework.json — the cold-start document a new Builder session
 *  opens with. PIN the required-field set to schemas/framework.v1.json. */
function emptyFramework(): Framework {
  return { /* name, version, agents: [], … per the generated type's required fields */ } as Framework;
}

export const useBuilderStore = create<BuilderState>((set) => ({
  framework: emptyFramework(),
  diskFramework: null,
  selectedNodeId: null,
  validation: null,
  replaceFramework: (fw) => set({ framework: fw }),
  setDiskFramework: (fw) => set({ diskFramework: fw }),
  selectNode: (id) => set({ selectedNodeId: id }),
  // D1/D2 replace these stub bodies; shipping them typed keeps C's store
  // shape final so D1/D2 add behavior without re-shaping the store.
  addNode: () => set((s) => s),
  updateNode: () => set((s) => s),
  connectEdge: () => set((s) => s),
  removeNode: () => set((s) => s),
  setValidation: (report) => set({ validation: report }),
}));
```

Shipping `addNode` / `updateNode` / `connectEdge` / `removeNode` as **typed no-op stubs** rather than omitting them keeps the store shape final at C — D1/D2 fill the bodies without re-shaping the interface, so no later stage breaks a `useBuilderStore` selector. C's tests cover only the three actions C implements. The `framework` field is the generated `Framework` type (CLAUDE.md §14 Hard Rule 5 — schema-generated, never hand-written); `emptyFramework()` must produce a document the generated type accepts (pin its required fields against `schemas/framework.v1.json`).

#### C.3.4 The Builder shell — `src/components/builder/BuilderShell.tsx`

The three-panel layout: Palette left, Canvas center, Inspector right — a CSS grid. C ships the shell with a fully working Palette (C.3.5), an **empty Canvas region** that is a valid React-Flow drop target (D1 makes drops instantiate nodes), and an **empty Inspector region** (E fills it). Shipping the regions as placeholders filled by later stages is deliberate incremental construction (the M07.E "wired-but-pending" precedent) — not dead code: each region is a real DOM landmark with a `data-testid` D1/E later target.

```tsx
// src/components/builder/BuilderShell.tsx — illustrative.
import { Palette } from './Palette';

export function BuilderShell(): JSX.Element {
  return (
    <div className="builder-shell" data-testid="builder-shell">
      <aside className="builder-shell__palette" data-testid="builder-palette-region">
        <Palette />
      </aside>
      <section className="builder-shell__canvas" data-testid="builder-canvas-region">
        {/* D1 mounts <BuilderCanvas/> here; until then an empty drop target. */}
      </section>
      <aside className="builder-shell__inspector" data-testid="builder-inspector-region">
        {/* E mounts the Inspector here. */}
      </aside>
    </div>
  );
}
```

The `builder-shell` grid columns (`palette | canvas | inspector`) use theme CSS variables (`--node-bg` / `--node-fg` / `--node-base-border`) — never hardcoded colors (M07-IRL #3 was a contrast bug from a literal color against the dark theme; Stage A fixed the root cause, C must not reintroduce it).

#### C.3.5 The Palette — `src/components/builder/Palette.tsx`

Five tabs — **Tools / Skills / Agents / HITL / Hooks** (spec Phase 9). Tools/Skills/Agents tabs list **installed / imported** artifacts (built-ins + whatever `listInstalledArtifacts` returns — M09 later adds "Generate…" buttons; not M08). HITL lists the §6a trigger types (`per_task` / `per_epic` / `on_gap` / …); Hooks lists the §4a firing points (`pre_task` / `post_task` / `verify` / …). Each tab has a text filter (per spec: Tools by capability, Skills by tag — C ships a name-substring filter as the v0.1 baseline; capability/tag-keyed filtering is a refinement, not a separate command). Every Palette item is a native-HTML **drag source** (`draggable` + `dataTransfer` carrying the item kind + ref) — D1 makes the canvas the drop target.

```tsx
// src/components/builder/Palette.tsx — illustrative.
import { useEffect, useMemo, useState } from 'react';
import { listInstalledArtifacts, type InstalledArtifact } from '../../lib/ipc';
import type { BuilderNodeKind } from '../../lib/builderStore';

const PALETTE_TABS = ['tools', 'skills', 'agents', 'hitl', 'hooks'] as const;
type PaletteTab = (typeof PALETTE_TABS)[number];

/** Built-in trigger / firing-point lists (static — not from skills.lock). */
const HITL_TRIGGERS = ['per_task', 'per_epic', 'on_gap', 'on_uncertainty'] as const;
const HOOK_POINTS = ['pre_task', 'post_task', 'pre_epic', 'post_epic', 'verify'] as const;

interface PaletteItem {
  kind: BuilderNodeKind;
  /** The artifact ref / trigger name — D1's `addNode` second arg. */
  ref: string;
  label: string;
}

export function Palette(): JSX.Element {
  const [tab, setTab] = useState<PaletteTab>('tools');
  const [filter, setFilter] = useState('');
  const [installed, setInstalled] = useState<InstalledArtifact[]>([]);

  useEffect(() => {
    // Installed artifacts feed the Tools/Skills/Agents tabs (M07-IRL #6 —
    // the same list_installed read the ImportPanel uses).
    void listInstalledArtifacts()
      .then(setInstalled)
      .catch((e) => console.error('list_installed_artifacts error:', e));
  }, []);

  const items: PaletteItem[] = useMemo(
    () => paletteItemsForTab(tab, installed), // built-ins + installed, filtered by `tab`
    [tab, installed],
  );
  const shown = items.filter((it) => it.label.toLowerCase().includes(filter.toLowerCase()));

  return (
    <div className="builder-palette" data-testid="builder-palette">
      <nav className="builder-palette__tabs" role="tablist">
        {PALETTE_TABS.map((t) => (
          <button
            key={t}
            role="tab"
            aria-selected={t === tab}
            className={`builder-palette__tab ${t === tab ? 'builder-palette__tab--active' : ''}`}
            data-testid={`palette-tab-${t}`}
            onClick={() => setTab(t)}
          >
            {t}
          </button>
        ))}
      </nav>
      <input
        className="builder-palette__filter"
        data-testid="palette-filter"
        placeholder={`Filter ${tab}…`}
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
      />
      <ul className="builder-palette__list">
        {shown.map((it) => (
          <li
            key={`${it.kind}:${it.ref}`}
            className="builder-palette__item"
            data-testid={`palette-item-${it.ref}`}
            draggable
            onDragStart={(e) => {
              // D1's onDrop reads this payload via screenToFlowPosition.
              e.dataTransfer.setData(
                'application/x-builder-node',
                JSON.stringify({ kind: it.kind, ref: it.ref }),
              );
              e.dataTransfer.effectAllowed = 'copy';
            }}
          >
            {it.label}
          </li>
        ))}
      </ul>
    </div>
  );
}
```

The Palette is **fully functional at C** — tabs switch, the filter narrows the list, items are draggable with a well-formed `dataTransfer` payload. It is not a stub; D1 only adds the drop *target*. The `application/x-builder-node` MIME type is the contract D1's `onDrop` reads — pinning it here keeps C↔D1 decoupled (D1 doesn't need to re-derive the payload shape).

#### C.3.6 The `ipc.ts` wrappers — `listInstalledArtifacts` + `pickLocalArtifactFile`

Two new wrappers following the existing `ipc.ts` idiom (doc comment naming the backing command + provenance; `invoke` with camelCased args; a hand-mirrored interface for the non-schema-generated return — the documented `McpServerSummary` / `McpTool` precedent):

```ts
// src/lib/ipc.ts (additions).
import { open } from '@tauri-apps/plugin-dialog';

/**
 * One installed/imported artifact row. Mirrors the serde shape of
 * `runtime_main::builder::InstalledArtifact` (Stage B — NOT
 * schema-generated; the struct crosses the Tauri bridge as-is, the
 * `McpServerSummary` precedent). `list_installed_artifacts` returns
 * `InstalledArtifact[]`. PIN the field set to the Stage B-shipped struct.
 */
export interface InstalledArtifact {
  /** The `name@version` lock key. */
  key: string;
  kind: 'skill' | 'tool' | 'agent' | 'mcp_server';
  /** Where it was imported from (a URL or a local file). */
  source: unknown;
  /** RFC-3339 install timestamp. */
  installed_at: string;
}

/**
 * List artifacts recorded in `skills.lock` (Stage B `list_installed_artifacts`
 * — the first production `skills.lock` reader). The Palette + the ImportPanel
 * call this on mount so installed artifacts survive an app restart
 * (M07-IRL #6). An absent lock returns `[]` (Stage B B.4: absent → empty,
 * not error).
 */
export async function listInstalledArtifacts(): Promise<InstalledArtifact[]> {
  return await invoke<InstalledArtifact[]>('list_installed_artifacts');
}

/**
 * Open the native file picker for a local artifact file (M07.V 🟡 #4 —
 * `@tauri-apps/plugin-dialog`). Returns the chosen absolute path, or
 * `null` if the user cancelled. The caller passes the path to
 * `importArtifact('file', path, kind)` — the backend already accepts
 * `ImportSource::File`; only this renderer surface was missing.
 */
export async function pickLocalArtifactFile(): Promise<string | null> {
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Artifact', extensions: ['json', 'md'] }],
  });
  return typeof picked === 'string' ? picked : null;
}
```

`pickLocalArtifactFile` returns `string | null` rather than throwing on cancel — cancel is a normal user action, not an error (the caller short-circuits on `null`). `@tauri-apps/plugin-dialog`'s `open` returns `string | string[] | null`; `multiple: false` narrows it to `string | null`, the `typeof` guard makes that explicit for TS.

#### C.3.7 The local-file picker wiring + the Import-panel `skills.lock` reload

`ImportPanel.tsx` gains two additions, both small, both closing a named carry-forward:

```tsx
// src/components/ImportPanel.tsx — additions (illustrative; the existing
// URL form + review/blocked/installed rendering is unchanged).

// (1) M07-IRL #6 — reload installed artifacts on mount so the panel is
//     not empty after a restart. `skills.lock` is durable; nothing read it.
useEffect(() => {
  void listInstalledArtifacts()
    .then((arts) => setInstalledFromLock(arts)) // surface as `installed` rows
    .catch((e) => console.error('list_installed_artifacts error:', e));
}, []);

// (2) M07.V 🟡 #4 — a "Browse…" companion to the URL field. The backend
//     `import_artifact` already accepts ImportSource::File; only the
//     picker surface was missing.
async function handleBrowse(): Promise<void> {
  const path = await pickLocalArtifactFile();
  if (path === null) return; // user cancelled
  setSubmitting(true);
  setError(null);
  try {
    const outcome = await importArtifact('file', path, kind);
    recordImport(outcome);
  } catch (e) {
    console.error('import_artifact (file) error:', e);
    setError(unwrapCmdError(e));
  } finally {
    setSubmitting(false);
  }
}
// … <button type="button" data-testid="import-browse" onClick={() => void handleBrowse()}>
//     Browse…</button>  beside the URL input …
```

The Browse path reuses the **existing** `importArtifact` wrapper (`ImportSourceKind` already includes `'file'`) and the existing `recordImport` store action — the only new code is the picker call. The `ImportPanel` file-comment that currently reads *"Local-file import via a native picker is deferred to a future stage"* is updated — that stage is C. The same `listInstalledArtifacts` feeds the Palette's installed lists (C.3.5), so M07-IRL #6 and the Palette's installed-artifact source are one backend call. (The M07-phase-doc retroactive `<scope_change>` annotation M07.V Decision 4 floated is an M07-side concern — **not** M08 scope; M08 just ships the picker.)

### C.4 Tests

#### C.4.1 Playwright behavior — `tests/e2e/builder_shell.spec.ts`

Vite dev server, `@tauri-apps/api` **and** `@tauri-apps/plugin-dialog` module-mocked (the M02/M07 Stage E pattern; gotcha #23 — Playwright cannot drive the Tauri window or a native dialog):

- `switching_to_builder_renders_the_three_panel_shell` — click the view switch → `builder-shell` with its palette / canvas / inspector regions present.
- `switching_back_to_runtime_renders_the_live_graph` — toggle back → `graph-canvas` present, `builder-shell` gone.
- `palette_renders_five_tabs` — Tools / Skills / Agents / HITL / Hooks tabs visible.
- `clicking_a_palette_tab_switches_the_listed_items` — click Hooks → hook firing points listed.
- `filtering_a_tab_narrows_the_item_list` — type in the filter → only matching items remain.
- `a_palette_item_is_draggable` — a Palette item carries `draggable` + a well-formed `application/x-builder-node` payload.

#### C.4.2 `builderStore` unit tests — `tests/unit/lib/builderStore.test.ts`

- `initial_state_has_an_empty_framework_and_null_disk_snapshot`
- `replaceFramework_swaps_the_whole_document`
- `setDiskFramework_records_the_snapshot_for_the_inspector_diff`
- `selectNode_sets_and_clears_selectedNodeId`
- `builderStore_is_a_distinct_store_instance_from_graphStore` (the SEPARATE-store invariant — mutating one does not touch the other)

#### C.4.3 Palette unit tests — `tests/unit/components/builder/Palette.test.tsx`

- `renders_the_active_tab_items_only`
- `switching_tabs_changes_the_listed_items`
- `the_filter_input_narrows_the_list_case_insensitively`
- `installed_artifacts_from_list_installed_appear_in_the_tools_tab` (`listInstalledArtifacts` mocked)
- `dragStart_sets_the_application_x_builder_node_payload` (the C↔D1 contract)

#### C.4.4 BuilderShell + ViewSwitch unit tests

`tests/unit/components/builder/BuilderShell.test.tsx`:
- `renders_palette_canvas_and_inspector_regions`

`tests/unit/components/builder/ViewSwitch.test.tsx`:
- `renders_runtime_and_builder_options`
- `clicking_builder_calls_onChange_with_builder`

#### C.4.5 ImportPanel extension tests — `tests/unit/components/ImportPanel.test.tsx`

- `calls_list_installed_artifacts_on_mount_and_renders_the_rows` (M07-IRL #6)
- `clicking_browse_opens_the_picker_and_imports_the_chosen_file` (`pickLocalArtifactFile` mocked → resolves a path → `importArtifact('file', …)` called)
- `cancelling_the_picker_does_not_call_importArtifact` (picker resolves `null`)

#### C.4.6 Acceptance criteria

- [ ] `npm run test` — all vitest tests pass; renderer coverage ≥80% on `src/lib/builderStore.ts` + `src/components/builder/*`
- [ ] `npx tsc --noEmit` — clean (the `Framework` type is the generated `src/types/framework.ts`)
- [ ] `npx eslint .` + `npx prettier --check '**/*.{ts,tsx,js,jsx,json}'` — clean
- [ ] `npm run test:e2e -- builder_shell.spec.ts` — passes against the Vite dev server (`@tauri-apps/plugin-dialog` module-mocked)
- [ ] Every new CSS class has a corresponding rule in `src/styles.css` (gotcha #67); Builder styles use theme variables (no literal colors — M07-IRL #3)
- [ ] `@tauri-apps/plugin-dialog` registered all three places (npm dep, `Cargo.toml` + builder `.plugin(...)`, capability entry); `cargo deny check` + `npm audit --audit-level=high` clean
- [ ] ADR-0020 filed (`Proposed`); the M08 PR flips it to `Accepted`
- [ ] `<wire_signature_audit>` + `shape=` claims match the shipped Stage B `list_installed_artifacts` command
- [ ] CI-parity per G6; strict two-commit (v1.7 renderer default) — `git diff <red>..<impl> -- '**/tests/**'` EMPTY

### C.5 CLI Prompt

```xml
<work_stage_prompt id="M08.C">
  <context>
    M08 Stage C — the Builder workbench shell: a top-level Runtime↔Builder
    view switch in App.tsx (the existing live-graph layout extracted into
    RuntimeLayout, conditionally rendered, behavior UNCHANGED), the
    three-panel BuilderShell (Palette / empty Canvas region / empty
    Inspector region — placeholders D1/E fill), the NEW builderStore
    (src/lib/builderStore.ts — framework.json as the single source of
    truth, the canvas a projection; ADR-0020; SEPARATE from graphStore),
    the five-tab filterable drag-source Palette (Tools / Skills / Agents /
    HITL / Hooks), and the @tauri-apps/plugin-dialog native local-file
    picker. The picker closes M07.V 🟡 #4 (wired into the M07 ImportPanel
    as a "Browse…" companion to the URL field; the backend already accepts
    ImportSource::File); the ImportPanel + Palette call Stage B's
    list_installed_artifacts on mount so installed artifacts survive an
    app restart (M07-IRL #6). Pin the shipped Stage B wire BEFORE any
    component pseudocode. The Canvas region ships as an empty React-Flow
    drop target D1 fills; the Inspector region as a stub E fills —
    incremental construction, not dead code. Renderer ≥80 (vitest) +
    Playwright. NO backend logic beyond the plugin-dialog registration.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — the wire_signature_audit + phase_doc_inventory_audit shape= authoring-time slots)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Pre-existing legacy file inventory, Stage C C.1–C.4)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543 — the three-panel layout + the Palette tab list) + §0d (runtime + build modes) + §6a (HITL triggers) + §4a (hook firing points)</file>
    <file>docs/MVP-v0.1.md §M8 (the workbench acceptance criteria) + §M7 (the local-file-picker criterion the picker closes)</file>
    <file>docs/build-prompts/retrospectives/M08.B-retrospective.md (the [END] Decisions — apply them; the shipped list_installed_artifacts signature)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/gotchas.md (#23 Playwright cannot drive the Tauri window / native dialog; #27 vitest re-query after await; #30 unwrapCmdError; #54 window.__graphStore; #67 component+CSS contract; #75 useShallow)</file>
    <file>docs/M07-irl-findings.md (#6 — the ImportPanel-empty-after-restart finding C's skills.lock-on-mount reload closes; #3 — the contrast bug Stage A fixed, do not reintroduce)</file>
    <file>the current @tauri-apps/plugin-dialog Tauri 2.x docs — web-check the npm version, the Rust crate name, the init() call, and the capability permission identifier BEFORE wiring (CLAUDE.md §12 — plugin APIs + permission identifiers changed across Tauri 2.x point releases)</file>
  </read_first>

  <read_reference>
    <file purpose="The flat live-graph layout C wraps in a view switch — the graph-layout div + subscribeAgentEvents + localStorage replay move VERBATIM into RuntimeLayout; the __graphStore Playwright affordance is untouched">src/App.tsx</file>
    <file purpose="The existing live-execution Zustand store — C's builderStore is SEPARATE; read for the create() idiom + selector discipline, do NOT overload this store">src/lib/graphStore.ts</file>
    <file purpose="The ipc.ts wrapper idiom — doc comment naming the backing command, invoke with camelCased args, a hand-mirrored interface for the non-schema-generated return (the McpServerSummary / McpTool precedent C's InstalledArtifact follows)">src/lib/ipc.ts</file>
    <file purpose="The M07.5 Import panel — C adds a Browse… file-picker companion + a list_installed-on-mount reload; read the existing URL form + importArtifact('file', …) call site (ImportSourceKind already includes 'file')">src/components/ImportPanel.tsx</file>
    <file purpose="The shipped Stage B Tauri command surface — PIN the list_installed_artifacts signature + the InstalledArtifact return struct verbatim before writing the ipc.ts wrapper">src-tauri/src/commands.rs</file>
    <file purpose="The Tauri builder + invoke_handler — C adds .plugin(tauri_plugin_dialog::init())">src-tauri/src/main.rs</file>
    <file purpose="The existing renderer panel pattern (M04.E / M07.5) — BuilderShell + Palette mirror the section/header/list idiom">src/components/MCPServerSettings.tsx</file>
    <file purpose="The generated framework.json TS type — builderStore.framework IS this type; emptyFramework() must satisfy its required fields (CLAUDE.md §14)">src/types/framework.ts</file>
    <file purpose="The theme CSS variables (--node-bg / --node-fg / --node-fg-muted / --node-base-border) the Builder styles MUST use — no literal colors (M07-IRL #3)">src/styles.css</file>
    <file purpose="The Playwright spec idiom + @tauri-apps/api module-mock setup C extends to also mock @tauri-apps/plugin-dialog">tests/e2e</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M08.A" retro="docs/build-prompts/retrospectives/M08.A-retrospective.md"/>
    <stage id="M08.B" retro="docs/build-prompts/retrospectives/M08.B-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M08-workbench.md" section="C.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the C.4 buckets: Playwright
      (builder_shell.spec.ts — view switch → three-panel shell; Palette
      five tabs + filter + draggable-item payload) + vitest (builderStore
      actions + the distinct-store invariant; Palette tab/filter/dragStart
      logic; BuilderShell three regions; ViewSwitch onChange; the
      ImportPanel list_installed-on-mount + Browse-picker wiring). Stub
      the production surfaces just enough that the test files compile
      (empty components / typed no-op store actions are fine — the goal is
      link-time discovery, not behavior). Confirm right-reason red per
      CLAUDE.md §5 — cannot-find-module / unresolved-import / assertion
      failures, NOT test-file compile errors and NOT tautological passes.
      Commit as a STANDALONE `test(M08.C): failing tests for the Builder
      shell + Palette + file picker` commit on the M08 branch BEFORE the
      green-phase impl; the body pastes the first ~40 lines of the
      vitest/Playwright output proving the expected-failure class. Surface
      the red-phase commit; the user approves before green begins.
    </red_phase>
    <green_phase>
      Implement the view switch / builderStore / BuilderShell / Palette /
      ViewSwitch / the two ipc.ts wrappers / the ImportPanel additions /
      the plugin-dialog registration until ALL failing tests pass. Do NOT
      modify the test files during implementation — if a test is wrong,
      fix it in a SEPARATE labelled follow-up commit with explanation,
      never silently in the impl commit. The impl commit body MUST state
      the verifiable invariant: `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      prettier/eslint fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message.
    </green_phase>
  </tdd_discipline>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="C.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>

  <gates milestone="M08"/>

  <coverage_gate>
    <gate scope="renderer" target_lines="80" ignore_filename_regex="generated"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <adr_triggers>
    <trigger>ADR-0020 — the Builder canvas↔framework.json state model (framework.json as the single source of truth, the canvas a projection derived from it). Filed THIS stage from docs/adr/0000-template.md; Status Proposed → Accepted in the M08 PR before merge. This is the model D1/D2/E build on and M09's Generators write into — get the ADR text right, it is load-bearing for four downstream stages.</trigger>
    <trigger>@tauri-apps/plugin-dialog is a spec-implied dependency (the local-file picker is named in MVP §M7 as an acceptance criterion) — no ADR required, but audit it via &lt;dependency_audit_check&gt; + cargo deny check + npm audit. If the audit surfaces a license/supply-chain issue, surface before red.</trigger>
  </adr_triggers>

  <dependency_audit_check>
    <dep name="@tauri-apps/plugin-dialog" min_version="2.0" prefer_crates_io_name="true" source_authority="web-check the current Tauri 2.x plugin docs for the npm version + the Rust crate name (tauri-plugin-dialog) + the capability permission identifier" required_features="N/A" audit="new renderer + Rust dependency — spec-implied (MVP §M7 file picker); must pass cargo deny check (no GPL/AGPL) + npm audit --audit-level=high"/>
  </dependency_audit_check>

  <wire_signature_audit>
    <wrapper ipc_command="list_installed_artifacts" actual_params="PIN from the Stage B-shipped src-tauri/src/commands.rs signature — confirm whether the command takes zero JS args (the Tauri shell resolves the skills.lock path internally per B.3.5's path-agnostic archetype) or a path arg" phase_doc_assumed="zero JS args; the shell resolves AppHandle::path().app_local_data_dir().join('skills.lock') — author CONFIRMS against the shipped signature"/>
  </wire_signature_audit>

  <existing_pattern_audit>
    <pattern grep_for="graph-layout" rationale="the existing App.tsx live-graph JSX must move VERBATIM into RuntimeLayout — pin the exact element set (SetupPanel / SmokeButton / graph-layout div / HITLModal / HITLToast / RecoveryDialog / UncertaintyPrompt / SqlInspector) so the extraction is behavior-preserving" affected_files="src/App.tsx" remediation="extract, do not rewrite; the useEffect / handlers / __graphStore affordance stay unchanged"/>
    <pattern grep_for="useShallow" rationale="derived array/object selectors over a Zustand store need useShallow (gotcha #75) — the Palette's installed-list selector + any builderStore derived selector follow the ImportPanel precedent" affected_files="src/components/builder/Palette.tsx" remediation="wrap derived selectors; primitive selectors do not need it"/>
  </existing_pattern_audit>

  <zustand_selector_audit>
    <selector store="builderStore" slice="framework / diskFramework / selectedNodeId / validation" requires_use_shallow="false" import_path="zustand/react/shallow" note="C ships primitive + whole-object selectors; D1/D2 add derived canvasNodes/canvasEdges selectors that DO need useShallow"/>
  </zustand_selector_audit>

  <pre_flight_check>
    <check name="branch" gate="git rev-parse --abbrev-ref HEAD must equal the M08 parent-milestone branch; the M08.B impl commit must be present in git log --oneline main..HEAD"/>
    <check name="stage_b_wire" gate="grep-confirm the Stage B list_installed_artifacts command + the InstalledArtifact return struct are shipped in src-tauri/src/commands.rs + crates/runtime-main/src/builder/ before pinning the ipc.ts wrapper"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="src/App.tsx" verified="true" note="C extracts RuntimeLayout + adds the view switch"/>
    <claim type="file" path="src/lib/graphStore.ts" verified="true" note="read-only reference — builderStore is SEPARATE"/>
    <claim type="file" path="src/lib/ipc.ts" verified="true" note="C adds listInstalledArtifacts + pickLocalArtifactFile"/>
    <claim type="file" path="src/components/ImportPanel.tsx" verified="true" note="C adds the Browse… companion + the skills.lock-on-mount reload"/>
    <claim type="file" path="src/components/builder/BuilderShell.tsx" verified="false" note="C creates"/>
    <claim type="file" path="src/components/builder/Palette.tsx" verified="false" note="C creates"/>
    <claim type="file" path="src/lib/builderStore.ts" verified="false" note="C creates"/>
    <claim type="store_slot" path="src/lib/builderStore.ts" symbol="framework" shape="the generated Framework type from src/types/framework.ts — PIN its required-field set; emptyFramework() must satisfy it" verified="false" note="C creates; ADR-0020 records the source-of-truth model"/>
    <claim type="command" path="src-tauri/src/commands.rs" symbol="list_installed_artifacts" verified="true" note="Stage B shipped it — C pins the signature via wire_signature_audit"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M08.B-retrospective.md" verified="true"/>
  </phase_doc_inventory_audit>

  <runtime_environment os="windows" note="Vitest + Playwright run against the Vite dev server; @tauri-apps/api AND @tauri-apps/plugin-dialog are module-mocked (gotcha #23 — Playwright cannot drive the native file dialog). The plugin-dialog Rust registration is compile-checked on all three CI OSes."/>

  <gotchas>
    <trap>#23 — Playwright drives the Vite dev server with @tauri-apps/api AND @tauri-apps/plugin-dialog module-mocked; it cannot drive the native file dialog. The picker's behavior is covered by the vitest ImportPanel test (mock pickLocalArtifactFile); Playwright covers the shell/Palette only.</trap>
    <trap>builderStore is SEPARATE from graphStore — a NEW create() store. Do not overload the live-execution store with build-time state; the distinct-store invariant is a named C.4.2 test.</trap>
    <trap>M07-IRL #3 contrast — the new Builder styles use the theme variables (--node-bg / --node-fg / --node-fg-muted / --node-base-border); do not reintroduce the low-contrast literal-color bug Stage A fixed.</trap>
    <trap>The Canvas region ships EMPTY (a valid React-Flow drop target D1 fills); the Inspector region ships as a stub E fills — incremental construction with real data-testid landmarks, not dead code.</trap>
    <trap>#67 — every new className gets a corresponding src/styles.css rule + a static test confirming it; the builder-shell grid + builder-palette classes all need rules.</trap>
    <trap>#75 — the Palette's derived installed-list selector + any derived builderStore selector use useShallow (zustand/react/shallow); a naive filter/map selector re-renders every commit.</trap>
    <trap>@tauri-apps/plugin-dialog needs THREE registrations or the picker silently no-ops: the npm dep, the Rust crate + the builder .plugin(tauri_plugin_dialog::init()) call, and the Tauri capability/allowlist permission entry. Missing any one fails differently — confirm all three.</trap>
    <trap>The generated Framework type is schema-generated (CLAUDE.md §14 Hard Rule 5) — emptyFramework() must produce a value the type accepts; do NOT hand-write a divergent framework shape.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT rewrite the live-graph layout — extract the existing App.tsx graph-layout JSX VERBATIM into a RuntimeLayout component. The live graph's behavior, the subscribeAgentEvents effect, the replay-on-mount, and the __graphStore Playwright affordance must be byte-for-byte unchanged. C adds a view; it does not touch the runtime view.</warning>
    <warning>DO NOT add backend logic in C — the only Rust change is the @tauri-apps/plugin-dialog registration (Cargo.toml + the builder .plugin call + the capability entry). C consumes Stage B's commands; it ships no new command and no new builder-module code.</warning>
    <warning>DO NOT overload graphStore with Builder state — builderStore is a separate store. The two have disjoint lifecycles (build-time vs run-time); conflating them is the dual-purpose-store anti-pattern.</warning>
    <warning>DO NOT ship the Canvas region with a partial node editor — D1 owns the interactive canvas. C ships the region as an empty drop target with a data-testid only. Shipping a half-canvas at C crosses the C/D1 stage boundary.</warning>
    <warning>DO NOT build a second skills.lock reader — list_installed_artifacts (Stage B) is the production reader; the Palette + the ImportPanel both call the same ipc.ts wrapper over it.</warning>
  </execution_warnings>

  <time_box hours="9-12"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the PINNED list_installed_artifacts params + the InstalledArtifact field set (from the shipped Stage B code — flag any drift from this phase doc's assumption). Confirm builderStore is a separate store from graphStore + holds framework.json as the single source of truth (ADR-0020). Confirm the file picker closes M07.V 🟡 #4 (wired into the ImportPanel) and the skills.lock-on-mount reload closes M07-IRL #6. Record the @tauri-apps/plugin-dialog version + the three registration points + the cargo deny / npm audit result. Note whether emptyFramework() satisfied the generated Framework type without friction. Note the RuntimeLayout extraction — was the App.tsx graph-layout JSX cleanly liftable, or did the extraction surface coupling?</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="C.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state — git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M08.*-retrospective.md</item>
    <item>strict-TDD proof — git diff &lt;red-sha&gt;..&lt;impl-sha&gt; -- '**/tests/**' is EMPTY</item>
    <item>diff stat</item>
    <item>wire_signature_audit + phase_doc_inventory_audit shape= reconciliation against the shipped Stage B list_installed_artifacts wire</item>
    <item>ADR-0020 (the Builder canvas↔framework.json state model) filed as Proposed</item>
    <item>@tauri-apps/plugin-dialog — version + the three registration points + cargo deny / npm audit result</item>
    <item>gate results (v1.6 canonical order; renderer vitest ≥80%; Playwright green; every-class-has-CSS-rule confirmation; CI-parity per G6)</item>
    <item>M08.C retrospective [END] section</item>
    <item>draft commit message from C.6</item>
    <item>explicit statement: "Stage M08.C is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### C.6 Commit Message

```
feat(renderer): M08 Stage C — Builder shell + Palette + local-file picker

The workbench shell that makes v0.1 a workbench rather than a runtime
spectator: a top-level Runtime↔Builder view switch (the existing
live-graph layout extracted verbatim into RuntimeLayout, conditionally
rendered, behavior unchanged), the three-panel BuilderShell, the new
builderStore, the five-tab filterable drag-source Palette, and the
@tauri-apps/plugin-dialog native local-file picker.

Components:
- src/App.tsx: a `view` state + ViewSwitch; the graph-layout JSX lifted
  into RuntimeLayout. The subscribeAgentEvents effect, replay-on-mount,
  and the __graphStore affordance are untouched.
- src/lib/builderStore.ts (new): the Builder Zustand store — holds
  framework.json as the single source of truth, the canvas a projection
  (ADR-0020). SEPARATE from graphStore. C ships the store shape +
  replaceFramework / setDiskFramework / selectNode; D1/D2 fill the
  canvas-mutation actions (shipped as typed stubs so the shape is final).
- src/components/builder/BuilderShell.tsx (new): the three-panel grid —
  Palette / empty Canvas region (a drop target D1 fills) / empty
  Inspector region (a stub E fills).
- src/components/builder/Palette.tsx (new): Tools/Skills/Agents/HITL/
  Hooks tabs; per-tab filter; every item a native drag source carrying
  an application/x-builder-node payload (the C↔D1 contract).
- src/components/builder/ViewSwitch.tsx (new): the Runtime↔Builder
  chrome toggle.

IPC:
- src/lib/ipc.ts: listInstalledArtifacts (Stage B's list_installed_
  artifacts — the first production skills.lock reader) + pickLocalArtifactFile
  (a thin @tauri-apps/plugin-dialog wrapper). InstalledArtifact is a
  hand-mirrored serde shape (the McpServerSummary precedent).

Carry-forwards closed:
- M07.V 🟡 #4 — the local-file picker, wired into ImportPanel as a
  "Browse…" companion to the URL field (the backend already accepted
  ImportSource::File; only the renderer surface was missing).
- M07-IRL #6 — ImportPanel + Palette call list_installed_artifacts on
  mount so installed artifacts survive an app restart.

@tauri-apps/plugin-dialog registered three places: npm dep, src-tauri
Cargo.toml + the builder .plugin() call, and the Tauri capability entry.

ADR-0020 (Builder canvas↔framework.json state model) Proposed.

v1.8 wire_signature_audit + phase_doc_inventory_audit shape= pinned to
the shipped Stage B wire. Playwright + vitest; renderer ≥80. Strict
two-commit TDD: '**/tests/**' diff EMPTY.

Not in this stage: drag-DROP instantiation + the interactive Canvas
(D1/D2), the Inspector content (E), "Generate…" Palette buttons (M09).

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---
## Stage D1 — Builder Canvas: node editor

### D1.1 Problem Statement

Spec Phase 9: *"Drag palette items onto the canvas; React Flow renders nodes per §3 Phase 3 conventions … Configure each node inline (right-click for properties) … Capability disclosure (§8.security L1) rendered live below each node — plain English, derived from declared `allowed_*`."* MVP §M8 criterion 1: *"User drags an Agent node onto empty canvas; sets role, model, `allowed_tools`/`skills` via inline properties; capability disclosure renders below in plain English."*

Stage C shipped the workbench shell with an **empty Canvas region** — a valid React-Flow drop target with a `data-testid` landmark, nothing more. D1 makes it an **interactive node editor**: a Palette item dragged onto the canvas instantiates a node into `builderStore.framework` (the source of truth — ADR-0020) and the canvas projection re-derives; each node is configurable inline (role / model / `allowed_*`); the per-node plain-English capability disclosure renders live below each node. D1 is the **node half** of the Canvas; **D2 is edges + capability narrowing + continuous validation**. The D1/D2 split (the M05.C1/C2 precedent) is at the node/edge boundary — D1 makes nodes reachable on the canvas, D2 connects them — because one D stage would span drag-drop instantiation + inline config + four edge types + the spec-critical narrowing rule + continuous validation, too wide for one coherent red→green unit.

D1 is **pure renderer** — it mutates `builderStore` and reads the generated `Framework` type; it needs **no new backend** (Stage B's `validate_framework` is D2's concern; D1's canvas is unvalidated until D2 wires continuous validation).

Concrete deliverables:

1. **`src/components/builder/BuilderCanvas.tsx`** — a new interactive React-Flow component, distinct from the read-only `GraphCanvas.tsx`. A drop target (`onDrop` / `onDragOver`); renders nodes from the `builderStore.framework` projection; supports node selection (`onNodeClick` → `builderStore.selectNode`). Module-level `builderNodeTypes` map (the `GraphCanvas` `nodeTypes` trap — redefining per render re-mounts every node).
2. **`src/components/builder/nodes/*.tsx`** — interactive Builder node components: `BuilderAgentNode`, `BuilderToolNode`, `BuilderSkillNode`, `BuilderHitlNode`, `BuilderHookNode`. They reuse the §3 visual CSS (the `agent-node` / `tool-node` class families) but are **new** components — the read-only live-graph nodes are not editable; the `<existing_pattern_audit>` pins reuse-vs-new per node.
3. **The `framework` → canvas-projection derivation** — `builderStore` exposes `canvasNodes` / `canvasEdges` selectors that derive React-Flow `Node[]` / `Edge[]` from `framework.agents[]` + the inline tool/skill/hook references. D1 implements the node side of the projection; D2 adds edges.
4. **`builderStore.addNode`** — the C-stubbed action, implemented: instantiate a Palette item into `framework` (an Agent → an `agents[]` entry; a Tool/Skill/Hook → an available-artifact entry a D2 edge later wires into an agent). Plus a `nodePositions` slot so user-placed nodes keep manual coordinates (the projection carries position).
5. **`src/components/builder/NodeConfigPanel.tsx`** (or an inline popover) — the inline node-configuration surface (spec Phase 9 "right-click for properties"): for an Agent, `role` (text), `model` (the Anthropic model dropdown), `allowed_tools` / `allowed_skills` (editable lists). Every edit calls `builderStore.updateNode`.
6. **`builderStore.updateNode`** — the C-stubbed action, implemented: apply an inline-config patch to the node's entry in `framework`.
7. **Per-node capability disclosure** — **reuse** the M07.E plain-English capability-disclosure surface (the `import-capability-disclosure` list pattern, itself the M05 §8.security L1 disclosure reused) to render each node's declared `allowed_*` in plain English below the node. Derived live from `framework`, so an inline-config edit updates the disclosure immediately.
8. **Renderer ≥80 (vitest)** + Playwright behavior coverage demonstrating MVP §M8 criterion 1 end-to-end.

Not in this stage:

- **Edges** — the four edge types (Agent→Skill / Agent→Tool / Agent→Agent / Hook→Task), `onConnect`, and edge rendering are D2. D1's `BuilderCanvas` ships `onConnect` unset.
- **Capability narrowing** — the Agent→Agent intersection (D2.3.3) and the narrowed-caps reflection in a child's disclosure are D2.
- **Continuous validation + red badges** — `validateFramework`, the debounced trigger, and per-node error badges are D2. D1's nodes carry no validation state.
- **The Inspector** — the `framework.json` preview / disk-diff / Validate/Test buttons are Stage E.
- **Any backend change** — D1 is pure renderer over `builderStore`.

### D1.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/components/builder/BuilderCanvas.tsx` | **new** | The interactive React-Flow editor — drop target, node rendering from the `builderStore.framework` projection, node selection. Distinct from the read-only `GraphCanvas.tsx`. |
| `src/components/builder/nodes/BuilderAgentNode.tsx` | **new** | Interactive Agent node — reuses the `agent-node` CSS; renders the inline capability disclosure; opens the config panel on select/right-click. |
| `src/components/builder/nodes/BuilderToolNode.tsx` | **new** | Interactive Tool node — reuses the `tool-node` CSS. |
| `src/components/builder/nodes/BuilderSkillNode.tsx` | **new** | Interactive Skill node — reuses the `skill-node` CSS. |
| `src/components/builder/nodes/BuilderHitlNode.tsx` | **new** | Interactive HITL node — reuses the `hitl-node` CSS. |
| `src/components/builder/nodes/BuilderHookNode.tsx` | **new** | Interactive Hook node — reuses the `hook-node` CSS. |
| `src/components/builder/NodeConfigPanel.tsx` | **new** | Inline node configuration (role / model / `allowed_*` editable lists). |
| `src/lib/builderStore.ts` | exists | Implement `addNode` / `updateNode` (C-stubbed); add the `canvasNodes` / `canvasEdges` projection selectors + the `nodePositions` slot. |
| `src/styles.css` | exists | Builder canvas + Builder node + config-panel classes (theme variables — M07-IRL #3). |
| `tests/e2e/builder_canvas_nodes.spec.ts` | **new** | Playwright: drag an Agent onto the canvas → configure → disclosure (MVP §M8 criterion 1). |
| `tests/unit/lib/builderStore.test.ts` | exists | Vitest: extend — `addNode` / `updateNode` + the projection derivation. |
| `tests/unit/components/builder/BuilderCanvas.test.tsx` | **new** | Vitest: the drop handler + node rendering from the projection. |
| `tests/unit/components/builder/nodes/BuilderAgentNode.test.tsx` | **new** | Vitest: the Agent node + its capability disclosure. |
| `tests/unit/components/builder/NodeConfigPanel.test.tsx` | **new** | Vitest: the config fields → `updateNode`. |
| `CHANGELOG.md` / retro | exist/new | Stage D1 entries. |

Effort budget: ~10–13 hours. The largest pieces are the `framework`→canvas projection (the derivation D2 + E both build on — the inverse of E's canvas→JSON binding, so get the shape coherent) and the five Builder node components (interactive, but the §3 CSS reuse keeps each small). Drag-drop via React-Flow's `screenToFlowPosition` + native `dataTransfer` is conventional; the M05 disclosure-component reuse means no plain-English logic to write.

### D1.3 Detailed Changes

#### D1.3.1 Existing-pattern pinning (v1.8 discipline — BEFORE pseudocode)

- `<existing_pattern_audit>` (D1.5): read `src/components/GraphCanvas.tsx` + `src/components/nodes/*.tsx` — pin the React-Flow `nodeTypes` **module-level** convention (the existing `GraphCanvas` comment: "Defined OUTSIDE the component … redefining it on each render forces React Flow to re-mount every node"), the node-component prop shape (`NodeProps<TNode>` with a typed `data`), and the §3 CSS classes (`agent-node` / `agent-node--<status>` / `agent-node__name` / `agent-node__id`; `tool-node`; `skill-node`; etc.). The Builder nodes **reuse the visual CSS** but are **interactive** (the read-only live-graph nodes have no inline config) — default to **new** `builder/nodes/*.tsx` components; conflating read-only and editable modes in one component is the dual-mode complexity to avoid.
- `<existing_pattern_audit>`: pin the **plain-English capability-disclosure surface** — the M05 §8.security L1 disclosure that M07.E reused as the `import-capability-disclosure` list in `ImportPanel.tsx` (`describeProvenance` / the `<ul className="import-capability-disclosure">` rendering). D1.3.5 reuses it a third time — confirm the exact component/render shape before authoring, do not rebuild plain-English disclosure.

#### D1.3.2 The Builder Canvas — `src/components/builder/BuilderCanvas.tsx`

A **new** React-Flow component, interactive — not the read-only `GraphCanvas.tsx` (which keeps its dagre auto-layout + `MiniMap` + read-only behavior for the live-execution view). The Builder Canvas renders nodes from the `builderStore.framework` projection (ADR-0020 — the canvas is *derived* from the framework document), is a drop target (D1.3.3), and supports node selection (selection drives the inline config panel +, later, Stage E's Inspector). It must be wrapped in `ReactFlowProvider` so `useReactFlow().screenToFlowPosition` is available to the drop handler:

```tsx
// src/components/builder/BuilderCanvas.tsx — illustrative.
import {
  Background,
  Controls,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  type NodeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { useShallow } from 'zustand/react/shallow';
import { useBuilderStore, type BuilderNodeKind } from '../../lib/builderStore';
import { BuilderAgentNode } from './nodes/BuilderAgentNode';
import { BuilderToolNode } from './nodes/BuilderToolNode';
import { BuilderSkillNode } from './nodes/BuilderSkillNode';
import { BuilderHitlNode } from './nodes/BuilderHitlNode';
import { BuilderHookNode } from './nodes/BuilderHookNode';

// Module-level per @xyflow/react v12 docs + the GraphCanvas.tsx trap:
// nodeTypes is a stable-reference map; redefining it per render re-mounts
// every node. The trap re-applies for the Builder nodeTypes — keep it here.
const builderNodeTypes: NodeTypes = {
  agent: BuilderAgentNode as NodeTypes[string],
  tool: BuilderToolNode as NodeTypes[string],
  skill: BuilderSkillNode as NodeTypes[string],
  hitl: BuilderHitlNode as NodeTypes[string],
  hook: BuilderHookNode as NodeTypes[string],
};

const DND_MIME = 'application/x-builder-node'; // the C↔D1 contract (Palette dragStart)

function BuilderCanvasInner(): JSX.Element {
  // Derived projection selectors — useShallow so the canvas re-renders
  // only when the projected slice changes (gotcha #75; the GraphCanvas
  // selector-discipline precedent). canvasEdges is empty until D2.
  const nodes = useBuilderStore(useShallow((s) => s.canvasNodes()));
  const edges = useBuilderStore(useShallow((s) => s.canvasEdges()));
  const addNode = useBuilderStore((s) => s.addNode);
  const selectNode = useBuilderStore((s) => s.selectNode);
  const { screenToFlowPosition } = useReactFlow();

  function onDragOver(e: React.DragEvent): void {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  }

  function onDrop(e: React.DragEvent): void {
    e.preventDefault();
    const raw = e.dataTransfer.getData(DND_MIME);
    if (raw.length === 0) return; // not a Palette drag
    const { kind, ref } = JSON.parse(raw) as { kind: BuilderNodeKind; ref: string };
    // screenToFlowPosition converts the cursor point to canvas coords so
    // the node lands where the user dropped it.
    const position = screenToFlowPosition({ x: e.clientX, y: e.clientY });
    addNode(kind, ref, position); // mutates framework; the projection re-derives
  }

  return (
    <div className="builder-canvas" data-testid="builder-canvas" onDrop={onDrop} onDragOver={onDragOver}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={builderNodeTypes}
        onNodeClick={(_, node) => selectNode(node.id)}
        onPaneClick={() => selectNode(null)}
        /* onConnect is D2 — left unset here */
        fitView
      >
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}

export function BuilderCanvas(): JSX.Element {
  // ReactFlowProvider so screenToFlowPosition is available to onDrop.
  return (
    <ReactFlowProvider>
      <BuilderCanvasInner />
    </ReactFlowProvider>
  );
}
```

D1's `BuilderCanvas` deliberately leaves `onConnect` **unset** — edges are D2. It mounts inside Stage C's `builder-canvas-region` (replacing C's empty placeholder). It does **not** reuse `GraphCanvas.tsx`'s dagre `useMemo` — Builder nodes keep **user-placed manual positions** (the projection carries each node's `position` from `nodePositions`); an optional auto-layout affordance reusing `src/lib/layout.ts` is a possible refinement, not D1 scope.

#### D1.3.3 Drag-drop instantiation (Palette → canvas → `builderStore`)

A Palette item — Stage C made every Palette item a drag source carrying an `application/x-builder-node` payload — dropped on the canvas instantiates a node. The drop handler (D1.3.2) reads the payload, uses React-Flow's `screenToFlowPosition` to convert the cursor point to canvas coordinates, then calls `builderStore.addNode(kind, ref, position)`. `addNode` mutates `framework` (the source of truth) and records the position; the canvas projection re-derives:

```ts
// src/lib/builderStore.ts — addNode + the projection, illustrative.
import type { Node, Edge } from '@xyflow/react';

interface BuilderState {
  // … framework / diskFramework / selectedNodeId / validation (C) …
  /** User-placed node coordinates, keyed by node id — the projection
   *  reads these so dropped nodes keep where the user put them. */
  nodePositions: Record<string, { x: number; y: number }>;

  /** Derive the React-Flow node array from `framework` + `nodePositions`. */
  canvasNodes: () => Node[];
  /** Derive the React-Flow edge array — empty in D1, D2 fills it. */
  canvasEdges: () => Edge[];
}

// addNode — implemented (C shipped the typed stub):
addNode: (kind, ref, position) =>
  set((s) => {
    const nodeId = `${kind}:${ref}`;
    if (nodeId in s.nodePositions) return s; // idempotent on re-drop
    let framework = s.framework;
    if (kind === 'agent') {
      // An Agent → a new agents[] entry. ref is the agent name/id.
      framework = {
        ...s.framework,
        agents: [
          ...s.framework.agents,
          { id: ref, role: '', model: DEFAULT_MODEL, allowed_tools: [], allowed_skills: [] },
        ],
      };
    }
    // A Tool/Skill/Hook drop adds an available artifact a D2 edge wires
    // into an agent's allowed_* — it does not mutate agents[] here.
    return { framework, nodePositions: { ...s.nodePositions, [nodeId]: position } };
  }),

// canvasNodes — the framework → React-Flow projection (node side):
canvasNodes: () => {
  const s = get();
  return s.framework.agents.map((a) => ({
    id: `agent:${a.id}`,
    type: 'agent',
    position: s.nodePositions[`agent:${a.id}`] ?? { x: 0, y: 0 },
    data: { agentId: a.id, role: a.role, model: a.model,
            allowedTools: a.allowed_tools, allowedSkills: a.allowed_skills },
  }));
  // Tool/Skill/Hook nodes added to the projection the same way once D2's
  // edges make their framework placement meaningful.
},
```

Dropping an **Agent** adds an `agents[]` entry — MVP §M8 criterion 1's *"drags an Agent node onto empty canvas"* is the headline path D1 must demonstrate. A Tool/Skill/Hook drop records an available artifact that a D2 edge later wires into an agent's `allowed_*`. `addNode` is **idempotent** on re-drop of the same item (keyed by `${kind}:${ref}`) — re-dropping does not duplicate the `agents[]` entry. The projection (`canvasNodes`) is a pure function of `framework` + `nodePositions`, called through a `useShallow` selector so the canvas re-renders only on a real projection change.

#### D1.3.4 Inline node configuration — `src/components/builder/NodeConfigPanel.tsx`

Selecting a node (`onNodeClick` → `selectNode`) — or right-clicking it (spec Phase 9 "right-click for properties") — opens the inline configuration surface. For an Agent node: `role` (text input), `model` (a dropdown over the Anthropic model set), `allowed_tools` / `allowed_skills` (editable lists — D2's drag-to-connect edges are the alternative; both mutate the same `framework` field). Every edit calls `builderStore.updateNode`, which patches the node's entry in `framework`:

```tsx
// src/components/builder/NodeConfigPanel.tsx — illustrative.
import { useBuilderStore } from '../../lib/builderStore';

/** The Anthropic model set offered in the model dropdown — v0.1 is
 *  Anthropic-only (§0d); keep this list aligned with the provider. */
const MODELS = ['claude-opus-4', 'claude-sonnet-4', 'claude-haiku-4'] as const;

export function NodeConfigPanel(): JSX.Element | null {
  const selectedNodeId = useBuilderStore((s) => s.selectedNodeId);
  const framework = useBuilderStore((s) => s.framework);
  const updateNode = useBuilderStore((s) => s.updateNode);

  if (selectedNodeId === null) return null; // nothing selected — panel hidden
  const agent = framework.agents.find((a) => `agent:${a.id}` === selectedNodeId);
  if (agent === undefined) return null; // D1 configures Agent nodes; D2 widens

  return (
    <div className="builder-node-config" data-testid="builder-node-config" role="group">
      <h3 className="builder-node-config__title">Configure {agent.id}</h3>
      <label className="builder-node-config__field">
        <span>Role</span>
        <input
          data-testid="node-config-role"
          value={agent.role}
          onChange={(e) => updateNode(selectedNodeId, { role: e.target.value })}
        />
      </label>
      <label className="builder-node-config__field">
        <span>Model</span>
        <select
          data-testid="node-config-model"
          value={agent.model}
          onChange={(e) => updateNode(selectedNodeId, { model: e.target.value })}
        >
          {MODELS.map((m) => (
            <option key={m} value={m}>{m}</option>
          ))}
        </select>
      </label>
      {/* allowed_tools / allowed_skills editable lists — add/remove rows;
          each mutation calls updateNode with the new array. D2's edges
          write the same framework fields. */}
    </div>
  );
}
```

```ts
// src/lib/builderStore.ts — updateNode, implemented (C shipped the stub):
updateNode: (nodeId, patch) =>
  set((s) => {
    const agentId = nodeId.replace(/^agent:/, '');
    return {
      framework: {
        ...s.framework,
        agents: s.framework.agents.map((a) =>
          a.id === agentId ? { ...a, ...patch } : a,
        ),
      },
    };
  }),
```

The config surface is plain renderer state over the store — **no backend**. Every field edit flows `onChange` → `updateNode` → `framework` mutation → the canvas projection re-derives → the node + its capability disclosure (D1.3.5) re-render. The `NodeConfigPanel` mounts in the `BuilderShell` (beside the canvas, or as an inline popover anchored to the node — the build picks the cleaner layout; both satisfy "inline properties"). It renders `null` when nothing is selected.

#### D1.3.5 Per-node capability disclosure (reuse the M05 / M07.E disclosure surface)

Below each node, the **plain-English capability-disclosure surface** renders the node's declared `allowed_*` in plain English (MVP §M8 criterion 1: *"capability disclosure renders below in plain English"*). This is the M05 §8.security L1 disclosure — the same surface M07.E reused as the `import-capability-disclosure` list in `ImportPanel.tsx`. D1 **reuses it a third time** — do not rebuild plain-English disclosure:

```tsx
// src/components/builder/nodes/BuilderAgentNode.tsx — illustrative.
import { Handle, Position, type NodeProps } from '@xyflow/react';

interface BuilderAgentNodeData extends Record<string, unknown> {
  agentId: string;
  role: string;
  model: string;
  allowedTools: string[];
  allowedSkills: string[];
}

export function BuilderAgentNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderAgentNodeData;
  return (
    <div className="agent-node builder-agent-node" data-testid={`builder-agent-node-${d.agentId}`}>
      <Handle type="target" position={Position.Top} />
      <div className="agent-node__name">{d.agentId}</div>
      <div className="agent-node__id">{d.role || 'no role set'}</div>
      {/* The plain-English capability disclosure — REUSE the M05 / M07.E
          surface (the import-capability-disclosure list pattern). Derived
          live from allowedTools/allowedSkills, so an inline-config edit
          (D1.3.4) updates this immediately. D2's Agent→Agent narrowing
          later feeds the NARROWED set here (D2.3.3). */}
      <CapabilityDisclosure
        allowedTools={d.allowedTools}
        allowedSkills={d.allowedSkills}
        data-testid={`builder-node-disclosure-${d.agentId}`}
      />
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
```

`CapabilityDisclosure` is the **reused** M05 L1 surface — the `<ul className="import-capability-disclosure">` plain-English rendering M07.E built (the `<existing_pattern_audit>` pins its exact location + props). If that surface is currently coupled to `ImportPanel.tsx`'s `ImportRecord` shape, D1's reuse is the point at which it is lifted into a shared `src/components/CapabilityDisclosure.tsx` taking a plain `allowed_*` input — a behavior-preserving extraction, **not** a rebuild (and `ImportPanel.tsx` then consumes the extracted component). The disclosure is derived live from the node's `allowed_*` in `builderStore.framework`, so an inline-config edit (D1.3.4) updates the disclosure with no extra wiring. Once D2 lands Agent→Agent narrowing, the disclosure on a *child* agent reflects the **narrowed** caps the Stage B report computes (D2.3.3) — D1 ships the declared-caps disclosure; D2 swaps in narrowed-caps for child agents.

#### D1.3.6 The Builder node CSS — reuse with a thin Builder layer

The five Builder node components reuse the §3 node CSS class families (`agent-node`, `tool-node`, `skill-node`, `hitl-node`, `hook-node` — already in `styles.css` against theme variables). D1 adds a thin `builder-*-node` layer **only** for the interactive affordances the read-only nodes lack (a selected-state outline, the inline-disclosure spacing, drop-cursor feedback):

```css
/* src/styles.css (additions) — illustrative. Theme variables only. */
.builder-canvas {
  width: 100%;
  height: 100%;
  background: var(--node-bg);
}
.builder-agent-node {
  /* inherits .agent-node; adds the interactive affordance only */
  cursor: grab;
}
.builder-agent-node--selected {
  outline: 2px solid var(--node-active);
}
.builder-node-config {
  background: var(--node-bg-alt);
  color: var(--node-fg);
  border: 1px solid var(--node-base-border);
}
```

Per gotcha #67 (a component rendered in the DOM ≠ its CSS exists), every new className gets a corresponding rule + a static `every_class_has_a_corresponding_CSS_rule` test. All colors come from theme variables (`--node-bg` / `--node-fg` / `--node-active` / `--node-base-border`) — no literals (M07-IRL #3 was a literal-color contrast bug).

### D1.4 Tests

#### D1.4.1 Playwright behavior — `tests/e2e/builder_canvas_nodes.spec.ts`

Vite dev server, `@tauri-apps/api` module-mocked (gotcha #23); drag-drop via Playwright's drag API against the dev server. This spec demonstrates **MVP §M8 criterion 1 end-to-end**:

- `dragging_an_agent_palette_item_onto_the_canvas_instantiates_an_agent_node` — switch to Builder → drag the Agent Palette item onto the empty canvas → a `builder-agent-node` appears.
- `selecting_a_node_opens_the_inline_config_panel` — click the node → `builder-node-config` renders.
- `setting_role_and_model_updates_the_node` — type a role, pick a model → the node reflects the new role.
- `the_capability_disclosure_renders_plain_english_below_the_node` — the node shows the plain-English disclosure derived from `allowed_*`.
- `dropping_a_tool_palette_item_adds_a_tool_node` — a Tool drop instantiates a Tool node (the non-Agent path).
- `re_dropping_the_same_palette_item_does_not_duplicate_the_node` — idempotency of `addNode`.

#### D1.4.2 `builderStore` unit tests — `tests/unit/lib/builderStore.test.ts` (extended)

- `addNode_with_an_agent_appends_an_agents_entry_to_framework`
- `addNode_records_the_drop_position_in_nodePositions`
- `addNode_is_idempotent_on_re_drop_of_the_same_item`
- `addNode_with_a_tool_does_not_mutate_the_agents_array`
- `updateNode_patches_the_selected_agents_role_and_model`
- `updateNode_patches_allowed_tools_and_allowed_skills`
- `canvasNodes_derives_a_react_flow_node_per_framework_agent` (the projection)
- `canvasNodes_carries_the_user_placed_position_from_nodePositions`

#### D1.4.3 `BuilderCanvas` unit tests — `tests/unit/components/builder/BuilderCanvas.test.tsx`

- `renders_a_node_per_canvasNodes_projection_entry`
- `onDrop_parses_the_palette_payload_and_calls_addNode`
- `onDrop_ignores_a_drag_without_the_builder_node_mime_type`
- `clicking_a_node_calls_selectNode`
- `clicking_the_pane_clears_the_selection`

#### D1.4.4 `BuilderAgentNode` + `NodeConfigPanel` unit tests

`tests/unit/components/builder/nodes/BuilderAgentNode.test.tsx`:
- `renders_the_agent_id_and_role`
- `renders_the_plain_english_capability_disclosure_from_allowed_star`
- `renders_a_placeholder_when_no_role_is_set`

`tests/unit/components/builder/NodeConfigPanel.test.tsx`:
- `renders_nothing_when_no_node_is_selected`
- `renders_role_model_and_allowed_lists_for_a_selected_agent`
- `editing_the_role_field_calls_updateNode`
- `selecting_a_model_calls_updateNode_with_the_model_patch`

#### D1.4.5 Acceptance criteria

- [ ] `npm run test` — all vitest tests pass; renderer coverage ≥80% on `src/components/builder/BuilderCanvas.tsx` + `src/components/builder/nodes/*` + `src/components/builder/NodeConfigPanel.tsx` + the `builderStore` additions
- [ ] `npx tsc --noEmit` — clean
- [ ] `npx eslint .` + `npx prettier --check '**/*.{ts,tsx,js,jsx,json}'` — clean
- [ ] `npm run test:e2e -- builder_canvas_nodes.spec.ts` — passes; **MVP §M8 criterion 1 demonstrated end-to-end** (drag Agent → set properties → disclosure renders)
- [ ] Every new CSS class has a corresponding rule in `src/styles.css` (gotcha #67); Builder node styles use theme variables (no literals — M07-IRL #3)
- [ ] `GraphCanvas.tsx` and `src/components/nodes/*.tsx` are **untouched** — the Builder Canvas is a new interactive component, the read-only live-graph nodes are not edited
- [ ] The M05 / M07.E plain-English capability-disclosure surface is **reused** (not rebuilt) — confirmed via the `<existing_pattern_audit>`
- [ ] `builderNodeTypes` is module-level (the `GraphCanvas` `nodeTypes` trap)
- [ ] CI-parity per G6; strict two-commit (v1.7 renderer default) — `git diff <red>..<impl> -- '**/tests/**'` EMPTY

### D1.5 CLI Prompt

```xml
<work_stage_prompt id="M08.D1">
  <context>
    M08 Stage D1 — the Builder Canvas node editor: a NEW interactive
    React-Flow component (src/components/builder/BuilderCanvas.tsx —
    distinct from the read-only GraphCanvas.tsx, which is untouched),
    drag-drop instantiation (a Palette item dropped on the canvas →
    builderStore.addNode → a framework.agents[] entry → the canvas
    projection re-derives), inline node configuration (NodeConfigPanel —
    role / model / allowed_* on right-click / select), and per-node
    plain-English capability disclosure that REUSES the M05 §8.security
    L1 disclosure surface (the same surface M07.E reused as the
    import-capability-disclosure list — its third reuse; do not rebuild
    plain-English disclosure). D1 implements the C-stubbed builderStore
    addNode / updateNode actions + the framework→canvas projection
    selectors. Pure renderer over builderStore (ADR-0020) — NO new
    backend. D1 is the NODE half of the Canvas; edges + capability
    narrowing + continuous validation + red badges are D2 — D1's
    BuilderCanvas ships onConnect UNSET. MVP §M8 criterion 1 (drag an
    Agent → set role/model → disclosure renders) is demonstrated
    end-to-end in Playwright. Renderer ≥80 (vitest).
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Pre-existing legacy file inventory, Stage D1 D1.1–D1.4)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543 — drag palette items / configure inline / capability disclosure below each node) + §3 (the graph node visual conventions) + §0b (Agent) + §8.security L1 (the plain-English capability disclosure)</file>
    <file>docs/MVP-v0.1.md §M8 (criterion 1 — the headline path D1's Playwright spec demonstrates)</file>
    <file>docs/build-prompts/retrospectives/M08.C-retrospective.md (the [END] Decisions — apply them; the shipped builderStore shape + the Palette dragStart payload contract)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/gotchas.md (#23 Playwright cannot drive the Tauri window; #27 vitest re-query after await; #67 component+CSS contract; #75 useShallow for derived selectors)</file>
    <file>docs/M07-irl-findings.md (#3 — the contrast bug Stage A fixed; the Builder node styles must use theme variables, not literals)</file>
  </read_first>

  <read_reference>
    <file purpose="The read-only live-graph React-Flow component — PIN the module-level nodeTypes convention (the OUTSIDE-the-component comment), the NodeProps prop shape, and the dagre/MiniMap setup the Builder Canvas does NOT reuse (Builder nodes keep user-placed manual positions). The Builder Canvas is a NEW component — do NOT edit this file.">src/components/GraphCanvas.tsx</file>
    <file purpose="The 11 read-only node components + CapabilityBadge — PIN the §3 CSS class families (agent-node / agent-node__name / tool-node / skill-node / hitl-node / hook-node) the Builder nodes reuse; the Builder nodes are NEW interactive components, the build pins reuse-vs-new per node">src/components/nodes</file>
    <file purpose="The M07.E plain-English capability-disclosure surface (the import-capability-disclosure list + describeProvenance) — D1 REUSES this; if it is coupled to ImportRecord, D1 lifts it into a shared CapabilityDisclosure.tsx as a behavior-preserving extraction, NOT a rebuild">src/components/ImportPanel.tsx</file>
    <file purpose="The Builder store Stage C shipped — D1 implements the C-stubbed addNode / updateNode actions + adds the canvasNodes / canvasEdges projection selectors + the nodePositions slot. builderStore is the source of truth (ADR-0020).">src/lib/builderStore.ts</file>
    <file purpose="The Stage C Builder shell — D1 mounts BuilderCanvas into C's empty builder-canvas-region placeholder">src/components/builder/BuilderShell.tsx</file>
    <file purpose="The Stage C Palette — read the application/x-builder-node dragStart payload shape D1's onDrop handler consumes (the C↔D1 contract)">src/components/builder/Palette.tsx</file>
    <file purpose="dagre layout (layoutGraph, pure) — D1 does NOT use it (Builder nodes keep manual positions); referenced only if an optional auto-layout affordance is added — out of D1 scope">src/lib/layout.ts</file>
    <file purpose="The theme CSS variables (--node-bg / --node-fg / --node-active / --node-base-border) the Builder node + canvas + config-panel styles MUST use — no literal colors (M07-IRL #3)">src/styles.css</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M08.B" retro="docs/build-prompts/retrospectives/M08.B-retrospective.md"/>
    <stage id="M08.C" retro="docs/build-prompts/retrospectives/M08.C-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M08-workbench.md" section="D1.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the D1.4 buckets: Playwright
      (builder_canvas_nodes.spec.ts — drag an Agent onto the canvas → a
      node appears; select → the config panel opens; set role/model → the
      node + the disclosure update; the disclosure renders plain English)
      + vitest (builderStore addNode/updateNode + the framework→projection
      derivation; BuilderCanvas drop handler + node rendering;
      BuilderAgentNode + its capability disclosure; NodeConfigPanel
      fields → updateNode). Stub the production surfaces just enough that
      the test files compile (empty components are fine — the goal is
      link-time discovery, not behavior). Confirm right-reason red per
      CLAUDE.md §5 — cannot-find-module / unresolved-import / assertion
      failures, NOT test-file compile errors and NOT tautological passes.
      Commit as a STANDALONE `test(M08.D1): failing tests for the Builder
      Canvas node editor` commit on the M08 branch BEFORE the green-phase
      impl; the body pastes the first ~40 lines of the vitest/Playwright
      output proving the expected-failure class. Surface the red-phase
      commit; the user approves before green begins.
    </red_phase>
    <green_phase>
      Implement the BuilderCanvas / the five Builder node components / the
      NodeConfigPanel / the builderStore addNode + updateNode + the
      projection selectors until ALL failing tests pass. Do NOT modify the
      test files during implementation — if a test is wrong, fix it in a
      SEPARATE labelled follow-up commit with explanation, never silently
      in the impl commit. The impl commit body MUST state the verifiable
      invariant: `git diff &lt;red-sha&gt;..&lt;impl-sha&gt; --
      '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      prettier/eslint fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message.
    </green_phase>
  </tdd_discipline>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="D1.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>

  <gates milestone="M08"/>

  <coverage_gate>
    <gate scope="renderer" target_lines="80" ignore_filename_regex="generated"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <adr_triggers>
    <trigger>None new in D1 — ADR-0020 (the Builder canvas↔framework.json state model) was filed at Stage C; D1 implements against it (the framework→canvas projection IS the ADR-0020 model in code). D1 is pure renderer — no schema, no IPC, no new dependency. If the framework→projection derivation surfaces a genuine model-shape problem with ADR-0020, surface BEFORE red rather than diverging silently.</trigger>
  </adr_triggers>

  <existing_pattern_audit>
    <pattern grep_for="nodeTypes" rationale="pin the React-Flow module-level nodeTypes convention from GraphCanvas.tsx (the OUTSIDE-the-component comment — redefining it per render re-mounts every node) before authoring the Builder builderNodeTypes; the same trap re-applies" affected_files="src/components/builder/BuilderCanvas.tsx" remediation="define builderNodeTypes at module scope"/>
    <pattern grep_for="import-capability-disclosure" rationale="pin the EXACT M07.E plain-English capability-disclosure surface (the list rendering + describeProvenance) for reuse — D1 must not rebuild plain-English disclosure; if coupled to ImportRecord, lift into a shared CapabilityDisclosure.tsx as a behavior-preserving extraction" affected_files="src/components/ImportPanel.tsx" remediation="reuse / extract — never reimplement"/>
    <pattern grep_for="agent-node" rationale="pin the §3 node CSS class families the Builder nodes reuse — the Builder nodes add only a thin builder-*-node interactive layer, not a parallel node-CSS system" affected_files="src/styles.css" remediation="reuse the existing class families; add builder-* only for the interactive affordances"/>
  </existing_pattern_audit>

  <zustand_selector_audit>
    <selector store="builderStore" slice="canvasNodes() / canvasEdges()" requires_use_shallow="true" import_path="zustand/react/shallow" rationale="the canvas reads derived projection arrays — without useShallow the canvas re-renders on every store commit (gotcha #75; the GraphCanvas selector-discipline precedent). canvasEdges() is empty in D1 but the selector form is established here for D2."/>
    <selector store="builderStore" slice="selectedNodeId / framework / addNode / updateNode / selectNode" requires_use_shallow="false" import_path="zustand/react/shallow" rationale="primitive + whole-object + action selectors do not need useShallow"/>
  </zustand_selector_audit>

  <pre_flight_check>
    <check name="branch" gate="git rev-parse --abbrev-ref HEAD must equal the M08 parent-milestone branch; the M08.C impl commit must be present in git log --oneline main..HEAD"/>
    <check name="stage_c_surfaces" gate="grep-confirm Stage C shipped src/components/builder/BuilderShell.tsx (with the empty builder-canvas-region), src/components/builder/Palette.tsx (with the application/x-builder-node dragStart payload), and src/lib/builderStore.ts (with the typed-stub addNode/updateNode + the framework slot) before D1 builds on them"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="src/components/GraphCanvas.tsx" verified="true" note="read-only reference — D1 does NOT edit it; the Builder Canvas is a new component"/>
    <claim type="file" path="src/components/ImportPanel.tsx" verified="true" note="hosts the M07.E plain-English capability-disclosure surface D1 reuses"/>
    <claim type="file" path="src/lib/builderStore.ts" verified="true" note="Stage C shipped it; D1 implements the C-stubbed addNode/updateNode + adds the projection selectors"/>
    <claim type="file" path="src/components/builder/BuilderShell.tsx" verified="true" note="Stage C shipped it; D1 mounts BuilderCanvas in the empty Canvas region"/>
    <claim type="file" path="src/components/builder/Palette.tsx" verified="true" note="Stage C shipped it; D1's onDrop reads its dragStart payload"/>
    <claim type="file" path="src/components/builder/BuilderCanvas.tsx" verified="false" note="D1 creates"/>
    <claim type="file" path="src/components/builder/NodeConfigPanel.tsx" verified="false" note="D1 creates"/>
    <claim type="store_slot" path="src/lib/builderStore.ts" symbol="canvasNodes" shape="() => Node[] — the framework→React-Flow projection D1 implements (node side); D2 adds canvasEdges" verified="false" note="D1 adds"/>
    <claim type="store_slot" path="src/lib/builderStore.ts" symbol="nodePositions" shape="Record&lt;string, {x:number;y:number}&gt; — user-placed coordinates the projection reads" verified="false" note="D1 adds"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M08.C-retrospective.md" verified="true"/>
  </phase_doc_inventory_audit>

  <runtime_environment os="windows" note="Vitest + Playwright run against the Vite dev server; @tauri-apps/api module-mocked (gotcha #23). React-Flow drag-drop is exercised through Playwright's drag API against the dev server."/>

  <gotchas>
    <trap>#23 — Playwright drives the Vite dev server with @tauri-apps/api module-mocked; drag-drop is exercised via Playwright's drag API against the dev server (it cannot drive the Tauri window).</trap>
    <trap>React-Flow nodeTypes is module-level — redefining builderNodeTypes per render re-mounts every node (the GraphCanvas.tsx OUTSIDE-the-component comment). The same trap re-applies for the Builder nodeTypes.</trap>
    <trap>The Builder Canvas is a NEW interactive component — do NOT edit the read-only GraphCanvas.tsx; do NOT conflate editable + read-only modes in one node component. The read-only live-graph nodes have no inline config; the Builder nodes do.</trap>
    <trap>Reuse the M05 / M07.E plain-English capability-disclosure surface — do NOT rebuild plain-English disclosure (third reuse). If coupled to ImportRecord, lift into a shared component (behavior-preserving extraction).</trap>
    <trap>#75 — the canvasNodes() / canvasEdges() derived projection selectors use useShallow (zustand/react/shallow) or the canvas re-renders on every store commit.</trap>
    <trap>#67 — every new className (builder-canvas / builder-agent-node / builder-node-config / …) gets a corresponding src/styles.css rule + a static every_class_has_a_corresponding_CSS_rule test.</trap>
    <trap>screenToFlowPosition is only available inside a ReactFlowProvider — the BuilderCanvas wraps its inner component in ReactFlowProvider so the onDrop handler can convert cursor coords to canvas coords.</trap>
    <trap>addNode must be idempotent on re-drop of the same Palette item (keyed by ${kind}:${ref}) — re-dropping must not duplicate the framework.agents[] entry.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT edit GraphCanvas.tsx or src/components/nodes/*.tsx — the Builder Canvas + the Builder nodes are NEW components. The read-only live-graph view is untouched; D1 reuses the §3 CSS, not the read-only components.</warning>
    <warning>DO NOT add edges in D1 — the four edge types, onConnect, edge rendering, and the Agent→Agent narrowing are ALL D2. D1's BuilderCanvas ships onConnect UNSET. Adding any edge logic crosses the D1/D2 stage boundary.</warning>
    <warning>DO NOT add validation in D1 — continuous validation, the validateFramework call, the debounced trigger, and the per-node red badges are D2. D1's nodes carry no validation state.</warning>
    <warning>DO NOT rebuild plain-English capability disclosure — reuse the M05 / M07.E surface (its third reuse). The only acceptable code change to that surface is a behavior-preserving extraction into a shared component if it is currently coupled to ImportRecord.</warning>
    <warning>DO NOT add a backend command — D1 is pure renderer over builderStore. Stage B's validate_framework is D2's; the canvas mutates the framework document client-side (ADR-0020).</warning>
  </execution_warnings>

  <time_box hours="10-13"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>Confirm the Builder Canvas is a NEW interactive component (GraphCanvas.tsx + src/components/nodes/*.tsx untouched). Confirm the M05 / M07.E plain-English capability-disclosure surface was REUSED — and if it required a behavior-preserving extraction into a shared component, document the extraction (and that ImportPanel.tsx now consumes the shared component). State the framework→canvas projection derivation (canvasNodes / nodePositions) — this is the ADR-0020 model in code and the inverse of E's canvas→JSON binding; note any shape friction. Record the MVP §M8 criterion 1 Playwright result (drag Agent → configure → disclosure). Note whether addNode's idempotency + the projection re-derivation behaved cleanly under repeated drops.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="D1.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state — git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M08.*-retrospective.md</item>
    <item>strict-TDD proof — git diff &lt;red-sha&gt;..&lt;impl-sha&gt; -- '**/tests/**' is EMPTY</item>
    <item>diff stat</item>
    <item>existing_pattern_audit reconciliation — the module-level builderNodeTypes convention + the M05/M07.E disclosure reuse (or the behavior-preserving extraction)</item>
    <item>confirmation GraphCanvas.tsx + src/components/nodes/*.tsx are untouched</item>
    <item>MVP §M8 criterion 1 Playwright demonstration (drag Agent → configure → disclosure renders)</item>
    <item>gate results (v1.6 canonical order; renderer vitest ≥80%; Playwright green; every-class-has-CSS-rule confirmation; CI-parity per G6)</item>
    <item>M08.D1 retrospective [END] section</item>
    <item>draft commit message from D1.6</item>
    <item>explicit statement: "Stage M08.D1 is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### D1.6 Commit Message

```
feat(renderer): M08 Stage D1 — Builder Canvas node editor (drag-drop + inline config + capability disclosure)

The Builder Canvas as an interactive React-Flow node editor — a NEW
component distinct from the read-only GraphCanvas (which is untouched).
D1 is the node half of the Canvas; edges + narrowing + validation are D2.

Components:
- src/components/builder/BuilderCanvas.tsx (new): the interactive
  React-Flow editor — a drop target, nodes rendered from the
  builderStore.framework projection, node selection. Wrapped in
  ReactFlowProvider so the onDrop handler can screenToFlowPosition.
  Module-level builderNodeTypes (the GraphCanvas re-mount trap).
  onConnect is left unset — edges are D2.
- src/components/builder/nodes/Builder{Agent,Tool,Skill,Hitl,Hook}Node.tsx
  (new): interactive Builder node components — reuse the §3 node CSS
  class families, add a thin builder-*-node interactive layer.
- src/components/builder/NodeConfigPanel.tsx (new): inline node
  configuration — role / model (Anthropic model dropdown) /
  allowed_tools / allowed_skills; every edit calls updateNode.

Store:
- src/lib/builderStore.ts: addNode + updateNode implemented (Stage C
  shipped them as typed stubs); the canvasNodes / canvasEdges
  framework→React-Flow projection selectors (the ADR-0020 model in
  code); the nodePositions slot so dropped nodes keep user-placed
  coordinates. addNode is idempotent on re-drop.

Drag-drop:
- A Palette item dropped on the canvas (the application/x-builder-node
  payload from Stage C) → addNode → a framework.agents[] entry (for an
  Agent) → the canvas projection re-derives.

Capability disclosure:
- Each node renders the plain-English declared-allowed_* disclosure by
  REUSING the M05 §8.security L1 surface (the same surface M07.E reused
  as the import-capability-disclosure list — its third reuse). Derived
  live from framework, so an inline-config edit updates it immediately.

MVP §M8 criterion 1 demonstrated end-to-end in Playwright: drag an
Agent onto the empty canvas → set role/model → the plain-English
capability disclosure renders below.

Pure renderer over builderStore (ADR-0020) — no new backend, no schema,
no new dependency. Playwright + vitest; renderer ≥80. Strict two-commit
TDD: '**/tests/**' diff EMPTY.

Not in this stage: edges + the four edge types + Agent→Agent capability
narrowing + continuous validation + red badges (D2); the Inspector (E).

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---
## Stage D2 — Builder Canvas: edges + capability narrowing + continuous validation

### D2.1 Problem Statement

Spec Phase 9: *"Connect nodes with edges — Agent→Skill = `allowed_skills` entry; Agent→Tool = `allowed_tools` entry; Agent→Agent = `spawns` entry (capability narrowing rule applied automatically: child's `allowed_*` is intersected with parent's); Hook→Task = `post_hooks` entry on `task_defaults`. Validation: `framework.v1.json` schema runs continuously; errors surfaced as red badges."* MVP §M8 criteria 2/3/4: connect Agent→Skill / Agent→Agent (with the narrowing surfaced) / click Validate. Stage D1 shipped the node half — drag a Palette item onto the canvas, instantiate a node into `builderStore.framework`, configure it inline. D2 is the **edge half**: it makes the canvas nodes *connectable*, turns each connection into the correct `framework.json` mutation, and closes the validation loop so a half-built framework tells the user — live — what is wrong.

Three things land here, each a coherent slice of the spec quote. (1) **Edge creation.** React Flow v12's `onConnect` fires when the user drags a wire between two node handles; D2 wires it to a `builderStore.connectEdge` action that maps the `(source kind, target kind)` pair to one of the four spec edge types and **rejects** any other pair. (2) **Automatic Agent→Agent capability narrowing.** Drawing an Agent→Agent (`spawns`) edge means the child agent's `allowed_*` must be intersected with the parent's — spec §8.security L2a. The intersection is computed by `capability/narrowing.rs::narrow()` in the Rust main process (M05.B); D2 **surfaces** the result Stage B's `validate_framework` report already carries, per spawn edge, as the `{parent_caps, child_declared_caps, narrowed_caps}` triple. Spec §9 forbids a second copy of that logic in TS — the renderer renders the triple, it never computes an intersection. (3) **Continuous validation with red badges.** Every canvas edit — D1's add/config and D2's connect — triggers a *debounced* `validate_framework` call; the `FrameworkValidationReport` lands in `builderStore.validation`; each node component renders a **red badge** when the report holds a `schema_errors` / `capability_errors` entry keyed to that node's path.

D2 is pure renderer. It adds **no schema, no backend, no new Tauri command** — it consumes the `validate_framework` command Stage B shipped. The narrowing rule and the schema validator both live in Rust precisely so the Builder cannot drift from the runtime; D2 is the consumer that proves that wire works end-to-end.

Concrete deliverables:

1. **`builderStore.connectEdge`** — the action React Flow's `onConnect` calls. Maps `(sourceKind, targetKind)` → the right `framework` mutation for each of the four spec edge types (Agent→Skill = `allowed_skills`; Agent→Tool = `allowed_tools`; Agent→Agent = `spawns`; Hook→Task = `task_defaults.post_hooks`); **rejects** every other pair (returns without mutating `framework`, no edge created). Idempotent — connecting an already-present edge does not double the `framework` array entry.
2. **`BuilderCanvas.tsx` `onConnect` wiring + edge projection** — pass `onConnect` to `<ReactFlow>` (the D1 skeleton left the `onConnect` slot commented `// onConnect is D2`); render `canvasEdges` derived from the `framework` projection so a successful `connectEdge` paints a wire.
3. **Continuous debounced validation** — a `builderStore`-level debounced trigger: any `framework` mutation (D1 `addNode`/`updateNode`, D2 `connectEdge`) schedules a single `validateFramework` call after a quiet interval; the resulting `FrameworkValidationReport` is stored in `builderStore.validation`. A burst of keystrokes fires the command once, not once per keystroke.
4. **`validateFramework` IPC wrapper** — `src/lib/ipc.ts`; params PINNED to the shipped Stage B `validate_framework` command via the `<wire_signature_audit>`.
5. **Red validation badge on builder node components** — `src/components/builder/nodes/*.tsx` render a red badge when `builderStore.validation` carries a `schema_errors` / `capability_errors` entry whose `node_path` keys that node.
6. **`NarrowingNotice.tsx`** — a new component surfacing the Agent→Agent narrowing decision (what the child declared, and either the surviving set or — when the child declares a capability the parent does not hold — the rejection) read from the `validate_framework` report's `capability_summary.spawn_edges[]`. The child node's capability disclosure (D1.3.5) reflects the narrowing result.
7. **Playwright + vitest behavior coverage** — renderer ≥80.

Not in this stage:

- The Inspector panel and its explicit **Validate** button (Stage E — D2 ships the *continuous* trigger; E adds the *explicit* trigger of the **same** validator).
- The Canvas | JSON two-way binding (Stage E).
- Save / load to disk (Stage E).
- The Tester (Stages F1/F2).
- Any new schema, backend, or Tauri command — D2 consumes Stage B's `validate_framework`.

### D2.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/components/builder/BuilderCanvas.tsx` | exists (D1) | Pass `onConnect={onConnect}` to `<ReactFlow>` (the D1 `// onConnect is D2` slot); render `canvasEdges` from the `framework` projection. |
| `src/components/builder/nodes/*.tsx` | exists (D1) | Render a red validation badge on a node whose path keys a `schema_errors` / `capability_errors` entry in `builderStore.validation`. |
| `src/lib/builderStore.ts` | exists (C/D1) | `connectEdge` action (the four edge types + the reject path); the `validation` slot; the debounced continuous-validation trigger fired by every `framework` mutation. |
| `src/lib/ipc.ts` | exists | `validateFramework` wrapper — params PINNED to the Stage B command via `<wire_signature_audit>`. |
| `src/components/builder/NarrowingNotice.tsx` | **new** | Surfaces the Agent→Agent narrowing decisions (MVP §M8 criterion 3) from the `validate_framework` report's per-spawn-edge triple. |
| `src/styles.css` | exists | `.builder-node--invalid` (red badge), `.builder-edge`, `.narrowing-notice` + `.narrowing-notice__rejected` classes — theme variables, no contrast regression. |
| `tests/unit/lib/builderStore.connectEdge.test.ts` | **new** | `connectEdge` per edge type + the reject path + idempotency + the debounced trigger. |
| `tests/unit/components/builder/NarrowingNotice.test.tsx` | **new** | The narrowing notice renders the triple. |
| `tests/unit/components/builder/BuilderNodeBadge.test.tsx` | **new** | The red-badge logic on the builder node components. |
| `tests/e2e/builder_edges.spec.ts` | **new** | Playwright — Agent→Skill edge; Agent→Agent narrowing surfaced; red badges; invalid-pair reject. |
| `CHANGELOG.md` / retro | exist/new | Stage D2 entries. |

Effort budget: ~10–13 hours. The largest piece is `connectEdge`'s four-type mapping + reject path and getting the debounced validation cycle right; the narrowing notice and badge rendering are thin presentational surfaces over Stage B's report — no logic, only display.

### D2.3 Detailed Changes

#### D2.3.1 Wire pinning (v1.8 discipline — BEFORE pseudocode)

Pin to the **shipped** Stage B wire, not assumptions — the M06.E / M07.E drift lesson (five `wire_signature_audit` drifts caught at M06.E alone; `ipc.ts:194` documents the `mcp_test_connection {config}`-vs-`{name}` reconciliation). Before authoring any of the component or store pseudocode below:

- `<wire_signature_audit>` (D2.5): read `src-tauri/src/commands.rs` for the **committed** Stage B `validate_framework` signature; record `actual_params` verbatim. Stage B's B.3.1 ships `validate_framework` as a Tauri command returning `FrameworkValidationReport` synchronously (B's §12-owned command-return decision); the wrapper is a thin `invoke` mirroring the M07 `importArtifact` pattern.
- `<phase_doc_inventory_audit shape=…>` (D2.5): pin the `FrameworkValidationReport` / `NodeError` / `FrameworkCapabilitySummary` / `SpawnEdgeNarrowing` TS types Stage B's report carries — specifically that `capability_summary` (B.3.4) rides on the report and the per-spawn-edge narrowing triple `{parent_caps, child_declared_caps, narrowed_caps}` lives at `capability_summary.spawn_edges[]`, with `narrowed_caps` a serde-tagged `Result` (`{Ok}` | `{Err}`). Whether the report ships as a schema-generated type or a hand-mirrored interface (the `McpTool` / `ImportOutcome` precedent in `ipc.ts`) is decided by Stage B's actual output — pin the real shape, do not assume.

The `validateFramework` wrapper, illustrative — final param shape comes from the pinned signature:

```ts
// src/lib/ipc.ts (additions) — params PINNED to the Stage B-shipped
// `validate_framework` command via <wire_signature_audit>.

import type { CapabilityDeclaration } from '../types/capability';

/**
 * Validate an in-progress framework document against the schemas +
 * the capability primitive (M08 Stage B `validate_framework`). The
 * report is keyed to offending node paths; D2's continuous pass and
 * Stage E's explicit Validate button both call this — one Rust
 * validator, two triggers (spec §9, no TS duplication). `doc` is the
 * canvas's serialized framework.json candidate; it MAY be invalid —
 * that is the point. Returns synchronously (Stage B B.3.1's §12 call).
 */
export async function validateFramework(
  doc: unknown,
): Promise<FrameworkValidationReport> {
  return await invoke<FrameworkValidationReport>('validate_framework', { doc });
}

/** One offending node — mirrors `runtime_main::builder::NodeError`. */
export interface NodeError {
  node_path: string;
  message: string;
}

/**
 * One Agent→Agent spawn edge's L2a narrowing decision — mirrors
 * `runtime_main::builder::SpawnEdgeNarrowing` (B.3.4). The decision is
 * computed by `capability/narrowing.rs::narrow()` in Rust (spec §9 —
 * NOT re-implemented in TS). `narrowed_caps` is a serde-serialized
 * `Result`, externally tagged: `{ Ok: [...] }` carries the surviving
 * set (all-or-nothing — `Ok` is the child's declared set verbatim),
 * `{ Err: "..." }` names the capability the parent does not hold
 * (Stage B folds that into `capability_errors` — B.3.2 step 4).
 */
export interface SpawnEdgeNarrowing {
  parent_id: string;
  child_id: string;
  parent_caps: CapabilityDeclaration[];
  child_declared_caps: CapabilityDeclaration[];
  narrowed_caps: { Ok: CapabilityDeclaration[] } | { Err: string };
}

/**
 * Whole-framework capability picture — mirrors
 * `runtime_main::builder::FrameworkCapabilitySummary` (B.3.4). Rides on
 * `FrameworkValidationReport.capability_summary`; the Inspector (E) and
 * the canvas narrowing notice (D2) both read it off the one report.
 */
export interface FrameworkCapabilitySummary {
  files_read: string[];
  files_written: string[];
  network_hosts: string[];
  any_shell: boolean;
  spawn_edges: SpawnEdgeNarrowing[];
}

/**
 * Mirrors `runtime_main::builder::FrameworkValidationReport` (M08
 * Stage B). PIN the exact shape via <phase_doc_inventory_audit
 * shape=...> against the Stage B-shipped commands.rs — whether it is
 * schema-generated or hand-mirrored (the McpTool precedent) is Stage
 * B's call. `capability_summary` rides on this report (B.3.4) and is
 * `null` when schema validation fails (no parsed framework); Stage E
 * reads it and D2 reads its `spawn_edges` for the narrowing notice —
 * neither calls a separate command.
 */
export interface FrameworkValidationReport {
  schema_errors: NodeError[];
  capability_errors: NodeError[];
  ok: boolean;
  capability_summary: FrameworkCapabilitySummary | null;
}
```

Per gotcha #30, an error thrown across the bridge is unwrapped via the existing `unwrapCmdError` helper.

#### D2.3.2 Edge creation + the four edge types — `builderStore.connectEdge`

React Flow v12's `onConnect` fires with a `Connection` (`{ source, target, sourceHandle, targetHandle }`) when the user drags a wire between two node handles. `BuilderCanvas` translates that into a `builderStore.connectEdge` call; `connectEdge` looks up each endpoint's kind in `framework`, maps the `(sourceKind, targetKind)` pair to the correct `framework` mutation, and **rejects** any pair that is not one of the four spec edge types — a rejected pair mutates nothing and paints no edge:

```ts
// src/lib/builderStore.ts — connectEdge action, illustrative.
// builderStore.framework is the single source of truth (ADR-0020);
// connectEdge mutates `framework`, the canvas edge projection re-derives.

connectEdge: (source: string, target: string): void =>
  set((state) => {
    const srcKind = nodeKind(state.framework, source); // 'agent'|'tool'|'skill'|'hook'|...
    const tgtKind = nodeKind(state.framework, target);

    // The four spec Phase 9 edge types — every other (src,tgt) pair is
    // rejected (return state unchanged: no framework mutation, no edge).
    switch (`${srcKind}->${tgtKind}`) {
      case 'agent->skill':
        // Agent→Skill : push the skill name to agents[source].allowed_skills
        return pushUnique(state, source, 'allowed_skills', skillName(state.framework, target));
      case 'agent->tool':
        // Agent→Tool  : push the tool name to agents[source].allowed_tools
        return pushUnique(state, source, 'allowed_tools', toolName(state.framework, target));
      case 'agent->agent':
        // Agent→Agent : push the child id to agents[source].spawns.
        // The narrowing (D2.3.3) is NOT computed here — drawing the
        // edge re-runs validate_framework (D2.3.4), whose Rust report
        // carries the intersection. connectEdge only records the spawn.
        return pushUnique(state, source, 'spawns', agentId(state.framework, target));
      case 'hook->task':
        // Hook→Task   : push the hook id to task_defaults.post_hooks
        return pushHook(state, target, hookId(state.framework, source));
      default:
        // Reject — not one of the four spec edge types. No edge created.
        return state;
    }
  }),
```

`pushUnique` makes `connectEdge` **idempotent** — re-connecting an already-recorded edge does not append a duplicate `allowed_skills` / `spawns` entry. Each successful branch mutates `builderStore.framework`; the canvas edge projection (`canvasEdges`, a pure function of `framework` — ADR-0020) re-derives and React Flow paints the wire. MVP §M8 criterion 2 ("Connect Agent→Skill → the skill name is added to `allowed_skills`") is the headline path; the reject branch is what keeps a user from drawing, say, a Tool→Tool wire that has no `framework.json` meaning.

#### D2.3.3 `onConnect` wiring + the edge projection — `BuilderCanvas.tsx`

Stage D1 shipped `BuilderCanvas` with the `onConnect` slot left explicitly for D2 (the D1 skeleton's `/* onConnect is D2 */` comment). D2 fills it and renders edges from the projection:

```tsx
// src/components/builder/BuilderCanvas.tsx — D2 additions.
// nodeTypes stays MODULE-LEVEL (the GraphCanvas.tsx trap — redefining
// it per render re-mounts every node); D1 already pinned this.

export function BuilderCanvas(): JSX.Element {
  const nodes = useBuilderStore((s) => s.canvasNodes);   // D1 — projection of s.framework
  const edges = useBuilderStore((s) => s.canvasEdges);   // D2 — projection of s.framework
  const connectEdge = useBuilderStore((s) => s.connectEdge);
  const addNode = useBuilderStore((s) => s.addNode);     // D1

  // React Flow v12 onConnect — fires on a handle-to-handle drag.
  const onConnect = useCallback(
    (c: Connection) => {
      if (c.source && c.target) connectEdge(c.source, c.target);
      // connectEdge rejects invalid pairs internally — no edge appears
      // because no framework mutation happened (the projection is pure).
    },
    [connectEdge],
  );

  return (
    <ReactFlow
      nodes={nodes}
      edges={edges}
      nodeTypes={builderNodeTypes}   // module-level — D1
      onConnect={onConnect}          // D2 — was the /* onConnect is D2 */ slot
      onDrop={/* D1 */ undefined}
      onDragOver={/* D1 */ undefined}
    >
      <Background />
      <Controls />
    </ReactFlow>
  );
}
```

The edge projection mirrors D1's node projection: `canvasEdges` is derived from `framework` (the `spawns` / `allowed_*` / `post_hooks` arrays become React Flow `Edge[]`), so a `connectEdge` mutation is the *only* way an edge appears — there is no separate edge state to keep in sync. This is the ADR-0020 source-of-truth model holding for edges exactly as D1 held it for nodes. A connection React Flow itself would draw on a rejected pair never persists, because the projection re-derives from `framework` and `framework` did not change.

#### D2.3.4 Continuous debounced validation + the `validation` slot

Spec Phase 9: *"`framework.v1.json` schema runs continuously."* "Continuously" cannot mean "on every keystroke" — that would fire a Tauri command per character typed into an inline-config field. D2 ships a **debounced** trigger: every `framework` mutation (D1's `addNode`/`updateNode`, D2's `connectEdge`) schedules a single `validateFramework` call after a short quiet interval; a burst of edits collapses to one command:

```ts
// src/lib/builderStore.ts — the debounced continuous-validation trigger.

// Module-scoped debounce handle — one in-flight scheduled validation
// per store, coalescing a burst of framework mutations into one call.
let validateTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleValidation(get: () => BuilderState, set: BuilderSet): void {
  if (validateTimer) clearTimeout(validateTimer);
  validateTimer = setTimeout(() => {
    void validateFramework(get().framework).then((report) => {
      set({ validation: report });   // → red badges (D2.3.5) re-derive
    });
    // a failed validate surfaces via unwrapCmdError; the prior report
    // is left in place rather than cleared (no flicker to "no errors").
  }, VALIDATION_DEBOUNCE_MS);
}

// Every framework-mutating action — addNode / updateNode (D1),
// connectEdge / removeNode (D2) — calls scheduleValidation(get, set)
// as its last step. One debounce; one validator; spec §9 "no
// duplication" — the Validate button (Stage E) calls the SAME
// validateFramework with no debounce (the explicit trigger).
```

The `FrameworkValidationReport` lands in `builderStore.validation` (the C-declared slot, typed by the D2.3.1-pinned shape). Two triggers, one validator: D2's continuous debounced pass and Stage E's explicit **Validate** button both call `validateFramework` — there is exactly one validator and it lives in Rust (Stage B). The renderer never reimplements schema or capability checks (spec §9). A `null` `validation` slot means "not yet validated"; an empty-`schema_errors`/`capability_errors` report means "validated, clean".

#### D2.3.5 Red validation badges on the builder node components

Spec Phase 9: *"errors surfaced as red badges."* Each builder node component (D1's `src/components/builder/nodes/*.tsx`) reads `builderStore.validation` and renders a **red badge** when the report holds a `schema_errors` or `capability_errors` entry whose `node_path` keys that node:

```tsx
// src/components/builder/nodes/BuilderAgentNode.tsx — the badge addition
// (every builder node component gets the same treatment).

export function BuilderAgentNode({ id, data }: NodeProps<BuilderAgentNodeData>): JSX.Element {
  // Selector form so a node re-renders only when ITS validation state
  // changes — derive the per-node error list, not the whole report.
  const errors = useBuilderStore(
    useShallow((s) => nodeErrorsFor(s.validation, data.nodePath)),
  );
  return (
    <div className={`builder-node builder-node--agent${errors.length ? ' builder-node--invalid' : ''}`}>
      <header>{data.role}</header>
      {/* D1: inline config trigger + the M05 §8.security L1 capability
          disclosure — reflecting the narrowing result for a spawned
          child (D2.3.6). */}
      {errors.length > 0 && (
        <span className="builder-node__badge" role="alert" title={errors.map((e) => e.message).join('\n')}>
          {errors.length}
        </span>
      )}
    </div>
  );
}

// nodeErrorsFor: pure — filters the report's schema_errors +
// capability_errors to the entries whose node_path keys this node.
function nodeErrorsFor(
  report: FrameworkValidationReport | null,
  nodePath: string,
): NodeError[] {
  if (!report) return [];
  return [...report.schema_errors, ...report.capability_errors].filter(
    (e) => e.node_path === nodePath,
  );
}
```

Per gotcha #75 + the v1.6 `<zustand_selector_audit>`: the derived per-node error list is wrapped in `useShallow` so a node does not re-render on every unrelated `validation` change. The badge is a *consequence* of the continuous pass (D2.3.4) — drawing an invalid edge or mis-configuring a node re-runs `validate_framework`, the new report lands in `validation`, and the offending node's badge appears within one debounce interval. The full per-error messages surface in Stage E's Inspector; the node badge is the at-a-glance count.

#### D2.3.6 Agent→Agent capability narrowing — surface Stage B's report (`NarrowingNotice.tsx`)

Spec Phase 9 + MVP §M8 criterion 3: an Agent→Agent (`spawns`) edge intersects the child's `allowed_*` with the parent's; the UI surfaces the narrowing decision. **The decision is Stage B's `validate_framework` report** — `capability/narrowing.rs::narrow()`, Rust, M05.B L2a. Spec §9 forbids a TS re-implementation of it, and D2 does not write one. When an Agent→Agent edge is drawn, the continuous pass (D2.3.4) re-runs `validate_framework`; the report's `capability_summary.spawn_edges[]` (D2.3.1's `SpawnEdgeNarrowing`) carries, per spawn edge, the `{parent_caps, child_declared_caps, narrowed_caps}` triple. `narrowed_caps` is a serde-tagged `Result`: `Ok` is the surviving set (all-or-nothing — the child's declared set verbatim), `Err` names the capability the parent does not hold. `NarrowingNotice` finds its edge in `spawn_edges` and renders whichever arm the backend returned — it computes nothing:

```tsx
// src/components/builder/NarrowingNotice.tsx — new component.
// SURFACES Stage B's Rust narrowing decision; computes NOTHING.

interface NarrowingNoticeProps {
  /** Spawn-edge id `agent:<parent>-><child>` — D2.3.3's edge id scheme. */
  spawnEdgeId: string;
}

/** `kind:resource` — a stable display label for one declaration. */
function capLabel(c: CapabilityDeclaration): string {
  return `${c.kind}:${c.resource}`;
}

export function NarrowingNotice({ spawnEdgeId }: NarrowingNoticeProps): JSX.Element | null {
  // The narrowing decisions ride on the validate_framework report's
  // capability_summary.spawn_edges[] (Stage B B.3.4). Find this edge by
  // its `agent:<parent>-><child>` id; useShallow so the component does
  // not re-render on unrelated `validation` changes (gotcha #75).
  const edge = useBuilderStore(
    useShallow(
      (s) =>
        s.validation?.capability_summary?.spawn_edges.find(
          (e) => `agent:${e.parent_id}->${e.child_id}` === spawnEdgeId,
        ) ?? null,
    ),
  );
  if (!edge) return null;

  const declared = edge.child_declared_caps.map(capLabel).join(', ') || '(none)';

  // narrowed_caps is a serde-tagged Result. `narrow` is all-or-nothing
  // (capability/narrowing.rs — M05.B L2a): `Err` means the child
  // declared a capability the parent does not hold; the edge is
  // rejected (Stage B folds it into capability_errors → a red badge).
  // The renderer DISPLAYS the Rust decision — it never recomputes it.
  if ('Err' in edge.narrowed_caps) {
    return (
      <aside className="narrowing-notice" role="note">
        <h4>Capability narrowing rejected</h4>
        <p>Child declared: {declared}</p>
        <p className="narrowing-notice__rejected">{edge.narrowed_caps.Err}</p>
      </aside>
    );
  }

  // `Ok` — every child capability is subsumed by the parent. All-or-
  // nothing narrowing carries `proposed` verbatim, so the surviving set
  // is rendered straight from the backend value (the no-TS-intersection
  // contract test, D2.4.3, asserts exactly this).
  const survives = edge.narrowed_caps.Ok.map(capLabel).join(', ') || '(none)';
  return (
    <aside className="narrowing-notice" role="note">
      <h4>Capability narrowing applied</h4>
      <p>Child declared: {declared}</p>
      <p>Survives intersection with the parent: {survives}</p>
    </aside>
  );
}
```

The child agent node's capability disclosure (D1.3.5 — the reused M05 §8.security L1 plain-English component) reflects the narrowing result: on a valid (`Ok`) spawn edge the child runs with its declared set (all-or-nothing narrowing carries `proposed` verbatim — M05.B `narrow`); on an invalid (`Err`) edge the node also carries the red badge Stage B's folded `capability_error` drives (B.3.2 step 4). The renderer **never computes an intersection** — every capability list `NarrowingNotice` renders comes straight from the backend `validate_framework` report. If a future need arises to show *why* an edge was rejected beyond the `NarrowingError` message, that is an enrichment of Stage B's report — a Rust change — not a TS computation added here.

#### D2.3.7 Styles

`.builder-node--invalid` (red border + the `__badge` red dot — the same red token as the live-graph `error` status for visual consistency), `.builder-edge` (the four edge types share one style; the spec does not call for per-type edge colors at v0.1), `.narrowing-notice` (a muted callout using `--node-fg-muted`) + `.narrowing-notice__rejected` (the narrowing-failure line — the same red token as `--invalid`). Per gotcha #67 (a className rendered in the DOM with no CSS rule is an invisible bug), every new class gets a corresponding rule in `src/styles.css` and the styles test asserts it via the existing `every_class_has_a_corresponding_CSS_rule` pattern. Theme variables only — do not reintroduce the M07-IRL #3 low-contrast bug Stage A fixed.

### D2.4 Tests

#### D2.4.1 `connectEdge` unit tests — `tests/unit/lib/builderStore.connectEdge.test.ts`

- `connect_agent_to_skill_pushes_skill_name_to_allowed_skills`
- `connect_agent_to_tool_pushes_tool_name_to_allowed_tools`
- `connect_agent_to_agent_pushes_child_id_to_spawns`
- `connect_hook_to_task_pushes_hook_id_to_task_defaults_post_hooks`
- `connect_tool_to_tool_is_rejected_no_framework_mutation` (the reject path — a non-spec pair)
- `connect_skill_to_agent_is_rejected_wrong_direction` (reject — direction matters)
- `connect_agent_to_skill_twice_does_not_duplicate_the_allowed_skills_entry` (idempotency)
- `connectEdge_mutates_framework_and_the_canvasEdges_projection_re_derives` (ADR-0020 — the projection is a pure function of `framework`)

#### D2.4.2 Continuous-validation trigger unit tests — same file

- `addNode_schedules_a_debounced_validateFramework_call`
- `a_burst_of_framework_mutations_fires_validateFramework_once` (debounce coalescing — use `vi.useFakeTimers()`)
- `validateFramework_result_lands_in_the_validation_slot`
- `a_failed_validateFramework_leaves_the_prior_validation_report_in_place` (no flicker-to-clean)

#### D2.4.3 Narrowing notice + badge component tests

`tests/unit/components/builder/NarrowingNotice.test.tsx`:

- `renders_nothing_when_no_spawn_edge_in_capability_summary_matches_the_id`
- `renders_child_declared_and_surviving_capability_lists_for_an_ok_narrowing`
- `renders_the_rejection_message_when_the_child_exceeds_the_parent` (the `Err` arm — `.narrowing-notice__rejected`)
- `surfaces_the_backend_narrowed_caps_verbatim_no_TS_intersection` (contract test, gotcha #66 — assert the rendered surviving set is exactly the report's `narrowed_caps.Ok`, proving no TS recomputation)

`tests/unit/components/builder/BuilderNodeBadge.test.tsx`:

- `node_with_no_validation_errors_renders_no_badge`
- `node_with_a_schema_error_keyed_to_its_path_renders_the_red_badge`
- `node_with_a_capability_error_keyed_to_its_path_renders_the_red_badge`
- `badge_count_reflects_the_number_of_errors_for_that_node`
- `a_node_not_keyed_by_any_error_renders_no_badge_even_when_the_report_has_errors` (the per-node filter)

#### D2.4.4 Playwright behavior test — `tests/e2e/builder_edges.spec.ts`

Drives the renderer against the Vite dev server with `@tauri-apps/api` module-mocked (gotcha #23 — Playwright cannot drive the Tauri window; `validate_framework` is mocked to return a scripted `FrameworkValidationReport`). React Flow edge creation goes through the handle-drag in Playwright's drag API.

```ts
test.describe.configure({ timeout: 90_000 }); // gotcha #53 (Vite cold-start)

test('Agent→Skill edge adds the skill to allowed_skills', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click();   // C's view switch
  // (drag an Agent + a Skill from the Palette — D1 path — then:)
  await dragEdge(page, '[data-builder-node="agent"] .source-handle',
                       '[data-builder-node="skill"] .target-handle');
  await expect(page.locator('.builder-edge')).toHaveCount(1);
  // the framework projection reflects allowed_skills — assert via the
  // JSON the Inspector will render in E; here assert the edge painted.
});

test('Agent→Agent edge whose child over-declares surfaces a narrowing rejection', async ({ page }) => {
  // validate_framework mock returns a capability_summary whose spawn edge
  // has narrowed_caps = { Err: "...exec:net.fetch..." }: the child
  // declared exec:net.fetch and the parent does not hold it. All-or-
  // nothing narrowing (M05.B) rejects the whole edge.
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click();
  await dragEdge(page, '[data-builder-node="agent"][data-role="parent"] .source-handle',
                       '[data-builder-node="agent"][data-role="child"] .target-handle');
  await expect(page.locator('.narrowing-notice__rejected')).toContainText('net.fetch');
});

test('an invalid framework paints red badges on the offending nodes', async ({ page }) => {
  // validate_framework mock returns a schema_error keyed to the agent node path.
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click();
  await expect(page.locator('.builder-node--invalid .builder-node__badge')).toBeVisible();
});

test('an invalid node-pair connection is rejected — no edge appears', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click();
  await dragEdge(page, '[data-builder-node="tool"] .source-handle',
                       '[data-builder-node="tool"]:nth-of-type(2) .target-handle');
  await expect(page.locator('.builder-edge')).toHaveCount(0);
});
```

#### D2.4.5 Schema regen check

- N/A — Stage D2 ships no schema changes (it consumes Stage B's `validate_framework`).

#### D2.4.6 Acceptance criteria

- [ ] `npm run test` — all vitest tests pass; renderer ≥80 (vitest, `vitest.config.ts` gate) on `src/lib/builderStore.ts` + `src/components/builder/NarrowingNotice.tsx` + the builder node components' badge path
- [ ] `npx tsc --noEmit` — clean
- [ ] `npx eslint .` — clean
- [ ] `npx prettier --check '**/*.{ts,tsx,js,jsx,json}'` — clean
- [ ] `npm run test:e2e -- builder_edges.spec.ts` — passes against the Vite dev server (curl warmup probe per the v1.6 `<playwright_warmup_recipe>`)
- [ ] MVP §M8 criterion 2 (Agent→Skill edge → `allowed_skills`) + criterion 3 (Agent→Agent narrowing surfaced) + the badge half of criterion 4 demonstrated in Playwright
- [ ] The four edge types + the reject path are unit-tested; `connectEdge` is idempotent
- [ ] The Agent→Agent narrowing intersection is Stage B's Rust report — **no TS re-implementation** (the `surfaces_the_backend_narrowed_caps_verbatim` contract test confirms it; spec §9)
- [ ] Continuous validation is debounced; it shares the `validate_framework` validator with Stage E's explicit Validate trigger (one validator, two triggers)
- [ ] `<wire_signature_audit>` + `shape=` claims match the shipped Stage B `validate_framework` wire
- [ ] Every new CSS class has a corresponding rule in `src/styles.css` (gotcha #67 + the `every_class_has_a_corresponding_CSS_rule` pattern)
- [ ] Strict v1.7 two-commit TDD: `git diff <red>..<impl> -- '**/tests/**'` is EMPTY
- [ ] CI-parity per G6

### D2.5 CLI Prompt

```xml
<work_stage_prompt id="M08.D2">
  <context>
    M08 Stage D2 — the Builder Canvas edge editor: the four edge types
    (Agent→Skill = allowed_skills, Agent→Tool = allowed_tools,
    Agent→Agent = spawns, Hook→Task = task_defaults.post_hooks; every
    other node-pair rejected), automatic Agent→Agent capability
    narrowing SURFACED from Stage B's validate_framework report (the
    intersection is Rust — capability/narrowing.rs — NEVER
    re-implemented in TS, spec §9), and continuous DEBOUNCED schema
    validation with red badges on offending nodes. Pure renderer over
    Stage B's validate_framework command + D1's BuilderCanvas + the
    C/D1 builderStore (ADR-0020 — framework.json the source of truth).
    NO new schema, NO backend, NO new Tauri command. Pin the shipped
    Stage B wire BEFORE any pseudocode.
  </context>
  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — wire_signature_audit + phase_doc_inventory_audit shape=)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Stage D2 D2.1–D2.4 + Stage B B.3.1/B.3.4 + Stage D1 as the immediate predecessor)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543; the four edge types + the narrowing rule + "schema runs continuously, errors as red badges") + §8.security L2a (narrowing) + §9 (no TS/Rust validation duplication)</file>
    <file>docs/MVP-v0.1.md §M8 (criteria 2 / 3 / 4)</file>
    <file>docs/build-prompts/retrospectives/M08.D1-retrospective.md + RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/gotchas.md (#23 Playwright cannot drive the Tauri window, #27 vitest re-query after await, #30 unwrapCmdError, #53 Vite cold-start, #66 contract tests, #67 component+CSS contract, #75 useShallow for derived selectors)</file>
  </read_first>
  <read_reference>
    <file purpose="The SHIPPED Stage B validate_framework command signature + the FrameworkValidationReport return shape — PIN params verbatim before authoring the wrapper">src-tauri/src/commands.rs</file>
    <file purpose="Stage B builder module — validate.rs FrameworkValidationReport / NodeError; the per-spawn-edge narrowing triple rides on the report's capability_summary.spawn_edges[] (defined in summary.rs)">crates/runtime-main/src/builder/validate.rs</file>
    <file purpose="Stage B summary.rs — FrameworkCapabilitySummary + SpawnEdgeNarrowing; capability_summary rides on the validation report (B.3.4). D2 reads its spawn_edges[] for NarrowingNotice; Stage E reads the whole-framework totals — both off the one report, no separate command">crates/runtime-main/src/builder/summary.rs</file>
    <file purpose="M05.B L2a narrow() — the capability intersection Stage B's report computes; D2 SURFACES the result, NEVER re-implements it (spec §9)">crates/runtime-main/src/capability/narrowing.rs</file>
    <file purpose="D1's BuilderCanvas — D2 fills the /* onConnect is D2 */ slot + adds the canvasEdges projection">src/components/builder/BuilderCanvas.tsx</file>
    <file purpose="C/D1 builderStore — D2 adds connectEdge, the validation slot, the debounced trigger; framework.json is the source of truth (ADR-0020)">src/lib/builderStore.ts</file>
    <file purpose="D1's builder node components — D2 adds the red validation badge">src/components/builder/nodes</file>
    <file purpose="ipc.ts existing wrapper idiom — invoke + camelCase args + discriminated returns + unwrapCmdError; the importArtifact / mcpTestConnection wire_signature_audit precedent (ipc.ts:194,266 document prior drift)">src/lib/ipc.ts</file>
    <file purpose="The read-only GraphCanvas — the module-level nodeTypes convention + the React Flow v12 onConnect/edge idiom; D2's BuilderCanvas is a SEPARATE interactive component">src/components/GraphCanvas.tsx</file>
    <file purpose="styles.css class conventions + theme variables (--node-fg / --node-fg-muted) — no M07-IRL #3 contrast regression">src/styles.css</file>
  </read_reference>
  <read_prior_stages>
    <stage id="M08.D1" retro="docs/build-prompts/retrospectives/M08.D1-retrospective.md"/>
  </read_prior_stages>
  <deliverable ref="docs/build-prompts/M08-workbench.md" section="D2.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the D2.4 buckets — vitest
      (connectEdge per edge type + the reject path + idempotency; the
      debounced validation trigger with vi.useFakeTimers(); the
      NarrowingNotice triple rendering; the red-badge per-node filter)
      and Playwright (Agent→Skill edge; Agent→Agent narrowing surfaced;
      red badges; invalid-pair reject — validate_framework mocked).
      Stub the production surfaces just enough that the test files
      compile (the connectEdge / NarrowingNotice / validateFramework
      symbols resolve). Confirm tests fail with right-reason errors per
      CLAUDE.md §5 (assertion failed / cannot find name / module not
      found — NOT a test-file compile error and NOT a tautological
      pass). Commit as a STANDALONE `test(M08.D2): failing tests for
      the Builder Canvas edge editor` on the M08 parent-milestone
      branch BEFORE green-phase impl; the commit body pastes the first
      ~40 lines of the vitest/Playwright run proving the
      expected-failure class. Surface the red-phase commit; user
      approves before green phase begins.
    </red_phase>
    <green_phase>
      Implement connectEdge + the four edge types + the reject path,
      the onConnect wiring + the edge projection, the debounced
      continuous-validation trigger + the validation slot, the red
      validation badges, and NarrowingNotice until ALL failing tests
      pass. Do NOT modify the test files during implementation — if a
      test is wrong, fix it in a SEPARATE labelled follow-up commit
      with explanation, never silently in the impl commit. The impl
      commit body MUST state the verifiable invariant: `git diff
      &lt;red-sha&gt;..&lt;impl-sha&gt; -- '**/tests/**'` is EMPTY.
      Net-new additive tests + mechanical prettier/eslint fixes to test
      files go in the separate follow-up commit. No Co-Authored-By in
      any commit message.
    </green_phase>
  </tdd_discipline>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="D2.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>
  <gates milestone="M08"/>
  <coverage_gate>
    <gate scope="renderer" target_lines="80" ignore_filename_regex="generated"/>
  </coverage_gate>
  <self_correction_budget>3</self_correction_budget>
  <adr_triggers>
    <trigger>None new in D2 — it implements against ADR-0020 (the Builder canvas↔framework.json state model, filed at Stage C). D2 adds no schema, no backend, no IPC protocol change, no dependency — it consumes Stage B's validate_framework command. If the build finds it NEEDS a push validation event rather than the command return (e.g. background validation), that is a §14 trigger (schema + cargo xtask regenerate-types + ADR) — surface BEFORE the red phase; do not author it silently.</trigger>
  </adr_triggers>
  <gotchas>
    <trap>#23 — Playwright drives the Vite dev server with @tauri-apps/api module-mocked; validate_framework is mocked to a scripted FrameworkValidationReport. React Flow edge creation goes through the handle-to-handle drag in Playwright's drag API; it cannot drive the Tauri window.</trap>
    <trap>Spec §9 — the Agent→Agent narrowing decision is Stage B's Rust report (capability/narrowing.rs::narrow()). The renderer SURFACES the SpawnEdgeNarrowing record ({parent_caps, child_declared_caps, narrowed_caps}) from capability_summary.spawn_edges[]; it NEVER computes an intersection in TS. narrowed_caps is a serde-tagged Result — render the `Ok` surviving set or the `Err` rejection message verbatim; do not recompute either, and do NOT compute a `removed` set-difference (narrow is all-or-nothing — `Ok` is `proposed` verbatim, there is no partial clamp).</trap>
    <trap>Continuous validation is DEBOUNCED — every keystroke into an inline-config field must NOT fire validate_framework; coalesce a burst into one call. The Validate button (Stage E) is the explicit trigger of the SAME validator with no debounce.</trap>
    <trap>connectEdge must REJECT every node-pair that is not one of the four spec edge types — a rejected pair mutates nothing and paints no edge (the canvas edge projection is a pure function of framework — ADR-0020).</trap>
    <trap>#75 — the per-node validation-error list + the per-edge narrowing triple are DERIVED selectors; wrap them in useShallow (zustand/react/shallow) so a node/notice re-renders only on its own slice.</trap>
    <trap>#67 — every new className (.builder-node--invalid, .builder-edge, .narrowing-notice, .narrowing-notice__rejected) gets a corresponding src/styles.css rule + the every_class_has_a_corresponding_CSS_rule static test.</trap>
    <trap>React Flow v12 nodeTypes stays MODULE-LEVEL (the GraphCanvas.tsx trap — redefining it per render re-mounts every node). D1 already pinned this; D2 must not regress it when touching BuilderCanvas.</trap>
    <trap>#30 — a validate_framework error thrown across the bridge is unwrapped via the existing unwrapCmdError helper, not String(e).</trap>
  </gotchas>
  <execution_warnings>
    <warning>DO NOT add a Tauri command in D2 — Stage B shipped validate_framework. D2 wires a thin ipc.ts wrapper around it. If a new command appears necessary, surface in the retrospective; the default is "D2 consumes Stage B's wire".</warning>
    <warning>DO NOT compute the capability intersection in TS — that is a spec §9 violation and a duplication of capability/narrowing.rs. D2 reads narrowed_caps from Stage B's report. The contract test surfaces_the_backend_narrowed_caps_verbatim is the guard.</warning>
    <warning>DO NOT edit the read-only GraphCanvas.tsx or the live-execution graphStore.ts — D2 acts only on the SEPARATE builderStore + BuilderCanvas. Conflating the live-execution graph with the Builder canvas is the dual-mode complexity the D-split exists to avoid.</warning>
    <warning>DO NOT fire validate_framework on every keystroke — the debounce is load-bearing. An un-debounced continuous pass floods the Tauri bridge during inline-config typing.</warning>
    <warning>DO NOT introduce per-edge-type edge colors / styles — v0.1 spec does not call for them; the four edge types share one .builder-edge style. Scope creep beyond MVP §M8.</warning>
  </execution_warnings>
  <pre_flight_check>
    <check name="branch">HEAD == the M08 parent-milestone branch; the M08.D1 impl commit is present (the BuilderCanvas + builder node components + the C/D1 builderStore must be shipped)</check>
    <check name="stage_b_wire">grep-confirm the Stage B validate_framework command in src-tauri/src/commands.rs + the FrameworkValidationReport (incl. its capability_summary field carrying FrameworkCapabilitySummary.spawn_edges) in crates/runtime-main/src/builder/ are shipped before pinning the wrapper</check>
    <check name="d1_onconnect_slot">grep-confirm BuilderCanvas.tsx has the D1 onConnect slot (the /* onConnect is D2 */ marker) — D2 fills it, it does not re-architect the canvas</check>
  </pre_flight_check>
  <phase_doc_inventory_audit>
    <claim type="file" path="src/components/builder/BuilderCanvas.tsx" verified="true" note="D1 shipped; D2 adds onConnect + the edge projection"/>
    <claim type="file" path="src/lib/builderStore.ts" verified="true" note="C/D1 shipped; D2 adds connectEdge + validation slot + debounced trigger"/>
    <claim type="file" path="src/components/builder/NarrowingNotice.tsx" verified="false" note="D2 creates"/>
    <claim type="ipc_command" path="src-tauri/src/commands.rs" symbol="validate_framework" verified="true" note="Stage B shipped"/>
    <claim type="store_slot" path="src/lib/builderStore.ts" symbol="validation" shape="PIN the FrameworkValidationReport TS type from the Stage B-shipped commands.rs / builder module — schema-generated or hand-mirrored per Stage B's actual output (the McpTool / ImportOutcome precedent)" verified="true"/>
  </phase_doc_inventory_audit>
  <wire_signature_audit>
    <wrapper ipc_command="validate_framework" actual_params="PIN from the Stage B-shipped src-tauri/src/commands.rs signature (B.3.1 ships it as a synchronous command return — confirm the param object name, e.g. { doc })" phase_doc_assumed="{ doc } — author reconciles against the shipped signature"/>
  </wire_signature_audit>
  <runtime_environment os="windows" note="Vitest + Playwright run against the Vite dev server; Vite cold-start mitigated per gotcha #53"/>
  <time_box hours="10-13"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the pinned validate_framework params + the FrameworkValidationReport / NodeError / FrameworkCapabilitySummary / SpawnEdgeNarrowing shape (from the shipped Stage B code — was it schema-generated or hand-mirrored? Confirm capability_summary is a report field carrying spawn_edges[], and narrowed_caps is a serde-tagged Result). Confirm the Agent→Agent narrowing is Stage B's Rust report with NO TS intersection (cite the contract test). Confirm continuous validation is debounced + shares the validate_framework validator with Stage E's explicit Validate trigger. Note whether connectEdge's reject path needed a node-kind lookup helper that Stage E / M09 should reuse. MVP §M8 criteria 2/3 Playwright results.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="D2.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M08.*-retrospective.md); strict-TDD '**/tests/**' EMPTY proof</item>
    <item>wire_signature_audit + phase_doc_inventory_audit shape= reconciliation against the shipped Stage B validate_framework wire</item>
    <item>the four edge types + the reject path (unit-tested + idempotent); the narrowing surfaced from Rust with NO TS intersection (the contract test)</item>
    <item>continuous validation debounced; one validator, two triggers confirmed</item>
    <item>gate results (v1.6 canonical order; renderer ≥80 vitest; Playwright + warmup; every-class-has-CSS-rule; CI-parity per G6)</item>
    <item>MVP §M8 criteria 2/3 + the badge half of 4 Playwright demonstration</item>
    <item>M08.D2 retrospective [END]; explicit "Stage M08.D2 is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### D2.6 Commit Message

```
feat(renderer): M08 Stage D2 — Builder Canvas edges + capability narrowing + continuous validation

The Builder Canvas edge editor: React Flow onConnect wired to a
builderStore.connectEdge action mapping the four spec edge types
(Agent→Skill = allowed_skills, Agent→Tool = allowed_tools,
Agent→Agent = spawns, Hook→Task = task_defaults.post_hooks) — every
other node-pair rejected, no edge created. connectEdge mutates
builderStore.framework (the source of truth, ADR-0020); the canvas
edge projection re-derives.

Automatic Agent→Agent capability narrowing surfaced from Stage B's
validate_framework report — the {parent_caps, child_declared_caps,
narrowed_caps} triple on report.capability_summary.spawn_edges[],
computed by capability/narrowing.rs in Rust (spec §9 — never
re-implemented in TS). narrowed_caps is the all-or-nothing Result:
NarrowingNotice renders the surviving set on an Ok edge and the
rejection on an Err edge; the child node's capability disclosure
reflects the narrowing result.

Continuous schema validation: every framework mutation triggers a
debounced validate_framework call; the FrameworkValidationReport
lands in builderStore.validation; each builder node renders a red
badge when the report keys an error to its path — including a
folded capability_error for an over-declaring Agent→Agent edge. One
Rust validator, two triggers — D2's continuous debounced pass and
Stage E's explicit Validate button.

MVP §M8 criteria 2 + 3 + the badge half of 4. No new schema, no
backend, no new Tauri command — D2 consumes Stage B's wire.

v1.8 wire_signature_audit + phase_doc_inventory_audit shape= pinned
to the shipped Stage B validate_framework wire. Playwright + vitest;
renderer ≥80. Strict two-commit TDD: '**/tests/**' diff EMPTY.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage E — Inspector + canvas↔JSON two-way binding

### E.1 Problem Statement

Spec Phase 9, the Inspector (right sidebar): *"Live preview of generated `framework.json`; Diff view against the file on disk; Capability summary across the entire framework; Export: write `framework.json` + companion `skill.md`/`tool.md`/`agent.md` files to a directory; 'Validate' button runs `schemas/*.json` validation explicitly; 'Test' button runs L3 sandbox."* MVP §M8 criteria 4 (Validate), 5 (Test), 6 (Canvas | JSON tabs with two-way binding), 7 (Save to disk), 8 (Reload from disk). Stage C shipped the three-panel `BuilderShell` with an **empty Inspector region**; D1/D2 made the center Canvas a live node+edge editor. Stage E fills the Inspector and adds the **Canvas | JSON** tab toggle — the last piece of the workbench shell before the Tester (F1/F2) and Settings (G).

The Inspector is the user's window onto what the canvas has actually built. It shows the live `framework.json`, diffs it against the last-saved file on disk, summarizes the whole-framework capability footprint, and gives two action buttons: **Validate** (run the schema + capability check explicitly) and **Test** (open the Tester). The Canvas | JSON toggle gives the experienced user a second editor — a raw JSON view — over the *same document*. This is where ADR-0020 pays off: because `builderStore.framework` is the single source of truth, the canvas and the JSON tab are simply two editors over one document. Canvas edits already mutate `framework` (D1/D2); the JSON view renders it. A JSON edit is parsed and, on valid JSON, *replaces* the document via `replaceFramework`, and the canvas re-derives. The two-way binding is a *consequence* of the source-of-truth model — not separate two-way-sync machinery to build and keep consistent. The one place this needs care is the **invalid-JSON path**: a half-typed or malformed JSON edit must surface a parse error inline and leave the store untouched — never call `replaceFramework` with garbage and desync the canvas.

E is pure renderer. It consumes Stage B's `save_framework` / `load_framework` / `validate_framework` commands (no new backend) and Stage C's `@tauri-apps/plugin-dialog` file picker. The Inspector's **Test** button is shipped *inert-but-wired*: E ships the button and the `builderStore.openTester` action it calls; Stage F2 delivers the actual Tester modal that renders on that state. This is deliberate incremental construction — the M07.E "wired-but-pending" precedent — and is stated here and again in F2.1, so it is not mistaken for dead code.

Concrete deliverables:

1. **`Inspector.tsx`** — the right-panel Inspector: a live `framework.json` preview, a disk diff (`framework` vs `diskFramework`), the whole-framework capability summary (read from the `validate_framework` report's capability-summary field — not a separate command), a **Validate** button, and a **Test** button.
2. **`JsonView.tsx`** — the JSON-tab editor over `builderStore.framework`: a valid edit parses → `replaceFramework`; invalid JSON surfaces an inline parse error and does **not** mutate the store.
3. **`BuilderShell.tsx` extension** — the **Canvas | JSON** tab toggle in the center region; mount `Inspector` in the right region (Stage C's stub).
4. **`builderStore` extensions** — `replaceFramework` (the JSON-tab edit + the load path); `openTester` / `closeTester` + the `testerOpen` slot (the Test button's target state — F2's modal renders on it); `diskFramework` set on save/load (so the disk diff zeroes after a save).
5. **`saveFramework` / `loadFramework` IPC wrappers** — `src/lib/ipc.ts`; params PINNED to the shipped Stage B commands via the `<wire_signature_audit>`.
6. **Save / Load affordances** — Export/Save opens the file picker for a directory → `save_framework`; Open/Load picks a directory → `load_framework` → `replaceFramework` + `diskFramework`.
7. **Playwright + vitest behavior coverage** — renderer ≥80.

Not in this stage:

- The Tester modal itself (Stage F2 — E ships the Test button + `openTester`; the button is inert until F2).
- The Tester backend / isolated session (Stage F1).
- The Settings panel + tier promotion (Stage G).
- Any new schema, backend, or Tauri command — E consumes Stage B's `save_framework` / `load_framework` / `validate_framework`.
- A new validator — the **Validate** button runs the **same** `validate_framework` D2's continuous pass uses (spec §9, "no duplication").

### E.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/components/builder/Inspector.tsx` | **new** | Live `framework.json` preview; disk diff; whole-framework capability summary; Validate + Test buttons. |
| `src/components/builder/JsonView.tsx` | **new** | The JSON-tab editor; parse → `replaceFramework`; inline parse-error surfacing on invalid JSON. |
| `src/components/builder/BuilderShell.tsx` | exists (C) | The Canvas \| JSON tab toggle in the center region; mount `Inspector` in the right region (the C stub). |
| `src/lib/builderStore.ts` | exists (C/D1/D2) | `replaceFramework` (JSON-tab edit + load); `openTester` / `closeTester` + the `testerOpen` slot (the Test button's target — F2's modal renders on it); `diskFramework` set on save/load. |
| `src/lib/ipc.ts` | exists | `saveFramework` / `loadFramework` wrappers — params PINNED to the Stage B commands via `<wire_signature_audit>`. |
| `src/styles.css` | exists | `.builder-tab`, `.builder-tab--active`, `.inspector-section`, `.inspector__diff`, `.json-view`, `.json-view__error` classes — theme variables, no contrast regression. |
| `tests/unit/lib/builderStore.replaceFramework.test.ts` | **new** | `replaceFramework`, the diff computation, `openTester`, the parse-error-does-not-desync path. |
| `tests/unit/components/builder/Inspector.test.tsx` | **new** | Preview, diff, capability summary, Validate/Test button wiring. |
| `tests/unit/components/builder/JsonView.test.tsx` | **new** | Valid edit → `replaceFramework`; invalid JSON → parse error, store untouched. |
| `tests/e2e/builder_inspector.spec.ts` | **new** | Playwright — Validate; JSON edit round-trips to the canvas; invalid JSON; Save; Load. |
| `CHANGELOG.md` / retro | exist/new | Stage E entries. |

Effort budget: ~9–12 hours. The largest piece is `JsonView`'s parse-and-route logic with the invalid-JSON no-desync guard and the save/load file-picker plumbing; the Inspector's four sections are presentational reads over `builderStore` and Stage B's report.

### E.3 Detailed Changes

#### E.3.1 Wire pinning (v1.8 discipline — BEFORE pseudocode)

Pin to the **shipped** Stage B wire, not assumptions — the M06.E / M07.E drift lesson (`ipc.ts:194,266` document two prior `wire_signature_audit` reconciliations). Before authoring any of the component or store pseudocode below:

- `<wire_signature_audit>` (E.5): read `src-tauri/src/commands.rs` for the **committed** Stage B `save_framework` and `load_framework` signatures; record `actual_params` verbatim. `validate_framework` was pinned at D2 — **reuse that report shape**; the whole-framework capability summary is the `capability_summary` *field on* the `FrameworkValidationReport` (Stage B B.3.2 ships the field; B.3.4 defines `FrameworkCapabilitySummary` — the report carries it), not a separate command.
- `<phase_doc_inventory_audit shape=…>` (E.5): pin the `LoadedFramework` TS type (Stage B's `load_framework` return — B.3.2). Whether it is schema-generated or hand-mirrored (the `McpTool` / `ImportOutcome` precedent) is decided by Stage B's actual output.

The save/load wrappers, illustrative — final param shapes come from the pinned signatures:

```ts
// src/lib/ipc.ts (additions) — params PINNED to the Stage B-shipped
// `save_framework` / `load_framework` commands via <wire_signature_audit>.

/**
 * Write framework.json + companion skill.md/tool.md/agent.md to `dir`
 * (M08 Stage B `save_framework`). `dir` is the directory the
 * @tauri-apps/plugin-dialog picker returned (Stage C). The backend
 * persistence is path-agnostic (&Path — CLAUDE.md §9); the renderer
 * supplies the resolved path. Errors → unwrapCmdError.
 */
export async function saveFramework(dir: string, fw: unknown): Promise<void> {
  await invoke('save_framework', { dir, fw });
}

/** Mirrors `runtime_main::builder::LoadedFramework` (M08 Stage B). */
export interface LoadedFramework {
  framework: unknown; // the generated Framework shape — PIN via shape=
  // (any companion-artifact metadata Stage B's load_framework returns)
}

/**
 * Read framework.json from `dir` (M08 Stage B `load_framework` — reuses
 * the Phase 6 framework_loader walker for the read). A save→load→save
 * cycle is byte-stable (Stage B B.3.2). The renderer feeds the result
 * to builderStore.replaceFramework; the canvas re-derives (ADR-0020).
 */
export async function loadFramework(dir: string): Promise<LoadedFramework> {
  return await invoke<LoadedFramework>('load_framework', { dir });
}
```

#### E.3.2 The Inspector — `src/components/builder/Inspector.tsx`

The right-panel Inspector renders four sections plus two buttons, all reading from `builderStore` (and, for the capability summary, the `validate_framework` report Stage B already produces — D2 stores it in `builderStore.validation`):

```tsx
// src/components/builder/Inspector.tsx — new component.

export function Inspector(): JSX.Element {
  const framework = useBuilderStore((s) => s.framework);
  const diskFramework = useBuilderStore((s) => s.diskFramework);
  const validation = useBuilderStore((s) => s.validation);   // D2's slot
  const openTester = useBuilderStore((s) => s.openTester);
  const [report, setReport] = useState<FrameworkValidationReport | null>(null);

  // The Validate button runs the SAME validate_framework D2's
  // continuous pass uses — one Rust validator, two triggers (spec §9).
  // The continuous pass is debounced; this explicit run is immediate.
  const onValidate = useCallback(async () => {
    setReport(await validateFramework(framework));
  }, [framework]);

  return (
    <aside className="builder-inspector">
      {/* 1. Live framework.json preview — updates as the canvas edits it. */}
      <section className="inspector-section inspector-section--preview">
        <h3>framework.json</h3>
        <pre>{JSON.stringify(framework, null, 2)}</pre>
      </section>

      {/* 2. Disk diff — framework vs diskFramework; shown only when the
            framework has a disk origin (diskFramework !== null). */}
      {diskFramework !== null && (
        <section className="inspector-section inspector__diff">
          <h3>Changes since save</h3>
          {/* a line/field diff of framework vs diskFramework */}
        </section>
      )}

      {/* 3. Capability summary — the whole-framework totals carried on
            the validate_framework report's capability_summary field
            (Stage B B.3.4; NOT a separate command). */}
      <section className="inspector-section inspector-section--capabilities">
        <h3>Capability summary</h3>
        {/* render (report ?? validation)?.capability_summary */}
      </section>

      {/* 4. Action buttons. */}
      <div className="inspector__actions">
        <button onClick={onValidate}>Validate</button>
        {/* Test ships INERT-but-wired — it sets builderStore.openTester;
            Stage F2 delivers the modal that renders on that state.
            Incremental construction, not dead code (see F2.1). */}
        <button onClick={() => openTester()}>Test</button>
      </div>
    </aside>
  );
}
```

The Validate button surfaces the full `FrameworkValidationReport` — the per-node `schema_errors` / `capability_errors` messages D2's badges only counted. The capability summary reads the `capability_summary` field on that report (Stage B B.3.4 explicitly designed the summary to ride on the report so the Inspector and the canvas render one consistent capability picture from one backend computation) — there is no separate `framework_capability_summary` command for the renderer to call. The **Test** button is wired to `builderStore.openTester` and ships inert: F2 renders the Tester modal on the `openTester` state. E's tests assert the button calls `openTester`; they do not assert a modal opens (F2's concern).

#### E.3.3 Save / load — `framework.json` + companion `.md` (MVP §M8 criteria 7 + 8)

The Inspector's **Export/Save** affordance opens Stage C's `@tauri-apps/plugin-dialog` directory picker, then calls Stage B's `save_framework`; on success it records `builderStore.diskFramework` so the disk diff (E.3.2 section 2) zeroes. The **Open/Load** affordance picks a directory, calls `load_framework`, and feeds the result through `replaceFramework` + sets `diskFramework`:

```ts
// In Inspector.tsx (or a small save/load module) — illustrative.
import { open } from '@tauri-apps/plugin-dialog'; // Stage C registered it

async function onSave(framework: unknown): Promise<void> {
  const dir = await open({ directory: true });        // the native picker
  if (typeof dir !== 'string') return;                // user cancelled
  await saveFramework(dir, framework);
  // diskFramework := framework so the Inspector diff zeroes (E.3.2 §2).
  useBuilderStore.getState().setDiskFramework(framework);
}

async function onLoad(): Promise<void> {
  const dir = await open({ directory: true });
  if (typeof dir !== 'string') return;
  const loaded = await loadFramework(dir);
  // replaceFramework swaps the source of truth; the canvas re-derives
  // (ADR-0020). diskFramework := the loaded doc so the diff starts clean.
  useBuilderStore.getState().replaceFramework(loaded.framework);
  useBuilderStore.getState().setDiskFramework(loaded.framework);
}
```

MVP §M8 criterion 7: Save → `framework.json` + companion `skill.md`/`tool.md`/`agent.md` at the chosen directory (Stage B writes the companion `.md` for any artifact the canvas defined inline). MVP §M8 criterion 8: a reload reconstructs the canvas **identical** to save state — guaranteed by Stage B's byte-stable save→load round-trip (B.3.2) *and* the canvas projection being a pure function of `framework` (ADR-0020). E does not re-verify byte-stability — that is Stage B's tested contract; E verifies the renderer wiring (picker → command → store → re-derive).

#### E.3.4 The `builderStore` extensions — `replaceFramework` / `openTester` / `diskFramework`

```ts
// src/lib/builderStore.ts — Stage E additions to the C/D1/D2 store.

// replaceFramework — swap the entire source-of-truth document. Used by
// the JSON tab (E.3.5, on a VALID parse) and the load path (E.3.3).
// The canvas node/edge projections are pure functions of `framework`,
// so they re-derive automatically — no separate canvas reset (ADR-0020).
replaceFramework: (fw: Framework): void =>
  set(() => ({ framework: fw /* canvasNodes/canvasEdges re-derive */ })),

// setDiskFramework — record the last-saved/loaded snapshot for the
// Inspector disk diff (E.3.2 §2). Set on save success + on load.
setDiskFramework: (fw: Framework): void => set(() => ({ diskFramework: fw })),

// testerOpen / openTester / closeTester — the Inspector Test button's
// target. E ships the `testerOpen: boolean` slot + openTester (sets it
// true) + closeTester (sets it false); F2's modal (F2.3.2) renders
// when testerOpen is true. INERT-but-wired at E (the M07.E precedent).
openTester: (): void => set(() => ({ testerOpen: true })),
closeTester: (): void => set(() => ({ testerOpen: false })),
```

`replaceFramework` is the single mutation both the JSON tab and the load path use to swap the document; because `canvasNodes` / `canvasEdges` are pure projections of `framework` (D1/D2), the canvas re-derives with no extra wiring. `diskFramework` was declared as a slot in Stage C (the `BuilderState` shape); E is the stage that actually *sets* it (on save and load) and *reads* it (the Inspector diff). `openTester` / `closeTester` are shipped inert-but-wired per the incremental-construction pattern — F2's modal consumes the `testerOpen` slot they set.

#### E.3.5 Canvas | JSON two-way binding — `src/components/builder/JsonView.tsx`

`BuilderShell` gains a **Canvas | JSON** tab toggle in the center region. The JSON tab is a text editor over `builderStore.framework`:

```tsx
// src/components/builder/JsonView.tsx — new component.
// The JSON tab is just ANOTHER editor over builderStore.framework,
// exactly as the canvas is — ADR-0020. There is no separate two-way
// sync to maintain; the binding falls out of the source-of-truth model.

export function JsonView(): JSX.Element {
  const framework = useBuilderStore((s) => s.framework);
  const replaceFramework = useBuilderStore((s) => s.replaceFramework);
  // Local editing buffer — the textarea is uncontrolled-against-store so
  // a half-typed (invalid) edit does NOT round-trip through the store.
  const [draft, setDraft] = useState(() => JSON.stringify(framework, null, 2));
  const [parseError, setParseError] = useState<string | null>(null);

  // Canvas → JSON: when the canvas mutates `framework` (D1/D2) and the
  // user is NOT mid-edit, re-seed the draft from the store.
  useEffect(() => {
    setDraft(JSON.stringify(framework, null, 2));
  }, [framework]);

  const onChange = (text: string): void => {
    setDraft(text);
    // JSON → Canvas: parse the edit.
    try {
      const parsed = JSON.parse(text);
      setParseError(null);
      replaceFramework(parsed);   // valid → swap the doc; canvas re-derives
    } catch (e) {
      // INVALID JSON: surface the parse error inline; do NOT call
      // replaceFramework — leaving the store untouched (no desync).
      setParseError(e instanceof Error ? e.message : 'Invalid JSON');
    }
  };

  return (
    <div className="json-view">
      <textarea value={draft} onChange={(e) => onChange(e.target.value)} spellCheck={false} />
      {parseError !== null && (
        <p className="json-view__error" role="alert">{parseError}</p>
      )}
    </div>
  );
}
```

- **Canvas → JSON:** canvas edits already mutate `builderStore.framework` (D1/D2); the JSON view re-renders from it — automatic.
- **JSON → Canvas:** an edit in the JSON tab is parsed; on **valid** JSON → `builderStore.replaceFramework(parsed)` → the canvas projection re-derives (ADR-0020); on **invalid** JSON → **no** `replaceFramework`, a parse error surfaced inline. This is the load-bearing guard: a malformed or half-typed JSON edit must never reach `replaceFramework` and desync the canvas from the store.

MVP §M8 criterion 6: edit the JSON directly, switch back to Canvas, the canvas shows the update. ADR-0020 makes this fall out of the source-of-truth model — `JsonView` is just another editor over `framework`, exactly as the canvas is. There is no bespoke two-way-sync state machine to build or keep consistent; the only thing E adds beyond "render `framework` / call `replaceFramework`" is the invalid-JSON rejection path.

#### E.3.6 Styles

`.builder-tab` / `.builder-tab--active` (the Canvas | JSON toggle — the active tab uses the same accent token as the existing app chrome), `.inspector-section` (a panel section), `.inspector__diff` (the disk-diff block — additions green, removals red, the live-graph status tokens), `.json-view` (the JSON textarea — monospace), `.json-view__error` (the inline parse error — the red `error` token). Per gotcha #67, every new class gets a corresponding rule in `src/styles.css` and the styles test asserts it via the `every_class_has_a_corresponding_CSS_rule` pattern. Theme variables only — do not reintroduce the M07-IRL #3 low-contrast bug Stage A fixed.

### E.4 Tests

#### E.4.1 `builderStore` unit tests — `tests/unit/lib/builderStore.replaceFramework.test.ts`

- `replaceFramework_swaps_the_framework_document`
- `replaceFramework_causes_the_canvasNodes_projection_to_re_derive` (ADR-0020 — the projection is pure)
- `setDiskFramework_records_the_snapshot_for_the_diff`
- `openTester_sets_the_tester_open_state` (inert-but-wired — assert the slot, not a modal)
- `the_inspector_diff_is_empty_when_framework_equals_diskFramework`
- `the_inspector_diff_is_non_empty_after_a_canvas_edit_following_a_save`

#### E.4.2 Inspector component tests — `tests/unit/components/builder/Inspector.test.tsx`

- `renders_the_live_framework_json_preview`
- `the_preview_updates_when_the_framework_changes`
- `renders_the_disk_diff_only_when_diskFramework_is_set`
- `renders_the_capability_summary_from_the_validate_framework_report` (the summary is a report field — no separate command)
- `clicking_validate_calls_validateFramework_and_surfaces_the_full_report`
- `clicking_test_calls_builderStore_openTester` (the Test button wiring — not a modal)
- `the_validate_button_uses_the_same_validate_framework_the_continuous_pass_uses` (spec §9 — one validator)

#### E.4.3 JsonView component tests — `tests/unit/components/builder/JsonView.test.tsx`

- `renders_the_current_framework_as_pretty_printed_json`
- `a_valid_json_edit_calls_replaceFramework_with_the_parsed_document`
- `an_invalid_json_edit_does_NOT_call_replaceFramework` (the no-desync guard)
- `an_invalid_json_edit_surfaces_an_inline_parse_error`
- `a_canvas_edit_re_seeds_the_json_draft_from_the_store` (Canvas → JSON)
- `recovering_from_invalid_to_valid_json_clears_the_parse_error_and_calls_replaceFramework`

#### E.4.4 Playwright behavior test — `tests/e2e/builder_inspector.spec.ts`

Drives the renderer against the Vite dev server with `@tauri-apps/api` + `@tauri-apps/plugin-dialog` module-mocked (gotcha #23 — Playwright cannot drive the Tauri window or the native file dialog; `save_framework` / `load_framework` / `validate_framework` are mocked).

```ts
test.describe.configure({ timeout: 90_000 }); // gotcha #53 (Vite cold-start)

test('Validate surfaces the full validation report', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click();   // C's view switch
  await page.getByRole('button', { name: 'Validate' }).click();
  await expect(page.locator('.inspector-section--capabilities')).toBeVisible();
});

test('a JSON-tab edit round-trips to the canvas', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click();
  await page.getByRole('tab', { name: 'JSON' }).click();
  // edit the JSON to add an agent, switch back to Canvas:
  await page.locator('.json-view textarea').fill(JSON_WITH_ONE_AGENT);
  await page.getByRole('tab', { name: 'Canvas' }).click();
  await expect(page.locator('[data-builder-node="agent"]')).toBeVisible();
});

test('invalid JSON surfaces a parse error and leaves the canvas unchanged', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click();
  await page.getByRole('tab', { name: 'JSON' }).click();
  await page.locator('.json-view textarea').fill('{ not valid json');
  await expect(page.locator('.json-view__error')).toBeVisible();
  await page.getByRole('tab', { name: 'Canvas' }).click();
  // the canvas reflects the last VALID framework — the bad edit did not desync.
});

test('Save calls save_framework with the picked directory', async ({ page }) => {
  // the @tauri-apps/plugin-dialog mock returns a fixed directory path.
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click();
  await page.getByRole('button', { name: /save|export/i }).click();
  // assert the mocked save_framework invoke received the picked dir.
});
```

#### E.4.5 Schema regen check

- N/A — Stage E ships no schema changes (it consumes Stage B's commands).

#### E.4.6 Acceptance criteria

- [ ] `npm run test` — all vitest tests pass; renderer ≥80 (vitest, `vitest.config.ts` gate) on `src/components/builder/Inspector.tsx` + `src/components/builder/JsonView.tsx` + the `builderStore` E additions
- [ ] `npx tsc --noEmit` — clean
- [ ] `npx eslint .` — clean
- [ ] `npx prettier --check '**/*.{ts,tsx,js,jsx,json}'` — clean
- [ ] `npm run test:e2e -- builder_inspector.spec.ts` — passes against the Vite dev server (curl warmup probe per the v1.6 `<playwright_warmup_recipe>`)
- [ ] MVP §M8 criterion 4 (Validate) + criterion 6 (Canvas | JSON two-way binding, including the invalid-JSON path) + criterion 7 (Save) + criterion 8 (Load round-trip) demonstrated in Playwright
- [ ] The **Test** button + `openTester` are shipped (inert-but-wired); the modal is Stage F2 — E's tests assert the `openTester` call, not a modal
- [ ] The capability summary reads the `validate_framework` report's `capability_summary` field — **no separate command** (Stage B B.3.4)
- [ ] The Validate button runs the **same** `validate_framework` D2's continuous pass uses (spec §9 — one validator, two triggers)
- [ ] An invalid JSON edit does **not** call `replaceFramework` — the store is not desynced (the `an_invalid_json_edit_does_NOT_call_replaceFramework` test confirms it)
- [ ] `<wire_signature_audit>` + `shape=` claims match the shipped Stage B `save_framework` / `load_framework` wire
- [ ] Every new CSS class has a corresponding rule in `src/styles.css` (gotcha #67 + the `every_class_has_a_corresponding_CSS_rule` pattern)
- [ ] Strict v1.7 two-commit TDD: `git diff <red>..<impl> -- '**/tests/**'` is EMPTY
- [ ] CI-parity per G6

### E.5 CLI Prompt

```xml
<work_stage_prompt id="M08.E">
  <context>
    M08 Stage E — the Builder Inspector + the Canvas|JSON two-way
    binding: a live framework.json preview, a disk diff (framework vs
    diskFramework), the whole-framework capability summary (read from
    the validate_framework report's capability-summary field — NOT a
    separate command), a Validate button (an explicit run of the SAME
    validate_framework D2's continuous pass uses — spec §9, no
    duplication), a Test button (sets builderStore.openTester — F2
    delivers the modal; inert-but-wired at E), Save/Load to disk via
    Stage B's save_framework/load_framework + Stage C's
    @tauri-apps/plugin-dialog picker, and the Canvas|JSON tab toggle
    (the JSON tab parses → replaceFramework; the canvas re-derives —
    ADR-0020; invalid JSON surfaces a parse error WITHOUT desyncing
    the store). Pure renderer over builderStore (C/D1/D2) + Stage B's
    commands. NO new schema, NO backend, NO new Tauri command. Pin the
    shipped Stage B save/load wire BEFORE any pseudocode.
  </context>
  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — wire_signature_audit + phase_doc_inventory_audit shape=)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Stage E E.1–E.4 + Stage B B.3.1/B.3.2/B.3.4/B.3.6 + Stages C/D1/D2 as predecessors)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543; the Inspector section — preview / diff / capability summary / Validate / Test / Export) + §9 (no TS/Rust validation duplication)</file>
    <file>docs/MVP-v0.1.md §M8 (criteria 4 / 5 / 6 / 7 / 8)</file>
    <file>docs/adr/0020-builder-canvas-state-model.md (framework.json as source of truth, the canvas + the JSON tab both editors over it — the two-way binding falls out of this)</file>
    <file>docs/build-prompts/retrospectives/M08.D2-retrospective.md + RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/gotchas.md (#23 Playwright cannot drive the Tauri window or the native file dialog, #27 vitest re-query after await, #30 unwrapCmdError, #53 Vite cold-start, #66 contract tests, #67 component+CSS contract, #75 useShallow for derived selectors)</file>
  </read_first>
  <read_reference>
    <file purpose="The SHIPPED Stage B save_framework / load_framework command signatures — PIN params verbatim before authoring the wrappers; validate_framework was pinned at D2">src-tauri/src/commands.rs</file>
    <file purpose="Stage B persist.rs — the save_framework / load_framework + LoadedFramework shape (the byte-stable round-trip Stage E's reload relies on)">crates/runtime-main/src/builder/persist.rs</file>
    <file purpose="Stage B summary.rs — confirm the whole-framework capability summary rides on the FrameworkValidationReport (a report field — Stage E reads it; NOT a separate command)">crates/runtime-main/src/builder/summary.rs</file>
    <file purpose="C/D1/D2 builderStore — E adds replaceFramework, setDiskFramework, openTester; reads the D2 validation slot; framework.json is the source of truth (ADR-0020)">src/lib/builderStore.ts</file>
    <file purpose="Stage C BuilderShell — E adds the Canvas|JSON tab toggle + mounts Inspector in the right region (the C stub)">src/components/builder/BuilderShell.tsx</file>
    <file purpose="D1/D2 BuilderCanvas — the Canvas tab content the JSON tab toggles against">src/components/builder/BuilderCanvas.tsx</file>
    <file purpose="Stage C ImportPanel — the @tauri-apps/plugin-dialog file-picker idiom Stage C registered; E reuses open({directory:true}) for save/load">src/components/ImportPanel.tsx</file>
    <file purpose="ipc.ts existing wrapper idiom — invoke + camelCase args + discriminated returns + unwrapCmdError; the importArtifact / mcpTestConnection wire_signature_audit precedent (ipc.ts:194,266 document prior drift)">src/lib/ipc.ts</file>
    <file purpose="styles.css class conventions + theme variables (--node-fg / --node-fg-muted) — no M07-IRL #3 contrast regression">src/styles.css</file>
  </read_reference>
  <read_prior_stages>
    <stage id="M08.D2" retro="docs/build-prompts/retrospectives/M08.D2-retrospective.md"/>
  </read_prior_stages>
  <deliverable ref="docs/build-prompts/M08-workbench.md" section="E.3 Detailed Changes"/>
  <test_plan_required>true</test_plan_required>
  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the E.4 buckets — vitest
      (replaceFramework + the canvas-projection re-derive; setDiskFramework
      + the diff; openTester as an inert slot; Inspector preview / diff /
      capability-summary / Validate / Test wiring; JsonView valid-edit →
      replaceFramework AND invalid-JSON → parse error WITHOUT
      replaceFramework) and Playwright (Validate; JSON edit round-trips
      to the canvas; invalid JSON surfaces a parse error + leaves the
      canvas unchanged; Save calls save_framework — Stage B's commands
      + the file picker mocked). Stub the production surfaces just
      enough that the test files compile (the Inspector / JsonView /
      replaceFramework / openTester / saveFramework / loadFramework
      symbols resolve). Confirm tests fail with right-reason errors per
      CLAUDE.md §5 (assertion failed / cannot find name / module not
      found — NOT a test-file compile error and NOT a tautological
      pass). Commit as a STANDALONE `test(M08.E): failing tests for the
      Inspector + canvas↔JSON binding` on the M08 parent-milestone
      branch BEFORE green-phase impl; the commit body pastes the first
      ~40 lines of the vitest/Playwright run proving the
      expected-failure class. Surface the red-phase commit; user
      approves before green phase begins.
    </red_phase>
    <green_phase>
      Implement the Inspector (four sections + Validate/Test buttons),
      JsonView (the parse-and-route logic + the invalid-JSON no-desync
      guard), the BuilderShell Canvas|JSON tab toggle, the builderStore
      replaceFramework/setDiskFramework/openTester additions, and the
      save/load file-picker plumbing until ALL failing tests pass. Do
      NOT modify the test files during implementation — if a test is
      wrong, fix it in a SEPARATE labelled follow-up commit with
      explanation, never silently in the impl commit. The impl commit
      body MUST state the verifiable invariant: `git diff
      &lt;red-sha&gt;..&lt;impl-sha&gt; -- '**/tests/**'` is EMPTY.
      Net-new additive tests + mechanical prettier/eslint fixes to test
      files go in the separate follow-up commit. No Co-Authored-By in
      any commit message.
    </green_phase>
  </tdd_discipline>
  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>
  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="E.4 Tests"/>
  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>
  <gates milestone="M08"/>
  <coverage_gate>
    <gate scope="renderer" target_lines="80" ignore_filename_regex="generated"/>
  </coverage_gate>
  <self_correction_budget>3</self_correction_budget>
  <adr_triggers>
    <trigger>None new in E — it implements against ADR-0020 (the Builder canvas↔framework.json state model, filed at Stage C); the Canvas|JSON two-way binding is a consequence of the source-of-truth model, not new architecture. E adds no schema, no backend, no IPC protocol change, no dependency — it consumes Stage B's save_framework/load_framework/validate_framework commands + Stage C's @tauri-apps/plugin-dialog. If a stage need surfaces (e.g. a structured-diff backend command rather than a TS diff), surface BEFORE the red phase; do not author it silently.</trigger>
  </adr_triggers>
  <gotchas>
    <trap>#23 — Playwright drives the Vite dev server with @tauri-apps/api AND @tauri-apps/plugin-dialog module-mocked; save_framework/load_framework/validate_framework are mocked. Playwright cannot drive the Tauri window or the native file dialog.</trap>
    <trap>Invalid JSON in the JSON tab must NOT call replaceFramework — surface an inline parse error, leave builderStore untouched (no canvas↔store desync). This is the load-bearing guard of the two-way binding.</trap>
    <trap>The Validate button runs the SAME validate_framework D2's continuous pass uses — one Rust validator, two triggers (spec §9 — no TS/Rust duplication). D2's pass is debounced; the Validate button is immediate.</trap>
    <trap>The whole-framework capability summary is the `capability_summary` FIELD on the validate_framework report (Stage B B.3.2 ships the field; B.3.4 defines FrameworkCapabilitySummary) — read it from the report; do NOT call a separate framework_capability_summary command from the renderer.</trap>
    <trap>The Test button is INERT until F2 — it ships with builderStore.openTester; F2 delivers the Tester modal that renders on that state. Incremental construction (the M07.E precedent), not dead code. E's tests assert the openTester call, not a modal.</trap>
    <trap>#75 — any derived builderStore selector (the diff inputs, the validation slice) is wrapped in useShallow (zustand/react/shallow).</trap>
    <trap>#67 — every new className (.builder-tab, .builder-tab--active, .inspector-section, .inspector__diff, .json-view, .json-view__error) gets a corresponding src/styles.css rule + the every_class_has_a_corresponding_CSS_rule static test.</trap>
    <trap>#30 — a save_framework/load_framework/validate_framework error thrown across the bridge is unwrapped via the existing unwrapCmdError helper, not String(e).</trap>
  </gotchas>
  <execution_warnings>
    <warning>DO NOT add a Tauri command in E — Stage B shipped save_framework/load_framework/validate_framework. E wires thin ipc.ts wrappers. If a new command appears necessary, surface in the retrospective; the default is "E consumes Stage B's wire".</warning>
    <warning>DO NOT build bespoke two-way-sync machinery — the Canvas|JSON binding falls out of ADR-0020. The JSON tab is just another editor over builderStore.framework, exactly as the canvas is. The only thing beyond "render framework / call replaceFramework" is the invalid-JSON rejection path.</warning>
    <warning>DO NOT call replaceFramework on invalid JSON — that desyncs the canvas from the store. Parse, and on failure surface a parse error and leave the store alone.</warning>
    <warning>DO NOT build the Tester modal in E — that is Stage F2. E ships the Test button + openTester only (inert-but-wired). Building the modal here conflates the E shell-completion scope with F2.</warning>
    <warning>DO NOT re-verify the save→load byte-stable round-trip in E — that is Stage B's tested contract (B.3.2 / B.4). E verifies the renderer wiring (picker → command → store → canvas re-derive).</warning>
    <warning>DO NOT edit the read-only GraphCanvas.tsx or the live-execution graphStore.ts — E acts only on the SEPARATE builderStore + the builder/ components.</warning>
  </execution_warnings>
  <pre_flight_check>
    <check name="branch">HEAD == the M08 parent-milestone branch; the M08.D2 impl commit is present (the BuilderShell + BuilderCanvas + the C/D1/D2 builderStore with the validation slot must be shipped)</check>
    <check name="stage_b_wire">grep-confirm the Stage B save_framework + load_framework commands in src-tauri/src/commands.rs are shipped before pinning the wrappers; grep-confirm the capability summary is a field on the FrameworkValidationReport (Stage B builder module) — not a separate command</check>
    <check name="stage_c_picker">grep-confirm @tauri-apps/plugin-dialog is registered (Stage C) — E reuses open({directory:true}) for save/load</check>
  </pre_flight_check>
  <phase_doc_inventory_audit>
    <claim type="file" path="src/components/builder/Inspector.tsx" verified="false" note="E creates"/>
    <claim type="file" path="src/components/builder/JsonView.tsx" verified="false" note="E creates"/>
    <claim type="file" path="src/components/builder/BuilderShell.tsx" verified="true" note="Stage C shipped; E adds the Canvas|JSON toggle + mounts Inspector"/>
    <claim type="ipc_command" path="src-tauri/src/commands.rs" symbol="save_framework" verified="true" note="Stage B shipped"/>
    <claim type="ipc_command" path="src-tauri/src/commands.rs" symbol="load_framework" verified="true" note="Stage B shipped"/>
    <claim type="store_slot" path="src/lib/builderStore.ts" symbol="diskFramework" shape="PIN the generated Framework type (nullable) from src/types/ — Stage C declared the slot; E sets/reads it" verified="true"/>
    <claim type="store_slot" path="src/lib/builderStore.ts" symbol="validation" shape="the D2-pinned FrameworkValidationReport — E reads its capability_summary field" verified="true"/>
  </phase_doc_inventory_audit>
  <wire_signature_audit>
    <wrapper ipc_command="save_framework" actual_params="PIN from the Stage B-shipped src-tauri/src/commands.rs signature (confirm the dir + framework param names, e.g. { dir, fw })" phase_doc_assumed="{ dir, fw } — author reconciles against the shipped signature"/>
    <wrapper ipc_command="load_framework" actual_params="PIN from the Stage B-shipped signature (confirm the param name + the LoadedFramework return shape)" phase_doc_assumed="{ dir } → LoadedFramework — author reconciles against the shipped signature"/>
  </wire_signature_audit>
  <runtime_environment os="windows" note="Vitest + Playwright run against the Vite dev server; Vite cold-start mitigated per gotcha #53; the native file dialog is module-mocked"/>
  <time_box hours="9-12"/>
  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the pinned save_framework / load_framework params + the LoadedFramework shape (from the shipped Stage B code — schema-generated or hand-mirrored?). Confirm the Canvas|JSON two-way binding falls out of ADR-0020 (the JSON tab is an editor over builderStore.framework — no bespoke sync machinery). Confirm the Validate button shares the D2 validate_framework validator (one validator, two triggers). Confirm the capability summary is read from the report field, not a separate command. Confirm the invalid-JSON path does NOT call replaceFramework (no desync). Confirm the Test button is inert-but-wired (openTester shipped; the modal is F2). MVP §M8 criteria 4/6/7/8 Playwright results.</special_log>
  </retrospective_requirements>
  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="E.6 Commit Message"/>
  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M08.*-retrospective.md); strict-TDD '**/tests/**' EMPTY proof</item>
    <item>wire_signature_audit + phase_doc_inventory_audit shape= reconciliation against the shipped Stage B save_framework / load_framework wire</item>
    <item>the Inspector four sections + Validate/Test buttons; the capability summary read from the report field (no separate command)</item>
    <item>the Canvas|JSON two-way binding; the invalid-JSON no-desync guard confirmed; the Test button inert-but-wired</item>
    <item>gate results (v1.6 canonical order; renderer ≥80 vitest; Playwright + warmup; every-class-has-CSS-rule; CI-parity per G6)</item>
    <item>MVP §M8 criteria 4/6/7/8 Playwright demonstration</item>
    <item>M08.E retrospective [END]; explicit "Stage M08.E is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### E.6 Commit Message

```
feat(renderer): M08 Stage E — Inspector + canvas↔JSON two-way binding

The Builder Inspector: a live framework.json preview, a disk diff
(framework vs diskFramework), the whole-framework capability summary
read from the validate_framework report's capability-summary field
(a report field — not a separate command), an explicit Validate
button (the same validate_framework D2's continuous pass uses — spec
§9, one validator, two triggers), and a Test button (sets
builderStore.openTester; inert-but-wired — F2 delivers the modal).

Save/Load to disk via Stage B's save_framework/load_framework + the
Stage C @tauri-apps/plugin-dialog directory picker: Save writes
framework.json + companion skill.md/tool.md/agent.md and records
diskFramework so the diff zeroes; Load reads a directory, calls
replaceFramework, and the canvas re-derives.

The Canvas|JSON tab toggle — the JSON tab is another editor over
builderStore.framework, exactly as the canvas is (ADR-0020). A valid
JSON edit calls replaceFramework and the canvas re-derives; invalid
JSON surfaces an inline parse error and leaves the store untouched
(no canvas↔store desync). The two-way binding falls out of the
source-of-truth model — no bespoke sync machinery.

builderStore additions: replaceFramework (JSON-tab edit + load),
setDiskFramework (the disk-diff snapshot), openTester (the Test
button's target). MVP §M8 criteria 4 + 6 + 7 + 8. No new schema, no
backend, no new Tauri command — E consumes Stage B's wire.

v1.8 wire_signature_audit + phase_doc_inventory_audit shape= pinned
to the shipped Stage B save/load wire. Playwright + vitest; renderer
≥80. Strict two-commit TDD: '**/tests/**' diff EMPTY.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---
## Stage F1 — Tester backend: isolated session + the M07.V Dec-6 discharge

### F1.1 Problem Statement

Spec Phase 9 Tester: *"Load the framework from canvas (does NOT need to save first); define a test task; run in an isolated session with a separate SQLite database; capability violations during test surfaced as test failures, not as live HITL prompts (test sessions don't block on user input — defaults applied); test runs do not write to any user data directory; results discarded on close unless explicitly saved."* MVP §M8 criterion 5: click Test → enter a task → a sandboxed session runs → graph + VDR + token spend + pass/fail surface.

Stage F1 builds the **Tester backend** — `test_framework`: an isolated, throwaway test session. It is also the milestone's **carry-forward discharge stage**: the Tester is the production tool-driving session that closes the coupled M07.V Dec-6 set Stage A mapped — 🟡 #5 (the agent-with-tools production driver), 🟡 #2 (`skills_lock::verify` on the artifact-load path), 🟡 #3 (`McpDispatcher::on_server_connected` connect handler). F1 inverts Stage A's `<construction_reachability_check>` from `inputs_reachable="false"` to `"true"`. **ADR-0019** records the Tester isolated-session model.

The single most important design constraint: F1 does **not** build a new session engine. It reuses the existing smoke-session infrastructure — `run_smoke_session_with` (`src-tauri/src/commands.rs:194`), `DroneLifecycle` (`src-tauri/src/drone_lifecycle.rs`), and `AgentSdk::run_agent` (`crates/runtime-main/src/sdk/agent_sdk.rs:256`, the M07.D2 multi-turn loop) — pointed at a throwaway SQLite path and configured for test mode. The *only* novel surfaces are (a) a throwaway-DB resolution + teardown wrapper, (b) a test-mode `HitlSeam` that auto-applies defaults instead of prompting, and (c) the three M07.V Dec-6 production-caller wires that the Tester's tool-bearing run naturally exercises. F1 is CODEOWNERS-flagged (the sandbox/isolation boundary + capability-enforcement *behavior*: violations become test failures rather than HITL prompts) — the `<construction_reachability_check>` closure + this stage doc are the plan-first surface.

Concrete deliverables:

1. **`crates/runtime-main/src/builder/tester.rs`** — the Tester module. `run_test_session_with(framework: &Framework, task: &str, db_path: &Path, /* injected drone/provider/dispatch seams */) -> Result<TestOutcome, TesterError>` — builds a session over a throwaway SQLite path, runs the candidate framework + the task through `AgentSdk::run_agent`, collects the outcome. The `*_with` seam follows the CLAUDE.md §5 archetype; the production `run_test_session` wrapper constructs the real drone + provider.
2. **`TestOutcome` struct** — `passed: bool`, `capability_failures: Vec<CapabilityFailure>` (§8.security L2 violations collected as failures, **not** raised as HITL), `token_spend`, `timing`, `vdr`, `trace`. Crosses the Tauri wire to F2.
3. **`TesterError` enum** — `thiserror`-based; `From` conversions from `SdkError`, `LockError`, `CmdError`, `io::Error`. The hash-mismatch and capability-violation paths are *not* `TesterError` variants — a tampered artifact / a capability violation is a **failed test** (`TestOutcome { passed: false, .. }`), not a Rust error; `TesterError` is reserved for infrastructure failure (drone spawn failed, temp-file IO failed).
4. **A test-mode `HitlSeam`** — `HitlSeam::test_defaults()` (or equivalent constructor) that auto-resolves every decision with the default instead of blocking on user input. F1.3.3 + ADR-0019.
5. **`test_framework` Tauri command** (`src-tauri/src/commands.rs`) — `(framework_doc: Framework, task: String) -> Result<TestOutcome, CmdError>`; resolves a throwaway temp DB path via the Tauri shell, calls `run_test_session_with`, returns the outcome. Wired into the `invoke_handler`.
6. **The `skills_lock::verify` wire** — when the test session byte-loads an imported skill/tool/agent for execution, the load path calls `skills_lock::verify`; `LockError::HashMismatch` → `AgentEvent::ArtifactHashMismatch` + refuses the load (the test fails with a clear reason). Discharges M07.V 🟡 #2.
7. **The `McpDispatcher::on_server_connected` wire** — the test session's MCP connect handler calls `on_server_connected` / `on_server_disconnected`; each returned `NewAmbiguity` → `AgentEvent::ToolAliasAmbiguous` (spec §5a step 5). Discharges M07.V 🟡 #3.
8. **The agent-with-tools production driver** — the Tester runs a tool-bearing framework through `AgentSdk::run_agent`, so a real `ProviderEvent::ToolUse` is dispatched through the concrete `McpDispatcher` in a **production** code path for the first time. Discharges M07.V 🟡 #5.
9. **`docs/adr/0019-tester-isolated-session-model.md`** — ADR-0019: the throwaway-DB model, test-defaults for capability violations, discard-on-close, and the §1c-vs-v0.1-scope reconciliation.
10. **The assembled-app regression** — `crates/runtime-main/tests/tester_isolated_session.rs`: a real drone subprocess + a real `run_test_session_with` against a `tempfile` DB + a tool-bearing framework. Asserts the session runs, signals persist under the test session id, `token_usage > 0`, and the throwaway DB is isolated from the user session DB.
11. **≥95 per-crate coverage on the `builder::tester` module** within `runtime-main`; any OS-call-holdout exclusion added syncs all four coverage mirrors in this stage's commit.

Not in this stage:
- The Tester modal renderer (Stage F2 — task input, the smaller graph pane, the result surfaces, discard-on-close).
- The explicit "Save results / Promote to main session" affordance (F2 — the renderer surface; F1 only guarantees that *nothing* is persisted to a user data directory unless a caller explicitly does so).
- Any new schema (F1 reuses `AgentEvent::ArtifactHashMismatch` + `AgentEvent::ToolAliasAmbiguous` — both shipped at M07).
- Any change to capability-enforcement *logic* (F1 changes only the *response* to a violation, scoped to the test session — Hard Rule 8).

### F1.2 Files to Change

| File | Status | Change |
|---|---|---|
| `crates/runtime-main/src/builder/tester.rs` | new | The isolated test session: `run_test_session_with` + `run_test_session` + `TestOutcome` + `TesterError` + `CapabilityFailure`. Builds a session over a throwaway SQLite path; runs the candidate framework + task through `AgentSdk::run_agent`; collects pass/fail + trace + VDR + token/timing. |
| `crates/runtime-main/src/builder/mod.rs` | exists (B) | `pub mod tester;` + re-export `run_test_session` / `TestOutcome` / `TesterError`. |
| `crates/runtime-main/src/hitl/seam.rs` | exists | Add a test-defaults constructor (`HitlSeam::test_defaults()`) — auto-resolves every decision with the default; the session never blocks on user input. |
| `crates/runtime-main/src/builder/` (artifact-load path) | exists (B) | Call `skills_lock::verify` when the test session byte-loads an imported skill/tool/agent (M07.V 🟡 #2); `HashMismatch` → `ArtifactHashMismatch` → refuse. |
| `src-tauri/src/commands.rs` | exists | `test_framework` command (framework doc + task string → `TestOutcome`); reuses `run_smoke_session_with`'s construction with a throwaway DB path. |
| `src-tauri/src/drone_lifecycle.rs` | exists | The test session spawns a drone via `DroneLifecycle::spawn(throwaway_db_path)` — `spawn` already takes `db_path: PathBuf`; no signature change, just a throwaway path resolved by the shell. |
| `src-tauri/src/main.rs` | exists | Register `test_framework` in the `invoke_handler`; resolve the throwaway DB temp path. |
| `docs/adr/0019-tester-isolated-session-model.md` | new | ADR-0019 — the Tester isolation model. |
| `crates/runtime-main/tests/tester_isolated_session.rs` | new | The assembled regression — real drone subprocess + real loop + tool-bearing framework. |
| `CHANGELOG.md` | exists | `[Unreleased]` Stage F1 entries. |
| `docs/build-prompts/retrospectives/M08.F1-retrospective.md` | new | Stage F1 retrospective. |

Effort budget: ~9–12 hours. The largest piece is the `tester.rs` module + the assembled regression; the three Dec-6 wires are *small* (each is a single call site placed on a path the reused infra already walks) — their cost is correctness verification, not volume. The `test_framework` command is a thin reuse of `run_smoke_session_with`'s construction.

### F1.3 Detailed Changes

#### F1.3.1 The Tester module + `test_framework` — reuse the smoke-session infra

The Tester does **not** build a new session engine. `run_smoke_session_with` (`src-tauri/src/commands.rs:194`) already encapsulates the entire construction — `AgentSdk::new(provider, event_tx, drone, session_id)` → optional `.with_mcp_dispatch(..)` → `.run_agent(config)`. The Tester reuses that exact construction, differing in only three respects: the drone is spawned against a **throwaway** DB path, the `AgentConfig` carries the **candidate framework** (not the hardcoded smoke prompt), and the `HitlSeam` is the **test-defaults** variant. `test_framework` takes the candidate framework document (from `builderStore.framework` — "load from canvas without saving", so the doc crosses the wire directly; no disk round-trip) + the user's task string, and returns a `TestOutcome` the modal (F2) renders.

```rust
// crates/runtime-main/src/builder/tester.rs — the Tester backend.
// Reuses the smoke-session construction (run_smoke_session_with);
// it does NOT rebuild a session engine.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use runtime_core::event::AgentEvent;
use runtime_core::generated::framework::Framework;

use crate::drone_ipc::DroneClient;
use crate::providers::{AgentConfig, LLMProvider};
use crate::sdk::{AgentSdk, McpToolDispatch, SessionId};
use crate::skills_lock::LockError;

/// The result of one Tester run. Crosses the Tauri wire to F2 (the
/// modal renders every field). `passed == false` covers BOTH a model
/// that produced a wrong answer AND a session that hit a capability
/// violation / hash mismatch — those are *failed tests*, not
/// `TesterError`s (F1.3.3 / F1.3.5).
pub struct TestOutcome {
    /// Whether the test session completed without a capability failure
    /// or an integrity block. A clean run with a tool-using agent that
    /// converged is `true`; any `capability_failures` entry forces
    /// `false`.
    pub passed: bool,
    /// §8.security L2 capability violations collected during the run.
    /// Non-empty ⇒ `passed == false`. Surfaced by F2 as test failures,
    /// NEVER raised as a live HITL/gap prompt (F1.3.3).
    pub capability_failures: Vec<CapabilityFailure>,
    /// Token spend for the run (in / out / total) — F2 renders it
    /// against the typical-session benchmark.
    pub token_spend: TokenSpend,
    /// Wall-clock duration of the test session.
    pub timing: Duration,
    /// The VDR (Verification & Decision Record) the test session
    /// produced — reuses the existing VDR shape.
    pub vdr: serde_json::Value,
    /// The full ordered `AgentEvent` trace, so F2 can render the
    /// smaller graph pane + the pass/fail trace from one payload.
    pub trace: Vec<AgentEvent>,
}

/// One §8.security L2 capability violation observed in the test
/// session. Collected onto `TestOutcome`, not raised as HITL.
pub struct CapabilityFailure {
    /// The runtime agent id that attempted the denied action.
    pub agent_id: String,
    /// The capability that was missing/denied (human-readable).
    pub needed: String,
    /// The enforcer's reason string.
    pub reason: String,
}

/// Token in/out/total for a test run.
pub struct TokenSpend {
    pub input: u64,
    pub output: u64,
    pub total: u64,
}

/// Infrastructure failure of the Tester itself — NOT a failed test.
/// A capability violation / hash mismatch is `TestOutcome { passed:
/// false, .. }`; `TesterError` is reserved for "the test could not be
/// run at all" (drone spawn failed, the throwaway temp file could not
/// be created, the provider stream open failed).
#[derive(Debug, thiserror::Error)]
pub enum TesterError {
    /// The throwaway test database could not be created / resolved.
    #[error("test database setup failed: {0}")]
    DbSetup(#[from] std::io::Error),
    /// The drone subprocess for the test session failed to spawn or
    /// connect.
    #[error("test-session drone failed: {0}")]
    Drone(String),
    /// The agent-with-tools loop surfaced an infrastructure error
    /// (provider stream open failed, channel closed). A *capability*
    /// failure is a failed test, not this.
    #[error("test session run failed: {0}")]
    Run(String),
}

/// Test-seam: run an isolated test session against caller-supplied
/// drone / provider / dispatch collaborators over a caller-supplied
/// throwaway `db_path`.
///
/// `db_path` MUST be a throwaway temp-file path — NEVER the user
/// session DB (ADR-0019; the production wrapper + the Tauri command
/// guarantee this). Unit tests pass a `tempfile`-backed path.
///
/// Reuses the smoke-session construction
/// (`AgentSdk::new` → optional `with_mcp_dispatch` → `run_agent`); it
/// does not rebuild a session engine. The `HitlSeam` woven into the
/// session is the test-defaults variant (F1.3.3) so the run never
/// blocks on user input.
///
/// # Errors
///
/// [`TesterError`] for infrastructure failure only. A capability
/// violation / a hash mismatch produces `Ok(TestOutcome { passed:
/// false, .. })`, NOT an `Err`.
pub async fn run_test_session_with<P: LLMProvider + 'static>(
    framework: &Framework,
    task: &str,
    db_path: &Path,
    provider: P,
    drone: Arc<DroneClient>,
    mcp_dispatch: Option<Arc<dyn McpToolDispatch>>,
    session_id: SessionId,
) -> Result<TestOutcome, TesterError> {
    let started = Instant::now();
    // 1. Build the AgentConfig from the candidate framework + the task
    //    (the framework crossed the wire from the canvas — no disk save).
    // 2. Construct the SDK exactly as run_smoke_session_with does:
    //    AgentSdk::new(provider, event_tx, drone, session_id), then
    //    .with_mcp_dispatch(dispatch) when Some.
    // 3. The session is woven with HitlSeam::test_defaults() (F1.3.3) —
    //    capability violations DO NOT raise a live prompt.
    // 4. Drive AgentSdk::run_agent — the M07.D2 multi-turn loop. Because
    //    the candidate framework declares tools, this dispatches a real
    //    ProviderEvent::ToolUse through the concrete McpDispatcher in a
    //    production path for the first time (M07.V 🟡 #5 — F1.3.4).
    // 5. Collect the AgentEvent trace; fold CapabilityViolation events
    //    into `capability_failures`; sum TokenUsage into `token_spend`.
    // 6. passed = capability_failures.is_empty() && no ArtifactHashMismatch.
    let _ = (framework, task, db_path, provider, drone, mcp_dispatch, session_id);
    Ok(TestOutcome { /* … folded from the trace … */
        passed: true,
        capability_failures: Vec::new(),
        token_spend: TokenSpend { input: 0, output: 0, total: 0 },
        timing: started.elapsed(),
        vdr: serde_json::Value::Null,
        trace: Vec::new(),
    })
}
```

The seam keeps the OS-touching pieces (drone spawn, the throwaway temp-file IO) on the *production* side so the unit tests inject fakes — the `import/fetch.rs` seam-vs-wrapper precedent. The `test_framework` Tauri command is a thin production wrapper, mirroring `run_smoke_session`'s relationship to `run_smoke_session_with`:

```rust
// src-tauri/src/commands.rs — the production wrapper. Mirrors
// run_smoke_session (commands.rs:120): resolve key + drone, construct
// the real provider, delegate to the *_with seam.
//
// The candidate framework crosses the wire directly from
// builderStore.framework (spec Phase 9 "load from canvas — does NOT
// need to save first"); `task` is the user's natural-language task.

/// Run the Builder's Tester against a candidate framework.
///
/// Spawns an ISOLATED test session — a throwaway SQLite DB resolved
/// here in the shell (never the user session DB; ADR-0019), a
/// test-defaults `HitlSeam` (capability violations → test failures,
/// not live HITL), no user-data-dir writes — and runs `framework_doc`
/// + `task` through `AgentSdk::run_agent`. The throwaway DB is deleted
/// on teardown (F1.3.2).
///
/// # Errors
///
/// - [`CmdError::SetupRequired`] if no API key is in the keychain.
/// - [`CmdError::Internal`] for a `TesterError` (drone spawn / temp-DB
///   setup failure). A *failed test* is `Ok(TestOutcome { passed:
///   false, .. })`, not an `Err`.
#[tauri::command]
pub async fn test_framework(
    app: AppHandle,
    framework_doc: Framework,
    task: String,
) -> Result<TestOutcome, CmdError> {
    let api_key = read_api_key()?;
    let provider = AnthropicProvider::new(api_key.clone());
    // Throwaway temp DB — resolved in the shell, NEVER the user session
    // DB. std::env::temp_dir() join a per-run UUID; deleted on teardown.
    let db_path = std::env::temp_dir()
        .join(format!("runtime-tester-{}.sqlite", Uuid::new_v4()));
    // Spawn a drone against the throwaway path (drone_lifecycle::spawn
    // already takes db_path: PathBuf — no signature change).
    let lifecycle = DroneLifecycle::spawn(db_path.clone())
        .await
        .map_err(|e| CmdError::internal(e.to_string()))?;
    let session_id = lifecycle.sdk_session_id();
    let outcome = builder::tester::run_test_session_with(
        &framework_doc, &task, &db_path, provider,
        Arc::clone(&lifecycle.client), /* mcp_dispatch */ None, session_id,
    )
    .await;
    // Teardown: shut the drone down, delete the throwaway DB (F1.3.2).
    let _ = lifecycle.shutdown().await;
    let _ = std::fs::remove_file(&db_path);
    drop(api_key);
    outcome.map_err(|e| CmdError::internal(e.to_string()))
}
```

#### F1.3.2 Isolated session: throwaway SQLite, no user-data writes, teardown (ADR-0019)

Per spec Phase 9 + the Stage A intake (A.3.7 — confirmed: a sequential, throwaway, build-time test session, **not** §1c multi-session), the test session runs against a **throwaway SQLite database** and writes **nothing to any user data directory**. The throwaway path is `std::env::temp_dir().join("runtime-tester-<uuid>.sqlite")` — resolved in the Tauri shell, never `AppHandle::path().app_local_data_dir()`. `DroneLifecycle::spawn(db_path: PathBuf)` already accepts an arbitrary path (`drone_lifecycle.rs:87`); the Tester passes the throwaway path and the drone seeds *its* `sessions` row in *that* DB. The session is **torn down on close**: `DroneLifecycle::shutdown()` reaps the drone subprocess, then `std::fs::remove_file(&db_path)` deletes the temp DB. Results are discarded unless the user explicitly saves them — the explicit-save surface is F2's modal; F1's contract is simply that the backend persists nothing to a user directory.

**ADR-0019** records this isolation model: the throwaway DB, the test-defaults (F1.3.3), discard-on-close, and the explicit reconciliation that the v0.1 Tester is *not* §1c multi-session — it never runs concurrently with a live runtime session (it is invoked from build mode, where no live session is executing), so it needs *an* isolated session, not the §1c concurrent-session pool. The drone-spawn + the temp-file IO are OS-call holdouts: the `run_test_session_with` *seam* is unit-tested with injected fakes; the *production wrapper* + the teardown path are exercised by the F1.3.8 assembled regression. If F1 adds an `--ignore-filename-regex` exclusion for `src.builder.tester_io\.rs` (or equivalent), the v1.8 four-mirror sync (CLAUDE.md §5/§6 + `docs/coverage-policy.md` §A/§C + `codecov.yml`) lands in this stage's commit.

#### F1.3.3 Test-defaults for capability violations (no HITL)

Spec Phase 9: *"Capability violations during test (§8.security L2) surfaced as test failures, not as live HITL prompts (test sessions don't block on user input — defaults applied)."* The test session is woven with a **test-mode `HitlSeam`** that auto-applies the default for every decision instead of prompting — the session never blocks on user input. §8.security L2 capability violations are **collected as `CapabilityFailure` entries on `TestOutcome`** rather than raised as live HITL / gap prompts.

```rust
// crates/runtime-main/src/hitl/seam.rs — a test-defaults constructor.
// The live HitlSeam awaits a user decision via an in-process channel
// (ADR-0007). The test variant resolves every prompt immediately with
// the default so a Tester run never blocks on user input.

impl HitlSeam {
    /// A `HitlSeam` for the Builder's Tester (ADR-0019). Every
    /// `await_decision` / `on_capability_violation` call resolves
    /// IMMEDIATELY with the prompt's default choice — the test session
    /// runs unattended (spec Phase 9 "test sessions don't block on
    /// user input — defaults applied").
    ///
    /// This changes the capability-enforcement *response* (a violation
    /// → the default + a `CapabilityViolation` event the Tester folds
    /// into `capability_failures`), NOT the enforcement *logic*
    /// (Hard Rule 8 — the `CapabilityEnforcer::check` path is byte-
    /// identical to a live session).
    #[must_use]
    pub fn test_defaults() -> Self {
        // Construct with an auto-responder that picks the default choice
        // for every HitlPrompt instead of awaiting a renderer response.
        /* … */
        todo!()
    }
}
```

The enforcement *logic* is unchanged (reuse — Hard Rule 8): `CapabilityEnforcer::check` runs exactly as it does in a live session. Only the *response to a violation* differs — fail-the-test (collect a `CapabilityFailure`) vs prompt-the-user (raise a HITL/gap modal). This capability-enforcement *behavior variation* scoped to the test session is the §11 trigger recorded in ADR-0019; the `<construction_reachability_check>` closure + this stage doc are the Hard-Rule-8 plan-first surface.

#### F1.3.4 Discharge M07.V 🟡 #5 — the agent-with-tools production driver

`AgentSdk::run_agent` (`agent_sdk.rs:256`, M07.D2) is the multi-turn agent-with-tools loop — for each turn, `provider.stream(config)` → `drive_stream` → if any tool dispatched, feed the `tool_result`s back and re-stream, up to `MAX_AGENT_TURNS`. On `main`, its only **production** caller is `run_smoke_session_with`, which builds the no-tools `smoke_config()` — so a real `ProviderEvent::ToolUse` is dispatched through the concrete `McpDispatcher` only inside the `agent_with_tools_loop.rs` *integration test*, never in production. The Tester runs the **candidate framework** — which can declare tools / MCP servers — through `run_agent`, so a real `ProviderEvent::ToolUse` flows through the concrete `McpDispatcher` in a **production** code path for the first time. M07.V 🟡 #5 is discharged: the Tester *is* the production tool-driving session ADR-0011 (d) named M08 as the home for. F1's `<construction_reachability_check>` records `builder::tester::run_test_session_with → AgentSdk::run_agent` as the now-reachable constructor, with the concrete file:line.

#### F1.3.5 Discharge M07.V 🟡 #2 — `skills_lock::verify` on the artifact-load path

Spec §2214: *"Runtime validates `content_hash` on every load."* `skills_lock::verify(path, artifact_ref, artifact_bytes)` (`skills_lock/mod.rs:107`) recomputes the SRI hash and returns `LockError::HashMismatch` on drift — but on `main` it has only test callers. When the test session byte-loads an imported skill/tool/agent for execution, the load path calls `verify`:

```rust
// crates/runtime-main/src/builder/ — the test session's artifact-load
// path. When the Tester byte-loads an imported artifact for execution,
// it verifies the bytes against skills.lock BEFORE running them.

/// Load an imported artifact's bytes for execution in the test
/// session, verifying integrity against `skills.lock` first.
///
/// `verify` (skills_lock::verify) recomputes the SRI content hash and
/// HARD-BLOCKS on drift (integrity > availability; ADR-0014). A
/// `HashMismatch` maps to `AgentEvent::ArtifactHashMismatch` and the
/// load is REFUSED — the test fails with a clear hash-mismatch reason.
/// This is the FIRST production load-path caller of `verify`
/// (M07.V 🟡 #2 — discharged here).
fn load_verified_artifact(
    lock_path: &Path,
    artifact_ref: &str,    // the `name@version`
    bytes: &[u8],
    emit: &impl Fn(AgentEvent),
) -> Result<Vec<u8>, LockError> {
    match crate::skills_lock::verify(lock_path, artifact_ref, bytes) {
        Ok(()) => Ok(bytes.to_vec()),
        Err(LockError::HashMismatch { artifact_ref, expected, actual }) => {
            // Spec §2214 — refuse the drifted bytes; surface the
            // schema-faithful event the renderer maps to a
            // Reinstall / Remove prompt (M07 Stage E).
            emit(AgentEvent::ArtifactHashMismatch { artifact_ref: artifact_ref.clone(), expected, actual });
            Err(LockError::HashMismatch { artifact_ref, expected, actual: String::new() })
        }
        Err(other) => Err(other),
    }
}
```

A `HashMismatch` refuses the load (the test fails with a clear hash-mismatch reason). M07.V 🟡 #2 is discharged — `skills_lock::verify` gets its first production load-path caller. (Stage B already shipped the `skills.lock` *read* path; F1 ships the *verify-on-load* path — they are distinct: B reads the ledger, F1 checks bytes against it at execution time.)

#### F1.3.6 Discharge M07.V 🟡 #3 — `McpDispatcher::on_server_connected` connect handler

When the test session connects the framework's MCP servers, the connect handler calls `McpDispatcher::on_server_connected` (`dispatch.rs:141`) and `on_server_disconnected` (`dispatch.rs:155`) on teardown. `on_server_connected` snapshots the connected server's tool set into the `NamespaceResolver` and returns the short names that **became ambiguous**; each `NewAmbiguity` is translated to `AgentEvent::ToolAliasAmbiguous` per spec §5a step 5:

```rust
// the test session's MCP connect handler — the FIRST production caller
// of McpDispatcher::on_server_connected (M07.V 🟡 #3 — discharged).

/// Connect the candidate framework's MCP servers for the test
/// session, driving §5a re-resolution.
///
/// `on_server_connected` re-snapshots the resolver and returns the
/// short names that became newly ambiguous across the connected set;
/// each becomes an `AgentEvent::ToolAliasAmbiguous` (spec §5a step 5).
/// The trace is on `McpDispatcher` (NOT `McpClient`) — the M06.V Dec-6
/// `<wire_trace_vs_adr_reconcile>` #6, already honored in M07.D1.
async fn connect_test_session_mcp(
    dispatcher: &McpDispatcher,
    servers: &[String],
    emit: &impl Fn(AgentEvent),
) -> Result<(), McpError> {
    for server in servers {
        for ambiguity in dispatcher.on_server_connected(server).await? {
            emit(AgentEvent::ToolAliasAmbiguous {
                short_name: ambiguity.short_name,
                candidates: ambiguity.candidates,
            });
        }
    }
    Ok(())
}
// teardown calls dispatcher.on_server_disconnected(server) per server.
```

M07.V 🟡 #3 is discharged — the §5a re-resolution driver gets its first production connect-handler caller. (Pin the exact `NewAmbiguity` field names — `short_name` / `candidates` — and the `ToolAliasAmbiguous` variant fields from `crates/runtime-mcp/src/namespace.rs` + `crates/runtime-core/src/generated/event.rs:1926` via the `<phase_doc_inventory_audit>` at authoring time.)

#### F1.3.7 Construction-reachability closure (inverts Stage A's map)

Stage A authored the `<construction_reachability_check>` with all three wires `inputs_reachable="false"` (no production caller on `main`). F1 inverts each to `inputs_reachable="true"` with the concrete file:line of the production caller F1.3.4/5/6 placed — the A→F1 construction graph completing, exactly the M07 A→D1→D2 pattern for ADR-0011. The F1.5 prompt carries the inverted `<construction_reachability_check>`; Stage V's Inventory pass verifies each `true` claim's file:line resolves to a real call site.

#### F1.3.8 The assembled-app regression + coverage

Per the §6 / v1.8 assembled-app-regression mandate, F1 ships an assembled regression — `crates/runtime-main/tests/tester_isolated_session.rs` — that drives the **real Tester path**: a real `runtime-drone` subprocess, a real `run_test_session_with` against a `tempfile` DB, a tool-bearing framework. The phase-doc root cause — "the Tester is the production tool-driving session; running it persists signals + non-zero `token_usage` under the test session id, isolated from the user DB" — is a **falsifiable hypothesis this test must disprove**, not a premise. It mirrors the `smoke_signal_persistence.rs` + `agent_with_tools_loop.rs` archetype (the `ensure_drone_built` / `spawn_drone` / `connect_with_retry` / `poll_until` harness that already runs cross-platform in CI) and uses a **concrete** `McpDispatcher` (MockTransport-scripted, real `CapabilityEnforcer` + `NamespaceResolver`) — *not* a mock `McpToolDispatch` seam — so it kills the Stage-V blind spot (gotcha #66). It asserts: the test session runs to completion, signals persist under the *test* session id, `token_usage > 0`, and the throwaway DB is a **distinct file** from any user session DB (open both, assert the user DB is untouched).

The `builder::tester` module is in `runtime-main` → the **≥95 gate**. The drone-spawn + the throwaway-DB temp-file IO are OS-call holdouts seam-tested with injected fakes (the `import/fetch.rs` precedent); the `run_test_session_with` seam is fully unit-testable. If F1 places the OS-touching production wrapper in its own file and adds an `--ignore-filename-regex` exclusion, the v1.8 four-mirror sync lands in this stage's commit and the closeout `<coverage_policy_reconciliation>` verifies it.

### F1.4 Tests

#### F1.4.1 Tester module unit tests — `crates/runtime-main/src/builder/tester.rs` `#[cfg(test)]`

The `run_test_session_with` seam is exercised with an in-memory provider stub + `DroneClient::noop()` + a `tempfile`-backed `db_path`:

- `run_test_session_with_returns_test_outcome_for_a_clean_run`
- `run_test_session_with_folds_capability_violation_into_capability_failures`
- `run_test_session_with_capability_violation_sets_passed_false`
- `run_test_session_with_does_not_raise_hitl_on_capability_violation` — the test-defaults `HitlSeam` auto-resolves; no prompt is raised
- `run_test_session_with_sums_token_usage_into_token_spend`
- `test_outcome_passed_true_requires_empty_capability_failures_and_no_hash_mismatch`
- `tester_error_is_infrastructure_only_not_a_failed_test` — a capability violation is `Ok(TestOutcome { passed: false })`, not `Err(TesterError)`
- `run_test_session_with_twice_in_sequence_with_distinct_db_paths_both_succeed` (multi-call; gotcha #69)

#### F1.4.2 Test-defaults `HitlSeam` tests — `crates/runtime-main/src/hitl/seam.rs` `#[cfg(test)]`

- `test_defaults_seam_resolves_await_decision_with_the_default_immediately`
- `test_defaults_seam_does_not_block_when_no_renderer_is_attached`
- `test_defaults_seam_on_capability_violation_applies_the_default`

#### F1.4.3 Throwaway-DB isolation + teardown tests

- `throwaway_db_path_is_distinct_from_a_user_session_db_path`
- `test_session_teardown_deletes_the_throwaway_db`
- `test_session_writes_nothing_to_a_user_data_directory` — run against a `tempfile` dir; assert no file lands outside the throwaway path

#### F1.4.4 The `skills_lock::verify` load-path discharge (M07.V 🟡 #2)

- `load_verified_artifact_passes_for_matching_bytes`
- `load_verified_artifact_tampered_bytes_emit_artifact_hash_mismatch`
- `load_verified_artifact_hash_mismatch_refuses_the_load` — the load returns `Err`, the artifact does not run
- `test_session_with_a_tampered_artifact_fails_the_test_with_a_hash_mismatch_reason`

#### F1.4.5 The `on_server_connected` connect-handler discharge (M07.V 🟡 #3)

- `connect_test_session_mcp_calls_on_server_connected_per_server`
- `connect_test_session_mcp_new_ambiguity_emits_tool_alias_ambiguous`
- `connect_test_session_mcp_teardown_calls_on_server_disconnected`

#### F1.4.6 The assembled-app regression — `crates/runtime-main/tests/tester_isolated_session.rs`

The mandatory assembled regression (F1.3.8) — a real drone subprocess + a real `run_test_session_with` + a concrete `McpDispatcher` + a tool-bearing framework:

- `tester_runs_a_tool_bearing_framework_through_the_real_loop_and_persists_signals` — the production-driver discharge (M07.V 🟡 #5): a real `ProviderEvent::ToolUse` flows through the concrete `McpDispatcher`; signals land under the test session id
- `tester_run_persists_non_zero_token_usage_under_the_test_session_id` — asserts `token_usage > 0`, not merely `signals > 0`
- `tester_throwaway_db_is_isolated_from_a_user_session_db` — open a separate user DB; run the Tester against the throwaway DB; assert the user DB is untouched
- `tester_session_teardown_removes_the_throwaway_db` — post-run, the temp DB file is gone

#### F1.4.7 `test_framework` command — `src-tauri/src/commands.rs` `#[cfg(test)]`

The binary-crate scoped-diff variant applies (the in-source `#[cfg(test)]` block byte-identical red→impl):

- `test_framework_returns_test_outcome` (against the injected construction)
- `test_framework_resolves_a_throwaway_db_path_not_the_user_session_db`

#### F1.4.8 Acceptance criteria

- [ ] `cargo test -p runtime-main` — the Tester unit tests + the `tester_isolated_session` assembled regression pass
- [ ] `cargo test -p runtime-main --test tester_isolated_session` — the real-drone-subprocess regression green cross-platform
- [ ] M07.V 🟡 #2 / #3 / #5 discharged — the `<construction_reachability_check>` inverted to `inputs_reachable="true"` with a concrete file:line per wire
- [ ] `cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs" --fail-under-lines 95` — the gate holds with the new `builder::tester` module; any new exclusion synced across all four mirrors
- [ ] `cargo llvm-cov --workspace … --fail-under-lines 80` holds
- [ ] `test_framework` compiles + is registered in the `main.rs` `invoke_handler`
- [ ] ADR-0019 filed (`Proposed`)
- [ ] strict v1.8 two-commit invariant: `git diff <red>..<impl> -- '**/tests/**'` EMPTY (binary-crate scoped-diff for the `commands.rs` `#[cfg(test)]` block)
- [ ] CI-parity per G6

### F1.5 CLI Prompt

```xml
<work_stage_prompt id="M08.F1">
  <context>
    M08 Stage F1 — the Tester backend: test_framework runs the candidate
    framework (from the canvas, no save) in an ISOLATED session — a
    throwaway SQLite DB, test-defaults for capability violations (→ test
    failures, NOT live HITL), no user-data-dir writes, torn down on
    close. Reuse the smoke-session infra (run_smoke_session_with /
    drone_lifecycle / AgentSdk::run_agent) — do NOT rebuild a session
    engine. This stage DISCHARGES the coupled M07.V Dec-6 set Stage A
    mapped: 🟡 #5 (agent-with-tools production driver), 🟡 #2
    (skills_lock::verify on the artifact-load path), 🟡 #3
    (McpDispatcher::on_server_connected connect handler). Invert Stage
    A's <construction_reachability_check> to true with file:line.
    CODEOWNERS-flagged (sandbox/isolation + capability-enforcement
    behavior — the violation→test-failure response variation) — this
    stage doc + the reachability closure are the plan-first surface.
    File ADR-0019.
  </context>

  <read_first>
    <file>CLAUDE.md (§5 *_with seam archetype; §10 don't-touch + capability adherence; §11 ADR triggers; §6 assembled-regression mandate)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — construction_reachability_check; §1052 tag ordering; strict tdd_discipline two-commit)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Stage A A.3.7+A.3.8, Stage F1 F1.1–F1.4)</file>
    <file>docs/build-prompts/retrospectives/M07.V-retrospective.md ([END] Decisions 2/3/5 — the coupled Dec-6 set this stage discharges)</file>
    <file>docs/build-prompts/retrospectives/M08.E-retrospective.md ([END] Decisions — Stage E shipped openTester; F2 consumes F1's test_framework)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543; the Tester) + §1c + §0d (the §1c-vs-Tester scope — A.3.7 confirmed reading) + §5a step 5 (re-resolution) + §2214 (hash-on-load) + §8.security L2</file>
    <file>docs/adr/0011 (the (d) carry-forward — M08 is the named home for the agent-with-tools production driver) + docs/adr/0014 (skills.lock integrity > availability) + docs/adr/0007 (the in-process HitlSeam) + docs/adr/0000-template.md</file>
    <file>docs/gotchas.md (#22 current_exe binary location; #66 tests-pass-contract-fails; #69 multi-call; #74 cfg-Linux first-compile; #81 llvm-cov clean)</file>
    <file>docs/coverage-policy.md (§A current exclusion set; §B baselines; the four-mirror change protocol — read before adding any exclusion)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="run_smoke_session_with (line 194) — the construction F1 reuses; run_smoke_session (120) the production-wrapper archetype; build_mcp_dispatcher (241) the concrete-dispatcher build">src-tauri/src/commands.rs</file>
    <file purpose="DroneLifecycle::spawn (87, takes db_path: PathBuf — F1 passes a throwaway path); sdk_session_id (176); shutdown (192) — teardown reaps the drone">src-tauri/src/drone_lifecycle.rs</file>
    <file purpose="AgentSdk::run_agent (256) — the M07.D2 multi-turn agent-with-tools loop the Tester reuses; AgentSdk::new; with_mcp_dispatch">crates/runtime-main/src/sdk/agent_sdk.rs</file>
    <file purpose="skills_lock::verify (107) — F1 wires the FIRST production load-path caller; LockError::HashMismatch is the integrity block (M07.V 🟡 #2)">crates/runtime-main/src/skills_lock/mod.rs</file>
    <file purpose="McpDispatcher::on_server_connected (141) / on_server_disconnected (155) — F1 wires the FIRST production connect-handler caller; NewAmbiguity is the §5a re-resolution delta (M07.V 🟡 #3)">crates/runtime-mcp/src/dispatch.rs</file>
    <file purpose="NewAmbiguity field names — pin short_name/candidates for the ToolAliasAmbiguous mapping">crates/runtime-mcp/src/namespace.rs</file>
    <file purpose="ArtifactHashMismatch (1504) + ToolAliasAmbiguous (1448/1926) variant fields — pin before authoring the emit calls">crates/runtime-core/src/generated/event.rs</file>
    <file purpose="HitlSeam — F1 adds a test_defaults() constructor; HitlChoice/HitlPrompt the prompt shape">crates/runtime-main/src/hitl/seam.rs</file>
    <file purpose="the assembled-regression harness archetype — ensure_drone_built/spawn_drone/connect_with_retry/poll_until; real drone subprocess; cross-platform in CI">crates/runtime-main/tests/smoke_signal_persistence.rs</file>
    <file purpose="the concrete-McpDispatcher assembled regression — token_usage>0 assertion + MockTransport-scripted concrete dispatcher (NOT a mock seam — gotcha #66)">crates/runtime-main/tests/agent_with_tools_loop.rs</file>
    <file purpose="the seam-vs-wrapper / OS-call-holdout precedent — F1's drone-spawn + temp-file IO mirror this exclusion pattern">crates/runtime-main/src/import/fetch.rs</file>
    <file purpose="Stage B's builder module — F1 adds builder::tester + the artifact-load path">crates/runtime-main/src/builder/mod.rs</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M08.E" retro="docs/build-prompts/retrospectives/M08.E-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M08-workbench.md" section="F1.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      One standalone `test(M08.F1): …` commit on the M08
      parent-milestone branch with the failing tests across F1.4's
      buckets: run_test_session_with over a tempfile DB → a
      TestOutcome; a framework whose agent violates a capability → a
      CapabilityFailure on the outcome with NO HITL prompt raised; the
      throwaway DB is a distinct path from a user session DB; teardown
      deletes the temp DB; the assembled regression (real drone + real
      loop + a tool-bearing framework + a concrete McpDispatcher →
      signals persisted under the test session id, token_usage > 0);
      skills_lock::verify on the artifact-load path (tampered bytes →
      ArtifactHashMismatch → load refused → test fails);
      on_server_connected called on the test-session MCP connect (a
      collision → ToolAliasAmbiguous). Stub production surfaces just
      enough to compile (todo!()/unimplemented!() bodies); confirm
      right-reason failure per CLAUDE.md §5 (assertion failed / cannot
      find function / unresolved import / not-yet-implemented panic —
      NOT a test-file compile error, NOT a tautological pass). The
      commit body pastes the first ~40 lines of `cargo test` output
      proving the expected-failure class. Surface for red approval.
    </red_phase>
    <green_phase>
      Seam-first impl reusing the smoke-session infra; implement until
      ALL failing tests pass. Do NOT modify the test files during
      implementation — a wrong test is fixed in a SEPARATE labelled
      follow-up commit, never silently in the impl commit. The impl
      commit body MUST prove `git diff &lt;red&gt;..&lt;impl&gt; --
      '**/tests/**'` is EMPTY (the binary-crate scoped-diff variant —
      an in-source `#[cfg(test)]` block byte-identical red→impl via a
      scoped `git diff … src-tauri/src/commands.rs` — applies to the
      test_framework command tests). Net-new additive tests + mechanical
      rustfmt/clippy fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message.
    </green_phase>
  </tdd_discipline>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="F1.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>

  <gates milestone="M08"/>

  <coverage_gate>
    <gate scope="workspace" target_lines="80" ignore_filename_regex="src.main\.rs|generated"/>
    <gate scope="package" name="runtime-main" target_lines="95" ignore_filename_regex="src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <adr_triggers>
    <trigger>ADR-0019 — the Tester isolated-session model: the throwaway SQLite DB, test-defaults for capability violations (the §8.security L2 violation→test-failure RESPONSE variation), discard-on-close, and the explicit §1c-vs-v0.1-scope reconciliation (the v0.1 Tester is a sequential throwaway build-time session, NOT the §1c concurrent-session pool — Stage A A.3.7 confirmed this). Filed THIS stage; Proposed → Accepted in the M08 PR before merge. If the token-bearing path or any wire needs a schema change to event.v1.json that triggers §14 (schema + regenerate + ADR) — F1 expects NONE (ArtifactHashMismatch + ToolAliasAmbiguous shipped at M07) — surface before red if one surfaces. If a new OS-call-holdout coverage exclusion is added for the drone-spawn / temp-file-IO wrapper, the v1.8 four-mirror sync (CLAUDE.md §5/§6 + docs/coverage-policy.md §A/§C + codecov.yml) lands in this stage's commit.</trigger>
  </adr_triggers>

  <architecture_check>
    <claim description="The Tester reuses the smoke-session construction — AgentSdk::new → optional with_mcp_dispatch → run_agent — it does NOT introduce a new session engine" verify="grep -n 'AgentSdk::new\\|run_agent' crates/runtime-main/src/builder/tester.rs ; expect the reused construction, no bespoke turn loop"/>
    <claim description="The throwaway test DB path is resolved in the Tauri shell via std::env::temp_dir(), NEVER AppHandle::path().app_local_data_dir() — no user-data-dir writes (spec Phase 9)" verify="grep -n 'app_local_data_dir\\|temp_dir' src-tauri/src/commands.rs ; expect temp_dir for the Tester path, no app_local_data_dir join for it"/>
    <claim description="run_test_session_with is the *_with seam (injectable provider/drone/dispatch); the OS-touching drone-spawn + temp-file IO live on the production side — the import/fetch.rs seam-vs-wrapper pattern" verify="grep -n 'pub async fn run_test_session_with\\|pub async fn run_test_session' crates/runtime-main/src/builder/tester.rs ; expect both, the seam injectable"/>
    <claim description="A capability violation / hash mismatch is a FAILED TEST (TestOutcome { passed: false }), not a TesterError — TesterError is infrastructure-only" verify="grep -n 'enum TesterError\\|CapabilityFailure' crates/runtime-main/src/builder/tester.rs ; confirm HashMismatch/violation are NOT TesterError variants"/>
    <claim description="No new HITL/gap trigger for the test session — F1 adds a test-defaults HitlSeam variant; the live HitlSeam path is unchanged (ADR-0007 reuse)" verify="grep -rn 'TesterSeam\\|test_defaults' crates/runtime-main/src/hitl/ ; expect a constructor on the EXISTING HitlSeam, not a new seam type"/>
  </architecture_check>

  <construction_reachability_check>
    <wire claim="agent-with-tools production driver (a real tool-emitting ProviderEvent::ToolUse dispatched through the concrete McpDispatcher in production)" constructor="crates/runtime-main/src/builder/tester.rs run_test_session_with → AgentSdk::run_agent with a tool-bearing candidate framework (file:line filled at impl)" inputs_reachable="true — F1 makes reachable; the Tester runs the candidate framework's declared tools through run_agent" resolution="discharged at F1 (M07.V 🟡 #5)"/>
    <wire claim="skills_lock::verify called on the artifact-load path" constructor="crates/runtime-main/src/builder/ the test session's artifact-load path — load_verified_artifact calls skills_lock::verify before execution (file:line filled at impl)" inputs_reachable="true — F1 makes reachable; verify is called when the test session byte-loads an imported artifact" resolution="discharged at F1 (M07.V 🟡 #2)"/>
    <wire claim="McpDispatcher::on_server_connected production connect-handler" constructor="crates/runtime-main/src/builder/tester.rs the test session's MCP connect handler — connect_test_session_mcp calls on_server_connected per server (file:line filled at impl)" inputs_reachable="true — F1 makes reachable; on_server_connected is called when the test session connects the framework's MCP servers" resolution="discharged at F1 (M07.V 🟡 #3)"/>
  </construction_reachability_check>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch">HEAD == the M08 parent-milestone branch; M08.E impl commit present</check>
    <check name="stage_a_map">read Stage A's &lt;construction_reachability_check&gt; — F1 inverts all three wires to inputs_reachable="true" with a concrete file:line each</check>
    <check name="reuse_surfaces">grep-confirm run_smoke_session_with + DroneLifecycle::spawn + AgentSdk::run_agent + skills_lock::verify + McpDispatcher::on_server_connected exist with the F1.3-cited signatures before wiring</check>
    <check name="builder_module">grep-confirm Stage B shipped crates/runtime-main/src/builder/mod.rs before adding builder::tester</check>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="file" path="src-tauri/src/drone_lifecycle.rs" verified="true"/>
    <claim type="file" path="crates/runtime-main/src/builder/mod.rs" verified="true" note="Stage B created"/>
    <claim type="method" path="crates/runtime-main/src/sdk/agent_sdk.rs" symbol="AgentSdk::run_agent" verified="true" note="M07.D2 multi-turn loop"/>
    <claim type="method" path="src-tauri/src/commands.rs" symbol="run_smoke_session_with" verified="true" note="the construction F1 reuses"/>
    <claim type="method" path="src-tauri/src/drone_lifecycle.rs" symbol="DroneLifecycle::spawn" verified="true" note="takes db_path: PathBuf — F1 passes a throwaway path"/>
    <claim type="method" path="crates/runtime-main/src/skills_lock/mod.rs" symbol="verify" verified="true" note="signature verify(path, artifact_ref, artifact_bytes) -> Result&lt;(), LockError&gt;"/>
    <claim type="method" path="crates/runtime-mcp/src/dispatch.rs" symbol="McpDispatcher::on_server_connected" verified="true" note="returns Vec&lt;NewAmbiguity&gt;"/>
    <claim type="enum_variant" path="crates/runtime-core/src/generated/event.rs" symbol="ArtifactHashMismatch" verified="true" note="pin the field set before authoring the emit call"/>
    <claim type="enum_variant" path="crates/runtime-core/src/generated/event.rs" symbol="ToolAliasAmbiguous" verified="true" note="pin the field set (short_name + candidates) before authoring the emit call"/>
    <claim type="method" path="crates/runtime-main/src/hitl/seam.rs" symbol="HitlSeam::test_defaults" verified="false" note="F1 adds this constructor"/>
    <claim type="file" path="crates/runtime-main/src/builder/tester.rs" verified="false" note="F1 creates"/>
    <claim type="ipc_command" path="src-tauri/src/commands.rs" symbol="test_framework" verified="false" note="F1 creates + registers in main.rs invoke_handler"/>
  </phase_doc_inventory_audit>

  <existing_pattern_audit>
    <pattern grep_for="DroneLifecycle::spawn" rationale="DroneLifecycle::spawn already takes db_path: PathBuf — the Tester passes a throwaway path; F1 must NOT add a spawn variant or change the signature" affected_files="src-tauri/src/drone_lifecycle.rs (0 signature changes)" remediation="reuse spawn(throwaway_path); if a new spawn surface seems necessary, surface in the retrospective"/>
    <pattern grep_for="*_with" rationale="every command/session surface has a *_with injectable seam (CLAUDE.md §5); run_test_session_with must follow it so the OS-touching drone-spawn stays on the production wrapper side" affected_files="crates/runtime-main/src/builder/tester.rs" remediation="seam is injectable provider/drone/dispatch; the production run_test_session constructs the real collaborators"/>
  </existing_pattern_audit>

  <gotchas>
    <trap>Reuse the smoke-session infra (run_smoke_session_with / drone_lifecycle / AgentSdk::run_agent) — do NOT rebuild a session engine. The Tester differs only in: a throwaway DB path, the candidate-framework AgentConfig, and the test-defaults HitlSeam.</trap>
    <trap>Hard Rule 8 — F1 changes the capability-enforcement RESPONSE (a §8.security L2 violation → a CapabilityFailure on TestOutcome, not a live HITL/gap prompt) scoped to the test session; it does NOT change enforcement LOGIC (CapabilityEnforcer::check is byte-identical). The plan-first surface is this stage doc + the reachability closure; ADR-0019 records the behavior variation.</trap>
    <trap>The throwaway test DB is NEVER the user session DB — a distinct std::env::temp_dir() path; deleted on teardown; no app_local_data_dir writes (spec Phase 9 "test runs do not write to any user data directory").</trap>
    <trap>The §1c phrasing — the v0.1 Tester is a sequential throwaway build-time session, NOT §1c multi-session (Stage A A.3.7 confirmed this; ADR-0019 records it). It does not run concurrently with a live runtime session.</trap>
    <trap>#66 tests-pass-contract-fails — the assembled regression drives the REAL Tester path with a CONCRETE McpDispatcher (MockTransport-scripted, real CapabilityEnforcer + NamespaceResolver), NOT a mock McpToolDispatch seam (the mock seam already passes in mcp_dispatch_runloop.rs — that is exactly the blind spot). Assert token_usage > 0, not merely signals > 0.</trap>
    <trap>#22 — the assembled regression locates the runtime-drone binary via current_exe(); reuse the smoke_signal_persistence.rs `common::drone_binary`/`ensure_drone_built` harness so it works under both `cargo test` and `cargo llvm-cov`.</trap>
    <trap>#69 multi-call — run_test_session_with has a *_twice_in_sequence test (distinct throwaway DB paths).</trap>
    <trap>#81 — `cargo llvm-cov clean --workspace` before the coverage gates; a stale .profraw merge produces phantom numbers.</trap>
    <trap>A capability violation / hash mismatch is a FAILED TEST (Ok(TestOutcome { passed: false })), not Err(TesterError). TesterError is infrastructure-only (drone spawn failed, temp-file IO failed).</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT build a new session/turn engine. AgentSdk::run_agent IS the agent-with-tools loop (M07.D2); run_smoke_session_with IS the construction. The Tester reuses both. If the Tester appears to need a new loop, the design is wrong — surface it.</warning>
    <warning>DO NOT raise a live HITL or gap prompt from the test session. The test-defaults HitlSeam auto-resolves; capability violations are collected onto TestOutcome.capability_failures. A test session that blocks on user input is a spec violation (Phase 9).</warning>
    <warning>DO NOT point the test session's drone at the user session DB. Resolve a fresh std::env::temp_dir() path per run; delete it on teardown. A test run that mutates the user's session.sqlite is a data-integrity failure.</warning>
    <warning>DO NOT add a new event variant for the test session. AgentEvent::ArtifactHashMismatch (M07.B) and AgentEvent::ToolAliasAmbiguous (M06.D) already exist — the discharges reuse them. If a genuinely new variant seems necessary, that is a §14 schema trigger — surface before red, do not author a hand-rolled type.</warning>
    <warning>DO NOT change the drone IPC protocol or DroneCommand variants. The Tester uses the existing drone exactly as a smoke session does, just against a different DB path.</warning>
  </execution_warnings>

  <runtime_environment os="windows" note="Build on Windows (the v0.1 target); the assembled regression spawns a real runtime-drone subprocess — the smoke_signal_persistence.rs harness already runs cross-platform in CI; the throwaway temp DB uses std::env::temp_dir() which is platform-correct"/>

  <time_box hours="9-12"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the inverted &lt;construction_reachability_check&gt; — all three M07.V Dec-6 wires now inputs_reachable="true", each with the concrete file:line of the production caller. Confirm the smoke-session-infra reuse (no new session engine — name the reused surfaces). Confirm the throwaway-DB isolation + the test-defaults HitlSeam + teardown + no-user-data-writes. Record the assembled-regression result (real drone subprocess, token_usage > 0, user DB untouched). ADR-0019 status. Any new coverage exclusion + the four-mirror sync. Note any cross-stack-integration friction (CLAUDE.md §7 — escalate at iteration 2 if iteration 2 advances to a new error class).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="F1.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls M08.*-retrospective.md)</item>
    <item>strict-TDD invariant: git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**' EMPTY (binary-crate scoped-diff for the commands.rs #[cfg(test)] block)</item>
    <item>the inverted &lt;construction_reachability_check&gt; — M07.V 🟡 #2/#3/#5 discharged with a concrete file:line per wire</item>
    <item>the smoke-session-infra reuse confirmation (named reused surfaces, no new session engine); the throwaway-DB isolation + test-defaults model (ADR-0019)</item>
    <item>the assembled-regression result (real drone subprocess, real loop, token_usage > 0, the throwaway DB isolated from a user session DB)</item>
    <item>gate results (v1.6 order; runtime-main ≥95 incl. the builder::tester module; workspace ≥80; any four-mirror coverage sync; CI-parity per G6)</item>
    <item>ADR-0019 (Proposed)</item>
    <item>M08.F1 retrospective [END]; explicit "Stage M08.F1 is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### F1.6 Commit Message

```
feat(runtime): M08 Stage F1 — Tester backend (isolated session + the M07.V Dec-6 discharge)

test_framework runs the candidate framework (from the canvas, no
save) in an isolated session — a throwaway SQLite DB, test-defaults
for capability violations (→ test failures, not live HITL), no
user-data-dir writes, torn down on close — reusing the smoke-session
infra (run_smoke_session_with / drone_lifecycle / AgentSdk::run_agent)
rather than building a new session engine.

Discharges the coupled M07.V Dec-6 set Stage A mapped:
- 🟡 #5 — the Tester runs a tool-bearing framework through
  AgentSdk::run_agent, so a real ProviderEvent::ToolUse is dispatched
  through the concrete McpDispatcher in a production path for the
  first time (the agent-with-tools production driver).
- 🟡 #2 — the test session's artifact-load path calls
  skills_lock::verify; a HashMismatch → AgentEvent::ArtifactHashMismatch
  + the load is refused (verify gets its first production caller).
- 🟡 #3 — the test session's MCP connect handler calls
  McpDispatcher::on_server_connected; each NewAmbiguity →
  AgentEvent::ToolAliasAmbiguous (§5a step 5 — first production
  connect-handler caller).

Stage A's <construction_reachability_check> inverted to
inputs_reachable="true" with a file:line per wire. The assembled
regression drives the real Tester path — a real runtime-drone
subprocess, a concrete McpDispatcher, a tool-bearing framework —
and asserts signals persist under the test session id, token_usage
> 0, and the throwaway DB is isolated from a user session DB.

ADR-0019 (Tester isolated-session model — throwaway DB, test-defaults
for capability violations, discard-on-close, the §1c-vs-v0.1-scope
reconciliation) Proposed. Strict v1.8 two-commit TDD: '**/tests/**'
diff EMPTY. runtime-main ≥95 incl. the builder::tester module.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Stage F2 — Tester modal renderer

### F2.1 Problem Statement

Spec Phase 9 Tester (modal, opens from the Inspector): *"Load the framework from canvas; define a test task (natural-language input); run … watch graph render in a smaller pane; review VDR + signals output; check token spend and timing; pass / fail with full trace."* MVP §M8 criterion 5: click Test → enter a task → Tester modal opens → sandboxed session runs → graph + VDR + token spend + pass/fail surface.

Stage E shipped the Test button + `builderStore.openTester`; Stage F1 shipped the `test_framework` backend + the `TestOutcome` shape. F2 builds the **modal** — the renderer surface that opens on `openTester`, takes the task, calls `test_framework`, and renders the test run. F2 reuses the existing live-graph rendering for the smaller graph pane and the existing result-surface idioms (VDR / token / signals panels); it builds **no backend**.

The single load-bearing renderer constraint: the test session's graph pane must be **scoped** to the test session. The test session emits `AgentEvent`s on the existing `agent_event` channel under its **own** session id; if the modal fed those into the live `useGraphStore` module singleton (`src/lib/graphStore.ts:752` — `create<GraphState>(...)`), the test run would corrupt the live runtime graph. F2 renders the test run in a graph instance scoped to the test session so the live graph is untouched. The scoping mechanism is pinned at authoring time (a Zustand store-factory instance vs a scoped local reducer over the same `applyEvent` logic) via the `<phase_doc_inventory_audit>` — whichever it is, the invariant is: `useGraphStore` (the live singleton) is never written to by the test run.

Concrete deliverables:

1. **`src/components/builder/TesterModal.tsx`** — the modal. Renders when `builderStore`'s Tester-open state is set (E's Test button); a natural-language task-description input + a Run button; the smaller graph pane; the result surfaces (VDR/signals, token spend + timing, pass/fail with the full trace); discard-on-close + an explicit Save/Promote affordance.
2. **`testFramework` IPC wrapper** (`src/lib/ipc.ts`) — a typed wrapper around the Stage F1 `test_framework` command (`(framework, task) → TestOutcome`); params PINNED to the shipped F1 signature via `<wire_signature_audit>`.
3. **`TestOutcome` TS type** — the renderer-side mirror of F1's `TestOutcome` serde shape (`passed`, `capability_failures`, `token_spend`, `timing`, `vdr`, `trace`). Hand-mirrored — `TestOutcome` is not schema-generated; it lives in `crates/runtime-main/src/builder/tester.rs` and crosses the Tauri bridge as-is (the `McpTool` / `McpServerSummary` precedent in `ipc.ts`).
4. **The scoped test-session graph instance** (`src/lib/builderStore.ts`) — the Tester-open state (set by E's `openTester`) + the test-session graph instancing, scoped so it does not pollute the live `graphStore`.
5. **`BuilderShell.tsx` mount** — `TesterModal` mounted in the Builder shell so `openTester` renders it.
6. **Vitest unit tests** + **a Playwright behavior test** (gotcha #23) for the Test → task → modal → graph + VDR + token + pass/fail flow. Renderer ≥80.

Not in this stage:
- Any backend (F1 shipped `test_framework`; F2 only calls it).
- A new node type or a new `AgentEvent` variant (F2 reuses the 11 live-graph node components + the existing event reducer logic).
- The §1c concurrent-multi-session feature (the Tester is a sequential, throwaway, build-time session — ADR-0019).
- Token in/out *projection* fixes (Stage A discharged M07-IRL #2 — F2 *renders* the in/out split; it does not re-diagnose the projection).

### F2.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/components/builder/TesterModal.tsx` | new | The modal: task input + Run; the smaller graph pane (scoped to the test session); VDR/signals + token/timing + pass/fail trace; discard-on-close + an explicit Save/Promote affordance. |
| `src/lib/ipc.ts` | exists | `testFramework(framework, task) → TestOutcome` wrapper + the `TestOutcome` TS type — params PINNED to the Stage F1 command via `<wire_signature_audit>`. |
| `src/lib/builderStore.ts` | exists | The Tester-open state (set by E's `openTester`); the test-session graph instancing (scoped — must not pollute the live `graphStore`). |
| `src/components/builder/BuilderShell.tsx` | exists | Mount `TesterModal`; the modal renders on the `builderStore` Tester-open state. |
| `src/styles.css` | exists | Classes for `.tester-modal`, `.tester-modal__task-input`, `.tester-graph-pane`, `.tester-result--pass`, `.tester-result--fail`, `.tester-result__vdr`, `.tester-result__tokens`, `.tester-capability-failure`. Every className paired with a CSS rule (gotcha #67). |
| `tests/unit/components/builder/TesterModal.test.tsx` | new | Vitest — modal state, the scoped graph instancing, the result surfaces, the discard path. |
| `tests/e2e/tester_modal.spec.ts` | new | Playwright — the Test → task → modal → graph + VDR + token + pass/fail flow (gotcha #23; `test_framework` mocked + scripted test-session events). |
| `CHANGELOG.md` | exists | `[Unreleased]` Stage F2 entries. |
| `docs/build-prompts/retrospectives/M08.F2-retrospective.md` | new | Stage F2 retrospective. |

Effort budget: ~8–11 hours. Renderer-only; the modal/panel pattern locks from M04.E (HITL modals) + M06.E (the MCP Settings modal) carry forward. The one piece of genuine renderer design is the **scoped graph instancing** — reusing the React-Flow rendering without writing to the live `graphStore` singleton; pin the mechanism before authoring component pseudocode.

### F2.3 Detailed Changes

#### F2.3.1 Wire pinning (v1.8 discipline — BEFORE pseudocode)

`<wire_signature_audit>` (F2.5): pin the Stage F1 `test_framework` command signature from the **shipped** `src-tauri/src/commands.rs` — F1's parameter names (`framework_doc` / `task` per F1.3.1) and the `TestOutcome` return shape. `<phase_doc_inventory_audit shape=…>`: pin the `TestOutcome` serde shape (field names + types — `passed`, `capability_failures`, `token_spend`, `timing`, `vdr`, `trace`) so the TS mirror is exact, and pin **how the test session's `AgentEvent`s are scoped** (the test session id F1 spawns the drone under). The M06.E / M07.E lesson: pin the actual Tauri command params + the store-slot TS types *before* authoring component pseudocode — a TS mirror drafted against an assumed shape that diverges from the shipped serde shape is a silent wire bug.

#### F2.3.2 The Tester modal — task input + Run

`TesterModal` renders when `builderStore`'s Tester-open state is set (E's Test button calls `builderStore.openTester()`). It takes a **natural-language task description** and, on Run, calls `testFramework(builderStore.framework, task)` — the candidate framework crosses the wire directly (spec Phase 9: "load from canvas — does NOT need to save first"; no disk round-trip). The modal does not block the Builder shell; closing it discards the run (F2.3.5).

```tsx
// src/components/builder/TesterModal.tsx
import { useState } from 'react';
import { useBuilderStore } from '../../lib/builderStore';
import { testFramework, type TestOutcome } from '../../lib/ipc';
import { TesterGraphPane } from './TesterGraphPane';

/**
 * The Builder's Tester modal (spec Phase 9; MVP §M8 criterion 5).
 * Opens on `builderStore`'s Tester-open state (Stage E's Test button).
 * Takes a natural-language task, runs the candidate framework through
 * Stage F1's `test_framework`, and renders the test run — a smaller
 * graph pane + VDR/signals + token/timing + pass/fail trace.
 *
 * Discard-on-close is the default (F2.3.5); the explicit Save/Promote
 * affordance is the only persist path.
 */
export function TesterModal(): JSX.Element | null {
  const isOpen = useBuilderStore((s) => s.testerOpen);
  const framework = useBuilderStore((s) => s.framework);
  const closeTester = useBuilderStore((s) => s.closeTester);
  const [task, setTask] = useState('');
  const [outcome, setOutcome] = useState<TestOutcome | null>(null);
  const [running, setRunning] = useState(false);

  if (!isOpen) return null; // Stage E ships `openTester`; F2 renders on it.

  const handleRun = async (): Promise<void> => {
    setRunning(true);
    // The candidate framework crosses the wire directly — spec Phase 9
    // "load from canvas — does NOT need to save first".
    const result = await testFramework(framework, task);
    setOutcome(result);
    setRunning(false);
  };

  const handleClose = (): void => {
    // Discard-on-close (F2.3.5): F1's backend deletes the throwaway DB;
    // the modal drops the scoped graph instance + the outcome.
    setOutcome(null);
    closeTester();
  };

  return (
    <div className="tester-modal" role="dialog" aria-label="Tester">
      <header>
        <h2>Test framework</h2>
        <button onClick={handleClose} aria-label="Close tester">×</button>
      </header>
      <textarea
        className="tester-modal__task-input"
        placeholder="Describe a task for the test session…"
        value={task}
        onChange={(e) => setTask(e.target.value)}
      />
      <button onClick={handleRun} disabled={running || task.trim() === ''}>
        {running ? 'Running…' : 'Run'}
      </button>
      {/* The smaller graph pane — scoped to the test session (F2.3.3). */}
      <TesterGraphPane />
      {outcome && <TesterResult outcome={outcome} />}
    </div>
  );
}
```

The Run button is disabled while a run is in flight and while the task is empty. The modal pattern (header + close + body) follows the M04.E HITL-modal idiom.

#### F2.3.3 The smaller graph pane (reuse the live-graph rendering, scoped)

The test session emits `AgentEvent`s on the existing `agent_event` channel under the **test session's own session id** (F1 spawns the drone under a throwaway session id). The modal renders a **smaller graph pane** for the test run — reusing the React-Flow node rendering + the 11 node components + the `graphStore` event-reducer *logic*, in an instance **scoped to the test session** so it does not pollute the live `graphStore`. `useGraphStore` (`graphStore.ts:752`) is a `create<GraphState>(...)` module singleton — feeding the test session's events into it would corrupt the live runtime graph.

```tsx
// src/components/builder/TesterGraphPane.tsx — the smaller graph pane.
// Reuses the React-Flow rendering + the 11 node components, scoped to
// the test session so the live graphStore singleton is UNTOUCHED.
import { ReactFlow, Background, Controls } from '@xyflow/react';
import { layoutGraph } from '../../lib/layout';
import { nodeTypes } from '../GraphCanvas'; // reuse the 11-entry node map

/**
 * The Tester's smaller graph pane (spec Phase 9 "watch graph render in
 * a smaller pane"). Renders the TEST session's nodes/edges from a
 * graph instance SCOPED to the test session — NOT the live
 * `useGraphStore` module singleton (writing the test run into the live
 * store would corrupt the runtime graph).
 *
 * The scoped instance is pinned via <phase_doc_inventory_audit> — a
 * Zustand store-factory instance OR a scoped local reducer over the
 * same `applyEvent` logic. Whichever: the live graph is untouched.
 */
export function TesterGraphPane(): JSX.Element {
  // The scoped test-session graph — sourced from builderStore's
  // test-session graph instance, NOT useGraphStore. The instance is
  // fed by the test session's `agent_event`s (filtered to the test
  // session id) and applies the SAME reducer logic as the live graph.
  const { nodes, edges } = useTestSessionGraph();
  const laid = layoutGraph(nodes, edges);
  return (
    <div className="tester-graph-pane">
      <ReactFlow nodes={laid} edges={edges} nodeTypes={nodeTypes} fitView>
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}
```

The build pins the instancing mechanism via `<phase_doc_inventory_audit>` — a Zustand store-factory instance (a second `create<GraphState>(...)` per test run) vs a scoped local reducer over the same `applyEvent` logic. Whichever is chosen, the invariant is non-negotiable: the live `useGraphStore` singleton is never written to by the test run, and a Playwright assertion confirms it (F2.4 — run a test, assert the live graph is unchanged). Reuse the module-level `nodeTypes` map (`GraphCanvas.tsx:22`) — it must stay a stable reference (the @xyflow/react v12 trap), so import it rather than redefining it.

#### F2.3.4 Results — VDR/signals, token/timing, pass/fail trace

From the `TestOutcome` (F1) the modal renders three result surfaces — reusing the live-graph inspector's display idioms rather than rebuilding:

- **VDR + signals** — `TestOutcome.vdr` + the test session's signals: the Verification & Decision Record review.
- **Token spend + timing** — `TestOutcome.token_spend` (`input` / `output` / `total`) + `TestOutcome.timing`, rendered against the typical-session benchmark per spec. (F2 *renders* the in/out split; Stage A discharged the M07-IRL #2 projection bug — F2 does not re-diagnose it.)
- **Pass / fail with the full trace** — `TestOutcome.passed` + `TestOutcome.trace`, including the `TestOutcome.capability_failures` entries surfaced as **test failures** (not HITL — F1.3.3). Each `CapabilityFailure` (`agent_id` / `needed` / `reason`) renders as a failure line.

```tsx
// src/components/builder/TesterModal.tsx — the result surface.
function TesterResult({ outcome }: { outcome: TestOutcome }): JSX.Element {
  return (
    <section className={`tester-result tester-result--${outcome.passed ? 'pass' : 'fail'}`}>
      <header>{outcome.passed ? 'PASS' : 'FAIL'}</header>
      {/* Capability violations surface as test failures, NOT HITL prompts
          (F1.3.3 — the test-defaults HitlSeam never prompted). */}
      {outcome.capability_failures.length > 0 && (
        <ul className="tester-capability-failures">
          {outcome.capability_failures.map((f, i) => (
            <li key={i} className="tester-capability-failure">
              <code>{f.agent_id}</code> — {f.needed}: {f.reason}
            </li>
          ))}
        </ul>
      )}
      <div className="tester-result__tokens">
        in {outcome.token_spend.input} · out {outcome.token_spend.output} ·
        total {outcome.token_spend.total} · {outcome.timing}ms
      </div>
      <pre className="tester-result__vdr">{JSON.stringify(outcome.vdr, null, 2)}</pre>
      {/* The full AgentEvent trace — spec Phase 9 "pass/fail with full trace". */}
    </section>
  );
}
```

MVP §M8 criterion 5's "graph + VDR + token spend + pass/fail" all land in F2 — the graph pane (F2.3.3) + these three surfaces.

#### F2.3.5 Discard-on-close + explicit save

Spec Phase 9: *"Test runs do not write to any user data directory; results discarded on close unless explicitly saved."* Closing the modal tears down the test run: F1's backend already deletes the throwaway DB (F1.3.2) on the command path; the modal drops the scoped graph instance + the `TestOutcome`. An **explicit "Save results" / "Promote to main session"** affordance is the only path that persists anything (spec: "full graph available by promoting test session to main"). Default: discard. The `closeTester` handler in F2.3.2 clears `outcome` and the scoped graph; no `TestOutcome` is written to disk or to the live store unless the user explicitly invokes Save/Promote.

#### F2.3.6 Styles

`.tester-result--pass` (green) / `.tester-result--fail` (red) reuse the M05.F `.capability-badge--<tier>` colour tokens for visual consistency. Per gotcha #67 (a component rendered in the DOM ≠ a CSS rule exists), every new className gets a corresponding CSS rule in `src/styles.css`; a static test asserts via the existing `every_class_has_a_corresponding_CSS_rule` pattern (M04.F).

### F2.4 Tests

#### F2.4.1 `TesterModal` vitest tests — `tests/unit/components/builder/TesterModal.test.tsx`

- `does_not_render_when_tester_open_state_is_false`
- `renders_when_builderStore_tester_open_state_is_set`
- `run_button_disabled_when_task_is_empty`
- `run_calls_testFramework_with_builderStore_framework_and_the_task`
- `renders_pass_result_when_outcome_passed_is_true`
- `renders_fail_result_when_outcome_passed_is_false`
- `renders_capability_failures_as_test_failure_lines_not_hitl_prompts`
- `renders_token_spend_in_out_total_and_timing`
- `close_clears_the_outcome_and_calls_closeTester` (discard-on-close)
- `survives_repeated_open_close_cycles_with_a_fresh_run_each` (gotcha #66 — contract test)

#### F2.4.2 Scoped graph-instance tests

- `tester_graph_pane_renders_the_test_session_nodes`
- `running_a_test_does_not_write_to_the_live_useGraphStore_singleton` — the load-bearing scoping invariant: assert the live `graphStore` `nodes` is unchanged after a test run
- `closing_the_modal_drops_the_scoped_graph_instance`

#### F2.4.3 Playwright behavior test — `tests/e2e/tester_modal.spec.ts`

`test_framework` mocked + scripted test-session events emitted onto the `agent_event` channel under the test session id (gotcha #23 — Playwright drives the Vite dev server with `@tauri-apps/api` module-mocked):

- `the_Test_button_opens_the_Tester_modal` — E's Test button → `openTester` → the modal renders
- `entering_a_task_and_clicking_Run_renders_the_smaller_graph_pane` — Run → the scripted test-session nodes appear in the pane
- `the_VDR_token_and_pass_fail_surfaces_populate_from_the_TestOutcome`
- `a_capability_violating_framework_surfaces_a_test_failure_no_hitl_prompt` — no HITL modal appears; a failure line renders
- `closing_the_modal_discards_the_run_and_leaves_the_live_graph_untouched`

`test.describe.configure({ timeout: 90_000 })` per gotcha #53 (Vite cold-start); the curl warmup probe per the `<playwright_warmup_recipe>`.

#### F2.4.4 Acceptance criteria

- [ ] `npm run test` — all vitest tests pass (renderer ≥80 on `src/components/builder/TesterModal.tsx` + `TesterGraphPane.tsx`)
- [ ] `npx tsc --noEmit` — clean; the `TestOutcome` TS type matches the shipped F1 serde shape
- [ ] `npx eslint .` / `npx prettier --check` — clean
- [ ] `npm run test:e2e -- tester_modal.spec.ts` — passes against the Vite dev server with the curl warmup probe
- [ ] MVP §M8 criterion 5 demonstrated end-to-end in Playwright (Test → task → modal → graph + VDR + token + pass/fail)
- [ ] the smaller graph pane is scoped to the test session — `running_a_test_does_not_write_to_the_live_useGraphStore_singleton` green
- [ ] discard-on-close verified; the explicit Save/Promote affordance is the only persist path
- [ ] every new CSS class has a corresponding rule in `src/styles.css` (gotcha #67 + the `every_class_has_a_corresponding_CSS_rule` static check)
- [ ] `<wire_signature_audit>` matches the shipped Stage F1 `test_framework` wire
- [ ] strict v1.8 two-commit invariant: `git diff <red>..<impl> -- '**/tests/**'` EMPTY
- [ ] CI-parity per G6

### F2.5 CLI Prompt

```xml
<work_stage_prompt id="M08.F2">
  <context>
    M08 Stage F2 — the Tester modal renderer: opens on builderStore's
    Tester-open state (Stage E's Test button), takes a natural-language
    task, calls Stage F1's test_framework with builderStore.framework,
    renders the test run in a smaller graph pane (reuse the live-graph
    rendering, SCOPED to the test session — must NOT pollute the live
    useGraphStore module singleton), surfaces VDR/signals + token/timing
    + pass/fail trace (capability violations as test failures, NOT HITL
    — F1.3.3), and discards the run on close (the explicit Save/Promote
    affordance is the only persist path). Pin the Stage F1 wire BEFORE
    pseudocode. NO backend.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — wire_signature_audit + phase_doc_inventory_audit shape=; strict tdd_discipline two-commit)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Stage E E.3.2 the openTester action, Stage F1 F1.3.1 the TestOutcome shape, Stage F2 F2.1–F2.4)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543; the Tester modal) + MVP §M8 criterion 5</file>
    <file>docs/build-prompts/retrospectives/M08.F1-retrospective.md ([END] Decisions — the shipped test_framework wire + the TestOutcome shape this stage consumes)</file>
    <file>docs/adr/0019 (the Tester isolated-session model — F2 implements the renderer half) + docs/adr/0020 (the Builder canvas↔framework.json state model)</file>
    <file>docs/gotchas.md (#23 Playwright/Tauri module-mock; #27 vitest re-query after await; #30 unwrapCmdError; #53 Vite cold-start; #66 contract tests; #67 component+CSS contract; #75 useShallow)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="the SHIPPED Stage F1 test_framework command signature + the TestOutcome serde shape — PIN both before authoring the TS wrapper/type">src-tauri/src/commands.rs</file>
    <file purpose="the F1 TestOutcome Rust struct — the TS mirror must match field names + types exactly (not schema-generated; crosses the bridge as-is)">crates/runtime-main/src/builder/tester.rs</file>
    <file purpose="the live-graph rendering F2 reuses, SCOPED — React Flow setup + the module-level 11-entry nodeTypes map (stable reference — import, do not redefine)">src/components/GraphCanvas.tsx</file>
    <file purpose="the live graphStore — create&lt;GraphState&gt;() module singleton; F2 must NOT write the test run into it; the applyEvent reducer logic is what a scoped instance reuses">src/lib/graphStore.ts</file>
    <file purpose="ipc.ts existing wrapper pattern — invoke&lt;T&gt;(); the McpTool/McpServerSummary hand-mirrored-serde-type precedent; unwrapCmdError; subscribeAgentEvents">src/lib/ipc.ts</file>
    <file purpose="Stage E's Inspector — the Test button + builderStore.openTester this modal renders on">src/components/builder/Inspector.tsx</file>
    <file purpose="builderStore — F2 adds the Tester-open state + the scoped test-session graph instance">src/lib/builderStore.ts</file>
    <file purpose="Stage C's BuilderShell — F2 mounts TesterModal here">src/components/builder/BuilderShell.tsx</file>
    <file purpose="M04.E modal pattern — TesterModal mirrors the header+close+body idiom">src/components/HITLModal.tsx</file>
    <file purpose="dagre layout (layoutGraph, pure) — the scoped graph pane reuses it">src/lib/layout.ts</file>
    <file purpose="every_class_has_a_corresponding_CSS_rule static-check pattern (M04.F)">tests/unit/components/BudgetHeaderBar.test.tsx</file>
    <file purpose="Playwright spec pattern + the @tauri-apps/api module-mock + scripted agent_event emission">tests/e2e/gap_panel.spec.ts</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M08.F1" retro="docs/build-prompts/retrospectives/M08.F1-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M08-workbench.md" section="F2.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      One standalone `test(M08.F2): …` commit on the M08
      parent-milestone branch with the failing tests across F2.4's
      buckets: Playwright (the Test button → the modal opens; a task →
      Run → the scoped graph pane renders the test-session nodes;
      VDR/token/pass-fail populate from the TestOutcome; a
      capability-violating framework → a test failure with NO HITL
      prompt; close → the run is discarded, the live graph untouched) +
      vitest (modal state, the scoped graph instancing, the
      not-written-to-the-live-singleton invariant, the discard path).
      Stub the production surfaces just enough to compile; confirm
      right-reason failure per CLAUDE.md §5 (assertion failed / cannot
      find component / unresolved import — NOT a test-file compile
      error, NOT a tautological pass). The commit body pastes the first
      ~40 lines of the test output proving the expected-failure class.
      Surface for red approval.
    </red_phase>
    <green_phase>
      Implement the modal + the scoped graph pane + the result surfaces
      until ALL failing tests pass. Do NOT modify the test files during
      implementation — a wrong test is fixed in a SEPARATE labelled
      follow-up commit, never silently in the impl commit. The impl
      commit body MUST prove `git diff &lt;red&gt;..&lt;impl&gt; --
      '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      prettier/eslint fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message.
    </green_phase>
  </tdd_discipline>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="F2.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>

  <gates milestone="M08"/>

  <coverage_gate>
    <gate scope="renderer" target_lines="80" ignore_filename_regex="generated"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <adr_triggers>
    <trigger>None new in F2 (implements against ADR-0019 — the Tester isolated-session model — and ADR-0020 — the Builder state model). F2 adds no schema, no backend. If the scoped-graph-instance design appears to need a store-architecture change beyond a store-factory instance / a scoped reducer, surface it — but the default is no ADR.</trigger>
  </adr_triggers>

  <wire_signature_audit>
    <wrapper ipc_command="test_framework" actual_params="PIN from the Stage F1-shipped src-tauri/src/commands.rs signature (framework_doc + task per F1.3.1 — confirm against the shipped code)" phase_doc_assumed="(author fills from the shipped F1 signature — do not assume; the F1.3.1 names are illustrative)"/>
  </wire_signature_audit>

  <pre_flight_check>
    <check name="branch">HEAD == the M08 parent-milestone branch; M08.F1 impl commit present</check>
    <check name="stage_f1_wire">grep-confirm the Stage F1 test_framework command + the TestOutcome shape are shipped in src-tauri/src/commands.rs + crates/runtime-main/src/builder/tester.rs before pinning the wrapper</check>
    <check name="stage_e_opentester">grep-confirm Stage E shipped builderStore.openTester + the Inspector Test button before mounting the modal on it</check>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="ipc_command" path="src-tauri/src/commands.rs" symbol="test_framework" verified="true" note="Stage F1 shipped — PIN the param names + return shape"/>
    <claim type="struct" path="crates/runtime-main/src/builder/tester.rs" symbol="TestOutcome" shape="PIN the serde field set — passed, capability_failures, token_spend, timing, vdr, trace — so the TS mirror is exact" verified="true"/>
    <claim type="file" path="src/components/builder/Inspector.tsx" verified="true" note="Stage E — the Test button"/>
    <claim type="store_slot" path="src/lib/builderStore.ts" symbol="openTester" verified="true" note="Stage E shipped the action; F2 adds the testerOpen state + closeTester"/>
    <claim type="store_slot" path="src/lib/builderStore.ts" symbol="(scoped test-session graph instance)" shape="PIN the scoped instancing mechanism — a Zustand store-factory instance OR a scoped local reducer over the same applyEvent logic; it MUST NOT be the live useGraphStore singleton" verified="false" note="F2 adds"/>
    <claim type="file" path="src/components/builder/TesterModal.tsx" verified="false" note="F2 creates"/>
    <claim type="symbol" path="src/components/GraphCanvas.tsx" symbol="nodeTypes" verified="true" note="the module-level 11-entry map — import it, do not redefine (the @xyflow/react v12 stable-reference trap)"/>
  </phase_doc_inventory_audit>

  <architecture_check>
    <claim description="The test-session graph instance is SCOPED — F2 must NOT feed the test session's AgentEvents into the live useGraphStore module singleton (graphStore.ts:752 create&lt;GraphState&gt;())" verify="grep -n 'useGraphStore' src/components/builder/TesterModal.tsx src/components/builder/TesterGraphPane.tsx ; expect ZERO writes to the live store — reads of the scoped instance only"/>
    <claim description="The nodeTypes map is imported from GraphCanvas (a stable module-level reference), not redefined per render" verify="grep -n 'nodeTypes' src/components/builder/TesterGraphPane.tsx ; expect an import, not a new const NodeTypes literal"/>
    <claim description="testFramework is a thin ipc.ts wrapper over invoke('test_framework', ...) — no business logic in the wrapper; TestOutcome is the hand-mirrored serde type (the McpTool/McpServerSummary precedent)" verify="grep -n 'testFramework\\|TestOutcome' src/lib/ipc.ts ; expect a wrapper + an interface mirroring the F1 struct"/>
    <claim description="every new className has a corresponding CSS rule (gotcha #67)" verify="for each className in F2.3.6, grep src/styles.css for the rule; expect every class found"/>
  </architecture_check>

  <runtime_environment os="windows" note="Vitest + Playwright run against the Vite dev server with @tauri-apps/api module-mocked (gotcha #23 — Playwright cannot drive the Tauri window); Vite cold-start mitigated per gotcha #53 + the playwright_warmup_recipe"/>

  <gotchas>
    <trap>#23 — Playwright drives the Vite dev server; test_framework is mocked and the test-session AgentEvents are scripted onto the agent_event channel under the test session id.</trap>
    <trap>The test-session graph instance MUST be SCOPED — it must NOT pollute the live graphStore (graphStore.ts:752 is a create&lt;GraphState&gt;() module singleton). Pin a store-factory instance OR a scoped local reducer; either way the live graph is untouched and a Playwright assertion confirms it.</trap>
    <trap>Discard-on-close is the DEFAULT — only the explicit Save/Promote affordance persists anything (spec Phase 9). closeTester clears the outcome + the scoped graph; F1's backend already deleted the throwaway DB.</trap>
    <trap>Reuse the live-graph node rendering (the 11-entry nodeTypes map — import it, do not redefine; @xyflow/react v12 stable-reference trap) + the VDR/token display idioms — do not rebuild result surfaces.</trap>
    <trap>#27 vitest re-query after await — when asserting after `userEvent.click` (Run), re-query the DOM rather than reusing a captured handle.</trap>
    <trap>#30 unwrapCmdError — a test_framework error comes as a CmdError-shape object; use the existing helper to render it.</trap>
    <trap>#67 component+CSS contract — every className gets a CSS rule + the every_class_has_a_corresponding_CSS_rule static test.</trap>
    <trap>#53 Vite cold-start — the Playwright first-spec timeout is configured per the playwright_warmup_recipe; the curl warmup probe runs before the first spec.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT write the test session's AgentEvents into the live useGraphStore. The live store is the runtime graph; the Tester is a scoped, throwaway test run. A test run that mutates the live graph is the load-bearing bug F2 must not ship.</warning>
    <warning>DO NOT add a backend command or change test_framework — Stage F1 shipped it. F2 only calls it. If a new command appears necessary, surface in the retrospective; the default is "F2 renders only".</warning>
    <warning>DO NOT add a new node type or a new AgentEvent variant — F2 reuses the 11 live-graph node components + the existing reducer logic. The test session emits the SAME event vocabulary as a live session.</warning>
    <warning>DO NOT re-diagnose the token in/out split — Stage A discharged M07-IRL #2 (the projection bug). F2 RENDERS TestOutcome.token_spend.{input,output,total}; it does not touch the projection.</warning>
    <warning>DO NOT persist the TestOutcome to disk or to the live store on close — discard is the default; only the explicit Save/Promote affordance persists (spec Phase 9 "results discarded on close unless explicitly saved").</warning>
    <warning>DO NOT raise an HITL modal from the Tester — F1's test-defaults HitlSeam never prompts; capability violations arrive as TestOutcome.capability_failures and render as test-failure lines. If a HITL modal appears during a test run, the wiring is wrong.</warning>
  </execution_warnings>

  <time_box hours="8-11"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the pinned test_framework params + the TestOutcome serde shape (and confirm the TS mirror matches it field-for-field). State HOW the test-session graph is scoped (store-factory instance vs scoped reducer) and confirm the live useGraphStore singleton is untouched by a test run (cite the green test). Confirm discard-on-close + the explicit Save/Promote path. MVP §M8 criterion 5 Playwright result. Note any drift from the F1-shipped wire vs the F1.3.1 illustrative names.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="F2.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls M08.*-retrospective.md)</item>
    <item>strict-TDD invariant: git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**' EMPTY</item>
    <item>wire_signature_audit reconciliation against the shipped Stage F1 test_framework wire (params + TestOutcome shape)</item>
    <item>the scoped test-session graph (the live useGraphStore singleton untouched — cite the green test); discard-on-close</item>
    <item>MVP §M8 criterion 5 Playwright demonstration (Test → task → modal → graph + VDR + token + pass/fail); renderer ≥80</item>
    <item>every-class-has-a-CSS-rule confirmation (the M04.F static check)</item>
    <item>gate results (v1.6 order; renderer ≥80; Playwright + warmup; CI-parity per G6)</item>
    <item>M08.F2 retrospective [END]; explicit "Stage M08.F2 is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### F2.6 Commit Message

```
feat(renderer): M08 Stage F2 — Tester modal (task → sandboxed run → graph + VDR + pass/fail)

The Tester modal: opens on builderStore's Tester-open state (Stage E's
Test button + openTester), takes a natural-language task, calls Stage
F1's test_framework with the candidate framework (from the canvas — no
save), renders the test run in a smaller graph pane, surfaces
VDR/signals + token/timing + the pass/fail trace, and discards the
run on close.

- src/components/builder/TesterModal.tsx (new): the modal — task
  input + Run, the result surfaces (VDR, token in/out/total + timing,
  pass/fail with capability failures as test-failure lines, NOT HITL
  prompts), discard-on-close + an explicit Save/Promote affordance.
- src/components/builder/TesterGraphPane.tsx (new): the smaller graph
  pane — reuses the React-Flow rendering + the 11-entry nodeTypes map,
  in an instance SCOPED to the test session so the live useGraphStore
  module singleton is untouched (a test run never corrupts the runtime
  graph).
- src/lib/ipc.ts: the testFramework wrapper + the TestOutcome TS type
  (hand-mirrored to the Stage F1 serde shape — TestOutcome crosses the
  Tauri bridge as-is, the McpTool/McpServerSummary precedent).
- src/lib/builderStore.ts: the Tester-open state + the scoped
  test-session graph instance.
- src/components/builder/BuilderShell.tsx: mounts TesterModal.
- src/styles.css: .tester-modal + .tester-graph-pane +
  .tester-result--pass/--fail + the result-surface classes; every
  className paired with a CSS rule (gotcha #67).

Capability violations arrive as TestOutcome.capability_failures and
render as test failures (F1's test-defaults HitlSeam never prompts).
Discard-on-close is the default; the explicit Save/Promote affordance
is the only persist path (spec Phase 9). MVP §M8 criterion 5.

v1.8 wire_signature_audit pinned to the shipped Stage F1 wire.
Playwright + vitest; renderer ≥80. Strict two-commit TDD:
'**/tests/**' diff EMPTY.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---
## Stage G — Settings panel + Novice↔Promoted tier promotion

### G.1 Problem Statement

The post-M07.5 gate-7 IRL walk-through (`docs/M07-irl-findings.md` #5 — a 🔴-candidate) found: there is **no UI anywhere to promote Novice → Promoted** — no settings surface, no tier control; the only tier indicator (a node `N` badge) is not interactive. The backend `request_tier_transition` command **exists and is unit-tested** (`src-tauri/src/commands.rs:542` + the `_with` seam at `:568`); `graphStore.ts` already carries a `currentTier` slot (`src/lib/graphStore.ts:513`) and already reduces the `tier_transition` event (`:1543`); the slot's own doc-comment names a "Settings panel" that is *supposed* to host promotion — but no Settings panel ships and there is no `ipc.ts` wrapper for the command. The consequence: a v0.1 user is **permanently Novice**, so MCP-server management (all of M06) is unreachable and the **Promoted tier — a §0d v0.1-scope capability — cannot be reached at all**.

Stage A dispositioned this to Stage G (the maintainer directed #5 into M08, overriding the MVP doc's M10 tier-toggle slotting — A.3.6). G ships the missing **Settings panel** with the **Novice ↔ Promoted tier control** over the existing backend command — closing #5 — and absorbs **M06.5 IRL 🟡-4** (budget settings not state-wired). G is a renderer-surface stage over a working backend; it touches the tier surface (CODEOWNERS-adjacent — it *surfaces* `request_tier_transition`, it does **not** modify tier evaluation or enforcement; Hard Rule 8). The Settings panel is the M08 finale's user-visible payoff: once it ships, the Promoted tier — and therefore MCP-server install, the M06 Add modal, and the post-M08 IRL re-confirm of M06.5 🔴-1 — all become reachable for the first time.

Concrete deliverables:

1. **`src/components/SettingsPanel.tsx`** — a new focused settings surface. Sub-components `TierControl` (current-tier display + Novice↔Promoted control) and `BudgetControl` (the budget-cap input wired to state + persistence). Mounted at `App.tsx` **top level** as cross-mode chrome — a sibling of `BudgetHeaderBar` / `ViewSwitch`, outside the Runtime↔Builder `view` conditional, so the tier control is reachable from both modes (C.3.2 specifies the Settings panel as cross-mode). No Settings-tab infrastructure — the M06.E "no routing chrome" rule still holds; see G.3.3.
2. **`src/lib/ipc.ts`** — a `requestTierTransition(targetTier, reason)` wrapper around the existing `request_tier_transition` Tauri command, params PINNED to the shipped signature (`target_tier: Tier`, `reason: String`) via `<wire_signature_audit>`. `invokeSetGlobalBudget(usdCap)` already exists (`ipc.ts:165`) — G reuses it, adds no budget command.
3. **`src/lib/graphStore.ts`** — a `globalBudgetCap: number` slot + a `setGlobalBudgetCap(cap)` action so the budget-cap input reflects + persists its value (M06.5 IRL 🟡-4). `currentTier` + the `tier_transition` reducer already exist — the panel only *reads* them; G adds **no** tier reducer logic.
4. **`src/App.tsx`** — mount `<SettingsPanel />` at App top level, outside the `view` conditional — cross-mode chrome, a sibling of `BudgetHeaderBar` / `ViewSwitch` (Stage C's `RuntimeLayout` extraction made `.graph-layout` Runtime-only).
5. **`src/styles.css`** — `.settings-panel` + descendant classes, theme-variable-driven (`--node-fg`, `--node-fg-muted`, `--node-bg`), each paired with a CSS rule per gotcha #67.

Not in this stage:

- **The Operator tier.** §0d locks the v0.1 tier system to **Novice + Promoted**; Operator is v1.0. The `TierControl` offers Novice↔Promoted only — no Operator option appears anywhere in the rendered DOM.
- **API-key entry.** The key UI stays in the existing `SetupPanel` (`src/components/SetupPanel.tsx`); Stage A fixed key persistence across restart (M07-IRL #7). G's Settings panel is a *focused* surface (tier + budget), **not** a catch-all that absorbs `SetupPanel`.
- **Tier evaluation / enforcement changes.** `request_tier_transition` already enforces §8.security L4 (promotion is renderer-authoritative, demotion is direct — see the command's doc-comment). G surfaces it; G does **not** touch `crates/runtime-main/src/tier/`.
- **Budget *primitive* work.** The four budget actions (`budget_warn` / `budget_downshift` / `budget_suspended` / `budget_exceeded`) shipped at M04; the `BudgetState` slot + `BudgetHeaderBar` already exist. G wires only the **settings-surface cap input** 🟡-4 flagged — it does not re-touch the budget engine.
- **MCP-panel changes.** Making Promoted reachable *unblocks* the (correctly tier-blocked) MCP Add modal — G does not rebuild `MCPServerSettings.tsx` / `MCPServerAddModal.tsx`; the unblock is a free consequence of the tier becoming reachable.

### G.2 Files to Change

| File | Status | Change |
|---|---|---|
| `src/components/SettingsPanel.tsx` | **new** | The Settings panel — `TierControl` (current-tier display + Novice↔Promoted control) + `BudgetControl` (budget-cap input). Focused surface; not a catch-all. |
| `src/lib/ipc.ts` | exists | Add `requestTierTransition(targetTier, reason)` — params PINNED to the existing `request_tier_transition` command via `<wire_signature_audit>`. `invokeSetGlobalBudget` already present — reused, not re-added. |
| `src/lib/graphStore.ts` | exists | Add a `globalBudgetCap: number` slot + `setGlobalBudgetCap` action (M06.5 IRL 🟡-4 state-wiring). `currentTier` + the `tier_transition` reducer already exist — read-only here. |
| `src/App.tsx` | exists | Mount `<SettingsPanel />` at App top level, outside the `view` conditional — cross-mode chrome alongside `BudgetHeaderBar` / `ViewSwitch` (not inside `.graph-layout`, which Stage C made Runtime-only). |
| `src/styles.css` | exists | `.settings-panel` + descendant classes (theme variables); every className paired with a CSS rule (gotcha #67). |
| `tests/unit/components/SettingsPanel.test.tsx` | **new** | Vitest — panel render, `TierControl`, `BudgetControl` state wiring. |
| `tests/e2e/settings_tier_promotion.spec.ts` | **new** | Playwright — open Settings → promote → tier updates; Operator absent; budget cap reflects+persists. |
| `CHANGELOG.md` | exists | `[Unreleased]` Stage G entry. |
| `docs/build-prompts/retrospectives/M08.G-retrospective.md` | **new** | Stage G retrospective. |

Effort budget: ~6–9 hours. Renderer-only over a working backend; the largest piece is the panel + its two controls + the Playwright tier-transition behavior test. No Rust changes, no schema changes — `request_tier_transition` and `set_global_budget` both predate M08. Pattern locks from M06.E (sibling-panel mount, Settings-section layout) carry forward cleanly.

### G.3 Detailed Changes

#### G.3.1 Wire pinning (v1.8 discipline — BEFORE pseudocode)

The v1.8 `<wire_signature_audit>` + `<phase_doc_inventory_audit shape=…>` slots are run **first** — the M06.E / M07.E lesson is that a renderer stage that authors component pseudocode before pinning the actual Tauri command params drifts (M07's `importArtifact` phase-doc assumed `{ src, kind }`; the shipped command took three flat camelCased args — `ipc.ts:262` documents the catch). Stage G surfaces a command that **predates M08 by three milestones** (`request_tier_transition` is M05 Stage D), so the pin is mandatory, not advisory.

The shipped `request_tier_transition` signature (`src-tauri/src/commands.rs:542`), verbatim:

```rust
#[tauri::command]
pub async fn request_tier_transition(
    app: AppHandle,
    target_tier: Tier,                              // Tier = "novice" | "promoted" (serde enum)
    reason: String,
    state: tauri::State<'_, CurrentTierState>,
) -> Result<(), CmdError> { /* … delegates to request_tier_transition_with */ }
```

`app` + `state` are Tauri-injected and do **not** cross the IPC bridge. The two renderer-supplied args are `target_tier` and `reason`; Tauri auto-converts the snake_case Rust `target_tier` to the camelCase JS key `targetTier`. The Rust `Tier` enum (`crates/runtime-main/src/tier/evaluator.rs:24`) serializes to `"novice" | "promoted"` — **exactly** the generated `TierRef` type (`src/types/agent_event.ts:157: export type TierRef = "novice" | "promoted"`), so the wrapper reuses `TierRef`, no new type. The command is **idempotent** when target == current (`commands.rs:590` returns `Ok` for the no-op) — the renderer may call it freely without a pre-check.

The `set_global_budget` wrapper already exists and is correct:

```ts
// src/lib/ipc.ts:165 — SHIPPED, reused verbatim by G; do NOT re-add.
export async function invokeSetGlobalBudget(usdCap: number): Promise<void> {
  await invoke('set_global_budget', { usdCap });   // 0 disables the cap
}
```

The `graphStore` slots G reads (`<phase_doc_inventory_audit shape=…>`):

```ts
// src/lib/graphStore.ts — SHIPPED slots; G reads currentTier, ADDS globalBudgetCap.
currentTier: TierRef;            // :513 — 'novice' default; reduced by tier_transition (:1543). READ-ONLY in G.
budget: BudgetState | null;      // :485 — per-session spend snapshot from budget_* events. NOT the cap input.
```

`currentTier` is preserved across `clear()` (`graphStore.ts:1707` — installation/preference state, not per-session graph state). `budget` is the per-session **spend** snapshot driven by the four `budget_*` events; it is **not** the user-configured **cap**. M06.5 IRL 🟡-4 is specifically that the configured-cap *input* has no state slot — G adds `globalBudgetCap` as that slot (G.3.4). Do not conflate `budget.capUsd` (the cap the *running session* observed) with `globalBudgetCap` (the cap the user *configures* in Settings).

#### G.3.2 `requestTierTransition` — the `ipc.ts` wrapper

A typed wrapper following the established `ipc.ts` pattern (doc comment naming the spec section + the backend command + the wire-pin provenance; `invoke` with camelCase keys):

```ts
// src/lib/ipc.ts (addition) — params PINNED to request_tier_transition
// (src-tauri/src/commands.rs:542); this command predates M08 (M05 Stage D).

/**
 * Request a Novice ↔ Promoted tier transition (M05 Stage D — spec
 * §8.security L4). Wraps the EXISTING `request_tier_transition` Tauri
 * command — Stage G surfaces it, it does NOT reimplement tier logic.
 *
 * The backend persists the new tier to `<app_data_dir>/tier.json`,
 * updates its in-memory cache, and emits a `tier_transition` event on
 * the `agent_event` channel — which `graphStore.applyEvent` already
 * reduces into `currentTier` (graphStore.ts:1543). The Settings panel's
 * displayed tier therefore updates through the EXISTING event path; the
 * wrapper does not return the new tier and the caller does not set it.
 *
 * Idempotent when `targetTier` equals the current tier (the backend
 * returns `Ok` for the no-op — commands.rs:590), so the panel may call
 * freely without a pre-check.
 *
 * `targetTier` is the generated {@link TierRef} ('novice' | 'promoted')
 * — byte-identical to the Rust `Tier` enum's serde form. Operator is
 * NOT a `TierRef` member (v1.0, §0d). Errors surface as the Tauri
 * `CmdError` shape — render via {@link unwrapCmdError}.
 */
export async function requestTierTransition(
  targetTier: TierRef,
  reason: string,
): Promise<void> {
  await invoke('request_tier_transition', { targetTier, reason });
}
```

`TierRef` is imported from `../types/agent_event` (where it is already exported and already used by `ipc.ts`'s sibling type imports). No new type, no schema touch — the wrapper rides the generated `TierRef`. The wrapper is intentionally `Promise<void>`: the new tier arrives via the `tier_transition` event the backend emits, not via a return value — the Settings panel must not optimistically set `currentTier` itself (that would double-source the slot the reducer owns).

#### G.3.3 The Settings panel — `src/components/SettingsPanel.tsx`

A new focused settings surface — the component the `graphStore.currentTier` doc-comment names ("the renderer's Settings panel reads this") but no file ever delivered (M07-IRL #5 ground truth). It hosts `TierControl` (G.3.3) and `BudgetControl` (G.3.4). **It is not a catch-all**: the Anthropic-API-key entry stays in the existing `SetupPanel` (Stage A fixed key persistence — M07-IRL #7; G does not absorb the key UI).

Mounting honors **two** prior decisions. First, the **M06.E "no routing chrome" rule**: v0.1 has no Settings-tab / routing infrastructure (`MCPServerSettings.tsx:12-15` documents this, reconciled against the actual DOM after the M06.E phase doc's `[data-test=open-settings]` tab pseudocode drifted) — `SettingsPanel` is a flat-mounted component, **not** a tab; do not invent a tab system. Second, **C.3.2's cross-mode requirement**: Stage C extracted the `.graph-layout` JSX into `RuntimeLayout` (rendered only when `view === 'runtime'`), and C.3.2 explicitly states G's Settings panel is cross-mode (*"budget settings live in G's Settings panel, also cross-mode"*). Mounting `SettingsPanel` inside `.graph-layout` would therefore make it **Runtime-only** — and the tier control would be unreachable from the Builder, defeating finding #5 (a user composing a framework in the Builder still could not promote). So `SettingsPanel` mounts at `App.tsx` **top level**, outside the `view` conditional, as a sibling of `BudgetHeaderBar` and `ViewSwitch` — the same cross-mode app-chrome slot `BudgetHeaderBar` already occupies (App-level chrome is not a tab system). It is **not** a `.graph-layout` sibling and **not** mounted inside `BuilderShell`.

```tsx
// src/components/SettingsPanel.tsx
import { useState } from 'react';
import { useGraphStore } from '../lib/graphStore';
import { requestTierTransition, invokeSetGlobalBudget, unwrapCmdError } from '../lib/ipc';
import type { TierRef } from '../types/agent_event';

/**
 * Settings panel (M08 Stage G). A focused settings surface hosting the
 * Novice↔Promoted tier control + the global-budget-cap control. Closes
 * M07-IRL #5 (no tier-promotion UI → the Promoted tier was unreachable)
 * and M06.5 IRL 🟡-4 (budget settings not state-wired).
 *
 * NOT a catch-all: the Anthropic API key stays in SetupPanel. Operator
 * tier is NOT surfaced (v1.0 — §0d locks v0.1 to Novice + Promoted).
 *
 * Mounted at App.tsx top level as cross-mode chrome — outside the
 * Runtime↔Builder view conditional (C.3.2), so the tier control is
 * reachable in both modes. v0.1 has no Settings-tab infrastructure
 * (the M06.E no-routing rule).
 */
export function SettingsPanel(): JSX.Element {
  return (
    <section className="settings-panel" data-testid="settings-panel">
      <header className="settings-panel__header">
        <h2 className="settings-panel__title">Settings</h2>
      </header>
      <TierControl />
      <BudgetControl />
    </section>
  );
}

/**
 * Current-tier display + the Novice↔Promoted transition control. Reads
 * `currentTier` from the store (the EXISTING slot, reduced by the
 * EXISTING `tier_transition` branch — graphStore.ts:513/:1543). The
 * control calls the EXISTING `request_tier_transition` backend command
 * via `requestTierTransition`; Stage G does NOT reimplement tier logic
 * (Hard Rule 8). The displayed tier updates when the backend's
 * `tier_transition` event flows through the existing reducer.
 */
function TierControl(): JSX.Element {
  const tier = useGraphStore((s) => s.currentTier);     // existing slot — READ-ONLY
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Novice → Promoted is the promotion; Promoted → Novice the demotion.
  // Operator is NOT a target — TierRef has only 'novice' | 'promoted'.
  const target: TierRef = tier === 'novice' ? 'promoted' : 'novice';
  const actionLabel = tier === 'novice' ? 'Promote to Promoted' : 'Demote to Novice';

  async function handleTransition(): Promise<void> {
    setPending(true);
    setError(null);
    try {
      // Promotion reason is a fixed string — the backend's `reason`
      // param is for the audit/event record, not user-entered here.
      await requestTierTransition(target, `user requested ${target} via Settings`);
      // NB: do NOT setState currentTier here — the backend's
      // tier_transition event updates it through the existing reducer.
    } catch (e) {
      setError(unwrapCmdError(e));
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="settings-panel__section settings-panel__section--tier" data-testid="tier-control">
      <h3 className="settings-panel__section-title">Capability tier</h3>
      <p className="settings-panel__tier-current" data-testid="tier-current">
        Current tier: <span className={`settings-panel__tier-value settings-panel__tier-value--${tier}`}>{tier}</span>
      </p>
      <p className="settings-panel__tier-explainer">
        {tier === 'novice'
          ? 'Novice restricts capabilities to safe defaults. Promote to enable MCP-server management and broader tool access.'
          : 'Promoted enables MCP-server management and broader tool access. Demote to return to Novice safe defaults.'}
      </p>
      <button
        className="settings-panel__tier-button"
        data-testid="tier-transition-button"
        disabled={pending}
        onClick={() => void handleTransition()}
      >
        {pending ? 'Applying…' : actionLabel}
      </button>
      {error !== null && (
        <p className="settings-panel__error" data-testid="tier-error">
          {error}
        </p>
      )}
    </div>
  );
}

/**
 * Global per-day budget-cap control (M06.5 IRL 🟡-4 state-wiring). Reads
 * the configured cap from the store's `globalBudgetCap` slot and
 * persists changes via the EXISTING `invokeSetGlobalBudget` command
 * (ipc.ts:165). The input REFLECTS the live slot value — the 🟡-4
 * complaint was that it did not. `0` disables the cap.
 *
 * Distinct from `graphStore.budget` (the per-session SPEND snapshot from
 * budget_* events) — this is the user-CONFIGURED cap. The budget
 * PRIMITIVE shipped at M04; G wires only the settings-surface input.
 */
function BudgetControl(): JSX.Element {
  const cap = useGraphStore((s) => s.globalBudgetCap);          // new slot — G.3.4
  const setCap = useGraphStore((s) => s.setGlobalBudgetCap);    // new action — G.3.4
  const [draft, setDraft] = useState<string>(String(cap));
  const [error, setError] = useState<string | null>(null);

  async function handleSave(): Promise<void> {
    const parsed = Number(draft);
    if (!Number.isFinite(parsed) || parsed < 0) {
      setError('Enter a non-negative dollar amount (0 disables the cap).');
      return;
    }
    setError(null);
    try {
      await invokeSetGlobalBudget(parsed);   // EXISTING command — persists the cap
      setCap(parsed);                        // mirror into the store so the input reflects it
    } catch (e) {
      setError(unwrapCmdError(e));
    }
  }

  return (
    <div className="settings-panel__section settings-panel__section--budget" data-testid="budget-control">
      <h3 className="settings-panel__section-title">Daily budget cap (USD)</h3>
      <label className="settings-panel__budget-label">
        Cap:{' '}
        <input
          className="settings-panel__budget-input"
          data-testid="budget-cap-input"
          type="number"
          min={0}
          step="0.01"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
        />
      </label>{' '}
      <button
        className="settings-panel__budget-button"
        data-testid="budget-save-button"
        onClick={() => void handleSave()}
      >
        Save cap
      </button>
      <p className="settings-panel__budget-hint">Set 0 to disable the cap.</p>
      {error !== null && (
        <p className="settings-panel__error" data-testid="budget-error">
          {error}
        </p>
      )}
    </div>
  );
}
```

Why this shape: `TierControl` derives the transition `target` purely from the current tier (`'novice' → 'promoted'`, `'promoted' → 'novice'`) — there is no third branch because `TierRef` has no third member, which is the type-level enforcement of "Operator is not surfaced." The control never sets `currentTier` itself — the backend's `tier_transition` event is the single writer, reduced by the existing branch; an optimistic local set would double-source the slot. `BudgetControl` keeps a `draft` string (the input's working value) separate from the committed `globalBudgetCap` slot — the input *reflects* the slot on mount and on each successful save, which is precisely the 🟡-4 fix (the old input did not reflect/persist anything).

#### G.3.4 Budget settings state-wiring — `graphStore.ts` slot + action (closes M06.5 IRL 🟡-4)

M06.5 IRL 🟡-4: the budget settings are not state-wired — the budget-cap input does not reflect or persist the configured cap. The cause is structural: `graphStore` has a per-session `budget` *spend* slot but **no slot for the user-configured cap**. G adds one:

```ts
// src/lib/graphStore.ts — additions to the store state interface + initial state.

  /**
   * Spec §2a (M08 Stage G — M06.5 IRL 🟡-4): the user-configured global
   * per-day budget cap in USD, set via the Settings panel. `0` means no
   * cap. Distinct from `budget` (the per-session SPEND snapshot from
   * budget_* events) — this is the CONFIGURED cap. Persisted to the
   * runtime via `set_global_budget`; held here so the Settings panel's
   * input reflects the live value. Preserved across `clear()` (a user
   * preference, like `currentTier`, not per-session graph state).
   */
  globalBudgetCap: number;

  /** Set the configured global budget cap (M08 Stage G). */
  setGlobalBudgetCap: (cap: number) => void;
```

```ts
// initial state (alongside `budget: null,` / `currentTier: 'novice',`):
  globalBudgetCap: 0,            // 0 = no cap until the user configures one

// action (alongside the other store actions):
  setGlobalBudgetCap: (cap: number): void => {
    set({ globalBudgetCap: cap });
  },
```

```ts
// in `clear()` — globalBudgetCap is a preference, preserved like currentTier:
  globalBudgetCap: state.globalBudgetCap,
```

Why a store slot (not component-local state): the IRL 🟡-4 complaint is that the value does not *reflect* — i.e., re-opening the panel shows a stale/empty input. A component-local `useState` would reset on unmount; the store slot survives panel open/close (and `clear()`) so the input reflects the last-configured cap. The v0.1 budget command (`set_global_budget`) holds the cap in process memory only (`ipc.ts:165` doc-comment — "M10 first-run UX persists it"); G's slot mirrors that process-memory value into the renderer so the input is consistent within a session. Cross-restart persistence remains M10's job and is **out of scope** here — 🟡-4 is the *state-wiring*, not disk persistence.

#### G.3.5 App mount + styles

`App.tsx` mounts `<SettingsPanel />` at top level, outside the `view` conditional — cross-mode chrome alongside `BudgetHeaderBar` / `ViewSwitch`. Stage C's `RuntimeLayout` extraction moved `.graph-layout` out of `App.tsx` and behind the `view === 'runtime'` branch; mounting `SettingsPanel` there would make it Runtime-only, which C.3.2 explicitly rules out:

```tsx
// src/App.tsx — SettingsPanel is App-level chrome, OUTSIDE the view
// conditional (C.3.2's structure), so the tier control renders in both
// Runtime and Builder modes. The view conditional itself is unchanged.
<main>
  <BudgetHeaderBar />
  <h1>Agent Runtime</h1>
  <ViewSwitch value={view} onChange={setView} />
  <SettingsPanel />     {/* M08 Stage G — cross-mode chrome */}
  {view === 'runtime' ? (
    <RuntimeLayout
      hasKey={hasKey}
      running={running}
      error={error}
      onSetKey={handleSetKey}
      onSmoke={handleSmoke}
      lastSessionId={lastSessionId}
    />
  ) : (
    <BuilderShell />
  )}
</main>
```

`src/styles.css` gains `.settings-panel` + every descendant class used above (`.settings-panel__header`, `__title`, `__section`, `__section-title`, `__tier-current`, `__tier-value` + `--novice` / `--promoted` variants, `__tier-explainer`, `__tier-button`, `__error`, `__budget-label`, `__budget-input`, `__budget-button`, `__budget-hint`). All colours reference the existing theme variables (`--node-fg` primary text, `--node-fg-muted` secondary, `--node-bg` panel background) — the M07-IRL #3 lesson (Stage A fixed import-panel contrast caused by hard-coded colours; G must not reintroduce the bug). Per gotcha #67, every className above gets a corresponding CSS rule, and the styles test (G.4.2) asserts it via the `every_class_has_a_corresponding_CSS_rule` pattern.

### G.4 Tests

Strict v1.8 two-commit TDD (`<tdd_discipline strict="true">`): all failing tests land in a standalone `test(M08.G): …` red commit, the impl commit leaves `**/tests/**` byte-identical (`git diff <red>..<impl> -- '**/tests/**'` EMPTY). Renderer-only — vitest + Playwright; the renderer ≥80 gate holds.

#### G.4.1 SettingsPanel vitest — `tests/unit/components/SettingsPanel.test.tsx`

Panel structure + the two controls' state wiring. `requestTierTransition` and `invokeSetGlobalBudget` are module-mocked (`vi.mock('../../src/lib/ipc')`); `useGraphStore` is reset in `beforeEach` (`currentTier` + the new `globalBudgetCap` slot — per the `<test_isolation_audit>` persistent-slot discipline).

- `renders_settings_panel_with_tier_and_budget_sections`
- `tier_control_displays_current_tier_from_store` — store `currentTier: 'novice'` → "Current tier: novice"
- `tier_control_button_label_is_promote_when_novice`
- `tier_control_button_label_is_demote_when_promoted` — store `currentTier: 'promoted'` → "Demote to Novice"
- `clicking_promote_calls_requestTierTransition_with_promoted` — asserts the mock received `('promoted', <reason string>)`
- `clicking_demote_calls_requestTierTransition_with_novice`
- `tier_control_does_not_optimistically_set_currentTier` — after the click, `currentTier` in the store is unchanged (the reducer, not the component, owns the write)
- `tier_control_never_renders_an_operator_option` — the rendered DOM contains no "operator" text and no third tier control (the §0d lock, asserted as a behavior)
- `tier_transition_error_surfaces_via_unwrapCmdError` — mock rejects → `[data-testid=tier-error]` shows the unwrapped message
- `budget_control_input_reflects_globalBudgetCap_slot` — store `globalBudgetCap: 25` → input value is `"25"`
- `clicking_save_cap_calls_invokeSetGlobalBudget_with_parsed_value` — input `"40"` → mock received `40`
- `saving_cap_updates_globalBudgetCap_slot_so_input_reflects_it` — the 🟡-4 contract: after save, the slot is `40` and a re-render shows `"40"`
- `budget_control_rejects_negative_input_without_calling_command` — input `"-5"` → no mock call, `[data-testid=budget-error]` shown
- `budget_save_command_error_surfaces_via_unwrapCmdError`

#### G.4.2 Styles contract — `tests/unit/components/SettingsPanel.test.tsx` (styles block)

Per gotcha #67, every `settings-panel*` className rendered by the component has a corresponding rule in `src/styles.css`, asserted via the `every_class_has_a_corresponding_CSS_rule` pattern (the M04.F / M06.E precedent):

- `every_settings_panel_class_has_a_corresponding_css_rule`

#### G.4.3 Playwright behavior test — `tests/e2e/settings_tier_promotion.spec.ts`

Drives the renderer against the Vite dev server with `@tauri-apps/api` module-mocked (gotcha #23 — Playwright cannot drive the Tauri window); `request_tier_transition` and `set_global_budget` are mocked at the `invoke` boundary; `tier_transition` events are scripted through `window.__graphStore` (gotcha #54 — the established renderer-test state-injection affordance).

```ts
// tests/e2e/settings_tier_promotion.spec.ts
test.describe.configure({ timeout: 90_000 }); // gotcha #53 — Vite cold-start

test('promoting via Settings updates the displayed tier through the tier_transition reducer', async ({ page }) => {
  await page.goto('/');

  // Settings panel shows the first-run tier.
  await expect(page.locator('[data-testid=tier-current]')).toContainText('novice');

  // Promote — the mocked request_tier_transition resolves; the renderer
  // must NOT optimistically flip the tier (the backend event does).
  await page.click('[data-testid=tier-transition-button]');

  // Script the backend's tier_transition event (what the real command emits).
  await page.evaluate(() =>
    window.__graphStore.getState().applyEvent({
      type: 'tier_transition',
      previous: 'novice',
      current: 'promoted',
      reason: 'user requested promoted via Settings',
    }),
  );

  // The displayed tier updates through the EXISTING reducer.
  await expect(page.locator('[data-testid=tier-current]')).toContainText('promoted');
  await expect(page.locator('[data-testid=tier-transition-button]')).toContainText('Demote');
});

test('Settings tier control never offers an Operator option', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('[data-testid=settings-panel]')).not.toContainText(/operator/i);
});

test('budget cap input reflects and persists a set value', async ({ page }) => {
  await page.goto('/');
  await page.fill('[data-testid=budget-cap-input]', '30');
  await page.click('[data-testid=budget-save-button]');
  // The input reflects the persisted value (M06.5 IRL 🟡-4 — it previously did not).
  await expect(page.locator('[data-testid=budget-cap-input]')).toHaveValue('30');
});

test('the Settings panel is reachable in Builder mode (cross-mode chrome)', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Builder' }).click(); // C's view switch
  // SettingsPanel mounts OUTSIDE the view conditional — switching to
  // Builder mode does not unmount it; the tier control stays reachable
  // (C.3.2 — defeating finding #5 would mean a Builder user is stuck Novice).
  await expect(page.locator('[data-testid=settings-panel]')).toBeVisible();
  await expect(page.locator('[data-testid=tier-current]')).toBeVisible();
});
```

#### G.4.4 Schema regen check

- N/A — Stage G ships no schema changes. `request_tier_transition` and `set_global_budget` predate M08; `TierRef` is already generated.

#### G.4.5 Acceptance criteria

- [ ] `npm run test` — all vitest tests pass; renderer coverage ≥80% on `src/components/SettingsPanel.tsx`
- [ ] `npx tsc --noEmit` — clean
- [ ] `npx eslint .` — clean
- [ ] `npx prettier --check '**/*.{ts,tsx,js,jsx,json}'` — clean
- [ ] `npm run test:e2e -- settings_tier_promotion.spec.ts` — passes against the Vite dev server (curl warmup probe per the `<playwright_warmup_recipe>`)
- [ ] **M07-IRL #5 closed** — a Settings panel exists; the Novice↔Promoted control calls the **existing** `request_tier_transition`; the displayed tier updates through the **existing** `tier_transition` reducer (no new tier logic)
- [ ] **Cross-mode mount** — `SettingsPanel` mounts at `App.tsx` top level outside the `view` conditional (a sibling of `BudgetHeaderBar` / `ViewSwitch`); the tier control is reachable in **both** Runtime and Builder modes (C.3.2), Playwright-asserted
- [ ] **Operator is not surfaced** anywhere in the rendered DOM (§0d — v0.1 is Novice + Promoted)
- [ ] **M06.5 IRL 🟡-4 closed** — the budget-cap input reflects the `globalBudgetCap` slot and persists changes via `invokeSetGlobalBudget`
- [ ] `<wire_signature_audit>` reconciles the `requestTierTransition` wrapper against the shipped `request_tier_transition` signature (`commands.rs:542`)
- [ ] Every new `settings-panel*` className has a corresponding rule in `src/styles.css` (gotcha #67)
- [ ] `tests/unit/components/SettingsPanel.test.tsx` resets `currentTier` + `globalBudgetCap` in `beforeEach` (`<test_isolation_audit>`)
- [ ] Strict v1.8 two-commit: `git diff <red>..<impl> -- '**/tests/**'` EMPTY
- [ ] CI-parity per the project gate list

### G.5 CLI Prompt

```xml
<work_stage_prompt id="M08.G">
  <context>
    M08 Stage G — the Settings panel + Novice↔Promoted tier promotion,
    closing the M07-IRL #5 🔴-candidate (no tier-promotion UI → the
    Promoted tier, a §0d v0.1-scope capability, is unreachable; MCP
    management unreachable). The backend request_tier_transition command
    EXISTS + is unit-tested (commands.rs:542 + the _with seam at :568);
    graphStore already carries the currentTier slot + reduces the
    tier_transition event. G ships the missing RENDERER surface — the
    Settings panel + the Novice↔Promoted control + the requestTierTransition
    ipc.ts wrapper — it does NOT reimplement tier-transition or
    enforcement logic (Hard Rule 8). Also closes M06.5 IRL 🟡-4 (budget
    settings state-wiring) by adding a globalBudgetCap store slot wired to
    the EXISTING set_global_budget command. Novice↔Promoted ONLY —
    Operator is v1.0 (§0d). Renderer-only stage over a working backend;
    no Rust changes, no schema changes. Pin the EXISTING command
    signatures BEFORE authoring component pseudocode.
  </context>

  <read_first>
    <file>CLAUDE.md (§10 capability adherence — G surfaces, does NOT modify, the tier surface; Hard Rule 8)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.8 — wire_signature_audit + phase_doc_inventory_audit shape=)</file>
    <file>docs/build-prompts/M08-workbench.md (Background, Key constraints, Stage A A.3.6 + A.3.7, Stage C C.3.2 — the view switch + the cross-mode chrome decision, Stage G G.1–G.4)</file>
    <file>docs/M07-irl-findings.md (#5 — the ground truth: request_tier_transition exists, the Settings panel was never built)</file>
    <file>agent-runtime-spec.md §8.security L4 (the tier system — promotion is renderer-authoritative, demotion direct) + §0d (v0.1 = Novice + Promoted; Operator is v1.0) + §2a (budget)</file>
    <file>docs/build-prompts/retrospectives/M08.F2-retrospective.md (the immediately prior stage's [END] Decisions — apply them)</file>
    <file>docs/build-prompts/retrospectives/M08.A-retrospective.md (Stage A's #5 → Stage G disposition + 🟡-4 routing)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/gotchas.md (#23 Playwright/Tauri module-mock; #53 Vite cold-start; #54 window.__graphStore; #67 component+CSS contract; #30 unwrapCmdError; #75 useShallow)</file>
  </read_first>

  <read_reference>
    <file purpose="The EXISTING request_tier_transition command at :542 + the _with seam at :568 + set_global_budget — PIN both verbatim; do NOT assume the shape">src-tauri/src/commands.rs</file>
    <file purpose="ipc.ts wrapper pattern (doc-comment + invoke camelCase keys); invokeSetGlobalBudget already at :165 — reuse, do NOT re-add; the importArtifact :262 comment documents the wire-drift lesson">src/lib/ipc.ts</file>
    <file purpose="currentTier slot at :513 + the tier_transition reducer branch at :1543 + the clear() preservation at :1707 — G READS currentTier, ADDS globalBudgetCap; do NOT add tier reducer logic">src/lib/graphStore.ts</file>
    <file purpose="TierRef = 'novice' | 'promoted' at :157 — the generated type the wrapper reuses; Operator is NOT a member">src/types/agent_event.ts</file>
    <file purpose="The Rust Tier enum at :24 — serde form 'novice' | 'promoted', byte-identical to TierRef">crates/runtime-main/src/tier/evaluator.rs</file>
    <file purpose="The EXISTING key-entry panel — the Settings panel is FOCUSED, it does NOT absorb the key UI">src/components/SetupPanel.tsx</file>
    <file purpose="M06.E 'no routing chrome' pattern + the documented no-Settings-tab DOM reconciliation (:12-15) — SettingsPanel is a flat-mounted component, NOT a tab. NOTE MCPServerSettings itself sits in .graph-layout (Runtime-only post-C); SettingsPanel is cross-mode and mounts at App top level instead (see G.3.3)">src/components/MCPServerSettings.tsx</file>
    <file purpose="App.tsx — Stage C's RuntimeLayout extraction + the Runtime↔Builder view switch; SettingsPanel mounts at App TOP LEVEL outside the view conditional (cross-mode chrome, a sibling of BudgetHeaderBar / ViewSwitch — NOT inside .graph-layout, which C made Runtime-only)">src/App.tsx</file>
    <file purpose="The every_class_has_a_corresponding_CSS_rule styles-contract precedent (gotcha #67)">tests/unit/components/MCPServerSettings.test.tsx</file>
  </read_reference>

  <read_prior_stages>
    <stage id="M08.F2" decisions_file="docs/build-prompts/retrospectives/M08.F2-retrospective.md"/>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M08-workbench.md" section="G.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <tdd_discipline strict="true">
    <red_phase>
      Write all failing tests across the G.4 buckets — vitest
      (SettingsPanel render, TierControl state wiring, BudgetControl
      state wiring, the styles contract) + Playwright
      (settings_tier_promotion.spec.ts — the Settings panel renders →
      current tier shows; the panel is reachable in Builder mode too;
      Promote → requestTierTransition('promoted', …) called; a
      scripted tier_transition event → the displayed tier updates;
      Operator never offered; the budget cap reflects + persists). Stub
      the SettingsPanel surface just enough to compile (the goal is
      link-time test discovery, not behavior). Confirm right-reason red
      per CLAUDE.md §5 (cannot find name 'SettingsPanel' / assertion
      failed — NOT a test-file compile error, NOT a tautological pass).
      Commit as a STANDALONE `test(M08.G): failing tests for Settings
      panel + tier promotion` commit on the M08 branch BEFORE the
      green-phase impl; the commit body pastes the first ~40 lines of
      the vitest run proving the expected-failure class. Surface the
      red-phase commit; the user approves before green begins.
    </red_phase>
    <green_phase>
      Implement until ALL failing tests pass. Do NOT modify the test
      files during implementation — if a test is wrong, fix it in a
      SEPARATE labelled follow-up commit with explanation, never
      silently in the impl commit. The impl commit body MUST state the
      verifiable invariant: `git diff &lt;red-sha&gt;..&lt;impl-sha&gt;
      -- '**/tests/**'` is EMPTY. Net-new additive tests + mechanical
      prettier/eslint fixes to test files go in the separate follow-up
      commit. No Co-Authored-By in any commit message.
    </green_phase>
  </tdd_discipline>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="red_phase_commit" budget="1"/>
    <step name="surface_for_red_approval"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="green_phase_commit" budget="1"/>
    <step name="surface_for_final_approval"/>
    <step name="fill_retrospective"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M08-workbench.md" section="G.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>

  <gates milestone="M08"/>

  <coverage_gate>
    <gate scope="renderer" target_lines="80" ignore_filename_regex="generated"/>
  </coverage_gate>

  <self_correction_budget>3</self_correction_budget>

  <pre_flight_check>
    <check name="branch" gate="git rev-parse --abbrev-ref HEAD is the M08 parent-milestone branch; the M08.F2 impl commit is present in git log main..HEAD"/>
    <check name="view_switch_exists" gate="grep-confirm src/App.tsx has Stage C's view switch — the `view` state + RuntimeLayout + BuilderShell + ViewSwitch — so SettingsPanel can mount as a sibling of BudgetHeaderBar / ViewSwitch OUTSIDE the view conditional (NOT inside .graph-layout, which C moved into RuntimeLayout)"/>
    <check name="backend_exists" gate="grep-confirm `request_tier_transition` at src-tauri/src/commands.rs:542 (+ `request_tier_transition_with` at :568), the graphStore `currentTier` slot, and the `tier_transition` reducer branch ALL exist before wiring — G surfaces them, it does not create them"/>
    <check name="budget_command_exists" gate="grep-confirm `invokeSetGlobalBudget` exists in src/lib/ipc.ts (do NOT re-add it) and `set_global_budget` exists in commands.rs"/>
  </pre_flight_check>

  <phase_doc_inventory_audit>
    <claim type="ipc_command" path="src-tauri/src/commands.rs" symbol="request_tier_transition" verified="true" note="M05 Stage D; predates M08 — G surfaces it"/>
    <claim type="ipc_command" path="src-tauri/src/commands.rs" symbol="set_global_budget" verified="true" note="M04 Stage F; G reuses via the existing invokeSetGlobalBudget wrapper"/>
    <claim type="method" path="src/lib/ipc.ts" symbol="invokeSetGlobalBudget" verified="true" note="exists at :165 — reuse, do NOT re-add"/>
    <claim type="method" path="src/lib/ipc.ts" symbol="requestTierTransition" verified="false" note="Stage G adds this wrapper"/>
    <claim type="store_slot" path="src/lib/graphStore.ts" symbol="currentTier" shape="TierRef ('novice' | 'promoted')" verified="true" note="exists at :513; reduced by tier_transition at :1543; READ-ONLY in G"/>
    <claim type="store_slot" path="src/lib/graphStore.ts" symbol="globalBudgetCap" shape="number" verified="false" note="Stage G adds this slot + the setGlobalBudgetCap action (M06.5 IRL 🟡-4)"/>
    <claim type="file" path="src/components/SettingsPanel.tsx" verified="false" note="Stage G creates"/>
    <claim type="file" path="src/App.tsx" verified="true" note="Stage C shipped the view switch (RuntimeLayout / BuilderShell / ViewSwitch); G adds &lt;SettingsPanel/&gt; as cross-mode chrome OUTSIDE the view conditional (C.3.2)"/>
    <claim type="file" path="src/components/SetupPanel.tsx" verified="true" note="the key UI stays here — G does NOT absorb it"/>
    <claim type="read_first_target" path="docs/build-prompts/retrospectives/M08.F2-retrospective.md" verified="true"/>
  </phase_doc_inventory_audit>

  <wire_signature_audit>
    <wrapper ipc_command="request_tier_transition"
             actual_params="target_tier: Tier (serde 'novice' | 'promoted'), reason: String — verbatim from src-tauri/src/commands.rs:542; `app` + `state` are Tauri-injected and do NOT cross the bridge; Tauri auto-camelCases target_tier → targetTier"
             phase_doc_assumed="requestTierTransition(targetTier: TierRef, reason: string) → invoke('request_tier_transition', { targetTier, reason })"/>
    <wrapper ipc_command="set_global_budget"
             actual_params="usd_cap: f64 — the EXISTING invokeSetGlobalBudget(usdCap: number) wrapper at src/lib/ipc.ts:165 is correct as-is; G reuses it unchanged"
             phase_doc_assumed="(no new wrapper — reuse the shipped invokeSetGlobalBudget)"/>
  </wire_signature_audit>

  <architecture_check>
    <claim description="G adds NO tier-transition or tier-enforcement logic — it surfaces the existing request_tier_transition command and reads the existing currentTier slot reduced by the existing tier_transition branch" verify="grep -rn 'tier' src/components/SettingsPanel.tsx — expect calls to requestTierTransition + reads of s.currentTier ONLY; zero new reducer cases; zero edits under crates/runtime-main/src/tier/"/>
    <claim description="The SettingsPanel does NOT optimistically set currentTier — the backend's tier_transition event is the single writer" verify="grep -n 'currentTier' src/components/SettingsPanel.tsx — expect READS only (useGraphStore((s) => s.currentTier)); no setState/set of currentTier"/>
    <claim description="SettingsPanel mounts at App.tsx top level OUTSIDE the view conditional (cross-mode chrome, per C.3.2) — no Settings-tab/routing infrastructure is invented (the M06.E no-routing rule still holds)" verify="grep -n 'SettingsPanel' src/App.tsx — expect &lt;SettingsPanel/&gt; as a sibling of BudgetHeaderBar / ViewSwitch, NOT nested inside RuntimeLayout / .graph-layout / BuilderShell"/>
    <claim description="every new settings-panel className has a corresponding CSS rule (gotcha #67)" verify="for each className in G.3.3/G.3.5, grep src/styles.css for the rule; expect every class found"/>
  </architecture_check>

  <fan_out_grep>
    <grep pattern="request_tier_transition" purpose="confirm the command's only callers post-G are the new ipc.ts wrapper + the existing src-tauri tests — G adds no second backend caller"/>
    <grep pattern="currentTier" purpose="enumerate currentTier readers — confirm SettingsPanel joins MCPServerAddModal as a READER; the tier_transition reducer branch stays the sole writer"/>
    <grep pattern="globalBudgetCap" purpose="confirm the new slot's only readers/writers are SettingsPanel's BudgetControl + the new setGlobalBudgetCap action — no leakage into the per-session budget slot"/>
  </fan_out_grep>

  <zustand_selector_audit>
    <selector pattern="filter|map|find|Object.values" requires_use_shallow="true" import_path="zustand/react/shallow" note="G's selectors (s.currentTier, s.globalBudgetCap, s.setGlobalBudgetCap) are scalar/function — useShallow NOT required; if any derived selector is added it must be useShallow-wrapped per gotcha #75"/>
  </zustand_selector_audit>

  <playwright_warmup_recipe url="http://localhost:1420" timeout_seconds="16" before_first_spec="true"/>

  <test_isolation_audit>
    <persistent_slot store="useGraphStore" field="currentTier" preserved_across_clear="true" required_reset="beforeEach(() => useGraphStore.setState({ currentTier: 'novice' }))"/>
    <persistent_slot store="useGraphStore" field="globalBudgetCap" preserved_across_clear="true" required_reset="beforeEach(() => useGraphStore.setState({ globalBudgetCap: 0 }))"/>
  </test_isolation_audit>

  <existing_pattern_audit>
    <pattern grep_for="BudgetHeaderBar" rationale="BudgetHeaderBar is App-level cross-mode chrome mounted outside the view conditional — SettingsPanel mounts the SAME way (C.3.2 specifies it cross-mode). Do NOT mount it inside .graph-layout (Stage C made that Runtime-only); do not introduce routing chrome" affected_files="src/App.tsx" remediation="add &lt;SettingsPanel /&gt; as a sibling of BudgetHeaderBar / ViewSwitch, outside the view conditional"/>
  </existing_pattern_audit>

  <runtime_environment os="windows" note="Vitest + Playwright run against the Vite dev server with @tauri-apps/api module-mocked (gotcha #23); Vite cold-start mitigated per the playwright_warmup_recipe"/>

  <adr_triggers>
    <trigger>None new in G. G surfaces the EXISTING request_tier_transition command (no enforcement change, no schema, no backend, no IPC-protocol change). If G is found to need a tier-transition BACKEND change, that is a Hard Rule 8 capability-surface change — STOP and surface the plan first; do not author it inside this stage.</trigger>
  </adr_triggers>

  <gotchas>
    <trap>#23 — Playwright cannot drive the Tauri window; drive the Vite dev server with @tauri-apps/api module-mocked; mock request_tier_transition + set_global_budget at the invoke boundary.</trap>
    <trap>#53 — Vite cold-start; the first Playwright spec uses the 90s describe timeout + the curl warmup probe.</trap>
    <trap>#54 — window.__graphStore is the renderer-test state-injection affordance; the Playwright test scripts the tier_transition event through it. Never call window.__graphStore from production code.</trap>
    <trap>#67 — every settings-panel className gets a CSS rule + the every_class_has_a_corresponding_CSS_rule static test.</trap>
    <trap>#30 — Tauri errors cross the bridge as objects; render via the existing unwrapCmdError helper, never String(e).</trap>
    <trap>Novice↔Promoted ONLY — Operator is v1.0 (§0d). TierRef has no 'operator' member; do not surface an Operator option. A behavior test asserts no "operator" text renders.</trap>
    <trap>G surfaces the EXISTING request_tier_transition — it does NOT reimplement tier-transition or enforcement logic (Hard Rule 8). The tier_transition event is already reduced by graphStore (:1543); the component must NOT optimistically set currentTier.</trap>
    <trap>The Settings panel is FOCUSED (tier + budget) — not a catch-all; the API-key entry stays in SetupPanel.</trap>
    <trap>graphStore.budget is the per-session SPEND snapshot; globalBudgetCap is the user-CONFIGURED cap — distinct slots, do not conflate budget.capUsd with globalBudgetCap.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT modify tier evaluation or enforcement — no edits under crates/runtime-main/src/tier/, no change to request_tier_transition or request_tier_transition_with. G is a renderer-surface stage; it WIRES the existing command. A backend tier change is a Hard Rule 8 CODEOWNERS-flagged change requiring a plan-first surface.</warning>
    <warning>DO NOT surface the Operator tier in any control, label, or option. §0d locks v0.1 to Novice + Promoted; Operator is v1.0. The TierControl's transition target is derived purely from the two-member TierRef.</warning>
    <warning>DO NOT add a new Tauri command. request_tier_transition (M05.D) and set_global_budget (M04.F) both exist; invokeSetGlobalBudget already wraps the latter. If a new command appears necessary, surface in the retrospective — the default is "G renders + wraps existing commands only".</warning>
    <warning>DO NOT have the Settings panel absorb the SetupPanel API-key UI — the key UI stays where it is (Stage A fixed key persistence). G's panel is a focused tier+budget surface.</warning>
    <warning>DO NOT optimistically set currentTier in the component after calling requestTierTransition — the backend emits a tier_transition event that the existing reducer applies. A local set would double-source the slot.</warning>
    <warning>DO NOT invent a Settings-tab / routing system (the M06.E no-routing rule). AND do NOT mount SettingsPanel inside .graph-layout — Stage C extracted that into RuntimeLayout (Runtime-only), so a .graph-layout mount would hide the tier control in Builder mode. SettingsPanel is cross-mode (C.3.2): mount it at App top level, outside the view conditional, alongside BudgetHeaderBar / ViewSwitch.</warning>
  </execution_warnings>

  <time_box hours="6-9"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>State the pinned request_tier_transition params (targetTier + reason; app/state Tauri-injected). Confirm G surfaces — does not reimplement — the tier command, and that the tier_transition event updates currentTier through the EXISTING reducer (no new reducer case, no edits under crates/runtime-main/src/tier/). Confirm Operator is not surfaced anywhere in the DOM. Confirm M07-IRL #5 closed (Settings panel + Novice↔Promoted control ship) + M06.5 IRL 🟡-4 closed (the globalBudgetCap slot + the budget input reflecting/persisting). Note that Promoted becoming reachable unblocks the M06 MCP Add modal and sets up the post-M08 IRL re-confirm of M06.5 🔴-1 (MCP-registry real-app re-confirm — "gated on #5"). Confirm SettingsPanel mounts at App top level outside the view conditional and renders in BOTH Runtime and Builder modes (C.3.2 cross-mode).</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="G.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (git log --oneline main..HEAD + ls docs/build-prompts/retrospectives/M08.*-retrospective.md)</item>
    <item>strict-TDD invariant: git diff &lt;red&gt;..&lt;impl&gt; -- '**/tests/**' EMPTY</item>
    <item>diff stat</item>
    <item>wire_signature_audit reconciliation against the EXISTING request_tier_transition (commands.rs:542) + the reused invokeSetGlobalBudget</item>
    <item>M07-IRL #5 closed — the Settings panel + the Novice↔Promoted control; Operator not surfaced (behavior-asserted)</item>
    <item>M06.5 IRL 🟡-4 closed — the globalBudgetCap slot; the budget input reflects + persists</item>
    <item>gate results (v1.6 canonical order; vitest renderer ≥80; Playwright + warmup; CI-parity)</item>
    <item>every-class-has-a-CSS-rule confirmation (gotcha #67 static check)</item>
    <item>M08.G retrospective filled-in [END] section</item>
    <item>draft commit message from G.6</item>
    <item>explicit statement: "Stage M08.G is ready. I will not push until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### G.6 Commit Message

```
feat(renderer): M08 Stage G — Settings panel + Novice↔Promoted tier promotion

The Settings panel the renderer's own type comments named (the
graphStore `currentTier` slot doc-comment references "the renderer's
Settings panel") but no component ever delivered: a current-tier
display + a Novice↔Promoted control over the EXISTING
request_tier_transition backend command. G surfaces that command — it
does NOT reimplement tier-transition or enforcement logic (Hard
Rule 8); the tier_transition event the backend emits is already
reduced by graphStore into `currentTier`, so the displayed tier
updates through the existing event path. Closes the M07-IRL #5
🔴-candidate — the Promoted tier (a §0d v0.1-scope capability) and,
through it, MCP-server management are now reachable for the first
time. Operator is NOT surfaced (v1.0, §0d — TierRef has no Operator
member). Also closes M06.5 IRL 🟡-4 — a new graphStore
`globalBudgetCap` slot wires the budget-cap input to reflect +
persist its value via the existing set_global_budget command.

Renderer surface:
- src/components/SettingsPanel.tsx (new): a focused settings surface
  with TierControl (current-tier display + Novice↔Promoted button)
  and BudgetControl (budget-cap input). Mounted at App.tsx top level
  as cross-mode chrome — outside the Runtime↔Builder view conditional
  (C.3.2), so the tier control is reachable in both modes; v0.1 has
  no Settings-tab infrastructure (the M06.E no-routing rule). Not a
  catch-all — the API-key entry stays in SetupPanel.

IPC:
- src/lib/ipc.ts: requestTierTransition(targetTier, reason) wrapper,
  params pinned to the existing request_tier_transition command
  (commands.rs:542) via the v1.8 wire_signature_audit. invokeSetGlobalBudget
  reused unchanged.

Store:
- src/lib/graphStore.ts: globalBudgetCap slot + setGlobalBudgetCap
  action (M06.5 IRL 🟡-4 state-wiring), preserved across clear() like
  currentTier. currentTier + the tier_transition reducer were already
  present — read-only here; no new tier reducer logic.

Styles:
- src/styles.css: .settings-panel + descendant classes, theme-variable
  driven; every className paired with a CSS rule per gotcha #67.

Tests:
- tests/unit/components/SettingsPanel.test.tsx (new): vitest — panel
  render, TierControl (tier display, promote/demote labels,
  requestTierTransition calls, no optimistic set, no Operator option,
  error surfacing) + BudgetControl (input reflects the slot, save
  persists, negative-input rejection) + the styles contract.
- tests/e2e/settings_tier_promotion.spec.ts (new): Playwright — the
  Settings panel renders → promote → a scripted tier_transition event
  updates the displayed tier through the existing reducer; the panel
  is reachable in Builder mode (cross-mode); Operator never offered;
  the budget cap reflects + persists.

No Rust changes, no schema changes — request_tier_transition (M05.D)
and set_global_budget (M04.F) predate M08; TierRef is already
generated. Renderer vitest ≥80%. Strict v1.8 two-commit TDD:
git diff <red>..<impl> -- '**/tests/**' EMPTY.

Not in this stage: the Operator tier (v1.0); API-key entry (stays in
SetupPanel); tier evaluation/enforcement changes; budget-primitive
work (the four budget actions shipped at M04); MCP-panel changes
(making Promoted reachable unblocks the already-correctly-tier-blocked
MCP Add modal as a free consequence).

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---
## Stage V — Verifier (four passes; mandatory integration smoke)

> Per ADR-0008 + `STAGE-PROMPT-PROTOCOL.md` §14. Runs in a fresh CLI session between Stage G (last work stage) and Stage H (closeout). Clear-and-paste bias guard: this session has NOT seen the M08.A–G retrospectives, the M08 summary (does not yet exist), or any gap-analysis entry beyond M07's. Four passes in order — Inventory → Wire → Behavior → Multi-call invariants. Findings tagged 🔴 (block merge → D.fix) / 🟡 (carry forward) / 🟢 (tech debt). Maximum 2 D.fix iterations before maintainer escalation. M08.V is the verifier run for the largest milestone in the MVP and the one that **discharges the entire post-M07 carry-forward backlog** — V's central duty is to prove that discharge is real.

### V.1 Problem Statement

Run the four-pass, fresh-context, contract-fidelity verifier (ADR-0008) against M08's deliverables — Stages A, B, C, D1, D2, E, F1, F2, G — and confirm they satisfy the spec (Phase 9, ~lines 2495–2543), MVP §M8 (the eight acceptance criteria), and ADR-0019 + ADR-0020, before M08 may merge. The verifier shows up knowing only what was *contracted* — the spec, the MVP criteria, the two ADRs, the M08 phase doc body — not the work narrative that justified what shipped. The clear-and-paste session pattern is the structural bias guard; the `<read_first>` omission of every `*-retrospective.md` / summary / prior-gap-analysis is the artifact-level discipline.

M08 is the milestone that **discharges** the coupled M07.V Dec-6 carry-forward set, so V's central job is the **discharge-is-real** check. The M07.V Dec-6 findings (🟡 #2 `skills_lock::verify`, 🟡 #3 `McpDispatcher::on_server_connected`, 🟡 #5 the agent-with-tools loop) were each a *primitive shipped, tested, and correct* — but with **no production caller**: the construction graph that would reach them did not exist until M08 built the Builder + Tester. That is precisely the **M07.V 🔴 #1-class trap** the verifier exists to catch: implementation tests green, contract tests missing — a primitive whose only callers are its own unit tests is not wired into the product. V must grep-verify that each of the three wires now has a genuine **production** caller on the Tester's real code path (not a test fixture, not a `#[cfg(test)]` harness), citing file:line. The phase doc hands V the evidence it needs: Stage A authored the `<construction_reachability_check>` with all three wires `inputs_reachable="false"`; Stage F1 inverted each to `inputs_reachable="true"` with the concrete production call site. V reads both blocks off its bias-guarded read-list, treats Stage F1's inverted-to-`true` closure as the **expected** outcome, and emits 🔴 only if the closure is *not* borne out by the shipped code — the M07 A→D1→D2 discharge-verification pattern, here A→F1.

The `--features integration` reference-MCP-server smoke is **mandatory** in the Behavior pass (M06.V Dec-7, codified in `STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` v1.8). M08's Tester *is* a real tool-driving session — it runs `AgentSdk::run_agent` over a tool-bearing framework against the live Anthropic endpoint and dispatches real `ProviderEvent::ToolUse` events — so the mock-only escape that earlier milestones could lean on is closed. A V run that records `0/0` for the integration smoke has not verified M08's headline deliverable. The smoke must run, must report `N/M` with `N>0`, and the Behavior pass observes the assembled outcome: `token_usage > 0`, a capability violation surfacing as a **test failure with defaults applied** (not a live HITL prompt), the throwaway test SQLite isolated from the user-session DB, and a tampered artifact refused at load via `ArtifactHashMismatch`.

V also applies the Dec-6 **standing rule** going forward: if M08 itself ships a *new* primitive that is delivered + tested but whose production driver is absent and root-caused to an accepted ADR's named carry-forward, that is a 🟡-with-mandatory-enumeration, not a 🔴 — and V records the carry-forward target for M09.

### V.2 Scope to verify

M08 deliverables A–G traced against spec Phase 9 (~2495–2543) + MVP §M8 (the eight acceptance criteria) + ADR-0019 (the Tester isolated-session model) + ADR-0020 (the Builder canvas↔`framework.json` state model). The verifier's four passes operate on the following surface; the per-pass instruction detail is in the V.5 `<verification_passes>` block.

**The eight MVP §M8 acceptance criteria** — each must have a shipped, exercised artifact:

| # | Criterion | Owning stage(s) |
|---|---|---|
| 1 | Drag an Agent node onto an empty canvas; set role / model / `allowed_*` inline; per-node capability disclosure renders in plain English | D1 |
| 2 | Connect Agent→Skill → the skill is added to the agent's `allowed_skills` | D2 |
| 3 | Connect Agent→Agent → child `allowed_*` intersected with parent (`narrowing.rs::narrow()` — Rust); the UI surfaces the narrowing decisions | D2 (narrowing computed in B) |
| 4 | Click Validate → continuous schema + capability validation runs; errors render as red badges at the offending node | B (`validate_framework`) + D2 (badges) + E (Validate button) |
| 5 | Click Test → enter a task → the Tester modal opens → a sandboxed isolated session runs → graph + VDR + signals + token spend + pass/fail trace surface | E (Test button) + F1 (backend) + F2 (modal) |
| 6 | Canvas \| JSON tabs; edit the JSON; switch back → the canvas shows the updated framework (two-way binding) | E |
| 7 | Save framework to disk → `framework.json` + companion `skill.md` / `tool.md` / `agent.md` written | B (backend) + E (Save affordance) |
| 8 | Reload from disk → the canvas reconstructs identical to the saved state | B (backend) + E (Load affordance) |

**ADR-0019 + ADR-0020 deliverables** — both ADRs are filed at their stage (F1 / C respectively) and flip `Proposed → Accepted` in the M08 PR; V confirms the *code* matches the *decision*: ADR-0020 — `framework.json` is the single source of truth and the canvas is a pure projection of it (`builderStore` holds the document; nodes/edges derive from it; a JSON-tab edit replaces the document and the canvas re-derives); ADR-0019 — the Tester runs an isolated, throwaway, **sequential** test session (its own throwaway SQLite; capability violations → test failures with defaults; no writes to any user data directory; teardown on close) and is explicitly **not** the §1c concurrent-multi-session feature (❌ for v0.1 per §0d).

**Builder backend (B)** — `validate_framework` (continuous schema + capability validation, errors keyed to the offending node/JSON-path; one validator, two triggers — continuous + explicit; reuses `runtime-core` capability + the CI schema checker, **no TS re-implementation** per spec §9); `save_framework` / `load_framework` (path-agnostic `&Path`; `framework.json` + companion `.md`; the load side reuses `framework_loader`); `framework_capability_summary` (whole-framework totals from `capability_map` + the per-spawn-edge narrowing triple — embedded in the `validate_framework` report's `capability_summary` field, not a separate command); `list_installed` (the `skills.lock` **production reader** — the first production consumer of the M07.B `skills.lock` primitive, closing M07-IRL #6 and the reader half of M07.V 🟡 #2).

**Workbench shell + Canvas (C, D1, D2, E)** — the Runtime↔Builder view switch + the three-panel build mode; the five-tab filterable drag-source Palette; the `@tauri-apps/plugin-dialog` local-file picker (closes M07.V 🟡 #4, wired into the Builder import affordance + the M07 Import panel); the interactive React-Flow node editor (drag-drop instantiation + inline node config + the reused M05 §8.security L1 capability-disclosure component); the four edge types (Agent→Skill = `allowed_skills`; Agent→Tool = `allowed_tools`; Agent→Agent = `spawns`; Hook→Task = `post_hooks`); automatic Agent→Agent capability narrowing surfaced from B's report; continuous-validation red badges; the Inspector (live `framework.json` preview + disk-diff + whole-framework capability summary + Validate/Test buttons); the Canvas | JSON two-way binding.

**Tester (F1, F2)** — `test_framework`: the isolated throwaway test session (`run_test_session_with` over a `tempfile` DB → a `TestOutcome`); capability violations → `CapabilityFailure` on the outcome with no HITL prompt raised; teardown deletes the temp DB; the assembled regression exercises a real drone + the real `AgentSdk::run_agent` loop + a tool-bearing framework; the Tester modal (task input + the smaller live-graph pane + VDR/signals + token/timing + the pass/fail trace).

**Settings panel + tier promotion (G)** — the Settings panel hosting Novice↔Promoted tier promotion (the renderer surface over the existing `request_tier_transition` command), closing the **M07-IRL #5 🔴-candidate** (in v0.1-today the Promoted tier — a §0d v0.1-scope capability — is unreachable, which leaves MCP-server management unreachable); the budget-settings state-wiring (M06.5 IRL 🟡-4).

**The carry-forward closures** — V independently confirms each disposition the M08 phase doc claims, against the shipped code:

| Carry-forward | Claimed M08 closure | What V checks |
|---|---|---|
| **M07.V 🟡 #2** — `skills_lock::verify` has no production load-path caller | **F1** (verify on the Tester's artifact-load path); **B** (the `skills.lock` reader) | Wire pass: grep finds `skills_lock::verify` called from the Tester's artifact-load path — a production call site, not a test |
| **M07.V 🟡 #3** — `McpDispatcher::on_server_connected` has no production connect-handler caller | **F1** (the Tester's MCP connect handler) | Wire pass: grep finds `on_server_connected` called from the test session's MCP connect handler — production, not test |
| **M07.V 🟡 #5** — agent-with-tools production driver absent | **F1** (the Tester runs a tool-bearing framework through `AgentSdk::run_agent`) | Wire + Behavior: a tool-bearing framework drives a real `ProviderEvent::ToolUse`; `token_usage > 0` in the integration smoke |
| **M07.V 🟡 #4** — local-file picker UI not shipped | **C** (`@tauri-apps/plugin-dialog`) | Inventory + Behavior: the picker is shipped + wired into both the Builder import affordance and the M07 Import panel |
| **M07-IRL #5 🔴-candidate** — no tier-promotion UI; Promoted tier + MCP management unreachable | **G** (the Settings panel) | Inventory + Behavior: a Novice can reach Promoted via the Settings panel; MCP-add becomes reachable in the Promoted tier |
| **M07-IRL #2 / #3 / #6 / #7** — token in/out split; import-panel contrast; Import panel reloads installed artifacts after restart; API key persists across restart | **A** (#2/#3/#7); **B**+**C** (#6 — converges with 🟡 #2) | Inventory + Behavior: each has a named green test or a documented structural close |
| **M06.5 IRL 🟡-1..4** — HITL `ui_variant`; `npx` `.cmd` shim; stale Test error banner; budget settings not state-wired | **A** (🟡-1/-2/-3); **G** (🟡-4) | Inventory + Behavior: each closed with a named test |
| **M04 `plan_loop`** — `plan_loop.rs` driver unwired | **A** (wire the driver into the session entrypoint) | Wire pass: `plan_loop` now has a production caller from the session entrypoint |

Additionally V records — but does not block on — that **M06.5 IRL 🔴-1** (MCP-registry real-app re-confirm) becomes *re-confirmable* in the post-M08 IRL pass: once Stage G makes the Promoted tier reachable, MCP-add is reachable, and 🔴-1 can re-confirm. That re-confirm is an IRL-pass item, not an M08.V finding.

### V.5 CLI Prompt

Paste into a **fresh** Claude Code session — the clear-and-paste pattern is the load-bearing bias guard.

```xml
<verifier_stage_prompt id="M08.V">
  <context>
    M08 Stage V — the fresh-context, four-pass, contract-fidelity
    verifier (ADR-0008). M08 is the largest MVP milestone — the
    Workbench / Builder Canvas + the sandboxed Tester + the Settings
    panel — and the milestone that DISCHARGES the entire post-M07
    carry-forward backlog. Run with empty session memory: you have NOT
    seen the M08.A–G retrospectives, the M08 summary (does not yet
    exist), or any gap-analysis entry beyond M07's. The clear-and-paste
    session is the structural bias guard.

    Four passes IN ORDER: Inventory → Wire → Behavior → Multi-call
    invariants. Findings tagged 🔴 (block merge → D.fix), 🟡 (carry
    forward to the M08 gap-analysis Carry-forward section), 🟢 (tech
    debt → docs/tech-debt.md). Maximum 2 D.fix iterations before
    maintainer escalation.

    V's CENTRAL DUTY is the discharge-is-real check. The coupled
    M07.V Dec-6 set — 🟡 #2 (skills_lock::verify), 🟡 #3
    (McpDispatcher::on_server_connected), 🟡 #5 (the agent-with-tools
    loop) — were each a primitive shipped + tested + correct but with
    NO production caller; M08's Tester is the production driver that
    reaches them. This is the M07.V 🔴 #1-class trap the verifier
    exists to catch: implementation tests green, contract tests
    missing — a primitive whose only callers are its own unit tests
    is NOT wired into the product. Grep-verify that each of the three
    wires now has a genuine PRODUCTION caller on the Tester's real
    code path (file:line; not a #[cfg(test)] harness, not a fixture).
    The phase doc hands you the evidence: Stage A authored the
    <construction_reachability_check> with all three wires
    inputs_reachable="false"; Stage F1 inverted each to "true" with
    the production call site. Read both blocks; Stage F1's
    inverted-to-true closure is the EXPECTED outcome — emit 🔴 only
    if the shipped code does NOT bear it out.

    The Behavior pass MUST run the --features integration
    reference-MCP-server smoke (M06.V Dec-7, codified in the v1.8
    STAGE-V template). M08's Tester IS a real tool-driving session —
    mock-only is not acceptable. A V run that records 0/0 for the
    integration smoke has not verified M08's headline deliverable.

    Apply the Dec-6 standing rule going forward: a NEW primitive M08
    itself ships that is delivered + tested but driver-absent and
    root-caused to an accepted ADR's named carry-forward is a
    🟡-with-mandatory-enumeration, not a 🔴 — record the carry-forward
    target for M09.
  </context>

  <read_first>
    <file>STAGE-PROMPT-PROTOCOL.md §14 (the verifier schema) + §8 (the v1.8-codified standing rules)</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (the verifier design rationale — the four passes + the bias guard)</file>
    <file>docs/adr/0019-tester-isolated-session-model.md (the Tester isolated-session model V reconciles the F1/F2 deliverables against)</file>
    <file>docs/adr/0020-builder-canvas-state-model.md (the canvas↔framework.json source-of-truth model V reconciles the C/D1/D2/E deliverables against)</file>
    <file>docs/adr/0011-m06f-scope-seam-not-running-app.md (the (d) construction carry-forward whose Tester-side driver F1 discharges — context for the discharge-is-real check)</file>
    <file>docs/build-prompts/M08-workbench.md (Background incl. the Carry-forward table — the M07.V 🟡 #2/#3/#4/#5 + M07-IRL #2/#3/#5/#6/#7 + M06.5-IRL 🟡-1..4 + M04 plan_loop set M08 discharges; the nine-stage structure; Key constraints; all stages A.1–G.6 incl. the Stage A and Stage F1 `<construction_reachability_check>` blocks; V.1/V.2 — but NOT any *-retrospective.md reference the doc may make; the phase doc itself carries every carry-forward detail you need)</file>
    <file>agent-runtime-spec.md Phase 9 (~2495–2543, the Visual Canvas + Tester) + §0b (the Tool/Skill/Agent three-concept model) + §0d (the v0.1 Release Scope Matrix — §1c multi-session is ❌; the Tester is a build-time sequential throwaway session, NOT §1c) + §3 (Visual Design — the node conventions the Builder Canvas reuses) + §3a (the plan loop) + §5a (Tool Namespace Resolution — re-resolution on connect) + §8.security L1/L2a/L4 (the capability gate / narrowing / tier filter) + §9 ("no duplication of validation logic between TS and Rust")</file>
    <file>docs/MVP-v0.1.md §M8 (the eight acceptance criteria)</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #23 Playwright-cannot-drive-the-Tauri-window, #54 renderer-level injection, #67 computed-style checks, #68 agent_id-correctness, #81 stale-profraw — and the M07-class IRL bug patterns)</file>
    <file>docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md (the output shape this stage fills in)</file>
    <file>docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md (v1.8 — the parameterization guidance, the Dec-6/Dec-7 standing rules, the mandatory-integration-smoke rule)</file>
    <file>docs/tech-debt.md (the TD-NNN ledger already logged; M08.V's 🟢 findings append here)</file>
  </read_first>

  <scope_to_verify ref="docs/build-prompts/M08-workbench.md" section="V.2 Scope to verify"/>

  <verification_passes>
    <pass name="inventory">
      For every MVP §M8 acceptance criterion (1–8), every ADR-0019
      and ADR-0020 deliverable, and every carry-forward closure the
      M08 phase doc claims (M07.V 🟡 #2/#3/#4/#5; M07-IRL #2/#3/#5/#6/#7;
      M06.5 IRL 🟡-1..4; M04 plan_loop), confirm a shipped artifact
      exists in `git ls-files` AND its shape matches the corresponding
      stage's X.2 "Files to Change" + X.3 "Detailed Changes" narrative.
      Missing → 🔴. Present but empty/stub → 🟡. Present with wrong
      scope/signature → 🟡. Pay attention to: the new
      crates/runtime-main/src/builder/ module (mod.rs + validate.rs +
      persist.rs + summary.rs) and its four thin src-tauri command
      wrappers; the new src/lib/builderStore.ts (the ADR-0020 store,
      SEPARATE from graphStore.ts); the new Builder Canvas component
      (interactive — distinct from the read-only GraphCanvas.tsx); the
      five-tab Palette; the @tauri-apps/plugin-dialog file picker; the
      Inspector + the Canvas|JSON tab toggle; the Tester backend
      (run_test_session_with) + the Tester modal; the Settings panel.
      READ the Stage A and Stage F1 <construction_reachability_check>
      blocks: Stage F1's inverted-to-inputs_reachable="true" closure
      of the three M07.V Dec-6 wires is the EXPECTED outcome — it is
      NOT an inventory gap. M08 is NOT expected to add a schema; if a
      stage authored a new event variant, confirm the regenerated
      types + the ADR are present.
    </pass>
    <pass name="wire">
      Run the 5-step data-path trace (gotcha #66) for every M08 spec
      claim; a trace that breaks at step 4 with zero matching
      consumers OR multiple plausible consumers → 🔴 ("wire incomplete"
      / "wire ambiguous"). The load-bearing traces:
      (1) ADR-0020 canvas↔framework.json — a JSON-tab edit replaces
      builderStore.framework and the canvas re-derives; a canvas edit
      (D1 add/config, D2 connect) mutates builderStore.framework; the
      canvas is a PURE PROJECTION of the document, never a parallel
      source of truth.
      (2) validate_framework — one validator, two triggers: continuous
      (debounced, on every canvas edit) AND explicit (the Inspector
      Validate button); both call the SAME builder::validate seam;
      errors key to the offending node/JSON-path and render as red
      badges; confirm runtime-core capability + the CI schema checker
      are REUSED, not re-implemented in TS (spec §9).
      (3) Agent→Agent narrowing — the decision is computed by
      narrowing.rs::narrow() in Rust and surfaced via Stage B's
      report at capability_summary.spawn_edges[] (each a
      SpawnEdgeNarrowing {parent_caps, child_declared_caps,
      narrowed_caps}; narrowed_caps a serde-tagged Result — an Err
      folds into capability_errors per B.3.2 step 4); D2 RENDERS the
      record; grep must find NO TS intersection re-implementation.
      (4) save→load round-trip — save_framework writes framework.json
      + companion .md; load_framework reuses framework_loader; the
      LoadedFramework round-trips the ADR-0020 canvas projection.
      CRITICAL — the discharge-is-real traces. Grep-verify the three
      M07.V Dec-6 wires now have PRODUCTION callers (file:line; NOT a
      test, NOT a fixture, NOT a #[cfg(test)] block):
      • skills_lock::verify — called on the Tester's artifact-load
        path (when the test session byte-loads an imported artifact).
      • McpDispatcher::on_server_connected — called from the test
        session's MCP connect handler.
      • AgentSdk::run_agent — driven by the Tester with a tool-bearing
        framework (the production tool-driving session).
      A primitive with only test callers is the M07.V 🔴 #1-class
      contract gap → 🔴. Reconcile against Stage F1's inverted
      <construction_reachability_check>: if F1 claims inputs_reachable
      ="true" with a file:line and the grep confirms it, the wire is
      DISCHARGED — not a finding. Also trace: the M04 plan_loop driver
      now has a production caller from the session entrypoint.
    </pass>
    <pass name="behavior">
      Runtime-render / runtime-execute checks — static analysis is
      insufficient for this class (the M04 BudgetHeaderBar-CSS bug
      passed every static check). Run the live harness:
      • MANDATORY --features integration reference-MCP-server smoke:
        the Tester runs a REAL tool-driving session; a real
        ProviderEvent::ToolUse dispatches; token_usage > 0; a
        capability violation surfaces as a TEST FAILURE WITH DEFAULTS
        APPLIED, NOT a live HITL prompt (test sessions don't block on
        user input); the throwaway test SQLite is a distinct path
        from the user-session DB; a tampered imported artifact →
        ArtifactHashMismatch → load refused. This smoke MUST run and
        report N/M with N>0 — 0/0 is a verification failure, not a
        pass.
      • cargo test for the runtime-main builder module + the Tester
        module; cargo llvm-cov for the runtime-main ≥95 gate
        (run `cargo llvm-cov clean --workspace` first — gotcha #81).
      • Vitest + jsdom for the Builder renderer surface: the Palette
        tabs/filter/drag-source; the Canvas node editor (drag-drop
        instantiation + inline config + the capability disclosure);
        the edge editor + the narrowing notice + the red badges; the
        Inspector + the Canvas|JSON two-way binding; the Tester modal;
        the Settings panel tier promotion. Renderer ≥80.
      • Playwright (gotcha #23 — against the Vite dev server with
        @tauri-apps/api module-mocked; React-Flow drag-drop + edge
        creation via Playwright's drag API): MVP §M8 criteria 1/2/3/4/6
        exercised end-to-end on the canvas; the file picker affordance;
        the Settings tier-promotion flow.
      For each failing test, trace which earlier pass would have
      caught it (Inventory → file missing; Wire → step-5 mismatch;
      Multi-call → second call broken). Findings cite the failing
      test name + the pass that should have caught it earlier.
      Coverage below a ≥95 / ≥80 gate → 🔴.
    </pass>
    <pass name="multi_call_invariants">
      Every public API / IPC method / Tauri command M08 adds must
      survive "called twice in sequence." Assert a sequential-call
      test exists OR run one inline:
      • save_framework → load_framework → save_framework is
        BYTE-STABLE (a save→load→save cycle does not drift).
      • the canvas projection is a PURE FUNCTION of framework.json —
        re-deriving it from the same document twice yields the same
        canvas (ADR-0020).
      • validate_framework is idempotent — the same document validated
        twice yields the same FrameworkValidationReport.
      • the Tester run twice in sequence is clean — each throwaway
        test DB is independent; the second run does not see the
        first's state; both temp DBs are torn down.
      • an invalid-JSON tab edit does NOT desync the builderStore —
        the canvas does not lose the last-valid framework.
      • regression: the M01–M07 multi-call tests still green (drone
        IPC, respond_hitl, framework_loader, capability_enforcer,
        sandbox round-trip, tier evaluator, audit writer, skills.lock
        reproducibility, the import pipeline, the concrete
        McpDispatcher re-resolution).
      Missing per-surface test → 🟡 (carry forward to TD-NNN);
      the second call does not pass → 🔴.
    </pass>
  </verification_passes>

  <findings_format ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md" section="Findings"/>

  <merge_gate red_blocks="true" dfix_iteration_cap="2" waiver_path="docs/adr/NNNN-waiver-M08-finding-N.md"/>

  <gates milestone="M08"/>

  <self_correction_budget>3</self_correction_budget>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md">
    <special_log>M08.V verifies the largest MVP milestone and the discharge of the entire post-M07 carry-forward backlog. Record explicitly: (1) Did the mandatory --features integration reference-MCP-server smoke run — report N/M, NOT 0/0? (2) Did Stage F1's `<construction_reachability_check>` closure hold — are the three M07.V Dec-6 wires (skills_lock::verify, McpDispatcher::on_server_connected, the agent-with-tools loop) confirmed to have genuine PRODUCTION callers, with grep evidence + file:line? Name each call site. (3) Did the ADR-0020 canvas↔framework.json two-way binding verify as a real contract — a JSON edit re-derives the canvas AND a canvas edit updates the JSON — not merely a one-way JSON preview? (4) Did ADR-0019's Tester isolation verify — throwaway DB, test-defaults-not-HITL, no user-data writes, teardown? (5) Apply + record the Dec-6 standing rule for any NEW delivered/driver-absent primitive M08 itself introduced, with the carry-forward target. (6) Note any carry-forward convergence for M09 (the Generators wiring into the Workbench shell). (7) Protocol-calibration observations for v1.9. The Decisions[END] section feeds the closeout's Carry-forward section.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="V.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (build machine `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M08.*-retrospective.md`)</item>
    <item>findings list, sorted by severity</item>
    <item>per-pass summary (counts + notable findings — Inventory files/shape-match; Wire traces; Behavior primitives exercised + coverage-gate findings; Multi-call surfaces)</item>
    <item>the mandatory --features integration reference-MCP-server smoke result (N/M, explicitly NOT 0/0)</item>
    <item>the M07.V Dec-6 discharge confirmation — the three wires (skills_lock::verify, McpDispatcher::on_server_connected, the agent-with-tools loop) have production callers, with grep evidence + file:line</item>
    <item>the ADR-0020 canvas↔JSON two-way-binding contract confirmation + the ADR-0019 Tester-isolation confirmation</item>
    <item>Dec-6 standing-rule rulings for any new delivered/driver-absent primitive M08 introduced (with the M09 carry-forward target)</item>
    <item>the VERIFIER-RETROSPECTIVE-TEMPLATE.md filled-in [END] section (verification axes + the integration-smoke + discharge observations + protocol-calibration notes)</item>
    <item>merge recommendation: "Proceed to closeout (Stage H)" | "Open D.fix for 🔴 findings: &lt;cite numbers&gt;" | "Re-tier"</item>
    <item>explicit statement: "Stage M08.V is ready. I will not commit until you approve."</item>
  </approval_surface>
</verifier_stage_prompt>
```

### V.6 Commit Message

```
verify(M08): in-band V — four passes + mandatory --features integration smoke

Four-pass contract-fidelity verifier. The integration smoke ran
(not 0/0). Findings: <N🔴 N🟡 N🟢>. Confirmed the M07.V Dec-6
discharge — skills_lock::verify / McpDispatcher::on_server_connected /
the agent-with-tools loop now have production callers (the Tester).
ADR-0019/0020 reconciled; the canvas↔JSON two-way binding verified
as a real contract. Dec-6 standing rule applied to <…>. Disposition:
<Proceed to closeout | D.fix>.

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

<!-- ============================================================ -->
<!-- Stage H — Phase Closeout (always FINAL, runs AFTER Stage V). -->
<!-- Per CLAUDE.md §20: append-only entry in docs/gap-analysis.md.  -->
<!-- v1.6: <simplify_pass> required child of <deliverables>.       -->
<!-- v1.8: <coverage_policy_reconciliation> required child.        -->
<!-- M08 uses work-stage letters A–G (D, F split); the closeout    -->
<!-- takes the next free letter — H. There is no rule fixing the   -->
<!-- closeout to "G".                                              -->
<!-- ============================================================ -->

## Stage H — Closeout

> Per CLAUDE.md §20 + `STAGE-PROMPT-PROTOCOL.md` §8. Runs after Stage V (and any D.fix iterations) commits. The FINAL stage of M08. Produces FIVE artifacts: the M08 parent-milestone summary, the immutable M08 gap-analysis entry (append-only), the v1.6 `<simplify_pass>` outcome, the v1.8 `<coverage_policy_reconciliation>`, and the M08 PR description draft. Cumulative product↔spec review across M01–M08. Append-only — never edit, reorder, or delete a prior gap-analysis entry.

### H.1 Problem Statement

Aggregate M08 across all nine work stages — A, B, C, D1, D2, E, F1, F2, G — plus the Stage V verifier, and close the milestone out. Concretely, Stage H produces:

1. **`docs/build-prompts/retrospectives/M08-summary.md`** — the parent-milestone summary per `SUMMARY-TEMPLATE.md`: aggregates the nine per-stage retrospectives + the Stage V verifier retrospective, scores the three axes across stages, and marks the milestone verdict. M08 is the largest milestone in the MVP — the summary aggregates per-primitive coverage outcomes for the new `builder` module and the Tester module, and records the protocol-level observations (the nine-stage A–G split holding at the M06/M07 cadence; the A→F1 construction-graph chain completing).

2. **The immutable M08 entry in `docs/gap-analysis.md`** — appended at the bottom per the six-section template (none of the six optional — write "None observed." rather than omit). The entry is cumulative product↔spec evaluation across M01–M08, not a retrospective and not a changelog. Its **Carry-forward section** records the closure of the entire post-M07 backlog M08 absorbed: the coupled M07.V Dec-6 set (🟡 #2/#3/#4/#5) RESOLVED; the M07-IRL findings (#5 🔴-candidate RESOLVED at G; #2/#3/#6/#7 RESOLVED); the M06.5 IRL 🟡-1..4 RESOLVED; and the M04 `plan_loop` driver RESOLVED. It also records that **M06.5 IRL 🔴-1** (the MCP-registry real-app re-confirm) is now **re-confirmable** in the post-M08 IRL pass — Stage G made the Promoted tier reachable, so MCP-add is reachable. Its **Spec-review section** records the two M08 spec-phrasing refinements the build surfaced as §12-owned technical decisions: the §1c-multi-session-vs-Tester-isolated-session clarification (the Tester needs *an* isolated session, not the §1c concurrent-session pool — §0d marks §1c ❌ for v0.1) and the `validate_framework` command-return-vs-spec-"posts results as events" refinement (continuous validation is a request/response interaction; the project's IPC matured to synchronous command returns since the spec was written).

3. **The v1.6 `<simplify_pass>` outcome** — run the `simplify` skill against the `M08.A..HEAD` cumulative diff. M08 is large and grew across nine stages; likely surfaces include builder-module duplication, Builder-vs-live-graph node-component parallelism, canvas-projection derivation seams, and Tester-vs-smoke-session parallel surfaces. The maintainer-approved subset lands as a focused refactor commit on the same branch **before** the PR opens; the non-approved remainder defers to `docs/tech-debt.md` per the ADR-0008 🟢 ledger. An empty pass ("no proposals surfaced") is a valid outcome — but the pass must run and the outcome must be recorded.

4. **The v1.8 `<coverage_policy_reconciliation>`** — M08 changed coverage gates: the new `builder` module enters the runtime-main ≥95 gate (Stage B), and the Tester module enters it (Stage F1), plus any new OS-call-holdout `--ignore-filename-regex` exclusion a stage surfaced. Per the v1.8 four-mirror rule, every coverage change syncs `docs/coverage-policy.md` (§B baselines + a §C M08 milestone entry), the CLAUDE.md §5 exclusion-category list, the CLAUDE.md §6 `cargo llvm-cov` commands, and `codecov.yml` — byte-consistently, in this commit. Note: Stage B's retro should confirm the `builder` module is pure/seam/`tempfile`-tested and added **no** new exclusion; if so, the reconciliation records that and only the §B/§C baseline entries are appended. Any drift between the four mirrors is a bug, fixed here.

5. **The M08 PR description draft** — per `.github/PULL_REQUEST_TEMPLATE.md`; drafted, surfaced, **not** opened until the maintainer explicitly asks. **ADR-0019** (the Tester isolated-session model) and **ADR-0020** (the Builder canvas↔`framework.json` state model) flip `Proposed → Accepted` in the M08 PR before merge.

The Stage V retrospective's findings feed the closeout directly: 🟡 findings go into the gap-analysis entry's Carry-forward section; 🟢 findings already logged to `docs/tech-debt.md` during V; 🔴 findings (if any) were resolved by a D.fix iteration OR a new waiver-as-ADR **before** this closeout commit. The gap-analysis Carry-forward section records the M07.V Dec-6 discharge disposition (V's Wire-pass production-caller confirmation) and the M07-IRL #5 close, and the Adherence-to-spec section cites the eight MVP §M8 criteria + ADR-0019/0020 against file:line.

### H.5 CLI Prompt

```xml
<closeout_stage_prompt id="M08.H">
  <context>
    M08 Stage H — the phase closeout. The FINAL stage of M08, the
    largest milestone in the MVP (the Workbench / Builder Canvas + the
    sandboxed Tester + the Settings panel + the discharge of the
    entire post-M07 carry-forward backlog). Runs after Stage V (and
    any D.fix iterations) commits. Produces FIVE artifacts: the M08
    parent-milestone summary + the immutable gap-analysis entry
    (append-only) + the v1.6 simplify_pass outcome + the v1.8
    coverage_policy_reconciliation + the M08 PR description draft. Per
    CLAUDE.md §20 + STAGE-PROMPT-PROTOCOL.md §8.

    M08 uses work-stage letters A–G (D split D1/D2, F split F1/F2);
    the closeout takes the next free letter — H. There is no rule
    fixing the closeout to "G".

    The gap-analysis entry's commit is the FINAL commit on the
    milestone branch and gates the M08 PR push — UNLESS the
    simplify_pass produces a maintainer-approved focused-refactor
    commit, in which case that refactor commit is the final commit
    and the gap-analysis lands one commit prior. ADR-0019 (the Tester
    isolated-session model) + ADR-0020 (the Builder canvas↔
    framework.json state model) flip Proposed → Accepted in the M08
    PR before merge.
  </context>

  <read_first>
    <file>CLAUDE.md (especially §5/§6 the coverage source-of-truth + change protocol, and §20 the Gap Analysis Protocol)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (§8 closeout-only tags; the v1.6 simplify_pass required child; the v1.8 coverage_policy_reconciliation required child)</file>
    <file>docs/build-prompts/M08-workbench.md (the entire phase doc — the Background incl. the Carry-forward table, the nine-stage structure, Key constraints, all stages A.1–G.6 + V.1/V.2)</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
    <file>docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md (incl. the v1.8 coverage-policy reconciliation check that must be ticked)</file>
    <file>docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md (to read the Stage V retro's shape + [END] Decisions)</file>
    <file>docs/gap-analysis.md (the six-section entry template defined at the top of the file)</file>
    <file>docs/adr/0008-milestone-stage-v-verifier.md (the 🟢 tech-debt ledger convention)</file>
    <file>docs/adr/0019-tester-isolated-session-model.md + docs/adr/0020-builder-canvas-state-model.md (flip Proposed → Accepted in the M08 PR)</file>
  </read_first>

  <cumulative_reads>
    <codebase>the entire shipped codebase through M08 (cumulative across the M01–M07.5 merges + the M08 stages A,B,C,D1,D2,E,F1,F2,G + V + any D.fix)</codebase>
    <spec>agent-runtime-spec.md (end-to-end, focus on Phase 9 ~2495–2543 + §0b + §0d + §3 + §3a + §5a + §8.security + §9)</spec>
    <gap_analysis>docs/gap-analysis.md (ALL prior entries — M01, M02, M03, M03.5, M04, M05, M06, M06.5, M07)</gap_analysis>
    <retrospectives>docs/build-prompts/retrospectives/M08.*-retrospective.md (all of A, B, C, D1, D2, E, F1, F2, G, V — the closeout reads these; the verifier deliberately did NOT)</retrospectives>
    <summary>docs/build-prompts/retrospectives/M08-summary.md (authored as part of this stage)</summary>
    <tech_debt>docs/tech-debt.md (the cumulative TD-NNN ledger — the M07-IRL 🟢 #1/#4/#8 + any M08.V 🟢 findings should be present from V's run)</tech_debt>
    <coverage_policy>docs/coverage-policy.md (§A current exclusion set / §B baselines / §C history — the four-mirror reconciliation target)</coverage_policy>
  </cumulative_reads>

  <scope_locks ref="docs/build-prompts/M08-workbench.md" section="Key constraints"/>

  <gates milestone="M08"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md">
    <special_log>M08 delivered the Phase 9 Workbench / Builder Canvas + the sandboxed Tester + the Settings panel, and absorbed the entire post-M07 carry-forward backlog — the coupled M07.V Dec-6 set, the M07-IRL findings, the M06.5 IRL 🟡-1..4, and the M04 plan_loop driver. Aggregate per-primitive coverage outcomes (the new builder module + the Tester module within runtime-main). Record the protocol-level observations: M08 is the largest milestone (nine work stages vs the 1–5 of M01–M07) — did the A–G split with D1/D2 + F1/F2 hold each stage to a clean red→green unit at the M06/M07 cadence? Did the A→F1 construction-graph chain complete as designed (Stage A's `<construction_reachability_check>` mapped false → Stage F1 inverted to true → Stage V confirmed)? Cite the simplify_pass outcome (proposals surfaced + the approved/deferred split) and the coverage_policy_reconciliation (which gates changed, whether the builder module needed a new exclusion, all four mirrors synced, §B/§C appended).</special_log>
  </retrospective_requirements>

  <deliverables>
    <milestone_summary>docs/build-prompts/retrospectives/M08-summary.md (per SUMMARY-TEMPLATE.md; aggregates the nine per-stage retros A,B,C,D1,D2,E,F1,F2,G + the Stage V verifier retro; scores the three axes across stages; marks the milestone verdict; ticks the v1.8 coverage-policy reconciliation check)</milestone_summary>
    <gap_analysis_entry>docs/gap-analysis.md (append the new immutable M08 entry at the bottom; the six required sections, none optional; a gotchas_graduation subsection covering all nine work stages A,B,C,D1,D2,E,F1,F2,G; the eight MVP §M8 acceptance criteria + ADR-0019 + ADR-0020 cited with file:line in the Adherence-to-spec section; the Carry-forward section records — M07.V 🟡 #2/#3/#4/#5 RESOLVED, M07-IRL #5/#2/#3/#6/#7 RESOLVED, M06.5 IRL 🟡-1..4 RESOLVED, M04 plan_loop RESOLVED, and M06.5 IRL 🔴-1 now re-confirmable in the post-M08 IRL pass since Stage G unblocked the Promoted tier; the Spec-review section records the §1c-multi-session-vs-Tester-isolated-session clarification + the validate_framework command-return-vs-spec-"events" refinement)</gap_analysis_entry>
    <coverage_policy_reconciliation>M08 changed coverage gates: the new builder module enters the runtime-main ≥95 gate (Stage B), the Tester module enters it (Stage F1), plus any new OS-call-holdout --ignore-filename-regex exclusion a stage surfaced. Per the v1.8 four-mirror rule, append a docs/coverage-policy.md §C M08 milestone entry + the §B baseline entries for the modules that entered a gate, and verify the CLAUDE.md §5 exclusion-category list, the CLAUDE.md §6 cargo llvm-cov commands, and codecov.yml are byte-consistent. If Stage B's retro confirms the builder module is pure/seam/tempfile-tested and added no new exclusion, record that — only the §B/§C baseline entries are appended. Any drift between the four mirrors is a bug, fixed in this commit.</coverage_policy_reconciliation>
    <simplify_pass>
      <invoke skill="simplify" against="the milestone cumulative diff (M08.A..HEAD)"/>
      <surface kind="refactor_proposals" examples="builder-module duplication / Builder-vs-live-graph node-component parallelism / canvas-projection derivation seams / Tester-vs-smoke-session parallel surfaces / modules grown across the nine stages / premature abstractions"/>
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
      <stage_review id="D1"/>
      <stage_review id="D2"/>
      <stage_review id="E"/>
      <stage_review id="F1"/>
      <stage_review id="F2"/>
      <stage_review id="G"/>
      <!-- Stage V's special_log observations also feed the graduation decisions -->
    </gotchas_graduation>
    <special_check>Verify the V→closeout handoff: 🟡 findings from the Stage V retro carry into the gap-analysis Carry-forward section; 🟢 findings are already in docs/tech-debt.md from V's run; 🔴 findings (if any) were resolved by a D.fix iteration OR a new waiver-as-ADR before this closeout commit. The M07.V Dec-6 discharge (🟡 #2/#3/#5 discharged at F1; #4 at C) and the M07-IRL #5 close (G) are explicitly cited in the Adherence-to-spec + Carry-forward sections. M06.5 IRL 🔴-1 is recorded as re-confirmable in the post-M08 IRL pass (Stage G unblocked the Promoted tier).</special_check>
    <special_check>Run the v1.6 simplify_pass against the M08.A..HEAD cumulative diff; apply the maintainer-approved subset as a focused refactor commit BEFORE the PR opens; defer the remainder to docs/tech-debt.md. Run the v1.8 coverage_policy_reconciliation; the SUMMARY-TEMPLATE coverage-policy reconciliation check must be ticked.</special_check>
    <special_check>Confirm ADR-0019 + ADR-0020 are filed (at F1 / C respectively) and staged to flip Proposed → Accepted in the M08 PR; confirm the §1c-vs-Tester scope clarification (the Tester is a build-time sequential throwaway session, NOT the §0d-❌ §1c multi-session) and the validate_framework command-return §12 decision are recorded in the gap-analysis Spec-review section.</special_check>
  </gap_analysis_requirements>

  <append_only_verification>
    <local_check>the prior content of docs/gap-analysis.md must be a literal prefix of HEAD before commit (the M08 entry only appends at the bottom; no prior entry is edited, reordered, or deleted)</local_check>
    <ci_check name="gap-analysis-append-only">fails if any prior line is modified</ci_check>
  </append_only_verification>

  <three_artifact_review>
    <artifact>the code diff (cumulative across M08 stages A,B,C,D1,D2,E,F1,F2,G + the Stage V findings absorbed + any simplify-pass refactor commit)</artifact>
    <artifact>the per-stage retrospectives (M08.A through M08.G) + the Stage V verifier retro + the M08 summary</artifact>
    <artifact>the new gap-analysis M08 entry — flagged "IMMUTABLE once committed" (the eight MVP §M8 criteria + ADR-0019/0020 + the carry-forward closures cited)</artifact>
    <pushback_blocks_pr>true</pushback_blocks_pr>
  </three_artifact_review>

  <self_correction_budget>3</self_correction_budget>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M08-workbench.md" section="H.6 Commit Message"/>

  <approval_surface>
    <item>cross-machine state (build machine `git log --oneline main..HEAD` + `ls docs/build-prompts/retrospectives/M08.*-retrospective.md` + M08-summary.md)</item>
    <item>diff stat (the gap-analysis.md additions + M08-summary.md + the CHANGELOG.md [Unreleased] edit + the coverage-policy.md §B/§C additions + any simplify-pass refactor commit)</item>
    <item>the immutable gap-analysis M08 entry (the six sections; the eight MVP §M8 criteria + ADR-0019/0020 cited; the carry-forward closures — M07.V 🟡 #2/#3/#4/#5, M07-IRL #5/#2/#3/#6/#7, M06.5 IRL 🟡-1..4, M04 plan_loop — and M06.5 IRL 🔴-1 re-confirmable; the §1c-vs-Tester + validate_framework spec refinements)</item>
    <item>the coverage_policy_reconciliation: which gates changed (the builder module ≥95, the Tester module), whether a new exclusion was needed, the four mirrors synced byte-consistently, §B/§C appended (the SUMMARY check ticked)</item>
    <item>the simplify_pass outcome: the refactor proposals surfaced + the maintainer-approved subset (the focused refactor commit, if any) + the non-approved remainder deferred to docs/tech-debt.md</item>
    <item>the three-artifact review bundle (the cumulative code diff + the per-stage retros/summary + the new gap-analysis entry) + the draft M08 PR description (ADR-0019/0020 staged Proposed → Accepted) — do NOT open the PR yet, surface only</item>
    <item>explicit statement: "Stage M08.H is ready. I will not commit until you approve."</item>
  </approval_surface>
</closeout_stage_prompt>
```

### H.6 Commit Message

```
docs(closeout): M08 — gap-analysis + summary + simplify_pass + coverage-policy reconciliation

M08 summary (A,B,C,D1,D2,E,F1,F2,G + V) + the immutable gap-analysis
entry (six sections; gotchas_graduation A..G; the eight MVP §M8
criteria + ADR-0019/0020 cited; the Carry-forward section records
M07.V 🟡 #2/#3/#4/#5 RESOLVED, M07-IRL #5/#2/#3/#6/#7 RESOLVED,
M06.5 IRL 🟡-1..4 RESOLVED, M04 plan_loop RESOLVED, M06.5 IRL 🔴-1
re-confirmable post-M08; the §1c-vs-Tester + validate_framework
spec-phrasing clarifications). v1.6 simplify_pass against M08.A..HEAD
(approved subset applied). v1.8 coverage_policy_reconciliation: the
builder module ≥95 + the Tester-module gate synced across
docs/coverage-policy.md §B/§C + CLAUDE.md §5/§6 + codecov.yml.
PR drafted (not opened).

https://claude.ai/code/session_01E2eRs1knKT1wePJmxFTKTM
```

---

## Acceptance criteria coverage map (MVP §M8 → stages)

| MVP §M8 acceptance criterion | Stage(s) |
|---|---|
| 1 — Drag an Agent node onto empty canvas; set role/model/`allowed_*` inline; capability disclosure renders | D1 |
| 2 — Connect Agent→Skill → skill added to `allowed_skills` | D2 |
| 3 — Connect Agent→Agent → child `allowed_*` intersected with parent; UI surfaces narrowing | D2 (narrowing computed in B) |
| 4 — Click Validate → schema validation runs; errors at the offending node | B (validator) + D2 (badges) + E (Validate button) |
| 5 — Click Test → task → Tester modal → sandboxed session → graph + VDR + token + pass/fail | E (Test button) + F1 (backend) + F2 (modal) |
| 6 — Canvas \| JSON tabs; edit JSON; switch back shows the updated canvas | E |
| 7 — Save framework to disk → `framework.json` + companion `.md` | B (backend) + E (Save affordance) |
| 8 — Reload from disk → canvas reconstructs identical | B (backend) + E (Load affordance) |

The Tester being the production tool-driving session also discharges the spec Phase 9 line *"Capability violations during test … surfaced as test failures, not as live HITL prompts"* (F1.3.3) and *"Test runs do not write to any user data directory; results discarded on close"* (F1.3.2 / F2.3.5).

