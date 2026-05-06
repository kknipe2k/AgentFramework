import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { GraphCanvas } from '../../src/components/GraphCanvas';
import { SetupPanel } from '../../src/components/SetupPanel';
import { SmokeButton } from '../../src/components/SmokeButton';
import { useGraphStore } from '../../src/lib/graphStore';

// Component-level renderer tests. These complement the graphStore + IPC
// unit tests by exercising the rendered output and UX invariants the
// Playwright skeleton (./e2e/smoke.spec.ts) would otherwise need a real
// Tauri shell to assert (per the M02.E retrospective Tauri 2.x E2E note).

describe('SetupPanel', () => {
  it('input_is_password_type_so_key_is_never_visible', () => {
    render(<SetupPanel onSave={vi.fn(async () => {})} />);
    const input = screen.getByLabelText(/anthropic api key/i);
    expect(input).toHaveAttribute('type', 'password');
  });

  it('save_button_disabled_until_minimum_key_length', async () => {
    const user = userEvent.setup();
    render(<SetupPanel onSave={vi.fn(async () => {})} />);
    const button = screen.getByRole('button', { name: /save key/i });
    expect(button).toBeDisabled();
    const input = screen.getByLabelText(/anthropic api key/i);
    await user.type(input, 'short');
    expect(button).toBeDisabled();
    await user.type(input, '-and-then-some');
    expect(button).toBeEnabled();
  });

  it('clears_input_after_save_succeeds', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn(async () => {});
    render(<SetupPanel onSave={onSave} />);
    const input = screen.getByLabelText(/anthropic api key/i);
    await user.type(input, 'sk-ant-1234567890');
    await user.click(screen.getByRole('button', { name: /save key/i }));
    expect(onSave).toHaveBeenCalledWith('sk-ant-1234567890');
    expect(input).toHaveValue('');
    expect(screen.getByLabelText(/saved/i)).toBeInTheDocument();
  });
});

describe('SmokeButton', () => {
  it('respects_disabled_prop', () => {
    render(<SmokeButton disabled={true} onClick={vi.fn(async () => {})} />);
    expect(screen.getByRole('button', { name: /run smoke test/i })).toBeDisabled();
  });

  it('invokes_onClick_when_enabled', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn(async () => {});
    render(<SmokeButton disabled={false} onClick={onClick} />);
    await user.click(screen.getByRole('button', { name: /run smoke test/i }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });
});

describe('GraphCanvas', () => {
  it('renders_empty_canvas_before_any_events_arrive', () => {
    useGraphStore.getState().clear();
    render(<GraphCanvas />);
    const canvas = screen.getByTestId('graph-canvas');
    expect(canvas).toBeInTheDocument();
    // No nodes rendered before any events have arrived.
    expect(canvas.querySelectorAll('[data-testid^="agent-node-"]').length).toBe(0);
    expect(canvas.querySelectorAll('[data-testid^="tool-node-"]').length).toBe(0);
    expect(canvas.querySelectorAll('[data-testid^="skill-node-"]').length).toBe(0);
  });
});
