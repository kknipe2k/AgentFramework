//! Inherent impls and trait impls for the typify-generated [`CmdError`].
//!
//! M04 Stage A2 migrates `src-tauri/src/commands.rs` from a hand-rolled
//! `CmdError` enum (struct variants + thiserror derive) to the generated
//! [`crate::generated::error::CmdError`] (tuple variants over [`ErrorMessage`]
//! + serde-only derives). The wire shape is identical (`{"type":"provider",
//! "message":"..."}`) but the Rust callsite shape differs:
//!
//! - Hand-rolled: `CmdError::Provider { message: m }`, `Display` from `thiserror`.
//! - Generated:   `CmdError::Provider(ErrorMessage)`, no `Display`, no `Error`.
//!
//! This module restores the missing pieces:
//!
//! 1. **Inherent constructors** that wrap the [`ErrorMessage`] newtype's
//!    `minLength: 1` validation — `provider("...")` etc. accept any
//!    `Into<String>` and substitute `"(no message)"` for empty strings,
//!    so callers never have to handle the validation error.
//! 2. **`Display`** that matches the legacy `thiserror`-derived messages
//!    so `tracing::error!(error = %e, ...)` calls produce identical output.
//! 3. **`std::error::Error`** so `CmdError` works with `?` and `Box<dyn Error>`.
//! 4. **A `message()` accessor** that returns the body for variants that
//!    carry one (everything except `SetupRequired`).
//!
//! Per CLAUDE.md §14 (schemas as source of truth): the wire shape is owned
//! by `schemas/error.v1.json`. This file owns the Rust ergonomics around
//! the generated type and may evolve without a schema bump.

use std::fmt;

use crate::generated::error::{CmdError, ErrorMessage};

impl CmdError {
    /// Construct a [`CmdError::Provider`] from any `Into<String>`.
    ///
    /// Empty strings become `"(no message)"` because the `ErrorMessage`
    /// newtype enforces `minLength: 1` from `schemas/error.v1.json`.
    #[must_use]
    pub fn provider(msg: impl Into<String>) -> Self {
        Self::Provider(into_error_message(msg))
    }

    /// Construct a [`CmdError::Drone`] from any `Into<String>`.
    #[must_use]
    pub fn drone(msg: impl Into<String>) -> Self {
        Self::Drone(into_error_message(msg))
    }

    /// Construct a [`CmdError::KeyStore`] from any `Into<String>`.
    #[must_use]
    pub fn key_store(msg: impl Into<String>) -> Self {
        Self::KeyStore(into_error_message(msg))
    }

    /// Construct a [`CmdError::Internal`] from any `Into<String>`.
    #[must_use]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(into_error_message(msg))
    }

    /// The message body for variants that carry one.
    ///
    /// [`CmdError::SetupRequired`] is a unit variant and returns `None`.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::SetupRequired => None,
            Self::Provider(m) | Self::Drone(m) | Self::KeyStore(m) | Self::Internal(m) => {
                Some(m.as_str())
            }
        }
    }
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetupRequired => f.write_str("API key not set; call set_api_key first"),
            Self::Provider(m) => write!(f, "provider error: {}", m.as_str()),
            Self::Drone(m) => write!(f, "drone IPC unavailable: {}", m.as_str()),
            Self::KeyStore(m) => write!(f, "key store: {}", m.as_str()),
            Self::Internal(m) => write!(f, "internal: {}", m.as_str()),
        }
    }
}

impl std::error::Error for CmdError {}

impl ErrorMessage {
    /// Borrow the inner `&str`. The newtype is `transparent` over `String`
    /// but [`std::ops::Deref`] returns `&String`, which most callers don't
    /// want.
    #[must_use]
    pub fn as_str(&self) -> &str {
        // ErrorMessage has Deref<Target = String>; use the explicit deref
        // path so this never breaks if typify changes the inner type.
        let s: &String = self;
        s.as_str()
    }
}

fn into_error_message(msg: impl Into<String>) -> ErrorMessage {
    let s: String = msg.into();
    let candidate = if s.is_empty() {
        "(no message)".to_string()
    } else {
        s
    };
    // try_from validates minLength:1; the empty-string fallback above
    // guarantees success, so `.expect` here marks the invariant rather
    // than handling a real error.
    ErrorMessage::try_from(candidate).expect("non-empty after empty-string fallback")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_constructor_wraps_message() {
        let e = CmdError::provider("boom");
        assert!(matches!(e, CmdError::Provider(_)));
        assert_eq!(e.message(), Some("boom"));
    }

    #[test]
    fn drone_constructor_wraps_message() {
        let e = CmdError::drone("disconnected");
        assert!(matches!(e, CmdError::Drone(_)));
        assert_eq!(e.message(), Some("disconnected"));
    }

    #[test]
    fn key_store_constructor_wraps_message() {
        let e = CmdError::key_store("backend failure");
        assert!(matches!(e, CmdError::KeyStore(_)));
        assert_eq!(e.message(), Some("backend failure"));
    }

    #[test]
    fn internal_constructor_wraps_message() {
        let e = CmdError::internal("channel closed");
        assert!(matches!(e, CmdError::Internal(_)));
        assert_eq!(e.message(), Some("channel closed"));
    }

    #[test]
    fn empty_message_substitutes_placeholder_to_satisfy_min_length_one() {
        // `ErrorMessage` enforces `minLength: 1` from schemas/error.v1.json.
        // The constructor must never panic on an empty string from a caller.
        let e = CmdError::provider("");
        assert_eq!(e.message(), Some("(no message)"));
    }

    #[test]
    fn setup_required_message_is_none() {
        assert_eq!(CmdError::SetupRequired.message(), None);
    }

    #[test]
    fn display_setup_required_matches_legacy_thiserror_message() {
        // M02's hand-rolled enum used `#[error("API key not set; call set_api_key first")]`.
        // Tracing logs and renderer fallbacks pattern-match against the body of
        // these messages, so the migration must preserve them byte-for-byte.
        assert_eq!(
            CmdError::SetupRequired.to_string(),
            "API key not set; call set_api_key first"
        );
    }

    #[test]
    fn display_provider_uses_legacy_prefix() {
        assert_eq!(
            CmdError::provider("auth failed").to_string(),
            "provider error: auth failed"
        );
    }

    #[test]
    fn display_drone_uses_legacy_prefix() {
        assert_eq!(
            CmdError::drone("timeout").to_string(),
            "drone IPC unavailable: timeout"
        );
    }

    #[test]
    fn display_key_store_uses_legacy_prefix() {
        assert_eq!(
            CmdError::key_store("locked").to_string(),
            "key store: locked"
        );
    }

    #[test]
    fn display_internal_uses_legacy_prefix() {
        assert_eq!(CmdError::internal("oops").to_string(), "internal: oops");
    }

    #[test]
    fn cmd_error_implements_std_error() {
        // Compile-time assertion that CmdError is usable wherever
        // `&dyn std::error::Error` is expected (logging, `?` propagation
        // into `Box<dyn Error>` collectors).
        let e = CmdError::internal("boom");
        let _: &dyn std::error::Error = &e;
    }

    #[test]
    fn wire_shape_unchanged_for_setup_required_unit_variant() {
        // The renderer pattern-matches on the JSON shape from
        // src/types/error.ts. Migrating to the typify-generated enum must
        // not change the wire format.
        let json = serde_json::to_string(&CmdError::SetupRequired).unwrap();
        assert_eq!(json, r#"{"type":"setup_required"}"#);
    }

    #[test]
    fn wire_shape_unchanged_for_struct_variants() {
        // Generated CmdError uses #[serde(tag="type", content="message")]
        // on tuple variants — produces the same {"type":"...","message":"..."}
        // shape as M02's hand-rolled struct-variant enum with #[serde(tag="type")].
        let json = serde_json::to_string(&CmdError::provider("boom")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "provider");
        assert_eq!(value["message"], "boom");
    }

    #[test]
    fn error_message_as_str_returns_inner_string() {
        let m = ErrorMessage::try_from("hello").expect("non-empty");
        assert_eq!(m.as_str(), "hello");
    }
}
