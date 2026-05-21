import { test, expect } from '@playwright/test';

// M08.C — renderer-level Playwright for the Builder workbench shell.
// Drives the Vite dev server; @tauri-apps/api + @tauri-apps/plugin-dialog
// have no Tauri IPC backend here (gotcha #23 — Playwright cannot drive
// the native window or file dialog), so the Palette's
// list_installed_artifacts call rejects and is caught — the Palette
// renders its built-in items. The file picker's behavior is covered by
// the Vitest ImportPanel suite. This spec covers the view switch + the
// three-panel shell + the Palette.

test.describe('M08.C Builder workbench shell', () => {
  test.describe.configure({ timeout: 120_000 });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('switching_to_builder_renders_the_three_panel_shell', async ({ page }) => {
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-shell')).toBeVisible();
    await expect(page.getByTestId('builder-palette-region')).toBeVisible();
    await expect(page.getByTestId('builder-canvas-region')).toBeVisible();
    await expect(page.getByTestId('builder-inspector-region')).toBeVisible();
  });

  test('switching_back_to_runtime_renders_the_live_graph', async ({ page }) => {
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-shell')).toBeVisible();
    await page.getByTestId('view-switch-runtime').click();
    await expect(page.getByTestId('graph-canvas')).toBeVisible();
    await expect(page.getByTestId('builder-shell')).toHaveCount(0);
  });

  test('palette_renders_five_tabs', async ({ page }) => {
    await page.getByTestId('view-switch-builder').click();
    for (const tab of ['tools', 'skills', 'agents', 'hitl', 'hooks']) {
      await expect(page.getByTestId(`palette-tab-${tab}`)).toBeVisible();
    }
  });

  test('clicking_a_palette_tab_switches_the_listed_items', async ({ page }) => {
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('palette-item-Read')).toBeVisible();
    await page.getByTestId('palette-tab-hooks').click();
    await expect(page.getByTestId('palette-item-pre_task')).toBeVisible();
    await expect(page.getByTestId('palette-item-Read')).toHaveCount(0);
  });

  test('filtering_a_tab_narrows_the_item_list', async ({ page }) => {
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('palette-item-Read')).toBeVisible();
    await page.getByTestId('palette-filter').fill('read');
    await expect(page.getByTestId('palette-item-Read')).toBeVisible();
    await expect(page.getByTestId('palette-item-Write')).toHaveCount(0);
  });

  test('a_palette_item_is_draggable', async ({ page }) => {
    await page.getByTestId('view-switch-builder').click();
    const item = page.getByTestId('palette-item-Read');
    await expect(item).toBeVisible();
    await expect(item).toHaveAttribute('draggable', 'true');
  });
});
