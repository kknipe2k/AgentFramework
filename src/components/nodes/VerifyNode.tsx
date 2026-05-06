import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { VerifyNodeData, VerifyReactFlowNode } from '../../lib/graphStore';

export function VerifyNode({ data }: NodeProps<VerifyReactFlowNode>): JSX.Element {
  const { hookId, level, status, durationMs }: VerifyNodeData = data;
  return (
    <div
      className={`verify-node verify-node--${status}`}
      data-testid={`verify-node-${hookId}`}
      data-status={status}
      aria-label={`verify ${hookId} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="verify-node__hook">{hookId}</div>
      <div className="verify-node__level">{level}</div>
      {durationMs !== null && <div className="verify-node__duration">{durationMs} ms</div>}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
