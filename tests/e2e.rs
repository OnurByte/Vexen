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

fn run(args: &[&str], root: &Path) -> (bool, String, String) {
    let o = vexen(args, root);
    (
        o.status.success(),
        String::from_utf8(o.stdout).unwrap(),
        String::from_utf8(o.stderr).unwrap(),
    )
}

#[test]
fn record_then_check_surfaces_the_attempt_and_outcome_is_idempotent() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let (ok, out, err) = run(
        &[
            "record",
            "--path",
            "main.rs",
            "--summary",
            "retry loop duplicated a write",
        ],
        &root,
    );
    assert!(ok, "{err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    let id = value["items"][0]["event_id"].as_str().unwrap().to_owned();
    // Recording the same thing again is idempotent: same event id.
    let (_, again, _) = run(
        &[
            "record",
            "--path",
            "main.rs",
            "--summary",
            "retry loop duplicated a write",
        ],
        &root,
    );
    let again: serde_json::Value = serde_json::from_str(&again).unwrap();
    assert_eq!(again["items"][0]["event_id"], id.as_str());

    let (ok, _, err) = run(
        &["outcome", &id, "--test-id", "retry::duplicates", "--exit-code", "1"],
        &root,
    );
    assert!(ok, "{err}");
    let (_, check, _) = run(&["check", "--path", "main.rs"], &root);
    let check: serde_json::Value = serde_json::from_str(&check).unwrap();
    assert_ne!(check["status"], "no_relevant_evidence", "{check}");
    assert_eq!(Store::open(&root).unwrap().events().unwrap().len(), 2);
}

#[test]
fn record_refuses_secrets_missing_files_and_escapes() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    for (path, summary) in [
        ("main.rs", "token ghp_abcdefghijklmnopqrstuvwxyz0123456789"),
        ("nope.rs", "file does not exist"),
        ("../outside.rs", "path escapes the repo"),
    ] {
        let (ok, _, _) = run(&["record", "--path", path, "--summary", summary], &root);
        assert!(!ok, "{path} should be refused");
    }
    assert_eq!(Store::open(&root).unwrap().events().unwrap().len(), 0);
}

#[test]
fn record_can_store_an_accepted_decision() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let (ok, out, err) = run(
        &[
            "record",
            "--path",
            "main.rs",
            "--kind",
            "decision",
            "--summary",
            "keep one binary, no daemon",
        ],
        &root,
    );
    assert!(ok, "{err}");
    assert!(out.contains("accepted_decision"), "{out}");
}

#[test]
fn observe_runs_the_typed_command_and_stores_only_exit_code_and_hash() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let (_, rec, _) = run(&["record", "--path", "main.rs", "--summary", "flaky retry"], &root);
    let id = serde_json::from_str::<serde_json::Value>(&rec).unwrap()["items"][0]["event_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let o = Command::new(env!("CARGO_BIN_EXE_vexen"))
        .args(["observe", &id, "--test-id", "t::flaky", "--repo"])
        .arg(&root)
        .args(["--", "git", "--version"])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains("observed"));
    let events = Store::open(&root).unwrap().events().unwrap();
    let json = serde_json::to_string(&events[1]).unwrap();
    assert!(json.contains("\"exit_code\":0"), "{json}");
    assert!(json.contains("output_sha256"), "{json}");
    assert!(!json.contains("git version"), "raw output must not be stored: {json}");
}

#[test]
fn record_refuses_a_non_utf8_anchor_that_contains_a_secret() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    std::fs::write(root.join("blob.dat"), b"\xffAPI_KEY=abc\n").unwrap();
    let (ok, _, _) = run(&["record", "--path", "blob.dat", "--summary", "binary-ish file"], &root);
    assert!(!ok, "a secret in a non-UTF-8 file must not be recorded");
}

#[test]
fn record_refuses_an_oversized_anchor_without_reading_it_all() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let f = std::fs::File::create(root.join("big.bin")).unwrap();
    f.set_len(20 * 1024 * 1024).unwrap();
    let (ok, _, err) = run(&["record", "--path", "big.bin", "--summary", "too large"], &root);
    assert!(!ok);
    assert!(err.contains("too large"), "{err}");
}

#[test]
fn import_with_a_dangling_parent_writes_nothing() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let hash = "0".repeat(64);
    let record = format!(
        r#"{{"id":"a","parent_id":null,"idempotency_key":"k1","sequence":1,"kind":"attempt","payload":{{"type":"record","data":{{"idempotency_key":"k1","kind":"attempt","summary":"retry loop","anchor_path":"main.rs","anchor_sha256":"{hash}","git_commit":null,"conditions":[]}}}},"previous_hash":"","hash":"x"}}"#
    );
    let outcome = r#"{"id":"b","parent_id":"missing","idempotency_key":"outcome:missing:o1","sequence":2,"kind":"outcome","payload":{"type":"outcome","data":{"idempotency_key":"o1","test_id":"t","exit_code":1,"output_sha256":null}},"previous_hash":"x","hash":"y"}"#;
    let file = root.join("dangling.jsonl");
    std::fs::write(&file, format!("{{\"vexen_export\":1}}\n{record}\n{outcome}\n")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_vexen"))
        .args(["import"])
        .arg(&file)
        .arg("--repo")
        .arg(&root)
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert_eq!(Store::open(&root).unwrap().events().unwrap().len(), 0, "nothing may be written");
}

#[cfg(windows)]
#[test]
fn record_refuses_hidden_paths_written_with_backslashes() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    std::fs::create_dir_all(root.join("config")).unwrap();
    std::fs::write(root.join("config").join(".env"), "A=1\n").unwrap();
    let (ok, _, _) = run(&["record", "--path", r"config\.env", "--summary", "hidden"], &root);
    assert!(!ok);
}

#[test]
fn item_status_uses_the_same_snake_case_names_as_the_envelope() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let (_, rec, _) = run(&["record", "--path", "main.rs", "--summary", "retry loop"], &root);
    let id = serde_json::from_str::<serde_json::Value>(&rec).unwrap()["items"][0]["event_id"]
        .as_str()
        .unwrap()
        .to_owned();
    run(&["outcome", &id, "--test-id", "t", "--exit-code", "1"], &root);
    let (_, check, _) = run(&["check", "--path", "main.rs"], &root);
    let v: serde_json::Value = serde_json::from_str(&check).unwrap();
    assert_eq!(v["status"], "observed_failure");
    assert_eq!(v["items"][0]["status"], "observed_failure", "{v}");
}

fn import_file(root: &Path, name: &str, lines: &[String]) -> Output {
    let file = root.join(name);
    std::fs::write(&file, format!("{{\"vexen_export\":1}}\n{}\n", lines.join("\n"))).unwrap();
    Command::new(env!("CARGO_BIN_EXE_vexen"))
        .args(["import"])
        .arg(&file)
        .arg("--repo")
        .arg(root)
        .output()
        .unwrap()
}

fn record_line(id: &str, key: &str, summary: &str, seq: u32) -> String {
    let hash = "0".repeat(64);
    format!(
        r#"{{"id":"{id}","parent_id":null,"idempotency_key":"{key}","sequence":{seq},"kind":"attempt","payload":{{"type":"record","data":{{"idempotency_key":"{key}","kind":"attempt","summary":"{summary}","anchor_path":"main.rs","anchor_sha256":"{hash}","git_commit":null,"conditions":[]}}}},"previous_hash":"","hash":"x"}}"#
    )
}

#[test]
fn import_rejects_conflicting_duplicate_keys_without_writes() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let o = import_file(
        &root,
        "dup.jsonl",
        &[record_line("a", "k1", "first", 1), record_line("b", "k1", "different", 2)],
    );
    assert!(!o.status.success());
    assert_eq!(Store::open(&root).unwrap().events().unwrap().len(), 0, "nothing may be written");
}

#[test]
fn import_rejects_a_key_that_conflicts_with_an_existing_record() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    Store::open(&root).unwrap().append(attempt("k1")).unwrap();
    let o = import_file(
        &root,
        "clash.jsonl",
        &[record_line("a", "fresh", "new one", 1), record_line("b", "k1", "conflicting summary", 2)],
    );
    assert!(!o.status.success());
    assert_eq!(
        Store::open(&root).unwrap().events().unwrap().len(),
        1,
        "the valid first line must not be written when a later one conflicts"
    );
}

#[test]
fn check_diff_returns_only_notes_for_changed_files() {
    let (_dir, root) = repository();
    for f in ["a.rs", "b.rs"] {
        std::fs::write(root.join(f), format!("fn {}() {{}}\n", &f[..1])).unwrap();
    }
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-qm", "add a and b"]);
    vexen(&["init"], &root);
    std::fs::write(root.join("a.rs"), "fn a() { /* changed */ }\n").unwrap();
    for f in ["a.rs", "b.rs"] {
        let (_, rec, _) = run(&["record", "--path", f, "--summary", &format!("failure in {f}")], &root);
        let id = serde_json::from_str::<serde_json::Value>(&rec).unwrap()["items"][0]["event_id"]
            .as_str()
            .unwrap()
            .to_owned();
        run(&["outcome", &id, "--test-id", "t", "--exit-code", "1"], &root);
    }
    let (ok, out, err) = run(&["check", "--diff", "HEAD"], &root);
    assert!(ok, "{err}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let items = v["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "{v}");
    assert!(items[0]["text"].as_str().unwrap().contains("a.rs"), "{v}");
    let (bad, _, _) = run(&["check", "--diff=--help"], &root);
    assert!(!bad, "an option-like ref must be rejected");
}

#[cfg(unix)]
#[test]
fn observe_records_signal_termination_as_minus_one() {
    let (_dir, root) = repository();
    vexen(&["init"], &root);
    let (_, rec, _) = run(&["record", "--path", "main.rs", "--summary", "killed test"], &root);
    let id = serde_json::from_str::<serde_json::Value>(&rec).unwrap()["items"][0]["event_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let o = Command::new(env!("CARGO_BIN_EXE_vexen"))
        .args(["observe", &id, "--test-id", "t::killed", "--repo"])
        .arg(&root)
        .args(["--", "sh", "-c", "kill -TERM $$"])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let json = serde_json::to_string(&Store::open(&root).unwrap().events().unwrap()[1]).unwrap();
    assert!(json.contains("\"exit_code\":-1"), "{json}");
}

#[cfg(unix)]
#[test]
fn init_refuses_a_symlinked_vexen_directory_and_writes_nothing_outside() {
    let (_dir, root) = repository();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), root.join(".vexen")).unwrap();
    let o = vexen(&["init"], &root);
    assert!(!o.status.success());
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0, "nothing may be written through the link");
}
