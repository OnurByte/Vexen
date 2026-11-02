> **Historical document, kept for provenance.** It records a plan, a milestone or a snapshot from earlier in the project and may no longer match the code. For the current state see the [status and roadmap](../roadmap.md).

Files: src/git/{history,changes,identity}.rs, src/analysis/cochange.rs, benchmarks/cochange/, tests/git/.
Interfaces: analyze_history(repo_root: &Path, limit: usize) -> Result<HistoryIndex>; recommend_related(index: &HistoryIndex, paths: &[RepoPath], max_results: usize) -> Vec<CochangeHint>; CochangeHint has path, support, confidence, lift and example_commits.
- Write fixture tests for ordinary commits, renames, merges, mass edits, shallow repos and empty history.
- Run cargo test --test git and confirm tests fail for the unimplemented interfaces.
- Implement safe Git history parsing and change-transaction filtering.
- Implement file-pair counts and ranked associations with explicit no-result behavior.
- Build chronological benchmark with baseline comparison; report accuracy and false hints.
- Run cargo test --test git and the benchmark; commit only after evidence is captured.
Gate: If predictive value does not beat simple baselines, remove the default cochange feature and revisit positioning before implementing further scope.
