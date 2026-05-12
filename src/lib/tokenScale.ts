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
 * Formula: `clamp(MIN, MIN + log10(tokens + 1) / DIVISOR, MAX)`.
 * With MIN=0.8, MAX=1.5, DIVISOR=6, scaling reaches max near 10^4
 * tokens. Sample points:
 *   - 0 → 0.80 (floor)
 *   - 10 → 0.97
 *   - 100 → 1.13
 *   - 1000 → 1.30
 *   - 10000 → 1.47
 *   - 50000+ → 1.50 (clamp)
 *
 * Logarithmic rather than linear so the smoke-session range (10–100
 * tokens) sits visibly mid-band rather than crowded at the floor.
 * M04 IRL: prior linear formula `1 + tokens/1000` clamped at 1.5
 * by 500 tokens, so smoke sessions (10–50 tokens) gave a
 * barely-visible 1.01–1.05 scale — looked identical to 0-token
 * agents at typical viewing zoom.
 */
const SCALE_MIN = 0.8;
const SCALE_MAX = 1.5;
const SCALE_DIVISOR = 6;

export function tokenScale(totalTokens: number): number {
  if (!Number.isFinite(totalTokens) || totalTokens <= 0) {
    return SCALE_MIN;
  }
  const raw = SCALE_MIN + Math.log10(totalTokens + 1) / SCALE_DIVISOR;
  return Math.min(SCALE_MAX, raw);
}
