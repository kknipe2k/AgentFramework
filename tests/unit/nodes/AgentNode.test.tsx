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
  tokensIn: 0,
  tokensOut: 0,
  tokensTotal: 0,
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

  it('reads_tokensTotal_for_visual_scale_not_tokensIn_plus_tokensOut', () => {
    // M04 IRL regression: AgentNode previously read
    // `tokensIn + tokensOut`, which only populate from `tool_result`
    // events. The session-cumulative count from `agent_complete.tokens_total`
    // landed in `tokensTotal` but was ignored — visual scaling was
    // always 0.8 (floor) regardless of actual spend.
    function scaleFor(data: AgentNodeData): string {
      const { unmount } = render(
        <ReactFlowProvider>
          <AgentNode
            id="agent:a"
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
      const root = screen.getByTestId(`agent-node-${data.agentId}`);
      const transform = root.getAttribute('style')?.match(/scale\(([\d.]+)\)/)?.[1] ?? '';
      unmount();
      return transform;
    }

    // tokensTotal drives the scale. Two agents with identical tokensIn /
    // tokensOut but different tokensTotal should render at different sizes.
    const lowScale = scaleFor({ ...baseData, agentId: 'low', tokensTotal: 10 });
    const highScale = scaleFor({ ...baseData, agentId: 'high', tokensTotal: 50000 });

    expect(Number(lowScale)).toBeLessThan(Number(highScale));
    // ≥1.5× delta keeps the side-by-side visual comparison readable.
    expect(Number(highScale) / Number(lowScale)).toBeGreaterThan(1.5);

    // tokensIn/tokensOut are NOT inputs — varying them must NOT change scale.
    const baselineScale = scaleFor({ ...baseData, agentId: 'a', tokensTotal: 100 });
    const sameWithIO = scaleFor({
      ...baseData,
      agentId: 'b',
      tokensTotal: 100,
      tokensIn: 5000,
      tokensOut: 5000,
    });
    expect(baselineScale).toBe(sameWithIO);
  });
});
