use super::{CliResult, RepoArg, path_str};
use crate::{
    mcp::{OutcomeInput, RecordInput},
    store::{EventKind, NewEvent, NewOutcome},
};
use sha2::{Digest, Sha256};
use std::process::Command;

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Kind {
    Attempt,
    Decision,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.len().to_le_bytes());
        hasher.update(part.as_bytes());
    }
    hex(&hasher.finalize())
}

pub fn record(
    repo: &RepoArg,
    path: String,
    summary: String,
    kind: Kind,
    key: Option<String>,
    conditions: Vec<String>,
) -> CliResult {
    let root = repo.root()?;
    // The store checks that the anchor hash matches the file; hash it here so
    // the caller only names the file. Safety checks stay in the shared path.
    let len = std::fs::metadata(root.join(&path))
        .map_err(|e| format!("cannot read anchor file `{path}`: {e}"))?
        .len();
    if len > crate::privacy::MAX_ANCHOR_BYTES {
        return Err(format!("anchor file `{path}` is too large ({len} bytes; the limit is {})", crate::privacy::MAX_ANCHOR_BYTES).into());
    }
    let bytes = std::fs::read(root.join(&path))
        .map_err(|e| format!("cannot read anchor file `{path}`: {e}"))?;
    let file_hash = hex(&Sha256::digest(&bytes));
    let head = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--verify", "HEAD^{commit}"])
        .output()?;
    let git_commit = head
        .status
        .success()
        .then(|| String::from_utf8_lossy(&head.stdout).trim().to_owned());
    let (kind, tag) = match kind {
        Kind::Attempt => (EventKind::Attempt, "attempt"),
        Kind::Decision => (EventKind::AcceptedDecision, "decision"),
    };
    let idempotency_key = key.unwrap_or_else(|| digest(&[tag, &path, &file_hash, &summary]));
    let envelope = crate::mcp::record(RecordInput {
        repo_root: path_str(&root),
        record: NewEvent {
            idempotency_key,
            kind,
            summary,
            anchor_path: path,
            anchor_sha256: file_hash,
            git_commit,
            conditions,
        },
    })?;
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}

pub fn outcome(
    repo: &RepoArg,
    id: String,
    test_id: String,
    exit_code: i32,
    key: Option<String>,
    output_sha256: Option<String>,
) -> CliResult {
    let root = repo.root()?;
    let idempotency_key =
        key.unwrap_or_else(|| digest(&[&test_id, &exit_code.to_string()]));
    let envelope = crate::mcp::outcome(OutcomeInput {
        repo_root: path_str(&root),
        event_id: id,
        outcome: Some(NewOutcome {
            idempotency_key,
            test_id,
            exit_code,
            output_sha256,
        }),
        supersede: None,
    })?;
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}
