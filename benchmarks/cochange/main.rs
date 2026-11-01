use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
    process,
};
use vexen::{
    analysis::cochange::recommend_related,
    analysis::json::escape_json_string,
    git::{
        history::{HistoryIndex, analyze_history},
        identity::RepoPath,
    },
};

#[derive(Default)]
struct Score {
    hits: usize,
    emitted: usize,
    false_hints: usize,
    covered: usize,
    queries: usize,
}
impl Score {
    fn add(&mut self, preds: &[RepoPath], truth: &BTreeSet<RepoPath>) {
        self.queries += 1;
        if !preds.is_empty() {
            self.covered += 1;
        }
        self.emitted += preds.len();
        let mut hit = false;
        for p in preds {
            if truth.contains(p) {
                hit = true;
            } else {
                self.false_hints += 1;
            }
        }
        if hit {
            self.hits += 1;
        }
    }
    fn json(&self) -> String {
        let denom = self.emitted.max(1) as f64;
        format!(
            "{{\"hit_at_3\":{:.6},\"precision_at_3\":{:.6},\"emitted_precision\":{:.6},\"coverage\":{:.6},\"false_advisory_rate\":{:.6},\"queries\":{},\"emitted\":{},\"false_hints\":{}}}",
            self.hits as f64 / self.queries.max(1) as f64,
            (self.emitted - self.false_hints) as f64 / (self.queries.max(1) * 3) as f64,
            (self.emitted - self.false_hints) as f64 / denom,
            self.covered as f64 / self.queries.max(1) as f64,
            self.false_hints as f64 / denom,
            self.queries,
            self.emitted,
            self.false_hints
        )
    }
}

fn plain_counts(index: &HistoryIndex, queries: &[RepoPath]) -> Vec<RepoPath> {
    let query_set: BTreeSet<_> = queries.iter().map(|q| index.canonical(q)).collect();
    let mut counts: BTreeMap<RepoPath, usize> = BTreeMap::new();
    for tx in index.transactions() {
        let paths: BTreeSet<_> = tx.paths.iter().map(|p| index.canonical(p)).collect();
        if paths.iter().any(|p| query_set.contains(p)) {
            for p in paths.difference(&query_set) {
                *counts.entry(p.clone()).or_default() += 1;
            }
        }
    }
    let mut ranked: Vec<_> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(3)
        .filter_map(|x| index.display_path(&x.0))
        .collect()
}

fn basename_proximity(index: &HistoryIndex, queries: &[RepoPath]) -> Vec<RepoPath> {
    let query_set: BTreeSet<_> = queries.iter().map(|q| index.canonical(q)).collect();
    let universe: BTreeSet<_> = index
        .transactions()
        .iter()
        .flat_map(|t| t.paths.iter().map(|p| index.canonical(p)))
        .filter(|p| !query_set.contains(p))
        .collect();
    let mut ranked: Vec<_> = universe
        .into_iter()
        .map(|p| {
            let display_candidate = index.display_path(&p).unwrap_or_else(|| p.clone());
            let score = queries
                .iter()
                .map(|query| {
                    let q = index
                        .display_path(query)
                        .unwrap_or_else(|| index.canonical(query));
                    if display_candidate.basename() == q.basename() {
                        2
                    } else if display_candidate.parent() == q.parent() {
                        1
                    } else {
                        0
                    }
                })
                .max()
                .unwrap_or(0);
            (p, score)
        })
        .filter(|(_, s)| *s > 0)
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(3)
        .filter_map(|x| index.display_path(&x.0))
        .collect()
}
fn json_paths(paths: &[RepoPath]) -> String {
    format!(
        "[{}]",
        paths
            .iter()
            .map(|p| format!("\"{}\"", escape_json_string(p.as_str())))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn json_set(paths: &BTreeSet<RepoPath>) -> String {
    format!(
        "[{}]",
        paths
            .iter()
            .map(|p| format!("\"{}\"", escape_json_string(p.as_str())))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn json_pred(paths: &[RepoPath]) -> String {
    format!(
        "[{}]",
        paths
            .iter()
            .map(|p| format!("\"{}\"", escape_json_string(p.as_str())))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn run() -> Result<(), String> {
    // `cargo bench` adds this cargo-only switch after user arguments.
    let args: Vec<_> = env::args().skip(1).filter(|s| s != "--bench").collect();
    if args.is_empty() {
        return Err("usage: cargo bench --bench cochange -- <repository> [train_fraction] [--per-query <jsonl-path>]".into());
    }
    let fraction = args
        .get(1)
        .map(|s| {
            s.parse::<f64>()
                .map_err(|e| format!("invalid fraction: {e}"))
        })
        .transpose()?
        .unwrap_or(0.8);
    if !(0.1..1.0).contains(&fraction) {
        return Err("train fraction must be >= 0.1 and < 1.0".into());
    }
    let repo = Path::new(&args[0]);
    let start = std::time::Instant::now();
    let history = analyze_history(repo, usize::MAX)?;
    let all_transactions = history.raw_transactions();
    let n = all_transactions.len();
    let split = if n < 2 {
        0
    } else {
        ((n as f64 * fraction).floor() as usize).clamp(1, n - 1)
    };
    let cutoff = all_transactions
        .get(split)
        .map(|tx| tx.committer_timestamp)
        .unwrap_or(i64::MAX);
    let initial_train = history.prefix_before(cutoff);
    let mut learned = Score::default();
    let mut plain = Score::default();
    let mut proximity = Score::default();
    let mut query_rows = Vec::new();
    let test_transactions: Vec<_> = all_transactions
        .iter()
        .filter(|tx| tx.committer_timestamp >= cutoff)
        .collect();
    let test_transaction_count = test_transactions.len();
    for test_tx in test_transactions {
        if test_tx.paths.len() < 2 {
            continue;
        }
        let full_prefix = history.prefix_before(test_tx.committer_timestamp);
        if !history
            .through_timestamp(test_tx.committer_timestamp)
            .commits()
            .any(|commit| commit == test_tx.commit)
        {
            continue;
        }
        let query_count = test_tx.paths.len().div_ceil(2);
        let queries = &test_tx.paths[..query_count];
        let query_set: BTreeSet<_> = queries.iter().map(|p| full_prefix.canonical(p)).collect();
        let truth: BTreeSet<_> = test_tx.paths[query_count..]
            .iter()
            .map(|p| {
                full_prefix
                    .display_path(p)
                    .unwrap_or_else(|| full_prefix.canonical(p))
            })
            .filter(|p| !query_set.contains(p))
            .collect();
        if truth.is_empty() {
            continue;
        }
        let preds: Vec<_> = recommend_related(&full_prefix, queries, 3)
            .into_iter()
            .map(|h| h.path)
            .collect();
        let plain_preds = plain_counts(&full_prefix, queries);
        let proximity_preds = basename_proximity(&full_prefix, queries);
        learned.add(&preds, &truth);
        plain.add(&plain_preds, &truth);
        proximity.add(&proximity_preds, &truth);
        query_rows.push(format!("{{\"commit\":\"{}\",\"training_cutoff_committer_time\":{},\"train_transactions\":{},\"inputs\":{},\"truth\":{},\"thresholded_cochange\":{},\"plain_cochange_count\":{},\"basename_proximity\":{}}}",
            test_tx.commit, test_tx.committer_timestamp, full_prefix.transaction_count(), json_paths(queries), json_set(&truth), json_pred(&preds), json_pred(&plain_preds), json_pred(&proximity_preds)));
    }
    println!(
        "{{\"schema_version\":1,\"repo\":\"{}\",\"transactions\":{},\"skipped_non_utf8_transactions\":{},\"skipped_ambiguous_identity_transactions\":{},\"train_transactions_at_cutoff\":{},\"test_transactions_at_cutoff\":{},\"training_cutoff_committer_time\":{},\"train_fraction_target\":{:.4},\"split_rule\":\"equal timestamps grouped; each query trains on committer timestamps strictly earlier than that held-out transaction\",\"query_rule\":\"lexicographically first ceil(half) of changed paths; predict the remainder\",\"false_advisory_denominator\":\"all emitted suggestions across evaluated queries\",\"latency_ms\":{},\"baselines\":{{\"thresholded_cochange\":{},\"plain_cochange_count\":{},\"basename_proximity\":{}}}}}",
        escape_json_string(&args[0]),
        n,
        history.skipped_non_utf8_transactions(),
        history.skipped_ambiguous_identity_transactions(),
        initial_train.transaction_count(),
        test_transaction_count,
        cutoff,
        fraction,
        start.elapsed().as_millis(),
        learned.json(),
        plain.json(),
        proximity.json()
    );
    if args.len() > 2 {
        if args.get(2).map(String::as_str) != Some("--per-query") || args.len() != 4 {
            return Err("optional detail output syntax: --per-query <jsonl-path>".into());
        }
        fs::write(
            &args[3],
            query_rows.join("\n") + if query_rows.is_empty() { "" } else { "\n" },
        )
        .map_err(|e| format!("cannot write per-query JSONL: {e}"))?;
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("cochange benchmark: {e}");
        process::exit(2);
    }
}
