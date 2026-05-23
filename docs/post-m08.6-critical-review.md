# Post-M08.6 + post-IRL critical review

Triggered by an external architectural audit (2026-05-23) flagging the IPC /
canvas-projection glue as the existential integration-risk surface. Recorded
here so the M08.6 closeout and the next HITL/IRL pass actively rule the
concerns in or out — surviving items fold into M08.6's gap-analysis entry at
closeout (CLAUDE.md §20).

## Audit verdicts (orchestration assessment)

### Right

- **IPC / state-sync is the real risk surface.** M08 IRL produced three
  UI-glue bugs (drag-drop, MCP modal, Tester root-agent naming) — all
  canvas-projection failures, none caught by the M08 Stage V verifier
  because the existing suites tested seams, not the assembled running app.
  M08.5 closed the three 🔴 with assembled-regression tests (ADR-0021
  real-app gate); M08.6's canonical-representation work (ADR-0022) is the
  structural response to the loader/canvas projection contract.
- **MCP timeout / heartbeat hardening is a valid carry-forward.** M06 ships
  the client; the post-M08.6 critical review must confirm every dispatch
  path has an explicit timeout + cancellation, with a hanging-server
  fixture proving it.

### Wrong

- **`.aria/tests/*.sh` are archive material**, not active CI (CLAUDE.md §10
  don't-touch). The runtime's tests are `cargo test` + Vitest + Playwright
  + tauri-driver. The audit didn't distinguish the reference framework
  from the runtime under build.
- **"Cryptographic ACK on every UI state transition" is overkill.** M02's
  event pipeline already has monotonic sequence numbers + snapshot /
  recovery; that IS the reconciliation primitive. Bolting Merkle trees on
  top would duplicate the existing ordering guarantee.
- **"Schema drift would collapse the repo" is overstated.** CI's typify
  regeneration gate + `xtask check-drift` enforce schema = Rust = TS
  alignment at every PR (CLAUDE.md §14). Risk exists but is gated, not
  unguarded.

### Partly

- **seccomp / landlock brittleness** — true principle, but v0.1 is
  Windows-only per spec §0d; Job Objects are the actual v0.1 sandbox
  boundary. Linux seccomp UX hardening is M11+. The gap-detection →
  GapNode design path is sound; M08 didn't IRL-test it because no
  generated artifact ran in M08 scope.
- **xtask drift checker is load-bearing** — overdramatized as "the only
  thing keeping the repo from collapsing," but worth a dedicated Stage V
  sweep post-M08.6 to confirm the gate fires on every covered shape (a
  drift-fuzz IRL test, below).

## IRL test items to add after M08.6 ships

These four IRL tests address the audit's structurally valid concerns. Each
is a real-app behavior assertion, not a seam test (ADR-0011 / ADR-0021).

1. **Event-pipeline reconciliation under drone failure.**
   Kill the drone subprocess mid-run; verify the canvas rolls back to the
   last snapshot with no phantom in-flight nodes and no desynced ordering.
   Pairs with M02's snapshot / recovery contract and ADR-0020's
   canvas-projection deterministic-replay invariant.

2. **MCP timeout / heartbeat audit (hanging-server fixture).**
   Every `runtime-mcp` dispatch path has an explicit timeout + cancellation
   path, proven by a fixture MCP server that accepts the connection then
   never responds. Pairs with `UncertaintyPrompt.tsx` /
   `RecoveryDialog.tsx` rendering on the detach event.

3. **Sandbox-error → GapNode end-to-end on Windows.**
   v0.1's actual sandbox boundary is Job Objects (not seccomp). Trigger
   a syscall the policy denies in a generated artifact; verify the error
   surfaces as a GapNode with a human-readable cause, not opaque "process
   died." The first time the spec §4 gap path is IRL-exercised end-to-end.

4. **Schema drift fuzz.**
   Mutate one field in a typify-generated struct; confirm CI's drift check
   fails before any test runs. Proves the gate is live, not silently broken
   (the "schema-as-source-of-truth" contract from CLAUDE.md §14).

## Why this lives here, not in `gap-analysis.md`

`docs/gap-analysis.md` is append-only per parent milestone (CLAUDE.md §20)
and entries land at milestone closeout, not mid-cycle. M08.6 hasn't shipped;
this is a forward-looking review checklist for that milestone's closeout
and the IRL pass that follows. At M08.6 closeout, surviving items here fold
into the M08.6 gap-analysis entry — Section 4 (fix backlog) for items that
became defects, Section 5 (carry-forward) for items deferred to a later
milestone — and this file is referenced from there for the audit context.
