# Coverage policy — ledger & rationale (the append-only detail behind CLAUDE.md §5/§6)

> **What this is.** The full per-module baselines, per-milestone
> exclusion history, carry-forwards, and gate-mechanism detail that used
> to live inline in `CLAUDE.md` §5. It was extracted to keep `CLAUDE.md`
> (auto-loaded every session) small; **nothing was deleted — it was
> relocated here losslessly.**
>
> **What is NOT here.** The *enforced* rules — the ≥80 / ≥95 thresholds,
> the current active `--ignore-filename-regex` exclusion list, and the
> exact `cargo llvm-cov` commands — stay inline in `CLAUDE.md` §5/§6
> (those are violated by not having them in working memory). This file
> is **consult-on-change** reference: read it before adding/altering any
> coverage exclusion or threshold.
>
> **Read-every-time wiring.** Registered in `CLAUDE.md` §2 (read-first)
> and §17 (reference index); listed in the stage-prompt template
> `<read_first>` for any coverage-touching stage. Same reliability
> mechanism as `docs/style.md` / `docs/gotchas.md`.
>
> **Change protocol (single source of truth).** This file is the
> source of truth for coverage *rationale + history*. The CLAUDE.md §5
> inline exclusion list, the §6 `cargo llvm-cov` commands, and
> `codecov.yml` are the *enforced mirrors*. **Any coverage change must
> update all four in the same PR**, and append a milestone entry to §C
> below. The closeout stage owns this reconciliation (see
> `SUMMARY-TEMPLATE.md` "Decisions to apply" + `STAGE-PROMPT-PROTOCOL.md`
> closeout `<deliverables>`).

---

## A. Rule + current active exclusions (mirror of CLAUDE.md §5)

- **≥80% line coverage** on all new code (Rust: `cargo-llvm-cov`;
  TS: `vitest --coverage`). Workspace gate excludes generated code +
  binary stubs: `--ignore-filename-regex "src.main\.rs|generated"`.
- **≥95% line on safety primitives**: drone (`crates/runtime-drone/`),
  provider/SSE pipeline + capability enforcer + plan state machine +
  snapshot/recovery (`crates/runtime-main/`), sandbox
  (`crates/runtime-sandbox/`), sandbox IPC client
  (`crates/runtime-main/src/sandbox_ipc/`), MCP
  (`crates/runtime-mcp/`).
- **Coverage must not drop vs the prior `main` block PR merge.** CI
  computes the delta (Codecov — §D).

**Rationale categories for every exclusion** (the "why" is always one
of these — name the category in the milestone entry when adding one):

1. **Binary stub** — `src/main.rs` (no testable logic; N/A).
2. **OS-signal-class holdout** — a function whose body installs an OS
   fence / wraps an OS-signal future / spawns an infinite loop on the
   calling process or thread; running it in-process poisons the test
   runner. Coverage attribution via subprocess integration tests.
   (drone `lib.rs`+`shutdown.rs`; sandbox `seccomp.rs`+`landlock.rs`;
   mcp `client/lifecycle.rs`.)
3. **Seam-vs-wrapper / OS-call holdout** — a thin wrapper that does a
   real OS/network call (`reqwest` to api.anthropic.com, OS keychain,
   `cfg`-platform `open()`), paired with a `*_with` / `from_streams`
   testable seam that IS unit-tested. (runtime-main
   `providers/anthropic.rs`, `key_store.rs`,
   `drone_ipc/connection.rs`, `sandbox_ipc/connection.rs`,
   `import/fetch.rs` (M07.C — the real `reqwest` artifact GET,
   seam-tested via `fetch_with` + injected `Fetcher`); mcp
   `transport/stdio.rs`+`http.rs`, `client/auth_keyring.rs`;
   src-tauri shell wrappers — §D Tauri patch-gate.)
4. **Pub-mod / re-export `lib.rs`** — declarations + re-exports only
   (sandbox `lib.rs`, mcp `lib.rs`).

**Current active exclusion set** (must equal the §6 commands +
`codecov.yml` byte-for-byte — drift = bug):

| Gate | `--ignore-filename-regex` | Floor |
|---|---|---|
| workspace | `src.main\.rs\|generated` | 80 |
| runtime-drone | `src.main\.rs\|generated\|src.lib\.rs\|src.shutdown\.rs` | 95 |
| runtime-main | `src.main\.rs\|generated\|src.providers.anthropic\.rs\|src.drone_ipc.connection\.rs\|src.sandbox_ipc.connection\.rs\|src.import.fetch\.rs` | 95 |
| runtime-sandbox | `src.main\.rs\|generated\|src.lib\.rs\|src.seccomp\.rs\|src.landlock\.rs` | 95 |
| runtime-mcp (`--features test-helpers`) | `src.main\.rs\|generated\|src.lib\.rs\|src.transport.stdio\.rs\|src.transport.http\.rs\|src.client.auth_keyring\.rs\|src.client.lifecycle\.rs` | 95 |

Note: the runtime-main 95 gate also conceptually excludes
`src/key_store.rs` (OS-keychain holdout) — it is not in the regex
because no production `key_store.rs` line is reached by the gated
crate's tests; documented here so a future change does not
"discover" it and add a redundant regex term. Wire-format logic for
the anthropic exclusion lives in `src/providers/anthropic_sse.rs`,
exercised end-to-end by `tests/anthropic_wiremock.rs` (8 tests:
happy path, auth, rate limit, tool use, thinking, server-emitted
error, malformed bytes skipped, partial-chunk reassembly).

---

## B. Per-module baselines (append-only — never regress without a retro entry)

A module must not regress below its recorded baseline without a
retrospective entry recording the reason.

**runtime-drone** (M01.C measured): `snapshot.rs` 100%; `db.rs`
98.82%; `heartbeat.rs` 98.59%; `command_handler.rs` 97.94%;
`ipc.rs` 84.70% (platform-cfg variants).

**runtime-sandbox**: `validator.rs` 96.30% line / 100% region
(M05.C1); `protocol.rs` 100% (M05.C1); `ipc.rs` baseline 92.58%
line / 94.01% region (M05.C1; lifted into the gate at C2 — must hold
≥95% on Linux CI); `seccomp.rs` 80.56% line / 72.09% region (M05 PR
#70 first Linux measurement — `install()` body untestable
in-process; **excluded from gate**); `landlock.rs` 76.25% line /
75.74% region (same; **excluded from gate**); `job_objects.rs`
95.12% line / 93.81% region (M05.C2; Windows-CI baseline — stays in
the gate via the seam-decomposed `create_job` / `apply_limits` /
`assign_process` / `win32_failure` test surface).

**runtime-main tier** (M05.D, Windows-local): `evaluator.rs` 100% /
100%; `matrix.rs` 100% / 100%; `persistence.rs` 97.45% line / 100%
region (4 uncovered = the `now_unix_ms` system-clock fallback that
can't fire post-epoch); `error.rs` data-only. The Stage D L4 +
Stage B L1+L2a layering is exercised end-to-end by
`crates/runtime-main/tests/tier_smoke.rs` (10 tests: layer
ordering, scope-conditional Network, persistence round-trip,
demotion invalidation).

**runtime-main audit + tier::transition** (M05.E, Windows-local):
`audit/writer.rs` 100% line / 98.43% region; `audit/entry.rs`
99.39% line / 97.50% region (1 uncovered = the
`Value::_ => Map::new()` fallback, structurally unreachable since
`json!()` always produces `Value::Object`); `audit/file_path.rs`
100% / 100%; `audit/error.rs` data-only; `tier/transition.rs`
99.24% line / 99.05% region (1 uncovered = the `tracing::error!`
branch fired only on underlying audit write failure — hard to
trigger from a freshly-opened tempfile). Exercised end-to-end by
`crates/runtime-main/tests/audit_smoke.rs` (7 tests:
capability_granted, capability_denied, capability_check_ok-no-audit,
tier_transition, framework_loaded, gap_detected, end-to-end
multi-seam, plus no-writer silent no-op). `capability/enforcer.rs`
dropped 100% (M05.B+D) → 94.24% with the new `audit_grant` +
`audit_check_result` + `audit_log` helper branches (the
TierForbidden audit branch in `audit_check_result` isn't exercised
by audit_smoke; a future tier_violation-then-audit path would lift
it). Still within the runtime-main 95 gate (96.56% workspace-wide).

**runtime-mcp** (M06.C measured Windows-local, after
`cargo llvm-cov clean`): `client/auth.rs` 100% / 100%;
`client/error.rs` 100% / 100%; `client/mod.rs` 94.64% line /
90.95% region (15 uncovered = `add_server`/`remove_server`
error-path branches the MockTransport surface can't easily reach);
`client/registry.rs` 94.12% line / 86.52% region (7 uncovered =
`insert`/`list`/`update_last_alive` error-path branches);
`transport/mod.rs` 95.35% line; `transport/mock.rs` 99.50% line;
`error.rs` 100% line; in-gate aggregate **96.64% line ≥95**.
Carry-forward: the `client/mod.rs` + `client/registry.rs`
error-path branches could be lifted via an injectable-failure
SecretStore/Registry seam (the `*_with` archetype) if a future
stage needs the extra coverage.

**runtime-main `skills_lock` + `import`** (M07.B/C/E, measured
Windows-local): `skills_lock/mod.rs` 98.15% line (M07.B; 1 missed =
the non-`NotFound` read-error branch in `write_entry`, a within-gate
holdout); `skills_lock/error.rs` is `thiserror`-derive-only (no
instrumentable lines). `import/mod.rs` 94.77% line at M07.C, within
the passing runtime-main aggregate (96.65% at C; 95.56% at E after
`ImportPanel`'s ADR-0015 enrichment exercised additional
`import_artifact_with` lines); `import/fetch.rs` excluded — Category 3,
the real `reqwest` artifact GET (§C M07.C entry). The import pipeline
is exercised end-to-end by
`crates/runtime-main/tests/import_pipeline_integration.rs`.

**runtime-mcp `connection_resolver`** (M07.D1, measured
Windows-local): `client/connection_resolver.rs` — the pure helpers
(`record_to_transport` / `lifecycle_to_mcp`) carry in-source unit
tests; the `connection()` happy-path delegate is the ADR-0011
seam↔concrete OS-call holdout (covered by the mandatory Stage V
`--features integration` reference-MCP-server smoke). runtime-mcp
in-gate aggregate 95.83% line ≥95 at D1 (91.49% → 95.83% after the
additive pure-helper tests). `transport/mod.rs` stays 87.50% line —
the M06.G CQ-1 carry-forward; the aggregate holds ≥95 without
touching it.

**runtime-drone `token_usage`** (M07.D2, measured Windows-local):
`crates/runtime-drone/src/token_usage.rs` — the new
signal→`token_usage` projector, INSIDE the runtime-drone ≥95 gate
(not excluded); runtime-drone in-gate aggregate 95.73% line (regions
94.57%, functions 96.84%). Projects in the same `handle_write_signal`
transaction as `vdr` + `plan_projector`; idempotent via PK = the
contributing signal id.

**runtime-main `builder`** (M08.B/F1, measured Windows-local): the new
`crates/runtime-main/src/builder/` module — INSIDE the existing
runtime-main ≥95 package gate, NOT a separate `cargo llvm-cov
--package` gate and NOT excluded. `validate.rs` 100% line (M08.B);
`persist.rs` 97.67% line (M08.B; 1 missed = the non-UTF-8
directory-entry `continue` in `read_companions`, unreachable on a
normal filesystem); `summary.rs` 96.23% line (M08.B; 2 missed in the
`dedup_sorted` / cold paths); `error.rs` is `thiserror`-derive-only
(no instrumentable lines; the `From<FrameworkLoadError>` conversion is
forward-looking surface). `tester.rs` 93.52% line (M08.F1 — the
isolated-session Tester; residual missed lines are test-scaffold stub
methods the run loop never invokes + the rare `SdkError`-infra error
map; the package gate 96.57% ≥95 holds). The whole module is pure /
seam / `tempfile`-tested — every filesystem call is `&Path`-
parameterised; the OS-touching wrappers (`test_framework`, the drone
spawn, teardown) live in `src-tauri/src/commands.rs`, not in the
`builder` module — so M08 added **no new `--ignore-filename-regex`
exclusion**. The Builder backend is exercised end-to-end by
`crates/runtime-main/tests/builder.rs` + the assembled Tester
regression `crates/runtime-main/tests/tester_isolated_session.rs`.

---

## C. Per-milestone exclusion history & carry-forwards (append-only ledger)

Append a dated entry here whenever an exclusion/threshold changes.
History is immutable (a measurement true for M0X stays true for M0X).

- **M01.C** — established the runtime-drone 95 gate; excluded
  `lib.rs` + `shutdown.rs` (OS-signal orchestrators wrapping
  testable `_inner`/`_with` variants — exercised by the Unix
  subprocess integration test in
  `crates/runtime-drone/tests/integration.rs`). Source: M01.C
  retrospective + `M01-foundation.md` Stage D §D.3 "Coverage gate
  semantics".
- **M02.C** — added the runtime-main 95 gate; excluded
  `providers/anthropic.rs` (real reqwest+SSE wrapper, POSTs to
  `https://api.anthropic.com`, structurally untestable
  cross-platform) + `drone_ipc/connection.rs` (cfg-platform
  `open()` OS-call wrapper). Testable seams
  `Connection::from_streams` / `send_with_reconnect` /
  `next_event` / `next_response` unit-tested via `tokio::io::duplex`
  + the loopback integration test. Added the wiremock coverage path.
  Source: `M02-event-pipeline.md` Stages C/D.
- **M02 PR #45 post-merge** — `codecov.yml` `tauri-shell` patch
  gate (see §D).
- **M05.C1** — added the runtime-sandbox 95 gate; excluded
  `src/main.rs` (binary stub) + `src/lib.rs` (run/run_inner
  OS-signal orchestrator).
- **M05.C2** — lifted `src/ipc.rs` into the runtime-sandbox gate
  (C1 carry-forward); added the OS-isolation modules: `seccomp.rs`
  + `landlock.rs` (`cfg(target_os = "linux")`); `job_objects.rs`
  (`cfg(windows)`, measured on CI Windows / Windows-local,
  seam-decomposed via `create_job` / `apply_limits` /
  `assign_process` / `win32_failure`).
- **M05.D** — added the tier system (`crates/runtime-main/src/tier/`)
  — no exclusions; pure data + path-agnostic persistence is fully
  testable.
- **M05.E** — added the audit log + `tier/transition.rs` — no new
  exclusions (mutex-guarded `tokio::fs::File`, path-agnostic, fully
  `tempfile`-testable). Documented the `capability/enforcer.rs`
  drop (§B).
- **M05 PR #70 post-merge** — excluded `seccomp.rs` + `landlock.rs`
  from the runtime-sandbox gate after their first Linux CI
  measurement (80.56% + 76.25% line) confirmed the `install()`
  bodies are structurally untestable in-process (seccomp's
  `KillProcess` BPF filter; landlock's `restrict_self` thread
  restriction would poison the test runner). OS-signal-class
  holdout, parallel to drone `lib.rs`+`shutdown.rs`. Coverage
  attribution via subprocess integration tests in
  `tests/integration.rs` (behavioral: `/proc/$pid/status` on Linux
  + `IsProcessInJob` on Windows). **Carry-forward to M06+:** seam
  decomposition —
  `seccomp::install_with(load_fn: impl FnOnce(&ScmpFilterContext) -> Result<()>)`
  +
  `landlock::install_with(restrict_fn: impl FnOnce(RulesetCreated) -> Result<RulesetStatus>)`
  — would lift these back into the gate by separating the OS-call
  line from the install logic; matches the `*_with` archetype.
- **M06.B** — added the runtime-mcp 95 gate; excluded `src/main.rs`
  (no binary), `src/lib.rs` (pub-mod + re-exports — parallel to
  runtime-sandbox `lib.rs`), `transport/stdio.rs` + `transport/http.rs`
  (rmcp-wrapper `connect()` happy paths need a real MCP peer —
  subprocess speaking JSON-RPC for stdio, full `initialize`
  handshake for HTTP; the rmcp wire-format is upstream, not ours;
  credit via `build_command`/pure-helper + connect-failure-path
  unit tests + feature-gated `tests/integration.rs` smoke).
- **M06.C** — added `client/auth_keyring.rs` (`KeyringSecretStore`
  wraps `keyring::Entry` — every method hits the real OS keychain:
  Linux Secret Service via D-Bus / macOS Keychain / Windows
  Credential Manager; untestable on CI without a session bus or
  signed-in user; parallel to runtime-main `key_store.rs` + the
  M02.C `anthropic.rs` precedent; credit via the `#[ignore]`-gated
  round-trip test) + `client/lifecycle.rs`
  (`spawn_health_pinger[_with_interval]` returns a `tokio::spawn`'d
  infinite-loop `JoinHandle`; the loop body delegates to the
  in-gate `McpClient::run_health_pass`; the spawn wrapper itself is
  structurally untestable cross-platform — parallel to drone
  `lib.rs`). The runtime-mcp gate REQUIRES `--features test-helpers`
  (M06.C's `client_lifecycle.rs` integration tests are
  `#![cfg(feature = "test-helpers")]`-gated; without it `client/`
  coverage craters). Added the `cargo llvm-cov clean`-before-measure
  rule (gotcha #81 — M06.C wasted ~10 min chasing a false 92.07%
  that was truly 96.64%); now canonical in CLAUDE.md §6 step 4.
- **M07.A** — no exclusion or threshold change. Two reconciles:
  (1) **TD-006** (M06.V finding #4): the M06 phase doc
  (`docs/build-prompts/M06-mcp-basic.md`) V.3/A.4.4 runtime-main
  `--ignore-filename-regex` carried a stray `|src.key_store\.rs`
  token (6 occurrences) the four canonical mirrors never had. The
  four mirrors (§A above line 74, CLAUDE.md §5 category list, CLAUDE.md
  §6 command, `codecov.yml`) were **already byte-consistent and
  canonical** — `key_store.rs` is the OS-keychain holdout that is
  *not* in the regex because no production `key_store.rs` line is
  reached by the gated tests (§A note already explains this). The
  drift was solely the M06 phase doc (a non-mirror surface);
  M07.A dropped all 6 stray tokens so the phase doc matches the
  canonical form. **No four-mirror value change** — the v1.8
  four-mirror sync rule is satisfied vacuously (the mirrors were
  never inconsistent with each other; only a downstream phase-doc
  copy drifted). (2) **TD-005** (M06.V finding #3): the runtime-main
  `cargo llvm-cov` gate is now Windows-local-measurable for the
  first time — the six integration test files that spawn the
  `runtime-drone` subprocess were de-duplicated onto a shared
  `crates/runtime-main/tests/common/mod.rs` fixture that builds the
  drone into a dedicated `target/drone-fixture` dir (no parent
  build-lock contention) with the workspace manifest + package
  pinned (CWD-independent) and the llvm-cov instrumentation env
  stripped. The gate command, regex, and threshold are **unchanged**;
  only the test harness was made robust. Measured Windows-local at
  M07.A: runtime-main 95.73% line ≥ 95 (exit 0) — previously the
  gate aborted before any measurement (the gotcha #56 nested-build
  break).
- **M07.C** — added `|src.import.fetch\.rs` to the **runtime-main 95**
  gate regex. Category 3 (seam-vs-wrapper / OS-call holdout):
  `crates/runtime-main/src/import/fetch.rs` is the real `reqwest`
  `HttpFetcher` — the ONLY outbound HTTP for an artifact import (Hard
  Rule 4: it GETs exactly the user-supplied URL, no phone-home). The
  capability gate + the whole pipeline are unit-tested through the
  injected `Fetcher` / `NetworkGate` / `Sandbox` / `McpRegistry`
  seams (`fetch_with` + `import_artifact_with`); `HttpFetcher` itself
  is exercised behaviourally against a local `wiremock` server (no
  live network in the gate) — the `--features integration` live
  smoke is the optional real-endpoint check, exactly the
  `providers/anthropic.rs` precedent. **Four-mirror sync done in the
  M07.C commit**: §A category-3 list + §A table row (above) +
  CLAUDE.md §6 runtime-main command updated byte-consistently.
  `codecov.yml` requires **no change** — exactly as for the
  `providers/anthropic.rs` / `key_store.rs` / `*_connection.rs`
  runtime-main holdouts: the per-file runtime-main exclusions are
  enforced by the §6 absolute-floor `cargo llvm-cov --fail-under-lines
  95` command, not by `codecov.yml`'s global `ignore:` (whose only
  entries are generated / `main.rs` / `build.rs` / the sandbox
  OS-signal files). Codecov's `project.runtime-main` flag gate (target
  95%, threshold 0.5%) legitimately still counts `import/fetch.rs`
  via the `wiremock` behavioural coverage — the same delta-vs-floor
  asymmetry the §A `anthropic_sse.rs` note documents. CLAUDE.md §5
  needs no edit (it names the four exclusion *categories*
  generically, not files; `import/fetch.rs` is category 3, already
  described). No threshold moved; no new §B baseline (the excluded
  file has no gated lines by construction).
- **M07.G (closeout `<coverage_policy_reconciliation>`)** — no
  threshold or `--ignore-filename-regex` value changed at M07.B / D1 /
  D2. The only enforced-mirror change this milestone was the
  `src.import.fetch.rs` runtime-main exclusion, added and
  four-mirror-synced in the M07.C commit (entry above). The M07.G
  reconciliation appends the §B per-module baselines above and records,
  per stage:
  - **M07.B** — the `skills_lock` module
    (`crates/runtime-main/src/skills_lock/`) is a new safety primitive
    (artifact integrity, CLAUDE.md §5) at ≥95%. It is **not** a new
    `cargo llvm-cov --package` gate — it is a module *inside* the
    existing runtime-main ≥95 package gate, so no §6 command, no §5
    category, and no `codecov.yml` change. Baseline `skills_lock/mod.rs`
    98.15% line; `skills_lock/error.rs` derive-only. §B appended.
  - **M07.D1** — `transport/mod.rs` stays at the M06.G CQ-1 baseline
    87.50% line (pre-existing carry-forward). The runtime-mcp ≥95 gate
    holds at the aggregate (95.83% at D1) **without** touching it —
    `connection_resolver.rs`'s pure helpers got in-source unit tests;
    the `connection()` happy-delegate is the ADR-0011 seam↔concrete
    OS-call holdout covered by the mandatory Stage V `--features
    integration` smoke. Phase-doc D1 option (b): `transport/mod.rs`
    87.50% is the same OS-call-holdout class regardless of which
    transport file `rmcp_tool_to_mcp_tool` lives in. No exclusion
    added; no four-mirror change. §B appended.
  - **M07.D2** — `crates/runtime-drone/src/token_usage.rs` (the new
    `token_usage` projector) is INSIDE the runtime-drone ≥95 gate, NOT
    excluded; runtime-drone aggregate 95.73% line (measured
    Windows-local at D2). No exclusion change. §B appended.
  The four canonical mirrors — CLAUDE.md §5 exclusion-category list,
  CLAUDE.md §6 `cargo llvm-cov` commands, `codecov.yml`, and §A above —
  are verified byte-consistent as of M07.G. **One CI-surface drift was
  found and fixed in this M07.G commit:** `.github/workflows/ci.yml`
  (the runtime-main `coverage` step ~line 391 + the runtime-main `lcov`
  step ~line 470) carried the canonical runtime-main regex *minus*
  `src.import.fetch\.rs` (the M07.C exclusion synced to §6 + §A in the
  M07.C commit never reached `ci.yml` — flagged by the M07.C retro's
  CI-workflow drift check as a Stage G item) and *plus* a stray
  `|src.key_store\.rs` token (the same stray-token class as TD-006 —
  TD-006's M07.A resolution corrected the M06 phase doc but did not
  inspect `ci.yml`, and the M07.A CI-drift check asserted consistency
  without catching it). Both `ci.yml` occurrences were corrected to the
  canonical CLAUDE.md §6 form
  (`…|src.sandbox_ipc.connection\.rs|src.import.fetch\.rs`) in this
  M07.G commit. The correction is a no-op for the measured number
  (`key_store.rs` has no gated lines per the §A note; `import/fetch.rs`
  is the Category-3 holdout local runs already exclude) — it is a
  CI-parity correction so CI measures the same runtime-main surface as
  the local §6 gate. After this fix all five surfaces — §A, CLAUDE.md
  §5, CLAUDE.md §6, `codecov.yml`, `ci.yml` — are byte-consistent.
- **M08 (closeout `<coverage_policy_reconciliation>` — Stage H)** — **no
  threshold and no `--ignore-filename-regex` value changed anywhere in
  M08.** M08 added two modules to the existing **runtime-main ≥95**
  package gate but **no new exclusion**, per stage:
  - **M08.A** — `crates/runtime-main/src/plan/plan_loop.rs` (the M04
    `plan_loop` driver shell) is new, pure-logic, `tempfile`-free, 100%
    line — INSIDE the runtime-main ≥95 gate, not excluded. `key_store.rs`
    gained one thin-wrapper line (`has_api_key`) untested by design —
    the same OS-keychain holdout as `read_api_key`/`write_api_key`/
    `delete_api_key` (the §A note already covers `key_store.rs`); the
    package gate absorbs it (95.53% ≥95). The `runtime-mcp` `stdio.rs`
    `resolve_program` addition is inside the already-excluded
    `transport/stdio.rs` (Category-3 OS-call holdout). No exclusion or
    threshold change; no four-mirror change.
  - **M08.B** — the new `crates/runtime-main/src/builder/` module
    (`validate.rs`/`persist.rs`/`summary.rs`/`error.rs`) entered the
    runtime-main ≥95 package gate. It is **not** a new `cargo llvm-cov
    --package` gate — it is a module *inside* the existing runtime-main
    gate, so no §6 command, no §5 category, and no `codecov.yml` change.
    Pure / seam / `&Path`+`tempfile`-tested — **no new exclusion** (the
    M07.B `skills_lock` precedent). runtime-main aggregate 96.72%. §B
    baseline appended.
  - **M08.C / D1 / D2 / E / F2 / G** — renderer-only stages; no
    `crates/**` or gated `src-tauri/src/` code touched (the
    `crates/xtask/src/main.rs` + `src-tauri/src/main.rs` C edits are in
    the coverage-excluded `src.main\.rs` pattern). No Rust coverage gate
    moved; the Vitest renderer gate (≥80% on `src/`) held every stage
    (97.07–97.38% line).
  - **M08.F1** — `crates/runtime-main/src/builder/tester.rs` (the
    isolated-session Tester) entered the runtime-main ≥95 package gate.
    Pure / seam / `tempfile`-tested; the OS-touching wrapper
    (`test_framework`, drone spawn, teardown) lives in
    `src-tauri/src/commands.rs` (the §D Tauri patch-gate surface), NOT
    in `tester.rs` — so **no new exclusion**, no four-mirror change.
    `tester.rs` itself 93.52% line; runtime-main aggregate 96.57% ≥95.
    §B baseline appended.
  The M08.H reconciliation appends the §B `runtime-main builder`
  per-module baseline (above) and this §C entry. The four canonical
  mirrors — CLAUDE.md §5 exclusion-category list, CLAUDE.md §6
  `cargo llvm-cov` commands, `codecov.yml`, and §A above — were
  **unchanged by M08** and are verified byte-consistent as of M08.H.
  No drift found.
- **M08.6 (closeout `<coverage_policy_reconciliation>` — Stage F)** —
  **no threshold and no `--ignore-filename-regex` value changed
  anywhere in M08.6.** M08.6 grew the existing
  `crates/runtime-main/src/builder/` module (in the **runtime-main ≥95**
  package gate since M08) with the ADR-0022 reference-resolution
  rewrite + the save-side re-split, per stage:
  - **M08.6.A** — intake stage. No production code beyond the
    `docs/M08-irl-test-plan.md` typo-fix (a markdown test-plan
    document outside any test or build path); no Rust diff, no
    coverage delta. No exclusion or threshold change; no four-mirror
    change.
  - **M08.6.B** — `crates/runtime-main/src/builder/persist.rs` grew
    by ~150 lines (the directory-walk resolver + `split_frontmatter`
    + the `serde_yaml` parse + the asymmetric agents/tools/skills
    handling) and `crates/runtime-main/src/builder/error.rs` gained
    one variant (`BuilderError::ReferenceResolution { reference,
    cause }`). The module remains pure / seam / `&Path` + `tempfile`-
    tested with no OS call — the **same M07.B `skills_lock` + M08.B
    `builder` precedent**: a new module INSIDE the runtime-main ≥95
    gate, NOT a separate `cargo llvm-cov --package` gate, NOT
    excluded. `crates/runtime-main/Cargo.toml` gained `serde_yaml =
    { workspace = true }` (an existing workspace dep already consumed
    by `runtime-core`; not a new external dependency — no `cargo
    deny` concern). Per-file `persist.rs` 88.61% line at B's
    measurement (the M08 baseline 97.67% reflects the smaller M08
    surface; the M08.6 additions introduced 4–5 error-path branches
    the strict v1.8 invariant locked out of the impl commit:
    broken-tool-ref, broken-skill-ref, no-frontmatter-found,
    YAML-parse-error). Per-package `runtime-main` 95.56% line ≥ 95
    PASSES at B (the per-package gate, the enforced mirror, holds
    above 95). No exclusion or threshold change; no four-mirror
    change.
  - **M08.6.C** — `save_framework` re-split + `synthesize_agent_md` +
    `write_artifact_md` + `is_outside_framework_dir` + the
    canonicalization-through-`serde_json::Value` for byte-stability
    against `Framework`'s three `HashMap<String, _>` fields
    (`hook_defs`, `mcp_aliases`, `per_mode_overrides`). Per-file
    `persist.rs` dropped to 84.73% line / 86.11% regions at C's
    measurement — ~120 new lines added several error-path branches
    not enumerated in the C.4 test set (the synthesize-body path
    with NO matching companion; the `is_outside_framework_dir` true
    branch; the Object-variant-with-matching-companion path; the
    Object-variant-with-no-companion silently-skipped path). Per-package
    `runtime-main` 95.40% line ≥ 95 PASSES (the aggregate gate
    holds; per-file under-coverage is recorded as M08.6.V 🟡 #2).
    No exclusion or threshold change; no four-mirror change.
  - **M08.6.D + .E** — renderer-only stages; no `crates/**` or gated
    `src-tauri/src/` code touched. The Vitest renderer gate (≥80% on
    `src/`) held every stage (D: 97.13% line; E: 97.15% line). No
    Rust coverage gate moved.
  - **M08.6.V** — verifier stage; no code change. V confirmed the
    runtime-main aggregate 95.40% ≥ 95 PASSES with `cargo llvm-cov
    clean --workspace` first (gotcha #81) — the per-package gate is
    the enforced mirror — and surfaced the per-file 84.73%
    `persist.rs` divergence as 🟡 #2 (carry-forward to M09.A).
  The M08.6.F reconciliation appends this §C entry and does **not**
  append a new §B baseline (the M08-era `runtime-main builder` §B
  row above stands; the M08.6 per-file numbers are documented in this
  §C entry as deliberate strict-TDD ceilings carrying forward, not
  as new baselines that would lock in regression). The four canonical
  mirrors — CLAUDE.md §5 exclusion-category list, CLAUDE.md §6
  `cargo llvm-cov` commands, `codecov.yml`, and §A above — were
  **unchanged by M08.6** and are verified byte-consistent as of
  M08.6.F. No drift found.

  **Policy note for the per-file divergence (M08.6.V 🟡 #2 surface).**
  The strict v1.8 two-commit TDD invariant (the test file is
  byte-identical between red commit and impl commit) creates a
  structural per-file coverage ceiling: the phase-doc-enumerated
  test set (B.4 = 7 tests; C.4 = 4 tests) pins what's testable in
  the impl commit; additional error-path tests would land as a
  separate labelled follow-up (`test(M08.6.X): error-path coverage`)
  per phase doc B.6 / C.6 ("net-new additive tests go in a separate
  labelled follow-up"). The **enforced gate (`--fail-under-lines 95`
  on the package, the §6 absolute floor) PASSES**; the per-file
  divergence is the M08.6.V finding routed to M09.A for resolution.
  Two resolution paths recorded in the M08.6 gap-analysis Carry-forward:
  (a) targeted error-path tests for the `BuilderError::ReferenceResolution`
  cause-string branches + the `is_outside_framework_dir` save-side
  branch + the Object-variant write branch (`persist.rs:141-153`) +
  the cross-framework save preserve path (~30–45 min; +10pp on
  persist.rs per-file); (b) explicit policy entry recording that
  aggregate-passes is the gate intent + per-file divergence is
  acceptable for this strict-TDD surface. Recommend (a) at M09.A;
  this entry pre-records (b)-style language in case (a) is declined.
  The aggregate-vs-per-file asymmetry is the same shape as the
  existing M07.G `transport/mod.rs` carry-forward (87.50% line below
  per-file aspiration; runtime-mcp aggregate holds ≥95) — pattern is
  durable, not novel.
- **M08.7 (closeout — Stage G)** — **no §C entry was appended at M08.7**:
  rungs 1–5 added four new modules to the existing **runtime-main ≥95**
  package gate (`builtin_tools.rs`/`load_skill.rs`/`request_capability.rs`
  + the `agent_sdk.rs` dispatch branches) all INSIDE the gate, NOT new
  `--package` gates and NOT excluded; no threshold or
  `--ignore-filename-regex` value moved (`runtime-main` 96.23% line ≥ 95;
  workspace 92.12% ≥ 80). The four canonical mirrors were byte-consistent
  as-is. Recorded here for the audit trail (the M08.7 sign-off stated "no
  §C entry required").
- **M08.8 (closeout `<coverage_policy_reconciliation>` — Stage G)** —
  **no threshold and no `--ignore-filename-regex` value changed anywhere
  in M08.8.** M08.8 is overwhelmingly renderer/CSS (Stages A, B, B.fix,
  B.fix2-renderer-half, C.fix) — the Vitest renderer gate (≥80% on `src/`)
  held every stage (B 93.58% → C.fix 93.47% line). The only `crates/**` /
  gated `src-tauri/src/` touches:
  - **M08.8.C (tier in the run loop)** — the wire threaded the tracked
    tier through `src-tauri/src/commands.rs` (`test_framework` reads
    `CurrentTierState`; `test_framework_with` delegates to
    `run_test_session_with_tier`) + reused the existing `tester.rs`
    `run_test_session_with_tier` seam (no `tester.rs` change). The
    `test_framework` tier read is an OS-touching command wrapper
    (needs an `AppHandle` — the **§D tauri-shell patch-gate** surface,
    target 50%): `commands.rs` measured **67.62% line ≥ 50** at C. The
    `test_framework_with` tier-threading IS covered by the 3 assembled
    tests (`commands.rs::test_framework_with_at_promoted_…` /
    `…_at_novice_…`). `runtime-main` aggregate **96.27% line ≥ 95**
    (unchanged — no `runtime-main` source moved; the seam pre-existed).
    **No new exclusion, no new `--package` gate, no `codecov.yml`
    change** — exactly the M08.F1 `tester.rs` precedent (non-wrapper
    logic lives in the gated seam; the OS wrapper is the §D patch-gate
    surface). The §D note's "work adding **non-wrapper** logic to
    `src-tauri/src/` must re-evaluate the 50% target" was considered:
    the new `test_framework` lines are a one-call tier read + a delegate
    (wrapper-class), not new non-wrapper logic, so the 50% target stays
    appropriate (67.62% clears it). No `src-tauri/**` exclusion exists to
    move.
  - **M08.8.B.fix2 (reload-reconstruct)** — touched
    `crates/runtime-main/src/sdk/replay.rs` (serde round-trip rewrite,
    INSIDE the runtime-main ≥95 gate, 97.27% per-file) + a new
    `src-tauri/src/commands.rs` `replay_latest_session` command (the §D
    patch-gate wrapper). `runtime-main` aggregate 96.27% ≥ 95; workspace
    91.92% ≥ 80. **No exclusion or threshold change.**
  The M08.8.G reconciliation appends this §C entry and **no new §B
  baseline** (no module entered a gate; the M08-era `runtime-main
  builder` row stands). The four canonical mirrors — CLAUDE.md §5
  exclusion-category list, CLAUDE.md §6 `cargo llvm-cov` commands,
  `codecov.yml`, and §A above — were **unchanged by M08.8** and are
  verified byte-consistent as of M08.8.G. No drift found.
- **M08.9 (closeout `<coverage_policy_reconciliation>` — Stage G)** —
  **no threshold and no `--ignore-filename-regex` value changed anywhere
  in M08.9** (the honest-Tester milestone: A truthful verdict, B run
  drill-down, V verifier, D.fix e2e teardown). The only gated `crates/**`
  touch:
  - **M08.9.A (truthful verdict)** — grew
    `crates/runtime-main/src/builder/tester.rs` (the new `TierBlock`
    struct + `TestVerdict` enum + the `fold_outcome` `TierViolation`
    match arm + the derived-verdict computation + the two new
    `TestOutcome` fields) **INSIDE the existing runtime-main ≥95 package
    gate** — NOT a new `--package` gate, NOT excluded. Every new line is
    exercised by the producer-driven fold units (a serialized real
    `AgentEvent::TierViolation` → `fold_outcome` → `tier_blocks` /
    `verdict=TierLimited` / `passed` still true; the both-tier-and-
    capability `Fail`-with-tier_blocks case). `runtime-main` aggregate
    **95.03% line ≥ 95** (measured at A, exit 0). No exclusion, no
    threshold, no `--package` gate moved.
  - **M08.9.B (run drill-down)** — renderer-only
    (`src/components/builder/{TesterModal,TraceDrilldown}.tsx` +
    `src/lib/formatPayload.ts` + `src/components/RawDisclosure.tsx` + the
    `InspectorPanel`/`ValidationCard` delegations); the Vitest renderer
    gate (≥80% on `src/`) held — global lines **93.54%**, exit 0;
    `TraceDrilldown.tsx` 97.67%. No `crates/**` coverage change.
  - **M08.9.D.fix (e2e teardown)** — test-harness-only
    (`tests/e2e-tauri/*.e2e.ts`, +26/-0); no source file changed, no
    `cargo llvm-cov` / `vitest --coverage` gate moved.
  The M08.9.G reconciliation appends this §C entry and **no new §B
  baseline** (no module entered a gate; the M08-era `runtime-main
  builder` row stands — `tester.rs` was already inside it from M08.8).
  The four canonical mirrors — CLAUDE.md §5 exclusion-category list,
  CLAUDE.md §6 `cargo llvm-cov` commands, `codecov.yml`, and §A above —
  were **unchanged by M08.9** and are verified byte-consistent as of
  M08.9.G. No drift found.
- **M09 (closeout `<coverage_policy_reconciliation>` — Stage G)** — **no
  threshold and no `--ignore-filename-regex` value changed anywhere in
  M09** (the first ADR-0032 vertical slice: A blank-create, B file_access
  editor, C attach an MCP tool, D the assembled IRL + two D.fix
  iterations, V the verifier). The gated `crates/**` touches all landed
  **inside existing package gates** — none excluded, none a new
  `--package` gate:
  - **M09.C (attach an MCP tool)** — added `mcp_list_server_tools` to
    `src-tauri/src/commands.rs` (the §D `tauri-shell` 50% patch-gate
    wrapper) over a new `list_server_tools(name)` seam in
    `crates/runtime-mcp/src/client/connection_resolver.rs` **INSIDE the
    existing runtime-mcp ≥95 gate**. The command is a **thin read-only
    OS/network wrapper paired with the unit-tested seam** — the
    seam-vs-wrapper split (a §5 named exclusion category) holds, so **no
    new exclusion** is needed (the seam carries the coverage; the
    `src-tauri` command rides the existing `tauri-shell` patch gate).
    `runtime-mcp` 96.00% line ≥ 95 (`connection_resolver.rs` 95.74%).
  - **M09.D + D.fix (the run-path injection + the dispatcher enforcer)** —
    grew `crates/runtime-main/src/builder/tester.rs` + `…/builder/mod.rs`
    (the `build_session_mcp_tool_defs` + `run_test_session_with_tools`
    model-facing injection) and added `build_session_mcp_enforcer` +
    both-dispatcher wiring in `src-tauri/src/commands.rs`, **INSIDE the
    existing runtime-main ≥95 gate** (the `src-tauri` enforcer/dispatcher
    construction rides the `tauri-shell` patch gate). Every new line is
    exercised by `mcp_tool_injection_execution.rs` (the tool reaches the
    model's list; dispatch→`Write` lands the file; authored-only) + the
    `build_session_mcp_enforcer` unit (Promoted⇒allowed / Novice⇒denied /
    unauthored⇒denied) + the real-`McpDispatcher` regression.
    `runtime-main` aggregate **96.32% line ≥ 95** (measured at D.fix
    iter2, exit 0).
  - **M09.A / M09.B + the D.fix UI v2** — renderer-only (`builderStore.ts`,
    `Palette.tsx`, `NodeConfigPanel.tsx`, `TesterModal.tsx`,
    `SettingsPanel.tsx` + CSS); the Vitest renderer gate (≥80% on `src/`)
    held — global lines **93.4%**, exit 0. No `crates/**` coverage change.
  The M09.G reconciliation appends this §C entry and **no new §B
  baseline** (no module entered a gate; the M08-era `runtime-main builder`
  + `runtime-mcp client` rows stand — `tester.rs`/`mod.rs` and
  `connection_resolver.rs` were already inside their package gates). The
  four canonical mirrors — CLAUDE.md §5 exclusion-category list, CLAUDE.md
  §6 `cargo llvm-cov` commands, `codecov.yml`, and §A above — were
  **unchanged by M09** and are verified byte-consistent as of M09.G. No
  drift found.

---

## D. Gate mechanisms

### Codecov delta gating (from M02 onward — enforced)

M01 used absolute thresholds (workspace ≥80, drone ≥95) because no
baseline existed. From M02 every PR also passes a delta-gate via
Codecov: project + patch thresholds in `codecov.yml`
(`target: auto`, `threshold: 0.5%`, `base: auto`). Codecov pulls the
LCOV uploaded by the existing `cargo-llvm-cov` step in
`.github/workflows/ci.yml`, compares to `main`'s last green build,
and fails the PR if any gated crate regresses >0.5pp (absolute) OR
patch coverage on changed lines drops below the project floor.

Codecov was advisory in M01 (commit `c04aac5`); M02 Stage A flipped
required-on via: a `codecov.yml` at repo root (project + patch
rules); `.github/workflows/ci.yml` keeps the upload step + per-crate
flag uploads (`workspace`, `runtime-drone`, `runtime-main`);
`informational: false` makes the check blocking and
`fail_ci_if_error: true` reds the build on upload failure. The
absolute-threshold gates (`cargo llvm-cov --fail-under-lines`) remain
authoritative for hard floors; Codecov gates the *delta*. No custom
bash scripts. (Pre-M01 carry-forward; resolved per the M01
gap-analysis Important "Coverage delta gating mechanism" item.)

### Tauri-shell patch-gate exception (PR #45 / M02 post-merge)

The `src-tauri/src/commands.rs` wrapper functions (`set_api_key`,
`run_smoke_session`, `forward_events`) call real OS APIs (keychain
via `keyring`, Tauri `AppHandle::emit`, `AnthropicProvider::new`
against the real network) and are structurally untestable on CI
Linux without a platform keychain or a Tauri runtime. The `*_with`
testable seams in the same file (`set_api_key_with`,
`run_smoke_session_with`) ARE unit-tested. To honor the
seam-vs-wrapper split honestly, `codecov.yml` defines a
`tauri-shell` patch gate at 50% (target) covering `src-tauri/**`,
and the default 80% patch gate excludes that path. Same
architectural rationale as the runtime-main OS-call-holdout
exclusions. Work that adds **non-wrapper** logic to `src-tauri/src/`
must re-evaluate whether the 50% target stays appropriate and
record any change as a milestone entry in §C. (M06.5 Stage A.fix
added `src-tauri/src/session_db.rs` with in-source `#[cfg(test)]`
tests at 100% — the seam stays testable; the 50% wrapper target was
not loosened.)
