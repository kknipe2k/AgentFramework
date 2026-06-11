//! Shared drone-subprocess fixture for `runtime-main` integration tests.
//!
//! TD-005 / gotcha #56 structural close. Six integration test files
//! (`drone_ipc_loopback`, `drone_reconnect_events`, `plan_lifecycle`,
//! `plan_recovery`, `recovery_lifecycle`, `smoke_signal_persistence`)
//! spawn the `runtime-drone` binary as a black-box subprocess. They
//! each carried a byte-identical `drone_binary` + `ensure_drone_built`
//! helper that built the drone via a nested `cargo build` into the
//! parent run's target dir. On the Windows host that nested build
//! failed two ways under `cargo llvm-cov`. First, the parent (the
//! llvm-cov-driven `cargo test`) holds the build lock on
//! `target/llvm-cov-target`, so a nested cargo into that same dir is a
//! lock conflict. Second, the test binary's CWD is the `runtime-main`
//! package dir, where a bare `--bin runtime-drone` does not resolve.
//! That is why the runtime-main `cargo llvm-cov` gate was not
//! Windows-local-measurable (M06.V finding #3, logged as TD-005).
//!
//! Structural close — TD-005 recommended approach (b), a pre-staged
//! fixture binary decoupled from the coverage run, implemented in-test
//! and centralized here so it cannot drift across the six files. The
//! drone is built once into a dedicated `target/drone-fixture` dir that
//! is never the parent run's target dir (no lock contention), with the
//! workspace manifest plus package pinned (CWD-independent) and the
//! llvm-cov instrumentation env stripped (no `RUSTC_WRAPPER` shim — the
//! drone is exec'd as a black box we do NOT measure here; runtime-drone
//! has its own per-crate gate). CI-parity-safe: the same code runs in
//! CI's llvm-cov job (it just builds the fixture once); it is not a
//! local-only `--test-threads` flag.

#![allow(
    dead_code,
    reason = "each test crate uses a subset of the fixture surface"
)]

/// Dedicated, coverage-decoupled target dir for the drone fixture
/// binary: `<workspace>/target/drone-fixture`.
fn drone_fixture_target_dir() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR = `<workspace>/crates/runtime-main`.
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("drone-fixture")
}

/// Locate the `runtime-drone` fixture binary inside the dedicated
/// drone-fixture target dir (always a normal un-instrumented `debug`
/// build — `cargo test` / `cargo llvm-cov` both run the test profile).
pub fn drone_binary() -> std::path::PathBuf {
    let mut p = drone_fixture_target_dir();
    p.push("debug");
    #[cfg(windows)]
    p.push("runtime-drone.exe");
    #[cfg(unix)]
    p.push("runtime-drone");
    p
}

/// Build the `runtime-drone` fixture binary. Always invokes `cargo
/// build` — cargo no-ops in ~1s when the fixture is fresh, while an
/// only-if-absent check goes stale after impl changes to the drone
/// (M09.5.C: the assembled oversize test would have validated the OLD
/// binary — a silent false-green). See module docs for the TD-005 /
/// gotcha #56 rationale.
pub fn ensure_drone_built() {
    let bin = drone_binary();
    let target_dir = drone_fixture_target_dir();
    let ws_manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("Cargo.toml");
    let mut cmd = std::process::Command::new(env!("CARGO"));
    // `-p runtime-drone` + explicit workspace `--manifest-path`:
    // the test binary's CWD is the runtime-main package dir, where
    // a bare `--bin runtime-drone` does not resolve ("no bin target
    // named runtime-drone in default-run packages"). Pinning the
    // package + workspace manifest makes the build CWD-independent.
    cmd.args([
        "build",
        "--manifest-path",
        &ws_manifest.to_string_lossy(),
        "-p",
        "runtime-drone",
        "--bin",
        "runtime-drone",
    ]);
    cmd.env("CARGO_TARGET_DIR", &target_dir);
    for var in [
        "RUSTFLAGS",
        "RUSTDOCFLAGS",
        "LLVM_PROFILE_FILE",
        "CARGO_LLVM_COV",
        "CARGO_LLVM_COV_SHOW_ENV",
        "CARGO_LLVM_COV_TARGET_DIR",
        "CARGO_LLVM_COV_BUILD_DIR",
        "CARGO_INCREMENTAL",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTC_WRAPPER",
        "__CARGO_LLVM_COV_RUSTC_WRAPPER",
        "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS",
        "__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES",
    ] {
        cmd.env_remove(var);
    }
    let out = cmd.output().expect("cargo build");
    assert!(
        out.status.success(),
        "drone build failed:\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(bin.exists(), "drone binary missing at {}", bin.display());
}
