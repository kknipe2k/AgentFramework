import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { SkillNodeData, SkillReactFlowNode } from '../../lib/graphStore';

export function SkillNode({ data }: NodeProps<SkillReactFlowNode>): JSX.Element {
  const { agentId, skillName, mode }: SkillNodeData = data;
  return (
    <div
      className="skill-node skill-node--dashed"
      data-testid={`skill-node-${agentId}-${skillName}`}
      aria-label={`skill ${skillName}`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="skill-node__name">{skillName}</div>
      {mode !== null && <div className="skill-node__mode">{mode}</div>}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
