import { Handle, Position, type NodeProps } from '@xyflow/react';
import { useShallow } from 'zustand/react/shallow';
import { useGraphStore, type MCPNodeData, type MCPReactFlowNode } from '../../lib/graphStore';

/**
 * MCPNode — spec §3 + §5.
 *
 * M03 shipped the static stub (NodeStatus from `data`). M06.E adds the
 * live connection-status indicator sourced from the store's
 * `currentMcpServers[serverName]` record (McpServerStatus: connected |
 * disconnected | health_pending | error) plus an active-call animation
 * driven by `activeMcpCalls`.
 *
 * The live connection status rides on a SEPARATE `mcp-node--conn-*`
 * class family so it never collides with the NodeStatus
 * `mcp-node--<active|complete|error>` classes (those reflect the
 * graph-node lifecycle, a different axis from transport connectivity).
 *
 * Per gotcha #75 + the v1.6 zustand_selector_audit: the
 * `currentMcpServers[serverName]` read is `useShallow`-wrapped so an
 * equivalent record re-creation (every `mcp_installed`) doesn't churn a
 * re-render.
 */
export function MCPNode({ data }: NodeProps<MCPReactFlowNode>): JSX.Element {
  const { serverId, serverName, status, discoveredToolCount }: MCPNodeData = data;
  const record = useGraphStore(useShallow((s) => s.currentMcpServers[serverName]));
  const activeCallId = useGraphStore((s) => s.activeMcpCalls[serverName]);
  const connStatus = record?.status ?? 'disconnected';
  const callActive = activeCallId !== undefined;
  return (
    <div
      className={`mcp-node mcp-node--${status} mcp-node--conn-${connStatus}${
        callActive ? ' mcp-node--call-active' : ''
      }`}
      data-testid={`mcp-node-${serverId}`}
      data-status={status}
      aria-label={`mcp ${serverName} (${status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="mcp-node__name">{serverName}</div>
      <div
        className="mcp-node__status-indicator"
        data-conn-status={connStatus}
        aria-label={`connection: ${connStatus}`}
      />
      {discoveredToolCount !== null && (
        <div className="mcp-node__tool-count">{discoveredToolCount} tools</div>
      )}
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
