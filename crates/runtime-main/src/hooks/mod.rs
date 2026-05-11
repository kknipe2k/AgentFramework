//! Verify & Rails primitive — spec §4a (M04 Stage D).
//!
//! Submodules:
//!
//! - `shell` — cross-platform shell hook execution (PowerShell on
//!   Windows, bash on Linux/macOS) via `tokio::process::Command`.
//! - `executor` — runs a `Hook` and returns a `HookOutcome` the SDK
//!   projects into `verify_*` events.
//! - `rails` — `JSONLogic`-evaluated policy checks (hard/soft) against
//!   a facts object. Operator allowlist locked at gotcha #18.
//! - `dont_touch` — globset-backed pre-edit rail that blocks Write
//!   tool calls against framework-declared protected globs. Integration
//!   site (the Write-tool dispatcher) lands at M05+; the evaluator
//!   itself is callable today.

pub mod dont_touch;
pub mod executor;
pub mod rails;
pub mod shell;

pub use dont_touch::{DontTouchDecision, DontTouchError, DontTouchEvaluator};
pub use executor::{execute_hook, HookContext, HookDeps, HookError, HookOutcome};
pub use rails::{evaluate as evaluate_jsonlogic, evaluate_rail, RailError, RailOutcome};
pub use shell::{
    build_spawn_args, execute_shell, execute_shell_with, ShellError, ShellOutput, ShellSpawner,
    SpawnArgs, TokioShellSpawner,
};
