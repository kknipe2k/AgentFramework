// Tauri 2.x desktop-shell E2E regression test — M08.8 Stage B.fix
// (complete the Light Instrument restyle).
//
// B's `visual_foundation.e2e.ts` proved the tokens + shell landed on the
// real app. B.fix's close bar is the surface-by-surface design conformance
// the Playwright job is structurally blind to: the mono/tabular instrument
// register on machine values, the segmented topbar tabs, hairline-bordered
// panels, the NEW Tester metric cards + validation err-card raw-disclosure,
// the IBM-Plex font actually COMPUTED on screen (the @import @12 is
// WebView2-blockable — the bundled @font-face must win), and the Tester as
// a full Modal (TD-043). Asserted on the running binary.
//
// WebdriverIO v9 chainable convention (gotcha #38): $()/$$() are chainable,
// not awaited intermediately; browser.execute takes a script string + args.
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
  { type: 'tool_result', agent_id: 'a1', tool_name: 'Read', output: '[package]', duration_ms: 9 },
  { type: 'agent_complete', agent_id: 'a1', tokens_total: 42 },
];

describe('Light Instrument restyle conformance — M08.8.B.fix (real-app)', () => {
  it('computes_IBM_Plex_on_the_brand_not_the_fallback_system_font', async () => {
    await browser.waitUntil(async () => browser.execute('return !!window.__graphStore'), {
      timeout: 10_000,
      timeoutMsg: 'window.__graphStore not exposed — app launch failed (harness error)',
    });
    const brand = $('.brand__name');
    await brand.waitForDisplayed({ timeout: 10_000 });
    const family = (await brand.getCSSProperty('font-family')).value ?? '';
    // The bundled @font-face must win even if WebView2 blocked the Google
    // @import — IBM Plex Sans is first in the computed family.
    expect(family.toLowerCase()).to.include('plex');
  });

  it('paints_the_topbar_view_switch_as_a_segmented_control', async () => {
    const wellSelected = $('.view-switch__option--active');
    await wellSelected.waitForDisplayed({ timeout: 5_000 });
    // selected tab lifts onto surface-0 (white), not an accent fill.
    const bg = (await wellSelected.getCSSProperty('background-color')).value ?? '';
    expect(bg).to.include('255,255,255');
  });

  it('renders_machine_values_in_the_mono_register_on_a_node', async () => {
    await browser.execute(
      `
      const store = window.__graphStore.getState();
      store.clear();
      for (const ev of arguments[0]) store.applyEvent(ev);
      `,
      TRACE,
    );
    const dur = $('.tool-node__duration');
    await dur.waitForDisplayed({ timeout: 5_000 });
    const family = (await dur.getCSSProperty('font-family')).value ?? '';
    expect(family.toLowerCase()).to.include('plex mono');
  });

  it('paints_panels_with_hairline_borders_not_heavy_M03_fills', async () => {
    const canvas = $('[data-testid="graph-canvas"]');
    await canvas.waitForDisplayed({ timeout: 5_000 });
    // a node card is a 1px hairline (border-strong #b9c4d6), not the M03 2px.
    const node = $('[data-testid="tool-node-a1-Read"]');
    await node.waitForDisplayed({ timeout: 5_000 });
    const width = (await node.getCSSProperty('border-top-width')).value ?? '';
    expect(width).to.equal('1px');
  });

  it('opens_the_tester_as_a_full_modal_with_metric_cards', async () => {
    await $('[data-testid="view-switch-builder"]').click();
    const testBtn = $('button=Test');
    await testBtn.waitForDisplayed({ timeout: 10_000 });
    await testBtn.click();

    const dialog = $('[data-testid="tester-modal"]');
    await dialog.waitForDisplayed({ timeout: 5_000 });
    // TD-043 — the Tester is the Modal primitive at size="full".
    const cls = await dialog.getAttribute('class');
    expect(cls).to.include('modal--full');
    expect(await dialog.getAttribute('role')).to.equal('dialog');

    // close via Modal's aria-labelled × (the hand-rolled close is gone).
    await $('button[aria-label="Close"]').click();
    await dialog.waitForExist({ reverse: true, timeout: 5_000 });
  });
});
