# MVP v0.1.0 Windows Preview — Build Checklist

**Source of truth for scope:** `agent-runtime-spec.md` §0d Release Scope Matrix
**Reference frameworks:** `examples/aria/`, `examples/ralph/` (ralph deferred to v1.0)
**Branch:** developed on per-phase feature branches off `main`; merged via PR per §12 Engineering Charter
**Estimated scope:** 200–400 hours Claude execution + ~150 hours human direction, **3–6 months elapsed**

---

## Purpose

This document is the **build checklist** that turns §0d release scope into actionable phases. Every milestone has:
- **Deliverable** — what code/artifact lands
- **Acceptance criteria** — how we know it's done (testable)
- **Dependencies** — what milestones must precede it
- **Out of scope** — explicit non-deliverables for that milestone

The MVP succeeds when **both** §0d MVP success-criterion paths (novice and experienced) work end-to-end on a fresh Windows VM.

---

## Milestone Overview

| # | Milestone | Weeks | Deliverable | Gate |
|---|---|---|---|---|
| **M1** | Foundation | 1–2 | Cargo workspace + CI green + drone + runtime-core types | CI passes; drone heartbeats to SQLite |
| **M2** | Event pipeline alive | 3–4 | Tauri shell + AnthropicProvider + AgentEvent flow | Renderer logs events from a real Claude API call |
| **M3** | Live Graph | 5–6 | React Flow + all node types + VDR projection | Graph renders Anthropic call as nodes + edges in real time |
| **M4** | Plan + Verify + HITL + Budget | 7–8 | §3a + §4a + §6a + §2a + §1b | One end-to-end task: plan → approve → execute → verify → commit |
| **M5** | Gap detection + Capability enforcement | 9–10 | §4b + §8.security L1 + L2a + L3 + L4 (Novice + Promoted) + L5 basic | request_capability fires GapNode; capability_violation blocks; Promoted auto-accepts validated |
| **M6** | MCP basic | 11–12 | Phase 5 add/connect/list + per-server auth | Connect a real MCP server, agent uses its tools |
| **M7** | Registry import | 13 | Phase 7 import-by-URL + import-by-file + skills.lock | Paste GitHub raw URL of a skill.md → installed and loadable |
| **M8** | Workbench (Builder Canvas) | 14–17 | Phase 9 palette + drag-drop + JSON preview + Tester | Build a simple framework via canvas, save, reload, run |
| **M9** | Generators | 18–20 | Phase 8a/8b/8c with Novice review + Promoted auto-accept | Generate a tool, skill, agent via natural-language prompt; install |
| **M10** | First-run + polish | 21–22 | §14 onboarding + Settings + Help | Fresh user installs and reaches first session in <10 minutes |
| **M11** | Ship prep | 23–24 | Signed .msi + README + release artifact | Two-path success criterion (novice + experienced) on fresh Windows VM |

Total: ~24 weeks elapsed at sustained pace. Compresses with parallel work; expands with rework.

---

## M1 — Foundation (weeks 1–2)

> **Split into 4 sub-milestones** per the TEMPLATE.md scope-split rule (the single M1 prompt was 540 lines — too much for a fresh-session opening message). Each sub-milestone is its own branch and PR.
>
> - **M01.1** (~5–8h) — Workspace skeleton: Cargo.toml + empty crates + Tauri stub + CI green. See `docs/build-prompts/M01.1-workspace-skeleton.md`.
> - **M01.2** (~6–10h) — Type generation: xtask + typify + runtime-core types from schemas + drift check. See `docs/build-prompts/M01.2-type-generation.md`.
> - **M01.3** (~12–18h) — Drone Phase 1: heartbeat + snapshot + IPC + SIGTERM + 100% coverage. See `docs/build-prompts/M01.3-drone-implementation.md`.
> - **M01.4** (~4–6h) — Fuzz harness + workspace coverage + per-crate READMEs + cross-OS verification. See `docs/build-prompts/M01.4-fuzz-and-polish.md`.
>
> Sub-milestones are sequential; each PR must merge before the next session opens. M1 is "done" when all 4 have merged.

**Deliverable** (across all 4 sub-milestones)
- Cargo workspace at repo root with `crates/{runtime-core,runtime-main,runtime-drone,runtime-sandbox,xtask}` and `src-tauri/`
- `runtime-core` crate with types generated from `schemas/*.json` via `typify`
- `runtime-drone` crate implementing Phase 1 per spec, with 100% line coverage
- `xtask regenerate-types` subcommand + CI drift check
- Fuzz harness for `drone_command_decode` + nightly fuzz workflow
- Per-crate READMEs documenting the public API surface
- CI pipeline (`.github/workflows/ci.yml`) running fmt + clippy + test + audit + deny + coverage + fuzz-smoke on Linux/macOS/Windows
- `rust-toolchain.toml` pinning Rust version

**Acceptance criteria** (final, across all 4 sub-milestones)
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] CI green on all 3 OS targets × stable + MSRV
- [ ] `runtime-drone --session-id test --db-path /tmp/d.sqlite --ipc-socket /tmp/d.sock` starts, writes heartbeat rows to SQLite, accepts `SnapshotNow` command via socket, accepts `GracefulShutdown`, handles SIGTERM with emergency snapshot
- [ ] Drone test coverage **100%** (cargo-llvm-cov per-package); workspace coverage ≥80%
- [ ] `runtime-core` types compile and round-trip-serialize via proptest
- [ ] `cargo xtask regenerate-types --check` passes (no schema/type drift)
- [ ] `cargo +nightly fuzz run drone_command_decode -- -max_total_time=30` runs without panic in CI
- [ ] All `pub` APIs in `runtime-core` and `runtime-drone` doc-commented with compile-checked examples

**Dependencies** — none (root milestone)

**Out of scope** — anything that isn't workspace skeleton + types + drone + fuzz/coverage/docs (each sub-milestone has its own out-of-scope list)

---

## M2 — Event pipeline alive (weeks 3–4)

**Deliverable**
- `runtime-main` crate with `AgentSdk` + `LLMProvider` trait + `AnthropicProvider` (direct HTTP+SSE)
- Tauri shell (`src-tauri/`) with allowlisted command `run_smoke_session`
- Skeleton renderer (`src/`) — single page that lists `AgentEvent`s as they arrive
- Tauri typed IPC: `app.emit("agent_event", ...)` flowing to renderer
- Main↔drone IPC over Unix socket (Linux/macOS dev) or Windows named pipe

**Acceptance criteria**
- [ ] User clicks "Run smoke test" in renderer; main calls Anthropic with a hardcoded prompt; renderer shows `agent_spawned` → `tool_invoked` (LoadSkill) → `stream_text` chunks → `agent_complete`
- [ ] Drone snapshots fire on `task_started` events (none yet at this stage; just verify the wiring)
- [ ] Anthropic API key read from OS keychain via `keyring` crate
- [ ] Provider integration tests use `wiremock` for offline CI; real-API smoke is a manual `cargo test --features integration` run
- [ ] No third-party SDK in `Cargo.toml` for Anthropic — direct `reqwest` + `eventsource-stream` only

**Dependencies** — M1

**Out of scope** — graph rendering (M3), plan model (M4), generators (M9)

---

## M3 — Live Graph (weeks 5–6)

**Deliverable**
- React Flow integration in renderer
- All node types per spec §3: `AgentNode`, `ToolNode`, `SkillNode`, `MCPNode`, `GapNode`, `HITLNode`, `PlanNode`, `TaskNode`, `VerifyNode`, `HookNode`, `FrameworkNode`
- Animated edges (active call) + dashed edges (skill load)
- Click-to-inspect side panel
- Token-spend visualization (node weight)
- VDR projection from signal stream — populated table + simple SQL inspector

**Acceptance criteria**
- [ ] Repeat M2 smoke test; renderer shows graph instead of event log
- [ ] Click any node → side panel shows full event payload + correlated VDR row
- [ ] Graph reconstructs after page reload (state from SQLite)
- [ ] React + React Flow + Zustand for state; no Redux, no MobX
- [ ] Renderer Vitest coverage ≥80% on graph reducers

**Dependencies** — M2

**Out of scope** — plan/task nodes wired to data (data lands in M4); MCP node connecting to a real server (M6)

---

## M4 — Plan + Verify + HITL + Budget (weeks 7–8)

**Deliverable**
- §3a Plan / Task primitive — `Plan`, `Task` types + `fresh_context_per_task` loop policy + 11 plan/task events
- §4a Verify hooks (post_task, pre_commit, post_file_edit) + Rails (hard/soft) + dont_touch globs + revert_to_snapshot drone command
- §6a HITL primitive — full 9-trigger set + 3 UI variants (panel/modal/toast) + 3 built-in notifiers (terminal_bell, desktop, sound)
- §2a Budget primitive — all 4 actions (warn/downshift/hitl/hard_stop) + downshift_hook
- §1b Recovery from snapshot — resume rebuilds history, tool-call-uncertain prompt
- ApprovalPanel for plan approval gate

**Acceptance criteria**
- [ ] Load `examples/aria/framework.json` (stripped to v0.1-compatible: STANDARD mode hardcoded, no MCP tools, no generators referenced); orchestrator agent spawns; planner generates a 3-task plan; HITL approval panel surfaces; user approves; tasks execute one at a time
- [ ] Each `task_completed` triggers `post_task` hook (`bash .aria/verify.sh` shim that returns 0 on Windows via PowerShell wrapper); pass → next task; fail with `on_failure: rollback` → drone reverts to snapshot, task retries
- [ ] Hit `failure_count >= max_failures` → HITL escalation panel opens; user picks retry/skip/abort
- [ ] Budget threshold breach → `budget_warning` toast at 50%, `budget_downshift` triggers (manual hook for now: hardcoded opus→sonnet→haiku); `budget_suspended` opens HITL approval at 90%
- [ ] User closes app mid-session; reopens; recovery dialog offers resume; resumed session continues from last snapshot with task pointer reset

**Dependencies** — M3

**Out of scope** — gap detection (M5), capability enforcement (M5), generators (M9)

---

## M5 — Gap detection + Capability enforcement (weeks 9–10)

**Deliverable**
- §4b Detection — Layer 1 (static at load + spawn time) + Layer 2 (`request_capability` meta-tool auto-injected into every agent's tool list)
- §8.security L1 Capability disclosure — `capabilities` block enforced as required for generated artifacts; rendered plain-English in UI
- §8.security L2a Application-level enforcement — every operation passes through `crates/runtime-core/src/capability.rs`; `capability_violation` event fires + HITL grant prompt
- §8.security L3 Sandboxed validation — `runtime-sandbox` process spawned for L3 runs; schema check + declared examples + capability-bound execution + red-flag scan
- §8.security L4 Tier system — Novice (manual review every install) + Promoted (auto-accept validated artifacts within bounds; blocked from `shell:true` and `network:["*"]`); Operator stays deferred to v1.0
- §8.security L5 Provenance + audit — `provenance` block in artifacts; `skills.audit.jsonl` append-only, hash-chained, secret-redacted
- Severity matrix wired: `tool_missing` suspends, `skill_missing` warns
- GapPanel UI + Resume button

**Acceptance criteria**
- [ ] Load a framework with a deliberately-missing tool reference; runtime emits `tool_missing` at spawn time; GapPanel opens with "Install tool" link; session is `suspended`
- [ ] Add the missing tool (manual file copy is fine here); `gap_resolved` fires; session resumes from drone snapshot
- [ ] Agent calls `request_capability { kind: 'tool', name: 'unknown_thing' }`; same flow
- [ ] Skill missing at load fires warning toast; session continues; user can install async
- [ ] Generated tool that declares `tools_called: ["WebFetch"]` then attempts `Bash` is blocked at L2a; `capability_violation` event + HITL prompt
- [ ] Promoted tier: install a generated artifact with safe capabilities (`shell: false`, `network: []`) → auto-installs after L3 pass
- [ ] Promoted tier: install a generated artifact with `shell: true` → blocked from auto-install; falls back to Novice review even though user is Promoted
- [ ] `skills.audit.jsonl` records every install/violation/tier-change
- [ ] L3 sandbox process is a `runtime-sandbox` child of the drone (not main); SIGKILL after `timeout_ms`

**Dependencies** — M4

**Out of scope** — full L2b OS sandboxing (process boundary only in v0.1; full seccomp/landlock/sandbox-exec is v1.0); Operator tier (v1.0)

---

## M6 — MCP basic (weeks 11–12)

**Deliverable**
- Phase 5 MCP Manager — add server by URL or local path, test connection, list discovered tools, per-server auth in keychain
- §5a Tool namespace resolution — canonical `<server>__<tool>` names + short-name aliasing + `mcp_aliases` framework JSON field
- MCPNode in graph with live connection status
- MCP integration uses `rmcp` crate if feature-complete enough; fallback is direct JSON-RPC over stdio

**Acceptance criteria**
- [ ] User adds an MCP server (e.g., a local stdio MCP for filesystem access) via Settings → MCP Servers → Add
- [ ] Connection tested in-UI; success surfaces tool list; failure shows error
- [ ] Agent in a running session calls a tool from the MCP server; ToolNode renders inside MCPNode in graph; tool result streams back
- [ ] Two MCP servers exposing same-named tool — short name fails with `tool_alias_ambiguous` error listing canonicals; user adds `mcp_aliases` to framework JSON to disambiguate
- [ ] MCP server crashes mid-session — MCPNode goes offline; calls to its tools route through gap flow

**Dependencies** — M5 (gap flow needed for MCP tool failures)

**Out of scope** — multi-server collision UI (v1.0); MCP discovery / browsing (v1.0); auto-reconnect with exponential backoff beyond default

---

## M7 — Registry import (week 13)

**Deliverable**
- Phase 7 Registry — import-by-URL dialog, import-by-file file picker
- `skills.lock` file at framework root with `{ name@version: { kind, source, content_hash, installed_at, tier_at_install, validation_report_id } }`
- Hash validation on every artifact load; `artifact_hash_mismatch` event blocks with re-install prompt
- `Import` panel in Builder: paste a GitHub raw URL → fetch → schema-validate → L3 sandbox → tier-gate review → install → `skills.lock` updated
- Same flow for skill / tool / agent / MCP server config (latter installs into MCP Manager from M6)

**Acceptance criteria**
- [ ] User pastes raw GitHub URL of a `skill.md` (e.g., from a community repo); skill is fetched, validated against `schemas/skill.v1.json`, L3-sandbox-tested, tier-gated, installed; `skills.lock` updated; appears in Builder palette
- [ ] User picks a local `tool.md` file; same flow
- [ ] Tampered file (hash mismatch on subsequent load) → load blocked with prompt to reinstall or remove
- [ ] `skills.lock` is reproducible across machines (commit it to user's framework repo for team consistency)
- [ ] No Anthropic-upstream search UI in v0.1 — just URL/file import

**Dependencies** — M5 (L3 validation), M6 optional (MCP server import uses MCP Manager from M6)

**Out of scope** — Anthropic upstream search UI (v1.0); pluggable community registries (v2.0); Sigstore signature verification (v1.0)

---

## M8 — Workbench (Builder Canvas) (weeks 14–17)

**Deliverable**
- Phase 9 Visual Canvas at three-panel layout per spec §9
- Palette (left): Tools / Skills / Agents / HITL / Hooks tabs; filterable; drag-drop
- Canvas (center): React Flow node-and-edge editor with capability narrowing applied automatically on Agent→Agent edges
- Inspector (right): live `framework.json` preview with diff view; capability summary across whole framework; Validate + Test buttons
- Tester modal: load framework from canvas without saving; isolated session in separate SQLite; smaller graph pane; full results
- Canvas state ↔ JSON state two-way binding (canvas edits update JSON; JSON edits re-render canvas)
- Schema validation runs continuously; errors as red badges on offending nodes

**Acceptance criteria**
- [ ] User drags an Agent node onto empty canvas; sets role, model, allowed_tools/skills via inline properties; capability disclosure renders below in plain English
- [ ] Connect Agent → Skill (drag edge) → skill name added to `allowed_skills`
- [ ] Connect Agent → Agent (parent → child spawn) → child's `allowed_*` automatically intersected with parent's; UI surfaces narrowing decisions
- [ ] Click "Validate" → full schema validation runs; surface any errors at offending node
- [ ] Click "Test" → enter a task description → Tester modal opens → sandboxed session runs → graph + VDR + token spend + pass/fail surface
- [ ] Switch to JSON view (tabs: Canvas | JSON); edit JSON directly; switching back shows updated canvas
- [ ] Save framework to disk → file at chosen path with valid `framework.json` + companion `.md` files for any new skills/tools/agents
- [ ] Reload from disk → canvas reconstructs identical to save state

**Dependencies** — M3 (graph rendering primitives), M4 (plan primitive for Tester), M5 (capability enforcement for Tester sandboxing)

**Out of scope** — generator integration (M9 wires it in); multi-framework comparison view (v1.0); plugin node types (v2.0)

---

## M9 — Generators (weeks 18–20)

**Deliverable**
- Phase 8a Tool Writer — generate `tool.md` with `mcp_binding` (against an installed MCP server's tool schema) or `inline_implementation` (declarative decision table); never `shell_binding` in v0.1 generated output
- Phase 8b Skill Writer — generate `skill.md` instruction-set markdown with frontmatter (capabilities, mode_variants, triggers)
- Phase 8c Agent Composer — generate framework JSON `agents[]` entry composing existing tools + skills; capability narrowing from parent enforced
- Generator UI integrated into Builder: "Generate Tool" / "Generate Skill" / "Generate Agent" buttons in palette + canvas
- Manual review screen for Novice tier: full diff + capability disclosure plain-English + L3 validation report + Install/Reject/Edit
- Auto-accept toast for Promoted tier (when within bounds)
- Provenance block populated automatically: generator, model, prompt_hash, generated_at, validated_at, content_hash

**Acceptance criteria**
- [ ] Click "Generate Tool" → describe in natural language ("a tool that fetches GitHub PR comments by PR number") → optionally point at an installed MCP server → Tool Writer generates a `tool.md`
- [ ] Generated `tool.md` schema-validates against `schemas/tool.v1.json`
- [ ] L3 sandbox runs declared examples; capability check confirms artifact stays within `capabilities` block
- [ ] Novice user: review screen surfaces; user clicks Install; artifact appears in palette; `skills.lock` updated; audit entry written
- [ ] Promoted user (with safe capabilities): auto-accept toast appears; artifact installed; review accessible via toast link
- [ ] Promoted user attempts to generate something that declares `shell: true`: gets Novice-style review screen even though they're Promoted (per L4 forbidden rule)
- [ ] Generator output is deterministic given same prompt (model temperature pinned for reproducibility); two generations from same prompt produce identical content_hash
- [ ] Reject button discards artifact; logs decision to audit

**Dependencies** — M5 (L1/L2a/L3/L4/L5), M7 (skills.lock), M8 (Builder Canvas integration)

**Out of scope** — Operator tier (v1.0); cross-generator composition (e.g., "generate this tool, then this skill that uses it" as a single workflow — v1.1); template gallery (v1.0)

---

## M10 — First-run + polish (weeks 21–22)

**Deliverable**
- §14 First-Run state machine: Welcome → API key → Import or skip → First session prompt → Running session
- "Build your first agent" guided path: 5-step interactive overlay walks novice through generating a tool + skill, wiring them, running a Test
- Settings panel: API key (read/edit), Tier (Novice/Promoted toggle with one-time warning), Privacy (data export, delete-all-local), Updates (off in v0.1, surfaced as "v1.0 feature"), MCP Servers (from M6), Frameworks (list + activate)
- Help menu: Show me around (60-second graph tour from §14), Open spec, Open this build's MVP doc, Report issue (links to GitHub), About (version + license + signed cert info)
- Session recovery dialog wired to first-run flow (offered on launch if any session was interrupted)
- Keyboard shortcuts: cmd+, for Settings; cmd+shift+S for power-user skip; cmd+? for Help
- Empty states throughout: "No skills installed yet — try Import or Generate"
- Loading states + skeleton placeholders throughout
- Error states: API key invalid, MCP unreachable, capability validation failed — all with clear next-step copy

**Acceptance criteria**
- [ ] Fresh Windows VM, .msi installed, app opens — Welcome screen appears in <3 seconds
- [ ] User completes API key step (test-connection succeeds against real Anthropic), reaches first session in <10 minutes total
- [ ] "Build your first agent" overlay completes successfully: user generates a Tool, generates a Skill, wires them, runs Test, sees pass — without reading the spec or external docs
- [ ] Cmd+shift+S from Welcome bypasses to empty Builder canvas (power-user skip works)
- [ ] Settings → Data → Export creates `.aria-runtime-export.tar.gz` with everything except secrets
- [ ] Settings → Data → Delete All Local Data wipes `$DATA_DIR` after confirmation
- [ ] Closing app mid-session, reopening → recovery dialog with snapshot timestamp + Resume/Inspect/Discard
- [ ] Every error state has a copy-pasteable next-step instruction
- [ ] Keyboard nav works for the entire onboarding flow (no mouse required)

**Dependencies** — M9 (generators integrated); M8 (Builder); §13 (privacy controls)

**Out of scope** — tutorial videos (v1.0); sample completed sessions to walk through (v1.0); multi-language welcome (v2.0)

---

## M11 — Ship prep (weeks 23–24)

**Deliverable**
- `examples/aria/` stripped to v0.1-compatible (mode hardcoded to STANDARD, no MCP-dependent tools by default — but loadable into v0.1)
- `cargo tauri build` produces unsigned `.msi` for Windows x64 (paid code-signing cert deferred — see "Why unsigned" below)
- SHA-256 checksum generated for the `.msi` and published in release notes
- Sigstore provenance attestation via GitHub Actions OIDC (free; cryptographically attests "GitHub Actions for this repo built this binary from this commit")
- README.md (root) updated for v0.1 (replaces shell ARIA README, which moves to `archive/aria-shell/README.md`); includes prominent SmartScreen-warning explainer + checksum verification instructions
- SECURITY.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md finalized
- LICENSE (Apache 2.0)
- CHANGELOG.md with v0.1.0 entry
- `.github/workflows/release.yml` automates build + SHA-256 + Sigstore attestation + SBOM + GitHub Release on tag push
- ADRs 0001–0003 in `docs/adr/`
- v0.1.0 git tag + GitHub Release with unsigned .msi + SHA-256 + Sigstore attestation + SBOM (CycloneDX) attached

**Why unsigned**

EV code-signing certificates run $300–600/year and require business verification (LLC + notarized identity + USB hardware token). For a v0.1 OSS project with no validated audience, that's premature spending and procurement friction. Most successful OSS desktop tools ship unsigned at first release; trust comes from SHA-256 checksums + Sigstore provenance + transparent build process + the project's public reputation over time. Paid signing gets revisited at v0.5 or v1.0 when adoption is proven (or when a sponsor/employer offers to cover it). See ADR-0004 for the full reasoning.
- 30-90 second screen recording for launch

**Acceptance criteria — the MVP success criterion (BLOCKING)**

This is the ship gate. v0.1.0 ships only when **both** paths complete on a fresh Windows VM:

**Novice path** — fresh Windows VM, no prior knowledge:
- [ ] Download `.msi` from GitHub Release; verify SHA-256 against the release notes (matches → continue; mismatch → stop and report)
- [ ] Click through the SmartScreen warning ("More info" → "Run anyway"); the warning is expected and documented in the README
- [ ] Install + launch
- [ ] Welcome → API key (entered, test-connect succeeds) → "Build my own" → empty canvas
- [ ] Generate Tool ("fetch URL contents") → review → install
- [ ] Generate Skill ("summarize web articles") → review → install
- [ ] Drag Skill into existing orchestrator agent's `allowed_skills`; drag Tool into `allowed_tools`
- [ ] Click Test → enter "summarize https://example.com" → graph runs → Tester reports pass with sensible output
- [ ] Total elapsed: <30 minutes; no JSON edited; no spec read

**Experienced path** — same VM, after wiping state:
- [ ] Same install
- [ ] Welcome → API key → "Use ARIA template"
- [ ] Switch tier to Promoted (warning shown, accepted)
- [ ] Import skill from URL: paste GitHub raw URL of a known-good third-party `skill.md`
- [ ] Skill installed via Promoted auto-accept (within bounds); appears in palette
- [ ] Open a real codebase in another window; start a session against it via "Start session" with task description
- [ ] Session runs; orchestrator → planner → analyzer → implementer pipeline executes
- [ ] At some task, agent calls `request_capability { kind: 'tool', name: 'something' }` → GapPanel
- [ ] Click "Generate Tool" inline; describe; Promoted auto-accept; resume
- [ ] Session completes successfully; report-writer fires session_end hook with summary
- [ ] Audit log inspectable; full VDR queryable

**Other ship gates:**
- [ ] All §12 quality gates pass on `main` branch HEAD
- [ ] CHANGELOG complete and dated
- [ ] Release artifact: SHA-256 generated, Sigstore attestation present, signature verifies via `cosign verify-blob` (paid code-signing deferred per ADR-0004)
- [ ] SBOM attached to release
- [ ] README links work; LICENSE displays in app's About dialog
- [ ] No known critical bugs in issue tracker

**Dependencies** — every prior milestone

**Out of scope** — anything in v1.0 column of §0d release scope matrix

---

## Demo recording plan (the launch video)

Recorded after M11 acceptance, before public launch post.

**Duration:** 60–90 seconds. Single take preferred; 2–3 cuts max.

**Script:**
1. **0–10s:** Cold open — fresh Windows desktop. App icon double-clicked. Welcome screen.
2. **10–20s:** Skip Welcome (cmd+shift+S → power-user mode). Empty Builder canvas.
3. **20–35s:** Drag a pre-existing Agent onto canvas. Click "Generate Tool" → describe ("summarize unread emails by sender") → review screen with capability disclosure → Install.
4. **35–55s:** Drag tool onto agent. Click Test → enter task → Live graph runs (orchestrator → planner → tasks → tools). VerifyNode flashes green. Result panel shows summary.
5. **55–80s:** Side-by-side: same task, same agent, capability_violation simulated — agent attempts undeclared shell call. GapPanel opens. Tier-gated grant prompt: Block (default).
6. **80–90s:** Cut to GitHub repo open. Cursor on Releases tab. v0.1.0 highlighted. End frame: project name + license + repo URL.

**No voiceover.** Captions only — short, factual, no marketing language. ~80 chars max per caption.

**Recording tools:** OBS Studio for capture; ScreenToGif or LICEcap for short clips embedded in README; YouTube unlisted for the full 90s.

**Failure modes to avoid:**
- Cuts that hide what the runtime is actually doing (lose trust)
- Sped-up footage (looks fake; users can't tell what's real)
- Stock-music montage (signals marketing, not engineering)
- Logos / branding that doesn't yet exist (signals premature scaling)

---

## Pre-ship hygiene checklist (per §12 + OSS launch best practices)

Block release until ALL of these are checked:

### Code quality
- [ ] CI green on Linux/macOS/Windows for the release commit
- [ ] Coverage ≥80% line, 100% on safety primitives (drone, capability enforcer, plan state machine, snapshot/recovery)
- [ ] `cargo audit` and `npm audit`: zero high/critical
- [ ] `cargo deny check`: passing
- [ ] All public Rust API has doc comments; rustdoc clean
- [ ] All public TS API has typedoc; clean
- [ ] No `#[allow(...)]` or `// @ts-ignore` without an issue link

### Security
- [ ] SECURITY.md present with disclosure flow
- [ ] `docs/SECURITY.md` threat model up to date with what v0.1 actually does (and doesn't — L2b best-effort, Operator tier missing)
- [ ] Unsigned `.msi` built reproducibly; SHA-256 generated; Sigstore attestation produced via GitHub Actions OIDC; `cosign verify-blob` confirms attestation. Paid code-signing deferred — see ADR-0004 for rationale and the trigger criteria for revisiting.
- [ ] No secrets committed to repo (gitleaks scan)
- [ ] SBOM (CycloneDX) generated and attached to release
- [ ] No `.env` files in repo; `.gitignore` covers them

### Documentation
- [ ] README.md is honest, under 300 lines, has a screen recording or animated GIF, clear "what works / what doesn't" section
- [ ] CONTRIBUTING.md walks through clone → setup → first build → first test → first PR
- [ ] CODE_OF_CONDUCT.md (Contributor Covenant 2.1)
- [ ] CHANGELOG.md follows Keep-a-Changelog format
- [ ] LICENSE (Apache 2.0)
- [ ] NOTICE file with third-party Apache-licensed dep attributions
- [ ] AI-assisted development disclosed in README (per launch authenticity principles)
- [ ] All TODOs / FIXMEs in code link to issues

### Distribution
- [ ] GitHub Release v0.1.0 created with .msi attached
- [ ] SBOM attached
- [ ] `skills.lock` example committed in `examples/aria/`
- [ ] First-run UX tested on a fresh Windows VM (genuinely fresh, not your dev machine)
- [ ] Two-path success criterion executed end-to-end on that fresh VM by someone who hasn't seen the project (your son or another tester)

### Communication
- [ ] Launch post drafted (see `docs/launch/`)
- [ ] Demo video recorded
- [ ] Roadmap published (link to §0d v1.0 column)
- [ ] No "coming soon" promises without dates

---

## Risk register

Risks ordered by probability × impact. Each has a mitigation.

| # | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Anthropic API breaking change mid-build | low | high | LLMProvider trait abstracts; pin SDK version; test against `wiremock` in CI; small surface (HTTP+SSE direct) is easier to fix than a SDK shim |
| R2 | React Flow can't handle large graphs (100+ nodes) | medium | medium | v0.1 sessions are small; profile early at M3; if needed, use clustering/collapse; React Flow has known patterns for this |
| R3 | Tauri webview behavior diverges across OS | low (Windows-only in v0.1) | medium | Test on Windows 10 + Windows 11 + Windows Server in CI; v1.0 multi-OS port is when this risk realizes |
| R4 | Windows SmartScreen warns on unsigned .msi, friction for novices | high | low | Documented in README + first-run UX warns user; SHA-256 + Sigstore attestation give technically-inclined users verifiable provenance; paid EV signing revisited at v0.5+ per ADR-0004 |
| R5 | Generator output quality is inconsistent | high | medium | Pin model temperature; use deterministic seeds; capability validation L3 catches most failures; manual review at Novice tier; iterate prompts in M9 |
| R6 | Capability enforcement L2a has a bypass | medium | high | Property tests + fuzz testing in M5; security review before M11; document L2b is v1.0 publicly |
| R7 | SQLite WAL contention under multi-process load | low (single-session in v0.1) | low | Single-session simplifies; busy_timeout handles transients; v1.0 multi-session needs more rigor |
| R8 | First-run UX is too complex for novices | medium | high | User-test with at least 3 non-technical people during M10; iterate copy; reduce decision points |
| R9 | Generator hallucinates dependencies / undeclared capabilities | medium | high | L3 sandbox catches; static red-flag scan; prompt engineering in M9; explicit "what the generator can produce" docs |
| R10 | OSS launch attracts trolls / bad-faith issues | medium | low | Issue templates + clear contribution policy + maintainer onboarding doc; ignore + label rather than engage |
| R11 | Son or other Linux/macOS porter not available | low | low | v1.0 multi-OS is a v1.0 problem; v0.1 ships Windows-only by design; community contribution welcomed |
| R12 | Anthropic deprecates the model versions used | low | medium | Model IDs in framework JSON not constants; runtime uses `provider.list_models()`; framework can be updated without rebuild |

---

## Definition of done (per phase)

Adapted from §12 Engineering Charter; applied to every milestone.

A phase is done when:
1. **Code lands on `main`** via squash-merge from a feature branch.
2. **CI is green** on the merge commit (all OSes, all gates).
3. **Test coverage** ≥80% line, 100% on any safety-primitive code added.
4. **Doc tests / examples** for any new public API.
5. **ADR filed** for any §0a matrix-row primitive or schema change.
6. **CHANGELOG entry** under "Unreleased" header.
7. **Acceptance criteria** in this doc all checked.
8. **No new clippy or rustdoc warnings** vs prior `main`.
9. **No new third-party deps** without `cargo deny` review.
10. **Manual smoke test** on the contributor's local machine (Windows for the moment) — does the milestone behave as the acceptance criteria say?

---

## How this doc is used

- **At sprint planning:** pick the next unstarted milestone; estimate; assign.
- **During development:** acceptance-criteria checkboxes in the relevant milestone are kept up to date in PR descriptions.
- **At release time:** M11 "MVP success criterion" is the ship gate.
- **In post-mortems:** review whether risk register predictions held; update for next time.

This doc evolves as we learn. Edits to milestone scope require an ADR; reordering or splitting milestones does not.

