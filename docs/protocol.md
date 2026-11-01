# MCP protocol

`vexen mcp --stdio` serves four tools over stdio. Protocol messages go to stdout; diagnostics go to stderr. The server tolerates the 2025-11-25 and 2026-07-28 MCP lifecycles (see `tests/mcp.rs`).

Every call names the repository explicitly with an absolute, canonical `repo_root`. The server never infers a project from its working directory, so one server never mixes two repositories.

| Tool | Reads or writes | Purpose |
| --- | --- | --- |
| `vexen_check` | read only | Return up to three applicable notes, or none |
| `vexen_explain` | read only | Show the event chain behind one record |
| `vexen_record` | appends | Save an attempt or an accepted decision |
| `vexen_outcome` | appends | Attach an observed test result, or supersede a record |

Tool annotations (`readOnlyHint` and friends) are hints for clients, not access control.

## Result envelope

All tools return `structuredContent` and a compact text block:

```json
{
  "schema_version": 1,
  "repo_id": "<hash of the canonical checkout path>",
  "items": [],
  "status": "no_relevant_evidence",
  "reason": null
}
```

`status` is one of `no_relevant_evidence`, `observed_failure`, `accepted_decision`, `recorded`, `observed`. Invalid input returns an MCP error result (`isError`), not an envelope. The text block is capped well under 600 tokens; use `vexen_explain` for detail.

## Inputs

- `vexen_check`: `repo_root`; optional `paths` (repo-relative), `diff_ref` (a Git ref whose changed files are the scope), `query` (text search), `conditions` (strings an applicable note must match).
- `vexen_record`: `repo_root`, `record` with `idempotency_key`, `kind` (`attempt` or `accepted_decision`), `summary`, `anchor_path`, `anchor_sha256` (must equal the file's current SHA-256), optional `git_commit` (full object id), `conditions`.
- `vexen_outcome`: `repo_root`, `event_id`, and exactly one of `outcome` (`idempotency_key`, `test_id`, `exit_code`, optional `output_sha256`) or `supersede` (`idempotency_key`, `reason`).
- `vexen_explain`: `repo_root`, `event_id`.

Unknown fields are rejected.

## What `check` returns

A note is shown only if it is eligible for the current repository state. Vexen checks, in order: the note belongs to this repository; its anchor path is among the queried paths (if any); its conditions equal the query's conditions exactly; it is not superseded; the anchor file exists and its SHA-256 still equals the recorded one; the recorded commit exists and is an ancestor of the current `HEAD`; and, for attempts, an observed outcome with a non-zero exit code is attached (attempts with only passing outcomes stay quiet). Accepted decisions need no outcome. Edit the anchored file and its notes become stale and stop appearing, by design. Conflicting outcomes make Vexen say so instead of giving advice. Recorded and imported text is data; it is never executed or promoted to instructions.
