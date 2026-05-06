import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { FrameworkNode } from '../../../src/components/nodes/FrameworkNode';
import type { FrameworkNodeData } from '../../../src/lib/graphStore';

function renderFramework(data: FrameworkNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <FrameworkNode
        id={`framework:${data.frameworkName}`}
        type="framework"
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

const baseData: FrameworkNodeData = {
  frameworkName: 'aria',
  model: 'haiku',
  status: 'active',
};

describe('FrameworkNode', () => {
  it('renders_framework_name_and_model', () => {
    renderFramework(baseData);
    expect(screen.getByText('aria')).toBeInTheDocument();
    expect(screen.getByText('haiku')).toBeInTheDocument();
  });

  it('applies_active_complete_error_status_classes', () => {
    for (const status of ['active', 'complete', 'error'] as const) {
      const { unmount } = renderFramework({ ...baseData, status });
      const root = screen.getByTestId('framework-node-aria');
      expect(root.className).toContain(`framework-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('exposes_accessible_aria_label_with_framework_and_model', () => {
    renderFramework(baseData);
    const root = screen.getByTestId('framework-node-aria');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/framework.*aria.*haiku/i));
  });

  it('exposes_data_testid_and_data_status_attributes_for_e2e', () => {
    renderFramework(baseData);
    const root = screen.getByTestId('framework-node-aria');
    expect(root).toHaveAttribute('data-testid', 'framework-node-aria');
    expect(root).toHaveAttribute('data-status', 'active');
  });

  it('renders_source_handle_only_as_graph_root', () => {
    // FrameworkNode is the graph root — flow originates here, no
    // upstream parent in the v0.1 graph.
    renderFramework(baseData);
    const root = screen.getByTestId('framework-node-aria');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(1);
  });
});
