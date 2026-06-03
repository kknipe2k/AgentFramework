import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M08.F2 — the Builder Tester modal (spec Phase 9; MVP §M8 criterion 5).
// M08.8.B.fix — MIGRATED onto the reusable Modal primitive (TD-043): the
// hand-rolled `.tester-modal` shell is gone; the Tester is now
// `<Modal size="full">`. This rewrite rides the RED commit (the structural
// migration is test-bearing — the old test pinned `tester-close` /
// `tester-modal__*` classes a pure repaint could not change). The
// behavioral contract (run / pass-fail / scoped graph / promote) is
// unchanged; only the shell + close affordance moved onto Modal.
//
// Two mocks:
//  1. @xyflow/react — TesterGraphPane mounts a real <ReactFlow>, whose
//     rendering needs a measured pane happy-dom does not provide. The
//     deterministic double renders one testid div per node.
//  2. ../../../../src/lib/ipc — `testFramework` is replaced; the rest
//     (unwrapCmdError) stays real so the error path is genuinely
//     exercised (gotcha #30).
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

import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { TesterModal, _testing } from '../../../../src/components/builder/TesterModal';
import { useBuilderStore, useTestGraphStore } from '../../../../src/lib/builderStore';
import { useGraphStore } from '../../../../src/lib/graphStore';
import type { TestOutcome } from '../../../../src/lib/ipc';
import type { AgentEvent } from '../../../../src/types/agent_event';

// ── Fixtures ────────────────────────────────────────────────────────

const SESSION_START: AgentEvent = {
  type: 'session_start',
  session_id: 's-tester-1',
  framework: 'tester-fixture',
  model: 'claude-haiku-4-5',
};
const AGENT_SPAWNED: AgentEvent = {
  type: 'agent_spawned',
  agent_id: 'worker',
  agent_name: 'worker',
  session_id: 's-tester-1',
};
const CAPABILITY_VIOLATION: AgentEvent = {
  type: 'capability_violation',
  agent_id: 'worker',
  capability_kind: 'read',
  declared_scope: 'none',
  requested_action: 'read /etc/passwd',
};

/** A clean run — `passed: true`, a two-node trace, a non-null VDR. */
function passOutcome(): TestOutcome {
  return {
    passed: true,
    capability_failures: [],
    token_spend: { input: 120, output: 45, total: 165 },
    timing: { secs: 1, nanos: 250_000_000 },
    vdr: { decision: 'ok' },
    trace: [SESSION_START, AGENT_SPAWNED],
  };
}

/** A failed run — a §8.security L2 capability violation folded onto
 *  `capability_failures` (F1.3.3: a test failure, never a HITL prompt). */
function failOutcome(): TestOutcome {
  return {
    passed: false,
    capability_failures: [
      {
        agent_id: 'worker',
        needed: 'read',
        reason: 'requested `read /etc/passwd` — declared scope `none`',
      },
    ],
    token_spend: { input: 80, output: 10, total: 90 },
    timing: { secs: 0, nanos: 500_000_000 },
    vdr: null,
    trace: [SESSION_START, AGENT_SPAWNED, CAPABILITY_VIOLATION],
  };
}

function openTester(): void {
  act(() => {
    useBuilderStore.getState().openTester();
  });
}

function typeTask(text: string): void {
  fireEvent.change(screen.getByTestId('tester-task-input'), { target: { value: text } });
}

/** Close via the Modal primitive's × button (the migration replaced the
 *  hand-rolled `tester-close` button with Modal's aria-labelled Close). */
function closeViaModal(): void {
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
}

describe('TesterModal', () => {
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

  it('does_not_render_when_tester_open_state_is_false', () => {
    const { container } = render(<TesterModal />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByTestId('tester-modal')).toBeNull();
  });

  it('renders_when_builderStore_tester_open_state_is_set', () => {
    render(<TesterModal />);
    openTester();
    const modal = screen.getByTestId('tester-modal');
    expect(modal).toBeInTheDocument();
    expect(modal).toHaveAttribute('role', 'dialog');
  });

  it('run_button_disabled_when_task_is_empty', () => {
    render(<TesterModal />);
    openTester();
    expect(screen.getByTestId('tester-run')).toBeDisabled();
    typeTask('summarize the input');
    expect(screen.getByTestId('tester-run')).toBeEnabled();
  });

  it('run_calls_testFramework_with_builderStore_framework_and_the_task', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('summarize the input');
    fireEvent.click(screen.getByTestId('tester-run'));
    await waitFor(() => expect(testFrameworkMock).toHaveBeenCalledTimes(1));
    expect(testFrameworkMock).toHaveBeenCalledWith(
      useBuilderStore.getState().framework,
      'summarize the input',
    );
  });

  it('renders_pass_result_when_outcome_passed_is_true', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    const verdict = await screen.findByTestId('tester-result-verdict');
    expect(verdict).toHaveTextContent('PASS');
    expect(screen.getByTestId('tester-result')).toHaveClass('tester-result--pass');
  });

  it('renders_fail_result_when_outcome_passed_is_false', async () => {
    testFrameworkMock.mockResolvedValue(failOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    const verdict = await screen.findByTestId('tester-result-verdict');
    expect(verdict).toHaveTextContent('FAIL');
    expect(screen.getByTestId('tester-result')).toHaveClass('tester-result--fail');
  });

  it('renders_capability_failures_as_test_failure_lines_not_hitl_prompts', async () => {
    testFrameworkMock.mockResolvedValue(failOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('read a file');
    fireEvent.click(screen.getByTestId('tester-run'));
    const failures = await screen.findByTestId('tester-capability-failures');
    expect(failures).toHaveTextContent('worker');
    expect(failures).toHaveTextContent('read');
    expect(failures).toHaveTextContent('declared scope `none`');
    expect(screen.queryByTestId('hitl-modal')).toBeNull();
  });

  it('renders_token_spend_in_out_total_and_timing', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    const tokens = await screen.findByTestId('tester-result-tokens');
    expect(tokens).toHaveTextContent('in 120');
    expect(tokens).toHaveTextContent('out 45');
    expect(tokens).toHaveTextContent('total 165');
    expect(tokens).toHaveTextContent('1250 ms');
  });

  it('renders_the_result_metric_cards_result_and_tokens', async () => {
    // M08.8.B.fix — the mockup's `.metric` grid (Result / Verify / Tokens /
    // Spend). Styled here from the real TestOutcome where available.
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    expect(await screen.findByTestId('metric-result')).toHaveTextContent('PASS');
    expect(screen.getByTestId('metric-tokens')).toHaveTextContent('165');
  });

  it('renders_the_vdr_record', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    const vdr = await screen.findByTestId('tester-result-vdr');
    expect(vdr).toHaveTextContent('"decision": "ok"');
  });

  it('close_clears_the_outcome_and_calls_closeTester', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    await screen.findByTestId('tester-result');
    closeViaModal();
    expect(useBuilderStore.getState().testerOpen).toBe(false);
    expect(screen.queryByTestId('tester-modal')).toBeNull();
    openTester();
    expect(screen.queryByTestId('tester-result')).toBeNull();
  });

  it('a_failed_test_framework_call_surfaces_the_error', async () => {
    testFrameworkMock.mockRejectedValue({ type: 'internal', message: 'drone spawn failed' });
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    const err = await screen.findByTestId('tester-error');
    expect(err).toHaveTextContent('drone spawn failed');
    expect(screen.queryByTestId('tester-result')).toBeNull();
  });

  it('survives_repeated_open_close_cycles_with_a_fresh_run_each', async () => {
    testFrameworkMock.mockResolvedValue(failOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('first task');
    fireEvent.click(screen.getByTestId('tester-run'));
    expect(await screen.findByTestId('tester-result-verdict')).toHaveTextContent('FAIL');
    closeViaModal();

    testFrameworkMock.mockResolvedValue(passOutcome());
    openTester();
    expect(screen.queryByTestId('tester-result')).toBeNull();
    typeTask('second task');
    fireEvent.click(screen.getByTestId('tester-run'));
    expect(await screen.findByTestId('tester-result-verdict')).toHaveTextContent('PASS');
    expect(testFrameworkMock).toHaveBeenCalledTimes(2);
  });

  it('formatTiming_folds_the_serde_duration_shape_to_milliseconds', () => {
    expect(_testing.formatTiming({ secs: 0, nanos: 5_000_000 })).toBe('5 ms');
    expect(_testing.formatTiming({ secs: 2, nanos: 340_000_000 })).toBe('2340 ms');
    expect(_testing.formatTiming({ secs: 0, nanos: 0 })).toBe('0 ms');
  });
});

// ── TD-043: the Tester IS a Modal (migrated off the hand-rolled shell) ──

describe('TesterModal — migrated onto the Modal primitive (TD-043)', () => {
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

  it('renders_as_a_full_size_modal_dialog', () => {
    render(<TesterModal />);
    openTester();
    const dialog = screen.getByTestId('tester-modal');
    expect(dialog).toHaveAttribute('role', 'dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    // size="full" — the near-full-screen candidate-run surface.
    expect(dialog).toHaveClass('modal--full');
  });

  it('closes_on_escape', () => {
    render(<TesterModal />);
    openTester();
    expect(screen.getByTestId('tester-modal')).toBeInTheDocument();
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(useBuilderStore.getState().testerOpen).toBe(false);
    expect(screen.queryByTestId('tester-modal')).toBeNull();
  });

  it('closes_on_scrim_click', () => {
    render(<TesterModal />);
    openTester();
    fireEvent.mouseDown(screen.getByTestId('modal-scrim'));
    expect(useBuilderStore.getState().testerOpen).toBe(false);
    expect(screen.queryByTestId('tester-modal')).toBeNull();
  });
});

// ── The scoped test-session graph (the load-bearing F2 invariant) ────

describe('TesterModal — the scoped test-session graph', () => {
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

  it('tester_graph_pane_renders_the_test_session_nodes', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    expect(await screen.findByTestId('rf-node-agent:worker')).toBeInTheDocument();
    expect(screen.getByTestId('rf-node-framework:tester-fixture')).toBeInTheDocument();
  });

  it('running_a_test_does_not_write_to_the_live_useGraphStore_singleton', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    await screen.findByTestId('tester-result');
    expect(useGraphStore.getState().nodes).toHaveLength(0);
    expect(useTestGraphStore.getState().nodes.length).toBeGreaterThan(0);
  });

  it('closing_the_modal_drops_the_scoped_graph_instance', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    await screen.findByTestId('tester-result');
    expect(useTestGraphStore.getState().nodes.length).toBeGreaterThan(0);
    closeViaModal();
    expect(useTestGraphStore.getState().nodes).toHaveLength(0);
  });

  it('promote_writes_the_test_run_into_the_live_graph_store', async () => {
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    await screen.findByTestId('tester-result');
    expect(useGraphStore.getState().nodes).toHaveLength(0);
    fireEvent.click(screen.getByTestId('tester-promote'));
    expect(useGraphStore.getState().nodes.some((n) => n.id === 'agent:worker')).toBe(true);
    expect(useBuilderStore.getState().testerOpen).toBe(false);
  });
});
