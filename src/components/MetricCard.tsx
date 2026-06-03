import type { ReactNode } from 'react';

export interface MetricCardProps {
  /** Uppercase-tracked micro-label (e.g. "Tokens", "Spend"). */
  label: string;
  /** The machine value — rendered in the IBM Plex Mono tabular register. */
  value: ReactNode;
  /** Pass/fail tint on the value (DESIGN.md ok/error families). */
  tone?: 'default' | 'ok' | 'bad';
  /** Optional sub-line under the value (e.g. an in/out breakdown). */
  delta?: string;
}

/**
 * A Tester-results metric card (M08.8.B.fix; the mockup's `.metric`,
 * workbench.css). An instrument tile: an uppercase-tracked label over a
 * large IBM-Plex-Mono tabular value, with an optional tone and delta.
 * Presentational — the caller formats the value; F wires the real run data.
 */
export function MetricCard({
  label,
  value,
  tone = 'default',
  delta,
}: MetricCardProps): JSX.Element {
  const toneClass = tone === 'ok' ? ' ok' : tone === 'bad' ? ' bad' : '';
  return (
    <div className="metric" data-testid={`metric-${label.toLowerCase()}`}>
      <div className="label t-label">{label}</div>
      <div className={`value mono tnum${toneClass}`}>{value}</div>
      {delta !== undefined && <div className="delta">{delta}</div>}
    </div>
  );
}
