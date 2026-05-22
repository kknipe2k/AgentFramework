/** The top-level app view — the live-execution Runtime view or the
 *  build-time Builder view (spec §0d runtime + build modes). */
export type AppView = 'runtime' | 'builder';

export interface ViewSwitchProps {
  value: AppView;
  onChange: (view: AppView) => void;
}

const VIEWS: readonly { id: AppView; label: string }[] = [
  { id: 'runtime', label: 'Runtime' },
  { id: 'builder', label: 'Builder' },
];

/**
 * The Runtime <-> Builder top-level view toggle (M08.C — App chrome).
 *
 * A controlled component: App.tsx owns the `view` state; ViewSwitch
 * renders the current value and reports a chosen view via `onChange`.
 * It sits above the view conditional so it is reachable in both modes.
 */
export function ViewSwitch({ value, onChange }: ViewSwitchProps): JSX.Element {
  return (
    <div className="view-switch" role="tablist" aria-label="Runtime or Builder view">
      {VIEWS.map((v) => (
        <button
          key={v.id}
          type="button"
          role="tab"
          aria-selected={v.id === value}
          className={`view-switch__option${v.id === value ? ' view-switch__option--active' : ''}`}
          data-testid={`view-switch-${v.id}`}
          onClick={() => onChange(v.id)}
        >
          {v.label}
        </button>
      ))}
    </div>
  );
}
