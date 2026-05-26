import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M08.6.D — the Builder store's load-applying action wires the dagre
// `layoutGraph` (src/lib/layout.ts) into the framework-load path so a
// loaded framework's nodes are positioned as a graph, not stacked at
// {0,0} (🟡 #4). Auto-layout fires on LOAD only — an interactive edit
// (drag, JSON-tab swap via replaceFramework) preserves the user's
// manual positions (ADR-0020: nodePositions is editor-local view
// state, not framework data).
//
// Mock `validateFramework` so the addNode debounce never reaches the
// real Tauri bridge — the existing builderStore.test.ts /
// .replaceFramework.test.ts pattern.
const { validateFrameworkMock } = vi.hoisted(() => ({ validateFrameworkMock: vi.fn() }));
vi.mock('../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../src/lib/ipc')>();
  return { ...actual, validateFramework: validateFrameworkMock };
});

import { emptyFramework, useBuilderStore } from '../../../src/lib/builderStore';
import type { Agent, Framework } from '../../../src/types/framework';

/** A minimal inline agent — `isInlineAgent` requires `role` to be set,
 *  so the projection treats this entry as inline (Stage B's loader
 *  produces this shape for path-ref agents post-resolution). */
function agentEntry(id: string, spawns: string[] = []): Agent {
  return {
    id,
    role: '',
    model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
    allowed_tools: [],
    allowed_skills: [],
    spawns,
  } as unknown as Agent;
}

beforeEach(() => {
  vi.useFakeTimers();
  validateFrameworkMock.mockReset().mockResolvedValue({
    schema_errors: [],
    capability_errors: [],
    ok: true,
    capability_summary: null,
  });
  useBuilderStore.setState({
    framework: emptyFramework(),
    diskFramework: null,
    selectedNodeId: null,
    validation: null,
    testerOpen: false,
    nodePositions: {},
  });
});

afterEach(() => {
  vi.runOnlyPendingTimers();
  vi.useRealTimers();
});

describe('builderStore — applyLoadedFramework lays out nodes (M08.6.D)', () => {
  it('applyLoadedFramework_seeds_nodePositions_with_distinct_coords_per_node', () => {
    // A two-agent framework whose orchestrator spawns the planner —
    // the dagre top-down layout (layoutGraph) ranks parent above child
    // and yields distinct positions for each node id.
    const fw: Framework = {
      ...emptyFramework(),
      agents: [
        agentEntry('orchestrator', ['planner']),
        agentEntry('planner'),
      ] as Framework['agents'],
    };

    useBuilderStore.getState().applyLoadedFramework(fw);

    const positions = useBuilderStore.getState().nodePositions;
    expect(positions['agent:orchestrator']).toBeDefined();
    expect(positions['agent:planner']).toBeDefined();
    // Distinct — the auto-layout actually placed them apart. A {0,0}
    // pile (the pre-D state) would have both at the same coords.
    const orchestrator = positions['agent:orchestrator']!;
    const planner = positions['agent:planner']!;
    const samePosition = orchestrator.x === planner.x && orchestrator.y === planner.y;
    expect(samePosition, 'auto-layout must place nodes at distinct positions').toBe(false);
  });

  it('moveNode_after_applyLoadedFramework_still_overrides_a_seeded_position', () => {
    const fw: Framework = {
      ...emptyFramework(),
      agents: [agentEntry('orchestrator')] as Framework['agents'],
    };
    useBuilderStore.getState().applyLoadedFramework(fw);
    const seeded = useBuilderStore.getState().nodePositions['agent:orchestrator']!;

    // User drags the node to a new position post-load — the existing
    // React Flow v12 controlled-drag path (BuilderCanvas.applyPositionChanges
    // → moveNode) must still win over the layout-seeded position.
    const dragged = { x: seeded.x + 100, y: seeded.y + 50 };
    useBuilderStore.getState().moveNode('agent:orchestrator', dragged);

    expect(useBuilderStore.getState().nodePositions['agent:orchestrator']).toEqual(dragged);
  });

  it('replaceFramework_does_not_auto_layout_so_JSON_tab_edits_preserve_manual_positions', () => {
    // A user drops an agent at a manually-chosen position (the
    // addNode path the Stage E JSON tab cannot reproduce on its own).
    useBuilderStore.getState().addNode('agent', 'orchestrator', { x: 250, y: 175 });
    const manual = useBuilderStore.getState().nodePositions['agent:orchestrator']!;
    expect(manual).toEqual({ x: 250, y: 175 });

    // A JSON-tab edit (Stage E) routes through replaceFramework. It
    // must NOT auto-layout — that would discard the user's manual
    // position. ADR-0020: nodePositions is editor-local view state,
    // not framework data; only LOAD seeds it.
    const edited: Framework = {
      ...useBuilderStore.getState().framework,
      name: 'renamed-via-json-tab',
    };
    useBuilderStore.getState().replaceFramework(edited);

    expect(useBuilderStore.getState().nodePositions['agent:orchestrator']).toEqual(manual);
  });
});
