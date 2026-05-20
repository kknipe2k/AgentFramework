# M07 IRL Findings — gate-7 walk-through (post-M07.5)

> **What this is.** Results of an interactive, agent-guided IRL
> walk-through of the running app after the **M07.5 tier-gate fix-cycle**
> merge (PR #90). Companion to no separate test-plan doc — the
> walk-through was conducted live (orchestrator-guided, one step at a
> time) rather than from a runbook. Severity: 🔴 fix before M08 ·
> 🟡 M08 Stage A absorbs · 🟢 `docs/tech-debt.md` / doc-fix. Severity is
> non-elastic.
>
> **Environment.** Windows; dev build (`npm run tauri dev`); app built
> from `main` after the M07.5 merge; Anthropic key entered into the app;
> import test artifact a minimal agent JSON hosted as a public GitHub
> gist. No mocks — real app, real backend, real renderer.

---

## Verdict

**0 🔴 confirmed-blocking · 1 🔴-candidate · 4 🟡 · 3 🟢.**

**The M07 + M07.5 deliverables pass IRL.** The two manual repros that
M07.5 Stage D.fix deferred (gotcha #23 — a Tauri 2.x window cannot be
agent-driven) are now confirmed end-to-end in the assembled app:

- **M07.5 🔴 #1 — tier-gate Reject/Install.** A Novice **Reject** leaves
  no `skills.lock` entry; a Novice **Install** writes it. Confirmed.
- **M07.5 CQ-M07-1 — SSRF egress.** An `http://` URL is blocked on
  scheme; a `https://127.0.0.1/…` URL is blocked on private-address
  classification. Confirmed.

The walk additionally surfaced **8 findings** — UI polish + cross-
milestone gaps, **none a failure of the M07/M07.5 code**. One is a
🔴-candidate (no tier-promotion UI) needing an M08-intake disposition.

---

## Scenario results

| Scenario | Result |
|---|---|
| 1 — Cold start & key entry | PASS (key entry works; persistence ✗ → #7) |
| 2 — Smoke run & live graph | PASS (agent node, inspector, completes; M06.5 🔴-2 re-confirmed — see Confirmed below) |
| 3 — Import panel happy path | PASS (review modal renders capability disclosure + L3 report + §15d secrets + provenance) |
| 4 — Tier-gate Reject & Install | **PASS ✓✓ — M07.5 🔴 #1 confirmed IRL** |
| 5 — SSRF egress | **PASS ✓✓ — M07.5 CQ-M07-1 confirmed IRL** |
| 6 — MCP servers | PARTIAL (panel + Add modal render; Add correctly tier-blocked for Novice; actual add unreachable → #5) |
| 7 — Restart integrity | PARTIAL (session DB + `skills.lock` durable ✓; key persistence ✗ #7; panel reload ✗ #6) |
| Graph canvas | PASS (pan / zoom / fit / window-resize ✓; minimap → #8) |

---

## 🔴-candidate — M08 Stage A intake must disposition

### #5 — No tier-promotion UI; Promoted tier + MCP management unreachable

- **Observed (Scenario 6).** The Add-MCP-Server modal correctly
  hard-blocks Novice ("MCP tools run with the Exec capability — install
  requires Promoted tier; Novice forbids Exec at §8.security L4" + Add
  disabled). But there is **no UI anywhere to promote Novice →
  Promoted** — no settings surface, no tier control; the only tier
  indicator (the node `N` badge) is not interactive.
- **Ground truth.** The backend `request_tier_transition` command
  exists and is unit-tested (`src-tauri/src/commands.rs`). The
  renderer's own type comments + CSS reference a **"Settings panel"**
  that is supposed to host promotion ("Settings panel modal calls
  `request_tier_transition`") — but no Settings panel component ships,
  and there is no `ipc.ts` wrapper for the command. The wire references
  a consumer that was never built.
- **Consequence.** A v0.1 user is permanently Novice → MCP server
  management (all of M06) is unreachable, and the **Promoted tier —
  which §0d places in v0.1 scope** — cannot be reached at all.
- **Disposition.** 🔴-candidate. **M08 Stage A intake must decide:**
  pre-M08 fix vs M08.A-absorbs. First check the M05 / M06 gap-analysis
  + M05.V — the missing Settings panel may already be a known
  carry-forward; if it is, this is a documented-gap re-confirmation; if
  not, it is a live v0.1-scope gap.

---

## 🟡 — M08 Stage A `<read_prior_milestones>` carry-forward

### #2 — Token in/out breakdown not populated

The agent-node inspector after a smoke run shows `tokensIn: 0`,
`tokensOut: 0`, `tokensTotal: 34`. The total is wired (the M07.D2
`token_usage` projector works — see Confirmed) but the in/out split is
not. Internally inconsistent (a total with no components). M08.A —
populate the in/out breakdown.

### #3 — Import UI text near-invisible (contrast)

The Import panel header, the tier-gate review-modal header, and the
installed-artifact row name all render with text colour ≈ the dark
background — the user had to poke around to find the panel. Systemic
across the M07.E import components (the MCP Add modal, by contrast,
renders fine — so it is localized to the import UI, one CSS root
cause). M08.A — correct the import components' text colour to the theme
variable.

### #6 — Import panel does not reload installed artifacts after restart

After an app restart the Import panel is empty although `skills.lock`
still contains `demo-agent@1.0.0` (the backend install is durable). The
panel does not read `skills.lock` on startup. Same root family as
**M07.V 🟡 #2** (skills.lock has no production reader in v0.1) —
converges with that M08.A carry-forward.

### #7 — API key does not persist across an app restart

A key entered in the session works ("✓ stored in OS keychain" shown,
smoke run succeeded) — but after an app restart the app comes up as a
fresh first-start (empty key field, smoke button disabled); the key
must be re-entered every launch. The session DB and `skills.lock`
persisted across the same restart — only the key did not. Root cause
(keychain write-fail vs startup-read-fail) not pinpointed in this pass;
a Windows Credential Manager check would localize it. M08.A.

---

## 🟢 — docs/tech-debt.md

### #1 — Smoke run too fast to observe streaming

The smoke prompt ("say only the word: hello") is a ~1-second round
trip, so the token streaming into the live graph is not perceptible —
the node goes spawned → complete with nothing visibly streaming. Not a
defect (minimal prompt by design); verified by the end state. A longer
demo prompt or a replay-at-speed control would make the live-graph
streaming observable.

### #4 — No bundled importable example artifact

The import URL field carries only a placeholder; the repo's own skills
are `.md` files (the import pipeline validates JSON); no example
importable artifact ships. A user cannot exercise the import feature
out-of-the-box without sourcing/hosting a JSON artifact externally
(this pass used a hand-made gist). Ship an example artifact, or
pre-fill a working example URL.

### #8 — Graph minimap renders blank

The React Flow minimap is a blank white square — it works as a
click-to-navigate control but shows no miniature node representation,
and it is unthemed (white, against the dark canvas). Theme the minimap
+ node colour so node representations render.

---

## Confirmed — so M08 does not re-investigate

- **M07.5 🔴 #1 (tier-gate Reject/Install)** — IRL-confirmed end-to-end.
  Novice Reject → no `skills.lock` entry, no MCP registry row; Novice
  Install → `skills.lock` written with the correct entry (SRI
  `content_hash`, `source` URL round-tripped, `tier_at_install:
  "novice"`, `kind`, `validation_report_id`). The deferred M07.5 D.fix
  manual repro is now closed.
- **M07.5 CQ-M07-1 (SSRF egress)** — IRL-confirmed. `http://` → blocked
  `Scheme("http")`; `https://127.0.0.1/…` → blocked
  `PrivateAddress(127.0.0.1)`; neither reached a fetch or a review
  modal.
- **M06.5 🔴-2 + the `token_usage` carry-forward** — re-confirmed
  RESOLVED IRL. The SQL inspector after a smoke run + restart showed
  `signals = 4`, `token_usage = 1`, `heartbeats = 16038`,
  `snapshots = 17` — all > 0, surviving the restart. (The M06-IRL
  `token_usage = 0` distinct finding is closed; the in/out split within
  it is the separate #2 above.)
- **The import pipeline** — fetch → validate → §15c → L3 → tier-gate →
  review/install — works; the review screen renders the full §M7
  primitive.
- **Session DB durability** across an app restart.
- **Graph canvas** — pan / zoom / fit / window-resize-to-fill all work
  (gotcha #70 not present).
- **First-run key entry** + the smoke→graph→inspector path.

---

## Deferred / not reached

- **M06.5 🔴-1 (MCP registry path) real-app re-confirmation** — still
  deferred. The M06-IRL post-M07 carry-forward expected this pass to
  re-run the MCP add→list repro; it could not — MCP-add is unreachable
  for a Novice user (finding #5). Carries forward to whenever MCP-add
  becomes reachable (gated on #5).
- **DevTools panel sweep** (plan / HITL / budget / gap panels) — not
  run. Optional; those panels are pre-M07 and unchanged, and the M06
  IRL pass already swept them.
- **MCP "Test connection"** — not run (needs `npx` + a reference
  server).

---

## Disposition / routing

- **#5 → M08 Stage A intake.** 🔴-candidate; decide pre-M08 fix-cycle
  vs M08.A-absorbs after checking the M05/M06 gap-analysis for a known
  deferral.
- **#2, #3, #6, #7 → M08 Stage A `<read_prior_milestones>`
  carry-forward** (mirroring M06-IRL → M07.A). #6 converges with
  M07.V 🟡 #2.
- **#1, #4, #8 → `docs/tech-debt.md`.**
- **M06.5 🔴-1 real-app re-confirmation** — remains deferred, gated on
  #5.
- Per `CLAUDE.md` §20 this IRL pass adds **no `docs/gap-analysis.md`
  entry**; it feeds M08 Stage A's read-prior + M08's gap-analysis
  Carry-forward.

---

## Sign-off

The gate-7 IRL walk-through verified the M07.5 fix-cycle's two
deliverables (🔴 #1, CQ-M07-1) end-to-end in the assembled app — the
manual repros D.fix deferred are now confirmed. M07 + M07.5 ship clean
on their own contract. 8 findings surfaced across the broader app
(1 🔴-candidate, 4 🟡, 3 🟢); all route to M08 Stage A intake / its
carry-forward / tech-debt. No finding is a regression in the M07 or
M07.5 code.
