use crate::git::{history::HistoryIndex, identity::RepoPath};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub struct CochangeHint {
    pub path: RepoPath,
    pub support: usize,
    pub confidence: f64,
    pub lift: f64,
    pub example_commits: Vec<String>,
}

fn rankings(index: &HistoryIndex, inputs: &[RepoPath]) -> Vec<CochangeHint> {
    let queries: BTreeSet<_> = inputs.iter().map(|p| index.canonical(p)).collect();
    if queries.is_empty() {
        return Vec::new();
    }
    let mut input_tx = 0;
    let mut total = 0;
    let mut counts: BTreeMap<RepoPath, usize> = BTreeMap::new();
    let mut examples: BTreeMap<RepoPath, Vec<String>> = BTreeMap::new();
    for tx in &index.transactions {
        total += 1;
        let canonical: BTreeSet<_> = tx.paths.iter().map(|p| index.canonical(p)).collect();
        if queries.iter().any(|q| canonical.contains(q)) {
            input_tx += 1;
            for p in canonical.difference(&queries) {
                *counts.entry(p.clone()).or_default() += 1;
                let e = examples.entry(p.clone()).or_default();
                if e.len() < 3 {
                    e.push(tx.commit.clone());
                }
            }
        }
    }
    if total == 0 || input_tx == 0 {
        return Vec::new();
    }
    if total < 3 {
        return Vec::new();
    }
    let mut hints: Vec<_> = counts
        .into_iter()
        .filter_map(|(path, support)| {
            let confidence = support as f64 / input_tx as f64;
            let prevalence = index
                .transactions
                .iter()
                .filter(|t| {
                    t.paths
                        .iter()
                        .map(|p| index.canonical(p))
                        .any(|p| p == path)
                })
                .count() as f64
                / total as f64;
            let lift = if prevalence > 0.0 {
                confidence / prevalence
            } else {
                0.0
            };
            // Explicitly abstain when evidence is not stronger than chance.
            if support < 2 || confidence < 0.2 || lift <= 1.0 {
                return None;
            }
            let example_commits = examples.remove(&path).unwrap_or_default();
            Some(CochangeHint {
                path,
                support,
                confidence,
                lift,
                example_commits,
            })
        })
        .collect();
    hints.sort_by(|a, b| {
        b.support
            .cmp(&a.support)
            .then_with(|| b.lift.total_cmp(&a.lift))
            .then_with(|| a.path.cmp(&b.path))
    });
    hints
}

pub fn recommend_related(
    index: &HistoryIndex,
    paths: &[RepoPath],
    max_results: usize,
) -> Vec<CochangeHint> {
    if max_results == 0 {
        return Vec::new();
    }
    rankings(index, paths)
        .into_iter()
        .take(max_results.min(3))
        .collect()
}
