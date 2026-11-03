<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="icon.svg">
    <img src="icon-light.svg" width="84" height="78" alt="Vexen" />
  </picture>
</p>

<h1 align="center">Vexen</h1>

<p align="center"><strong>Your AI agent forgot what failed. Vexen didn't.</strong></p>
<p align="center">Vexen saves what failed in your project, with its test result, so your next agent, even in another tool, checks it before trying again.</p>

<p align="center">
  <a href="https://github.com/OnurByte/Vexen/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/OnurByte/Vexen?label=release&color=78a9ff&labelColor=262626" /></a>
  <a href="https://github.com/OnurByte/Vexen/stargazers"><img alt="GitHub stars" src="https://img.shields.io/github/stars/OnurByte/Vexen?color=78a9ff&labelColor=262626" /></a>
  <img alt="Free and open source" src="https://img.shields.io/badge/free-open%20source-78a9ff?labelColor=262626" />
  <img alt="Local, no account" src="https://img.shields.io/badge/local-no%20account-78a9ff?labelColor=262626" />
</p>

<p align="center">
  <a href="https://www.producthunt.com/products/vexen?embed=true&utm_source=badge-featured&utm_medium=badge&utm_campaign=badge-vexen" target="_blank" rel="noopener noreferrer"><img alt="Vexen - Memory that stops AI agents from repeating failed fixes | Product Hunt" width="250" height="54" src="https://api.producthunt.com/widgets/embed-image/v1/featured.svg?post_id=1303498&theme=dark" /></a>
</p>

<p align="center"><a href="#install"><strong>Install in one command ↓</strong></a> · <a href="https://github.com/OnurByte/Vexen">⭐ Star on GitHub</a> · <a href="#the-evidence">See the evidence</a></p>

```sh
npx @onurbyte/vexen setup
```

<p align="center"><img src="docs/assets/readme-demo.svg" alt="Real output of the vexen demo command, shortened. A lesson about retry.py is saved with a failing test, shown as observed_failure before the file is edited, then no_relevant_evidence after the file changes. The failure is invented for the demo." /></p>

<p align="center">
  <a href="docs/assets/vexen-launch.mp4"><img src="docs/assets/vexen-launch.gif" width="800" alt="Animated 15-second Vexen launch demo: repository memory, pilot results, and one-command setup for macOS, Linux, and Windows." /></a>
</p>
<p align="center"><a href="docs/assets/vexen-launch.mp4"><strong>▶ Open the full-resolution 15-second video (MP4)</strong></a></p>

## The problem, and the fix

<p align="center"><img src="docs/assets/readme-before-after.svg" alt="Illustrative scenario. Without Vexen: Claude tries a fix on Monday, the test fails, and a fresh Codex chat on Friday may try it again. With Vexen: the failure and test result are saved, so Codex can check them and decide whether the lesson still applies." /></p>

**You don't need to save a conversation.** Your agent saves a short note: what failed, in which file, with its test result. A later session checks it before editing the same code. If the file changes, the note steps aside.

<p align="center"><img src="docs/assets/readme-benefits.svg" alt="Keep the lesson: failed fixes and key decisions for the next session. See the proof: the file and test result behind a reminder. Keep it local: on your machine, with no account, cloud or telemetry." /></p>

## The evidence

A solved task cost **77.7% less** with Vexen memory in our pilot: 4.5× cheaper.

<p align="center"><img src="docs/assets/readme-cost.svg" alt="Small internal pilot with one model: a solved task cost 77.7% less with relevant Vexen memory, 4.5 times cheaper: about 1.1 cents instead of 4.8 cents. A single run cost about 11% less; most of the saving comes from more runs succeeding (60% against 15%). One check adds at most 1,800 bytes, roughly 450 tokens. Twenty runs per group, synthetic tasks, not a general cost-saving claim." /></p>

The saving comes from more runs succeeding, not from shorter runs. In the first published pilot, Claude Code with Haiku solved **55% of tasks with relevant memory, against 10% without** (11 of 20 against 2 of 20).

<p align="center"><img src="docs/assets/readme-evidence.svg" alt="Small internal coding pilot: 55% of runs solved with relevant Vexen memory (11 of 20), versus 10% without memory (2 of 20), 45 percentage points more. Five synthetic tasks, one model, pre-recorded evidence and explicit instructions to check memory. Not a general coding performance claim." /></p>

These are small, honest numbers: one model, synthetic tasks, notes prepared in advance. They do **not** show general coding improvement. [Method, raw data, stale-note and control checks, and limits →](docs/research.md)

## Install

<p align="center"><img src="docs/assets/readme-install.svg" alt="Vexen setup runs on macOS (arm64 and x64), Linux (arm64 and x64) and Windows (x64) with npx @onurbyte/vexen setup. Windows PowerShell uses npx.cmd. Node.js is only needed for the one-time installer." /></p>

Run it **inside the Git project you want Vexen to remember**. You need [Node.js 18 or newer](https://nodejs.org/) (only for this installer) and Git.

On Windows PowerShell use `npx.cmd @onurbyte/vexen setup`. `setup` creates your project's memory (a local `.vexen/` folder), keeps a permanent copy of Vexen on your machine, connects the coding tools it finds, then **starts Vexen once and checks that it answers**. It prints what it did and what it could not do. Nothing runs in the background.

**Then:** restart your coding tool and ask:

```text
Use Vexen to check this project's README.md for past failures or decisions.
```

On a new project, **“no relevant evidence” is the expected answer**. When a test fails during your work, ask your agent to save the failed attempt with its file and test result. If your agent can't find Vexen, run `npx @onurbyte/vexen doctor --mcp`.

[Full setup guide, manual steps and uninstall →](docs/getting-started.md) · [Download without Node.js →](https://github.com/OnurByte/Vexen/releases/latest)

## Works across coding tools

Start a task in Claude Code. Continue in Codex or Cursor. **Saved lessons stay with the project.**

<p align="center"><img src="docs/assets/clients.svg" alt="Coding tools with Vexen setup or runtime test coverage, including Claude Code, Codex, Cursor, Gemini CLI and VS Code. Setup steps and test coverage differ by tool." /></p>

The installer supports eight tools; more have completed real Vexen tool calls. Setup and coverage vary. [See what was tested →](docs/client-smoke-tests.md)

## Your project. Your memory.

- **Local storage.** Notes live in your project's `.vexen/` folder. Database files are ignored by Git.
- **No Vexen cloud or telemetry.** Vexen doesn't record your chats. Your coding tool's own data policies still apply.
- **A source for each reminder.** See the file, commit and recorded result behind a note.
- **Nothing relevant? It says so.** Old or unrelated evidence should not become advice for new code.

Version 1.0: the command line, the four MCP tools and the data format are stable and follow [semantic versioning](CHANGELOG.md#versioning). Vexen gives your agent context to consider; the agent still needs to inspect the current code and test its work. What 1.0 does not claim is in the [roadmap](docs/roadmap.md#what-10-means).

[Privacy](docs/security.md) · [Documentation](docs/README.md) · [Compatibility](docs/client-smoke-tests.md) · [Roadmap](docs/roadmap.md) · [Report an issue](https://github.com/OnurByte/Vexen/issues)

GPL-3.0-or-later · [License](LICENSE). Independent; not affiliated with Vercel. [Geist fonts](https://github.com/vercel/geist-font) are used under the SIL Open Font License. Product and platform marks identify compatibility and are not endorsements.
