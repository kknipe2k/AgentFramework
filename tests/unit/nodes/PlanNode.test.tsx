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
    // Stage C extends to all 7 PlanStatus values per spec §3a:
    // pending_approval, awaiting_approval, approved, in_progress,
    // awaiting_replan, complete, aborted.
    for (const status of [
      'pending_approval',
      'awaiting_approval',
      'approved',
      'in_progress',
      'awaiting_replan',
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

  it('renders_status_badge_text_matching_status', () => {
    for (const status of [
      'awaiting_approval',
      'awaiting_replan',
      'in_progress',
      'complete',
      'aborted',
    ] as const) {
      const { unmount } = renderPlan({ ...baseData, status });
      // Badge surfaces the status — test on a status-specific element so
      // both the className-driven color and the textual label are checked.
      const badge = screen.getByTestId('plan-node-p1').querySelector('.plan-node__status');
      expect(badge).not.toBeNull();
      expect(badge!.textContent).toMatch(new RegExp(status.replace('_', '.'), 'i'));
      unmount();
    }
  });

  it('renders_revision_reason_when_status_is_awaiting_replan', () => {
    renderPlan({
      ...baseData,
      status: 'awaiting_replan',
      lastTransitionReason: 'expand risk callouts',
    });
    expect(screen.getByText(/expand risk callouts/)).toBeInTheDocument();
  });

  it('renders_abort_reason_when_status_is_aborted', () => {
    renderPlan({
      ...baseData,
      status: 'aborted',
      lastTransitionReason: 'user cancelled',
    });
    expect(screen.getByText(/user cancelled/)).toBeInTheDocument();
  });

  it('renders_duration_when_status_is_complete', () => {
    renderPlan({
      ...baseData,
      status: 'complete',
      completedCount: 3,
      durationMs: 4567,
    });
    // 4567ms → 4.6s (one decimal) — the renderer formats for human reading.
    const root = screen.getByTestId('plan-node-p1');
    expect(root.textContent).toMatch(/4\.[0-9]\s*s/);
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
