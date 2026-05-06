import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { SkillNode } from '../../../src/components/nodes/SkillNode';
import type { SkillNodeData } from '../../../src/lib/graphStore';

function renderSkill(data: SkillNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <SkillNode
        id={`skill:${data.agentId}:${data.skillName}`}
        type="skill"
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

const baseData: SkillNodeData = {
  skillName: 'planner',
  agentId: 'a1',
  mode: null,
};

describe('SkillNode', () => {
  it('renders_skill_name', () => {
    renderSkill(baseData);
    expect(screen.getByText('planner')).toBeInTheDocument();
  });

  it('applies_skill_node_class_with_dashed_modifier', () => {
    renderSkill(baseData);
    const root = screen.getByTestId('skill-node-a1-planner');
    expect(root.className).toContain('skill-node');
    // Spec §3 Behavior: dashed outline; no flow animation. The CSS hook
    // is `skill-node--dashed`.
    expect(root.className).toContain('skill-node--dashed');
  });

  it('renders_mode_when_present', () => {
    renderSkill({ ...baseData, mode: 'LITE' });
    expect(screen.getByText(/LITE/i)).toBeInTheDocument();
  });

  it('exposes_accessible_aria_label_with_name', () => {
    renderSkill(baseData);
    const root = screen.getByTestId('skill-node-a1-planner');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/skill.*planner/i));
  });

  it('renders_source_and_target_handle_elements', () => {
    renderSkill(baseData);
    const root = screen.getByTestId('skill-node-a1-planner');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
