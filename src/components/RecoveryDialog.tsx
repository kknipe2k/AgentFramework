import { useEffect, useState } from 'react';
import { invokeRequestResume, unwrapCmdError } from '../lib/ipc';
import { useGraphStore } from '../lib/graphStore';

const LAST_SESSION_KEY = 'lastSessionId';

interface RecoveryDialogProps {
  /** Callback fired after the user picks Resume or Discard. */
  onClose?: () => void;
}

/**
 * RecoveryDialog — spec §1b (M04 Stage F).
 *
 * Cold-start surface: when a prior session id is in localStorage,
 * surface "Previous session detected. Resume?" with Resume / Discard
 * options.
 *
 * Resume calls `invokeRequestResume(sessionId)` which translates to a
 * drone IPC `RecoverSession` round-trip; the returned `ResumePlan`
 * populates the uncertain-invocations list (drives
 * {@link UncertaintyPrompt}).
 *
 * Discard clears the cached session id and dismisses without resuming.
 *
 * The dialog dismisses itself after either action; idempotent if mounted
 * multiple times in the same session.
 */
export function RecoveryDialog({ onClose }: RecoveryDialogProps): JSX.Element | null {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const recordUncertain = useGraphStore((s) => s.recordUncertainInvocation);

  useEffect(() => {
    if (typeof localStorage === 'undefined') {
      return;
    }
    const id = localStorage.getItem(LAST_SESSION_KEY);
    if (id !== null && id.length > 0) {
      setSessionId(id);
    }
  }, []);

  if (dismissed || sessionId === null) {
    return null;
  }

  async function handleResume(): Promise<void> {
    try {
      const plan = await invokeRequestResume(sessionId as string);
      for (const invocationId of plan.uncertain_tool_invocations) {
        recordUncertain({ invocationId });
      }
      setDismissed(true);
      onClose?.();
    } catch (e) {
      console.error('request_resume error:', e);
      setError(unwrapCmdError(e));
    }
  }

  function handleDiscard(): void {
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem(LAST_SESSION_KEY);
    }
    setDismissed(true);
    onClose?.();
  }

  return (
    <div
      className="recovery-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="recovery-dialog-title"
      data-testid="recovery-dialog"
    >
      <h2 id="recovery-dialog-title" className="recovery-dialog__title">
        Previous session detected
      </h2>
      <p className="recovery-dialog__body">
        Resume session <code data-testid="recovery-dialog-session-id">{sessionId}</code>?
      </p>
      <p className="recovery-dialog__hint">
        Resume rebuilds the prior message history from the snapshot. Tool calls in flight at the
        time of interruption will surface for review.
      </p>
      {error !== null && (
        <p className="recovery-dialog__error" data-testid="recovery-dialog-error">
          {error}
        </p>
      )}
      <div className="recovery-dialog__actions">
        <button
          type="button"
          className="recovery-dialog__resume"
          data-testid="recovery-dialog-resume"
          onClick={() => void handleResume()}
        >
          Resume
        </button>
        <button
          type="button"
          className="recovery-dialog__discard"
          data-testid="recovery-dialog-discard"
          onClick={handleDiscard}
        >
          Discard
        </button>
      </div>
    </div>
  );
}
