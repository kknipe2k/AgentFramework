import { Handle, Position, type NodeProps } from '@xyflow/react';

/** The data the canvas projection feeds a Builder Skill node. */
interface BuilderSkillNodeData extends Record<string, unknown> {
  name: string;
}

/**
 * The interactive Builder Skill node (M08.D1). Reuses the §3
 * `skill-node` visual CSS. A D2 edge wires it into an agent's
 * `allowed_skills`.
 */
export function BuilderSkillNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderSkillNodeData;
  return (
    <div className="skill-node builder-skill-node" data-testid={`builder-skill-node-${d.name}`}>
      <Handle type="target" position={Position.Top} />
      <div className="skill-node__name">{d.name}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
