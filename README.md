<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="icon.svg">
    <img src="icon-light.svg" width="84" height="78" alt="Vexen mark: two white and two blue paths converge into a forward-pointing V." />
  </picture>
</p>

<h1 align="center">Vexen</h1>

<p align="center"><strong>Your agent forgets. Your project shouldn’t.</strong><br/>Keep the lessons from one coding session. Give the next one a head start.</p>

<p align="center">
  <img alt="Status: early development" src="https://img.shields.io/badge/status-early%20development-78a9ff?labelColor=262626" />
  <img alt="Local first" src="https://img.shields.io/badge/local--first-no%20cloud-78a9ff?labelColor=262626" />
  <img alt="npm version" src="https://img.shields.io/npm/v/@onurbyte/vexen?color=78a9ff&labelColor=262626&label=npm" />
  <img alt="Written in Rust" src="https://img.shields.io/badge/built%20with-Rust-78a9ff?labelColor=262626" />
  <img alt="MCP server" src="https://img.shields.io/badge/speaks-MCP-78a9ff?labelColor=262626" />
  <img alt="License GPL-3.0-or-later" src="https://img.shields.io/badge/license-GPL--3.0--or--later-78a9ff?labelColor=262626" />
</p>

<p align="center"><a href="https://github.com/OnurByte/Vexen/releases/latest"><strong>Download Vexen →</strong></a><br/><sub><img src="docs/assets/icons/windows-11.svg" width="24" height="24" alt="Windows 11" /> Windows &nbsp; <img src="docs/assets/icons/macos.svg" width="24" height="24" alt="macOS" /> macOS &nbsp; <img src="docs/assets/icons/linux.svg" width="24" height="24" alt="Linux" /> Linux</sub></p>

<p align="center"><img src="docs/assets/hero.svg" alt="Vexen, local repository memory for coding agents: labeled flow from your repo to saved evidence to the next session." /></p>

Vexen gives your AI coding agents a shared, local memory of what you’ve tried, what failed, and why. Save a lesson once. Let your next agent check it before starting over.

**Free. Open source. No account required.**

[Get started](#get-started) · [Supported agents](#works-with-your-coding-setup) · [Docs](#learn-more)

## Stop starting from scratch

<p align="center"><img src="docs/assets/problem.svg" alt="Illustrative and fictional: in session one an agent retries a charge after a timeout and a test fails with a double charge. A week later a new agent tries the same thing, because it has no memory of the first failure." /></p>

A new chat shouldn’t mean losing everything you learned in the last one. Keep failed attempts and important decisions with your project, ready for the next session.

## Save the lesson. Keep the proof.

<p align="center"><img src="docs/assets/workflow.svg" alt="Three steps: record what failed, keep the proof, and recall it only when it fits current code. Vexen can stay quiet when evidence does not apply." /></p>

Your agent records an attempt and its result. Vexen keeps the source and checks whether it still fits your code before bringing it back.

<p align="center"><img src="docs/assets/example.svg" alt="ILLUSTRATIVE · FICTIONAL: a sample retry-timeout failure cites a mock source file, test, and commit, and says to check current code before relying on it." /></p>

*Illustrative example. When nothing applies, Vexen stays quiet.*

## Your project. Your memory.

<p align="center"><img src="docs/assets/principles.svg" alt="Four design principles: local first with no account, cloud or telemetry; every reminder cites its source; no relevant evidence is a valid answer; one small Rust and SQLite binary with nothing running in the background." /></p>

Everything stays on your machine. No cloud, no telemetry, no chat recording. You and your agent stay in control.

## Works with your coding setup

<p align="center"><img src="docs/assets/clients.svg" alt="Agent and IDE icons in Vexen blue: Claude Code, Codex, Cursor, Gemini CLI, VS Code, OpenCode, Windsurf, Kiro, GitHub Copilot, Cline, Zed, Hermes, Qwen Code, Continue, Goose, Kilo, Mistral Vibe and OpenHands. Setup and test coverage vary by client." /></p>

[Find your agent’s setup →](docs/getting-started.md#2-connect-your-agents) · [Compatibility & test results](docs/client-smoke-tests.md)

## Get started

Open a terminal in your Git project. Requires [Node.js 18+](https://nodejs.org/) and Git.

**<img src="docs/assets/icons/macos.svg" width="24" height="24" alt="macOS" /> macOS / <img src="docs/assets/icons/linux.svg" width="24" height="24" alt="Linux" /> Linux**

```sh
npx @onurbyte/vexen init
npx @onurbyte/vexen install --scope global --npx
```

**<img src="docs/assets/icons/windows-11.svg" width="24" height="24" alt="Windows 11" /> Windows · PowerShell**

```powershell
npx.cmd @onurbyte/vexen init
npx.cmd @onurbyte/vexen install --scope global --npx
```

Restart your coding agent. [Add the Vexen skill](docs/getting-started.md#3-use-it) so it knows when to save and check lessons.

[Download without Node.js →](https://github.com/OnurByte/Vexen/releases/latest)

<p align="center"><img src="docs/assets/terminal.svg" alt="Real terminal output from running vexen init, vexen doctor and vexen check on a fresh Git repository: the store is created, reports integrity ok with zero events, and check answers no relevant evidence." /></p>

A new project starts with an empty memory. It grows as you and your agent record what you learn.

[Setup help →](docs/getting-started.md)

## Built, tested, still growing

<p align="center"><img src="docs/assets/status.svg" alt="Working now: evidence store, MCP server, command line, installer for eight agents, test-result capture, npx launcher on npm, Git-history analysis for research. Next: live smoke tests with more clients, signed release binaries, Windows and macOS test fixtures. Later: paired agent evaluation, published negative results." /></p>

An early release with real tool calls verified in 13 additional clients. [See the results and test limits →](docs/client-smoke-tests.md)

## Learn more

[Privacy](docs/security.md) · [Agent compatibility](docs/client-smoke-tests.md) · [Evaluation results](docs/pilot-results.md) · [Roadmap](docs/specification.md)

GPL-3.0-or-later · [License](LICENSE). Independent; not affiliated with Vercel. [Geist fonts](https://github.com/vercel/geist-font) are used under the SIL Open Font License.
