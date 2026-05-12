import { describe, expect, it } from 'vitest';
import { tokenScale } from '../../../src/lib/tokenScale';

describe('tokenScale', () => {
  it('zero_tokens_returns_minimum_scale', () => {
    expect(tokenScale(0)).toBeCloseTo(0.8);
  });

  it('low_token_range_scales_visibly_above_floor', () => {
    // 10 tokens (smoke-session floor) → 0.80 + log10(11)/6 ≈ 0.97.
    // Linear v1 formula returned 1.01 here — barely above floor; the
    // log10 formula sits the smoke-session range visibly mid-band.
    expect(tokenScale(10)).toBeCloseTo(0.97, 2);
  });

  it('mid_range_token_count_sits_in_mid_band', () => {
    // 100 tokens → 0.80 + log10(101)/6 ≈ 1.13.
    expect(tokenScale(100)).toBeCloseTo(1.13, 2);
    // 1000 tokens → 0.80 + log10(1001)/6 ≈ 1.30.
    expect(tokenScale(1000)).toBeCloseTo(1.3, 2);
  });

  it('large_token_count_approaches_max', () => {
    // 10000 tokens → 0.80 + log10(10001)/6 ≈ 1.47.
    expect(tokenScale(10000)).toBeCloseTo(1.47, 2);
  });

  it('very_large_token_count_clamps_at_max', () => {
    // 50000 tokens would compute ≈ 1.58 → clamps to 1.5.
    expect(tokenScale(50000)).toBeCloseTo(1.5);
    expect(tokenScale(1_000_000)).toBeCloseTo(1.5);
  });

  it('low_vs_large_token_delta_is_visible', () => {
    // M04 IRL LG-03 regression: low- and high-token agents must render
    // distinguishably. 10 tokens vs 50000 tokens should yield ≥1.5×
    // size delta so side-by-side comparison reads clearly.
    const low = tokenScale(10);
    const high = tokenScale(50000);
    expect(high / low).toBeGreaterThan(1.5);
  });

  it('negative_or_nonfinite_input_returns_minimum_scale', () => {
    expect(tokenScale(-50)).toBeCloseTo(0.8);
    expect(tokenScale(Number.NaN)).toBeCloseTo(0.8);
    expect(tokenScale(Number.POSITIVE_INFINITY)).toBeCloseTo(0.8);
  });
});
