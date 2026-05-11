import { useEffect, useState } from 'react';
import { invokeRespondHitl, unwrapCmdError } from '../lib/ipc';
import { useGraphStore, type PendingHitl } from '../lib/graphStore';

const TOAST_AUTO_DISMISS_MS = 30_000;

/**
 * Non-blocking auto-dismiss HITL Toast — spec §6a (M04 Stage E).
 *
 * Surfaces when any pending HITL prompt has `ui_variant: 'toast'`. Renders
 * in the corner; clicking expands to options. Auto-dismisses after 30s if
 * the user doesn't interact — the dismiss is treated as a soft `notifier_failed`
 * locally (the SDK seam keeps awaiting; the underlying prompt stays in
 * `pendingHitl` until resolved/timed-out via the seam's own timer).
 *
 * `role="status"` for low-urgency announcements; `aria-live="polite"`
 * so screenreaders announce without interrupting.
 */
export function HITLToast(): JSX.Element | null {
  const pending = useGraphStore((s) => firstToastPrompt(s.pendingHitl));
  const [expanded, setExpanded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const promptId = pending?.promptId ?? null;

  useEffect(() => {
    setExpanded(false);
    setError(null);
    setDismissed(false);
  }, [promptId]);

  useEffect(() => {
    if (!pending || dismissed) {
      return undefined;
    }
    const timer = window.setTimeout(() => {
      setDismissed(true);
    }, TOAST_AUTO_DISMISS_MS);
    return () => window.clearTimeout(timer);
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
    <div
      className="hitl-toast"
      role="status"
      aria-live="polite"
      aria-label={`HITL prompt — ${pending.trigger}`}
      data-testid="hitl-toast"
    >
      {!expanded && (
        <button
          type="button"
          className="hitl-toast__summary"
          data-testid="hitl-toast-summary"
          onClick={() => setExpanded(true)}
        >
          <span className="hitl-toast__trigger">{pending.trigger}</span>
          <span className="hitl-toast__hint">click to expand</span>
        </button>
      )}
      {expanded && (
        <div className="hitl-toast__expanded">
          <p className="hitl-toast__question" data-testid="hitl-toast-question">
            {pending.question}
          </p>
          {error !== null && <p className="hitl-toast__error">{error}</p>}
          <div className="hitl-toast__actions">
            {pending.options.map((option) => (
              <button
                key={option}
                type="button"
                className={`hitl-toast__action hitl-toast__action--${option}`}
                data-testid={`hitl-toast-option-${option}`}
                onClick={() => void respond(option)}
              >
                {option}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function firstToastPrompt(pending: Record<string, PendingHitl>): PendingHitl | null {
  for (const id of Object.keys(pending)) {
    const p = pending[id];
    if (p !== undefined && p.uiVariant === 'toast') {
      return p;
    }
  }
  return null;
}

export const _testing = { firstToastPrompt, TOAST_AUTO_DISMISS_MS };
