import { Handle, Position, type NodeProps } from '@xyflow/react';
import { NodeValidationBadge, useNodeErrors } from './NodeValidationBadge';

/** The data the canvas projection feeds a Builder Tool node. */
interface BuilderToolNodeData extends Record<string, unknown> {
  name: string;
  /** The key a validate_framework `NodeError` attributes to this node. */
  nodePath: string;
}

/**
 * The interactive Builder Tool node (M08.D1/D2). Reuses the §3
 * `tool-node` visual CSS. A D2 edge wires it into an agent's
 * `allowed_tools`; D2 also adds the red validation badge keyed by
 * `nodePath`.
 */
export function BuilderToolNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderToolNodeData;
  const errors = useNodeErrors(d.nodePath);
  return (
    <div
      className={`tool-node builder-tool-node${errors.length > 0 ? ' builder-node--invalid' : ''}`}
      data-testid={`builder-tool-node-${d.name}`}
    >
      <Handle type="target" position={Position.Top} />
      <NodeValidationBadge errors={errors} />
      <div className="tool-node__name">{d.name}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
