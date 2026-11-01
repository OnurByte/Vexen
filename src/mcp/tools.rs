use super::schema::{CheckInput, ExplainInput, OutcomeInput, RecordInput};
use crate::{
    analysis::{AdvisoryStatus, RepoContext, rank_applicable},
    privacy::{contains_secret, resolve_repo_path},
    store::{EventId, EventKind, NewEvent, Store, StoreQuery},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use thiserror::Error;

pub use super::schema::Envelope;
#[derive(Debug, Error)]
pub enum McpInputError {
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("repository or evidence operation failed: {0}")]
    Store(String),
    #[error("requested evidence was not found")]
    NotFound,
}
type Result<T> = std::result::Result<T, McpInputError>;

/// Same snake_case vocabulary as the envelope's `status`.
fn advisory_status(status: AdvisoryStatus) -> &'static str {
    match status {
        AdvisoryStatus::ObservedFailure => "observed_failure",
        AdvisoryStatus::AcceptedDecision => "accepted_decision",
        AdvisoryStatus::Conflicted => "conflicted",
    }
}

pub fn check(input: CheckInput) -> Result<Envelope<serde_json::Value>> {
    let root = canonical_root(&input.repo_root)?;
    let mut query_paths = input.paths.unwrap_or_default();
    if let Some(reference) = &input.diff_ref {
        let diff_paths = validated_diff_paths(&root, reference, &query_paths)?;
        query_paths = if query_paths.is_empty() {
            diff_paths
        } else {
            query_paths
                .into_iter()
                .filter(|p| diff_paths.contains(p))
                .collect()
        };
    }
    let store = Store::open_read_only(&root).map_err(store_error)?;
    for p in &query_paths {
        resolve_repo_path(&root, Path::new(p))
            .map_err(|e| McpInputError::Invalid(e.to_string()))?;
    }
    let conditions = input.conditions.unwrap_or_default();
    let ctx = RepoContext::new(&store, query_paths, conditions).map_err(McpInputError::Invalid)?;
    let records = store
        .search(StoreQuery {
            text: input.query.unwrap_or_default(),
            limit: 100,
        })
        .map_err(store_error)?;
    let advisories = rank_applicable(&records, &ctx, 3);
    let items = advisories.iter().map(|a| serde_json::json!({"id":a.id,"status":advisory_status(a.status),"text":a.text})).collect::<Vec<_>>();
    let status = if advisories.is_empty() {
        "no_relevant_evidence"
    } else if advisories
        .iter()
        .any(|a| a.status != AdvisoryStatus::AcceptedDecision)
    {
        "observed_failure"
    } else {
        "accepted_decision"
    };
    Ok(envelope(store.repo_id(), items, status, None))
}

pub fn record(input: RecordInput) -> Result<Envelope<serde_json::Value>> {
    let root = canonical_root(&input.repo_root)?;
    validate_record(&root, &input.record)?;
    let store = Store::open(&root).map_err(store_error)?;
    let status = if input.record.kind == EventKind::AcceptedDecision {
        "accepted_decision"
    } else {
        "recorded"
    };
    let id = store.append(input.record).map_err(store_error)?;
    Ok(envelope(
        store.repo_id(),
        vec![serde_json::json!({"event_id":id.as_str()})],
        status,
        None,
    ))
}

pub fn outcome(input: OutcomeInput) -> Result<Envelope<serde_json::Value>> {
    if input.outcome.is_some() == input.supersede.is_some() {
        return Err(McpInputError::Invalid(
            "provide exactly one of outcome or supersede".into(),
        ));
    }
    let root = canonical_root(&input.repo_root)?;
    let store = Store::open(&root).map_err(store_error)?;
    let target = EventId::from(input.event_id);
    let (id, status) = if let Some(data) = input.outcome {
        (
            store.outcome(target, data).map_err(store_error)?,
            "observed",
        )
    } else {
        let data = input.supersede.unwrap();
        (
            store
                .supersede(target, data.idempotency_key, data.reason)
                .map_err(store_error)?,
            "recorded",
        )
    };
    Ok(envelope(
        store.repo_id(),
        vec![serde_json::json!({"event_id":id.as_str()})],
        status,
        None,
    ))
}

pub fn explain(input: ExplainInput) -> Result<Envelope<serde_json::Value>> {
    let root = canonical_root(&input.repo_root)?;
    let store = Store::open_read_only(&root).map_err(store_error)?;
    let chain = store
        .get(EventId::from(input.event_id))
        .map_err(|e| match e {
            crate::store::StoreError::NotFound(_) => McpInputError::NotFound,
            other => store_error(other),
        })?;
    let items = chain
        .events
        .iter()
        .map(|e| serde_json::to_value(e).unwrap_or_default())
        .collect();
    Ok(envelope(store.repo_id(), items, "explained", None))
}

fn validate_record(root: &Path, event: &NewEvent) -> Result<()> {
    if contains_secret(&event.summary)
        || event
            .conditions
            .iter()
            .any(|condition| contains_secret(condition))
    {
        return Err(McpInputError::Invalid(
            "secret-like text cannot be recorded".into(),
        ));
    }
    let path = resolve_repo_path(root, Path::new(&event.anchor_path))
        .map_err(|e| McpInputError::Invalid(e.to_string()))?;
    // Treat both separators alike: `config\.env` is as hidden as `config/.env`.
    let backslash = char::from(92u8);
    let normalized = event.anchor_path.to_ascii_lowercase().replace(backslash, "/");
    if normalized.split('/').any(|p| p.starts_with('.'))
        || normalized.contains("/secret")
        || normalized.starts_with("secret")
    {
        return Err(McpInputError::Invalid(
            "private or hidden paths cannot be recorded".into(),
        ));
    }
    if path.is_file() {
        let len = std::fs::metadata(&path)
            .map_err(|e| McpInputError::Invalid(e.to_string()))?
            .len();
        if len > crate::privacy::MAX_ANCHOR_BYTES {
            return Err(McpInputError::Invalid(format!(
                "anchor file is too large ({len} bytes; the limit is {})",
                crate::privacy::MAX_ANCHOR_BYTES
            )));
        }
        let bytes = std::fs::read(path).map_err(|e| McpInputError::Invalid(e.to_string()))?;
        // Scan lossy text so a stray invalid byte cannot hide a secret.
        if contains_secret(&String::from_utf8_lossy(&bytes)) {
            return Err(McpInputError::Invalid(
                "secret-like content cannot be recorded".into(),
            ));
        }
        let hash = hex(&Sha256::digest(&bytes));
        if hash != event.anchor_sha256.to_ascii_lowercase() {
            return Err(McpInputError::Invalid(
                "anchor_sha256 does not match current file contents".into(),
            ));
        }
    } else {
        return Err(McpInputError::Invalid(
            "anchor path must identify an existing file".into(),
        ));
    }
    Ok(())
}

/// Drop the Windows `\\?\` prefix from a drive-letter path; other paths are returned as is.
fn strip_verbatim(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with(r"UNC\") => PathBuf::from(rest),
        _ => path.to_path_buf(),
    }
}

fn canonical_root(raw: &str) -> Result<PathBuf> {
    let p = Path::new(raw);
    if !p.is_absolute() {
        return Err(McpInputError::Invalid(
            "repo_root must be an explicit absolute path".into(),
        ));
    }
    let root = std::fs::canonicalize(p).map_err(|e| McpInputError::Invalid(e.to_string()))?;
    // Windows canonicalizes to `\?\C:\...`; accept the ordinary `C:\...`
    // spelling of the same directory, but still reject `..`, symlinks and other
    // non-canonical spellings. The canonical form is what the store sees.
    if root != p && strip_verbatim(&root) != p {
        return Err(McpInputError::Invalid("repo_root must be canonical".into()));
    }
    Ok(root)
}

fn validated_diff_paths(root: &Path, reference: &str, paths: &[String]) -> Result<Vec<String>> {
    if reference.is_empty() || reference.starts_with('-') || reference.chars().any(char::is_control)
    {
        return Err(McpInputError::Invalid("invalid diff_ref".into()));
    }
    let verify = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--verify", "--end-of-options"])
        .arg(format!("{reference}^{{commit}}"))
        .output()
        .map_err(|e| McpInputError::Invalid(e.to_string()))?;
    if !verify.status.success() {
        return Err(McpInputError::Invalid("diff_ref is not a commit".into()));
    }
    let oid = String::from_utf8_lossy(&verify.stdout).trim().to_owned();
    if !matches!(oid.len(), 40 | 64) || !oid.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(McpInputError::Invalid(
            "diff_ref resolved to an invalid commit".into(),
        ));
    }
    for path in paths {
        resolve_repo_path(root, Path::new(path))
            .map_err(|e| McpInputError::Invalid(e.to_string()))?;
    }
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .args(["diff", "--name-only", "-z"])
        .arg(&oid)
        .arg("--");
    command.args(paths);
    let output = command
        .output()
        .map_err(|e| McpInputError::Invalid(e.to_string()))?;
    if !output.status.success() {
        return Err(McpInputError::Invalid("could not inspect diff_ref".into()));
    }
    Ok(output
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .filter_map(|s| std::str::from_utf8(s).ok().map(str::to_owned))
        .collect())
}

fn envelope<T: Serialize>(
    repo_id: &str,
    items: Vec<T>,
    status: &str,
    reason: Option<String>,
) -> Envelope<T> {
    Envelope {
        schema_version: 1,
        repo_id: repo_id.to_owned(),
        items,
        status: status.to_owned(),
        reason,
    }
}
fn store_error(e: crate::store::StoreError) -> McpInputError {
    McpInputError::Store(e.to_string())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
