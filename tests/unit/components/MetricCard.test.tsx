import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';

// M08.8.B.fix — the Tester-results metric card (the mockup's `.metric`
// surface; workbench.css:322-329). A small instrument card: an
// uppercase-tracked micro-label over a big IBM-Plex-Mono tabular value,
// with an optional pass/fail tone and a delta sub-line. Styled here with
// stub/static content; F wires the real run data.
import { MetricCard } from '../../../src/components/MetricCard';

describe('MetricCard', () => {
  it('renders_the_label_and_the_value', () => {
    render(<MetricCard label="Tokens" value="165" />);
    const card = screen.getByTestId('metric-tokens');
    expect(card).toHaveClass('metric');
    expect(card).toHaveTextContent('Tokens');
    expect(card).toHaveTextContent('165');
  });

  it('renders_the_value_in_the_mono_tabular_instrument_register', () => {
    // The machine value reads in IBM Plex Mono with tabular figures — the
    // instrument register the whole design hangs on.
    render(<MetricCard label="Spend" value="$0.0042" />);
    const value = screen.getByTestId('metric-spend').querySelector('.value');
    expect(value).not.toBeNull();
    expect(value).toHaveClass('mono');
    expect(value).toHaveClass('tnum');
  });

  it('applies_the_ok_tone_class_for_a_passing_metric', () => {
    render(<MetricCard label="Result" value="PASS" tone="ok" />);
    expect(screen.getByTestId('metric-result').querySelector('.value')).toHaveClass('ok');
  });

  it('applies_the_bad_tone_class_for_a_failing_metric', () => {
    render(<MetricCard label="Result" value="FAIL" tone="bad" />);
    expect(screen.getByTestId('metric-result').querySelector('.value')).toHaveClass('bad');
  });

  it('renders_an_optional_delta_sub_line', () => {
    render(<MetricCard label="Tokens" value="165" delta="in 120 · out 45" />);
    const delta = screen.getByTestId('metric-tokens').querySelector('.delta');
    expect(delta).not.toBeNull();
    expect(delta).toHaveTextContent('in 120 · out 45');
  });

  it('omits_the_delta_when_not_given', () => {
    render(<MetricCard label="Verify" value="OK" />);
    expect(screen.getByTestId('metric-verify').querySelector('.delta')).toBeNull();
  });

  it('styles_css_defines_the_metric_card_rules', () => {
    // gotcha #67 — a class with no CSS rule renders unstyled. The metric
    // grid + card + mono-tabular value are NEW surfaces this stage adds.
    const css = readFileSync(resolve(__dirname, '../../../src/styles.css'), 'utf8');
    expect(css).toMatch(/\.metrics[\s,{]/);
    expect(css).toMatch(/\.metric[\s,{]/);
    expect(css).toMatch(/\.metric\s+\.value/);
    // the value carries tabular figures (the instrument register).
    expect(css).toMatch(/\.metric\s+\.value[\s\S]*?tabular-nums/);
  });
});
