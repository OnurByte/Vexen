use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use vexen::store::{EventKind, EventPayload, NewEvent, NewOutcome, Store, StoreQuery};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);
impl Repo {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "vexen-store-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        assert!(
            Command::new("git")
                .arg("init")
                .arg("-q")
                .arg(&root)
                .status()
                .unwrap()
                .success()
        );
        Self(root)
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn event(key: &str, description: &str) -> NewEvent {
    NewEvent {
        idempotency_key: key.into(),
        kind: EventKind::Attempt,
        summary: description.into(),
        anchor_path: "src/lib.rs".into(),
        anchor_sha256: "a".repeat(64),
        git_commit: None,
        conditions: vec!["platform=linux".into()],
    }
}

#[test]
fn initialize_is_explicit_and_open_requires_initialized_checkout() {
    let repo = Repo::new();
    assert!(Store::open(&repo.0).is_err());
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    assert_eq!(store.schema_version().unwrap(), 1);
    assert!(repo.0.join(".vexen/.gitignore").exists());
}

#[test]
fn append_outcome_and_search_preserve_typed_append_only_chain() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    let id = store
        .append(event("first", "retry without a lock"))
        .unwrap();
    let outcome_id = store
        .outcome(
            id.clone(),
            NewOutcome {
                idempotency_key: "test-run-1".into(),
                test_id: "cargo test retry::duplicate_charge".into(),
                exit_code: 101,
                output_sha256: Some("b".repeat(64)),
            },
        )
        .unwrap();
    assert_ne!(id, outcome_id);
    let chain = store.get(id.clone()).unwrap();
    assert_eq!(chain.events.len(), 2);
    assert_eq!(chain.events[0].kind, EventKind::Attempt);
    assert_eq!(chain.events[1].parent_id.as_ref(), Some(&id));
    let EventPayload::Outcome(outcome) = &chain.events[1].payload else {
        panic!("expected outcome event")
    };
    assert_eq!(outcome.exit_code, 101);
    assert_eq!(
        store
            .search(StoreQuery {
                text: "retry".into(),
                limit: 10
            })
            .unwrap()
            .len(),
        1
    );
    let record = store.search(StoreQuery::default()).unwrap().pop().unwrap();
    assert_eq!(record.status, vexen::store::RecordStatus::Observed);
    assert_eq!(record.outcomes.len(), 1);
}

#[test]
fn duplicate_key_is_idempotent_but_different_payload_is_rejected() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    let id = store.append(event("same", "same payload")).unwrap();
    assert_eq!(store.append(event("same", "same payload")).unwrap(), id);
    assert!(store.append(event("same", "different payload")).is_err());
    assert_eq!(store.events().unwrap().len(), 1);
}

#[test]
fn concurrent_connections_serialize_writes_without_losing_events() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let mut workers = Vec::new();
    for n in 0..8 {
        let root = repo.0.clone();
        workers.push(thread::spawn(move || {
            let store = Store::open(&root).unwrap();
            store
                .append(event(&format!("key-{n}"), &format!("event {n}")))
                .unwrap()
        }));
    }
    let ids: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    let store = Store::open(&repo.0).unwrap();
    assert_eq!(
        ids.iter().collect::<std::collections::HashSet<_>>().len(),
        8
    );
    assert_eq!(store.events().unwrap().len(), 8);
}

#[test]
fn committed_events_survive_reopen_and_hash_chain_verifies() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let id = Store::open(&repo.0)
        .unwrap()
        .append(event("durable", "survive close"))
        .unwrap();
    let reopened = Store::open(&repo.0).unwrap();
    assert_eq!(reopened.get(id).unwrap().events.len(), 1);
    assert!(reopened.verify_integrity().unwrap());
}

#[test]
fn outcome_retry_is_idempotent_and_conflicting_outcome_is_preserved_as_rejection() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    let id = store.append(event("attempt", "one approach")).unwrap();
    let first = NewOutcome {
        idempotency_key: "run-1".into(),
        test_id: "cargo test x".into(),
        exit_code: 1,
        output_sha256: None,
    };
    let outcome_id = store.outcome(id.clone(), first.clone()).unwrap();
    assert_eq!(store.outcome(id.clone(), first).unwrap(), outcome_id);
    assert!(
        store
            .outcome(
                id.clone(),
                NewOutcome {
                    idempotency_key: "run-1".into(),
                    test_id: "cargo test x".into(),
                    exit_code: 0,
                    output_sha256: None
                }
            )
            .is_err()
    );
    store
        .outcome(
            id,
            NewOutcome {
                idempotency_key: "run-2".into(),
                test_id: "cargo test x".into(),
                exit_code: 0,
                output_sha256: None,
            },
        )
        .unwrap();
    let record = store.search(StoreQuery::default()).unwrap().pop().unwrap();
    assert_eq!(record.outcomes.len(), 2);
    assert_eq!(record.repo_id, store.repo_id());
}

#[test]
fn event_rows_reject_update_and_delete() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    let id = store
        .append(event("immutable", "cannot rewrite history"))
        .unwrap();
    drop(store);
    let conn = rusqlite::Connection::open(repo.0.join(".vexen/state.sqlite")).unwrap();
    assert!(
        conn.execute(
            "UPDATE events SET payload_json='{}' WHERE id=?1",
            [id.as_str()]
        )
        .is_err()
    );
    assert!(
        conn.execute("DELETE FROM events WHERE id=?1", [id.as_str()])
            .is_err()
    );
}

#[test]
fn initialized_database_cannot_be_reused_for_another_checkout() {
    let first = Repo::new();
    let second = Repo::new();
    Store::initialize(&first.0).unwrap();
    Store::initialize(&second.0).unwrap();
    fs::copy(
        first.0.join(".vexen/state.sqlite"),
        second.0.join(".vexen/state.sqlite"),
    )
    .unwrap();
    assert!(Store::open(&second.0).is_err());
}

#[test]
fn derived_search_projection_is_rebuilt_from_immutable_events_on_open() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    store
        .append(event("projection", "authoritative event text"))
        .unwrap();
    drop(store);
    let conn = rusqlite::Connection::open(repo.0.join(".vexen/state.sqlite")).unwrap();
    conn.execute("UPDATE records SET summary='tampered projection'", [])
        .unwrap();
    drop(conn);
    let reopened = Store::open(&repo.0).unwrap();
    assert_eq!(
        reopened
            .search(StoreQuery {
                text: "authoritative".into(),
                limit: 10
            })
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        reopened
            .search(StoreQuery {
                text: "tampered".into(),
                limit: 10
            })
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn read_only_open_searches_without_writing_or_rebuilding() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let writer = Store::open(&repo.0).unwrap();
    writer
        .append(event("reader", "read only lookup phrase"))
        .unwrap();
    drop(writer);
    let db = repo.0.join(".vexen/state.sqlite");
    let before = fs::read(&db).unwrap();
    let reader = Store::open_read_only(&repo.0).unwrap();
    assert_eq!(
        reader
            .search(StoreQuery {
                text: "lookup".into(),
                limit: 10
            })
            .unwrap()
            .len(),
        1
    );
    assert!(reader.append(event("no-write", "must not write")).is_err());
    drop(reader);
    assert_eq!(fs::read(&db).unwrap(), before);
}

#[test]
fn fts_search_treats_operators_quotes_and_punctuation_as_literal_text() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    store.append(event("alpha", "alpha only phrase")).unwrap();
    store.append(event("beta", "beta only phrase")).unwrap();
    let literal = store
        .append(event("literal", "alpha OR beta literal"))
        .unwrap();
    let found = store
        .search(StoreQuery {
            text: "alpha OR beta".into(),
            limit: 10,
        })
        .unwrap();
    assert_eq!(
        found.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
        vec![literal.clone()]
    );
    let found = store
        .search(StoreQuery {
            text: "alpha \"OR\" (beta) *".into(),
            limit: 10,
        })
        .unwrap();
    assert_eq!(
        found.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
        vec![literal]
    );
}

#[test]
fn integrity_rejects_tampered_parent_and_unknown_event_kind() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    let first = store.append(event("first-root", "first root")).unwrap();
    let other = store.append(event("second-root", "second root")).unwrap();
    store
        .outcome(
            first.clone(),
            NewOutcome {
                idempotency_key: "run".into(),
                test_id: "cargo test x".into(),
                exit_code: 1,
                output_sha256: None,
            },
        )
        .unwrap();
    drop(store);
    let db = repo.0.join(".vexen/state.sqlite");
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch("DROP TRIGGER events_no_update;")
        .unwrap();
    conn.execute(
        "UPDATE events SET parent_id=?1 WHERE kind='outcome'",
        [other.as_str()],
    )
    .unwrap();
    drop(conn);
    assert!(Store::open_read_only(&repo.0).is_err());

    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    store.append(event("kind", "kind tamper")).unwrap();
    drop(store);
    let db = repo.0.join(".vexen/state.sqlite");
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch("DROP TRIGGER events_no_update;")
        .unwrap();
    conn.execute("UPDATE events SET kind='mystery'", [])
        .unwrap();
    drop(conn);
    assert!(Store::open_read_only(&repo.0).is_err());
}

#[test]
fn integrity_authenticates_event_id_and_idempotency_key_columns() {
    for tamper_id in [false, true] {
        let repo = Repo::new();
        Store::initialize(&repo.0).unwrap();
        let store = Store::open(&repo.0).unwrap();
        store.append(event("auth-column", "column tamper")).unwrap();
        drop(store);
        let db = repo.0.join(".vexen/state.sqlite");
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch("DROP TRIGGER events_no_update;")
            .unwrap();
        if tamper_id {
            conn.pragma_update(None, "foreign_keys", "OFF").unwrap();
            let fake = "f".repeat(64);
            conn.execute("UPDATE records SET id=?1", [&fake]).unwrap();
            conn.execute("UPDATE events SET id=?1", [&fake]).unwrap();
        } else {
            conn.execute("UPDATE events SET idempotency_key='rewritten-key'", [])
                .unwrap();
        }
        drop(conn);
        assert!(Store::open_read_only(&repo.0).is_err());
    }
}

#[test]
fn failed_schema_migration_rolls_back_all_partial_ddl() {
    let repo = Repo::new();
    let db = repo.0.join(".vexen");
    fs::create_dir_all(&db).unwrap();
    let db_file = db.join("state.sqlite");
    let conn = rusqlite::Connection::open(&db_file).unwrap();
    conn.execute("CREATE TABLE records (sentinel TEXT)", [])
        .unwrap();
    drop(conn);
    assert!(Store::initialize(&repo.0).is_err());
    let conn = rusqlite::Connection::open(&db_file).unwrap();
    let metadata_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='metadata')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let events_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='events')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let version: u32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert!(!metadata_exists);
    assert!(!events_exists);
    assert_eq!(version, 0);
    conn.execute("DROP TABLE records", []).unwrap();
    drop(conn);
    Store::initialize(&repo.0).unwrap();
    assert_eq!(
        Store::open_read_only(&repo.0)
            .unwrap()
            .schema_version()
            .unwrap(),
        1
    );
}

#[test]
fn supersession_is_a_typed_append_only_event_and_survives_projection_rebuild() {
    let repo = Repo::new();
    Store::initialize(&repo.0).unwrap();
    let store = Store::open(&repo.0).unwrap();
    let id = store
        .append(event("supersede-root", "old observation"))
        .unwrap();
    let supersession = store
        .supersede(
            id.clone(),
            "replaced".into(),
            "newer evidence recorded".into(),
        )
        .unwrap();
    assert_eq!(
        store.search(StoreQuery::default()).unwrap()[0].status,
        vexen::store::RecordStatus::Superseded
    );
    let chain = store.get(id.clone()).unwrap();
    assert_eq!(chain.events.len(), 2);
    assert_eq!(chain.events[1].id, supersession);
    assert_eq!(chain.events[1].kind, EventKind::Supersession);
    assert!(matches!(
        chain.events[1].payload,
        EventPayload::Supersession { .. }
    ));
    assert!(store.verify_integrity().unwrap());
    drop(store);
    let reopened = Store::open_read_only(&repo.0).unwrap();
    assert_eq!(
        reopened.search(StoreQuery::default()).unwrap()[0].status,
        vexen::store::RecordStatus::Superseded
    );
}

#[cfg(unix)]
#[test]
fn repository_identity_preserves_non_utf8_canonical_path_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let suffix = format!("-{}", std::process::id());
    let mut raw = b"vexen-store-\xff".to_vec();
    raw.extend_from_slice(suffix.as_bytes());
    let raw_path = std::env::temp_dir().join(std::ffi::OsString::from_vec(raw));
    let unicode_path = std::env::temp_dir().join(format!("vexen-store-\u{fffd}{suffix}"));
    for path in [&raw_path, &unicode_path] {
        fs::create_dir_all(path).unwrap();
        assert!(
            Command::new("git")
                .arg("init")
                .arg("-q")
                .arg(path)
                .status()
                .unwrap()
                .success()
        );
        Store::initialize(path).unwrap();
    }
    let raw_id = Store::open_read_only(&raw_path)
        .unwrap()
        .repo_id()
        .to_string();
    let unicode_id = Store::open_read_only(&unicode_path)
        .unwrap()
        .repo_id()
        .to_string();
    assert_ne!(raw_id, unicode_id);
    let _ = fs::remove_dir_all(raw_path);
    let _ = fs::remove_dir_all(unicode_path);
}
