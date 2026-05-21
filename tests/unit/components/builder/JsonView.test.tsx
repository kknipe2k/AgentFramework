import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { JsonView } from '../../../../src/components/builder/JsonView';
import { emptyFramework, useBuilderStore } from '../../../../src/lib/builderStore';
import type { Framework } from '../../../../src/types/framework';

// M08.E — the Canvas | JSON two-way binding's JSON tab (spec Phase 9 /
// MVP §M8 criterion 6). The JSON tab is just another editor over
// builderStore.framework, exactly as the canvas is (ADR-0020): a valid
// edit routes through replaceFramework and the canvas re-derives; an
// invalid (half-typed / malformed) edit surfaces an inline parse error
// and leaves the store UNTOUCHED — the load-bearing no-desync guard.

function namedFramework(name: string): Framework {
  return { ...emptyFramework(), name };
}

describe('JsonView', () => {
  beforeEach(() => {
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('renders_the_current_framework_as_pretty_printed_json', () => {
    render(<JsonView />);
    expect(screen.getByTestId('builder-json-textarea')).toHaveValue(
      JSON.stringify(emptyFramework(), null, 2),
    );
  });

  it('a_valid_json_edit_calls_replaceFramework_with_the_parsed_document', () => {
    const replaceFramework = vi.fn();
    useBuilderStore.setState({ replaceFramework });
    render(<JsonView />);
    const edited = JSON.stringify(namedFramework('json-edited'), null, 2);
    fireEvent.change(screen.getByTestId('builder-json-textarea'), {
      target: { value: edited },
    });
    expect(replaceFramework).toHaveBeenCalledWith(expect.objectContaining({ name: 'json-edited' }));
  });

  it('an_invalid_json_edit_does_NOT_call_replaceFramework', () => {
    // The no-desync guard: a malformed edit must never reach
    // replaceFramework, or the canvas desyncs from the store.
    const replaceFramework = vi.fn();
    useBuilderStore.setState({ replaceFramework });
    render(<JsonView />);
    fireEvent.change(screen.getByTestId('builder-json-textarea'), {
      target: { value: '{ not valid json' },
    });
    expect(replaceFramework).not.toHaveBeenCalled();
  });

  it('an_invalid_json_edit_surfaces_an_inline_parse_error', () => {
    render(<JsonView />);
    fireEvent.change(screen.getByTestId('builder-json-textarea'), {
      target: { value: '{ not valid json' },
    });
    expect(screen.getByTestId('builder-json-error')).toBeInTheDocument();
  });

  it('a_canvas_edit_re_seeds_the_json_draft_from_the_store', () => {
    // Canvas → JSON: a framework change from elsewhere (a canvas edit)
    // re-seeds the JSON draft so the tab reflects the source of truth.
    render(<JsonView />);
    act(() => {
      useBuilderStore.getState().replaceFramework(namedFramework('from-the-canvas'));
    });
    expect(screen.getByTestId('builder-json-textarea')).toHaveValue(
      JSON.stringify(namedFramework('from-the-canvas'), null, 2),
    );
  });

  it('recovering_from_invalid_to_valid_json_clears_the_parse_error_and_calls_replaceFramework', () => {
    const replaceFramework = vi.fn();
    useBuilderStore.setState({ replaceFramework });
    render(<JsonView />);
    const textarea = screen.getByTestId('builder-json-textarea');
    fireEvent.change(textarea, { target: { value: '{ broken' } });
    expect(screen.getByTestId('builder-json-error')).toBeInTheDocument();
    expect(replaceFramework).not.toHaveBeenCalled();
    fireEvent.change(textarea, {
      target: { value: JSON.stringify(namedFramework('recovered'), null, 2) },
    });
    expect(screen.queryByTestId('builder-json-error')).not.toBeInTheDocument();
    expect(replaceFramework).toHaveBeenCalledWith(expect.objectContaining({ name: 'recovered' }));
  });
});
