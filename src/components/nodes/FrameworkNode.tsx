import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { FrameworkNodeData, FrameworkReactFlowNode } from '../../lib/graphStore';

export function FrameworkNode({ data }: NodeProps<FrameworkReactFlowNode>): JSX.Element {
  const { frameworkName, model, status }: FrameworkNodeData = data;
  return (
    <div
      className={`framework-node framework-node--${status}`}
      data-testid={`framework-node-${frameworkName}`}
      data-status={status}
      aria-label={`framework ${frameworkName} on ${model}`}
    >
      <div className="framework-node__name">{frameworkName}</div>
      <div className="framework-node__model">{model}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
