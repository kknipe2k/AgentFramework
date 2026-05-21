import { Handle, Position, type NodeProps } from '@xyflow/react';

/** The data the canvas projection feeds a Builder Hook node. */
interface BuilderHookNodeData extends Record<string, unknown> {
  point: string;
}

/**
 * The interactive Builder Hook node (M08.D1) — one §4a hook firing
 * point. Reuses the §3 `hook-node` visual CSS.
 */
export function BuilderHookNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderHookNodeData;
  return (
    <div className="hook-node builder-hook-node" data-testid={`builder-hook-node-${d.point}`}>
      <Handle type="target" position={Position.Top} />
      <div className="hook-node__name">{d.point}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
