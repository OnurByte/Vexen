use crate::{
    privacy::resolve_repo_path,
    store::{MemoryRecord, Store},
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Eligibility {
    Eligible,
    Stale,
    Superseded,
    WrongScope,
    MissingEvidence,
}

/// Immutable snapshot used by applicability checks. Constructed only from an
/// explicitly opened store and explicit query paths/conditions.
#[derive(Clone, Debug)]
pub struct RepoContext {
    root: Option<PathBuf>,
    repo_id: String,
    head: Option<String>,
    query_paths: BTreeSet<String>,
    conditions: Vec<String>,
}

impl RepoContext {
    pub fn new(
        store: &Store,
        query_paths: Vec<String>,
        conditions: Vec<String>,
    ) -> Result<Self, String> {
        let root = fs::canonicalize(store.root()).map_err(|e| e.to_string())?;
        if root != store.root() {
            return Err("store root is not canonical".into());
        }
        let mut paths = BTreeSet::new();
        for path in query_paths {
            let rel = Path::new(&path);
            resolve_repo_path(&root, rel).map_err(|e| e.to_string())?;
            paths.insert(path);
        }
        let output = git(&root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
        let head = String::from_utf8(output.stdout)
            .map_err(|_| "Git returned non-UTF8 HEAD".to_string())?;
        let head = head.trim().to_owned();
        if !valid_oid(&head) {
            return Err("Git returned an invalid HEAD object id".into());
        }
        Ok(Self {
            root: Some(root),
            repo_id: store.repo_id().to_owned(),
            head: Some(head),
            query_paths: paths,
            conditions,
        })
    }

    /// Empty context is useful for explicit abstention when repository
    /// resolution failed; it cannot make any record eligible.
    pub fn unavailable() -> Self {
        Self {
            root: None,
            repo_id: String::new(),
            head: None,
            query_paths: BTreeSet::new(),
            conditions: Vec::new(),
        }
    }

    pub fn repo_id(&self) -> &str {
        &self.repo_id
    }
    pub fn head(&self) -> Option<&str> {
        self.head.as_deref()
    }
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }
    pub fn query_paths(&self) -> &BTreeSet<String> {
        &self.query_paths
    }
}

pub fn check_applicability(record: &MemoryRecord, ctx: &RepoContext) -> Eligibility {
    let (Some(root), Some(head)) = (&ctx.root, &ctx.head) else {
        return Eligibility::WrongScope;
    };
    if record.repo_id != ctx.repo_id
        || !ctx.query_paths.contains(&record.anchor_path)
        || record.conditions != ctx.conditions
    {
        return Eligibility::WrongScope;
    }
    if record.anchor_path.is_empty()
        || record.anchor_sha256.len() != 64
        || !record.anchor_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        || record
            .git_commit
            .as_deref()
            .is_none_or(|oid| !valid_oid(oid))
    {
        return Eligibility::MissingEvidence;
    }
    let path = match resolve_repo_path(root, Path::new(&record.anchor_path)) {
        Ok(path) => path,
        Err(_) => return Eligibility::WrongScope,
    };
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Eligibility::MissingEvidence,
        Err(_) => return Eligibility::MissingEvidence,
    };
    if sha256(&bytes) != record.anchor_sha256.to_ascii_lowercase() {
        return Eligibility::Stale;
    }
    let commit = record.git_commit.as_deref().expect("validated above");
    let resolved = match git(
        root,
        &["rev-parse", "--verify", &format!("{commit}^{{commit}}")],
    ) {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        }
        _ => return Eligibility::Stale,
    };
    if !valid_oid(&resolved) {
        return Eligibility::Stale;
    }
    match git(root, &["merge-base", "--is-ancestor", &resolved, head]) {
        Ok(output) if output.status.success() => Eligibility::Eligible,
        _ => Eligibility::Stale,
    }
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())
}
fn valid_oid(value: &str) -> bool {
    (value.len() == 40 || value.len() == 64) && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
