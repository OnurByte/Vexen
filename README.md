# Vexen

Vexen aims to help coding agents learn from repository history and recorded outcomes. Product development is in progress. **This README documents the Task 1 Git-history research slice and its evaluation harness.** That slice is not the complete memory product and does not provide CLI, MCP, client installation, or agent integrations.

## Current findings

The Task 1 evaluation covered ten frozen public repositories. Thresholded co-change had lower Hit@3 than plain historical co-change counts on all ten. It therefore remains research-only; this code does not ship default change-impact hints. The evaluation is a file-retrieval proxy and does not show that memory improves coding-agent task success.

See [the benchmark report](docs/benchmarks.md) for per-repository and aggregate results, method, limitations and reproduction details. Frozen repository sources and Git HEADs are in [evaluation-inputs.json](docs/evaluation-inputs.json).

## Try the research slice

```sh
cargo test --all-targets
```

Analyze one locally available bare repository and emit machine-readable metrics:

```sh
cargo bench --bench cochange -- /path/to/repository.git 0.8 --per-query /tmp/queries.jsonl
```

Reproduce the frozen ten-repository evaluation using local bare repositories named after the inputs (for example `bat.git`, `fd.git`, `ripgrep.git`):

```sh
bash benchmarks/cochange/run-corpus.sh /path/to/evaluation /tmp/vexen-results
```

The corpus runner checks each expected Git HEAD and does not download repositories. The standalone benchmark accepts any local Git repository. When invoked without a repository, it reports that no benchmark ran and exits successfully so standard Cargo all-target checks work.

## Privacy and compatibility

The library reads local Git history through the Git executable. It does not collect prompts, transcripts, command output, telemetry or API credentials. The corpus script operates on repositories already present on disk; it does not download them. Git's own repository configuration remains in effect.

This research slice requires Rust 2024 edition and a local Git executable. No coding-agent harness or operating-system compatibility claim is made because the CLI, MCP and installer layers are not implemented here.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
