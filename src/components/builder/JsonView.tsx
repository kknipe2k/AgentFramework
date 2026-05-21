// M08.E — the Canvas | JSON binding's JSON tab (spec Phase 9 / MVP §M8
// criterion 6).
//
// STUB (M08.E red phase) — the raw-JSON editor + the parse-and-route
// logic + the invalid-JSON no-desync guard are implemented in the green
// phase. The placeholder mounts the landmark so the test files compile.

/** The Canvas | JSON binding's JSON tab — implemented in the green phase. */
export function JsonView(): JSX.Element {
  return <div className="json-view" data-testid="builder-json-view" />;
}
