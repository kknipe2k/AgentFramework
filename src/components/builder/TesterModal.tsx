import { useState } from 'react';
import { useBuilderStore, useTestGraphStore } from '../../lib/builderStore';
import { useGraphStore } from '../../lib/graphStore';
import { testFramework, unwrapCmdError, type TestOutcome, type WireDuration } from '../../lib/ipc';
import { InspectorPanel } from '../InspectorPanel';
import { TesterGraphPane } from './TesterGraphPane';

const MS_PER_SEC = 1000;
const NANOS_PER_MS = 1_000_000;

/**
 * Fold serde's `Duration` wire shape (`{ secs, nanos }`) to a
 * millisecond label. `TestOutcome.timing` crosses the Tauri bridge as a
 * struct — a Rust `Duration` `#[derive(Serialize)]`s to `{ secs, nanos }`,
 * NOT a bare millisecond count.
 */
function formatTiming(duration: WireDuration): string {
  const ms = duration.secs * MS_PER_SEC + Math.round(duration.nanos / NANOS_PER_MS);
  return `${ms} ms`;
}

/**
 * The result surface — pass/fail verdict, capability failures as
 * test-failure lines (F1.3.3 — never HITL prompts), token spend +
 * timing, and the VDR record. The Promote button is the only persist
 * path (spec Phase 9).
 */
function TesterResult({
  outcome,
  onPromote,
}: {
  outcome: TestOutcome;
  onPromote: () => void;
}): JSX.Element {
  return (
    <section
      className={`tester-result tester-result--${outcome.passed ? 'pass' : 'fail'}`}
      data-testid="tester-result"
    >
      <header className="tester-result__verdict" data-testid="tester-result-verdict">
        {outcome.passed ? 'PASS' : 'FAIL'}
      </header>
      {/* §8.security L2 violations surface as test-failure lines, never
          as HITL prompts (F1.3.3 — the test-defaults HitlSeam never
          prompted; capability failures arrive folded onto TestOutcome). */}
      {outcome.capability_failures.length > 0 && (
        <ul className="tester-capability-failures" data-testid="tester-capability-failures">
          {outcome.capability_failures.map((failure, i) => (
            <li key={i} className="tester-capability-failure">
              <code>{failure.agent_id}</code> — {failure.needed}: {failure.reason}
            </li>
          ))}
        </ul>
      )}
      <div className="tester-result__tokens" data-testid="tester-result-tokens">
        in {outcome.token_spend.input} · out {outcome.token_spend.output} · total{' '}
        {outcome.token_spend.total} · {formatTiming(outcome.timing)}
      </div>
      <pre className="tester-result__vdr" data-testid="tester-result-vdr">
        {JSON.stringify(outcome.vdr, null, 2)}
      </pre>
      <div className="tester-result__actions">
        <button
          type="button"
          className="tester-promote"
          data-testid="tester-promote"
          onClick={onPromote}
        >
          Promote to main session
        </button>
      </div>
    </section>
  );
}

/**
 * The Builder's Tester modal (spec Phase 9; MVP §M8 criterion 5;
 * ADR-0019). Opens on `builderStore`'s Tester-open state (Stage E's
 * Inspector Test button). Takes a natural-language task, runs the
 * candidate framework through Stage F1's `test_framework`, and renders
 * the run — a smaller graph pane SCOPED to the test session + the
 * VDR/token/pass-fail surfaces.
 *
 * Discard-on-close is the default: closing drops the `TestOutcome` and
 * `closeTester` clears the scoped graph. The explicit "Promote to main
 * session" affordance is the only path that persists anything — it
 * replays the run's trace into the live `useGraphStore` (spec Phase 9:
 * "full graph available by promoting test session to main").
 */
export function TesterModal(): JSX.Element | null {
  const isOpen = useBuilderStore((s) => s.testerOpen);
  const framework = useBuilderStore((s) => s.framework);
  const closeTester = useBuilderStore((s) => s.closeTester);
  const [task, setTask] = useState('');
  const [outcome, setOutcome] = useState<TestOutcome | null>(null);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Stage E ships `openTester`; F2 renders the modal on that state.
  if (!isOpen) {
    return null;
  }

  const handleRun = async (): Promise<void> => {
    setRunning(true);
    setError(null);
    setOutcome(null);
    // The scoped test-session graph is rebuilt per run — clear it so a
    // re-run does not stack onto the prior run's nodes.
    useTestGraphStore.getState().clear();
    try {
      // The candidate framework crosses the wire straight from the
      // canvas (spec Phase 9 — "does NOT need to save first").
      const result = await testFramework(framework, task);
      // Reduce the returned trace into the SCOPED store — never the live
      // useGraphStore singleton (the load-bearing F2 invariant). The
      // shipped F1 `test_framework` returns the full trace in
      // TestOutcome; it emits no live `agent_event`s.
      const applyToScopedGraph = useTestGraphStore.getState().applyEvent;
      for (const event of result.trace) {
        applyToScopedGraph(event);
      }
      setOutcome(result);
    } catch (e) {
      // A test_framework infrastructure failure crosses as a
      // CmdError-shape object — unwrapCmdError renders it (gotcha #30).
      setError(unwrapCmdError(e));
    } finally {
      setRunning(false);
    }
  };

  const handleClose = (): void => {
    // Discard-on-close (spec Phase 9): drop the TestOutcome, the task,
    // and the error; `closeTester` clears the scoped graph. Nothing is
    // persisted — F1's backend already deleted the throwaway test DB.
    setOutcome(null);
    setTask('');
    setError(null);
    closeTester();
  };

  const handlePromote = (): void => {
    // The explicit Save/Promote affordance — the ONLY persist path
    // (spec Phase 9: "full graph available by promoting test session to
    // main"). Replay the run's trace into the LIVE useGraphStore, then
    // close the Tester (the run is now the main session).
    if (outcome === null) {
      return;
    }
    useGraphStore.getState().clear();
    const applyToLiveGraph = useGraphStore.getState().applyEvent;
    for (const event of outcome.trace) {
      applyToLiveGraph(event);
    }
    handleClose();
  };

  return (
    <div className="tester-modal" role="dialog" aria-label="Tester" data-testid="tester-modal">
      <header className="tester-modal__header">
        <h2>Test framework</h2>
        <button
          type="button"
          className="tester-modal__close"
          data-testid="tester-close"
          aria-label="Close tester"
          onClick={handleClose}
        >
          ×
        </button>
      </header>
      <div className="tester-modal__body">
        <textarea
          className="tester-modal__task-input"
          data-testid="tester-task-input"
          placeholder="Describe a task for the test session…"
          value={task}
          onChange={(e) => setTask(e.target.value)}
        />
        <button
          type="button"
          className="tester-modal__run"
          data-testid="tester-run"
          disabled={running || task.trim() === ''}
          onClick={() => void handleRun()}
        >
          {running ? 'Running…' : 'Run'}
        </button>
        {error !== null && (
          <p className="tester-modal__error" data-testid="tester-error" role="alert">
            {error}
          </p>
        )}
        {/* The smaller graph pane — scoped to the test session — beside
            the Output/Inspector rail bound to the SAME scoped store
            (M08.8.A: a Tester run's agent text + tool payloads are
            observable in-app, not only via RUST_LOG). */}
        <div className="tester-modal__watch">
          <TesterGraphPane />
          <InspectorPanel store={useTestGraphStore} />
        </div>
        {outcome !== null && <TesterResult outcome={outcome} onPromote={handlePromote} />}
      </div>
    </div>
  );
}

export const _testing = { formatTiming };
