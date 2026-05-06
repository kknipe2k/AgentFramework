import { Background, Controls, ReactFlow, type NodeTypes } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
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
const nodeTypes: NodeTypes = {
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

  return (
    <div className="graph-canvas" data-testid="graph-canvas">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        fitView
        onNodeClick={(_, node) => selectNode(node.id)}
        onPaneClick={() => selectNode(null)}
      >
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}
