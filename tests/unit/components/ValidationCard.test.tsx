import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';

// M08.8.B.fix — the validation error surface (the mockup's `.err-card`;
// workbench.css:316-320; DESIGN.md rules 6+7). A plain-English cause, a
// `→ fix` suggestion, and a progressive "Show raw error" disclosure that
// keeps the raw validator output one click away (never silently dropped,
// never dumped by default). Styled here with stub content; F wires the
// real validator output.
import { ValidationCard } from '../../../src/components/ValidationCard';

describe('ValidationCard', () => {
  it('renders_the_plain_english_cause', () => {
    render(<ValidationCard plain="The framework has no agents." />);
    const card = screen.getByTestId('validation-card');
    expect(card).toHaveClass('err-card');
    expect(card.querySelector('.err-plain')).toHaveTextContent('The framework has no agents.');
  });

  it('renders_the_fix_suggestion_with_an_arrow', () => {
    render(<ValidationCard plain="No agents." fix="Add at least one agent node." />);
    const fix = screen.getByTestId('validation-card').querySelector('.err-fix');
    expect(fix).not.toBeNull();
    expect(fix?.textContent).toContain('→');
    expect(fix).toHaveTextContent('Add at least one agent node.');
  });

  it('keeps_the_raw_error_behind_a_disclosure_collapsed_by_default', () => {
    render(<ValidationCard plain="Invalid JSON." raw="Unexpected token } at position 12" />);
    const toggle = screen.getByTestId('validation-show-raw');
    expect(toggle).toHaveTextContent('Show raw error');
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    // collapsed — the raw output is not in the DOM until asked for.
    expect(screen.queryByTestId('validation-raw')).toBeNull();
  });

  it('reveals_the_raw_error_when_the_disclosure_is_opened', () => {
    render(<ValidationCard plain="Invalid JSON." raw="Unexpected token } at position 12" />);
    fireEvent.click(screen.getByTestId('validation-show-raw'));
    const raw = screen.getByTestId('validation-raw');
    expect(raw).toHaveClass('err-raw');
    expect(raw).toHaveTextContent('Unexpected token } at position 12');
    expect(screen.getByTestId('validation-show-raw')).toHaveAttribute('aria-expanded', 'true');
  });

  it('omits_the_disclosure_entirely_when_there_is_no_raw_error', () => {
    render(<ValidationCard plain="No agents." fix="Add one." />);
    expect(screen.queryByTestId('validation-show-raw')).toBeNull();
  });

  it('styles_css_defines_the_err_card_rules', () => {
    // gotcha #67 — the err-card is a NEW surface this stage adds.
    const css = readFileSync(resolve(__dirname, '../../../src/styles.css'), 'utf8');
    expect(css).toMatch(/\.err-card[\s,{]/);
    expect(css).toMatch(/\.err-plain[\s,{]/);
    expect(css).toMatch(/\.err-fix[\s,{]/);
    expect(css).toMatch(/\.err-raw[\s,{]/);
  });
});
