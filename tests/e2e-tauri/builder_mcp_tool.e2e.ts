// Tauri 2.x desktop-shell E2E regression test — M09.C (the vertical
// slice's third stage: attach a real MCP server's tool to an agent).
//
// Closes the M09.C wiring gap (docs/build-prompts/M09-workbench-vertical-slice.md
// Stage M09.C): an installed MCP server's tools are now Palette-draggable
// (source:'mcp', the canonical `<server>__<tool>` ref) and dropping one +
// drawing the Agent→Tool edge records the tool in BOTH the agent's
// `allowed_tools` (the offered tool — connectEdgeReducer:451) AND its
// `capabilities.tools_called` (the declared capability — common.v1.json
// "tools this artifact may invoke"). MCP dispatch itself already executes
// (agent_sdk.rs:884 try_mcp_dispatch) and the Tester already wires the
// dispatcher (commands.rs:1769-1775), so the M09.C gap was enumeration +
// palette + the recorded capability declaration — NOT execution wiring.
//
// Falsifiable hypothesis the red phase disproves (CLAUDE.md §6 v1.8): on
// `main` today, drawing an Agent→Tool edge in the running app records the
// tool in `allowed_tools` but NOT in `capabilities.tools_called` (the edge
// reducer only touched allowed_tools). After M09.C lands, both lists carry
// the canonical ref off the real `window.__builderStore`.
//
// Why store-driven rather than a palette drag: a `source:'mcp'` Palette
// item requires a *connected* MCP server, which CI has none of — the
// palette-from-a-real-server is the maintainer IRL close bar (close_gate
// real_app_irl). The recorded-state deliverable IS CI-runnable: it drives
// the real builder-store reducer in the assembled WebView2 app (the
// builder_file_access.e2e.ts window.__builderStore precedent), which the
// Palette/Vitest jsdom suite cannot stand in for. The drag → addNode →
// projection path itself is already covered by builder_create_agent /
// builder_file_access; the NEW behavior pinned here is the tools_called
// mirror on the assembled store. No Anthropic key, skills.lock, or MCP
// server needed — it runs unconditionally in CI.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

describe('Builder — attach an MCP server tool to an agent (M09.C real-app)', () => {
  it('wiring_an_agent_to_an_mcp_tool_records_allowed_tools_and_tools_called', async () => {
    // Switch to Builder so the store seam (window.__builderStore, App.tsx:79)
    // and the canvas are mounted (key-independent; the builder_file_access
    // precedent).
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();
    await $('[data-testid="builder-canvas"]').waitForDisplayed({ timeout: 5_000 });

    // Drive the real builder store in the running app: reset to a clean
    // document, create an agent, drop an MCP-canonical tool node (the ref a
    // source:'mcp' Palette item carries — a real server's tools come from
    // mcp_list_server_tools, exercised by the maintainer IRL), and draw the
    // Agent→Tool edge through the real connectEdge reducer.
    const recorded = await browser.execute(`
      window.__builderStore.setState(window.__builderStore.getInitialState(), true);
      var store = window.__builderStore.getState();
      store.addNode('agent', 'agent-1', { x: 0, y: 0 });
      store.addNode('tool', 'fs__read_file', { x: 240, y: 0 });
      store.connectEdge('agent:agent-1', 'tool:fs__read_file');
      var fw = window.__builderStore.getState().framework;
      var agent = fw.agents.find(function (a) { return a.id === 'agent-1'; });
      return {
        allowedTools: agent ? agent.allowed_tools : null,
        toolsCalled: agent ? agent.capabilities.tools_called : null,
      };
    `);

    // The Agent→Tool edge records the offered tool AND the declared
    // capability — both carry the dispatchable canonical `<server>__<tool>`.
    expect(recorded).to.deep.equal({
      allowedTools: ['fs__read_file'],
      toolsCalled: ['fs__read_file'],
    });
  });
});
