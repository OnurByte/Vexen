use super::{
    changes::{self, ChangeTransaction},
    identity::RepoPath,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Default)]
pub struct HistoryIndex {
    pub(crate) transactions: Vec<ChangeTransaction>,
    pub(crate) aliases: BTreeMap<RepoPath, RepoPath>,
    pub(crate) paths: BTreeSet<RepoPath>,
    skipped_non_utf8: usize,
}
impl HistoryIndex {
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }
    pub fn skipped_non_utf8_transactions(&self) -> usize {
        self.skipped_non_utf8
    }
    pub fn path_count(&self, path: &RepoPath) -> usize {
        let key = self.canonical(path);
        self.transactions
            .iter()
            .filter(|t| t.paths.iter().any(|p| self.canonical(p) == key))
            .count()
    }
    pub fn commits(&self) -> impl Iterator<Item = &str> {
        self.transactions.iter().map(|t| t.commit.as_str())
    }
    pub fn transactions(&self) -> &[ChangeTransaction] {
        &self.transactions
    }
    /// Prefix with aliases learned only from transactions in that prefix.
    pub fn prefix(&self, count: usize) -> Self {
        let transactions = self
            .transactions
            .iter()
            .take(count)
            .cloned()
            .collect::<Vec<_>>();
        let mut aliases = BTreeMap::new();
        let mut paths = BTreeSet::new();
        for tx in &transactions {
            for (old, new) in &tx.renames {
                aliases.insert(old.clone(), new.clone());
                paths.insert(old.clone());
                paths.insert(new.clone());
            }
            paths.extend(tx.paths.iter().cloned());
        }
        Self {
            transactions,
            aliases,
            paths,
            skipped_non_utf8: 0,
        }
    }
    pub fn paths_in_commit(&self, commit: &str) -> Option<&[RepoPath]> {
        self.transactions
            .iter()
            .find(|t| t.commit == commit)
            .map(|t| t.paths.as_slice())
    }
    pub fn canonical(&self, p: &RepoPath) -> RepoPath {
        let mut cur = p.clone();
        for _ in 0..self.aliases.len().saturating_add(1) {
            match self.aliases.get(&cur) {
                Some(next) if next != &cur => cur = next.clone(),
                _ => break,
            }
        }
        cur
    }
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("failed to start git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

fn is_oid(token: &[u8]) -> bool {
    (token.len() == 40 || token.len() == 64) && token.iter().all(u8::is_ascii_hexdigit)
}

/// Index reachable non-merge commits, oldest to newest within the requested limit.
/// Shallow repositories naturally yield only the history made available by Git.
pub fn analyze_history(repo_root: &Path, limit: usize) -> Result<HistoryIndex> {
    let mut index = HistoryIndex::default();
    if limit == 0 {
        return Ok(index);
    }
    let root = git(repo_root, &["rev-parse", "--git-dir"])?;
    if root.is_empty() {
        return Err("not a Git repository".into());
    }
    if git(repo_root, &["rev-parse", "--verify", "HEAD"]).is_err() {
        return Ok(index);
    }
    let limit_s = limit.min(i32::MAX as usize).to_string();
    // One batched history walk avoids a process per commit. NUL-delimited fields
    // preserve arbitrary path bytes; a 40/64-hex token is a commit header only
    // when the parser expects a status, never when it expects a path.
    let raw = git(
        repo_root,
        &[
            "log",
            "--reverse",
            "--topo-order",
            "--no-merges",
            "--max-count",
            &limit_s,
            "--name-status",
            "-z",
            "--find-renames=50%",
            "--format=%x1e%H",
        ],
    )?;
    let fields: Vec<&[u8]> = raw.split(|b| *b == 0).collect();
    let mut i = 0;
    while i < fields.len() {
        while i < fields.len() && fields[i].is_empty() {
            i += 1;
        }
        if i == fields.len() {
            break;
        }
        let commit_token = fields[i].strip_prefix(b"\x1e").unwrap_or(fields[i]);
        if !is_oid(commit_token) {
            return Err("malformed Git log commit header".into());
        }
        let oid = std::str::from_utf8(commit_token)
            .map_err(|e| format!("invalid commit id: {e}"))?
            .to_string();
        i += 1;
        let mut paths = BTreeSet::new();
        let mut renames = Vec::new();
        let mut invalid_path = false;
        while i < fields.len() {
            let token = fields[i];
            if token.is_empty() {
                i += 1;
                continue;
            }
            let status_bytes = token.strip_prefix(b"\n").unwrap_or(token);
            let next_header = status_bytes.strip_prefix(b"\x1e").unwrap_or(status_bytes);
            if is_oid(next_header) {
                break;
            }
            let status = std::str::from_utf8(status_bytes)
                .map_err(|e| format!("invalid Git name-status: {e}"))?;
            i += 1;
            if status.starts_with('R') || status.starts_with('C') {
                if i + 1 >= fields.len() {
                    return Err("truncated Git rename record".into());
                }
                match (
                    RepoPath::from_git_bytes(fields[i]),
                    RepoPath::from_git_bytes(fields[i + 1]),
                ) {
                    (Ok(old), Ok(new)) => {
                        paths.insert(new.clone());
                        renames.push((old, new));
                    }
                    _ => invalid_path = true,
                }
                i += 2;
            } else {
                if i >= fields.len() {
                    return Err("truncated Git path record".into());
                }
                match RepoPath::from_git_bytes(fields[i]) {
                    Ok(path) => {
                        paths.insert(path);
                    }
                    Err(_) => invalid_path = true,
                }
                i += 1;
            }
        }
        if invalid_path {
            // Never silently drop a path and turn one transaction into false pairs.
            index.skipped_non_utf8 += 1;
            continue;
        }
        let paths: Vec<_> = paths.into_iter().collect();
        if changes::eligible(&paths) {
            for (old, new) in &renames {
                index.aliases.insert(old.clone(), new.clone());
                index.paths.insert(old.clone());
                index.paths.insert(new.clone());
            }
            index.paths.extend(paths.iter().cloned());
            index.transactions.push(ChangeTransaction {
                commit: oid,
                paths,
                renames,
            });
        }
    }
    Ok(index)
}
