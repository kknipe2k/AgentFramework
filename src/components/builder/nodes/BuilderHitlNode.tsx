import { Handle, Position, type NodeProps } from '@xyflow/react';

/** The data the canvas projection feeds a Builder HITL node. */
interface BuilderHitlNodeData extends Record<string, unknown> {
  trigger: string;
}

/**
 * The interactive Builder HITL node (M08.D1) — one §6a HITL trigger.
 * Reuses the §3 `hitl-node` visual CSS.
 */
export function BuilderHitlNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderHitlNodeData;
  return (
    <div className="hitl-node builder-hitl-node" data-testid={`builder-hitl-node-${d.trigger}`}>
      <Handle type="target" position={Position.Top} />
      <div className="hitl-node__prompt">{d.trigger}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
