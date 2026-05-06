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

  it('stream_text_and_decision_record_are_no_ops', () => {
    // Stage C wires session_start to spawn a FrameworkNode, so it is no
    // longer in the no-op set. stream_text + decision_record remain
    // store-no-ops in v0.1 (Stage D's inspector consumes them as detail).
    useGraphStore.getState().applyEvent(spawnA);
    const before = useGraphStore.getState();
    const noopEvents: AgentEvent[] = [
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
    // emit but the store does NOT surface as nodes leaves the store
    // unchanged. Stage C lit up session_start (FrameworkNode root) so
    // it leaves this list; the remaining variants stay no-ops until M4+.
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

  // ---- Stage C: FrameworkNode root + MCP lazy spawn + animated edges ----

  const sessionStart: AgentEvent = {
    type: 'session_start',
    session_id: 's1',
    framework: 'aria',
    model: 'haiku',
  };

  const mcpToolInvoked: AgentEvent = {
    type: 'tool_invoked',
    agent_id: 'a1',
    tool_name: 'list_prs',
    source: 'mcp',
    server: 'github-mcp',
    input: { repo: 'kknipe2k/agent-runtime' },
  };

  const mcpToolInvokedSecond: AgentEvent = {
    type: 'tool_invoked',
    agent_id: 'a1',
    tool_name: 'comment_pr',
    source: 'mcp',
    server: 'github-mcp',
    input: {},
  };

  const mcpToolResult: AgentEvent = {
    type: 'tool_result',
    agent_id: 'a1',
    tool_name: 'list_prs',
    output: { ok: true },
    duration_ms: 7,
  };

  it('session_start_spawns_FrameworkNode_at_root', () => {
    useGraphStore.getState().applyEvent(sessionStart);
    const { nodes } = useGraphStore.getState();
    const fw = nodes.find((n) => n.type === 'framework');
    expect(fw).toBeDefined();
    expect(fw!.id).toBe('framework:aria');
    expect(fw!.data).toMatchObject({
      frameworkName: 'aria',
      model: 'haiku',
      status: 'active',
    });
  });

  it('session_start_is_idempotent_on_same_framework', () => {
    useGraphStore.getState().applyEvent(sessionStart);
    useGraphStore.getState().applyEvent(sessionStart);
    const fw = useGraphStore.getState().nodes.filter((n) => n.type === 'framework');
    expect(fw).toHaveLength(1);
  });

  it('tool_invoked_with_source_mcp_lazily_spawns_parent_MCPNode', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(mcpToolInvoked);
    const { nodes, edges } = useGraphStore.getState();
    const mcp = nodes.find((n) => n.type === 'mcp');
    const tool = nodes.find((n) => n.type === 'tool');
    expect(mcp).toBeDefined();
    expect(mcp!.id).toBe('mcp:github-mcp');
    expect(mcp!.data).toMatchObject({
      serverId: 'github-mcp',
      serverName: 'github-mcp',
      status: 'active',
    });
    expect(tool).toBeDefined();
    // Edge wiring: agent → MCP and MCP → tool. NOT agent → tool.
    const agentToTool = edges.find(
      (e) => e.source === 'agent:a1' && e.target === 'tool:a1:list_prs',
    );
    expect(agentToTool).toBeUndefined();
    const agentToMcp = edges.find((e) => e.source === 'agent:a1' && e.target === 'mcp:github-mcp');
    expect(agentToMcp).toBeDefined();
    const mcpToTool = edges.find(
      (e) => e.source === 'mcp:github-mcp' && e.target === 'tool:a1:list_prs',
    );
    expect(mcpToTool).toBeDefined();
  });

  it('tool_invoked_with_source_mcp_reuses_MCPNode_across_tools', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(mcpToolInvoked);
    useGraphStore.getState().applyEvent(mcpToolInvokedSecond);
    const { nodes, edges } = useGraphStore.getState();
    const mcps = nodes.filter((n) => n.type === 'mcp');
    expect(mcps).toHaveLength(1);
    const tools = nodes.filter((n) => n.type === 'tool');
    expect(tools).toHaveLength(2);
    // Two MCP→tool edges, one per tool, but only one agent→MCP edge.
    const agentToMcpEdges = edges.filter(
      (e) => e.source === 'agent:a1' && e.target === 'mcp:github-mcp',
    );
    expect(agentToMcpEdges).toHaveLength(1);
    const mcpToToolEdges = edges.filter((e) => e.source === 'mcp:github-mcp');
    expect(mcpToToolEdges).toHaveLength(2);
  });

  it('tool_invoked_creates_edge_with_animated_true', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(toolInvoked);
    const edge = useGraphStore.getState().edges.find((e) => e.target === 'tool:a1:read_file');
    expect(edge).toBeDefined();
    expect(edge!.animated).toBe(true);
  });

  it('tool_result_clears_animated_flag_on_matching_edge', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(toolInvoked);
    useGraphStore.getState().applyEvent(toolResult);
    const edge = useGraphStore.getState().edges.find((e) => e.target === 'tool:a1:read_file');
    expect(edge).toBeDefined();
    expect(edge!.animated).toBe(false);
  });

  it('mcp_tool_result_clears_animated_flag_on_mcp_to_tool_edge', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(mcpToolInvoked);
    useGraphStore.getState().applyEvent(mcpToolResult);
    const edge = useGraphStore
      .getState()
      .edges.find((e) => e.source === 'mcp:github-mcp' && e.target === 'tool:a1:list_prs');
    expect(edge).toBeDefined();
    expect(edge!.animated).toBe(false);
  });

  // ---- Stage D: token-spend tracking on AgentNode + ToolNode ----

  // Schema bump (additive minor in-place): tool_result + agent_complete
  // gain optional snake_case token fields (`tokens_in`, `tokens_out`,
  // `tokens_total`). The reducer translates these to camelCase data
  // fields on the renderer node interfaces.

  it('tool_result_with_tokens_populates_ToolNode_tokensIn_and_tokensOut', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(toolInvoked);
    const toolResultWithTokens: AgentEvent = {
      type: 'tool_result',
      agent_id: 'a1',
      tool_name: 'read_file',
      output: { ok: true },
      duration_ms: 12,
      tokens_in: 80,
      tokens_out: 35,
    };
    useGraphStore.getState().applyEvent(toolResultWithTokens);
    const tool = useGraphStore.getState().nodes.find((n) => n.id === 'tool:a1:read_file')!;
    expect(tool.data).toMatchObject({ tokensIn: 80, tokensOut: 35 });
  });

  it('tool_result_with_tokens_accumulates_AgentNode_totals', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(toolInvoked);
    useGraphStore.getState().applyEvent({
      type: 'tool_result',
      agent_id: 'a1',
      tool_name: 'read_file',
      output: { ok: true },
      duration_ms: 12,
      tokens_in: 80,
      tokens_out: 35,
    });
    // Add a second tool call so the agent total is the sum.
    useGraphStore.getState().applyEvent({
      type: 'tool_invoked',
      agent_id: 'a1',
      tool_name: 'write_file',
      source: 'builtin',
      server: null,
      input: {},
    });
    useGraphStore.getState().applyEvent({
      type: 'tool_result',
      agent_id: 'a1',
      tool_name: 'write_file',
      output: { ok: true },
      duration_ms: 5,
      tokens_in: 20,
      tokens_out: 10,
    });
    const agent = useGraphStore.getState().nodes.find((n) => n.id === 'agent:a1')!;
    expect(agent.data).toMatchObject({ tokensIn: 100, tokensOut: 45 });
  });

  it('agent_complete_with_tokens_total_updates_AgentNode_tokensTotal', () => {
    useGraphStore.getState().applyEvent(spawnA);
    const completeWithTokens: AgentEvent = {
      type: 'agent_complete',
      agent_id: 'a1',
      result: 'done',
      tokens_total: 250,
    };
    useGraphStore.getState().applyEvent(completeWithTokens);
    const agent = useGraphStore.getState().nodes.find((n) => n.id === 'agent:a1')!;
    expect(agent.data).toMatchObject({ status: 'complete', tokensTotal: 250 });
  });

  it('tool_result_without_optional_token_fields_leaves_token_state_at_zero', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(toolInvoked);
    // Original toolResult fixture has no token fields — covers the
    // additive optionality path. State must remain at the default 0,
    // never NaN/undefined.
    useGraphStore.getState().applyEvent(toolResult);
    const tool = useGraphStore.getState().nodes.find((n) => n.id === 'tool:a1:read_file')!;
    expect(tool.data).toMatchObject({ tokensIn: 0, tokensOut: 0 });
    const agent = useGraphStore.getState().nodes.find((n) => n.id === 'agent:a1')!;
    expect(agent.data).toMatchObject({ tokensIn: 0, tokensOut: 0 });
  });
});
