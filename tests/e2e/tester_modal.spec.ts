import { test, expect, type Page } from '@playwright/test';

// M08.F2 — renderer-level Playwright for the Builder Tester modal (spec
// Phase 9; MVP §M8 criterion 5). Drives the Vite dev server (gotcha #23
// — Playwright cannot drive the Tauri window); `test_framework` routes
// through the __TAURI_INTERNALS__.invoke boundary, so one mock covers it.
//
// DRIFT NOTE — the shipped Stage F1 `test_framework` does NOT emit live
// `agent_event`s during the run: `run_test_session_with` collects the
// session's events into a local channel and returns the complete
// `TestOutcome.trace`. So the modal reduces `outcome.trace` into the
// scoped graph AFTER the invoke resolves — there is nothing to script
// onto the `agent_event` channel. This spec mocks `test_framework` to
// RETURN a TestOutcome-with-trace.

interface TestOutcome {
  passed: boolean;
  capability_failures: { agent_id: string; needed: string; reason: string }[];
  token_spend: { input: number; output: number; total: number };
  timing: { secs: number; nanos: number };
  vdr: unknown;
  trace: { type: string; [k: string]: unknown }[];
}

const SESSION_START = {
  type: 'session_start',
  session_id: 's-tester-e2e',
  framework: 'tester-fixture',
  model: 'claude-haiku-4-5',
};
const AGENT_SPAWNED = {
  type: 'agent_spawned',
  agent_id: 'worker',
  agent_name: 'worker',
  session_id: 's-tester-e2e',
};

const PASS_OUTCOME: TestOutcome = {
  passed: true,
  capability_failures: [],
  token_spend: { input: 120, output: 45, total: 165 },
  timing: { secs: 1, nanos: 250_000_000 },
  vdr: { decision: 'ok' },
  trace: [SESSION_START, AGENT_SPAWNED],
};

const FAIL_OUTCOME: TestOutcome = {
  passed: false,
  capability_failures: [
    {
      agent_id: 'worker',
      needed: 'read',
      reason: 'requested `read /etc/passwd` — declared scope `none`',
    },
  ],
  token_spend: { input: 80, output: 10, total: 90 },
  timing: { secs: 0, nanos: 500_000_000 },
  vdr: null,
  trace: [
    SESSION_START,
    AGENT_SPAWNED,
    {
      type: 'capability_violation',
      agent_id: 'worker',
      capability_kind: 'read',
      declared_scope: 'none',
      requested_action: 'read /etc/passwd',
    },
  ],
};

async function installTauriMock(page: Page, outcome: TestOutcome): Promise<void> {
  await page.addInitScript((injectedOutcome) => {
    let callbackId = 0;
    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      transformCallback: (): number => {
        callbackId += 1;
        return callbackId;
      },
      invoke: async (command: string): Promise<unknown> => {
        if (command === 'test_framework') return injectedOutcome;
        if (command === 'has_api_key') return false;
        if (command === 'list_installed_artifacts') return [];
        if (command === 'validate_framework') {
          return { schema_errors: [], capability_errors: [], ok: true, capability_summary: null };
        }
        return undefined;
      },
    };
  }, outcome);
}

async function openTesterModal(page: Page): Promise<void> {
  await page.getByTestId('view-switch-builder').click();
  await expect(page.getByTestId('builder-shell')).toBeVisible();
  await page.getByRole('button', { name: 'Test', exact: true }).click();
  await expect(page.getByTestId('tester-modal')).toBeVisible();
}

async function runTask(page: Page, task: string): Promise<void> {
  await page.getByTestId('tester-task-input').fill(task);
  await page.getByTestId('tester-run').click();
}

test.describe('M08.F2 Builder Tester modal', () => {
  test.describe.configure({ timeout: 120_000 });

  test('the_Test_button_opens_the_Tester_modal', async ({ page }) => {
    await installTauriMock(page, PASS_OUTCOME);
    await page.goto('/');
    await openTesterModal(page);
  });

  test('entering_a_task_and_clicking_Run_renders_the_smaller_graph_pane', async ({ page }) => {
    await installTauriMock(page, PASS_OUTCOME);
    await page.goto('/');
    await openTesterModal(page);
    await runTask(page, 'summarize the input');
    // The trace's agent_spawned reduces into the scoped graph; the
    // smaller pane renders the live-graph AgentNode for it.
    const pane = page.getByTestId('tester-graph-pane');
    await expect(pane.getByTestId('agent-node-worker')).toBeVisible();
  });

  test('the_VDR_token_and_pass_fail_surfaces_populate_from_the_TestOutcome', async ({ page }) => {
    await installTauriMock(page, PASS_OUTCOME);
    await page.goto('/');
    await openTesterModal(page);
    await runTask(page, 'summarize the input');
    await expect(page.getByTestId('tester-result-verdict')).toHaveText('PASS');
    await expect(page.getByTestId('tester-result-tokens')).toContainText('total 165');
    await expect(page.getByTestId('tester-result-tokens')).toContainText('1250 ms');
    await expect(page.getByTestId('tester-result-vdr')).toContainText('"decision": "ok"');
  });

  test('a_capability_violating_framework_surfaces_a_test_failure_no_hitl_prompt', async ({
    page,
  }) => {
    await installTauriMock(page, FAIL_OUTCOME);
    await page.goto('/');
    await openTesterModal(page);
    await runTask(page, 'read a protected file');
    // The violation is a test FAILURE line — never a HITL prompt
    // (F1.3.3: the test-defaults HitlSeam never prompts).
    await expect(page.getByTestId('tester-result-verdict')).toHaveText('FAIL');
    await expect(page.getByTestId('tester-capability-failures')).toContainText('read');
    await expect(page.getByTestId('hitl-modal')).toHaveCount(0);
  });

  test('closing_the_modal_discards_the_run_and_leaves_the_live_graph_untouched', async ({
    page,
  }) => {
    await installTauriMock(page, PASS_OUTCOME);
    await page.goto('/');
    await openTesterModal(page);
    await runTask(page, 'summarize the input');
    await expect(page.getByTestId('tester-result')).toBeVisible();
    await page.getByTestId('tester-close').click();
    await expect(page.getByTestId('tester-modal')).toHaveCount(0);
    // Discard-on-close: the test run never wrote into the live runtime
    // graph (the load-bearing F2 scoping invariant).
    const liveNodeCount = await page.evaluate(() => {
      const w = window as unknown as {
        __graphStore?: { getState: () => { nodes: unknown[] } };
      };
      return w.__graphStore?.getState().nodes.length ?? -1;
    });
    expect(liveNodeCount).toBe(0);
  });
});
