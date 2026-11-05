//! Read-only view of one checkout's memory, built from Vexen's own library.
use std::path::{Path, PathBuf};
use vexen::{
    mcp::{CheckInput, check},
    store::{EventKind, MemoryRecord, Store, StoreQuery},
};

const MAX_NOTES: usize = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Filter {
    All,
    Applies,
    Quiet,
}

#[derive(Clone)]
pub struct Row {
    pub record: MemoryRecord,
    /// The note would be shown for its file right now (the same check an agent runs).
    pub applies: bool,
}

impl Row {
    pub fn verdict(&self) -> &'static str {
        if self.applies {
            "Still applies to the current file"
        } else {
            "Quiet: nothing to show now"
        }
    }

    pub fn explanation(&self) -> &'static str {
        if self.applies {
            "The file still matches what was saved, so an agent that checks this file will see this note."
        } else {
            "The file changed since this was saved, or the note was replaced, so an agent will not be shown it. No relevant evidence is a useful result."
        }
    }
}

#[derive(Clone)]
pub struct Snapshot {
    pub root: PathBuf,
    pub intact: bool,
    pub rows: Vec<Row>,
}

impl Snapshot {
    pub fn load(start: &Path) -> Result<Self, String> {
        let root = find_root(start).ok_or_else(|| {
            format!(
                "{} is not inside a Vexen repository (no .vexen folder found).",
                start.display()
            )
        })?;
        let store = Store::open_read_only(&root).map_err(|e| e.to_string())?;
        let intact = store.verify_integrity().unwrap_or(false);
        let records = store
            .search(StoreQuery {
                text: String::new(),
                limit: MAX_NOTES,
            })
            .map_err(|e| e.to_string())?;
        let rows = records
            .into_iter()
            .filter(|r| matches!(r.kind, EventKind::Attempt | EventKind::AcceptedDecision))
            .map(|record| {
                let applies = applies_now(&root, &record);
                Row { record, applies }
            })
            .collect();
        Ok(Self { root, intact, rows })
    }

    pub fn visible(&self, filter: Filter) -> Vec<usize> {
        (0..self.rows.len())
            .filter(|&i| match filter {
                Filter::All => true,
                Filter::Applies => self.rows[i].applies,
                Filter::Quiet => !self.rows[i].applies,
            })
            .collect()
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        let applies = self.rows.iter().filter(|r| r.applies).count();
        (self.rows.len(), applies, self.rows.len() - applies)
    }
}

fn find_root(start: &Path) -> Option<PathBuf> {
    let start = std::fs::canonicalize(start).ok()?;
    start
        .ancestors()
        .find(|p| p.join(".vexen").is_dir())
        .map(plain)
}

fn applies_now(root: &Path, record: &MemoryRecord) -> bool {
    let input = CheckInput {
        repo_root: root.to_string_lossy().into_owned(),
        paths: Some(vec![record.anchor_path.clone()]),
        diff_ref: None,
        query: None,
        // A note with conditions is shown only under exactly those conditions.
        conditions: Some(record.conditions.clone()),
    };
    check(input)
        .map(|e| {
            e.items
                .iter()
                .any(|i| i["id"].as_str() == Some(record.id.as_str()))
        })
        .unwrap_or(false)
}

pub fn kind_label(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Attempt => "Attempt (a fix that was tried)",
        EventKind::AcceptedDecision => "Accepted decision",
        EventKind::Outcome => "Test result",
        EventKind::Supersession => "Replacement",
    }
}

/// Windows canonical paths start with a verbatim prefix; show the ordinary form.
fn plain(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(&text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vexen::store::{EventId, RecordStatus};

    fn row(applies: bool) -> Row {
        Row {
            record: MemoryRecord {
                repo_id: String::new(),
                id: EventId::from("a".to_string()),
                kind: EventKind::Attempt,
                summary: "s".into(),
                anchor_path: "src/a.rs".into(),
                anchor_sha256: "0".repeat(64),
                git_commit: None,
                conditions: vec![],
                status: RecordStatus::Observed,
                outcomes: vec![],
            },
            applies,
        }
    }

    #[test]
    fn a_folder_without_vexen_is_explained_not_a_crash() {
        let dir = std::env::temp_dir().join(format!("vexen-desktop-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let error = Snapshot::load(&dir).err().expect("no .vexen folder");
        assert!(error.contains("no .vexen folder found"), "{error}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn filters_split_notes_by_whether_they_apply_now() {
        let snapshot = Snapshot {
            root: PathBuf::new(),
            intact: true,
            rows: vec![row(true), row(false), row(true)],
        };
        assert_eq!(snapshot.counts(), (3, 2, 1));
        assert_eq!(snapshot.visible(Filter::Applies), vec![0, 2]);
        assert_eq!(snapshot.visible(Filter::Quiet), vec![1]);
        assert_eq!(snapshot.visible(Filter::All), vec![0, 1, 2]);
    }
}
