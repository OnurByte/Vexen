//! Per-checkout, append-only evidence store.
mod events;
mod migrations;
mod query;

pub use events::{
    Event, EventChain, EventId, EventKind, EventPayload, MemoryRecord, NewEvent, NewOutcome,
    RecordStatus,
};
pub use migrations::SCHEMA_VERSION;
pub use query::StoreQuery;

use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::Duration,
};
use thiserror::Error;

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_QUERY_LIMIT: usize = 1000;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("invalid checkout: {0}")]
    InvalidCheckout(String),
    #[error("store is not initialized at {0}")]
    NotInitialized(PathBuf),
    #[error("invalid event: {0}")]
    InvalidEvent(String),
    #[error("idempotency key was already used with a different payload")]
    IdempotencyConflict,
    #[error("event not found: {0}")]
    NotFound(EventId),
    #[error("store integrity check failed: {0}")]
    Integrity(String),
    #[error("unsupported or inconsistent store schema")]
    Schema,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// One open connection to a database bound to one canonical Git checkout.
pub struct Store {
    root: PathBuf,
    repo_id: String,
    conn: Mutex<Connection>,
}

#[derive(Serialize, Deserialize)]
struct RecordPayload {
    event: NewEvent,
}

impl Store {
    /// Explicitly initialize `.vexen` in an existing non-bare Git checkout.
    /// Only the three mutable SQLite files are ignored locally.
    pub fn initialize(repo_root: &Path) -> Result<()> {
        let root = canonical_checkout(repo_root)?;
        let dir = root.join(".vexen");
        reject_symlink(&dir)?;
        fs::create_dir_all(&dir)?;
        set_dir_mode(&dir)?;
        let ignore = dir.join(".gitignore");
        reject_symlink(&ignore)?;
        let mut contents = if ignore.exists() {
            fs::read_to_string(&ignore)?
        } else {
            String::new()
        };
        for entry in ["state.sqlite", "state.sqlite-wal", "state.sqlite-shm"] {
            if !contents.lines().any(|line| line.trim() == entry) {
                if !contents.is_empty() && !contents.ends_with('\n') {
                    contents.push('\n');
                }
                contents.push_str(entry);
                contents.push('\n');
            }
        }
        fs::write(ignore, contents)?;
        let db = dir.join("state.sqlite");
        reject_symlink(&db)?;
        let mut conn = Connection::open_with_flags(
            &db,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        configure(&conn)?;
        let repo_id = repo_id(&root);
        migrations::migrate(&mut conn, &repo_id).map_err(map_migration_error)?;
        set_file_mode(&db)?;
        Ok(())
    }

    /// Open an initialized store. Does not create directories or databases.
    pub fn open(repo_root: &Path) -> Result<Self> {
        let root = canonical_checkout(repo_root)?;
        let dir = root.join(".vexen");
        let db = dir.join("state.sqlite");
        if !db.is_file() {
            return Err(StoreError::NotInitialized(db));
        }
        reject_symlink(&dir)?;
        reject_symlink(&db)?;
        let rid = repo_id(&root);
        let mut conn = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        configure(&conn)?;
        let version: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version == 0 {
            return Err(StoreError::Schema);
        }
        migrations::migrate(&mut conn, &rid).map_err(map_migration_error)?;
        set_file_mode(&db)?;
        let store = Self {
            root,
            repo_id: rid,
            conn: Mutex::new(conn),
        };
        if !store.verify_integrity()? {
            return Err(StoreError::Integrity("event hash chain mismatch".into()));
        }
        store.rebuild_projection()?;
        Ok(store)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn repo_id(&self) -> &str {
        &self.repo_id
    }
    pub fn schema_version(&self) -> Result<u32> {
        Ok(self
            .conn
            .lock()
            .map_err(|_| StoreError::Integrity("connection mutex poisoned".into()))?
            .pragma_query_value(None, "user_version", |r| r.get(0))?)
    }

    /// Append an attempt or accepted decision. Repeated identical keys return
    /// the existing ID; reusing a key for different data is an error.
    pub fn append(&self, event: NewEvent) -> Result<EventId> {
        validate_event(&event)?;
        let payload = EventPayload::Record(event.clone());
        let key = event.idempotency_key.clone();
        let mut conn = self.lock()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(existing) = existing_key(&tx, &key)? {
            let (id, existing_payload): (String, String) = existing;
            let parsed: EventPayload = serde_json::from_str(&existing_payload)?;
            if parsed != payload {
                return Err(StoreError::IdempotencyConflict);
            }
            tx.commit()?;
            return Ok(EventId(id));
        }
        let id = EventId(stable_id(&self.repo_id, &key));
        let sequence = next_sequence(&tx)?;
        let (_previous, prev_hash) = previous_hash(&tx)?;
        insert_event(
            &tx,
            &self.repo_id,
            &id,
            None,
            sequence,
            &key,
            &payload,
            &prev_hash,
        )?;
        let RecordPayload { event: data } = RecordPayload { event };
        tx.execute("INSERT INTO records(id,kind,summary,anchor_path,anchor_sha256,git_commit,conditions_json,status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", params![id.0, kind_name(data.kind), data.summary, data.anchor_path, data.anchor_sha256, data.git_commit, serde_json::to_string(&data.conditions)?, initial_status(data.kind)])?;
        tx.commit()?;
        Ok(id)
    }

    /// Append observed test metadata linked to its record. Does not mark the
    /// record supported or current; later validity checks decide applicability.
    pub fn outcome(&self, id: EventId, data: NewOutcome) -> Result<EventId> {
        validate_outcome(&data)?;
        let mut conn = self.lock()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let parent = root_for(&tx, &id)?.ok_or_else(|| StoreError::NotFound(id.clone()))?;
        let key = format!("outcome:{}:{}", parent, data.idempotency_key);
        let payload = EventPayload::Outcome(data.clone());
        if let Some((existing_id, json)) = existing_key(&tx, &key)? {
            if serde_json::from_str::<EventPayload>(&json)? != payload {
                return Err(StoreError::IdempotencyConflict);
            }
            tx.commit()?;
            return Ok(EventId(existing_id));
        }
        let event_id = EventId(stable_id(&self.repo_id, &key));
        let sequence = next_sequence(&tx)?;
        let (_previous, prev_hash) = previous_hash(&tx)?;
        insert_event(
            &tx,
            &self.repo_id,
            &event_id,
            Some(&parent),
            sequence,
            &key,
            &payload,
            &prev_hash,
        )?;
        let mut outcomes: Vec<NewOutcome> = tx.query_row(
            "SELECT outcomes_json FROM records WHERE id=?1",
            [&parent],
            |r| {
                let json: String = r.get(0)?;
                serde_json::from_str(&json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            },
        )?;
        outcomes.push(data);
        tx.execute(
            "UPDATE records SET outcomes_json=?2,status='observed' WHERE id=?1",
            params![parent, serde_json::to_string(&outcomes)?],
        )?;
        tx.commit()?;
        Ok(event_id)
    }

    pub fn get(&self, id: EventId) -> Result<EventChain> {
        let conn = self.lock()?;
        let root = root_for(&conn, &id)?.ok_or_else(|| StoreError::NotFound(id.clone()))?;
        let mut stmt = conn.prepare("SELECT id,parent_id,sequence,kind,payload_json,previous_hash,hash FROM events WHERE id=?1 OR parent_id=?1 ORDER BY sequence")?;
        let rows = stmt.query_map([root], decode_event)?;
        Ok(EventChain {
            events: rows.collect::<std::result::Result<Vec<_>, _>>()?,
        })
    }

    /// Read every immutable event in append order for export/import tooling.
    pub fn events(&self) -> Result<Vec<Event>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare("SELECT id,parent_id,sequence,kind,payload_json,previous_hash,hash FROM events ORDER BY sequence")?;
        let rows = stmt.query_map([], decode_event)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn search(&self, query: StoreQuery) -> Result<Vec<MemoryRecord>> {
        let conn = self.lock()?;
        let limit = query.limit.min(MAX_QUERY_LIMIT);
        if limit == 0 {
            return Ok(Vec::new());
        }
        let sql = if query.text.trim().is_empty() {
            "SELECT id,kind,summary,anchor_path,anchor_sha256,git_commit,conditions_json,status,outcomes_json FROM records ORDER BY rowid DESC LIMIT ?1"
        } else {
            "SELECT r.id,r.kind,r.summary,r.anchor_path,r.anchor_sha256,r.git_commit,r.conditions_json,r.status,r.outcomes_json FROM records_fts f JOIN records r ON r.rowid=f.rowid WHERE records_fts MATCH ?1 ORDER BY bm25(records_fts) LIMIT ?2"
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = if query.text.trim().is_empty() {
            stmt.query_map([limit as i64], decode_record)?
        } else {
            stmt.query_map(params![query.text, limit as i64], decode_record)?
        };
        let mut records = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        for record in &mut records {
            record.repo_id = self.repo_id.clone();
        }
        Ok(records)
    }

    /// SQLite's structural check plus full event-chain hash validation.
    pub fn verify_integrity(&self) -> Result<bool> {
        let conn = self.lock()?;
        let check: String = conn.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        if check != "ok" {
            return Ok(false);
        }
        let mut stmt = conn.prepare("SELECT id,parent_id,sequence,idempotency_key,payload_json,previous_hash,hash FROM events ORDER BY sequence")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        })?;
        let mut previous = String::new();
        for (expected_sequence, row) in (1i64..).zip(rows) {
            let (id, parent, seq, key, json, prev, hash) = row?;
            if seq != expected_sequence || prev != previous || id != stable_id(&self.repo_id, &key)
            {
                return Ok(false);
            }
            if let Some(ref p) = parent
                && (p == &id
                    || !conn.query_row(
                        "SELECT EXISTS(SELECT 1 FROM events WHERE id=?1 AND parent_id IS NULL)",
                        [p],
                        |r| r.get::<_, bool>(0),
                    )?)
            {
                return Ok(false);
            }
            let payload: EventPayload = serde_json::from_str(&json)?;
            if event_hash(&self.repo_id, seq as u64, prev.as_str(), &payload)? != hash {
                return Ok(false);
            }
            previous = hash;
        }
        Ok(true)
    }

    fn rebuild_projection(&self) -> Result<()> {
        let mut conn = self.lock()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("DELETE FROM records", [])?;
        let rows = {
            let mut stmt = tx.prepare("SELECT id,parent_id,sequence,kind,payload_json,previous_hash,hash FROM events ORDER BY sequence")?;
            stmt.query_map([], decode_event)?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        for event in rows {
            match event.payload {
                EventPayload::Record(data) => {
                    tx.execute("INSERT INTO records(id,kind,summary,anchor_path,anchor_sha256,git_commit,conditions_json,status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", params![event.id.0, kind_name(data.kind), data.summary, data.anchor_path, data.anchor_sha256, data.git_commit, serde_json::to_string(&data.conditions)?, initial_status(data.kind)])?;
                }
                EventPayload::Outcome(outcome) => {
                    if let Some(parent) = event.parent_id {
                        let mut outcomes: Vec<NewOutcome> = tx.query_row(
                            "SELECT outcomes_json FROM records WHERE id=?1",
                            [&parent.0],
                            |r| {
                                let json: String = r.get(0)?;
                                serde_json::from_str(&json).map_err(|e| {
                                    rusqlite::Error::FromSqlConversionFailure(
                                        0,
                                        rusqlite::types::Type::Text,
                                        Box::new(e),
                                    )
                                })
                            },
                        )?;
                        outcomes.push(outcome);
                        tx.execute(
                            "UPDATE records SET outcomes_json=?2,status='observed' WHERE id=?1",
                            params![parent.0, serde_json::to_string(&outcomes)?],
                        )?;
                    }
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| StoreError::Integrity("connection mutex poisoned".into()))
    }
}

fn canonical_checkout(path: &Path) -> Result<PathBuf> {
    let canonical =
        fs::canonicalize(path).map_err(|e| StoreError::InvalidCheckout(e.to_string()))?;
    let out = Command::new("git")
        .arg("-C")
        .arg(&canonical)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| StoreError::InvalidCheckout(e.to_string()))?;
    if !out.status.success() {
        return Err(StoreError::InvalidCheckout("not a Git checkout".into()));
    }
    let top = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    let top = fs::canonicalize(top)?;
    if top != canonical {
        return Err(StoreError::InvalidCheckout(
            "path must be the canonical checkout root".into(),
        ));
    }
    let bare = Command::new("git")
        .arg("-C")
        .arg(&canonical)
        .args(["rev-parse", "--is-bare-repository"])
        .output()?;
    if !bare.status.success() || String::from_utf8_lossy(&bare.stdout).trim() != "false" {
        return Err(StoreError::InvalidCheckout(
            "bare repositories are not supported".into(),
        ));
    }
    Ok(canonical)
}

fn repo_id(root: &Path) -> String {
    hex(&Sha256::digest(
        root.as_os_str().to_string_lossy().as_bytes(),
    ))
}
fn stable_id(repo: &str, key: &str) -> String {
    hex(&Sha256::digest(format!("{repo}\n{key}").as_bytes()))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn kind_name(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Attempt => "attempt",
        EventKind::AcceptedDecision => "accepted_decision",
        EventKind::Outcome => "outcome",
    }
}
fn initial_status(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Attempt => "candidate",
        EventKind::AcceptedDecision => "accepted_decision",
        EventKind::Outcome => "candidate",
    }
}
fn event_hash(repo: &str, sequence: u64, previous: &str, payload: &EventPayload) -> Result<String> {
    let bytes = serde_json::to_vec(payload)?;
    let mut digest = Sha256::new();
    digest.update(repo.as_bytes());
    digest.update(sequence.to_be_bytes());
    digest.update(previous.as_bytes());
    digest.update(bytes);
    Ok(hex(&digest.finalize()))
}
fn validate_digest(value: &str, field: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(StoreError::InvalidEvent(format!(
            "{field} must be a 64-character SHA-256 hex digest"
        )));
    }
    Ok(())
}
fn validate_event(e: &NewEvent) -> Result<()> {
    if e.kind == EventKind::Outcome {
        return Err(StoreError::InvalidEvent(
            "outcomes must be appended with Store::outcome".into(),
        ));
    }
    if e.idempotency_key.is_empty()
        || e.idempotency_key.len() > 256
        || e.summary.trim().is_empty()
        || e.summary.len() > 4096
    {
        return Err(StoreError::InvalidEvent(
            "idempotency key and summary must be non-empty and bounded".into(),
        ));
    }
    let path = Path::new(&e.anchor_path);
    if e.anchor_path.is_empty()
        || path.is_absolute()
        || path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir
                    | std::path::Component::CurDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(StoreError::InvalidEvent(
            "anchor path must be a safe repo-relative path".into(),
        ));
    }
    validate_digest(&e.anchor_sha256, "anchor_sha256")?;
    if let Some(commit) = &e.git_commit
        && (!(commit.len() == 40 || commit.len() == 64)
            || !commit.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(StoreError::InvalidEvent(
            "git_commit must be a full object ID".into(),
        ));
    }
    if e.conditions.len() > 64 || e.conditions.iter().any(|c| c.is_empty() || c.len() > 512) {
        return Err(StoreError::InvalidEvent(
            "conditions must be bounded non-empty strings".into(),
        ));
    }
    Ok(())
}
fn validate_outcome(e: &NewOutcome) -> Result<()> {
    if e.idempotency_key.is_empty()
        || e.idempotency_key.len() > 256
        || e.test_id.is_empty()
        || e.test_id.len() > 512
        || e.test_id.chars().any(char::is_control)
    {
        return Err(StoreError::InvalidEvent(
            "outcome key and test_id must be allowlisted bounded metadata".into(),
        ));
    }
    if let Some(hash) = &e.output_sha256 {
        validate_digest(hash, "output_sha256")?;
    }
    Ok(())
}
fn configure(conn: &Connection) -> rusqlite::Result<()> {
    conn.busy_timeout(BUSY_TIMEOUT)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    Ok(())
}
fn map_migration_error(e: rusqlite::Error) -> StoreError {
    match e {
        rusqlite::Error::InvalidQuery | rusqlite::Error::QueryReturnedNoRows => StoreError::Schema,
        other => StoreError::Sqlite(other),
    }
}
fn reject_symlink(path: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.file_type().is_symlink()
    {
        return Err(StoreError::InvalidCheckout(format!(
            "refusing symlink at {}",
            path.display()
        )));
    }
    Ok(())
}
#[cfg(unix)]
fn set_dir_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
#[cfg(not(unix))]
fn set_dir_mode(_path: &Path) -> Result<()> {
    Ok(())
}
#[cfg(unix)]
fn set_file_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
#[cfg(not(unix))]
fn set_file_mode(_path: &Path) -> Result<()> {
    Ok(())
}

fn existing_key(tx: &Transaction<'_>, key: &str) -> Result<Option<(String, String)>> {
    Ok(tx
        .query_row(
            "SELECT id,payload_json FROM events WHERE idempotency_key=?1",
            [key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?)
}
fn next_sequence(tx: &Transaction<'_>) -> Result<u64> {
    Ok(
        tx.query_row("SELECT COALESCE(MAX(sequence),0)+1 FROM events", [], |r| {
            r.get::<_, i64>(0)
        })? as u64,
    )
}
fn previous_hash(tx: &Transaction<'_>) -> Result<(String, String)> {
    Ok(tx
        .query_row(
            "SELECT id,hash FROM events ORDER BY sequence DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or_default())
}
#[allow(clippy::too_many_arguments)]
fn insert_event(
    tx: &Transaction<'_>,
    repo: &str,
    id: &EventId,
    parent: Option<&str>,
    sequence: u64,
    key: &str,
    payload: &EventPayload,
    prev_hash: &str,
) -> Result<()> {
    let json = serde_json::to_string(payload)?;
    let hash = event_hash(repo, sequence, prev_hash, payload)?;
    let kind = match payload {
        EventPayload::Record(e) => kind_name(e.kind),
        EventPayload::Outcome(_) => "outcome",
    };
    tx.execute("INSERT INTO events(id,parent_id,sequence,idempotency_key,kind,payload_json,previous_hash,hash) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![id.0,parent,sequence as i64,key,kind,json,prev_hash,hash])?;
    Ok(())
}
fn root_for(conn: &Connection, id: &EventId) -> Result<Option<String>> {
    let root = conn
        .query_row(
            "SELECT COALESCE(parent_id,id) FROM events WHERE id=?1",
            [&id.0],
            |r| r.get(0),
        )
        .optional()?;
    Ok(root)
}
fn decode_event(r: &rusqlite::Row<'_>) -> rusqlite::Result<Event> {
    let id: String = r.get(0)?;
    let parent: Option<String> = r.get(1)?;
    let seq: i64 = r.get(2)?;
    let kind: String = r.get(3)?;
    let json: String = r.get(4)?;
    let previous_hash: String = r.get(5)?;
    let hash: String = r.get(6)?;
    let payload: EventPayload = serde_json::from_str(&json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let event_kind = if kind == "outcome" {
        EventKind::Outcome
    } else if kind == "accepted_decision" {
        EventKind::AcceptedDecision
    } else {
        EventKind::Attempt
    };
    Ok(Event {
        id: EventId(id),
        parent_id: parent.map(EventId),
        sequence: seq as u64,
        kind: event_kind,
        payload,
        previous_hash,
        hash,
    })
}
fn decode_record(r: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryRecord> {
    let kind: String = r.get(1)?;
    let conditions: String = r.get(6)?;
    let outcomes: String = r.get(8)?;
    let event_kind = if kind == "accepted_decision" {
        EventKind::AcceptedDecision
    } else {
        EventKind::Attempt
    };
    let conditions = serde_json::from_str(&conditions).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let outcomes = serde_json::from_str(&outcomes).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let status: String = r.get(7)?;
    let status = match status.as_str() {
        "candidate" => RecordStatus::Candidate,
        "accepted_decision" => RecordStatus::AcceptedDecision,
        "observed" => RecordStatus::Observed,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(MemoryRecord {
        repo_id: String::new(),
        id: EventId(r.get(0)?),
        kind: event_kind,
        summary: r.get(2)?,
        anchor_path: r.get(3)?,
        anchor_sha256: r.get(4)?,
        git_commit: r.get(5)?,
        conditions,
        status,
        outcomes,
    })
}
