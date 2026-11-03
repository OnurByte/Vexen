<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="icon.svg">
    <img src="icon-light.svg" width="84" height="78" alt="Vexen" />
  </picture>
</p>

<h1 align="center">Vexen</h1>

<p align="center"><strong>Your AI agent forgot what failed. Vexen didn't.</strong></p>
<p align="center">Stop losing lessons between coding sessions.<br/>Save failed fixes and important decisions so your next agent can check them before trying again.</p>

<p align="center">
  <img alt="Free and open source" src="https://img.shields.io/badge/free-open%20source-78a9ff?labelColor=262626" />
  <img alt="Private and local" src="https://img.shields.io/badge/private-local-78a9ff?labelColor=262626" />
  <img alt="No account required" src="https://img.shields.io/badge/account-not%20required-78a9ff?labelColor=262626" />
  <img alt="Version 1.0" src="https://img.shields.io/badge/version-1.0-78a9ff?labelColor=262626" />
</p>

<p align="center"><a href="#get-started"><strong>Get started →</strong></a> · <a href="#does-it-help">See the evidence</a> · <a href="docs/getting-started.md">Setup help</a></p>

Vexen is a shared memory for AI coding tools such as **Claude Code, Codex and Cursor**. It keeps notes with your project, so switching tools or opening a new chat doesn't have to mean explaining the same failed fix again.

## Here's what changes

<p align="center"><img src="docs/assets/readme-before-after.svg" alt="Illustrative scenario. Without Vexen: Claude tries a fix on Monday, the test fails, and a fresh Codex chat on Friday may try it again. With Vexen: the failure and test result are saved, so Codex can check them and decide whether the lesson still applies." /></p>

**You don't need to save an entire conversation.** Your agent saves a short note about what failed, along with its test result, or a decision you've accepted. A later session can check those notes before editing the same code.

*Illustrative scenario. Notes need to be recorded; Vexen does not automatically remember your chats.*

<p align="center"><img src="docs/assets/readme-benefits.svg" alt="Keep the lesson: failed fixes and key decisions for the next session. See the proof: the file and test result behind a reminder. Keep it local: on your machine, with no account, cloud or telemetry." /></p>

## Get started

Open a terminal **inside the Git project you want Vexen to remember**. You'll need [Node.js 18 or newer](https://nodejs.org/) (only to run the one-time installer) and Git.

**1. One command.**

macOS / Linux:

```sh
npx @onurbyte/vexen setup
```

Windows PowerShell:

```powershell
npx.cmd @onurbyte/vexen setup
```

`setup` does everything for you: it creates your project's memory (a local `.vexen/` folder), puts a permanent copy of Vexen on your machine, connects the coding tools it finds (Claude Code, Codex, Cursor and others), adds the optional skill (Claude Code), then **starts Vexen once and checks that it answers**. It prints what it did and what it could not do. Nothing runs in the background. After setup, Node.js is no longer needed.

Want to see it work before touching your own project? `npx @onurbyte/vexen demo` runs the whole loop on a throwaway synthetic repository and deletes it afterwards.

**2. Restart your coding tool, then try it in the chat.**

```text
Use Vexen to check this project's README.md for past failures or decisions.
```

On a new project, **“no relevant evidence” is the expected answer**. Your memory starts empty and grows as lessons are saved. If your agent can't find Vexen, run `npx @onurbyte/vexen doctor --mcp` or follow the [connection guide](docs/getting-started.md#2-connect-your-agents).

**3. Save a useful lesson when one happens.**

After a test fails during your work, ask your agent:

```text
Save this failed attempt in Vexen with the affected file and the actual
failing test result, so a future session can check what happened.
```

Vexen only brings back failed attempts with a recorded failing test result, or decisions you've accepted. A note by itself is not a verified failure. If the file changes, an old note stops appearing.

[Full setup guide, manual steps and uninstall →](docs/getting-started.md) · [Download without Node.js →](https://github.com/OnurByte/Vexen/releases/latest)

## Works across coding tools

Start a task in Claude Code. Continue in Codex or Cursor. **Saved lessons stay with the project.** Each tool needs to be connected to Vexen, and your agent needs to check for relevant notes.

<p align="center"><img src="docs/assets/clients.svg" alt="Coding tools with Vexen setup or runtime test coverage, including Claude Code, Codex, Cursor, Gemini CLI and VS Code. Setup steps and test coverage differ by tool." /></p>

The installer supports eight tools. Additional clients have been tested with real Vexen tool calls; setup and coverage vary. [Find your tool and see what was tested →](docs/client-smoke-tests.md)

## Does it help?

<p align="center"><img src="docs/assets/readme-evidence.svg" alt="Small internal coding pilot: 11 successful runs out of 20 with relevant Vexen memory, versus 2 out of 20 without memory. Five synthetic tasks, one model, pre-recorded evidence and explicit instructions to check memory. Not a general coding performance claim." /></p>

In our controlled pilot, Claude Code using Haiku solved **11 of 20 targeted task runs with relevant Vexen memory**, compared with **2 of 20 without memory**.

This was a small internal study using five synthetic tasks, pre-recorded evidence and explicit instructions to check memory. It does **not** establish general coding improvement.

[See the method, the full 76-run pilot (this comparison uses 40 of its runs) and limitations →](docs/pilot-results.md) · [Inspect the raw data →](benchmarks/memory/results/pilot-2.jsonl)

**Does it save money?** Not by making runs shorter: a run cost about the same. It saves by making more runs succeed, so each solved task costs less.

<p align="center"><img src="docs/assets/readme-cost.svg" alt="Small internal pilot with one model: a solved task cost about 4.8 cents without memory and 1.1 cents with relevant Vexen memory. A run cost about the same either way; more runs succeeded with memory. One check adds at most 1,800 bytes, roughly 450 tokens. Twenty runs per group, synthetic tasks, not a general cost-saving claim." /></p>

Cost comes from the [instrumented run](benchmarks/memory/results/pilot-3.jsonl) of the same pilot. [How it was computed, and the limits →](docs/research.md#what-a-solved-task-cost)

**Useful memory also needs to know when to stay quiet.** We checked what happened when a saved note was out of date or about a different file.

<p align="center"><img src="docs/assets/readme-relevance.svg" alt="Relevance checks in the same pilot: all 20 stale-memory runs returned no relevant evidence. On two control tasks with unrelated notes, all eight runs passed with Vexen and all eight passed without it. These small synthetic checks do not establish general safety or performance." /></p>

The stale-note check measured whether an old record was withheld; those runs solved 0 of 20 tasks. The control result means no drop was observed on those two tasks. Neither is a guarantee for other projects. [Read the relevance checks →](docs/research.md#relevance-checks)

**Tested connections. Published misses.** We also test whether coding tools can actually retrieve a saved note, and publish experiments that did not earn a place in the default product.

<p align="center"><img src="docs/assets/readme-open-tests.svg" alt="Compatibility study: 13 additional installed clients completed a Vexen read call on Linux, mostly using deterministic model drivers. Separate file-hint study: across 1,874 queries in ten repositories, the experimental method lost to simple history counts in every repository and remains research-only. Neither study measures coding improvement." /></p>

[See the 13 client checks →](docs/client-smoke-tests.md) · [Read the unsuccessful file-hint experiment →](docs/benchmarks.md)

<details>
<summary><strong>What did the coding tasks actually test?</strong></summary>

These were small Python tasks with requirements hidden from the visible tests. The saved notes described earlier failures involving those requirements. Each row below includes all four repetitions per condition, rather than a selected successful run.

| Task | What the saved lesson explained | Without memory | Vexen enabled |
|---|---|---:|---:|
| Read an exported file | A special UTF-8 file marker needs to be handled | 1 / 4 | 3 / 4 |
| Calculate a price | `0.125` must round to `0.13` | 1 / 4 | 2 / 4 |
| Build a URL slug | `München` must become `muenchen` | 0 / 4 | 2 / 4 |
| Parse an export date | `04/03/2025` means 4 March | 0 / 4 | 2 / 4 |
| Read exported rows | Accept semicolons as well as commas | 0 / 4 | 2 / 4 |

These examples show the kind of project-specific lesson Vexen can carry. They are synthetic test cases, not customer stories. The notes were prepared in advance and the agent was told to check memory. In the Vexen-enabled group, all 20 runs called the tool; the last logged response contained a failure record in 12 and no relevant evidence in eight. Even with Vexen enabled, nine runs failed.

[Task definitions](benchmarks/memory/tasks.py) · [Additional tasks](benchmarks/memory/tasks_extra.py) · [Full results](docs/pilot-results.md)

</details>

[Explore the research, evidence and limits →](docs/research.md)

## Your project. Your memory.

- **Local storage.** Notes live in your project's `.vexen/` folder. Database files are ignored by Git.
- **No Vexen cloud or telemetry.** Vexen doesn't record your chats. Your coding tool's own data policies still apply.
- **A source for each reminder.** Check the file, commit and recorded result behind a note.
- **Nothing relevant? It says so.** Old or unrelated evidence should not become advice for new code.

Version 1.0: the command line, the four MCP tools and the data format are stable, and changes to them follow [semantic versioning](CHANGELOG.md#versioning). Vexen gives your agent context to consider; the agent still needs to inspect the current code and test its work. What 1.0 does not claim is listed in the [roadmap](docs/roadmap.md#what-10-means).

[Privacy](docs/security.md) · [Documentation](docs/README.md) · [Compatibility](docs/client-smoke-tests.md) · [Roadmap](docs/roadmap.md) · [Report an issue](https://github.com/OnurByte/Vexen/issues)

GPL-3.0-or-later · [License](LICENSE). Independent; not affiliated with Vercel. [Geist fonts](https://github.com/vercel/geist-font) are used under the SIL Open Font License.
