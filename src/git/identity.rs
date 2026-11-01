use std::fmt;

/// A repository-relative Git path. Traversal and absolute paths are rejected.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepoPath(String);

impl RepoPath {
    pub fn new(path: impl AsRef<str>) -> Result<Self, String> {
        let original = path.as_ref();
        let s = original;
        let portable = if cfg!(windows) {
            s.replace('\\', "/")
        } else {
            s.to_string()
        };
        let bytes = s.as_bytes();
        let windows_drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
        if s.is_empty()
            || s.starts_with('/')
            || windows_drive
            || s.contains('\0')
            || portable
                .split('/')
                .any(|part| part.is_empty() || part == ".." || part == ".")
        {
            return Err(format!("not a safe repository-relative path: {s:?}"));
        }
        Ok(Self(s.to_string()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub(crate) fn from_git_bytes(b: &[u8]) -> Result<Self, String> {
        let s = std::str::from_utf8(b).map_err(|e| format!("Git path is not UTF-8: {e}"))?;
        Self::new(s)
    }
    pub fn parent(&self) -> &str {
        self.0.rsplit_once('/').map(|(p, _)| p).unwrap_or("")
    }
    pub fn basename(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or("")
    }
}
impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
