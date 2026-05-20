//! Main-side IPC client for the runtime-sandbox subprocess (M05 Stage C1).
//!
//! Wire format: `LinesCodec`-framed JSON over Unix domain socket
//! (Linux/macOS) or Windows named pipe. Mirrors the server side at
//! `crates/runtime-sandbox/src/ipc.rs`.
//!
//! Stage C1 ships strict request-response (`ValidateArtifact` →
//! `ValidationResult`, `Shutdown` → no response). Stage C2 doesn't
//! change this surface — it adds OS-level isolation inside the sandbox
//! subprocess. M09 (generators) is the first production caller.

mod client;
mod connection;

pub use client::SandboxClient;
pub use connection::SandboxIpcError;
/// The L3 sandbox decision (`Ok` / `Reject { reasons }`) — re-exported
/// so shell callers map it without a direct `runtime-sandbox` dep.
pub use runtime_sandbox::validator::ValidationResult;
