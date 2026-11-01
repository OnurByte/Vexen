//! Local privacy and repository path guards.
mod paths;
mod redaction;

/// Largest file Vexen will anchor a note to; it is read whole to hash and scan it.
pub const MAX_ANCHOR_BYTES: u64 = 16 * 1024 * 1024;

pub use paths::{PathError, resolve_repo_path};
pub use redaction::{contains_secret, redact_secrets};
