import { useState } from 'react';
import { invokeRespondUncertainty, unwrapCmdError, type UncertaintyAction } from '../lib/ipc';
import { useGraphStore } from '../lib/graphStore';

const ACTIONS: { value: UncertaintyAction; label: string; testid: string; key: string }[] = [
  { value: 'retry', label: 'Retry', testid: 'uncertainty-action-retry', key: 'r' },
  { value: 'skip', label: 'Skip', testid: 'uncertainty-action-skip', key: 's' },
  { value: 'mark_complete', label: 'Mark complete', testid: 'uncertainty-action-mark', key: 'm' },
  { value: 'abort', label: 'Abort', testid: 'uncertainty-action-abort', key: 'a' },
];

interface UncertaintyPromptProps {
  /**
   * Active session id used by the underlying `respond_uncertainty`
   * IPC. v0.1 single-session per spec §0d so the caller supplies the
   * id directly; multi-session (v1.0) will route via context.
   */
  sessionId: string;
}

/**
 * UncertaintyPrompt — spec §1b (M04 Stage F).
 *
 * Modal dialog. Iterates `state.uncertainInvocations`; for each one
 * presents the 4 spec §1b actions:
 *
 * - [r]etry — re-invoke from scratch
 * - [s]kip — assume the call returned nothing
 * - [m]ark complete — assume it succeeded
 * - [a]bort — cancel the session
 *
 * Each click dispatches `respond_uncertainty` which writes a
 * `tool_call_uncertainty_resolved` decision signal to the VDR.
 *
 * Renders only when `uncertainInvocations` is non-empty.
 */
export function UncertaintyPrompt({ sessionId }: UncertaintyPromptProps): JSX.Element | null {
  const uncertain = useGraphStore((s) => s.uncertainInvocations);
  const resolve = useGraphStore((s) => s.resolveUncertainInvocation);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const head = uncertain[0];
  if (head === undefined) {
    return null;
  }
  // Rebind to a non-undefined locally — TS narrows `head` but loses the
  // narrowing across the closure boundary for `handlePick` below.
  const current = head;

  async function handlePick(action: UncertaintyAction): Promise<void> {
    setBusyId(current.invocationId);
    setError(null);
    try {
      await invokeRespondUncertainty(sessionId, current.invocationId, action, current.agentId);
      resolve(current.invocationId);
    } catch (e) {
      console.error('respond_uncertainty error:', e);
      setError(unwrapCmdError(e));
    } finally {
      setBusyId(null);
    }
  }

  return (
    <div
      className="uncertainty-prompt"
      role="dialog"
      aria-modal="true"
      aria-labelledby="uncertainty-prompt-title"
      data-testid="uncertainty-prompt"
    >
      <h2 id="uncertainty-prompt-title" className="uncertainty-prompt__title">
        Tool call uncertainty
      </h2>
      <p className="uncertainty-prompt__body">
        Tool invocation{' '}
        <code data-testid="uncertainty-prompt-invocation-id">{current.invocationId}</code>
        {current.toolName !== undefined && (
          <>
            {' '}
            (<code>{current.toolName}</code>)
          </>
        )}{' '}
        was in flight when the session was interrupted. What happened?
      </p>
      {error !== null && (
        <p className="uncertainty-prompt__error" data-testid="uncertainty-prompt-error">
          {error}
        </p>
      )}
      <div className="uncertainty-prompt__actions">
        {ACTIONS.map((a) => (
          <button
            key={a.value}
            type="button"
            className={`uncertainty-prompt__action uncertainty-prompt__action--${a.value}`}
            data-testid={a.testid}
            disabled={busyId === current.invocationId}
            onClick={() => void handlePick(a.value)}
            title={`Press [${a.key}] — ${a.label}`}
          >
            <span className="uncertainty-prompt__hotkey">[{a.key}]</span> {a.label}
          </button>
        ))}
      </div>
      {uncertain.length > 1 && (
        <p className="uncertainty-prompt__remaining" data-testid="uncertainty-prompt-remaining">
          {uncertain.length - 1} more invocation{uncertain.length - 1 === 1 ? '' : 's'} pending
        </p>
      )}
    </div>
  );
}

export const _testing = { ACTIONS };
