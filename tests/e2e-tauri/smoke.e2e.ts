// Tauri 2.x desktop-shell E2E smoke tests (M03.F).
//
// Drives the built Tauri app via `tauri-driver` + WebdriverIO v9. Six tests
// covering the M03 user-facing surfaces: app launch + setup panel; save-key
// flow + keychain indicator; smoke happy path with real Anthropic API call;
// click-to-inspect; SQL inspector execute; reload reconstructs graph from
// persisted signals.
//
// Tests 3 and 6 require a real Anthropic API key — the smoke session calls
// `/v1/messages` against Haiku 4.5. CI provides this via the
// `ANTHROPIC_TEST_KEY` repo secret (~$0.001 per CI run × 2 OS = ~$0.002 per
// PR run). Locally, the key is read from the user's OS keychain after
// running test 2 to set it through the UI; manual runs only.
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

  it('graph renders after smoke test (real Anthropic API call)', async () => {
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

  it('click AgentNode → InspectorPanel opens with node data', async () => {
    const agentNode = $('[data-testid^="agent-node-"]');
    await agentNode.click();
    const inspector = $('[role="dialog"][aria-label="node inspector"]');
    await inspector.waitForDisplayed({ timeout: 5_000 });
    const inspectorText = await inspector.getText();
    expect(inspectorText).to.include('agentId');
    expect(inspectorText).to.include('status');
  });

  it('SQL inspector executes SELECT * FROM signals LIMIT 5', async () => {
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
        const tableCount = await $$('table.sql-results').length;
        const errorCount = await $$('p[role="alert"]').length;
        return tableCount > 0 || errorCount > 0;
      },
      { timeout: 5_000, timeoutMsg: 'SQL inspector returned neither rows nor error' },
    );
  });

  it('reload reconstructs the graph from persisted signals', async () => {
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
