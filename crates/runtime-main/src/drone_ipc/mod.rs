//! Main-side IPC client for the runtime-drone subprocess (M01).
//!
//! Wire format: `LinesCodec`-framed JSON over Unix domain socket
//! (Linux/macOS) or Windows named pipe. Identical framing to the drone
//! server side at `crates/runtime-drone/src/ipc.rs`; this module is the
//! client mirror.
//!
//! M02 only exercises [`runtime_core::DroneCommand::SnapshotNow`] (driven
//! from `AgentSdk` on task lifecycle events). M03+ adds `SpawnProcess`,
//! `StopProcess`, etc. as new subsystems land.

mod client;
mod connection;

pub use client::DroneClient;
pub use connection::DroneIpcError;
