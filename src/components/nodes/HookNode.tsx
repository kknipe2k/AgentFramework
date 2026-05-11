import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { HookNodeData, HookReactFlowNode } from '../../lib/graphStore';

export function HookNode({ data }: NodeProps<HookReactFlowNode>): JSX.Element {
  const { hookId, hookName, category, firingPoint, status, durationMs, error }: HookNodeData = data;
  return (
    <div
      className={`hook-node hook-node--${status} hook-node--${category}`}
      data-testid={`hook-node-${hookId}`}
      data-status={status}
      data-category={category}
      data-firing-point={firingPoint}
      aria-label={`hook ${hookName} (${category})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="hook-node__name">{hookName}</div>
      <div className="hook-node__category">{category}</div>
      <div className="hook-node__firing-point">{firingPoint}</div>
      {durationMs !== null && <div className="hook-node__duration">{durationMs} ms</div>}
      {status === 'error' && error !== null && (
        <div className="hook-node__error" title={error}>
          {truncateForBadge(error)}
        </div>
      )}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}

const BADGE_MAX = 60;

function truncateForBadge(s: string): string {
  return s.length <= BADGE_MAX ? s : `${s.slice(0, BADGE_MAX)}…`;
}
