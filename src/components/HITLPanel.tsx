import { useEffect, useRef, useState } from 'react';
import { invokeRespondHitl, unwrapCmdError } from '../lib/ipc';
import { useGraphStore, type PendingHitl } from '../lib/graphStore';

/**
 * Non-modal HITLPanel — spec §6a (M04 Stage E).
 *
 * Surfaces when any pending HITL prompt has `ui_variant: 'panel'`. Same
 * non-modal pattern as Stage C's ApprovalPanel: `aria-modal="false"`, the
 * graph behind stays interactive, ESC dismisses the panel locally without
 * resolving the seam.
 *
 * Each option becomes a button that dispatches `respond_hitl(prompt_id,
 * choice)`. Free-text response (when `options` is empty) renders a
 * textarea + submit button.
 *
 * Per WAI APG: `aria-modal="false"` + `role="region"` + an explicit
 * heading. The panel is keyboard-focusable so screenreaders land on it.
 */
export function HITLPanel(): JSX.Element | null {
  const pending = useGraphStore((s) => firstPanelPrompt(s.pendingHitl));
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const promptId = pending?.promptId ?? null;

  useEffect(() => {
    setDraft('');
    setError(null);
    setDismissed(false);
  }, [promptId]);

  useEffect(() => {
    if (pending && !dismissed && panelRef.current) {
      panelRef.current.focus();
    }
  }, [pending, dismissed]);

  useEffect(() => {
    if (!pending || dismissed) {
      return undefined;
    }
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        setDismissed(true);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [pending, dismissed]);

  if (!pending || dismissed || promptId === null) {
    return null;
  }

  async function respond(choice: string): Promise<void> {
    try {
      await invokeRespondHitl(promptId as string, choice);
    } catch (e) {
      console.error('respond_hitl error:', e);
      setError(unwrapCmdError(e));
    }
  }

  const hasOptions = pending.options.length > 0;

  return (
    <aside
      ref={panelRef}
      className="hitl-panel"
      role="region"
      aria-label={`HITL prompt — ${pending.trigger}`}
      aria-modal="false"
      tabIndex={-1}
      data-testid="hitl-panel"
    >
      <header className="hitl-panel__header">
        <h2 className="hitl-panel__title">Human input requested</h2>
        <span className="hitl-panel__trigger" data-testid="hitl-panel-trigger">
          {pending.trigger}
        </span>
      </header>
      <p className="hitl-panel__question" data-testid="hitl-panel-question">
        {pending.question}
      </p>
      {error !== null && <p className="hitl-panel__error">{error}</p>}

      {hasOptions && (
        <div className="hitl-panel__actions">
          {pending.options.map((option) => (
            <button
              key={option}
              type="button"
              className={`hitl-panel__action hitl-panel__action--${option}`}
              data-testid={`hitl-panel-option-${option}`}
              onClick={() => void respond(option)}
            >
              {option}
            </button>
          ))}
        </div>
      )}

      {!hasOptions && (
        <form
          className="hitl-panel__form"
          onSubmit={(e) => {
            e.preventDefault();
            void respond(draft);
          }}
        >
          <label className="hitl-panel__label">
            <span>Response</span>
            <textarea
              className="hitl-panel__textarea"
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              rows={4}
              maxLength={2000}
              data-testid="hitl-panel-textarea"
            />
          </label>
          <button
            type="submit"
            className="hitl-panel__action"
            disabled={draft.trim().length === 0}
            data-testid="hitl-panel-submit"
          >
            Submit
          </button>
        </form>
      )}
    </aside>
  );
}

function firstPanelPrompt(pending: Record<string, PendingHitl>): PendingHitl | null {
  for (const id of Object.keys(pending)) {
    const p = pending[id];
    if (p !== undefined && p.uiVariant === 'panel') {
      return p;
    }
  }
  return null;
}

export const _testing = { firstPanelPrompt };
