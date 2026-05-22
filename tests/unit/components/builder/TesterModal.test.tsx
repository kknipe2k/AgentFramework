import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M08.F2 — the Builder Tester modal (spec Phase 9; MVP §M8 criterion 5).
// The modal opens on builderStore's Tester-open state, takes a
// natural-language task, runs the candidate framework through Stage F1's
// `test_framework`, and renders the test run — a smaller graph pane
// SCOPED to the test session + VDR/token/pass-fail surfaces.
//
// Two mocks:
//  1. @xyflow/react — TesterGraphPane mounts a real <ReactFlow>, whose
//     rendering needs a measured pane happy-dom does not provide. The
//     deterministic double renders one testid div per node (the
//     BuilderCanvas.test.tsx precedent) so the scoped-graph assertions
//     are exact.
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
    // The candidate framework crosses the wire straight from the canvas
    // (spec Phase 9 — no disk round-trip).
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
    // The violation renders as a failure LINE (F1.3.3 — never a HITL
    // prompt; F1's test-defaults HitlSeam never prompts).
    const failures = await screen.findByTestId('tester-capability-failures');
    expect(failures).toHaveTextContent('worker');
    expect(failures).toHaveTextContent('read');
    expect(failures).toHaveTextContent('declared scope `none`');
    // No HITL surface is ever raised by a test run.
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
    // `timing` crosses the wire as serde's Duration shape { secs, nanos }
    // — 1s + 250ms folds to 1250 ms.
    expect(tokens).toHaveTextContent('1250 ms');
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
    fireEvent.click(screen.getByTestId('tester-close'));
    // closeTester flips the builderStore flag — the modal unmounts.
    expect(useBuilderStore.getState().testerOpen).toBe(false);
    expect(screen.queryByTestId('tester-modal')).toBeNull();
    // The outcome is dropped — a re-open shows no stale result.
    openTester();
    expect(screen.queryByTestId('tester-result')).toBeNull();
  });

  it('a_failed_test_framework_call_surfaces_the_error', async () => {
    // A test_framework infrastructure failure crosses as a CmdError-shape
    // object — unwrapCmdError renders it, not String(e) (gotcha #30).
    testFrameworkMock.mockRejectedValue({ type: 'internal', message: 'drone spawn failed' });
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    const err = await screen.findByTestId('tester-error');
    expect(err).toHaveTextContent('drone spawn failed');
    // A failed call leaves no result surface.
    expect(screen.queryByTestId('tester-result')).toBeNull();
  });

  it('survives_repeated_open_close_cycles_with_a_fresh_run_each', async () => {
    // gotcha #66 contract test — the modal must be re-runnable: each
    // open/run/close cycle starts from a clean slate.
    testFrameworkMock.mockResolvedValue(failOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('first task');
    fireEvent.click(screen.getByTestId('tester-run'));
    expect(await screen.findByTestId('tester-result-verdict')).toHaveTextContent('FAIL');
    fireEvent.click(screen.getByTestId('tester-close'));

    testFrameworkMock.mockResolvedValue(passOutcome());
    openTester();
    // The prior run's result did not leak into the fresh open.
    expect(screen.queryByTestId('tester-result')).toBeNull();
    typeTask('second task');
    fireEvent.click(screen.getByTestId('tester-run'));
    expect(await screen.findByTestId('tester-result-verdict')).toHaveTextContent('PASS');
    expect(testFrameworkMock).toHaveBeenCalledTimes(2);
  });

  it('formatTiming_folds_the_serde_duration_shape_to_milliseconds', () => {
    // serde serializes a Rust Duration as { secs, nanos } — NOT a bare
    // ms count. The helper folds both parts.
    expect(_testing.formatTiming({ secs: 0, nanos: 5_000_000 })).toBe('5 ms');
    expect(_testing.formatTiming({ secs: 2, nanos: 340_000_000 })).toBe('2340 ms');
    expect(_testing.formatTiming({ secs: 0, nanos: 0 })).toBe('0 ms');
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
    // The trace's session_start + agent_spawned reduce into the scoped
    // graph; the smaller pane renders both nodes.
    expect(await screen.findByTestId('rf-node-agent:worker')).toBeInTheDocument();
    expect(screen.getByTestId('rf-node-framework:tester-fixture')).toBeInTheDocument();
  });

  it('running_a_test_does_not_write_to_the_live_useGraphStore_singleton', async () => {
    // THE load-bearing scoping invariant: a test run reduces its trace
    // into the SCOPED store only; the live runtime graph is untouched.
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
    fireEvent.click(screen.getByTestId('tester-close'));
    // Discard-on-close — the scoped graph is cleared.
    expect(useTestGraphStore.getState().nodes).toHaveLength(0);
  });

  it('promote_writes_the_test_run_into_the_live_graph_store', async () => {
    // The explicit Save/Promote affordance is the ONLY persist path
    // (spec Phase 9 "full graph available by promoting test session to
    // main"). Promote replays the trace into the live useGraphStore.
    testFrameworkMock.mockResolvedValue(passOutcome());
    render(<TesterModal />);
    openTester();
    typeTask('a task');
    fireEvent.click(screen.getByTestId('tester-run'));
    await screen.findByTestId('tester-result');
    expect(useGraphStore.getState().nodes).toHaveLength(0);
    fireEvent.click(screen.getByTestId('tester-promote'));
    expect(useGraphStore.getState().nodes.some((n) => n.id === 'agent:worker')).toBe(true);
    // Promote closes the Tester (the run is now the main session).
    expect(useBuilderStore.getState().testerOpen).toBe(false);
  });
});

// gotcha #67 — a className with no styles.css rule renders unstyled and
// the user sees nothing. Every Tester class M08.F2 introduces must have
// a corresponding rule, and use --node-* theme tokens (M07-IRL #3).
describe('M08.F2 Tester modal styles (gotcha #67)', () => {
  const css = readFileSync(resolve(__dirname, '../../../../src/styles.css'), 'utf8');
  const F2_CLASSES = [
    'tester-modal',
    'tester-modal__header',
    'tester-modal__close',
    'tester-modal__body',
    'tester-modal__task-input',
    'tester-modal__run',
    'tester-modal__error',
    'tester-graph-pane',
    'tester-result',
    'tester-result--pass',
    'tester-result--fail',
    'tester-result__verdict',
    'tester-capability-failures',
    'tester-capability-failure',
    'tester-result__tokens',
    'tester-result__vdr',
    'tester-result__actions',
    'tester-promote',
  ] as const;

  it.each(F2_CLASSES)('styles.css defines a rule for .%s', (cls) => {
    expect(css).toMatch(new RegExp(`\\.${cls}[\\s,{]`));
  });

  it('Tester modal styles use theme variables, not literal colors (M07-IRL #3)', () => {
    expect(css).toMatch(/\.tester-modal[\s\S]*?var\(--node-/);
  });
});
