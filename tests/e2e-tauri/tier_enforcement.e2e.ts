// Tauri 2.x real-app E2E — M08.8.C tier in the run loop (TD-036).
//
// Drives the BUILT Tauri app via `tauri-driver` + WebdriverIO v9 (ADR-0021;
// the merge-blocking `e2e-tauri-driver` job). This is the real-app close for
// the tier-UI-truth half of Stage C — principles 2 (state-visible) and 8
// (labels-true) of DESIGN.md:
//
//   * the topbar tier chip shows the user's CURRENT (and, post-wire,
//     ENFORCED) tier — #19's desync root was UI-tier ≠ enforced-tier;
//   * the tier-transition button reads a TRUTHFUL "Promote" at Novice, not
//     the redundant "Promote to Promoted" (#20).
//
// The enforcement half — a Promoted run reaching the L1 SCOPE gate rather
// than the L4 TIER gate — is proven in the assembled Rust test
// (`src-tauri/src/commands.rs::test_framework_with_at_promoted_…`) and the
// maintainer IRL walkthrough; it needs a live Anthropic round-trip, which
// CI does not provide (CLAUDE.md §10 / the smoke.e2e.ts key-guard pattern).
// This file closes the key-independent UI-truth surface.
//
// WebdriverIO v9 note: `$()` returns a chainable, not a promise — call
// methods on it directly. Per <https://webdriver.io/docs/api/element>.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $ } from '@wdio/globals';
import { expect } from 'chai';

describe('Tauri real-app E2E — M08.8.C tier UI truth', () => {
  it('the topbar tier chip shows the current tier (state-visible — principle 2)', async () => {
    const tierChip = $('[data-testid="topbar-tier-chip"]');
    await tierChip.waitForDisplayed({ timeout: 10_000 });
    // First-run default is Novice (loaded from tier.json; commands.rs
    // CurrentTierState). The chip title-cases it.
    expect((await tierChip.getText()).toLowerCase()).to.include('novice');
  });

  it('the tier-transition button reads a truthful "Promote" at Novice (#20 / principle 8)', async () => {
    const button = $('[data-testid="tier-transition-button"]');
    await button.waitForDisplayed({ timeout: 10_000 });
    // The label is the redundant "Promote to Promoted" no longer — exact
    // match so the old string cannot pass as a substring.
    expect((await button.getText()).trim()).to.equal('Promote');
  });
});
