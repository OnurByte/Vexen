# Status and roadmap

Where Vexen stands today, in one page. Anything not listed under *Shipped* is not a current capability. The old design and task backlog is kept as a [historical specification](archive/specification.md); it is not the current plan.

## Shipped and tested

- **Evidence store:** per-checkout SQLite, append-only and hash-chained; notes are tied to a file's contents, a commit and an observed test result. See [data formats](formats.md).
- **Command line:** `init`, `check`, `explain`, `doctor`, `record`, `outcome`, `observe`, `export`, `import`, `install`, `uninstall`.
- **MCP server:** four tools (`vexen_check`, `vexen_record`, `vexen_outcome`, `vexen_explain`) over stdio. See the [protocol](protocol.md).
- **Installer:** writes the MCP entry for eight clients, with preview, one-time backup and uninstall; `--npx` for people who never installed the binary.
- **Distribution:** `npx @onurbyte/vexen`, prebuilt binaries for five platforms with SHA-256 files and build-provenance attestations, source install with `cargo`.
- **Agent skill:** [`skills/vexen/SKILL.md`](../skills/vexen/SKILL.md).
- **Safeguards:** secret-like text is refused, paths are confined to the repository, output is bounded, stale or unrelated notes stay silent, conflicting results are reported as conflicts. See [security and privacy](security.md).

## Checked with real clients and agents

- Memory pilot after the delivery fixes: 3 of 20 without memory, 12 of 20 with memory (lesson delivered in 14), then 17 of 20 with memory and 20 of 20 delivered after a `query` fix. One model, synthetic tasks, caveats in the [research summary](research.md#memory-pilot-follow-up-after-the-delivery-fixes).
- Claude Code, OpenCode and Codex CLI: one live session each on Windows. 13 more clients completed a real `vexen_check` call on Linux. Per-client detail and limits: [client checks](client-smoke-tests.md).
- A paired coding pilot (synthetic tasks, one model) and a retrieval analysis: [research summary](research.md).

## Experimental, off by default

- **File hints from Git history ("what changes together").** It lost to plain co-change counts on all ten evaluated repositories, so it is not shown by default. See [the evaluation](benchmarks.md).
- **Code-fragment recall** and similar ideas stay off until a comparative evaluation supports them.

## Shipped in 1.0

- **`vexen setup`:** Rust-native, no Node.js or Python after the first run. Starts project memory, places a permanent binary, writes MCP entries, installs the skill, then starts the server and checks that four tools answer. Evidence: `tests/setup.rs` (12 tests, including a home path with a space and a non-ASCII letter, idempotence, dry run, an edited skill left alone, a config it cannot edit safely).
- **`vexen demo`:** a synthetic lesson in a throwaway repository; shows `observed_failure` before an edit and `no_relevant_evidence` after, then deletes itself.
- **`vexen doctor --mcp`:** a real stdio handshake. It proves the server works, not that an agent loads it.
- **Skill activation:** the skill is MCP-first and records the command for the machine. Verified with a real agent for Claude Code only (one run, one model, Windows); other agents have no verified skill path and are not claimed.
- **Lifecycle tests:** property-based (`tests/lifecycle.rs`): never a stale or superseded note, a failing result is not hidden, at most three items, intact chain, lossless idempotent export and import.
- **MCP Inspector smoke:** `scripts/inspector_smoke.py` drives the server with the official Inspector (pinned 2.13.0) in CI. A development check, not a runtime dependency.
- **`vexen observe-junit`:** attaches failing tests from a JUnit XML report (nextest, pytest, Maven, Jest) to a record, without storing messages or output; idempotent. Chosen from the [GitHub research](development/github-strengthening-opportunities.md) (item 1) because it adds the structured test evidence the product already depends on without a new runtime service. Tests: `tests/junit.rs` and parser unit tests.
- **A real-agent finding, fixed:** free-text `query` used to hide the note for the asked file (0 of 6 such runs got the lesson); it is now a hint when `paths` is given.
- **Full test suite on Windows, macOS and Linux:** the control-character path test now builds its commit from the index, so it runs everywhere.
- **README cards** are 720 units wide with no text under 24 units, checked at 360 and 768 px in light and dark.

## Evaluated and deferred

- `cargo-nextest` as a dependency (its reports are read through the generic JUnit importer instead), `add-mcp`, `cargo-dist`, `self_update`/`axoupdater`, tree-sitter: not adopted. Each would add a dependency or a second installer without closing a gap that current tests leave open. Revisit when a concrete need appears.
- A verified skill path for agents other than Claude Code, and live checks of more clients: need those apps and accounts installed.
- Project-key signing of releases: needs a repository secret only the maintainers can add.

## What 1.0 means

1.0 is a compatibility promise, not a claim that every question is answered.

- **Stable:** the command-line commands and flags, the four MCP tools and their schemas, and the data formats ([formats](formats.md); store schema version 1 is the only one ever shipped, and a change to it will come with a tested migration). Breaking changes need a major version.
- **Checked live with a real agent:** Claude Code, OpenCode and Codex CLI (one session each). **Beta:** the other clients; they completed a real `vexen_check` call but were not used for real work. See the [client checks](client-smoke-tests.md).
- **Not part of the 1.0 claim:** a paired evaluation on real repository tasks with more than one agent (the pilots use one model and synthetic tasks); an independent human security review; releases signed with a project key (builds carry GitHub build-provenance attestations and SHA-256 files); file hints and code-fragment recall, which stay experimental and off by default.

History of releases: [CHANGELOG](../CHANGELOG.md).
