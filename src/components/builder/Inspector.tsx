import { open } from '@tauri-apps/plugin-dialog';
import { useCallback, useMemo, useState } from 'react';
import { diffFramework } from '../../lib/frameworkDiff';
import {
  loadFramework,
  saveFramework,
  unwrapCmdError,
  validateFramework,
  type FrameworkValidationReport,
  type NodeError,
} from '../../lib/ipc';
import { useBuilderStore } from '../../lib/builderStore';

/**
 * The Builder Inspector (M08.E — spec Phase 9 right sidebar). Four
 * presentational reads over `builderStore` + four action buttons:
 *
 * 1. a live `framework.json` preview — updates as the canvas edits it,
 * 2. a disk diff (`framework` vs `diskFramework` — `diffFramework`),
 *    shown only once the framework has a disk origin,
 * 3. the whole-framework capability summary, read from the
 *    `validate_framework` report's `capability_summary` FIELD (Stage B
 *    B.3.4 — NOT a separate command),
 * 4. an explicit `Validate` button (the SAME `validate_framework` D2's
 *    debounced continuous pass uses — spec §9, one validator two
 *    triggers; this run is immediate and surfaces the full per-node
 *    report) and a `Test` button (sets `builderStore.openTester` —
 *    INERT-but-wired; Stage F2 delivers the Tester modal).
 *
 * Plus Save / Load: the `@tauri-apps/plugin-dialog` directory picker
 * feeds Stage B's `save_framework` / `load_framework` (MVP §M8 criteria
 * 7 + 8). A loaded framework is swapped in via `replaceFramework`; the
 * canvas re-derives (ADR-0020).
 */
export function Inspector(): JSX.Element {
  const framework = useBuilderStore((s) => s.framework);
  const diskFramework = useBuilderStore((s) => s.diskFramework);
  const validation = useBuilderStore((s) => s.validation);
  const replaceFramework = useBuilderStore((s) => s.replaceFramework);
  const setDiskFramework = useBuilderStore((s) => s.setDiskFramework);
  const openTester = useBuilderStore((s) => s.openTester);
  const [report, setReport] = useState<FrameworkValidationReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  // The explicit Validate trigger — the SAME validate_framework D2's
  // debounced continuous pass calls (spec §9 — one Rust validator, two
  // triggers; no TS re-implementation). Surfaces the full per-node
  // report the canvas badges only counted.
  const onValidate = useCallback(async () => {
    try {
      setReport(await validateFramework(framework));
      setError(null);
    } catch (e) {
      setError(unwrapCmdError(e));
    }
  }, [framework]);

  // Export/Save — pick a directory, write framework.json + companions,
  // then record diskFramework so the disk diff zeroes after the save.
  const onSave = useCallback(async () => {
    try {
      const dir = await open({ directory: true });
      if (typeof dir !== 'string') {
        return; // a cancelled picker is a normal user action, not an error
      }
      await saveFramework(dir, framework);
      setDiskFramework(framework);
      setError(null);
    } catch (e) {
      setError(unwrapCmdError(e));
    }
  }, [framework, setDiskFramework]);

  // Open/Load — pick a directory, read it, swap the source-of-truth
  // document via replaceFramework; the canvas re-derives (ADR-0020).
  const onLoad = useCallback(async () => {
    try {
      const dir = await open({ directory: true });
      if (typeof dir !== 'string') {
        return;
      }
      const loaded = await loadFramework(dir);
      replaceFramework(loaded.framework);
      setDiskFramework(loaded.framework);
      setError(null);
    } catch (e) {
      setError(unwrapCmdError(e));
    }
  }, [replaceFramework, setDiskFramework]);

  // The capability summary rides on the validate_framework report's
  // capability_summary field (Stage B B.3.4) — prefer the explicit
  // Validate result, fall back to D2's continuous validation slot.
  const summary = (report ?? validation)?.capability_summary ?? null;
  // The disk diff — only meaningful once the framework has a disk origin.
  const diff = useMemo(
    () => (diskFramework !== null ? diffFramework(framework, diskFramework) : null),
    [framework, diskFramework],
  );
  const reportErrors: NodeError[] =
    report !== null ? [...report.schema_errors, ...report.capability_errors] : [];

  return (
    <aside className="builder-inspector" data-testid="builder-inspector">
      {/* 1. Live framework.json preview — updates as the canvas edits it. */}
      <section className="inspector-section inspector-section--preview">
        <h3>framework.json</h3>
        <pre data-testid="inspector-preview">{JSON.stringify(framework, null, 2)}</pre>
      </section>

      {/* 2. Disk diff — framework vs diskFramework; shown only once the
            framework has a disk origin (diskFramework !== null). */}
      {diff !== null && (
        <section className="inspector-section inspector__diff" data-testid="inspector-diff">
          <h3>Changes since save</h3>
          {diff.changed ? (
            <pre>
              {diff.lines.map((line, i) => (
                <span
                  key={i}
                  className={
                    line.tag === 'added'
                      ? 'inspector__diff-add'
                      : line.tag === 'removed'
                        ? 'inspector__diff-remove'
                        : undefined
                  }
                >
                  {line.tag === 'added' ? '+ ' : line.tag === 'removed' ? '- ' : '  '}
                  {line.text}
                  {'\n'}
                </span>
              ))}
            </pre>
          ) : (
            <p>No changes since the last save.</p>
          )}
        </section>
      )}

      {/* 3. Capability summary — the whole-framework totals carried on
            the validate_framework report's capability_summary field
            (Stage B B.3.4; NOT a separate command). */}
      <section
        className="inspector-section inspector-section--capabilities"
        data-testid="inspector-capabilities"
      >
        <h3>Capability summary</h3>
        {summary !== null ? (
          <dl>
            <dt>Files read</dt>
            <dd>{summary.files_read.join(', ') || '(none)'}</dd>
            <dt>Files written</dt>
            <dd>{summary.files_written.join(', ') || '(none)'}</dd>
            <dt>Network hosts</dt>
            <dd>{summary.network_hosts.join(', ') || '(none)'}</dd>
            <dt>Shell</dt>
            <dd>{summary.any_shell ? 'yes' : 'no'}</dd>
          </dl>
        ) : (
          <p>Validate the framework to compute its capability summary.</p>
        )}
      </section>

      {/* The full validation report — the per-node messages the canvas
          badges only counted; populated by the explicit Validate run. */}
      {report !== null && (
        <section className="inspector-section" data-testid="inspector-report">
          <h3>Validation</h3>
          {reportErrors.length === 0 ? (
            <p>No problems — the framework is valid.</p>
          ) : (
            <ul className="inspector__errors">
              {reportErrors.map((e, i) => (
                <li key={i}>
                  <code>{e.node_path}</code>: {e.message}
                </li>
              ))}
            </ul>
          )}
        </section>
      )}

      {error !== null && (
        <p className="error" role="alert">
          {error}
        </p>
      )}

      {/* 4. Action buttons. */}
      <div className="inspector__actions">
        <button type="button" onClick={() => void onValidate()}>
          Validate
        </button>
        {/* Test ships INERT-but-wired — it sets builderStore.openTester;
            Stage F2 delivers the modal that renders on that state. */}
        <button type="button" onClick={() => openTester()}>
          Test
        </button>
        <button type="button" onClick={() => void onSave()}>
          Save
        </button>
        <button type="button" onClick={() => void onLoad()}>
          Load
        </button>
      </div>
    </aside>
  );
}
