use super::{CliResult, RepoArg, path_str};
use crate::mcp::{CheckInput, ExplainInput};

pub fn run(
    repo: &RepoArg,
    diff: Option<String>,
    paths: Vec<String>,
    query: Option<String>,
) -> CliResult {
    let envelope = crate::mcp::check(CheckInput {
        repo_root: path_str(&repo.root()?),
        paths: (!paths.is_empty()).then_some(paths),
        diff_ref: diff,
        query,
        conditions: None,
    })?;
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}

pub fn explain(repo: &RepoArg, id: String) -> CliResult {
    let envelope = crate::mcp::explain(ExplainInput {
        repo_root: path_str(&repo.root()?),
        event_id: id,
    })?;
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}
