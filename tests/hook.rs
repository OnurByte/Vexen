//! `vexen hook claude-code`: the PreToolUse hook that surfaces a note before an edit.
//! It must speak only when a note applies, and must never fail or block the edit.

use std::{fs, io::Write, path::Path, process::{Command, Stdio}};

const BIN: &str = env!("CARGO_BIN_EXE_vexen");

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git").arg("-C").arg(dir).args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"]).args(args).status().unwrap().success();
    assert!(ok, "git {args:?}");
}

fn vexen(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(BIN).current_dir(dir).args(args).output().unwrap();
    assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A repository with `src/pay.rs` carrying a recorded, failing lesson.
fn repo_with_lesson() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    git(&root, &["init", "-q"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/pay.rs"), "fn pay() {}\n").unwrap();
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-qm", "init"]);
    vexen(&root, &["init"]);
    let recorded: serde_json::Value = serde_json::from_str(&vexen(&root, &["record", "--path", "src/pay.rs", "--summary", "Retrying the charge twice billed the customer"])).unwrap();
    let id = recorded["items"][0]["event_id"].as_str().unwrap().to_owned();
    vexen(&root, &["outcome", &id, "--test-id", "pay_once", "--exit-code", "1"]);
    (tmp, root)
}

fn hook(input: &str) -> (bool, String) {
    let mut child = Command::new(BIN).args(["hook", "claude-code"]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned())
}

fn event(tool: &str, cwd: &Path, file: &Path) -> String {
    serde_json::json!({"tool_name": tool, "cwd": cwd.to_string_lossy(), "tool_input": {"file_path": file.to_string_lossy()}}).to_string()
}

#[test]
fn an_edit_of_a_file_with_a_valid_note_gets_the_note_as_context() {
    let (_tmp, root) = repo_with_lesson();
    let (ok, out) = hook(&event("Edit", &root, &root.join("src/pay.rs")));
    assert!(ok);
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    let context = v["hookSpecificOutput"]["additionalContext"].as_str().unwrap();
    assert!(context.contains("Retrying the charge twice billed the customer"), "{context}");
    assert!(context.contains("advisory"), "{context}");
    assert!(v.get("decision").is_none() && v["hookSpecificOutput"].get("permissionDecision").is_none(), "the hook must never decide for the user");
}

#[test]
fn it_works_from_a_subfolder_and_with_a_relative_path() {
    let (_tmp, root) = repo_with_lesson();
    let input = serde_json::json!({"tool_name": "Write", "cwd": root.join("src").to_string_lossy(), "tool_input": {"file_path": "pay.rs"}}).to_string();
    let (ok, out) = hook(&input);
    assert!(ok);
    assert!(out.contains("Retrying the charge"), "{out}");
}

#[test]
fn it_is_silent_when_nothing_applies() {
    let (_tmp, root) = repo_with_lesson();
    // another file, a new file, the same file after it changed, a tool that is not an edit
    fs::write(root.join("other.rs"), "x\n").unwrap();
    for (tool, file) in [("Edit", root.join("other.rs")), ("Write", root.join("brand_new.rs")), ("Read", root.join("src/pay.rs"))] {
        let (ok, out) = hook(&event(tool, &root, &file));
        assert!(ok && out.trim().is_empty(), "{tool} {}: {out}", file.display());
    }
    fs::write(root.join("src/pay.rs"), "fn pay() { /* fixed */ }\n").unwrap();
    let (ok, out) = hook(&event("Edit", &root, &root.join("src/pay.rs")));
    assert!(ok && out.trim().is_empty(), "a stale note must stay quiet: {out}");
}

#[test]
fn it_never_fails_on_bad_input_or_outside_a_project() {
    for input in ["", "not json", "{}", "{\"tool_name\":\"Edit\"}"] {
        let (ok, out) = hook(input);
        assert!(ok && out.trim().is_empty(), "{input:?}: {out}");
    }
    let plain = tempfile::tempdir().unwrap();
    fs::write(plain.path().join("a.rs"), "x").unwrap();
    let (ok, out) = hook(&event("Edit", plain.path(), &plain.path().join("a.rs")));
    assert!(ok && out.trim().is_empty());
}
