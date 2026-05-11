import { test, expect, type Page } from '@playwright/test';

// Renderer-level Playwright covering the M04.E HITL Panel + Modal + Toast.
// tauri-driver E2E remains disabled per Key constraints (M03 PR #47
// carry-forward); this spec drives graph state via the `window.__graphStore`
// affordance (App.tsx) so the HITL surfaces render against the live store
// without spinning up an SDK or routing through real Tauri IPC.
//
// Module mocking across the @tauri-apps/api ESM boundary doesn't work in
// Playwright; Vitest covers the click→invoke linkage in the per-component
// test files. This spec covers the surface-on-state-change +
// dismiss-on-state-change flow that only renders correctly inside a real
// browser layout.

interface HitlRequested {
  type: 'hitl_requested';
  prompt_id: string;
  trigger: string;
  agent_id: string | null;
  question: string;
  options: string[];
  ui_variant: 'panel' | 'modal' | 'toast';
  timeout_at_unix_ms: number;
}
interface HitlResolved {
  type: 'hitl_resolved';
  prompt_id: string;
  choice: string;
  duration_ms: number;
}
interface NotifierDispatched {
  type: 'notifier_dispatched';
  notifier_type: string;
  trigger: string;
  success: boolean;
}
type DriverEvent = HitlRequested | HitlResolved | NotifierDispatched;

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

const failureThresholdPrompt: HitlRequested = {
  type: 'hitl_requested',
  prompt_id: 'p-failure',
  trigger: 'on_failure_threshold',
  agent_id: null,
  question: 'Task t-1 exceeded failure budget after 3 attempts. Retry, skip, or abort?',
  options: ['retry', 'skip', 'abort'],
  ui_variant: 'panel',
  timeout_at_unix_ms: 9_000_000_000_000,
};

const riskyToolPrompt: HitlRequested = {
  type: 'hitl_requested',
  prompt_id: 'p-risky',
  trigger: 'on_risky_tool',
  agent_id: 'agent-1',
  question: 'Run Bash:rm -rf /tmp/foo?',
  options: ['allow', 'block'],
  ui_variant: 'modal',
  timeout_at_unix_ms: 9_000_000_000_000,
};

const perTaskPrompt: HitlRequested = {
  type: 'hitl_requested',
  prompt_id: 'p-pertask',
  trigger: 'per_task',
  agent_id: null,
  question: 'Approve next task?',
  options: ['ok', 'skip'],
  ui_variant: 'toast',
  timeout_at_unix_ms: 9_000_000_000_000,
};

test.describe('HITL failure-escalation flow (spec §6a)', () => {
  test.describe.configure({ timeout: 90_000 });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await clearStore(page);
  });

  test('panel_surfaces_when_on_failure_threshold_fires', async ({ page }) => {
    await dispatch(page, [failureThresholdPrompt]);
    const panel = page.getByTestId('hitl-panel');
    await expect(panel).toBeVisible();
    await expect(panel).toContainText('exceeded failure budget');
    // All three options render as buttons.
    await expect(page.getByTestId('hitl-panel-option-retry')).toBeVisible();
    await expect(page.getByTestId('hitl-panel-option-skip')).toBeVisible();
    await expect(page.getByTestId('hitl-panel-option-abort')).toBeVisible();
  });

  test('panel_dismisses_when_hitl_resolved_arrives', async ({ page }) => {
    await dispatch(page, [failureThresholdPrompt]);
    await expect(page.getByTestId('hitl-panel')).toBeVisible();
    await dispatch(page, [
      { type: 'hitl_resolved', prompt_id: 'p-failure', choice: 'skip', duration_ms: 100 },
    ]);
    await expect(page.getByTestId('hitl-panel')).toHaveCount(0);
  });

  test('modal_surfaces_for_on_risky_tool_with_aria_modal_true', async ({ page }) => {
    await dispatch(page, [riskyToolPrompt]);
    const modal = page.getByTestId('hitl-modal');
    await expect(modal).toBeVisible();
    await expect(modal).toHaveAttribute('aria-modal', 'true');
    await expect(modal).toContainText('Bash:rm');
  });

  test('toast_surfaces_for_per_task_with_role_status', async ({ page }) => {
    await dispatch(page, [perTaskPrompt]);
    const toast = page.getByTestId('hitl-toast');
    await expect(toast).toBeVisible();
    await expect(toast).toHaveAttribute('role', 'status');
    await expect(page.getByTestId('hitl-toast-summary')).toBeVisible();
  });

  test('notifier_records_attach_to_pending_prompt_on_dispatch', async ({ page }) => {
    await dispatch(page, [
      failureThresholdPrompt,
      {
        type: 'notifier_dispatched',
        notifier_type: 'terminal_bell',
        trigger: 'on_failure_threshold',
        success: true,
      },
    ]);
    // Verify the record landed in the store. The renderer surface for
    // notifier records lands in M5+; this assertion proves the wiring.
    const records = await page.evaluate(() => {
      const w = window as unknown as {
        __graphStore?: {
          getState: () => {
            notifierRecords: Record<string, Array<{ notifierType: string; outcome: string }>>;
          };
        };
      };
      return w.__graphStore?.getState().notifierRecords ?? {};
    });
    expect(records['p-failure']).toHaveLength(1);
    expect(records['p-failure']?.[0]).toMatchObject({
      notifierType: 'terminal_bell',
      outcome: 'dispatched',
    });
  });

  test('escape_key_dismisses_panel_locally_without_resolving', async ({ page }) => {
    await dispatch(page, [failureThresholdPrompt]);
    await expect(page.getByTestId('hitl-panel')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('hitl-panel')).toHaveCount(0);
    // Underlying prompt is still in pendingHitl — the seam keeps awaiting.
    const stillPending = await page.evaluate(() => {
      const w = window as unknown as {
        __graphStore?: {
          getState: () => { pendingHitl: Record<string, unknown> };
        };
      };
      return Boolean(w.__graphStore?.getState().pendingHitl['p-failure']);
    });
    expect(stillPending).toBe(true);
  });
});
