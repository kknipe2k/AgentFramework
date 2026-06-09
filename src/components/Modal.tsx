import { useEffect, useId, useRef, type ReactNode } from 'react';
import { createPortal } from 'react-dom';

export interface ModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  /** `full` is the Tester's near-full-screen surface; default is bounded. */
  size?: 'default' | 'full';
  footer?: ReactNode;
  children: ReactNode;
  /** Stamped on the dialog so callers keep their existing test selectors. */
  testId?: string;
}

const FOCUSABLE =
  'a[href], button:not([disabled]), textarea, input:not([disabled]), select, [tabindex]:not([tabindex="-1"])';

/**
 * The reusable Modal primitive (M08.8.B; DESIGN.md "Modals"; closes #24).
 * Portals to `document.body` so it escapes any ancestor stacking context;
 * overlay at z-index 300 over a blurred scrim; `lg` radius + `e3`; the
 * body scrolls within a bounded `86vh`; Esc / scrim-click / × all close;
 * focus moves in on open and restores on close; button labels are
 * complete + untruncated (the #24 "Canc" fix is structural — the bounded
 * scroll body keeps the action row reachable on any viewport).
 *
 * Each modal that hand-rolled its own overlay migrates onto this
 * (MCPServerAddModal closes #24; TesterModal uses `size="full"`).
 */
export function Modal({
  open,
  onClose,
  title,
  size = 'default',
  footer,
  children,
  testId,
}: ModalProps): JSX.Element | null {
  const dialogRef = useRef<HTMLDivElement>(null);
  const titleId = useId();

  // Esc closes (window-level so it fires regardless of which descendant
  // holds focus). Gated on `open` so a closed modal adds no listener.
  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        onClose();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  // Move focus into the dialog on open; restore it to the opener on close
  // (WAI APG dialog pattern). The cleanup runs when `open` flips false.
  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const prevFocus = document.activeElement as HTMLElement | null;
    const dialog = dialogRef.current;
    const focusables = dialog?.querySelectorAll<HTMLElement>(FOCUSABLE);
    if (focusables !== undefined && focusables.length > 0) {
      focusables[0]?.focus();
    } else {
      dialog?.focus();
    }
    return () => {
      prevFocus?.focus?.();
    };
  }, [open]);

  if (!open) {
    return null;
  }

  // Trap Tab within the dialog so focus never leaks to the chrome behind.
  const trapTab = (e: React.KeyboardEvent): void => {
    if (e.key !== 'Tab') {
      return;
    }
    const dialog = dialogRef.current;
    if (dialog === null) {
      return;
    }
    const focusables = Array.from(dialog.querySelectorAll<HTMLElement>(FOCUSABLE));
    if (focusables.length === 0) {
      e.preventDefault();
      return;
    }
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last?.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first?.focus();
    }
  };

  return createPortal(
    <div
      className="modal-overlay"
      data-testid="modal-scrim"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        ref={dialogRef}
        className={`modal modal--${size}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
        data-testid={testId}
        onKeyDown={trapTab}
      >
        <header className="modal__head">
          <h3 id={titleId} className="modal__title">
            {title}
          </h3>
          <button type="button" className="modal__close" aria-label="Close" onClick={onClose}>
            ×
          </button>
        </header>
        <div
          className="modal__body"
          data-testid="modal-body"
          style={{ maxHeight: size === 'full' ? '92vh' : '86vh' }}
        >
          {children}
        </div>
        {footer !== undefined && <div className="modal__foot">{footer}</div>}
      </div>
    </div>,
    document.body,
  );
}
