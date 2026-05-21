import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M07.5 / ADR-0017 — ImportPanel renderer tests. The invoke boundary is
// mocked at @tauri-apps/api/core. `import_artifact` returns the
// discriminated ImportOutcome the A.fix Rust anchor proves (commands.rs
// `ImportOutcome` — `#[serde(tag = "status")]`; the in-source
// `import_outcome_*` tests pin the JSON keys). A Novice import resolves
// to a `pending` outcome carrying the `pending_review_id` the review
// modal echoes back to `complete_import_artifact` / `cancel_pending_import`
// (the M07.V 🔴 #1 renderer wiring). The Install/Reject tests assert the
// BACKEND invoke fires — not merely the local store transition.

const invokeMock = vi.fn(async (..._args: unknown[]) => undefined as unknown);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

// M08.C — the @tauri-apps/plugin-dialog native file picker. `open`
// resolves the chosen absolute path, or null when the user cancels.
const openMock = vi.fn(async (..._args: unknown[]) => undefined as unknown);

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: (...args: unknown[]) => openMock(...args),
}));

import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ImportPanel } from '../../../src/components/ImportPanel';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { ImportOutcome } from '../../../src/lib/ipc';
import type { AgentEvent } from '../../../src/types/agent_event';

// A held Novice import — the A.fix `pending` wire arm. `pending_review_id`
// is the `PendingImportState` key the modal echoes to the backend.
const pendingOutcome: ImportOutcome = {
  status: 'pending',
  pending_review_id: 'pri-1',
  lock_key: 'fs-test@2.0.0',
  requires_secrets: ['OPENAI_API_KEY'],
  capabilities: ['network: api.example.com', 'shell: true'],
  l3_report: { report_id: 'vr-1', passed: true, reasons: [] },
  share_provenance: { exported_by: 'share-it@0.1.0', rebake_changes: [] },
};

function dispatch(events: AgentEvent[]): void {
  act(() => {
    for (const e of events) useGraphStore.getState().applyEvent(e);
  });
}

/**
 * Order-independent invoke mock (M08.C). ImportPanel now calls
 * `list_installed_artifacts` on mount (M07-IRL #6), so a queued
 * `mockResolvedValueOnce` would be consumed by that mount call rather
 * than by the command under test. `resolved` maps a command name to its
 * resolved value; `rejected` maps a command name to a pre-formatted
 * error message it rejects with. `list_installed_artifacts` defaults to
 * `[]`; any unlisted command resolves `undefined`.
 */
function routeInvoke(
  resolved: Record<string, unknown>,
  rejected: Record<string, string> = {},
): void {
  invokeMock.mockImplementation(async (cmd: unknown) => {
    const key = String(cmd);
    if (key in rejected) {
      throw new Error(rejected[key]);
    }
    if (key in resolved) {
      return resolved[key];
    }
    return key === 'list_installed_artifacts' ? [] : undefined;
  });
}

describe('ImportPanel', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    useGraphStore.setState({ imports: {} });
  });
  afterEach(() => {
    useGraphStore.setState({ imports: {} });
  });

  it('pasting_a_url_and_importing_invokes_import_artifact_with_pinned_args', async () => {
    routeInvoke({ import_artifact: pendingOutcome });
    render(<ImportPanel />);

    await userEvent.type(
      screen.getByTestId('import-url'),
      'https://raw.githubusercontent.com/o/r/main/fs.json',
    );
    await userEvent.selectOptions(screen.getByTestId('import-kind'), 'skill');
    await userEvent.click(screen.getByTestId('import-submit'));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('import_artifact', {
        sourceKind: 'url',
        location: 'https://raw.githubusercontent.com/o/r/main/fs.json',
        artifactKind: 'skill',
      }),
    );
  });

  it('review_modal_renders_disclosure_l3_provenance_and_secrets', async () => {
    routeInvoke({ import_artifact: pendingOutcome });
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));

    const modal = await screen.findByTestId('import-review-modal');
    expect(modal).toBeInTheDocument();
    // Capability disclosure = the artifact's REAL declared capabilities.
    const disc = screen.getByTestId('import-capability-disclosure');
    expect(disc).toHaveTextContent('network: api.example.com');
    expect(disc).toHaveTextContent('shell: true');
    // L3 report.
    expect(screen.getByTestId('import-l3-report')).toHaveTextContent(/pass/i);
    // §15d secrets notice.
    expect(screen.getByTestId('import-requires-secrets')).toHaveTextContent('OPENAI_API_KEY');
    // Provenance trust line — runtime-to-runtime, no rebaking (ADR-0005).
    expect(screen.getByTestId('import-trust-line')).toHaveTextContent(/no rebaking/i);
  });

  it('trust_line_says_no_provenance_when_share_provenance_is_null', async () => {
    routeInvoke({ import_artifact: { ...pendingOutcome, share_provenance: null } });
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    const trust = await screen.findByTestId('import-trust-line');
    expect(trust).toHaveTextContent(/no provenance/i);
  });

  it('install_invokes_complete_import_artifact', async () => {
    // M07.5 / ADR-0017 — Install at the tier-gate review must run the
    // backend install half the A.fix pipeline held back. The load-bearing
    // assertion is the BACKEND invoke, not merely the store transition.
    routeInvoke({ import_artifact: pendingOutcome });
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    await screen.findByTestId('import-review-modal');

    await userEvent.click(screen.getByTestId('import-install'));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('complete_import_artifact', {
        pendingReviewId: 'pri-1',
      }),
    );
    await waitFor(() =>
      expect(useGraphStore.getState().imports['fs-test@2.0.0']?.phase).toBe('installed'),
    );
  });

  it('install_confirms_the_review_and_closes_the_modal', async () => {
    routeInvoke({ import_artifact: pendingOutcome });
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    await screen.findByTestId('import-review-modal');

    await userEvent.click(screen.getByTestId('import-install'));
    await waitFor(() =>
      expect(screen.queryByTestId('import-review-modal')).not.toBeInTheDocument(),
    );
    expect(useGraphStore.getState().imports['fs-test@2.0.0']?.phase).toBe('installed');
  });

  it('install_surfaces_an_error_when_complete_import_artifact_fails', async () => {
    // A backend failure of the held install half surfaces via setError;
    // the record stays in 'review' — it is NOT promoted to 'installed'.
    routeInvoke(
      { import_artifact: pendingOutcome },
      { complete_import_artifact: 'internal: install failed' },
    );
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    await screen.findByTestId('import-review-modal');

    await userEvent.click(screen.getByTestId('import-install'));
    expect(await screen.findByText(/internal: install failed/)).toBeInTheDocument();
    expect(useGraphStore.getState().imports['fs-test@2.0.0']?.phase).toBe('review');
  });

  it('reject_invokes_cancel_pending_import', async () => {
    // M07.V 🔴 #1 closure. The prior reject_dismisses_the_import_record
    // (below) asserted ONLY that the local store record was deleted —
    // never that the backend was told to drop the held PendingImport.
    // That store-only check was the precise Stage-V blind spot. This
    // test pins the BACKEND command fire.
    routeInvoke({ import_artifact: pendingOutcome });
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    await screen.findByTestId('import-review-modal');

    await userEvent.click(screen.getByTestId('import-reject'));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('cancel_pending_import', {
        pendingReviewId: 'pri-1',
      }),
    );
  });

  it('reject_dismisses_the_import_record', async () => {
    routeInvoke({ import_artifact: pendingOutcome });
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    await screen.findByTestId('import-review-modal');

    await userEvent.click(screen.getByTestId('import-reject'));
    await waitFor(() => expect(useGraphStore.getState().imports['fs-test@2.0.0']).toBeUndefined());
  });

  it('reject_surfaces_an_error_when_cancel_pending_import_fails', async () => {
    // A backend failure of cancel_pending_import surfaces via setError;
    // the local record is NOT dismissed (the store action runs only
    // after the IPC resolves).
    routeInvoke(
      { import_artifact: pendingOutcome },
      { cancel_pending_import: 'internal: cancel failed' },
    );
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    await screen.findByTestId('import-review-modal');

    await userEvent.click(screen.getByTestId('import-reject'));
    expect(await screen.findByText(/internal: cancel failed/)).toBeInTheDocument();
    expect(useGraphStore.getState().imports['fs-test@2.0.0']).toBeDefined();
  });

  it('install_and_reject_are_no_ops_when_the_review_record_lacks_a_pending_id', async () => {
    // Defensive guard: a 'review' record with no pendingReviewId is never
    // produced by recordImport, but the field is optional on ImportRecord.
    // Install / Reject must not fire a backend command for such a record.
    useGraphStore.setState({
      imports: {
        'fs-test@2.0.0': {
          ref: 'fs-test@2.0.0',
          phase: 'review',
          capabilities: [],
          requiresSecrets: [],
          l3Report: null,
          shareProvenance: null,
        },
      },
    });
    routeInvoke({});
    render(<ImportPanel />);
    await screen.findByTestId('import-review-modal');

    await userEvent.click(screen.getByTestId('import-install'));
    await userEvent.click(screen.getByTestId('import-reject'));
    // The mount-time list_installed_artifacts call (M07-IRL #6) is
    // expected; the load-bearing assertion is that NO backend
    // install/reject command fired for a pending-id-less record.
    expect(invokeMock).not.toHaveBeenCalledWith('import_artifact', expect.anything());
    expect(invokeMock).not.toHaveBeenCalledWith('complete_import_artifact', expect.anything());
    expect(invokeMock).not.toHaveBeenCalledWith('cancel_pending_import', expect.anything());
  });

  it('artifact_hash_mismatch_surfaces_a_blocking_reinstall_remove_prompt', async () => {
    render(<ImportPanel />);
    dispatch([
      {
        type: 'artifact_hash_mismatch',
        artifact_ref: 'fs-test@2.0.0',
        expected: 'sha256-AAAA',
        actual: 'sha256-BBBB',
      },
    ]);
    const prompt = await screen.findByTestId('import-reinstall-prompt');
    expect(prompt).toBeInTheDocument();
    expect(prompt).toHaveTextContent('fs-test@2.0.0');
    expect(screen.getByTestId('import-reinstall-fs-test@2.0.0')).toBeInTheDocument();

    await userEvent.click(screen.getByTestId('import-remove-fs-test@2.0.0'));
    await waitFor(() => expect(useGraphStore.getState().imports['fs-test@2.0.0']).toBeUndefined());
  });
});

// ── M08.A: import-panel contrast (M07-IRL #3) ──
// The .import-* selectors set `background` but never `color`, so the
// import UI text rendered ≈ the dark panel background. The fix pins each
// selector to the established --node-fg / --node-fg-muted theme tokens.
// Asserted against the stylesheet source: vitest's happy-dom does not
// apply Vite-imported CSS to getComputedStyle, so the rule's declared
// color is the deterministic regression surface.

/**
 * Return the declaration body of the CSS rule whose comma-separated
 * selector list contains `selector` exactly. Throws if absent.
 */
function ruleBodyFor(css: string, selector: string): string {
  const ruleRe = /([^{}]+)\{([^{}]*)\}/g;
  let m: RegExpExecArray | null;
  while ((m = ruleRe.exec(css)) !== null) {
    const selectorList = m[1] ?? '';
    const body = m[2] ?? '';
    const selectors = selectorList.split(',').map((s) => s.trim());
    if (selectors.includes(selector)) {
      return body;
    }
  }
  throw new Error(`no CSS rule found for selector ${selector}`);
}

describe('ImportPanel contrast (M07-IRL #3)', () => {
  const css = readFileSync(resolve(__dirname, '../../../src/styles.css'), 'utf8');

  it('import_panel_title_uses_node_fg_theme_token', () => {
    expect(ruleBodyFor(css, '.import-panel__title')).toMatch(/color:\s*var\(--node-fg\)/);
  });

  it('import_row_ref_uses_node_fg_theme_token', () => {
    expect(ruleBodyFor(css, '.import-row__ref')).toMatch(/color:\s*var\(--node-fg\)/);
  });

  it('import_review_modal_title_uses_node_fg_theme_token', () => {
    const body = ruleBodyFor(css, '.import-review-modal__title');
    expect(body).toMatch(/color:\s*var\(--node-fg\)/);
    expect(body).not.toMatch(/color:\s*var\(--node-bg\)/);
  });
});

// ── M08.C: local-file picker + skills.lock-on-mount reload ──
// M07.V 🟡 #4 — the @tauri-apps/plugin-dialog "Browse…" companion to the
// URL field (the backend already accepted ImportSource::File; only the
// renderer surface was missing). M07-IRL #6 — the panel was empty after
// an app restart because nothing read the durable skills.lock on
// startup; list_installed_artifacts (Stage B) is the production reader.

describe('ImportPanel — file picker + skills.lock reload (M08.C)', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    openMock.mockReset();
    useGraphStore.setState({ imports: {} });
  });
  afterEach(() => {
    useGraphStore.setState({ imports: {} });
  });

  it('calls_list_installed_artifacts_on_mount_and_renders_the_rows', async () => {
    // M07-IRL #6 — reload the durable skills.lock so installed artifacts
    // survive an app restart.
    invokeMock.mockImplementation(async (cmd: unknown) => {
      if (cmd === 'list_installed_artifacts') {
        return [
          {
            key: 'demo-agent@1.0.0',
            kind: 'agent',
            source: { type: 'url', url: 'https://example.com/a.json' },
            installed_at: '2026-05-21T00:00:00Z',
          },
        ];
      }
      return undefined;
    });
    render(<ImportPanel />);
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('list_installed_artifacts', undefined),
    );
    expect(await screen.findByTestId('import-lock-row-demo-agent@1.0.0')).toBeInTheDocument();
  });

  it('clicking_browse_opens_the_picker_and_imports_the_chosen_file', async () => {
    // M07.V 🟡 #4 — Browse → native picker → import_artifact('file', …).
    openMock.mockResolvedValue('C:/tmp/demo-skill.json');
    invokeMock.mockImplementation(async (cmd: unknown) => {
      if (cmd === 'list_installed_artifacts') return [];
      return {
        status: 'installed',
        lock_key: 'demo-skill@1.0.0',
        capabilities: [],
        l3_report: { report_id: 'vr-1', passed: true, reasons: [] },
        requires_secrets: [],
        share_provenance: null,
      };
    });
    render(<ImportPanel />);
    await userEvent.click(screen.getByTestId('import-browse'));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('import_artifact', {
        sourceKind: 'file',
        location: 'C:/tmp/demo-skill.json',
        artifactKind: 'skill',
      }),
    );
  });

  it('cancelling_the_picker_does_not_call_importArtifact', async () => {
    // A cancelled picker resolves null — a normal user action, not an
    // error. import_artifact must not fire.
    openMock.mockResolvedValue(null);
    invokeMock.mockImplementation(async (cmd: unknown) => {
      if (cmd === 'list_installed_artifacts') return [];
      return undefined;
    });
    render(<ImportPanel />);
    await userEvent.click(screen.getByTestId('import-browse'));
    await waitFor(() => expect(openMock).toHaveBeenCalled());
    const importCalls = invokeMock.mock.calls.filter((c) => c[0] === 'import_artifact');
    expect(importCalls).toHaveLength(0);
  });
});
