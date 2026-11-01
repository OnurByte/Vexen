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
    raw_transactions: Vec<ChangeTransaction>,
    pub(crate) aliases: BTreeMap<RepoPath, RepoPath>,
    display_paths: BTreeMap<RepoPath, RepoPath>,
    skipped_non_utf8_timestamps: Vec<i64>,
    skipped_ambiguous_identity: usize,
}
impl HistoryIndex {
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }
    pub fn skipped_non_utf8_transactions(&self) -> usize {
        self.skipped_non_utf8_timestamps.len()
    }
    pub fn skipped_ambiguous_identity_transactions(&self) -> usize {
        self.skipped_ambiguous_identity
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
    pub fn raw_transactions(&self) -> &[ChangeTransaction] {
        &self.raw_transactions
    }
    /// Prefix with aliases learned only from transactions in that prefix.
    pub fn prefix(&self, count: usize) -> Self {
        let cutoff = count
            .checked_sub(1)
            .and_then(|i| self.raw_transactions.get(i))
            .map(|tx| tx.committer_timestamp);
        let raw = self
            .raw_transactions
            .iter()
            .filter(|tx| cutoff.is_some_and(|t| tx.committer_timestamp <= t))
            .cloned()
            .collect();
        let skipped = self
            .skipped_non_utf8_timestamps
            .iter()
            .filter(|t| cutoff.is_some_and(|end| **t <= end))
            .copied()
            .collect();
        project_history(raw, skipped)
    }
    /// Build an as-of index using only commits with committer time strictly before the cutoff.
    pub fn prefix_before(&self, timestamp: i64) -> Self {
        let raw = self
            .raw_transactions
            .iter()
            .filter(|tx| tx.committer_timestamp < timestamp)
            .cloned()
            .collect();
        let skipped = self
            .skipped_non_utf8_timestamps
            .iter()
            .filter(|time| **time < timestamp)
            .copied()
            .collect();
        project_history(raw, skipped)
    }
    /// Build an inclusive snapshot. Equal timestamps are evaluated together.
    pub fn through_timestamp(&self, timestamp: i64) -> Self {
        let raw = self
            .raw_transactions
            .iter()
            .filter(|tx| tx.committer_timestamp <= timestamp)
            .cloned()
            .collect();
        let skipped = self
            .skipped_non_utf8_timestamps
            .iter()
            .filter(|time| **time <= timestamp)
            .copied()
            .collect();
        project_history(raw, skipped)
    }
    pub fn paths_in_commit(&self, commit: &str) -> Option<&[RepoPath]> {
        self.transactions
            .iter()
            .find(|t| t.commit == commit)
            .map(|t| t.paths.as_slice())
    }
    pub fn canonical(&self, p: &RepoPath) -> RepoPath {
        self.aliases.get(p).cloned().unwrap_or_else(|| p.clone())
    }
    pub fn display_path(&self, p: &RepoPath) -> Option<RepoPath> {
        let identity = self.canonical(p);
        self.display_paths.get(&identity).cloned()
    }
}

fn rename_component(seed: &RepoPath, edges: &[(RepoPath, RepoPath)]) -> BTreeSet<RepoPath> {
    let mut found = BTreeSet::from([seed.clone()]);
    loop {
        let before = found.len();
        for (a, b) in edges {
            if found.contains(a) || found.contains(b) {
                found.insert(a.clone());
                found.insert(b.clone());
            }
        }
        if before == found.len() {
            return found;
        }
    }
}

fn project_history(
    raw: Vec<ChangeTransaction>,
    skipped_non_utf8_timestamps: Vec<i64>,
) -> HistoryIndex {
    let mut rename_edges = Vec::new();
    let mut retired = BTreeSet::new();
    let mut unsafe_paths = BTreeSet::new();
    let mut current_paths = BTreeSet::new();
    for tx in &raw {
        for path in &tx.added_paths {
            if retired.contains(path) || rename_edges.iter().any(|(a, b)| a == path || b == path) {
                unsafe_paths.extend(rename_component(path, &rename_edges));
            }
        }
        for (old, new) in &tx.renames {
            if unsafe_paths.contains(old) || unsafe_paths.contains(new) {
                unsafe_paths.extend(rename_component(old, &rename_edges));
                unsafe_paths.extend(rename_component(new, &rename_edges));
                unsafe_paths.insert(old.clone());
                unsafe_paths.insert(new.clone());
            }
        }
        rename_edges.extend(tx.renames.iter().cloned());
        retired.extend(tx.deleted_paths.iter().cloned());
        retired.extend(tx.renames.iter().map(|(old, _)| old.clone()));
        current_paths.extend(tx.paths.iter().cloned());
        for path in &tx.deleted_paths {
            current_paths.remove(path);
        }
        for (old, new) in &tx.renames {
            current_paths.remove(old);
            current_paths.insert(new.clone());
        }
    }
    let touches_unsafe = |tx: &&ChangeTransaction| {
        tx.paths.iter().any(|p| unsafe_paths.contains(p))
            || tx
                .renames
                .iter()
                .any(|(a, b)| unsafe_paths.contains(a) || unsafe_paths.contains(b))
    };
    let transactions: Vec<_> = raw
        .iter()
        .filter(|tx| !touches_unsafe(tx))
        .cloned()
        .collect();
    let skipped_ambiguous_identity = raw.iter().filter(touches_unsafe).count();
    let safe_edges: Vec<_> = transactions
        .iter()
        .flat_map(|tx| tx.renames.iter().cloned())
        .collect();
    let mut aliases = BTreeMap::new();
    let mut visited = BTreeSet::new();
    for (a, b) in &safe_edges {
        if visited.contains(a) {
            continue;
        }
        let component = rename_component(a, &safe_edges);
        let canonical = component
            .iter()
            .next()
            .cloned()
            .unwrap_or_else(|| a.clone());
        for path in component {
            visited.insert(path.clone());
            aliases.insert(path, canonical.clone());
        }
        visited.insert(b.clone());
    }
    let mut display_paths = BTreeMap::new();
    for path in &current_paths {
        let identity = aliases.get(path).cloned().unwrap_or_else(|| path.clone());
        display_paths
            .entry(identity)
            .or_insert_with(|| path.clone());
    }
    for tx in &transactions {
        for (_, new) in &tx.renames {
            let identity = aliases.get(new).cloned().unwrap_or_else(|| new.clone());
            if current_paths.contains(new) {
                display_paths.insert(identity, new.clone());
            }
        }
    }
    HistoryIndex {
        transactions,
        raw_transactions: raw,
        aliases,
        display_paths,
        skipped_non_utf8_timestamps,
        skipped_ambiguous_identity,
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
    if limit == 0 {
        return Ok(HistoryIndex::default());
    }
    let root = git(repo_root, &["rev-parse", "--git-dir"])?;
    if root.is_empty() {
        return Err("not a Git repository".into());
    }
    if git(repo_root, &["rev-parse", "--verify", "HEAD"]).is_err() {
        return Ok(HistoryIndex::default());
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
            "--format=%x1e%H%x00%ct",
        ],
    )?;
    let fields: Vec<&[u8]> = raw.split(|b| *b == 0).collect();
    let mut i = 0;
    let mut raw_transactions = Vec::new();
    let mut skipped_non_utf8_timestamps = Vec::new();
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
        let timestamp =
            std::str::from_utf8(fields.get(i).ok_or("missing Git committer timestamp")?)
                .map_err(|e| format!("invalid Git committer timestamp: {e}"))?
                .parse::<i64>()
                .map_err(|e| format!("invalid Git committer timestamp: {e}"))?;
        i += 1;
        let mut paths = BTreeSet::new();
        let mut renames = Vec::new();
        let mut added_paths = Vec::new();
        let mut deleted_paths = Vec::new();
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
                        if status.starts_with('A') {
                            added_paths.push(path.clone());
                        }
                        if status.starts_with('D') {
                            deleted_paths.push(path.clone());
                        }
                        paths.insert(path);
                    }
                    Err(_) => invalid_path = true,
                }
                i += 1;
            }
        }
        if invalid_path {
            // Never silently drop a path and turn one transaction into false pairs.
            skipped_non_utf8_timestamps.push(timestamp);
            continue;
        }
        let paths: Vec<_> = paths.into_iter().collect();
        if changes::eligible(&paths) {
            raw_transactions.push(ChangeTransaction {
                commit: oid,
                committer_timestamp: timestamp,
                paths,
                renames,
                added_paths,
                deleted_paths,
            });
        }
    }
    // Stable sorting keeps Git's parent-before-child topological order for equal
    // committer timestamps. The benchmark groups equal timestamps at cutoffs.
    raw_transactions.sort_by_key(|tx| tx.committer_timestamp);
    Ok(project_history(
        raw_transactions,
        skipped_non_utf8_timestamps,
    ))
}
