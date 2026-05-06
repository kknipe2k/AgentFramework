import { describe, expect, it } from 'vitest';
import { tokenScale } from '../../../src/lib/tokenScale';

describe('tokenScale', () => {
  it('zero_tokens_returns_minimum_scale', () => {
    expect(tokenScale(0)).toBeCloseTo(0.8);
  });

  it('mid_range_token_count_scales_linearly_in_band', () => {
    // 200 tokens → 1 + 200/1000 = 1.2 (within [0.8, 1.5]).
    expect(tokenScale(200)).toBeCloseTo(1.2);
  });

  it('large_token_count_clamps_at_max', () => {
    // 10000 tokens would compute 11 → clamps to 1.5.
    expect(tokenScale(10000)).toBeCloseTo(1.5);
  });

  it('negative_or_nonfinite_input_returns_minimum_scale', () => {
    expect(tokenScale(-50)).toBeCloseTo(0.8);
    expect(tokenScale(Number.NaN)).toBeCloseTo(0.8);
    expect(tokenScale(Number.POSITIVE_INFINITY)).toBeCloseTo(0.8);
  });
});
