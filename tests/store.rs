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
