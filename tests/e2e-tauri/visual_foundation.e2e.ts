// Tauri 2.x desktop-shell E2E regression test — M08.8 Stage B
// (Light Instrument visual foundation).
//
// The close bar for B is the REAL Tauri app driven via tauri-driver +
// the maintainer "does it look like the mockup?" watch (rule 11 /
// ADR-0021) — Playwright-mock-green does not close this stage. A's
// Stage-D review surfaced three DESIGN.md divergences the Playwright job
// is structurally blind to: the dark canvas, no three-pane shell, dark
// node cards. They escaped because no gate ran the real app. This test
// asserts the ported system on the running app:
//   1. the canvas is the light dotted field (not the dark M03 #0e1014);
//   2. the 52/232/360 three-pane shell is present;
//   3. a node renders as a light card (surface-0), not a dark card;
//   4. the MCP Add modal (migrated onto the Modal primitive) shows the
//      untruncated "Cancel" label (closes #24);
//   5. a pushed Toast appears bottom-right and auto-dismisses.
//
// Trace + store injection mirrors execution_view.e2e.ts: the production
// applyEvent path a real run drives, minus the network. The toast push
// goes through window.__toastStore, exposed for the same reason
// App.tsx exposes window.__graphStore (the store carries no secrets).
//
// WebdriverIO v9 chainable convention (gotcha #38): $()/$$() are
// chainable, not awaited intermediately; browser.execute takes a script
// string + args, the body reads arguments[0].
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

const TRACE = [
  {
    type: 'agent_spawned',
    agent_id: 'a1',
    agent_name: 'reader',
    parent_id: null,
    session_id: 's1',
  },
  { type: 'stream_text', agent_id: 'a1', text: 'reading the manifest.' },
  {
    type: 'tool_invoked',
    agent_id: 'a1',
    tool_name: 'Read',
    source: 'builtin',
    input: { path: 'Cargo.toml' },
  },
  {
    type: 'tool_result',
    agent_id: 'a1',
    tool_name: 'Read',
    output: '[package]',
    duration_ms: 9,
  },
  { type: 'agent_complete', agent_id: 'a1', tokens_total: 42 },
];

describe('Light Instrument visual foundation — M08.8.B (real-app, DESIGN.md)', () => {
  it('paints_the_light_three_pane_shell_with_light_node_cards', async () => {
    await browser.waitUntil(async () => browser.execute('return !!window.__graphStore'), {
      timeout: 10_000,
      timeoutMsg: 'window.__graphStore not exposed — app launch failed (harness error)',
    });
    await browser.execute(
      `
      const store = window.__graphStore.getState();
      store.clear();
      for (const ev of arguments[0]) store.applyEvent(ev);
      `,
      TRACE,
    );

    // 2 — the three-pane shell is present in the running app.
    await $('[data-testid="rail-left"]').waitForDisplayed({ timeout: 10_000 });
    await $('[data-testid="pane-center"]').waitForDisplayed({ timeout: 5_000 });
    await $('[data-testid="rail-right"]').waitForDisplayed({ timeout: 5_000 });

    // 1 — the canvas is the light dotted field, not the dark M03 #0e1014
    // (rgb(14,16,20)). canvas-bg is #f1f4fa = rgb(241,244,250).
    const canvas = $('[data-testid="graph-canvas"]');
    await canvas.waitForDisplayed({ timeout: 5_000 });
    const canvasBg = (await canvas.getCSSProperty('background-color')).value ?? '';
    // WebView2 serializes rgba without spaces, e.g. rgba(241,244,250,1).
    expect(canvasBg, 'canvas must be light, not the dark M03 field').to.not.include('14,16,20');
    expect(canvasBg).to.include('241,244,250');

    // 3 — a node is a light card (surface-0 = rgb(255,255,255)), not dark.
    const node = $('[data-testid="tool-node-a1-Read"]');
    await node.waitForDisplayed({ timeout: 5_000 });
    const nodeBg = (await node.getCSSProperty('background-color')).value ?? '';
    expect(nodeBg, 'node card must be the light surface-0').to.include('255,255,255');
  });

  it('opens_the_mcp_add_modal_with_an_untruncated_cancel_label', async () => {
    const addButton = $('[data-testid="mcp-add-server-button"]');
    await addButton.waitForDisplayed({ timeout: 10_000 });
    await addButton.click();

    const modal = $('[data-testid="mcp-server-add-modal"]');
    await modal.waitForDisplayed({ timeout: 5_000 });

    // #24 was a "Canc" truncation in the hand-rolled modal; migrated onto
    // the Modal primitive (z-300, 86vh, bounded), the label is complete.
    const cancel = $('[data-testid="mcp-add-cancel"]');
    await cancel.waitForDisplayed({ timeout: 5_000 });
    expect(await cancel.getText()).to.equal('Cancel');

    await cancel.click();
    await modal.waitForExist({ reverse: true, timeout: 5_000 });
  });

  it('shows_a_toast_bottom_right_that_auto_dismisses', async () => {
    await browser.waitUntil(async () => browser.execute('return !!window.__toastStore'), {
      timeout: 5_000,
      timeoutMsg: 'window.__toastStore not exposed',
    });
    await browser.execute(`
      window.__toastStore.getState().push({ kind: 'ok', title: 'Budget cap saved' });
    `);

    const stack = $('[data-testid="toast-stack"]');
    await stack.waitForDisplayed({ timeout: 5_000 });
    await browser.waitUntil(async () => (await stack.getText()).includes('Budget cap saved'), {
      timeout: 3_000,
      timeoutMsg: 'pushed toast did not appear',
    });

    // Auto-dismiss (~4.2s) — non-blocking, no manual control needed.
    await browser.waitUntil(async () => !(await stack.getText()).includes('Budget cap saved'), {
      timeout: 8_000,
      timeoutMsg: 'toast did not auto-dismiss',
    });
  });
});
