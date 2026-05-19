import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M07.E / ADR-0015 — ImportPanel renderer tests. The invoke boundary is
// mocked at @tauri-apps/api/core returning the EXACT enriched
// ImportOutcome shape the Rust anchor proves (commands.rs
// `import_outcome_serializes_the_enriched_review_wire_for_the_renderer`
// + the runtime-main `enriched_install_*` integration tests drive the
// REAL pipeline). The fixture here is therefore the Rust-proven bridge
// contract, not a fabricated mock (the condition-2 anti-false-green
// linkage; the cross-language window itself is gotcha #23).

const invokeMock = vi.fn(async (..._args: unknown[]) => undefined as unknown);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ImportPanel } from '../../../src/components/ImportPanel';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { ImportOutcome } from '../../../src/lib/ipc';
import type { AgentEvent } from '../../../src/types/agent_event';

const enriched: ImportOutcome = {
  lock_key: 'fs-test@2.0.0',
  review_required: true,
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

describe('ImportPanel', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    useGraphStore.setState({ imports: {} });
  });
  afterEach(() => {
    useGraphStore.setState({ imports: {} });
  });

  it('pasting_a_url_and_importing_invokes_import_artifact_with_pinned_args', async () => {
    invokeMock.mockResolvedValueOnce(enriched);
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
    invokeMock.mockResolvedValueOnce(enriched);
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
    invokeMock.mockResolvedValueOnce({ ...enriched, share_provenance: null });
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    const trust = await screen.findByTestId('import-trust-line');
    expect(trust).toHaveTextContent(/no provenance/i);
  });

  it('install_confirms_the_review_and_closes_the_modal', async () => {
    invokeMock.mockResolvedValueOnce(enriched);
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

  it('reject_dismisses_the_import_record', async () => {
    invokeMock.mockResolvedValueOnce(enriched);
    render(<ImportPanel />);
    await userEvent.type(screen.getByTestId('import-url'), 'https://x/y.json');
    await userEvent.click(screen.getByTestId('import-submit'));
    await screen.findByTestId('import-review-modal');

    await userEvent.click(screen.getByTestId('import-reject'));
    await waitFor(() => expect(useGraphStore.getState().imports['fs-test@2.0.0']).toBeUndefined());
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
