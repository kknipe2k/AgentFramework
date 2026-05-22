import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn(async (..._args: unknown[]) => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { act, render, screen } from '@testing-library/react';
import { HITLModal } from '../../../src/components/HITLModal';
import { HITLPanel } from '../../../src/components/HITLPanel';
import { HITLToast } from '../../../src/components/HITLToast';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent, HitlUiVariantRef } from '../../../src/types/agent_event';

// ── M08.A: HITL ui_variant routing (M06.5 IRL 🟡-1) ──
// The hitl_requested event's ui_variant must route to exactly one
// surface. The graphStore reducer maps event.ui_variant into
// PendingHitl.uiVariant (graphStore.ts), and each of the three
// components filters pendingHitl by its own variant. This pins all
// three routes end-to-end: the chosen variant surfaces ITS component
// and NOT the other two (gotcha #66 — a contract test, not just a
// presence test).

function hitlRequested(uiVariant: HitlUiVariantRef): AgentEvent {
  return {
    type: 'hitl_requested',
    prompt_id: `p-${uiVariant}`,
    trigger: 'on_risky_tool',
    agent_id: 'a1',
    question: 'Proceed?',
    options: ['allow', 'block'],
    ui_variant: uiVariant,
    timeout_at_unix_ms: 9_999,
  };
}

function renderAllThree(): void {
  render(
    <>
      <HITLModal />
      <HITLPanel />
      <HITLToast />
    </>,
  );
}

describe('HITL ui_variant routing (M06.5 IRL 🟡-1)', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    useGraphStore.getState().clear();
  });

  it('hitl_requested_modal_variant_surfaces_modal_only', () => {
    act(() => useGraphStore.getState().applyEvent(hitlRequested('modal')));
    renderAllThree();
    expect(screen.getByTestId('hitl-modal')).toBeInTheDocument();
    expect(screen.queryByTestId('hitl-panel')).toBeNull();
    expect(screen.queryByTestId('hitl-toast')).toBeNull();
  });

  it('hitl_requested_panel_variant_surfaces_panel_only', () => {
    act(() => useGraphStore.getState().applyEvent(hitlRequested('panel')));
    renderAllThree();
    expect(screen.getByTestId('hitl-panel')).toBeInTheDocument();
    expect(screen.queryByTestId('hitl-modal')).toBeNull();
    expect(screen.queryByTestId('hitl-toast')).toBeNull();
  });

  it('hitl_requested_toast_variant_surfaces_toast_only', () => {
    act(() => useGraphStore.getState().applyEvent(hitlRequested('toast')));
    renderAllThree();
    expect(screen.getByTestId('hitl-toast')).toBeInTheDocument();
    expect(screen.queryByTestId('hitl-modal')).toBeNull();
    expect(screen.queryByTestId('hitl-panel')).toBeNull();
  });
});
