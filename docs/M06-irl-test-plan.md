# M06 IRL Test Plan — Windows Runbook (cumulative M01–M06)

Manual verification of the running app as of the M06 merge. Default shell is **PowerShell**. Every fenced block is paste-as-is. Blocks that are NOT PowerShell are labelled in the line above them: **[DevTools JS]** = paste into the app's F12 → Console; the others are PowerShell. No mocks anywhere — real app, real backend, real renderer. Findings feed M07 Stage A.

---

## 0. Do NOT log these as bugs (scope fence)

v0.1's only live agent path is the no-tools smoke session. These are shipped + Stage-V-verified but have no live trigger until M07 — you will still *see* them in Scenario D (driven via the app's own DevTools store affordance, real components in the real window), but do NOT log "the agent didn't trigger this on its own" as a bug: plan-approval, HITL, budget, recovery/uncertainty, gap, capability-violation, and an agent actually *using* an MCP tool end-to-end (ADR-0011 trace-#11b, M07).

`RecoveryDialog` is not in this pass — it reads `localStorage` before the renderer mounts, so it cannot be driven post-load. It is Vitest-covered.

---

## 1. Where each thing runs

| Context | What |
|---|---|
| **Window A** (PowerShell) | builds + runs the app; stays occupied (`npm run tauri dev` never returns) |
| **Window B** (PowerShell) | every verify command; `$AUDIT`/`$REG`/`$SCRATCH` persist here — keep it open |
| **App** | manual actions + visual inspection in the running window |
| **App DevTools** (F12 → Console) | the `[DevTools JS]` snippets — real components, real window, injected trigger |

Three MCP layers (all real): **A** `cargo test --features integration` (real rmcp transport), **B** MCP Inspector (real reference server, isolates server-health), **manual** Scenario B (real app add/connect/persist/offline). Nothing here uses Playwright or any mock.

---

## 2. One-time prerequisites

`sqlite3` is not on Windows by default. Install once (skip if present):

```powershell
winget install --id SQLite.SQLite -e --accept-source-agreements --accept-package-agreements
```

Confirm tooling:

```powershell
node -v; npx -v; (Get-Command sqlite3 -ErrorAction SilentlyContinue).Source
```

---

## 3. Setup

### Window A — build + run (this window is then occupied; nothing else goes here)

```powershell
npm ci
cargo build --workspace
npm run tauri dev
```

### Window B — open a second PowerShell, paste once, keep it open all run (these variables only live in this session)

```powershell
$APPDATA_DIR = "$env:LOCALAPPDATA\dev.aria-runtime.app"
$AUDIT = "$APPDATA_DIR\skills.audit.jsonl"
$REG   = "$APPDATA_DIR\session.sqlite"
$SCRATCH = (New-Item -ItemType Directory "$env:TEMP\m06irl-$(Get-Random)").FullName
"hello-irl" | Out-File -Encoding utf8 "$SCRATCH\probe.txt"
$SCRATCH
```

There is no standalone MCP registry file — the `mcp_servers` table lives in the drone session DB (`session.sqlite`). Per IRL finding 🔴-1, `add_server` currently mis-resolves and writes to a stray `mcp.sqlite` instead; `$REG` points at the *correct* live DB so the B-card queries surface that divergence rather than hide it. See `docs/M06-irl-findings.md`.

---

## 4. Walkthroughs — follow the numbered steps in order

Each step is tagged with where it runs in **[bold]**. Paste the block shown into exactly that window. Run Scenario A, then B, then D, then C.

### Scenario A — Cold start → first run

**A1. [App / Windows]** Control Panel → Credential Manager → Windows Credentials. Delete any entry whose name contains `agent-runtime`.

**A2. [Window A]** If the app is running, Ctrl+C. Then:

```powershell
npm run tauri dev
```

Wait for the app window.

**A3. [App]** At the key prompt, enter a valid Anthropic key, submit. Expect: app reaches the main graph screen.

**A4. [App]** Close the app window completely.

**A5. [Window A]** Ctrl+C if needed, then:

```powershell
npm run tauri dev
```

Expect: NO key prompt. PASS = no prompt. FAIL = it asks again.

**A6. [App]** Trigger the run/smoke action. Expect: an agent node appears and text/token activity streams in. PASS = renders. FAIL = nothing/hang.

**A7. [App → F12 → Console]** **[DevTools JS]**:

```js
const c=document.querySelector('.react-flow');const m=document.querySelector('main');[c?.clientWidth,m?.clientWidth,window.innerWidth]
```

Expect: first two numbers ≈ the third. PASS = canvas tracks window. FAIL = first number stuck near 720 while window is wide (gotcha #70).

**A8. [App DevTools]** **[DevTools JS]**:

```js
[...document.querySelectorAll('[data-testid^="rf__node"]')].map(n=>n.textContent?.trim())
```

Expect: every string meaningful — none empty / "untitled" / a raw id (gotcha #71). PASS/FAIL accordingly.

**A9. [App / Windows]** Close the app. Delete the `agent-runtime` credential again.

**A10. [Window A]** `npm run tauri dev`. At the prompt enter `bad-key`. In the app, trigger the run.

**A11. [App + Window A]** Expect: a readable error (e.g. "authentication failed"), NOT `[object Object]`; Window A shows a log line containing `error`. PASS = readable + logged. FAIL = `[object Object]` or no log.

Scenario A done.

### Scenario B — MCP lifecycle (all real)

Window B setup (§3) already pasted; app running with a valid key.

**B1. [Window B]**

```powershell
npx @modelcontextprotocol/inspector npx -y "@modelcontextprotocol/server-filesystem" $SCRATCH
```

Browser opens. In it: list tools, invoke `read_file` on `probe.txt`. Expect: returns `hello-irl`. Close the tab; Ctrl+C in Window B. PASS = `hello-irl` returned (reference server healthy in isolation).

**B2. [Window B]**

```powershell
cargo test -p runtime-mcp --features integration -- --nocapture
```

Expect: `test result: ok`, 0 failed (real rmcp transport). npx-not-found = BLOCKED (environment), not FAIL.

**B3. [Window B]**

```powershell
$SCRATCH
```

**B4. [App]** Settings → MCP Servers → Add. Name `fs-test`; Transport `stdio`; Command `npx`; Arguments: `-y @modelcontextprotocol/server-filesystem ` then the B3 path. Expect: tier-eval message before Confirm. Click Confirm.

**B5. [App]** Click Test connection on the `fs-test` row. Expect: a tool list with `read_file`, `write_file`, etc.; the row indicator is colored (connected), not blank.

**B6. [App → F12 → Console]** **[DevTools JS]** — proves the indicator has a real CSS color, not just a class (gotcha #67):

```js
getComputedStyle(document.querySelector('[data-testid^="mcp-status-indicator"]')).backgroundColor
```

Expect: NOT `rgba(0, 0, 0, 0)` and NOT `transparent`. PASS = a real color. FAIL = transparent (class present, no CSS rule).

**B7. [Window B]**

```powershell
sqlite3 $REG "SELECT name,transport_type,status FROM mcp_servers;"
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_installed"' }
```

Expect: a `fs-test|stdio|...` row; one `mcp_installed` line naming `fs-test`. PASS accordingly.

**B8. [App]** Add a second server: Name `auth-test`; any stdio command; Auth secret exactly `SENTINEL-9F2A`. Confirm.

**B9. [Window B]**

```powershell
if (Select-String -SimpleMatch 'SENTINEL-9F2A' $AUDIT -ErrorAction SilentlyContinue) { 'LEAK - FAIL' } else { 'no leak - ok' }
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_(installed|auth_granted)"' }
cmdkey /list | Select-String agent-runtime
```

Expect: prints `no leak - ok`; `mcp_installed` then `mcp_auth_granted` for `auth-test`; a credential entry. `LEAK - FAIL` = 🔴 STOP. PASS = no leak + ordered + entry.

**B10. [App]** Open Add modal. Type name `Bad_Name!`. Expect: confirm disabled. Change to `ok-name`. Expect: enables. Switch Transport to `http`. Expect: command/args replaced by one URL field; tier-eval still shows. Cancel (do not add).

**B11. [Window B]**

```powershell
Get-CimInstance Win32_Process -Filter "Name='node.exe'" | Where-Object { $_.CommandLine -like '*server-filesystem*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Start-Sleep -Seconds 35
```

**B12. [Window B]**

```powershell
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_missing"' }
sqlite3 $REG "SELECT name,status FROM mcp_servers WHERE name='fs-test';"
```

Expect: an `mcp_missing` line; `fs-test` status `error`/`disconnected`; the app node shows error. PASS accordingly.

**B13. [App, then Window A]** Close the app. In Window A: `npm run tauri dev`.

**B14. [App]** Settings → MCP Servers. Expect: `fs-test` and `auth-test` still listed. PASS = both present.

**B15. [App]** Remove `fs-test` (Remove → confirm).

**B16. [Window B]**

```powershell
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_uninstalled"' }
```

Expect: an `mcp_uninstalled` line for `fs-test`.

**B17. [App, then Window A]** Close app, relaunch, open Settings → MCP Servers. Expect: `fs-test` gone, `auth-test` present. PASS accordingly.

Scenario B done.

### Scenario D — UI sweep (real app: visual + interaction)

App running, valid key, on the main screen. The `[DevTools JS]` snippets drive the **real components in the real window** via the app's own store affordance — only the trigger is injected. After each panel: look (visual) + do (interaction) + the stated expect. Each snippet self-resets first.

**D1. [App]** Drag the window wide (≥1300px) then narrow. Look: graph canvas fills width at both sizes; no fixed ~720 column; header/panels not clipped or overlapping. Do: drag-pan the canvas; wheel-zoom. PASS = fills + pan/zoom work. FAIL = narrow column / clipping / dead pan-zoom (gotcha #70).

**D2. [App]** Trigger a smoke run. Look: agent node label is meaningful (not blank/"untitled"/uuid), node not clipped, activity updates live (gotcha #71).

**D3. [App → F12 → Console] [DevTools JS]** — Approval panel + PlanNode:

```js
{const s=window.__graphStore.getState();s.clear();s.applyEvent({type:'plan_created',plan_id:'p1',title:'Refactor auth flow',task_count:3,approval_required:true});s.applyEvent({type:'plan_approval_requested',plan_id:'p1'});}
```

Look: an Approval panel showing "Refactor auth flow" + task count, buttons visibly styled (not raw text); a PlanNode with a status style. Do: hover/click an approve control — it should visibly react. Then resolve:

```js
window.__graphStore.getState().applyEvent({type:'plan_approved',plan_id:'p1',approved_by:'user'})
```

Look: Approval panel disappears; PlanNode status visibly changes. PASS = renders + dismisses on resolve. FAIL = unstyled / does not dismiss.

**D4. [App DevTools] [DevTools JS]** — HITL panel:

```js
{const s=window.__graphStore.getState();s.clear();s.applyEvent({type:'hitl_requested',prompt_id:'p-f',trigger:'on_failure_threshold',agent_id:null,question:'Task t-1 exceeded failure budget after 3 attempts. Retry, skip, or abort?',options:['retry','skip','abort'],ui_variant:'panel',timeout_at_unix_ms:9000000000000});}
```

Look: HITL panel, question wraps readably, three styled buttons (retry/skip/abort). Do: press `Escape` — panel dismisses locally. PASS = readable + 3 buttons + Escape dismisses.

**D5. [App DevTools] [DevTools JS]** — HITL modal (risky tool):

```js
{const s=window.__graphStore.getState();s.clear();s.applyEvent({type:'hitl_requested',prompt_id:'p-r',trigger:'on_risky_tool',agent_id:'agent-1',question:'Run Bash:rm -rf /tmp/foo?',options:['allow','block'],ui_variant:'modal',timeout_at_unix_ms:9000000000000});}
```

Look: a true modal — dimmed/overlaid background, centered, the command text visible. Do: press `Tab` repeatedly — focus stays trapped inside the modal. PASS = modal chrome (not an inline panel) + focus trap. FAIL = renders as a plain inline block / no overlay.

**D6. [App DevTools] [DevTools JS]** — HITL toast:

```js
{const s=window.__graphStore.getState();s.clear();s.applyEvent({type:'hitl_requested',prompt_id:'p-t',trigger:'per_task',agent_id:null,question:'Approve next task?',options:['ok','skip'],ui_variant:'toast',timeout_at_unix_ms:9000000000000});}
```

Look: a small corner toast that does NOT dim the screen, summary visible. PASS = non-blocking toast styling. FAIL = it blocks/centers like a modal.

**D7. [App DevTools] [DevTools JS]** — Budget bar through all four states (color/badge must visibly differ — gotcha #67):

```js
{const s=window.__graphStore.getState();s.clear();s.applyEvent({type:'budget_warn',spent_usd:2.5,cap_usd:5.0,percent:50});}
```

Look: header bar appears, warn color, shows `$2.50` / `$5.00`. Do: click the bar — a settings panel opens. Then run each next line, looking for a visibly distinct state each time:

```js
window.__graphStore.getState().applyEvent({type:'budget_downshift',from_model:'claude-opus-4-7',to_model:'claude-sonnet-4-6',reason:'budget_threshold'})
```
```js
window.__graphStore.getState().applyEvent({type:'budget_suspended',spent_usd:4.5,cap_usd:5.0})
```
```js
window.__graphStore.getState().applyEvent({type:'budget_exceeded',spent_usd:5.0,cap_usd:5.0})
```

Look: downshift badge → suspended badge → exceeded banner "Session terminated", each a visibly different color/badge (not just changed text). PASS = four distinct visual states + click opens settings. FAIL = identical-looking states (missing CSS, gotcha #67).

**D8. [App DevTools] [DevTools JS]** — Gap panel:

```js
{const s=window.__graphStore.getState();s.clear();s.applyEvent({type:'tool_missing',agent_id:'worker',tool_name:'fetch_prs',severity:'critical',suggested_action:"Install tool 'fetch_prs' and click Resume.",requested_via:'loader'});}
```

Look: Gap panel with critical-severity styling (color), tool name + suggested action readable, a Resume affordance. Then resolve:

```js
window.__graphStore.getState().applyEvent({type:'gap_resolved',agent_id:'worker',kind:'tool',capability:'fetch_prs'})
```

Look: panel dismisses. PASS = severity-colored + readable + dismisses.

**D9. [App DevTools] [DevTools JS]** — Capability-violation modal (ADR-0007 reuse of the HITL modal — should look identical to D5's chrome):

```js
{const s=window.__graphStore.getState();s.clear();s.applyEvent({type:'hitl_requested',prompt_id:'p-cv',trigger:'on_capability_violation',agent_id:'worker',question:"Agent worker requested capability 'write' on /etc — not in granted scope. Allow once, deny, or abort?",options:['allow_once','deny','abort'],ui_variant:'modal',timeout_at_unix_ms:9000000000000});}
```

Look: modal with the violation text + three buttons; same modal chrome as D5 (consistency). PASS = consistent modal styling + violation text. FAIL = different/broken chrome vs D5.

**D10. [App DevTools] [DevTools JS]** — Uncertainty prompt (note: `recordUncertainInvocation`, not `applyEvent`):

```js
{const s=window.__graphStore.getState();s.clear();s.recordUncertainInvocation({invocationId:'sig-tool-1',toolName:'Read',agentId:'a1'});}
```

Look: a modal dialog showing invocation id `sig-tool-1` and four actions (retry/skip/mark/abort) visibly styled. Then add more:

```js
{const s=window.__graphStore.getState();s.recordUncertainInvocation({invocationId:'sig-2'});s.recordUncertainInvocation({invocationId:'sig-3'});}
```

Look: a remaining-count indicator (e.g. "2 more"). PASS = dialog + 4 actions + remaining count. FAIL = missing actions / no count.

**D11. [App DevTools] [DevTools JS]** — cleanup:

```js
window.__graphStore.getState().clear()
```

Look: all injected panels gone, main screen normal (no stuck overlay). PASS = clean return.

Scenario D done.

### Scenario C — Restart integrity

**C-1. [App]** With a valid key, trigger the run/smoke action; let it finish.

**C-2. [Window B]**

```powershell
$SCRATCH
```

**C-3. [App]** Settings → MCP Servers → Add: Name `persist-test`; stdio; Command `npx`; Arguments `-y @modelcontextprotocol/server-filesystem ` + the C-2 path. Confirm.

**C-4. [Window A]** Hard-stop: Ctrl+C in Window A (or kill the app process).

**C-5. [Window A]**

```powershell
npm run tauri dev
```

**C-6. [App]** v0.1 starts a **fresh session** on relaunch — there is no auto history-replay (resume is the RecoveryDialog path, out of scope per §0). A blank graph here is **expected, not a finding**. The valid integrity check is drone-DB durability, not UI replay.

**C-7. [Window B]** Append-only durability across restart — did prior data survive:

```powershell
sqlite3 $REG "SELECT (SELECT count(*) FROM signals) AS sig,(SELECT count(*) FROM token_usage) AS tok,(SELECT count(*) FROM heartbeats) AS hb,(SELECT count(*) FROM snapshots) AS snap;"
```

Expect: `hb` and `snap` > 0 (drone persisted across restart). Per IRL finding 🔴-2, `sig`/`tok` are currently `0` after successful smoke runs while `hb`/`snap` populate — that is the blocking signal-persistence defect, not a test artifact. PASS = hb/snap > 0; the sig/tok=0 result is the logged 🔴-2 (see `docs/M06-irl-findings.md`), not re-discovered here.

Scenario C done.

Hard gate: B9 must print `no leak - ok`. `LEAK - FAIL` is 🔴 — stop and log before anything else.

---

## 5. Findings (feeds M07 Stage A)

🔴 fix before M07 · 🟡 M07 Stage A absorbs · 🟢 `docs/tech-debt.md` · BLOCKED = environment, not a bug.

| # | Step | Sev | Observed vs expected | Class / gotcha | Disposition |
|---|---|---|---|---|---|
|  |  |  |  |  |  |

Header to record: machine / Windows version / app SHA / Anthropic key source / date.

---

## 6. Sign-off

- [ ] Scenario A (cold start) walked.
- [ ] Scenario B (MCP, all real): Inspector healthy + `cargo --features integration` green + add/test/persist/offline/remove + **B9 prints `no leak - ok`** (hard §13.5 gate).
- [ ] Scenario D (UI sweep): D1–D11 — every panel eyeballed for visual + interaction.
- [ ] Scenario C (restart integrity).
- [ ] Findings dispositioned; 🔴 count = 0 to enter M07 Stage A without a fix cycle.
- [ ] Scope fence (§0) respected — RecoveryDialog correctly out of scope; no shipped-but-M07 surface logged as a bug.
