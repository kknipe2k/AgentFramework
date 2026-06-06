import { useEffect, useRef, useState } from 'react';
import { useBuilderStore } from '../../lib/builderStore';
import type { Framework } from '../../types/framework';
import { ValidationCard } from '../ValidationCard';

/**
 * The Canvas | JSON binding's JSON tab (M08.E — spec Phase 9 / MVP §M8
 * criterion 6). A raw-JSON editor over `builderStore.framework` — just
 * another editor over the source-of-truth document, exactly as the
 * canvas is (ADR-0020). There is no bespoke two-way-sync machinery:
 *
 * - Canvas → JSON: a canvas edit mutates `framework`; the `useEffect`
 *   below re-seeds the draft from it.
 * - JSON → Canvas: a VALID edit routes through `replaceFramework` and
 *   the canvas projection re-derives. An INVALID (half-typed /
 *   malformed) edit surfaces an inline parse error and leaves the
 *   store UNTOUCHED — the load-bearing no-desync guard: garbage must
 *   never reach `replaceFramework` and desync the canvas.
 */
export function JsonView(): JSX.Element {
  const framework = useBuilderStore((s) => s.framework);
  const replaceFramework = useBuilderStore((s) => s.replaceFramework);
  // The textarea editing buffer — held local so a half-typed (invalid)
  // edit does NOT round-trip through the store.
  const [draft, setDraft] = useState(() => JSON.stringify(framework, null, 2));
  const [parseError, setParseError] = useState<string | null>(null);
  // True for the one render after THIS view's own valid edit — so the
  // re-seed effect does not re-stringify (and clobber) the user's
  // in-progress draft. A framework change from elsewhere (a canvas
  // edit) leaves the flag false and the draft re-seeds.
  const selfEdit = useRef(false);

  useEffect(() => {
    if (selfEdit.current) {
      selfEdit.current = false;
      return;
    }
    setDraft(JSON.stringify(framework, null, 2));
  }, [framework]);

  function onChange(text: string): void {
    setDraft(text);
    let parsed: Framework;
    try {
      parsed = JSON.parse(text) as Framework;
    } catch (e) {
      // Invalid JSON — surface the parse error inline and leave the
      // store untouched. replaceFramework is NEVER called with garbage.
      setParseError(e instanceof Error ? e.message : 'Invalid JSON');
      return;
    }
    setParseError(null);
    selfEdit.current = true;
    replaceFramework(parsed);
  }

  return (
    <div className="json-view" data-testid="builder-json-view">
      <textarea
        data-testid="builder-json-textarea"
        value={draft}
        onChange={(e) => onChange(e.target.value)}
        spellCheck={false}
      />
      {parseError !== null && (
        <div className="json-view__error" data-testid="builder-json-error">
          {/* M08.8.B.fix — the validation err-card (plain cause + → fix +
              Show-raw disclosure). The plain/fix copy is the stable
              JSON-parse case; the raw parser message rides the disclosure. */}
          <ValidationCard
            plain="This JSON isn't valid, so the canvas was left unchanged."
            fix="Fix the highlighted syntax — the canvas updates as soon as it parses."
            raw={parseError}
          />
        </div>
      )}
    </div>
  );
}
