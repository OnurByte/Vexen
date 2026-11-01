use super::identity::RepoPath;

#[derive(Clone, Debug)]
pub struct ChangeTransaction {
    pub commit: String,
    pub paths: Vec<RepoPath>,
    pub renames: Vec<(RepoPath, RepoPath)>,
}

/// Keep transactions small enough to represent a plausible code change.
pub fn eligible(paths: &[RepoPath]) -> bool {
    if paths.is_empty() || paths.len() > 20 {
        return false;
    }
    !paths.iter().any(|p| {
        let lower = p.as_str().to_ascii_lowercase();
        ["/vendor/", "/generated/", "/dist/", "/build/", "/target/"]
            .iter()
            .any(|needle| lower.contains(needle))
            || lower.starts_with("vendor/")
            || lower.starts_with("generated/")
            || lower.starts_with("dist/")
            || lower.starts_with("build/")
            || lower.starts_with("target/")
    })
}
