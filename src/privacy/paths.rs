use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("path must be a safe repository-relative path")]
    Unsafe,
    #[error("path resolves outside the initialized repository")]
    OutsideRoot,
    #[error("path cannot be inspected: {0}")]
    Io(#[from] std::io::Error),
}

/// Resolve an untrusted repository-relative path while rejecting traversal and
/// symlink escapes. A missing leaf is allowed for query paths, but its nearest
/// existing parent must still be inside the checkout.
pub fn resolve_repo_path(root: &Path, relative: &Path) -> Result<PathBuf, PathError> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(PathError::Unsafe);
    }
    let canonical_root = fs::canonicalize(root)?;
    let joined = canonical_root.join(relative);
    let mut cursor = joined.as_path();
    loop {
        if cursor.exists() {
            let canonical = fs::canonicalize(cursor)?;
            if !canonical.starts_with(&canonical_root) {
                return Err(PathError::OutsideRoot);
            }
            let tail = joined.strip_prefix(cursor).map_err(|_| PathError::Unsafe)?;
            let result = if tail.as_os_str().is_empty() {
                canonical
            } else {
                canonical.join(tail)
            };
            if !result.starts_with(&canonical_root) {
                return Err(PathError::OutsideRoot);
            }
            return Ok(result);
        }
        cursor = cursor.parent().ok_or(PathError::Unsafe)?;
        if !cursor.starts_with(&canonical_root) {
            return Err(PathError::OutsideRoot);
        }
    }
}
