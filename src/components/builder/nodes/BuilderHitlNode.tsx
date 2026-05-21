import { Handle, Position, type NodeProps } from '@xyflow/react';
import { NodeValidationBadge, useNodeErrors } from './NodeValidationBadge';

/** The data the canvas projection feeds a Builder HITL node. */
interface BuilderHitlNodeData extends Record<string, unknown> {
  trigger: string;
  /** The key a validate_framework `NodeError` attributes to this node. */
  nodePath: string;
}

/**
 * The interactive Builder HITL node (M08.D1/D2) — one §6a HITL trigger.
 * Reuses the §3 `hitl-node` visual CSS; D2 adds the red validation
 * badge keyed by `nodePath`.
 */
export function BuilderHitlNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderHitlNodeData;
  const errors = useNodeErrors(d.nodePath);
  return (
    <div
      className={`hitl-node builder-hitl-node${errors.length > 0 ? ' builder-node--invalid' : ''}`}
      data-testid={`builder-hitl-node-${d.trigger}`}
    >
      <Handle type="target" position={Position.Top} />
      <NodeValidationBadge errors={errors} />
      <div className="hitl-node__prompt">{d.trigger}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
