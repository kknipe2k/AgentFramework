import { test, expect, type Page } from '@playwright/test';

// M08.E — renderer-level Playwright for the Builder Inspector + the
// Canvas | JSON two-way binding. Drives the Vite dev server (gotcha #23
// — Playwright cannot drive the Tauri window or the native file
// dialog); validate_framework / save_framework / load_framework AND the
// @tauri-apps/plugin-dialog directory picker all route through the
// __TAURI_INTERNALS__.invoke boundary (plugin-dialog's `open` calls
// invoke('plugin:dialog|open')), so one mock covers them. Demonstrates
// MVP §M8 criterion 4 (Validate), 6 (Canvas | JSON binding incl. the
// invalid-JSON no-desync guard), 7 (Save), 8 (Load).

const OK_REPORT = {
  schema_errors: [],
  capability_errors: [],
  ok: true,
  capability_summary: {
    files_read: ['src/**'],
    files_written: [],
    network_hosts: ['api.example.com'],
    any_shell: false,
    spawn_edges: [],
  },
};

const ONE_AGENT_FRAMEWORK = {
  name: 'loaded-fw',
  version: '0.1.0',
  description: 'A framework with one agent.',
  model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
  tools: [],
  skills: [],
  agents: [
    {
      id: 'planner',
      role: 'Lead',
      model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
      allowed_tools: [],
      allowed_skills: [],
      spawns: [],
    },
  ],
  session_root_agent: 'planner',
};

async function installTauriMock(page: Page): Promise<void> {
  await page.addInitScript(
    ({ okReport, loadedFw }) => {
      let callbackId = 0;
      (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
        transformCallback: (): number => {
          callbackId += 1;
          return callbackId;
        },
        invoke: async (command: string, args?: unknown): Promise<unknown> => {
          const w = window as unknown as { __invokeCalls?: { command: string; args?: unknown }[] };
          (w.__invokeCalls ??= []).push({ command, args });
          if (command === 'list_installed_artifacts') return [];
          if (command === 'has_api_key') return false;
          if (command === 'validate_framework') return okReport;
          if (command === 'save_framework') return null;
          if (command === 'load_framework') return { framework: loadedFw, companions: [] };
          // M09.5.A (TD-051): Save/Load migrated from plugin-dialog's
          // open() to the Rust-side pick_framework_dir command (registers
          // the chosen dir as a permitted root). The picked path now comes
          // back from this invoke.
          if (command === 'pick_framework_dir') return 'C:/picked-framework-dir';
          return undefined;
        },
      };
    },
    { okReport: OK_REPORT, loadedFw: ONE_AGENT_FRAMEWORK },
  );
}

test.describe('M08.E Builder Inspector + Canvas | JSON binding', () => {
  test.describe.configure({ timeout: 120_000 });

  test.beforeEach(async ({ page }) => {
    await installTauriMock(page);
    await page.goto('/');
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-shell')).toBeVisible();
  });

  test('Validate runs validate_framework and surfaces the capability summary', async ({ page }) => {
    // MVP §M8 criterion 4 — the explicit Validate trigger of the SAME
    // validate_framework D2's continuous pass uses (spec §9).
    await page.getByRole('button', { name: 'Validate', exact: true }).click();
    await expect(page.getByTestId('inspector-capabilities')).toContainText('src/**');
  });

  test('a JSON-tab edit round-trips to the canvas', async ({ page }) => {
    // MVP §M8 criterion 6 — edit the JSON directly, switch to Canvas,
    // the canvas shows the update (ADR-0020 — the canvas re-derives).
    await page.getByRole('tab', { name: 'JSON', exact: true }).click();
    await page.locator('.json-view textarea').fill(JSON.stringify(ONE_AGENT_FRAMEWORK, null, 2));
    await page.getByRole('tab', { name: 'Canvas', exact: true }).click();
    await expect(page.locator('.builder-agent-node')).toBeVisible();
  });

  test('invalid JSON surfaces a parse error and leaves the canvas unchanged', async ({ page }) => {
    // The load-bearing no-desync guard — a malformed edit never reaches
    // replaceFramework, so the canvas keeps its last valid framework.
    await page.getByRole('tab', { name: 'JSON', exact: true }).click();
    await page.locator('.json-view textarea').fill('{ not valid json');
    await expect(page.locator('.json-view__error')).toBeVisible();
    await page.getByRole('tab', { name: 'Canvas', exact: true }).click();
    await expect(page.locator('.builder-agent-node')).toHaveCount(0);
  });

  test('Save calls save_framework with the picked directory', async ({ page }) => {
    // MVP §M8 criterion 7 — Save opens the directory picker, then
    // writes through Stage B's save_framework.
    await page.getByRole('button', { name: 'Save', exact: true }).click();
    await expect
      .poll(async () =>
        page.evaluate(() => {
          const calls =
            (
              window as unknown as {
                __invokeCalls?: { command: string; args?: { dir?: string } }[];
              }
            ).__invokeCalls ?? [];
          return calls.find((c) => c.command === 'save_framework')?.args?.dir ?? null;
        }),
      )
      .toBe('C:/picked-framework-dir');
  });

  test('Load reads a directory and the canvas re-derives', async ({ page }) => {
    // MVP §M8 criterion 8 — Load picks a directory, calls
    // load_framework, and replaceFramework re-derives the canvas.
    await page.getByRole('button', { name: 'Load', exact: true }).click();
    await expect(page.locator('.builder-agent-node')).toBeVisible();
  });
});
