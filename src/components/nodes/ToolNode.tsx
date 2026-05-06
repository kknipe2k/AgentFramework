import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { ToolNodeData, ToolReactFlowNode } from '../../lib/graphStore';

export function ToolNode({ data }: NodeProps<ToolReactFlowNode>): JSX.Element {
  const { agentId, toolName, status, durationMs }: ToolNodeData = data;
  return (
    <div
      className={`tool-node tool-node--${status}`}
      data-testid={`tool-node-${agentId}-${toolName}`}
      data-status={status}
      aria-label={`tool ${toolName} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="tool-node__name">{toolName}</div>
      {durationMs !== null && <div className="tool-node__duration">{durationMs} ms</div>}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
