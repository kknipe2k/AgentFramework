//! Shared constants for the framed-JSON IPC channels (spec §1d).
//!
//! Covers main ↔ drone and main ↔ sandbox, over Unix domain socket /
//! Windows named pipe with newline-delimited JSON via
//! `tokio_util::codec::LinesCodec`.

/// Maximum accepted length of one framed-JSON IPC line, in bytes.
///
/// Applied via `LinesCodec::new_with_max_length` at every IPC codec
/// site (TD-053, external review 2026-06-09 C4): an uncapped
/// `LinesCodec::new()` decodes with max length `usize::MAX`, so a peer
/// (or corrupted pipe) writing bytes without a newline buffers
/// unbounded memory — an OOM-DoS of the drone or sandbox daemon. The
/// cap bounds a hostile writer to one frame buffer and makes the
/// previously-dead `MaxLineLengthExceeded` error arms live: the
/// connection fails with a typed error and the daemon serves the next
/// connection.
///
/// **Why 4 MiB.** The largest legitimate frames are the drone's
/// `SessionRecovered` (full snapshot `state_json` plus plan/task
/// projections), `SignalLog` (a session's entire signal log), and
/// `QueryResult` rows carrying `snapshots.state_json`; the sandbox's
/// ceiling is `ValidateArtifact.artifact_code`. Frames observed today
/// are KB-scale; the bounding future case is a full-context
/// conversation serialized into snapshot state — roughly 1–2 MB of
/// escaped JSON at a ~200K-token context. 4 MiB covers that with ≥2×
/// margin.
///
/// **Boundary semantics** (pinned from the tokio-util 0.7.18
/// `LinesCodec` source): the limit is delimiter-exclusive. A line of
/// exactly this many content bytes followed by `\n` decodes; one more
/// content byte without a newline errors. The limit is enforced on
/// decode only — write halves use the same constructor for uniformity,
/// but encoding is not length-checked.
pub const MAX_IPC_FRAME_BYTES: usize = 4 * 1024 * 1024;
