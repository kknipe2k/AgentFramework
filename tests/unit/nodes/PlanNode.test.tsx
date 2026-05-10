import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { PlanNode } from '../../../src/components/nodes/PlanNode';
import type { PlanNodeData } from '../../../src/lib/graphStore';

function renderPlan(data: PlanNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <PlanNode
        id={`plan:${data.planId}`}
        type="plan"
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

const baseData: PlanNodeData = {
  planId: 'p1',
  title: 'Refactor auth flow',
  status: 'in_progress',
  taskCount: 3,
  completedCount: 1,
  approvalRequired: false,
  lastTransitionReason: null,
  durationMs: null,
};

describe('PlanNode', () => {
  it('renders_plan_title_and_progress', () => {
    renderPlan(baseData);
    expect(screen.getByText('Refactor auth flow')).toBeInTheDocument();
    expect(screen.getByText(/1\s*\/\s*3/)).toBeInTheDocument();
  });

  it('applies_status_class_for_each_status_value', () => {
    for (const status of [
      'pending_approval',
      'approved',
      'in_progress',
      'complete',
      'aborted',
    ] as const) {
      const { unmount } = renderPlan({ ...baseData, status });
      const root = screen.getByTestId('plan-node-p1');
      expect(root.className).toContain(`plan-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('exposes_accessible_aria_label_with_title_and_progress', () => {
    renderPlan(baseData);
    const root = screen.getByTestId('plan-node-p1');
    expect(root).toHaveAttribute(
      'aria-label',
      expect.stringMatching(/plan.*Refactor auth flow.*in_progress/i),
    );
  });

  it('exposes_data_testid_and_data_status_attributes_for_e2e', () => {
    renderPlan(baseData);
    const root = screen.getByTestId('plan-node-p1');
    expect(root).toHaveAttribute('data-testid', 'plan-node-p1');
    expect(root).toHaveAttribute('data-status', 'in_progress');
  });

  it('renders_source_and_target_handle_elements', () => {
    renderPlan(baseData);
    const root = screen.getByTestId('plan-node-p1');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
