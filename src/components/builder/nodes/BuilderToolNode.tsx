import { Handle, Position, type NodeProps } from '@xyflow/react';

/** The data the canvas projection feeds a Builder Tool node. */
interface BuilderToolNodeData extends Record<string, unknown> {
  name: string;
}

/**
 * The interactive Builder Tool node (M08.D1). Reuses the §3 `tool-node`
 * visual CSS. A D2 edge wires it into an agent's `allowed_tools`.
 */
export function BuilderToolNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderToolNodeData;
  return (
    <div className="tool-node builder-tool-node" data-testid={`builder-tool-node-${d.name}`}>
      <Handle type="target" position={Position.Top} />
      <div className="tool-node__name">{d.name}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
