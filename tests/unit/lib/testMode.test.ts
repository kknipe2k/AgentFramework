// M09.5.A (TD-050 / review C1) — unit contract for the store-exposure
// gate `shouldExposeStores` in src/lib/testMode.ts.
//
// App.tsx exposes the four Zustand stores (`window.__graphStore`,
// `__builderStore`, `__testGraphStore`, `__toastStore`) for the e2e
// harnesses — 12 e2e-tauri specs drive the BUILT binary through them,
// so a bare `import.meta.env.DEV` gate would break the merge-blocking
// e2e-tauri job. The shell-resolved seam: main.rs reads
// AGENT_RUNTIME_E2E=1 and injects an initialization script setting
// `window.__E2E__ = true`; App.tsx gates the assignments on
// `shouldExposeStores(import.meta.env.DEV, window.__E2E__)`.
//
// The adversarial contract: a production launch (not DEV, no harness
// env) exposes NOTHING — and the flag check is strict (`=== true`), so
// injected code cannot re-enable the exposure with a truthy garbage
// value it managed to plant before App evaluated.

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { shouldExposeStores } from '../../../src/lib/testMode';

describe('shouldExposeStores — the store-exposure gate (M09.5.A)', () => {
  it('exposes in DEV regardless of the e2e flag', () => {
    expect(shouldExposeStores(true, undefined)).toBe(true);
    expect(shouldExposeStores(true, true)).toBe(true);
  });

  it('exposes in a production build when the shell-resolved __E2E__ flag is exactly true', () => {
    expect(shouldExposeStores(false, true)).toBe(true);
  });

  it('does NOT expose in a production build without the flag — the bare-launch case', () => {
    expect(shouldExposeStores(false, undefined)).toBe(false);
  });

  it('rejects truthy-but-not-true flag values — the gate is strict', () => {
    expect(shouldExposeStores(false, '1')).toBe(false);
    expect(shouldExposeStores(false, 1)).toBe(false);
    expect(shouldExposeStores(false, {})).toBe(false);
    expect(shouldExposeStores(false, 'true')).toBe(false);
    expect(shouldExposeStores(false, null)).toBe(false);
  });

  // Wiring assertion (gotcha #67 class — a gate that exists but is not
  // called protects nothing): App.tsx must route its window.__*Store
  // assignments through shouldExposeStores. The unconditional
  // `window.__graphStore = useGraphStore;` block this stage removes
  // must not survive outside the gate.
  it('App.tsx routes the store exposure through shouldExposeStores', () => {
    const appSource = readFileSync(resolve(__dirname, '../../../src/App.tsx'), 'utf8');
    expect(appSource).toContain('shouldExposeStores(');
  });
});
