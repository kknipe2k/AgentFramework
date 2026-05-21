import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { emptyFramework, useBuilderStore } from '../../../src/lib/builderStore';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { Agent, Framework } from '../../../src/types/framework';

// M08.C — the Builder store (ADR-0020). builderStore holds the
// in-progress framework.json as the single source of truth; the canvas
// (D1/D2) is a projection derived from it. It is a SEPARATE Zustand
// store from graphStore (the live-execution store) — the two have
// disjoint lifecycles (build-time vs run-time) and must not be
// conflated.
//
// M08.D2 — addNode / updateNode now schedule a debounced
// validate_framework call; mock the ipc command (partial — the other
// exports stay real) and run on fake timers so these C/D1 store tests
// stay deterministic and never reach the real Tauri bridge.
const { validateFrameworkMock } = vi.hoisted(() => ({ validateFrameworkMock: vi.fn() }));
vi.mock('../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../src/lib/ipc')>();
  return { ...actual, validateFramework: validateFrameworkMock };
});

function namedFramework(name: string): Framework {
  return { ...emptyFramework(), name };
}

describe('builderStore', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    validateFrameworkMock
      .mockReset()
      .mockResolvedValue({
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
    });
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  it('initial_state_has_an_empty_framework_and_null_disk_snapshot', () => {
    const s = useBuilderStore.getState();
    // The cold-start document — framework.json carrying the required
    // top-level fields, with no tools / skills / agents yet (the user
    // adds them on the canvas, Stage D1).
    expect(s.framework.agents).toHaveLength(0);
    expect(s.framework.tools).toHaveLength(0);
    expect(s.framework.skills).toHaveLength(0);
    expect(s.framework.name.length).toBeGreaterThan(0);
    expect(s.framework.version.length).toBeGreaterThan(0);
    // diskFramework starts null — nothing saved or loaded; the Inspector
    // disk-diff (Stage E) renders the "no file on disk" state from this.
    expect(s.diskFramework).toBeNull();
    expect(s.selectedNodeId).toBeNull();
    expect(s.validation).toBeNull();
  });

  it('replaceFramework_swaps_the_whole_document', () => {
    // The JSON-tab edit (Stage E) + load_framework feed replaceFramework;
    // the canvas re-derives its projection from the new document.
    useBuilderStore.getState().replaceFramework(namedFramework('swapped-framework'));
    expect(useBuilderStore.getState().framework.name).toBe('swapped-framework');
  });

  it('setDiskFramework_records_the_snapshot_for_the_inspector_diff', () => {
    useBuilderStore.getState().setDiskFramework(namedFramework('on-disk-framework'));
    expect(useBuilderStore.getState().diskFramework?.name).toBe('on-disk-framework');
    // Clearing back to null is supported (Stage E's "no file" state).
    useBuilderStore.getState().setDiskFramework(null);
    expect(useBuilderStore.getState().diskFramework).toBeNull();
  });

  it('selectNode_sets_and_clears_selectedNodeId', () => {
    useBuilderStore.getState().selectNode('agent:planner');
    expect(useBuilderStore.getState().selectedNodeId).toBe('agent:planner');
    useBuilderStore.getState().selectNode(null);
    expect(useBuilderStore.getState().selectedNodeId).toBeNull();
  });

  it('builderStore_is_a_distinct_store_instance_from_graphStore', () => {
    // The SEPARATE-store invariant. builderStore and graphStore are
    // different create() instances; a builderStore mutation must not
    // touch graphStore.
    expect(useBuilderStore).not.toBe(useGraphStore);

    const graphStateBefore = useGraphStore.getState();
    useBuilderStore.getState().selectNode('builder-only-node');
    expect(useBuilderStore.getState().selectedNodeId).toBe('builder-only-node');
    // graphStore's state object is unchanged by a builderStore action.
    expect(useGraphStore.getState()).toBe(graphStateBefore);

    // builderStore carries the framework slot; graphStore does not —
    // overloading graphStore with build-time state is the anti-pattern
    // this store exists to avoid.
    expect('framework' in useBuilderStore.getState()).toBe(true);
    expect('framework' in useGraphStore.getState()).toBe(false);
  });
});

// M08.D1 — the canvas-mutation actions C shipped as typed no-op stubs,
// now implemented: addNode (drop a Palette item into framework) /
// updateNode (inline-config patch) / moveNode (controlled-drag layout)
// + the framework -> React-Flow projection selectors (canvasNodes /
// canvasEdges). Per ADR-0020 every edit mutates `framework` (the source
// of truth) and the canvas re-derives; nodePositions is editor-local
// layout state, never part of the framework document.
describe('builderStore — D1 canvas node actions', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    validateFrameworkMock
      .mockReset()
      .mockResolvedValue({
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
      nodePositions: {},
    });
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  it('addNode_with_an_agent_appends_an_agents_entry_to_framework', () => {
    useBuilderStore.getState().addNode('agent', 'planner', { x: 10, y: 20 });
    const agents = useBuilderStore.getState().framework.agents;
    expect(agents).toHaveLength(1);
    expect((agents[0] as Agent).id).toBe('planner');
  });

  it('addNode_records_the_drop_position_in_nodePositions', () => {
    useBuilderStore.getState().addNode('agent', 'planner', { x: 42, y: 99 });
    expect(useBuilderStore.getState().nodePositions['agent:planner']).toEqual({ x: 42, y: 99 });
  });

  it('addNode_is_idempotent_on_re_drop_of_the_same_item', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('agent', 'planner', { x: 500, y: 500 });
    const s = useBuilderStore.getState();
    // The re-drop neither duplicates the agents[] entry nor moves the node.
    expect(s.framework.agents).toHaveLength(1);
    expect(s.nodePositions['agent:planner']).toEqual({ x: 0, y: 0 });
  });

  it('addNode_with_a_tool_does_not_mutate_the_agents_array', () => {
    useBuilderStore.getState().addNode('tool', 'Read', { x: 1, y: 2 });
    const s = useBuilderStore.getState();
    expect(s.framework.agents).toHaveLength(0);
    // A Tool drop lands in framework.tools — the projection derives a
    // Tool node from it; a D2 edge later wires it into an agent.
    expect(s.framework.tools.some((t) => t.name === 'Read')).toBe(true);
  });

  it('updateNode_patches_the_selected_agents_role_and_model', () => {
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    useBuilderStore.getState().updateNode('agent:planner', { role: 'Lead planner' });
    useBuilderStore
      .getState()
      .updateNode('agent:planner', { model: { provider: 'anthropic', id: 'claude-opus-4-7' } });
    const agent = useBuilderStore.getState().framework.agents[0] as Agent;
    expect(agent.role).toBe('Lead planner');
    expect(agent.model.id).toBe('claude-opus-4-7');
  });

  it('updateNode_patches_allowed_tools_and_allowed_skills', () => {
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    useBuilderStore.getState().updateNode('agent:planner', { allowed_tools: ['Read', 'Write'] });
    useBuilderStore.getState().updateNode('agent:planner', { allowed_skills: ['planning'] });
    const agent = useBuilderStore.getState().framework.agents[0] as Agent;
    expect(agent.allowed_tools).toEqual(['Read', 'Write']);
    expect(agent.allowed_skills).toEqual(['planning']);
  });

  it('canvasNodes_derives_a_react_flow_node_per_framework_agent', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('agent', 'researcher', { x: 0, y: 0 });
    const nodes = useBuilderStore.getState().canvasNodes();
    expect(nodes).toHaveLength(2);
    expect(nodes.map((n) => n.id).sort()).toEqual(['agent:planner', 'agent:researcher']);
    expect(nodes.every((n) => n.type === 'agent')).toBe(true);
  });

  it('canvasNodes_carries_the_user_placed_position_from_nodePositions', () => {
    useBuilderStore.getState().addNode('agent', 'planner', { x: 123, y: 456 });
    const node = useBuilderStore.getState().canvasNodes()[0];
    expect(node?.position).toEqual({ x: 123, y: 456 });
  });

  it('canvasNodes_includes_tool_skill_hitl_and_hook_nodes', () => {
    const store = useBuilderStore.getState();
    store.addNode('tool', 'Read', { x: 0, y: 0 });
    store.addNode('skill', 'planning', { x: 0, y: 0 });
    store.addNode('hitl', 'on_gap', { x: 0, y: 0 });
    store.addNode('hook', 'pre_task', { x: 0, y: 0 });
    const ids = useBuilderStore
      .getState()
      .canvasNodes()
      .map((n) => n.id)
      .sort();
    expect(ids).toEqual(['hitl:on_gap', 'hook:pre_task', 'skill:planning', 'tool:Read']);
  });

  it('canvasNodes_returns_a_stable_reference_when_inputs_are_unchanged', () => {
    // useSyncExternalStore calls the selector repeatedly per render for
    // its consistency check — a fresh array each call infinite-loops
    // ("Maximum update depth exceeded"; gotcha #75). The projection
    // memoizes on the framework + nodePositions identities so an
    // unchanged document yields a referentially stable array.
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    const first = useBuilderStore.getState().canvasNodes();
    const second = useBuilderStore.getState().canvasNodes();
    expect(second).toBe(first);
  });

  it('canvasEdges_is_empty_in_D1', () => {
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    // Edges + the four edge types are D2 — D1 ships canvasEdges empty.
    expect(useBuilderStore.getState().canvasEdges()).toEqual([]);
  });

  it('moveNode_updates_nodePositions_and_leaves_framework_untouched', () => {
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    const frameworkBefore = useBuilderStore.getState().framework;
    useBuilderStore.getState().moveNode('agent:planner', { x: 300, y: 400 });
    const s = useBuilderStore.getState();
    expect(s.nodePositions['agent:planner']).toEqual({ x: 300, y: 400 });
    // A drag is canvas-UI state — it must not dirty the framework document
    // (so a reposition never shows up in Stage E's framework-vs-disk diff).
    expect(s.framework).toBe(frameworkBefore);
  });
});
