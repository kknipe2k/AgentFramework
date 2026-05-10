import { useEffect, useRef, useState } from 'react';
import { invokeAbortPlan, invokeApprovePlan, invokeRevisePlan, unwrapCmdError } from '../lib/ipc';
import { useGraphStore, type PlanNodeData } from '../lib/graphStore';

type DraftMode = 'idle' | 'revise' | 'abort';

/**
 * Non-modal ApprovalPanel — spec §3a Approval-gate primitive. Surfaces
 * when any plan in graphStore reaches `awaiting_approval`. Three actions:
 *
 * - **Approve** — dispatches `invokeApprovePlan`; SDK wakes from
 *   `ApprovalSeam::await_approval` and emits `plan_approved`. Panel
 *   dismisses on the resulting state transition (`status` → `in_progress`).
 * - **Revise** — opens an inline textarea; submit dispatches
 *   `invokeRevisePlan(planId, text)`. Free-text passes through opaque
 *   per CLAUDE.md §8.security.
 * - **Cancel plan** — opens an inline textarea for reason; submit
 *   dispatches `invokeAbortPlan(planId, reason)`.
 *
 * **Non-modal:** `aria-modal="false"` per WAI APG dialog pattern (M03.D
 * InspectorPanel discipline) — the graph behind stays interactive. ESC
 * dismisses the panel locally without resolving the seam; the SDK keeps
 * awaiting and the user can return to the plan via state.
 */
export function ApprovalPanel(): JSX.Element | null {
  // Subscribe to the first plan in awaiting_approval. v0.1 single-session
  // per spec §0d means concurrent approval gates aren't expected; if state
  // ever surfaces multiple, render only the first to avoid panel stacking.
  const awaitingPlan = useGraphStore((s) =>
    s.nodes.find(
      (
        n,
      ): n is {
        type: 'plan';
        id: string;
        data: PlanNodeData;
        position: { x: number; y: number };
      } => n.type === 'plan' && n.data.status === 'awaiting_approval',
    ),
  );
  const [mode, setMode] = useState<DraftMode>('idle');
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const planId = awaitingPlan?.data.planId ?? null;

  // Reset local state whenever a new plan becomes the surfaced subject.
  // Prevents a stale draft from a prior plan from leaking into the next.
  useEffect(() => {
    setMode('idle');
    setDraft('');
    setError(null);
    setDismissed(false);
  }, [planId]);

  useEffect(() => {
    if (awaitingPlan && !dismissed && panelRef.current) {
      panelRef.current.focus();
    }
  }, [awaitingPlan, dismissed]);

  // ESC dismisses without aborting (panel-local hide). The SDK keeps
  // awaiting; state change to in_progress / aborted re-resolves visibility.
  useEffect(() => {
    if (!awaitingPlan || dismissed) {
      return undefined;
    }
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        setDismissed(true);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [awaitingPlan, dismissed]);

  if (!awaitingPlan || dismissed || planId === null) {
    return null;
  }
  const data = awaitingPlan.data;

  async function dispatch(action: () => Promise<void>, label: string): Promise<void> {
    try {
      await action();
    } catch (e) {
      console.error(`${label} error:`, e);
      setError(unwrapCmdError(e));
    }
  }

  return (
    <aside
      ref={panelRef}
      className="approval-panel"
      role="region"
      aria-label="Plan approval"
      aria-modal="false"
      tabIndex={-1}
      data-testid="approval-panel"
    >
      <header className="approval-panel__header">
        <h2 className="approval-panel__title">Plan approval</h2>
        <span className="approval-panel__plan-title">{data.title}</span>
      </header>
      <p className="approval-panel__summary">
        {data.taskCount} {data.taskCount === 1 ? 'task' : 'tasks'} planned.
      </p>
      {error !== null && <p className="approval-panel__error">{error}</p>}

      {mode === 'idle' && (
        <div className="approval-panel__actions">
          <button
            type="button"
            className="approval-panel__action approval-panel__action--approve"
            onClick={() => void dispatch(() => invokeApprovePlan(planId), 'approve_plan')}
          >
            Approve
          </button>
          <button
            type="button"
            className="approval-panel__action approval-panel__action--revise"
            onClick={() => {
              setMode('revise');
              setDraft('');
            }}
          >
            Revise
          </button>
          <button
            type="button"
            className="approval-panel__action approval-panel__action--abort"
            onClick={() => {
              setMode('abort');
              setDraft('');
            }}
          >
            Cancel plan
          </button>
        </div>
      )}

      {mode === 'revise' && (
        <form
          className="approval-panel__form"
          onSubmit={(e) => {
            e.preventDefault();
            void dispatch(() => invokeRevisePlan(planId, draft), 'revise_plan');
          }}
        >
          <label className="approval-panel__label">
            <span>Revisions</span>
            <textarea
              className="approval-panel__textarea"
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              rows={4}
              maxLength={2000}
            />
          </label>
          <div className="approval-panel__actions">
            <button
              type="submit"
              className="approval-panel__action approval-panel__action--revise"
              disabled={draft.trim().length === 0}
            >
              Submit revisions
            </button>
            <button
              type="button"
              className="approval-panel__action"
              onClick={() => setMode('idle')}
            >
              Back
            </button>
          </div>
        </form>
      )}

      {mode === 'abort' && (
        <form
          className="approval-panel__form"
          onSubmit={(e) => {
            e.preventDefault();
            void dispatch(() => invokeAbortPlan(planId, draft), 'abort_plan');
          }}
        >
          <label className="approval-panel__label">
            <span>Reason</span>
            <textarea
              className="approval-panel__textarea"
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              rows={3}
              maxLength={2000}
            />
          </label>
          <div className="approval-panel__actions">
            <button
              type="submit"
              className="approval-panel__action approval-panel__action--abort"
              disabled={draft.trim().length === 0}
            >
              Confirm cancel
            </button>
            <button
              type="button"
              className="approval-panel__action"
              onClick={() => setMode('idle')}
            >
              Back
            </button>
          </div>
        </form>
      )}
    </aside>
  );
}
