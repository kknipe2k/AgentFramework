import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { HookNodeData, HookReactFlowNode } from '../../lib/graphStore';

export function HookNode({ data }: NodeProps<HookReactFlowNode>): JSX.Element {
  const { hookId, hookName, category, status }: HookNodeData = data;
  return (
    <div
      className={`hook-node hook-node--${status}`}
      data-testid={`hook-node-${hookId}`}
      data-status={status}
      aria-label={`hook ${hookName} (${category})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="hook-node__name">{hookName}</div>
      <div className="hook-node__category">{category}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
