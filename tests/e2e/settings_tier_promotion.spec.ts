import { test, expect, type Page } from '@playwright/test';

// M08.G — renderer-level Playwright for the Settings panel + the
// Novice↔Promoted tier control (closes M07-IRL #5). Drives the Vite dev
// server (gotcha #23 — Playwright cannot drive the Tauri window);
// `request_tier_transition` + `set_global_budget` route through the
// __TAURI_INTERNALS__.invoke boundary, so one mock covers them.
//
// The backend's `tier_transition` event is scripted through
// `window.__graphStore` (gotcha #54 — App.tsx exposes the store
// unconditionally): the renderer must NOT optimistically flip the tier;
// the existing `tier_transition` reducer (graphStore.ts:1549) is the
// single writer.

async function installTauriMock(page: Page): Promise<void> {
  await page.addInitScript(() => {
    let callbackId = 0;
    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      transformCallback: (): number => {
        callbackId += 1;
        return callbackId;
      },
      invoke: async (command: string): Promise<unknown> => {
        // request_tier_transition / set_global_budget resolve Ok — the
        // backend returns `()`; the displayed tier updates via the
        // scripted tier_transition event, not this return value.
        if (command === 'request_tier_transition') return undefined;
        if (command === 'set_global_budget') return undefined;
        if (command === 'has_api_key') return false;
        if (command === 'list_installed_artifacts') return [];
        if (command === 'validate_framework') {
          return { schema_errors: [], capability_errors: [], ok: true, capability_summary: null };
        }
        return undefined;
      },
    };
  });
}

interface TierTransitionEvent {
  type: 'tier_transition';
  previous: 'novice' | 'promoted';
  current: 'novice' | 'promoted';
  reason: string;
}

async function scriptTierTransition(page: Page, event: TierTransitionEvent): Promise<void> {
  await page.evaluate((e) => {
    const w = window as unknown as {
      __graphStore?: { getState: () => { applyEvent: (ev: unknown) => void } };
    };
    if (!w.__graphStore) {
      throw new Error('window.__graphStore not exposed — App.tsx affordance missing');
    }
    w.__graphStore.getState().applyEvent(e);
  }, event);
}

// Vite re-optimizes deps on first request to new code paths — the first
// page.goto here loads SettingsPanel + its imports, absorbing an extra
// optimization pass on top of the per-test budget. 90s matches the
// webServer.timeout in playwright.config.ts (gotcha #53).
test.describe('M08.G Settings panel + tier promotion', () => {
  test.describe.configure({ timeout: 90_000 });

  test.beforeEach(async ({ page }) => {
    await installTauriMock(page);
  });

  test('promoting via Settings updates the displayed tier through the tier_transition reducer', async ({
    page,
  }) => {
    await page.goto('/');

    // The Settings panel shows the first-run tier.
    await expect(page.getByTestId('tier-current')).toContainText('novice');

    // Promote — the mocked request_tier_transition resolves; the
    // renderer must NOT optimistically flip the tier.
    await page.getByTestId('tier-transition-button').click();

    // Script the backend's tier_transition event (what the real command
    // emits on the agent_event channel).
    await scriptTierTransition(page, {
      type: 'tier_transition',
      previous: 'novice',
      current: 'promoted',
      reason: 'user requested promoted via Settings',
    });

    // The displayed tier updates through the EXISTING reducer.
    await expect(page.getByTestId('tier-current')).toContainText('promoted');
    await expect(page.getByTestId('tier-transition-button')).toContainText('Demote');
  });

  test('the Settings tier control never offers an Operator option', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByTestId('settings-panel')).toBeVisible();
    // §0d locks v0.1 to Novice + Promoted; Operator is v1.0.
    await expect(page.getByTestId('settings-panel')).not.toContainText(/operator/i);
  });

  test('the budget cap input reflects and persists a set value', async ({ page }) => {
    await page.goto('/');
    // M09.D.fix: the budget section now defaults collapsed (DESIGN.md
    // disclosure) — expand it before driving its input.
    await page.getByTestId('settings-section-toggle-budget').click();
    await page.getByTestId('budget-cap-input').fill('30');
    await page.getByTestId('budget-save-button').click();
    // The input reflects the persisted value (M06.5 IRL 🟡-4 — the
    // budget-cap input previously did not reflect/persist anything).
    await expect(page.getByTestId('budget-cap-input')).toHaveValue('30');
  });

  test('the Settings panel is reachable in Builder mode (cross-mode chrome)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-shell')).toBeVisible();
    // SettingsPanel mounts OUTSIDE the view conditional — switching to
    // Builder mode does not unmount it; the tier control stays reachable
    // (C.3.2 — a Builder user must still be able to promote).
    await expect(page.getByTestId('settings-panel')).toBeVisible();
    await expect(page.getByTestId('tier-current')).toBeVisible();
  });
});
