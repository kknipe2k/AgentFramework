import { test, expect, type Page } from '@playwright/test';

// Renderer-level Playwright covering the M04.C ApprovalPanel + PlanNode.
// tauri-driver E2E remains disabled per Key constraints (M03 PR #47
// carry-forward); this spec runs against the Vite dev server using
// `window.__graphStore` (App.tsx exposes the Zustand store unconditionally
// — see comment there) to drive graph state via `page.evaluate`.
//
// Module mocking across the @tauri-apps/api ESM boundary doesn't work in
// Playwright; Vitest covers the click→invoke linkage in
// `tests/unit/components/ApprovalPanel.test.tsx`. This spec covers the
// surface-on-state-change + dismiss-on-state-change flow that only
// renders correctly inside an actual browser layout.

// Driver events shaped per the live `AgentEvent` discriminator. The wire
// format is the schema source of truth; we re-declare a narrow subset
// here rather than importing from `src/types/agent_event` because the
// test harness runs in node-side TS (Playwright) but the assignment
// happens via `page.evaluate` whose serialization layer doesn't transit
// the schema's union types cleanly.
interface PlanCreated {
  type: 'plan_created';
  plan_id: string;
  title: string;
  task_count: number;
  approval_required: boolean;
}
interface PlanApprovalRequested {
  type: 'plan_approval_requested';
  plan_id: string;
}
interface PlanApproved {
  type: 'plan_approved';
  plan_id: string;
  // ApprovedBy from src/types/agent_event.ts is `'user' | 'auto'`. Mirror
  // exactly so the cast inside page.evaluate stays type-safe.
  approved_by: 'user' | 'auto';
}
type DriverEvent = PlanCreated | PlanApprovalRequested | PlanApproved;

async function dispatch(page: Page, events: DriverEvent[]): Promise<void> {
  // The store on `window.__graphStore` (App.tsx affordance) carries the
  // live `applyEvent` typed against the canonical `AgentEvent` union;
  // narrow `DriverEvent` is structurally a subset, so the cast is sound
  // for the variants this spec touches. ESLint's no-explicit-any is
  // suppressed because `page.evaluate` requires a serializable signature
  // and the harness's typing of `Window.__graphStore` lives in App.tsx
  // (the canonical declaration) — re-declaring it here would shadow.
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

// Vite re-optimizes deps on first request to new code paths. The first
// page.goto in this file's first test loads ApprovalPanel + its imports,
// which weren't reached by the smoke spec's navigations — so this file's
// cold-start absorbs an additional optimization pass on top of the baseline
// per-test 60s budget. 90s matches the webServer.timeout in playwright.config.ts.
test.describe('plan approval flow', () => {
  test.describe.configure({ timeout: 90_000 });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await clearStore(page);
  });

  test('approval_panel_surfaces_when_plan_awaits_approval', async ({ page }) => {
    await dispatch(page, [
      {
        type: 'plan_created',
        plan_id: 'p-fixture',
        title: 'Refactor auth flow',
        task_count: 3,
        approval_required: true,
      },
      { type: 'plan_approval_requested', plan_id: 'p-fixture' },
    ]);
    const panel = page.getByTestId('approval-panel');
    await expect(panel).toBeVisible();
    await expect(panel).toContainText('Refactor auth flow');
  });

  test('approval_panel_dismisses_when_plan_transitions_to_in_progress', async ({ page }) => {
    await dispatch(page, [
      {
        type: 'plan_created',
        plan_id: 'p-fixture',
        title: 'Refactor auth flow',
        task_count: 3,
        approval_required: true,
      },
      { type: 'plan_approval_requested', plan_id: 'p-fixture' },
    ]);
    await expect(page.getByTestId('approval-panel')).toBeVisible();
    await dispatch(page, [{ type: 'plan_approved', plan_id: 'p-fixture', approved_by: 'user' }]);
    await expect(page.getByTestId('approval-panel')).toHaveCount(0);
  });

  test('plan_node_status_class_transitions_with_state', async ({ page }) => {
    await dispatch(page, [
      {
        type: 'plan_created',
        plan_id: 'p-fixture',
        title: 'Tiny plan',
        task_count: 1,
        approval_required: true,
      },
      { type: 'plan_approval_requested', plan_id: 'p-fixture' },
    ]);
    const planNode = page.getByTestId('plan-node-p-fixture');
    await expect(planNode).toHaveAttribute('data-status', 'awaiting_approval');
    await dispatch(page, [{ type: 'plan_approved', plan_id: 'p-fixture', approved_by: 'user' }]);
    await expect(planNode).toHaveAttribute('data-status', 'in_progress');
  });
});
