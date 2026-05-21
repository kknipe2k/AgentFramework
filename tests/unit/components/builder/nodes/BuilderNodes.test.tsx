import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider, type NodeProps } from '@xyflow/react';
import { BuilderToolNode } from '../../../../../src/components/builder/nodes/BuilderToolNode';
import { BuilderSkillNode } from '../../../../../src/components/builder/nodes/BuilderSkillNode';
import { BuilderHitlNode } from '../../../../../src/components/builder/nodes/BuilderHitlNode';
import { BuilderHookNode } from '../../../../../src/components/builder/nodes/BuilderHookNode';

// M08.D1 follow-up — net-new additive coverage for the four simple
// Builder node components (Tool / Skill / HITL / Hook). The phase-doc
// D1.4.4 test plan named only BuilderAgentNode.test.tsx; these four
// are exercised end-to-end by the Playwright spec but were never
// mounted in a vitest test (the BuilderCanvas mock renders placeholder
// nodes), leaving them at ~20% line coverage. Rendered directly inside
// a ReactFlowProvider (the MCPNode / BuilderAgentNode precedent).

type BuilderNodeComponent = (props: NodeProps) => JSX.Element;

function renderBuilderNode(
  Component: BuilderNodeComponent,
  type: string,
  data: Record<string, unknown>,
): void {
  render(
    <ReactFlowProvider>
      <Component
        id={`${type}:x`}
        type={type}
        data={data}
        dragging={false}
        zIndex={0}
        selectable
        deletable
        selected={false}
        draggable
        isConnectable
        positionAbsoluteX={0}
        positionAbsoluteY={0}
      />
    </ReactFlowProvider>,
  );
}

describe('Builder Tool / Skill / HITL / Hook nodes', () => {
  it('BuilderToolNode_renders_the_tool_name', () => {
    renderBuilderNode(BuilderToolNode, 'tool', { name: 'Read' });
    expect(screen.getByTestId('builder-tool-node-Read')).toBeInTheDocument();
    expect(screen.getByText('Read')).toBeInTheDocument();
  });

  it('BuilderSkillNode_renders_the_skill_name', () => {
    renderBuilderNode(BuilderSkillNode, 'skill', { name: 'planning' });
    expect(screen.getByTestId('builder-skill-node-planning')).toBeInTheDocument();
    expect(screen.getByText('planning')).toBeInTheDocument();
  });

  it('BuilderHitlNode_renders_the_trigger', () => {
    renderBuilderNode(BuilderHitlNode, 'hitl', { trigger: 'on_gap' });
    expect(screen.getByTestId('builder-hitl-node-on_gap')).toBeInTheDocument();
    expect(screen.getByText('on_gap')).toBeInTheDocument();
  });

  it('BuilderHookNode_renders_the_firing_point', () => {
    renderBuilderNode(BuilderHookNode, 'hook', { point: 'pre_task' });
    expect(screen.getByTestId('builder-hook-node-pre_task')).toBeInTheDocument();
    expect(screen.getByText('pre_task')).toBeInTheDocument();
  });

  it('a_builder_node_renders_a_source_and_target_react_flow_handle', () => {
    renderBuilderNode(BuilderToolNode, 'tool', { name: 'Read' });
    const root = screen.getByTestId('builder-tool-node-Read');
    expect(root.querySelectorAll('.react-flow__handle')).toHaveLength(2);
  });
});
