# Build-machine tauri-driver + msedgedriver setup (Windows)

One-time install on the Windows build machine so `npm run test:e2e:tauri`
runs locally. Enables the `diagnose_root_cause` step on X.5 fix cycles to
do REAL live-DOM inspection instead of phase-doc D.3.2's defensive
fallback. Implements Revision 1 of
`docs/protocol-revisions-irl-fail-rate.md`.

**Total time**: ~30 min active + ~30-60 min waiting for the Tauri release
build the first time.

**Shell**: PowerShell (the same one used for the M08.5 IRL pass). Each step
labels admin requirement.

---

## Phase 0 — Verify prerequisites (PowerShell, NO ADMIN)

Open a regular PowerShell window. None of these need admin.

### 0.1 — Check Edge version (needed to match msedgedriver)

```powershell
(Get-Item "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe").VersionInfo.ProductVersion
```

**Expected output**: a version string like `139.0.3405.86` (any version is
fine, but record the FULL number — you need to match msedgedriver to at
least the first three components).

If the path doesn't exist, try:
```powershell
(Get-Item "C:\Program Files\Microsoft\Edge\Application\msedge.exe").VersionInfo.ProductVersion
```

Or open Edge → Settings → About Microsoft Edge — the version is shown
there. **Write down the version**; you'll need it in Phase 2.

### 0.2 — Check Rust / cargo (installed if M08.5 work ran)

```powershell
cargo --version
```

**Expected output**: `cargo 1.80.0` or newer. If "command not found",
Rust isn't installed — fix that first via <https://rustup.rs/> (separate
out-of-scope task; M08.5 work requires it).

### 0.3 — Check Node / npm (installed if M08.5 work ran)

```powershell
node --version
npm --version
```

**Expected output**: `v20.x.x` or newer for node; `10.x.x` or newer for
npm. If missing, install Node 20 from <https://nodejs.org/>.

### 0.4 — Check the cargo bin directory is on PATH (cargo install target)

```powershell
$env:Path -split ';' | Select-String 'cargo'
```

**Expected output**: a line like `C:\Users\kknip\.cargo\bin`. If empty,
Rust install didn't add it; fix by running:

```powershell
[Environment]::SetEnvironmentVariable("Path", [Environment]::GetEnvironmentVariable("Path", "User") + ";$env:USERPROFILE\.cargo\bin", "User")
```

Then **close and reopen PowerShell** for the change to take effect.

---

## Phase 1 — Install tauri-driver (PowerShell, NO ADMIN)

The Tauri WebDriver bridge. Installs to `%USERPROFILE%\.cargo\bin\tauri-driver.exe`
— same directory as your other Rust binaries.

```powershell
cargo install tauri-driver --locked
```

**Expected wait**: 2-5 minutes (compiles from source).

**Expected output ending**: `Installed package ‘tauri-driver vX.Y.Z‘ (executable ‘tauri-driver.exe‘)`

### 1.1 — Verify

```powershell
tauri-driver --help
```

**Expected output**: the tauri-driver usage menu listing `--port`,
`--native-port`, `--native-host`, `--native-driver`, `-h, --help`. (Note:
tauri-driver does NOT support `--version` despite the common convention —
running `tauri-driver --version` returns `Error: unused arguments left:
["--version"]`. That error itself confirms the binary is installed and on
PATH — but use `--help` for the clean verify.)

If you get "command not found", PowerShell is using a stale PATH — close
and reopen.

---

## Phase 2 — Install msedgedriver (PowerShell + browser, NO ADMIN)

This is manual because the version MUST match your Edge version (Phase 0.1).

### 2.1 — Determine your msedgedriver version

Use the FIRST THREE COMPONENTS of your Edge version. Example: if Edge is
`139.0.3405.86`, you need msedgedriver `139.0.3405.x` where x is the
closest available patch.

### 2.2 — Open the download page

Open <https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/>
in a browser. The page lists current "Stable Channel" versions.

If your Edge version is the current Stable, just click the **x64 (Windows)**
download link under the latest version. If your Edge is older, scroll
down to find the matching version (Microsoft keeps several recent ones).
If your version isn't listed exactly, pick the closest x.y.z (first three
components) — patch level can differ.

You'll get a file like `edgedriver_win64.zip` (about 6-8 MB).

### 2.3 — Extract to a user folder (NO ADMIN)

Use a folder under your user profile so no admin is needed. Convention:
`%USERPROFILE%\webdriver`.

In PowerShell:

```powershell
$dest = "$env:USERPROFILE\webdriver"
New-Item -ItemType Directory -Path $dest -Force | Out-Null
$dest
```

This prints the path (e.g., `C:\Users\kknip\webdriver`).

Then extract the downloaded zip. Replace `<download-path>` with where your
browser saved it (probably `$env:USERPROFILE\Downloads\edgedriver_win64.zip`):

```powershell
Expand-Archive -Path "$env:USERPROFILE\Downloads\edgedriver_win64.zip" -DestinationPath $dest -Force
Get-ChildItem $dest
```

**Expected output**: should include `msedgedriver.exe` (the binary) and a
few other files (`Driver_Notes/`, `LICENSE`, etc.).

### 2.4 — Add the folder to your User PATH (NO ADMIN)

```powershell
$existing = [Environment]::GetEnvironmentVariable("Path", "User")
if ($existing -notlike "*$env:USERPROFILE\webdriver*") {
    [Environment]::SetEnvironmentVariable("Path", "$existing;$env:USERPROFILE\webdriver", "User")
    Write-Host "Added $env:USERPROFILE\webdriver to User PATH"
} else {
    Write-Host "Already on User PATH"
}
```

### 2.5 — Close and reopen PowerShell

PowerShell only re-reads PATH at startup. Close the window. Open a NEW
PowerShell (regular, no admin).

### 2.6 — Verify

```powershell
msedgedriver --version
```

**Expected output**: `Microsoft Edge WebDriver 139.0.3405.86 (...)` (your
version).

If "command not found": the PATH change didn't take. Re-run 2.4 and
verify the result of:

```powershell
$env:Path -split ';' | Select-String 'webdriver'
```

Should show your `webdriver` folder.

---

## Phase 3 — Build the Tauri release binary (PowerShell, NO ADMIN)

tauri-driver runs the REAL release binary (not the dev binary `npm run
tauri dev` builds). Building it is slow the first time — release mode
optimizes everything.

### 3.1 — Navigate to the repo

```powershell
cd C:\agent-runtime
```

### 3.2 — Ensure dependencies are current

```powershell
npm ci
cargo build --workspace
```

**Expected wait**: a few minutes each if recently run; longer if not.

### 3.3 — Build the release Tauri binary AND the sibling subprocess binaries

The Tauri app launches `runtime-drone.exe` and `runtime-sandbox.exe` as
sibling subprocesses at startup. `npx tauri build` only builds the Tauri
app itself — you also need the two sibling binaries in `target/release/`,
or the app will crash with `drone IPC unavailable: spawn drone subprocess:
The system cannot find the file specified`. CI builds these in two named
steps (A.fix follow-ups #3 and #4); local install needs both.

```powershell
npx tauri build --no-bundle
cargo build --release -p runtime-drone -p runtime-sandbox
```

The `--no-bundle` flag matches CI — it skips the MSI/NSIS installer
generation, saves ~5-10 min, and is what we need for tauri-driver.

**Expected wait first time**: 30-60 minutes. Subsequent rebuilds are
much faster (~5-10 min).

**Expected output ending**: `Finished `release` profile [optimized] target(s)`
+ a path to the produced binary.

### 3.4 — Verify the binary exists (workspace-layout note)

`npx tauri build --no-bundle` prints the binary's actual path at the end —
read that line carefully. In THIS repo's workspace layout (src-tauri is a
Cargo workspace member, shares the root `target/`), the build lands at:

```
C:\agent-runtime\target\release\agent-runtime.exe
```

NOT at `src-tauri\target\release\` (the non-workspace default). Verify:

```powershell
Get-Item "C:\agent-runtime\target\release\agent-runtime.exe"
```

**Expected output**: a file listing showing `agent-runtime.exe` with a
reasonable size (50-150 MB) and recent timestamp.

### 3.5 — Junction so wdio.conf.ts finds the binary (workaround)

`wdio.conf.ts:31` hardcodes `src-tauri/target/release/` — stale path
from the M03.F harness, never updated for the workspace layout. The
binary IS at workspace-root `target/release/`. Create a Windows junction
so the harness's hardcoded path resolves correctly without a commit:

```powershell
New-Item -ItemType Directory -Path "C:\agent-runtime\src-tauri\target" -Force | Out-Null
if (Test-Path "C:\agent-runtime\src-tauri\target\release") {
    Remove-Item "C:\agent-runtime\src-tauri\target\release" -Recurse -Force
}
New-Item -ItemType Junction -Path "C:\agent-runtime\src-tauri\target\release" -Target "C:\agent-runtime\target\release"
Get-Item "C:\agent-runtime\src-tauri\target\release\agent-runtime.exe"
```

**Junction vs symlink on Windows**: junction needs NO admin (unlike
symlinks, which require admin or Developer Mode). Junction works on the
local volume only — fine since both source and target are on C:.

The last `Get-Item` should now show the file (resolved through the
junction). This is a workaround — the real fix is updating
`wdio.conf.ts:31` to use workspace-root `target/release/`. Logged as a
🟡 finding for Revision 1's protocol commit (`docs/protocol-revisions-irl-fail-rate.md`).

---

## Phase 4 — Run the harness (PowerShell, NO ADMIN)

### 4.1 — Run the tests

```powershell
npm run test:e2e:tauri
```

**Expected output**: WebdriverIO + tauri-driver launch in sequence, the
app window briefly opens (you may see it flash), the 8 tests run, and a
summary at the end.

**Expected result**: of 8 tests, **2 should pass** (tests 1 + 2 — the
key-independent smoke tests) and **6 should skip** (tests 3-6 chain off
test 3's session which requires an Anthropic API key; the runtime
skip-guard added in M08.5 A.fix follow-up #1 makes them gracefully skip
when `ANTHROPIC_TEST_KEY` is unset). Plus the two M08.5 regression tests
(B.fix builder_drag + D.fix mcp_modal) should also pass.

**Expected total**: 2 + 2 = 4 passed, 4 skipped, 0 failed.

If you see failures, **paste the output back** when this session resumes.
Common first-run failures:

- `tauri-driver: command not found` → close and reopen PowerShell (Phase 1
  PATH didn't propagate).
- `msedgedriver: command not found` → close and reopen PowerShell (Phase 2
  PATH didn't propagate).
- `cannot find ...agent-runtime.exe` → Phase 3 build didn't complete or
  produced the binary elsewhere; check `src-tauri\target\release\`.
- `session not created: Microsoft Edge WebDriver only supports Microsoft Edge version XYZ` →
  Edge auto-updated; msedgedriver is now stale. Re-download Phase 2 with
  the new version.
- A test timeout out (no skip, no pass, hangs ~60s then errors) → likely a
  real-app regression to investigate; paste output.

---

## Phase 5 — One-time gotcha addition

After completing Phases 1-4, add a line to `docs/gotchas.md`:

```markdown
- **Gotcha #N (next available number): Build-machine tauri-driver setup**.
  See `docs/build-machine-tauri-driver-setup.md`. Edge auto-updates can
  invalidate msedgedriver; re-download per Phase 2 when version mismatch
  errors appear.
```

This goes in the same commit that updates `CLAUDE.md` §6 to add
`npm run test:e2e:tauri` to the local gate list (Revision 1 of the
protocol changes).

---

## Resume context for the next Claude session

After completing Phases 1-4:

1. The build machine can run `npm run test:e2e:tauri` locally.
2. Next X.5 fix cycle's `diagnose_root_cause` step does REAL live-DOM
   inspection.
3. The red-phase test commit can be EXECUTED LOCALLY (not just predicted)
   to capture right-reason failure output verbatim — same TDD evidence
   strength as C.fix had for pure Rust.
4. CLAUDE.md §6 + STAGE-PROMPT-PROTOCOL.md §10 get updated to reflect
   tauri-driver as a local-runnable gate.

Paste this file path (`docs/build-machine-tauri-driver-setup.md`) back to
Claude on resume — it'll know Phases 1-4 are done and Phase 5 is the
remaining work.

---

## Troubleshooting addendum

### "Cargo install tauri-driver" fails with linker errors

Usually means MSVC Build Tools aren't installed. Open
<https://visualstudio.microsoft.com/visual-cpp-build-tools/>, download
"Build Tools for Visual Studio", install with "Desktop development with
C++" workload selected. ~6 GB install, one-time. Then re-run Phase 1.

This is a Rust prerequisite, not a tauri-driver-specific need. If `cargo
build --workspace` works (Phase 0.2), MSVC is already there.

### "npx tauri build" fails with "cannot find module @tauri-apps/cli"

```powershell
npm install -D @tauri-apps/cli@latest
```

(NO ADMIN — `npm install -D` writes to the project's `node_modules/`,
not globally.)

### Antivirus quarantines msedgedriver.exe

Some corporate AV products flag WebDriver binaries. If `msedgedriver
--version` reports "Access is denied" or the file disappears, add an
exclusion for `%USERPROFILE%\webdriver\msedgedriver.exe` in your AV
console (admin needed for AV settings). Microsoft Defender exclusions:

```powershell
# REQUIRES ADMIN — only run if you have admin and your AV is Defender
Start-Process powershell -Verb RunAs -ArgumentList "-Command Add-MpPreference -ExclusionPath '$env:USERPROFILE\webdriver\msedgedriver.exe'"
```

### "ResourceUnavailable" on Expand-Archive

Means the zip download didn't complete. Re-download from Phase 2.2.

### Edge updates and breaks msedgedriver

Edge auto-updates on Windows; msedgedriver doesn't. When you see
`session not created: Microsoft Edge WebDriver only supports Microsoft Edge version XYZ`,
re-run Phase 2 with the new Edge version. Takes ~5 min.

To freeze Edge updates (NOT recommended; security risk): possible via
group policy, but you should just re-download msedgedriver every couple
months instead.
