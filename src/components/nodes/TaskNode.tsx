import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { TaskNodeData, TaskReactFlowNode } from '../../lib/graphStore';

const TITLE_MAX = 30;

function truncate(s: string, max: number): string {
  return s.length <= max ? s : `${s.slice(0, max - 1)}…`;
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function formatFailureCount(count: number, max: number | null): string {
  return max === null ? `⚠ ${count}` : `⚠ ${count}/${max}`;
}

export function TaskNode({ data }: NodeProps<TaskReactFlowNode>): JSX.Element {
  const { taskId, title, status, hitl, failureCount, maxFailures, durationMs }: TaskNodeData = data;
  return (
    <div
      className={`task-node task-node--${status}`}
      data-testid={`task-node-${taskId}`}
      data-status={status}
      aria-label={`task ${title} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="task-node__title">{truncate(title, TITLE_MAX)}</div>
      {hitl && <div className="task-node__hitl-badge">HITL</div>}
      {failureCount > 0 && (
        <div className="task-node__failure-badge">
          {formatFailureCount(failureCount, maxFailures)}
        </div>
      )}
      {status === 'done' && durationMs !== null && (
        <div className="task-node__duration">{formatDuration(durationMs)}</div>
      )}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
