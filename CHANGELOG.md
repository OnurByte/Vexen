# Changelog

Vexen is in early development. Versions below 1.0 may change behavior between releases.

## Versioning

- Releases are patch bumps by default. A minor or major bump needs `Release-Bump: minor` or `Release-Bump: major` on the tagged commit; the release workflow rejects skipped or accidental jumps.
- The published sequence is `0.1.0, 0.1.1, 0.2.0, 0.2.1, 0.2.2, 0.2.3`. `0.1.1` to `0.2.0` was a deliberate minor bump (the installer and `observe` were new features) made before the guard existed. Published tags, releases and npm versions are not renumbered: an npm version number cannot be reused once published.
- `0.2.1` carried no code changes (npm publishing moved to trusted OIDC).

## Unreleased

- `benchmarks/memory`: second pilot round (76 runs, 7 tasks) with per-run logging of `vexen_check` calls; results in docs/pilot-results.md.
- Codex CLI 0.166.0 smoke-tested live on Windows with the official npm package (a real model called `vexen_check`). The registry label for Codex is updated.
- OpenCode 2.2.5 smoke-tested live on Windows (installer-written config, `vexen_check` called by a real model). The registry label for OpenCode is updated.

## 0.2.3

Hardening from an independent multi-model review; every finding was verified against the code first.

- Installer: backups are created exclusively and never overwritten; the temp file is created exclusively with the original's permissions; the write is abandoned if the config changed meanwhile; symbolic-link configs and JSON with duplicate keys are refused instead of rewritten.
- `record`: files over 16 MiB are refused, an invalid byte can no longer hide a secret from the scan, and hidden/secret path checks treat `\` like `/`.
- Redaction: `Authorization` headers and every PEM private-key header are detected; tokens are redacted even when a colon follows them.
- `import`: a dangling parent reference is rejected before anything is written.
- `check` items use snake_case statuses (`observed_failure`, `accepted_decision`, `conflicted`); previously `observedfailure` and `accepteddecision`. This changes the item `status` strings, so clients that matched the old ones must update.
- Earlier unreleased items: OpenCode 2.x layout, the memory pilot harness and results.

- `benchmarks/memory`: paired memory-on/off pilot harness with a real agent, and its first results (docs/pilot-results.md). Small and toy-sized; no general effect is claimed.
- Fix: the OpenCode adapter wrote the legacy `mcp.<name>` layout. OpenCode 2.x's own `opencode mcp add` writes `mcp.servers.<name>`, so that is now the default; a file that already uses the legacy layout keeps it. Found by comparing with the client's own writer. A live model call through OpenCode was not possible (no API key for its configured provider), so OpenCode stays `config-tested`.

## 0.2.2

- Fix: on Windows the MCP tools rejected an ordinary `C:\...` `repo_root` ("must be canonical"); the ordinary spelling of the same directory is now accepted. Found by running a real client against the release.
- Fix: tool output schemas no longer advertise the non-standard `uint32` format, which made Claude Code print a warning on every call.
- Release archives get build-provenance attestations.
- CI now runs the npm launcher against the latest published release on Linux, macOS and Windows.

## 0.2.1

- Release plumbing only: npm publishing switched to trusted OIDC publishing. The npm package `@onurbyte/vexen` is published, so `npx @onurbyte/vexen` works.

## 0.2.0

- `vexen install` and `vexen uninstall` for eight clients (Claude Code, Codex CLI, Cursor, Gemini CLI, Windsurf, Kiro, VS Code, OpenCode), with `--dry-run`, `--print-config`, `--scope global|project`, one-time backups, atomic writes and refusal on files that cannot be edited losslessly. Zed and Cline print a snippet only. Formats are fixture-tested; no live client smoke tests yet.
- `vexen observe <id> --test-id <t> -- <command>` runs a command you type and records only its exit code and an output hash.
- Documentation: MCP protocol, data formats, security model.

## 0.1.1

- `vexen record` and `vexen outcome` so notes can be saved without MCP.
- Agent Skill at `skills/vexen/SKILL.md` (`npx skills add OnurByte/Vexen`).

## 0.1.0

- First release: Git-history analysis (research only), SQLite evidence store, MCP server with four tools, CLI (`init`, `check`, `explain`, `doctor`, `export`, `import`), checksummed binaries for five platforms, npm launcher (not yet published).
