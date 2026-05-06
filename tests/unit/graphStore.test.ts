import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { AgentEvent } from '../../src/types/agent_event';
import { useGraphStore } from '../../src/lib/graphStore';

// Helpers — use schema-snake_case field names per CLAUDE.md §14
// (src/types/agent_event.ts is generated from schemas/event.v1.json).

const spawnA: AgentEvent = {
  type: 'agent_spawned',
  agent_id: 'a1',
  agent_name: 'smoke',
  parent_id: null,
  session_id: 's1',
};

const spawnB: AgentEvent = {
  type: 'agent_spawned',
  agent_id: 'a2',
  agent_name: 'sub',
  parent_id: 'a1',
  session_id: 's1',
};

const completeA: AgentEvent = {
  type: 'agent_complete',
  agent_id: 'a1',
  result: 'done',
};

const errorA: AgentEvent = {
  type: 'agent_error',
  agent_id: 'a1',
  error: 'kaboom',
};

const toolInvoked: AgentEvent = {
  type: 'tool_invoked',
  agent_id: 'a1',
  tool_name: 'read_file',
  source: 'builtin',
  server: null,
  input: { path: '/etc/hosts' },
};

const toolResult: AgentEvent = {
  type: 'tool_result',
  agent_id: 'a1',
  tool_name: 'read_file',
  output: { ok: true },
  duration_ms: 12,
};

const skillLoaded: AgentEvent = {
  type: 'skill_loaded',
  agent_id: 'a1',
  skill_name: 'planner',
  mode: null,
};

function reset(): void {
  useGraphStore.getState().clear();
}

describe('graphStore.applyEvent', () => {
  beforeEach(reset);
  afterEach(reset);

  it('agent_spawned_adds_AgentNode_with_active_status', () => {
    useGraphStore.getState().applyEvent(spawnA);
    const { nodes } = useGraphStore.getState();
    expect(nodes).toHaveLength(1);
    const node = nodes[0]!;
    expect(node.id).toBe('agent:a1');
    expect(node.type).toBe('agent');
    expect(node.data).toMatchObject({
      agentId: 'a1',
      agentName: 'smoke',
      status: 'active',
      parentAgentId: null,
    });
  });

  it('agent_spawned_with_parent_id_adds_parent_edge', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(spawnB);
    const { nodes, edges } = useGraphStore.getState();
    expect(nodes).toHaveLength(2);
    expect(edges).toHaveLength(1);
    const edge = edges[0]!;
    expect(edge.source).toBe('agent:a1');
    expect(edge.target).toBe('agent:a2');
  });

  it('agent_spawned_idempotent_on_duplicate', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(spawnA);
    const { nodes } = useGraphStore.getState();
    expect(nodes).toHaveLength(1);
  });

  it('agent_complete_updates_AgentNode_status_to_complete', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(completeA);
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'agent:a1')!;
    expect(node.data).toMatchObject({ status: 'complete' });
  });

  it('agent_error_updates_AgentNode_status_to_error', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(errorA);
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'agent:a1')!;
    expect(node.data).toMatchObject({ status: 'error' });
  });

  it('tool_invoked_adds_ToolNode_and_edge_from_agent', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(toolInvoked);
    const { nodes, edges } = useGraphStore.getState();
    const tool = nodes.find((n) => n.type === 'tool');
    expect(tool).toBeDefined();
    expect(tool!.id).toBe('tool:a1:read_file');
    expect(tool!.data).toMatchObject({
      toolName: 'read_file',
      agentId: 'a1',
      status: 'active',
      durationMs: null,
    });
    const edge = edges.find((e) => e.target === 'tool:a1:read_file');
    expect(edge).toBeDefined();
    expect(edge!.source).toBe('agent:a1');
  });

  it('tool_result_updates_ToolNode_to_complete_with_duration', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(toolInvoked);
    useGraphStore.getState().applyEvent(toolResult);
    const tool = useGraphStore.getState().nodes.find((n) => n.id === 'tool:a1:read_file')!;
    expect(tool.data).toMatchObject({ status: 'complete', durationMs: 12 });
  });

  it('skill_loaded_adds_SkillNode_with_dashed_edge_from_agent', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(skillLoaded);
    const { nodes, edges } = useGraphStore.getState();
    const skill = nodes.find((n) => n.type === 'skill');
    expect(skill).toBeDefined();
    expect(skill!.id).toBe('skill:a1:planner');
    expect(skill!.data).toMatchObject({
      skillName: 'planner',
      agentId: 'a1',
    });
    const edge = edges.find((e) => e.target === 'skill:a1:planner');
    expect(edge).toBeDefined();
    expect(edge!.source).toBe('agent:a1');
    // Spec §3 Behavior: skill-load edges are dashed; no-flow-animation
    // sentinel (Stage C will drive edge.style/className from this).
    expect(edge!.data).toMatchObject({ kind: 'skill-load' });
  });

  it('stream_text_decision_record_session_start_are_no_ops', () => {
    useGraphStore.getState().applyEvent(spawnA);
    const before = useGraphStore.getState();
    const noopEvents: AgentEvent[] = [
      { type: 'session_start', session_id: 's1', framework: 'aria', model: 'haiku' },
      { type: 'stream_text', agent_id: 'a1', text: 'hello' },
      {
        type: 'decision_record',
        agent_id: 'a1',
        decision: 'd',
        rationale: 'r',
        tool_used: null,
      },
    ];
    for (const e of noopEvents) {
      useGraphStore.getState().applyEvent(e);
    }
    const after = useGraphStore.getState();
    // No new nodes or edges added by these events.
    expect(after.nodes).toHaveLength(before.nodes.length);
    expect(after.edges).toHaveLength(before.edges.length);
  });

  it('clear_empties_nodes_edges_and_selectedNodeId', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(spawnB);
    useGraphStore.getState().selectNode('agent:a1');
    useGraphStore.getState().clear();
    const { nodes, edges, selectedNodeId } = useGraphStore.getState();
    expect(nodes).toHaveLength(0);
    expect(edges).toHaveLength(0);
    expect(selectedNodeId).toBeNull();
  });

  it('selectNode_sets_selectedNodeId', () => {
    useGraphStore.getState().selectNode('agent:a1');
    expect(useGraphStore.getState().selectedNodeId).toBe('agent:a1');
    useGraphStore.getState().selectNode(null);
    expect(useGraphStore.getState().selectedNodeId).toBeNull();
  });

  it('agent_spawned_layout_staggers_x_position_per_root_agent', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent({
      type: 'agent_spawned',
      agent_id: 'a3',
      agent_name: 'second',
      parent_id: null,
      session_id: 's1',
    });
    const positions = useGraphStore
      .getState()
      .nodes.filter((n) => n.type === 'agent')
      .map((n) => n.position);
    // Two agents at distinct x coordinates (Stage B's naive horizontal
    // stagger; Stage D adds dagre).
    expect(positions[0]!.x).not.toBe(positions[1]!.x);
  });

  it('every_other_AgentEvent_variant_is_a_safe_no_op', () => {
    // Coverage discipline: assert that every variant the v0.1 schema can
    // emit but the Stage B store does NOT surface as nodes leaves the
    // store unchanged. Locks the exhaustiveness contract — Stage C
    // adding new wiring (e.g., session_start → FrameworkNode) lights
    // up the switch case, not a `default` accident.
    useGraphStore.getState().applyEvent(spawnA);
    const before = useGraphStore.getState();
    const noopVariants: AgentEvent[] = [
      { type: 'session_end', session_id: 's1', duration_ms: 100, end_reason: 'ok' },
      { type: 'tool_error', agent_id: 'a1', tool_name: 't', error: 'e' },
      { type: 'plan_created', plan_id: 'p1', task_count: 3 },
      { type: 'plan_approved', plan_id: 'p1' },
      { type: 'plan_rejected', plan_id: 'p1', reason: 'no' },
      { type: 'task_started', plan_id: 'p1', task_id: 't1', agent_id: 'a1' },
      { type: 'task_completed', plan_id: 'p1', task_id: 't1', duration_ms: 5 },
      { type: 'task_failed', plan_id: 'p1', task_id: 't1', error: 'e', failure_count: 1 },
      { type: 'task_rolled_back', plan_id: 'p1', task_id: 't1', snapshot_id: 'snap' },
      { type: 'task_escalated', plan_id: 'p1', task_id: 't1', reason: 'r' },
      { type: 'mode_changed', from: 'STANDARD', to: 'PROMOTED', reason: 'r' },
      { type: 'verify_started', hook_id: 'h', level: 'L1' },
      { type: 'verify_passed', hook_id: 'h', duration_ms: 5 },
      { type: 'verify_failed', hook_id: 'h', error: 'e' },
      { type: 'rail_triggered', rail_id: 'r', severity: 'warn', message: 'm' },
      { type: 'skill_missing', agent_id: 'a1', skill_name: 's', severity: 'warn' },
      { type: 'tool_missing', agent_id: 'a1', tool_name: 't', severity: 'block' },
      { type: 'gap_resolved', agent_id: 'a1', capability: 'c', kind: 'k' },
      { type: 'hitl_requested', agent_id: 'a1', prompt: 'p', hitl_kind: 'k' },
      { type: 'hitl_resolved', agent_id: 'a1', response: 'r', duration_ms: 100 },
      { type: 'capability_violation', agent_id: 'a1', declared: 'd', attempted: 'a' },
      { type: 'capability_grant', agent_id: 'a1', capability: 'c', scope: 's' },
      { type: 'budget_warn', spent_usd: 1, cap_usd: 10, percent: 0.1 },
      { type: 'budget_downshift', from_model: 'a', to_model: 'b', reason: 'r' },
      { type: 'budget_suspended', spent_usd: 9, cap_usd: 10 },
      { type: 'budget_exceeded', spent_usd: 11, cap_usd: 10 },
      { type: 'token_usage', input: 100, output: 50, model: 'haiku', cost_usd: 0.01 },
    ];
    for (const e of noopVariants) {
      useGraphStore.getState().applyEvent(e);
    }
    const after = useGraphStore.getState();
    expect(after.nodes).toEqual(before.nodes);
    expect(after.edges).toEqual(before.edges);
  });
});
