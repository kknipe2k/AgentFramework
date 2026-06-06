// Tauri 2.x real-app E2E — M08.9.A the truthful Tester verdict (TD-047).
//
// Drives the BUILT Tauri app via `tauri-driver` + WebdriverIO v9 (ADR-0021;
// the merge-blocking `e2e-tauri-driver` job). M08.9.A made the Tester verdict
// 3-state (Pass / Fail / Tier-limited): `fold_outcome` folds the L4
// `AgentEvent::TierViolation` (previously dropped at `_ => {}`) into
// `TestOutcome.tier_blocks` + a derived `verdict`, and `TesterModal` reads
// `outcome.verdict` instead of the binary `outcome.passed`. A Novice
// tier-blocked run must read TIER-LIMITED, NOT a green PASS — DESIGN.md
// principle 8 (labels-true).
//
// The verdict that proves TD-047 closed is KEY-DEPENDENT: producing a real
// tier block needs a live model to emit a Write `ToolUse` that the Novice L4
// gate rejects (TesterModal's `outcome` is component-local state set from the
// backend `test_framework` round-trip — there is no inject seam). So the
// substantive verdict test is runtime-skip-guarded exactly like the
// smoke.e2e.ts real-Anthropic chain: it runs only when `ANTHROPIC_API_KEY` is
// reachable by this process, and `this.skip()`s otherwise. CI provides no key,
// so the maintainer IRL walkthrough is the authoritative close (CLAUDE.md §10;
// the smoke.e2e.ts / tier_enforcement.e2e.ts precedent). The key-independent
// test below guards the surface mount + the no-premature-verdict invariant.
//
// WebdriverIO v9 note: `$()` returns a chainable, not a promise — call methods
// on it directly. Per <https://webdriver.io/docs/api/element>.
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';

const hasAnthropicKey = (process.env.ANTHROPIC_API_KEY ?? '').trim().length > 0;

// A single-agent framework whose worker is allowed the in-process Write
// built-in. At the default Novice tier the L4 gate forbids every Write BEFORE
// the L1 scope check — so a model that emits a Write `ToolUse` is tier-blocked,
// which is exactly the TD-047 path (verdict = tier_limited, passed = true).
const WRITE_FRAMEWORK = {
  name: 'm089a-tier-limited-fixture',
  version: '1.0.0',
  description: 'M08.9.A — a Write-using framework to exercise the Novice tier block',
  model: { provider: 'anthropic', id: 'claude-haiku-4-5' },
  agents: [
    {
      id: 'worker',
      role: 'worker',
      model: { provider: 'anthropic', id: 'claude-haiku-4-5' },
      capabilities: {
        tools_called: [],
        skills_loaded: [],
        file_access: { read: [], write: ['report.md'] },
        network: [],
        shell: false,
        spawn_agents: [],
      },
      allowed_tools: ['Write'],
      allowed_skills: [],
      spawns: [],
    },
  ],
  tools: [],
  skills: [],
  session_root_agent: 'worker',
};

async function openBuilderTester(): Promise<void> {
  const builderTab = $('[data-testid="view-switch-builder"]');
  await builderTab.waitForDisplayed({ timeout: 10_000 });
  await builderTab.click();
  await browser.waitUntil(async () => browser.execute('return !!window.__builderStore'), {
    timeout: 5_000,
    timeoutMsg: 'window.__builderStore not exposed',
  });
  await browser.execute(
    `
    window.__builderStore.getState().replaceFramework(arguments[0]);
    window.__builderStore.getState().openTester();
    `,
    WRITE_FRAMEWORK,
  );
  const testerModal = $('[data-testid="tester-modal"]');
  await testerModal.waitForDisplayed({ timeout: 5_000 });
}

describe('Tauri real-app E2E — M08.9.A truthful Tester verdict (TD-047)', () => {
  it('the Tester opens without a premature verdict (no PASS before a run)', async () => {
    await openBuilderTester();
    // Before any run, the result surface — and therefore any verdict badge —
    // must NOT be present. A verdict shown without a run would be the lie this
    // milestone kills, just one step earlier.
    const verdict = $('[data-testid="tester-result-verdict"]');
    expect(await verdict.isExisting()).to.equal(false);
    const runButton = $('[data-testid="tester-run"]');
    await runButton.waitForDisplayed({ timeout: 5_000 });
  });

  it('a Novice run that attempts a Write reads TIER-LIMITED, not PASS', async function () {
    if (!hasAnthropicKey) {
      this.skip();
    }
    await openBuilderTester();
    // The default tier is Novice (tier.json first-run default); a Write is
    // L4-forbidden there. Instruct the model to write a file.
    const taskInput = $('[data-testid="tester-task-input"]');
    await taskInput.setValue('Use the Write tool to create report.md with the text "hello".');
    const runButton = $('[data-testid="tester-run"]');
    await runButton.click();

    const verdict = $('[data-testid="tester-result-verdict"]');
    await verdict.waitForDisplayed({ timeout: 60_000 });
    const text = (await verdict.getText()).trim();
    // The verdict tells the truth: a tier-blocked run is TIER-LIMITED, never a
    // clean PASS (TD-047 / ADR-0030 — the framework is fine; the user's tier
    // blocked the action).
    expect(text).to.equal('TIER-LIMITED');
    const result = $('[data-testid="tester-result"]');
    expect(await result.getAttribute('class')).to.include('tester-result--tier-limited');
    // The blocked action + the Promote affordance are surfaced.
    const blocks = $('[data-testid="tester-tier-blocks"]');
    await blocks.waitForDisplayed({ timeout: 5_000 });
    expect(await $('[data-testid="tester-tier-promote"]').isExisting()).to.equal(true);
  });
});
