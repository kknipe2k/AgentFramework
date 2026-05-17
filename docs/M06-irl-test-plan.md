# M06 IRL Test Plan — Whole-App Manual Verification (cumulative M01–M06)

> **Purpose.** Executable manual verification of the *running* app as of the M06 merge (`081044a`+). Every card carries runnable commands or literal expected output — not prose. Findings feed **M07 Stage A** (🔴 fix-first / 🟡 M07.A absorbs / 🟢 `docs/tech-debt.md`), mirroring `docs/M04-irl-test-plan.md` → M05.A.

---

## 0. Scope fence (read first — prevents chasing non-bugs)

v0.1's only live agentic path is `run_smoke_session` (hardcoded no-tools "say hello" — emits no `ProviderEvent::ToolUse`). The following are **shipped + Stage-V-verified but NOT reachable by normal app use until M07**; do **not** log them as bugs:

- Plan-approval / HITL-escalation / budget-bar / recovery+uncertainty / GapPanel / capability-violation-modal / tier-badge-change → need M07's plan + tool loop (M04/M05 Stage-V verified).
- **Agent *using* an MCP tool end-to-end** → concrete `McpDispatcher` construction + `ConnectionResolver for McpClient` + live loop = ADR-0011 trace-**#11b M07 carry-forward**. M06 wires the SDK seam only (M06.V #11a mock-verified).
- §5a alias/ambiguity *at dispatch* → dispatch-path, M06.V #6 🟡→M07.A.

**Live + user-exercisable in v0.1 (what this plan tests):** first-run/key, the no-tools smoke + its live graph render, the full MCP server-management UI + backend, persistence/restart, error-surfacing + dev-logging.

---

## 1. Automation reality matrix (honest — what each layer can and cannot do)

| Layer | Drives | Tests | Does NOT test | Command |
|---|---|---|---|---|
| **A. cargo integration** | real rmcp transport, real subprocess, real SQLite/keychain | the actual MCP client/lifecycle/dispatch backend | the UI; the real Tauri window | `cargo test -p runtime-mcp --features integration` |
| **B. MCP Inspector** | the reference MCP server alone | "is the server itself healthy" (isolates server-ok from our-client-ok) | our app at all | `npx @modelcontextprotocol/inspector npx -y @modelcontextprotocol/server-filesystem <dir>` |
| **C. renderer Playwright** | Vite dev server, `@tauri-apps/api` **mocked**, `window.__graphStore` injection | MCP UI wiring + store reducers + render-contract (incl. computed-style, gotcha #67) | the real rmcp transport; the real Tauri window | `npm run test:e2e -- m06_mcp_irl` (spec in §3.3) |
| **D. manual (real Tauri window)** | the packaged app | true end-to-end (keychain, audit file, restart) | — (no automation: gotcha #23 — `tauri-driver` disabled, no macOS) | walkthrough §4–§5 |

**Why no full-stack Playwright:** Playwright cannot drive a Tauri 2.x WebView2/WebKitGTK window (gotcha #23; `e2e-tauri-driver` job DISABLED, M03 carry-forward). The renderer spec runs against Vite with the Tauri API mocked — it proves UI logic, not transport. Real transport = Layer A. True end-to-end = Layer D (manual).

---

## 2. Environment setup

```bash
# 2.1 Build + run the app (Layer D)
npm ci && cargo build --workspace
npm run tauri dev          # or run the packaged binary

# 2.2 App-local-data dir (bundle id = dev.aria-runtime.app)
#   Windows : %LOCALAPPDATA%\dev.aria-runtime.app\
#   Linux   : ~/.local/share/dev.aria-runtime.app/
#   macOS   : ~/Library/Application Support/dev.aria-runtime.app/
# Export it for the verify commands below:
export APPDATA_DIR="$HOME/.local/share/dev.aria-runtime.app"   # adjust per OS
export AUDIT="$APPDATA_DIR/skills.audit.jsonl"
export REG="$APPDATA_DIR/mcp_servers.sqlite"

# 2.3 Tooling sanity
node -v && npx -v
npx -y @modelcontextprotocol/server-filesystem --help 2>&1 | head -1   # reference server reachable
which sqlite3 jq                                                       # for verify cmds

# 2.4 Scratch dir the filesystem MCP server will expose
export SCRATCH="$(mktemp -d)"; echo "hello-irl" > "$SCRATCH/probe.txt"

# 2.5 Keychain service names: API key = "agent-runtime"; MCP secrets = "agent-runtime/mcp"
#   Windows: Credential Manager → Windows Credentials
#   Linux  : secret-tool search service agent-runtime
#   macOS  : security find-generic-password -s agent-runtime
```

Record machine / OS / app SHA / Anthropic-key-source in the Findings header.

---

## 3. MCP — three runnable layers

### 3.1 Layer A — real rmcp transport (automated, precise)

```bash
# Real reference-server smoke (Stage B's feature-gated integration suite —
# spawns `npx @modelcontextprotocol/server-filesystem`, real JSON-RPC stdio):
cargo test -p runtime-mcp --features integration -- --nocapture

# The full client/lifecycle/dispatch/namespace/auth/registry suite:
cargo test -p runtime-mcp --features test-helpers

# Coverage gate parity (clean first per gotcha #81):
cargo llvm-cov clean --workspace
cargo llvm-cov -p runtime-mcp --features test-helpers \
  --ignore-filename-regex 'src.main\.rs|generated|src.lib\.rs|src.transport.stdio\.rs|src.transport.http\.rs|src.client.auth_keyring\.rs|src.client.lifecycle\.rs' \
  --fail-under-lines 95
```
**Expected:** `integration.rs` connects → `list_tools` includes `read_file`/`write_file`/`list_directory` → invokes `read_file` on a tempfile → disconnects, all green. Suite 0 failed. Coverage ≥95% line, exit 0.
**Pass/Fail:** PASS = integration test green (proves the real rmcp stdio path works). FAIL/skip-with-message = `npx` unavailable (environment, not a bug — log BLOCKED) OR a real transport error (🔴).

### 3.2 Layer B — MCP Inspector (isolate "is the server ok")

Run **before** blaming the app — proves the reference server itself is healthy, so a failure in §3.1/§4 is *our client*, not the server:

```bash
npx @modelcontextprotocol/inspector npx -y @modelcontextprotocol/server-filesystem "$SCRATCH"
# Opens the Inspector UI: list tools, invoke read_file on $SCRATCH/probe.txt,
# confirm "hello-irl" returns. No LLM involved.
```
**Expected:** Inspector lists the filesystem tools and `read_file` returns `hello-irl`. **Pass/Fail:** PASS = server healthy in isolation. FAIL = the *reference server* is broken (environment) — do not attribute to our code.

### 3.3 Layer C — renderer Playwright spec (UI wiring + render-contract)

Save as `tests/e2e/m06_mcp_irl.spec.ts`, align the 4 `// CONFIRM` testids to the live components, run `npm run test:e2e -- m06_mcp_irl`. **Not committed as a CI gate** — IRL specs align testids against the components in front of you at execution (the shipped `mcp_server_add.spec.ts` already covers the verified-testid surface in CI). This spec extends that exact pattern (state-injection → render-contract; the `invoke` linkage is Vitest-covered per the shipped spec's own caveat).

```ts
import { test, expect, type Page } from '@playwright/test';

// IRL companion to tests/e2e/mcp_server_add.spec.ts. Renderer-level only:
// Vite dev server, @tauri-apps/api NOT reachable (mocking across the ESM
// boundary doesn't work in Playwright — shipped-spec caveat). Asserts the
// state-injection → render contract incl. computed-style (gotcha #67, the
// M04 "class present but no CSS rule" IRL bug class). Real transport =
// `cargo test -p runtime-mcp --features integration` (Layer A).

interface McpInstalled {
  type: 'mcp_installed';
  name: string;
  transport_kind: 'stdio' | 'http';
  has_auth: boolean;
}
interface McpStatus {
  type: 'mcp_status_changed';        // CONFIRM event name vs graphStore.ts applyEvent
  name: string;
  status: 'connected' | 'disconnected' | 'health_pending' | 'error';
  tools?: string[];
}

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

test.describe('M06 IRL — MCP UI render-contract', () => {
  test.describe.configure({ timeout: 120_000 });   // gotcha #53 Vite cold-start

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await resetMcpState(page);
  });

  test('C7-ui: connected server row renders with the right STATUS COLOR (gotcha #67)', async ({ page }) => {
    await dispatch(page, [
      { type: 'mcp_installed', name: 'fs-test', transport_kind: 'stdio', has_auth: false } as McpInstalled,
      { type: 'mcp_status_changed', name: 'fs-test', status: 'connected',
        tools: ['fs-test__read_file', 'fs-test__write_file'] } as McpStatus,
    ]);
    const row = page.getByTestId('mcp-server-row-fs-test');           // shipped testid
    await expect(row).toBeVisible();
    await expect(row).toHaveClass(/mcp-server-row--connected/);       // shipped class
    // gotcha #67: class present is NOT enough — assert the rule exists (real color):
    const indicator = page.getByTestId('mcp-status-indicator-fs-test'); // CONFIRM testid
    const bg = await indicator.evaluate(
      (el) => getComputedStyle(el).backgroundColor,
    );
    expect(bg).not.toBe('rgba(0, 0, 0, 0)');                          // not transparent
    expect(bg).not.toBe('transparent');
    // tool list populated from the discovered tools (gotcha #68 wrong-field):
    const tools = page.getByTestId('mcp-tool-list-fs-test');          // CONFIRM testid
    await expect(tools).toContainText('read_file');
    await expect(tools).toContainText('write_file');
  });

  test('C9-ui: error status flips the row class + indicator color', async ({ page }) => {
    await dispatch(page, [
      { type: 'mcp_installed', name: 'fs-test', transport_kind: 'stdio', has_auth: false } as McpInstalled,
      { type: 'mcp_status_changed', name: 'fs-test', status: 'error' } as McpStatus,
    ]);
    const row = page.getByTestId('mcp-server-row-fs-test');
    await expect(row).toHaveClass(/mcp-server-row--error/);
    const bg = await page
      .getByTestId('mcp-status-indicator-fs-test')                    // CONFIRM testid
      .evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(bg).not.toBe('rgba(0, 0, 0, 0)');
  });

  test('C10-ui: add-modal opens; invalid name blocks submit', async ({ page }) => {
    await page.getByTestId('mcp-add-server-button').click();          // shipped testid
    await expect(page.getByTestId('mcp-server-add-modal')).toBeVisible(); // shipped testid
    await page.getByTestId('mcp-add-name').fill('Bad_Name!');         // CONFIRM testid
    await expect(page.getByTestId('mcp-add-submit')).toBeDisabled();  // CONFIRM testid
    await page.getByTestId('mcp-add-name').fill('fs-test');
    await expect(page.getByTestId('mcp-add-submit')).toBeEnabled();
  });

  test('empty state when no servers (shipped-pattern regression guard)', async ({ page }) => {
    await expect(page.getByTestId('mcp-server-settings-empty')).toBeVisible(); // shipped testid
  });
});
```
`// CONFIRM` testids (`mcp-status-indicator-<name>`, `mcp-tool-list-<name>`, `mcp-add-name`, `mcp-add-submit`, the `mcp_status_changed` event name) — grep `src/components/MCPServerSettings.tsx`, `MCPServerAddModal.tsx`, `src/components/nodes/MCPNode.tsx`, `src/lib/graphStore.ts` and align. The shipped ones (`mcp-server-row-<name>`, `mcp-add-server-button`, `mcp-server-add-modal`, `mcp-server-settings-empty`, classes `mcp-server-row--{connected,error}`) are verified from `mcp_server_add.spec.ts`.

---

## 4. Per-card executable procedures (C1–C10)

> **Action** (CLI / Playwright / manual) → **Verify** (copy-paste cmd + literal expected) → **Pass/Fail** → **Targets**.

### C1 — First-run key + persistence (M02; gotcha #29)
- **Action.** Clear the key, relaunch twice.
  ```bash
  # Linux: secret-tool clear service agent-runtime ; Win: Credential Manager → remove "agent-runtime"
  npm run tauri dev    # launch 1: enter a valid key, proceed
  # quit, then:
  npm run tauri dev    # launch 2
  ```
- **Verify.** Launch 2 shows no key prompt. Confirm backend, not the silent stub (gotcha #29):
  ```bash
  cargo test -p runtime-main key_store -- --nocapture        # key_store_with seam green
  # Linux: secret-tool search service agent-runtime          # entry EXISTS
  ```
- **Pass/Fail.** PASS = no re-prompt + keychain entry exists + no plaintext key on disk (`grep -r "<key-prefix>" "$APPDATA_DIR"` → no hits). FAIL = re-prompt (stub backend) or key on disk.
- **Targets.** gotcha #29; M02 keychain.

### C2 — Live smoke session end-to-end (M02 — the one live path)
- **Action.** Trigger the smoke/run affordance in the running app.
- **Verify.**
  ```bash
  # signals landed in the drone DB:
  sqlite3 "$APPDATA_DIR"/*.sqlite "SELECT type,count(*) FROM signals GROUP BY type;" 2>/dev/null
  ```
  UI: an agent node appears + token/event activity updates as the stream arrives.
- **Pass/Fail.** PASS = streamed render + node populates + signals rows present. FAIL = silent no-op / hang / node never populates.
- **Targets.** M02 pipeline; gotcha #66 (render reflects the stream, not "call returned ok").

### C3 — Graph fills the window (M03; gotcha #70 — real M04 bug)
- **Action.** Post-C2, resize window ≥1280px and small.
- **Verify (DevTools console in the running app):**
  ```js
  const c = document.querySelector('.react-flow'); const m = document.querySelector('main');
  [c?.clientWidth, m?.clientWidth, window.innerWidth]   // canvas ≈ window, not pinned ~720
  ```
- **Pass/Fail.** PASS = canvas tracks window; pan/zoom work. FAIL = pinned narrow column at large sizes.
- **Targets.** gotcha #70 (the literal M04 IRL bug).

### C4 — Non-blank node labels (M03/M04; gotcha #71)
- **Action.** Inspect every node from C2.
- **Verify (console):**
  ```js
  [...document.querySelectorAll('[data-testid^="rf__node"]')].map(n => n.textContent?.trim())
  // none empty / "untitled" / a raw uuid where a name belongs
  ```
- **Pass/Fail.** PASS = all meaningful. FAIL = blank/"untitled"/raw-id.
- **Targets.** gotcha #71 (M04 IRL class).

### C5 — Error surfacing readable (M02; gotcha #30/#31)
- **Action.** Set an empty/bad key, trigger smoke.
- **Verify.** UI shows a readable error (e.g., "authentication failed"); stdout/log shows a structured line:
  ```bash
  npm run tauri dev 2>&1 | grep -iE 'error|auth' | head    # structured tracing line present
  ```
- **Pass/Fail.** PASS = readable UI msg + structured log. FAIL = `"[object Object]"` in UI, or zero log from the command.
- **Targets.** gotcha #30 (`unwrapCmdError`); #31 (tracing init).

### C6 — Restart / recovery integrity (M01/M04; gotcha #15/#69)
- **Action.** After C2, hard-kill + relaunch.
- **Verify.** History re-renders; **no second Anthropic call** — token usage unchanged:
  ```bash
  sqlite3 "$APPDATA_DIR"/*.sqlite "SELECT count(*) FROM signals;"   # same count pre/post restart
  ```
- **Pass/Fail.** PASS = replayed, zero re-execution. FAIL = blank state OR signal count grows on restart (re-executed).
- **Targets.** gotcha #15 (resume rebuilds, not re-executes); #69 (post-restart drone IPC multi-call).

### C7 — MCP add stdio + Test + status-color + persist (M06)
- **Action (manual, Layer D).** Settings → MCP Servers → Add: name `fs-test`, stdio, command `npx`, args `-y @modelcontextprotocol/server-filesystem $SCRATCH`. Note tier-eval pre-Confirm. Confirm → Test connection. Then quit + relaunch.
- **Verify.**
  ```bash
  sqlite3 "$REG" "SELECT name,transport_type,status FROM mcp_servers;"   # fs-test,stdio,<status>
  jq -c 'select(.kind=="mcp_installed")' "$AUDIT" | tail -1              # one line, name=fs-test
  ```
  UI: tool list shows `fs-test__read_file` etc.; MCPNode connected indicator has a real color (run §3.3 `C7-ui`). After relaunch `fs-test` still listed.
- **Pass/Fail.** PASS = tool list populated + real-color indicator + survives restart (registry row persists). FAIL = empty tool list on reachable server (gotcha #68), class-without-color (gotcha #67), or gone after restart.
- **Targets.** #67, #68; Stage C registry/SQLite; §5a canonical names.

### C8 — MCP auth: keychain, never in audit (M06; §13.5)
- **Action (manual).** Add a server with an `auth_secret` (use a recognizable sentinel, e.g. `SENTINEL-9F2A`).
- **Verify.**
  ```bash
  grep -F 'SENTINEL-9F2A' "$AUDIT" && echo "LEAK 🔴" || echo "no leak ✓"
  jq -rc 'select(.kind|test("mcp_installed|mcp_auth_granted")) | [.kind,.name] | @csv' "$AUDIT" | tail -2
  # expect: mcp_installed THEN mcp_auth_granted, same name, in order
  # Linux: secret-tool search service agent-runtime/mcp        # entry exists
  ```
- **Pass/Fail.** PASS = sentinel absent everywhere in `$AUDIT`/logs, keychain entry present, ordered correlated lines. FAIL = sentinel anywhere in audit/log (🔴), or no keychain entry (#29).
- **Targets.** §13.5; #66 ordering; #29.

### C9 — MCP offline detection (M06 health-ping)
- **Action (manual).** With `fs-test` connected, kill the server subprocess:
  ```bash
  pkill -f 'server-filesystem' ; sleep 35   # > one 30s ping interval
  ```
- **Verify.**
  ```bash
  jq -c 'select(.kind=="mcp_missing")' "$AUDIT" | tail -1   # mcp_missing emitted
  sqlite3 "$REG" "SELECT name,status FROM mcp_servers WHERE name='fs-test';"  # error/disconnected
  ```
  UI: MCPNode flips to error (run §3.3 `C9-ui` for the render-contract).
- **Pass/Fail.** PASS = node→error + `mcp_missing` within ~1–2 intervals, no crash. FAIL = stuck "connected" / crash / hang.
- **Targets.** Stage C lifecycle; reuse of `mcp_missing`/`on_gap`.

### C10 — Add-modal validation + transport + tier (M06 Stage E)
- **Action.** Playwright `C10-ui` (§3.3) for name-validation + modal; manual for tier-eval display + http url-field swap + http persist.
- **Verify.**
  ```bash
  npm run test:e2e -- m06_mcp_irl -g C10-ui          # invalid name blocks submit
  sqlite3 "$REG" "SELECT name,transport_type FROM mcp_servers WHERE transport_type='http';"
  ```
- **Pass/Fail.** PASS = invalid name blocked, fields swap per transport, tier-eval shown pre-Confirm, http persists as `http`. FAIL = any of those wrong.
- **Targets.** Stage E validation/transport-conditional fields; #67; M05.D tier.

---

## 5. Scenarios (sequenced)

- **A — Cold start → first run** (C1→C5): clear key → launch → enter key → smoke → resize/inspect labels → bad key → readable error.
- **B — MCP lifecycle + audit + persistence** (Layer B sanity → C7 → C8 → C10 → C9 → restart-persist → remove):
  1. `npx @modelcontextprotocol/inspector npx -y @modelcontextprotocol/server-filesystem "$SCRATCH"` (server healthy in isolation).
  2. `cargo test -p runtime-mcp --features integration` (real transport green — Layer A).
  3. App: add `fs-test` (C7), Test, check `$REG`/`$AUDIT`.
  4. Add auth server with `SENTINEL-9F2A` (C8) — leak grep MUST be empty.
  5. Invalid-name + http (C10).
  6. `pkill -f server-filesystem` → offline (C9).
  7. Relaunch → servers persist; Remove `fs-test` → `jq 'select(.kind=="mcp_uninstalled")' "$AUDIT"` present, gone after relaunch.
- **C — Restart/recovery integrity** (C6 + cross-cut): smoke run + add a server → record `signals` count → hard-restart → count unchanged (no re-exec), server still listed.

---

## 6. Findings Log (feeds M07 Stage A)

> 🔴 fix before M07.A · 🟡 M07.A absorbs · 🟢 `docs/tech-debt.md`. BLOCKED (env, not a bug) is distinct from FAIL.

| # | Card | Sev | Observed vs expected | Class / gotcha | Disposition |
|---|---|---|---|---|---|
| _(fill on execution)_ | | | | | |

**Header:** machine / OS / app SHA / key source / date.

## 7. Sign-off

- [ ] Layer A green (`cargo test -p runtime-mcp --features integration`).
- [ ] Layer B done (Inspector — reference server healthy in isolation).
- [ ] §3.3 spec saved, `// CONFIRM` testids aligned, run green.
- [ ] C1–C10 executed or BLOCKED-with-reason; Scenarios A/B/C walked.
- [ ] C8 sentinel-leak grep empty (hard requirement — §13.5).
- [ ] Findings dispositioned; 🔴 count = 0 to enter M07.A without a fix cycle.
- [ ] Scope fence (§0) respected — no shipped-but-M07 surface logged as a bug.

> IRL verifies the lived experience the Stage-V contract pass + automated gates structurally cannot (the gotcha #66/#67/#68/#70/#71 classes). Findings carry into M07 Stage A per the M04-IRL → M05.A precedent.
