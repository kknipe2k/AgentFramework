// Tauri 2.x desktop-shell E2E regression test — M09.5.A (TD-050 + TD-051;
// external review C1/C2, docs/code-design-review-2026-06-09.md §2).
//
// The ADVERSARIAL cases ARE the acceptance (phase doc A.4): this spec
// sends exactly what no UI ever sends — a traversal `dir` to
// save_framework/load_framework, an out-of-roots file path to
// import_artifact, an injected inline <script> — and asserts the shell
// perimeter now refuses each one with no side effect, while the
// harness seam (AGENT_RUNTIME_E2E → window.__E2E__) keeps the 12
// store-dependent e2e-tauri specs driveable against the built binary.
//
// Why this can only live as a tauri-driver real-app test (ADR-0021 /
// gotcha #82): the CSP is enforced by the real WebView2/WebKitGTK (the
// Playwright Chromium never loads tauri.conf.json), the store-exposure
// seam is resolved by the real shell process env, and the path
// confinement guards real #[tauri::command] invokes — none of which
// exist in the renderer-level suite.
//
// On the pre-fix tree this spec fails right-reason: the traversal
// invokes are ACCEPTED (save_framework create_dir_all's the escape
// target), the injected inline script EXECUTES (csp: null), and
// window.__E2E__ does not exist (no seam). That failing run is the
// review's finding reproduced (CLAUDE.md §5 red phase).
/// <reference types="mocha" />
import type {} from 'webdriverio';
import { $, browser } from '@wdio/globals';
import { expect } from 'chai';
import { existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

// Traversal target, kept INSIDE the OS temp dir so the red-phase run
// (where the hole is real and the write lands) never touches the
// machine outside a cleanable scratch area. `escapeBase` is never a
// registered root; the `..` components resolve to a sibling OUTSIDE it.
const ESCAPE_BASE = join(tmpdir(), 'agent-runtime-perimeter');
const TRAVERSAL_DIR = join(ESCAPE_BASE, 'inner', '..', '..', 'agent-runtime-escaped');
const ESCAPED_TARGET = resolve(tmpdir(), 'agent-runtime-escaped');

// One invoke wrapper shape for all refusal cases: resolve with the
// outcome instead of throwing across the WebDriver boundary, so the
// assertion can distinguish "accepted" (the pre-fix hole) from
// "refused with the typed error" (the contract). The caught error is
// flattened to plain primitives (errorType / errorMessage) BEFORE the
// return — returning the raw Tauri error object trips WebView2's
// execute/sync structured-clone serializer ("WebDriverError: [object
// Object]"), masking the assertion.
function invokeOutcomeScript(command: string): string {
  return `
    return (async () => {
      const args = arguments[0];
      try {
        await window.__TAURI_INTERNALS__.invoke('${command}', args);
        return { outcome: 'accepted', errorType: null, errorMessage: null };
      } catch (e) {
        const t = e && typeof e === 'object' ? e.type : null;
        const m = e && typeof e === 'object' ? String(e.message) : String(e);
        return { outcome: 'refused', errorType: t === undefined ? null : t, errorMessage: m };
      }
    })();
  `;
}

interface InvokeOutcome {
  outcome: 'accepted' | 'refused';
  errorType: string | null;
  errorMessage: string | null;
}

describe('Shell perimeter rejects what no UI ever sends (M09.5.A real-app)', () => {
  before(async () => {
    // App ready: the ViewSwitch mounts unconditionally after first paint
    // (key-independent — the builder_drag.e2e.ts precedent).
    await $('[data-testid="view-switch-builder"]').waitForDisplayed({ timeout: 10_000 });
  });

  after(() => {
    rmSync(ESCAPE_BASE, { recursive: true, force: true });
    rmSync(ESCAPED_TARGET, { recursive: true, force: true });
  });

  it('save_framework_with_a_traversal_dir_is_refused_and_writes_nothing', async () => {
    const result = await browser.execute<InvokeOutcome, [{ dir: string }]>(
      `
      return (async () => {
        const dir = arguments[0].dir;
        const framework = window.__builderStore.getState().framework;
        try {
          await window.__TAURI_INTERNALS__.invoke('save_framework', {
            dir: dir,
            framework: framework,
            companions: [],
          });
          return { outcome: 'accepted', errorType: null, errorMessage: null };
        } catch (e) {
          const t = e && typeof e === 'object' ? e.type : null;
          const m = e && typeof e === 'object' ? String(e.message) : String(e);
          return { outcome: 'refused', errorType: t === undefined ? null : t, errorMessage: m };
        }
      })();
      `,
      { dir: TRAVERSAL_DIR },
    );

    expect(result.outcome, 'a traversal save dir must be refused').to.equal('refused');
    expect(result.errorType, 'the refusal must be the typed path_not_permitted error').to.equal(
      'path_not_permitted',
    );
    expect(
      existsSync(ESCAPED_TARGET),
      `nothing may be written outside the registered roots (found ${ESCAPED_TARGET})`,
    ).to.equal(false);
  });

  it('load_framework_with_a_relative_traversal_dir_is_refused_with_the_typed_error', async () => {
    const result = await browser.execute<InvokeOutcome, [{ dir: string }]>(
      invokeOutcomeScript('load_framework'),
      { dir: '../../agent-runtime-escaped-load' },
    );

    expect(result.outcome, 'a traversal load dir must be refused').to.equal('refused');
    expect(result.errorType, 'the refusal must be the typed path_not_permitted error').to.equal(
      'path_not_permitted',
    );
  });

  it('import_artifact_file_source_outside_registered_roots_is_refused', async () => {
    const result = await browser.execute<InvokeOutcome, [Record<string, string>]>(
      invokeOutcomeScript('import_artifact'),
      {
        sourceKind: 'file',
        location: join(tmpdir(), 'agent-runtime-no-such-artifact.skill.md'),
        artifactKind: 'skill',
      },
    );

    expect(result.outcome, 'an out-of-roots file import must be refused').to.equal('refused');
    expect(
      result.errorType,
      'the refusal must be the typed path_not_permitted error (confinement BEFORE any IO)',
    ).to.equal('path_not_permitted');
  });

  it('an_injected_inline_script_is_inert_under_the_csp', async () => {
    // Dynamically-appended inline <script> executes synchronously on
    // insertion — unless the CSP forbids it (script-src 'self', no
    // unsafe-inline). The probe's side effect must never appear.
    const probe = await browser.execute<string | null, []>(`
      var s = document.createElement('script');
      s.textContent = 'window.__cspProbe = "executed";';
      document.body.appendChild(s);
      return window.__cspProbe === undefined ? null : window.__cspProbe;
    `);

    expect(probe, 'the injected inline script must NOT execute under the CSP').to.equal(null);
  });

  it('the_harness_seam_is_shell_resolved_and_the_stores_stay_driveable', async () => {
    // Under the harness (wdio exports AGENT_RUNTIME_E2E=1; tauri-driver's
    // app child inherits it) the shell's initialization script sets
    // window.__E2E__ = true and App.tsx exposes the four stores — the
    // regression canary for the 12 store-dependent specs. The
    // bare-launch counterpart (stores undefined WITHOUT the env) is
    // asserted per phase doc A.3 step 2's landed form.
    const seam = await browser.execute<{ e2e: boolean; stores: string[] }, []>(`
      return {
        e2e: window.__E2E__ === true,
        stores: ['__graphStore', '__builderStore', '__testGraphStore', '__toastStore']
          .filter(function (k) { return typeof window[k] !== 'undefined'; }),
      };
    `);

    expect(seam.e2e, 'window.__E2E__ must be exactly true under the harness env').to.equal(true);
    expect(seam.stores, 'all four stores must stay driveable under the seam').to.have.members([
      '__graphStore',
      '__builderStore',
      '__testGraphStore',
      '__toastStore',
    ]);
  });
});
