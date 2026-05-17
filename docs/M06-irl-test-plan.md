# M06 IRL Test Plan — Windows Runbook (cumulative M01–M06)

Manual verification of the running app as of the M06 merge. Default shell is **PowerShell** (Windows). Every fenced block is paste-as-is. Blocks that are NOT PowerShell are labelled in the line above them (the Playwright spec is a file to save; the DevTools snippets go in the app's console). Findings feed M07 Stage A.

---

## 0. Do NOT log these as bugs (scope fence)

v0.1's only live agent path is the no-tools smoke session. These are shipped + Stage-V-verified but unreachable by normal use until M07 — exercising them looks like a bug but is not: plan-approval, HITL/escalation, budget bar, recovery/uncertainty, GapPanel, capability-violation modal, tier-badge change, **an agent actually using an MCP tool end-to-end** (ADR-0011 trace-#11b M07 carry-forward), §5a alias/ambiguity at dispatch.

Live and testable in v0.1: first-run/key, the smoke session + its live graph render, the full MCP server-management UI + backend, persistence/restart, error surfacing.

---

## 1. Where each thing runs

| Context | What | How you know |
|---|---|---|
| **Window A** (PowerShell) | builds + runs the app; stays occupied | `npm run tauri dev` does not return |
| **Window B** (PowerShell) | every verify command; variables persist here | keep it open the whole run |
| **App GUI** | manual card actions (Settings → MCP Servers, etc.) | the running app window |
| **App DevTools** (F12 → Console) | the C3/C4 JS snippets | inside the app, not a terminal |
| **Playwright** | §3.3 UI spec | `npm run test:e2e`, own browser |

Nothing here needs bash. PowerShell only.

---

## 2. One-time prerequisites

`sqlite3` is not on Windows by default. Install once (skip if you already have it):

```powershell
winget install --id SQLite.SQLite -e --accept-source-agreements --accept-package-agreements
```

Confirm tooling:

```powershell
node -v; npx -v; (Get-Command sqlite3 -ErrorAction SilentlyContinue).Source
```

---

## 3. Setup

### Window A — build + run (this window is then occupied; do not paste anything else here)

```powershell
npm ci
cargo build --workspace
npm run tauri dev
```

### Window B — open a second PowerShell. Paste once. Keep this window open for the whole run (these variables only live in this session).

```powershell
$APPDATA_DIR = "$env:LOCALAPPDATA\dev.aria-runtime.app"
$AUDIT = "$APPDATA_DIR\skills.audit.jsonl"
$REG   = "$APPDATA_DIR\mcp_servers.sqlite"
$SCRATCH = (New-Item -ItemType Directory "$env:TEMP\m06irl-$(Get-Random)").FullName
"hello-irl" | Out-File -Encoding utf8 "$SCRATCH\probe.txt"
$SCRATCH
```

---

## 4. MCP — three layers

Read these before the UI cards: Layer A proves the real transport, Layer B proves the reference server itself is healthy (so a later failure is our client, not the server), Layer C proves the UI.

### Layer A — real rmcp transport (automated)

```powershell
cargo test -p runtime-mcp --features integration -- --nocapture
```

Expect: connects to a real `npx @modelcontextprotocol/server-filesystem`, lists tools incl. `read_file`/`write_file`, invokes one, disconnects — 0 failed. If it skips with an `npx`-not-found message that is an environment block, not a bug. A real transport error is 🔴.

Do not run `cargo llvm-cov` here — the Windows-local drone build-fail (gotcha #56) is known; coverage is CI-Linux-authoritative.

### Layer B — MCP Inspector (isolate "is the server healthy")

```powershell
npx @modelcontextprotocol/inspector npx -y "@modelcontextprotocol/server-filesystem" $SCRATCH
```

In the Inspector browser UI: list tools, invoke `read_file` on `probe.txt`, expect `hello-irl`. If this fails, the reference server is broken (environment) — do not attribute to our code. Close the Inspector before continuing.

### Layer C — renderer Playwright (UI render-contract; mocked backend)

This is a file, not a paste-in-terminal. Save exactly as `tests\e2e\m06_mcp_irl.spec.ts`. It extends the shipped `mcp_server_add.spec.ts` pattern (verified testids reused; four `CONFIRM` testids you align against the live components before running).

```ts
import { test, expect, type Page } from '@playwright/test';

async function dispatch(page: Page, events: unknown[]): Promise<void> {
  await page.evaluate((evts) => {
    const w = window as unknown as {
      __graphStore?: { getState: () => { applyEvent: (e: unknown) => void } };
    };
    if (!w.__graphStore) throw new Error('window.__graphStore missing (App.tsx affordance)');
    const s = w.__graphStore.getState();
    for (const e of evts as unknown[]) s.applyEvent(e);
  }, events);
}

async function resetMcpState(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as {
      __graphStore?: {
        getState: () => { clear: () => void };
        setState: (s: Record<string, unknown>) => void;
      };
    };
    w.__graphStore?.getState().clear();
    w.__graphStore?.setState({ currentMcpServers: {}, activeMcpCalls: {} });
  });
}

test.describe('M06 IRL UI render-contract', () => {
  test.describe.configure({ timeout: 120_000 });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await resetMcpState(page);
  });

  test('connected row has a real status color (gotcha 67)', async ({ page }) => {
    await dispatch(page, [
      { type: 'mcp_installed', name: 'fs-test', transport_kind: 'stdio', has_auth: false },
      { type: 'mcp_status_changed', name: 'fs-test', status: 'connected',
        tools: ['fs-test__read_file', 'fs-test__write_file'] },
    ]);
    const row = page.getByTestId('mcp-server-row-fs-test');
    await expect(row).toBeVisible();
    await expect(row).toHaveClass(/mcp-server-row--connected/);
    const bg = await page
      .getByTestId('mcp-status-indicator-fs-test')
      .evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(bg).not.toBe('rgba(0, 0, 0, 0)');
    expect(bg).not.toBe('transparent');
    const tools = page.getByTestId('mcp-tool-list-fs-test');
    await expect(tools).toContainText('read_file');
    await expect(tools).toContainText('write_file');
  });

  test('error status flips row class + indicator color', async ({ page }) => {
    await dispatch(page, [
      { type: 'mcp_installed', name: 'fs-test', transport_kind: 'stdio', has_auth: false },
      { type: 'mcp_status_changed', name: 'fs-test', status: 'error' },
    ]);
    await expect(page.getByTestId('mcp-server-row-fs-test')).toHaveClass(/mcp-server-row--error/);
    const bg = await page
      .getByTestId('mcp-status-indicator-fs-test')
      .evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(bg).not.toBe('rgba(0, 0, 0, 0)');
  });

  test('add modal opens; invalid name blocks submit', async ({ page }) => {
    await page.getByTestId('mcp-add-server-button').click();
    await expect(page.getByTestId('mcp-server-add-modal')).toBeVisible();
    await page.getByTestId('mcp-add-name').fill('Bad_Name!');
    await expect(page.getByTestId('mcp-add-submit')).toBeDisabled();
    await page.getByTestId('mcp-add-name').fill('fs-test');
    await expect(page.getByTestId('mcp-add-submit')).toBeEnabled();
  });

  test('empty state with no servers', async ({ page }) => {
    await expect(page.getByTestId('mcp-server-settings-empty')).toBeVisible();
  });
});
```

Before running, grep `src\components\MCPServerSettings.tsx`, `MCPServerAddModal.tsx`, `src\components\nodes\MCPNode.tsx`, `src\lib\graphStore.ts` and align these four if they differ: `mcp-status-indicator-fs-test`, `mcp-tool-list-fs-test`, `mcp-add-name`, `mcp-add-submit`, and the `mcp_status_changed` event name. The rest are verified from the shipped spec. Then, in Window B:

```powershell
npm run test:e2e -- m06_mcp_irl
```

---

## 5. Cards C1–C10

Each card: an action (App GUI / DevTools / PowerShell), then one PowerShell verify block in Window B, then expected + pass/fail.

### C1 — First-run key + persistence

Action (App GUI): remove the `agent-runtime` entry in Windows Credential Manager (Control Panel → Credential Manager → Windows Credentials), then in Window A stop and rerun `npm run tauri dev`. Enter a valid key on launch 1. Quit. Rerun launch 2.

Verify (Window B):

```powershell
cmdkey /list | Select-String agent-runtime
Get-ChildItem -Recurse $APPDATA_DIR -ErrorAction SilentlyContinue | Select-String -SimpleMatch (Read-Host 'paste first 6 chars of your key') 2>$null
```

Expect: credential entry present; the key-prefix search returns nothing (no plaintext on disk). Launch 2 shows no key prompt. PASS = no re-prompt + entry present + no disk hit. FAIL = re-prompt (stub backend, gotcha #29) or a disk hit.

### C2 — Live smoke session

Action (App GUI): trigger the smoke/run affordance. Watch the graph.

Verify: an agent node appears and token/event activity updates as the response streams. No PowerShell needed.

PASS = streamed render + node populates. FAIL = silent no-op / hang / node never populates.

### C3 — Graph fills the window (gotcha #70 — a real M04 bug)

Action: resize the app window large (≥1280) and small. Then App DevTools → F12 → Console, paste (this is JavaScript in the app console, not PowerShell):

```js
const c = document.querySelector('.react-flow'); const m = document.querySelector('main');
[c?.clientWidth, m?.clientWidth, window.innerWidth]
```

Expect: canvas width tracks window width (not pinned ~720). PASS = tracks + pan/zoom work. FAIL = narrow fixed column at large sizes.

### C4 — Non-blank node labels (gotcha #71)

Action: App DevTools Console (JavaScript):

```js
[...document.querySelectorAll('[data-testid^="rf__node"]')].map(n => n.textContent?.trim())
```

Expect: no entry empty / "untitled" / a raw uuid. PASS = all meaningful. FAIL = blank/untitled/raw-id.

### C5 — Error surfacing readable (gotcha #30/#31)

Action (App GUI): set an empty/bad key, trigger the smoke run. Window A shows the app's stdout.

Verify: the app UI shows a readable error (e.g. "authentication failed"), not `[object Object]`; Window A shows a structured log line containing `error`. PASS = readable UI + structured log. FAIL = `[object Object]` in UI or zero log output.

### C6 — Restart / recovery integrity (gotcha #15)

Action (App GUI): after a successful C2 run, hard-kill the app (close Window A with Ctrl+C / kill the process), relaunch `npm run tauri dev`.

Verify: prior session history re-renders and there is no second Anthropic call (no new streaming, token activity does not increase on restart).

PASS = history replays, zero re-execution. FAIL = blank state, or a second live call fires on restart.

### C7 — MCP add stdio + Test + status color + persist

Action (App GUI): Settings → MCP Servers → Add. Name `fs-test`, transport stdio, command `npx`, arguments `-y @modelcontextprotocol/server-filesystem` then the scratch path printed by the Window B setup block. Note the tier-eval shown before Confirm. Confirm. Click Test connection. Then quit and relaunch the app.

Verify (Window B):

```powershell
sqlite3 $REG "SELECT name,transport_type,status FROM mcp_servers;"
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_installed"' }
```

Expect: a `fs-test,stdio,...` row; one `mcp_installed` line naming `fs-test`. In the app: tool list shows `fs-test__read_file` etc.; the MCPNode connected indicator has a real color (run §3.3 spec for the computed-style proof). After relaunch `fs-test` is still listed.

PASS = tool list populated + colored indicator + survives restart. FAIL = empty tool list on a reachable server (gotcha #68), class but no color (gotcha #67), or gone after restart.

### C8 — MCP auth: keychain, never in audit (§13.5)

Action (App GUI): Add a server with an auth secret set to exactly `SENTINEL-9F2A`.

Verify (Window B):

```powershell
if (Select-String -SimpleMatch 'SENTINEL-9F2A' $AUDIT -ErrorAction SilentlyContinue) { 'LEAK - FAIL' } else { 'no leak - ok' }
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_(installed|auth_granted)"' }
cmdkey /list | Select-String agent-runtime
```

Expect: prints `no leak - ok`; `mcp_installed` then `mcp_auth_granted` in that order for the same name; a credential entry present. PASS = no sentinel anywhere + ordered lines + keychain entry. FAIL = sentinel found (🔴), missing order, or no keychain entry.

### C9 — MCP offline detection

Action (App GUI): with `fs-test` connected, kill the server subprocess in Window B, then wait one ping interval:

```powershell
Get-CimInstance Win32_Process -Filter "Name='node.exe'" | Where-Object { $_.CommandLine -like '*server-filesystem*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Start-Sleep -Seconds 35
```

Verify (Window B):

```powershell
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_missing"' }
sqlite3 $REG "SELECT name,status FROM mcp_servers WHERE name='fs-test';"
```

Expect: an `mcp_missing` line; status `error` or `disconnected`; the app's MCPNode flips to error. PASS = node→error + `mcp_missing` within ~1–2 intervals, no crash. FAIL = stuck connected / crash / hang.

### C10 — Add-modal validation + transport + tier

Action: run the §3.3 Playwright spec for name-validation; do the http transport + tier-eval display manually in the App GUI (switch transport to http, confirm the url field replaces command/args, confirm tier-eval shows before Confirm, add an http server).

Verify (Window B):

```powershell
npm run test:e2e -- m06_mcp_irl
sqlite3 $REG "SELECT name,transport_type FROM mcp_servers WHERE transport_type='http';"
```

Expect: spec green; an http row present after adding one. PASS = invalid name blocked + fields swap per transport + tier-eval shown + http persists as `http`. FAIL = any of those wrong.

---

## 6. Walkthroughs — follow the numbered steps in order

§5 above is reference detail. To actually run the test, ignore §5 and just do these steps top to bottom. Each step says where it runs in **[bold]**. If a code block is shown, paste it exactly into the window named in that step.

### Scenario A — Cold start → first run

**A1. [App / Windows]** Control Panel → Credential Manager → Windows Credentials. Delete any entry whose name contains `agent-runtime`.

**A2. [Window A]** If the app is running, press Ctrl+C. Then:

```powershell
npm run tauri dev
```

Wait for the app window to open.

**A3. [App]** At the key prompt, enter a valid Anthropic API key, submit. Expect: app proceeds to the main graph screen.

**A4. [App]** Close the app window completely.

**A5. [Window A]** Press Ctrl+C if needed, then:

```powershell
npm run tauri dev
```

Expect: NO key prompt this time. PASS if no prompt. FAIL if it asks again.

**A6. [App]** Trigger the run/smoke action. Expect: an agent node appears and text/token activity streams in. PASS if it renders. FAIL if nothing or hang.

**A7. [App]** Drag the window wider than ~1300px, then smaller.

**A8. [App → press F12 → Console tab]** Paste this JavaScript (not PowerShell), press Enter:

```js
const c=document.querySelector('.react-flow');const m=document.querySelector('main');[c?.clientWidth,m?.clientWidth,window.innerWidth]
```

Expect: first two numbers ≈ the third. PASS if canvas tracks window width. FAIL if first number stuck near 720 while the window is wide.

**A9. [App DevTools Console]** Paste this JavaScript, press Enter:

```js
[...document.querySelectorAll('[data-testid^="rf__node"]')].map(n=>n.textContent?.trim())
```

Expect: every string meaningful — none empty, none "untitled", none a raw id. PASS/FAIL accordingly.

**A10. [App / Windows]** Close the app. In Credential Manager delete the `agent-runtime` entry again.

**A11. [Window A]** Run `npm run tauri dev`. At the key prompt enter `bad-key`. In the app, trigger the run action.

**A12. [App + Window A]** Expect: app shows a readable error (e.g. "authentication failed"), NOT `[object Object]`; Window A shows a log line containing `error`. PASS = readable message + log line. FAIL = `[object Object]` or no log.

Scenario A done.

### Scenario B — MCP lifecycle

Before B1: the Window B setup block in §3 must already be pasted in this PowerShell session, and the app must be running with a valid key.

**B1. [Window B]**

```powershell
npx @modelcontextprotocol/inspector npx -y "@modelcontextprotocol/server-filesystem" $SCRATCH
```

A browser opens. In it: list tools, invoke `read_file` on `probe.txt`. Expect: returns `hello-irl`. Close that browser tab, then press Ctrl+C in Window B to stop the inspector. PASS = `hello-irl` returned.

**B2. [Window B]**

```powershell
cargo test -p runtime-mcp --features integration -- --nocapture
```

Expect: `test result: ok`, 0 failed. PASS = green. If it says npx not found = BLOCKED (environment), not a FAIL.

**B3. [Window B]** Show the scratch path you will type next:

```powershell
$SCRATCH
```

**B4. [App]** Settings → MCP Servers → Add. Name `fs-test`; Transport `stdio`; Command `npx`; Arguments: `-y @modelcontextprotocol/server-filesystem ` followed by the path B3 printed. Expect: a tier-eval message shows before Confirm. Click Confirm.

**B5. [App]** Click Test connection on the `fs-test` row. Expect: a tool list with `read_file`, `write_file`, etc.; the row indicator is colored (connected), not blank.

**B6. [Window B]**

```powershell
sqlite3 $REG "SELECT name,transport_type,status FROM mcp_servers;"
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_installed"' }
```

Expect: a `fs-test|stdio|...` row; one line mentioning `mcp_installed` and `fs-test`. PASS accordingly.

**B7. [App]** Add a second server: Name `auth-test`; any stdio command; set the Auth secret field to exactly `SENTINEL-9F2A`. Confirm.

**B8. [Window B]**

```powershell
if (Select-String -SimpleMatch 'SENTINEL-9F2A' $AUDIT -ErrorAction SilentlyContinue) { 'LEAK - FAIL' } else { 'no leak - ok' }
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_(installed|auth_granted)"' }
cmdkey /list | Select-String agent-runtime
```

Expect: prints `no leak - ok`; shows `mcp_installed` then `mcp_auth_granted` for `auth-test`; a credential entry listed. `LEAK - FAIL` = 🔴 STOP. PASS = no leak + ordered lines + entry.

**B9. [App]** Open the Add modal. Type name `Bad_Name!`. Expect: confirm button disabled. Change to `ok-name`. Expect: button enables. Switch Transport to `http`. Expect: command/args replaced by one URL field; tier-eval still shows. Cancel the modal (do not add).

**B10. [Window B]**

```powershell
Get-CimInstance Win32_Process -Filter "Name='node.exe'" | Where-Object { $_.CommandLine -like '*server-filesystem*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Start-Sleep -Seconds 35
```

**B11. [Window B]**

```powershell
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_missing"' }
sqlite3 $REG "SELECT name,status FROM mcp_servers WHERE name='fs-test';"
```

Expect: an `mcp_missing` line; `fs-test` status `error` or `disconnected`; in the app the `fs-test` node shows an error state. PASS accordingly.

**B12. [App, then Window A]** Close the app. In Window A run `npm run tauri dev`.

**B13. [App]** Settings → MCP Servers. Expect: `fs-test` and `auth-test` still listed (persisted). PASS = both present.

**B14. [App]** Remove `fs-test` (Remove button → confirm).

**B15. [Window B]**

```powershell
Get-Content $AUDIT -Tail 10 | Where-Object { $_ -match '"kind":"mcp_uninstalled"' }
```

Expect: an `mcp_uninstalled` line for `fs-test`.

**B16. [App, then Window A]** Close the app, relaunch `npm run tauri dev`, open Settings → MCP Servers. Expect: `fs-test` gone, `auth-test` still present. PASS accordingly.

Scenario B done.

### Scenario C — Restart integrity

**C-1. [App]** With a valid key, trigger the run/smoke action. Let it finish (text streamed, node rendered).

**C-2. [App]** Note that the run completed (rough token/event activity).

**C-3. [Window B]**

```powershell
$SCRATCH
```

**C-4. [App]** Settings → MCP Servers → Add: Name `persist-test`; stdio; Command `npx`; Arguments `-y @modelcontextprotocol/server-filesystem ` + the C-3 path. Confirm.

**C-5. [Window A]** Hard-stop the app: press Ctrl+C in Window A (or kill the app process).

**C-6. [Window A]**

```powershell
npm run tauri dev
```

**C-7. [App]** Expect: the previous session's graph/history re-renders WITHOUT a new run starting and with NO new streaming/token activity (replays, does not re-execute). PASS = history shown + no new API call. FAIL = blank, or a fresh run starts on its own.

**C-8. [App]** Settings → MCP Servers. Expect: `persist-test` still listed. PASS = present.

Scenario C done.

Hard gate: B8 must print `no leak - ok`. `LEAK - FAIL` is 🔴 — stop and log it before anything else.

---

## 7. Findings (feeds M07 Stage A)

🔴 fix before M07 · 🟡 M07 Stage A absorbs · 🟢 `docs/tech-debt.md` · BLOCKED = environment, not a bug.

| # | Card | Sev | Observed vs expected | Class / gotcha | Disposition |
|---|---|---|---|---|---|
|  |  |  |  |  |  |

Header to record: machine / Windows version / app SHA / Anthropic key source / date.

---

## 8. Sign-off

- [ ] Layer A green; Layer B healthy; §3.3 spec testids aligned + green.
- [ ] C1–C10 executed or BLOCKED-with-reason; Scenarios A/B/C walked.
- [ ] C8 prints `no leak - ok` (hard §13.5 gate).
- [ ] Findings dispositioned; 🔴 count = 0 to enter M07 Stage A without a fix cycle.
- [ ] Scope fence (§0) respected.
