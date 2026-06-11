import { useState } from 'react';
import { useBuilderStore, useTestGraphStore } from '../../lib/builderStore';
import { useGraphStore } from '../../lib/graphStore';
import {
  isSetupRequired,
  requestTierTransition,
  testFramework,
  unwrapCmdError,
  type TestOutcome,
  type TestVerdict,
  type WireDuration,
} from '../../lib/ipc';
import { refreshHasKey } from '../../lib/keyState';
import { InspectorPanel } from '../InspectorPanel';
import { MetricCard } from '../MetricCard';
import { Modal } from '../Modal';
import { TesterGraphPane } from './TesterGraphPane';
import { TraceDrilldown } from './TraceDrilldown';

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
 * The verdict's UI presentation — the 3-state truth (TD-047). Distinct
 * from `outcome.passed` (the framework-defect bool): a `tier_limited` run
 * has `passed === true` but must read TIER-LIMITED, never a clean PASS
 * (ADR-0030 — a tier block is the user's setting, not a defect).
 */
const VERDICT_PRESENTATION: Record<
  TestVerdict,
  { label: string; modifier: string; tone: 'ok' | 'bad' | 'default' }
> = {
  pass: { label: 'PASS', modifier: 'pass', tone: 'ok' },
  fail: { label: 'FAIL', modifier: 'fail', tone: 'bad' },
  tier_limited: { label: 'TIER-LIMITED', modifier: 'tier-limited', tone: 'default' },
};

/**
 * The result surface — the 3-state verdict (Pass / Fail / Tier-limited;
 * TD-047), capability failures as test-failure lines (F1.3.3 — never HITL
 * prompts), tier blocks as a "promote to exercise" affordance, token spend
 * + timing, and the VDR record. "Promote to main session" is the only
 * persist path (spec Phase 9).
 */
function TesterResult({
  outcome,
  onPromote,
}: {
  outcome: TestOutcome;
  onPromote: () => void;
}): JSX.Element {
  const verdict = VERDICT_PRESENTATION[outcome.verdict];
  // M09.D.fix iter2 (DESIGN.md principle 3): the verdict + metrics stay
  // visible; the detail (trace drill-down + tokens + VDR) collapses behind a
  // disclosure toggle. Default-open after a run.
  const [detailOpen, setDetailOpen] = useState(true);
  return (
    <section
      className={`tester-result tester-result--${verdict.modifier}`}
      data-testid="tester-result"
    >
      <header className="tester-result__verdict" data-testid="tester-result-verdict">
        {verdict.label}
      </header>
      {/* The mockup's metric grid (M08.8.B.fix) — Result / Verify / Tokens /
          Spend in the mono-tabular instrument register. Spend has no wire
          field yet (a stub dash; F wires the real cost). */}
      <div className="metrics" data-testid="tester-metrics">
        <MetricCard label="Result" value={verdict.label} tone={verdict.tone} />
        <MetricCard label="Verify" value={outcome.vdr !== null ? 'OK' : '—'} />
        <MetricCard
          label="Tokens"
          value={String(outcome.token_spend.total)}
          delta={`in ${outcome.token_spend.input} · out ${outcome.token_spend.output}`}
        />
        <MetricCard label="Spend" value="—" />
      </div>
      {/* §8.security L4 tier blocks (TD-047): the framework is fine, but the
          user's tier forbade these actions. Surfaced distinctly from a
          framework defect — with a "Promote?" affordance to lift the tier
          (the SettingsPanel `request_tier_transition` path) so the user can
          re-run and exercise the blocked actions. */}
      {outcome.tier_blocks.length > 0 && (
        <div className="tester-tier-blocks" data-testid="tester-tier-blocks">
          <p className="tester-tier-blocks__lead">
            This run was tier-limited — your current tier forbade{' '}
            {outcome.tier_blocks.length === 1 ? 'an action' : 'these actions'}:
          </p>
          <ul className="tester-tier-blocks__list">
            {outcome.tier_blocks.map((block, i) => (
              <li key={i} className="tester-tier-block">
                <code>{block.agent_id}</code> — {block.kind}: {block.attempted_action}
              </li>
            ))}
          </ul>
          <button
            type="button"
            className="tester-tier-promote"
            data-testid="tester-tier-promote"
            onClick={() => {
              void requestTierTransition(
                'promoted',
                'user promoted from a tier-limited Tester run',
              ).catch((e) => {
                console.error('request_tier_transition error:', e);
              });
            }}
          >
            Promote?
          </button>
        </div>
      )}
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
      {/* M09.D.fix iter2: the results DETAIL (drill-down + tokens + VDR)
          collapses behind a disclosure toggle (DESIGN.md principle 3) — the
          verdict + metrics above stay visible. */}
      <button
        type="button"
        className="tester-result__toggle"
        data-testid="tester-result-toggle"
        aria-expanded={detailOpen}
        onClick={() => setDetailOpen((open) => !open)}
      >
        {detailOpen ? 'Hide run detail' : 'Show run detail'}
      </button>
      {detailOpen && (
        <div className="tester-result__detail" data-testid="tester-result-detail">
          {/* The run drill-down (M08.9.B): verdict → per-tool-call
              input/result → raw, over outcome.trace. Pure disclosure — reuses
              the M08.8.A payload formatter + the shared Show-raw disclosure. */}
          <TraceDrilldown trace={outcome.trace} />
          <div className="tester-result__tokens" data-testid="tester-result-tokens">
            in {outcome.token_spend.input} · out {outcome.token_spend.output} · total{' '}
            {outcome.token_spend.total} · {formatTiming(outcome.timing)}
          </div>
          <pre className="tester-result__vdr" data-testid="tester-result-vdr">
            {JSON.stringify(outcome.vdr, null, 2)}
          </pre>
        </div>
      )}
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
  // M09.D.fix: the watch pane is expandable so a run is observable in a usable
  // window (DESIGN.md Modals: content scrolls within a bounded height; the
  // M09.D IRL re-verify needs the run pane growable).
  const [watchExpanded, setWatchExpanded] = useState(false);

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
    // M09.5.F (honest key chip): same contract as App.handleSmoke — a
    // SetupRequired failure flips the shared chip state false off the
    // run loop's own read, sticky (no re-poll may override it); every
    // other settled run re-polls has_api_key.
    let setupRequiredSeen = false;
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
      if (isSetupRequired(e)) {
        setupRequiredSeen = true;
        useGraphStore.getState().setHasKey(false);
      }
      setError(unwrapCmdError(e));
    } finally {
      setRunning(false);
      if (!setupRequiredSeen) {
        void refreshHasKey();
      }
    }
  };

  const handleClose = (): void => {
    // Discard-on-close (spec Phase 9): drop the TestOutcome, the task,
    // and the error; `closeTester` clears the scoped graph. Nothing is
    // persisted — F1's backend already deleted the throwaway test DB.
    setOutcome(null);
    setTask('');
    setError(null);
    setWatchExpanded(false);
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
    <Modal
      open={isOpen}
      onClose={handleClose}
      title="Test framework"
      size="full"
      testId="tester-modal"
    >
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
            observable in-app, not only via RUST_LOG). M09.D.fix: an expand
            toggle grows the pane for a usable re-verify window. */}
        <div className="tester-modal__watch-bar">
          <button
            type="button"
            className="tester-modal__expand"
            data-testid="tester-expand"
            aria-pressed={watchExpanded}
            onClick={() => setWatchExpanded((expanded) => !expanded)}
          >
            {watchExpanded ? 'Collapse run view' : 'Expand run view'}
          </button>
        </div>
        <div
          className={`tester-modal__watch${watchExpanded ? ' tester-modal__watch--expanded' : ''}`}
          data-testid="tester-watch"
        >
          <TesterGraphPane />
          <InspectorPanel store={useTestGraphStore} />
        </div>
        {outcome !== null && <TesterResult outcome={outcome} onPromote={handlePromote} />}
      </div>
    </Modal>
  );
}

export const _testing = { formatTiming };
