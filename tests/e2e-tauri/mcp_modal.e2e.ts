// Tauri 2.x desktop-shell E2E regression test — M08.5 Stage D.fix (🔴-3).
//
// Closes docs/M08-irl-findings.md 🔴-3 — the "Add MCP Server" modal
// renders (the tier banner shows) but Test / Add / Cancel are all
// non-responsive in the real Tauri app on `main`. The renderer code,
// the handlers, and the modal CSS are each correct in isolation:
// MCPServerAddModal.tsx (handlers defined + bound; Test/Cancel are
// `type="button"` and unconditionally enabled; Add is `type="submit"`
// gated on `nameValid`), MCPServerSettings.tsx:88 mounts it as
// `{showAdd && <MCPServerAddModal …/>}`, and
// .mcp-server-add-modal-backdrop is `position: fixed; inset: 0;
// z-index: 50`. No app element stacks above z-index 50 in the Runtime
// view (the only other z-index:50 element, .gap-panel, is earlier in
// DOM order so the modal paints over it; HITLModal / HITLToast /
// RecoveryDialog / UncertaintyPrompt return null with nothing pending;
// SqlInspector is a normal-flow <section>). No capture-phase handler
// or stopPropagation intercepts the click anywhere in `src/`.
//
// The dead buttons are therefore a real-WebView2-runtime behavior the
// static renderer code does not explain (gotcha #41 — do NOT assert an
// unverified root cause). This test is the reproduction; the fix is
// the defensive portal + z-index + max-height/overflow hardening per
// phase doc D.3.2 — robust across the candidate causes (trapped
// stacking context, transparent overlay, off-viewport action row).
//
// Why this can only live as a tauri-driver real-app test (gotcha #66 —
// tests-pass-but-contract-fails): the existing Playwright spec
// `tests/e2e/mcp_server_add.spec.ts` runs in a plain Chromium with
// `@tauri-apps/api` mocked and asserts only that the modal opens —
// it never clicks a button inside it (the structural blind spot that
// let 🔴-3 escape). A Playwright test that clicked Cancel would still
// pass on `main`: plain Chromium does not reproduce the WebView2
// interception. Only the Stage A.fix `tauri-driver` harness drives
// the real running app where the bug exists.
//
// Falsifiable hypothesis the red phase disproves (CLAUDE.md §6 v1.8 +
// gotcha #82): on `main` today, clicking `[data-testid="mcp-add-cancel"]`
// in the running Tauri app does NOT remove `[data-testid="mcp-server-add-modal"]`
// from the DOM — the click is intercepted at the WebView2 layer (the
// candidate causes per phase doc D.3.2 are trapped-stacking-context,
// a transparent fixed interceptor, or an off-viewport action row;
// the modal CSS has no `max-height`/`overflow` to keep the action row
// reachable on a small viewport). After the D.fix portal + z-index
// + max-height/overflow hardening lands, the same click runs the
// bound onClose handler, MCPServerSettings flips `showAdd` to false,
// the conditional unmounts the modal, and `waitForExist({ reverse:
// true })` resolves green.
//
// Right-reason RED vs. harness-error rule-out:
// - Harness error would be: `mcp-add-server-button` not displayed
//   within 10s (Runtime view not rendered) → the harness itself
//   broke and the bug class is harness-revival, not modal-rendering.
// - Right-reason red is: button displays, modal opens, but
//   `waitForExist({ reverse: true })` on the modal panel TIMES OUT
//   AFTER `mcp-add-cancel.click()` returns. That means Cancel was
//   clicked but `onClose` never ran — the WebView2 interception.
// The test header above + each `waitForDisplayed`'s timeout/timeoutMsg
// separate these two failure modes; CI logs surface which one fires.
//
// Cancel is the cleanest assertion (phase doc D.4) — it is a pure
// renderer state change (`onClick={onClose}` → `setShowAdd(false)`
// in MCPServerSettings) with no IPC, so a dead Cancel isolates the
// click-interception defect from any backend concern. Test / Add
// reach the Tauri command layer (mcp_test_connection / mcp_add_server)
// and would conflate the assertion.
//
// Selector choices: MCPServerSettings mounts unconditionally in
// `.graph-layout` (App.tsx:88), so the Add-Server button is reachable
// from the Runtime view without switching tiers or stubbing backend
// state. The tier banner inside the modal varies with `currentTier`
// (Novice vs Promoted), but the modal itself opens on both tiers —
// the tier text content is not asserted here.
//
// WebdriverIO v9 chainable convention (gotcha #38): `$()` returns a
// chainable, not a `PromiseLike`; method calls go on the chainable
// directly without intermediate `await`. The side-effect `webdriverio`
// import pulls in the `WebdriverIO.Browser` augmentations needed for
// `waitUntil` / element method types.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $ } from '@wdio/globals';
import { expect } from 'chai';

describe('MCP Add-Server modal — M08.5 🔴-3 (real-app regression)', () => {
  it('mcp_add_server_modal_buttons_are_responsive', async () => {
    // Wait past app launch — SetupPanel is the M03 first-paint surface
    // used by the smoke tests; once visible the renderer is mounted
    // and the Runtime view's `.graph-layout` (which contains
    // MCPServerSettings, App.tsx:88) is in the DOM.
    const setupPanel = $('section[aria-label="api key setup"]');
    await setupPanel.waitForDisplayed({
      timeout: 10_000,
      timeoutMsg:
        'SetupPanel did not appear within 10s — app launch failed (harness error, not a 🔴-3 reproduction)',
    });

    // The Add-Server button lives in MCPServerSettings, which renders
    // unconditionally in `.graph-layout` regardless of tier (the
    // tier-gate is on the MODAL's banner, not on the button).
    const addButton = $('[data-testid="mcp-add-server-button"]');
    await addButton.waitForDisplayed({
      timeout: 5_000,
      timeoutMsg:
        'Add-Server button not displayed — MCPServerSettings did not render (harness error, not a 🔴-3 reproduction)',
    });
    await addButton.click();

    // Modal mount is the precondition for the bug — the IRL finding
    // reports "modal renders (tier banner shows)". If the modal does
    // not even mount, that is a different bug class than 🔴-3.
    const modal = $('[data-testid="mcp-server-add-modal"]');
    await modal.waitForDisplayed({
      timeout: 5_000,
      timeoutMsg:
        'Modal did not mount after clicking Add-Server — different bug class than 🔴-3 (which requires the modal to render)',
    });
    expect(await modal.isDisplayed(), 'modal must be displayed before the Cancel click').to.equal(
      true,
    );

    // The load-bearing click. On `main` today, the IRL pass observed
    // this click has no effect: the modal stays open. The post-fix
    // expectation: `onClose` runs, MCPServerSettings flips showAdd to
    // false, the conditional `{showAdd && <… />}` unmounts the modal,
    // and the modal element leaves the DOM.
    const cancelButton = $('[data-testid="mcp-add-cancel"]');
    await cancelButton.waitForDisplayed({
      timeout: 5_000,
      timeoutMsg:
        'Cancel button not displayed — modal mounted incompletely (harness error, not the 🔴-3 reproduction; the modal panel is always rendered when the modal element is displayed)',
    });
    await cancelButton.click();

    // The assertion. `waitForExist({ reverse: true })` resolves when
    // the element is no longer in the DOM. A 5s budget is generous —
    // a working onClose unmounts synchronously on the next React
    // commit (<50ms typical). Timing out HERE is the right-reason
    // 🔴-3 reproduction: Cancel was clicked, the click event was
    // dispatched at the WebDriver layer, but the bound React onClick
    // never ran (the interception). Pre-fix this assertion fails;
    // post-fix it resolves immediately.
    await modal.waitForExist({
      reverse: true,
      timeout: 5_000,
      timeoutMsg:
        '🔴-3 reproduced: clicking Cancel did not close the Add-Server modal in the running Tauri app — the WebView2 click interception this stage fixes',
    });
  });
});
