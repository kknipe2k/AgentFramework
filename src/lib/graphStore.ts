import type { Edge, Node } from '@xyflow/react';
import { create } from 'zustand';
import type {
  AgentEvent,
  CapabilityKindRef2,
  GapSeverity,
  GapSource,
  HitlTriggerRef,
  HitlUiVariantRef,
  HookCategoryRef,
  McpTransportKind,
  OnFailureRef,
  RailPolicy,
  TierRef,
} from '../types/agent_event';
import type { ImportOutcome } from './ipc';

/**
 * Status field shared by Agent / Tool / MCP / Hook / Framework / Verify
 * nodes. Drives color encoding (per spec §3 Visual Design: blue=active,
 * green=complete, red=error). Gap and HITL nodes have their own
 * type-specific status palettes — see GapNodeData / HITLNodeData.
 */
export type NodeStatus = 'active' | 'complete' | 'error';

/**
 * Plan-state status per spec §3a. M04 Stage B added 'awaiting_approval'
 * (transient between plan_created with approval_required and the
 * subsequent plan_approved) and 'awaiting_replan' (after plan_revised).
 */
export type PlanStatus =
  | 'pending_approval'
  | 'awaiting_approval'
  | 'approved'
  | 'in_progress'
  | 'awaiting_replan'
  | 'complete'
  | 'aborted';

/**
 * Task-state status per spec §3a. M04 Stage B added 'escalated' (post
 * failure_count >= max_failures).
 */
export type TaskStatus =
  | 'pending'
  | 'running'
  | 'done'
  | 'blocked'
  | 'failed'
  | 'skipped'
  | 'escalated';

/**
 * Verify-hook status per spec §4a. M04 Stage D wires the live event
 * stream — `active` on verify_started, `pass` on verify_passed, `fail`
 * on verify_failed (the M03.C synthetic-state pattern remains for
 * tests that drive the renderer in isolation).
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
 * GapNode — spec §3 + §4b. M05.A wires gap events end-to-end: the four
 * `*_missing` event variants mount a GapNode; `gap_resolved` dismisses it
 * by `gapId`. `severity` drives the visual tier (critical/important/advisory/
 * requested → red/orange/amber/blue pulse); `suggestedAction` surfaces in
 * the GapPanel; `requestedVia` distinguishes loader-driven (Layer 1) from
 * `request_capability`-driven (Layer 2) gaps for renderer copy.
 */
export interface GapNodeData extends Record<string, unknown> {
  gapId: string;
  kind: 'tool_missing' | 'skill_missing' | 'mcp_missing' | 'agent_missing';
  missingName: string;
  agentId: string;
  severity: GapSeverity;
  suggestedAction: string;
  requestedVia: GapSource;
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
 * PlanNode — spec §3 + §3a. M04 Stage B drives live state from
 * plan_created / plan_approval_requested / plan_approved / plan_revised
 * / plan_aborted / plan_complete events; Stage C lights up the visual
 * surface (status badge + animated edge to currently-running task).
 */
export interface PlanNodeData extends Record<string, unknown> {
  planId: string;
  title: string;
  status: PlanStatus;
  taskCount: number;
  completedCount: number;
  approvalRequired: boolean;
  /** Free-text reason recorded for revised / aborted transitions. */
  lastTransitionReason: string | null;
  /** End-to-end duration recorded on plan_complete. */
  durationMs: number | null;
}

/**
 * TaskNode — spec §3 + §3a. M04 Stage B drives live state from
 * task_started / task_completed / task_failed / task_skipped /
 * task_escalated / task_rolled_back events.
 */
export interface TaskNodeData extends Record<string, unknown> {
  taskId: string;
  planId: string;
  agentId: string | null;
  title: string;
  status: TaskStatus;
  hitl: boolean;
  failureCount: number;
  maxFailures: number | null;
  /** Recorded on task_failed / task_skipped / task_rolled_back. */
  lastError: string | null;
  /** Recorded on task_completed. */
  durationMs: number | null;
  /** Recorded on task_rolled_back (drift carve-out per Stage B). */
  rollbackSnapshotId: string | null;
}

/**
 * VerifyNode — spec §3 + §4a. M04 Stage D drives live state from
 * verify_started / verify_passed / verify_failed events for hooks
 * with `category === 'verify'`. Other categories route to HookNode.
 */
export interface VerifyNodeData extends Record<string, unknown> {
  hookId: string;
  /** `quick | standard | full` if the framework set a level. */
  level: string | null;
  /** Lifecycle moment the hook fired (e.g., `post_task`). */
  firingPoint: string;
  status: VerifyStatus;
  durationMs: number | null;
  /** Captured stdout preview from verify_passed (truncated upstream). */
  outputPreview: string | null;
  /** Failure message from verify_failed. */
  error: string | null;
  /** `block | warn | rollback` from verify_failed. */
  onFailure: OnFailureRef | null;
}

/**
 * HookNode — spec §3 + §4a. Generic hook surface for non-verify
 * categories (`lint | build | test | custom`). M04 Stage D drives
 * live state from verify_started / verify_passed / verify_failed.
 */
export interface HookNodeData extends Record<string, unknown> {
  hookId: string;
  hookName: string;
  category: HookCategoryRef;
  /** Lifecycle moment the hook fired. */
  firingPoint: string;
  status: NodeStatus;
  durationMs: number | null;
  error: string | null;
}

/**
 * Triggered rail entry — spec §4a. Driven by `rail_triggered` events.
 * Stored on the store rather than as a node since rails are
 * cross-cutting policy events; M05 wires them into the capability
 * enforcer's UI surface.
 */
export interface TriggeredRail {
  railId: string;
  policy: RailPolicy;
  firingPoint: string;
  message: string;
  agentId: string | null;
}

/**
 * Outstanding HITL prompt — spec §6a (M04 Stage E). Driven by
 * `hitl_requested` events; cleared on the matching `hitl_resolved` /
 * `hitl_timeout`. The renderer's HITLPanel / HITLModal / HITLToast
 * subscribes and dispatches `respond_hitl(prompt_id, choice)` on user
 * action.
 */
export interface PendingHitl {
  promptId: string;
  trigger: HitlTriggerRef;
  agentId: string | null;
  question: string;
  options: string[];
  uiVariant: HitlUiVariantRef;
  timeoutAtUnixMs: number;
}

/**
 * Notifier dispatch record — spec §6a. Driven by `notifier_dispatched`
 * / `notifier_failed` events. Append-only diagnostic log surfaced in
 * the inspector when a HITL prompt is open; cleared at session reset.
 */
export interface NotifierRecord {
  notifierType: string;
  trigger: HitlTriggerRef;
  outcome: 'dispatched' | 'failed';
  error: string | null;
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

/**
 * Budget status driving the BudgetHeaderBar color gradient
 * (spec §2a — green/amber/orange/red/exceeded).
 */
export type BudgetStatus = 'ok' | 'warn' | 'downshift' | 'suspended' | 'exceeded';

/**
 * Session-level budget snapshot. Driven by the four budget events
 * (budget_warn / budget_downshift / budget_suspended / budget_exceeded).
 * v0.1's BudgetHeaderBar reads spent_usd + cap_usd to render the color
 * gradient + numeric badge.
 */
export interface BudgetState {
  spentUsd: number;
  capUsd: number;
  percent: number;
  status: BudgetStatus;
}

/**
 * One uncertain tool invocation surfaced by the recovery flow
 * (spec §1b). The 4-action prompt is rendered by `UncertaintyPrompt`;
 * resolved invocations are removed from the list.
 */
export interface UncertainInvocation {
  invocationId: string;
  toolName?: string;
  agentId?: string;
}

/**
 * Spec §8.security L2a (M05 Stage B): per-agent record of the most-recent
 * `capability_violation` event. Map shape (keyed by `agent_id`) so the
 * inspector + capability-violation modal can look up the violation that
 * is currently blocking a given agent without scanning a log.
 * Last-write-wins on repeat violations from the same agent — the renderer
 * surfaces the most recent rejection state.
 */
export interface CapabilityViolationRecord {
  capabilityKind: string;
  requestedAction: string;
  declaredScope: string;
  timestamp: number;
}

/**
 * Spec §M7 (M07 Stage E; ADR-0015): one imported artifact's review +
 * install + integrity state, keyed by `ref` (the `lock_key` for a
 * successful import, or the `artifact_ref` for a hash-mismatch). The
 * boundary mapping from the snake_case wire (`ImportOutcome`) lives in
 * `recordImport`; the renderer reads camelCase here.
 *
 * `phase` semantics:
 * - `'review'` — Novice (`review_required: true`) sees the disclosure
 *   modal before the artifact surfaces. Promoted-within-bounds is
 *   `'installed'` directly (L4 auto-accept; the backend installed
 *   atomically — confirm-before-use is a renderer concern).
 * - `'installed'` — the artifact is live in the framework; the panel
 *   shows it as a simple row.
 * - `'blocked'` — `artifact_hash_mismatch` fired (spec §2214); the
 *   panel surfaces the Reinstall / Remove prompt; the artifact does
 *   NOT run with drifted bytes.
 */
export interface ImportRecord {
  ref: string;
  phase: 'review' | 'installed' | 'blocked';
  capabilities: string[];
  requiresSecrets: string[];
  l3Report: { reportId: string; passed: boolean; reasons: string[] } | null;
  /** ADR-0005 trust block when present; `null` when unexported. `unknown` covers `null` for ESLint's redundancy rule. */
  shareProvenance: unknown;
  error?: string;
  expected?: string;
  actual?: string;
}

/**
 * Spec §8.security L2a (M05 Stage B): one row of the capability-grant
 * log. Append-only. Driven by `capability_grant` events emitted by the
 * framework loader (root grants; `parentAgentId` absent) and by the
 * SDK's L2a narrowing path (sub-agent spawns; `parentAgentId` present).
 * The inspector renders the log so the user can audit "what was granted
 * to whom, from where."
 */
export interface CapabilityGrantRecord {
  parentAgentId: string | null;
  grantedTo: string;
  capabilityKind: string;
  resource: string;
  narrowedFrom: string | null;
  timestamp: number;
}

/**
 * Spec §8.security L4 (M05 Stage D): per-agent record of the most-recent
 * `tier_violation` event. Distinct from `capabilityViolations` because
 * the L4 gate fires BEFORE L1 — same event-store shape but different
 * routing (tier-violation modal in Settings vs. capability-violation
 * inspector). Last-write-wins per agent.
 */
export interface TierViolationRecord {
  tier: TierRef;
  capabilityKind: CapabilityKindRef2;
  attemptedAction: string;
  timestamp: number;
}

/**
 * Spec §5 (M06.D): live state of one installed MCP server, keyed by
 * server name in `currentMcpServers`. Driven by the M06.C lifecycle
 * events (`mcp_installed` / `mcp_uninstalled` / `mcp_auth_granted`);
 * Stage E's MCPNode + Settings panel read this. `currentMcpServers`
 * follows the M05.D `currentTier` pattern — persistent across `clear()`
 * because MCP servers are registry-backed install state, not
 * per-session graph state (test files reset it explicitly in
 * `beforeEach`).
 */
export interface McpServerStatusRecord {
  name: string;
  transportKind: McpTransportKind | null;
  hasAuth: boolean;
  status: string;
}

/**
 * Spec §5a step 5 (M06.D): one `tool_alias_ambiguous` warning. The
 * runtime emits this when a server connect makes a short tool name
 * ambiguous; the framework silences it by pinning the name in
 * `mcp_aliases`. Append-only, per-session (cleared on `clear()`).
 */
export interface ToolAliasWarning {
  name: string;
  candidates: string[];
  timestamp: number;
}

interface GraphState {
  nodes: GraphNode[];
  edges: GraphEdge[];
  selectedNodeId: string | null;
  /**
   * Spec §4a: the full triggered-rail history for the current session.
   * M04 Stage D wires `rail_triggered` events into this list; M05
   * surfaces them in the capability-enforcer UI. Append-only.
   */
  triggeredRails: TriggeredRail[];
  /**
   * Spec §6a (M04 Stage E): outstanding HITL prompts keyed by prompt_id.
   * `hitl_requested` inserts; `hitl_resolved` / `hitl_timeout` deletes.
   * v0.1 single-session per spec §0d expects at most one entry at a
   * time, but the keyed map keeps the wiring sound for concurrent
   * prompts (per_task / per_epic in framework JSON).
   */
  pendingHitl: Record<string, PendingHitl>;
  /**
   * Spec §6a: per-prompt notifier dispatch records. Cleared when the
   * owning prompt resolves or times out. Renders in the inspector as a
   * "Notifications fired" list alongside the prompt UI.
   */
  notifierRecords: Record<string, NotifierRecord[]>;
  /**
   * Spec §2a (M04 Stage F): session-level budget snapshot. Driven by
   * the four budget_* events. `null` until the first event lands so the
   * header bar can render an inactive state distinct from "$0.00 of $0".
   */
  budget: BudgetState | null;
  /**
   * Spec §1b (M04 Stage F): uncertain tool invocations awaiting user
   * resolution. Populated by the cold-start resume flow; resolved
   * invocations are removed in-place by `respond_uncertainty`.
   */
  uncertainInvocations: UncertainInvocation[];
  /**
   * Spec §8.security L2a (M05 Stage B): per-agent most-recent
   * capability-violation state, keyed by `agent_id`. Inspector +
   * capability-violation modal read this to surface what's blocking
   * an agent's dispatch. Last-write-wins on repeat violations.
   */
  capabilityViolations: Record<string, CapabilityViolationRecord>;
  /**
   * Spec §8.security L2a (M05 Stage B): append-only log of every
   * granted capability — root grants from the framework loader +
   * narrowed grants from sub-agent spawns. Inspector renders the log
   * for audit-visibility.
   */
  capabilityGrants: CapabilityGrantRecord[];
  /**
   * Spec §8.security L4 (M05 Stage D): current user tier. First-run
   * default is `'novice'` (matches the runtime's `Tier::default()`).
   * Updated by `tier_transition` events; the renderer's Settings panel
   * reads this to display the current tier + render the promote/demote
   * affordance correctly.
   */
  currentTier: TierRef;
  /**
   * Spec §8.security L4 (M05 Stage D): per-agent most-recent
   * `tier_violation` state, keyed by `agent_id`. Renderer's
   * tier-violation modal reads this. Last-write-wins on repeat
   * violations from the same agent.
   */
  tierViolations: Record<string, TierViolationRecord>;
  /**
   * Spec §5 (M06.D): installed MCP servers keyed by server name.
   * Driven by `mcp_installed` / `mcp_uninstalled` / `mcp_auth_granted`.
   * Persistent across `clear()` (registry-backed install state, like
   * `currentTier`) — test files reset it in `beforeEach`. Stage E's
   * MCPNode + Settings panel consume it.
   */
  currentMcpServers: Record<string, McpServerStatusRecord>;
  /**
   * Spec §5a step 5 (M06.D): append-only `tool_alias_ambiguous`
   * warnings for the current session. Renderer surfaces a warning
   * toast; cleared on `clear()`.
   */
  toolAliasWarnings: ToolAliasWarning[];
  /**
   * Spec §5 (M06.E): in-flight MCP tool calls keyed by server name,
   * value = the active tool's node id (`tool:<agent>:<tool>`). Set when
   * a `tool_invoked` with `source: 'mcp'` lands; cleared on the matching
   * `tool_result`. Drives the MCPNode active-call animation. Per-session
   * state — reset on `clear()` (an active-call glow must not leak into a
   * new session); diverges from the `<test_isolation_audit>` slot claim
   * deliberately. Test files still reset it in `beforeEach`.
   */
  activeMcpCalls: Record<string, string>;
  /**
   * Spec §M7 (M07 Stage E; ADR-0015): import review + install +
   * integrity state, keyed by the artifact ref. `recordImport` maps a
   * successful `import_artifact` outcome in; the `artifact_hash_mismatch`
   * reducer branch transitions an artifact to `'blocked'`. The
   * ImportPanel renders directly from this slot. Preserved across
   * `clear()` (parallels currentMcpServers / currentTier —
   * installation/integrity state, not per-session graph state).
   */
  imports: Record<string, ImportRecord>;

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

  /**
   * Stage F: record an uncertain tool invocation from the recovery flow.
   * Idempotent on `invocationId`. The renderer's UncertaintyPrompt
   * iterates this list.
   */
  recordUncertainInvocation: (invocation: UncertainInvocation) => void;

  /**
   * Stage F: remove an invocation from the uncertain list (called by
   * UncertaintyPrompt after a successful respond_uncertainty IPC).
   */
  resolveUncertainInvocation: (invocationId: string) => void;

  /**
   * M07.E: map a successful `import_artifact` outcome into the
   * `imports` slot. `review_required: true` → `'review'`; otherwise
   * `'installed'`. Snake_case wire keys are mapped to camelCase here.
   */
  recordImport: (outcome: ImportOutcome) => void;

  /** M07.E: promote a `'review'` record to `'installed'` (user Install). */
  confirmImport: (ref: string) => void;

  /** M07.E: drop a record (user Reject on review, or Remove on blocked). */
  dismissImport: (ref: string) => void;
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

function updatePlanData(
  state: GraphState,
  planId: string,
  updater: (data: PlanNodeData) => PlanNodeData,
): GraphState {
  const target = `plan:${planId}`;
  return {
    ...state,
    nodes: state.nodes.map((n) =>
      n.id === target && n.type === 'plan' ? { ...n, data: updater(n.data) } : n,
    ),
  };
}

function updateTaskData(
  state: GraphState,
  taskId: string,
  updater: (data: TaskNodeData) => TaskNodeData,
): GraphState {
  const target = `task:${taskId}`;
  return {
    ...state,
    nodes: state.nodes.map((n) =>
      n.id === target && n.type === 'task' ? { ...n, data: updater(n.data) } : n,
    ),
  };
}

/**
 * Stack hook nodes vertically below the task layer (y=180 lane). M04.D
 * keeps positioning naive — actual hook→task linking lands when the
 * SDK's plan loop wires hooks to specific tasks (M07).
 */
function nextHookPosition(existing: GraphNode[]): { x: number; y: number } {
  const hookCount = existing.filter((n) => n.type === 'hook' || n.type === 'verify').length;
  return { x: hookCount * 160, y: 320 };
}

function updateVerifyData(
  state: GraphState,
  hookId: string,
  updater: (data: VerifyNodeData) => VerifyNodeData,
): GraphState {
  const target = `verify:${hookId}`;
  return {
    ...state,
    nodes: state.nodes.map((n) =>
      n.id === target && n.type === 'verify' ? { ...n, data: updater(n.data) } : n,
    ),
  };
}

function updateHookData(
  state: GraphState,
  hookId: string,
  updater: (data: HookNodeData) => HookNodeData,
): GraphState {
  const target = `hook:${hookId}`;
  return {
    ...state,
    nodes: state.nodes.map((n) =>
      n.id === target && n.type === 'hook' ? { ...n, data: updater(n.data) } : n,
    ),
  };
}

/**
 * Mount a GapNode keyed by `${kind}:${missingName}:${agentId}` so that
 * re-emissions of the same gap (same kind + same missing primitive + same
 * agent) are idempotent. Multi-source emissions (Layer 1 loader vs Layer 2
 * request_capability) for the same primitive collapse to one node — the
 * latest event wins on severity / suggestedAction / requestedVia.
 */
function gapNodeId(kind: GapNodeData['kind'], missingName: string, agentId: string): string {
  return `gap:${kind}:${missingName}:${agentId}`;
}

function nextGapPosition(existing: GraphNode[]): { x: number; y: number } {
  // GapNodes sit in the right margin so the agent-tool-skill layout stays
  // unobstructed. Stagger vertically by gap count.
  const gapCount = existing.filter((n) => n.type === 'gap').length;
  return { x: 600, y: gapCount * 100 };
}

function mountOrUpdateGap(state: GraphState, data: GapNodeData): GraphState {
  const id = gapNodeId(data.kind, data.missingName, data.agentId);
  const existing = state.nodes.find((n) => n.id === id);
  if (existing) {
    return {
      ...state,
      nodes: state.nodes.map((n) => (n.id === id && n.type === 'gap' ? { ...n, data } : n)),
    };
  }
  const newNode: GapReactFlowNode = {
    id,
    type: 'gap',
    position: nextGapPosition(state.nodes),
    data,
  };
  return { ...state, nodes: [...state.nodes, newNode] };
}

export const useGraphStore = create<GraphState>((set) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,
  triggeredRails: [],
  pendingHitl: {},
  notifierRecords: {},
  budget: null,
  uncertainInvocations: [],
  capabilityViolations: {},
  capabilityGrants: [],
  currentTier: 'novice',
  tierViolations: {},
  currentMcpServers: {},
  toolAliasWarnings: [],
  activeMcpCalls: {},
  imports: {},

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
              // Spec §5 (M06.E): mark the call in-flight so the MCPNode
              // animates. Keyed by server name, value = the tool node id
              // so `tool_result` can clear it without server context.
              activeMcpCalls: { ...state.activeMcpCalls, [event.server]: id },
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
          // Spec §5 (M06.E): the in-flight MCP call (if any) for this
          // tool node id completes — drop it so the MCPNode animation
          // stops. `tool_result` has no server context, so match by the
          // tool node id stored as the value.
          const activeMcpCalls = Object.fromEntries(
            Object.entries(state.activeMcpCalls).filter(([, toolId]) => toolId !== id),
          );
          return {
            ...state,
            activeMcpCalls,
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

        // ── Plan / Task lifecycle (spec §3a; M04 Stage B) ──
        // Stage B implements pass-through state mutations; Stage C
        // wires the visual surface (status badges + ApprovalPanel +
        // animated edge from PlanNode → currently-running TaskNode).

        case 'plan_created': {
          const id = `plan:${event.plan_id}`;
          if (state.nodes.some((n) => n.id === id)) {
            return state;
          }
          const newNode: PlanReactFlowNode = {
            id,
            type: 'plan',
            position: { x: 0, y: -300 },
            data: {
              planId: event.plan_id,
              title: event.title,
              status: event.approval_required ? 'pending_approval' : 'approved',
              taskCount: event.task_count,
              completedCount: 0,
              approvalRequired: event.approval_required,
              lastTransitionReason: null,
              durationMs: null,
            },
          };
          return { ...state, nodes: [...state.nodes, newNode] };
        }

        case 'plan_approval_requested':
          return updatePlanData(state, event.plan_id, (data) => ({
            ...data,
            status: 'awaiting_approval',
          }));

        case 'plan_approved':
          return updatePlanData(state, event.plan_id, (data) => ({
            ...data,
            status: 'in_progress',
          }));

        case 'plan_revised':
          return updatePlanData(state, event.plan_id, (data) => ({
            ...data,
            status: 'awaiting_replan',
            lastTransitionReason: event.revision_reason,
          }));

        case 'plan_aborted':
          return updatePlanData(state, event.plan_id, (data) => ({
            ...data,
            status: 'aborted',
            lastTransitionReason: event.reason,
          }));

        case 'plan_complete':
          return updatePlanData(state, event.plan_id, (data) => ({
            ...data,
            status: 'complete',
            durationMs: event.duration_ms,
          }));

        case 'task_started': {
          const id = `task:${event.task_id}`;
          const exists = state.nodes.some((n) => n.id === id);
          if (exists) {
            return updateTaskData(state, event.task_id, (data) => ({
              ...data,
              status: 'running',
              agentId: event.agent_id,
            }));
          }
          const newNode: TaskReactFlowNode = {
            id,
            type: 'task',
            position: { x: 0, y: -180 },
            data: {
              taskId: event.task_id,
              planId: event.plan_id,
              agentId: event.agent_id,
              title: '',
              status: 'running',
              hitl: false,
              failureCount: 0,
              maxFailures: null,
              lastError: null,
              durationMs: null,
              rollbackSnapshotId: null,
            },
          };
          return { ...state, nodes: [...state.nodes, newNode] };
        }

        case 'task_completed': {
          const next = updateTaskData(state, event.task_id, (data) => ({
            ...data,
            status: 'done',
            durationMs: event.duration_ms,
          }));
          return updatePlanData(next, event.plan_id, (data) => ({
            ...data,
            completedCount: data.completedCount + 1,
          }));
        }

        case 'task_failed':
          return updateTaskData(state, event.task_id, (data) => ({
            ...data,
            status: 'failed',
            failureCount: event.failure_count,
            lastError: event.error,
          }));

        case 'task_skipped':
          return updateTaskData(state, event.task_id, (data) => ({
            ...data,
            status: 'skipped',
            lastError: event.reason,
          }));

        case 'task_escalated':
          return updateTaskData(state, event.task_id, (data) => ({
            ...data,
            status: 'escalated',
            failureCount: event.failure_count,
            maxFailures: event.max_failures,
          }));

        case 'task_rolled_back':
          return updateTaskData(state, event.task_id, (data) => ({
            ...data,
            status: 'failed',
            rollbackSnapshotId: event.snapshot_id,
          }));

        // ── Verify / Hook / Rail (spec §4a; M04 Stage D) ──
        // verify_started: spawn a VerifyNode (`category === 'verify'`)
        // or HookNode (other categories), keyed by hook_id. Idempotent
        // on re-emit — re-emitting verify_started for the same hook_id
        // updates the existing node's status back to active.

        case 'verify_started': {
          if (event.category === 'verify') {
            const id = `verify:${event.hook_id}`;
            const exists = state.nodes.some((n) => n.id === id);
            if (exists) {
              return updateVerifyData(state, event.hook_id, (data) => ({
                ...data,
                status: 'active',
                firingPoint: event.firing_point,
                level: event.level ?? null,
                durationMs: null,
                outputPreview: null,
                error: null,
                onFailure: null,
              }));
            }
            const newNode: VerifyReactFlowNode = {
              id,
              type: 'verify',
              position: nextHookPosition(state.nodes),
              data: {
                hookId: event.hook_id,
                level: event.level ?? null,
                firingPoint: event.firing_point,
                status: 'active',
                durationMs: null,
                outputPreview: null,
                error: null,
                onFailure: null,
              },
            };
            return { ...state, nodes: [...state.nodes, newNode] };
          }
          const id = `hook:${event.hook_id}`;
          const exists = state.nodes.some((n) => n.id === id);
          if (exists) {
            return updateHookData(state, event.hook_id, (data) => ({
              ...data,
              status: 'active',
              firingPoint: event.firing_point,
              category: event.category,
              durationMs: null,
              error: null,
            }));
          }
          const newNode: HookReactFlowNode = {
            id,
            type: 'hook',
            position: nextHookPosition(state.nodes),
            data: {
              hookId: event.hook_id,
              hookName: event.hook_id,
              category: event.category,
              firingPoint: event.firing_point,
              status: 'active',
              durationMs: null,
              error: null,
            },
          };
          return { ...state, nodes: [...state.nodes, newNode] };
        }

        case 'verify_passed': {
          // The event payload doesn't carry category; both VerifyNode
          // (verify category) and HookNode (other categories) are
          // candidates. Update whichever exists for this hook_id.
          const verifyTarget = `verify:${event.hook_id}`;
          if (state.nodes.some((n) => n.id === verifyTarget)) {
            return updateVerifyData(state, event.hook_id, (data) => ({
              ...data,
              status: 'pass',
              durationMs: event.duration_ms,
              outputPreview: event.output_preview ?? null,
            }));
          }
          return updateHookData(state, event.hook_id, (data) => ({
            ...data,
            status: 'complete',
            durationMs: event.duration_ms,
          }));
        }

        case 'verify_failed': {
          const verifyTarget = `verify:${event.hook_id}`;
          if (state.nodes.some((n) => n.id === verifyTarget)) {
            return updateVerifyData(state, event.hook_id, (data) => ({
              ...data,
              status: 'fail',
              durationMs: event.duration_ms,
              error: event.error,
              onFailure: event.on_failure,
            }));
          }
          return updateHookData(state, event.hook_id, (data) => ({
            ...data,
            status: 'error',
            durationMs: event.duration_ms,
            error: event.error,
          }));
        }

        case 'rail_triggered':
          return {
            ...state,
            triggeredRails: [
              ...state.triggeredRails,
              {
                railId: event.rail_id,
                policy: event.policy,
                firingPoint: event.firing_point,
                message: event.message,
                agentId: event.agent_id ?? null,
              },
            ],
          };

        // ── HITL (spec §6a; M04 Stage E) ──
        case 'hitl_requested': {
          const { prompt_id } = event;
          return {
            ...state,
            pendingHitl: {
              ...state.pendingHitl,
              [prompt_id]: {
                promptId: prompt_id,
                trigger: event.trigger,
                agentId: event.agent_id ?? null,
                question: event.question,
                options: event.options,
                uiVariant: event.ui_variant,
                timeoutAtUnixMs: event.timeout_at_unix_ms,
              },
            },
          };
        }

        case 'hitl_resolved': {
          const { [event.prompt_id]: _resolved, ...rest } = state.pendingHitl;
          // The notifier records for this prompt are no longer surfaced
          // once the prompt resolves; clear them to keep the map bounded.
          const { [event.prompt_id]: _records, ...remainingRecords } = state.notifierRecords;
          return { ...state, pendingHitl: rest, notifierRecords: remainingRecords };
        }

        case 'hitl_timeout': {
          const { [event.prompt_id]: _timedOut, ...rest } = state.pendingHitl;
          const { [event.prompt_id]: _records, ...remainingRecords } = state.notifierRecords;
          return { ...state, pendingHitl: rest, notifierRecords: remainingRecords };
        }

        case 'notifier_dispatched': {
          // Notifier records are tagged by trigger; the prompt_id isn't
          // on the event, so we attach the record to every pending HITL
          // for that trigger. v0.1 single-session means at most one open
          // prompt; this remains sound when multiple are open per the
          // multi-prompt map shape.
          const next = { ...state.notifierRecords };
          for (const id of Object.keys(state.pendingHitl)) {
            const pending = state.pendingHitl[id];
            if (pending !== undefined && pending.trigger === event.trigger) {
              next[id] = [
                ...(next[id] ?? []),
                {
                  notifierType: event.notifier_type,
                  trigger: event.trigger,
                  outcome: 'dispatched',
                  error: null,
                },
              ];
            }
          }
          return { ...state, notifierRecords: next };
        }

        case 'notifier_failed': {
          const next = { ...state.notifierRecords };
          for (const id of Object.keys(state.pendingHitl)) {
            const pending = state.pendingHitl[id];
            if (pending !== undefined && pending.trigger === event.trigger) {
              next[id] = [
                ...(next[id] ?? []),
                {
                  notifierType: event.notifier_type,
                  trigger: event.trigger,
                  outcome: 'failed',
                  error: event.error,
                },
              ];
            }
          }
          return { ...state, notifierRecords: next };
        }

        case 'budget_warn': {
          // Spec §2a (M04 Stage F): the four budget events drive the
          // BudgetHeaderBar's color gradient + spend display.
          return {
            ...state,
            budget: {
              spentUsd: event.spent_usd,
              capUsd: event.cap_usd,
              percent: event.percent,
              status: 'warn',
            },
          };
        }

        case 'budget_downshift': {
          // BudgetDownshift carries `from_model` / `to_model` (not
          // spend/cap in the current schema); preserve last-known
          // spend/cap snapshot and flip status. The header bar shows
          // "downshift" badge regardless of cap data.
          const prev = state.budget;
          return {
            ...state,
            budget: {
              spentUsd: prev?.spentUsd ?? 0,
              capUsd: prev?.capUsd ?? 0,
              percent: prev?.percent ?? 75,
              status: 'downshift',
            },
          };
        }

        case 'budget_suspended':
          return {
            ...state,
            budget: {
              spentUsd: event.spent_usd,
              capUsd: event.cap_usd,
              percent: Math.round((event.spent_usd / Math.max(event.cap_usd, 1e-9)) * 100),
              status: 'suspended',
            },
          };

        case 'budget_exceeded':
          return {
            ...state,
            budget: {
              spentUsd: event.spent_usd,
              capUsd: event.cap_usd,
              percent: 100,
              status: 'exceeded',
            },
          };

        // Spec §4b M05 Stage A — light up the four `*_missing` variants
        // and `gap_resolved`. Idempotent on `${kind}:${missingName}:${agentId}`.
        // capability_violation + capability_grant remain no-op (Stage B
        // lights up after the L1+L2a enforcer lands per phase doc).
        case 'tool_missing':
          return mountOrUpdateGap(state, {
            gapId: gapNodeId('tool_missing', event.tool_name, event.agent_id),
            kind: 'tool_missing',
            missingName: event.tool_name,
            agentId: event.agent_id,
            severity: event.severity,
            suggestedAction: event.suggested_action,
            requestedVia: event.requested_via,
            status: 'gap',
          });

        case 'skill_missing':
          return mountOrUpdateGap(state, {
            gapId: gapNodeId('skill_missing', event.skill_name, event.agent_id),
            kind: 'skill_missing',
            missingName: event.skill_name,
            agentId: event.agent_id,
            severity: event.severity,
            suggestedAction: event.suggested_action,
            requestedVia: event.requested_via,
            status: 'gap',
          });

        case 'mcp_missing':
          return mountOrUpdateGap(state, {
            gapId: gapNodeId('mcp_missing', event.server_name, event.agent_id),
            kind: 'mcp_missing',
            missingName: event.server_name,
            agentId: event.agent_id,
            severity: event.severity,
            suggestedAction: event.suggested_action,
            requestedVia: event.requested_via,
            status: 'gap',
          });

        case 'agent_missing':
          return mountOrUpdateGap(state, {
            gapId: gapNodeId('agent_missing', event.missing_agent_id, event.agent_id),
            kind: 'agent_missing',
            missingName: event.missing_agent_id,
            agentId: event.agent_id,
            severity: event.severity,
            suggestedAction: event.suggested_action,
            requestedVia: event.requested_via,
            status: 'gap',
          });

        case 'gap_resolved': {
          // Match by (kind, capability, agent_id). The `kind` field on
          // gap_resolved is "tool" / "skill" / "mcp" / "agent" (singular);
          // map to the `*_missing` discriminator we stored at mount.
          const kindMap: Record<string, GapNodeData['kind']> = {
            tool: 'tool_missing',
            skill: 'skill_missing',
            mcp: 'mcp_missing',
            agent: 'agent_missing',
          };
          const targetKind = kindMap[event.kind];
          if (!targetKind) {
            return state;
          }
          const targetId = gapNodeId(targetKind, event.capability, event.agent_id);
          return {
            ...state,
            nodes: state.nodes.filter((n) => n.id !== targetId),
          };
        }

        case 'capability_violation': {
          // Spec §8.security L2a (M05 Stage B): record the rejection
          // keyed by agent. Last-write-wins on repeat violations from
          // the same agent — the inspector + capability-violation modal
          // read this to surface the current blocking state.
          return {
            ...state,
            capabilityViolations: {
              ...state.capabilityViolations,
              [event.agent_id]: {
                capabilityKind: event.capability_kind,
                requestedAction: event.requested_action,
                declaredScope: event.declared_scope,
                timestamp: Date.now(),
              },
            },
          };
        }

        case 'capability_grant': {
          // Spec §8.security L2a (M05 Stage B): append the grant to the
          // audit log. parent_agent_id present → narrowed sub-agent grant;
          // absent → root grant from the framework loader.
          return {
            ...state,
            capabilityGrants: [
              ...state.capabilityGrants,
              {
                parentAgentId: event.parent_agent_id ?? null,
                grantedTo: event.granted_to,
                capabilityKind: event.capability_kind,
                resource: event.resource,
                narrowedFrom: event.narrowed_from ?? null,
                timestamp: Date.now(),
              },
            ],
          };
        }

        case 'tier_violation': {
          // Spec §8.security L4 (M05 Stage D): the L4 tier gate rejected
          // this dispatch before L1 ran. Record keyed by agent so the
          // tier-violation modal can surface the current blocking state.
          // Last-write-wins on repeat violations.
          return {
            ...state,
            tierViolations: {
              ...state.tierViolations,
              [event.agent_id]: {
                tier: event.tier,
                capabilityKind: event.capability_kind,
                attemptedAction: event.attempted_action,
                timestamp: Date.now(),
              },
            },
          };
        }

        case 'tier_transition': {
          // Spec §8.security L4 (M05 Stage D): the user's tier changed.
          // Settings panel reads `currentTier` to render the toggle's
          // current state; toast surfaces the transition.
          return { ...state, currentTier: event.current };
        }

        // No-op variants — stream_text + decision_record + token_usage
        // feed the inspector, not the graph; session_end + tool_error +
        // mode_changed are graph-no-op by design. The exhaustive default
        // below is the forcing function: any new variant added to the
        // schema breaks the compile until handled.
        //
        case 'mcp_installed': {
          // Spec §5 (M06.D): server installed → track live status keyed
          // by name. currentMcpServers is registry-backed install state
          // (preserved across clear(), like currentTier). Idempotent:
          // a repeat install overwrites the same key with an equivalent
          // record.
          return {
            ...state,
            currentMcpServers: {
              ...state.currentMcpServers,
              [event.name]: {
                name: event.name,
                transportKind: event.transport_kind,
                hasAuth: event.has_auth,
                status: 'connected',
              },
            },
          };
        }

        case 'mcp_uninstalled': {
          // Spec §5 (M06.D): server removed → drop it from the map so
          // the MCPNode + Settings list reflect the post-state.
          if (!(event.name in state.currentMcpServers)) {
            return state;
          }
          const next = { ...state.currentMcpServers };
          delete next[event.name];
          return { ...state, currentMcpServers: next };
        }

        case 'mcp_auth_granted': {
          // Spec §5 + §13.5 (M06.D): a per-server secret was stored.
          // Flip the credential indicator. No-op if the server isn't
          // tracked (auth without a prior install is not expected;
          // mirror the M05.D defensive shape rather than synthesize a
          // partial record).
          const existing = state.currentMcpServers[event.name];
          if (!existing) {
            return state;
          }
          return {
            ...state,
            currentMcpServers: {
              ...state.currentMcpServers,
              [event.name]: { ...existing, hasAuth: true },
            },
          };
        }

        case 'mcp_request_blocked': {
          // Spec §5a + §8.security (M06.D): an MCP tool dispatch was
          // denied. Record into capabilityViolations keyed by agent
          // (same shape + last-write-wins as capability_violation) so
          // the inspector + capability-violation modal surface it; the
          // MCP server + tool ride in requestedAction so the renderer
          // can attribute the block to the MCPNode.
          return {
            ...state,
            capabilityViolations: {
              ...state.capabilityViolations,
              [event.agent_id]: {
                capabilityKind: 'exec',
                requestedAction: `invoke MCP tool '${event.server}__${event.tool}'`,
                declaredScope: event.reason,
                timestamp: Date.now(),
              },
            },
          };
        }

        case 'tool_alias_ambiguous': {
          // Spec §5a step 5 (M06.D): a short name became ambiguous on
          // re-resolution. Append a per-session warning the renderer
          // surfaces as a toast; the framework silences it via
          // mcp_aliases.
          return {
            ...state,
            toolAliasWarnings: [
              ...state.toolAliasWarnings,
              {
                name: event.name,
                candidates: event.candidates,
                timestamp: Date.now(),
              },
            ],
          };
        }

        case 'artifact_hash_mismatch': {
          // Spec §2214 (M07 Stage B → Stage E): an installed artifact's
          // recomputed SRI hash drifted from the locked hash. Block the
          // artifact's use and surface the Reinstall / Remove prompt;
          // the artifact does NOT run with drifted bytes (integrity >
          // availability — ADR-0014). The panel reads `phase==='blocked'`
          // + the drift detail; clear() preserves the slot so a blocked
          // artifact stays blocked until the user reinstalls or removes.
          return {
            ...state,
            imports: {
              ...state.imports,
              [event.artifact_ref]: {
                ref: event.artifact_ref,
                phase: 'blocked',
                capabilities: [],
                requiresSecrets: [],
                l3Report: null,
                shareProvenance: null,
                error: `hash mismatch — expected ${event.expected}, got ${event.actual}`,
                expected: event.expected,
                actual: event.actual,
              },
            },
          };
        }

        // No-op variants — stream_text + decision_record + token_usage
        // feed the inspector, not the graph; session_end + tool_error +
        // mode_changed are graph-no-op by design.
        case 'session_end':
        case 'tool_error':
        case 'mode_changed':
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

  clear: (): void =>
    set((state) => ({
      nodes: [],
      edges: [],
      selectedNodeId: null,
      triggeredRails: [],
      pendingHitl: {},
      notifierRecords: {},
      budget: null,
      uncertainInvocations: [],
      capabilityViolations: {},
      capabilityGrants: [],
      // tierViolations are per-session — clear them. currentTier is a
      // per-installation user preference loaded from tier.json at app
      // start + mutated by tier_transition events; preserved across
      // session clears so the Settings toggle keeps its state.
      tierViolations: {},
      currentTier: state.currentTier,
      // currentMcpServers is registry-backed install state (like
      // currentTier), not per-session graph state — preserved across
      // session clears. toolAliasWarnings are per-session — cleared.
      currentMcpServers: state.currentMcpServers,
      toolAliasWarnings: [],
      // activeMcpCalls is per-session animation state — reset on clear()
      // (unlike currentMcpServers): an in-flight glow must not leak into
      // a new session.
      activeMcpCalls: {},
      // imports is install/integrity state (M07.E / ADR-0015) —
      // preserved across session clears so a blocked-by-hash-mismatch
      // artifact stays blocked until the user reinstalls or removes.
      imports: state.imports,
    })),

  selectNode: (id) => set({ selectedNodeId: id }),

  recordUncertainInvocation: (invocation) =>
    set((state) => {
      if (state.uncertainInvocations.some((u) => u.invocationId === invocation.invocationId)) {
        return state;
      }
      return {
        ...state,
        uncertainInvocations: [...state.uncertainInvocations, invocation],
      };
    }),

  resolveUncertainInvocation: (invocationId) =>
    set((state) => ({
      ...state,
      uncertainInvocations: state.uncertainInvocations.filter(
        (u) => u.invocationId !== invocationId,
      ),
    })),

  recordImport: (outcome) =>
    set((state) => ({
      ...state,
      imports: {
        ...state.imports,
        [outcome.lock_key]: {
          ref: outcome.lock_key,
          // Promoted-within-bounds skips the disclosure modal (L4
          // auto-accept); Novice always gets the review modal.
          phase: outcome.review_required ? 'review' : 'installed',
          capabilities: outcome.capabilities,
          requiresSecrets: outcome.requires_secrets,
          l3Report: {
            reportId: outcome.l3_report.report_id,
            passed: outcome.l3_report.passed,
            reasons: outcome.l3_report.reasons,
          },
          shareProvenance: outcome.share_provenance,
        },
      },
    })),

  confirmImport: (ref) =>
    set((state) => {
      const existing = state.imports[ref];
      if (!existing) {
        return state;
      }
      return {
        ...state,
        imports: { ...state.imports, [ref]: { ...existing, phase: 'installed' } },
      };
    }),

  dismissImport: (ref) =>
    set((state) => {
      if (!(ref in state.imports)) {
        return state;
      }
      const next = { ...state.imports };
      delete next[ref];
      return { ...state, imports: next };
    }),
}));
