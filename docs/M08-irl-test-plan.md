# M08 IRL Test Plan — Windows Runbook (the Workbench: Builder Canvas + Tester + Settings)

Manual verification of the running app as of the M08 merge. Default shell is **PowerShell**. Every fenced block is paste-as-is; `[DevTools JS]` = paste into the app's F12 → Console, everything else is PowerShell. No mocks — real app, real backend, real renderer. Most of M08 is directly manipulable (drag / click / type) — far less DevTools injection than the M06 pass. Findings feed M09 Stage A.

---

## 0. Do NOT log these as bugs (scope fence)

These are known and carried — seeing them is expected, not a finding:

- **The Tester's VDR panel shows `null`.** `TestOutcome.vdr` is structurally dead (M08.V 🟡 #1) — carried to M09.A. The other four Tester surfaces (graph / signals / token spend / pass-fail) are live.
- **You cannot delete a node from the canvas.** `removeNode` is a no-op stub (TD-020).
- **A JSON-tab edit or a Load leaves red badges stale** until you click **Validate** (TD-019) — continuous validation only re-fires on canvas edits.
- **No Task node in the Palette** → a Hook→Task edge is not canvas-drawable in v0.1 (known product↔spec gap).
- **The plan-loop driver never fires on its own** — `drive_plan` shipped tested but unwired (M08.V 🟡 #2).

In scope to actively re-check: **M06.5 IRL 🔴-1** (the MCP-registry mis-resolve) — Stage G made the Promoted tier reachable, so the MCP Add modal can finally be exercised (Scenario E).

---

## 1. Where each thing runs

| Context | What |
|---|---|
| **Window A** (PowerShell) | builds + runs the app; stays occupied (`npm run tauri dev` never returns) |
| **Window B** (PowerShell) | every verify command; `$APPDATA_DIR` / `$REG` / `$SCRATCH` persist here — keep it open |
| **App** | manual drag / click / type + visual inspection in the running window |
| **App DevTools** (F12 → Console) | the few `[DevTools JS]` checks |

---

## 2. Setup

### Window A — build + run (then occupied)

```powershell
npm ci
cargo build --workspace
npm run tauri dev
```

### Window B — open a second PowerShell, paste once, keep it open

```powershell
$APPDATA_DIR = "$env:LOCALAPPDATA\dev.aria-runtime.app"
$AUDIT = "$APPDATA_DIR\skills.audit.jsonl"
$REG   = "$APPDATA_DIR\session.sqlite"
$SCRATCH = (New-Item -ItemType Directory "$env:TEMP\m08irl-$(Get-Random)").FullName
$SCRATCH
```

---

## 3. Walkthroughs — run A → B → C → D → E in order

### Scenario A — Runtime ↔ Builder view switch + carry-forward fixes

**A1. [App]** App on the main screen. Look: a **Runtime / Builder** view-switch control (chrome, near the title). PASS = the switch is present.

**A2. [App]** Trigger a smoke run; let it finish. Look: in the live graph an agent node shows token activity. After it finishes, hover/inspect token usage. Expect: token in/out are **split and non-zero** (M07-IRL #2). PASS = distinct in + out values.

**A3. [App]** Close the app fully; in **Window A** Ctrl+C then `npm run tauri dev`. Expect: **no API-key prompt** — the key persists across restart (M07-IRL #7). PASS = no prompt.

**A4. [App]** Click the Import panel (Runtime mode). Look: the panel text is **readable** — no low-contrast / invisible text (M07-IRL #3). After a restart the panel still lists previously installed artifacts (M07-IRL #6). PASS = readable + survives restart.

**A5. [App]** Click **Builder** on the view switch. Look: the three-panel Builder shell — **Palette** (left, 5 tabs: Tools / Skills / Agents / HITL / Hooks), an empty **Canvas** (center), an **Inspector** (right). Switch back to **Runtime** — the live graph is exactly as you left it. PASS = both views render, switching is lossless.

### Scenario B — Build a framework on the Builder Canvas

App in **Builder** mode.

**B1. [App]** Palette → Agents tab. Drag an Agent item onto the empty canvas. Look: an Agent node appears where you dropped it. Drag it to a new spot — it moves and stays. PASS = node instantiates + is draggable.

**B2. [App]** Click the Agent node. Look: an inline config panel — role (text), model (dropdown), allowed_tools / allowed_skills. Set a role; pick a model. Look: below the node, a plain-English capability disclosure. PASS = config edits reflect on the node + the disclosure.

**B3. [App]** Drag a Tool and a Skill from the Palette onto the canvas. Drag an edge from the Agent's handle to the Skill, then Agent → Tool. Look: a wire paints for each. PASS = both edges draw.

**B4. [App]** Drag a wire between two nodes that is **not** a spec edge type (e.g. Tool → Tool). Expect: **no edge appears** — the pair is rejected. PASS = no wire.

**B5. [App]** Drag a second Agent on; draw an **Agent → Agent** edge. Look: a **narrowing notice** — the child's declared capabilities vs the parent's. Now give the child a capability the parent lacks and re-draw: expect a **rejection** notice + a **red badge** on the child node. PASS = narrowing surfaces; an over-declaring child is flagged red.

**B6. [App]** Mis-configure a node (e.g. clear a required field). Within ~1s: a **red badge** appears on that node (continuous validation). Fix it: the badge clears. PASS = badges track validity live.

### Scenario C — Inspector + Canvas|JSON + Save / Load

**C1. [App]** Look at the **Inspector** (right): a live `framework.json` preview that updates as you edit the canvas; a whole-framework **capability summary**. PASS = preview is live + summary populates.

**C2. [App]** Click **Validate** in the Inspector. Look: the full validation result — per-node schema / capability errors. PASS = the result matches the badges from B5/B6.

**C3. [App]** Switch the center region to the **JSON** tab. Edit a value (valid JSON). Switch back to **Canvas**. Look: the canvas reflects the JSON edit. Now type **invalid** JSON → an inline parse error, and the canvas does **not** change (no desync). PASS = round-trips both ways; invalid JSON is contained.

**C4. [App]** Inspector → **Save** / Export. Pick `$SCRATCH` (the Window-B path) in the native picker.

**C5. [Window B]**

```powershell
Get-ChildItem $SCRATCH
```

Expect: `framework.json` plus companion `skill.md` / `tool.md` / `agent.md` for inline artifacts. PASS = files written.

**C6. [App]** Inspector → **Load** / Open → pick `$SCRATCH`. Look: the canvas reconstructs the saved framework. PASS = reload matches the saved state.

### Scenario D — The Tester (isolated sandbox session)

**D1. [App]** Inspector → **Test**. Look: the Tester modal opens. Enter a task; click **Run**. Look: a **scoped graph pane** renders the test run; **token spend** and a **pass / fail** result populate. (The **VDR** panel shows `null` — §0, expected.) PASS = the run executes + 4 surfaces populate.

**D2. [Window B]**

```powershell
sqlite3 $REG "SELECT count(*) FROM signals;"
```

Note the count. The Tester uses a **throwaway** DB under `%TEMP%`, never `session.sqlite` — this count must **not** jump because of the test run. PASS = the live DB is untouched by the run.

**D3. [App]** Build a framework whose child agent over-declares a capability (B5). Run the Tester on it. Expect: a **FAIL** line naming the capability violation — and **no HITL modal** pops (test-defaults auto-resolve). PASS = capability violation → test failure, no prompt.

**D4. [App]** **Close** the Tester modal. Switch to **Runtime**. Look: the live graph is exactly as before — the test run left no trace. PASS = discard-on-close; the live graph is untouched.

### Scenario E — Settings panel + tier promotion + the M06.5 🔴-1 re-confirm

**E1. [App]** Find the **Settings** panel (cross-mode chrome — present in both Runtime and Builder). Look: a tier display (currently **Novice**) + a budget-cap input. Operator must **not** appear anywhere. PASS = panel visible in both modes; no Operator.

**E2. [App]** Click **Promote**. Look: the tier display updates to **Promoted**. PASS = promotion works (the displayed tier updates).

**E3. [App]** Set a budget cap, save it, re-open the panel. Expect: the input still shows the value (M06.5 🟡-4). PASS = the cap reflects + persists.

**E4. [App]** Now Promoted: Settings → MCP Servers → **Add**. Name `m08-irl-fs`; Transport `stdio`; Command `npx`; Arguments `-y @modelcontextprotocol/server-filesystem ` + the `$SCRATCH` path. Confirm.

**E5. [Window B]** — the M06.5 IRL 🔴-1 re-confirm:

```powershell
sqlite3 $REG "SELECT name,transport,status FROM mcp_servers;"
Get-ChildItem "$APPDATA_DIR\mcp.sqlite" -ErrorAction SilentlyContinue
```

Expect: an `m08-irl-fs` row **in `session.sqlite`** (`$REG`); and **no stray `mcp.sqlite`** file. PASS = the server is in the correct DB, no stray file. FAIL / stray file = M06.5 🔴-1 still open — log it.

---

## 4. Findings (feeds M09 Stage A)

🔴 fix before M09 · 🟡 M09 Stage A absorbs · 🟢 `docs/tech-debt.md` · BLOCKED = environment, not a bug.

| # | Step | Sev | Observed vs expected | Disposition |
|---|---|---|---|---|
|  |  |  |  |  |

Header to record: machine / Windows version / app SHA / Anthropic key source / date.

---

## 5. Sign-off

- [ ] Scenario A — view switch + the four M07-IRL carry-forward fixes.
- [ ] Scenario B — drag-build a framework: nodes, edges, reject path, narrowing, live badges.
- [ ] Scenario C — Inspector, Validate, Canvas|JSON two-way binding, Save/Load round-trip.
- [ ] Scenario D — the Tester: isolated run, capability-violation → fail, discard-on-close.
- [ ] Scenario E — Settings + Novice→Promoted + the M06.5 🔴-1 MCP-registry re-confirm.
- [ ] Findings dispositioned; scope fence (§0) respected — no known-carried item logged as a bug.
