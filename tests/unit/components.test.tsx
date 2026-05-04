import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { EventList } from '../../src/components/EventList';
import { SetupPanel } from '../../src/components/SetupPanel';
import { SmokeButton } from '../../src/components/SmokeButton';
import type { AgentEvent } from '../../src/types/agent_event';

// Component-level renderer tests. These complement the eventReducer/IPC
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
    // Input cleared, "stored" indicator shown.
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

describe('EventList', () => {
  it('renders_empty_list_with_aria_label', () => {
    render(<EventList events={[]} />);
    const list = screen.getByRole('list', { name: /agent events/i });
    expect(list).toBeInTheDocument();
    expect(list.children).toHaveLength(0);
  });

  it('renders_each_event_with_data_event_type_attr', () => {
    const events: AgentEvent[] = [
      {
        type: 'agent_spawned',
        agent_id: 'a1',
        agent_name: 'smoke',
        parent_id: null,
        session_id: 's1',
      },
      { type: 'stream_text', agent_id: 'a1', text: 'hello' },
      { type: 'agent_complete', agent_id: 'a1', result: 'done' },
    ];
    render(<EventList events={events} />);
    const items = screen.getAllByRole('listitem');
    expect(items).toHaveLength(3);
    expect(items[0]).toHaveAttribute('data-event-type', 'agent_spawned');
    expect(items[1]).toHaveAttribute('data-event-type', 'stream_text');
    expect(items[2]).toHaveAttribute('data-event-type', 'agent_complete');
  });

  it('summarizes_each_variant_safely', () => {
    const variants: AgentEvent[] = [
      { type: 'session_start', session_id: 's1', framework: 'aria', model: 'haiku' },
      {
        type: 'agent_spawned',
        agent_id: 'a1',
        agent_name: 'n',
        parent_id: null,
        session_id: 's1',
      },
      { type: 'agent_complete', agent_id: 'a1', result: 'r' },
      { type: 'agent_error', agent_id: 'a1', error: 'e' },
      {
        type: 'tool_invoked',
        agent_id: 'a1',
        tool_name: 't',
        source: 'builtin',
        server: null,
        input: {},
      },
      {
        type: 'tool_result',
        agent_id: 'a1',
        tool_name: 't',
        output: {},
        duration_ms: 12,
      },
      { type: 'stream_text', agent_id: 'a1', text: 'x' },
      {
        type: 'decision_record',
        agent_id: 'a1',
        decision: 'd',
        rationale: 'r',
        tool_used: null,
      },
      { type: 'task_started', plan_id: 'p1', task_id: 't1', agent_id: 'a1' },
      {
        type: 'task_completed',
        plan_id: 'p1',
        task_id: 't1',
        duration_ms: 7,
      },
    ];
    render(<EventList events={variants} />);
    expect(screen.getAllByRole('listitem')).toHaveLength(variants.length);
  });
});
