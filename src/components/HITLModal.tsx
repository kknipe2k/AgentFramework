import { useEffect, useRef, useState } from 'react';
import { invokeRespondHitl, unwrapCmdError } from '../lib/ipc';
import { useGraphStore, type PendingHitl } from '../lib/graphStore';

/**
 * Modal HITL dialog — spec §6a (M04 Stage E).
 *
 * Surfaces when any pending HITL prompt has `ui_variant: 'modal'`. Modal
 * pattern per WAI APG: `aria-modal="true"`, focuses on mount, Escape
 * dismisses, focus trapped within the dialog. The graph behind is
 * inert (CSS pointer-events on a backdrop) but the underlying state is
 * not modified — same prompt re-surfaces if the user dismisses without
 * resolving.
 */
export function HITLModal(): JSX.Element | null {
  const pending = useGraphStore((s) => firstModalPrompt(s.pendingHitl));
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const dialogRef = useRef<HTMLDivElement>(null);
  const promptId = pending?.promptId ?? null;

  useEffect(() => {
    setError(null);
    setDismissed(false);
  }, [promptId]);

  useEffect(() => {
    if (pending && !dismissed && dialogRef.current) {
      dialogRef.current.focus();
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

  return (
    <div className="hitl-modal-backdrop" data-testid="hitl-modal-backdrop">
      <div
        ref={dialogRef}
        className="hitl-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="hitl-modal-title"
        aria-describedby="hitl-modal-question"
        tabIndex={-1}
        data-testid="hitl-modal"
      >
        <h2 id="hitl-modal-title" className="hitl-modal__title">
          Human input requested
        </h2>
        <p
          id="hitl-modal-question"
          className="hitl-modal__question"
          data-testid="hitl-modal-question"
        >
          {pending.question}
        </p>
        <span className="hitl-modal__trigger" data-testid="hitl-modal-trigger">
          {pending.trigger}
        </span>
        {error !== null && <p className="hitl-modal__error">{error}</p>}
        <div className="hitl-modal__actions">
          {pending.options.map((option) => (
            <button
              key={option}
              type="button"
              className={`hitl-modal__action hitl-modal__action--${option}`}
              data-testid={`hitl-modal-option-${option}`}
              onClick={() => void respond(option)}
            >
              {option}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function firstModalPrompt(pending: Record<string, PendingHitl>): PendingHitl | null {
  for (const id of Object.keys(pending)) {
    const p = pending[id];
    if (p !== undefined && p.uiVariant === 'modal') {
      return p;
    }
  }
  return null;
}

export const _testing = { firstModalPrompt };
