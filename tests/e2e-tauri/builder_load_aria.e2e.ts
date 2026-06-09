// Tauri 2.x desktop-shell E2E regression test — M08.6 Stage D
// (canvas auto-layout on framework load; ADR-0022).
//
// Closes the M08.6 layout half: after Stage B's loader resolves
// {id,path} agents to inline, projectCanvasEdges already paints the
// edges, but every node still falls back to {0,0} in
// projectCanvasNodes (src/lib/builderStore.ts:210-211) — a pile, not a
// workflow. Stage D wires src/lib/layout.ts::layoutGraph (the existing
// dagre top-down layout the live GraphCanvas uses) into a NEW
// "load-applying" store action so a loaded framework seeds
// nodePositions with the dagre result.
//
// Why this can only live as a tauri-driver real-app test (gotcha #66 +
// #70): a {0,0} pile vs. a laid-out graph is a viewport / DOM-layout
// property unit tests do not simulate (Vitest + jsdom render React
// Flow without real layout); the @tauri-apps/api module mocks
// Playwright runs against don't exercise the assembled
// load → resolve → project → layout path either. Only the running app
// against the real loader exhibits the pile bug on `main` pre-D and
// the laid-out graph post-D.
//
// Stage 0 manual real-app walk is deferred to Stage V per CLAUDE.md
// §20 (the build agent's user-direction at red-phase) — this test IS
// the assembled regression that V's central duty re-verifies.
//
// The OS-native directory picker that Inspector.tsx's Load button
// opens cannot be driven by tauri-driver (WebDriver controls the
// WebView, not the OS file dialog). The test therefore invokes the
// `load_framework` Tauri command directly + routes the result through
// the new `applyLoadedFramework` store action — the EXACT code path
// Inspector.onLoad runs, minus the dialog click. This is the precedent
// established by App.tsx:50-52 exposing `window.__graphStore` for
// `tests/e2e/plan_approval.spec.ts`; Stage D adds the matching
// `window.__builderStore` expose for the Builder side.
//
// WebdriverIO v9 + executeScript notes (gotcha #38 chainable
// convention): script string + args array, the script body uses
// `arguments[0]` for the first arg. async script bodies wrap an IIFE
// + return the Promise; wdio awaits it.
//
// Stage M08.6.A authored the construction-reachability wire
// "resolve_to_canvas" as inputs_reachable="false"; this test pins the
// post-D inversion to "true" at the assembled-app boundary.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, $$, browser } from '@wdio/globals';
import { expect } from 'chai';
import { resolve } from 'node:path';

// The archetype framework — Stage B's loader resolves its 8 path-ref
// agents to inline; Stage D's auto-layout lays them out as a graph.
// `process.cwd()` is the repo root: wdio runs from there via the
// `test:e2e:tauri` npm script (matching `wdio.conf.ts` APP_BIN_PATH).
const ARIA_DIR = resolve(process.cwd(), 'examples', 'aria');

describe('Builder loads ARIA archetype — M08.6.D (real-app regression)', () => {
  it('loading_aria_seeds_distinct_node_positions_and_renders_edges', async () => {
    // Switch to Builder mode. The ViewSwitch mounts unconditionally
    // after first paint (independent of has_api_key — see the
    // builder_drag.e2e.ts precedent for why the SetupPanel wait is
    // brittle across machines).
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();

    // Wait for the Builder canvas to mount.
    await $('[data-testid="builder-canvas"]').waitForDisplayed({ timeout: 5_000 });

    // Drive the load via the production Tauri command + the new
    // load-applying store action. The OS file dialog Inspector.onLoad
    // opens is non-driveable by tauri-driver; the underlying load
    // path it triggers IS the production path under test.
    //
    // On `main` pre-D this throws (window.__builderStore + the
    // applyLoadedFramework action do not exist yet) — the red-phase
    // right-reason failure per CLAUDE.md §5 (a behavioral test fails
    // loudly when its required production export is missing).
    await browser.execute(
      `
      return (async () => {
        const dir = arguments[0];
        const loaded = await window.__TAURI_INTERNALS__.invoke('load_framework', { dir });
        window.__builderStore.getState().applyLoadedFramework(loaded.framework);
      })();
      `,
      ARIA_DIR,
    );

    // Stage B's resolver flips ARIA's 8 {id,path} agents to inline;
    // projectCanvasNodes / BuilderAgentNode renders each as
    // `[data-testid="builder-agent-node-<id>"]`. Wait for ≥2 to render
    // before checking layout — the dagre layout needs >1 node to mean
    // anything.
    await browser.waitUntil(
      async () => (await $$('[data-testid^="builder-agent-node-"]').length) >= 2,
      {
        timeout: 10_000,
        timeoutMsg: 'expected ≥2 builder agent nodes after loading examples/aria/',
      },
    );

    // Distinct-position check: a {0,0} pile (the pre-D state) places
    // every React Flow node at the same screen coordinates after
    // fitView; an actual layout spreads them. getBoundingClientRect()
    // accounts for React Flow's transform + zoom-to-fit, so identical
    // rects across all nodes is the discriminator. Gotcha #70: this
    // viewport-level property is invisible to unit tests.
    const layoutInfo = await browser.execute<{ count: number; allSamePosition: boolean }, []>(`
      const nodes = Array.from(document.querySelectorAll('[data-testid^="builder-agent-node-"]'));
      const rects = nodes.map((n) => {
        const r = n.getBoundingClientRect();
        return [r.left, r.top];
      });
      const first = rects[0];
      const allSamePosition = first
        ? rects.every(([x, y]) => x === first[0] && y === first[1])
        : true;
      return { count: rects.length, allSamePosition };
    `);

    expect(layoutInfo.count, 'at least 2 builder agent nodes must render').to.be.greaterThan(1);
    expect(
      layoutInfo.allSamePosition,
      'auto-layout must place nodes at distinct positions — a {0,0} pile is the pre-D bug',
    ).to.equal(false);

    // Stage B's inline-flip lets projectCanvasEdges paint edges for a
    // loaded framework's `allowed_skills` / `allowed_tools` / `spawns`
    // relationships. ARIA's orchestrator spawns sub-agents + allows
    // tools / skills, so the projection produces at least one edge.
    //
    // React Flow projects edges AFTER it measures node geometry; a cold
    // `$$('.react-flow__edge')` read races that measure→project pass
    // (M08.8.B.fix's node-resize — 22px glyph, 1px border — shifted the
    // timing enough to expose the race). Wait for the projection to land,
    // mirroring the node waitUntil @94.
    await browser.waitUntil(async () => (await $$('.react-flow__edge').length) > 0, {
      timeout: 10_000,
      timeoutMsg: 'expected ≥1 projected edge after loading examples/aria/',
    });
    const edgeCount = await $$('.react-flow__edge').length;
    expect(
      edgeCount,
      'at least one edge must render — ARIA orchestrator spawns sub-agents and allows tools',
    ).to.be.greaterThan(0);
  });
});
