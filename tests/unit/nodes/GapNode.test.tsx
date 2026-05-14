import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { GapNode } from '../../../src/components/nodes/GapNode';
import type { GapNodeData } from '../../../src/lib/graphStore';

function renderGap(data: GapNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <GapNode
        id={`gap:${data.gapId}`}
        type="gap"
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

const baseData: GapNodeData = {
  gapId: 'gap:tool_missing:fetch_prs:worker',
  kind: 'tool_missing',
  missingName: 'fetch_prs',
  agentId: 'worker',
  severity: 'critical',
  suggestedAction: "Install tool 'fetch_prs' and click Resume.",
  requestedVia: 'loader',
  status: 'gap',
};

describe('GapNode', () => {
  it('renders_missing_artifact_name', () => {
    renderGap(baseData);
    expect(screen.getByText('fetch_prs')).toBeInTheDocument();
  });

  it('renders_severity_and_suggested_action', () => {
    // M05 Stage A: GapNode surfaces severity (visual tier) + suggested_action
    // (plain-English next step) per spec §4b. Without this assertion the
    // M04.V-class bug (renderer doesn't read enriched payload — gotcha #68)
    // can survive: data fields populated upstream but invisible in DOM.
    renderGap(baseData);
    const root = screen.getByTestId(baseData.gapId);
    expect(root).toHaveAttribute('data-severity', 'critical');
    expect(screen.getByText("Install tool 'fetch_prs' and click Resume.")).toBeInTheDocument();
  });

  it('four_kinds_render_distinguishable_visual', () => {
    // Each of the four `*_missing` kinds must paint a distinguishable
    // surface; the DOM-readable discriminators are `data-kind` (the wire
    // discriminator) and the rendered text. Without this test a renderer
    // regression collapsing all four kinds to one visual would pass unit
    // coverage while breaking the spec §4b severity matrix's UX contract.
    const kinds: GapNodeData['kind'][] = [
      'tool_missing',
      'skill_missing',
      'mcp_missing',
      'agent_missing',
    ];
    for (const kind of kinds) {
      const { unmount } = renderGap({
        ...baseData,
        gapId: `gap:${kind}:x:worker`,
        kind,
        missingName: `x-${kind}`,
      });
      const root = screen.getByTestId(`gap:${kind}:x:worker`);
      expect(root).toHaveAttribute('data-kind', kind);
      // The rendered text mentions the kind so a screen-reader user can
      // distinguish without color.
      expect(root.textContent).toContain(kind);
      unmount();
    }
  });

  it('exposes_data_kind_attribute_distinguishing_tool_vs_skill_missing', () => {
    const { unmount } = renderGap(baseData);
    const root = screen.getByTestId(baseData.gapId);
    expect(root).toHaveAttribute('data-kind', 'tool_missing');
    unmount();

    renderGap({
      ...baseData,
      gapId: 'gap:skill_missing:planner:worker',
      kind: 'skill_missing',
      missingName: 'planner',
      severity: 'advisory',
    });
    const skillRoot = screen.getByTestId('gap:skill_missing:planner:worker');
    expect(skillRoot).toHaveAttribute('data-kind', 'skill_missing');
    expect(skillRoot).toHaveAttribute('data-severity', 'advisory');
  });

  it('applies_severity_modifier_class_for_pulse_keyframe', () => {
    // Spec §3 Visual Design: severity drives color + pulse. The CSS hook
    // is gap-node--<severity> which drives the @keyframes gap-pulse-* in
    // styles.css. Stage F polishes the actual CSS; Stage A pins the class.
    renderGap(baseData);
    const root = screen.getByTestId(baseData.gapId);
    expect(root.className).toContain('gap-node--critical');
  });

  it('exposes_accessible_aria_label_naming_the_gap_kind_and_severity', () => {
    renderGap(baseData);
    const root = screen.getByTestId(baseData.gapId);
    expect(root).toHaveAttribute(
      'aria-label',
      expect.stringMatching(/gap.*tool_missing.*fetch_prs.*worker.*critical/i),
    );
  });

  it('exposes_requested_via_discriminator', () => {
    // Loader-driven gaps render the same shape as request_capability-driven
    // gaps but carry a different `data-requested-via` attribute so the
    // renderer + e2e tests can assert the wire path.
    renderGap({ ...baseData, requestedVia: 'request_capability', severity: 'requested' });
    const root = screen.getByTestId(baseData.gapId);
    expect(root).toHaveAttribute('data-requested-via', 'request_capability');
  });

  it('renders_target_handle_only_no_source', () => {
    // GapNode is a terminal — agents fail INTO it; the gap doesn't
    // call out further. Per spec §3 Behavior.
    renderGap(baseData);
    const root = screen.getByTestId(baseData.gapId);
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(1);
  });
});
