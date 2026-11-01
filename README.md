<p align="center">
  <img src="icon.svg" width="84" height="84" alt="Vexen mark: two cyan and violet strokes meet in a forward-pointing V." />
</p>

<h1 align="center">Vexen — give your next coding agent a head start</h1>

<p align="center">Local, evidence-based memory for coding agents working in the same repository.</p>

<p align="center"><img src="docs/assets/hero.svg" alt="A repository sends a small, cited memory forward to the next coding session. The memory is local, and the agent remains in control." /></p>

Vexen is being built to help a coding agent carry useful repository context into the next session. It focuses on two things: what a past attempt actually showed, and where the supporting evidence came from. It is designed to work locally across coding-agent harnesses.

## What exists today

This is an early development project, not a ready-to-install tool. The current repository contains a Rust Git-history research library and evaluation harness. It does **not** yet include a usable Vexen command, MCP server, client installer, or working cross-session memory workflow.

The first change-impact experiment did not beat plain historical co-change counts on any of ten evaluated repositories. Change-impact suggestions therefore remain research-only. The results are a file-retrieval proxy, not evidence that Vexen improves coding-agent task success. See the [evaluation and its limitations](docs/benchmarks.md).

## The idea

<p align="center"><img src="docs/assets/workflow.svg" alt="Three steps: a coding agent records a failed approach with its test outcome; Vexen keeps a local evidence note; a later session receives a short reminder with its source and applicability. If the evidence does not fit, Vexen can return nothing." /></p>

A useful reminder should say what happened, point to its source, and make clear when it may no longer apply. A test result is evidence about one code state; it is not a guarantee. Vexen should be able to say “no relevant evidence.”

## See the shape of a reminder

<p align="center"><img src="docs/assets/example.svg" alt="Illustrative, not measured: a sample reminder says a retry change once duplicated a charge, cites a hypothetical test and commit, and warns that the evidence may be stale." /></p>

The example is fictional and illustrates the planned format. It is not a result from a Vexen run.

## Explore the research slice

Requires Rust 2024 and Git. Run the available tests:

```sh
cargo test --all-targets
```

Run the co-change benchmark against a **local** Git repository:

```sh
cargo bench --bench cochange -- /path/to/repository.git 0.8 --per-query /tmp/queries.jsonl
```

The benchmark reads local history; the frozen-corpus runner also requires repositories already downloaded. See [reproduction details](docs/benchmarks.md).

## Installation status

There is no installable Vexen binary yet. After the CLI is implemented and the repository is published at the intended address, the planned source install is:

```sh
cargo install --git https://github.com/OnurByte/Vexen --locked
```

This command is a plan, not a working installation today. No crates.io package or release binary is claimed.

## Privacy and scope

The research slice reads local Git history. It does not collect prompts, transcripts, command output, telemetry, or API credentials. Planned Vexen memory is local-first; its storage and security behavior still need implementation and validation. Vexen is not an agent, permission gate, or guarantee against regressions.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
