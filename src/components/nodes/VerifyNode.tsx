import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { VerifyNodeData, VerifyReactFlowNode } from '../../lib/graphStore';

export function VerifyNode({ data }: NodeProps<VerifyReactFlowNode>): JSX.Element {
  const {
    hookId,
    level,
    firingPoint,
    status,
    durationMs,
    outputPreview,
    error,
    onFailure,
  }: VerifyNodeData = data;
  const ariaLabel = `verify ${hookId} (${status})`;
  return (
    <div
      className={`verify-node verify-node--${status}`}
      data-testid={`verify-node-${hookId}`}
      data-status={status}
      data-firing-point={firingPoint}
      aria-label={ariaLabel}
    >
      <Handle type="target" position={Position.Top} />
      <div className="verify-node__hook">{hookId}</div>
      {level !== null && <div className="verify-node__level">{level}</div>}
      <div className="verify-node__firing-point">{firingPoint}</div>
      {durationMs !== null && <div className="verify-node__duration">{durationMs} ms</div>}
      {status === 'pass' && outputPreview !== null && (
        <div className="verify-node__output" title={outputPreview}>
          {truncateForBadge(outputPreview)}
        </div>
      )}
      {status === 'fail' && error !== null && (
        <div className="verify-node__error" title={error}>
          {truncateForBadge(error)}
        </div>
      )}
      {status === 'fail' && onFailure !== null && (
        <div className={`verify-node__on-failure verify-node__on-failure--${onFailure}`}>
          {onFailure}
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
