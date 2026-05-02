# M01 — Parent-Milestone Summary

> **Parent milestone:** M01 of M11 in `docs/MVP-v0.1.md`
> **Authored by:** Claude (per `CLAUDE.md` §19)
> **Aggregates:** M01.A, M01.B, M01.C, M01.D stage retrospectives
> **Created at:** 2026-05-02 (UTC)
> **Total elapsed:** ~10.5 hours (A 2h + B 4h + C 3h + D 1.5h)
> **Estimated:** 2 weeks elapsed per `docs/MVP-v0.1.md` (29–50h Claude time across A 5–8h, B 5–8h, C 12–18h, D 4–6h)

---

## Stage trail

| Stage | Status | Stage commit | Retrospective | Outcome |
|---|---|---|---|---|
| Stage A | Committed | `3a72727` (+ retro `0a49b4d`) | `M01.A-retrospective.md` | Sound |
| Stage B | Committed | `a3c8188` | `M01.B-retrospective.md` | Sound |
| Stage C | Committed | `064f8f4` (+ codification `1dec4ba`) | `M01.C-retrospective.md` | Sound but rough (G4 contingent on user direction; codification commit `1dec4ba` resolved the open coverage-gate decision) |
| Stage D | Awaiting approval | uncommitted | `M01.D-retrospective.md` | Sound |

All stages on parent-milestone feature branch `claude/m01-foundation`. The M01 PR drafts after this summary lands and surfaces all stage commits + retrospectives + this summary together. Per `CLAUDE.md` §20, Stage E (Phase Closeout: Gap Analysis) appends a gap-analysis entry as the final commit on this branch before the PR pushes.

---

## Aggregate scoring (sum across stages)

### Process axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | 38 | /40 |
| Stage B | 36 | /40 |
| Stage C | 37 | /40 |
| Stage D | 38 | /40 |
| **Mean** | **37.25** | /40 |

### Product axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | 32 | /35 (one row N/A) |
| Stage B | 37 | /40 |
| Stage C | 35 | /40 |
| Stage D | 39 | /40 |
| **Mean** | **35.75** (with A pro-rated to /40) | /40 |

### Pattern axis

| Stage | Total | /35 |
|---|---|---|
| Stage A | 29 | /35 |
| Stage B | 26 | /35 |
| Stage C | 27 | /35 |
| Stage D | 27 | /35 |
| **Mean** | **27.25** | /35 |

All four stages cleared all hard gates (Stage C's G4 was "needs user input" rather than failed and was resolved by the user-approved codification commit `1dec4ba`). All four stages cleared all soft gates. No row in any stage scored below 3/5.

---

## Cross-stage trends

### Friction patterns that recurred

- **CI yaml `rustdoc::missing_docs` lint name** — appeared in M01.A friction events (Sev 2), then in M01.B as "still not applied" (Sev 1), then in M01.C as "still not applied to Stage C" (Sev 1). **Resolved at the codification commit `1dec4ba` and confirmed clean in Stage D**: the current `.github/workflows/ci.yml` uses `RUSTDOCFLAGS: '-D missing_docs -D rustdoc::broken_intra_doc_links'`. *Lesson: prior-stage retro decisions need a forcing function — not just "next stage will fix this" but a CI gate or hook that fails until applied. The decision survived three stages without being applied.*
- **Template / spec drift surfacing as Sev-1/2 friction** — every stage hit one or more "the template/spec said X but reality required Y" moments: Stage A toolchain pin (`1.80` → `stable`); Stage B typify version (`0.1` → `0.6`) + external `$ref` resolver; Stage C `JsonCodec` pseudo-code → `LinesCodec` + manual JSON; Stage D fuzz Cargo.toml empty `[workspace]` line. *Lesson: every stage's retrospective should call out these as "milestone document needs an edit before re-running this stage from a fresh session" rather than treating them as one-off friction. Stage D's "Decisions for the next stage" section captures this for Stage D's items.*
- **Time-box estimates systematically over-shoot for Claude-driven work** — Stage A 5–8h → ~2h actual (0.3×); Stage B 5–8h → ~4h actual (0.6×); Stage C 12–18h → ~3h actual (0.2×); Stage D 4–6h → ~1.5h actual (0.3×). *Lesson: the milestone-prompt time estimates were likely calibrated for human-driven work or for early-Claude-Code maturity. Tighten estimates by ~3× for analogous future stages.*

### Pattern-level wins

- **The do-not-commit-until-approved rule held in every stage.** Hard Gate G1 cleared 4/4 stages; no surprise commits. The retrospective + diff-stat + commit-message + PR-draft surface pattern is reliable.
- **Pre-flight checklist surfaces orientation gaps early.** Every stage stated deliverable + test plan before code (`CLAUDE.md` §16). Stage B and Stage C explicitly read prior retrospectives' Decisions sections, applying many of them. Stage D applied the M01.C codification decision (option a.2).
- **TDD discipline + structural-holdout transparency.** When Stage C couldn't hit a literal 100% drone coverage gate cross-platform, the response was to surface the structural infeasibility honestly with three concrete options rather than burning rounds on increasingly contrived scaffolding. The codification commit `1dec4ba` codified option (a.2). This pattern — surface, propose options, wait, codify — is the prototype for future cross-platform test-coverage decisions.
- **Property-test-driven serde round-trip discipline.** Stage B and Stage C added `proptest` suites for serialize→deserialize→serialize stability across `AgentEvent`, `DroneEvent`, `DroneCommand`. Stage D's fuzz harness on `drone_command_decode` extends the same surface (untrusted-bytes-don't-panic invariant) without adding any new test scaffolding. The wire format is now triple-protected: schema validation in the type system, property tests on round-trip stability, and a fuzz harness on the decode path.
- **The `*_with` / `*_inner` test-seam pattern.** M01.C surfaced this as a coverage-lifting refactor (`run_inner` + `wait_and_handle_with` accepting injectable signal futures). Adopted in `lib.rs` and `shutdown.rs`; both modules score ≥84% line on the modules that matter while the OS-signal entry points are documented holdouts. *Recommendation:* document this pattern in `CLAUDE.md` §9 before M02; it generalizes everywhere we wrap an OS signal / timer / socket future.

### Surprises across the parent milestone

- **Tauri 2.x dependency tree depth.** 444 transitive deps; release builds take 3.5+ minutes for placeholder crates. *M02+ implication:* aggressive caching (`Swatinem/rust-cache@v2` is already in CI) and possibly a Tauri-skipping CI lane for code-only changes.
- **Generated-code volume.** ~9000 lines of typify output across 5 files. Required `--ignore-filename-regex` exclusions for coverage and an explicit allow-list (`clippy::pedantic`, `clippy::nursery`, `clippy::all`, `missing_docs`, `unused_imports`, `rustdoc::invalid_html_tags`) on every generated file. *M02+ implication:* every schema change carries a measurable CI cost; bundled schema bumps over scattered ones.
- **Coverage gate at literal 100% is structurally infeasible cross-platform** for OS-signal-driven async fns. The codification of option (a.2) — `--fail-under-lines 95` with `lib.rs` + `shutdown.rs` excluded — is the durable answer for safety primitives that wrap real OS signals. Future safety primitives (M02 `AnthropicProvider`'s SSE retry loop, M05 capability enforcer) will need the same template: testable `_inner`/`_with` variants + a thin OS-fronted wrapper, with the wrapper documented as a coverage holdout.
- **Schema cross-file `$ref` requires custom resolution.** typify (any version) does not handle external refs. M01.B's `xtask::resolve_external_refs` (~150 lines including helper fns) is the project's solution; it imports ALL `$defs` from referenced files (not just the named ones) to satisfy transitive internal refs. *M02+ implication:* the resolver is robust but is a one-time learning cost; future schema changes are cheap.

### Hard gate violations across the milestone

- **None.** All four stages cleared G1–G5. Stage C's G4 was technically "needs user input" rather than "failed," and that was resolved by the user-approved codification commit `1dec4ba`. M01 is ready to merge from the hard-gates perspective.

---

## Time-box accuracy

| Stage | Estimated | Actual | Ratio |
|---|---|---|---|
| Stage A | 5–8 h | ~2 h | 0.3× |
| Stage B | 5–8 h | ~4 h | 0.6× |
| Stage C | 12–18 h | ~3 h | 0.2× |
| Stage D | 4–6 h | ~1.5 h | 0.3× |
| **Total** | 26–40 h | **~10.5 h** | **~0.3×** |

The total ratio is 0.3× — **much faster than estimated, in the safe direction**. This is consistent across all four stages. The estimation method for Claude-driven implementation stages on a well-specified milestone prompt is systematically high by ~3×. **Correction for M02:** Author M02 stage prompts with time-box estimates 1/3 the size of M01's stage estimates (so M02 Stage A target ~2–3h instead of 5–8h, etc.). Cross-check at M02 Stage A retrospective; if the new estimates also come in fast, tighten further.

---

## Decisions to apply before the next parent milestone

Drives `CLAUDE.md` / `TEMPLATE.md` / per-milestone-prompt updates that landed (or should land) before M02.A's session opens.

### `CLAUDE.md` updates carrying forward

- [ ] Apply M01.A retro decisions (gotchas #21 rust-toolchain stable, #22 lints workspace+override).
- [ ] Apply M01.B retro decisions (gotchas #23 typify external `$ref`, #24 generated-code coverage exclusion + allow-list, #25 round-trip stability for typify-emitted types, #26 `.cargo/config.toml` xtask alias).
- [ ] Apply M01.C retro decisions (gotchas #27 intra-workspace path-dep version, #28 `*_with`/`_inner` OS-signal test-seam pattern). Codification of §6 coverage gate semantics is **already applied** by commit `1dec4ba`.
- [ ] Apply M01.D retro decisions (gotcha #29 Windows libfuzzer DLL, #30 cargo-fuzz empty `[workspace]` table).
- [ ] Update §5 Coverage thresholds note: define the M02 baseline-delta gating mechanism (per `docs/gap-analysis.md` Pre-M01 carry-forward).
- [ ] Document the `*_with` / `*_inner` test-seam pattern in §9 (style + naming) as the canonical TDD-friendly approach to OS-signal-driven async functions. See M01.C surprise events row 2.
- [ ] **Pattern axis friction class.** M01 Pattern axis aggregated 27.25/35 — driven by Stage C 'Sound but rough' coverage-gate ambiguity, codified at commit `1dec4ba`. Verify `CLAUDE.md` §5/§6 and `TEMPLATE.md` carry the codification forward so M02 Stage C-equivalent work doesn't re-litigate. Per `CLAUDE.md` §10 (don't-touch zones additions): no agent should propose alternate coverage interpretations without an ADR.

### `TEMPLATE.md` updates carrying forward

- [ ] Consider adding a "Tooling artifacts" sub-section to the [LIVE] tables for events that are neither friction (no rework time) nor surprise (tooling behaves as documented). Windows libfuzzer DLL pairing is the prototype.
- [ ] Consider adding a "Coverage holdouts" subsection — coverage gaps that are structural (cross-platform infeasibility) are a different category from procedural friction. M01.C surfaced this; M01.D applied it via the codification commit.
- [ ] **TEMPLATE.md Stage D STEP 5 fix** (post-M01 protocol-iteration work). `TEMPLATE.md` Stage D STEP 5 currently says "commit + push + open PR after Stage D approval" which contradicts `CLAUDE.md` §20 (Stage E gates the PR). Fix in protocol-iteration PR before M02 authoring: Stage D = commit only; Stage E = commit + push + PR.

### Per-milestone-prompt template updates carrying forward

- [ ] Tighten time-box estimates in `TEMPLATE.md` and the M02-onward stage prompts: Claude-driven implementation stages on a well-specified prompt come in at ~0.3× the human-calibrated estimate. Apply correction.
- [ ] Add a "Prior-stage decisions to apply" preamble to every non-first stage's CLI prompt that explicitly enumerates the decisions Claude must apply before any code, with file:line citations. The current "read prior retrospectives" instruction works but a concrete checklist would forcing-function the application (see M01.A→C `rustdoc::missing_docs` recurrence).
- [ ] Add a "What to verify locally vs CI-only" subsection to STEP 3 (Verify) for any stage that uses cross-platform-finicky tooling (cargo-fuzz on Windows, integration features that may not exist yet, OS-signal tests).
- [ ] **Time-box estimation method tightening for M02.** M01 estimated 29–46h, ran in ~9–14h (0.3× ratio). M01's method overestimated by ~3×. For M02, recalibrate: estimate by reading actual M01 stage durations (from retrospectives) for analogous work, not from intuition.

### M02 stage prompts — known constraints to encode

- [ ] M02 Stage A "Read first" must include the M01.A/B/C/D retrospectives + this summary + the M01 gap-analysis entry. The pattern of reading prior decisions is load-bearing.
- [ ] M02 must define the coverage baseline-delta gating mechanism in `CLAUDE.md` §5 BEFORE Stage A's main work, since `main` will accumulate a baseline once M01 merges.
- [ ] M02 polish stage (if any) target ~2h, not ~4–6h.
- [ ] M02 must apply the `*_with` / `*_inner` pattern to the `AnthropicProvider` SSE loop — it's the same shape (async fn wrapping a long-lived I/O future).
- [ ] M02 must align the renderer's component patterns with whatever exists in M01 (none yet — but M03 onward must respect this; see pre-M01 addendum carry-forward "UI consistency").
- [ ] M03 prep must include the Phase 3 React Flow + Zustand spec expansion (per pre-M01 carry-forward; flagged again by M01-summary).

### Open issues filed

- [ ] None (no GitHub issues opened during M01; all friction/decisions resolved within the session loop or surfaced for retrospective application).

---

## Verdict

Mark one:

- [x] **Pattern held across M01.** Proceed to M02.A with the protocol updates above applied. Confidence in the prompt-driven approach: **high**.
- [ ] **Pattern held but with friction.** Apply soft-gate fixes from stage retrospectives before M02.A. Confidence: medium.
- [ ] **Pattern strained.** A hard gate failed in one or more stages; or aggregate scores indicate sustained pattern-level friction.

**Rationale:** All hard gates passed in all four stages (G4 in Stage C was contingent on user direction and was resolved by the user-approved codification commit). Aggregate axis means: Process 37.25/40, Product 35.75/40 (with Stage A's coverage row pro-rated), Pattern 27.25/35. Time-box accuracy was systematically fast (0.3× ratio) — a calibration concern, not a quality concern. No blocking surprises; no friction event ≥ Sev 4 except the M01.C coverage gate (resolved before M01 PR opens).

The protocol is working. The biggest improvement opportunity for M02 is the prior-decision application mechanism — three stages let the `rustdoc::missing_docs` lint-name fix slip without applying it. M02 stage prompts should treat prior-stage decisions as a hard checklist applied before code, not as guidance.

---

## User-review notes

> User reviews this summary as part of the final stage's PR. Approval here gates the next parent milestone.

User-review notes:

- [Empty until user reviews]

---

## Sign-off

**Claude:** This summary aggregates the per-stage retrospectives for M01. It is my honest assessment of how the parent milestone went and what the protocol should carry forward. M01 is ready to merge from a hard-gates and product-quality perspective. M02 (event pipeline + AnthropicProvider + Tauri shell) does not begin until this summary is approved alongside the M01 PR.

**Surfaced at:** 2026-05-02 (UTC).
