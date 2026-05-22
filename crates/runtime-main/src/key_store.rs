//! OS-keychain-backed Anthropic API key storage (M02 Stage E).
//!
//! Reads/writes the API key under service `agent-runtime`, user `anthropic`
//! via the [`keyring`] crate. Reads are wrapped in `SecretString` so the
//! key never `Debug`-prints; writes accept `&str` and rely on
//! `keyring::Entry::set_password` to drop the input after storing.
//!
//! Per spec §13 zero-telemetry, the key is never logged, never serialized
//! over IPC, and never returned to the renderer (the renderer's only
//! interactions are `invokeSetApiKey(key)` and `invokeRunSmokeSession()`,
//! the latter of which reads the key main-side and constructs the provider).
//!
//! The platform backend is provided by the `keyring` crate:
//! - Linux: Secret Service via D-Bus.
//! - macOS: Keychain Services.
//! - Windows: Credential Manager.
//!
//! Tests requiring a real platform keychain are gated `#[ignore]` (CI cells
//! without a session bus or keychain skip them automatically). Unit-level
//! coverage is provided by the `KeyStoreError`-construction tests below.

use keyring::Entry;
use runtime_core::CmdError;
use secrecy::SecretString;
use thiserror::Error;

const SERVICE: &str = "agent-runtime";
const USER: &str = "anthropic";

/// Errors raised by the key-store layer.
#[derive(Debug, Error)]
pub enum KeyStoreError {
    /// No entry exists for the configured service+user pair.
    #[error("API key not found in OS keychain (service={SERVICE}, user={USER})")]
    NotFound,
    /// Underlying keyring failure (platform backend error).
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}

/// Translate a [`KeyStoreError`] into the wire-format [`CmdError`] the
/// renderer pattern-matches on. `NotFound` is the user-actionable case
/// (renderer prompts "set your key first"); other backend failures
/// surface as `KeyStore` with the underlying `Display` body.
///
/// Lives here rather than in `src-tauri/src/commands.rs` because of
/// orphan rules — `KeyStoreError` is local to this crate, so this is
/// the only place a `From<KeyStoreError>` for the foreign `CmdError`
/// type is permissible.
impl From<KeyStoreError> for CmdError {
    fn from(e: KeyStoreError) -> Self {
        match e {
            KeyStoreError::NotFound => Self::SetupRequired,
            other @ KeyStoreError::Keyring(_) => Self::key_store(other.to_string()),
        }
    }
}

/// Read the Anthropic API key from the OS keychain.
///
/// # Errors
///
/// Returns [`KeyStoreError::NotFound`] if no entry exists,
/// [`KeyStoreError::Keyring`] for any other backend failure.
pub fn read_api_key() -> Result<SecretString, KeyStoreError> {
    let entry = Entry::new(SERVICE, USER)?;
    match entry.get_password() {
        Ok(s) => Ok(SecretString::from(s)),
        Err(keyring::Error::NoEntry) => Err(KeyStoreError::NotFound),
        Err(e) => Err(e.into()),
    }
}

/// Whether an Anthropic API key is present in the OS keychain.
///
/// `read_api_key().is_ok()`: [`KeyStoreError::NotFound`] → `false`, any
/// other backend error → `false`. The renderer treats "can't tell" the
/// same as "absent" — the user can always re-enter the key, and an app
/// launch must not fail on a transiently locked keychain. M08 Stage A
/// (M07-IRL #7) — the renderer reads this at mount to seed `hasKey` so
/// a key entered once survives an app restart.
#[must_use]
pub fn has_api_key() -> bool {
    read_api_key().is_ok()
}

/// Write the Anthropic API key to the OS keychain. Overwrites any prior value.
///
/// # Errors
///
/// Returns [`KeyStoreError::Keyring`] on any backend failure.
pub fn write_api_key(key: &str) -> Result<(), KeyStoreError> {
    let entry = Entry::new(SERVICE, USER)?;
    entry.set_password(key)?;
    Ok(())
}

/// Delete the Anthropic API key entry. Idempotent — calling on a missing
/// entry returns `Ok(())` so test setup/teardown can run without ordering
/// constraints.
///
/// # Errors
///
/// Returns [`KeyStoreError::Keyring`] on any backend failure other than
/// "no entry" (which is treated as success).
pub fn delete_api_key() -> Result<(), KeyStoreError> {
    let entry = Entry::new(SERVICE, USER)?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn not_found_error_message_carries_service_and_user_for_setup_diagnostics() {
        // Pure construction test — does not touch the platform keychain.
        // Verifies the error renders the SERVICE/USER constants so a fresh
        // user looking at the renderer's surfaced error knows which entry
        // to populate.
        let e = KeyStoreError::NotFound;
        let s = e.to_string();
        assert!(
            s.contains(SERVICE),
            "NotFound message should cite service name: {s}"
        );
        assert!(
            s.contains(USER),
            "NotFound message should cite user name: {s}"
        );
    }

    #[test]
    fn from_keystore_error_not_found_maps_to_setup_required() {
        // The keychain "not found" condition is the user-actionable path:
        // renderer surfaces "set your key first" rather than a generic
        // backend error. M04 Stage A2 moves this conversion from
        // src-tauri/src/commands.rs into runtime-main/src/key_store.rs to
        // satisfy the orphan rule when the destination type
        // (runtime_core::CmdError) is foreign.
        let cmd_err: CmdError = KeyStoreError::NotFound.into();
        assert!(
            matches!(cmd_err, CmdError::SetupRequired),
            "got {cmd_err:?}"
        );
    }

    #[test]
    fn from_keystore_error_keyring_maps_to_key_store_with_display_body() {
        // Non-NotFound backend errors carry the underlying Display body
        // through to the renderer so the user sees the failing platform
        // detail (locked keychain, D-Bus offline, etc.).
        let cmd_err: CmdError = KeyStoreError::Keyring(keyring::Error::NoEntry).into();
        let CmdError::KeyStore(msg) = &cmd_err else {
            panic!("expected CmdError::KeyStore, got {cmd_err:?}");
        };
        assert!(
            msg.as_str().starts_with("keyring error:"),
            "expected keyring prefix in {}",
            msg.as_str()
        );
    }

    #[test]
    fn keyring_error_wraps_underlying_via_from() {
        // The `#[from] keyring::Error` derive should surface the underlying
        // error via Display. We construct a NoEntry to exercise the From impl
        // even though `read_api_key` translates NoEntry to NotFound.
        let raw = keyring::Error::NoEntry;
        let wrapped: KeyStoreError = raw.into();
        let s = wrapped.to_string();
        assert!(
            s.starts_with("keyring error:"),
            "wrapped error should start with keyring prefix: {s}"
        );
    }

    // The two tests below exercise a real platform keychain and are gated
    // `#[ignore]` so CI cells without one do not fail. Locally, run with
    // `cargo test --package runtime-main key_store -- --ignored`.

    #[test]
    #[ignore = "requires a platform keychain — Linux Secret Service / macOS Keychain / Windows Credential Manager"]
    fn read_after_write_roundtrips() {
        // Ensure clean slate.
        delete_api_key().expect("delete (initial)");
        write_api_key("sk-ant-test-roundtrip").expect("write");
        let got = read_api_key().expect("read");
        assert_eq!(got.expose_secret(), "sk-ant-test-roundtrip");
        delete_api_key().expect("delete (cleanup)");
    }

    #[test]
    #[ignore = "requires a platform keychain"]
    fn read_when_missing_returns_not_found() {
        delete_api_key().expect("delete");
        match read_api_key() {
            Err(KeyStoreError::NotFound) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }
}
