# Task 1 report: Git history and co-change

**Status: DONE_WITH_CONCERNS.** The Task 1 implementation and fixture suite are committed. The predictive gate is unresolved until the current chronological run over the frozen public repositories completes. This report does not claim predictive success or authorize later tasks.

## Delivered

- `analyze_history(repo_root, limit)` reads the reachable, non-merge history in one batched Git invocation and returns eligible change transactions in chronological order. Git is invoked with argument arrays; paths and commit messages are never interpolated into a shell command.
- `RepoPath` rejects absolute paths, traversal components, empty components and Windows drive prefixes. Git's NUL-delimited records preserve paths containing newlines and record-separator bytes. A transaction containing a non-UTF-8 path is skipped whole, counted, and exposed through `HistoryIndex::skipped_non_utf8_transactions()`; this prevents silently creating false file pairs.
- Git rename records preserve file identity through eligible transactions. Prefix indexes rebuild aliases from only their own earlier transactions, so the chronological benchmark does not apply future rename knowledge to training queries. Renames in filtered transactions do not affect aliases.
- Merge commits are excluded, transactions with more than 20 paths and transactions touching generated/vendor/build output are filtered. Shallow history is limited to what Git exposes. Empty repositories and empty commits are supported.
- `recommend_related` reports support, confidence, lift and up to three example commit IDs. It abstains for fewer than three eligible transactions, support below two, confidence below 0.2, or lift at/below 1.0. These fixed conservative rules are uncalibrated association thresholds, not probabilities.
- `cargo bench --bench cochange -- <bare-repository> 0.8 --per-query <output.jsonl>` evaluates a chronological 80/20 split. The first `ceil(half)` lexicographic paths in each held-out transaction are inputs and the rest are targets. It compares thresholded co-change, plain co-change counts, and a basename/same-parent ranking. Summary output is machine-readable JSON; optional JSONL provides each commit, query, truth set and each method's predictions.

The metrics define a hit as retrieval of a file that changed in the held-out commit; this does not show that the file needed to change or that a hint would help an agent. `precision_at_3` divides correct emitted hints by three slots per query; `emitted_precision` divides by actual emitted hints. `false_advisory_rate` is non-target emitted hints divided by all emitted hints. Coverage is the fraction of queries receiving at least one hint.

## Test evidence

Test-first evidence: before implementations existed, `cargo test --test git` failed to compile because `src/git/{history,changes,identity}.rs` and `src/analysis/cochange.rs` were missing. After adding the non-UTF-8 behavior regression first, its test also failed to compile because `HistoryIndex::skipped_non_utf8_transactions` did not yet exist. Both failures preceded their corresponding implementations.

Passing evidence after implementation:

```text
RUSTUP_TOOLCHAIN=1.99.1 cargo fmt -- --check     passed
RUSTUP_TOOLCHAIN=1.99.1 cargo clippy --all-targets -- -D warnings  passed
cargo test --test git                            7 passed, 0 failed
```

The seven fixture tests cover ordinary commits and evidence, renames, merge exclusion, generated and >20-file mass edits, empty history, shallow history, control-separator paths, empty commits, and explicit whole-transaction abstention/counting for non-UTF-8 paths.

## Benchmark status and limitations

The frozen ten-repository corpus run is being performed by the parent agent. Results were not available when this report was committed; therefore this report has no predictive accuracy results and no comparison-based gate decision. The runner itself had an initial invocation failure because Git rejects `usize::MAX` as `--max-count`; the history limit is now bounded to Git's supported integer range. A corpus run also exposed a non-UTF-8 path. The implementation now skips/counts the entire affected transaction and tests that behavior rather than failing the full corpus or dropping only the offending path.

The benchmark uses one chronological split and fixed thresholds, without duplicate-patch purge windows, validation-based tuning, confidence intervals, multi-seed sensitivity, rename-only slice reporting, or human/task relevance labels. Repositories are not independent experiments, and a co-change target is not a necessity label. Synthetic fixtures establish mechanics only. Do not ship default hints or claim agent improvement unless the frozen real-repository results materially beat both simple baselines with acceptable coverage and false-advisory rate; matched agent outcome evaluation remains future work.
