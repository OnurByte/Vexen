> **Historical document, kept for provenance.** It records a plan, a milestone or a snapshot from earlier in the project and may no longer match the code. For the current state see the [status and roadmap](../roadmap.md).

# Frozen change-impact evaluation plan

Selected before implementation results: ripgrep, fd, bat, jq, just, fzf, delta, hyperfine, tokei, zoxide. These public CLI repositories span Rust, C and Go and different maintainers, but overrepresent CLI projects and Rust; results cannot establish broad ecosystem generalization.

Use reachable ordered history, a chronological 80/20 split and no future commits. For each eligible held-out transaction with at least two files, use the lexicographically first half as query and remaining files as targets. Evaluate maximum three suggestions from the trained model. Report hit@3, precision@3, coverage, false-advisory rate and latency; compare basename/proximity and unweighted historical cochange counts on exactly the same queries. Freeze source HEAD hashes in raw output. Keep empty/merge/large/shallow history cases in fixtures; report any omitted repo or unsupported path.

Gate: do not claim useful superiority from synthetic data or from beating only the weakest baseline. If measured hints fail to materially beat simple Git cochange, disable default cochange and revisit scope before tasks 2–9, as the supplied specification requires. No agent-outcome improvement claim without a paired outcome study.
