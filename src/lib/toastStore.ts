import { create } from 'zustand';

/** Toast severity — drives the left status border + icon color. */
export type ToastKind = 'ok' | 'error' | 'warn' | 'info';

export interface ToastItem {
  id: number;
  kind: ToastKind;
  title: string;
  message?: string;
}

const DEFAULT_TTL_MS = 4200;
const LONG_TTL_MS = 6000;
const LONG_MESSAGE_THRESHOLD = 80;

interface ToastState {
  toasts: ToastItem[];
  /** Queue a toast; returns its id. Auto-dismisses after its ttl. */
  push: (toast: { kind: ToastKind; title: string; message?: string }) => number;
  dismiss: (id: number) => void;
}

// Module-scoped monotonic id — deterministic (no Math.random), unique for
// the session. A toast's identity never collides within a render tree.
let nextId = 1;

/**
 * The reusable notification store (M08.8.B). DESIGN.md principle 1 —
 * every action gives feedback. C (cap saved) + F (validate/save) push
 * through this; the {@link useToast} hook is the consumer surface.
 *
 * A long message gets a longer time on screen (6s vs 4.2s) so there is
 * time to read it.
 */
export const useToastStore = create<ToastState>((set, get) => ({
  toasts: [],
  push: ({ kind, title, message }) => {
    const id = nextId;
    nextId += 1;
    set((s) => ({ toasts: [...s.toasts, { id, kind, title, message }] }));
    const ttl =
      message !== undefined && message.length > LONG_MESSAGE_THRESHOLD
        ? LONG_TTL_MS
        : DEFAULT_TTL_MS;
    setTimeout(() => get().dismiss(id), ttl);
    return id;
  },
  dismiss: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));

export const _testing = { DEFAULT_TTL_MS, LONG_TTL_MS, LONG_MESSAGE_THRESHOLD };
