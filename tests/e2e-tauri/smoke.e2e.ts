// Tauri 2.x desktop-shell E2E smoke tests (M03.F).
//
// Drives the built Tauri app via `tauri-driver` + WebdriverIO v9. Six tests
// covering the M03 user-facing surfaces: app launch + setup panel; save-key
// flow + keychain indicator; smoke happy path with real Anthropic API call;
// click-to-inspect; SQL inspector execute; reload reconstructs graph from
// persisted signals.
//
// Tests 3–6 form one real-session chain: test 3 runs the smoke session
// (a real Anthropic `/v1/messages` call against Haiku 4.5), test 4 inspects
// the agent node it spawns, test 5 queries the signals it writes, test 6
// reloads and expects the graph it built. They are runtime-skip-guarded as
// a group: they run only when an Anthropic key is exposed to this process
// via the `ANTHROPIC_API_KEY` env var, and `this.skip()` otherwise. CI
// provides no key — the maintainer keeps no `ANTHROPIC_TEST_KEY` secret;
// the real-Anthropic path is covered by the HITL/IRL pass — so in CI
// tests 3–6 skip and the job gates on the key-independent tests 1 & 2
// (2 passed + 4 skipped). To run all six locally, export `ANTHROPIC_API_KEY`
// and ensure the app can reach a real key (set it through the UI as in
// test 2's flow).
//
// The four test.skip()-with-rationale entries that M02.E carried forward in
// `tests/e2e/smoke.spec.ts` are deleted in this stage; this file is the
// replacement coverage.
//
// Selectors prefer `aria-label` / `role` queries over raw `data-testid`
// where possible (matches the existing Vitest+RTL idiom; future a11y
// refactor doesn't break tests).
//
// WebdriverIO v9 note: `$()` and `$$()` return chainable objects, not
// promises. We don't intermediate-await them — methods are called on the
// chainable directly. Per <https://webdriver.io/docs/api/element>.
/// <reference types="mocha" />
// Side-effect import: pulls in `webdriverio`'s augmentation of the
// `WebdriverIO.Browser` interface (waitUntil, reloadSession, etc.).
// Without this the `Browser` type from `@wdio/globals` is empty.
import type {} from 'webdriverio';
import { $, $$, browser } from '@wdio/globals';
import { expect } from 'chai';

// Tests 3–6 run only when a real Anthropic key is reachable by this
// process. Evaluated once at load — the env does not change mid-run. The
// guarded tests use `this.skip()`, which requires a non-arrow callback so
// `this` is the Mocha context.
const hasAnthropicKey = (process.env.ANTHROPIC_API_KEY ?? '').trim().length > 0;

describe('Tauri shell E2E — M03 live graph', () => {
  it('app launches with SetupPanel visible', async () => {
    const setupPanel = $('section[aria-label="api key setup"]');
    await setupPanel.waitForDisplayed({ timeout: 10_000 });
    expect(await setupPanel.isDisplayed()).to.equal(true);
  });

  it('save-key flow: paste key → save → ✓ stored in OS keychain', async () => {
    const input = $('input[type="password"]');
    // Length ≥ 10 enables the Save button (renderer-side validation only;
    // not a key-format check). ANTHROPIC_TEST_KEY (when present in env) is
    // injected at app-launch time; this test exercises the UI write path.
    await input.setValue('sk-ant-test-1234567890123456');
    const saveButton = $('button*=Save key');
    await saveButton.click();
    const savedIndicator = $('span[aria-label="saved"]');
    await savedIndicator.waitForDisplayed({ timeout: 5_000 });
    expect(await savedIndicator.getText()).to.include('stored in OS keychain');
  });

  it('graph renders after smoke test (real Anthropic API call)', async function () {
    if (!hasAnthropicKey) {
      this.skip();
    }
    const smokeButton = $('button*=Run smoke test');
    await smokeButton.click();
    const agentNode = $('[data-testid^="agent-node-"]');
    // Anthropic Haiku 4.5's first SSE event arrives ~1-2s after the request;
    // 30s gives margin for slow CI network without masking real regressions.
    await agentNode.waitForDisplayed({ timeout: 30_000 });
    expect(await agentNode.getAttribute('data-status')).to.equal('active');
    // Wait for completion — Haiku 4.5 typically completes the smoke prompt
    // in ~3s. The status flips active → complete on agent_complete event.
    await browser.waitUntil(
      async () => (await agentNode.getAttribute('data-status')) === 'complete',
      { timeout: 30_000, timeoutMsg: 'agent_complete event did not arrive within 30s' },
    );
  });

  it('click AgentNode → InspectorPanel opens with node data', async function () {
    // Depends on the agent node test 3's session spawns — guarded with it.
    if (!hasAnthropicKey) {
      this.skip();
    }
    const agentNode = $('[data-testid^="agent-node-"]');
    await agentNode.click();
    const inspector = $('[role="dialog"][aria-label="node inspector"]');
    await inspector.waitForDisplayed({ timeout: 5_000 });
    const inspectorText = await inspector.getText();
    expect(inspectorText).to.include('agentId');
    expect(inspectorText).to.include('status');
  });

  it('SQL inspector executes SELECT * FROM signals LIMIT 5', async function () {
    // Depends on the signals test 3's session writes — guarded with it.
    if (!hasAnthropicKey) {
      this.skip();
    }
    const sqlTextarea = $('textarea[aria-label="SQL query"]');
    await sqlTextarea.setValue('SELECT * FROM signals LIMIT 5;');
    const executeButton = $('button*=Execute');
    await executeButton.click();
    // Either the results table renders (success path) or an error paragraph
    // surfaces (e.g., empty signals on a fresh session). Both prove the
    // inspector reaches the drone subprocess and surfaces a structured
    // response — what we're asserting at the E2E layer.
    await browser.waitUntil(
      async () => {
        const tableCount = await $$('table.sql-inspector__results').length;
        const errorCount = await $$('p[role="alert"]').length;
        return tableCount > 0 || errorCount > 0;
      },
      { timeout: 5_000, timeoutMsg: 'SQL inspector returned neither rows nor error' },
    );
  });

  it('reload reconstructs the graph from persisted signals', async function () {
    // Depends on the session test 3 persists — guarded with it.
    if (!hasAnthropicKey) {
      this.skip();
    }
    // tauri-driver does not currently expose a "restart application" hook,
    // so we use WebdriverIO's `reloadSession` which re-attaches to a fresh
    // window. The persisted `lastSessionId` in localStorage drives the
    // replay-on-mount path that M03.E added; the graph reconstructs from
    // the drone's signal log.
    await browser.reloadSession();
    const agentNode = $('[data-testid^="agent-node-"]');
    await agentNode.waitForDisplayed({ timeout: 15_000 });
    expect(await agentNode.getAttribute('data-status')).to.equal('complete');
  });
});
