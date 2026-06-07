// Tauri 2.x real-app E2E — M08.9.B the Tester run drill-down.
//
// Drives the BUILT Tauri app via `tauri-driver` + WebdriverIO v9 (ADR-0021;
// the merge-blocking `e2e-tauri-driver` job). M08.9.B renders `outcome.trace`
// as a step list under the verdict: each tool call expands to its input /
// result payload (reusing the M08.8.A Output-rail payload formatter) and a
// Show-raw disclosure (reusing ValidationCard) reveals the raw event.
// DESIGN.md principle 1 (feedback) + principle 3 (progressive disclosure).
//
// The drill-down that proves B is KEY-DEPENDENT: producing a real tool call
// in the trace needs a live model to emit a `ToolUse` the runtime executes
// (TesterModal's `outcome` is component-local state set from the backend
// `test_framework` round-trip — there is no inject seam). So the substantive
// drill test is runtime-skip-guarded exactly like smoke.e2e.ts /
// tester_verdict.e2e.ts: it runs only when `ANTHROPIC_API_KEY` is reachable,
// and `this.skip()`s otherwise. CI provides no key, so the maintainer IRL
// walkthrough is the authoritative close (CLAUDE.md §10). The key-independent
// test below guards the surface mount + the no-premature-drilldown invariant.
//
// WebdriverIO v9 note: `$()` returns a chainable, not a promise — call methods
// on it directly. Per <https://webdriver.io/docs/api/element>.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

const hasAnthropicKey = (process.env.ANTHROPIC_API_KEY ?? '').trim().length > 0;

// A single-agent framework whose worker may use the in-process Read built-in.
// Read is allowed at the default Novice tier, so a model that emits a Read
// `ToolUse` produces a clean PASS run with a drillable tool call in the trace —
// exactly the B drill-down path (a tool step that expands to input/result).
const READ_FRAMEWORK = {
  name: 'm089b-drilldown-fixture',
  version: '1.0.0',
  description: 'M08.9.B — a Read-using framework to exercise the run drill-down',
  model: { provider: 'anthropic', id: 'claude-haiku-4-5' },
  agents: [
    {
      id: 'worker',
      role: 'worker',
      model: { provider: 'anthropic', id: 'claude-haiku-4-5' },
      capabilities: {
        tools_called: [],
        skills_loaded: [],
        file_access: { read: ['Cargo.toml'], write: [] },
        network: [],
        shell: false,
        spawn_agents: [],
      },
      allowed_tools: ['Read'],
      allowed_skills: [],
      spawns: [],
    },
  ],
  tools: [],
  skills: [],
  session_root_agent: 'worker',
};

async function openBuilderTester(): Promise<void> {
  const builderTab = $('[data-testid="view-switch-builder"]');
  await builderTab.waitForDisplayed({ timeout: 10_000 });
  await builderTab.click();
  await browser.waitUntil(async () => browser.execute('return !!window.__builderStore'), {
    timeout: 5_000,
    timeoutMsg: 'window.__builderStore not exposed',
  });
  await browser.execute(
    `
    window.__builderStore.getState().replaceFramework(arguments[0]);
    window.__builderStore.getState().openTester();
    `,
    READ_FRAMEWORK,
  );
  const testerModal = $('[data-testid="tester-modal"]');
  await testerModal.waitForDisplayed({ timeout: 5_000 });
}

describe('Tauri real-app E2E — M08.9.B Tester run drill-down', () => {
  // Teardown (M08.9.D.fix — V 🔴 #1): every test in this describe shares one
  // app session, so an `it` that opens the Tester modal leaves it open for the
  // next `it` — whose `openBuilderTester` view-switch click is then intercepted
  // by the modal scrim (crash with a key; the substantive case never runs).
  // Close the modal after each case and wait for it to disappear.
  afterEach(async () => {
    await browser.execute('window.__builderStore.getState().closeTester();');
    await $('[data-testid="tester-modal"]').waitForDisplayed({
      timeout: 5_000,
      reverse: true,
    });
  });

  it('shows no drill-down before a run (no trace steps without a result)', async () => {
    await openBuilderTester();
    // Before any run there is no result surface, hence no drill-down and no
    // trace step rows — the drill-down is a property of a completed run.
    expect(await $('[data-testid="trace-drilldown"]').isExisting()).to.equal(false);
    expect(await $('[data-testid="trace-step"]').isExisting()).to.equal(false);
    const runButton = $('[data-testid="tester-run"]');
    await runButton.waitForDisplayed({ timeout: 5_000 });
  });

  it('drills a tool call into its input/result and the raw event', async function () {
    if (!hasAnthropicKey) {
      this.skip();
    }
    await openBuilderTester();
    const taskInput = $('[data-testid="tester-task-input"]');
    await taskInput.setValue('Use the Read tool to read Cargo.toml and report its package name.');
    const runButton = $('[data-testid="tester-run"]');
    await runButton.click();

    // The run completes and the result surface renders.
    const verdict = $('[data-testid="tester-result-verdict"]');
    await verdict.waitForDisplayed({ timeout: 60_000 });

    // A drillable tool-call row appears under the verdict.
    const step = $('[data-testid="trace-step"]');
    await step.waitForDisplayed({ timeout: 5_000 });
    // Expanding it reveals the input + result payload.
    const toggle = $('[data-testid="trace-step-toggle"]');
    await toggle.click();
    const input = $('[data-testid="trace-step-input"]');
    await input.waitForDisplayed({ timeout: 5_000 });
    expect(await $('[data-testid="trace-step-output"]').isExisting()).to.equal(true);
    // Show-raw reveals the literal AgentEvent JSON (progressive disclosure).
    const rawToggle = $('[data-testid="trace-step-raw-toggle"]');
    await rawToggle.click();
    const raw = $('[data-testid="trace-step-raw"]');
    await raw.waitForDisplayed({ timeout: 5_000 });
    expect(await raw.getText()).to.include('tool_invoked');
  });
});
