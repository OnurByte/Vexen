//! `vexen observe-junit`: a test report becomes evidence for a record, once.

use std::{fs, path::Path, process::Command};

const BIN: &str = env!("CARGO_BIN_EXE_vexen");

fn run(dir: &Path, args: &[&str]) -> (bool, String) {
    let o = Command::new(BIN).current_dir(dir).args(args).output().unwrap();
    (o.status.success(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    for args in [vec!["init", "-q"], vec!["config", "user.email", "t@example.invalid"], vec!["config", "user.name", "t"]] {
        assert!(Command::new("git").arg("-C").arg(tmp.path()).args(&args).status().unwrap().success());
    }
    fs::write(tmp.path().join("retry.rs"), "fn retry() {}\n").unwrap();
    assert!(Command::new("git").arg("-C").arg(tmp.path()).args(["add", "-A"]).status().unwrap().success());
    assert!(Command::new("git").arg("-C").arg(tmp.path()).args(["commit", "-qm", "i"]).status().unwrap().success());
    assert!(run(tmp.path(), &["init"]).0);
    tmp
}

const REPORT: &str = r#"<testsuites><testsuite name="s">
<testcase classname="retry" name="no_double_charge"><failure message="a secret-looking message">stack</failure></testcase>
<testcase classname="retry" name="fine"/>
</testsuite></testsuites>"#;

fn record(dir: &Path) -> String {
    let (ok, out) = run(dir, &["record", "--path", "retry.rs", "--summary", "Retrying after a timeout charged twice"]);
    assert!(ok, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    v["items"][0]["event_id"].as_str().unwrap().to_owned()
}

#[test]
fn a_failing_test_in_a_report_makes_the_note_appear_and_importing_twice_adds_nothing() {
    let tmp = repo();
    let id = record(tmp.path());
    let report = tmp.path().join("junit.xml");
    fs::write(&report, REPORT).unwrap();
    let report = report.to_string_lossy().into_owned();

    let (ok, out) = run(tmp.path(), &["observe-junit", &id, &report]);
    assert!(ok, "{out}");
    assert!(out.contains("1 failing and 0 passing"), "{out}");
    assert!(out.contains("not a run Vexen observed"), "{out}");
    let (_, checked) = run(tmp.path(), &["check", "--path", "retry.rs"]);
    assert!(checked.contains("observed_failure"), "{checked}");

    let events = |dir: &Path| run(dir, &["export", "--format", "jsonl"]).1.lines().count();
    let before = events(tmp.path());
    assert!(run(tmp.path(), &["observe-junit", &id, &report]).0);
    assert_eq!(events(tmp.path()), before, "the same report twice must not add records");

    // Nothing from the report body reaches the store.
    let exported = run(tmp.path(), &["export", "--format", "jsonl"]).1;
    assert!(!exported.contains("secret-looking") && !exported.contains("stack"), "{exported}");
}

#[test]
fn a_report_without_failures_records_nothing_and_a_non_report_is_refused() {
    let tmp = repo();
    let id = record(tmp.path());
    let ok_report = tmp.path().join("green.xml");
    fs::write(&ok_report, r#"<testsuite><testcase classname="a" name="b"/></testsuite>"#).unwrap();
    let (ok, out) = run(tmp.path(), &["observe-junit", &id, &ok_report.to_string_lossy()]);
    assert!(ok && out.contains("nothing recorded"), "{out}");

    let bad = tmp.path().join("bad.xml");
    fs::write(&bad, "<html/>").unwrap();
    let (ok, out) = run(tmp.path(), &["observe-junit", &id, &bad.to_string_lossy()]);
    assert!(!ok && out.contains("JUnit"), "{out}");
}
