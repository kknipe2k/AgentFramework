import { test, expect, type Page } from '@playwright/test';

// Renderer-level Playwright covering the M04.F BudgetHeaderBar.
//
// tauri-driver E2E remains disabled per M04 Stage F Key constraints
// (M03 PR #47 carry-forward); this spec drives graph state via the
// `window.__graphStore` affordance (App.tsx) so the BudgetHeaderBar
// renders against the live store without spinning up an SDK or routing
// through real Tauri IPC.

interface BudgetWarn {
  type: 'budget_warn';
  spent_usd: number;
  cap_usd: number;
  percent: number;
}
interface BudgetDownshift {
  type: 'budget_downshift';
  from_model: string;
  to_model: string;
  reason: string;
}
interface BudgetSuspended {
  type: 'budget_suspended';
  spent_usd: number;
  cap_usd: number;
}
interface BudgetExceeded {
  type: 'budget_exceeded';
  spent_usd: number;
  cap_usd: number;
}
type DriverEvent = BudgetWarn | BudgetDownshift | BudgetSuspended | BudgetExceeded;

async function dispatch(page: Page, events: DriverEvent[]): Promise<void> {
  await page.evaluate((evts) => {
    const w = window as unknown as {
      __graphStore?: { getState: () => { applyEvent: (e: unknown) => void } };
    };
    if (!w.__graphStore) {
      throw new Error('window.__graphStore not exposed — App.tsx affordance missing');
    }
    const store = w.__graphStore.getState();
    for (const e of evts) {
      store.applyEvent(e);
    }
  }, events);
}

async function clearStore(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as {
      __graphStore?: { getState: () => { clear: () => void } };
    };
    w.__graphStore?.getState().clear();
  });
}

test.describe('M04.F budget threshold flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await clearStore(page);
  });

  test('header bar dormant until first budget event', async ({ page }) => {
    await expect(page.getByTestId('budget-header-bar')).toBeHidden();
  });

  test('header bar surfaces at 50 percent with warn color', async ({ page }) => {
    await dispatch(page, [{ type: 'budget_warn', spent_usd: 2.5, cap_usd: 5.0, percent: 50 }]);
    const bar = page.getByTestId('budget-header-bar');
    await expect(bar).toBeVisible();
    await expect(bar).toHaveAttribute('data-status', 'warn');
    await expect(page.getByTestId('budget-bar-spent')).toHaveText('$2.50');
    await expect(page.getByTestId('budget-bar-cap')).toHaveText('$5.00');
  });

  test('header bar transitions through warn → downshift → suspended → exceeded', async ({
    page,
  }) => {
    const bar = page.getByTestId('budget-header-bar');

    await dispatch(page, [{ type: 'budget_warn', spent_usd: 2.5, cap_usd: 5.0, percent: 50 }]);
    await expect(bar).toHaveAttribute('data-status', 'warn');

    await dispatch(page, [
      {
        type: 'budget_downshift',
        from_model: 'claude-opus-4-7',
        to_model: 'claude-sonnet-4-6',
        reason: 'budget_threshold',
      },
    ]);
    await expect(bar).toHaveAttribute('data-status', 'downshift');
    await expect(page.getByTestId('budget-bar-downshift-badge')).toBeVisible();

    await dispatch(page, [{ type: 'budget_suspended', spent_usd: 4.5, cap_usd: 5.0 }]);
    await expect(bar).toHaveAttribute('data-status', 'suspended');
    await expect(page.getByTestId('budget-bar-suspended-badge')).toBeVisible();

    await dispatch(page, [{ type: 'budget_exceeded', spent_usd: 5.0, cap_usd: 5.0 }]);
    await expect(bar).toHaveAttribute('data-status', 'exceeded');
    await expect(page.getByTestId('budget-bar-exceeded-banner')).toContainText(
      'Session terminated',
    );
  });

  test('clicking bar opens settings panel', async ({ page }) => {
    await dispatch(page, [{ type: 'budget_warn', spent_usd: 2.5, cap_usd: 5.0, percent: 50 }]);
    await expect(page.getByTestId('budget-bar-settings')).toBeHidden();
    await page.getByTestId('budget-bar-button').click();
    await expect(page.getByTestId('budget-bar-settings')).toBeVisible();
  });
});
