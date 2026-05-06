/**
 * Map a cumulative token count to a CSS transform-scale factor.
 *
 * Per spec §3 Visual Design ("Token spend shown as node weight —
 * larger spend = visually larger node"). Pure function so the
 * AgentNode + ToolNode use identical scaling and Vitest covers it
 * directly. Clamped to `[MIN, MAX]` so a runaway token count cannot
 * blow up the layout, and a 0-token node still renders at slightly
 * less than full size to leave headroom for non-zero nodes.
 *
 * Range: clamp(0.8, 1 + tokens/1000, 1.5). At 0 tokens → 0.8; at 200
 * tokens → 1.2; at 500+ tokens → 1.5 cap. Tuned for v0.1's smoke
 * session (~10–50 tokens — barely visible scaling but the mechanism
 * is wired). Stage E may revisit if persistence-replay surfaces
 * sessions with much larger ranges.
 */
const SCALE_MIN = 0.8;
const SCALE_MAX = 1.5;
const SCALE_DIVISOR = 1000;

export function tokenScale(totalTokens: number): number {
  if (!Number.isFinite(totalTokens) || totalTokens <= 0) {
    return SCALE_MIN;
  }
  const raw = 1 + totalTokens / SCALE_DIVISOR;
  return Math.max(SCALE_MIN, Math.min(SCALE_MAX, raw));
}
