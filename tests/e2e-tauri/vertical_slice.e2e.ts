// Tauri 2.x real-app E2E — M09.D the vertical slice, assembled end-to-end.
//
// Drives the BUILT Tauri app via `tauri-driver` + WebdriverIO v9 (ADR-0021;
// the merge-blocking `e2e-tauri-driver` job). This is the assembled close of
// the first ADR-0032 vertical slice: a from-scratch canvas-authored agent +
// file_access.write + an installed MCP server's tool, RUN in the Tester, must
// write a real file within scope from real MCP data — and be denied outside
// it (CLAUDE.md §4 rule 11: the on-disk effect, not an event).
//
// The falsifiable hypothesis (CLAUDE.md v1.8 assembled mandate): a
// canvas-authored agent — capabilities.file_access + an MCP tool authored
// through M09.A–C, serialized across the Tauri IPC into `framework_doc` —
// runs EXACTLY as a hand-written-JSON framework does. The individual wires
// are already present and confirmed (the MCP dispatcher + the tracked tier,
// commands.rs:1769-1775/:1758; the enforcer built from the framework's agents
// via grant_framework_capabilities → AgentSdk::with_capability_wiring,
// tester.rs:36/445). The genuine unknown is the COMPOSITION — no
// canvas-authored framework had ever produced a *runnable* framework_doc.
//
// The composition defect this disproves (the red-phase right-reason): on the
// pre-M09.D build, authoring an agent on the canvas left
// `framework.session_root_agent` EMPTY (emptyFramework() opens with '' and no
// reducer ever set it). The run path picks the dispatch agent off
// session_root_agent (agent_sdk.rs:780), so a canvas-authored framework was
// NOT runnable as hand-written JSON. M09.D roots the session on the first
// authored agent — the composed authored→serialize→run framework_doc now
// matches a hand-written one.
//
// Two layers, by close bar:
//   1. Composition (KEY-INDEPENDENT, runs in CI): author the whole framework
//      through the real UI surfaces and read the assembled framework_doc back
//      off `window.__builderStore` — it must be a complete, RUNNABLE doc
//      (session_root_agent rooted; role + file_access.write granted; the MCP
//      tool in allowed_tools + capabilities.tools_called). This is the
//      composition hypothesis, CI-runnable.
//   2. The real run (KEY+SERVER-gated; this.skip() in CI): with a live key,
//      Run the authored framework in the Tester and observe it execute
//      through the real test_framework path (the assembled-run-reachability
//      check). The on-disk file within scope + the out-of-scope denial need
//      a live model AND an installed MCP server — the maintainer real-app IRL
//      is the AUTHORITATIVE close (rule 11 / ADR-0021); this regression
//      guards the composition + the run reaches test_framework.
//
// Drag mechanism (quoted from builder_file_access.e2e.ts / builder_drag.e2e.ts,
// gotcha #32): the W3C WebDriver Actions pointer drag no longer synthesizes
// HTML5 dragstart on Chromium 148+, so the drag is driven by JavaScript
// DragEvent dispatch via browser.executeScript, threading one DataTransfer
// across dragstart → dragover → drop → dragend.
//
// WebdriverIO v9 note: `$()` returns a chainable, not a promise — call
// methods on it directly. Per <https://webdriver.io/docs/api/element>.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

const hasAnthropicKey = (process.env.ANTHROPIC_API_KEY ?? '').trim().length > 0;

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

// The canonical `<server>__<tool>` ref a `source:'mcp'` Palette item carries
// (a real server's tools come from mcp_list_server_tools, exercised by the
// maintainer IRL). CI has no connected server, so the attach is driven
// through the same real reducers a palette drop + Agent→Tool edge runs
// (the builder_mcp_tool.e2e.ts precedent).
const MCP_TOOL_REF = 'fs__read_file';

describe('Tauri real-app E2E — M09.D the vertical slice (assembled)', () => {
  afterEach(async () => {
    // Tests share one app session; close the Tester so a left-open modal
    // scrim cannot intercept the next test's clicks (tester_verdict.e2e.ts
    // V🔴#1 precedent).
    await browser.execute(`
      if (window.__builderStore) window.__builderStore.getState().closeTester();
    `);
  });

  it('a_canvas_authored_framework_composes_into_a_runnable_framework_doc', async () => {
    // Switch to Builder and reset to a clean cold-start document so the
    // "+ New agent" item carries the fresh agent-1 ref.
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();
    await browser.waitUntil(async () => browser.execute('return !!window.__builderStore'), {
      timeout: 5_000,
      timeoutMsg: 'window.__builderStore not exposed',
    });
    await browser.execute(
      'window.__builderStore.setState(window.__builderStore.getInitialState(), true);',
    );

    await $('[data-testid="palette-tab-agents"]').waitForDisplayed({ timeout: 5_000 });
    await $('[data-testid="palette-tab-agents"]').click();
    await $('[data-testid="palette-item-agent-1"]').waitForDisplayed({ timeout: 5_000 });
    await $('[data-testid="builder-canvas"]').waitForDisplayed({ timeout: 5_000 });

    // M09.A — author the agent from scratch (real drag → addNode).
    await browser.executeScript(DRAG_DISPATCH, ['[data-testid="palette-item-agent-1"]']);
    const node = $('[data-testid="builder-agent-node-agent-1"]');
    await node.waitForExist({ timeout: 5_000 });
    await node.waitForDisplayed({ timeout: 5_000 });
    await node.click();
    await $('[data-testid="builder-node-config"]').waitForDisplayed({ timeout: 5_000 });

    // Give the agent a role (a runnable agent names what it does).
    const roleInput = $('[data-testid="node-config-role"]');
    await roleInput.waitForDisplayed({ timeout: 5_000 });
    await roleInput.setValue('writer');

    // M09.B — grant the file_access.write scope that makes its Write land,
    // plus a read scope, through the real File-access editor.
    const writeInput = $('[data-testid="node-config-add-fa-write-input"]');
    await writeInput.waitForDisplayed({ timeout: 5_000 });
    await writeInput.setValue('out/**');
    await $('[data-testid="node-config-add-fa-write"]').click();
    const readInput = $('[data-testid="node-config-add-fa-read-input"]');
    await readInput.setValue('data/**');
    await $('[data-testid="node-config-add-fa-read"]').click();

    // M09.C — attach a real MCP server's tool. CI has no connected server, so
    // the attach runs through the same real reducers a palette drop + the
    // Agent→Tool edge drives (allowed_tools + capabilities.tools_called).
    await browser.execute(
      `
      var store = window.__builderStore.getState();
      store.addNode('tool', arguments[0], { x: 240, y: 0 });
      store.connectEdge('agent:agent-1', 'tool:' + arguments[0]);
      `,
      MCP_TOOL_REF,
    );

    // The composition hypothesis: the canvas-authored framework_doc is a
    // complete, RUNNABLE document — exactly what a hand-written framework is.
    const doc = (await browser.execute(`
      var fw = window.__builderStore.getState().framework;
      var agent = fw.agents.find(function (a) { return a.id === 'agent-1'; });
      return {
        sessionRoot: fw.session_root_agent,
        role: agent ? agent.role : null,
        fileAccess: agent ? agent.capabilities.file_access : null,
        allowedTools: agent ? agent.allowed_tools : null,
        toolsCalled: agent ? agent.capabilities.tools_called : null,
      };
    `)) as {
      sessionRoot: string;
      role: string | null;
      fileAccess: { read: string[]; write: string[] } | null;
      allowedTools: string[] | null;
      toolsCalled: string[] | null;
    };

    // The session is rooted on the authored agent — the red-phase
    // right-reason failure (pre-M09.D this is '' and the framework cannot
    // run as hand-written JSON does).
    expect(doc.sessionRoot).to.equal('agent-1');
    expect(doc.role).to.equal('writer');
    expect(doc.fileAccess).to.deep.equal({ read: ['data/**'], write: ['out/**'] });
    expect(doc.allowedTools).to.deep.equal([MCP_TOOL_REF]);
    expect(doc.toolsCalled).to.deep.equal([MCP_TOOL_REF]);
  });

  it('the_authored_framework_runs_through_the_assembled_tester', async function () {
    // KEY+SERVER-gated. The on-disk file within scope + the out-of-scope
    // denial need a live model AND an installed MCP server — the maintainer
    // real-app IRL is the authoritative close (rule 11). Here, with a key,
    // assert the authored framework reaches test_framework and produces an
    // observable verdict (the assembled-run-reachability check).
    if (!hasAnthropicKey) {
      this.skip();
    }
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();
    await browser.waitUntil(async () => browser.execute('return !!window.__builderStore'), {
      timeout: 5_000,
      timeoutMsg: 'window.__builderStore not exposed',
    });

    // Author the runnable single-agent framework directly on the store (the
    // composition is proven by the test above; here we exercise the run).
    await browser.execute(
      `
      window.__builderStore.setState(window.__builderStore.getInitialState(), true);
      var s = window.__builderStore.getState();
      s.addNode('agent', 'agent-1', { x: 0, y: 0 });
      s.updateNode('agent:agent-1', {
        role: 'writer',
        capabilities: {
          tools_called: [arguments[0]],
          skills_loaded: [],
          file_access: { read: [], write: ['out/**'] },
          network: [], shell: false, spawn_agents: [],
        },
        allowed_tools: ['Write', arguments[0]],
      });
      window.__builderStore.getState().openTester();
      `,
      MCP_TOOL_REF,
    );

    const testerModal = $('[data-testid="tester-modal"]');
    await testerModal.waitForDisplayed({ timeout: 5_000 });
    const taskInput = $('[data-testid="tester-task-input"]');
    await taskInput.setValue('Write the file out/report.md containing the word "hello".');
    const runButton = $('[data-testid="tester-run"]');
    await runButton.click();

    // The run reaches the real test_framework path and returns a verdict —
    // the authored framework executes (the on-disk effect is the IRL close).
    const verdict = $('[data-testid="tester-result-verdict"]');
    await verdict.waitForDisplayed({ timeout: 60_000 });
    expect((await verdict.getText()).trim().length).to.be.greaterThan(0);
  });
});
