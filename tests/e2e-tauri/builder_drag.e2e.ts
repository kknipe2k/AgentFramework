// Tauri 2.x desktop-shell E2E regression test — M08.5 Stage B.fix (🔴-1).
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
// WebDriver HTML5-DnD mechanism (gotcha #32 — cross-stack integration,
// quoted from an upstream reference, not hand-authored). The W3C
// WebDriver Actions API is the gesture path that engages the Tauri OS
// drag-drop handler (msedgedriver on Windows / WebKitWebDriver on Linux
// pushes real OS pointer events into the platform driver, which the
// `dragDropEnabled: true` handler intercepts at OS level). A bare
// `dragAndDrop()` / two-point pointer sequence does NOT fire HTML5
// `dragstart` on Chromium-based webviews — the documented Chromium
// constraint (https://github.com/webdriverio/webdriverio/issues/274,
// https://bugs.chromium.org/p/chromedriver/issues/detail?id=841). The
// well-established workaround is the multi-step pointer sequence: a
// small intermediate move past Chromium's `dragstart` threshold (~5px)
// after `pointerDown`, then a duration-paced move to the target, then
// `pointerUp`. Pattern per the WebdriverIO v9 Actions API docs at
// https://webdriver.io/docs/api/browser/action and the W3C WebDriver
// Actions specification at
// https://www.w3.org/TR/webdriver2/#dfn-perform-actions.
//
// The hypothesis the red phase must falsify (per CLAUDE.md §6 v1.8 +
// gotcha #82): on `main` (dragDropEnabled defaults true), the multi-step
// pointer sequence reaches the OS handler, the OS handler swallows the
// drag, no HTML5 dragstart fires in the webview, BuilderCanvas's onDrop
// never runs, and no `[data-testid^="builder-tool-node-"]` element is
// rendered on the canvas — the `waitForExist` assertion times out. After
// `dragDropEnabled: false` lands in `tauri.conf.json`, the OS handler is
// disabled, the same pointer sequence reaches the WebView2/WebKitGTK
// directly, HTML5 DnD synthesizes, `addNode('tool', 'Read', …)` runs,
// and the `[data-testid="builder-tool-node-Read"]` element renders.
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
    // Wait past app launch — the SetupPanel is the M03 first-paint
    // surface used by the smoke tests; once it is visible the renderer
    // is mounted and the ViewSwitch is reachable.
    const setupPanel = $('section[aria-label="api key setup"]');
    await setupPanel.waitForDisplayed({ timeout: 10_000 });

    // Switch to the Builder view (ViewSwitch.tsx data-testid).
    const builderTab = $('[data-testid="view-switch-builder"]');
    await builderTab.waitForDisplayed({ timeout: 5_000 });
    await builderTab.click();

    // The drag source — `Read` is a BUILTIN_TOOL (Palette.tsx
    // BUILTIN_TOOLS) on the Tools tab (the default tab), so the
    // selector resolves with no backend data + no test fixture.
    const paletteItem = $('[data-testid="palette-item-Read"]');
    await paletteItem.waitForDisplayed({ timeout: 5_000 });

    // The drop target.
    const canvas = $('[data-testid="builder-canvas"]');
    await canvas.waitForDisplayed({ timeout: 5_000 });

    // Pre-condition: no tool nodes on the canvas yet.
    const preDrop = await $$('[data-testid^="builder-tool-node-"]').length;
    expect(preDrop, 'no tool nodes should exist before the drag').to.equal(0);

    // W3C Actions API multi-step pointer drag (see file header for the
    // upstream-reference rationale; the small intermediate move past
    // Chromium's ~5px dragstart threshold is the load-bearing step).
    //
    // `origin: <element>` places the pointer at the element's center.
    // The intermediate `move({ origin: paletteItem, x: 5, y: 5 })`
    // crosses the dragstart threshold while the pointer is still over
    // the source; the duration-paced second move emulates a real user
    // drag (WebView2 synthesizes a smooth move from the duration field
    // per the W3C WebDriver Actions algorithm).
    await browser
      .action('pointer', { parameters: { pointerType: 'mouse' } })
      .move({ origin: paletteItem })
      .down({ button: 0 })
      .move({ origin: paletteItem, x: 5, y: 5 })
      .move({ duration: 200, origin: canvas })
      .up({ button: 0 })
      .perform();

    // Post-condition: BuilderCanvas's onDrop ran addNode('tool', 'Read', …)
    // and the projection rendered a BuilderToolNode whose testid is
    // `builder-tool-node-Read` (BuilderToolNode.tsx:23). The 5s budget
    // gives WebView2 time to synthesize the HTML5 DnD events and React
    // Flow to commit the node — typical observed latency is <100ms.
    const droppedNode = $('[data-testid="builder-tool-node-Read"]');
    await droppedNode.waitForExist({ timeout: 5_000 });
    expect(await droppedNode.isDisplayed()).to.equal(true);
  });
});
