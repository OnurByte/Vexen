# Data formats

## Storage

Each initialized checkout has `.vexen/state.sqlite` (SQLite, WAL). `.vexen/.gitignore` ignores the database and its `-wal`/`-shm` files; Vexen does not edit the repository's own `.gitignore`. Events are append-only and hash-chained; `vexen doctor` verifies the chain and SQLite's structural check.

Schema version is stored in `PRAGMA user_version` (currently 1). Opening a store runs migrations inside transactions; read-only opens refuse a store whose version differs.

## Events

Three event types share one chain: `record` (an attempt or an accepted decision), `outcome` (child of a record) and `supersession` (child of a record). Each event has `id`, `parent_id`, `idempotency_key`, `sequence`, `kind`, `payload`, `previous_hash` and `hash`. Repeating an event with the same key and content returns the existing id; the same key with different content is an error.

## JSONL export

`vexen export --format jsonl` writes one JSON value per line:

1. A header: `{"vexen_export":1,"repo_id":"...","event_count":N}`
2. One event per line, in append order, exactly as stored.

`vexen import <file>` validates the whole file first (header version, every line parses, no secret-like text), then replays it in order. Parent links are remapped to the importing checkout's ids, so importing is idempotent and works across checkouts. Hashes and ids differ between checkouts because ids derive from the repository identity; the *content* is what is preserved.

## Markdown export

`vexen export --format markdown` is a readable list of records with their outcomes and supersessions. It is for people; use JSONL for lossless transfer.

## Versioning

The export header version and the store schema version change independently. A breaking change to either bumps its number and is listed in the [changelog](../CHANGELOG.md).
