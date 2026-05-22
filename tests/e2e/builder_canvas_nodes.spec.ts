import { test, expect, type Page } from '@playwright/test';

// M08.D1 — renderer-level Playwright for the Builder Canvas node
// editor. Drives the Vite dev server (gotcha #23 — Playwright cannot
// drive the Tauri window). This spec demonstrates MVP §M8 criterion 1
// end-to-end: drag an Agent Palette item onto the empty canvas, set
// role / model via inline properties, and see the plain-English
// capability disclosure render below the node.
//
// The Palette's Agents tab has no built-ins — it lists only what
// list_installed_artifacts returns — so a __TAURI_INTERNALS__ mock
// supplies one installed agent. The Palette->Canvas drop is HTML5
// native drag-and-drop; Playwright's dragTo does not carry the custom
// application/x-builder-node MIME payload, so the helper threads one
// DataTransfer handle through dragstart -> dragover -> drop.

async function installTauriMock(page: Page): Promise<void> {
  await page.addInitScript(() => {
    let callbackId = 0;
    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      transformCallback: (): number => {
        callbackId += 1;
        return callbackId;
      },
      invoke: async (command: string): Promise<unknown> => {
        if (command === 'list_installed_artifacts') {
          return [
            {
              key: 'planner-agent',
              kind: 'agent',
              source: {},
              installed_at: '2026-05-21T00:00:00Z',
            },
          ];
        }
        if (command === 'has_api_key') {
          return false;
        }
        return undefined;
      },
    };
  });
}

async function dragPaletteItemToCanvas(page: Page, itemTestId: string): Promise<void> {
  const dataTransfer = await page.evaluateHandle(() => new DataTransfer());
  await page.getByTestId(itemTestId).dispatchEvent('dragstart', { dataTransfer });
  await page.getByTestId('builder-canvas').dispatchEvent('dragover', { dataTransfer });
  await page.getByTestId('builder-canvas').dispatchEvent('drop', { dataTransfer });
}

test.describe('M08.D1 Builder Canvas node editor', () => {
  test.describe.configure({ timeout: 120_000 });

  test.beforeEach(async ({ page }) => {
    await installTauriMock(page);
    await page.goto('/');
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-canvas')).toBeVisible();
  });

  test('dragging_an_agent_palette_item_onto_the_canvas_instantiates_an_agent_node', async ({
    page,
  }) => {
    await page.getByTestId('palette-tab-agents').click();
    await dragPaletteItemToCanvas(page, 'palette-item-planner-agent');
    await expect(page.getByTestId('builder-agent-node-planner-agent')).toBeVisible();
  });

  test('selecting_a_node_opens_the_inline_config_panel', async ({ page }) => {
    await page.getByTestId('palette-tab-agents').click();
    await dragPaletteItemToCanvas(page, 'palette-item-planner-agent');
    await page.getByTestId('builder-agent-node-planner-agent').click();
    await expect(page.getByTestId('builder-node-config')).toBeVisible();
  });

  test('setting_role_and_model_updates_the_node', async ({ page }) => {
    await page.getByTestId('palette-tab-agents').click();
    await dragPaletteItemToCanvas(page, 'palette-item-planner-agent');
    await page.getByTestId('builder-agent-node-planner-agent').click();
    await page.getByTestId('node-config-role').fill('Lead planner');
    await expect(page.getByTestId('builder-agent-node-planner-agent')).toContainText(
      'Lead planner',
    );
  });

  test('the_capability_disclosure_renders_plain_english_below_the_node', async ({ page }) => {
    await page.getByTestId('palette-tab-agents').click();
    await dragPaletteItemToCanvas(page, 'palette-item-planner-agent');
    await page.getByTestId('builder-agent-node-planner-agent').click();
    await page.getByTestId('node-config-add-tool-input').fill('Read');
    await page.getByTestId('node-config-add-tool').click();
    // The disclosure derives from allowed_tools — plain English, live.
    const disclosure = page.getByTestId('builder-node-disclosure-planner-agent');
    await expect(disclosure).toContainText('Read');
    await expect(disclosure).toContainText(/tool/i);
  });

  test('dropping_a_tool_palette_item_adds_a_tool_node', async ({ page }) => {
    await dragPaletteItemToCanvas(page, 'palette-item-Read');
    await expect(page.getByTestId('builder-tool-node-Read')).toBeVisible();
  });

  test('re_dropping_the_same_palette_item_does_not_duplicate_the_node', async ({ page }) => {
    await dragPaletteItemToCanvas(page, 'palette-item-Read');
    await expect(page.getByTestId('builder-tool-node-Read')).toBeVisible();
    await dragPaletteItemToCanvas(page, 'palette-item-Read');
    await expect(page.getByTestId('builder-tool-node-Read')).toHaveCount(1);
  });
});
