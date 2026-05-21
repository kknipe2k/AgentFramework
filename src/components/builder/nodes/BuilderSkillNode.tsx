import { Handle, Position, type NodeProps } from '@xyflow/react';
import { NodeValidationBadge, useNodeErrors } from './NodeValidationBadge';

/** The data the canvas projection feeds a Builder Skill node. */
interface BuilderSkillNodeData extends Record<string, unknown> {
  name: string;
  /** The key a validate_framework `NodeError` attributes to this node. */
  nodePath: string;
}

/**
 * The interactive Builder Skill node (M08.D1/D2). Reuses the §3
 * `skill-node` visual CSS. A D2 edge wires it into an agent's
 * `allowed_skills`; D2 also adds the red validation badge keyed by
 * `nodePath`.
 */
export function BuilderSkillNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderSkillNodeData;
  const errors = useNodeErrors(d.nodePath);
  return (
    <div
      className={`skill-node builder-skill-node${errors.length > 0 ? ' builder-node--invalid' : ''}`}
      data-testid={`builder-skill-node-${d.name}`}
    >
      <Handle type="target" position={Position.Top} />
      <NodeValidationBadge errors={errors} />
      <div className="skill-node__name">{d.name}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
