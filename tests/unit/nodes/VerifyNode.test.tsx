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
  hookId: 'verify',
  level: 'standard',
  firingPoint: 'post_task',
  status: 'active',
  durationMs: null,
  outputPreview: null,
  error: null,
  onFailure: null,
};

describe('VerifyNode', () => {
  it('renders_hook_id_level_firing_point', () => {
    renderVerify(baseData);
    expect(screen.getByText('verify')).toBeInTheDocument();
    expect(screen.getByText(/standard/i)).toBeInTheDocument();
    expect(screen.getByText('post_task')).toBeInTheDocument();
  });

  it('omits_level_div_when_level_is_null', () => {
    renderVerify({ ...baseData, level: null });
    expect(screen.queryByText(/standard/i)).not.toBeInTheDocument();
  });

  it('applies_status_class_for_active_pass_fail', () => {
    for (const status of ['active', 'pass', 'fail'] as const) {
      const { unmount } = renderVerify({ ...baseData, status });
      const root = screen.getByTestId('verify-node-verify');
      expect(root.className).toContain(`verify-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('renders_duration_when_provided', () => {
    renderVerify({ ...baseData, status: 'pass', durationMs: 120 });
    expect(screen.getByText(/120\s*ms/i)).toBeInTheDocument();
  });

  it('renders_output_preview_only_on_pass_status', () => {
    renderVerify({
      ...baseData,
      status: 'pass',
      durationMs: 80,
      outputPreview: 'All checks passed',
    });
    expect(screen.getByText('All checks passed')).toBeInTheDocument();
  });

  it('renders_error_and_on_failure_only_on_fail_status', () => {
    renderVerify({
      ...baseData,
      status: 'fail',
      durationMs: 80,
      error: 'verify.sh exited 1',
      onFailure: 'rollback',
    });
    expect(screen.getByText(/verify\.sh exited 1/)).toBeInTheDocument();
    expect(screen.getByText('rollback')).toBeInTheDocument();
  });

  it('truncates_long_output_preview_with_ellipsis', () => {
    const long = 'x'.repeat(200);
    renderVerify({ ...baseData, status: 'pass', durationMs: 1, outputPreview: long });
    const node = screen.getByTestId('verify-node-verify');
    const txt = node.textContent ?? '';
    expect(txt).toContain('…');
  });

  it('exposes_accessible_aria_label_with_hook_and_status', () => {
    renderVerify(baseData);
    const root = screen.getByTestId('verify-node-verify');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/verify.*active/i));
  });

  it('exposes_data_firing_point_for_e2e_assertions', () => {
    renderVerify(baseData);
    const root = screen.getByTestId('verify-node-verify');
    expect(root).toHaveAttribute('data-firing-point', 'post_task');
  });

  it('renders_source_and_target_handle_elements', () => {
    renderVerify(baseData);
    const root = screen.getByTestId('verify-node-verify');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
