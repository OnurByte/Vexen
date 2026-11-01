use super::{CliResult, RepoArg};
use crate::store::Store;

pub fn run(repo: &RepoArg) -> CliResult {
    let root = repo.root()?;
    Store::initialize(&root)?;
    println!("initialized {}", root.join(".vexen").display());
    Ok(())
}
