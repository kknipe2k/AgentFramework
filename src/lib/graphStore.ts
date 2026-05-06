import type { Edge, Node } from '@xyflow/react';
import { create } from 'zustand';
import type { AgentEvent } from '../types/agent_event';

/**
 * Status field shared by every spec §3 node type. Drives color encoding
 * (per spec §3 Visual Design: blue=active, green=complete, red=error).
 * Stage C extends with `gap` (amber) and `hitl` (white/bright).
 */
export type NodeStatus = 'active' | 'complete' | 'error';

/**
 * Data attached to AgentNode instances in the React Flow graph.
 */
export interface AgentNodeData extends Record<string, unknown> {
  agentId: string;
  agentName: string;
  status: NodeStatus;
  parentAgentId: string | null;
}

/**
 * Data attached to ToolNode instances. Stage B handles the basic shape;
 * Stage C extends with `source` ("builtin" | "mcp" | "generated") +
 * `server` for MCP tools.
 */
export interface ToolNodeData extends Record<string, unknown> {
  toolName: string;
  agentId: string;
  status: NodeStatus;
  durationMs: number | null;
}

/**
 * Data attached to SkillNode instances. Skills are loaded into context
 * (not called); the edge from the parent agent is dashed (no flow
 * animation per spec §3 Behavior).
 */
export interface SkillNodeData extends Record<string, unknown> {
  skillName: string;
  agentId: string;
  mode: string | null;
}

export type AgentReactFlowNode = Node<AgentNodeData, 'agent'>;
export type ToolReactFlowNode = Node<ToolNodeData, 'tool'>;
export type SkillReactFlowNode = Node<SkillNodeData, 'skill'>;

/**
 * Discriminated union over the three Stage B node types. Stage C extends
 * with the remaining eight spec §3 types.
 */
export type GraphNode = AgentReactFlowNode | ToolReactFlowNode | SkillReactFlowNode;

interface EdgeData extends Record<string, unknown> {
  kind: 'agent-spawn' | 'tool-call' | 'skill-load';
}

/**
 * Edge variants Stage B emits. Stage C adds animated active-call edges
 * + dashed skill-load edge styling per spec §3 Behavior.
 */
export type GraphEdge = Edge<EdgeData>;

interface GraphState {
  nodes: GraphNode[];
  edges: GraphEdge[];
  selectedNodeId: string | null;

  /**
   * Single entry point for translating AgentEvent into node + edge
   * mutations. Idempotent on duplicate events; order-independent for
   * non-causal events. Exhaustive over the 36-variant AgentEvent union
   * (variants Stage B doesn't render are explicit no-ops; the
   * `_exhaustive: never` check at the bottom turns any future schema
   * addition into a TS compile error rather than a silent drop).
   */
  applyEvent: (event: AgentEvent) => void;

  /** Clear all nodes + edges + selection. Called when a new session begins. */
  clear: () => void;

  /** Set the currently-selected node (Stage D inspector panel uses this). */
  selectNode: (id: string | null) => void;
}

const AGENT_X_STRIDE = 220;

function nextAgentPosition(existing: GraphNode[]): { x: number; y: number } {
  // Naive layout for Stage B: stagger horizontally by root-agent index.
  // Stage D adds dagre. The position is the React Flow default coordinate
  // space (px); React Flow's `fitView` re-centers on mount.
  const agentCount = existing.filter((n) => n.type === 'agent').length;
  return { x: agentCount * AGENT_X_STRIDE, y: 0 };
}

function toolPosition(existing: GraphNode[], agentId: string): { x: number; y: number } {
  const parent = existing.find(
    (n): n is AgentReactFlowNode => n.type === 'agent' && n.id === `agent:${agentId}`,
  );
  const px = parent ? parent.position.x : 0;
  const siblings = existing.filter(
    (n) => n.type === 'tool' && n.data.agentId === agentId,
  ).length;
  return { x: px + siblings * 120, y: 140 };
}

function skillPosition(existing: GraphNode[], agentId: string): { x: number; y: number } {
  const parent = existing.find(
    (n): n is AgentReactFlowNode => n.type === 'agent' && n.id === `agent:${agentId}`,
  );
  const px = parent ? parent.position.x : 0;
  const siblings = existing.filter(
    (n) => n.type === 'skill' && n.data.agentId === agentId,
  ).length;
  return { x: px - 140 - siblings * 120, y: 140 };
}

function withAgentStatus(state: GraphState, agentId: string, status: NodeStatus): GraphState {
  const target = `agent:${agentId}`;
  return {
    ...state,
    nodes: state.nodes.map((n) =>
      n.id === target && n.type === 'agent' ? { ...n, data: { ...n.data, status } } : n,
    ),
  };
}

export const useGraphStore = create<GraphState>((set) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,

  applyEvent: (event) =>
    set((state) => {
      switch (event.type) {
        case 'agent_spawned': {
          const id = `agent:${event.agent_id}`;
          if (state.nodes.some((n) => n.id === id)) {
            return state;
          }
          const newNode: AgentReactFlowNode = {
            id,
            type: 'agent',
            position: nextAgentPosition(state.nodes),
            data: {
              agentId: event.agent_id,
              agentName: event.agent_name,
              status: 'active',
              parentAgentId: event.parent_id ?? null,
            },
          };
          const nodes = [...state.nodes, newNode];
          const edges = event.parent_id
            ? [
                ...state.edges,
                {
                  id: `edge:agent:${event.parent_id}->${event.agent_id}`,
                  source: `agent:${event.parent_id}`,
                  target: id,
                  data: { kind: 'agent-spawn' as const },
                },
              ]
            : state.edges;
          return { ...state, nodes, edges };
        }

        case 'agent_complete':
          return withAgentStatus(state, event.agent_id, 'complete');

        case 'agent_error':
          return withAgentStatus(state, event.agent_id, 'error');

        case 'tool_invoked': {
          const id = `tool:${event.agent_id}:${event.tool_name}`;
          if (state.nodes.some((n) => n.id === id)) {
            return state;
          }
          const newNode: ToolReactFlowNode = {
            id,
            type: 'tool',
            position: toolPosition(state.nodes, event.agent_id),
            data: {
              toolName: event.tool_name,
              agentId: event.agent_id,
              status: 'active',
              durationMs: null,
            },
          };
          return {
            ...state,
            nodes: [...state.nodes, newNode],
            edges: [
              ...state.edges,
              {
                id: `edge:agent:${event.agent_id}->${id}`,
                source: `agent:${event.agent_id}`,
                target: id,
                data: { kind: 'tool-call' as const },
              },
            ],
          };
        }

        case 'tool_result': {
          const id = `tool:${event.agent_id}:${event.tool_name}`;
          return {
            ...state,
            nodes: state.nodes.map((n) =>
              n.id === id && n.type === 'tool'
                ? {
                    ...n,
                    data: {
                      ...n.data,
                      status: 'complete',
                      durationMs: event.duration_ms,
                    },
                  }
                : n,
            ),
          };
        }

        case 'skill_loaded': {
          const id = `skill:${event.agent_id}:${event.skill_name}`;
          if (state.nodes.some((n) => n.id === id)) {
            return state;
          }
          const newNode: SkillReactFlowNode = {
            id,
            type: 'skill',
            position: skillPosition(state.nodes, event.agent_id),
            data: {
              skillName: event.skill_name,
              agentId: event.agent_id,
              mode: event.mode ?? null,
            },
          };
          return {
            ...state,
            nodes: [...state.nodes, newNode],
            edges: [
              ...state.edges,
              {
                id: `edge:agent:${event.agent_id}->${id}`,
                source: `agent:${event.agent_id}`,
                target: id,
                data: { kind: 'skill-load' as const },
              },
            ],
          };
        }

        // No-op variants — Stage B has no node representation. Stage C
        // wires session_start → FrameworkNode, MCP-source tool_invoked →
        // MCPNode, etc. M4 wires plan/task/verify; M5 wires gap/HITL;
        // budget + token are Stage D inspector data.
        case 'session_start':
        case 'session_end':
        case 'tool_error':
        case 'plan_created':
        case 'plan_approved':
        case 'plan_rejected':
        case 'task_started':
        case 'task_completed':
        case 'task_failed':
        case 'task_rolled_back':
        case 'task_escalated':
        case 'mode_changed':
        case 'verify_started':
        case 'verify_passed':
        case 'verify_failed':
        case 'rail_triggered':
        case 'skill_missing':
        case 'tool_missing':
        case 'gap_resolved':
        case 'hitl_requested':
        case 'hitl_resolved':
        case 'capability_violation':
        case 'capability_grant':
        case 'budget_warn':
        case 'budget_downshift':
        case 'budget_suspended':
        case 'budget_exceeded':
        case 'stream_text':
        case 'decision_record':
        case 'token_usage':
          return state;

        default: {
          // Exhaustiveness check — TS narrows event to `never` here. If
          // the AgentEvent union grows (M4+ wires plan/task variants;
          // M6 adds MCP variants), this line errors at compile time
          // and forces explicit handling above rather than silent drop.
          const _exhaustive: never = event;
          return _exhaustive;
        }
      }
    }),

  clear: () => set({ nodes: [], edges: [], selectedNodeId: null }),

  selectNode: (id) => set({ selectedNodeId: id }),
}));
