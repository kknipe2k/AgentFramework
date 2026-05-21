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
 * Diff two pretty-printed JSON renderings line by line.
 *
 * Not a full LCS: it trims the common prefix and the common suffix and
 * reports the divergent middle as removed-then-added. A Builder edit is
 * localized (the canvas changed one agent / one field) and the
 * framework document has a stable key order (serde) — so prefix/suffix
 * trimming yields the tight changed block rather than a whole-document
 * re-listing, which is exactly what "Changes since save" should show.
 */
function lineDiff(before: string[], after: string[]): DiffLine[] {
  let start = 0;
  while (start < before.length && start < after.length && before[start] === after[start]) {
    start += 1;
  }
  let endBefore = before.length;
  let endAfter = after.length;
  while (endBefore > start && endAfter > start && before[endBefore - 1] === after[endAfter - 1]) {
    endBefore -= 1;
    endAfter -= 1;
  }
  const lines: DiffLine[] = [];
  for (const text of before.slice(0, start)) {
    lines.push({ tag: 'context', text });
  }
  for (const text of before.slice(start, endBefore)) {
    lines.push({ tag: 'removed', text });
  }
  for (const text of after.slice(start, endAfter)) {
    lines.push({ tag: 'added', text });
  }
  for (const text of after.slice(endAfter)) {
    lines.push({ tag: 'context', text });
  }
  return lines;
}

/**
 * Diff the current framework against the last-saved disk snapshot for
 * the Inspector's "Changes since save" section (M08.E — spec Phase 9).
 * Compares the two pretty-printed JSON renderings; `changed` is false
 * when they are byte-identical (the disk diff zeroes after a save).
 */
export function diffFramework(current: unknown, disk: unknown): FrameworkDiff {
  const before = JSON.stringify(disk, null, 2);
  const after = JSON.stringify(current, null, 2);
  if (before === after) {
    return { changed: false, lines: [] };
  }
  return { changed: true, lines: lineDiff(before.split('\n'), after.split('\n')) };
}
