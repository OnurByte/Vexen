use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
use vexen::{
    analysis::cochange::recommend_related,
    git::{history::analyze_history, identity::RepoPath},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Repo(PathBuf);
impl Repo {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "vexen-git-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn git(&self, args: &[&str]) -> String {
        self.git_env(args, &[])
    }
    fn git_env(&self, args: &[&str], vars: &[(&str, &str)]) -> String {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.0).args(args);
        for (key, value) in vars {
            command.env(key, value);
        }
        let out = command.output().unwrap();
        assert!(
            out.status.success(),
            "git {:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
    fn commit(&self, files: &[(&str, &str)], message: &str) -> String {
        for (path, body) in files {
            let p = self.0.join(path);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, body).unwrap();
        }
        self.git(&["add", "-A"]);
        self.git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            message,
        ]);
        self.git(&["rev-parse", "HEAD"])
    }
    fn commit_at(&self, files: &[(&str, &str)], message: &str, seconds: i64) -> String {
        for (path, body) in files {
            let p = self.0.join(path);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, body).unwrap();
        }
        self.git(&["add", "-A"]);
        let date = format!("@{seconds} +0000");
        self.git_env(
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                message,
            ],
            &[("GIT_AUTHOR_DATE", &date), ("GIT_COMMITTER_DATE", &date)],
        );
        self.git(&["rev-parse", "HEAD"])
    }
    fn init(&self) {
        self.git(&["init", "--quiet", "--initial-branch=main"]);
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn path(s: &str) -> RepoPath {
    RepoPath::new(s).unwrap()
}

#[test]
fn ordinary_commits_produce_ranked_pair_with_commit_evidence() {
    let r = Repo::new();
    r.init();
    let c1 = r.commit(&[("src/a.rs", "a1"), ("src/b.rs", "b1")], "coupled");
    r.commit(&[("src/a.rs", "a2"), ("src/b.rs", "b2")], "coupled again");
    r.commit(&[("docs/notes.md", "unrelated")], "unrelated");
    let index = analyze_history(&r.0, 100).unwrap();
    let hints = recommend_related(&index, &[path("src/a.rs")], 3);
    assert_eq!(hints.first().unwrap().path, path("src/b.rs"));
    assert_eq!(hints[0].support, 2);
    assert!(
        hints[0].example_commits.contains(&c1),
        "examples={:?}, c1={c1}",
        hints[0].example_commits
    );
}

#[test]
fn rename_tracks_identity_and_reports_current_path() {
    let r = Repo::new();
    r.init();
    r.commit(&[("old/name.rs", "one")], "create");
    fs::create_dir_all(r.0.join("new")).unwrap();
    r.git(&["mv", "old/name.rs", "new/name.rs"]);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "rename",
    ]);
    r.commit(&[("new/name.rs", "two"), ("src/peer.rs", "two")], "paired");
    r.commit(
        &[("new/name.rs", "three"), ("src/peer.rs", "three")],
        "paired again",
    );
    r.commit(&[("README.md", "unrelated")], "unrelated");
    let idx = analyze_history(&r.0, 100).unwrap();
    assert_eq!(
        idx.path_count(&path("old/name.rs")),
        idx.path_count(&path("new/name.rs"))
    );
    assert_eq!(
        recommend_related(&idx, &[path("new/name.rs")], 2)[0].path,
        path("src/peer.rs")
    );
}

#[test]
fn cochange_hints_use_the_current_rename_target_and_abstain_after_deletion() {
    let r = Repo::new();
    r.init();
    r.commit(&[("src/a.rs", "one"), ("src/peer.rs", "one")], "pair one");
    r.commit(&[("src/a.rs", "two"), ("src/peer.rs", "two")], "pair two");
    r.git(&["mv", "src/a.rs", "src/b.rs"]);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "rename candidate",
    ]);
    r.commit(
        &[("src/b.rs", "three"), ("src/peer.rs", "three")],
        "pair three",
    );
    r.commit(
        &[("src/b.rs", "four"), ("src/peer.rs", "four")],
        "pair four",
    );
    r.commit(&[("notes.md", "unrelated")], "unrelated");
    let idx = analyze_history(&r.0, 100).unwrap();
    let hints = recommend_related(&idx, &[path("src/peer.rs")], 3);
    assert!(hints.iter().any(|hint| hint.path == path("src/b.rs")));
    assert!(!hints.iter().any(|hint| hint.path == path("src/a.rs")));

    r.git(&["rm", "src/b.rs"]);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "delete renamed candidate",
    ]);
    let after_delete = analyze_history(&r.0, 100).unwrap();
    assert!(
        !recommend_related(&after_delete, &[path("src/peer.rs")], 3)
            .iter()
            .any(|hint| hint.path == path("src/b.rs"))
    );
}

#[test]
fn merges_are_excluded_and_mass_edits_are_filtered() {
    let r = Repo::new();
    r.init();
    r.commit(&[("src/a.rs", "base")], "base");
    r.git(&["checkout", "-b", "side"]);
    r.commit(&[("src/a.rs", "side"), ("src/b.rs", "side")], "side");
    r.git(&["checkout", "main"]);
    r.commit(&[("src/c.rs", "main")], "main");
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "merge",
        "--no-ff",
        "side",
        "-m",
        "merge",
    ]);
    for n in 0..30 {
        r.commit(&[(&format!("generated/f{n}.txt"), "x")], "bulk");
    }
    let mass: Vec<_> = (0..25)
        .map(|n| (format!("mass/f{n}.rs"), "format\n"))
        .collect();
    let refs: Vec<_> = mass.iter().map(|(p, body)| (p.as_str(), *body)).collect();
    r.commit(&refs, "mass edit");
    let idx = analyze_history(&r.0, 100).unwrap();
    assert!(idx.transaction_count() < 35);
    assert!(recommend_related(&idx, &[path("generated/f1.txt")], 3).is_empty());
    assert!(
        recommend_related(&idx, &[path("src/a.rs")], 3)
            .iter()
            .all(|hint| hint.support <= 1)
    );
}

#[test]
fn empty_history_and_unknown_paths_abstain() {
    let r = Repo::new();
    r.init();
    let idx = analyze_history(&r.0, 100).unwrap();
    assert_eq!(idx.transaction_count(), 0);
    assert!(recommend_related(&idx, &[path("src/missing.rs")], 3).is_empty());
}

#[test]
fn shallow_history_is_bounded_to_available_commits() {
    let source = Repo::new();
    source.init();
    source.commit(&[("a", "1")], "one");
    source.commit(&[("a", "2"), ("b", "2")], "two");
    let shallow = Repo::new();
    let url = format!("file://{}", source.0.display());
    let output = Command::new("git")
        .args(["clone", "--quiet", "--depth", "1", &url])
        .arg(&shallow.0)
        .output()
        .unwrap();
    assert!(output.status.success());
    let idx = analyze_history(&shallow.0, 100).unwrap();
    assert!(idx.transaction_count() <= 1);
}

#[test]
fn control_separator_in_path_does_not_split_commits_and_empty_commits_are_safe() {
    let r = Repo::new();
    r.init();
    r.commit(
        &[("odd\u{1e}name.rs", "one"), ("peer.rs", "one")],
        "separator path",
    );
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--allow-empty",
        "--quiet",
        "-m",
        "empty",
    ]);
    let idx = analyze_history(&r.0, 100).unwrap();
    assert_eq!(idx.transaction_count(), 1);
    assert_eq!(idx.transactions()[0].paths.len(), 2);
}

#[cfg(unix)]
#[test]
fn non_utf8_paths_skip_the_whole_transaction_and_are_counted() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let r = Repo::new();
    r.init();
    r.commit_at(&[("before.rs", "before")], "before invalid path", 100);
    fs::write(r.0.join(OsString::from_vec(b"bad-\xff.rs".to_vec())), "bad").unwrap();
    r.git(&["add", "-A"]);
    let date = "@200 +0000";
    r.git_env(
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "bad-path",
        ],
        &[("GIT_AUTHOR_DATE", date), ("GIT_COMMITTER_DATE", date)],
    );
    r.commit_at(&[("after.rs", "after")], "after invalid path", 300);
    let idx = analyze_history(&r.0, 100).unwrap();
    assert_eq!(idx.transaction_count(), 2);
    assert_eq!(idx.skipped_non_utf8_transactions(), 1);
    assert_eq!(idx.prefix_before(200).skipped_non_utf8_transactions(), 0);
    assert_eq!(idx.prefix_before(300).skipped_non_utf8_transactions(), 1);
    assert_eq!(idx.prefix(1).skipped_non_utf8_transactions(), 0);
    assert_eq!(idx.prefix(2).skipped_non_utf8_transactions(), 1);
}

#[test]
fn unix_backslashes_remain_distinct_from_git_directory_separators() {
    let backslash = path("a\\b");
    let slash = path("a/b");
    assert_ne!(backslash, slash);
    assert_eq!(backslash.as_str(), "a\\b");
}

#[test]
fn rename_cycle_has_one_stable_canonical_identity() {
    let r = Repo::new();
    r.init();
    r.commit(&[("a.rs", "one")], "create");
    r.git(&["mv", "a.rs", "b.rs"]);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "a to b",
    ]);
    r.git(&["mv", "b.rs", "a.rs"]);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "b to a",
    ]);
    let idx = analyze_history(&r.0, 100).unwrap();
    assert_eq!(idx.canonical(&path("a.rs")), idx.canonical(&path("b.rs")));
}

#[test]
fn recreated_renamed_path_abstains_from_mixed_identity_history() {
    let r = Repo::new();
    r.init();
    r.commit(&[("a.rs", "old")], "old identity");
    r.git(&["mv", "a.rs", "b.rs"]);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "rename",
    ]);
    r.commit(&[("a.rs", "new"), ("peer.rs", "new")], "recreate old path");
    let idx = analyze_history(&r.0, 100).unwrap();
    assert_ne!(idx.canonical(&path("a.rs")), idx.canonical(&path("b.rs")));
    assert!(idx.skipped_ambiguous_identity_transactions() > 0);
}

#[test]
fn sibling_history_is_sorted_by_committer_time_and_cutoff_is_strict() {
    let r = Repo::new();
    r.init();
    r.commit_at(&[("base.rs", "base")], "base", 100);
    r.git(&["checkout", "-b", "side"]);
    let side = r.commit_at(&[("side.rs", "side")], "side", 200);
    r.git(&["checkout", "main"]);
    let main = r.commit_at(&[("main.rs", "main")], "main", 200);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "merge",
        "--no-ff",
        "side",
        "-m",
        "merge",
    ]);
    let idx = analyze_history(&r.0, 100).unwrap();
    let timestamps: Vec<_> = idx
        .transactions()
        .iter()
        .map(|tx| tx.committer_timestamp)
        .collect();
    assert!(timestamps.windows(2).all(|w| w[0] <= w[1]));
    assert_eq!(idx.prefix_before(200).transaction_count(), 1);
    assert!(
        !idx.prefix_before(200)
            .commits()
            .any(|c| c == side || c == main)
    );
}
