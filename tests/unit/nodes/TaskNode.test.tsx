import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { TaskNode } from '../../../src/components/nodes/TaskNode';
import type { TaskNodeData } from '../../../src/lib/graphStore';

function renderTask(data: TaskNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <TaskNode
        id={`task:${data.taskId}`}
        type="task"
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

const baseData: TaskNodeData = {
  taskId: 't1',
  planId: 'p1',
  title: 'Read source files',
  status: 'running',
  hitl: false,
  agentId: null,
  failureCount: 0,
  maxFailures: null,
  lastError: null,
  durationMs: null,
  rollbackSnapshotId: null,
};

describe('TaskNode', () => {
  it('renders_task_title', () => {
    renderTask(baseData);
    expect(screen.getByText('Read source files')).toBeInTheDocument();
  });

  it('applies_status_class_for_each_status_value', () => {
    for (const status of ['pending', 'running', 'done', 'blocked', 'failed', 'skipped'] as const) {
      const { unmount } = renderTask({ ...baseData, status });
      const root = screen.getByTestId('task-node-t1');
      expect(root.className).toContain(`task-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('renders_hitl_badge_when_flag_is_true', () => {
    renderTask({ ...baseData, hitl: true });
    const root = screen.getByTestId('task-node-t1');
    expect(root.querySelector('.task-node__hitl-badge')).not.toBeNull();
  });

  it('exposes_accessible_aria_label_with_title_and_status', () => {
    renderTask(baseData);
    const root = screen.getByTestId('task-node-t1');
    expect(root).toHaveAttribute(
      'aria-label',
      expect.stringMatching(/task.*Read source files.*running/i),
    );
  });

  it('renders_source_and_target_handle_elements', () => {
    renderTask(baseData);
    const root = screen.getByTestId('task-node-t1');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
