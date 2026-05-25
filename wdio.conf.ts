// Tauri 2.x desktop-shell E2E config (M03.F; M08.5 + M08.5.5 hardening).
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
//
// .env.local loader (M08.5.5 Stage A.fix; drafted on review branch
// 0239804). Allows local devs to set ANTHROPIC_TEST_KEY +
// ANTHROPIC_API_KEY without committing them. File is gitignored via
// `.gitignore:38` (.env.*). CI uses the ANTHROPIC_TEST_KEY GitHub
// secret directly; the local-loader path complements that. Pair with
// the ADR-0025 env-var override in `crates/runtime-main/src/key_store.rs`
// so a key in .env.local outranks any stale OS-keychain placeholder
// the M08.5 smoke-test #2 may have saved.
import { spawn, spawnSync, type ChildProcess } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

if (existsSync('.env.local')) {
  for (const line of readFileSync('.env.local', 'utf8').split('\n')) {
    const trimmed = line.trim();
    if (trimmed.length === 0 || trimmed.startsWith('#')) continue;
    const eq = trimmed.indexOf('=');
    if (eq <= 0) continue;
    const key = trimmed.slice(0, eq).trim();
    const value = trimmed
      .slice(eq + 1)
      .trim()
      .replace(/^['"]|['"]$/g, '');
    process.env[key] = value;
  }
}

if (process.platform === 'darwin') {
  console.log('tauri-driver E2E skipped on macOS (unsupported by tauri-driver upstream).');
  process.exit(0);
}

const TAURI_DRIVER_PORT = 4444;
const APP_BIN_NAME = process.platform === 'win32' ? 'agent-runtime.exe' : 'agent-runtime';
// `src-tauri` is a member of the Cargo workspace rooted at the repo root
// (root `Cargo.toml` `members = [.., "src-tauri"]`), so `cargo` / `tauri
// build` emit the binary to the SHARED workspace target dir at the repo
// root — `target/release/`, NOT `src-tauri/target/release/`. Handing
// tauri-driver the latter is the M03 PR #47 Linux failure ("could not
// exec the app binary"). `process.cwd()` is the repo root: wdio runs from
// there via the `test:e2e:tauri` npm script.
const APP_BIN_PATH = resolve(process.cwd(), 'target', 'release', APP_BIN_NAME);

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
      // Per the official Tauri 2.x WebDriver example
      // (https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/Tests/WebDriver/Example/webdriverio.mdx),
      // the capabilities object intentionally OMITS `browserName` — tauri-driver
      // constructs the native value when proxying to the platform driver
      // (WebKitWebDriver on Linux, msedgedriver against WebView2 on Windows).
      // Setting `browserName` here breaks the match: Linux returns "Failed to
      // match capabilities" from POST /session, Windows returns "no msedge
      // binary at <APP_BIN_PATH>" because msedgedriver tries to launch the
      // application as if it were Edge.
      'tauri:options': { application: APP_BIN_PATH },
    },
  ],
  hostname: '127.0.0.1',
  port: TAURI_DRIVER_PORT,
  logLevel: 'info' as const,

  // Lifecycle: build the Tauri app + sibling subprocess binaries
  // (idempotent — both invocations are no-ops when already up-to-date),
  // then spawn tauri-driver before WebdriverIO connects; SIGTERM it
  // after. Per https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/.
  //
  // M08.5.5 Stage A.fix: the build sequence closes the
  // missing-sibling-subprocesses install-pain — `npx tauri build`
  // only emits `agent-runtime.exe`, but the app spawns
  // `runtime-drone` + `runtime-sandbox` as subprocesses at startup
  // and crashes immediately if they are absent from the workspace-
  // root `target/release/` directory. CI builds them in two named
  // steps; the local harness now does the same in one shot here.
  //
  // The two invocations exist for a Tauri compile-mode reason:
  // `npx tauri build` (NOT `cargo build -p agent-runtime`) is what
  // produces a release binary that serves the bundled `dist/`
  // assets — running raw `cargo build --release -p agent-runtime`
  // omits the Tauri build env vars and the resulting binary tries
  // to load the renderer from `tauri.conf.json`'s `devUrl`
  // (`http://localhost:1420`), which is unreachable at e2e-run
  // time. The sibling crates are plain bins and `cargo build`
  // suffices for them. This matches the CI `e2e-tauri-driver` job
  // (.github/workflows/ci.yml:723 + :747).
  onPrepare(): void {
    console.log('M08.5.5 wdio.conf.ts: building agent-runtime via tauri (idempotent)…');
    // `shell: true` routes through cmd.exe on Windows so the `npx.cmd`
    // batch shim resolves correctly. Direct invocation of `npx` from a
    // Node `spawn` produces `exit null` (signal-terminated) on Windows
    // because `spawn` does not interpret `.cmd` extensions without a
    // shell. Same pattern matches the CI step `run: npx tauri build
    // --no-bundle` (which runs in `bash` on Linux + `bash` on Windows
    // GHA runners, where shell-interpretation is automatic).
    const tauriBuild = spawnSync('npx tauri build --no-bundle', {
      stdio: 'inherit',
      shell: true,
    });
    if (tauriBuild.status !== 0) {
      throw new Error(`npx tauri build --no-bundle failed with exit ${String(tauriBuild.status)}`);
    }
    console.log('M08.5.5 wdio.conf.ts: building runtime-drone + runtime-sandbox (release)…');
    const cargoBuild = spawnSync(
      'cargo build --release -p runtime-drone -p runtime-sandbox --bins',
      { stdio: 'inherit', shell: true },
    );
    if (cargoBuild.status !== 0) {
      throw new Error(`cargo build --release failed with exit ${String(cargoBuild.status)}`);
    }
    tauriDriverProc = spawn('tauri-driver', ['--port', String(TAURI_DRIVER_PORT)], {
      stdio: 'inherit',
    });
  },

  onComplete(): void {
    tauriDriverProc?.kill('SIGTERM');
  },
};
