// Tauri 2.x desktop-shell E2E regression test — M08.6 Stage E
// (Palette populated from the loaded framework's resolved artifacts;
// ADR-0022).
//
// Closes the M08.6 "if there are defined agents they do not show up in
// the tools as sharable" IRL observation: after Stage B's loader
// resolves examples/aria/'s {id,path} agents to inline and Stage D
// lays them out on the canvas, the Palette's Agents tab still does NOT
// surface them as reusable drag sources. Stage E adds
// builderStore.framework as a third source for paletteItemsForTab,
// de-duplicated by (kind, ref) against built-ins + installed
// artifacts, so a loaded framework's defined agents / tools / skills
// appear in the matching tab as drag-source Palette items.
//
// Why this can only live as a tauri-driver real-app test (gotcha #66):
// the IRL observation surfaced exactly because no gate ran the real
// app — a Vitest test with mocked invoke + a hand-built framework can
// pass without the loader ever running. The assembled
// load_framework → applyLoadedFramework → Palette-source chain runs
// only when the shipped backend resolves examples/aria/ end-to-end.
//
// Stage 0 manual real-app walk is deferred to Stage V per CLAUDE.md
// §20 (the M08.6.D precedent) — this test IS the assembled regression
// V's central duty re-verifies.
//
// The OS-native directory picker that Inspector.tsx's Load button
// opens cannot be driven by tauri-driver (WebDriver controls the
// WebView, not the OS file dialog). The test invokes the
// `load_framework` Tauri command directly + routes the result through
// the `applyLoadedFramework` store action — the EXACT code path
// Inspector.onLoad runs, minus the dialog click. Established at
// Stage D (`tests/e2e-tauri/builder_load_aria.e2e.ts`); the
// `window.__builderStore` expose lives at `src/App.tsx:61`.
//
// Stage M08.6.A authored the construction-reachability wire
// "resolve_to_palette" as inputs_reachable="false"; this test pins
// the post-E inversion to "true" at the assembled-app boundary.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, $$, browser } from '@wdio/globals';
import { expect } from 'chai';
import { resolve } from 'node:path';

// The archetype framework. `process.cwd()` is the repo root: wdio runs
// from there via the `test:e2e:tauri` npm script (matching
// `wdio.conf.ts` APP_BIN_PATH). Stage B's loader resolves ARIA's 8
// {id,path} agents to inline; Stage E surfaces each as a Palette item.
const ARIA_DIR = resolve(process.cwd(), 'examples', 'aria');

describe('Builder Palette surfaces loaded-framework agents — M08.6.E (real-app regression)', () => {
  it('loading_aria_populates_the_palette_agents_tab_with_aria_agents', async () => {
    // Switch to Builder mode. Mirrors the Stage D precedent — the
    // ViewSwitch mounts unconditionally after first paint.
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();
    await $('[data-testid="builder-canvas"]').waitForDisplayed({ timeout: 5_000 });

    // Drive the load via the production Tauri command + the
    // load-applying store action — the OS directory picker is
    // non-driveable, the underlying load path it triggers IS the
    // production path under test. On `main` pre-E the load resolves
    // (Stage B) + the canvas lays out (Stage D) but the Palette
    // Agents tab still has no framework-sourced items — the
    // red-phase right-reason failure per CLAUDE.md §5.
    await browser.execute(
      `
      return (async () => {
        const dir = arguments[0];
        // M09.5.A (TD-051): load_framework now confines its path to the
        // dialog-registered roots. The OS picker is non-driveable, so
        // register the dir through pick_framework_dir's test-mode arm
        // (AGENT_RUNTIME_E2E is set by wdio.conf.ts) — the faithful
        // register-then-load production flow minus the dialog click.
        await window.__TAURI_INTERNALS__.invoke('pick_framework_dir', { testDir: dir });
        const loaded = await window.__TAURI_INTERNALS__.invoke('load_framework', { dir });
        window.__builderStore.getState().applyLoadedFramework(loaded.framework);
      })();
      `,
      ARIA_DIR,
    );

    // Default Palette tab is Tools; switch to Agents to see ARIA's
    // resolved agents.
    const agentsTab = $('[data-testid="palette-tab-agents"]');
    await agentsTab.waitForDisplayed({ timeout: 5_000 });
    await agentsTab.click();

    // ARIA's framework.json declares 8 agents (orchestrator, router,
    // planner, analyzer, implementer, verify-app, simplifier,
    // report-writer); Stage B resolves them to inline; Stage E
    // surfaces each as a drag-source Palette item.
    //
    // The orchestrator is the canonical root agent
    // (session_root_agent = "orchestrator") — its presence is the
    // discriminator between the pre-E state (no framework-sourced
    // agents in the Palette) and the post-E state.
    const orchestratorItem = $('[data-testid="palette-item-orchestrator"]');
    await orchestratorItem.waitForDisplayed({
      timeout: 5_000,
      timeoutMsg:
        'expected the Palette Agents tab to surface ARIA orchestrator after loading examples/aria/',
    });
    expect(await orchestratorItem.isDisplayed()).to.equal(true);

    // Phase doc E.3: framework-sourced items are distinguishable via
    // a data-source attribute so the user can see which artifacts
    // come from the open framework (drag payload remains identical).
    expect(await orchestratorItem.getAttribute('data-source')).to.equal('framework');

    // Sanity: the assembled load → resolve → Palette-source chain
    // surfaces multiple ARIA agents, not just one accidentally.
    const frameworkAgentItems = $$('[data-testid^="palette-item-"][data-source="framework"]');
    expect(
      await frameworkAgentItems.length,
      'the Palette Agents tab must surface multiple framework-sourced agents after loading ARIA',
    ).to.be.greaterThan(1);
  });
});
