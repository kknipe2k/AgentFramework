//! `skills.audit.jsonl` writer — spec §8.security L5 (M05 Stage E).
//!
//! Append-only, newline-delimited JSON. One `AuditEntry` per call to
//! `AuditWriter::log`; the writer serializes via `serde_json::to_string`
//! + emits the line + emits a `\n` + flushes.
//!
//! Mutex-guarded around the underlying `tokio::fs::File` handle so
//! concurrent callers serialize and don't interleave bytes. Per phase
//! doc gotcha trap #3, the lock is async-safe (`tokio::sync::Mutex`)
//! because callers are in async contexts.
//!
//! Best-effort observability: write failures surface as `AuditError`
//! to the call site; the call site MUST log via `tracing::error!` and
//! continue rather than propagate into dispatch (phase doc E.3.4 + spec
//! §13.5 dev-logging discipline).

use std::path::Path;

use runtime_core::generated::audit::AuditEntry;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::audit::error::AuditError;

/// Append-only writer for `skills.audit.jsonl`.
///
/// Cheap to construct (one OS file open); cheap to share across tasks
/// (the inner `tokio::sync::Mutex` serializes per-`log` calls).
/// Production wiring (Tauri shell) constructs one writer at app startup
/// and manages it in `Arc<AuditWriter>` so the capability enforcer,
/// tier evaluator, and `framework_loader` all hold the same handle.
/// Tests pass a `tempfile::TempDir`-derived path; the path-agnostic
/// surface mirrors the M05.D `tier::persistence` pattern.
#[derive(Debug)]
pub struct AuditWriter {
    file: Mutex<File>,
}

impl AuditWriter {
    /// Open or create the audit log file at `path`. The parent directory
    /// must already exist; the Tauri shell wires `app_local_data_dir()`
    /// which is created by [`std::fs::create_dir_all`] before the first
    /// audit-write attempt (mirrors the `resolve_db_path` archetype in
    /// `src-tauri/src/main.rs`).
    ///
    /// # Errors
    ///
    /// - [`AuditError::Io`] when the file cannot be opened in append
    ///   mode (e.g., parent directory missing, permission denied).
    pub async fn open(path: &Path) -> Result<Self, AuditError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        Ok(Self {
            file: Mutex::new(file),
        })
    }

    /// Append one `AuditEntry` as a JSONL line. The whole call
    /// (serialize + write + newline + flush) holds the lock so
    /// concurrent callers serialize end-to-end.
    ///
    /// # Errors
    ///
    /// - [`AuditError::Json`] when the entry fails to serialize. Rare;
    ///   the schema-generated `AuditEntry` derives Serialize.
    /// - [`AuditError::Io`] when the underlying file write or flush
    ///   fails (transient disk pressure / permission flip / disk full).
    ///
    /// Callers MUST NOT propagate this into dispatch — log via
    /// `tracing::error!` and continue. Audit availability is not a
    /// dispatch gate (phase doc E.3.4 + spec §13.5).
    pub async fn log(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        let line = serde_json::to_string(entry)?;
        let mut file = self.file.lock().await;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        drop(file);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::entry;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::io::AsyncReadExt;

    async fn read_audit_lines(path: &Path) -> Vec<String> {
        let mut file = tokio::fs::File::open(path).await.expect("open audit");
        let mut buf = String::new();
        file.read_to_string(&mut buf).await.expect("read audit");
        buf.lines().map(str::to_string).collect()
    }

    #[tokio::test]
    async fn log_single_entry_writes_one_line() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills.audit.jsonl");
        let writer = AuditWriter::open(&path).await.expect("open");
        let e = entry::framework_loaded("sess-1", "aria", 1);
        writer.log(&e).await.expect("log");
        let lines = read_audit_lines(&path).await;
        assert_eq!(lines.len(), 1, "single log call must produce one line");
        let parsed: serde_json::Value =
            serde_json::from_str(&lines[0]).expect("line parses as JSON");
        assert_eq!(parsed["kind"], "framework_loaded");
    }

    #[tokio::test]
    async fn two_sequential_entries_two_lines() {
        // Gotcha #69: multi-call invariant. Two sequential log calls
        // against the same writer must both produce parseable lines —
        // not a single concatenated line, not a single re-overwritten
        // line.
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills.audit.jsonl");
        let writer = AuditWriter::open(&path).await.expect("open");
        let e1 = entry::framework_loaded("sess-1", "aria", 1);
        let e2 = entry::capability_granted("sess-1", "worker", "read", "src/**");
        writer.log(&e1).await.expect("log #1");
        writer.log(&e2).await.expect("log #2");
        let lines = read_audit_lines(&path).await;
        assert_eq!(lines.len(), 2, "two log calls must produce two lines");
        let p1: serde_json::Value = serde_json::from_str(&lines[0]).expect("line 1 parses");
        let p2: serde_json::Value = serde_json::from_str(&lines[1]).expect("line 2 parses");
        assert_eq!(p1["kind"], "framework_loaded");
        assert_eq!(p2["kind"], "capability_granted");
    }

    #[tokio::test]
    async fn three_sequential_entries_preserve_order() {
        // Multi-call invariant extended — order is preserved across
        // sequential calls. The renderer + a human reader rely on this
        // to reconstruct the security-decision trace.
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills.audit.jsonl");
        let writer = AuditWriter::open(&path).await.expect("open");
        let kinds = ["framework_loaded", "capability_granted", "tier_transition"];
        writer
            .log(&entry::framework_loaded("s", "fw", 0))
            .await
            .expect("1");
        writer
            .log(&entry::capability_granted("s", "a", "read", "r"))
            .await
            .expect("2");
        writer
            .log(&entry::tier_transition(
                "s",
                crate::tier::Tier::Novice,
                crate::tier::Tier::Promoted,
                "reason",
            ))
            .await
            .expect("3");
        let lines = read_audit_lines(&path).await;
        assert_eq!(lines.len(), 3);
        for (line, expected) in lines.iter().zip(kinds.iter()) {
            let parsed: serde_json::Value = serde_json::from_str(line).expect("parse");
            assert_eq!(&parsed["kind"], expected);
        }
    }

    #[tokio::test]
    async fn concurrent_writes_serialized_by_mutex() {
        // Mutex around the file handle is required because tokio::fs::File
        // writes are not atomic at the OS layer for byte sequences larger
        // than the smallest atomic write unit — without the lock,
        // concurrent log() calls could interleave bytes mid-line. With
        // the lock, every line is intact + parseable as JSON.
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills.audit.jsonl");
        let writer = Arc::new(AuditWriter::open(&path).await.expect("open"));
        let mut handles = Vec::new();
        for i in 0..10 {
            let w = Arc::clone(&writer);
            handles.push(tokio::spawn(async move {
                let e =
                    entry::capability_granted("sess-1", &format!("agent-{i}"), "read", "src/**");
                w.log(&e).await.expect("log");
            }));
        }
        for h in handles {
            h.await.expect("task");
        }
        let lines = read_audit_lines(&path).await;
        assert_eq!(lines.len(), 10, "10 concurrent calls must produce 10 lines");
        for line in &lines {
            let parsed: serde_json::Value =
                serde_json::from_str(line).expect("each line must be parseable JSON");
            assert_eq!(parsed["kind"], "capability_granted");
        }
    }

    #[tokio::test]
    async fn append_mode_preserves_pre_existing_content() {
        // Open-once, write-twice via separate AuditWriter constructions —
        // the second writer must see the first writer's line still on
        // disk (append-mode, no truncate).
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills.audit.jsonl");
        {
            let writer = AuditWriter::open(&path).await.expect("open #1");
            writer
                .log(&entry::framework_loaded("s", "fw", 0))
                .await
                .expect("log #1");
        }
        {
            let writer = AuditWriter::open(&path).await.expect("open #2");
            writer
                .log(&entry::capability_granted("s", "a", "read", "r"))
                .await
                .expect("log #2");
        }
        let lines = read_audit_lines(&path).await;
        assert_eq!(lines.len(), 2, "second open must not truncate first line");
    }

    #[tokio::test]
    async fn open_in_missing_directory_returns_io_error() {
        // Per the open() doc: the parent directory must already exist.
        // Opening into a missing directory surfaces an IO error rather
        // than panicking.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("does-not-exist").join("skills.audit.jsonl");
        let err = AuditWriter::open(&missing)
            .await
            .expect_err("missing parent must err");
        assert!(matches!(err, AuditError::Io(_)));
    }
}
