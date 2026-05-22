import { Handle, Position, type NodeProps } from '@xyflow/react';
import { useShallow } from 'zustand/react/shallow';
import { useBuilderStore } from '../../../lib/builderStore';
import { CapabilityDisclosure } from '../../CapabilityDisclosure';
import { NarrowingNotice } from '../NarrowingNotice';
import { NodeValidationBadge, useNodeErrors } from './NodeValidationBadge';

/** The data the canvas projection feeds a Builder Agent node. */
interface BuilderAgentNodeData extends Record<string, unknown> {
  agentId: string;
  /** The key a validate_framework `NodeError` attributes to this node. */
  nodePath: string;
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
 * The interactive Builder Agent node (M08.D1/D2). Reuses the §3
 * `agent-node` visual CSS; the per-node plain-English capability
 * disclosure reuses the shared {@link CapabilityDisclosure} surface.
 * D2 adds the red validation badge (keyed by `nodePath`) and — for each
 * Agent→Agent spawn edge where this agent is the CHILD — a
 * {@link NarrowingNotice} surfacing the §8.security L2a narrowing
 * decision. Distinct from the read-only live-graph `AgentNode`.
 */
export function BuilderAgentNode({ data }: NodeProps): JSX.Element {
  const d = data as BuilderAgentNodeData;
  const errors = useNodeErrors(d.nodePath);
  // The spawn-edge ids where this agent is the CHILD — the narrowing
  // notice surfaces on the spawned (child) agent (D2.3.6). useShallow
  // so the node re-renders only when its own spawn edges change.
  const incomingSpawnEdgeIds = useBuilderStore(
    useShallow((s) =>
      (s.validation?.capability_summary?.spawn_edges ?? [])
        .filter((edge) => edge.child_id === d.agentId)
        .map((edge) => `agent:${edge.parent_id}->${edge.child_id}`),
    ),
  );
  return (
    <div
      className={`agent-node builder-agent-node${errors.length > 0 ? ' builder-node--invalid' : ''}`}
      data-testid={`builder-agent-node-${d.agentId}`}
    >
      <Handle type="target" position={Position.Top} />
      <NodeValidationBadge errors={errors} />
      <div className="agent-node__name">{d.agentId}</div>
      <div className="agent-node__id">{d.role.length > 0 ? d.role : 'no role set'}</div>
      <CapabilityDisclosure
        capabilities={capabilityLines(d.allowedTools, d.allowedSkills)}
        emptyMessage="No tools or skills assigned yet."
        data-testid={`builder-node-disclosure-${d.agentId}`}
      />
      {incomingSpawnEdgeIds.map((id) => (
        <NarrowingNotice key={id} spawnEdgeId={id} />
      ))}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
