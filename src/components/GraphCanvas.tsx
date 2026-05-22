import { Background, Controls, MiniMap, ReactFlow, type NodeTypes } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { useMemo } from 'react';
import { layoutGraph } from '../lib/layout';
import { useGraphStore } from '../lib/graphStore';
import { AgentNode } from './nodes/AgentNode';
import { FrameworkNode } from './nodes/FrameworkNode';
import { GapNode } from './nodes/GapNode';
import { HITLNode } from './nodes/HITLNode';
import { HookNode } from './nodes/HookNode';
import { MCPNode } from './nodes/MCPNode';
import { PlanNode } from './nodes/PlanNode';
import { SkillNode } from './nodes/SkillNode';
import { TaskNode } from './nodes/TaskNode';
import { ToolNode } from './nodes/ToolNode';
import { VerifyNode } from './nodes/VerifyNode';

// Defined OUTSIDE the component per @xyflow/react v12 docs: nodeTypes is
// a stable-reference map; redefining it on each render forces React Flow
// to re-mount every node, which kills the streaming UX. The trap
// re-applies with 11 entries — keep it module-level.
//
// Exported so the M08.F2 Tester graph pane reuses the SAME 11-entry map
// (a stable reference — importing it, never redefining it, keeps the
// @xyflow/react v12 stable-reference contract).
export const nodeTypes: NodeTypes = {
  agent: AgentNode as NodeTypes[string],
  tool: ToolNode as NodeTypes[string],
  skill: SkillNode as NodeTypes[string],
  mcp: MCPNode as NodeTypes[string],
  gap: GapNode as NodeTypes[string],
  hitl: HITLNode as NodeTypes[string],
  plan: PlanNode as NodeTypes[string],
  task: TaskNode as NodeTypes[string],
  verify: VerifyNode as NodeTypes[string],
  hook: HookNode as NodeTypes[string],
  framework: FrameworkNode as NodeTypes[string],
};

export function GraphCanvas(): JSX.Element {
  // Selector form so the component re-renders only when the selected
  // slice of state changes (per Zustand v5 docs).
  const nodes = useGraphStore((s) => s.nodes);
  const edges = useGraphStore((s) => s.edges);
  const selectNode = useGraphStore((s) => s.selectNode);

  // Layout is a visualization concern, not state — keep dagre out of
  // the store. Cache POSITIONS only (memoized by node + edge count, so
  // dagre re-runs only on graph topology changes). Then merge fresh node
  // data on every render so per-status / per-data updates flow through
  // to React Flow without re-running layout. M04.C surfaced the original
  // count-only useMemo bug: plan transitions (`pending_approval` →
  // `awaiting_approval` → `in_progress`) keep array length constant, so
  // a count-keyed memo returned a stale layoutedNodes array carrying
  // stale `data.status`. The Playwright plan_approval spec failed at
  // `expect(planNode).toHaveAttribute('data-status', 'in_progress')`.
  const positions = useMemo(
    () => {
      const laid = layoutGraph(nodes, edges);
      return new Map(laid.map((n) => [n.id, n.position]));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [nodes.length, edges.length],
  );
  const layoutedNodes = nodes.map((n) => ({
    ...n,
    position: positions.get(n.id) ?? n.position,
  }));

  return (
    <div className="graph-canvas" data-testid="graph-canvas">
      <ReactFlow
        nodes={layoutedNodes}
        edges={edges}
        nodeTypes={nodeTypes}
        fitView
        onNodeClick={(_, node) => selectNode(node.id)}
        onPaneClick={() => selectNode(null)}
      >
        <Background />
        <Controls />
        <MiniMap nodeStrokeWidth={3} pannable zoomable />
      </ReactFlow>
    </div>
  );
}
