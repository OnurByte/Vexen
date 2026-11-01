use super::RepoArg;
use crate::store::{Store, StoreError};
use std::process::ExitCode;

pub fn run(repo: &RepoArg) -> ExitCode {
    let root = match repo.root() {
        Ok(root) => root,
        Err(error) => {
            println!("checkout: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("checkout: {}", root.display());
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
    ExitCode::SUCCESS
}
