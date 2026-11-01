use super::{CliResult, Format, RepoArg};
use crate::store::{EventPayload, Store};

pub const EXPORT_VERSION: u32 = 1;

pub fn run(repo: &RepoArg, format: Format) -> CliResult {
    let store = Store::open_read_only(&repo.root()?)?;
    let events = store.events()?;
    match format {
        Format::Jsonl => {
            let header = serde_json::json!({
                "vexen_export": EXPORT_VERSION,
                "repo_id": store.repo_id(),
                "event_count": events.len(),
            });
            println!("{header}");
            for event in &events {
                println!("{}", serde_json::to_string(event)?);
            }
        }
        Format::Markdown => {
            println!("# Vexen evidence export\n");
            for event in &events {
                match &event.payload {
                    EventPayload::Record(data) => println!(
                        "- **{:?}** `{}` — {}",
                        data.kind, data.anchor_path, data.summary
                    ),
                    EventPayload::Outcome(data) => println!(
                        "  - outcome `{}` exit code {}",
                        data.test_id, data.exit_code
                    ),
                    EventPayload::Supersession { reason, .. } => {
                        println!("  - superseded: {reason}")
                    }
                }
            }
        }
    }
    Ok(())
}
