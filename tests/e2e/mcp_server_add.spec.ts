import { test, expect, type Page } from '@playwright/test';

// Renderer-level Playwright covering the M06.E MCPServerSettings wire
// path. tauri-driver E2E remains disabled per M03 PR #47 carry-forward;
// this spec runs against the Vite dev server using `window.__graphStore`
// (App.tsx affordance) to drive store state via `page.evaluate` per
// gotcha #54.
//
// Module mocking across the @tauri-apps/api ESM boundary does NOT work in
// Playwright (only Vitest's vi.mock) — so the Add-submit → invoke linkage
// is covered by the Vitest suites. This spec asserts the state-injection
// → Settings-row-render contract that only renders correctly inside a
// real browser layout (the E.4.4 phase-doc pseudocode referenced a
// `[data-test=open-settings]` tab that does not exist — App.tsx mounts
// panels directly into `.graph-layout`; reconciled against actual DOM).

interface McpInstalled {
  type: 'mcp_installed';
  name: string;
  transport_kind: 'stdio' | 'http';
  has_auth: boolean;
}

async function dispatch(page: Page, events: McpInstalled[]): Promise<void> {
  await page.evaluate((evts) => {
    const w = window as unknown as {
      __graphStore?: { getState: () => { applyEvent: (e: unknown) => void } };
    };
    if (!w.__graphStore) {
      throw new Error('window.__graphStore not exposed — App.tsx affordance missing');
    }
    const store = w.__graphStore.getState();
    for (const e of evts) store.applyEvent(e);
  }, events);
}

async function resetMcpState(page: Page): Promise<void> {
  // currentMcpServers + activeMcpCalls survive clear() (registry-backed /
  // per-session animation) — reset explicitly per the v1.6
  // <test_isolation_audit> discipline.
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

// Vite re-optimizes deps on first request to new code paths. Per gotcha
// #53, the first spec covering a new component pays a cold-start
// optimizer pass. 120s holds margin above webServer.timeout.
test.describe('M06.E MCP Servers settings wire', () => {
  test.describe.configure({ timeout: 120_000 });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await resetMcpState(page);
  });

  test('injecting_mcp_installed_event_surfaces_server_row', async ({ page }) => {
    await dispatch(page, [
      { type: 'mcp_installed', name: 'filesystem', transport_kind: 'stdio', has_auth: false },
    ]);
    const row = page.getByTestId('mcp-server-row-filesystem');
    await expect(row).toBeVisible();
    await expect(row).toContainText('filesystem');
    await expect(row).toHaveClass(/mcp-server-row--connected/);
  });

  test('empty_state_renders_when_no_servers_installed', async ({ page }) => {
    await expect(page.getByTestId('mcp-server-settings-empty')).toBeVisible();
  });

  test('clicking_add_server_opens_the_add_modal', async ({ page }) => {
    await page.getByTestId('mcp-add-server-button').click();
    await expect(page.getByTestId('mcp-server-add-modal')).toBeVisible();
  });
});
