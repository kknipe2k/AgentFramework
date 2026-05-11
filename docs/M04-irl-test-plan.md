# M04 IRL Test Plan

> Manual (in-real-life) walkthrough of every M04-and-prior feature touchable from the running app. Every event payload, store method, Tauri command name, and UI affordance below is **grep-verified against the codebase at `origin/main` `3c60022`** (post-PR #59 merge). Per `docs/gotchas.md` #41.

## 0. Purpose & Scope

**Goal:** exercise every M01 → M04 feature end-to-end before M05 lands, catch latent bugs while context is hot, and produce a known-good baseline so M05 IRL testing can attribute failures cleanly.

**In scope (what M04 actually wired):**
- M01: Tauri shell + API-key keychain storage + setup panel.
- M02: Smoke session against live Anthropic (Haiku) + event streaming.
- M03: Live graph (11 node types) + SQL inspector + cold-start replay.
- M04.B: Plan/Task FSM + WriteSignal IPC + projection.
- M04.C: ApprovalPanel + PlanNode/TaskNode rendering.
- M04.D: Verify hooks + Rails (`block` / `warn` / `rollback` × `hard` / `soft`).
- M04.E: HITL primitive — 3 UI variants (panel / modal / toast) + 9 triggers + notifier outcomes.
- M04.F: Budget enforcer (4 thresholds) + Recovery (`RecoveryDialog` + `UncertaintyPrompt`).

**Out of scope (deferred — not wired yet, do not test):**
- Plan loop driver (M07 per ADR-0007). No natural `plan_created` emission from running a framework.
- Capability enforcer + Write-tool dispatcher (M05). Don't-touch-glob, capability-violation modal not wired.
- Framework loader (M05). Loading `examples/aria/framework.json` doesn't execute a multi-task plan.
- Live drone-disconnect banner. There is **no** "drone reconnecting" UI in M04 — drone subprocess restarts log to terminal only.
- GapPanel (M05). `gap` node type exists in graphStore but no events emit it.

**Test injection model:**
- **Natural triggers** for M02 baseline (smoke session) — exercise the real network path.
- **Synthetic injection** via `window.__graphStore.getState().applyEvent(...)` for M04 events, since the SDK plan-loop driver isn't shipped.
- **Real UI buttons** for responses (Approve / Revise / Abort / HITL choice / Recovery / Uncertainty action) — these are wired end-to-end via Tauri commands.

---

## 1. Prerequisites & Setup

### 1.1 Environment

```cmd
:: From any PowerShell or cmd prompt on the Windows build machine
rustup show
:: Must show 1.95.0 active (per rust-toolchain.toml pin, gotcha #65)

node --version
:: 20+

npm --version
:: 10+
```

API key — either set the env var:
```cmd
set ANTHROPIC_API_KEY=sk-ant-...
```
or use the in-app keychain (recommended; covered by test SP-01 below).

### 1.2 Build & launch

```cmd
cd C:\agent-runtime
git fetch origin main
git reset --hard origin/main
:: Should land you at 3c60022 (PR #59 merge)

npm install
:: Wait for completion; first run downloads node_modules.

npm run tauri dev
:: First cold build: 5-10 min (~444 transitive Rust deps).
:: Subsequent: ~30-60s. Window opens automatically.
```

### 1.3 Open DevTools

`F12` (or `Ctrl+Shift+I`) in the Tauri window opens DevTools. Keep the **Console** tab open throughout — it's where store-injection happens and where `console.error` lands from `unwrapCmdError` (gotcha #30).

### 1.4 Pass/fail conventions

- **PASS** = expected outcome observed AND no errors in DevTools Console or `tauri dev` terminal.
- **FAIL** = expected outcome NOT observed, OR errors appear in either surface, OR the app freezes/crashes.
- **PARTIAL** = expected outcome observed but with cosmetic / non-blocking issues; record specifics.

---

## 2. Test Index

| # | ID | Feature | Path |
|---|---|---|---|
| 1 | SP-01..03 | Setup panel + API key keychain | M01 |
| 2 | SM-01..03 | Smoke session (live Anthropic) | M02 |
| 3 | LG-01..04 | Live graph rendering + inspector | M03 |
| 4 | SQL-01..05 | SQL inspector + validator | M03.E |
| 5 | RP-01..02 | Cold-start replay | M03.E |
| 6 | PLAN-01..06 | Plan events + ApprovalPanel | M04.B/C |
| 7 | TASK-01..06 | Task events + TaskNode | M04.B/C |
| 8 | VR-01..05 | Verify + Rails | M04.D |
| 9 | HITL-01..12 | HITL primitive (3 variants × triggers) | M04.E |
| 10 | NOTIF-01..02 | Notifier records | M04.E |
| 11 | BUD-01..06 | Budget states + thresholds | M04.F |
| 12 | REC-01..05 | Recovery + Uncertainty | M04.F |
| 13 | DR-01 | Drone subprocess restart (no UI banner) | M04.A2 |
| 14 | ET-01..02 | Error toasts via `unwrapCmdError` | M02+ |
| 15 | INSP-01 | Node inspector behavior | M03.D |

Each section below has a consistent shape: **Goal**, **Setup**, **Steps**, **Expected**, **Pass/fail criteria**, plus **Notes / known limits** when relevant.

---

## 3. M01 — Setup Panel + API Key Keychain

### SP-01 — Save API key via the setup panel

**Goal:** verify keychain write succeeds and the "✓ stored in OS keychain" feedback appears.

**Setup:** App freshly launched. SetupPanel visible (no key stored). Smoke button visible but disabled.

**Steps:**
1. Type `sk-ant-...` (your real key, ≥10 chars) into the password input.
2. Click **Save key**.

**Expected:**
- Input clears after save.
- "✓ stored in OS keychain" feedback appears.
- Smoke button (labelled **Run smoke test**) becomes enabled.
- Windows Credential Manager → Generic Credentials shows an entry whose service starts with `agent-runtime`.

**Pass/fail criteria:** all four bullets observed; no `[object Object]` toast (gotcha #30); DevTools Console clean.

### SP-02 — Save key disabled below 10 chars

**Goal:** verify the **Save key** button is disabled until the input is ≥10 chars.

**Steps:**
1. Reload the app (Ctrl+R in the Tauri window).
2. Type 9 chars into the input.

**Expected:** Save key button stays disabled. Type one more char → enables.

### SP-03 — Re-save replaces existing key silently

**Goal:** verify the keychain write is idempotent (does not warn / prompt for a second time).

**Steps:**
1. With a key already saved, type a different (real) key.
2. Click **Save key**.

**Expected:** same flow as SP-01 — no "already exists" prompt; new key replaces old; "✓ stored in OS keychain" appears again.

---

## 4. M02 — Smoke Session (live Anthropic)

> Each smoke costs ~$0.001 USD on Haiku. Budget accordingly if running through the full plan.

### SM-01 — Smoke session emits agent_spawned → stream_text → agent_complete

**Goal:** end-to-end verify the live SSE pipeline.

**Setup:** API key saved (SP-01). Empty graph.

**Steps:**
1. Click **Run smoke test**.
2. Watch the canvas as events arrive.

**Expected:**
- Within 1–3s, an **AgentNode** appears in the canvas.
- Text streams under the node (or in the inspector if you click the node).
- An **AgentComplete** state shows on the node when done (typically <10s for Haiku, max_tokens=16).
- No CmdError toasts, no errors in DevTools Console.
- The `tauri dev` terminal shows `set_api_key succeeded` and SSE event traces (look for `event=content_block_delta`).

### SM-02 — Smoke without a saved key surfaces "setup_required"

**Goal:** verify the `setup_required` error variant is presented correctly.

**Setup:** Either fresh install OR `Get-Credential -Verbose` + delete the keychain entry, then reload the app.

**Steps:**
1. Click **Run smoke test**.

**Expected:**
- Error message reads `"API key not set..."` (per `ipc.ts:192`), NOT `[object Object]` (gotcha #30).
- DevTools Console shows the full structured `{ type: 'setup_required' }` object.
- App does not crash; Setup panel reappears (or stays visible).

### SM-03 — Smoke with a bad key surfaces "provider" error

**Goal:** verify provider-level errors surface cleanly.

**Setup:** Save a deliberately-invalid key like `sk-ant-invalid-key-1234567890`.

**Steps:**
1. Click **Run smoke test**.

**Expected:**
- Error toast or feedback reads something like `"provider: <message from Anthropic>"` (per `ipc.ts:197`).
- DevTools Console shows the full structured `{ type: 'provider', message: '...' }` object.
- App stays responsive.

---

## 5. M03 — Live Graph Rendering

### LG-01 — Graph renders with dagre layout

**Goal:** confirm the 11 node types are dispatched correctly and dagre auto-layout positions them.

**Setup:** Run SM-01 once so signals exist.

**Expected:** Single AgentNode + edges (no overlap; positioned via dagre per gotcha #52). Zoom and pan controls work. Minimap visible.

### LG-02 — Inject all 11 node types via synthetic events

**Goal:** confirm every node type renders without error.

**Steps:** In DevTools Console:

```javascript
const s = window.__graphStore.getState();
s.clear();

// 1. session_start (framework root)
s.applyEvent({ type: 'session_start', session_id: 's-test', framework: 'aria', model: 'claude-3-haiku-20240307' });

// 2. agent_spawned (agent)
s.applyEvent({ type: 'agent_spawned', agent_id: 'a-1', agent_name: 'planner', parent_id: null, session_id: 's-test' });

// 3. skill_loaded (skill)
s.applyEvent({ type: 'skill_loaded', agent_id: 'a-1', skill_name: 'plan', mode: 'STANDARD' });

// 4. tool_invoked (tool)
s.applyEvent({ type: 'tool_invoked', agent_id: 'a-1', tool_name: 'Read', source: 'builtin', server: null, input: { path: '/tmp/x' } });

// 5. tool_result (closes the tool node)
s.applyEvent({ type: 'tool_result', agent_id: 'a-1', tool_name: 'Read', output: 'file contents...', duration_ms: 42, tokens_in: null, tokens_out: null });

// 6. plan_created (plan)
s.applyEvent({ type: 'plan_created', plan_id: 'p-1', title: 'Test plan', task_count: 3, approval_required: false });

// 7. task_started (task)
s.applyEvent({ type: 'task_started', plan_id: 'p-1', task_id: 't-1', agent_id: 'a-1' });

// 8. verify_started (verify)
s.applyEvent({ type: 'verify_started', hook_id: 'h-1', category: 'verify', firing_point: 'post_task', level: 'task' });

// 9. agent_complete
s.applyEvent({ type: 'agent_complete', agent_id: 'a-1', result: 'done', tokens_total: 100 });
```

**Expected:** Every node type renders (framework, agent, skill, tool, plan, task, verify). Click each → InspectorPanel opens with full event payload. No errors in Console.

> `gap`, `hitl`, `mcp`, `hook` node types: defined in graphStore (`graphStore.ts:264-291`) but no event emits them in M04. **Skip those four for IRL test** — they're M05+ territory.

### LG-03 — Token-weight scales AgentNode size

**Goal:** verify M03.D token-weight visualization works.

**Setup:** Run SM-01, watch the AgentNode while streaming.

**Expected:** AgentNode size grows as token count accumulates (visual; check pre- and post-smoke screenshots).

### LG-04 — clear() resets everything

**Steps:** In DevTools Console: `window.__graphStore.getState().clear()`.

**Expected:** Canvas empties immediately. Inspector closes. Rails / HITL / Budget / Uncertain stores all reset to empty.

### INSP-01 — Inspector behavior

**Goal:** confirm click-to-select and ESC-to-deselect.

**Steps:**
1. After LG-02, click any node.
2. Press ESC.

**Expected:** Inspector opens with full event payload (look for the event type + ID at the top). ESC closes it. Clicking another node opens its data without leaving prior selection.

---

## 6. M03.E — SQL Inspector

> SQL Inspector is the bottom-panel or dedicated button (`SqlInspector.tsx`). Default query: `SELECT * FROM signals LIMIT 10;`.

### SQL-01 — Default SELECT returns rows

**Goal:** verify the happy path.

**Setup:** Run SM-01 first to populate `signals`.

**Steps:**
1. Open SQL Inspector.
2. Press Run (default query already loaded).

**Expected:** Table renders with columns from the first row (`session_id`, `event_type`, `payload`, etc.). At least 3 rows (session_start + agent_spawned + agent_complete) appear. No errors.

### SQL-02 — Schema enumeration

**Steps:** Run:
```sql
SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;
```

**Expected:** rows for at least `signals`, `vdr`, `sessions`, `token_usage`. (Drone schema may include others.)

### SQL-03 — DROP rejected

**Steps:** Run:
```sql
DROP TABLE signals;
```

**Expected:** Error toast/feedback rejecting the query. DevTools Console shows a structured `{ type: 'invalid_sql', ... }` or similar. **Signals table remains intact** — verify by re-running SQL-01.

### SQL-04 — Reject all forbidden verbs

**Goal:** confirm the validator's deny list (`vdr.rs:258` comment).

**Steps:** Run each, one at a time:
```sql
DELETE FROM signals;
INSERT INTO signals (event_type) VALUES ('x');
UPDATE signals SET event_type = 'x';
ALTER TABLE signals ADD COLUMN foo TEXT;
CREATE TABLE foo (id INTEGER);
ATTACH DATABASE ':memory:' AS x;
REPLACE INTO signals (event_type) VALUES ('x');
REINDEX signals;
ANALYZE signals;
PRAGMA table_info(signals);
```

**Expected:** Every query rejected with an error. `signals` table unchanged. **Critical:** if any of these succeed, file a 🔴 finding.

### SQL-05 — Compound statement rejected

**Steps:** Run:
```sql
SELECT * FROM signals; DROP TABLE signals;
```

**Expected:** Rejected (the validator runs single-statement-only per `vdr.rs:270`).

---

## 7. M03.E — Cold-Start Replay

### RP-01 — Replay reconstructs the graph

**Goal:** confirm the localStorage-driven replay path (`App.tsx:52-77`).

**Setup:**
1. Run SM-01 — produces signals + sets `localStorage.lastSessionId`.
2. Note the graph state (number of nodes, AgentNode token-weight visual).

**Steps:**
1. Close the Tauri window entirely (X button or `Ctrl+W`).
2. Re-launch: `npm run tauri dev` (already-built artifact; <30s).

**Expected:**
- On window open, the graph immediately replays — same nodes, same layout. No "Loading" delay beyond a few seconds.
- `tauri dev` terminal shows `replay_session` invocation traces.
- DevTools Console shows the `agent_event` channel re-firing.

### RP-02 — Replay survives DOM-store clear

**Steps:**
1. After RP-01, run `localStorage.clear()` in DevTools Console.
2. Reload the app (Ctrl+R).

**Expected:** No replay this time — empty canvas. Confirms localStorage is the source of truth.

---

## 8. M04.B + M04.C — Plan + Approval

Plan events are injected synthetically since plan_loop driver is M07-deferred.

### PLAN-01 — plan_created → PlanNode renders

**Steps:** DevTools Console:
```javascript
window.__graphStore.getState().applyEvent({
  type: 'plan_created',
  plan_id: 'p-1',
  title: 'Refactor auth flow',
  task_count: 3,
  approval_required: true,
});
```

**Expected:** PlanNode renders with title "Refactor auth flow". Status visual = `created` or `pending_approval`. Click → inspector shows `taskCount: 3`, `approvalRequired: true`.

### PLAN-02 — plan_approval_requested → ApprovalPanel surfaces

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'plan_approval_requested',
  plan_id: 'p-1',
});
```

**Expected:** ApprovalPanel mounts on the right side. Title "Refactor auth flow" shown. Three buttons: **Approve**, **Revise**, **Abort**. PlanNode status transitions to `awaiting_approval` (gotcha #52 — the layout-memo bug fix: status change must persist correctly).

### PLAN-03 — Approve button → plan_approved emits + ApprovalPanel dismisses

**Steps:**
1. Click **Approve** in the ApprovalPanel.

**Expected:**
- ApprovalPanel auto-dismisses.
- DevTools Console shows `approve_plan` Tauri invocation succeeds.
- PlanNode status transitions to `approved`.
- A `plan_approved` event lands in the store with `approved_by: 'user'` (per `ApprovedBy` enum, gotcha #51).

> Note: this returns **soft-Ok** (`approve_plan_with` line 625-627) because the SDK plan-loop driver isn't shipped — there's no `oneshot::Receiver` to actually deliver the decision to. That's expected M04 behavior.

### PLAN-04 — Revise

**Setup:** Re-inject PLAN-01 + PLAN-02 (or reload + re-inject).

**Steps:**
1. Click **Revise**.
2. If a text input prompts for revision reason: type "Re-scope step 2 to extract auth helper".
3. Submit.

**Expected:** `revise_plan` Tauri command fires with the revision string. ApprovalPanel dismisses. PlanNode transitions to `revising` or back to `created`.

### PLAN-05 — Abort

**Setup:** Re-inject PLAN-01 + PLAN-02.

**Steps:** Click **Abort**, provide reason "Out of scope" if prompted.

**Expected:** `abort_plan` fires. PlanNode status → `aborted`. ApprovalPanel dismisses.

### PLAN-06 — plan_complete

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'plan_complete',
  plan_id: 'p-1',
  duration_ms: 12345,
});
```

**Expected:** PlanNode status → `complete`. Inspector shows `durationMs: 12345`.

### TASK-01..06 — Task events

For each: inject the event, observe the TaskNode rendering or transition.

```javascript
const s = window.__graphStore.getState();

// TASK-01: task_started
s.applyEvent({ type: 'task_started', plan_id: 'p-1', task_id: 't-1', agent_id: 'a-1' });
// Expected: TaskNode renders, status 'in_progress'.

// TASK-02: task_completed
s.applyEvent({ type: 'task_completed', plan_id: 'p-1', task_id: 't-1', duration_ms: 5000 });
// Expected: status 'completed'.

// TASK-03: task_failed
s.applyEvent({ type: 'task_started', plan_id: 'p-1', task_id: 't-2', agent_id: 'a-1' });
s.applyEvent({ type: 'task_failed', plan_id: 'p-1', task_id: 't-2', error: 'Read returned EACCES', failure_count: 1 });
// Expected: status 'failed', failureCount 1, lastError 'Read returned EACCES'.

// TASK-04: task_skipped
s.applyEvent({ type: 'task_started', plan_id: 'p-1', task_id: 't-3', agent_id: 'a-1' });
s.applyEvent({ type: 'task_skipped', plan_id: 'p-1', task_id: 't-3', reason: 'Prerequisite t-2 failed' });
// Expected: status 'skipped'.

// TASK-05: task_escalated
s.applyEvent({ type: 'task_failed', plan_id: 'p-1', task_id: 't-2', error: 'retry 1', failure_count: 2 });
s.applyEvent({ type: 'task_failed', plan_id: 'p-1', task_id: 't-2', error: 'retry 2', failure_count: 3 });
s.applyEvent({ type: 'task_escalated', plan_id: 'p-1', task_id: 't-2', failure_count: 3, max_failures: 3 });
// Expected: TaskNode shows escalated state; hitl badge if HITL fires on the threshold.

// TASK-06: task_rolled_back
s.applyEvent({ type: 'task_started', plan_id: 'p-1', task_id: 't-4', agent_id: 'a-1' });
s.applyEvent({ type: 'task_rolled_back', plan_id: 'p-1', task_id: 't-4', snapshot_id: 'snap-abc-123' });
// Expected: status 'rolled_back', rollbackSnapshotId 'snap-abc-123'.
```

**Expected (all):** Each transition is reflected in TaskNode visual + inspector data. Layout-memo bug (gotcha #52) means status change MUST persist across re-renders — if a task flips back to its prior state when you click around, that's a regression.

---

## 9. M04.D — Verify + Rails

### VR-01 — verify_started → VerifyNode renders

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'verify_started',
  hook_id: 'h-1',
  category: 'verify',
  firing_point: 'post_task',
  level: 'task',
});
```

**Expected:** VerifyNode renders. Status visual = `active`. Click → inspector shows `category: 'verify'`, `firingPoint: 'post_task'`.

### VR-02 — verify_passed transitions VerifyNode

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'verify_passed',
  hook_id: 'h-1',
  duration_ms: 800,
  output_preview: 'all tests pass',
});
```

**Expected:** VerifyNode status → `pass`. Inspector shows `durationMs: 800`, `outputPreview: 'all tests pass'`.

### VR-03 — verify_failed (block / warn / rollback variants)

For each `on_failure` value:

```javascript
const s = window.__graphStore.getState();

// block
s.applyEvent({ type: 'verify_started', hook_id: 'h-b', category: 'verify', firing_point: 'post_task', level: 'task' });
s.applyEvent({ type: 'verify_failed', hook_id: 'h-b', duration_ms: 500, error: 'lint errors', on_failure: 'block' });

// warn
s.applyEvent({ type: 'verify_started', hook_id: 'h-w', category: 'lint', firing_point: 'post_task', level: 'task' });
s.applyEvent({ type: 'verify_failed', hook_id: 'h-w', duration_ms: 500, error: 'soft lint', on_failure: 'warn' });

// rollback
s.applyEvent({ type: 'verify_started', hook_id: 'h-r', category: 'test', firing_point: 'post_task', level: 'task' });
s.applyEvent({ type: 'verify_failed', hook_id: 'h-r', duration_ms: 500, error: 'tests broke', on_failure: 'rollback' });
```

**Expected:** Three VerifyNodes, each in `fail` status, each with the corresponding `onFailure` value visible in inspector. Different visual cue per variant (if the renderer surfaces it).

### VR-04 — rail_triggered (hard policy)

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'rail_triggered',
  rail_id: 'r-1',
  policy: 'hard',
  firing_point: 'pre_tool_use',
  message: 'Tool Bash blocked: Cargo.lock in arg list',
  agent_id: 'a-1',
});
```

**Expected:** Entry appended to `state.triggeredRails`. Verify in Console:
```javascript
window.__graphStore.getState().triggeredRails
// Should show array with one entry, policy 'hard'.
```

> **Important:** M04 does NOT yet render rails as nodes or surface a panel — `triggeredRails` is store-state only, M05 capability-enforcer UI will surface it (graphStore.ts:335 comment). Confirm via Console only; visual surface is M05 territory.

### VR-05 — rail_triggered (soft policy, no agent_id)

```javascript
window.__graphStore.getState().applyEvent({
  type: 'rail_triggered',
  rail_id: 'r-2',
  policy: 'soft',
  firing_point: 'pre_skill_load',
  message: 'Soft warning',
  agent_id: null,
});
```

**Expected:** Second entry in `triggeredRails`; `agentId: null`.

---

## 10. M04.E — HITL Primitive

> Three UI variants × nine triggers. The renderer routes by `ui_variant` field, NOT by trigger value (per `graphStore.ts:1023-1039` + `HITLPanel.tsx:141`). Each variant has different behavior.

### HITL-01 — `panel` variant

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'hitl_requested',
  prompt_id: 'h-panel-1',
  trigger: 'on_gap',
  agent_id: 'a-1',
  question: 'Skill `plan` not declared in framework. Load it?',
  options: ['load', 'skip', 'abort'],
  ui_variant: 'panel',
  timeout_at_unix_ms: Date.now() + 3600000,
});
```

**Expected:**
- HITLPanel mounts on the right side.
- `aria-modal="false"` (panel is non-blocking; graph stays interactive).
- Title shows the question.
- Three buttons labelled `load`, `skip`, `abort`.
- ESC key dismisses (locally) without responding.

### HITL-02 — `modal` variant

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'hitl_requested',
  prompt_id: 'h-modal-1',
  trigger: 'on_risky_tool',
  agent_id: 'a-1',
  question: 'Tool `Bash` invoked with rm command. Confirm?',
  options: ['allow', 'deny'],
  ui_variant: 'modal',
  timeout_at_unix_ms: Date.now() + 3600000,
});
```

**Expected:**
- HITLModal renders centered with backdrop (`hitl-modal-backdrop` CSS).
- `aria-modal="true"`. Focus trapped inside the modal.
- Background graph interactions blocked.
- ESC key dismisses locally.

### HITL-03 — `toast` variant

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'hitl_requested',
  prompt_id: 'h-toast-1',
  trigger: 'per_task',
  agent_id: 'a-1',
  question: 'Task t-1 complete — review?',
  options: ['view', 'dismiss'],
  ui_variant: 'toast',
  timeout_at_unix_ms: Date.now() + 30000,
});
```

**Expected:**
- HITLToast appears bottom-right (or wherever toasts surface).
- `role="status"`, `aria-live="polite"`.
- Auto-dismisses after 30 seconds (per `TOAST_AUTO_DISMISS_MS` in `HITLToast.tsx:5`).
- Click to expand reveals options.

### HITL-04 — Respond via Approve button → hitl_resolved emits

**Setup:** Inject HITL-01 (panel variant, with options `['load', 'skip', 'abort']`).

**Steps:** Click the **load** button in the HITLPanel.

**Expected:**
- `respond_hitl` Tauri command fires with `prompt_id: 'h-panel-1'`, `choice: 'load'`.
- HITLPanel dismisses.
- `pendingHitl['h-panel-1']` removed from store state. Verify:
```javascript
window.__graphStore.getState().pendingHitl
// Should not contain h-panel-1.
```
- Soft-Ok response from Tauri (per `respond_hitl_with` line 602-603) since no SDK awaiter is registered.

### HITL-05..12 — All 9 trigger values

For each of the 9 HitlTriggerRef values, inject one `hitl_requested` event and verify it routes to the variant + the trigger string appears in the inspector.

```javascript
const s = window.__graphStore.getState();
const triggers = [
  ['on_gap',                  'panel'],
  ['on_risky_tool',           'modal'],
  ['on_dont_touch_edit',      'modal'],
  ['on_failure_threshold',    'panel'],
  ['on_capability_violation', 'modal'],
  ['on_budget_threshold',     'modal'],
  ['on_plan_approval',        'panel'],
  ['per_task',                'modal'],
  ['per_epic',                'panel'],
];
// (Mapping is the M04 default per hitl/policy.rs lines 18-19;
// the renderer obeys whatever ui_variant arrives — these are sensible defaults.)
triggers.forEach(([trig, variant], i) => {
  s.applyEvent({
    type: 'hitl_requested',
    prompt_id: `h-trig-${i}`,
    trigger: trig,
    agent_id: 'a-1',
    question: `Trigger: ${trig}`,
    options: ['ack'],
    ui_variant: variant,
    timeout_at_unix_ms: Date.now() + 3600000,
  });
});
```

**Expected:** Each renders correctly; click "ack" on each to dismiss. After loop, `pendingHitl` should be empty.

### HITL-13 — hitl_timeout

**Steps:**
```javascript
const s = window.__graphStore.getState();
s.applyEvent({
  type: 'hitl_requested',
  prompt_id: 'h-timeout',
  trigger: 'per_task',
  agent_id: 'a-1',
  question: 'Will time out',
  options: ['ack'],
  ui_variant: 'panel',
  timeout_at_unix_ms: Date.now() + 5000,
});
// Wait or simulate timeout:
s.applyEvent({
  type: 'hitl_timeout',
  prompt_id: 'h-timeout',
  trigger: 'per_task',
  default_action: 'abort',
});
```

**Expected:** HITLPanel dismisses. `pendingHitl['h-timeout']` removed. Inspector or audit log shows `defaultAction: 'abort'` applied.

### NOTIF-01 — notifier_dispatched (success)

**Setup:** Inject HITL-01 first (so a pending prompt exists for the notifier to attach to).

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'notifier_dispatched',
  notifier_type: 'terminal_bell',
  trigger: 'on_gap',
  success: true,
});
```

**Expected:** Entry appended to `state.notifierRecords['h-panel-1']` (or however the prompt_id is keyed). Verify in Console:
```javascript
window.__graphStore.getState().notifierRecords
```

### NOTIF-02 — notifier_failed

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'notifier_failed',
  notifier_type: 'desktop',
  trigger: 'on_gap',
  error: 'win32 desktop API returned 0x80004005',
});
```

**Expected:** Entry appended with `outcome: 'failed'`, error string preserved. After the HITL prompt resolves, `notifierRecords[prompt_id]` is cleared (per `graphStore.ts:1043-1054`).

---

## 11. M04.F — Budget States + Recovery

### BUD-01 — budget_warn

**Setup:** Reset state: `window.__graphStore.getState().clear()`.

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'budget_warn',
  spent_usd: 2.50,
  cap_usd: 5.00,
  percent: 50,
});
```

**Expected:**
- BudgetHeaderBar appears at the top of the window (was hidden when budget state was `ok`).
- Color class `budget-bar__bar--warn` applied (visual: yellow / amber band).
- Shows `$2.50 / $5.00` and `50%`.

### BUD-02 — budget_downshift

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'budget_downshift',
  from_model: 'claude-3-opus-20240229',
  to_model: 'claude-3-haiku-20240307',
  reason: 'spend at 75% — auto-downshift to Haiku',
});
```

**Expected:**
- Badge "Downshifted" appears (`BudgetHeaderBar.tsx:131-138`).
- Both model names visible (`claude-3-opus-20240229 → claude-3-haiku-20240307`).
- Spend / cap from BUD-01 preserved (per the last-known-spend pattern at `graphStore.ts:1118-1125`).

### BUD-03 — budget_suspended

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'budget_suspended',
  spent_usd: 4.50,
  cap_usd: 5.00,
});
```

**Expected:**
- Badge "Suspended — awaiting approval" (`BudgetHeaderBar.tsx:123-130`).
- A `hitl_requested` SHOULD be paired with this in production (trigger=`on_budget_threshold`). In synthetic testing, you'd inject both side-by-side.

### BUD-04 — budget_exceeded

**Steps:**
```javascript
window.__graphStore.getState().applyEvent({
  type: 'budget_exceeded',
  spent_usd: 5.10,
  cap_usd: 5.00,
});
```

**Expected:**
- Banner "Session terminated — budget exceeded" (`BudgetHeaderBar.tsx:115-122`).
- In production, drone would receive `StopProcess` IPC; in synthetic test, just the visual.
- `state.budget.status === 'exceeded'`.

### BUD-05 — threshold defaults (50 / 75 / 90 / 100)

**Goal:** verify defaults from `enforcer.rs:154-171`.

**Setup:** N/A — this is documentation only. Confirm by reading `crates/runtime-main/src/budget/enforcer.rs:154-171`.

**Expected (no UI test):** `warn_at: 50, downshift_at: 75, suspend_at: 90, hard_stop_at: 100`.

### BUD-06 — Budget cap settings form

**Goal:** verify the cap-setting UI doesn't suffer gotcha #63 (stale closure).

**Steps:**
1. Open budget settings (location: in or near `BudgetHeaderBar`; likely a gear / settings icon).
2. Type `5.00` into the cap input.
3. Submit / Save.
4. Type `10.00` into the same input.
5. **Submit immediately without leaving the input** (i.e., press Enter or Save on the same render).

**Expected:**
- Both values save correctly. The second save is `10.00`, NOT `5.00` (the stale-closure bug from gotcha #63).
- Confirm via:
  ```javascript
  // No direct getter; check the tauri dev terminal for the set_global_budget invocation log.
  ```
- Reject invalid: type `-1.00` → error. Type `NaN` → error. Type `0` → stores `None` (clears the cap).

---

## 12. M04.F — Recovery + Uncertainty

### REC-01 — RecoveryDialog appears on cold-start with prior session

**Setup:**
1. Run SM-01 to populate signals + set `localStorage.lastSessionId`.
2. Close the app.

**Steps:**
1. Reload `npm run tauri dev`.

**Expected:**
- RecoveryDialog mounts (`role="dialog"`, `aria-modal="true"`, test ID `recovery-dialog` per `RecoveryDialog.tsx:74-77`).
- Two buttons: **Resume** and **Discard**.

> Replay (RP-01) also fires on cold-start — RecoveryDialog and replay are separate. If both appear, RecoveryDialog is in front and the user chooses. If you click **Resume** and there are `uncertain_tool_invocations`, those surface as UncertaintyPrompts (REC-04).

### REC-02 — Resume button

**Steps:** Click **Resume**.

**Expected:**
- `request_resume` Tauri command fires (`commands.rs:399`).
- Returns `ResumePlan` with `uncertain_tool_invocations[]` populated if any were mid-flight.
- For each: `recordUncertainInvocation` called → UncertaintyPrompts surface.
- RecoveryDialog dismisses.

### REC-03 — Discard button

**Steps:** Re-create the cold-start state, then click **Discard**.

**Expected:**
- `localStorage.lastSessionId` cleared.
- RecoveryDialog dismisses.
- Empty canvas remains.

### REC-04 — UncertaintyPrompt (4 actions)

**Goal:** verify the 4 valid actions (`retry`, `skip`, `mark_complete`, `abort` per `ipc.ts:103`).

**Setup:** Inject manually (don't wait for a real recovery):
```javascript
window.__graphStore.getState().recordUncertainInvocation({
  invocationId: 'inv-test-1',
  toolName: 'Bash',
  agentId: 'a-1',
});
```

**Expected:** UncertaintyPrompt appears with 4 buttons (or a select). For each action:

| Action | Steps | Expected |
|---|---|---|
| retry | Click **retry** | `respond_uncertainty` fires with `action: 'retry'`. Prompt dismisses. |
| skip | Re-inject + click **skip** | `respond_uncertainty` fires with `action: 'skip'`. |
| mark_complete | Re-inject + click **mark_complete** | `respond_uncertainty` fires with `action: 'mark_complete'`. |
| abort | Re-inject + click **abort** | `respond_uncertainty` fires with `action: 'abort'`. |

### REC-05 — UncertaintyPrompt with unknown tool

**Setup:** Inject without toolName:
```javascript
window.__graphStore.getState().recordUncertainInvocation({
  invocationId: 'inv-no-name',
  toolName: null,
  agentId: 'a-1',
});
```

**Expected:** Prompt shows fallback text like "(unknown tool)" or similar — no crash. M05.A adds the `toolName` field requirement, so pre-M05 prompts may show fallback text.

---

## 13. M04.A2 — Drone Subprocess Restart (no UI banner)

### DR-01 — Kill drone, observe Rust-side reconnect

**Goal:** confirm drone reconnect works at the IPC layer (200ms exp backoff, 5 attempts per `drone_lifecycle.rs:47`). **Confirm there is NO renderer-side "reconnecting" banner** — that was misstated in the prior walkthrough; M04 has no such UI.

**Setup:** Run SM-01 (so drone is alive + producing signals).

**Steps:**
1. In a separate PowerShell terminal:
   ```powershell
   Get-Process runtime-drone | Stop-Process -Force
   ```
2. Watch the `tauri dev` terminal.

**Expected:**
- Terminal log: `drone disconnected` (or similar) + reconnect attempts at 200ms / 400ms / 800ms / 1.6s / 3.2s exponential backoff.
- After successful reconnect (or 5 failed attempts): terminal logs continue.
- **Renderer:** no banner, no toast, no visual change. The user notices nothing unless they're watching the terminal — this is the deliberate M04 behavior pending M05.
- If signals were mid-flight when the drone died, they may be lost; if a `tool_invoked` was mid-flight, UncertaintyPrompt would surface on next cold-start (REC-04). In live mid-session, the renderer just stops receiving events until reconnect succeeds.

> **Caveat:** Windows may briefly hold the runtime-drone binary's file lock after SIGKILL (gotcha #47). If you see `Access is denied (os error 5)` in the terminal, that's the lock release lag — wait 2-3 seconds.

---

## 14. Cross-Cutting Error Handling

### ET-01 — set_api_key error toast

**Goal:** verify all `CmdError` variants render with `unwrapCmdError` (gotcha #30).

**Steps:**
1. With no key saved, click **Run smoke test**.

**Expected:** Toast / feedback reads `"API key not set..."` (NOT `[object Object]`). DevTools Console shows full `{ type: 'setup_required' }` object. (Same as SM-02.)

### ET-02 — Tauri command failure toast

**Goal:** verify the unwrapCmdError fallback for unrecognized error shapes.

**Setup:** Hard to trigger without intentionally breaking something. Best path: inspect `ipc.ts:186-207` — the fallback path is `String(e)` only after exhausting `setup_required`, `provider`, `internal` variants.

**Expected (manual code-read):** if a new `CmdError` variant lands without a renderer-side branch, it shows `String({type: 'newvariant'})` — readable but ugly. This is the fallback; not a bug.

---

## 15. What's NOT Wired in M04 (do NOT test)

Listed here so you don't waste time:

| Feature | Wired In | Skip In M04 IRL Test |
|---|---|---|
| Plan loop driver (real multi-task plan execution) | M07 | YES |
| Capability enforcer (don't-touch glob, Write-tool dispatcher) | M05 | YES |
| Capability-violation modal triggered naturally | M05 | YES |
| GapPanel + `gap` node emission | M05 | YES |
| Framework loader (`request_capability` meta-tool) | M05 | YES |
| Tier system (Novice / Promoted gates) | M05 | YES |
| Audit log (`skills.audit.jsonl`) | M05 | YES |
| Live drone-disconnect banner | (no plan yet) | YES |
| Triggered rails UI panel | M05 | YES |
| `mcp` / `hook` / `gap` / `hitl` node types as natural-graph entries | M05 / M06 | YES |
| Real downshift hook executing from framework JSON | M07 | YES |

---

## 16. Issue Reporting Template

For each failure or anomaly, capture:

```markdown
**Test ID:** SM-01 (etc.)
**Severity:** 🔴 Critical / 🟡 Important / 🟢 Nice-to-have
**Expected:** <what the plan said should happen>
**Observed:** <what actually happened>
**DevTools Console output:** <paste full structured object if any>
**`tauri dev` terminal output:** <paste relevant lines>
**Reproduction steps:** <minimal steps to reproduce>
**Screenshot:** <attach if visual>
**Hypothesis:** <if you have one>
```

Surface findings to me (in chat) or aggregate in a single message at the end of the run.

---

## 17. Cleanup

After the run:

```javascript
// In DevTools Console:
window.__graphStore.getState().clear();
localStorage.clear();
```

Then close the app cleanly (X button, not force-kill — gotcha #47).

Delete keychain entry if you used a throwaway key:
- Windows Credential Manager → Generic Credentials → find `agent-runtime` → Remove.

Verify clean state for the next test run:
```cmd
git status
:: Should be clean (no untracked artifacts).
```

---

## Appendix A: Event payload cheat sheet

All payloads omit `timestamp` — the schema-generated TS types at `src/types/agent_event.ts` don't include it; the projector ignores it.

```javascript
// M02 baseline (natural-triggered by run_smoke_session)
session_start:   { type, session_id, framework, model }
agent_spawned:   { type, agent_id, agent_name, parent_id, session_id }
agent_complete:  { type, agent_id, result, tokens_total }
agent_error:     { type, agent_id, error }
tool_invoked:    { type, agent_id, tool_name, source, server, input }
tool_result:     { type, agent_id, tool_name, output, duration_ms, tokens_in, tokens_out }
skill_loaded:    { type, agent_id, skill_name, mode }
stream_text:     { type, agent_id, text }

// M04.B/C plan + task
plan_created:             { type, plan_id, title, task_count, approval_required }
plan_approval_requested:  { type, plan_id }
plan_approved:            { type, plan_id, approved_by: 'user' | 'auto' }
plan_revised:             { type, plan_id, revision_reason }
plan_aborted:             { type, plan_id, reason }
plan_complete:            { type, plan_id, duration_ms }
task_started:             { type, plan_id, task_id, agent_id }
task_completed:           { type, plan_id, task_id, duration_ms }
task_failed:              { type, plan_id, task_id, error, failure_count }
task_skipped:             { type, plan_id, task_id, reason }
task_escalated:           { type, plan_id, task_id, failure_count, max_failures }
task_rolled_back:         { type, plan_id, task_id, snapshot_id }

// M04.D verify + rails
verify_started:  { type, hook_id, category, firing_point, level }
verify_passed:   { type, hook_id, duration_ms, output_preview }
verify_failed:   { type, hook_id, duration_ms, error, on_failure: 'block'|'warn'|'rollback' }
rail_triggered:  { type, rail_id, policy: 'hard'|'soft', firing_point, message, agent_id }

// M04.E HITL
hitl_requested:      { type, prompt_id, trigger, agent_id, question, options, ui_variant, timeout_at_unix_ms }
hitl_resolved:       { type, prompt_id, choice, duration_ms }
hitl_timeout:        { type, prompt_id, trigger, default_action }
notifier_dispatched: { type, notifier_type, trigger, success }
notifier_failed:     { type, notifier_type, trigger, error }

// M04.F budget
budget_warn:       { type, spent_usd, cap_usd, percent }
budget_downshift:  { type, from_model, to_model, reason }
budget_suspended:  { type, spent_usd, cap_usd }
budget_exceeded:   { type, spent_usd, cap_usd }

// Store methods (not events)
recordUncertainInvocation({ invocationId, toolName, agentId })
resolveUncertainInvocation(invocationId)
```

## Appendix B: Tauri command reference

| Command | Args | Returns | UI trigger |
|---|---|---|---|
| `set_api_key` | `key: String` | `()` | SetupPanel → Save key |
| `run_smoke_session` | (none — uses AppHandle) | `()` | SmokeButton → Run smoke test |
| `query_session_db` | `sql: String` | `Vec<Value>` | SqlInspector → Run |
| `replay_session` | `session_id: String` | `()` | Cold-start (App.tsx auto) |
| `approve_plan` | `plan_id: String` | `()` | ApprovalPanel → Approve |
| `revise_plan` | `plan_id, revisions: String` | `()` | ApprovalPanel → Revise |
| `abort_plan` | `plan_id, reason: String` | `()` | ApprovalPanel → Abort |
| `request_resume` | `session_id: String` | `ResumePlan` | RecoveryDialog → Resume |
| `respond_uncertainty` | `session_id, invocation_id, action, agent_id?` | `UncertaintyResolution` | UncertaintyPrompt → action button |
| `set_global_budget` | `usd_cap: f64` | `()` | Budget settings → Save |
| `respond_hitl` | `prompt_id, choice: String` | `()` | HITLPanel/Modal/Toast → option button |

## Appendix C: HITL trigger → default UI variant (per `policy.rs:18-19`)

Note: the renderer routes by `ui_variant` field on the event, NOT by trigger. These are the defaults M04 emits in production; testing can use any variant per trigger.

| Trigger | Default variant |
|---|---|
| `on_gap` | panel |
| `on_risky_tool` | modal |
| `on_dont_touch_edit` | modal |
| `on_failure_threshold` | panel |
| `on_capability_violation` | modal |
| `on_budget_threshold` | modal |
| `on_plan_approval` | panel |
| `per_task` | modal |
| `per_epic` | panel |

---

**End of test plan.** Estimated runtime: 60–90 minutes if working through every test linearly; 30 minutes if you skip the "synthetic injection × 9 triggers" loop (HITL-05..12).
