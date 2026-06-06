import { useState } from 'react';

export interface RawDisclosureProps {
  /** The raw text kept one click away (never silently dropped, never dumped). */
  raw: string;
  /** Toggle label when collapsed (e.g. `Show raw error`). */
  showLabel: string;
  /** Toggle label when expanded (e.g. `Hide raw error`). */
  hideLabel: string;
  /** Test/landmark id placed on the toggle button. */
  toggleTestId: string;
  /** Test/landmark id placed on the revealed `<pre>`. */
  rawTestId: string;
  /** Optional wrapper class. */
  wrapClassName?: string;
  /** Optional toggle-button class. */
  toggleClassName?: string;
  /** Optional revealed-`<pre>` class. */
  rawClassName?: string;
}

/**
 * The progressive "Show raw" disclosure (DESIGN.md rule 7 / principle 3 —
 * keep dense raw output one click away). Lifted out of `ValidationCard` at
 * M08.9.B as a behavior-preserving extraction so the Tester run drill-down
 * reuses the SAME disclosure the validation surface uses, rather than
 * rolling a second one. Fully class/label/testid configurable so each host
 * (the err-card vs the trace row) keeps its own styling and landmarks.
 */
export function RawDisclosure({
  raw,
  showLabel,
  hideLabel,
  toggleTestId,
  rawTestId,
  wrapClassName,
  toggleClassName,
  rawClassName,
}: RawDisclosureProps): JSX.Element {
  const [showRaw, setShowRaw] = useState(false);
  return (
    <div className={wrapClassName}>
      <button
        type="button"
        className={toggleClassName}
        aria-expanded={showRaw}
        data-testid={toggleTestId}
        onClick={() => setShowRaw((v) => !v)}
      >
        {showRaw ? hideLabel : showLabel}
      </button>
      {showRaw && (
        <pre className={rawClassName} data-testid={rawTestId}>
          {raw}
        </pre>
      )}
    </div>
  );
}
