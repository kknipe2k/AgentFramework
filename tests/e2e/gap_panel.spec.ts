import { test, expect, type Page } from '@playwright/test';

// Renderer-level Playwright covering the M05.F GapPanel + CapabilityBadge
// + capability-violation modal wire path. tauri-driver E2E remains
// disabled per M03 PR #47 carry-forward; this spec runs against the Vite
// dev server using `window.__graphStore` (App.tsx affordance) to drive
// graph state via `page.evaluate` per gotcha #54.
//
// Capability-violation modal: per ADR-0007, NO new modal component lands
// in Stage F. The existing M04.E HITLModal already routes `pendingHitl`
// entries with `ui_variant: 'modal'` — the runtime emits a `hitl_requested`
// event with `trigger: 'on_capability_violation'` and the modal mounts.
// This spec asserts the wire path holds end-to-end at the renderer layer.

interface ToolMissing {
  type: 'tool_missing';
  agent_id: string;
  tool_name: string;
  severity: 'critical' | 'important' | 'advisory' | 'requested';
  suggested_action: string;
  requested_via: 'loader' | 'request_capability';
}
interface GapResolved {
  type: 'gap_resolved';
  agent_id: string;
  kind: 'tool' | 'skill' | 'mcp' | 'agent';
  capability: string;
}
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
type DriverEvent = ToolMissing | GapResolved | HitlRequested;

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

const toolGap: ToolMissing = {
  type: 'tool_missing',
  agent_id: 'worker',
  tool_name: 'fetch_prs',
  severity: 'critical',
  suggested_action: "Install tool 'fetch_prs' and click Resume.",
  requested_via: 'loader',
};

const capabilityViolationPrompt: HitlRequested = {
  type: 'hitl_requested',
  prompt_id: 'p-capviol',
  trigger: 'on_capability_violation',
  agent_id: 'worker',
  question:
    "Agent worker requested capability 'write' on /etc — not in granted scope. Allow once, deny, or abort?",
  options: ['allow_once', 'deny', 'abort'],
  ui_variant: 'modal',
  timeout_at_unix_ms: 9_000_000_000_000,
};

// Vite re-optimizes deps on first request to new code paths. Per gotcha #53,
// the first spec covering a new component pays a cold-start optimizer pass
// (Vite 7 + Rolldown-scout warmup; ~30–60s typical on Windows). 120s holds
// margin above `webServer.timeout` so the cold-start absorbs cleanly when
// run in isolation.
test.describe('M05.F gap panel + capability-violation modal wire', () => {
  test.describe.configure({ timeout: 120_000 });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await clearStore(page);
  });

  test('injecting_tool_missing_event_surfaces_gap_panel', async ({ page }) => {
    await dispatch(page, [toolGap]);
    const panel = page.getByTestId('gap-panel');
    await expect(panel).toBeVisible();
    await expect(panel).toContainText('fetch_prs');
    await expect(panel).toContainText("Install tool 'fetch_prs'");
  });

  test('gap_resolved_event_dismisses_panel', async ({ page }) => {
    await dispatch(page, [toolGap]);
    await expect(page.getByTestId('gap-panel')).toBeVisible();
    await dispatch(page, [
      { type: 'gap_resolved', agent_id: 'worker', kind: 'tool', capability: 'fetch_prs' },
    ]);
    await expect(page.getByTestId('gap-panel')).toHaveCount(0);
  });

  test('capability_violation_hitl_event_mounts_existing_HITLModal_per_ADR_0007', async ({
    page,
  }) => {
    // ADR-0007 reuse: no new modal component. The HITLModal subscribes to
    // pendingHitl entries with ui_variant: 'modal' — the on_capability_violation
    // trigger routes through this existing surface.
    await dispatch(page, [capabilityViolationPrompt]);
    const modal = page.getByTestId('hitl-modal');
    await expect(modal).toBeVisible();
    await expect(modal).toHaveAttribute('aria-modal', 'true');
    await expect(modal).toContainText("capability 'write'");
    await expect(page.getByTestId('hitl-modal-trigger')).toHaveText('on_capability_violation');
    // Three response options surface as buttons.
    await expect(page.getByTestId('hitl-modal-option-allow_once')).toBeVisible();
    await expect(page.getByTestId('hitl-modal-option-deny')).toBeVisible();
    await expect(page.getByTestId('hitl-modal-option-abort')).toBeVisible();
  });
});
