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
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';

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

// M09.5.A (TD-050) — the store-exposure test-mode seam. The built
// production binary gates `window.__*Store` on `window.__E2E__`, which
// main.rs's e2e-seam plugin sets ONLY when launched with
// AGENT_RUNTIME_E2E=1. Set it here at the top of the runner so it is in
// `process.env` before onPrepare spawns tauri-driver: tauri-driver
// inherits the runner env, and the app process tauri-driver launches
// inherits tauri-driver's env in turn — so the flag reaches the app. The
// 12 store-dependent e2e-tauri specs (and perimeter_paths' seam
// assertion) depend on this; without it the production build exposes no
// stores and they fail (the intended production behavior — see
// perimeter_paths' bare-launch contract).
process.env.AGENT_RUNTIME_E2E = '1';

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

// M08.8.G (Stage-V finding 🟡 #1 — hermetic tier gate). The app reads the
// persisted tier from `app_local_data_dir/tier.json` at BACKEND-process
// startup (commands.rs `CurrentTierState`), so a stale `{"tier":"promoted"}`
// from a prior run makes `tier_display`/`tier_enforcement` mount Promoted and
// fail their Novice baseline assertions — and because `tier_display` only
// restores Novice in its end-of-test cleanup, an earlier failure wedges every
// subsequent LOCAL run red (CI is unaffected: a fresh runner has no
// `tier.json` → Novice default). Seeding `tier.json`=novice BEFORE each
// session's app launch makes the gate independent of persisted state. A mocha
// `before()` is too late — the backend already read the file when the session
// (and thus the app process) started; `beforeSession` runs before that, so it
// is the correct seam.
//
// Tauri 2.x `app_local_data_dir` for identifier `dev.aria-runtime.app`:
// Windows `%LOCALAPPDATA%\dev.aria-runtime.app`; Linux
// `$XDG_DATA_HOME/dev.aria-runtime.app` (else `~/.local/share/...`). macOS is
// skipped above (tauri-driver unsupported).
const APP_IDENTIFIER = 'dev.aria-runtime.app';

function appLocalDataDir(): string {
  if (process.platform === 'win32') {
    const base = process.env.LOCALAPPDATA ?? join(homedir(), 'AppData', 'Local');
    return join(base, APP_IDENTIFIER);
  }
  const base = process.env.XDG_DATA_HOME ?? join(homedir(), '.local', 'share');
  return join(base, APP_IDENTIFIER);
}

function seedNoviceTier(): void {
  const dir = appLocalDataDir();
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, 'tier.json'), '{"tier":"novice"}');
}

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

  // Seed Novice before each spec's app process launches, so the tier gate is
  // hermetic w.r.t. any `tier.json` a prior run left behind (Stage-V 🟡 #1).
  beforeSession(): void {
    seedNoviceTier();
  },

  onComplete(): void {
    tauriDriverProc?.kill('SIGTERM');
  },
};
