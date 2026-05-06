import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { ToolNodeData, ToolReactFlowNode } from '../../lib/graphStore';
import { tokenScale } from '../../lib/tokenScale';

export function ToolNode({ data }: NodeProps<ToolReactFlowNode>): JSX.Element {
  const { agentId, toolName, status, durationMs, tokensIn, tokensOut }: ToolNodeData = data;
  const scale = tokenScale(tokensIn + tokensOut);
  return (
    <div
      className={`tool-node tool-node--${status}`}
      data-testid={`tool-node-${agentId}-${toolName}`}
      data-status={status}
      style={{ transform: `scale(${scale})`, transformOrigin: 'center' }}
      aria-label={`tool ${toolName} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="tool-node__name">{toolName}</div>
      {durationMs !== null && <div className="tool-node__duration">{durationMs} ms</div>}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
