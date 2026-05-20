import { useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import {
  cancelPendingImport,
  completeImportArtifact,
  importArtifact,
  unwrapCmdError,
  type ImportArtifactKind,
  type ImportOutcome,
} from '../lib/ipc';
import { useGraphStore, type ImportRecord } from '../lib/graphStore';

const ARTIFACT_KINDS: readonly ImportArtifactKind[] = [
  'skill',
  'tool',
  'agent',
  'mcp_server',
] as const;

/**
 * Builder Import panel (M07 Stage E; M07.5 / ADR-0017; MVP §M7). Paste a
 * raw GitHub URL, pick a kind, click Import → the `import_artifact`
 * Tauri command runs the fetch / schema-validate / §15c gate / L3 /
 * tier-gate pipeline and returns a discriminated `ImportOutcome`.
 *
 * For Novice the backend HOLDS the import at the tier-gate (a `'pending'`
 * outcome — nothing installed or locked); the disclosure modal genuinely
 * gates the install. Install runs the held backend install half
 * (`complete_import_artifact`); Reject drops the held pending import
 * (`cancel_pending_import`) — the M07.V 🔴 #1 fix. Promoted-within-bounds
 * installs inline (L4 auto-accept). `artifact_hash_mismatch` (spec
 * §2214) transitions a record to `'blocked'` and renders the Reinstall /
 * Remove prompt (integrity > availability — ADR-0014).
 *
 * Local-file import via a native picker is deferred to a future stage
 * (needs `@tauri-apps/plugin-dialog` + Rust registration). The wrapper
 * already accepts `'file'` sources; the surface lands when the picker
 * does.
 */
export function ImportPanel(): JSX.Element {
  const imports = useGraphStore(useShallow((s) => Object.values(s.imports)));
  const recordImport = useGraphStore((s) => s.recordImport);
  const confirmImport = useGraphStore((s) => s.confirmImport);
  const dismissImport = useGraphStore((s) => s.dismissImport);

  const [url, setUrl] = useState('');
  const [kind, setKind] = useState<ImportArtifactKind>('skill');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // The most-recent review record drives the modal — Novice imports
  // surface one disclosure at a time (single-session per §0d).
  const reviewItem = imports.find((it) => it.phase === 'review') ?? null;
  const blocked = imports.filter((it) => it.phase === 'blocked');
  const installed = imports.filter((it) => it.phase === 'installed');

  async function handleSubmit(): Promise<void> {
    if (url.trim().length === 0 || submitting) {
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const outcome: ImportOutcome = await importArtifact('url', url.trim(), kind);
      recordImport(outcome);
      setUrl('');
    } catch (e) {
      console.error('import_artifact error:', e);
      setError(unwrapCmdError(e));
    } finally {
      setSubmitting(false);
    }
  }

  // M07.5 / ADR-0017 — the tier-gate review handlers. Install runs the
  // held backend install half; Reject drops the held pending import.
  // The IPC lives here (the handleSubmit pattern); confirmImport /
  // dismissImport stay pure store mutations run only after the backend
  // command resolves.
  async function handleInstall(item: ImportRecord): Promise<void> {
    if (item.pendingReviewId === undefined) {
      return;
    }
    setError(null);
    try {
      await completeImportArtifact(item.pendingReviewId);
      confirmImport(item.ref);
    } catch (e) {
      console.error('complete_import_artifact error:', e);
      setError(unwrapCmdError(e));
    }
  }

  async function handleReject(item: ImportRecord): Promise<void> {
    if (item.pendingReviewId === undefined) {
      return;
    }
    setError(null);
    try {
      await cancelPendingImport(item.pendingReviewId);
      dismissImport(item.ref);
    } catch (e) {
      console.error('cancel_pending_import error:', e);
      setError(unwrapCmdError(e));
    }
  }

  return (
    <section className="import-panel" data-testid="import-panel">
      <header className="import-panel__header">
        <h2 className="import-panel__title">Import</h2>
      </header>
      {error !== null && <p className="import-panel__error">{error}</p>}
      <form
        className="import-panel__form"
        onSubmit={(e) => {
          e.preventDefault();
          void handleSubmit();
        }}
      >
        <label className="import-panel__label">
          <span>GitHub raw URL</span>
          <input
            type="url"
            data-testid="import-url"
            placeholder="https://raw.githubusercontent.com/owner/repo/main/skill.json"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
          />
        </label>
        <label className="import-panel__label">
          <span>Kind</span>
          <select
            data-testid="import-kind"
            value={kind}
            onChange={(e) => setKind(e.target.value as ImportArtifactKind)}
          >
            {ARTIFACT_KINDS.map((k) => (
              <option key={k} value={k}>
                {k}
              </option>
            ))}
          </select>
        </label>
        <button
          type="submit"
          data-testid="import-submit"
          disabled={submitting || url.trim().length === 0}
        >
          Import
        </button>
      </form>

      {blocked.length > 0 && (
        <div
          className="import-panel__reinstall-prompt"
          data-testid="import-reinstall-prompt"
          role="alert"
        >
          <p className="import-panel__reinstall-headline">
            Integrity check failed for <strong>{blocked.map((b) => b.ref).join(', ')}</strong>. The
            artifact will not run with drifted bytes (spec §2214). Reinstall to refresh the locked
            hash, or remove the entry.
          </p>
          {blocked.map((b) => (
            <BlockedRow key={b.ref} item={b} onRemove={() => dismissImport(b.ref)} />
          ))}
        </div>
      )}

      {installed.length > 0 && (
        <ul className="import-panel__installed" data-testid="import-installed-list">
          {installed.map((it) => (
            <li
              key={it.ref}
              className="import-row import-row--installed"
              data-testid={`import-row-${it.ref}`}
            >
              <span className="import-row__ref">{it.ref}</span>
              <span className="import-row__status">installed</span>
            </li>
          ))}
        </ul>
      )}

      {reviewItem !== null && (
        <ReviewModal
          item={reviewItem}
          onInstall={() => void handleInstall(reviewItem)}
          onReject={() => void handleReject(reviewItem)}
        />
      )}
    </section>
  );
}

interface BlockedRowProps {
  item: ImportRecord;
  onRemove: () => void;
}

function BlockedRow({ item, onRemove }: BlockedRowProps): JSX.Element {
  // Reinstall flow needs the original source (URL or file path) to
  // re-run the pipeline; v0.1 does NOT persist the import source past
  // the install (Stage C `Installed` carries lock_key but no source
  // round-trip — a Stage V / gap-analysis carry-forward). The button is
  // rendered (the panel surface honors the spec §M7 / E.3.5 Reinstall
  // affordance) and emits a click; wiring it to a re-fetch is the
  // closure of that carry-forward.
  return (
    <div className="import-row import-row--blocked" data-testid={`import-row-${item.ref}`}>
      <span className="import-row__ref">{item.ref}</span>
      {item.error !== undefined && <span className="import-row__error">{item.error}</span>}
      <div className="import-row__actions">
        <button
          type="button"
          data-testid={`import-reinstall-${item.ref}`}
          onClick={() => {
            // Hook for the source-round-trip closure (above).
            console.warn(
              `Reinstall requested for ${item.ref}; source round-trip is a Stage V / gap carry-forward.`,
            );
          }}
        >
          Reinstall
        </button>
        <button type="button" data-testid={`import-remove-${item.ref}`} onClick={onRemove}>
          Remove
        </button>
      </div>
    </div>
  );
}

interface ReviewModalProps {
  item: ImportRecord;
  onInstall: () => void;
  onReject: () => void;
}

function ReviewModal({ item, onInstall, onReject }: ReviewModalProps): JSX.Element {
  const trustLine = describeProvenance(item.shareProvenance);
  return (
    <div className="import-review-modal-backdrop">
      <div
        className="import-review-modal"
        role="dialog"
        aria-modal="true"
        aria-label={`Review import: ${item.ref}`}
        data-testid="import-review-modal"
      >
        <h2 className="import-review-modal__title">Review import — {item.ref}</h2>
        <p className="import-review-modal__intro">
          Your tier requires reviewing this artifact&apos;s declared capabilities + sandbox report
          before it can run.
        </p>

        <section className="import-review-modal__section">
          <h3 className="import-review-modal__section-title">Declared capabilities</h3>
          {item.capabilities.length === 0 ? (
            <p className="import-review-modal__none" data-testid="import-capability-disclosure">
              No capabilities declared.
            </p>
          ) : (
            <ul className="import-capability-disclosure" data-testid="import-capability-disclosure">
              {item.capabilities.map((c) => (
                <li key={c} className="import-capability-disclosure__item">
                  {c}
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="import-review-modal__section" data-testid="import-l3-report">
          <h3 className="import-review-modal__section-title">L3 sandbox report</h3>
          {item.l3Report === null ? (
            <p className="import-review-modal__none">No L3 report.</p>
          ) : (
            <p className="import-l3-report__body">
              {item.l3Report.passed ? 'L3 sandbox: passed.' : 'L3 sandbox: failed.'}
              {item.l3Report.reasons.length > 0 && ` Reasons: ${item.l3Report.reasons.join('; ')}`}
            </p>
          )}
        </section>

        {item.requiresSecrets.length > 0 && (
          <section className="import-review-modal__section" data-testid="import-requires-secrets">
            <h3 className="import-review-modal__section-title">
              Secrets required before first run (§15d)
            </h3>
            <ul>
              {item.requiresSecrets.map((s) => (
                <li key={s}>{s}</li>
              ))}
            </ul>
          </section>
        )}

        <p className="import-trust-line" data-testid="import-trust-line">
          {trustLine}
        </p>

        <div className="import-review-modal__actions">
          <button type="button" data-testid="import-install" onClick={onInstall}>
            Install
          </button>
          <button type="button" data-testid="import-reject" onClick={onReject}>
            Reject
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * Render the share_provenance trust line per ADR-0005.
 *
 * v0.1 is runtime-to-runtime ONLY (`rebake_changes` is always `[]`).
 * `null` (unexported) renders the "No provenance" state — never a
 * synthesized empty block.
 */
function describeProvenance(prov: unknown): string {
  if (prov === null || prov === undefined) {
    return 'No provenance — this artifact was not exported through the trust chain.';
  }
  // A non-object provenance value is malformed — treat it as absent
  // rather than letting the cast below fall through to the "rebaked"
  // branch (CQ-M07-4). `null` is already handled above.
  if (typeof prov !== 'object') {
    return 'No provenance — this artifact was not exported through the trust chain.';
  }
  // ADR-0005: rebake_changes [] = runtime-to-runtime transfer, no
  // rebaking. A non-empty rebake_changes is a v1.0 Share It path.
  const rec = prov as { rebake_changes?: unknown };
  const changes = Array.isArray(rec.rebake_changes) ? rec.rebake_changes : null;
  if (changes !== null && changes.length === 0) {
    return 'Provenance: runtime-to-runtime — no rebaking (v0.1 trust posture, ADR-0005).';
  }
  return 'Provenance: rebaked at export — review the rebake_changes block in the artifact JSON.';
}
