use super::{CliResult, RepoArg, export::EXPORT_VERSION};
use crate::{
    privacy::contains_secret,
    store::{Event, EventId, EventPayload, Store},
};
use std::{collections::HashMap, path::Path};

/// Parse and screen the whole file first, then replay; a bad line writes nothing.
pub fn run(repo: &RepoArg, file: &Path) -> CliResult {
    let text = std::fs::read_to_string(file)?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header: serde_json::Value = serde_json::from_str(lines.next().ok_or("empty import file")?)
        .map_err(|e| format!("invalid header: {e}"))?;
    if header["vexen_export"] != EXPORT_VERSION {
        return Err("unsupported export version".into());
    }
    let mut events = Vec::new();
    for (number, line) in lines.enumerate() {
        let event: Event =
            serde_json::from_str(line).map_err(|e| format!("event {}: {e}", number + 1))?;
        if let EventPayload::Record(data) = &event.payload
            && (contains_secret(&data.summary)
                || data.conditions.iter().any(|c| contains_secret(c)))
        {
            return Err(format!("event {}: secret-like text refused", number + 1).into());
        }
        events.push(event);
    }
    let store = Store::open(&repo.root()?)?;
    let mut ids: HashMap<EventId, EventId> = HashMap::new();
    for event in events {
        let parent = |ids: &HashMap<EventId, EventId>| {
            event
                .parent_id
                .as_ref()
                .and_then(|p| ids.get(p))
                .cloned()
                .ok_or("event references a parent absent from the file")
        };
        let new_id = match event.payload.clone() {
            EventPayload::Record(data) => store.append(data)?,
            EventPayload::Outcome(data) => store.outcome(parent(&ids)?, data)?,
            EventPayload::Supersession {
                idempotency_key,
                reason,
            } => store.supersede(parent(&ids)?, idempotency_key, reason)?,
        };
        ids.insert(event.id, new_id);
    }
    println!("imported {} events", ids.len());
    Ok(())
}
