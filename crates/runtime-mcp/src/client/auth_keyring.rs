//! Production keychain-backed [`SecretStore`] implementation (M06 Stage C).
//!
//! Wraps the `keyring` crate with the MCP-namespaced
//! [`MCP_KEYRING_SERVICE`]. Per gotcha #29 the platform backend is
//! selected via the workspace `keyring` feature flags
//! (`apple-native`, `windows-native`, `sync-secret-service`) — no
//! runtime selection here.
//!
//! ## Coverage holdout
//!
//! Excluded from the runtime-mcp ≥95% gate per the M02
//! `runtime-main::key_store` precedent. Every method touches the real
//! OS keychain (Linux Secret Service / macOS Keychain Services /
//! Windows Credential Manager) and is structurally untestable on CI
//! cells without a live session bus or signed in user. Coverage
//! attribution comes via the `#[ignore]`-gated round-trip tests below;
//! enable locally with
//! `cargo test --package runtime-mcp keyring -- --ignored`.

use async_trait::async_trait;

use crate::client::auth::{SecretStore, MCP_KEYRING_SERVICE};
use crate::client::error::LifecycleError;

/// Production keychain-backed secret store.
#[derive(Debug)]
pub struct KeyringSecretStore;

impl KeyringSecretStore {
    /// New keyring-backed store. The platform backend is selected at
    /// compile time via the workspace `keyring` feature flags.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for KeyringSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretStore for KeyringSecretStore {
    async fn store_secret(&self, ref_: &str, secret: &str) -> Result<(), LifecycleError> {
        // The `keyring` crate is sync; offload to a blocking thread so
        // the async runtime's executor stays free during OS-keychain I/O.
        let ref_owned = ref_.to_string();
        let secret_owned = secret.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), LifecycleError> {
            let entry = keyring::Entry::new(MCP_KEYRING_SERVICE, &ref_owned)
                .map_err(LifecycleError::auth)?;
            entry
                .set_password(&secret_owned)
                .map_err(LifecycleError::auth)?;
            Ok(())
        })
        .await
        .map_err(LifecycleError::auth)??;
        Ok(())
    }

    async fn fetch_secret(&self, ref_: &str) -> Result<String, LifecycleError> {
        let ref_owned = ref_.to_string();
        let ref_for_err = ref_.to_string();
        tokio::task::spawn_blocking(move || -> Result<String, LifecycleError> {
            let entry = keyring::Entry::new(MCP_KEYRING_SERVICE, &ref_owned)
                .map_err(LifecycleError::auth)?;
            match entry.get_password() {
                Ok(s) => Ok(s),
                Err(keyring::Error::NoEntry) => Err(LifecycleError::NotFound(ref_owned)),
                Err(e) => Err(LifecycleError::auth(e)),
            }
        })
        .await
        .map_err(|e| LifecycleError::auth(format!("spawn_blocking ({ref_for_err}): {e}")))?
    }

    async fn remove_secret(&self, ref_: &str) -> Result<(), LifecycleError> {
        let ref_owned = ref_.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), LifecycleError> {
            let entry = keyring::Entry::new(MCP_KEYRING_SERVICE, &ref_owned)
                .map_err(LifecycleError::auth)?;
            // Idempotent — NoEntry is not an error.
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(LifecycleError::auth(e)),
            }
        })
        .await
        .map_err(LifecycleError::auth)??;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise the Default impl without tripping
    // clippy::default_constructed_unit_structs (the lint fires on
    // `KeyringSecretStore::default()` for a unit struct; routing
    // through a generic keeps the impl under test).
    fn via_default<T: Default>() -> T {
        T::default()
    }

    #[test]
    fn new_and_default_construct_the_unit_store() {
        let _ = KeyringSecretStore::new();
        let _ = KeyringSecretStore;
        let _: KeyringSecretStore = via_default();
    }

    // The round-trip tests below exercise a real platform keychain and
    // are gated `#[ignore]` so CI cells without one do not fail. Locally,
    // run with `cargo test --package runtime-mcp keyring -- --ignored`.

    #[tokio::test]
    #[ignore = "requires a platform keychain — Linux Secret Service / macOS Keychain / Windows Credential Manager"]
    async fn read_after_write_roundtrips() {
        let store = KeyringSecretStore::new();
        let _ = store.remove_secret("mcp.test-roundtrip").await;
        store
            .store_secret("mcp.test-roundtrip", "abc123")
            .await
            .expect("store");
        let got = store
            .fetch_secret("mcp.test-roundtrip")
            .await
            .expect("fetch");
        assert_eq!(got, "abc123");
        store
            .remove_secret("mcp.test-roundtrip")
            .await
            .expect("cleanup");
    }
}
