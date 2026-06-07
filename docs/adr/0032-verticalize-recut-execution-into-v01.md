# ADR-0032: Re-verticalize M10–M13; execution breadth + shell-exec into v0.1 (OS-native sandbox, semi-trusted threat model)

**Status:** Accepted (2026-06-07, PR #127)
**Date:** 2026-06-07
**Deciders:** @kknipe2k (maintainer / product owner)
**Tags:** scope, product, security, capability

## Context

ADR-0031 re-cut v0.1 around **author-and-run** and introduced the **M09 walking
skeleton** (author one agent + `file_access` + a real MCP tool → run → write a real
file). M09 was correct. But ADR-0031 then sliced **M10–M13 horizontally** — a whole
*author-anything* layer (M10), a whole *real-data* layer (M11), a whole
*execution-breadth* layer (M12: sub-agents/plans/hooks), then hardening (M13) — and
**deferred execution breadth (M12) + Generators to v1.0** (v0.1 = M09+M10+M11+Release).

Two problems with that, established this session:

1. **Horizontal layering produces no shippable, IRL-verifiable increment until an
   entire layer is finished**, and it bolts authoring onto a separately-built engine —
   re-creating the author↔run integration risk at the end. The established practice is
   **vertical slicing / walking skeleton**: each milestone cuts canvas→engine→run and
   ships one capability the maintainer can **author AND run AND IRL-watch** end-to-end,
   proving the integration continuously. ADR-0031's M09 was already a correct walking
   skeleton; only the rest was flipped horizontal.

2. **The H (shell/code execution) deferral violated CLAUDE.md §12** ("do not scope-down
   based on perceived complexity, time, or budget"). It was deferred on
   complexity/security-review grounds — but the **v0.1 lighthouse deliverable requires
   shell execution**: the product *is* the workbench, and the lighthouse workflow is the
   maintainer's own **software-development loop** — research → PRD → plan → implement →
   **verify** — multi-agent, with an **objective verify gate that is literally
   `bash verify.sh`**, plus HITL approval, rails, and gap suspend→resume. No shell exec,
   no verify gate, no deliverable.

The seams confirm a vertical re-cut is feasible (rule 11, validated file:line):
- A plan's tasks execute on the **single-agent** loop — `plan_loop.rs:7-8,128`
  ("Task execution is `AgentSdk::run_agent` … this shell runs no tasks"), **not**
  sub-agents. So plans (M10) validly precede sub-agents (M11) — no dependency inversion.
- `runtime-sandbox` **already** ships the OS-native fences — `seccomp` (Linux BPF),
  `landlock` (Linux FS), `job_objects` (Windows) with `install_isolation` wiring them
  per-OS — so shell-exec is a **finish**, not new infrastructure. The fences are
  currently **no-exec by design** (`landlock.rs:198` asserts `!AccessFs::Execute`;
  `SandboxRequest` is `ValidateArtifact|Shutdown`), so H adds a *controlled-exec
  profile* + `SandboxRequest::Execute` + the command-spawn.

## Decision

**We keep ADR-0031's M09 walking skeleton and re-verticalize M10–M13; we pull
execution breadth into v0.1; and we put shell execution in scope as an OS-native
sandbox under an explicit semi-trusted threat model.**

**M09 stays the thinnest end-to-end thread (pure):** author one agent + `file_access`
+ one real MCP tool → run → write a real file at the enforced tier. It ships with
**"suspends cleanly"** (E-04 already). Gap *resolve→resume* moves to M10 (it is ~1–2
stages of separable backend + UI and must not fatten the skeleton's "does it run" proof).

**Each subsequent milestone is a vertical slice** — author-on-canvas + run-for-real +
maintainer IRL + an `execution-status.md` row flip:

- **M10 — HITL steers the run.** Gap **resolve→resume** (ADR-0029) + **plan-approval**
  + **plan task execution** (`drive_plan` production caller, ADR-0026; tasks run on the
  single-agent loop). Unifying theme: a human approves/grants to let execution proceed.
- **M11 — Sub-agents (sequential).** Orchestrator spawns a child with narrowed grants;
  the child runs its loop and returns a summary (`agent_sdk.rs:467`, rung 6).
  `spawn_constraints.max_concurrent: 1` — the dev loop is inherently sequential
  (research → PRD → plan → implement). The entry phase (research-agent + PRD-writer) is
  composition over this.
- **M12 — The verify loop.** A post-task **shell verify hook** + a `dont_touch` **rail**
  → runs the tests, **green→next / red→rollback+retry**, a forbidden edit is blocked.
  Hooks/rails firing engine (rung 8) **+ H (controlled shell exec)**. One vertical
  capability, **staged heavily**; H is its own Hard-Rule-8 sub-ladder (new controlled-exec
  profile flipping `landlock.rs:198`, `SandboxRequest::Execute`, the command-spawn,
  `// SAFETY:` discipline, security-review posture, its own ADR).
- **M13 — Industrialize + ship.** MCP data-source catalog (install GitHub/Postgres/… in
  the app) + validated whole-workflow import/export + save-path + first-run.

**Shell-exec sandbox threat model (mandatory, rule 11 — do not over-claim isolation):**
v0.1's executed code is **semi-trusted** — the maintainer's own framework + own
`verify.sh`, local, single-user, no-telemetry (§13). For *that* threat model, OS-native
`landlock`+`seccomp`+`Job Objects` is the **correct** fit, and aligns with the 2026
local-desktop sandbox direction (Sandlock). It is **explicitly weaker than a
microVM/gVisor** for *arbitrary untrusted* code; **microVM/gVisor is named the v1.0+
upgrade IF the product ever runs untrusted third-party code at scale.** The H sub-ladder's
ADR records this threat model verbatim.

**Scope re-line:** **v0.1 = M09 + M10 + M11 + M12 + M13** — the software-development loop
**builds, runs, and ships**, industrial-strength. **v1.0 = concurrent/parallel
multi-agent** (fan-out / agent-pool / teams — the P2–P4 orchestration model) **+ the
ML/data framework** (a structurally identical pipeline, mostly tool/skill swaps) **+ the
microVM/gVisor sandbox upgrade + Generators** (the LLM build-assist, old M9) + the
remainder of §0d's v1.0 column.

This **supersedes ADR-0031's M10–M12 structure and its v0.1/v1.0 line** (M09 retained)
and **re-amends §0d**. `docs/workbench-delivery-plan.md` (§4 rewritten with this ADR) is
the authoritative detailed roadmap; MVP-v0.1.md is the milestone index.

## Consequences

### Positive
- **Every milestone ships a real, demoable, rule-11-verifiable increment** (author +
  run + IRL) and proves author→run integration continuously, not at the end.
- **Execution breadth is no longer deferred** — v0.1 *runs* the full multi-agent dev
  loop, not a composer that paints.
- **§12 honored** — the deliverable's required capability (shell exec) is built, not
  scoped down; staging absorbs the volume.
- **Honest isolation claim** — the sandbox's semi-trusted threat model is recorded, not
  over-stated (the same discipline as `execution-status.md`).

### Negative
- **v0.1 is larger** — execution breadth (old M12) is pulled in from v1.0; the timeline
  lengthens.
- **M12 is the long pole** — the verify loop bundles the firing engine + the
  security-boundary shell-exec sub-ladder + rollback/retry; many stages.
- **OS-native is weaker than microVM** for untrusted code — accepted for the
  semi-trusted local threat model, recorded, with the microVM upgrade named for v1.0+.

### Neutral / future implications
- Re-homes ADR-0029 (gap-resume → **M10**) and ADR-0026 (plan_loop → **M10**) from
  ADR-0031's M12; sub-agents → **M11**; hooks/rails → **M12**.
- A new §0a capability-matrix primitive — **shell/code execution via
  `SandboxRequest::Execute`** — is introduced; the H sub-ladder's own ADR is the §11 gate
  for it (schema/enforcer/IPC + sandbox-protocol change, Hard Rule 8).
- Generators (old M9) remain v1.0; parallel multi-agent (P2–P4) is v1.0.

## Alternatives Considered

### Alternative A: Keep ADR-0031's horizontal M10–M12 + defer execution + H to v1.0
**Rejected because:** horizontal layering yields no IRL-verifiable increment until a
whole layer is built, defers the "does it actually run" proof to the end, and the H
deferral was an illegitimate §12 complexity scope-down — the verify gate (`bash
verify.sh`) is core to the v0.1 deliverable.

### Alternative B: M08.9.1 — wire ALL painted execution first (a horizontal engine layer) against hand-written JSON, before the authoring milestones
**Rejected (withdrawn):** this is pure horizontal slicing — it finishes an entire
execution layer with **no author→run increment** and tests against hand-written JSON
rather than the canvas, getting the worst of both: no shippable increment *and* the
integration risk at the end. (This was the prior plan; withdrawn here.)

### Alternative C: microVM / gVisor / Firecracker sandbox for H
**Rejected because:** those are Linux-KVM or cloud (E2B/Daytona/Firecracker) — wrong for
a **Windows-first, local-first, no-telemetry desktop** runtime; OS-native
(`landlock`/`seccomp`/`Job Objects`, which `runtime-sandbox` already has) is the correct
local equivalent for the semi-trusted threat model. microVM/gVisor is the v1.0+ upgrade
if the threat model ever changes to arbitrary untrusted code.

## Related
- **Supersedes (in part):** ADR-0031 — its M10–M12 structure + v0.1/v1.0 line.
  **M09 walking skeleton retained.**
- Re-homed: ADR-0029 (gap-resume → M10), ADR-0026 (plan_loop → M10).
- Spec: **§0d Release Scope Matrix** (re-amended), **§0a Capability Matrix** (new
  shell-exec primitive — the H sub-ladder's ADR is the gate), §8.security L3 (sandbox),
  §13 (no-telemetry). CLAUDE.md §12 (no complexity scope-down), rule 11.
- Seams validated: `plan_loop.rs:7-8,128`, `agent_sdk.rs:467`,
  `runtime-sandbox/src/{seccomp,landlock,job_objects}.rs`, `landlock.rs:198`,
  `protocol.rs:29`.
- Docs reconciled with this ADR: `docs/workbench-delivery-plan.md` (§1/§4/§5 rewritten),
  `docs/MVP-v0.1.md` (index), `docs/execution-status.md` (lifespan note).
- External: vertical slicing / walking skeleton — scrum.org, monday.com, Agile Alliance;
  2026 sandbox landscape — Northflank, Sandlock (arXiv 2605.26298), Firecrawl.

## Notes
Maintainer-directed product + sequencing re-cut (2026-06-07 session), validated against
the real seams before authoring (rule 11 / §2 / §12). The detailed slices + file:line
seams land in `docs/workbench-delivery-plan.md` §4 (rewritten with this ADR);
MVP-v0.1.md + execution-status.md are reconciled in the same PR. Each slice closes with
an assembled-app IRL + an `execution-status.md` row flip — never a code-read.
