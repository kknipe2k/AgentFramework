// Tauri 2.x desktop-shell E2E regression test — M08.8 Stage A
// (live-graph execution view; TD-034).
//
// Closes TD-034: a real run's agent output was invisible in the running
// app — tool calls / agent text / completions surfaced only through
// `log_event_debug` at RUST_LOG=debug, never on the graph. Two halves:
//   1. `stream_text` was a graphStore no-op — the agent's reply text
//      rendered NOWHERE. M08.8.A buffers it into `outputBuffer` and the
//      Output rail renders it (DESIGN principle 1; mono register).
//   2. Tool nodes carried `toolName` but no {input, output} payload and
//      were not clickable — a Read node exposed nothing. M08.8.A retains
//      the payload and makes node-click → selection feed the Inspector.
//
// Why this can only live as a tauri-driver real-app test (rule 11 /
// ADR-0021): the three M08-IRL 🔴 escaped because no gate ran the real
// app; "the reply is visible in the Output rail" is an assembled-shell
// property the Playwright @tauri-apps/api mocks do not exercise. Only the
// running app against the real renderer composition exhibits the
// rail/inspector wiring.
//
// The real-Anthropic run path (smoke.e2e.ts tests 3–6) needs a key and is
// non-deterministic; this test instead injects a fixed built-in-tool
// trace through the exposed store singletons — the EXACT applyEvent path a
// live `agent_event` drives, minus the network. Precedent: App.tsx exposes
// `window.__graphStore` (live) + `window.__builderStore` (Builder) for the
// same reason; M08.8.A adds `window.__testGraphStore` (the Tester scope).
//
// WebdriverIO v9 + executeScript notes (gotcha #38): `$()`/`$$()` are
// chainable, not awaited intermediately; `browser.execute` takes a script
// string + args, the body reads `arguments[0]`.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

// A fixed built-in-tool trace: spawn → the agent streams a reply → it
// reads a file → the read returns the file body. The same shape a rung-1
// Read run emits over `agent_event`.
const TRACE = [
  {
    type: 'agent_spawned',
    agent_id: 'a1',
    agent_name: 'reader',
    parent_id: null,
    session_id: 's1',
  },
  { type: 'stream_text', agent_id: 'a1', text: 'The file begins with [package].' },
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
    output: '[package]\nname = "agent-runtime"',
    duration_ms: 9,
  },
  { type: 'agent_complete', agent_id: 'a1', tokens_total: 42 },
];

describe('Live-graph execution view — M08.8.A (real-app regression, TD-034)', () => {
  it('streams_agent_text_into_the_output_rail_and_exposes_tool_payload_on_click', async () => {
    // Default view is 'runtime' — the live graph + the rail mount on
    // launch. Drive the live store with the fixed trace (the production
    // applyEvent path a real run feeds).
    await browser.waitUntil(async () => browser.execute('return !!window.__graphStore'), {
      timeout: 10_000,
      timeoutMsg: 'window.__graphStore not exposed',
    });
    await browser.execute(
      `
      const store = window.__graphStore.getState();
      store.clear();
      for (const ev of arguments[0]) store.applyEvent(ev);
      `,
      TRACE,
    );

    // Half 1 — the agent's streamed reply is visible in the Output rail
    // (not only RUST_LOG). On `main` pre-A this element does not exist
    // (stream_text is dropped) — the red-phase right-reason failure.
    const outputRail = $('[data-testid="output-rail"]');
    await outputRail.waitForDisplayed({ timeout: 10_000 });
    expect(await outputRail.getText()).to.include('The file begins with [package].');

    // The Tool node renders the two-axis kind/status model: teal kind
    // (data-kind="tool") AND green complete status (data-status="complete").
    const toolNode = $('[data-testid="tool-node-a1-Read"]');
    await toolNode.waitForDisplayed({ timeout: 10_000 });
    expect(await toolNode.getAttribute('data-kind')).to.equal('tool');
    expect(await toolNode.getAttribute('data-status')).to.equal('complete');

    // Half 2 — clicking the Read tool node selects it; the Inspector shows
    // its input path + the file contents it returned. On `main` pre-A the
    // node is not clickable / carries no payload.
    //
    // The click is dispatched as a bubbling DOM event rather than a
    // WebDriver pointer click: with this synthetic 2-node trace, `fitView`
    // centers the graph so the lower (tool) node lands under the
    // bottom-right React Flow MiniMap, whose SVG intercepts a pointer
    // click at the node's center (a real-app finding the tauri-driver run
    // surfaced — minimap overlay; pairs with TD-027). A real multi-node
    // session spreads nodes clear of the corner minimap, and the
    // real-pointer AgentNode click is already covered by smoke.e2e.ts
    // test 4. A bubbling click still exercises the production
    // `onNodeClick` → `selectNode` wiring (React's synthetic handler
    // listens at the root), so this faithfully asserts "the tool node is
    // selectable and surfaces its payload".
    await browser.execute(`
      const el = document.querySelector('[data-testid="tool-node-a1-Read"]');
      el.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    `);
    const input = $('[data-testid="inspector-tool-input"]');
    const output = $('[data-testid="inspector-tool-output"]');
    await input.waitForDisplayed({ timeout: 5_000 });
    expect(await input.getText()).to.include('Cargo.toml');
    expect(await output.getText()).to.include('[package]');
  });

  it('mounts_the_same_rail_over_the_scoped_tester_store', async () => {
    // "Works over BOTH stores": the Tester mounts the SAME rail bound to
    // the scoped useTestGraphStore. Switch to Builder, open the Tester,
    // and inject the trace into the scoped store — the rail must surface
    // the streamed reply there too (the BDD's "run it in the app (Tester)"
    // watch surface).
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();

    await browser.waitUntil(async () => browser.execute('return !!window.__testGraphStore'), {
      timeout: 5_000,
      timeoutMsg: 'window.__testGraphStore not exposed',
    });
    await browser.execute(
      `
      window.__builderStore.getState().openTester();
      const store = window.__testGraphStore.getState();
      store.clear();
      for (const ev of arguments[0]) store.applyEvent(ev);
      `,
      TRACE,
    );

    const testerModal = $('[data-testid="tester-modal"]');
    await testerModal.waitForDisplayed({ timeout: 5_000 });
    const railInTester = testerModal.$('[data-testid="output-rail"]');
    await railInTester.waitForDisplayed({ timeout: 5_000 });
    expect(await railInTester.getText()).to.include('The file begins with [package].');
  });
});
