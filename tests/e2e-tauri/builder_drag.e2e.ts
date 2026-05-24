// Tauri 2.x desktop-shell E2E regression test — M08.5 Stage B.fix (🔴-1)
// + M08.5.5 Stage A.fix (mechanism revival).
//
// Closes docs/M08-irl-findings.md 🔴-1 — Palette → Builder-canvas
// drag-to-instantiate was dead in the real app on `main`. The renderer DnD
// contract is correct end-to-end (src/components/builder/Palette.tsx:138-156
// HTML5 `draggable` + `dataTransfer.setData('application/x-builder-node', …)`
// / src/components/builder/BuilderCanvas.tsx:81-97 onDragOver + onDrop reading
// the same MIME); the defect was the Tauri shell:
// `src-tauri/tauri.conf.json` declared the `main` window with no
// `dragDropEnabled` key, so it took Tauri 2.x's default `true` — per the
// official docs (https://v2.tauri.app/develop/tests/webdriver/ + the
// `WindowConfig.dragDropEnabled` docs), that "enables Tauri's internal
// drag-and-drop system and disables DOM drag and drop." On Windows
// `dragDropEnabled: false` is required to use HTML5 DnD on the frontend.
//
// Why this can only live as a tauri-driver real-app test (gotcha #66): a
// Playwright/Vitest test runs in a plain Chromium where HTML5 DnD works
// natively and `@tauri-apps/api` is mocked — the Tauri-shell
// `dragDropEnabled` default is invisible to it. Only the real running app
// exhibits the bug; only the Stage A.fix `tauri-driver` harness catches it.
//
// Drag mechanism (M08.5.5 Stage A.fix — gotcha #32 cross-stack
// integration, quoted from upstream not hand-authored). The W3C
// WebDriver Actions API multi-step pointer drag no longer synthesizes
// HTML5 `dragstart` on Chromium 148+ (the threshold tightened; a 5px
// intermediate `pointerMove` no longer trips it). The previous test
// (M08.5 commit 00e4f5e — the `it.skip` precursor) carried four
// candidate fixes documented at docs/M08-irl-findings.md 🟡 #11; the
// chosen path is JavaScript event dispatch via `browser.executeScript`,
// which bypasses the WebDriver / OS-driver layer entirely and reaches
// the WebView2 / WebKitGTK DOM directly.
//
// The DISPATCH SEQUENCE — dragstart on source, dragover then drop on
// target, dragend on source — is quoted from
// ePages-de/chromedriver-html5-dragdrop @ commit
// 201e5a26e926547368c6618a66a9010ee93ce245 (master HEAD, dated
// 2019-01-08), file dragdrop-chromedriver.js, the `dragstartIfDraggable`
// / `dragoverAndCheckIfValidDropTarget` / `drop` / `dragend`
// sub-helpers. The per-event constructor is modernized: the 2019
// upstream uses plain `new Event('dragstart', {bubbles: true})` (Chrome
// did not yet support the `DragEvent` constructor) and warns synthetic
// events have no usable `dataTransfer`; 2026 WebView2 + Chromium-Edge
// support `new DragEvent(type, { dataTransfer: new DataTransfer(),
// clientX, clientY, bubbles, cancelable })` natively per the HTML
// living spec (https://html.spec.whatwg.org/multipage/dnd.html#the-dragevent-interface),
// so the test constructs a single `DataTransfer` and threads it across
// the dragstart → dragover → drop chain. This lets the production
// handlers (Palette.tsx `setData` + BuilderCanvas.tsx `getData`)
// receive a real DataTransfer without modification — the cross-stack
// integration the rest of v0.1 depends on.
//
// The hypothesis the red phase falsifies (per CLAUDE.md §6 v1.8 +
// gotcha #82): on M08.5 + M08.5.5-pre `main` the `it.skip` decorates
// out and the regression is uncaught; replacing the W3C Actions API
// drag with `executeScript`-driven event dispatch routes around the
// Chromium 148+ dragstart-synthesis threshold and the test becomes a
// real green-pass against the assembled Tauri app — the
// `[data-testid="builder-tool-node-Read"]` element renders after the
// drop because BuilderCanvas's onDrop runs `addNode('tool', 'Read',
// position)` and React Flow commits the projection.
//
// Selectors chosen to require no backend data: the Tools tab is the
// default Palette tab; `Read` is a runtime built-in tool (BUILTIN_TOOLS
// in Palette.tsx); the Builder view is reachable via the ViewSwitch.
// The test therefore depends on no Anthropic key, no skills.lock, no
// MCP server — it runs unconditionally in CI.
//
// WebdriverIO v9 chainable convention (gotcha #38): `$()` returns a
// chainable, not a `PromiseLike`; method calls go on the chainable
// directly without intermediate `await`. The side-effect `webdriverio`
// import below pulls in the `WebdriverIO.Browser` augmentations.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, $$, browser } from '@wdio/globals';
import { expect } from 'chai';

describe('Builder palette drag — M08.5 🔴-1 (real-app regression)', () => {
  it('palette_drag_instantiates_a_canvas_node', async () => {
    // Wait for the ViewSwitch — it mounts unconditionally after
    // first paint, independent of `has_api_key()` (the SetupPanel
    // is hidden when the OS keychain already holds a key, e.g.
    // after a prior smoke-test #2 run on the same machine, so the
    // earlier `section[aria-label="api key setup"]` wait is
    // brittle across machines).
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 10_000 });
    await builderTab.click();

    // The drag source — `Read` is a BUILTIN_TOOL (Palette.tsx
    // BUILTIN_TOOLS) on the Tools tab (the default tab), so the
    // selector resolves with no backend data + no test fixture.
    await $('[data-testid="palette-item-Read"]').waitForDisplayed({ timeout: 5_000 });

    // The drop target.
    await $('[data-testid="builder-canvas"]').waitForDisplayed({ timeout: 5_000 });

    // Pre-condition: no tool nodes on the canvas yet.
    const preDrop = await $$('[data-testid^="builder-tool-node-"]').length;
    expect(preDrop, 'no tool nodes should exist before the drag').to.equal(0);

    // Dispatch sequence quoted verbatim from
    // https://github.com/ePages-de/chromedriver-html5-dragdrop/blob/201e5a26e926547368c6618a66a9010ee93ce245/dragdrop-chromedriver.js
    // (the `dragstartIfDraggable` / `dragoverAndCheckIfValidDropTarget`
    // / `drop` / `dragend` sub-helpers, lines ~95–174). Per-event
    // constructor modernized to `DragEvent` + `new DataTransfer()` for
    // WebView2 2026 — see file header for the full rationale.
    //
    // Single `executeScript` so the full chain runs in one browser-side
    // pass; each individual `dispatchEvent` is synchronous within the
    // script, so React's onDragStart / onDragOver / onDrop fire before
    // the next event is constructed.
    await browser.executeScript(
      `
      // Resolve source + target inside the browser context. Passing
      // wdio v9 chainables as executeScript args sends a deferred
      // proxy object, not a DOM Element — querySelector inside the
      // script is the documented browser-side path (and what the
      // upstream chromedriver-html5-dragdrop did via its inner
      // helpers that took raw element references).
      var sourceElement = document.querySelector(
        '[data-testid="palette-item-Read"]'
      );
      var targetElement = document.querySelector(
        '[data-testid="builder-canvas"]'
      );
      if (!sourceElement) {
        throw new Error('source element not found in DOM');
      }
      if (!targetElement) {
        throw new Error('target element not found in DOM');
      }

      // One DataTransfer threaded across the chain so the Palette's
      // setData('application/x-builder-node', …) survives into the
      // BuilderCanvas onDrop getData(...). The 2019 upstream omits
      // dataTransfer entirely (Chrome had no constructor); modern
      // WebView2 + Chromium-Edge support new DataTransfer() per the
      // HTML living spec.
      var dataTransfer = new DataTransfer();

      // Drop point at the center of the canvas (BuilderCanvas onDrop
      // uses e.clientX / e.clientY via screenToFlowPosition, so the
      // synthetic events need real client coordinates — the 2019
      // upstream wrote pageX / pageY post-construction which is
      // fragile on read-only properties; modern DragEvent accepts
      // clientX / clientY in the options bag).
      var targetRect = targetElement.getBoundingClientRect();
      var dropClientX = Math.round(targetRect.left + targetRect.width / 2);
      var dropClientY = Math.round(targetRect.top + targetRect.height / 2);

      // — upstream sub-helper: dragstartIfDraggable — dispatch
      // dragstart on the source element. Palette.tsx onDragStart runs
      // and calls dataTransfer.setData(…) + sets effectAllowed.
      if (sourceElement.draggable) {
        sourceElement.dispatchEvent(new DragEvent('dragstart', {
          bubbles: true,
          cancelable: true,
          dataTransfer: dataTransfer,
        }));
      } else {
        throw new Error('trying to drag non-draggable element');
      }

      // — upstream sub-helper: dragoverAndCheckIfValidDropTarget —
      // dispatch dragover on the target. BuilderCanvas.tsx onDragOver
      // runs and calls e.preventDefault() + sets dropEffect = 'copy'.
      targetElement.dispatchEvent(new DragEvent('dragover', {
        bubbles: true,
        cancelable: true,
        dataTransfer: dataTransfer,
        clientX: dropClientX,
        clientY: dropClientY,
      }));

      // — upstream sub-helper: drop — dispatch drop on the target.
      // BuilderCanvas.tsx onDrop runs and reads
      // dataTransfer.getData('application/x-builder-node'), parses
      // the JSON payload, calls addNode('tool', 'Read', position) via
      // screenToFlowPosition({x: clientX, y: clientY}).
      targetElement.dispatchEvent(new DragEvent('drop', {
        bubbles: true,
        cancelable: true,
        dataTransfer: dataTransfer,
        clientX: dropClientX,
        clientY: dropClientY,
      }));

      // — upstream sub-helper: dragend — dispatch dragend on the
      // source so any onDragEnd cleanup runs.
      sourceElement.dispatchEvent(new DragEvent('dragend', {
        bubbles: true,
        cancelable: true,
        dataTransfer: dataTransfer,
      }));
      `,
      [],
    );

    // Post-condition: BuilderCanvas's onDrop ran addNode('tool', 'Read', …)
    // and the projection rendered a BuilderToolNode whose testid is
    // `builder-tool-node-Read` (BuilderToolNode.tsx:23). `waitForDisplayed`
    // polls visibility with a budget — `isDisplayed()` called
    // synchronously after `waitForExist` flakes when React Flow's
    // commit lands a render-tick after the DOM insertion (typical
    // observed latency <100ms but the prior synchronous form raced
    // it on slower machines).
    const droppedNode = $('[data-testid="builder-tool-node-Read"]');
    await droppedNode.waitForExist({ timeout: 5_000 });
    await droppedNode.waitForDisplayed({ timeout: 5_000 });
  });
});
