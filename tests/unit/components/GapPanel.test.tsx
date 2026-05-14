import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { act, render, screen } from '@testing-library/react';
import { GapPanel } from '../../../src/components/GapPanel';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

// Spec §4b — GapPanel is the right-rail surface for unresolved gaps. The
// projection (graphStore.applyEvent) mounts GapNodes on the four `*_missing`
// variants and dismisses them on `gap_resolved`. GapPanel subscribes via
// a selector — it has no event handlers of its own. Per gotcha #68 (read
// the fields the projection writes) and gotcha #67 (every class set must
// have a CSS rule), this suite drives both contracts.

function dispatch(events: AgentEvent[]): void {
  act(() => {
    for (const e of events) {
      useGraphStore.getState().applyEvent(e);
    }
  });
}

const toolMissing: AgentEvent = {
  type: 'tool_missing',
  agent_id: 'worker',
  tool_name: 'fetch_prs',
  severity: 'critical',
  suggested_action: "Install tool 'fetch_prs' and click Resume.",
  requested_via: 'loader',
};

const skillMissing: AgentEvent = {
  type: 'skill_missing',
  agent_id: 'worker',
  skill_name: 'planner',
  severity: 'advisory',
  suggested_action: "Add skill 'planner' to the framework.",
  requested_via: 'loader',
};

const mcpMissing: AgentEvent = {
  type: 'mcp_missing',
  agent_id: 'worker',
  server_name: 'github-mcp',
  severity: 'important',
  suggested_action: 'Configure the github-mcp server.',
  requested_via: 'request_capability',
};

describe('GapPanel', () => {
  beforeEach(() => {
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    useGraphStore.getState().clear();
  });

  it('renders_nothing_when_no_gaps', () => {
    const { container } = render(<GapPanel />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByTestId('gap-panel')).toBeNull();
  });

  it('renders_one_item_per_unresolved_gap', () => {
    dispatch([toolMissing, skillMissing, mcpMissing]);
    render(<GapPanel />);
    const panel = screen.getByTestId('gap-panel');
    expect(panel).toBeInTheDocument();
    // Three gaps → three list items.
    const items = panel.querySelectorAll('[data-testid^="gap-item-"]');
    expect(items.length).toBe(3);
  });

  it('item_shows_kind_missing_name_suggested_action_and_agent_id_per_gotcha_68', () => {
    // Gotcha #68: the projection writes `kind`, `missingName`, `severity`,
    // `suggestedAction`, `agentId`; the consumer MUST read all five. A
    // regression that drops any of them would render an empty / wrong
    // surface while unit-coverage of the math layer stays green.
    dispatch([toolMissing]);
    render(<GapPanel />);
    const panel = screen.getByTestId('gap-panel');
    expect(panel).toHaveTextContent('tool_missing');
    expect(panel).toHaveTextContent('fetch_prs');
    expect(panel).toHaveTextContent("Install tool 'fetch_prs' and click Resume.");
    // First 8 chars of agent_id; mirrors AgentNode's truncation policy
    // so the surface stays visually compact when ids are uuids.
    expect(panel).toHaveTextContent('worker');
  });

  it('applies_severity_modifier_class_per_item', () => {
    dispatch([toolMissing, skillMissing, mcpMissing]);
    render(<GapPanel />);
    // The Stage A projection keys gap nodes by `gap:${kind}:${name}:${agentId}`.
    const critical = screen.getByTestId('gap-item-gap:tool_missing:fetch_prs:worker');
    expect(critical.className).toContain('gap-panel__item--critical');
    const advisory = screen.getByTestId('gap-item-gap:skill_missing:planner:worker');
    expect(advisory.className).toContain('gap-panel__item--advisory');
    const important = screen.getByTestId('gap-item-gap:mcp_missing:github-mcp:worker');
    expect(important.className).toContain('gap-panel__item--important');
  });

  it('dismisses_item_when_gap_resolved_event_fires', () => {
    dispatch([toolMissing]);
    render(<GapPanel />);
    expect(screen.getByTestId('gap-panel')).toBeInTheDocument();
    dispatch([
      {
        type: 'gap_resolved',
        agent_id: 'worker',
        kind: 'tool',
        capability: 'fetch_prs',
      },
    ]);
    // After resolution the projection removes the GapNode; with zero gaps
    // the panel returns null.
    expect(screen.queryByTestId('gap-panel')).toBeNull();
  });

  it('shows_count_in_title_matching_number_of_unresolved_gaps', () => {
    dispatch([toolMissing, skillMissing]);
    render(<GapPanel />);
    const title = screen.getByTestId('gap-panel-title');
    expect(title.textContent).toMatch(/2/);
  });

  it('every_severity_class_has_a_corresponding_CSS_rule_in_styles_css', () => {
    // Gotcha #67 (PR #64 pattern). Component sets `.gap-panel__item--<severity>`
    // for each of the four GapSeverity values; styles.css MUST define each
    // selector or the visual differentiation is invisible to the user even
    // though the className output is correct.
    const cssPath = resolve(__dirname, '../../../src/styles.css');
    const css = readFileSync(cssPath, 'utf8');
    for (const severity of ['critical', 'important', 'advisory', 'requested'] as const) {
      const selector = `.gap-panel__item--${severity}`;
      expect(css, `missing CSS rule for ${selector}`).toContain(selector);
    }
    // Base container class also needs a rule (otherwise the rail has no
    // background and floats invisibly over the canvas).
    expect(css, 'missing CSS rule for .gap-panel').toContain('.gap-panel');
  });

  it('exposes_accessible_region_role_and_aria_label', () => {
    dispatch([toolMissing]);
    render(<GapPanel />);
    const panel = screen.getByTestId('gap-panel');
    expect(panel).toHaveAttribute('role', 'region');
    const label = panel.getAttribute('aria-label') ?? '';
    expect(label.toLowerCase()).toContain('gap');
  });
});
