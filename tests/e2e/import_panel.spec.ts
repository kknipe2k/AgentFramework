import { test, expect, type Page } from '@playwright/test';

// M07.E / ADR-0015 — renderer-level Playwright for the Builder Import
// panel. Module mocking across the @tauri-apps/api ESM boundary does NOT
// work in Playwright (only Vitest) — so the paste-URL → invoke linkage +
// the enriched-ImportOutcome → disclosure render are covered by the
// Vitest suites (ImportPanel.test.tsx, against the Rust-proven shape).
// This spec asserts the state-injection → panel-render contract that
// only renders correctly inside a real browser layout, via the
// App.tsx `window.__graphStore` affordance (gotcha #54). Full
// Tauri-shell E2E remains the gotcha #23 carry-forward.

interface HashMismatch {
  type: 'artifact_hash_mismatch';
  artifact_ref: string;
  expected: string;
  actual: string;
}

async function emitHashMismatch(page: Page, ev: HashMismatch): Promise<void> {
  await page.evaluate((e) => {
    const w = window as unknown as {
      __graphStore?: { getState: () => { applyEvent: (x: unknown) => void } };
    };
    if (!w.__graphStore) {
      throw new Error('window.__graphStore not exposed — App.tsx affordance missing');
    }
    w.__graphStore.getState().applyEvent(e);
  }, ev);
}

async function injectReview(page: Page): Promise<void> {
  // Drive the store through the same recordImport boundary the panel
  // uses post-invoke, with the Rust-proven enriched ImportOutcome shape.
  await page.evaluate(() => {
    const w = window as unknown as {
      __graphStore?: { getState: () => { recordImport: (o: unknown) => void } };
    };
    w.__graphStore?.getState().recordImport({
      lock_key: 'fs-test@2.0.0',
      review_required: true,
      requires_secrets: ['OPENAI_API_KEY'],
      capabilities: ['network: api.example.com', 'shell: true'],
      l3_report: { report_id: 'vr-1', passed: true, reasons: [] },
      share_provenance: { exported_by: 'share-it@0.1.0', rebake_changes: [] },
    });
  });
}

async function resetImports(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as {
      __graphStore?: { setState: (s: Record<string, unknown>) => void };
    };
    w.__graphStore?.setState({ imports: {} });
  });
}

test.describe('M07.E Builder Import panel', () => {
  test.describe.configure({ timeout: 120_000 });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await resetImports(page);
  });

  test('panel_is_mounted_in_the_app_layout', async ({ page }) => {
    await expect(page.getByTestId('import-panel')).toBeVisible();
  });

  test('injected_review_record_renders_the_tier_gate_review_modal', async ({ page }) => {
    await injectReview(page);
    await expect(page.getByTestId('import-review-modal')).toBeVisible();
    await expect(page.getByTestId('import-capability-disclosure')).toContainText(
      'network: api.example.com',
    );
    await expect(page.getByTestId('import-trust-line')).toContainText(/no rebaking/i);
  });

  test('hash_mismatch_event_blocks_use_and_prompts_reinstall', async ({ page }) => {
    await emitHashMismatch(page, {
      type: 'artifact_hash_mismatch',
      artifact_ref: 'fs-test@2.0.0',
      expected: 'sha256-AAAA',
      actual: 'sha256-BBBB',
    });
    const prompt = page.getByTestId('import-reinstall-prompt');
    await expect(prompt).toBeVisible();
    await expect(prompt).toContainText('fs-test@2.0.0');
    await expect(page.getByTestId('import-reinstall-fs-test@2.0.0')).toBeVisible();
    await expect(page.getByTestId('import-remove-fs-test@2.0.0')).toBeVisible();
  });
});
