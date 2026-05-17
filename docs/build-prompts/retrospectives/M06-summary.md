# M06 — Parent-Milestone Summary

> **Parent milestone:** M06 of M11 in `docs/MVP-v0.1.md` (MCP Basic)
> **Authored by:** Claude (per `CLAUDE.md` §19)
> **Aggregates:** M06.A, M06.B, M06.C, M06.D, M06.E, M06.F stage retrospectives + M06.V verifier retrospective
> **Created at:** 2026-05-16 (UTC)
> **Total elapsed:** ~31 h across A–F + V (code/test/commit on the build machine)
> **Estimated:** ~36–48 h (phase doc Document Structure table; ~24–33 h at M05's 0.64× calibration baseline)

---

## Stage trail

| Stage | Status | Stage commit | Retrospective | Outcome |
|---|---|---|---|---|
| Stage A | Committed | `030089f` | `M06.A-retrospective.md` | **Not-ready → maintainer override** (G5 fail: TDD ordering Process Q3=2; work-quality gates all green; v1.7 `<tdd_discipline>` is the structural close) |
| Stage B | Committed | `c0d001d` | `M06.B-retrospective.md` | Sound |
| Stage C | Committed | `2ff18ca` (red) + `c9d3000` (impl) + `7a83ff2` (additive) | `M06.C-retrospective.md` | Sound |
| Stage D | Committed | `17aeb9b` (red) + `23bc369` (impl) + `1d377d0` (style) + `30b5f67` (additive) | `M06.D-retrospective.md` | Sound |
| Stage E | Committed | `37d96bf` (red) + `a172ba0` (impl) + `c9e7eff` (style) | `M06.E-retrospective.md` | Sound |
| Stage F | Committed | `1e2bc23` (red) + `2adaf6a` (impl) + `24670e5` (style) | `M06.F-retrospective.md` | Sound |
| Stage V | Committed | `20c3da5` | `M06.V-retrospective.md` | Sound (0🔴 / 2🟡 / 2🟢) |

All stages on parent-milestone feature branch `claude/m06-mcp-basic`. The M06 PR drafts after this summary + the gap-analysis entry land and surfaces all stage commits + retrospectives + the summary + the gap-analysis entry together. (Stage A's `1270429` merge of PR #74 + `f9efb25`/`bc9fbc3`/`586f1ed` protocol-v1.7 merges + `b651981`/`80c36b2` proposal-0001 merges are interleaved on the branch per the M06 git log.)

---

## Aggregate scoring (sum across stages)

### Process axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | 34 | /40 |
| Stage B | 40 | /40 |
| Stage C | 37 | /40 |
| Stage D | 39 | /40 |
| Stage E | 39 | /40 |
| Stage F | 39 | /40 |
| **Mean** | **38.0** | /40 |

### Product axis

| Stage | Total | /40 |
|---|---|---|
| Stage A | 37 | /40 |
| Stage B | 38 | /40 |
| Stage C | 38 | /40 |
| Stage D | 38 | /40 |
| Stage E | 39 | /40 |
| Stage F | 39 | /40 |
| **Mean** | **38.17** | /40 |

### Pattern axis

| Stage | Total | /35 |
|---|---|---|
| Stage A | 29 | /35 |
| Stage B | 32 | /35 |
| Stage C | 31 | /35 |
| Stage D | 29 | /35 |
| Stage E | 30 | /35 |
| Stage F | 30 | /35 |
| **Mean** | **30.17** | /35 |

### Stage V verification axes

| Axis | Score | /5 |
|---|---|---|
| Coverage adequacy | 4 | /5 (runtime-main ≥95% gate not Windows-local-measurable — gotcha #56; runtime-mcp gate independently green at 97.16%) |
| Finding signal-to-noise | 5 | /5 (every finding traces to file:line + spec/phase-doc claim; no false positives) |
| Fresh-context discipline | 5 | /5 (clear-and-paste held; ADR-0011-into-V `<scope_change>` mechanism applied correctly) |
| **Sum** | **14** | /15 (≥9 soft-signal threshold) |

---

## Cross-stage trends

### Friction patterns that recurred

- **Phase-doc-vs-codebase drift, surfaced at pre-flight every stage.** A: `tests/unit/lib/graphStore.test.ts` path + 3-vs-4 schema kinds (pre-existing M05.A class); B: `NonEmptyString` $ref non-existence + rmcp API drift (caught by `<dependency_audit_check>`); D: ADR-0010 cross-crate dependency cycle (`<phase_doc_inventory_audit verified>` over-trusted symbol existence); E: renderer pseudocode drifted from shipped store shape; F: F.1 over-framed an injection whose ctor inputs don't exist in-shell (→ ADR-0011). The v1.6 slot suite caught every instance at pre-flight, but the recurrence (5 of 6 work stages) confirms the M05 finding that phase-doc-authoring discipline — not per-stage slots alone — is the structural fix.
- **Windows-local `cargo llvm-cov` subprocess-build flake (gotcha #56).** Recurred at D, E (renderer-only, structurally unaffected), F, and V — `drone_ipc_loopback.rs` nested-cargo build aborts under llvm-cov instrumentation on Windows; `--skip` name-substring mitigation does not apply (test names lack the filename prefix). Logged TD-005 at V. CI-Linux remains the authoritative gate; runtime-mcp gate independently green.
- **TDD-discipline tightening across A→C.** A failed G5 on TDD ordering (Process Q3=2) → maintainer override; B closed the gap via an explicit pre-implementation surface (Process Q3 lifted 2→5); C ran the strict two-commit red-phase pattern (user override + web evidence); D/E/F ran it as settled discipline. The v1.7 `<tdd_discipline strict="true">` protocol (merged PR #76, commit `2649c36`) is the structural close — M06.C is its empirical reference implementation (~10% overhead, structural defense against gotcha #66).

### Pattern-level wins

- **ADR-0009 closure satisfied with verified production call sites.** Stage A wired `enforcer.check` before the `ToolInvoked` push (`event_pipeline.rs:215`) + `narrow` before `AgentSpawned` (`agent_sdk.rs:356`); M06.V Wire pass traces #1 + #2 **SATISFIED** — the first real-world test of the ADR-0008 waiver→next-milestone-deliverable chain closing cleanly.
- **v1.6 phase-doc slot suite delivered measurable time savings.** `<dependency_audit_check>` caught 3 rmcp API drifts at design time (B); `<scope_change>` gave V visibility into the M07 boundary (A, D, F); novel-protocol Stage B came in at 0.35× estimate (vs the M02.C anthropic_sse 1× anchor).
- **ADR-driven scope discipline.** ADR-0010 (dependency inversion for MCP dispatch) + ADR-0011 (F-scope = seam, not running app) kept the cross-crate dependency direction clean and the M07 carry-forward explicit rather than silently cut. The `Arc<dyn _>` injection-seam archetype (M02.C/M05 precedent) held across the SDK→src-tauri boundary.
- **The `*_with` / mock-transport / path-agnostic-persistence archetypes ported cleanly to a new crate.** `runtime-mcp` reuses `runtime_drone::db::init` for WAL+migrations; `Registry::open(path: &Path)` follows the tier/audit path-agnostic archetype; MockTransport mirrors the M05.C1 `sandbox_ipc` test archetype.

### Surprises across the parent milestone

- **First milestone under v1.6 protocol; first `<simplify_pass>` at closeout.** The simplify pass (three parallel review agents against the M06.A..HEAD cumulative diff) surfaced a small, mostly-mechanical proposal set — the cumulative diff was found broadly clean (good layering, ADR-0011 scope boundary cleanly respected). Highest-signal items: transport-helper copy-paste (stdio/http), `kind_to_ref`/`tier_to_ref` 3-module duplication, a misleading `spawn_health_pinger` "abort on drop" docstring, and the documented-by-design `apply_mcp_dispatch` Invoked-arm empty-`agent_id` split (gotcha #68 / D-frozen wire test). See the gap-analysis Simplify-pass subsection for disposition.
- **Novel-protocol calibration anchor dropped sharply.** B's rmcp transport stage hit 0.35× (vs the assumed ~1× M02.C anchor) because rmcp's `ServiceExt::serve` + `Peer::call_tool` is far more abstracted than hand-rolled reqwest+SSE — fewer wire-format bytes to own. Recommendation: future novel-protocol stages anchor at ~0.4–0.5× when the phase doc has full v1.6 slot coverage and a feature-complete SDK exists.
- **gotcha #68 (empty `agent_id`) became a load-bearing design constraint, not just a trap.** D flagged it; F resolved it by run-loop-emits-Invoked-directly (leaving `apply_mcp_dispatch` + the D-frozen `mcp_dispatch_wire.rs` untouched) rather than a signature change; V confirmed the split is intentional and the Ambiguous/Blocked paths ride the frozen mapping verbatim.

### Hard gate violations across the milestone

- **M06.A — G5 FAIL (Process axis Q3 = 2, TDD ordering lapse).** Disposition: documented maintainer override per `CLAUDE.md` §12 (project process governance is user-domain). Work-quality gates all green (workspace 93.22% / drone 95.79% / main 97.14% / sandbox 96.11%; 8 integration tests covering ADR-0009 closure; 0 clippy; 0 audit). Structural close: v1.7 `<tdd_discipline>` slot (proposed in A's retro, validated in C, **merged in PR #76 / commit `2649c36`** before D ran). No other hard-gate violations across B–F or V. This is the negative case the v1.7 protocol exists to prevent; M06.C is the positive reference implementation.

---

## Time-box accuracy

| Stage | Estimated | Actual | Ratio |
|---|---|---|---|
| Stage A | 7 h | ~6 h | 0.86× |
| Stage B | 10 h | ~3.5 h | 0.35× |
| Stage C | 8 h | ~5 h | 0.625× |
| Stage D | 6 h | ~6 h | 1.0× |
| Stage E | 5 h | ~4.5 h | 0.9× |
| Stage F | 3.5 h | ~4 h | ~1.1× |
| Stage V | ~3 h | ~2 h | ~0.7× |
| **Total** | **~42.5 h** | **~31 h** | **~0.73×** |

Total ratio 0.73× — within the cumulative band (M01 0.3× / M02 0.7× / M03 0.32× / M04 0.55× / M05 0.64× / **M06 0.73×**). The trend is converging toward 1× as archetypes settle and novel-protocol stages benefit from the v1.6 slot suite. Estimation method is sound; no correction needed for M07. Note the calibration refinement: novel-protocol stages (Stage B class) should anchor at ~0.4–0.5×, not the M02.C-era ~1×, when an abstracted SDK + full v1.6 slot coverage are present.

---

## Decisions to apply before the next parent milestone

### `CLAUDE.md` updates carrying forward

- §5/§6 runtime-mcp gate semantics + per-module baseline + `cargo llvm-cov clean` note — **already landed** during M06.B/C (CLAUDE.md §5 documents the runtime-mcp gate, the lib.rs/transport/auth_keyring/lifecycle exclusions, the M06.C `clean`-before-measure gotcha). No further §5 edit required at G beyond confirming the M06.F `sdk/agent_sdk.rs` `try_mcp_dispatch`/`with_mcp_dispatch` per-module coverage baseline is recorded in the gap-analysis entry (no new exclusions).
- The v1.7 strict two-commit `<tdd_discipline>` is already encoded in CLAUDE.md §5/§6 (merged PR #76). No carry-forward edit.

### `STAGE-PROMPT-PROTOCOL.md` / protocol updates carrying forward

- **`<construction_reachability_check>` slot proposal** (M06.D + M06.F). `<phase_doc_inventory_audit verified="true">` over-trusted symbol/file existence twice — D (cross-crate dependency cycle → ADR-0010) and F (an injection whose ctor inputs don't exist in-shell → ADR-0011). Proposal: the inventory audit must assert *construction-graph reachability*, not just file/symbol existence. Carry to the post-M06 protocol-iteration session ("M06.6").
- **`<wire_signature_audit>` slot proposal** (M06.E). Phase-doc renderer pseudocode drifted from the shipped Tauri-command param shape (`mcp_test_connection {config}`); a slot pinning ipc wrappers to actual command signatures would catch it at authoring time.
- **`<wire_trace_vs_adr_reconcile>` authoring check** (M06.V Decision 6). A phase-doc Wire trace (V.3 trace #6) was written against an architecture later changed by an ADR (ADR-0010 moved the resolver into `McpDispatcher`); reconcile Wire traces against accepted ADRs at phase-doc-authoring time.
- **Verifier-template codification** (M06.V Decision 6). Codify the rule "primitive delivered+tested but production driver absent, root cause = an already-accepted ADR's named carry-forward ⇒ 🟡-with-mandatory-carry-forward-enumeration" so V doesn't re-derive it each milestone.
- **M07.V `--features integration` reference-MCP-server smoke** (M06.V Decision 7). The mock-only Behavior pass cannot rule out rmcp wire-format correctness (the `transport/stdio.rs`+`http.rs` excluded holdout); by M07 a real dispatch path exists — require the integration smoke in M07.V's Behavior pass.

### M07 stage prompts — known constraints to encode

- **ADR-0011 (a)–(d) concrete-construction carry-forward** is M07's load-bearing input: `impl ConnectionResolver for McpClient`; shell-constructed `CapabilityEnforcer` + `NamespaceResolver` (populated from connected servers); construct + pass `Some(dispatch)` through `run_smoke_session_with` (or successor); the first agent-with-tools loop emitting a resolvable MCP `ToolUse`.
- **M06.V 🟡 #1** — extend ADR-0011 carry-forward (b) to explicitly enumerate: `McpClient` (or its M07 successor) drives `NamespaceResolver` connect/disconnect re-resolution on server add/remove + emits `tool_alias_ambiguous` for newly-ambiguous short names. M07.V Wire trace #6 is the expected endpoint.
- **M06.V 🟡 #2** — M07 Stage A pre-flight X.2 truth-up (M05.V-#3 precedent): correct M06 D.2 line 1887 (`crates/runtime-mcp/tests/mcp_dispatch_integration.rs` + note the `runtime-main` counterpart `mcp_dispatch_wire.rs`) + the V.3 Behavior-harness `cargo test -p runtime-main … mcp_dispatch_integration` crate scope. Bundle with the TD-006 V.3/A.4.4/§6 regex reconcile.
- **MCP-schema divergence ADR** (M02–M05 carry-forward) — ADR-0006 (`mcp-servers-schema`) + ADR-0010/0011 now cover the MCP surface; confirm at M07 whether a dedicated divergence ADR is still owed or subsumed.

### Open issues filed

- None opened as GitHub issues (per `CLAUDE.md` §8/§12 the carry-forwards live in this summary + the gap-analysis Carry-forward section + `docs/tech-debt.md`, not the issue tracker, for the prompt-driven build).

---

## Verdict

Mark one:

- [ ] **Pattern held across M06.** Proceed to M07.1 with the protocol updates above applied. Confidence in the prompt-driven approach: high.
- [x] **Pattern held but with friction.** Apply soft-gate fixes from stage retrospectives before M07.1. Confidence: medium-high.
- [ ] **Pattern strained.** A hard gate failed in one or more stages; or aggregate scores indicate sustained pattern-level friction. Spend a session iterating before M07.1. Confidence: low until protocol is updated.

**Verdict rationale:** Five of six work stages (B–F) plus V were Sound with healthy axis means (Process 38.0/40, Product 38.17/40, Pattern 30.17/35) and a 0.73× time-box ratio in-band. The one hard-gate violation (M06.A G5 TDD-ordering) was a *protocol-fit* failure, not a *work-correctness* failure — every work-quality gate passed, the ADR-0009 closure is verified, and the structural close (v1.7 `<tdd_discipline>`) was proposed in A's retro, empirically validated in C, and **merged before D ran**. That is the protocol self-correcting mid-milestone exactly as designed, which is why the verdict is "held but with friction" rather than "strained." The recurring phase-doc-vs-codebase drift (5/6 stages, all caught at pre-flight) and the gotcha #56 Windows-local llvm-cov gap (TD-005) are the soft-gate items to fold into the M06.6 protocol-iteration session before M07.1.

---

## User-review notes

> User reviews this summary as part of the final stage's PR. Approval here gates the next parent milestone.

User-review notes:

- [Empty until user reviews]

---

## Sign-off

**Claude:** This summary aggregates the per-stage retrospectives for M06 (A–F) + the Stage V verifier retrospective. It is my honest assessment of how the parent milestone went and what the protocol should carry forward. The one hard-gate violation (M06.A) is recorded with its documented maintainer override and its structural close (v1.7, merged mid-milestone). User review and approval pending. The next parent milestone (M07) does not begin until this summary + the M06 gap-analysis entry are approved.

**Surfaced at:** 2026-05-16 (UTC)
