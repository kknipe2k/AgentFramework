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
  // Fallback when the projection has no title yet: the `task_started`
  // event schema has no `title` field, so graphStore creates the TaskNode
  // with title=''. M07 plan-loop driver may populate titles from plan
  // metadata; until then, render `task <id-prefix>` so the node has a
  // readable label rather than appearing blank. M04 IRL: TaskNodes
  // surfaced as "untitled" in the inspector + canvas.
  const displayTitle = title || `task ${taskId.slice(0, 8)}`;
  return (
    <div
      className={`task-node task-node--${status}`}
      data-testid={`task-node-${taskId}`}
      data-status={status}
      aria-label={`task ${displayTitle} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="task-node__title">{truncate(displayTitle, TITLE_MAX)}</div>
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
