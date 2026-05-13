//! `SandboxError` — top-level error type for the runtime-sandbox crate.
//!
//! Mirrors `runtime_drone::DroneError`: a thiserror enum that wraps each
//! subsystem's error so the binary's `main` can `process::exit(1)` on
//! a single error type. Stage C2 adds the `Isolation` variant covering
//! seccomp / landlock / Job Objects setup errors.

use thiserror::Error;

/// IO-class error (socket bind / pipe open / framed-codec).
#[derive(Debug, Error)]
pub enum IpcError {
    /// I/O error binding or accepting on the socket.
    #[error("ipc io: {0}")]
    Io(#[from] std::io::Error),
    /// JSON (de)serialization error on the IPC line.
    #[error("ipc json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Top-level error raised inside the sandbox subprocess.
///
/// Surfaced to the binary entry point in `main.rs`; per spec §8.security
/// L3, any error here aborts the subprocess (the main process spawns a
/// fresh one).
#[derive(Debug, Error)]
pub enum SandboxError {
    /// IPC server / client framing error.
    #[error(transparent)]
    Ipc(#[from] IpcError),
    /// OS isolation setup failed: seccomp filter, landlock ruleset, or
    /// Windows Job Object setup error. The string carries the
    /// platform-specific error message; a failed install aborts the
    /// subprocess so main can spawn a fresh one in a known-good state.
    #[error("isolation: {0}")]
    Isolation(String),
}
