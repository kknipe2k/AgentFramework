import type { ReactNode } from 'react';
import { useToastStore, type ToastKind } from '../lib/toastStore';

/**
 * The reusable Toast primitive (M08.8.B; DESIGN.md "Toasts"). Bottom-right
 * stack, left status border, icon + title + plain-language message,
 * auto-dismiss, non-blocking. Consumed by C (cap saved) + F (validate /
 * save). The HITL-specific `HITLToast` is a different surface (it carries
 * response controls); this is the generic notification primitive.
 *
 * The stack is a `role="status"` / `aria-live="polite"` live region with
 * NO interactive descendants — interactive controls inside a live region
 * fight the announcement semantics (Soueidan aria-live part 1; MDN status
 * role), so dismissal is auto-only. The container is always present in the
 * DOM (even empty) so injected toasts are announced (the live-region best
 * practice).
 */

/** Consumer hook — `useToast().push({ kind, title, message })`. */
export function useToast(): {
  push: (toast: { kind: ToastKind; title: string; message?: string }) => number;
  dismiss: (id: number) => void;
} {
  const push = useToastStore((s) => s.push);
  const dismiss = useToastStore((s) => s.dismiss);
  return { push, dismiss };
}

// A glyph per kind — text, not an interactive control (keeps the live
// region focusable-free). aria-hidden so the title carries the meaning.
const ICON: Record<ToastKind, string> = { ok: '✓', error: '!', warn: '!', info: 'i' };

function ToastViewport(): JSX.Element {
  const toasts = useToastStore((s) => s.toasts);
  return (
    <div className="toast-stack" data-testid="toast-stack" role="status" aria-live="polite">
      {toasts.map((t) => (
        <div key={t.id} className={`toast toast--${t.kind}`} data-testid={`toast-${t.kind}`}>
          <span className="toast__icon" aria-hidden="true">
            {ICON[t.kind]}
          </span>
          <div className="toast__body">
            <div className="toast__title">{t.title}</div>
            {t.message !== undefined && <div className="toast__message">{t.message}</div>}
          </div>
        </div>
      ))}
    </div>
  );
}

/** Mounts the toast live region alongside the app tree. */
export function ToastProvider({ children }: { children: ReactNode }): JSX.Element {
  return (
    <>
      {children}
      <ToastViewport />
    </>
  );
}
