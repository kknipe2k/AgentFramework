// Tauri 2.x desktop-shell E2E regression test — M09.A (the vertical
// slice's first stage: author an agent from scratch on the canvas).
//
// Closes the M09 authoring gap (docs/build-prompts/M09-workbench-vertical-slice.md
// Stage M09.A): a fresh project's Agents palette tab was empty
// (Palette.tsx:173-184 listed only installed + loaded-framework agents),
// so nothing could be authored on the canvas. M09.A prepends a
// "+ New agent" item carrying a fresh nextAgentRef id ({kind:'agent',
// ref:'agent-1'}) through the existing application/x-builder-node drag
// contract; BuilderCanvas.onDrop (:86-96) runs addNode('agent', 'agent-1',
// position) → applyDrop → builderAgent, and React Flow commits the
// projection as a BuilderAgentNode (testid builder-agent-node-agent-1).
//
// Why this can only live as a tauri-driver real-app test (ADR-0021 /
// gotcha #82): the Playwright/Vitest renderer suite runs in a plain
// Chromium with @tauri-apps/api mocked, blind to the Tauri-shell
// dragDropEnabled behaviour the M08.5 🔴-1 fix turned off so HTML5 DnD
// reaches the DOM. Only the assembled app exercises the full
// drag → addNode → projection path the slice rides.
//
// Drag mechanism (quoted from builder_drag.e2e.ts — M08.5.5 Stage A.fix,
// gotcha #32). The W3C WebDriver Actions API multi-step pointer drag no
// longer synthesizes HTML5 `dragstart` on Chromium 148+, so the drag is
// driven by JavaScript event dispatch via `browser.executeScript`, which
// reaches the WebView2 / WebKitGTK DOM directly. The dispatch sequence
// (dragstart on source → dragover then drop on target → dragend on
// source, threading one DataTransfer) is quoted from
// ePages-de/chromedriver-html5-dragdrop @ 201e5a2, modernized to
// `new DragEvent(type, { dataTransfer: new DataTransfer(), clientX,
// clientY, … })` per the HTML living spec — see builder_drag.e2e.ts for
// the full rationale.
//
// Selectors need no backend data: the Agents tab + the "+ New agent" item
// exist on a cold-start emptyFramework, so the test depends on no
// Anthropic key, no skills.lock, no MCP server — it runs unconditionally
// in CI.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, $$, browser } from '@wdio/globals';
import { expect } from 'chai';

// One DataTransfer threaded across dragstart → dragover → drop so the
// Palette's setData('application/x-builder-node', …) survives into the
// BuilderCanvas onDrop getData(...). `arguments[0]` is the source item's
// testid; the canvas is the drop target.
const DRAG_DISPATCH = `
  var sourceSel = arguments[0];
  var sourceElement = document.querySelector(sourceSel);
  var targetElement = document.querySelector('[data-testid="builder-canvas"]');
  if (!sourceElement) {
    throw new Error('source element not found in DOM: ' + sourceSel);
  }
  if (!targetElement) {
    throw new Error('builder-canvas not found in DOM');
  }
  var dataTransfer = new DataTransfer();
  var targetRect = targetElement.getBoundingClientRect();
  var dropClientX = Math.round(targetRect.left + targetRect.width / 2);
  var dropClientY = Math.round(targetRect.top + targetRect.height / 2);
  if (sourceElement.draggable) {
    sourceElement.dispatchEvent(new DragEvent('dragstart', {
      bubbles: true, cancelable: true, dataTransfer: dataTransfer,
    }));
  } else {
    throw new Error('trying to drag non-draggable element: ' + sourceSel);
  }
  targetElement.dispatchEvent(new DragEvent('dragover', {
    bubbles: true, cancelable: true, dataTransfer: dataTransfer,
    clientX: dropClientX, clientY: dropClientY,
  }));
  targetElement.dispatchEvent(new DragEvent('drop', {
    bubbles: true, cancelable: true, dataTransfer: dataTransfer,
    clientX: dropClientX, clientY: dropClientY,
  }));
  sourceElement.dispatchEvent(new DragEvent('dragend', {
    bubbles: true, cancelable: true, dataTransfer: dataTransfer,
  }));
`;

describe('Builder — author an agent from scratch (M09.A real-app)', () => {
  it('new_agent_palette_item_drags_a_fresh_agent_onto_the_canvas', async () => {
    // The ViewSwitch mounts unconditionally after first paint (independent
    // of has_api_key()), so switching to the Builder is key-independent.
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();

    // The Agents tab is empty pre-M09 — M09.A adds the "+ New agent" item.
    await $('[data-testid="palette-tab-agents"]').waitForDisplayed({ timeout: 5_000 });
    await $('[data-testid="palette-tab-agents"]').click();

    // The blank-create item — its testid suffix is the fresh ref (agent-1
    // on a cold-start framework).
    await $('[data-testid="palette-item-agent-1"]').waitForDisplayed({ timeout: 5_000 });
    await $('[data-testid="builder-canvas"]').waitForDisplayed({ timeout: 5_000 });

    // Pre-condition: no agent nodes on the canvas yet (cold-start project).
    const preDrop = await $$('[data-testid^="builder-agent-node-"]').length;
    expect(preDrop, 'no agent nodes should exist before the first create').to.equal(0);

    // Drag "+ New agent" → addNode('agent', 'agent-1', position) → the
    // projection commits a BuilderAgentNode with testid
    // builder-agent-node-agent-1 (BuilderAgentNode.tsx:55).
    await browser.executeScript(DRAG_DISPATCH, ['[data-testid="palette-item-agent-1"]']);
    const firstNode = $('[data-testid="builder-agent-node-agent-1"]');
    await firstNode.waitForExist({ timeout: 5_000 });
    await firstNode.waitForDisplayed({ timeout: 5_000 });

    // After the first create the Palette re-derives nextAgentRef from the
    // mutated framework, so the New-agent item now carries agent-2 — a
    // distinct id, proving repeated creates never collide.
    const secondItem = $('[data-testid="palette-item-agent-2"]');
    await secondItem.waitForDisplayed({ timeout: 5_000 });
    await browser.executeScript(DRAG_DISPATCH, ['[data-testid="palette-item-agent-2"]']);
    const secondNode = $('[data-testid="builder-agent-node-agent-2"]');
    await secondNode.waitForExist({ timeout: 5_000 });
    await secondNode.waitForDisplayed({ timeout: 5_000 });
  });
});
