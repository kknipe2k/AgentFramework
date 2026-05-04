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
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? [['html', { open: 'never' }], ['list']] : 'list',
  use: {
    baseURL: 'http://localhost:1420',
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
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
