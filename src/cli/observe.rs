use super::{CliResult, RepoArg, path_str};
use crate::mcp::OutcomeInput;
use crate::store::NewOutcome;
use sha2::{Digest, Sha256};
use std::process::Command;

/// Run a command the user typed, then attach its exit code and an output
/// fingerprint to a record. The output itself is shown to the user and is
/// never stored; only its SHA-256 is.
pub fn run(repo: &RepoArg, id: String, test_id: String, key: Option<String>, command: Vec<String>) -> CliResult {
    let root = repo.root()?;
    let (program, args) = command.split_first().ok_or("no command given after `--`")?;
    let output = Command::new(program)
        .args(args)
        .current_dir(&root)
        .output()
        .map_err(|e| format!("cannot run `{program}`: {e}"))?;
    use std::io::Write;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    let mut hasher = Sha256::new();
    hasher.update(&output.stdout);
    hasher.update(&output.stderr);
    let output_sha256: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    // A signal-terminated process has no exit code; record it as -1.
    let exit_code = output.status.code().unwrap_or(-1);
    let idempotency_key = key.unwrap_or_else(|| {
        let mut h = Sha256::new();
        h.update(test_id.as_bytes());
        h.update(exit_code.to_le_bytes());
        h.update(output_sha256.as_bytes());
        h.finalize().iter().take(16).map(|b| format!("{b:02x}")).collect()
    });
    let envelope = crate::mcp::outcome(OutcomeInput {
        repo_root: path_str(&root),
        event_id: id,
        outcome: Some(NewOutcome {
            idempotency_key,
            test_id,
            exit_code,
            output_sha256: Some(output_sha256),
        }),
        supersede: None,
    })?;
    eprintln!("vexen: command exited with {exit_code}; recorded as an observed outcome");
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}
