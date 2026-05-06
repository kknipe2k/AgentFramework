import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { TaskNodeData, TaskReactFlowNode } from '../../lib/graphStore';

export function TaskNode({ data }: NodeProps<TaskReactFlowNode>): JSX.Element {
  const { taskId, title, status, hitl }: TaskNodeData = data;
  return (
    <div
      className={`task-node task-node--${status}`}
      data-testid={`task-node-${taskId}`}
      data-status={status}
      aria-label={`task ${title} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="task-node__title">{title}</div>
      {hitl && <div className="task-node__hitl-badge">HITL</div>}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
