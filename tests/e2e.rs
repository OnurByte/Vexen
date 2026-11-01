use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;
use vexen::store::{EventKind, NewEvent, NewOutcome, Store};

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

fn repository() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(dir.path()).unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "e2e@example.invalid"]);
    git(&root, &["config", "user.name", "E2E test"]);
    std::fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();
    git(&root, &["add", "main.rs"]);
    git(&root, &["commit", "-qm", "initial"]);
    (dir, root)
}

fn vexen(args: &[&str], root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vexen"))
        .args(args)
        .arg("--repo")
        .arg(root)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn attempt(key: &str) -> NewEvent {
    NewEvent {
        idempotency_key: key.into(),
        kind: EventKind::Attempt,
        summary: "retry loop duplicated a write".into(),
        anchor_path: "main.rs".into(),
        anchor_sha256: "0".repeat(64),
        git_commit: None,
        conditions: vec![],
    }
}

#[test]
fn init_is_idempotent_and_touches_only_the_vexen_directory() {
    let (_dir, root) = repository();
    assert!(vexen(&["init"], &root).status.success());
    assert!(vexen(&["init"], &root).status.success());
    assert!(root.join(".vexen/state.sqlite").is_file());
    assert!(!root.join(".gitignore").exists());
    let status = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(status.stdout).unwrap().trim(),
        "?? .vexen/"
    );
}

#[test]
fn init_rejects_a_bare_repository() {
    let dir = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(dir.path()).unwrap();
    git(&root, &["init", "-q", "--bare"]);
    let output = vexen(&["init"], &root);
    assert!(!output.status.success());
    assert!(!root.join(".vexen").exists());
}

#[test]
fn doctor_reports_uninitialized_then_healthy() {
    let (_dir, root) = repository();
    let before = vexen(&["doctor"], &root);
    assert!(!before.status.success());
    assert!(stdout(&before).contains("not initialized"));
    vexen(&["init"], &root);
    let after = vexen(&["doctor"], &root);
    assert!(after.status.success());
    let text = stdout(&after);
    assert!(text.contains("integrity: ok"), "{text}");
    assert!(text.contains("events: 0"), "{text}");
}

#[test]
fn check_abstains_on_an_empty_store() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let output = vexen(&["check", "--diff", "HEAD"], &root);
    assert!(output.status.success(), "{:?}", output);
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["status"], "no_relevant_evidence");
}

#[test]
fn check_without_init_fails_cleanly() {
    let (_dir, root) = repository();
    let output = vexen(&["check", "--diff", "HEAD"], &root);
    assert!(!output.status.success());
    assert!(!root.join(".vexen").exists());
}

#[test]
fn export_import_round_trip_is_idempotent() {
    let (_a, source) = repository();
    vexen(&["init"], &source);
    let store = Store::open(&source).unwrap();
    let id = store.append(attempt("attempt-1")).unwrap();
    store
        .outcome(
            id,
            NewOutcome {
                idempotency_key: "run-1".into(),
                test_id: "retry::duplicates".into(),
                exit_code: 1,
                output_sha256: None,
            },
        )
        .unwrap();
    drop(store);

    let exported = vexen(&["export", "--format", "jsonl"], &source);
    assert!(exported.status.success());
    let jsonl = stdout(&exported);
    assert_eq!(jsonl.lines().count(), 3, "header + 2 events: {jsonl}");

    let (_b, target) = repository();
    vexen(&["init"], &target);
    let file = target.join("..").join("export.jsonl");
    std::fs::write(&file, &jsonl).unwrap();
    let file = std::fs::canonicalize(file).unwrap();
    for _ in 0..2 {
        let imported = Command::new(env!("CARGO_BIN_EXE_vexen"))
            .args(["import"])
            .arg(&file)
            .arg("--repo")
            .arg(&target)
            .output()
            .unwrap();
        assert!(imported.status.success(), "{:?}", imported);
    }
    let store = Store::open(&target).unwrap();
    assert_eq!(store.events().unwrap().len(), 2);
    let again = vexen(&["export", "--format", "jsonl"], &target);
    assert_eq!(stdout(&again).lines().count(), 3);
    std::fs::remove_file(file).unwrap();
}

#[test]
fn import_rejects_malformed_input_without_partial_writes() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let bad = root.join("bad.jsonl");
    std::fs::write(&bad, "{\"vexen_export\":1}\nnot json\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vexen"))
        .args(["import"])
        .arg(&bad)
        .arg("--repo")
        .arg(&root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(Store::open(&root).unwrap().events().unwrap().len(), 0);
}

#[test]
fn markdown_export_lists_record_summaries() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    Store::open(&root)
        .unwrap()
        .append(attempt("attempt-1"))
        .unwrap();
    let output = vexen(&["export", "--format", "markdown"], &root);
    assert!(output.status.success());
    assert!(stdout(&output).contains("retry loop duplicated a write"));
}
