import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { MCPNodeData, MCPReactFlowNode } from '../../lib/graphStore';

export function MCPNode({ data }: NodeProps<MCPReactFlowNode>): JSX.Element {
  const { serverId, serverName, status, discoveredToolCount }: MCPNodeData = data;
  return (
    <div
      className={`mcp-node mcp-node--${status}`}
      data-testid={`mcp-node-${serverId}`}
      data-status={status}
      aria-label={`mcp ${serverName} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="mcp-node__name">{serverName}</div>
      {discoveredToolCount !== null && (
        <div className="mcp-node__tool-count">{discoveredToolCount} tools</div>
      )}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
