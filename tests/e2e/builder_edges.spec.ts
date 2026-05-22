import { test, expect, type Page } from '@playwright/test';

// M08.D2 — renderer-level Playwright for the Builder Canvas edge editor.
// Drives the Vite dev server (gotcha #23 — Playwright cannot drive the
// Tauri window); validate_framework is mocked to a scripted
// FrameworkValidationReport. Demonstrates MVP §M8 criterion 2 (Agent→
// Skill edge), criterion 3 (Agent→Agent narrowing surfaced), and the
// red-badge half of criterion 4 — plus the invalid-pair reject path.
//
// React Flow edge creation is a pointer-based handle-to-handle drag:
// manual mouse.down → stepped mouse.move → mouse.up (Playwright's
// dragTo mismatches React Flow's pointer events).

interface InvokeArgs {
  doc?: { agents?: { spawns?: unknown[] }[] };
}

interface MockOptions {
  artifacts: Record<string, unknown>[];
  validation: Record<string, unknown>;
  /** When true, validate_framework returns `validation` only once the
   *  doc carries a non-empty `spawns` — so the narrowing surfaces as a
   *  consequence of drawing the Agent→Agent edge, not before it. */
  requireSpawnForReport?: boolean;
}

const OK_REPORT = {
  schema_errors: [],
  capability_errors: [],
  ok: true,
  capability_summary: null,
};

async function installTauriMock(page: Page, opts: MockOptions): Promise<void> {
  await page.addInitScript((o: MockOptions) => {
    let callbackId = 0;
    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      transformCallback: (): number => {
        callbackId += 1;
        return callbackId;
      },
      invoke: async (command: string, args?: InvokeArgs): Promise<unknown> => {
        if (command === 'list_installed_artifacts') {
          return o.artifacts;
        }
        if (command === 'has_api_key') {
          return false;
        }
        if (command === 'validate_framework') {
          if (o.requireSpawnForReport === true) {
            const agents = args?.doc?.agents ?? [];
            const spawned = agents.some((a) => Array.isArray(a.spawns) && a.spawns.length > 0);
            return spawned
              ? o.validation
              : { schema_errors: [], capability_errors: [], ok: true, capability_summary: null };
          }
          return o.validation;
        }
        return undefined;
      },
    };
  }, opts);
}

/** Drop a Palette item onto the canvas at an explicit cursor point so
 *  nodes land at distinct positions (a handle-to-handle edge drag needs
 *  separated nodes). Threads one DataTransfer through the HTML5 DnD. */
async function dropPaletteItem(
  page: Page,
  itemTestId: string,
  x: number,
  y: number,
): Promise<void> {
  const dataTransfer = await page.evaluateHandle(() => new DataTransfer());
  await page.getByTestId(itemTestId).dispatchEvent('dragstart', { dataTransfer });
  await page
    .getByTestId('builder-canvas')
    .dispatchEvent('dragover', { dataTransfer, clientX: x, clientY: y });
  await page
    .getByTestId('builder-canvas')
    .dispatchEvent('drop', { dataTransfer, clientX: x, clientY: y });
}

/** Drag a React Flow connection from one node's source handle to
 *  another node's target handle. */
async function drawEdge(
  page: Page,
  sourceNodeTestId: string,
  targetNodeTestId: string,
): Promise<void> {
  const source = page.getByTestId(sourceNodeTestId).locator('.react-flow__handle.source');
  const target = page.getByTestId(targetNodeTestId).locator('.react-flow__handle.target');
  const sb = await source.boundingBox();
  const tb = await target.boundingBox();
  if (sb === null || tb === null) {
    throw new Error('React Flow handles are not measurable');
  }
  await page.mouse.move(sb.x + sb.width / 2, sb.y + sb.height / 2);
  await page.mouse.down();
  // Stepped move so React Flow's connection state machine tracks the
  // pointer (gotcha #23 — a single jump can miss the connection).
  await page.mouse.move(tb.x + tb.width / 2, tb.y + tb.height / 2, { steps: 16 });
  await page.mouse.up();
}

/**
 * Drag an agent node so its centre sits at (toX, toY). Grabs the node
 * at its centre — clear of the top and bottom connection handles — so
 * the drag repositions the node rather than starting a connection;
 * routes through onNodesChange → moveNode.
 */
async function dragAgentNodeTo(
  page: Page,
  nodeTestId: string,
  toX: number,
  toY: number,
): Promise<void> {
  const box = await page.getByTestId(nodeTestId).boundingBox();
  if (box === null) {
    throw new Error(`${nodeTestId} is not measurable`);
  }
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(toX, toY, { steps: 12 });
  await page.mouse.up();
}

/**
 * Position the two agent nodes for a clean handle-to-handle edge drag.
 *
 * M08.E added the Canvas | JSON tab bar and M08.G the cross-mode
 * Settings panel above the Builder Canvas — each shrank the canvas, and
 * React Flow's fitView lands the two large agent nodes at max zoom,
 * too tall to stack with unobscured handles. Zoom out until both node
 * boxes together use under 60% of the canvas height, then place the
 * parent near the top and the child near the bottom — distinct,
 * unobscured, in-pane handles at any canvas height. The child moves
 * first: it is the last-added node so it sits on top, and clearing it
 * leaves the parent alone to grab.
 */
async function separateAgentNodes(page: Page): Promise<void> {
  const canvas = await page.getByTestId('builder-canvas').boundingBox();
  if (canvas === null) {
    throw new Error('builder-canvas is not measurable');
  }
  const zoomOut = page.getByRole('button', { name: 'Zoom Out' });
  for (let i = 0; i < 6; i += 1) {
    const p = await page.getByTestId('builder-agent-node-parent-agent').boundingBox();
    const c = await page.getByTestId('builder-agent-node-child-agent').boundingBox();
    if (p !== null && c !== null && p.height + c.height < canvas.height * 0.6) {
      break;
    }
    if (!(await zoomOut.isEnabled())) {
      break;
    }
    await zoomOut.click();
  }
  const colX = canvas.x + canvas.width / 2;
  await dragAgentNodeTo(
    page,
    'builder-agent-node-child-agent',
    colX,
    canvas.y + canvas.height - 70,
  );
  await dragAgentNodeTo(page, 'builder-agent-node-parent-agent', colX, canvas.y + 70);
}

function agentArtifact(key: string): Record<string, unknown> {
  return { key, kind: 'agent', source: {}, installed_at: '2026-05-21T00:00:00Z' };
}

function skillArtifact(key: string): Record<string, unknown> {
  return { key, kind: 'skill', source: {}, installed_at: '2026-05-21T00:00:00Z' };
}

test.describe('M08.D2 Builder Canvas edge editor', () => {
  test.describe.configure({ timeout: 120_000 });

  test('Agent→Skill edge adds the skill and paints a builder-edge wire', async ({ page }) => {
    await installTauriMock(page, {
      artifacts: [agentArtifact('planner-agent'), skillArtifact('research-skill')],
      validation: OK_REPORT,
    });
    await page.goto('/');
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-canvas')).toBeVisible();

    await page.getByTestId('palette-tab-agents').click();
    await dropPaletteItem(page, 'palette-item-planner-agent', 360, 170);
    await page.getByTestId('palette-tab-skills').click();
    await dropPaletteItem(page, 'palette-item-research-skill', 360, 430);

    await drawEdge(page, 'builder-agent-node-planner-agent', 'builder-skill-node-research-skill');
    // MVP §M8 criterion 2 — the connection paints exactly one wire.
    await expect(page.locator('.builder-edge')).toHaveCount(1);
  });

  test('Agent→Agent edge whose child over-declares surfaces a narrowing rejection', async ({
    page,
  }) => {
    // validate_framework returns a capability_summary whose spawn edge
    // carries narrowed_caps = { Err: "...net.fetch..." } — the child
    // declared a network capability the parent does not hold; L2a
    // all-or-nothing narrowing rejects the whole edge.
    await installTauriMock(page, {
      artifacts: [agentArtifact('parent-agent'), agentArtifact('child-agent')],
      requireSpawnForReport: true,
      validation: {
        schema_errors: [],
        capability_errors: [
          {
            node_path: 'child-agent',
            message: 'capability narrowing failed: network:net.fetch not held by parent',
          },
        ],
        ok: false,
        capability_summary: {
          files_read: [],
          files_written: [],
          network_hosts: [],
          any_shell: false,
          spawn_edges: [
            {
              parent_id: 'parent-agent',
              child_id: 'child-agent',
              parent_caps: [],
              child_declared_caps: [
                {
                  kind: 'network',
                  resource: 'net.fetch',
                  scope: { domain: 'net.fetch' },
                  side_effect_class: 'network_egress',
                },
              ],
              narrowed_caps: {
                Err: 'child declares network:net.fetch which the parent does not hold',
              },
            },
          ],
        },
      },
    });
    await page.goto('/');
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-canvas')).toBeVisible();

    await page.getByTestId('palette-tab-agents').click();
    await dropPaletteItem(page, 'palette-item-parent-agent', 360, 230);
    await dropPaletteItem(page, 'palette-item-child-agent', 360, 360);
    // M08.E's Canvas | JSON tab bar shrank the canvas; drag the child
    // clear of the parent so the two large agent nodes do not overlap
    // (which would bury the parent's source handle).
    await separateAgentNodes(page);

    await drawEdge(page, 'builder-agent-node-parent-agent', 'builder-agent-node-child-agent');
    // MVP §M8 criterion 3 — the narrowing decision surfaces, verbatim
    // from the Rust report (spec §9 — no TS intersection).
    await expect(page.locator('.narrowing-notice__rejected')).toContainText('net.fetch');
  });

  test('an invalid framework paints a red badge on the offending node', async ({ page }) => {
    await installTauriMock(page, {
      artifacts: [agentArtifact('planner-agent')],
      validation: {
        schema_errors: [{ node_path: 'planner-agent', message: 'session_root_agent is empty' }],
        capability_errors: [],
        ok: false,
        capability_summary: null,
      },
    });
    await page.goto('/');
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-canvas')).toBeVisible();

    await page.getByTestId('palette-tab-agents').click();
    await dropPaletteItem(page, 'palette-item-planner-agent', 360, 220);

    // The continuous debounced validation runs on the drop; the report
    // keys a schema error to this node → the red badge appears.
    await expect(page.locator('.builder-node--invalid .builder-node__badge')).toBeVisible();
  });

  test('an invalid node-pair connection is rejected — no edge appears', async ({ page }) => {
    await installTauriMock(page, { artifacts: [], validation: OK_REPORT });
    await page.goto('/');
    await page.getByTestId('view-switch-builder').click();
    await expect(page.getByTestId('builder-canvas')).toBeVisible();

    // Read / Write are built-in Tools — a Tool→Tool wire is not one of
    // the four spec edge types, so connectEdge rejects it.
    await dropPaletteItem(page, 'palette-item-Read', 280, 220);
    await dropPaletteItem(page, 'palette-item-Write', 280, 460);

    await drawEdge(page, 'builder-tool-node-Read', 'builder-tool-node-Write');
    await expect(page.locator('.builder-edge')).toHaveCount(0);
  });
});
