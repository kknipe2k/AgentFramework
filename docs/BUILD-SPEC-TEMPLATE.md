# Build Spec Template

> This file defines the **shape** of the `BUILD-SPEC-v0.1-*.md` files produced
> by the build prompts. Any fresh Claude Code session that runs
> `BUILD-PROMPT-architecture.md` or `BUILD-PROMPT-milestones.md` must produce
> output matching this template exactly — sections, order, required content.

**Two spec documents are produced:**

1. `BUILD-SPEC-v0.1-architecture.md` — the static "what are we building"
   document. Produced first. Answers structural questions.
2. `BUILD-SPEC-v0.1-milestones.md` — the sequenced, test-first "how do we
   build it" document. Produced second. References the architecture doc.

Both documents are mandatory. Neither document is permitted to contain
implementation code — only types, interfaces, schemas, test descriptions,
commands, and prose.

---

## `BUILD-SPEC-v0.1-architecture.md` — Required Sections

### 1. Executive Summary
One paragraph. What we're building, for whom, in how many hours.

### 2. Goals (numbered, measurable)
Each goal must be measurable. "Ship a polished UI" is not acceptable.
"Ship a Windows installer under 150MB that passes SmartScreen" is.

### 3. Non-Goals (numbered)
What v0.1 explicitly does NOT do. Prevents scope creep. Must include at
least: no Visual Builder, no Research Assistant framework, no Visual
ARIA framework, no Committees, no Optimizer, no Skill Writer, no MCP
Manager UI, no macOS support.

### 4. Target User & Acceptance Criteria
Named user persona (from `DESIGN-DECISIONS.md` Item 1). The specific
behaviors a v0.1 must exhibit for the user to say "this isn't
embarrassing."

### 5. Architecture Overview
Diagram (ASCII art) of the runtime tiers. Condensed version of Item 5's
three-tier survival system. Labeled boxes showing:

- Electron main process
- Renderer process (React UI)
- Drone Core (child)
- Process Supervisor (child)
- SQLite persistence
- Anthropic API (external)

### 6. Tech Stack (with exact versions, verified via web)
Table format:

| Layer | Package | Version | Verified |
|---|---|---|---|
| Shell | `electron` | X.Y.Z | YYYY-MM-DD via npm registry |
| ... | ... | ... | ... |

Every version must be verified by web search or npm registry lookup
within the session producing the spec. Document date of verification.
If a package has known incompatibilities (e.g., `better-sqlite3` vs.
specific Electron versions), note them here with source links.

### 7. Module Map
Exhaustive list of every file v0.1 will create. Grouped by phase.
Format per entry:

```
src/main/drone/core/heartbeat.ts
  purpose: Emit heartbeat events every 5s, persist to SQLite
  depends on: src/main/db/schema.ts, src/main/drone/core/clock.ts
  tested by: tests/unit/drone/core/heartbeat.test.ts
             tests/integration/drone/core/heartbeat-persistence.test.ts
  LOC budget: ~80 lines
```

Every file must have tested-by entries. No file is permitted without
corresponding tests. No file exceeds 300 LOC without justification.

### 8. Interfaces & Types
All public TypeScript interfaces, in their final form:

- `RuntimeContext`
- `LLMProvider` + `LLMRequest` + `LLMStreamEvent`
- `FrameworkManifest`
- `AgentRoleSpec`
- `MemoryClient` (shortTerm, longTerm)
- `CorpusClient` (declared but stubbed in v0.1)
- `HitlClient`
- `PolicyEnforcer`
- `AgentEvent` (full discriminated union)
- `BudgetCap`, `BudgetSettings`, `CostBreakdown`
- `OrchestrationStrategy` (and the v0.1 ReAct implementation's contract)
- `DroneEvent`, `DroneCommand`, `SupervisorEvent`, `SupervisorCommand`

Types only. No implementations. Inline doc comments explaining semantics
of any non-obvious field.

### 9. Data Models
SQLite DDL for all tables needed by v0.1:

- `sessions`
- `snapshots`
- `events` (every `AgentEvent` persisted for replay/debug/Optimizer)
- `token_usage`
- `budget_settings`
- `memory_short_term`
- `memory_long_term`
- `heartbeats`
- `outcomes` (for eventual Optimizer, populated from v0.1)

Every table: columns, types, indexes, constraints, comments. Migration
strategy (even if v0.1 only has v1 schema).

### 10. Event Flow Diagrams
For each of these flows, produce an ASCII sequence diagram:

1. **Happy path:** Open app → pick Quick Agent → type task → run completes
2. **Budget cap hit:** Run proceeds until per-run cap reached → pause → user resumes
3. **Drone core restart of supervisor:** Supervisor crashes → Core detects → restarts → sessions recover
4. **HITL interrupt:** User pauses an agent mid-run → types correction → agent resumes

### 11. Security & Secrets
- API key storage (keytar)
- Renderer ↔ main IPC surface (contextBridge)
- Skill sandboxing (not in v0.1 but noted as future)
- Policy enforcement gates in `RuntimeContext`
- What the child processes can and cannot access

### 12. Observability
- Logging strategy (pino, winston, or stdout JSON — decided with rationale)
- Log levels and what each captures
- Event persistence as the canonical audit log
- How a dev would debug a failing test or a production crash

### 13. Accessibility (non-optional)
Minimum bar: keyboard navigation works for the full happy path; screen
reader labels on all interactive elements; color contrast passes
WCAG AA. Tested by Playwright axe integration.

### 14. Risk Register
Ranked list of risks with mitigations:

- `better-sqlite3` native build failures on Windows
- Anthropic SDK breaking changes during development
- React Flow perf regressions on long sessions
- Code signing / SmartScreen / Defender false positives
- API rate limits during testing
- Flaky tests from model non-determinism
- Scope creep from "while we're at it"

Each risk: likelihood (L/M/H), impact (L/M/H), mitigation.

### 15. Open Questions
Pull from `OPEN-QUESTIONS.md` only the items that block or shape v0.1.
Mark resolved ones with answers; mark unresolved ones with proposed
default and "needs user confirm."

---

## `BUILD-SPEC-v0.1-milestones.md` — Required Sections

### 1. Overview
One paragraph. "14 milestones, approximately 88 hours, test-first, each
milestone ending with a passing `npm run verify`."

### 2. Test Philosophy (NON-NEGOTIABLE — verbatim required)

> **No implementation code is written before a failing test exists for it.**
>
> **Every milestone has three test tiers:**
>
> 1. **Unit tests** — fast, isolated, one function or class. Mocked deps.
> 2. **Integration tests** — multiple modules together, real SQLite, mocked
>    LLM. Exercise actual data flow, not just function signatures.
> 3. **Behavioral tests** — user-visible outcomes. End-to-end within a
>    milestone's scope. "When X happens, Y is observably true in the DB,
>    the UI, or the event stream."
>
> **Proof of life is not acceptance.** A test that asserts "the function
> returns without throwing" is not a test. A test must assert the specific
> observable effect the function is supposed to produce.
>
> **Examples of unacceptable tests:**
>
> - `expect(heartbeat()).not.toThrow()` — proves nothing
> - `expect(parser.parse(validJSON)).toBeDefined()` — proves nothing
> - `expect(component.render()).toBeTruthy()` — proves nothing
>
> **Examples of acceptable tests:**
>
> - After starting the drone core and waiting 6 seconds, the heartbeats
>   table contains at least one row with timestamp within the last 2
>   seconds and `status = "ok"`.
> - Given a mock LLM that streams `{type: 'text_delta', text: '4'}` then
>   `{type: 'stop', reason: 'end_turn'}`, the executor running a ReAct
>   strategy on task "What is 2+2?" emits events in this exact order:
>   `session_start`, `agent_spawned`, `stream_text`, `agent_complete`;
>   no other events; final message text equals "4".
> - When `BudgetCap(per_run: 1.00)` is set and a run's `token_usage`
>   accumulates to 1.01, a `budget_cap_reached` event is emitted within
>   one event cycle, the executor pauses (no further LLM calls for 500ms),
>   and a `hitl_requested` event with `reason: 'budget_cap'` follows.
>
> **Every milestone's "done" criteria must reference behavioral tests,
> not just unit tests passing.**

### 3. Coverage Gates

| Module | Unit % | Branch % | Notes |
|---|---|---|---|
| `drone/core/*` | 95 | 90 | Survival layer is critical |
| `drone/supervisor/*` | 90 | 85 | |
| `main/runtime/context.ts` | 95 | 90 | Gatekeeper, everything routes through |
| `main/runtime/executor.ts` | 90 | 85 | |
| `main/sdk/AnthropicProvider.ts` | 85 | 80 | |
| `main/db/*` | 90 | 85 | |
| `renderer/narration/*` | 85 | 80 | |
| `renderer/graph/*` | 75 | 70 | UI — lower bar acceptable |
| `renderer/cost/*` | 90 | 85 | Trust layer — high bar |

If coverage drops below gate, milestone is not complete.

### 4. Lint & Type Gates
- ESLint config (strict; extends `@typescript-eslint/recommended-type-checked`)
- Prettier config (enforced in CI, pre-commit optional)
- TypeScript `strict: true`, `noUncheckedIndexedAccess: true`,
  `noImplicitOverride: true`, `exactOptionalPropertyTypes: true`
- `tsc --noEmit` must pass before any commit
- Zero `any` types allowed in committed code (use `unknown` + narrow)
- Zero `// @ts-ignore` without a paired `// @ts-expect-error` with reason
- Zero ESLint warnings, not just errors

### 5. Milestone Sequence

Each milestone follows this template exactly:

```
## Milestone N: <name>

**Goal:** <user-visible or architecturally-visible outcome>
**Budget:** <hours>
**Depends on:** <previous milestones by number>

### Tests to write first (in this order)
1. Unit: <test name> — <what it asserts>
2. Unit: <test name> — <what it asserts>
3. Integration: <test name> — <what it asserts>
4. Behavioral: <test name> — <user-visible outcome asserted>

### Files created or modified
- <path> — <purpose>
- <path> — <purpose>

### Verification
- [ ] `npm run lint` passes with zero warnings
- [ ] `npm run typecheck` passes
- [ ] `npm run test:unit` passes with coverage >= gate
- [ ] `npm run test:integration` passes
- [ ] `npm run test:behavioral` passes
- [ ] `npm run verify` passes end-to-end
- [ ] <milestone-specific verification>

### Definition of done
A sentence that a human can read and say "yes, I observed that."
Not "the code compiles." Not "the tests pass." The specific user-
visible or system-visible behavior that proves the milestone.

### Commit message
`<type>(<scope>): <description>` — standard Conventional Commits.
```

Milestones must be numbered and sequenced. Aim for 12–18 milestones for
v0.1. Each milestone is 3–8 hours of work. No milestone exceeds 10 hours.
If a milestone looks bigger, split it.

### 6. v0.1 Acceptance Test (The Capstone)

One explicit behavioral test a human runs against the built app:

```
1. Fresh install on Windows 11
2. Launch app
3. Complete first-run wizard (API key, name, budget defaults)
4. Pick Quick Agent from gallery
5. Type: "List the files in my Desktop folder"
6. Confirm the pre-run cost estimate
7. Watch narration panel describe what's happening in plain English
8. Verify: cost widget shows a value within estimated range
9. Verify: run completes with a "What just happened?" summary
10. Click "Why did it do that?" on any step — see plain-English rationale
11. Close app; reopen; verify run history is visible
```

If this test fails for any reason, v0.1 is not done.

### 7. Rollback Strategy
If a milestone breaks a previous one, how do we roll back? Git branch
strategy, how to identify the last known-good state, how to resume.

### 8. Daily Standup Format (for long sessions)
When a build session lasts multiple hours, what's reported back to the
user at each checkpoint:

- Milestone(s) completed
- Tests added (unit / integration / behavioral counts)
- Coverage delta
- Time spent vs budget
- Blockers or ambiguities encountered
- Open questions needing user input
- Next milestone to start

---

## Rules That Apply to Both Documents

- **No implementation code.** Types, interfaces, schemas, test descriptions,
  commands, prose. Nothing else.
- **No hallucinated versions.** Every version number verified via web
  search or npm registry, with date and source.
- **Every claim is sourced.** If the spec says "React Flow handles 10k
  nodes with virtualization," there's a link to the docs or a benchmark.
- **Every risk has a mitigation.** Listing a risk without a plan to
  address it is not acceptable.
- **Ambiguities surface, not hide.** If the spec is unsure, it says so
  in Section "Open Questions" rather than picking silently.
- **No aspirational content.** If a feature is deferred, it's in
  Non-Goals. If a feature is in scope, it has milestones, tests, and
  acceptance criteria.
