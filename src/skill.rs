//! The agent skill that ships inside the binary, and how `vexen setup`
//! writes it without ever overwriting a file the user changed.
//!
//! The text is embedded from `skills/vexen/SKILL.md`, so the binary and the
//! skill cannot drift apart. The installed file gets a short block naming the
//! command that works on this machine (so the skill never depends on `PATH`)
//! and a last line recording a hash of everything above it. A file whose hash
//! no longer matches was edited by the user and is left alone.

use crate::install::{Launch, Scope};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

const EMBEDDED: &str = include_str!("../skills/vexen/SKILL.md");
const MARKER_PREFIX: &str = "<!-- vexen:managed sha256=";

#[derive(Debug, Eq, PartialEq)]
pub enum SkillOutcome {
    Created(PathBuf),
    Updated(PathBuf),
    Unchanged(PathBuf),
    Removed(PathBuf),
    /// The file exists and is not an unmodified copy of ours; nothing was changed.
    Foreign(PathBuf),
    /// Nothing to remove.
    Absent(PathBuf),
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest(text: &str) -> String {
    hex(&Sha256::digest(text.as_bytes()))
}

/// The embedded skill text with LF line endings (a Windows checkout may use CRLF).
pub fn embedded() -> String {
    EMBEDDED.replace("\r\n", "\n")
}

/// How the agent should run Vexen on this machine, as one inline command.
fn command_line(launch: &Launch) -> String {
    let quote = |s: &str| if s.contains(' ') { format!("\"{s}\"") } else { s.to_owned() };
    std::iter::once(launch.command.as_str())
        .chain(launch.args.iter().map(String::as_str))
        .take_while(|a| *a != "mcp")
        .map(quote)
        .collect::<Vec<_>>()
        .join(" ")
}

/// The file content to install: the embedded skill with a machine note after
/// the front matter and the ownership marker at the end.
pub fn render(launch: &Launch) -> String {
    let text = embedded();
    let note = format!(
        "\n> **On this machine, run Vexen as `{}`.** `vexen setup` wrote this line; use that command wherever this guide says `vexen`.\n",
        command_line(launch)
    );
    let body = match text.strip_prefix("---\n").and_then(|rest| rest.find("\n---\n").map(|i| (rest, i))) {
        Some((rest, i)) => {
            let split = 4 + i + 5;
            let _ = rest;
            format!("{}{}{}", &text[..split], note, &text[split..])
        }
        None => format!("{note}{text}"),
    };
    let body = if body.ends_with('\n') { body } else { format!("{body}\n") };
    format!("{body}{MARKER_PREFIX}{} -->\n", digest(&body))
}

/// True if `existing` is a file we wrote and nobody edited.
fn is_ours(existing: &str) -> bool {
    let Some(at) = existing.rfind(MARKER_PREFIX) else { return false };
    let (body, marker) = existing.split_at(at);
    let recorded = marker
        .trim_end()
        .strip_prefix(MARKER_PREFIX)
        .and_then(|m| m.strip_suffix("-->"))
        .map(str::trim);
    recorded == Some(digest(body).as_str())
}

/// Where each supported agent reads skills. Only paths that were checked are
/// listed: other agents get a pointer to `npx skills add` instead.
pub fn target(agent: &str, scope: Scope, home: &Path, repo: &Path) -> Option<PathBuf> {
    let base = match (agent, scope) {
        ("claude-code", Scope::Global) => home.join(".claude").join("skills"),
        ("claude-code", Scope::Project) => repo.join(".claude").join("skills"),
        _ => return None,
    };
    Some(base.join("vexen").join("SKILL.md"))
}

pub fn install(path: &Path, content: &str, dry_run: bool, force: bool) -> Result<SkillOutcome, String> {
    match fs::read_to_string(path) {
        Ok(existing) => {
            if existing == content {
                return Ok(SkillOutcome::Unchanged(path.to_owned()));
            }
            if !is_ours(&existing) && !force {
                return Ok(SkillOutcome::Foreign(path.to_owned()));
            }
            if !dry_run {
                write(path, content)?;
            }
            Ok(SkillOutcome::Updated(path.to_owned()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if !dry_run {
                write(path, content)?;
            }
            Ok(SkillOutcome::Created(path.to_owned()))
        }
        Err(e) => Err(format!("cannot read {}: {e}", path.display())),
    }
}

pub fn remove(path: &Path, dry_run: bool, force: bool) -> Result<SkillOutcome, String> {
    let existing = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(SkillOutcome::Absent(path.to_owned())),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    if !is_ours(&existing) && !force {
        return Ok(SkillOutcome::Foreign(path.to_owned()));
    }
    if !dry_run {
        fs::remove_file(path).map_err(|e| format!("cannot remove {}: {e}", path.display()))?;
        // Tidy the now-empty folder we created; never anything else.
        if let Some(dir) = path.parent() {
            let _ = fs::remove_dir(dir);
        }
    }
    Ok(SkillOutcome::Removed(path.to_owned()))
}

fn write(path: &Path, content: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("skill path has no parent")?;
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".vexen-tmp-{}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, content).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("cannot replace {}: {e}", path.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn launch() -> Launch {
        Launch::binary("C:/Program Files/Vexen/vexen.exe")
    }

    #[test]
    fn the_rendered_skill_keeps_its_front_matter_first_and_names_the_command() {
        let text = render(&launch());
        assert!(text.starts_with("---\nname: vexen\n"), "{}", &text[..60]);
        assert!(text.contains("\"C:/Program Files/Vexen/vexen.exe\""));
        assert!(text.trim_end().ends_with("-->"));
        assert!(is_ours(&text));
    }

    #[test]
    fn any_edit_makes_the_file_foreign() {
        let text = render(&launch());
        assert!(!is_ours(&text.replace("Vexen", "vexen!")));
        assert!(!is_ours(&embedded()));
    }

    #[test]
    fn the_embedded_skill_is_the_repository_file() {
        let on_disk = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/skills/vexen/SKILL.md")).unwrap();
        assert_eq!(embedded(), on_disk.replace("\r\n", "\n"));
    }

    #[test]
    fn npx_launch_is_described_without_the_server_arguments() {
        let line = command_line(&Launch::npx());
        assert!(line.contains("npx") && line.contains("@onurbyte/vexen"), "{line}");
        assert!(!line.contains("mcp") && !line.contains("--stdio"), "{line}");
    }
}
