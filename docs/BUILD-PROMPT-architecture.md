# Meta-Prompt 1 — Architecture Spec Generation

> **How to use this file:** Copy everything below the `--- PROMPT START ---`
> line and paste it as the first message to a fresh Claude Code session
> opened in this repository. The session will produce
> `docs/BUILD-SPEC-v0.1-architecture.md`. No code will be written in
> this session.

---

--- PROMPT START ---

You are producing the architecture specification for v0.1 of a product
codenamed **Loom** (working title — treat it as a variable the user may
override). Your sole deliverable in this session is a single file:
`docs/BUILD-SPEC-v0.1-architecture.md`. You will write **no
implementation code** in this session. You will write **no test code** in
this session. The output is pure specification.

## What the product is

Read these files in this exact order before producing anything:

1. `agent-runtime-spec.md` — the original vision document (historical)
2. `docs/DESIGN-DECISIONS.md` — the current source of truth
3. `docs/OPEN-QUESTIONS.md` — unresolved items
4. `docs/BUILD-SPEC-TEMPLATE.md` — the shape your output must follow

If any of these files are missing, stop and ask the user. Do not invent
their contents.

After reading, produce a one-paragraph summary of the product and wait
for user confirmation before proceeding. If your summary is wrong, the
user will correct it. Do not proceed until the user says "yes, that's
right" or equivalent.

## Your workflow (strict, in this order)

### Step 1 — Read
Read the four files above. Take no other actions.

### Step 2 — Summarize and confirm
Output a one-paragraph summary. Ask: *"Is this an accurate summary? If
yes, I'll proceed to research."* Wait.

### Step 3 — Research (web search mandatory)
Use web search to verify the following, and document source and date
for every finding in a scratch section you will include in the spec:

**Versions (check npm registry or official release pages):**
- `electron` — current stable
- `vite` + `vite-plugin-electron`
- `react` + `react-dom` + `@types/react` + `@types/react-dom`
- `typescript`
- `@anthropic-ai/sdk`
- `better-sqlite3` — **and** its compatibility matrix with the chosen
  Electron version
- `reactflow` or `@xyflow/react` (names changed; find the current one)
- `tailwindcss` + `postcss` + `autoprefixer`
- `vitest` + `@vitest/coverage-v8`
- `playwright` + `@playwright/test`
- `electron-builder`
- `eslint` + `@typescript-eslint/*` + `eslint-plugin-react`
- `prettier`
- `keytar` — **verify it still works with current Electron; if not,
  find an alternative**

**Current best practices (cite each):**
- Electron security posture (context isolation, sandbox, nodeIntegration,
  `webSecurity`, CSP headers)
- Electron + TypeScript + Vite project layout in 2026
- `better-sqlite3` + Electron native rebuild workflow on Windows
- React Flow performance patterns for long-running sessions (10k+ nodes)
- Anthropic SDK streaming with tool use in TypeScript
- MCP client implementation current state (spec version, reference
  client libraries)
- Windows code signing for small-team Electron apps (Azure Trusted
  Signing, Sectigo, SSL.com current pricing if findable)
- Vitest behavioral testing patterns
- Playwright Electron testing current approach

**Known landmines (search for these specifically):**
- Electron + `better-sqlite3` + Windows Defender / SmartScreen flagging
- React Flow memory issues on long sessions
- `@anthropic-ai/sdk` breaking changes in recent minor versions
- `keytar` maintenance status (it has been problematic historically)

For each finding, include in your scratch notes:
- Claim
- Source URL
- Date you retrieved it
- Relevance to our design decisions

If any finding contradicts a decision in `DESIGN-DECISIONS.md`, **do not
silently adjust the decision.** Surface it to the user as an
ambiguity to resolve.

### Step 4 — Clarify ambiguities
Produce a numbered list of clarifying questions. These come from:

- Conflicts between your research and the design decisions
- Gaps in `DESIGN-DECISIONS.md` that block writing the architecture
- Items in `OPEN-QUESTIONS.md` that block architecture decisions
- Places where multiple valid approaches exist and you need the user's
  preference

**Do not pick silently.** Ask. Wait for user answers before proceeding.

Questions must be concrete and answerable in one sentence. Not *"what
testing strategy should we use"* — ask *"Vitest + Playwright for unit
and E2E respectively, or add Testing Library for React component tests?
Recommend: yes to Testing Library, it's standard."*

Always include your recommended default so the user can approve
quickly.

### Step 5 — Produce the spec
Only after steps 1–4 are complete, write
`docs/BUILD-SPEC-v0.1-architecture.md` following `BUILD-SPEC-TEMPLATE.md`
exactly. Every section in the template is required. Sections the
template labels as "pulled from" must cite their source.

### Step 6 — Self-review
Before declaring done, re-read your output and check:

- [ ] Every version is verified with a URL and date
- [ ] Every interface compiles (run `tsc --noEmit` on a scratch file if
      necessary to verify — **this is the only implementation work
      permitted in this session**)
- [ ] Every risk has a mitigation
- [ ] No section is empty or says "TBD"
- [ ] The module map accounts for every file the milestones doc will
      later need to reference
- [ ] No implementation code slipped in
- [ ] Open questions are surfaced, not hidden
- [ ] The user's name for the product is used consistently (not "Loom"
      if they picked a different name)

### Step 7 — Commit
Stage `docs/BUILD-SPEC-v0.1-architecture.md` and commit with message:
`docs: BUILD-SPEC v0.1 architecture — generated by meta-prompt 1`.
Do not push unless the user explicitly asks.

### Step 8 — Report
Output a brief report:
- Hours spent (rough)
- Sections produced
- Ambiguities that were resolved (with the resolution)
- Any open questions that remain and need user answer before the
  milestones pass can begin
- Suggested next step: "ready to run `BUILD-PROMPT-milestones.md`"

## Rules (non-negotiable)

1. **No implementation code.** If you catch yourself writing a function
   body, stop and convert it to a type signature or a test description.
2. **No silent guessing.** Every unknown is either researched on the web
   or surfaced as a clarifying question.
3. **No hallucinated versions.** If you can't verify a version, say so
   and ask.
4. **No scope creep.** If a feature isn't in `DESIGN-DECISIONS.md` for
   v0.1, it doesn't belong in this spec.
5. **No aspirational promises.** Every acceptance criterion is something
   a human can observe and verify.
6. **No placeholders.** "TBD" and "TODO" are forbidden in the committed
   spec. Everything is resolved, even if the resolution is "deferred to
   Phase 2 — see open questions."
7. **Use the todo list tool.** Track your progress through these 8 steps
   visibly so the user can see where you are.
8. **If a step fails, stop and ask.** Do not improvise past a blocker.

## Success criteria for this session

The session is complete when:

1. `docs/BUILD-SPEC-v0.1-architecture.md` exists and matches the template
2. All versions in the spec are verified with sources
3. All clarifying questions have been resolved with the user
4. The file is committed to the current branch
5. The report has been delivered
6. The user has confirmed they're ready to proceed to meta-prompt 2

## What to do if you get stuck

- **Web search fails** → report which query, try an alternative, ask user
- **User's design conflicts with research** → surface both, ask
- **Scope feels too big** → stop, report what you've done, ask user to cut
- **You're uncertain whether something belongs in architecture vs milestones**
  → put it in architecture if it's "what/structure," milestones if it's
  "when/order/test." When unsure, ask.
- **You're about to write code** → you are off-track. This session
  produces a specification only. Stop and re-read this prompt.

--- PROMPT END ---

## Notes for the user (not part of the prompt)

- This prompt is designed for a single multi-hour Claude Code session.
  It may pause multiple times for your input. That's expected.
- The output `.md` file should be 1500–3500 lines of careful
  specification. If it's shorter than 1000 lines, it's probably missing
  sections. If it's longer than 5000 lines, it probably has
  implementation code smuggled in.
- When meta-prompt 1 completes, review the spec yourself before running
  meta-prompt 2. Fix anything that looks wrong, commit the corrections,
  then proceed.
- The session produced by this prompt will ask you for the product name.
  Decide it before running, or be ready to answer "Loom" or whatever
  you pick when it asks.
