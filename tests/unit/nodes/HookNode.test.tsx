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
  hookId: 'lint',
  hookName: 'lint',
  category: 'lint',
  firingPoint: 'post_file_edit',
  status: 'active',
  durationMs: null,
  error: null,
};

describe('HookNode', () => {
  it('renders_hook_name_category_firing_point', () => {
    renderHook(baseData);
    // hookId === hookName === 'lint' AND category === 'lint' so the
    // string 'lint' appears multiple times in the DOM; query by
    // class to disambiguate.
    expect(screen.getByTestId('hook-node-lint').textContent).toContain('lint');
    expect(screen.getByText('post_file_edit')).toBeInTheDocument();
  });

  it('applies_active_complete_error_status_classes', () => {
    for (const status of ['active', 'complete', 'error'] as const) {
      const { unmount } = renderHook({ ...baseData, status });
      const root = screen.getByTestId('hook-node-lint');
      expect(root.className).toContain(`hook-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('renders_duration_when_provided', () => {
    renderHook({ ...baseData, status: 'complete', durationMs: 42 });
    expect(screen.getByText(/42\s*ms/i)).toBeInTheDocument();
  });

  it('renders_error_only_on_error_status', () => {
    renderHook({
      ...baseData,
      status: 'error',
      durationMs: 50,
      error: 'lint warnings: 12',
    });
    expect(screen.getByText(/lint warnings: 12/)).toBeInTheDocument();
  });

  it('exposes_accessible_aria_label_with_hook_name_and_category', () => {
    renderHook(baseData);
    const root = screen.getByTestId('hook-node-lint');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/hook.*lint.*lint/i));
  });

  it('exposes_data_attributes_for_e2e', () => {
    renderHook(baseData);
    const root = screen.getByTestId('hook-node-lint');
    expect(root).toHaveAttribute('data-testid', 'hook-node-lint');
    expect(root).toHaveAttribute('data-status', 'active');
    expect(root).toHaveAttribute('data-category', 'lint');
    expect(root).toHaveAttribute('data-firing-point', 'post_file_edit');
  });

  it('renders_source_and_target_handle_elements', () => {
    renderHook(baseData);
    const root = screen.getByTestId('hook-node-lint');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
