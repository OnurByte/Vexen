//! `vexen demo`: show a lesson being saved and found, without touching your
//! project. Everything happens in a throwaway Git repository with an invented
//! failure; the calls are the same ones an agent's MCP tools make.

use super::{CliResult, display_path};
use crate::{
    mcp::{CheckInput, check},
    store::{EventKind, NewEvent, NewOutcome, Store},
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const FILE: &str = "retry.py";
const BEFORE: &str = "def charge(order):\n    for _ in range(3):\n        send_payment(order)   # retried on timeout\n";
const AFTER: &str = "def charge(order):\n    send_payment(order, idempotency_key=order.id)\n";
const LESSON: &str = "[synthetic demo] Retrying the payment request after a timeout charged the customer twice. The retry needs an idempotency key.";

#[derive(clap::Args)]
pub struct DemoArgs {
    /// Keep the demo repository instead of deleting it, and print where it is.
    #[arg(long)]
    keep: bool,
    /// Build the demo in this new or empty folder instead of a temporary one.
    #[arg(long)]
    dir: Option<PathBuf>,
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=Vexen demo", "-c", "user.email=demo@example.invalid"])
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(format!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn sha(text: &str) -> String {
    Sha256::digest(text.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

fn ask(root: &Path, what: &str) -> Result<(String, Vec<String>), String> {
    let envelope = check(CheckInput {
        repo_root: fs::canonicalize(root).map_err(|e| e.to_string())?.to_string_lossy().into_owned(),
        paths: Some(vec![FILE.into()]),
        diff_ref: None,
        query: None,
        conditions: None,
    })
    .map_err(|e| e.to_string())?;
    println!("  $ vexen check --path {FILE}    # {what}");
    println!("    status: {}", envelope.status);
    let texts: Vec<String> = envelope.items.iter().filter_map(|i| i["text"].as_str().map(str::to_owned)).collect();
    for t in &texts {
        println!("    note:   {t}");
    }
    if texts.is_empty() {
        println!("    note:   (nothing: no relevant evidence)");
    }
    Ok((envelope.status, texts))
}

pub fn run(args: DemoArgs) -> CliResult {
    let root = match &args.dir {
        Some(dir) => {
            if dir.exists() && fs::read_dir(dir).map_err(|e| e.to_string())?.next().is_some() {
                return Err(format!("{} is not empty; the demo only builds in a new or empty folder", dir.display()).into());
            }
            dir.clone()
        }
        None => {
            let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
            std::env::temp_dir().join(format!("vexen-demo-{}-{nanos}", std::process::id()))
        }
    };
    fs::create_dir_all(&root)?;
    let result = story(&root);
    if args.keep || args.dir.is_some() {
        println!("\nThe demo repository is kept at {}", display_path(&root));
    } else {
        let _ = fs::remove_dir_all(&root);
    }
    result.map_err(Into::into)
}

fn story(root: &Path) -> Result<(), String> {
    println!("Vexen demo: a synthetic example. Your own repository is not touched.");
    println!("Working in a throwaway repository: {}\n", display_path(root));
    git(root, &["init", "-q"])?;
    fs::write(root.join(FILE), BEFORE).map_err(|e| e.to_string())?;
    git(root, &["add", "-A"])?;
    git(root, &["commit", "-qm", "demo: retrying payment code"])?;
    let commit = git(root, &["rev-parse", "HEAD"])?;
    Store::initialize(root).map_err(|e| e.to_string())?;
    let store = Store::open(root).map_err(|e| e.to_string())?;

    println!("1. A fix fails a test. The lesson is saved with its file and its test result.");
    println!("   (This failure is invented for the demo; a real one comes from your agent's work.)");
    let id = store
        .append(NewEvent {
            idempotency_key: "demo-lesson".into(),
            kind: EventKind::Attempt,
            summary: LESSON.into(),
            anchor_path: FILE.into(),
            anchor_sha256: sha(BEFORE),
            git_commit: Some(commit),
            conditions: vec![],
        })
        .map_err(|e| e.to_string())?;
    store
        .outcome(
            id,
            NewOutcome {
                idempotency_key: "demo-run".into(),
                test_id: "demo::retry_does_not_double_charge".into(),
                exit_code: 1,
                output_sha256: None,
            },
        )
        .map_err(|e| e.to_string())?;
    drop(store);
    println!("   saved: {FILE} + failing test `demo::retry_does_not_double_charge`\n");

    println!("2. A later session checks the file before editing it.");
    let (status, notes) = ask(root, "same call the MCP tool makes")?;
    if status != "observed_failure" || notes.is_empty() {
        return Err(format!("the demo expected the saved lesson back but got `{status}`; this is a Vexen bug, please report it"));
    }

    println!("\n3. The file is fixed and changes. The old note no longer describes this code, so it stays quiet.");
    fs::write(root.join(FILE), AFTER).map_err(|e| e.to_string())?;
    let (status, notes) = ask(root, "after the file changed")?;
    if status != "no_relevant_evidence" || !notes.is_empty() {
        return Err(format!("the demo expected silence after the edit but got `{status}`; this is a Vexen bug, please report it"));
    }

    println!("\nThat is the whole idea: failures are saved with their evidence, shown while they still apply, and dropped when the code moves on.");
    println!("Next: `vexen setup` to connect your coding tools, then ask your agent to save a real lesson.");
    Ok(())
}
