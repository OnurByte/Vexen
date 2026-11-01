# Task 2 report: durable event store

**Status: DONE_WITH_CONCERNS.** The per-checkout SQLite store is implemented in schema version 1. This slice records local evidence; it does not decide whether that evidence is still applicable or “supported.”

## Public API for the next worker

- `Store::initialize(&Path)` explicitly creates `.vexen`, local ignore entries for `state.sqlite`, `state.sqlite-wal`, and `state.sqlite-shm`, and the database. `Store::open(&Path)` requires an initialized, canonical, non-bare Git checkout root. Different worktrees bind to different canonical paths and cannot open a copied database.
- `Store::append(NewEvent) -> EventId` accepts `EventKind::Attempt` or `EventKind::AcceptedDecision`, a unique idempotency key, summary, repo-relative anchor path and working-content SHA-256, optional full Git object ID, and bounded condition strings. Repeating the same key and payload returns its ID; a changed payload under that key errors.
- `Store::outcome(EventId, NewOutcome) -> EventId` appends an outcome with caller-supplied idempotency key, test identifier, exit code, and optional output digest. Raw output is not accepted or stored. Retries are idempotent; contradictory evidence with distinct keys remains in the event chain and projection.
- `Store::get(EventId)` returns the root event and its outcome chain; `Store::events()` enumerates the immutable log in append order. `Store::repo_id()` and `schema_version()` expose envelope metadata for export. Event hashes include repo ID, sequence, previous hash, and serialized payload; import must validate those values against the destination repo binding and must not rewrite them.
- `Store::search(StoreQuery { text, limit })` reads an FTS5-backed `MemoryRecord` projection. The projection carries `repo_id`, explicit `RecordStatus::{Candidate, AcceptedDecision, Observed}`, anchor fields, and **all** recorded outcomes. `Observed` means evidence was recorded, not that it is current or verified. The projection is regenerated from the verified event log when opening the store.
- `verify_integrity()` runs SQLite quick-check and validates sequence, repo-bound event IDs, parent roots, and the SHA-256 chain. `Event`, `EventPayload`, `NewEvent`, `NewOutcome`, `MemoryRecord`, and `EventId` are serde-serializable for future export work.

## Durability and migration

Schema version 1 creates metadata, immutable events, records projection, FTS5 table, FTS triggers, and event update/delete rejection triggers. Initial DDL and `user_version` update run in one exclusive SQLite transaction. Each append/outcome is one immediate transaction; WAL, `synchronous=FULL`, foreign keys, and a five-second busy timeout are configured for each connection. SQLite recovery is exercised by closing and reopening committed data. On Unix, `.vexen` is mode `0700` and the database mode `0600`; symlinked store paths are rejected.

Migration currently supports only fresh schema 0 to schema 1 and refuses unknown newer versions. There is no import path or killed-process migration fault-injection test in this slice. A failed first migration can be rerun because its schema DDL and version change roll back together. This version has no schema upgrade beyond 1 yet.

## Test-first evidence

- Initial RED: before the store module existed, `cargo test --test store` failed at compile time with `could not find store in vexen`, as expected for the requested API.
- Additional RED: before projection rebuild was implemented, `cargo test --test store derived_search_projection_is_rebuilt_from_immutable_events_on_open` failed because tampered derived text remained absent from FTS search (`left: 0`, expected `1`).
- GREEN: `RUSTUP_TOOLCHAIN=1.99.1 cargo test` passed: 1 library unit test, 12 Git integration tests, 9 store integration tests, and doc tests (22 tests total, no failures).
- `RUSTUP_TOOLCHAIN=1.99.1 cargo test --all-targets` passed the same 22 tests and compiled/launched the benchmark target; the benchmark reported that no repository was supplied, so it did not run an evaluation.
- `RUSTUP_TOOLCHAIN=1.99.1 cargo clippy --all-targets -- -D warnings` passed.
- Rustfmt check passed for all Task 2 Rust files using `RUSTUP_TOOLCHAIN=1.99.1 rustfmt --check --edition 2024 src/store/mod.rs src/store/events.rs src/store/migrations.rs src/store/query.rs tests/store.rs` and the corresponding `src/lib.rs` check with `--config skip_children=true`. `cargo fmt -- --check` reports formatting differences in the pre-existing `benchmarks/cochange/main.rs`; that unrelated file was left untouched.

Tests cover explicit initialization/open, schema version, appending and outcome linkage, FTS search, both record and outcome idempotency, conflicting payload rejection, concurrent independent connections, committed close/reopen/hash verification, update/delete rejection, cross-checkout database rejection, and rebuilding a tampered projection from events.

## Limitations

- The caller is responsible for ensuring `test_id` is an allowlisted identifier; the store bounds and rejects control characters but has no allowlist registry yet.
- Anchor digests are format-validated but not rechecked against the current filesystem here. The validity layer must perform that check before presenting evidence as applicable.
- SQLite's hash chain detects accidental or unsophisticated tampering; it is not a signature or protection against an attacker able to rewrite the entire database and recompute hashes.
- No history capture, commands, network requests, or automatic Git-derived hint activation is implemented by the store.
