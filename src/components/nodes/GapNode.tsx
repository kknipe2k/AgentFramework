import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { GapNodeData, GapReactFlowNode } from '../../lib/graphStore';

export function GapNode({ data }: NodeProps<GapReactFlowNode>): JSX.Element {
  const {
    gapId,
    kind,
    missingName,
    agentId,
    severity,
    suggestedAction,
    requestedVia,
  }: GapNodeData = data;
  // Visual tier per spec §4b severity matrix:
  //   critical  → red pulse  (tool / agent gaps from loader)
  //   important → orange     (request_capability tool gaps)
  //   advisory  → amber      (skill gaps — recoverable)
  //   requested → blue       (meta-tool marker)
  // CSS hook: `gap-node--${severity}` drives the keyframe animation in
  // src/styles.css.
  return (
    <div
      className={`gap-node gap-node--${severity}`}
      data-testid={gapId}
      data-kind={kind}
      data-severity={severity}
      data-requested-via={requestedVia}
      aria-label={`gap ${kind} ${missingName} for agent ${agentId} — ${severity}`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="gap-node__kind">{kind}</div>
      <div className="gap-node__name">{missingName}</div>
      <div className="gap-node__action" title={suggestedAction}>
        {suggestedAction}
      </div>
    </div>
  );
}
