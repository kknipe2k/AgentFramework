import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { AgentNode } from '../../../src/components/nodes/AgentNode';
import type { AgentNodeData } from '../../../src/lib/graphStore';

// React Flow custom nodes assume they're rendered inside a
// <ReactFlowProvider> (Handle / NodeId hooks read from context). All
// node-component tests wrap their renders accordingly.

function renderAgent(data: AgentNodeData): void {
  render(
    <ReactFlowProvider>
      <AgentNode
        id={`agent:${data.agentId}`}
        type="agent"
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

const baseData: AgentNodeData = {
  agentId: 'a1-uuid-12345678',
  agentName: 'smoke',
  status: 'active',
  parentAgentId: null,
};

describe('AgentNode', () => {
  it('renders_agent_name_and_truncated_id', () => {
    renderAgent(baseData);
    expect(screen.getByText('smoke')).toBeInTheDocument();
    // First 8 chars of agent_id only.
    expect(screen.getByText('a1-uuid-')).toBeInTheDocument();
  });

  it('applies_active_complete_error_status_classes', () => {
    for (const status of ['active', 'complete', 'error'] as const) {
      const { unmount } = render(
        <ReactFlowProvider>
          <AgentNode
            id="agent:a1"
            type="agent"
            data={{ ...baseData, status }}
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
      const root = screen.getByTestId(`agent-node-${baseData.agentId}`);
      expect(root.className).toContain(`agent-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('exposes_accessible_aria_label_with_name_and_status', () => {
    renderAgent(baseData);
    const root = screen.getByTestId(`agent-node-${baseData.agentId}`);
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/smoke.*active/i));
  });

  it('exposes_data_testid_and_data_status_attributes_for_e2e', () => {
    renderAgent(baseData);
    const root = screen.getByTestId(`agent-node-${baseData.agentId}`);
    expect(root).toHaveAttribute('data-testid', `agent-node-${baseData.agentId}`);
    expect(root).toHaveAttribute('data-status', 'active');
  });

  it('renders_source_and_target_handle_elements', () => {
    renderAgent(baseData);
    // React Flow's Handle component renders <div> elements with the
    // `react-flow__handle` class — assert both target (top) and source
    // (bottom) are present.
    const root = screen.getByTestId(`agent-node-${baseData.agentId}`);
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});
