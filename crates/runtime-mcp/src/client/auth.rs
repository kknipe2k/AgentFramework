//! Per-server auth secret store (M06 Stage C).
//!
//! The [`SecretStore`] trait is the abstraction MCP lifecycle code uses
//! to persist + fetch per-server auth secrets (bearer tokens, OAuth
//! access tokens, API keys). Two impls:
//!
//! - [`InMemorySecretStore`] — `HashMap`-backed; for tests + the v0.1
//!   pre-keychain path. Per-call `Mutex` guards concurrent access.
//! - [`KeyringSecretStore`] — production impl. Wraps the `keyring` crate
//!   directly with the MCP-namespaced service `agent-runtime/mcp` so
//!   per-server secrets land in OS-distinct entries vs the M02
//!   API-key keychain entry. Reuses the workspace `keyring` pin (with
//!   the platform-backend feature flags per gotcha #29).
//!
//! The trait's `ref_` parameter is the keychain key (e.g.
//! `mcp.github`). `McpClient` lifecycle generates these from the schema's
//! `auth_secret_ref` field (`mcp.v1.json::McpServerConfig.auth_secret_ref`).

use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::client::error::LifecycleError;

/// Keychain SERVICE namespace for per-MCP-server secrets. Distinct from
/// M02's `agent-runtime` SERVICE for the Anthropic API key so the two
/// secret families do not collide in the OS keychain.
pub const MCP_KEYRING_SERVICE: &str = "agent-runtime/mcp";

/// Persistent per-server auth secret surface.
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Store `secret` under `ref_`. Overwrites any prior value at the
    /// same ref.
    async fn store_secret(&self, ref_: &str, secret: &str) -> Result<(), LifecycleError>;

    /// Fetch the secret stored at `ref_`. Returns
    /// [`LifecycleError::NotFound`] when no secret exists.
    async fn fetch_secret(&self, ref_: &str) -> Result<String, LifecycleError>;

    /// Remove the secret stored at `ref_`. Idempotent — removing a
    /// missing ref returns `Ok(())` so test setup/teardown can run
    /// without ordering constraints.
    async fn remove_secret(&self, ref_: &str) -> Result<(), LifecycleError>;
}

/// In-memory secret store for tests + the v0.1 pre-keychain path.
#[derive(Debug, Default)]
pub struct InMemorySecretStore {
    inner: Mutex<HashMap<String, String>>,
}

impl InMemorySecretStore {
    /// New empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SecretStore for InMemorySecretStore {
    async fn store_secret(&self, ref_: &str, secret: &str) -> Result<(), LifecycleError> {
        self.inner
            .lock()
            .await
            .insert(ref_.to_string(), secret.to_string());
        Ok(())
    }

    async fn fetch_secret(&self, ref_: &str) -> Result<String, LifecycleError> {
        self.inner
            .lock()
            .await
            .get(ref_)
            .cloned()
            .ok_or_else(|| LifecycleError::NotFound(ref_.to_string()))
    }

    async fn remove_secret(&self, ref_: &str) -> Result<(), LifecycleError> {
        self.inner.lock().await.remove(ref_);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_store_default_construction_returns_empty_store() {
        let store = InMemorySecretStore::default();
        let err = store.fetch_secret("never").await.unwrap_err();
        assert!(matches!(err, LifecycleError::NotFound(_)));
    }
}
