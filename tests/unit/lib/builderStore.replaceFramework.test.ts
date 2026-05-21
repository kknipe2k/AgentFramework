import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M08.E — the builderStore additions for the Inspector + the Canvas|JSON
// two-way binding: `openTester` / `closeTester` (the Test button's
// target — F2 renders the modal on `testerOpen`), the ADR-0020
// re-derive contract that `replaceFramework` triggers, and the pure
// `diffFramework` helper the Inspector's "Changes since save" section
// reads. `replaceFramework` / `setDiskFramework` themselves shipped at
// Stage C (covered by builderStore.test.ts) — this file pins what E
// adds + the contracts E depends on.
//
// addNode schedules a debounced validate_framework call (D2.3.4); mock
// the ipc command (partial — the other exports stay real) and run on
// fake timers so the store stays deterministic and never reaches the
// real Tauri bridge.
const { validateFrameworkMock } = vi.hoisted(() => ({ validateFrameworkMock: vi.fn() }));
vi.mock('../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../src/lib/ipc')>();
  return { ...actual, validateFramework: validateFrameworkMock };
});

import { emptyFramework, useBuilderStore } from '../../../src/lib/builderStore';
import { diffFramework } from '../../../src/lib/frameworkDiff';
import type { FrameworkValidationReport } from '../../../src/lib/ipc';
import type { Agent, Framework } from '../../../src/types/framework';

/** A clean, passing validation report — the debounce-trigger default. */
function okReport(): FrameworkValidationReport {
  return { schema_errors: [], capability_errors: [], ok: true, capability_summary: null };
}

/** A minimal inline agent entry for the projection re-derive test. */
function agentEntry(id: string): Agent {
  return {
    id,
    role: '',
    model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
    allowed_tools: [],
    allowed_skills: [],
    spawns: [],
  } as unknown as Agent;
}

beforeEach(() => {
  vi.useFakeTimers();
  validateFrameworkMock.mockReset().mockResolvedValue(okReport());
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

describe('builderStore — replaceFramework re-derive + the Tester slot (M08.E)', () => {
  it('replaceFramework_causes_the_canvasNodes_projection_to_re_derive', () => {
    // ADR-0020: the canvas is a pure projection of `framework`, so
    // swapping the document re-derives the canvas with no extra wiring
    // — this is exactly why the JSON tab + load_framework can both
    // route through replaceFramework alone.
    expect(useBuilderStore.getState().canvasNodes()).toHaveLength(0);
    const withAgent: Framework = {
      ...emptyFramework(),
      agents: [agentEntry('planner')] as Framework['agents'],
    };
    useBuilderStore.getState().replaceFramework(withAgent);
    const nodes = useBuilderStore.getState().canvasNodes();
    expect(nodes).toHaveLength(1);
    expect(nodes[0]?.id).toBe('agent:planner');
  });

  it('openTester_sets_the_tester_open_state', () => {
    // INERT-but-wired at E — the test asserts the slot, not a modal
    // (Stage F2 renders the Tester modal on `testerOpen`).
    expect(useBuilderStore.getState().testerOpen).toBe(false);
    useBuilderStore.getState().openTester();
    expect(useBuilderStore.getState().testerOpen).toBe(true);
  });

  it('closeTester_clears_the_tester_open_state', () => {
    // Seed `testerOpen` true directly so this test genuinely exercises
    // closeTester (not the coincidental initial-false state).
    useBuilderStore.setState({ testerOpen: true });
    useBuilderStore.getState().closeTester();
    expect(useBuilderStore.getState().testerOpen).toBe(false);
  });
});

describe('frameworkDiff — the Inspector "Changes since save" disk diff (M08.E)', () => {
  it('the_inspector_diff_is_empty_when_framework_equals_diskFramework', () => {
    const fw = useBuilderStore.getState().framework;
    const diff = diffFramework(fw, fw);
    expect(diff.changed).toBe(false);
    expect(diff.lines).toEqual([]);
  });

  it('the_inspector_diff_is_non_empty_after_a_canvas_edit_following_a_save', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    // Save: the disk snapshot is the framework at this point.
    const saved = useBuilderStore.getState().framework;
    store.setDiskFramework(saved);
    // A further canvas edit diverges the framework from the snapshot.
    store.addNode('agent', 'worker', { x: 0, y: 0 });
    const diff = diffFramework(
      useBuilderStore.getState().framework,
      useBuilderStore.getState().diskFramework,
    );
    expect(diff.changed).toBe(true);
    expect(diff.lines.some((line) => line.tag === 'added')).toBe(true);
  });

  it('the_diff_marks_the_added_and_removed_lines_of_a_changed_field', () => {
    const disk = emptyFramework();
    const current: Framework = { ...emptyFramework(), name: 'renamed-framework' };
    const diff = diffFramework(current, disk);
    expect(diff.changed).toBe(true);
    expect(diff.lines.some((line) => line.tag === 'added' && line.text.includes('renamed'))).toBe(
      true,
    );
    expect(
      diff.lines.some((line) => line.tag === 'removed' && line.text.includes('untitled')),
    ).toBe(true);
  });
});
