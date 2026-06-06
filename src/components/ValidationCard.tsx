import { useState } from 'react';

export interface ValidationCardProps {
  /** The plain-English cause (DESIGN.md rule 6 — no raw stack by default). */
  plain: string;
  /** An optional `→ fix` suggestion (the next action to take). */
  fix?: string;
  /** The raw validator/parser output, kept one click away (rule 7). */
  raw?: string;
}

/**
 * The validation error surface (M08.8.B.fix; the mockup's `.err-card`,
 * workbench.css; DESIGN.md rules 6+7). A plain-English cause, an optional
 * `→ fix`, and a progressive "Show raw error" disclosure so the raw output
 * is never silently dropped nor dumped by default. Presentational — F wires
 * the real validator output into it.
 */
export function ValidationCard({ plain, fix, raw }: ValidationCardProps): JSX.Element {
  const [showRaw, setShowRaw] = useState(false);
  return (
    <div className="err-card" role="alert" data-testid="validation-card">
      <p className="err-plain">{plain}</p>
      {fix !== undefined && <p className="err-fix">→ {fix}</p>}
      {raw !== undefined && (
        <div className="err-raw-wrap">
          <button
            type="button"
            className="err-raw-toggle"
            aria-expanded={showRaw}
            data-testid="validation-show-raw"
            onClick={() => setShowRaw((v) => !v)}
          >
            {showRaw ? 'Hide raw error' : 'Show raw error'}
          </button>
          {showRaw && (
            <pre className="err-raw" data-testid="validation-raw">
              {raw}
            </pre>
          )}
        </div>
      )}
    </div>
  );
}
