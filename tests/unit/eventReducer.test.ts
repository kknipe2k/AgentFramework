import { describe, expect, it } from 'vitest';
import { type Action, type State, initialState, reducer } from '../../src/lib/eventReducer';
import type { AgentEvent } from '../../src/types/agent_event';

const spawnEvent: AgentEvent = {
  type: 'agent_spawned',
  agent_id: 'a1',
  agent_name: 'smoke',
  parent_id: null,
  session_id: 's1',
};

const textEvent: AgentEvent = {
  type: 'stream_text',
  agent_id: 'a1',
  text: 'hello',
};

const completeEvent: AgentEvent = {
  type: 'agent_complete',
  agent_id: 'a1',
  result: 'done',
};

describe('eventReducer', () => {
  it('initial_state_is_empty', () => {
    expect(initialState).toEqual({ events: [], error: null, running: false });
  });

  it('event_received_appends_immutably', () => {
    const prev = initialState;
    const next = reducer(prev, { type: 'event_received', event: spawnEvent });
    expect(next.events).toEqual([spawnEvent]);
    // Reference inequality — reducer must not mutate the prior events array.
    expect(next.events).not.toBe(prev.events);
    expect(prev.events).toEqual([]);
  });

  it('clear_resets_to_initial', () => {
    const populated: State = {
      events: [spawnEvent, textEvent],
      error: 'boom',
      running: true,
    };
    const next = reducer(populated, { type: 'clear' });
    expect(next).toEqual(initialState);
  });

  it('error_sets_message_and_clears_running', () => {
    const running: State = { events: [], error: null, running: true };
    const next = reducer(running, { type: 'error', message: 'kaboom' });
    expect(next.error).toBe('kaboom');
    expect(next.running).toBe(false);
  });

  it('started_sets_running_clears_error', () => {
    const errored: State = { events: [], error: 'old', running: false };
    const next = reducer(errored, { type: 'started' });
    expect(next.running).toBe(true);
    expect(next.error).toBeNull();
  });

  it('completed_clears_running', () => {
    const running: State = { events: [], error: null, running: true };
    const next = reducer(running, { type: 'completed' });
    expect(next.running).toBe(false);
  });

  it('multiple_events_preserve_order', () => {
    let s: State = initialState;
    const sequence: AgentEvent[] = [spawnEvent, textEvent, completeEvent];
    for (const event of sequence) {
      s = reducer(s, { type: 'event_received', event });
    }
    expect(s.events.map((e) => e.type)).toEqual(['agent_spawned', 'stream_text', 'agent_complete']);
  });

  it('clear_after_events_drops_them', () => {
    const populated = reducer(initialState, {
      type: 'event_received',
      event: spawnEvent,
    });
    const cleared = reducer(populated, { type: 'clear' });
    expect(cleared.events).toEqual([]);
  });

  it('error_during_running_keeps_existing_events', () => {
    const populated = reducer(initialState, {
      type: 'event_received',
      event: spawnEvent,
    });
    const running = reducer(populated, { type: 'started' });
    const errored = reducer(running, { type: 'error', message: 'mid-run' });
    expect(errored.events).toEqual([spawnEvent]);
    expect(errored.error).toBe('mid-run');
    expect(errored.running).toBe(false);
  });

  it('exhaustive_switch_returns_state_for_every_action', () => {
    // Compile-time + runtime guard: every Action variant returns a State.
    const actions: Action[] = [
      { type: 'event_received', event: spawnEvent },
      { type: 'clear' },
      { type: 'error', message: 'x' },
      { type: 'started' },
      { type: 'completed' },
    ];
    for (const a of actions) {
      const out = reducer(initialState, a);
      expect(out).toBeDefined();
    }
  });
});
