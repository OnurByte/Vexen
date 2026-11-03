# GitHub research behind the 1.0 feature choice (2026-11-03)

Status: research record. It explains why `vexen observe-junit` was built and what was left out. It extends the earlier [candidate list](github-strengthening-opportunities.md); nothing here is a claim about other projects' quality.

## Method

Repository searches on GitHub (`gh search repos`, sorted by stars) for four themes: MCP memory servers, coding-agent memory, Claude Code hooks, and JUnit tooling. Star counts and descriptions are as shown on 2026-11-03 and are popularity signals, not quality evidence. The search surface is thin for JUnit and Rust, so the earlier candidate list (which read source and documentation of each project) remains the main basis.

## What the landscape shows

- **Memory servers for coding agents are many and growing** (for example `vshulcz/deja-vu`, `riponcm/projectmem`, `memory-graph/memory-graph`, `mtrnix/metronix-memory`; 77 to 1,381 stars). Several keep notes, attempts or decisions. Vexen's difference is narrow and deliberate: a note carries a file hash, a commit and an observed test result, and stays silent when the code has moved on. That is the feature to deepen, not breadth of storage.
- **Hooks are a popular way to make agents use tools** (`disler/claude-code-hooks-mastery`, `karanb192/claude-code-hooks`, thousands of stars). It points at Vexen's largest open weakness: unprompted use.
- **JUnit XML is the common output of test runners** (nextest, pytest, Maven, Jest, Go via converters), and existing JUnit-to-MCP projects are analysis servers with no stars, not memory.

## Decisions

| Candidate | Fit with Vexen's rules | Decision |
|---|---|---|
| Attach failing tests from a JUnit XML report | Local, offline, no new service; the exit code and test name are exactly what a note needs | **Built**: `vexen observe-junit` (names and pass/fail only, idempotent, tests in `tests/junit.rs`) |
| Property-based lifecycle tests | Closes a gap in example tests | **Built** (earlier) |
| MCP Inspector as an independent protocol check | Development-time only | **Built** (earlier, CI job) |
| Claude Code hook that runs `vexen check` before an edit | Would fix unprompted use, the biggest caveat in the pilots | **Not built.** It edits an agent's settings file and needs its own paired evaluation without the "please check memory" instruction. Next candidate |
| Code-fragment recall, embeddings, vector database, separate memory server | Break the small, offline contract; no evidence they fix a current failure | Rejected for now |
| `cargo-nextest` as a dependency | A second tool for little gain | Rejected; its JUnit output is read by the generic importer instead |

## Limits

Popularity is not evidence of fit. The JUnit importer reads names and outcomes only; it does not store test output. No benchmark yet shows that importing a report changes agent results; it removes a manual step and a re-run.
