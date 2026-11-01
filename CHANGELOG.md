# Changelog

Vexen is in early development. Versions below 1.0 may change behavior between releases.

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
