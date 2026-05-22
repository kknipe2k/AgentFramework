import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M08.D2 — the Builder Canvas edge editor. connectEdge maps a
// (sourceKind, targetKind) pair to one of the four spec Phase 9 edge
// types and rejects every other pair; the canvas edge projection
// (canvasEdges) re-derives from `framework` (ADR-0020 — framework.json
// the single source of truth). Every framework mutation also schedules
// a debounced validate_framework call (D2.3.4).
//
// validate_framework is the Stage B Rust command — mocked here via a
// partial module mock so the unit tests assert against the debounce +
// the store, never the real Tauri bridge. The mock is partial
// (importOriginal spread) so unwrapCmdError and the other ipc exports
// stay real.
const { validateFrameworkMock } = vi.hoisted(() => ({ validateFrameworkMock: vi.fn() }));
vi.mock('../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../src/lib/ipc')>();
  return { ...actual, validateFramework: validateFrameworkMock };
});

import {
  emptyFramework,
  useBuilderStore,
  VALIDATION_DEBOUNCE_MS,
} from '../../../src/lib/builderStore';
import type { FrameworkValidationReport } from '../../../src/lib/ipc';
import type { Agent } from '../../../src/types/framework';

/** A clean, passing validation report — the debounce-trigger default. */
function okReport(): FrameworkValidationReport {
  return { schema_errors: [], capability_errors: [], ok: true, capability_summary: null };
}

/** The single inline agent the connectEdge tests build their edges from. */
function soleAgent(): Agent {
  return useBuilderStore.getState().framework.agents[0] as Agent;
}

describe('builderStore.connectEdge — the four spec edge types + the reject path', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    validateFrameworkMock.mockReset().mockResolvedValue(okReport());
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

  it('connect_agent_to_skill_pushes_skill_name_to_allowed_skills', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('skill', 'research', { x: 0, y: 0 });
    store.connectEdge('agent:planner', 'skill:research');
    // Agent→Skill = an allowed_skills entry (spec Phase 9 / MVP §M8
    // criterion 2): the SKILL NAME, not the prefixed canvas node id.
    expect(soleAgent().allowed_skills).toEqual(['research']);
  });

  it('connect_agent_to_tool_pushes_tool_name_to_allowed_tools', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('tool', 'Read', { x: 0, y: 0 });
    store.connectEdge('agent:planner', 'tool:Read');
    expect(soleAgent().allowed_tools).toEqual(['Read']);
  });

  it('connect_agent_to_agent_pushes_child_id_to_spawns', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('agent', 'worker', { x: 0, y: 0 });
    store.connectEdge('agent:planner', 'agent:worker');
    // Agent→Agent = a spawns entry (the child agent id). The narrowing
    // is NOT computed here — connectEdge only records the spawn; the
    // continuous validate_framework pass carries the intersection.
    expect(soleAgent().spawns).toEqual(['worker']);
  });

  it('connect_hook_to_task_pushes_hook_id_to_task_defaults_post_hooks', () => {
    const store = useBuilderStore.getState();
    store.addNode('hook', 'post_task', { x: 0, y: 0 });
    // The Builder has no Task palette node in v0.1 (the Palette has 5
    // tabs, none Tasks); connectEdge implements hook→task for spec
    // completeness — it is exercised directly here, not via the canvas.
    store.connectEdge('hook:post_task', 'task:default');
    const postHooks = useBuilderStore.getState().framework.task_defaults?.post_hooks ?? [];
    expect(postHooks).toHaveLength(1);
    expect((postHooks[0] as { $ref: string }).$ref).toBe('post_task');
  });

  it('connect_tool_to_tool_is_rejected_no_framework_mutation', () => {
    const store = useBuilderStore.getState();
    store.addNode('tool', 'Read', { x: 0, y: 0 });
    store.addNode('tool', 'Write', { x: 0, y: 0 });
    const frameworkBefore = useBuilderStore.getState().framework;
    store.connectEdge('tool:Read', 'tool:Write');
    // A non-spec pair mutates nothing — the framework reference is
    // identical (connectEdge returns state unchanged).
    expect(useBuilderStore.getState().framework).toBe(frameworkBefore);
  });

  it('connect_skill_to_agent_is_rejected_wrong_direction', () => {
    const store = useBuilderStore.getState();
    store.addNode('skill', 'research', { x: 0, y: 0 });
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    // Agent→Skill is a valid edge; Skill→Agent is not — direction
    // matters. The reject leaves allowed_skills empty.
    store.connectEdge('skill:research', 'agent:planner');
    expect(soleAgent().allowed_skills).toEqual([]);
  });

  it('connect_agent_to_skill_twice_does_not_duplicate_the_allowed_skills_entry', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('skill', 'research', { x: 0, y: 0 });
    store.connectEdge('agent:planner', 'skill:research');
    store.connectEdge('agent:planner', 'skill:research');
    // connectEdge is idempotent — re-connecting an already-recorded
    // edge does not append a duplicate allowed_skills entry.
    expect(soleAgent().allowed_skills).toEqual(['research']);
  });

  it('connectEdge_mutates_framework_and_the_canvasEdges_projection_re_derives', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('skill', 'research', { x: 0, y: 0 });
    // The edge projection is a pure function of `framework` (ADR-0020) —
    // before the edge it is empty, after the mutation it carries the wire.
    expect(useBuilderStore.getState().canvasEdges()).toHaveLength(0);
    store.connectEdge('agent:planner', 'skill:research');
    const edges = useBuilderStore.getState().canvasEdges();
    expect(edges).toHaveLength(1);
    expect(edges[0]?.source).toBe('agent:planner');
    expect(edges[0]?.target).toBe('skill:research');
  });
});

describe('builderStore — continuous debounced validation (D2.3.4)', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    validateFrameworkMock.mockReset().mockResolvedValue(okReport());
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

  it('addNode_schedules_a_debounced_validateFramework_call', async () => {
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    // Debounced — nothing fires until the quiet interval elapses.
    expect(validateFrameworkMock).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(VALIDATION_DEBOUNCE_MS);
    expect(validateFrameworkMock).toHaveBeenCalledTimes(1);
  });

  it('a_burst_of_framework_mutations_fires_validateFramework_once', async () => {
    const store = useBuilderStore.getState();
    // A burst of edits within the quiet interval — each mutation
    // reschedules the single in-flight timer, so the burst coalesces.
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('agent', 'worker', { x: 0, y: 0 });
    store.updateNode('agent:planner', { role: 'Lead' });
    store.addNode('skill', 'research', { x: 0, y: 0 });
    await vi.advanceTimersByTimeAsync(VALIDATION_DEBOUNCE_MS);
    expect(validateFrameworkMock).toHaveBeenCalledTimes(1);
  });

  it('validateFramework_result_lands_in_the_validation_slot', async () => {
    const report = okReport();
    validateFrameworkMock.mockResolvedValue(report);
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    await vi.advanceTimersByTimeAsync(VALIDATION_DEBOUNCE_MS);
    expect(useBuilderStore.getState().validation).toBe(report);
  });

  it('a_failed_validateFramework_leaves_the_prior_validation_report_in_place', async () => {
    const prior = okReport();
    useBuilderStore.setState({ validation: prior });
    validateFrameworkMock.mockRejectedValue(new Error('bridge failure'));
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => undefined);
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    await vi.advanceTimersByTimeAsync(VALIDATION_DEBOUNCE_MS);
    // A failed pass does NOT clear the slot — no flicker to "no errors";
    // the prior report stays until a successful pass replaces it.
    expect(useBuilderStore.getState().validation).toBe(prior);
    consoleError.mockRestore();
  });
});
