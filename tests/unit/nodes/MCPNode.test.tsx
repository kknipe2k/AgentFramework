import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { MCPNode } from '../../../src/components/nodes/MCPNode';
import type { MCPNodeData } from '../../../src/lib/graphStore';

function renderMCP(data: MCPNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <MCPNode
        id={`mcp:${data.serverId}`}
        type="mcp"
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

const baseData: MCPNodeData = {
  serverId: 'github-mcp',
  serverName: 'github-mcp',
  status: 'active',
  discoveredToolCount: null,
};

describe('MCPNode', () => {
  it('renders_server_name', () => {
    renderMCP(baseData);
    expect(screen.getByText('github-mcp')).toBeInTheDocument();
  });

  it('applies_active_complete_error_status_classes', () => {
    for (const status of ['active', 'complete', 'error'] as const) {
      const { unmount } = renderMCP({ ...baseData, status });
      const root = screen.getByTestId('mcp-node-github-mcp');
      expect(root.className).toContain(`mcp-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('renders_tool_count_when_provided', () => {
    renderMCP({ ...baseData, discoveredToolCount: 7 });
    expect(screen.getByText(/7\s*tools?/i)).toBeInTheDocument();
  });

  it('exposes_accessible_aria_label_with_server_name_and_status', () => {
    renderMCP(baseData);
    const root = screen.getByTestId('mcp-node-github-mcp');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/mcp.*github-mcp.*active/i));
  });

  it('renders_source_and_target_handle_elements', () => {
    renderMCP(baseData);
    const root = screen.getByTestId('mcp-node-github-mcp');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
