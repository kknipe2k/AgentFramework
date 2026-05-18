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
   `drone_ipc/connection.rs`, `sandbox_ipc/connection.rs`; mcp
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
| runtime-main | `src.main\.rs\|generated\|src.providers.anthropic\.rs\|src.drone_ipc.connection\.rs\|src.sandbox_ipc.connection\.rs` | 95 |
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
