# Vexen Task 1 change-impact evaluation

## Result and decision

The frozen ten-repository evaluation did **not** meet the gate for shipping default co-change hints. Thresholded co-change had lower Hit@3 than plain historical co-change counts in all ten repositories. Across 1,874 evaluated queries, it reached Hit@3 0.3511 versus 0.4274 for plain counts, while coverage fell from 0.9856 to 0.7327. Its emitted precision was higher and its false-advisory rate lower, but the gate requires useful superiority over the simple co-change baseline, not a precision/abstention tradeoff alone.

Co-change remains a research-only Task 1 slice. No default hints are enabled, and these results do not establish agent-task improvement. The user has directed continuation of other product scope despite this gate result; this evaluation does not authorize a release efficacy claim.

## Per-repository results

Hit@3, precision@3, coverage and false-advisory rate are for thresholded co-change unless the Hit@3 column says `thresholded / plain`. Precision@3 divides correct emitted suggestions by three slots per evaluated query. Skips are whole transactions excluded for non-UTF-8 paths or ambiguous rename/path reuse. Latency is the benchmark runner's own elapsed analysis/evaluation time.

| Repository | Hit@3 thresholded / plain | Precision@3 | Coverage | False-advisory rate | Skipped non-UTF-8 / ambiguous | Latency |
|---|---:|---:|---:|---:|---:|---:|
| bat | 0.3209 / 0.4590 | 0.1070 | 0.4813 | 0.5057 | 2 / 122 | 13.481 s |
| delta | 0.3947 / 0.4526 | 0.1316 | 0.7158 | 0.7170 | 0 / 43 | 3.142 s |
| fd | 0.6667 / 0.7361 | 0.2269 | 0.8333 | 0.3951 | 0 / 38 | 0.974 s |
| fzf | 0.4601 / 0.4867 | 0.1729 | 0.9362 | 0.7766 | 0 / 8 | 10.448 s |
| hyperfine | 0.4000 / 0.4960 | 0.1387 | 0.7200 | 0.7536 | 0 / 4 | 0.899 s |
| jq | 0.4588 / 0.5773 | 0.1667 | 0.8093 | 0.6560 | 0 / 24 | 2.413 s |
| just | 0.1604 / 0.1651 | 0.0582 | 0.9057 | 0.9311 | 0 / 828 | 31.351 s |
| ripgrep | 0.1659 / 0.2915 | 0.0553 | 0.6457 | 0.8384 | 0 / 49 | 22.404 s |
| tokei | 0.0330 / 0.1319 | 0.0110 | 0.0879 | 0.8636 | 0 / 327 | 9.867 s |
| zoxide | 0.2941 / 0.3333 | 0.0980 | 0.8824 | 0.8438 | 0 / 60 | 17.328 s |

## Micro-aggregates

| Method | Hit@3 | Precision@3 | Emitted precision | Coverage | False-advisory rate | Queries | Hints emitted |
|---|---:|---:|---:|---:|---:|---:|---:|
| Thresholded co-change | 0.3511 | 0.1236 | 0.2438 | 0.7327 | 0.7562 | 1,874 | 2,851 |
| Plain co-change count | 0.4274 | 0.1556 | 0.1624 | 0.9856 | 0.8376 | 1,874 | 5,389 |
| Basename/proximity | 0.0496 | 0.0192 | 0.0289 | 0.9338 | 0.9711 | 1,874 | 3,736 |

Hit@3 was lower for thresholded co-change than plain counts on every repository, and the micro-aggregate difference is −0.0763. Thresholding emitted 2,538 fewer hints and reduced false-advisory rate by 0.0814, but that restraint did not recover predictive coverage or hit rate.

## Method

The input set was frozen before implementation in [evaluation-inputs.json](evaluation-inputs.json): ten public repositories at fixed Git HEADs, mostly CLI projects and disproportionately Rust. The measured implementation revision was `53f7b5af14f9c15bd15b0dd84845645585bde229`. The runner used an 80% target split by committer timestamp; equal timestamps were kept on the same side. For each held-out transaction, the model used only transactions with strictly earlier committer timestamps. It queried with the lexicographically first `ceil(path_count / 2)` changed paths and predicted the rest. Merge, generated/vendor/build and over-20-file transactions were filtered. Whole transactions with non-UTF-8 paths or ambiguous path identity were counted and skipped.

The baselines used the same evaluated queries. Plain co-change ranked prior co-commit counts; basename/proximity ranked files with the same basename or parent directory. Suggestions were capped at three. A hit means an omitted path changed in that held-out commit; it does not mean that path was required or that a hint would improve a coding task. `false_advisory_rate` is non-target emitted hints divided by all emitted hints. This is a file retrieval proxy, not a human relevance measure.

The experiment has one split per repository, no validation-based threshold tuning, no duplicate-patch purge window, no confidence intervals, and no human or paired-agent outcome study. Committer timestamps are Git metadata and can be skewed. The repositories are not independent samples and overrepresent public CLI projects. Reported latency is measured on one machine and is not a performance guarantee. No success uplift or generalization claim follows.

## Reproduction and artifacts

The frozen machine-readable outputs contain only portable repository labels, source URLs and source HEADs. Per-query files include inputs, targets, all baseline predictions and per-query training cutoffs.

- [`results.json`](../benchmarks/cochange/results/results.json) and [`results.jsonl`](../benchmarks/cochange/results/results.jsonl): normalized aggregate runner output.
- [`queries/`](../benchmarks/cochange/results/queries/): one portable JSONL file per repository.
- [`run-corpus.sh`](../benchmarks/cochange/run-corpus.sh): verifies the frozen HEADs and runs against local bare repositories; it does not download repositories.

With those bare repositories under `/path/to/evaluation`:

```sh
bash benchmarks/cochange/run-corpus.sh /path/to/evaluation /tmp/vexen-cochange-results
```

For one repository:

```sh
cargo bench --bench cochange -- /path/to/evaluation/bat.git 0.8 --per-query /tmp/bat-queries.jsonl
```
