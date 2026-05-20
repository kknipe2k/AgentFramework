# M07 — Parent-Milestone Summary

> **Parent milestone:** M07 of M11 in `docs/MVP-v0.1.md`
> **Authored by:** Claude (per `CLAUDE.md` §19)
> **Aggregates:** M07.A, M07.B, M07.C, M07.D1, M07.D2, M07.E stage retrospectives + M07.V verifier retrospective
> **Created at:** 2026-05-19 (UTC)
> **Total elapsed:** ~28 h deliverable-time (A ~3.5 + B ~4 + C ~5 + D1 ~6.5 + E ~6 + V ~3) + the M07.D2 environment-diagnosis detour (deliverable ~on estimate; total elapsed past 2× — see Time-box)
> **Estimated:** ~35–47 h (phase doc; ~22–30 h at M06's converging ~0.7× calibration)

---

## Stage trail

| Stage | Status | Stage commit(s) | Retrospective | Outcome |
|---|---|---|---|---|
| Stage A — M06 carry-forward absorption + ADR-0011 construction-graph groundwork | Committed | red `348d1ef` → impl/docs `bd79083` | `M07.A-retrospective.md` | Sound |
| Stage B — `skills.lock` integrity primitive + ADR-0014 | Committed | red `11e698b` → impl `28c16b4` → style `afd1cee` | `M07.B-retrospective.md` | Sound |
| Stage C — import-pipeline backend | Committed | red `3a586be` → impl `849fd0c` → style `724e4cb` | `M07.C-retrospective.md` | Sound |
| Stage D1 — ADR-0011 (a)–(c) concrete construction + CQ-6/EFF-4 | Committed | red `689592f` → impl `f5ef25a` → additive `befb3a6` → style `3eae0fc` | `M07.D1-retrospective.md` | Sound but rough |
| Stage D2 — ADR-0011 (d) agent-with-tools loop + `token_usage` projector + CQ-2 | Committed | red `10dba9f` → impl `ab18302` → style `15694c1` → green-fix `90b18ac` | `M07.D2-retrospective.md` | Sound but rough |
| Stage E — Builder Import panel renderer + ADR-0015 | Committed | red `42186c1` → impl `f0a7721` | `M07.E-retrospective.md` | Sound |
| Stage V — in-band verifier (four passes; mandatory `--features integration` smoke) | Committed | `8bd9e57` | `M07.V-retrospective.md` | Sound but rough |
| (post-V) deadlock fix — D2-latent multi-turn deadlock in the M06.F injection-seam test | Committed | `8a861cd` | n/a (test-harness-only fix) | n/a |

All stages on parent-milestone feature branch `claude/m07-registry-import`. The M07 PR drafts after this summary + the gap-analysis entry land and surfaces all stage commits + retrospectives + this summary + the gap-analysis entry together.

---

## Aggregate scoring (across stages A–E)

### Process axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | 38 | /40 |
| Stage B | 39 | /40 |
| Stage C | 39 | /40 |
| Stage D1 | 36 | /40 |
| Stage D2 | 35 | /40 |
| Stage E | 38 | /40 |
| **Mean** | **37.5** | /40 |

### Product axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | 38 | /40 |
| Stage B | 38 | /40 |
| Stage C | 37 | /40 |
| Stage D1 | 38 | /40 |
| Stage D2 | 38 | /40 |
| Stage E | 38 | /40 |
| **Mean** | **37.83** | /40 |

### Pattern axis

| Stage | Total | /35 |
|---|---|---|
| Stage A | 30 | /35 |
| Stage B | 31 | /35 |
| Stage C | 30 | /35 |
| Stage D1 | 27 | /35 |
| Stage D2 | 27 | /35 |
| Stage E | 31 | /35 |
| **Mean** | **29.33** | /35 |

Stage V (verifier) is scored on its own three axes per `VERIFIER-RETROSPECTIVE-TEMPLATE.md`: Coverage adequacy 4/5, Finding signal-to-noise 5/5, Fresh-context discipline 5/5 — **14/15** (avg 4.67), clear sound signal.

---

## Cross-stage trends

### Friction patterns that recurred

- **Phase-doc-vs-shipped-code drift surfaced at pre-flight — every single work stage.** A (A.3.1 asserted a non-existent schema-generated `HitlContext` variant), B (B.3.1 `entries` vs spec §2200 `installed`; B.3.2 a literal cross-schema `$ref` the TS target can't resolve), C (C.3.1 `tier_gate?` propagation the committed tests contradict; C.3.3 §15c metadata carrier), D1 (D1.3.1/3.2/3.3 wrong trait shape, wrong ctor arity, under-scoped CQ-6), E (the v1.8 `<wire_signature_audit>` caught **6** drift points). Every instance was caught at pre-flight by the v1.8 audit slots + the surface-the-contradiction-don't-pick discipline; **no instance reached code.** The recurrence rate itself is the signal — this is the M06 first-class authoring-rigor finding continuing into M07. The structural fix (a phase-doc-authoring pre-PR checklist) remains a standing protocol carry-forward.
- **`rustdoc::broken_intra_doc_links` / `clippy::doc_link_code` on module-level `//!` and `pub use` docs** — recurred M07.B → C → D1 (three milestone stages running). Module-level cross-refs must be plain code spans, never intra-doc links. An M07.B gotcha-candidate that never graduated to `docs/gotchas.md` §15 and recurred verbatim.
- **Mechanical test-file fixes driving red-commit amendments** — B (rustfmt + last-use clones), C (test-file clippy), D1 (3 amends for the CQ-6→migration-003 ripple), E (3 amends: fixture schema-field defect → rustfmt → tsc-strict/prettier). A pre-red mechanical sweep (`cargo fmt` + `clippy --fix` + `prettier --check` + `tsc` on test files before the red commit) would have eliminated this class. Four stages running.
- **gotcha #56 / TD-005 Windows-local llvm-cov + `cargo test --workspace` non-measurability** — surfaced at C, D2, E. The runtime-main / `--workspace` llvm-cov gates are the documented CI-Linux-measured class; per-crate gates are the CI-faithful local path.

### Pattern-level wins

- **The v1.8 audit slots earned their place.** `<construction_reachability_check>` worked exactly as designed across the A→D1→D2 chain: A authored the ADR-0011 wires `inputs_reachable="false"`, D1 inverted each to `true` with file:line, D2 consumed them — the construction graph completing visibly across three stages. `<wire_trace_vs_adr_reconcile>` (D1 #6) correctly routed the §5a re-resolution driver onto `McpDispatcher` not `McpClient` — V verified it at `dispatch.rs:141`. `<wire_signature_audit>` (E) caught 6 phase-doc drifts before any pseudocode.
- **Strict v1.8 two-commit TDD held cleanly on every code stage.** The `git diff <red>..<impl> -- '**/tests/**'` EMPTY invariant was verified at B/C/D1/D2/E; the binary-crate scoped-diff variant (in-source `#[cfg(test)]` block byte-identical red→impl) held at D2 and E.
- **The mandatory `--features integration` reference-MCP-server smoke** (M06.V Decision 7, binding from M07.V) ran 1/1 — `stdio_against_reference_server_everything` PASS, confirming the rmcp wire-format excluded-holdout risk is clear.
- **Surface-the-contradiction-don't-pick held at every pre-flight.** Every phase-doc defect above was surfaced (to the maintainer when user-domain, owned per §12 when technical) before the red commit — never silently picked.
- **The ADR-0011 (a)–(d) discharge — the largest open architectural carry-forward in the ledger — closed cleanly.** D1's headline construction graph closed in roughly one pass once the shipped signatures were read; the roughness was isolated to the *un*-flagged CQ-6 schema-mirror ripple, not the construction.

### Surprises across the parent milestone

- **D1: CQ-6, framed by the phase doc as the "small" item, was the largest source of work.** A `String → generated-enum` field swap rippled into a stale SQLite `CHECK` constraint mirror → a table-rebuild migration (003) → 3 in-source test blocks across 2 crates → a coverage follow-up. The headline ADR-0011 (a)–(c) construction went cleanly; the "minor" CQ-6 dwarfed it.
- **D2: Windows `link.exe` LNK1201/1180/1318 errors were disk exhaustion (2 GB free), not a code/dependency-graph defect.** A multi-cycle misdiagnosis (dev-dep cycle → parallel-link/#56) ran before the disk precondition was checked; `cargo clean` freed 87.9 GiB and cleared the LNK class entirely. The parallel-link/#56 hypothesis was **retracted** — no `ci.yml` change.
- **D2: `cargo test --workspace` reproducibly stalls locally** (the in-test nested `cargo build` deadlocks under a `--workspace` parent on this box) while CI is green for the identical command — per-crate `cargo test -p X` is the CI-faithful local path.

### Hard gate violations across the milestone

- **None.** All six hard gates (G1 do-not-commit-until-approved, G2 no Sev-5 friction, G3 no unaddressed protocol drift, G4 stage completed, G5 every axis row ≥3, G6 CI-parity) passed in every stage A–E. D2's two **soft**-gate failures (S4 time-box, S5 Sev-3 count) were both driven by the environment-diagnosis detour, not the deliverable — the deliverable was ~on estimate and the assembled regression executed green (50.22 s real-drone). Stage V produced **1🔴** (a content finding, not a process hard-gate failure) — routed via the waiver mechanism (see Verdict).

---

## Time-box accuracy

| Stage | Estimated | Actual | Ratio |
|---|---|---|---|
| Stage A | 4–6 h | ~3.5 h | ~0.7× |
| Stage B | 6–8 h | ~4 h | ~0.6× |
| Stage C | 7–9 h | ~5 h | ~0.6× |
| Stage D1 | 4–5 h | ~6.5 h | ~1.4× |
| Stage D2 | 4–5 h | deliverable ~on estimate; total elapsed past 2× | >2× (environment detour) |
| Stage E | 5–6 h | ~6 h | ~1.0× |
| Stage V | 2–4 h | ~3 h | ~1.0× |

Deliverable-time across A–E+V (excluding D2's environment detour) ≈ 28 h vs a ~37.5 h midpoint estimate → ~0.75×, squarely in the M06 ~0.7× calibration band. The two over-runs are both **diagnosable and non-recurring**: D1's 1.4× was the under-scoped CQ-6 schema-mirror ripple (a phase-doc authoring gap, fixed by the proposed `<schema_mirror_ripple_check>` slot); D2's >2× was a Windows disk-exhaustion misdiagnosis (fixed by the proposed environment-precondition check). Neither is a calibration-method error — the estimation method is sound; the next parent milestone should keep the ~0.7× anchor.

---

## Decisions to apply before the next parent milestone

### Coverage-policy reconciliation (mandatory check — per `CLAUDE.md` §6)

- [x] **Changed this milestone** → all four mirrors verified byte-consistent and the ledger appended. The only *enforced-mirror* change was the `src.import.fetch.rs` runtime-main exclusion (Category 3 OS-call holdout), added and four-mirror-synced **in the M07.C commit** (`docs/coverage-policy.md` §A + §C, `CLAUDE.md` §6 command — `codecov.yml` and `CLAUDE.md` §5 need no change per the `providers/anthropic.rs` precedent). The M07.G `<coverage_policy_reconciliation>` appended `docs/coverage-policy.md` §C entries for M07.B / M07.D1 / M07.D2 and §B per-module baselines (`skills_lock` 98.15%, `import/mod.rs` 94.77%, `connection_resolver.rs`, `token_usage.rs` ≥95, `transport/mod.rs` 87.50% carry-forward) — ledger-only appends, no threshold or exclusion-regex value moved. CLAUDE.md §5 categories + §6 commands + `codecov.yml` confirmed byte-consistent with `docs/coverage-policy.md` §A. No drift.

### `CLAUDE.md` updates carrying forward

- No Hard-Rule changes. The recurring `docs/gotchas.md` §15 candidates (module-level intra-doc-link denial; `String→generated-enum` schema-mirror ripple; `Arc<dyn Trait>` results not `Debug`; `unknown | null` ESLint redundancy; Windows-LNK-check-disk-first; nested-`cargo build`-under-`--workspace` local deadlock) are graduation items — see the gap-analysis `Gotchas graduation` section. These belong in a `docs/gotchas.md` consolidation pass.

### `STAGE-PROMPT-PROTOCOL.md` updates carrying forward (v1.9 candidates)

- A pre-red `<pre_red_mechanical_sweep>` step (`cargo fmt` + `clippy --fix` + `prettier --check` + `tsc` on test files) — four stages ran red-commit amendments for this class.
- A `<schema_mirror_ripple_check>` authoring slot — the inverse of `<construction_reachability_check>`: any `String→generated-enum` / schema-field-shape change inventories the SQLite `CHECK` mirror + every in-source test asserting the old literal/migration-count (D1's CQ-6).
- Generalize `<wire_signature_audit>` to a `<data_flow_audit>` for renderer stages — assert the source data is renderer-reachable through the shipped bridge, not just that the IPC command exists (E's lock-reachability falsification).
- An environment-precondition check (free disk / process state) in the §7 self-correction loop for Windows OS-toolchain (linker) failures (D2's disk misdiagnosis).
- A "Carry-forward convergence" line in `VERIFIER-RETROSPECTIVE-TEMPLATE.md` (M07.V Decision 7 — three Dec-6 🟡s converging on one next-milestone stage should be flagged as a coupled set).

### M08 stage prompts — known constraints to encode

- **M08.A inherits a coupled set of four M07.V carry-forwards** that all converge on the Builder Canvas Test/Run path: 🟡 #2 (`skills_lock::verify` has no production load-path caller — wire into the artifact-load path), 🟡 #3 (`McpDispatcher::on_server_connected` has no production connect-handler caller), 🟡 #5 (agent-with-tools production driver absent — the loop is exercised only by the assembled integration test), and 🟡 #4 (local-file picker UI not shipped — MVP §M7 criterion). M08.A's `<read_prior_milestones>` must cite the M07.V retrospective and treat #2/#3/#5 as a coupled wire-up.
- **The M07.5 fix-cycle runs before M08 Stage A** — see Verdict.

### Open issues filed

- None blocking. The `docs/gotchas.md` consolidation and the v1.9 protocol-iteration session are the tracked non-blocking items.

---

## Verdict

- [ ] **Pattern held across M07.** ~~Proceed to M08.1.~~
- [x] **Pattern held but with friction.** Apply soft-gate fixes from the stage retrospectives before M08.1; **run the M07.5 fix-cycle (M07.V 🔴 #1) before M08 Stage A.** Confidence in the prompt-driven approach: **medium-to-high.**
- [ ] **Pattern strained.**

**Rationale.** All six hard gates passed in every stage A–E; aggregate axis means are healthy (Process 37.5/40, Product 37.83/40, Pattern 29.33/35) and consistent with M06. M07 shipped its full headline scope: the `skills.lock` integrity primitive (ADR-0014), the import-pipeline backend, the Builder Import panel (ADR-0015), and — the milestone's load-bearing achievement — the **ADR-0011 (a)–(d) concrete-construction discharge**, closing the largest open architectural carry-forward in the ledger (D1 = (a)–(c) construction; D2 = (d) the agent-with-tools loop). The M06.5 `token_usage = 0` finding is **RESOLVED in production at D2**, proven by the assembled regression. Stage V's mandatory integration smoke ran 1/1.

The friction was real but bounded and diagnosable: D1's 1.4× over-run (the under-scoped CQ-6 schema-mirror ripple) and D2's >2× total elapsed (a Windows disk-exhaustion misdiagnosis) — both non-recurring with named structural fixes; recurring phase-doc-vs-code drift caught at every pre-flight (no instance reached code); a recurring mechanical-test-file-amendment class.

Stage V (outcome **Sound but rough**) produced **1🔴 / 4🟡 / 1🟢**:
- **🔴 #1** (`tier_gate` defined but never invoked — a Novice "Reject" does not roll back the install) is a genuine spec §8.security L4 contract drift. Per the maintainer's adjudication (commit `8a861cd`) it is **waived to a dedicated post-M07 M07.5 fix-cycle** via **ADR-0016** (filed this closeout). M07 merges with finding #1 open and carried forward; the blast radius is bounded (no production code path loads/executes an imported artifact in v0.1 — V #2/#5) and M07.5 closes the gap before M08 Stage A introduces that path.
- **4🟡** (#2/#3/#5 Dec-6 driver-absent + #4 file-picker UI) → Carry-forward to **M08.A** (a coupled set).
- **1🟢** (#6 `token_usage.rs` flat path vs `projectors/`) → `docs/tech-debt.md` (TD-014, logged this closeout).

The verdict is "held but with friction" rather than "held": V's 🔴 is a real shipped contract gap (waived, not absent), and the recurring phase-doc-authoring friction is now a four-milestone trend warranting the v1.9 protocol iteration. It is not "strained" — no hard gate failed, the headline scope shipped complete and verified, and every finding has a named, scheduled resolution.

---

## User-review notes

> User reviews this summary as part of the final stage's PR (the three-artifact bundle: code diff + retrospectives/summary + gap-analysis entry). Approval here gates M07.5 + the next parent milestone.

- [Empty until user reviews]

---

## Sign-off

**Claude:** This summary aggregates the per-stage retrospectives for M07 (A, B, C, D1, D2, E) + the M07.V verifier retrospective. It is my honest assessment of how the parent milestone went and what the protocol should carry forward. M07 closed the ADR-0011 (a)–(d) carry-forward and shipped the registry-import feature + the `skills.lock` primitive; Stage V's 🔴 #1 is waived to the M07.5 fix-cycle via ADR-0016. User review and approval pending. The M07.5 fix-cycle runs before M08 Stage A; M08 does not begin until this summary and the M07.5 fix-cycle are approved.

**Surfaced at:** 2026-05-19 (UTC)
