// M09.5.A (TD-050 / external review C1) — the store-exposure gate.
//
// App.tsx exposes the four Zustand stores on `window` so the e2e
// harnesses can drive runtime state (12 e2e-tauri specs drive the BUILT
// binary through them, 9 Playwright specs through the dev server). That
// exposure is a typed write path into runtime state for any injected
// script, so it must be OFF in a production launch. A bare
// `import.meta.env.DEV` gate would break the merge-blocking
// e2e-tauri-driver job (it runs the production build), so the seam is
// shell-resolved: the Tauri shell sets `window.__E2E__ = true` (via a
// js_init_script plugin) only when launched with AGENT_RUNTIME_E2E=1.

/**
 * Whether App.tsx should expose the Zustand stores on `window`.
 *
 * Exposes when the build is a Vite dev build (`import.meta.env.DEV`) OR
 * the shell resolved the e2e test mode (`window.__E2E__ === true`). The
 * `=== true` check is strict on purpose: injected code that managed to
 * plant a truthy `window.__E2E__` before App evaluated cannot re-enable
 * the exposure with a garbage value.
 *
 * @param isDev `import.meta.env.DEV` — true under the Vite dev server.
 * @param e2eFlag `window.__E2E__` — set to `true` by the shell only when
 *   AGENT_RUNTIME_E2E=1; `undefined` in every production launch.
 */
export function shouldExposeStores(isDev: boolean, e2eFlag: unknown): boolean {
  return isDev || e2eFlag === true;
}
