//! Filesystem permissions, path policy, secret redaction and privacy settings
//! shared by the desktop app and the CLI.
//!
//! Security checks that decide whether a config may be written live in
//! [`crate::ops`] and [`crate::workflow`]. This module is the storage-facing
//! half: how bytes are written, which paths are legal, and how secrets are
//! removed before they reach `history.json`.

mod capabilities;
mod paths;
mod perms;
mod privacy;
mod redact;

pub use capabilities::{materialize_env, SecretSupport, CLIENT_SECRET_SUPPORT};
pub use paths::{resolve_internal_relative, validate_client_config_path};
pub use perms::{atomic_replace_file, restrict_new_dir, restrict_new_file, tighten_app_data_dirs};
pub use privacy::{automatic_update_check_allowed, load_privacy, save_privacy, PrivacySettings};
pub use redact::{config_fingerprint, redact_config, redact_text, value_looks_secret, REDACTED};

/// Retained for callers that classified I/O errors before structured validation
/// existed. New code should use [`crate::ops::validate`] and
/// [`validate_client_config_path`].
pub fn is_high_risk_condition(err: &str) -> bool {
    err.contains("permission") || err.contains("invalid") || err.contains("refusing")
}
