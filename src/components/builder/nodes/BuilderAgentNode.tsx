import { Handle, Position, type NodeProps } from '@xyflow/react';
import { CapabilityDisclosure } from '../../CapabilityDisclosure';

/** The data the canvas projection feeds a Builder Agent node. */
interface BuilderAgentNodeData extends Record<string, unknown> {
  agentId: string;
  role: string;
  model: string;
  allowedTools: string[];
  allowedSkills: string[];
}

/** Plain-English capability lines for the per-node disclosure
 *  (§8.security L1) — derived live from the agent's declared
 *  `allowed_*`, so an inline-config edit updates the disclosure with no
 *  extra wiring. */
function capabilityLines(allowedTools: string[], allowedSkills: string[]): string[] {
  return [
    ...allowedTools.map((t) => `Can use the ${t} tool`),
    ...allowedSkills.map((s) => `Can load the ${s} skill`),
  ];
}

/**
 * The interactive Builder Agent node (M08.D1). Reuses the §3
 * `agent-node` visual CSS; the per-node plain-English capability
 * disclosure reuses the shared {@link CapabilityDisclosure} surface.
 * Distinct from the read-only live-graph `AgentNode` — the Builder node
 * is configured inline (Stage D1 `NodeConfigPanel`).
 */
export function BuilderAgentNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderAgentNodeData;
  return (
    <div className="agent-node builder-agent-node" data-testid={`builder-agent-node-${d.agentId}`}>
      <Handle type="target" position={Position.Top} />
      <div className="agent-node__name">{d.agentId}</div>
      <div className="agent-node__id">{d.role.length > 0 ? d.role : 'no role set'}</div>
      <CapabilityDisclosure
        capabilities={capabilityLines(d.allowedTools, d.allowedSkills)}
        emptyMessage="No tools or skills assigned yet."
        data-testid={`builder-node-disclosure-${d.agentId}`}
      />
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
