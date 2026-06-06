// Tauri 2.x real-app E2E — M08.8.C.fix tier-display seed (#19) + the
// tier-change feedback toast (#20).
//
// Drives the BUILT Tauri app via `tauri-driver` + WebdriverIO v9 (ADR-0021;
// the merge-blocking `e2e-tauri-driver` job). This closes the DISPLAY half
// that parent Stage C missed: C wired enforcement (TD-036) + the truthful
// label, but the renderer never seeded `currentTier` from the backend — it
// defaulted 'novice' and was written ONLY by the tier_transition reducer. So
// after a restart with a Promoted backend the Settings display read Novice
// while the run enforced Promoted (#19 desync), and the button stuck on
// "Promote" with no Demote escape.
//
// This is the assembled-app regression: a reloaded renderer (a fresh window
// over the SAME backed-by-tier.json backend) must show the ENFORCED tier, not
// the 'novice' default. On `main` pre-fix the reloaded button reads "Promote"
// (the bug); post-fix the App mount calls get_current_tier and seeds the
// store, so it reads "Demote". Key-independent — request_tier_transition
// persists + emits with no Anthropic round-trip, so this runs in CI.
//
// The test restores Novice at the end so tier.json is clean for
// tier_enforcement.e2e.ts (which asserts the first-run Novice default and
// runs after this file in the alphabetical spec order — wdio.conf.ts
// maxInstances:1, declared order).
//
// WebdriverIO v9 note: `$()` returns a chainable, not a promise — call
// methods on it directly. Per <https://webdriver.io/docs/api/element>.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';

const button = (): ReturnType<typeof $> => $('[data-testid="tier-transition-button"]');
const tierCurrent = (): ReturnType<typeof $> => $('[data-testid="tier-current"]');

async function waitForLabel(expected: string, timeoutMsg: string): Promise<void> {
  await browser.waitUntil(async () => (await button().getText()).trim() === expected, {
    timeout: 10_000,
    timeoutMsg,
  });
}

describe('Tauri real-app E2E — M08.8.C.fix tier display seed (#19) + toast (#20)', () => {
  it('seeds the enforced tier into the display across a reload (#19 desync)', async () => {
    // Baseline: first-run Novice → the button reads the truthful "Promote".
    await button().waitForDisplayed({ timeout: 10_000 });
    await waitForLabel('Promote', 'baseline tier button should read "Promote" at Novice');

    // Promote — request_tier_transition persists to tier.json + emits a
    // tier_transition event the reducer folds into currentTier.
    await button().click();
    await waitForLabel('Demote', 'button should read "Demote" after promoting');
    await browser.waitUntil(
      async () => (await tierCurrent().getText()).toLowerCase().includes('promoted'),
      { timeout: 10_000, timeoutMsg: 'tier-current should read Promoted after promoting' },
    );

    // #20 feedback (DESIGN.md principle 1): the change surfaces a toast.
    const toastStack = $('[data-testid="toast-stack"]');
    await browser.waitUntil(
      async () => (await toastStack.isExisting()) && /promoted/i.test(await toastStack.getText()),
      { timeout: 5_000, timeoutMsg: 'a tier-change toast naming the new tier should appear' },
    );

    // The desync gate: reload to a fresh renderer window over the SAME
    // (promoted) backend. Pre-fix the store reverts to its 'novice'
    // default and the button reads "Promote"; post-fix the App mount
    // seeds currentTier from get_current_tier so it stays "Demote".
    await browser.reloadSession();
    await button().waitForDisplayed({ timeout: 15_000 });
    await waitForLabel(
      'Demote',
      'after reload the seeded display must still read Promoted ("Demote") — the #19 desync',
    );
    await browser.waitUntil(
      async () => (await tierCurrent().getText()).toLowerCase().includes('promoted'),
      { timeout: 10_000, timeoutMsg: 'reloaded tier-current must read the enforced Promoted' },
    );

    // Restore Novice so tier.json is clean for tier_enforcement.e2e.ts.
    await button().click();
    await waitForLabel('Promote', 'cleanup demote should return the button to "Promote"');
  });
});
