// M08.E — the Inspector's "Changes since save" disk diff (spec Phase 9).
// A pure function over the current framework and the last-saved disk
// snapshot; the Inspector renders the result, the builderStore test
// pins it. No store state — a standalone helper.

/** One line of the Inspector disk diff. */
export interface DiffLine {
  /** `context` — unchanged; `added` — present only in the current
   *  framework; `removed` — present only in the on-disk snapshot. */
  tag: 'context' | 'added' | 'removed';
  /** The line text (one line of the pretty-printed JSON). */
  text: string;
}

/** The Inspector disk diff — the current framework vs the disk snapshot. */
export interface FrameworkDiff {
  /** False when the framework is byte-identical to the disk snapshot. */
  changed: boolean;
  /** The line diff in display order; empty when unchanged. */
  lines: DiffLine[];
}

/**
 * Diff the current framework against the last-saved disk snapshot.
 *
 * STUB (M08.E red phase) — implemented in the green phase; throws here
 * so every disk-diff test fails for the right reason.
 */
export function diffFramework(_current: unknown, _disk: unknown): FrameworkDiff {
  throw new Error('M08.E: diffFramework is implemented in the green phase');
}
