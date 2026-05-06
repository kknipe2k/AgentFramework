import { defineConfig, devices } from '@playwright/test';

// Stage E renderer-level E2E.
//
// Tauri 2.x E2E requires tauri-driver + WebdriverIO per the official docs
// (https://v2.tauri.app/develop/tests/webdriver/); Playwright with `_electron`
// would not work for a Tauri app. Stage E ships Playwright running against
// the Vite dev server with `@tauri-apps/api` module-mocked — covers the
// renderer state-machine + UX invariants. Full Tauri-shell E2E is a M03
// carry-forward (see docs/build-prompts/retrospectives/M02.E-retrospective.md).
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  // Sequential locally too — Vite 7's dep optimizer thrashes when 3 chromium
  // workers hit a cold `node_modules/.vite/deps` cache concurrently. Per-test
  // timeout below absorbs the cold-start cost on the first navigation; once
  // optimization completes, subsequent navigations are fast.
  workers: 1,
  // The first `page.goto` after a clean dep cache pays the optimizer cost
  // (Vite 7's Rolldown-scout warmup runs ~30–45s for the React + Tauri stack
  // on Windows). 60s leaves margin without masking real regressions.
  timeout: 60_000,
  reporter: process.env.CI ? [['html', { open: 'never' }], ['list']] : 'list',
  use: {
    // IPv4 explicit — Vite 7 binds the dev server to 127.0.0.1 (per
    // vite.config.ts host: '127.0.0.1'). Using `localhost` here resolves to
    // ::1 on Node 17+ via Chromium's DNS path and the bundled Chrome
    // can't reach the IPv4-only listener (`net::ERR_CONNECTION_REFUSED`).
    baseURL: 'http://127.0.0.1:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'npm run dev',
    url: 'http://127.0.0.1:1420',
    reuseExistingServer: !process.env.CI,
    // Cold-start dep optimization on Vite 7 + Tauri/React stack can run
    // 30–60s on a fresh `node_modules/.vite` cache (especially on Windows
    // CI runners). 30s is too tight; 90s keeps a comfortable margin.
    timeout: 90_000,
  },
});
