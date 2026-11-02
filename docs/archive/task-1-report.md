> **Historical document, kept for provenance.** It records a plan, a milestone or a snapshot from earlier in the project and may no longer match the code. For the current state see the [status and roadmap](../roadmap.md).

# Task 1 report: Git history and co-change

**Status: DONE_WITH_CONCERNS.** The Task 1 implementation, fixture suite and frozen evaluation are complete. The predictive gate for default co-change hints failed: thresholded co-change had lower Hit@3 than plain historical co-change counts in all ten repositories. Keep co-change research-only with no default hints and make no release efficacy claim. Per the user's direction, continue the remaining product scope independently.

## Delivered

- `analyze_history(repo_root, limit)` reads the reachable, non-merge history in one batched Git invocation and returns eligible change transactions in chronological order. Git is invoked with argument arrays; paths and commit messages are never interpolated into a shell command.
- `RepoPath` rejects absolute paths, traversal components, empty components and Windows drive prefixes. Git's NUL-delimited records preserve paths containing newlines and record-separator bytes. A transaction containing a non-UTF-8 path is skipped whole, counted, and exposed through `HistoryIndex::skipped_non_utf8_transactions()`; this prevents silently creating false file pairs.
- Unix backslashes remain literal filename characters and are not conflated with `/` path separators. Git rename records preserve file identity through eligible transactions. Path reuse after a rename or deletion marks the affected rename component ambiguous; all eligible transactions that touched the component are excluded and counted by `HistoryIndex::skipped_ambiguous_identity_transactions()`. Rename cycles use a stable component representative. Public hints resolve to the current live name, and deleted candidates abstain.
- Each transaction carries its committer timestamp. History is stably sorted by timestamp, retaining Git's topological order for equal timestamps. Prefix projections rebuild identity state only from their own events. The benchmark groups equal timestamps and gives each held-out transaction a training index containing commits with strictly earlier committer timestamps; it iterates raw eligible transactions and does not use the final full-history projection to choose test membership.
- Merge commits are excluded, transactions with more than 20 paths and transactions touching generated/vendor/build output are filtered. Shallow history is limited to what Git exposes. Empty repositories and empty commits are supported.
- `recommend_related` reports support, confidence, lift and up to three example commit IDs. It abstains for fewer than three eligible transactions, support below two, confidence below 0.2, or lift at/below 1.0. These fixed conservative rules are uncalibrated association thresholds, not probabilities.
- `cargo bench --bench cochange -- <bare-repository> 0.8 --per-query <output.jsonl>` evaluates a chronological 80/20 target split by committer time. The first `ceil(half)` lexicographic paths in each held-out transaction are inputs and the rest are targets. It compares thresholded co-change, plain co-change counts, and a basename/same-parent ranking. Summary output is machine-readable JSON with all JSON control characters escaped; optional JSONL provides each commit, its timestamp cutoff, query, truth set and each method's predictions.

The metrics define a hit as retrieval of a file that changed in the held-out commit; this does not show that the file needed to change or that a hint would help an agent. `precision_at_3` divides correct emitted hints by three slots per query; `emitted_precision` divides by actual emitted hints. `false_advisory_rate` is non-target emitted hints divided by all emitted hints. Coverage is the fraction of queries receiving at least one hint.

## Test evidence

Test-first evidence: before implementations existed, `cargo test --test git` failed to compile because `src/git/{history,changes,identity}.rs` and `src/analysis/cochange.rs` were missing. After adding the non-UTF-8 behavior regression first, its test also failed to compile because `HistoryIndex::skipped_non_utf8_transactions` did not yet exist.

Round 1 RED command: `cargo test --test git rename_cycle_has_one_stable_canonical_identity`. It failed to compile with `no method named skipped_ambiguous_identity_transactions`, `no field committer_timestamp on ChangeTransaction`, and `no method named prefix_before`; these APIs and the associated behaviors were added afterward.

Passing evidence after implementation:

```text
RUSTUP_TOOLCHAIN=1.99.1 cargo fmt -- --check                 passed
RUSTUP_TOOLCHAIN=1.99.1 cargo clippy --all-targets -- -D warnings  passed
cargo test --test git                                        12 passed, 0 failed
cargo test --lib                                             1 passed, 0 failed
```

The command run for this green check was `RUSTUP_TOOLCHAIN=1.99.1 cargo fmt && RUSTUP_TOOLCHAIN=1.99.1 cargo fmt -- --check && RUSTUP_TOOLCHAIN=1.99.1 cargo clippy --all-targets -- -D warnings && cargo test --test git && cargo test --lib`.

The twelve integration fixtures cover ordinary commits and evidence, rename chains and cycles, recreated paths, current rename targets and deletion abstention, merge exclusion, generated and >20-file mass edits, empty history, shallow history, sibling commit times and strict cutoff grouping, control-separator paths, empty commits, whole-transaction abstention/counting for non-UTF-8 paths and prefix skip counts, and literal Unix backslashes. The library unit test verifies escaping all 32 JSON control code points.

## Benchmark status and limitations

The corrected evaluation used the ten frozen sources and revision `53f7b5af14f9c15bd15b0dd84845645585bde229`; portable aggregate and per-query outputs are under `benchmarks/cochange/results/`. Across 1,874 queries, thresholded co-change Hit@3 was 0.3511 versus 0.4274 for plain counts, and coverage was 0.7327 versus 0.9856. Thresholded emitted precision was higher (0.2438 vs 0.1624) and false-advisory rate lower (0.7562 vs 0.8376), but Hit@3 was lower in every repository, so the default-hint gate failed. See [the complete results](../benchmarks.md).

The runner initially failed because Git rejects `usize::MAX` as `--max-count`; the history limit is now bounded to Git's supported integer range. Corpus analysis encountered non-UTF-8 paths; the implementation skips/counts whole affected transactions rather than failing the run or dropping only the offending path.

The benchmark uses one committer-time chronological split and fixed thresholds, without duplicate-patch purge windows, validation-based tuning, confidence intervals, multi-seed sensitivity, rename-only slice reporting, or human/task relevance labels. Repositories are not independent experiments, and a co-change target is not a necessity label. Synthetic fixtures establish mechanics only. The observed file retrieval metrics do not measure agent utility; matched agent outcome evaluation remains future work.
