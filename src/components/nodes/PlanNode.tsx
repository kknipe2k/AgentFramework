import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { PlanNodeData, PlanReactFlowNode } from '../../lib/graphStore';

export function PlanNode({ data }: NodeProps<PlanReactFlowNode>): JSX.Element {
  const { planId, title, status, taskCount, completedCount }: PlanNodeData = data;
  return (
    <div
      className={`plan-node plan-node--${status}`}
      data-testid={`plan-node-${planId}`}
      data-status={status}
      aria-label={`plan ${title} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="plan-node__title">{title}</div>
      <div className="plan-node__progress">
        {completedCount} / {taskCount}
      </div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
