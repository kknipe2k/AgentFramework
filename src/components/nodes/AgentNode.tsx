import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { AgentNodeData, AgentReactFlowNode } from '../../lib/graphStore';

export function AgentNode({ data }: NodeProps<AgentReactFlowNode>): JSX.Element {
  const { agentId, agentName, status }: AgentNodeData = data;
  return (
    <div
      className={`agent-node agent-node--${status}`}
      data-testid={`agent-node-${agentId}`}
      data-status={status}
      aria-label={`agent ${agentName} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="agent-node__name">{agentName}</div>
      <div className="agent-node__id">{agentId.slice(0, 8)}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
