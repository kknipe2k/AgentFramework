import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { VerifyNode } from '../../../src/components/nodes/VerifyNode';
import type { VerifyNodeData } from '../../../src/lib/graphStore';

function renderVerify(data: VerifyNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <VerifyNode
        id={`verify:${data.hookId}`}
        type="verify"
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

const baseData: VerifyNodeData = {
  hookId: 'post_task',
  level: 'L1',
  status: 'active',
  durationMs: null,
};

describe('VerifyNode', () => {
  it('renders_hook_id_and_level', () => {
    renderVerify(baseData);
    expect(screen.getByText('post_task')).toBeInTheDocument();
    expect(screen.getByText(/L1/i)).toBeInTheDocument();
  });

  it('applies_status_class_for_active_pass_fail', () => {
    for (const status of ['active', 'pass', 'fail'] as const) {
      const { unmount } = renderVerify({ ...baseData, status });
      const root = screen.getByTestId('verify-node-post_task');
      expect(root.className).toContain(`verify-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('renders_duration_when_provided', () => {
    renderVerify({ ...baseData, status: 'pass', durationMs: 120 });
    expect(screen.getByText(/120\s*ms/i)).toBeInTheDocument();
  });

  it('exposes_accessible_aria_label_with_hook_and_status', () => {
    renderVerify(baseData);
    const root = screen.getByTestId('verify-node-post_task');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/verify.*post_task.*active/i));
  });

  it('renders_source_and_target_handle_elements', () => {
    renderVerify(baseData);
    const root = screen.getByTestId('verify-node-post_task');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
