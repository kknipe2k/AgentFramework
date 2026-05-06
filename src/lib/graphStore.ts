import type { Edge, Node } from '@xyflow/react';
import { create } from 'zustand';
import type { AgentEvent } from '../types/agent_event';

/**
 * Status field shared by Agent / Tool / MCP / Hook / Framework / Verify
 * nodes. Drives color encoding (per spec §3 Visual Design: blue=active,
 * green=complete, red=error). Gap and HITL nodes have their own
 * type-specific status palettes — see GapNodeData / HITLNodeData.
 */
export type NodeStatus = 'active' | 'complete' | 'error';

/**
 * Plan-state status per spec §3a (added by WI-03). The schema's
 * plan_created event lands at M4 — Stage C ships the type so PlanNode
 * can render synthetic placeholder data without future TS churn.
 */
export type PlanStatus = 'pending_approval' | 'approved' | 'in_progress' | 'complete' | 'aborted';

/**
 * Task-state status per spec §3a. Same M4-event-deferred pattern as
 * PlanStatus.
 */
export type TaskStatus = 'pending' | 'running' | 'done' | 'blocked' | 'failed' | 'skipped';

/**
 * Verify-hook status per spec §4a. The verify_passed/verify_failed
 * events land at M4; Stage C renders synthetic placeholder data.
 */
export type VerifyStatus = 'active' | 'pass' | 'fail';

export interface AgentNodeData extends Record<string, unknown> {
  agentId: string;
  agentName: string;
  status: NodeStatus;
  parentAgentId: string | null;
  /**
   * Cumulative input tokens charged across this agent's tool calls
   * (sum of `tool_result.tokens_in` per call). Stage D drives the
   * node-weight CSS scaling; defaults to 0 when no token data has
   * been observed.
   */
  tokensIn: number;
  /** Cumulative output tokens. Same semantics as `tokensIn`. */
  tokensOut: number;
  /**
   * Total session tokens reported on `agent_complete.tokens_total`
   * (Anthropic `message_delta.usage` running total). Distinct from the
   * `tokensIn + tokensOut` sum: providers report total + per-tool
   * separately, so the renderer carries both rather than re-deriving.
   */
  tokensTotal: number;
}

export interface ToolNodeData extends Record<string, unknown> {
  toolName: string;
  agentId: string;
  status: NodeStatus;
  durationMs: number | null;
  /**
   * Per-call input tokens (`tool_result.tokens_in`); 0 when the
   * provider does not surface per-tool attribution. Drives this
   * tool's weight in the node-scale CSS variable.
   */
  tokensIn: number;
  /** Per-call output tokens. Same semantics as `tokensIn`. */
  tokensOut: number;
}

export interface SkillNodeData extends Record<string, unknown> {
  skillName: string;
  agentId: string;
  mode: string | null;
}

/**
 * MCPNode — spec §3 + §5. Lazily spawned when a `tool_invoked` event
 * arrives with `source: 'mcp'`. Hosts the ToolNodes for its tools so
 * the agent → MCP → tool routing is visible.
 */
export interface MCPNodeData extends Record<string, unknown> {
  serverId: string;
  serverName: string;
  status: NodeStatus;
  discoveredToolCount: number | null;
}

/**
 * GapNode — spec §3 + §4b. v0.1 schema does not yet emit `gap_added`
 * events; M5 wires them. Stage C ships the component so the M5 wiring
 * lands without renderer churn.
 */
export interface GapNodeData extends Record<string, unknown> {
  gapId: string;
  kind: 'tool_missing' | 'skill_missing';
  missingName: string;
  status: 'gap';
}

/**
 * HITLNode — spec §3 + §6a. The schema declares `hitl_requested` /
 * `hitl_resolved` variants (renderer-no-op until M4 graphStore wires
 * them). Stage C ships the component for synthetic-state testing.
 */
export interface HITLNodeData extends Record<string, unknown> {
  hitlId: string;
  prompt: string;
  resolved: boolean;
}

/**
 * PlanNode — spec §3 + §3a. Synthetic placeholder fields (title,
 * taskCount, completedCount) until M4's plan primitive lands.
 */
export interface PlanNodeData extends Record<string, unknown> {
  planId: string;
  title: string;
  status: PlanStatus;
  taskCount: number;
  completedCount: number;
}

/**
 * TaskNode — spec §3 + §3a. Synthetic placeholder fields until M4.
 */
export interface TaskNodeData extends Record<string, unknown> {
  taskId: string;
  planId: string;
  title: string;
  status: TaskStatus;
  hitl: boolean;
}

/**
 * VerifyNode — spec §3 + §4a. Synthetic placeholder fields until M4
 * adds verify_started / verify_passed / verify_failed wiring.
 */
export interface VerifyNodeData extends Record<string, unknown> {
  hookId: string;
  level: string;
  status: VerifyStatus;
  durationMs: number | null;
}

/**
 * HookNode — spec §3 + §4a. Synthetic placeholder until M4 adds the
 * hook-fired primitive.
 */
export interface HookNodeData extends Record<string, unknown> {
  hookId: string;
  hookName: string;
  category: string;
  status: NodeStatus;
}

/**
 * FrameworkNode — spec §3. The graph's root, spawned on session_start.
 */
export interface FrameworkNodeData extends Record<string, unknown> {
  frameworkName: string;
  model: string;
  status: NodeStatus;
}

export type AgentReactFlowNode = Node<AgentNodeData, 'agent'>;
export type ToolReactFlowNode = Node<ToolNodeData, 'tool'>;
export type SkillReactFlowNode = Node<SkillNodeData, 'skill'>;
export type MCPReactFlowNode = Node<MCPNodeData, 'mcp'>;
export type GapReactFlowNode = Node<GapNodeData, 'gap'>;
export type HITLReactFlowNode = Node<HITLNodeData, 'hitl'>;
export type PlanReactFlowNode = Node<PlanNodeData, 'plan'>;
export type TaskReactFlowNode = Node<TaskNodeData, 'task'>;
export type VerifyReactFlowNode = Node<VerifyNodeData, 'verify'>;
export type HookReactFlowNode = Node<HookNodeData, 'hook'>;
export type FrameworkReactFlowNode = Node<FrameworkNodeData, 'framework'>;

/**
 * Discriminated union over all 11 spec §3 node types. Stage C lands
 * the eight types beyond the Stage B trio (agent/tool/skill).
 */
export type GraphNode =
  | AgentReactFlowNode
  | ToolReactFlowNode
  | SkillReactFlowNode
  | MCPReactFlowNode
  | GapReactFlowNode
  | HITLReactFlowNode
  | PlanReactFlowNode
  | TaskReactFlowNode
  | VerifyReactFlowNode
  | HookReactFlowNode
  | FrameworkReactFlowNode;

interface EdgeData extends Record<string, unknown> {
  kind: 'agent-spawn' | 'tool-call' | 'skill-load' | 'agent-mcp';
}

export type GraphEdge = Edge<EdgeData>;

interface GraphState {
  nodes: GraphNode[];
  edges: GraphEdge[];
  selectedNodeId: string | null;

  /**
   * Single entry point for translating AgentEvent into node + edge
   * mutations. Idempotent on duplicate events; order-independent for
   * non-causal events. Exhaustive over the AgentEvent union — variants
   * the renderer doesn't yet surface land in explicit no-op cases; the
   * `_exhaustive: never` check at the bottom turns any future schema
   * addition into a TS compile error rather than a silent drop.
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
  const siblings = existing.filter((n) => n.type === 'tool' && n.data.agentId === agentId).length;
  return { x: px + siblings * 120, y: 140 };
}

function skillPosition(existing: GraphNode[], agentId: string): { x: number; y: number } {
  const parent = existing.find(
    (n): n is AgentReactFlowNode => n.type === 'agent' && n.id === `agent:${agentId}`,
  );
  const px = parent ? parent.position.x : 0;
  const siblings = existing.filter((n) => n.type === 'skill' && n.data.agentId === agentId).length;
  return { x: px - 140 - siblings * 120, y: 140 };
}

function mcpPosition(existing: GraphNode[], agentId: string): { x: number; y: number } {
  // MCPNode parents the ToolNodes for its server. Layout: sit between
  // the agent and where the tools land — agent at y=0, MCP at y=70,
  // tools at y=140 (per toolPosition above).
  const parent = existing.find(
    (n): n is AgentReactFlowNode => n.type === 'agent' && n.id === `agent:${agentId}`,
  );
  const px = parent ? parent.position.x : 0;
  const mcpCount = existing.filter((n) => n.type === 'mcp').length;
  return { x: px + 60 + mcpCount * 80, y: 70 };
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
        case 'session_start': {
          // Spec §3: FrameworkNode is the graph root. Idempotent on
          // duplicate session_start with the same framework name (the
          // runtime emits one per session, but a reload-replay could
          // surface duplicates — protect the root from being doubled).
          const id = `framework:${event.framework}`;
          if (state.nodes.some((n) => n.id === id)) {
            return state;
          }
          const newNode: FrameworkReactFlowNode = {
            id,
            type: 'framework',
            position: { x: -200, y: -150 },
            data: {
              frameworkName: event.framework,
              model: event.model,
              status: 'active',
            },
          };
          return { ...state, nodes: [...state.nodes, newNode] };
        }

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
              tokensIn: 0,
              tokensOut: 0,
              tokensTotal: 0,
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

        case 'agent_complete': {
          const next = withAgentStatus(state, event.agent_id, 'complete');
          if (event.tokens_total === undefined || event.tokens_total === null) {
            return next;
          }
          const total = event.tokens_total;
          const target = `agent:${event.agent_id}`;
          return {
            ...next,
            nodes: next.nodes.map((n) =>
              n.id === target && n.type === 'agent'
                ? { ...n, data: { ...n.data, tokensTotal: total } }
                : n,
            ),
          };
        }

        case 'agent_error':
          return withAgentStatus(state, event.agent_id, 'error');

        case 'tool_invoked': {
          const id = `tool:${event.agent_id}:${event.tool_name}`;
          if (state.nodes.some((n) => n.id === id)) {
            return state;
          }
          // MCP routing per spec §3 Behavior: source='mcp' inserts an
          // MCPNode between the agent and the tool. Same MCP server
          // across multiple tools reuses the existing MCPNode.
          if (event.source === 'mcp' && event.server) {
            const mcpId = `mcp:${event.server}`;
            const mcpExists = state.nodes.some((n) => n.id === mcpId);
            const newMcp: MCPReactFlowNode | null = mcpExists
              ? null
              : {
                  id: mcpId,
                  type: 'mcp',
                  position: mcpPosition(state.nodes, event.agent_id),
                  data: {
                    serverId: event.server,
                    serverName: event.server,
                    status: 'active',
                    discoveredToolCount: null,
                  },
                };
            const newTool: ToolReactFlowNode = {
              id,
              type: 'tool',
              position: toolPosition(state.nodes, event.agent_id),
              data: {
                toolName: event.tool_name,
                agentId: event.agent_id,
                status: 'active',
                durationMs: null,
                tokensIn: 0,
                tokensOut: 0,
              },
            };
            const agentToMcpId = `edge:agent:${event.agent_id}->${mcpId}`;
            const agentToMcpExists = state.edges.some((e) => e.id === agentToMcpId);
            const newEdges: GraphEdge[] = [
              ...state.edges,
              ...(agentToMcpExists
                ? []
                : [
                    {
                      id: agentToMcpId,
                      source: `agent:${event.agent_id}`,
                      target: mcpId,
                      data: { kind: 'agent-mcp' as const },
                    },
                  ]),
              {
                id: `edge:${mcpId}->${id}`,
                source: mcpId,
                target: id,
                animated: true,
                data: { kind: 'tool-call' as const },
              },
            ];
            return {
              ...state,
              nodes: newMcp ? [...state.nodes, newMcp, newTool] : [...state.nodes, newTool],
              edges: newEdges,
            };
          }
          // Non-MCP tool — agent → tool directly (Stage B behavior +
          // animated flag).
          const newNode: ToolReactFlowNode = {
            id,
            type: 'tool',
            position: toolPosition(state.nodes, event.agent_id),
            data: {
              toolName: event.tool_name,
              agentId: event.agent_id,
              status: 'active',
              durationMs: null,
              tokensIn: 0,
              tokensOut: 0,
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
                animated: true,
                data: { kind: 'tool-call' as const },
              },
            ],
          };
        }

        case 'tool_result': {
          const id = `tool:${event.agent_id}:${event.tool_name}`;
          const tokensIn = event.tokens_in ?? 0;
          const tokensOut = event.tokens_out ?? 0;
          const agentTarget = `agent:${event.agent_id}`;
          return {
            ...state,
            nodes: state.nodes.map((n) => {
              if (n.id === id && n.type === 'tool') {
                return {
                  ...n,
                  data: {
                    ...n.data,
                    status: 'complete',
                    durationMs: event.duration_ms,
                    tokensIn,
                    tokensOut,
                  },
                };
              }
              if (n.id === agentTarget && n.type === 'agent') {
                return {
                  ...n,
                  data: {
                    ...n.data,
                    tokensIn: n.data.tokensIn + tokensIn,
                    tokensOut: n.data.tokensOut + tokensOut,
                  },
                };
              }
              return n;
            }),
            // Clear the animated flag on the inbound edge (whether
            // sourced from the agent directly or from an MCPNode parent
            // — match by target so both shapes resolve).
            edges: state.edges.map((e) => (e.target === id ? { ...e, animated: false } : e)),
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

        // No-op variants — Stage C added session_start (FrameworkNode)
        // and MCP routing inside tool_invoked. The remaining no-ops
        // light up at M4 (plan/task/verify/hook), M5 (gap/HITL), and
        // post-M5 (capability/budget) when the schema gains those
        // semantics. The exhaustive default below is the forcing
        // function: any new variant added to the schema breaks the
        // compile until handled here.
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
          const _exhaustive: never = event;
          return _exhaustive;
        }
      }
    }),

  clear: () => set({ nodes: [], edges: [], selectedNodeId: null }),

  selectNode: (id) => set({ selectedNodeId: id }),
}));
