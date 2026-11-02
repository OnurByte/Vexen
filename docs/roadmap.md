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

- Claude Code, OpenCode and Codex CLI: one live session each on Windows. 13 more clients completed a real `vexen_check` call on Linux. Per-client detail and limits: [client checks](client-smoke-tests.md).
- A paired coding pilot (synthetic tasks, one model) and a retrieval analysis: [research summary](research.md).

## Experimental, off by default

- **File hints from Git history ("what changes together").** It lost to plain co-change counts on all ten evaluated repositories, so it is not shown by default. See [the evaluation](benchmarks.md).
- **Code-fragment recall** and similar ideas stay off until a comparative evaluation supports them.

## Blocked

- **A fresh real-agent evaluation after the memory-delivery fixes.** It needs an authenticated coding agent; the exact blocker and the fixes are recorded in the [reliability investigation](memory-reliability-investigation.md).

## Proposed, not built

- A guided `setup` command that detects agents, initializes the repository, connects MCP, activates the skill and verifies it: [proposal](development/rust-native-installation.md).
- MCP Inspector and property-based tests as pinned development checks, only where they close a gap that current tests do not: [candidate list](development/github-strengthening-opportunities.md).
- A safe, clearly synthetic demo repository for first use.

## What 1.0 would need

These are proposed criteria, not commitments:

1. A paired evaluation on real repository tasks, with more than one agent and uncertainty estimates.
2. Frozen CLI, MCP and data-format contracts, with upgrade tests from every earlier store version.
3. Live checks for each client labelled stable; the rest labelled beta.
4. An independent human security review and a documented release-signing story.
5. Test fixtures that run on Windows, macOS and Linux without platform skips.

History of releases: [CHANGELOG](../CHANGELOG.md).
