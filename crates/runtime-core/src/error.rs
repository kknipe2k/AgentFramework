//! Runtime error types via thiserror.

use thiserror::Error;

/// Top-level error type for the agent runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Schema validation failed.
    #[error("schema validation failed: {0}")]
    SchemaValidation(String),

    /// Type generation drift detected between schemas and committed types.
    #[error("type generation drift detected: {0}")]
    TypeDrift(String),

    /// An event payload could not be deserialized.
    #[error("invalid event payload: {0}")]
    InvalidEvent(String),

    /// A drone command could not be deserialized.
    #[error("invalid drone command: {0}")]
    InvalidCommand(String),

    /// JSON serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// YAML serialization/deserialization error.
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
