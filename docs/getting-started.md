# Get started

This page takes you from nothing to a saved and retrieved lesson. You need Git and one of: Node.js 18 or newer (easiest), or a prebuilt binary, or a Rust toolchain.

## The short way: one command

Open a terminal **inside the Git project you want Vexen to remember**.

```sh
npx @onurbyte/vexen setup          # PowerShell: npx.cmd @onurbyte/vexen setup
```

`setup` creates the project's memory, puts a permanent copy of Vexen on your machine (so Node.js is not needed afterwards), connects the coding tools it finds, installs the skill where a verified path exists (Claude Code), then starts Vexen once and checks that its four tools answer. Useful options: `--dry-run` shows the plan and writes nothing, `--agent <name>` limits it to one tool, `--force` replaces a skill file you edited. It exits with code 2 when a step failed or no agent was found, and it never claims your agent loads the entry: restart the agent and ask the question in step 3.

Other commands: `vexen demo` (the whole loop on a throwaway synthetic repository, deleted afterwards), `vexen doctor --mcp` (start the server and list its tools), `vexen uninstall --skill` (remove what `setup` wrote; project memory is never deleted).

The rest of this page is the manual route and the reference.

## 1. Run it (pick one)

### With `npx`: no install, no Rust

Open a terminal **inside the Git project you want Vexen to remember**. The first run downloads the checksummed binary for your platform, verifies it and caches it; after that it starts instantly.

macOS / Linux:

```sh
npx @onurbyte/vexen init
npx @onurbyte/vexen doctor
npx @onurbyte/vexen check --diff HEAD
```

Windows PowerShell (use `npx.cmd` so PowerShell's script policy does not block the launcher):

```powershell
npx.cmd @onurbyte/vexen init
npx.cmd @onurbyte/vexen doctor
npx.cmd @onurbyte/vexen check --diff HEAD
```

### With a prebuilt binary: no Node.js, no Rust

1. [Download Vexen](https://github.com/OnurByte/Vexen/releases/latest) for your operating system. Choose Apple Silicon or Intel on macOS; most Windows and Linux computers use x64. Each archive has a `.sha256` file next to it.
2. Extract the ZIP or tar.gz archive. On macOS and Linux, make the binary executable if needed: `chmod +x vexen`.
3. Put the binary in a folder on your `PATH`. On macOS and Linux a common choice is `~/.local/bin` (the folder must already be on your `PATH`). On Windows, put `vexen.exe` in a permanent folder and add that folder to your user Path in **Environment Variables**, then reopen your terminal.
4. Inside your Git project run `vexen init`, `vexen doctor` and `vexen check --diff HEAD`.

The binary needs Git, but not Node.js or Rust.

### From source

Needs Rust 2024 and Git:

```sh
cargo install --git https://github.com/OnurByte/Vexen --locked
```

The rest of this page writes `vexen`; with `npx`, read it as `npx @onurbyte/vexen` (`npx.cmd` in PowerShell).

<p align="center"><img src="assets/terminal.svg" alt="Real terminal output from running vexen init, vexen doctor and vexen check on a fresh Git repository: the store is created, reports integrity ok with zero events, and check answers no relevant evidence." /></p>

On a fresh repository the honest answer is "no relevant evidence", and that is what you see. The store (a local `.vexen/` folder, with its database files ignored by Git) starts empty and grows as lessons are saved.

## 2. Connect your agents

`vexen install` adds the MCP entry to the config files of the agents it finds (Claude Code, Codex CLI, Cursor, Gemini CLI, Windsurf, Kiro, VS Code, OpenCode). Preview first, then apply; `vexen uninstall` removes only what it added.

```sh
# with the binary on your PATH
vexen install --scope global --dry-run
vexen install --scope global

# with npx only: the config makes your agents start Vexen through npx
npx @onurbyte/vexen install --scope global --npx
```

Read the output: some tools need a manual step, and **you need to restart your coding tool** afterwards. The installer writes only the `vexen` entry, keeps a one-time `.vexen-backup` of any file it edits, never changes auto-approval or permission settings, and refuses (printing the snippet to add by hand) when a file cannot be edited safely, for example JSON with comments or a symbolic link. Zed and Cline print a snippet only. Scopes differ: VS Code is project-only, Codex CLI and Windsurf are global-only, and `--agent auto` only picks clients whose config folder it finds.

### Check that it works

Writing a config file is not the same as your agent using Vexen. Four separate things can be true:

1. **Installed:** `vexen doctor` reports `integrity: ok`.
2. **Connected:** your agent lists Vexen among its MCP tools after a restart.
3. **Callable:** ask your agent, in its chat: *"Use Vexen to check this project's README.md for past failures or decisions."* On a new project the correct answer is "no relevant evidence".
4. **Used on its own:** whether an agent checks Vexen without being asked depends on the agent. The optional [agent skill](#3-use-it) tells it to.

Which tools have been tested, and how far, is in the [client checks](client-smoke-tests.md). The file formats of the automated clients are covered by tests; installer-written configuration was exercised live for Claude Code and OpenCode.

## 3. Use it

### Save your first lesson

When a test fails during your work, ask your agent to save it, or do it yourself. A note needs a file and, to be shown later, a failing test result:

```sh
vexen record --path src/retry.rs --summary "Retrying after a timeout charged the customer twice"
# prints an id, for example 3e7a...; run the failing test and attach its result:
vexen observe <id> --test-id retry_no_double_charge -- cargo test retry_no_double_charge
vexen check --path src/retry.rs
```

`vexen observe` runs the command you type and stores only its exit code and a hash of its output. The note now appears for `src/retry.rs` as long as that file is unchanged. Vexen only brings back attempts with a recorded **failing** test result, and decisions you have accepted (`vexen record --kind decision`). Edit the file and the old note stops appearing, by design.

### From a test report (JUnit XML)

Most test runners can write a JUnit XML report (`cargo nextest run --profile ci`, `pytest --junitxml=report.xml`, Maven, Jest). Instead of re-running a command, attach the failing tests of a report you already have:

```sh
vexen observe-junit <event-id> report.xml            # failing tests only
vexen observe-junit <event-id> report.xml --test retry   # also passing tests whose name contains "retry"
```

Only test names and pass/fail are stored; failure messages, stack traces and captured output are dropped. Skipped tests and tests that only passed after a retry are left out, at most 20 failures are attached per call, and importing the same report twice adds nothing. It is a result you supplied, not a run Vexen watched, and Vexen says so.

### From an agent over MCP

`vexen mcp --stdio` serves four tools: `vexen_check` reads past evidence for a file, `vexen_record` saves a failed attempt or a decision, `vexen_outcome` attaches a test result, and `vexen_explain` shows where a note came from. To wire a client by hand:

```json
{ "command": "npx", "args": ["-y", "@onurbyte/vexen", "mcp", "--stdio"] }
```

(or `{ "command": "vexen", "args": ["mcp", "--stdio"] }` with the binary on your `PATH`; on Windows start `npx` through `cmd /c`). Agents pass the repository root explicitly on every call, so one server never leaks memory between projects. Details: [MCP protocol](protocol.md).

### As an agent skill

[`skills/vexen/SKILL.md`](../skills/vexen/SKILL.md) teaches an agent when to run `vexen check` before an edit and `vexen record` after a failed attempt. It follows the Agent Skills format, so any compatible agent can install it from this repository:

```sh
npx skills add OnurByte/Vexen
```

The skill is MCP-first: it reads through `vexen_check` and `vexen_explain` when your agent has them, and saves through the `vexen` command. It does not install Vexen itself. It works with `vexen` on your `PATH` or, if you only use `npx`, through `npx @onurbyte/vexen`.

### Check automatically before every edit (Claude Code)

The skill only reminds an agent; a hook runs without being asked. Add this to `.claude/settings.local.json` in your project (or your user settings), with the full path to your `vexen` binary if it is not on your `PATH`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [{ "type": "command", "command": "vexen hook claude-code" }]
      }
    ]
  }
}
```

Before Claude edits a file, `vexen hook claude-code` looks for a note that still applies to that file and, if there is one, hands it to the agent as advisory context. When nothing applies it prints nothing. It never blocks an edit, never answers a permission question and makes no network call. In one real run (Claude Code with Haiku, no instruction to check memory) the agent received the note and mentioned it in its answer; that is a single observation, not a measured rate. Vexen does not edit your settings file for you.

### Moving data

`vexen export --format jsonl` and `vexen import` move a store as versioned JSONL; `--format markdown` prints a readable list. See [data formats](formats.md).

## Troubleshooting

- **"No relevant evidence" on a new project.** Expected. The store starts empty; save a lesson as above.
- **A saved note does not appear.** The usual reasons: the file changed since the note was recorded; the note has no failing test result (a note by itself is not a verified failure, and a passing result keeps it quiet); the query's `--condition` values differ from the note's; or you are in a different repository.
- **The agent cannot find Vexen.** Restart the coding tool; check the scope (`--scope global` or `project`) and that your tool is one the installer supports; re-run `vexen install --scope global --dry-run` to see what it would write; or use `--print-config` and add the snippet yourself.
- **`vexen: command not found`** after downloading a binary: the folder is not on your `PATH`. Use `npx` instead, or fix the `PATH`.
- **On Windows PowerShell, `npx` is blocked** by script policy: use `npx.cmd`.
- **The installer refused a config file.** It will not rewrite JSON with comments, duplicate keys or a symbolic link, to avoid losing your settings. It prints the snippet; add it by hand.

## See also

[Status and roadmap](roadmap.md) · [Client checks](client-smoke-tests.md) · [MCP protocol](protocol.md) · [Data formats](formats.md) · [Security and privacy](security.md)
