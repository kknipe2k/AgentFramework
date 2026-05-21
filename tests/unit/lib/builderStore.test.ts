import { beforeEach, describe, expect, it } from 'vitest';
import { emptyFramework, useBuilderStore } from '../../../src/lib/builderStore';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { Framework } from '../../../src/types/framework';

// M08.C — the Builder store (ADR-0020). builderStore holds the
// in-progress framework.json as the single source of truth; the canvas
// (D1/D2) is a projection derived from it. It is a SEPARATE Zustand
// store from graphStore (the live-execution store) — the two have
// disjoint lifecycles (build-time vs run-time) and must not be
// conflated.

function namedFramework(name: string): Framework {
  return { ...emptyFramework(), name };
}

describe('builderStore', () => {
  beforeEach(() => {
    useBuilderStore.setState({
      framework: emptyFramework(),
      diskFramework: null,
      selectedNodeId: null,
      validation: null,
    });
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
