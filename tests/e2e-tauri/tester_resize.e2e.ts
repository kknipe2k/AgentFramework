// Tauri 2.x real-app E2E — M09.D.fix the Tester resize/expand pass.
//
// Drives the BUILT Tauri app via `tauri-driver` + WebdriverIO v9 (ADR-0021;
// the merge-blocking `e2e-tauri-driver` job). The M09.D maintainer IRL found
// the Tester run unobservable in a usable window — the bundled DESIGN.md pass
// gives the Tester an expand/resize affordance over a scrollable watch pane
// (DESIGN.md Modals: "content scrolls within a bounded height"; the IRL
// re-verify needs the run pane expandable). This pins the affordance on the
// real app.
//
// Key-independent: the Tester opens on an empty cold-start framework and the
// expand affordance exists before any run — no Anthropic key, no MCP server.
//
// WebdriverIO v9 note: `$()` returns a chainable, not a promise — call
// methods on it directly. Per <https://webdriver.io/docs/api/element>.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

describe('Tauri real-app E2E — M09.D.fix Tester resize/expand', () => {
  afterEach(async () => {
    await browser.execute(`
      if (window.__builderStore) window.__builderStore.getState().closeTester();
    `);
  });

  it('the_tester_has_an_expand_affordance_that_grows_the_watch_pane', async () => {
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();
    await browser.waitUntil(async () => browser.execute('return !!window.__builderStore'), {
      timeout: 5_000,
      timeoutMsg: 'window.__builderStore not exposed',
    });
    await browser.execute('window.__builderStore.getState().openTester();');

    const testerModal = $('[data-testid="tester-modal"]');
    await testerModal.waitForDisplayed({ timeout: 5_000 });

    // The expand affordance exists (absent on the pre-D.fix Tester) …
    const expand = $('[data-testid="tester-expand"]');
    await expand.waitForDisplayed({ timeout: 5_000 });

    // … and toggling it grows the watch pane (a state-visible resize —
    // DESIGN.md principle 1 every action gives feedback).
    const watch = $('[data-testid="tester-watch"]');
    await watch.waitForDisplayed({ timeout: 5_000 });
    const before = (await watch.getAttribute('class')) ?? '';
    expect(before).to.not.include('tester-modal__watch--expanded');
    await expand.click();
    const after = (await watch.getAttribute('class')) ?? '';
    expect(after).to.include('tester-modal__watch--expanded');
  });
});
