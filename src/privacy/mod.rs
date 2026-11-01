//! Local privacy and repository path guards.
mod paths;
mod redaction;

pub use paths::{PathError, resolve_repo_path};
pub use redaction::{contains_secret, redact_secrets};
