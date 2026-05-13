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
    // Stage C extends to all 7 TaskStatus values per spec §3a, including
    // `escalated` (M04 Stage B added — failure_count >= max_failures).
    for (const status of [
      'pending',
      'running',
      'done',
      'blocked',
      'failed',
      'skipped',
      'escalated',
    ] as const) {
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

  it('renders_failure_count_badge_when_failure_count_gt_zero', () => {
    renderTask({ ...baseData, failureCount: 2, maxFailures: 3, status: 'failed' });
    const badge = screen.getByTestId('task-node-t1').querySelector('.task-node__failure-badge');
    expect(badge).not.toBeNull();
    // Format per spec §3a Visual Design: `⚠ N/M` (or just `⚠ N` when no
    // budget recorded yet — failure_count without max_failures is the
    // pre-escalation case).
    expect(badge!.textContent).toMatch(/2\s*\/\s*3/);
  });

  it('renders_failure_count_without_denominator_when_max_failures_is_null', () => {
    renderTask({ ...baseData, failureCount: 1, maxFailures: null, status: 'failed' });
    const badge = screen.getByTestId('task-node-t1').querySelector('.task-node__failure-badge');
    expect(badge).not.toBeNull();
    expect(badge!.textContent).toMatch(/1/);
    expect(badge!.textContent).not.toMatch(/\//);
  });

  it('omits_failure_count_badge_when_failure_count_is_zero', () => {
    renderTask({ ...baseData, failureCount: 0, status: 'running' });
    const badge = screen.getByTestId('task-node-t1').querySelector('.task-node__failure-badge');
    expect(badge).toBeNull();
  });

  it('renders_duration_when_status_is_done', () => {
    renderTask({ ...baseData, status: 'done', durationMs: 1234 });
    const root = screen.getByTestId('task-node-t1');
    expect(root.textContent).toMatch(/1\.[0-9]\s*s/);
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

  it('renders_task_id_prefix_fallback_when_title_is_empty', () => {
    // M04.V Decision 1 — IRL LG-02: TaskNodes from `task_started` events
    // showed up "untitled" because the schema has no `title` field on the
    // event. Fix lives at TaskNode.tsx:27 — `displayTitle = title || \`task
    // ${taskId.slice(0,8)}\``. Without this regression test the fix's
    // line is uncovered (per M04.V Finding #1) and a future refactor could
    // silently drop the fallback. Per gotcha #71 (Schema field "missing"
    // can render as blank string).
    renderTask({ ...baseData, taskId: 'a1b2c3d4-deadbeef', title: '' });
    const root = screen.getByTestId('task-node-a1b2c3d4-deadbeef');
    // The fallback uses the first 8 chars of the task id.
    expect(root.textContent).toMatch(/task a1b2c3d4/);
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/task task a1b2c3d4/));
  });
});
