import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { AgentNodeData, AgentReactFlowNode } from '../../lib/graphStore';
import { tokenScale } from '../../lib/tokenScale';
import { CapabilityBadge } from './CapabilityBadge';

export function AgentNode({ data }: NodeProps<AgentReactFlowNode>): JSX.Element {
  const { agentId, agentName, status, tokensTotal }: AgentNodeData = data;
  // Spec §3 Visual Design: "Token spend shown as node weight — larger
  // spend = visually larger node". CSS `transform: scale()` keeps the
  // renderer fast (GPU-accelerated, no layout cost). Tests can disable
  // by setting `data-token-scale-disabled` on a parent so visual
  // regressions don't trip the suite.
  //
  // Reads `tokensTotal` (from `agent_complete.tokens_total` — the
  // session-cumulative count), NOT `tokensIn + tokensOut` (which only
  // populate from `tool_result` events and never aggregate for the
  // overall agent). M04 IRL: prior version read the wrong fields, so
  // visual scaling was always 0.8 (scale clamp floor) regardless of
  // actual token spend.
  const scale = tokenScale(tokensTotal);
  return (
    <div
      className={`agent-node agent-node--${status}`}
      data-testid={`agent-node-${agentId}`}
      data-status={status}
      style={{ transform: `scale(${scale})`, transformOrigin: 'center' }}
      aria-label={`agent ${agentName} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="agent-node__name">{agentName}</div>
      <div className="agent-node__id">{agentId.slice(0, 8)}</div>
      <CapabilityBadge agentId={agentId} />
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
