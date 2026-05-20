import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { AgentEvent } from '../../src/types/agent_event';
import { useGraphStore } from '../../src/lib/graphStore';
import type { ImportOutcome } from '../../src/lib/ipc';

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

  // Spec §4b — M05 Stage A gap-event applyEvent branches.
  describe('gap events (M05 Stage A)', () => {
    const toolMissingLoader: AgentEvent = {
      type: 'tool_missing',
      agent_id: 'worker',
      tool_name: 'fetch_prs',
      severity: 'critical',
      suggested_action: "Install tool 'fetch_prs' and click Resume.",
      requested_via: 'loader',
    };
    const skillMissingMeta: AgentEvent = {
      type: 'skill_missing',
      agent_id: 'worker',
      skill_name: 'rag',
      severity: 'requested',
      suggested_action: 'Agent worker requested skill rag: needs to look up repo context',
      requested_via: 'request_capability',
    };
    const mcpMissing: AgentEvent = {
      type: 'mcp_missing',
      agent_id: 'worker',
      server_name: 'pdf-mcp',
      severity: 'requested',
      suggested_action: 'Agent worker requested mcp server pdf-mcp: extract text from a PDF',
      requested_via: 'request_capability',
    };
    const agentMissing: AgentEvent = {
      type: 'agent_missing',
      agent_id: 'orchestrator',
      missing_agent_id: 'nonexistent-child',
      severity: 'critical',
      suggested_action:
        "Sub-agent 'nonexistent-child' not declared in framework; fix framework JSON before reloading.",
      requested_via: 'loader',
    };

    it('applies_tool_missing_event_mounts_gap_node', () => {
      useGraphStore.getState().applyEvent(toolMissingLoader);
      const gaps = useGraphStore.getState().nodes.filter((n) => n.type === 'gap');
      expect(gaps).toHaveLength(1);
      const gap = gaps[0]!;
      expect(gap.data).toMatchObject({
        gapId: 'gap:tool_missing:fetch_prs:worker',
        kind: 'tool_missing',
        missingName: 'fetch_prs',
        agentId: 'worker',
        severity: 'critical',
        requestedVia: 'loader',
        status: 'gap',
      });
      expect(gap.data.suggestedAction).toContain('Resume');
    });

    it('applies_skill_missing_with_requested_via_distinguishes_layer', () => {
      useGraphStore.getState().applyEvent(skillMissingMeta);
      const gap = useGraphStore.getState().nodes.find((n) => n.type === 'gap')!;
      expect(gap.data).toMatchObject({
        kind: 'skill_missing',
        missingName: 'rag',
        severity: 'requested',
        requestedVia: 'request_capability',
      });
    });

    it('applies_mcp_missing_and_agent_missing_variants', () => {
      useGraphStore.getState().applyEvent(mcpMissing);
      useGraphStore.getState().applyEvent(agentMissing);
      const gaps = useGraphStore.getState().nodes.filter((n) => n.type === 'gap');
      expect(gaps).toHaveLength(2);
      const kinds = gaps.map((g) => g.data.kind);
      expect(kinds).toContain('mcp_missing');
      expect(kinds).toContain('agent_missing');
    });

    it('tool_missing_idempotent_on_re_emission', () => {
      // Re-emission of the same gap (same kind + missing primitive +
      // agent) must collapse to one node. Mirrors agent_spawned_idempotent
      // semantics so loader replays + duplicate request_capability calls
      // don't double-mount.
      useGraphStore.getState().applyEvent(toolMissingLoader);
      useGraphStore.getState().applyEvent(toolMissingLoader);
      const gaps = useGraphStore.getState().nodes.filter((n) => n.type === 'gap');
      expect(gaps).toHaveLength(1);
    });

    it('latest_event_wins_on_severity_when_same_gap_re_emitted', () => {
      // request_capability emission for an already-loader-detected gap
      // should update the visible severity to the more-recent emission.
      useGraphStore.getState().applyEvent(toolMissingLoader);
      useGraphStore.getState().applyEvent({
        ...toolMissingLoader,
        severity: 'requested',
        requested_via: 'request_capability',
      });
      const gap = useGraphStore.getState().nodes.find((n) => n.type === 'gap')!;
      expect(gap.data.severity).toBe('requested');
      expect(gap.data.requestedVia).toBe('request_capability');
    });

    it('applies_gap_resolved_dismisses_gap_node', () => {
      useGraphStore.getState().applyEvent(toolMissingLoader);
      expect(useGraphStore.getState().nodes.filter((n) => n.type === 'gap')).toHaveLength(1);
      useGraphStore.getState().applyEvent({
        type: 'gap_resolved',
        agent_id: 'worker',
        capability: 'fetch_prs',
        kind: 'tool',
      });
      expect(useGraphStore.getState().nodes.filter((n) => n.type === 'gap')).toHaveLength(0);
    });

    it('gap_resolved_with_unknown_kind_is_safe_noop', () => {
      // Defensive — `kind` is free-text on the schema; an unknown value
      // shouldn't crash applyEvent.
      useGraphStore.getState().applyEvent(toolMissingLoader);
      useGraphStore.getState().applyEvent({
        type: 'gap_resolved',
        agent_id: 'worker',
        capability: 'fetch_prs',
        kind: 'something-else',
      });
      expect(useGraphStore.getState().nodes.filter((n) => n.type === 'gap')).toHaveLength(1);
    });
  });

  // Spec §8.security L2a — M05 Stage B capability-event applyEvent branches.
  describe('capability events (M05 Stage B)', () => {
    const violation: AgentEvent = {
      type: 'capability_violation',
      agent_id: 'worker',
      capability_kind: 'exec',
      requested_action: "invoke tool 'Bash'",
      declared_scope: 'declared grants do not cover this request',
    };
    const rootGrant: AgentEvent = {
      type: 'capability_grant',
      granted_to: 'worker',
      capability_kind: 'read',
      resource: 'src/**',
    };
    const narrowedGrant: AgentEvent = {
      type: 'capability_grant',
      parent_agent_id: 'orchestrator',
      granted_to: 'subagent',
      capability_kind: 'network',
      resource: 'api.example.com',
      narrowed_from: 'any *.example.com host',
    };

    it('applies_capability_violation_records_state_keyed_by_agent', () => {
      useGraphStore.getState().applyEvent(violation);
      const record = useGraphStore.getState().capabilityViolations['worker'];
      expect(record).toBeDefined();
      expect(record!.capabilityKind).toBe('exec');
      expect(record!.requestedAction).toBe("invoke tool 'Bash'");
      expect(record!.declaredScope).toContain('declared grants');
      expect(record!.timestamp).toBeGreaterThan(0);
    });

    it('capability_violation_last_write_wins_on_same_agent', () => {
      useGraphStore.getState().applyEvent(violation);
      const later: AgentEvent = {
        ...violation,
        requested_action: "invoke tool 'WebFetch'",
      };
      useGraphStore.getState().applyEvent(later);
      const record = useGraphStore.getState().capabilityViolations['worker']!;
      expect(record.requestedAction).toBe("invoke tool 'WebFetch'");
      // Still only one entry for this agent — Map shape, not log.
      expect(Object.keys(useGraphStore.getState().capabilityViolations)).toEqual(['worker']);
    });

    it('applies_capability_grant_appends_to_log_with_parent_absent', () => {
      useGraphStore.getState().applyEvent(rootGrant);
      const grants = useGraphStore.getState().capabilityGrants;
      expect(grants).toHaveLength(1);
      const grant = grants[0]!;
      expect(grant.parentAgentId).toBeNull();
      expect(grant.grantedTo).toBe('worker');
      expect(grant.capabilityKind).toBe('read');
      expect(grant.resource).toBe('src/**');
      expect(grant.narrowedFrom).toBeNull();
      expect(grant.timestamp).toBeGreaterThan(0);
    });

    it('applies_capability_grant_appends_to_log_with_narrowed_metadata', () => {
      useGraphStore.getState().applyEvent(narrowedGrant);
      const grant = useGraphStore.getState().capabilityGrants[0]!;
      expect(grant.parentAgentId).toBe('orchestrator');
      expect(grant.grantedTo).toBe('subagent');
      expect(grant.narrowedFrom).toBe('any *.example.com host');
    });

    it('capability_grants_log_is_append_only_on_repeated_emission', () => {
      // Gotcha #69 / multi-call invariant: two sequential grants both
      // land. The log preserves order and doesn't dedupe (re-grant of
      // the same capability is a legitimate event — re-narrowing on a
      // re-spawn, for example).
      useGraphStore.getState().applyEvent(rootGrant);
      useGraphStore.getState().applyEvent(rootGrant);
      useGraphStore.getState().applyEvent(narrowedGrant);
      const grants = useGraphStore.getState().capabilityGrants;
      expect(grants).toHaveLength(3);
      expect(grants[2]!.parentAgentId).toBe('orchestrator');
    });

    it('clear_resets_capability_state_slots', () => {
      useGraphStore.getState().applyEvent(violation);
      useGraphStore.getState().applyEvent(rootGrant);
      useGraphStore.getState().clear();
      const { capabilityViolations, capabilityGrants } = useGraphStore.getState();
      expect(capabilityViolations).toEqual({});
      expect(capabilityGrants).toEqual([]);
    });
  });

  // Spec §8.security L4 — M05 Stage D tier-event applyEvent branches.
  describe('tier events (M05 Stage D)', () => {
    const tierViolation: AgentEvent = {
      type: 'tier_violation',
      agent_id: 'worker',
      tier: 'novice',
      capability_kind: 'write',
      attempted_action: "write 'src/lib.rs' under Novice tier",
    };
    const promoteTransition: AgentEvent = {
      type: 'tier_transition',
      previous: 'novice',
      current: 'promoted',
      reason: 'user confirmed in Settings panel',
    };
    const demoteTransition: AgentEvent = {
      type: 'tier_transition',
      previous: 'promoted',
      current: 'novice',
      reason: 'user demoted',
    };

    it('first_run_state_has_novice_tier_default', () => {
      // The runtime's first-run default is Novice; the renderer's
      // initial-state default must match.
      expect(useGraphStore.getState().currentTier).toBe('novice');
    });

    it('applies_tier_violation_updates_state_keyed_by_agent', () => {
      useGraphStore.getState().applyEvent(tierViolation);
      const record = useGraphStore.getState().tierViolations['worker'];
      expect(record).toBeDefined();
      expect(record!.tier).toBe('novice');
      expect(record!.capabilityKind).toBe('write');
      expect(record!.attemptedAction).toContain('src/lib.rs');
      expect(record!.timestamp).toBeGreaterThan(0);
    });

    it('applies_tier_transition_flips_current_tier', () => {
      useGraphStore.getState().applyEvent(promoteTransition);
      expect(useGraphStore.getState().currentTier).toBe('promoted');
      useGraphStore.getState().applyEvent(demoteTransition);
      expect(useGraphStore.getState().currentTier).toBe('novice');
    });

    it('tier_violation_last_write_wins_on_same_agent', () => {
      useGraphStore.getState().applyEvent(tierViolation);
      const later: AgentEvent = {
        ...tierViolation,
        attempted_action: 'spawn process under Novice tier',
      };
      useGraphStore.getState().applyEvent(later);
      const record = useGraphStore.getState().tierViolations['worker']!;
      expect(record.attemptedAction).toBe('spawn process under Novice tier');
      expect(Object.keys(useGraphStore.getState().tierViolations)).toEqual(['worker']);
    });

    it('clear_preserves_current_tier_but_resets_tier_violations', () => {
      // Tier is a per-installation user preference; clear() is for
      // per-session graph state. The runtime persists tier across
      // sessions via tier.json, so the renderer must NOT reset it on
      // session clear.
      useGraphStore.getState().applyEvent(promoteTransition);
      useGraphStore.getState().applyEvent(tierViolation);
      useGraphStore.getState().clear();
      expect(useGraphStore.getState().currentTier).toBe('promoted');
      // Per-session violations DO clear.
      expect(useGraphStore.getState().tierViolations).toEqual({});
    });
  });

  describe('MCP events (M06.D)', () => {
    // currentMcpServers persists across clear() (registry-backed, like
    // currentTier) so reset() alone won't zero it — explicitly reset
    // the slot per the v1.6 <test_isolation_audit> discipline.
    beforeEach(() => {
      useGraphStore.setState({ currentMcpServers: {}, toolAliasWarnings: [] });
    });

    const installed: AgentEvent = {
      type: 'mcp_installed',
      name: 'pdf-mcp',
      transport_kind: 'stdio',
      has_auth: false,
    };

    it('applyEvent_mcp_installed_adds_server_to_currentMcpServers', () => {
      useGraphStore.getState().applyEvent(installed);
      const server = useGraphStore.getState().currentMcpServers['pdf-mcp'];
      expect(server).toBeDefined();
      expect(server!.name).toBe('pdf-mcp');
      expect(server!.transportKind).toBe('stdio');
      expect(server!.hasAuth).toBe(false);
      expect(server!.status).toBe('connected');
    });

    it('applyEvent_mcp_uninstalled_removes_server_from_currentMcpServers', () => {
      useGraphStore.getState().applyEvent(installed);
      expect(useGraphStore.getState().currentMcpServers['pdf-mcp']).toBeDefined();
      useGraphStore.getState().applyEvent({ type: 'mcp_uninstalled', name: 'pdf-mcp' });
      expect(useGraphStore.getState().currentMcpServers['pdf-mcp']).toBeUndefined();
    });

    it('applyEvent_mcp_auth_granted_updates_server_has_auth_flag', () => {
      useGraphStore.getState().applyEvent(installed);
      expect(useGraphStore.getState().currentMcpServers['pdf-mcp']!.hasAuth).toBe(false);
      useGraphStore.getState().applyEvent({ type: 'mcp_auth_granted', name: 'pdf-mcp' });
      expect(useGraphStore.getState().currentMcpServers['pdf-mcp']!.hasAuth).toBe(true);
    });

    it('applyEvent_mcp_request_blocked_appends_to_capabilityViolations_list_with_mcp_context', () => {
      const blocked: AgentEvent = {
        type: 'mcp_request_blocked',
        agent_id: 'worker',
        server: 'pdf-mcp',
        tool: 'extract_text',
        reason: 'no capabilities declared',
      };
      useGraphStore.getState().applyEvent(blocked);
      const record = useGraphStore.getState().capabilityViolations['worker'];
      expect(record).toBeDefined();
      // The MCP server + tool context must be readable from the
      // recorded violation (gotcha #68 — every field read).
      expect(record!.requestedAction).toContain('pdf-mcp');
      expect(record!.requestedAction).toContain('extract_text');
      expect(record!.declaredScope).toBe('no capabilities declared');
      expect(record!.timestamp).toBeGreaterThan(0);
    });

    it('applyEvent_tool_alias_ambiguous_records_warning', () => {
      const ambiguous: AgentEvent = {
        type: 'tool_alias_ambiguous',
        name: 'extract_text',
        candidates: ['pdf-mcp__extract_text', 'image-mcp__extract_text'],
      };
      useGraphStore.getState().applyEvent(ambiguous);
      const warnings = useGraphStore.getState().toolAliasWarnings;
      expect(warnings).toHaveLength(1);
      expect(warnings[0]!.name).toBe('extract_text');
      expect(warnings[0]!.candidates).toEqual(['pdf-mcp__extract_text', 'image-mcp__extract_text']);
      expect(warnings[0]!.timestamp).toBeGreaterThan(0);
    });

    it('mcp_installed_is_idempotent_under_repeated_identical_events', () => {
      useGraphStore.getState().applyEvent(installed);
      useGraphStore.getState().applyEvent(installed);
      const servers = useGraphStore.getState().currentMcpServers;
      expect(Object.keys(servers)).toEqual(['pdf-mcp']);
      expect(servers['pdf-mcp']!.hasAuth).toBe(false);
    });
  });

  describe('activeMcpCalls (M06.E)', () => {
    // activeMcpCalls is per-session animation state — reset on clear()
    // (an active-call glow must not leak into a new session) AND reset
    // explicitly here per the v1.6 <test_isolation_audit> discipline.
    beforeEach(() => {
      useGraphStore.setState({ activeMcpCalls: {}, currentMcpServers: {} });
    });

    const mcpInvoke: AgentEvent = {
      type: 'tool_invoked',
      agent_id: 'a1',
      tool_name: 'extract_text',
      source: 'mcp',
      server: 'pdf-mcp',
      input: { path: '/x.pdf' },
    };

    it('tool_invoked_with_source_mcp_sets_activeMcpCalls_for_server', () => {
      useGraphStore.getState().applyEvent(mcpInvoke);
      expect(useGraphStore.getState().activeMcpCalls['pdf-mcp']).toBe('tool:a1:extract_text');
    });

    it('tool_invoked_builtin_does_not_set_activeMcpCalls', () => {
      useGraphStore.getState().applyEvent(toolInvoked); // source: 'builtin'
      expect(Object.keys(useGraphStore.getState().activeMcpCalls)).toHaveLength(0);
    });

    it('tool_result_clears_activeMcpCalls_for_server', () => {
      useGraphStore.getState().applyEvent(mcpInvoke);
      expect(useGraphStore.getState().activeMcpCalls['pdf-mcp']).toBeDefined();
      useGraphStore.getState().applyEvent({
        type: 'tool_result',
        agent_id: 'a1',
        tool_name: 'extract_text',
        output: { ok: true },
        duration_ms: 7,
      });
      expect(useGraphStore.getState().activeMcpCalls['pdf-mcp']).toBeUndefined();
    });

    it('clear_resets_activeMcpCalls_unlike_currentMcpServers', () => {
      useGraphStore.setState({
        activeMcpCalls: { 'pdf-mcp': 'tool:a1:extract_text' },
        currentMcpServers: {
          'pdf-mcp': {
            name: 'pdf-mcp',
            transportKind: 'stdio',
            hasAuth: false,
            status: 'connected',
          },
        },
      });
      useGraphStore.getState().clear();
      expect(useGraphStore.getState().activeMcpCalls).toEqual({});
      // currentMcpServers is registry-backed — survives clear().
      expect(useGraphStore.getState().currentMcpServers['pdf-mcp']).toBeDefined();
    });
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
    // unchanged. M04 Stage B lit up the 11 plan/task variants; this list
    // shrinks accordingly. Plan/task event coverage lives in the
    // dedicated test block below.
    useGraphStore.getState().applyEvent(spawnA);
    const before = useGraphStore.getState();
    const noopVariants: AgentEvent[] = [
      { type: 'session_end', session_id: 's1', duration_ms: 100, end_reason: 'ok' },
      { type: 'tool_error', agent_id: 'a1', tool_name: 't', error: 'e' },
      { type: 'mode_changed', from: 'STANDARD', to: 'PROMOTED', reason: 'r' },
      // verify_started / verify_passed / verify_failed / rail_triggered
      // moved to dedicated tests below — M04 Stage D wires them to live
      // VerifyNode/HookNode + triggeredRails state.
      // M05 Stage A: skill_missing / tool_missing / mcp_missing / agent_missing /
      // gap_resolved moved to the "gap events (M05 Stage A)" describe block above.
      // hitl_requested / hitl_resolved / hitl_timeout / notifier_dispatched /
      // notifier_failed moved to dedicated tests below — M04 Stage E wires
      // them to live pendingHitl + notifierRecords state.
      // M05 Stage B: capability_violation / capability_grant moved to
      // the "capability events (M05 Stage B)" describe block below —
      // they now mutate dedicated state slots (`capabilityViolations`,
      // `capabilityGrants`), no longer no-op.
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

  // ---- M04 Stage B: plan/task event-driven state mutations ----

  it('plan_created_with_approval_required_inserts_PlanNode_pending_approval', () => {
    useGraphStore.getState().applyEvent({
      type: 'plan_created',
      plan_id: 'p1',
      title: 'Migrate auth',
      task_count: 3,
      approval_required: true,
    });
    const plan = useGraphStore.getState().nodes.find((n) => n.id === 'plan:p1');
    expect(plan).toBeDefined();
    expect(plan?.type).toBe('plan');
    if (plan?.type === 'plan') {
      expect(plan.data.status).toBe('pending_approval');
      expect(plan.data.title).toBe('Migrate auth');
      expect(plan.data.taskCount).toBe(3);
      expect(plan.data.approvalRequired).toBe(true);
    }
  });

  it('plan_created_without_approval_required_starts_approved', () => {
    useGraphStore.getState().applyEvent({
      type: 'plan_created',
      plan_id: 'p1',
      title: 'Auto plan',
      task_count: 1,
      approval_required: false,
    });
    const plan = useGraphStore.getState().nodes.find((n) => n.id === 'plan:p1');
    if (plan?.type === 'plan') {
      expect(plan.data.status).toBe('approved');
    }
  });

  it('plan_created_is_idempotent_on_duplicate', () => {
    const e: AgentEvent = {
      type: 'plan_created',
      plan_id: 'p1',
      title: 'T',
      task_count: 1,
      approval_required: false,
    };
    useGraphStore.getState().applyEvent(e);
    useGraphStore.getState().applyEvent(e);
    const planNodes = useGraphStore.getState().nodes.filter((n) => n.id === 'plan:p1');
    expect(planNodes.length).toBe(1);
  });

  it('plan_approval_requested_advances_to_awaiting_approval', () => {
    useGraphStore.getState().applyEvent({
      type: 'plan_created',
      plan_id: 'p1',
      title: 'T',
      task_count: 1,
      approval_required: true,
    });
    useGraphStore.getState().applyEvent({ type: 'plan_approval_requested', plan_id: 'p1' });
    const plan = useGraphStore.getState().nodes.find((n) => n.id === 'plan:p1');
    if (plan?.type === 'plan') {
      expect(plan.data.status).toBe('awaiting_approval');
    }
  });

  it('plan_approved_advances_to_in_progress', () => {
    useGraphStore.getState().applyEvent({
      type: 'plan_created',
      plan_id: 'p1',
      title: 'T',
      task_count: 1,
      approval_required: true,
    });
    useGraphStore
      .getState()
      .applyEvent({ type: 'plan_approved', plan_id: 'p1', approved_by: 'user' });
    const plan = useGraphStore.getState().nodes.find((n) => n.id === 'plan:p1');
    if (plan?.type === 'plan') {
      expect(plan.data.status).toBe('in_progress');
    }
  });

  it('plan_revised_advances_to_awaiting_replan_with_reason', () => {
    useGraphStore.getState().applyEvent({
      type: 'plan_created',
      plan_id: 'p1',
      title: 'T',
      task_count: 1,
      approval_required: false,
    });
    useGraphStore
      .getState()
      .applyEvent({ type: 'plan_revised', plan_id: 'p1', revision_reason: 'expand scope' });
    const plan = useGraphStore.getState().nodes.find((n) => n.id === 'plan:p1');
    if (plan?.type === 'plan') {
      expect(plan.data.status).toBe('awaiting_replan');
      expect(plan.data.lastTransitionReason).toBe('expand scope');
    }
  });

  it('plan_aborted_carries_reason', () => {
    useGraphStore.getState().applyEvent({
      type: 'plan_created',
      plan_id: 'p1',
      title: 'T',
      task_count: 1,
      approval_required: false,
    });
    useGraphStore.getState().applyEvent({ type: 'plan_aborted', plan_id: 'p1', reason: 'cancel' });
    const plan = useGraphStore.getState().nodes.find((n) => n.id === 'plan:p1');
    if (plan?.type === 'plan') {
      expect(plan.data.status).toBe('aborted');
      expect(plan.data.lastTransitionReason).toBe('cancel');
    }
  });

  it('plan_complete_records_duration', () => {
    useGraphStore.getState().applyEvent({
      type: 'plan_created',
      plan_id: 'p1',
      title: 'T',
      task_count: 1,
      approval_required: false,
    });
    useGraphStore
      .getState()
      .applyEvent({ type: 'plan_complete', plan_id: 'p1', duration_ms: 1234 });
    const plan = useGraphStore.getState().nodes.find((n) => n.id === 'plan:p1');
    if (plan?.type === 'plan') {
      expect(plan.data.status).toBe('complete');
      expect(plan.data.durationMs).toBe(1234);
    }
  });

  it('task_started_inserts_TaskNode_running', () => {
    useGraphStore.getState().applyEvent({
      type: 'task_started',
      plan_id: 'p1',
      task_id: 't1',
      agent_id: 'a1',
    });
    const task = useGraphStore.getState().nodes.find((n) => n.id === 'task:t1');
    expect(task).toBeDefined();
    if (task?.type === 'task') {
      expect(task.data.status).toBe('running');
      expect(task.data.agentId).toBe('a1');
    }
  });

  it('task_completed_increments_plan_completed_count', () => {
    useGraphStore.getState().applyEvent({
      type: 'plan_created',
      plan_id: 'p1',
      title: 'T',
      task_count: 2,
      approval_required: false,
    });
    useGraphStore.getState().applyEvent({
      type: 'task_started',
      plan_id: 'p1',
      task_id: 't1',
      agent_id: 'a1',
    });
    useGraphStore.getState().applyEvent({
      type: 'task_completed',
      plan_id: 'p1',
      task_id: 't1',
      duration_ms: 50,
    });
    const task = useGraphStore.getState().nodes.find((n) => n.id === 'task:t1');
    const plan = useGraphStore.getState().nodes.find((n) => n.id === 'plan:p1');
    if (task?.type === 'task') {
      expect(task.data.status).toBe('done');
      expect(task.data.durationMs).toBe(50);
    }
    if (plan?.type === 'plan') {
      expect(plan.data.completedCount).toBe(1);
    }
  });

  it('task_failed_records_failure_count_and_error', () => {
    useGraphStore.getState().applyEvent({
      type: 'task_started',
      plan_id: 'p1',
      task_id: 't1',
      agent_id: 'a1',
    });
    useGraphStore.getState().applyEvent({
      type: 'task_failed',
      plan_id: 'p1',
      task_id: 't1',
      error: 'boom',
      failure_count: 2,
    });
    const task = useGraphStore.getState().nodes.find((n) => n.id === 'task:t1');
    if (task?.type === 'task') {
      expect(task.data.status).toBe('failed');
      expect(task.data.failureCount).toBe(2);
      expect(task.data.lastError).toBe('boom');
    }
  });

  it('task_skipped_records_reason', () => {
    useGraphStore.getState().applyEvent({
      type: 'task_started',
      plan_id: 'p1',
      task_id: 't1',
      agent_id: 'a1',
    });
    useGraphStore.getState().applyEvent({
      type: 'task_skipped',
      plan_id: 'p1',
      task_id: 't1',
      reason: 'HITL skip',
    });
    const task = useGraphStore.getState().nodes.find((n) => n.id === 'task:t1');
    if (task?.type === 'task') {
      expect(task.data.status).toBe('skipped');
      expect(task.data.lastError).toBe('HITL skip');
    }
  });

  it('task_escalated_records_failure_count_and_max', () => {
    useGraphStore.getState().applyEvent({
      type: 'task_started',
      plan_id: 'p1',
      task_id: 't1',
      agent_id: 'a1',
    });
    useGraphStore.getState().applyEvent({
      type: 'task_escalated',
      plan_id: 'p1',
      task_id: 't1',
      failure_count: 3,
      max_failures: 3,
    });
    const task = useGraphStore.getState().nodes.find((n) => n.id === 'task:t1');
    if (task?.type === 'task') {
      expect(task.data.status).toBe('escalated');
      expect(task.data.failureCount).toBe(3);
      expect(task.data.maxFailures).toBe(3);
    }
  });

  it('task_rolled_back_records_snapshot_id', () => {
    useGraphStore.getState().applyEvent({
      type: 'task_started',
      plan_id: 'p1',
      task_id: 't1',
      agent_id: 'a1',
    });
    useGraphStore.getState().applyEvent({
      type: 'task_rolled_back',
      plan_id: 'p1',
      task_id: 't1',
      snapshot_id: 'snap-1',
    });
    const task = useGraphStore.getState().nodes.find((n) => n.id === 'task:t1');
    if (task?.type === 'task') {
      expect(task.data.status).toBe('failed');
      expect(task.data.rollbackSnapshotId).toBe('snap-1');
    }
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

  // ── M04 Stage D: verify / hook / rail event-driven mutations (spec §4a) ──

  it('verify_started_with_verify_category_inserts_VerifyNode_active', () => {
    useGraphStore.getState().applyEvent({
      type: 'verify_started',
      hook_id: 'verify',
      category: 'verify',
      firing_point: 'post_task',
      level: 'standard',
    });
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'verify:verify');
    expect(node).toBeDefined();
    expect(node?.type).toBe('verify');
    expect(node?.data).toMatchObject({
      hookId: 'verify',
      level: 'standard',
      firingPoint: 'post_task',
      status: 'active',
      durationMs: null,
      outputPreview: null,
      error: null,
      onFailure: null,
    });
  });

  it('verify_started_with_non_verify_category_inserts_HookNode_active', () => {
    useGraphStore.getState().applyEvent({
      type: 'verify_started',
      hook_id: 'lint',
      category: 'lint',
      firing_point: 'post_file_edit',
      level: null,
    });
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'hook:lint');
    expect(node).toBeDefined();
    expect(node?.type).toBe('hook');
    expect(node?.data).toMatchObject({
      hookId: 'lint',
      hookName: 'lint',
      category: 'lint',
      firingPoint: 'post_file_edit',
      status: 'active',
      durationMs: null,
      error: null,
    });
  });

  it('verify_passed_transitions_VerifyNode_to_pass_with_duration_and_preview', () => {
    useGraphStore.getState().applyEvent({
      type: 'verify_started',
      hook_id: 'verify',
      category: 'verify',
      firing_point: 'post_task',
    });
    useGraphStore.getState().applyEvent({
      type: 'verify_passed',
      hook_id: 'verify',
      duration_ms: 1234,
      output_preview: 'Tests passed',
    });
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'verify:verify');
    expect(node?.data).toMatchObject({
      status: 'pass',
      durationMs: 1234,
      outputPreview: 'Tests passed',
    });
  });

  it('verify_failed_transitions_VerifyNode_to_fail_with_error_and_on_failure', () => {
    useGraphStore.getState().applyEvent({
      type: 'verify_started',
      hook_id: 'verify',
      category: 'verify',
      firing_point: 'post_task',
    });
    useGraphStore.getState().applyEvent({
      type: 'verify_failed',
      hook_id: 'verify',
      duration_ms: 800,
      error: 'verify.sh exited 1',
      on_failure: 'rollback',
    });
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'verify:verify');
    expect(node?.data).toMatchObject({
      status: 'fail',
      durationMs: 800,
      error: 'verify.sh exited 1',
      onFailure: 'rollback',
    });
  });

  it('verify_passed_for_non_verify_hook_transitions_HookNode_to_complete', () => {
    useGraphStore.getState().applyEvent({
      type: 'verify_started',
      hook_id: 'lint',
      category: 'lint',
      firing_point: 'post_file_edit',
    });
    useGraphStore.getState().applyEvent({
      type: 'verify_passed',
      hook_id: 'lint',
      duration_ms: 50,
    });
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'hook:lint');
    expect(node?.data).toMatchObject({ status: 'complete', durationMs: 50 });
  });

  it('verify_failed_for_non_verify_hook_transitions_HookNode_to_error', () => {
    useGraphStore.getState().applyEvent({
      type: 'verify_started',
      hook_id: 'lint',
      category: 'lint',
      firing_point: 'post_file_edit',
    });
    useGraphStore.getState().applyEvent({
      type: 'verify_failed',
      hook_id: 'lint',
      duration_ms: 30,
      error: 'lint warnings: 12',
      on_failure: 'warn',
    });
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'hook:lint');
    expect(node?.data).toMatchObject({
      status: 'error',
      durationMs: 30,
      error: 'lint warnings: 12',
    });
  });

  it('verify_started_re_emit_for_same_hook_id_resets_to_active', () => {
    // Lock idempotence + re-fire semantics: re-emitting verify_started
    // for the same hook_id (e.g., retry after rollback) updates status
    // back to active and clears duration/error fields.
    useGraphStore.getState().applyEvent({
      type: 'verify_started',
      hook_id: 'verify',
      category: 'verify',
      firing_point: 'post_task',
    });
    useGraphStore.getState().applyEvent({
      type: 'verify_failed',
      hook_id: 'verify',
      duration_ms: 100,
      error: 'first attempt failed',
      on_failure: 'rollback',
    });
    useGraphStore.getState().applyEvent({
      type: 'verify_started',
      hook_id: 'verify',
      category: 'verify',
      firing_point: 'post_task',
    });
    const verifyCount = useGraphStore
      .getState()
      .nodes.filter((n) => n.id === 'verify:verify').length;
    expect(verifyCount).toBe(1);
    const node = useGraphStore.getState().nodes.find((n) => n.id === 'verify:verify');
    expect(node?.data).toMatchObject({
      status: 'active',
      durationMs: null,
      error: null,
      onFailure: null,
    });
  });

  it('rail_triggered_appends_to_triggeredRails_state', () => {
    useGraphStore.getState().applyEvent({
      type: 'rail_triggered',
      rail_id: 'no_secrets',
      policy: 'hard',
      firing_point: 'pre_commit',
      message: 'Secret detected',
      agent_id: 'a1',
    });
    const rails = useGraphStore.getState().triggeredRails;
    expect(rails).toHaveLength(1);
    expect(rails[0]).toMatchObject({
      railId: 'no_secrets',
      policy: 'hard',
      firingPoint: 'pre_commit',
      message: 'Secret detected',
      agentId: 'a1',
    });
  });

  it('rail_triggered_without_agent_id_records_null', () => {
    useGraphStore.getState().applyEvent({
      type: 'rail_triggered',
      rail_id: 'dont_touch',
      policy: 'hard',
      firing_point: 'pre_file_edit',
      message: '.env matches dont_touch glob: .env*',
    });
    expect(useGraphStore.getState().triggeredRails[0]).toMatchObject({ agentId: null });
  });

  it('rail_triggered_appends_in_order_for_multiple_emits', () => {
    useGraphStore.getState().applyEvent({
      type: 'rail_triggered',
      rail_id: 'r1',
      policy: 'soft',
      firing_point: 'post_file_edit',
      message: '1',
    });
    useGraphStore.getState().applyEvent({
      type: 'rail_triggered',
      rail_id: 'r2',
      policy: 'hard',
      firing_point: 'pre_commit',
      message: '2',
    });
    const rails = useGraphStore.getState().triggeredRails;
    expect(rails).toHaveLength(2);
    expect(rails.map((r) => r.railId)).toEqual(['r1', 'r2']);
  });

  it('clear_resets_triggeredRails_along_with_nodes_and_edges', () => {
    useGraphStore.getState().applyEvent({
      type: 'rail_triggered',
      rail_id: 'r1',
      policy: 'soft',
      firing_point: 'post_file_edit',
      message: 'm',
    });
    useGraphStore.getState().clear();
    expect(useGraphStore.getState().triggeredRails).toEqual([]);
    expect(useGraphStore.getState().nodes).toEqual([]);
    expect(useGraphStore.getState().edges).toEqual([]);
  });

  // ── M04 Stage E: HITL events ────────────────────────────────────

  it('hitl_requested_inserts_pendingHitl_keyed_by_prompt_id', () => {
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-1',
      trigger: 'on_failure_threshold',
      agent_id: null,
      question: 'Continue?',
      options: ['retry', 'skip'],
      ui_variant: 'panel',
      timeout_at_unix_ms: 9_999,
    });
    const pending = useGraphStore.getState().pendingHitl;
    expect(Object.keys(pending)).toEqual(['u-1']);
    expect(pending['u-1']).toMatchObject({
      promptId: 'u-1',
      trigger: 'on_failure_threshold',
      agentId: null,
      question: 'Continue?',
      uiVariant: 'panel',
    });
    expect(pending['u-1']?.options).toEqual(['retry', 'skip']);
  });

  it('hitl_resolved_removes_pendingHitl_entry', () => {
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-2',
      trigger: 'on_risky_tool',
      agent_id: 'a1',
      question: 'Run Bash:rm?',
      options: ['allow', 'block'],
      ui_variant: 'modal',
      timeout_at_unix_ms: 1_000,
    });
    expect(useGraphStore.getState().pendingHitl['u-2']).toBeDefined();
    useGraphStore.getState().applyEvent({
      type: 'hitl_resolved',
      prompt_id: 'u-2',
      choice: 'allow',
      duration_ms: 500,
    });
    expect(useGraphStore.getState().pendingHitl['u-2']).toBeUndefined();
  });

  it('hitl_timeout_removes_pendingHitl_entry', () => {
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-3',
      trigger: 'on_gap',
      agent_id: null,
      question: '?',
      options: [],
      ui_variant: 'panel',
      timeout_at_unix_ms: 1,
    });
    useGraphStore.getState().applyEvent({
      type: 'hitl_timeout',
      prompt_id: 'u-3',
      trigger: 'on_gap',
      default_action: 'abort',
    });
    expect(useGraphStore.getState().pendingHitl['u-3']).toBeUndefined();
  });

  it('notifier_dispatched_appends_to_notifierRecords_per_matching_trigger', () => {
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-4',
      trigger: 'on_failure_threshold',
      agent_id: null,
      question: '?',
      options: [],
      ui_variant: 'panel',
      timeout_at_unix_ms: 1,
    });
    useGraphStore.getState().applyEvent({
      type: 'notifier_dispatched',
      notifier_type: 'terminal_bell',
      trigger: 'on_failure_threshold',
      success: true,
    });
    useGraphStore.getState().applyEvent({
      type: 'notifier_dispatched',
      notifier_type: 'desktop',
      trigger: 'on_failure_threshold',
      success: true,
    });
    const records = useGraphStore.getState().notifierRecords['u-4'] ?? [];
    expect(records).toHaveLength(2);
    expect(records.map((r) => r.notifierType)).toEqual(['terminal_bell', 'desktop']);
    expect(records.every((r) => r.outcome === 'dispatched')).toBe(true);
  });

  it('notifier_failed_appends_record_with_error_text', () => {
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-5',
      trigger: 'on_failure_threshold',
      agent_id: null,
      question: '?',
      options: [],
      ui_variant: 'panel',
      timeout_at_unix_ms: 1,
    });
    useGraphStore.getState().applyEvent({
      type: 'notifier_failed',
      notifier_type: 'desktop',
      trigger: 'on_failure_threshold',
      error: 'permission denied',
    });
    const records = useGraphStore.getState().notifierRecords['u-5'] ?? [];
    expect(records).toHaveLength(1);
    expect(records[0]).toMatchObject({
      notifierType: 'desktop',
      outcome: 'failed',
      error: 'permission denied',
    });
  });

  it('notifier_records_only_attach_to_matching_trigger', () => {
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-failure',
      trigger: 'on_failure_threshold',
      agent_id: null,
      question: '?',
      options: [],
      ui_variant: 'panel',
      timeout_at_unix_ms: 1,
    });
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-gap',
      trigger: 'on_gap',
      agent_id: null,
      question: '?',
      options: [],
      ui_variant: 'panel',
      timeout_at_unix_ms: 1,
    });
    // Notifier event for on_gap only — must NOT attach to u-failure.
    useGraphStore.getState().applyEvent({
      type: 'notifier_dispatched',
      notifier_type: 'terminal_bell',
      trigger: 'on_gap',
      success: true,
    });
    const records = useGraphStore.getState().notifierRecords;
    expect(records['u-gap']).toHaveLength(1);
    expect(records['u-failure']).toBeUndefined();
  });

  it('resolving_a_pending_hitl_clears_its_notifier_records', () => {
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-6',
      trigger: 'on_failure_threshold',
      agent_id: null,
      question: '?',
      options: [],
      ui_variant: 'panel',
      timeout_at_unix_ms: 1,
    });
    useGraphStore.getState().applyEvent({
      type: 'notifier_dispatched',
      notifier_type: 'terminal_bell',
      trigger: 'on_failure_threshold',
      success: true,
    });
    expect(useGraphStore.getState().notifierRecords['u-6']).toBeDefined();
    useGraphStore.getState().applyEvent({
      type: 'hitl_resolved',
      prompt_id: 'u-6',
      choice: 'skip',
      duration_ms: 100,
    });
    expect(useGraphStore.getState().notifierRecords['u-6']).toBeUndefined();
  });

  it('clear_resets_pendingHitl_and_notifierRecords', () => {
    useGraphStore.getState().applyEvent({
      type: 'hitl_requested',
      prompt_id: 'u-7',
      trigger: 'on_gap',
      agent_id: null,
      question: '?',
      options: [],
      ui_variant: 'panel',
      timeout_at_unix_ms: 1,
    });
    useGraphStore.getState().applyEvent({
      type: 'notifier_dispatched',
      notifier_type: 'sound',
      trigger: 'on_gap',
      success: true,
    });
    useGraphStore.getState().clear();
    expect(useGraphStore.getState().pendingHitl).toEqual({});
    expect(useGraphStore.getState().notifierRecords).toEqual({});
  });
});

// M07.5 / ADR-0017 — the import slot + the artifact_hash_mismatch
// reducer + the review confirm/dismiss actions. The slot is the single
// source of truth the ImportPanel renders from; recordImport maps the
// A.fix-shipped discriminated ImportOutcome wire (snake_case,
// `status: 'pending' | 'installed'`) into the store's camelCase
// ImportRecord at the boundary. A 'pending' outcome carries the
// pending_review_id the review modal echoes back to the backend.
describe('graphStore imports (M07.5)', () => {
  const pendingOutcome: ImportOutcome = {
    status: 'pending',
    pending_review_id: 'pri-1',
    lock_key: 'fs-test@2.0.0',
    requires_secrets: ['OPENAI_API_KEY'],
    capabilities: ['network: api.example.com', 'shell: true'],
    l3_report: { report_id: 'vr-1', passed: true, reasons: [] },
    share_provenance: { exported_by: 'share-it@0.1.0', rebake_changes: [] },
  };

  const installedOutcome: ImportOutcome = {
    status: 'installed',
    lock_key: 'x@1.0.0',
    requires_secrets: [],
    capabilities: ['read: *'],
    l3_report: { report_id: 'vr-2', passed: true, reasons: [] },
    share_provenance: null,
  };

  beforeEach(() => {
    useGraphStore.setState({ imports: {} });
  });
  afterEach(() => {
    useGraphStore.setState({ imports: {} });
  });

  it('recordImport_maps_enriched_outcome_into_a_review_record', () => {
    useGraphStore.getState().recordImport(pendingOutcome);
    const rec = useGraphStore.getState().imports['fs-test@2.0.0'];
    expect(rec).toBeDefined();
    expect(rec?.ref).toBe('fs-test@2.0.0');
    expect(rec?.phase).toBe('review'); // status === 'pending'
    expect(rec?.capabilities).toEqual(['network: api.example.com', 'shell: true']);
    expect(rec?.requiresSecrets).toEqual(['OPENAI_API_KEY']);
    expect(rec?.l3Report).toEqual({ reportId: 'vr-1', passed: true, reasons: [] });
    expect(rec?.shareProvenance).toEqual({
      exported_by: 'share-it@0.1.0',
      rebake_changes: [],
    });
  });

  it('recordImport_maps_pending_and_installed', () => {
    // M07.5 / ADR-0017: recordImport discriminates on the wire `status`,
    // not the removed `review_required` boolean. A 'pending' outcome is
    // a held Novice review carrying the pending_review_id; an 'installed'
    // outcome is terminal and carries no review id.
    const store = useGraphStore.getState();
    store.recordImport(pendingOutcome);
    store.recordImport(installedOutcome);
    const pending = useGraphStore.getState().imports['fs-test@2.0.0'];
    const installed = useGraphStore.getState().imports['x@1.0.0'];
    expect(pending?.phase).toBe('review');
    expect(pending?.pendingReviewId).toBe('pri-1');
    expect(installed?.phase).toBe('installed');
    expect(installed?.pendingReviewId).toBeUndefined();
  });

  it('confirmImport_promotes_a_review_record_to_installed', () => {
    useGraphStore.getState().recordImport(pendingOutcome);
    useGraphStore.getState().confirmImport('fs-test@2.0.0');
    expect(useGraphStore.getState().imports['fs-test@2.0.0']?.phase).toBe('installed');
  });

  it('dismissImport_removes_the_record', () => {
    useGraphStore.getState().recordImport(pendingOutcome);
    useGraphStore.getState().dismissImport('fs-test@2.0.0');
    expect(useGraphStore.getState().imports['fs-test@2.0.0']).toBeUndefined();
  });

  it('artifact_hash_mismatch_blocks_the_artifact_with_the_drift_detail', () => {
    const ev: AgentEvent = {
      type: 'artifact_hash_mismatch',
      artifact_ref: 'fs-test@2.0.0',
      expected: 'sha256-AAAA',
      actual: 'sha256-BBBB',
    };
    useGraphStore.getState().applyEvent(ev);
    const rec = useGraphStore.getState().imports['fs-test@2.0.0'];
    expect(rec?.phase).toBe('blocked');
    expect(rec?.expected).toBe('sha256-AAAA');
    expect(rec?.actual).toBe('sha256-BBBB');
    expect(rec?.error).toContain('sha256-AAAA');
    expect(rec?.error).toContain('sha256-BBBB');
  });

  it('imports_survive_clear_like_other_install_state', () => {
    // Integrity/install state (parallels currentMcpServers / currentTier
    // — preserved across session clear() so a blocked artifact stays
    // blocked until the user reinstalls or removes it).
    useGraphStore.getState().recordImport(pendingOutcome);
    useGraphStore.getState().clear();
    expect(useGraphStore.getState().imports['fs-test@2.0.0']).toBeDefined();
  });
});
