import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { PlanNodeData, PlanReactFlowNode } from '../../lib/graphStore';

const TITLE_MAX = 40;

function truncate(s: string, max: number): string {
  return s.length <= max ? s : `${s.slice(0, max - 1)}…`;
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

export function PlanNode({ data }: NodeProps<PlanReactFlowNode>): JSX.Element {
  const {
    planId,
    title,
    status,
    taskCount,
    completedCount,
    lastTransitionReason,
    durationMs,
  }: PlanNodeData = data;
  const showReason =
    (status === 'awaiting_replan' || status === 'aborted') &&
    lastTransitionReason !== null &&
    lastTransitionReason.length > 0;
  return (
    <div
      className={`plan-node plan-node--${status}`}
      data-testid={`plan-node-${planId}`}
      data-status={status}
      aria-label={`plan ${title} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="plan-node__title">{truncate(title, TITLE_MAX)}</div>
      <div className="plan-node__status">{status.replace(/_/g, ' ')}</div>
      <div className="plan-node__progress">
        {completedCount} / {taskCount}
      </div>
      {showReason && <div className="plan-node__reason">{lastTransitionReason}</div>}
      {status === 'complete' && durationMs !== null && (
        <div className="plan-node__duration">{formatDuration(durationMs)}</div>
      )}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
