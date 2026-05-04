import type { AgentEvent } from '../types/agent_event';

export interface State {
  readonly events: readonly AgentEvent[];
  readonly error: string | null;
  readonly running: boolean;
}

export const initialState: State = {
  events: [],
  error: null,
  running: false,
};

export type Action =
  | { type: 'event_received'; event: AgentEvent }
  | { type: 'clear' }
  | { type: 'error'; message: string }
  | { type: 'started' }
  | { type: 'completed' };

export function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'event_received':
      return { ...state, events: [...state.events, action.event] };
    case 'clear':
      return initialState;
    case 'error':
      return { ...state, error: action.message, running: false };
    case 'started':
      return { ...state, running: true, error: null };
    case 'completed':
      return { ...state, running: false };
  }
}
