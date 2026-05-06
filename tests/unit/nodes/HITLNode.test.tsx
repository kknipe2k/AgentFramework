import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { HITLNode } from '../../../src/components/nodes/HITLNode';
import type { HITLNodeData } from '../../../src/lib/graphStore';

function renderHITL(data: HITLNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <HITLNode
        id={`hitl:${data.hitlId}`}
        type="hitl"
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

const baseData: HITLNodeData = {
  hitlId: 'h1',
  prompt: 'Approve plan?',
  resolved: false,
};

describe('HITLNode', () => {
  it('renders_prompt_text', () => {
    renderHITL(baseData);
    expect(screen.getByText('Approve plan?')).toBeInTheDocument();
  });

  it('has_role_alert_and_aria_live_assertive_when_unresolved', () => {
    // WAI ARIA APG: blocking-on-input affordance uses role=alert +
    // aria-live=assertive so screenreaders announce the prompt
    // immediately when the node spawns.
    renderHITL(baseData);
    const root = screen.getByTestId('hitl-node-h1');
    expect(root).toHaveAttribute('role', 'alert');
    expect(root).toHaveAttribute('aria-live', 'assertive');
  });

  it('applies_hitl_modifier_class_when_unresolved', () => {
    renderHITL(baseData);
    const root = screen.getByTestId('hitl-node-h1');
    // Spec §3 Visual Design: hitl = white/bright.
    expect(root.className).toContain('hitl-node--hitl');
  });

  it('applies_complete_modifier_class_when_resolved', () => {
    renderHITL({ ...baseData, resolved: true });
    const root = screen.getByTestId('hitl-node-h1');
    expect(root.className).toContain('hitl-node--complete');
  });

  it('renders_target_handle_only_no_source', () => {
    // HITL blocks the graph flow — input lands here; flow only
    // resumes via UI side-channel, not via a graph edge.
    renderHITL(baseData);
    const root = screen.getByTestId('hitl-node-h1');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(1);
  });
});
