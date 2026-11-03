use super::{CliResult, path_str};
use crate::mcp::CheckInput;
use serde_json::{Value, json};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

/// Tools whose call edits a file; anything else is ignored.
const EDIT_TOOLS: [&str; 4] = ["Edit", "Write", "MultiEdit", "NotebookEdit"];

/// Claude Code `PreToolUse` hook. Reads the hook's JSON from standard input and, when the
/// file about to be edited has a note that still applies, answers with that note as
/// additional context. It never blocks the edit, never fails the tool call and prints
/// nothing when there is nothing to say, so it can run before every edit.
pub fn run_claude_code() -> CliResult {
    let mut input = String::new();
    std::io::stdin().take(1 << 20).read_to_string(&mut input)?;
    if let Some(output) = claude_code_context(&input) {
        println!("{output}");
    }
    Ok(())
}

pub(crate) fn claude_code_context(input: &str) -> Option<String> {
    let event: Value = serde_json::from_str(input).ok()?;
    if !EDIT_TOOLS.contains(&event.get("tool_name")?.as_str()?) {
        return None;
    }
    let cwd = PathBuf::from(event.get("cwd")?.as_str()?);
    let tool_input = event.get("tool_input")?;
    let file = ["file_path", "notebook_path", "path"]
        .iter()
        .find_map(|k| tool_input.get(*k)?.as_str())?;
    let root = repo_root(&cwd)?;
    if !root.join(".vexen").is_dir() {
        return None;
    }
    let relative = repo_relative(&root, &cwd, Path::new(file))?;
    let envelope = crate::mcp::check(CheckInput {
        repo_root: path_str(&root),
        paths: Some(vec![relative]),
        diff_ref: None,
        query: None,
        conditions: None,
    })
    .ok()?;
    let notes: Vec<&str> = envelope.items.iter().filter_map(|i| i["text"].as_str()).collect();
    if notes.is_empty() {
        return None;
    }
    let context = format!(
        "Vexen found earlier evidence about this file. It is advisory and describes one past code state; check it against the current code before relying on it. {}",
        notes.join(" | ")
    );
    Some(
        json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "additionalContext": context}})
            .to_string(),
    )
}

fn repo_root(cwd: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-cdup"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    std::fs::canonicalize(cwd.join(String::from_utf8_lossy(&out.stdout).trim())).ok()
}

/// The path as the repository sees it: relative, with forward slashes. The file may not
/// exist yet (a `Write` creating it), so only its folder is resolved.
fn repo_relative(root: &Path, cwd: &Path, file: &Path) -> Option<String> {
    let absolute = if file.is_absolute() { file.to_path_buf() } else { cwd.join(file) };
    let parent = std::fs::canonicalize(absolute.parent()?).ok()?;
    let full = parent.join(absolute.file_name()?);
    let relative = full.strip_prefix(root).ok()?;
    Some(relative.to_string_lossy().replace(char::from(92u8), "/"))
}
