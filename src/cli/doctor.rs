use super::RepoArg;
use crate::store::{Store, StoreError};
use crate::{install::Launch, verify::probe_mcp};
use std::{process::ExitCode, time::Duration};

pub fn run(repo: &RepoArg, mcp: bool) -> ExitCode {
    let root = match repo.root() {
        Ok(root) => root,
        Err(error) => {
            println!("checkout: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("checkout: {}", super::display_path(&root));
    let store = match Store::open_read_only(&root) {
        Ok(store) => store,
        Err(StoreError::NotInitialized(_)) => {
            println!("store: not initialized (run `vexen init`)");
            return ExitCode::FAILURE;
        }
        Err(error) => {
            println!("store: unhealthy: {error}");
            println!("integrity: FAILED");
            return ExitCode::FAILURE;
        }
    };
    println!("repo_id: {}", store.repo_id());
    println!("schema_version: {}", store.schema_version().unwrap_or(0));
    println!("integrity: ok");
    match store.events() {
        Ok(events) => println!("events: {}", events.len()),
        Err(error) => {
            println!("events: unreadable: {error}");
            return ExitCode::FAILURE;
        }
    }
    if mcp {
        let exe = match std::env::current_exe() {
            Ok(exe) => exe,
            Err(e) => {
                println!("mcp: cannot find this binary: {e}");
                return ExitCode::FAILURE;
            }
        };
        match probe_mcp(&Launch::binary(&exe.to_string_lossy()), Some(&root), Duration::from_secs(20)) {
            Ok(probe) => println!(
                "mcp: ok ({} tools; a vexen_check call answered `{}`). This proves the server works, not that your agent is connected to it.",
                probe.tools.len(),
                probe.check_status.unwrap_or_default()
            ),
            Err(e) => {
                println!("mcp: FAILED: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
