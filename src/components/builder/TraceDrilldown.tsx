import { useState } from 'react';
import { CapabilityDisclosure } from '../CapabilityDisclosure';
import { RawDisclosure } from '../RawDisclosure';
import { formatPayload } from '../../lib/formatPayload';
import type {
  AgentEvent,
  CapabilityViolation,
  TierViolation,
  ToolError,
  ToolInvoked,
  ToolResult,
} from '../../types/agent_event';

/**
 * One drillable step folded out of a Tester run's `outcome.trace`:
 *
 *  - `tool` — a `tool_invoked` paired with its `tool_result` / `tool_error`.
 *  - `tier` — an L4 `tier_violation` (the user's tier forbade the action).
 *  - `capability` — an L2 `capability_violation` (a scope defect).
 */
export type TraceStep =
  | {
      kind: 'tool';
      invoked: ToolInvoked;
      result: ToolResult | null;
      error: ToolError | null;
      key: string;
    }
  | { kind: 'tier'; event: TierViolation; key: string }
  | { kind: 'capability'; event: CapabilityViolation; key: string };

/**
 * Fold a raw `AgentEvent[]` trace into the ordered drillable steps the
 * Tester result surfaces. Each `tool_invoked` is paired with the next
 * not-yet-consumed `tool_result` / `tool_error` for the same agent + tool;
 * tier and capability violations become distinct steps. Session / agent
 * lifecycle events are not drillable (the graph pane already renders them)
 * and are dropped.
 */
export function foldTrace(trace: AgentEvent[]): TraceStep[] {
  const steps: TraceStep[] = [];
  const consumed = new Set<number>();
  trace.forEach((event, index) => {
    if (event.type === 'tool_invoked') {
      let result: ToolResult | null = null;
      let error: ToolError | null = null;
      for (let j = index + 1; j < trace.length; j += 1) {
        if (consumed.has(j)) {
          continue;
        }
        const candidate = trace[j];
        const matches =
          candidate !== undefined &&
          (candidate.type === 'tool_result' || candidate.type === 'tool_error') &&
          candidate.agent_id === event.agent_id &&
          candidate.tool_name === event.tool_name;
        if (matches) {
          consumed.add(j);
          if (candidate.type === 'tool_result') {
            result = candidate;
          } else {
            error = candidate;
          }
          break;
        }
      }
      steps.push({ kind: 'tool', invoked: event, result, error, key: `tool-${index}` });
    } else if (event.type === 'tier_violation') {
      steps.push({ kind: 'tier', event, key: `tier-${index}` });
    } else if (event.type === 'capability_violation') {
      steps.push({ kind: 'capability', event, key: `cap-${index}` });
    }
  });
  return steps;
}

/** A tool-call row: collapsed to name + status, expands to input/result + raw. */
function ToolStep({
  invoked,
  result,
  error,
}: {
  invoked: ToolInvoked;
  result: ToolResult | null;
  error: ToolError | null;
}): JSX.Element {
  const [open, setOpen] = useState(false);
  const status = error !== null ? 'error' : result !== null ? 'ok' : 'pending';
  const output = result !== null ? result.output : error !== null ? error.error : undefined;
  const rawEvents = [invoked, result ?? error].filter(
    (e): e is ToolInvoked | ToolResult | ToolError => e !== null,
  );
  return (
    <li className="trace-step trace-step--tool" data-testid="trace-step">
      <button
        type="button"
        className={`trace-step__summary trace-step__summary--${status}`}
        aria-expanded={open}
        data-testid="trace-step-toggle"
        onClick={() => setOpen((v) => !v)}
      >
        <span className="trace-step__name">{invoked.tool_name}</span>
        <span className="trace-step__status">{status}</span>
      </button>
      {open && (
        <div className="trace-step__detail" data-testid="trace-step-detail">
          <h4 className="trace-step__label">Input</h4>
          <pre className="trace-step__io" data-testid="trace-step-input">
            {formatPayload(invoked.input)}
          </pre>
          <h4 className="trace-step__label">Result</h4>
          <pre className="trace-step__io" data-testid="trace-step-output">
            {formatPayload(output)}
          </pre>
          <RawDisclosure
            raw={JSON.stringify(rawEvents, null, 2)}
            showLabel="Show raw event"
            hideLabel="Hide raw event"
            toggleTestId="trace-step-raw-toggle"
            rawTestId="trace-step-raw"
            wrapClassName="trace-step__raw-wrap"
            toggleClassName="trace-step__raw-toggle"
            rawClassName="trace-step__raw"
          />
        </div>
      )}
    </li>
  );
}

/** A tier-block row — distinct, linking to the tier explainer (the Promote
 *  control surfaced in the result's tier-blocks section). */
function TierStep({ event }: { event: TierViolation }): JSX.Element {
  return (
    <li className="trace-step trace-step--tier" data-testid="trace-step">
      <div className="trace-step__violation">
        <span className="trace-step__name">Tier-limited</span>
        <span className="trace-step__detail-text">
          <code>{event.agent_id}</code> — {event.capability_kind}: {event.attempted_action}
        </span>
        <a
          className="trace-step__explainer"
          href="#tester-tier-blocks"
          data-testid="trace-step-tier-explainer"
        >
          Why? Your tier blocked this
        </a>
      </div>
    </li>
  );
}

/** A capability-violation row — distinct, with the scope mismatch disclosed
 *  through the shared CapabilityDisclosure surface (its explainer). */
function CapabilityStep({ event }: { event: CapabilityViolation }): JSX.Element {
  return (
    <li className="trace-step trace-step--capability" data-testid="trace-step">
      <div className="trace-step__violation">
        <span className="trace-step__name">Capability violation</span>
        <CapabilityDisclosure
          capabilities={[
            `${event.capability_kind}: ${event.requested_action} (declared scope ${event.declared_scope})`,
          ]}
          emptyMessage="No capability detail."
          data-testid="trace-step-cap-explainer"
        />
      </div>
    </li>
  );
}

/**
 * The Tester run drill-down (M08.9.B): renders `outcome.trace` as a step
 * list under the verdict — each tool call expands to its input/result
 * payload (the M08.8.A Output-rail formatter) and a Show-raw disclosure
 * (the shared {@link RawDisclosure}) reveals the raw event; tier and
 * capability violations are distinct rows linking to their explainer.
 * Pure disclosure — no run change, no backend. Renders nothing when the
 * trace has no drillable steps.
 */
export function TraceDrilldown({ trace }: { trace: AgentEvent[] }): JSX.Element | null {
  const steps = foldTrace(trace);
  if (steps.length === 0) {
    return null;
  }
  return (
    <section className="trace-drilldown" data-testid="trace-drilldown">
      <h3 className="trace-drilldown__label">Run trace</h3>
      <ol className="trace-drilldown__list">
        {steps.map((step) => {
          if (step.kind === 'tool') {
            return (
              <ToolStep
                key={step.key}
                invoked={step.invoked}
                result={step.result}
                error={step.error}
              />
            );
          }
          if (step.kind === 'tier') {
            return <TierStep key={step.key} event={step.event} />;
          }
          return <CapabilityStep key={step.key} event={step.event} />;
        })}
      </ol>
    </section>
  );
}
