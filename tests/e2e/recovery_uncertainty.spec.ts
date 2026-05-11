import { test, expect, type Page } from '@playwright/test';

// Renderer-level Playwright covering the M04.F UncertaintyPrompt.
//
// Recovery flow is split: the RecoveryDialog cold-start prompt requires
// localStorage to be set BEFORE the renderer mounts (the dialog reads
// LAST_SESSION_KEY in useEffect). That timing is brittle in this
// "navigate-then-drive-store" architecture; the RecoveryDialog click
// flow is fully exercised by the vitest suite. Here we cover the
// UncertaintyPrompt — the post-resume surface that drives the 4-action
// prompt against renderer state seeded via `recordUncertainInvocation`.

interface StoreActions {
  recordUncertainInvocation: (i: {
    invocationId: string;
    toolName?: string;
    agentId?: string;
  }) => void;
  clear: () => void;
}

async function seedUncertain(
  page: Page,
  invocations: { invocationId: string; toolName?: string; agentId?: string }[],
): Promise<void> {
  await page.evaluate((invs) => {
    const w = window as unknown as {
      __graphStore?: { getState: () => StoreActions };
    };
    if (!w.__graphStore) {
      throw new Error('window.__graphStore not exposed — App.tsx affordance missing');
    }
    const store = w.__graphStore.getState();
    for (const i of invs) {
      store.recordUncertainInvocation(i);
    }
  }, invocations);
}

async function clearStore(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as {
      __graphStore?: { getState: () => StoreActions };
    };
    w.__graphStore?.getState().clear();
  });
}

test.describe('M04.F recovery uncertainty prompt', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await clearStore(page);
  });

  test('prompt hidden when no uncertain invocations', async ({ page }) => {
    await expect(page.getByTestId('uncertainty-prompt')).toBeHidden();
  });

  test('prompt surfaces all four spec §1b actions', async ({ page }) => {
    await seedUncertain(page, [{ invocationId: 'sig-tool-1', toolName: 'Read', agentId: 'a1' }]);
    await expect(page.getByTestId('uncertainty-prompt')).toBeVisible();
    await expect(page.getByTestId('uncertainty-prompt-invocation-id')).toHaveText('sig-tool-1');
    for (const a of ['retry', 'skip', 'mark', 'abort']) {
      await expect(page.getByTestId(`uncertainty-action-${a}`)).toBeVisible();
    }
  });

  test('prompt shows remaining count when multiple uncertain', async ({ page }) => {
    await seedUncertain(page, [
      { invocationId: 'sig-1' },
      { invocationId: 'sig-2' },
      { invocationId: 'sig-3' },
    ]);
    await expect(page.getByTestId('uncertainty-prompt-remaining')).toContainText('2 more');
  });

  test('aria attributes for modal dialog', async ({ page }) => {
    await seedUncertain(page, [{ invocationId: 'sig-1' }]);
    const dialog = page.getByTestId('uncertainty-prompt');
    await expect(dialog).toHaveAttribute('role', 'dialog');
    await expect(dialog).toHaveAttribute('aria-modal', 'true');
  });
});
