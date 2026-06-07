import { RawDisclosure } from './RawDisclosure';

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
 * the real validator output into it. The Show-raw disclosure is the shared
 * {@link RawDisclosure} (M08.9.B reuses it for the Tester run drill-down).
 */
export function ValidationCard({ plain, fix, raw }: ValidationCardProps): JSX.Element {
  return (
    <div className="err-card" role="alert" data-testid="validation-card">
      <p className="err-plain">{plain}</p>
      {fix !== undefined && <p className="err-fix">→ {fix}</p>}
      {raw !== undefined && (
        <RawDisclosure
          raw={raw}
          showLabel="Show raw error"
          hideLabel="Hide raw error"
          toggleTestId="validation-show-raw"
          rawTestId="validation-raw"
          wrapClassName="err-raw-wrap"
          toggleClassName="err-raw-toggle"
          rawClassName="err-raw"
        />
      )}
    </div>
  );
}
