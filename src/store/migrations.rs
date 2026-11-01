use rusqlite::{Connection, TransactionBehavior};

pub const SCHEMA_VERSION: u32 = 1;

pub(crate) fn migrate(conn: &mut Connection, repo_id: &str) -> rusqlite::Result<()> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if version == 0 {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;
        tx.execute_batch(
            "CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE events (
                 id TEXT PRIMARY KEY,
                 parent_id TEXT REFERENCES events(id),
                 sequence INTEGER NOT NULL UNIQUE,
                 idempotency_key TEXT NOT NULL UNIQUE,
                 kind TEXT NOT NULL,
                 payload_json TEXT NOT NULL,
                 previous_hash TEXT NOT NULL,
                 hash TEXT NOT NULL UNIQUE
             );
             CREATE INDEX events_parent ON events(parent_id, sequence);
             CREATE TABLE records (
                 id TEXT PRIMARY KEY REFERENCES events(id),
                 kind TEXT NOT NULL,
                 summary TEXT NOT NULL,
                 anchor_path TEXT NOT NULL,
                 anchor_sha256 TEXT NOT NULL,
                 git_commit TEXT,
                 conditions_json TEXT NOT NULL,
                 status TEXT NOT NULL,
                 outcomes_json TEXT NOT NULL DEFAULT '[]'
             );
             CREATE VIRTUAL TABLE records_fts USING fts5(
                 summary, anchor_path, conditions, content='records', content_rowid='rowid'
             );
             CREATE TRIGGER records_ai AFTER INSERT ON records BEGIN
                 INSERT INTO records_fts(rowid, summary, anchor_path, conditions)
                 VALUES (new.rowid, new.summary, new.anchor_path, new.conditions_json);
             END;
             CREATE TRIGGER records_au AFTER UPDATE ON records BEGIN
                 INSERT INTO records_fts(records_fts, rowid, summary, anchor_path, conditions)
                 VALUES ('delete', old.rowid, old.summary, old.anchor_path, old.conditions_json);
                 INSERT INTO records_fts(rowid, summary, anchor_path, conditions)
                 VALUES (new.rowid, new.summary, new.anchor_path, new.conditions_json);
             END;
             CREATE TRIGGER records_ad AFTER DELETE ON records BEGIN
                 INSERT INTO records_fts(records_fts, rowid, summary, anchor_path, conditions)
                 VALUES ('delete', old.rowid, old.summary, old.anchor_path, old.conditions_json);
             END;
             CREATE TRIGGER events_no_update BEFORE UPDATE ON events BEGIN
                 SELECT RAISE(ABORT, 'events are append-only');
             END;
             CREATE TRIGGER events_no_delete BEFORE DELETE ON events BEGIN
                 SELECT RAISE(ABORT, 'events are append-only');
             END;
             PRAGMA user_version=1;",
        )?;
        tx.execute(
            "INSERT INTO metadata(key,value) VALUES ('repo_id',?1)",
            [repo_id],
        )?;
        tx.commit()?;
    } else {
        let stored: String =
            conn.query_row("SELECT value FROM metadata WHERE key='repo_id'", [], |r| {
                r.get(0)
            })?;
        if stored != repo_id {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }
    Ok(())
}
