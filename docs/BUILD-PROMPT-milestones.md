# Meta-Prompt 2 — Milestone Spec Generation

> **How to use this file:** Copy everything below the `--- PROMPT START ---`
> line and paste it as the first message to a fresh Claude Code session
> opened in this repository. The session will produce
> `docs/BUILD-SPEC-v0.1-milestones.md`. No production code will be
> written. Small test-stub experiments are allowed only to verify test
> commands work.
>
> **Prerequisite:** `docs/BUILD-SPEC-v0.1-architecture.md` must exist
> and be committed. If it doesn't, run `BUILD-PROMPT-architecture.md`
> first.

---

--- PROMPT START ---

You are producing the test-first milestone specification for v0.1 of
the Loom project (or whatever name the architecture spec settled on).
Your sole deliverable in this session is a single file:
`docs/BUILD-SPEC-v0.1-milestones.md`. You will write **no production
code**. You may write tiny throwaway test-harness experiments if you
need to verify a command works — but those experiments are not
committed.

## What the product is (short version)

A desktop workbench for agentic-literate non-coders. v0.1 ships Quick
Agent (a ReAct-loop framework) end-to-end on Windows with plain-English
narration, budget caps, a live graph, and a first-run experience. Full
context in `docs/DESIGN-DECISIONS.md`.

## Required reading (in exact order)

1. `docs/DESIGN-DECISIONS.md` — what we're building
2. `docs/BUILD-SPEC-v0.1-architecture.md` — the architecture (must exist)
3. `docs/BUILD-SPEC-TEMPLATE.md` — the shape your output must follow,
   **specifically the milestones sections**
4. `docs/OPEN-QUESTIONS.md` — known unknowns

If `BUILD-SPEC-v0.1-architecture.md` does not exist, stop and tell the
user to run meta-prompt 1 first. Do not proceed.

## Non-negotiable rule: proof of life is not acceptance

**Re-read BUILD-SPEC-TEMPLATE.md Section 2 ("Test Philosophy") verbatim
before writing any milestone.** That section is the single most
important constraint on your output.

Summary: every milestone has three test tiers, and every milestone's
definition of done references behavioral tests that exercise real
user-visible or system-visible outcomes. Tests like
`expect(fn()).not.toThrow()` are **not permitted** as milestone
acceptance criteria. Every test you specify must assert a specific,
observable effect.

## Your workflow (strict, in this order)

### Step 1 — Read and understand
Read the four required files. Produce a two-paragraph summary:

- Paragraph 1: what's in the architecture spec, in your own words
- Paragraph 2: what the milestones spec needs to cover and any gaps
  you noticed in the architecture that block writing milestones

Wait for user to confirm or correct.

### Step 2 — Research (targeted)
Web-search anything the architecture spec didn't resolve about
**testing tooling**:

- Vitest current version and config for Electron main + renderer
- Playwright Electron test patterns in 2026
- `@testing-library/react` + Vitest + jsdom pitfalls
- Coverage reporting (v8 vs istanbul in current Vitest)
- Snapshot testing — whether it's appropriate for narration strings
  (recommendation: yes, snapshot the deterministic narration output)
- How to test streaming LLM responses with a mock (recommended pattern
  for Anthropic SDK specifically)
- How to test SQLite-backed code in parallel test runs without
  contention (in-memory DB vs temp files vs serialized workers)
- How to test Electron IPC boundaries (contextBridge mocking)
- How to behaviorally test a React Flow canvas (spoiler: it's hard —
  find current best practice)

Document every finding with source and date.

### Step 3 — Decompose the architecture into milestones
Using the architecture spec's module map, break the build into 12–18
sequenced milestones. Each milestone:

- Delivers one observable slice of functionality
- Builds on previous milestones (no forward references)
- Is 3–8 hours of work (rare exceptions up to 10)
- Ends with a `npm run verify` that passes

**Ordering principle:** prefer the order that gets to a clickable user
experience fastest, then adds rigor. Concretely, for Loom v0.1:

1. Project bootstrap + lint/typecheck/test infrastructure working
2. SQLite schema + DB layer with integration tests
3. `LLMProvider` interface + `AnthropicProvider` implementation +
   mock provider for tests
4. `RuntimeContext` skeleton + policy stubs
5. Event pipeline + deterministic `humanSummary()` templates
6. Minimal React shell + one route + IPC bridge
7. Narration panel renders a canned event stream (hardcoded, no
   executor yet) — **first clickable moment**
8. Generic executor + ReAct strategy with mock LLM
9. Wire executor events into the narration panel (first real run)
10. Live graph toggle + React Flow skeleton
11. Drone Core (heartbeat + snapshot + recovery) + tests
12. Process Supervisor + tests
13. Cost widget + 4-tier budget caps + pre-run estimator
14. First-run experience + card gallery + Quick Agent manifest
15. Integration test sweep + bug fixing + perf pass
16. Windows packaging + installer + capstone acceptance test

These are suggested — the architecture spec may have re-ordered things
or added modules. Adapt, but justify any deviation.

### Step 4 — Write each milestone
For each milestone, produce the full template block from
`BUILD-SPEC-TEMPLATE.md` Section 5. Each milestone must have:

- **Goal** (observable, specific)
- **Budget** (hours)
- **Depends on** (previous milestones)
- **Tests to write first** (at least 3 unit, 1 integration, 1
  behavioral; more if the milestone is bigger)
- **Files created or modified** (with LOC budget)
- **Verification checklist**
- **Definition of done** (human-observable sentence)
- **Commit message** (pre-written)

**The tests are the hardest part.** For each test, write:

- Test name (describe-it style)
- What it sets up (fixtures, mocks, seeds)
- What it exercises (the specific action)
- What it asserts (the specific observable outcome)

Do not hand-wave with *"test that it works"*. Write the assertion as
prose in enough detail that a developer can convert it directly into
a Vitest test body.

### Step 5 — Coverage gates
For each module introduced, pull the coverage gate from the template's
Section 3. If a milestone's verification doesn't meet the gate, the
milestone is not complete. Call this out explicitly.

### Step 6 — Cross-milestone invariants
Produce a section at the end of the file: "Invariants that every
milestone must preserve." Examples:

- Every committed state passes `npm run verify` — no broken mains
- No `any` types introduced
- No ESLint warnings introduced
- No test suite time regression > 20% vs previous milestone
- No new `TODO` or `FIXME` comments in committed code
- No reduction in coverage on any module

### Step 7 — The capstone acceptance test
Write out Section 6 of the milestones template (the 11-step capstone
test) in full. Number every step. Describe what success looks like at
each step. Specify what to do if any step fails (usually: stop, report,
and diagnose).

### Step 8 — Self-review
Before declaring done:

- [ ] Every milestone has at least one behavioral test, not just unit
      tests
- [ ] No milestone's "definition of done" is satisfied purely by unit
      tests passing
- [ ] Every test description is specific enough to become a Vitest
      body without further clarification
- [ ] Milestones are in an order that produces a clickable experience
      by roughly milestone 7
- [ ] The capstone is reachable and verifiable
- [ ] Total estimated hours is within 80–100
- [ ] Every milestone ends with `npm run verify` passing
- [ ] Coverage gates are applied
- [ ] Lint/type gates are applied
- [ ] No milestone is a code-dump — each one is a test-then-implement
      cycle
- [ ] No cross-milestone forward references (milestone 5 doesn't
      depend on something milestone 9 creates)

### Step 9 — Commit
Stage `docs/BUILD-SPEC-v0.1-milestones.md` and commit with message:
`docs: BUILD-SPEC v0.1 milestones — generated by meta-prompt 2`. Do
not push unless the user explicitly asks.

### Step 10 — Report
Output a brief report:
- Number of milestones produced
- Total estimated hours (sum of budgets)
- Number of behavioral tests specified (rough count)
- Any open questions that remain
- Any architecture-spec changes you suggest (list them; do not apply
  them — changes to architecture are done by re-running meta-prompt 1
  or by explicit user request)
- Suggested next step: "ready for user review, then begin Milestone 1
  in a fresh session"

## Rules (non-negotiable)

1. **No production code in this session.** Test descriptions yes;
   test implementations no; production code no. If you catch yourself
   writing logic, stop.
2. **Behavioral tests are mandatory at every milestone.** A milestone
   whose acceptance is only "unit tests pass" is wrong. Rewrite it.
3. **Assertions must be specific.** "Returns a value" is not an
   assertion. "Returns `{status: 'ok', ts: number}` where ts is
   within 2 seconds of now()" is.
4. **Forward references are forbidden.** A milestone cannot depend
   on code not yet built.
5. **Coverage gates are enforced.** A milestone that can't meet its
   gate is wrong or incomplete — fix the scope or the test plan.
6. **Linting is not optional.** Every milestone's verification
   includes `npm run lint` passing with zero warnings.
7. **Type checking is not optional.** Every milestone's verification
   includes `npm run typecheck` passing with zero errors.
8. **Ambiguities surface.** If you're unsure whether a test should
   be a unit or integration test, ask the user. If you're unsure
   how a milestone's acceptance should be measured, ask.
9. **Use the todo list tool** to show progress through the 10 steps.
10. **Stop on blockers.** Don't improvise past a missing architecture
    decision — ask.

## Success criteria for this session

The session is complete when:

1. `docs/BUILD-SPEC-v0.1-milestones.md` exists
2. 12–18 milestones sum to 80–100 hours
3. Every milestone has behavioral tests specified
4. Coverage/lint/type gates are applied per milestone
5. Capstone acceptance test is written in full
6. The file is committed
7. Report is delivered
8. User has confirmed readiness to start Milestone 1 in a new session

## What to do if you get stuck

- **Architecture is ambiguous** → ask, or recommend re-running meta-prompt 1
- **A milestone feels too big** → split it
- **A milestone feels too small** → merge it
- **You can't figure out how to test something behaviorally** → flag it;
  if it genuinely can't be behaviorally tested, it's probably not
  user-visible enough to be a milestone on its own
- **Totals exceed 100 hours** → recommend cuts to the user; do not
  silently defer things
- **You're about to write production code** → off-track; stop; re-read
  this prompt

--- PROMPT END ---

## Notes for the user (not part of the prompt)

- This prompt produces the hardest document in the project. The
  architecture is the skeleton; the milestones are the actual work.
  Expect a long session.
- When the output file is committed, **read it carefully**. Pay
  specific attention to: (a) whether every milestone has a real
  behavioral test, (b) whether the definitions of done are
  observable, (c) whether the hour totals match your expectation.
- If the milestones spec is wrong, running it produces wrong code.
  The cost of fixing it here is much lower than fixing it later.
- Once the milestones spec is good, every subsequent session is
  "execute Milestone N from the spec." Those sessions are bounded,
  testable, and short.
