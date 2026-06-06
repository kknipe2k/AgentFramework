import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M08.9.B — the run drill-down wired INTO the assembled Tester modal (not
// the standalone TraceDrilldown unit). Proves construction reachability:
// running a framework whose trace carries a tool call surfaces a drillable
// step row under the verdict, and expanding it reveals the payload.
//
// Same two mocks as TesterModal.test.tsx: @xyflow/react (TesterGraphPane
// mounts a real <ReactFlow> needing a measured pane happy-dom lacks) and
// ../../../../src/lib/ipc (testFramework replaced; the rest stays real).
interface MockFlowNode {
  id: string;
  type: string;
}
interface MockReactFlowProps {
  nodes: MockFlowNode[];
}

vi.mock('@xyflow/react', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@xyflow/react')>();
  return {
    ...actual,
    ReactFlow: ({ nodes }: MockReactFlowProps) => (
      <div data-testid="rf-mock">
        {nodes.map((n) => (
          <div key={n.id} data-testid={`rf-node-${n.id}`} data-node-type={n.type}>
            {n.id}
          </div>
        ))}
      </div>
    ),
    Background: () => null,
    Controls: () => null,
  };
});

const { testFrameworkMock } = vi.hoisted(() => ({ testFrameworkMock: vi.fn() }));
vi.mock('../../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../../src/lib/ipc')>();
  return { ...actual, testFramework: testFrameworkMock };
});

import { act, fireEvent, render, screen } from '@testing-library/react';
import { TesterModal } from '../../../../src/components/builder/TesterModal';
import { useBuilderStore, useTestGraphStore } from '../../../../src/lib/builderStore';
import { useGraphStore } from '../../../../src/lib/graphStore';
import type { TestOutcome } from '../../../../src/lib/ipc';
import type { AgentEvent } from '../../../../src/types/agent_event';

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
  output: '[package]',
  duration_ms: 7,
};

/** A clean run whose trace carries one drillable tool call. */
function passWithToolCall(): TestOutcome {
  return {
    passed: true,
    verdict: 'pass',
    capability_failures: [],
    tier_blocks: [],
    token_spend: { input: 120, output: 45, total: 165 },
    timing: { secs: 1, nanos: 0 },
    vdr: null,
    trace: [SESSION_START, AGENT_SPAWNED, READ_INVOKED, READ_RESULT],
  };
}

function openTester(): void {
  act(() => {
    useBuilderStore.getState().openTester();
  });
}

describe('TesterModal — the run drill-down (M08.9.B)', () => {
  beforeEach(() => {
    testFrameworkMock.mockReset();
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
    useGraphStore.getState().clear();
    useTestGraphStore.getState().clear();
  });

  afterEach(() => {
    vi.clearAllMocks();
    useGraphStore.getState().clear();
    useTestGraphStore.getState().clear();
  });

  it('surfaces a drillable tool-call row under the verdict after a run', async () => {
    testFrameworkMock.mockResolvedValue(passWithToolCall());
    render(<TesterModal />);
    openTester();
    fireEvent.change(screen.getByTestId('tester-task-input'), {
      target: { value: 'read the file' },
    });
    fireEvent.click(screen.getByTestId('tester-run'));
    // The drill-down mounts inside the result surface (under the verdict).
    const drilldown = await screen.findByTestId('trace-drilldown');
    expect(drilldown).toBeInTheDocument();
    const toggle = screen.getByTestId('trace-step-toggle');
    expect(toggle).toHaveTextContent('Read');
    fireEvent.click(toggle);
    expect(screen.getByTestId('trace-step-input')).toHaveTextContent('Cargo.toml');
    expect(screen.getByTestId('trace-step-output')).toHaveTextContent('[package]');
  });
});
