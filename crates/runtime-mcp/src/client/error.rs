//! Error type for the M06 Stage C lifecycle layer.
//!
//! [`LifecycleError`] is the surface returned by [`crate::client::McpClient`] +
//! [`crate::client::Registry`] + [`crate::client::SecretStore`] callers. Wraps
//! the lower-level `McpError` (transport-class) + `rusqlite::Error` (registry-
//! class) + `keyring::Error` (auth-class) into a single audit-friendly enum.

use thiserror::Error;

use crate::error::McpError;

/// Errors raised by the lifecycle / registry / auth surfaces.
#[derive(Debug, Error)]
pub enum LifecycleError {
    /// Underlying transport / protocol error from the rmcp wrapper.
    #[error("MCP transport: {0}")]
    Mcp(#[from] McpError),

    /// `SQLite` registry I/O or schema error.
    #[error("registry: {0}")]
    Registry(#[from] rusqlite::Error),

    /// Auth secret-store backend error.
    #[error("auth: {0}")]
    Auth(String),

    /// Server name not found in the registry.
    #[error("MCP server not found: {0}")]
    NotFound(String),

    /// Server name already exists in the registry (duplicate add).
    #[error("MCP server already exists: {0}")]
    AlreadyExists(String),

    /// JSON serialization error encoding/decoding registry rows.
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl LifecycleError {
    /// Build a [`LifecycleError::Auth`] from any displayable cause.
    #[must_use]
    pub fn auth(cause: impl std::fmt::Display) -> Self {
        Self::Auth(cause.to_string())
    }
}
