//! Generated lifecycles: random sequences of record, outcome, edit, commit and
//! supersede, checked against a small model of what Vexen must and must not show.
//! Failing cases are minimized by proptest; keep any minimized case as a fixed test.
//!
//! Invariants (the product's promises):
//! - the event chain stays intact;
//! - a note is shown only while its file still has the recorded content and it is
//!   not superseded (never a stale note), and only for the file asked about;
//! - a valid note with a failing result is not hidden (a failure is not buried);
//! - at most three items come back;
//! - an export imports into a fresh checkout without loss, and importing twice changes nothing.

use proptest::prelude::*;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};
use vexen::{
    mcp::{CheckInput, OutcomeInput, RecordInput, SupersedeInput, check, outcome, record},
    store::{EventKind, NewEvent, NewOutcome, Store},
};

const FILES: usize = 3;

#[derive(Clone, Debug)]
enum Op {
    Record { file: usize, text: u8 },
    Outcome { rec: usize, failing: bool },
    Edit { file: usize, variant: u8 },
    Commit,
    Supersede { rec: usize },
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => (0..FILES, 0u8..4).prop_map(|(file, text)| Op::Record { file, text }),
        3 => (0usize..16, any::<bool>()).prop_map(|(rec, failing)| Op::Outcome { rec, failing }),
        3 => (0..FILES, 0u8..3).prop_map(|(file, variant)| Op::Edit { file, variant }),
        1 => Just(Op::Commit),
        1 => (0usize..16).prop_map(|rec| Op::Supersede { rec }),
    ]
}

struct Rec {
    id: String,
    file: usize,
    hash: String,
    outcomes: Vec<i32>,
    superseded: bool,
}

fn sha(text: &str) -> String {
    Sha256::digest(text.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

fn content(file: usize, variant: u8) -> String {
    format!("fn f{file}() {{ /* v{variant} */ }}\n")
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

fn run_case(ops: &[Op]) -> Result<(), TestCaseError> {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    git(&root, &["init", "-q"]);
    let mut contents: Vec<String> = (0..FILES).map(|f| content(f, 0)).collect();
    for (f, c) in contents.iter().enumerate() {
        fs::write(root.join(format!("f{f}.rs")), c).unwrap();
    }
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-qm", "init"]);
    Store::initialize(&root).unwrap();
    let repo_root = root.to_string_lossy().into_owned();
    let mut recs: Vec<Rec> = Vec::new();

    for op in ops {
        match op {
            Op::Record { file, text } => {
                let hash = sha(&contents[*file]);
                let envelope = record(RecordInput {
                    repo_root: repo_root.clone(),
                    record: NewEvent {
                        idempotency_key: format!("k{}", recs.len()),
                        kind: EventKind::Attempt,
                        summary: format!("lesson {text} about f{file}.rs"),
                        anchor_path: format!("f{file}.rs"),
                        anchor_sha256: hash.clone(),
                        git_commit: Some(git(&root, &["rev-parse", "HEAD"])),
                        conditions: vec![],
                    },
                })
                .map_err(|e| TestCaseError::fail(format!("record refused: {e}")))?;
                let id = envelope.items[0]["event_id"].as_str().unwrap().to_owned();
                recs.push(Rec { id, file: *file, hash, outcomes: vec![], superseded: false });
            }
            Op::Outcome { rec, failing } if !recs.is_empty() => {
                let i = rec % recs.len();
                let code = i32::from(*failing);
                let r = outcome(OutcomeInput {
                    repo_root: repo_root.clone(),
                    event_id: recs[i].id.clone(),
                    outcome: Some(NewOutcome {
                        idempotency_key: format!("o{}", recs[i].outcomes.len()),
                        test_id: "t".into(),
                        exit_code: code,
                        output_sha256: None,
                    }),
                    supersede: None,
                });
                if r.is_ok() {
                    recs[i].outcomes.push(code);
                }
            }
            Op::Edit { file, variant } => {
                contents[*file] = content(*file, *variant);
                fs::write(root.join(format!("f{file}.rs")), &contents[*file]).unwrap();
            }
            Op::Commit => {
                git(&root, &["add", "-A"]);
                git(&root, &["commit", "-qm", "c", "--allow-empty"]);
            }
            Op::Supersede { rec } if !recs.is_empty() => {
                let i = rec % recs.len();
                let r = outcome(OutcomeInput {
                    repo_root: repo_root.clone(),
                    event_id: recs[i].id.clone(),
                    outcome: None,
                    supersede: Some(SupersedeInput { idempotency_key: format!("s{i}"), reason: "generated".into() }),
                });
                if r.is_ok() {
                    recs[i].superseded = true;
                }
            }
            _ => {}
        }
    }

    // The chain is intact.
    prop_assert!(Store::open_read_only(&root).unwrap().verify_integrity().unwrap());

    for (f, file_content) in contents.iter().enumerate() {
        let current = sha(file_content);
        let path = format!("f{f}.rs");
        let envelope = check(CheckInput {
            repo_root: repo_root.clone(),
            paths: Some(vec![path.clone()]),
            diff_ref: None,
            query: None,
            conditions: None,
        })
        .map_err(|e| TestCaseError::fail(format!("check failed: {e}")))?;
        prop_assert!(envelope.items.len() <= 3, "more than three items");
        for item in &envelope.items {
            let id = item["id"].as_str().unwrap();
            let rec = recs.iter().find(|r| r.id == id);
            prop_assert!(rec.is_some(), "an unknown record was shown");
            let rec = rec.unwrap();
            prop_assert_eq!(rec.file, f, "a note for another file was shown");
            prop_assert_eq!(&rec.hash, &current, "a stale note was shown: its file changed");
            prop_assert!(!rec.superseded, "a superseded note was shown");
        }
        let has_valid_failure =
            recs.iter().any(|r| r.file == f && r.hash == current && !r.superseded && r.outcomes.iter().any(|c| *c != 0));
        prop_assert_eq!(
            !envelope.items.is_empty(),
            has_valid_failure,
            "shown items must match the model for {} (items: {})",
            path,
            envelope.items.len()
        );
    }

    // Export, import into a fresh checkout, import again.
    let bin = env!("CARGO_BIN_EXE_vexen");
    let exported = Command::new(bin).args(["export", "--format", "jsonl", "--repo"]).arg(&root).output().unwrap();
    prop_assert!(exported.status.success());
    let other_tmp = tempfile::tempdir().unwrap();
    let other = fs::canonicalize(other_tmp.path()).unwrap();
    git(&other, &["init", "-q"]);
    Store::initialize(&other).unwrap();
    let file = other.join("export.jsonl");
    fs::write(&file, &exported.stdout).unwrap();
    let import = || Command::new(bin).arg("import").arg(&file).arg("--repo").arg(&other).output().unwrap();
    prop_assert!(import().status.success(), "import failed");
    let count = Store::open_read_only(&other).unwrap().events().unwrap().len();
    prop_assert_eq!(count, Store::open_read_only(&root).unwrap().events().unwrap().len());
    prop_assert!(import().status.success(), "second import failed");
    prop_assert_eq!(Store::open_read_only(&other).unwrap().events().unwrap().len(), count, "importing twice changed the store");
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, max_shrink_iters: 200, .. ProptestConfig::default() })]

    #[test]
    fn lifecycles_keep_the_promises(ops in prop::collection::vec(op(), 1..14)) {
        run_case(&ops)?;
    }
}

/// A minimized shape worth pinning: edit away from the recorded content and back
/// again. The note is valid again exactly when the bytes match, not before.
#[test]
fn a_note_comes_back_only_when_the_file_content_does() {
    run_case(&[
        Op::Record { file: 0, text: 0 },
        Op::Outcome { rec: 0, failing: true },
        Op::Edit { file: 0, variant: 1 },
        Op::Edit { file: 0, variant: 0 },
    ])
    .unwrap();
}
