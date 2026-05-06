import type { AgentEvent } from '../types/agent_event';

interface Props {
  events: readonly AgentEvent[];
}

export function EventList({ events }: Props): JSX.Element {
  return (
    <ul aria-label="agent events">
      {events.map((event, idx) => (
        <li key={idx} data-event-type={event.type}>
          <strong>{event.type}</strong> {renderSummary(event)}
        </li>
      ))}
    </ul>
  );
}

function renderSummary(event: AgentEvent): string {
  switch (event.type) {
    case 'agent_spawned':
      return `agent ${event.agent_id}`;
    case 'agent_complete':
      return `result: ${event.result}`;
    case 'agent_error':
      return `error: ${event.error}`;
    case 'tool_invoked':
      return `${event.tool_name} (${event.source})`;
    case 'tool_result':
      return `${event.tool_name} (${event.duration_ms}ms)`;
    case 'stream_text':
      return event.text;
    case 'decision_record':
      return event.decision;
    case 'session_start':
      return `session ${event.session_id}`;
    case 'task_started':
      return `task ${event.task_id}`;
    case 'task_completed':
      return `task ${event.task_id} (${event.duration_ms}ms)`;
    default:
      // Variants beyond the M02 smoke-session subset surface as the bare
      // `event.type` label until M03 Stage B replaces this list with the
      // React Flow Canvas + per-node renderers.
      return '';
  }
}
