// Tauri 2.x real-app E2E — M09.D.fix iteration 2, the Expand-grows-the-frame
// fix (DESIGN.md Modals: content scrolls within a bounded height — the frame,
// not chrome).
//
// The iteration-1 re-IRL found Expand grew the modal/canvas whitespace, not the
// watch FRAME (the canvas + OUTPUT + run-trace). This pins the v2 behavior on
// the real app: toggling Expand grows the run-view CONTENT (the graph pane),
// not just the surrounding container. Key-independent — the Tester opens on a
// cold-start framework and the affordance + frame exist before any run.
//
// WebdriverIO v9 note: `$()` returns a chainable, not a promise — call methods
// on it directly. Per <https://webdriver.io/docs/api/element>.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

describe('Tauri real-app E2E — M09.D.fix2 Expand grows the watch frame', () => {
  afterEach(async () => {
    await browser.execute(`
      if (window.__builderStore) window.__builderStore.getState().closeTester();
    `);
  });

  it('expanding_grows_the_run_view_content_not_just_the_chrome', async () => {
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();
    await browser.waitUntil(async () => browser.execute('return !!window.__builderStore'), {
      timeout: 5_000,
      timeoutMsg: 'window.__builderStore not exposed',
    });
    await browser.execute('window.__builderStore.getState().openTester();');
    await $('[data-testid="tester-modal"]').waitForDisplayed({ timeout: 5_000 });

    // The graph pane is the run-view content the frame must grow.
    const pane = $('[data-testid="tester-graph-pane"]');
    await pane.waitForDisplayed({ timeout: 5_000 });
    const before = (await pane.getSize()).height;

    const expand = $('[data-testid="tester-expand"]');
    await expand.waitForDisplayed({ timeout: 5_000 });
    await expand.click();

    // The CONTENT frame grows — not merely a container with added whitespace.
    // On iteration-1 the inner graph pane height did not change (the bug).
    await browser.waitUntil(async () => (await pane.getSize()).height > before + 100, {
      timeout: 5_000,
      timeoutMsg: 'the graph pane did not grow on Expand — the frame did not expand',
    });
    const after = (await pane.getSize()).height;
    expect(after).to.be.greaterThan(before + 100);
  });
});
