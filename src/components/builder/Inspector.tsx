// M08.E — the Builder Inspector (spec Phase 9 right sidebar).
//
// STUB (M08.E red phase) — the four sections + Validate/Test/Save/Load
// are implemented in the green phase. The placeholder below mounts the
// component landmark so the test files compile; the Inspector behavior
// tests fail for the right reason against it.

/** The Builder Inspector — implemented in the M08.E green phase. */
export function Inspector(): JSX.Element {
  return <aside className="builder-inspector" data-testid="builder-inspector" />;
}
