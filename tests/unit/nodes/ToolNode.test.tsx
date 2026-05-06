import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { ToolNode } from '../../../src/components/nodes/ToolNode';
import type { ToolNodeData } from '../../../src/lib/graphStore';

function renderTool(data: ToolNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <ToolNode
        id={`tool:${data.agentId}:${data.toolName}`}
        type="tool"
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

const baseData: ToolNodeData = {
  toolName: 'read_file',
  agentId: 'a1',
  status: 'active',
  durationMs: null,
  tokensIn: 0,
  tokensOut: 0,
};

describe('ToolNode', () => {
  it('renders_tool_name', () => {
    renderTool(baseData);
    expect(screen.getByText('read_file')).toBeInTheDocument();
  });

  it('applies_active_complete_error_status_classes', () => {
    for (const status of ['active', 'complete', 'error'] as const) {
      const { unmount } = renderTool({ ...baseData, status });
      const root = screen.getByTestId(`tool-node-a1-read_file`);
      expect(root.className).toContain(`tool-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('renders_duration_when_complete', () => {
    renderTool({ ...baseData, status: 'complete', durationMs: 42 });
    expect(screen.getByText(/42\s*ms/i)).toBeInTheDocument();
  });

  it('exposes_accessible_aria_label_with_name_and_status', () => {
    renderTool(baseData);
    const root = screen.getByTestId('tool-node-a1-read_file');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/read_file.*active/i));
  });

  it('renders_source_and_target_handle_elements', () => {
    renderTool(baseData);
    const root = screen.getByTestId('tool-node-a1-read_file');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
