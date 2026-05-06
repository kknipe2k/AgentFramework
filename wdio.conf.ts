// Tauri 2.x desktop-shell E2E config (M03.F).
//
// Runs the WebdriverIO v9 test runner against the built Tauri binary via
// `tauri-driver`. Per the official Tauri 2.x docs
// (https://v2.tauri.app/develop/tests/webdriver/), only Linux + Windows are
// supported — macOS lacks a WKWebView WebDriver tool. The CI matrix in
// `.github/workflows/ci.yml::e2e-tauri-driver` skips macOS via an `if:` guard;
// the early-exit below makes `npm run test:e2e:tauri` a no-op on macOS for
// local development on a Mac instead of a hard failure.
//
// `tauri-driver` is expected on PATH — installed in CI via `npm install -g
// @crabnebula/tauri-driver` (or `cargo install tauri-driver`). Locally,
// install once before running. The driver itself proxies WebDriver requests
// to either WebKitGTK's `WebKitWebDriver` (Linux) or `msedgedriver` (Windows,
// pre-installed on `windows-latest` GitHub runners).
//
// Tests live under `tests/e2e-tauri/` and use mocha BDD + chai assertions,
// matching the WebdriverIO v9 default. They are intentionally separate from
// the renderer-level Playwright suite at `tests/e2e/` (different test type,
// different driver, different CI job).
import { spawn, type ChildProcess } from 'node:child_process';
import { resolve } from 'node:path';

if (process.platform === 'darwin') {
  console.log('tauri-driver E2E skipped on macOS (unsupported by tauri-driver upstream).');
  process.exit(0);
}

const TAURI_DRIVER_PORT = 4444;
const APP_BIN_NAME = process.platform === 'win32' ? 'agent-runtime.exe' : 'agent-runtime';
const APP_BIN_PATH = resolve(process.cwd(), 'src-tauri', 'target', 'release', APP_BIN_NAME);

let tauriDriverProc: ChildProcess | undefined;

export const config = {
  runner: 'local' as const,
  framework: 'mocha' as const,
  mochaOpts: { ui: 'bdd' as const, timeout: 60_000 },
  reporters: ['spec' as const],
  specs: ['./tests/e2e-tauri/**/*.e2e.ts'],
  // tauri-driver does not parallelize within a single host (single
  // application process bound to one driver port). One worker, in declared
  // test order, so test 6 (reload) can rely on tests 1-5 having seeded state.
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      // The application binary tauri-driver launches and attaches to.
      'tauri:options': { application: APP_BIN_PATH },
      // Per Tauri 2.x WebDriver docs (https://v2.tauri.app/develop/tests/webdriver/),
      // capabilities must use `browserName: 'wry'` on every platform —
      // tauri-driver proxies WebDriver requests to the platform-native driver
      // (WebKitWebDriver on Linux, msedgedriver on Windows) under the hood.
      browserName: 'wry',
    },
  ],
  hostname: '127.0.0.1',
  port: TAURI_DRIVER_PORT,
  logLevel: 'info' as const,

  // Lifecycle: spawn tauri-driver before WebdriverIO connects; SIGTERM it
  // after. Per https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/.
  onPrepare(): void {
    tauriDriverProc = spawn('tauri-driver', ['--port', String(TAURI_DRIVER_PORT)], {
      stdio: 'inherit',
    });
  },

  onComplete(): void {
    tauriDriverProc?.kill('SIGTERM');
  },
};
