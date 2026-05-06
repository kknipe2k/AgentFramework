import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { HookNode } from '../../../src/components/nodes/HookNode';
import type { HookNodeData } from '../../../src/lib/graphStore';

function renderHook(data: HookNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <HookNode
        id={`hook:${data.hookId}`}
        type="hook"
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

const baseData: HookNodeData = {
  hookId: 'pre_commit',
  hookName: 'pre_commit',
  category: 'git',
  status: 'active',
};

describe('HookNode', () => {
  it('renders_hook_name_and_category', () => {
    renderHook(baseData);
    expect(screen.getByText('pre_commit')).toBeInTheDocument();
    expect(screen.getByText('git')).toBeInTheDocument();
  });

  it('applies_active_complete_error_status_classes', () => {
    for (const status of ['active', 'complete', 'error'] as const) {
      const { unmount } = renderHook({ ...baseData, status });
      const root = screen.getByTestId('hook-node-pre_commit');
      expect(root.className).toContain(`hook-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('exposes_accessible_aria_label_with_hook_name_and_category', () => {
    renderHook(baseData);
    const root = screen.getByTestId('hook-node-pre_commit');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/hook.*pre_commit.*git/i));
  });

  it('exposes_data_testid_and_data_status_attributes_for_e2e', () => {
    renderHook(baseData);
    const root = screen.getByTestId('hook-node-pre_commit');
    expect(root).toHaveAttribute('data-testid', 'hook-node-pre_commit');
    expect(root).toHaveAttribute('data-status', 'active');
  });

  it('renders_source_and_target_handle_elements', () => {
    renderHook(baseData);
    const root = screen.getByTestId('hook-node-pre_commit');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
