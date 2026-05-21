/** The top-level app view — the live-execution Runtime view or the
 *  build-time Builder view (spec §0d runtime + build modes). */
export type AppView = 'runtime' | 'builder';

export interface ViewSwitchProps {
  value: AppView;
  onChange: (view: AppView) => void;
}

// M08.C red-phase stub — green renders the Runtime / Builder options.
export function ViewSwitch(_props: ViewSwitchProps): JSX.Element {
  return <div data-testid="view-switch" />;
}
