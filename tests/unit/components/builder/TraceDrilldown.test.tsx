import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { TraceDrilldown, foldTrace } from '../../../../src/components/builder/TraceDrilldown';
import type { AgentEvent } from '../../../../src/types/agent_event';

// M08.9.B — the Tester run drill-down (verdict → per-tool-call input/result
// → raw). Renderer-only: maps `outcome.trace` to a step list, reusing the
// Output-rail payload formatter + the ValidationCard Show-raw disclosure.
// DESIGN.md principle 1 (feedback) + principle 3 (progressive disclosure).

const SESSION_START: AgentEvent = {
  type: 'session_start',
  session_id: 's1',
  framework: 'fixture',
  model: 'claude-haiku-4-5',
};
const AGENT_SPAWNED: AgentEvent = {
  type: 'agent_spawned',
  agent_id: 'worker',
  agent_name: 'worker',
  session_id: 's1',
};
const READ_INVOKED: AgentEvent = {
  type: 'tool_invoked',
  agent_id: 'worker',
  tool_name: 'Read',
  source: 'builtin',
  input: { path: 'Cargo.toml' },
};
const READ_RESULT: AgentEvent = {
  type: 'tool_result',
  agent_id: 'worker',
  tool_name: 'Read',
  output: '[package]\nname = "agent-runtime"',
  duration_ms: 7,
};
const WRITE_INVOKED: AgentEvent = {
  type: 'tool_invoked',
  agent_id: 'worker',
  tool_name: 'Write',
  source: 'builtin',
  input: { path: 'report.md', contents: 'hello' },
};
const WRITE_ERROR: AgentEvent = {
  type: 'tool_error',
  agent_id: 'worker',
  tool_name: 'Write',
  error: 'tier blocked',
};
const TIER_VIOLATION: AgentEvent = {
  type: 'tier_violation',
  agent_id: 'worker',
  tier: 'novice',
  capability_kind: 'write',
  attempted_action: 'write report.md',
};
const CAPABILITY_VIOLATION: AgentEvent = {
  type: 'capability_violation',
  agent_id: 'worker',
  capability_kind: 'read',
  declared_scope: 'src/**',
  requested_action: 'read /etc/passwd',
};

describe('foldTrace', () => {
  it('pairs a tool_invoked with its following tool_result', () => {
    const steps = foldTrace([SESSION_START, AGENT_SPAWNED, READ_INVOKED, READ_RESULT]);
    expect(steps).toHaveLength(1);
    const step = steps[0];
    expect(step?.kind).toBe('tool');
    if (step?.kind === 'tool') {
      expect(step.invoked.tool_name).toBe('Read');
      expect(step.result?.output).toBe('[package]\nname = "agent-runtime"');
      expect(step.error).toBeNull();
    }
  });

  it('pairs a tool_invoked with a tool_error when the call failed', () => {
    const steps = foldTrace([WRITE_INVOKED, WRITE_ERROR]);
    expect(steps).toHaveLength(1);
    const step = steps[0];
    expect(step?.kind).toBe('tool');
    if (step?.kind === 'tool') {
      expect(step.result).toBeNull();
      expect(step.error?.error).toBe('tier blocked');
    }
  });

  it('leaves an unmatched tool_invoked with a null result', () => {
    const steps = foldTrace([READ_INVOKED]);
    expect(steps).toHaveLength(1);
    const step = steps[0];
    if (step?.kind === 'tool') {
      expect(step.result).toBeNull();
      expect(step.error).toBeNull();
    }
  });

  it('folds tier_violation + capability_violation into distinct steps', () => {
    const steps = foldTrace([TIER_VIOLATION, CAPABILITY_VIOLATION]);
    expect(steps.map((s) => s.kind)).toEqual(['tier', 'capability']);
  });

  it('ignores session/agent lifecycle events (not drillable)', () => {
    expect(foldTrace([SESSION_START, AGENT_SPAWNED])).toHaveLength(0);
  });

  it('preserves trace order across mixed events', () => {
    const steps = foldTrace([
      READ_INVOKED,
      READ_RESULT,
      TIER_VIOLATION,
      WRITE_INVOKED,
      WRITE_ERROR,
    ]);
    expect(steps.map((s) => s.kind)).toEqual(['tool', 'tier', 'tool']);
  });
});

describe('TraceDrilldown', () => {
  afterEach(cleanup);

  it('renders nothing when there are no drillable steps', () => {
    const { container } = render(<TraceDrilldown trace={[SESSION_START, AGENT_SPAWNED]} />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByTestId('trace-drilldown')).toBeNull();
  });

  it('renders one collapsed row per tool call', () => {
    render(<TraceDrilldown trace={[READ_INVOKED, READ_RESULT]} />);
    const rows = screen.getAllByTestId('trace-step');
    expect(rows).toHaveLength(1);
    // Collapsed by default — the payload detail is not in the DOM.
    expect(screen.queryByTestId('trace-step-detail')).toBeNull();
    expect(screen.getByTestId('trace-step-toggle')).toHaveTextContent('Read');
  });

  it('expands a tool call to its input and result payload', () => {
    render(<TraceDrilldown trace={[READ_INVOKED, READ_RESULT]} />);
    fireEvent.click(screen.getByTestId('trace-step-toggle'));
    expect(screen.getByTestId('trace-step-detail')).toBeInTheDocument();
    expect(screen.getByTestId('trace-step-input')).toHaveTextContent('Cargo.toml');
    expect(screen.getByTestId('trace-step-output')).toHaveTextContent('[package]');
  });

  it('shows an em-dash for an unmatched tool call output', () => {
    render(<TraceDrilldown trace={[READ_INVOKED]} />);
    fireEvent.click(screen.getByTestId('trace-step-toggle'));
    expect(screen.getByTestId('trace-step-output')).toHaveTextContent('—');
  });

  it('reveals the raw event JSON behind a Show-raw disclosure', () => {
    render(<TraceDrilldown trace={[READ_INVOKED, READ_RESULT]} />);
    fireEvent.click(screen.getByTestId('trace-step-toggle'));
    // Collapsed by default — the raw event is one more click away.
    expect(screen.queryByTestId('trace-step-raw')).toBeNull();
    fireEvent.click(screen.getByTestId('trace-step-raw-toggle'));
    const raw = screen.getByTestId('trace-step-raw');
    expect(raw).toHaveTextContent('tool_invoked');
    expect(raw).toHaveTextContent('Cargo.toml');
  });

  it('renders a tier_violation as a distinct row linking to the tier explainer', () => {
    render(<TraceDrilldown trace={[TIER_VIOLATION]} />);
    const row = screen.getByTestId('trace-step');
    expect(row).toHaveClass('trace-step--tier');
    expect(row).toHaveTextContent('write report.md');
    expect(screen.getByTestId('trace-step-tier-explainer')).toBeInTheDocument();
  });

  it('renders a capability_violation as a distinct row linking to its explainer', () => {
    render(<TraceDrilldown trace={[CAPABILITY_VIOLATION]} />);
    const row = screen.getByTestId('trace-step');
    expect(row).toHaveClass('trace-step--capability');
    expect(screen.getByTestId('trace-step-cap-explainer')).toBeInTheDocument();
    expect(screen.getByTestId('trace-step-cap-explainer')).toHaveTextContent('read /etc/passwd');
  });

  it('styles.css defines the trace-step rules (gotcha #67 — new surface)', () => {
    const css = readFileSync(resolve(__dirname, '../../../../src/styles.css'), 'utf8');
    expect(css).toMatch(/\.trace-step[\s,{]/);
    expect(css).toMatch(/\.trace-drilldown[\s,{]/);
  });
});
