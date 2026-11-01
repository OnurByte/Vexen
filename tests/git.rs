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
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.0)
            .args(args)
            .output()
            .unwrap();
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
    fs::write(r.0.join(OsString::from_vec(b"bad-\xff.rs".to_vec())), "bad").unwrap();
    fs::write(r.0.join("good.rs"), "good").unwrap();
    r.git(&["add", "-A"]);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "bad-path",
    ]);
    let idx = analyze_history(&r.0, 100).unwrap();
    assert_eq!(idx.transaction_count(), 0);
    assert_eq!(idx.skipped_non_utf8_transactions(), 1);
}
