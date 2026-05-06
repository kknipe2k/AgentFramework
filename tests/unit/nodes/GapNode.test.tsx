import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { GapNode } from '../../../src/components/nodes/GapNode';
import type { GapNodeData } from '../../../src/lib/graphStore';

function renderGap(data: GapNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <GapNode
        id={`gap:${data.gapId}`}
        type="gap"
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

const baseData: GapNodeData = {
  gapId: 'tool-missing-fetch_prs',
  kind: 'tool_missing',
  missingName: 'fetch_prs',
  status: 'gap',
};

describe('GapNode', () => {
  it('renders_missing_artifact_name', () => {
    renderGap(baseData);
    expect(screen.getByText('fetch_prs')).toBeInTheDocument();
  });

  it('exposes_data_kind_attribute_distinguishing_tool_vs_skill_missing', () => {
    const { unmount } = renderGap(baseData);
    const root = screen.getByTestId('gap-node-tool-missing-fetch_prs');
    expect(root).toHaveAttribute('data-kind', 'tool_missing');
    unmount();

    renderGap({
      ...baseData,
      gapId: 'skill-missing-planner',
      kind: 'skill_missing',
      missingName: 'planner',
    });
    const skillRoot = screen.getByTestId('gap-node-skill-missing-planner');
    expect(skillRoot).toHaveAttribute('data-kind', 'skill_missing');
  });

  it('applies_gap_amber_modifier_class_for_pulse_keyframe', () => {
    renderGap(baseData);
    const root = screen.getByTestId('gap-node-tool-missing-fetch_prs');
    // Spec §3 Visual Design: gap = amber. The CSS hook is gap-node--gap
    // which drives the @keyframes gap-pulse animation.
    expect(root.className).toContain('gap-node--gap');
  });

  it('exposes_accessible_aria_label_naming_the_gap_kind', () => {
    renderGap(baseData);
    const root = screen.getByTestId('gap-node-tool-missing-fetch_prs');
    expect(root).toHaveAttribute(
      'aria-label',
      expect.stringMatching(/gap.*tool_missing.*fetch_prs/i),
    );
  });

  it('renders_target_handle_only_no_source', () => {
    // GapNode is a terminal — agents fail INTO it; the gap doesn't
    // call out further. Per spec §3 Behavior.
    renderGap(baseData);
    const root = screen.getByTestId('gap-node-tool-missing-fetch_prs');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(1);
  });
});
