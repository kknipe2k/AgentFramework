import { test, expect, type Page } from '@playwright/test';

// Renderer-level E2E running against the Vite dev server with @tauri-apps/api
// shimmed inside the page. Covers the renderer state-machine + UX invariants
// (page loads, password input typing, smoke-disabled-without-key) at a layer
// faster than the desktop-shell suite.
//
// Full Tauri 2.x desktop-shell E2E lives in `tests/e2e-tauri/` (M03.F) — uses
// `tauri-driver` + WebdriverIO per <https://v2.tauri.app/develop/tests/webdriver/>
// and exercises the built binary via WebView2 / WebKitGTK. Two test types,
// two CI jobs, two layers of regression detection.

interface AgentEvent {
  type: string;
  [key: string]: unknown;
}

type EventHandler = (e: { payload: AgentEvent }) => void;

declare global {
  interface Window {
    __invokes: { command: string; args: unknown }[];
    __emit: (e: AgentEvent) => void;
  }
}

async function installTauriShim(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const invokes: { command: string; args: unknown }[] = [];
    let handler: EventHandler | null = null;
    window.__invokes = invokes;
    window.__emit = (event: AgentEvent): void => {
      handler?.({ payload: event });
    };
    const moduleShim = {
      core: {
        invoke: async (command: string, args?: unknown): Promise<void> => {
          invokes.push({ command, args });
        },
      },
      event: {
        listen: async (_channel: string, cb: EventHandler): Promise<() => void> => {
          handler = cb;
          return (): void => {
            if (handler === cb) handler = null;
          };
        },
      },
    };
    // Install on the import-map level — Vite resolves @tauri-apps/api/{core,event}
    // to these shims inside the page.
    type W = typeof window & { __TAURI_API_SHIM__?: typeof moduleShim };
    (window as W).__TAURI_API_SHIM__ = moduleShim;
  });
}

test.describe('renderer smoke', () => {
  test.beforeEach(async ({ page }) => {
    await installTauriShim(page);
  });

  test('renderer_loads_with_setup_visible', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: /Agent Runtime/i })).toBeVisible();
    await expect(page.getByLabel(/anthropic api key/i)).toBeVisible();
  });

  test('set_key_input_is_password_type', async ({ page }) => {
    await page.goto('/');
    const input = page.getByLabel(/anthropic api key/i);
    await expect(input).toHaveAttribute('type', 'password');
  });

  test('smoke_disabled_when_no_key', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('button', { name: /run smoke test/i })).toBeDisabled();
  });
});
