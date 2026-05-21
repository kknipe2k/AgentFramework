import { Handle, Position, type NodeProps } from '@xyflow/react';
import { NodeValidationBadge, useNodeErrors } from './NodeValidationBadge';

/** The data the canvas projection feeds a Builder Hook node. */
interface BuilderHookNodeData extends Record<string, unknown> {
  point: string;
  /** The key a validate_framework `NodeError` attributes to this node. */
  nodePath: string;
}

/**
 * The interactive Builder Hook node (M08.D1/D2) — one §4a hook firing
 * point. Reuses the §3 `hook-node` visual CSS; D2 adds the red
 * validation badge keyed by `nodePath`.
 */
export function BuilderHookNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderHookNodeData;
  const errors = useNodeErrors(d.nodePath);
  return (
    <div
      className={`hook-node builder-hook-node${errors.length > 0 ? ' builder-node--invalid' : ''}`}
      data-testid={`builder-hook-node-${d.point}`}
    >
      <Handle type="target" position={Position.Top} />
      <NodeValidationBadge errors={errors} />
      <div className="hook-node__name">{d.point}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
