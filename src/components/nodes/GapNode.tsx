import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { GapNodeData, GapReactFlowNode } from '../../lib/graphStore';

export function GapNode({ data }: NodeProps<GapReactFlowNode>): JSX.Element {
  const { gapId, kind, missingName, status }: GapNodeData = data;
  return (
    <div
      className={`gap-node gap-node--${status}`}
      data-testid={`gap-node-${gapId}`}
      data-kind={kind}
      aria-label={`gap ${kind} ${missingName}`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="gap-node__kind">{kind}</div>
      <div className="gap-node__name">{missingName}</div>
    </div>
  );
}
