// Tauri 2.x desktop-shell E2E regression test — M09.B (the vertical
// slice's second stage: make an authored agent capable).
//
// Closes the M09 capability gap (docs/build-prompts/M09-workbench-vertical-slice.md
// Stage M09.B): builderAgent (builderStore.ts:145) omitted `capabilities`
// (required per agent.v1.json:9), and NodeConfigPanel (:105-146) had no
// capability surface — so a canvas agent had no file_access and EVERY Write
// was denied at the L2 enforcer (E-02 capability_live_tool.rs proves the
// in-scope/out-of-scope split). M09.B mints a minimal-valid Capabilities on
// drop and adds a File-access editor (Read/Write glob lists) writing
// capabilities.file_access via updateNode. Declaration-only — the enforcer
// (unchanged) consumes the grant at run time; the *enforced* write lands at
// M09.D.
//
// Why this can only live as a tauri-driver real-app test (ADR-0021 /
// gotcha #82): the drag → addNode → projection → onNodeClick selection path
// runs against the WebView2 / WebKitGTK DOM the Playwright/Vitest renderer
// suite (plain Chromium, @tauri-apps/api mocked) is blind to (the M08.5 🔴-1
// dragDropEnabled fix). Only the assembled app exercises create → select →
// edit file_access end to end.
//
// Drag mechanism (quoted from builder_create_agent.e2e.ts — itself quoting
// builder_drag.e2e.ts / gotcha #32): the W3C WebDriver Actions pointer drag no
// longer synthesizes HTML5 dragstart on Chromium 148+, so the drag is driven
// by JavaScript DragEvent dispatch via browser.executeScript, threading one
// DataTransfer across dragstart → dragover → drop → dragend.
//
// Selectors need no backend data: the Agents tab, the "+ New agent" item, and
// the File-access editor all exist on a cold-start emptyFramework, so the test
// depends on no Anthropic key, no skills.lock, no MCP server — it runs
// unconditionally in CI.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

// One DataTransfer threaded across dragstart → dragover → drop so the Palette's
// setData('application/x-builder-node', …) survives into BuilderCanvas onDrop.
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

describe('Builder — grant an agent file access (M09.B real-app)', () => {
  it('file_access_editor_writes_capabilities_file_access_on_the_selected_agent', async () => {
    // Switch to Builder — the ViewSwitch mounts unconditionally after first
    // paint (key-independent; the builder_create_agent.e2e.ts precedent).
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();

    await $('[data-testid="palette-tab-agents"]').waitForDisplayed({ timeout: 5_000 });
    await $('[data-testid="palette-tab-agents"]').click();
    await $('[data-testid="palette-item-agent-1"]').waitForDisplayed({ timeout: 5_000 });
    await $('[data-testid="builder-canvas"]').waitForDisplayed({ timeout: 5_000 });

    // Create the agent: drag "+ New agent" → addNode → BuilderAgentNode.
    await browser.executeScript(DRAG_DISPATCH, ['[data-testid="palette-item-agent-1"]']);
    const node = $('[data-testid="builder-agent-node-agent-1"]');
    await node.waitForExist({ timeout: 5_000 });
    await node.waitForDisplayed({ timeout: 5_000 });

    // Select it — the real onNodeClick (BuilderCanvas.tsx:111) sets
    // selectedNodeId, mounting NodeConfigPanel (BuilderShell).
    await node.click();
    await $('[data-testid="builder-node-config"]').waitForDisplayed({ timeout: 5_000 });

    // Grant a Write glob + a Read glob through the real File-access editor.
    const writeInput = $('[data-testid="node-config-add-fa-write-input"]');
    await writeInput.waitForDisplayed({ timeout: 5_000 });
    await writeInput.setValue('out/**');
    await $('[data-testid="node-config-add-fa-write"]').click();

    const readInput = $('[data-testid="node-config-add-fa-read-input"]');
    await readInput.setValue('data/**');
    await $('[data-testid="node-config-add-fa-read"]').click();

    // Read the grant back off the real builder store (window.__builderStore,
    // exposed by App.tsx:79 for the e2e seam — the builder_load_aria
    // precedent). The editor wrote capabilities.file_access via updateNode.
    const fileAccess = await browser.execute(`
      var fw = window.__builderStore.getState().framework;
      var agent = fw.agents.find(function (a) { return a.id === 'agent-1'; });
      return agent ? agent.capabilities.file_access : null;
    `);
    expect(fileAccess).to.deep.equal({ read: ['data/**'], write: ['out/**'] });

    // And the editor's lists render the granted globs (state-visible — DESIGN.md
    // principle 2).
    expect(await $('[data-testid="node-config-fa-write"]').getText()).to.contain('out/**');
    expect(await $('[data-testid="node-config-fa-read"]').getText()).to.contain('data/**');
  });
});
