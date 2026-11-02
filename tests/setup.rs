//! `vexen setup`, `vexen demo` and `vexen doctor --mcp`, driven through the real binary
//! against throwaway homes and repositories. Nothing here touches the real home.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_vexen");

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn git_repo(dir: &Path) {
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.email", "t@example.invalid"],
        vec!["config", "user.name", "t"],
    ] {
        assert!(Command::new("git").arg("-C").arg(dir).args(&args).status().unwrap().success());
    }
    fs::write(dir.join("f.txt"), "x\n").unwrap();
    assert!(Command::new("git").arg("-C").arg(dir).args(["add", "-A"]).status().unwrap().success());
    assert!(Command::new("git").arg("-C").arg(dir).args(["commit", "-qm", "i"]).status().unwrap().success());
}

/// A fresh sandbox: a home whose name has a space and a non-ASCII letter, and a Git repository.
struct Sandbox {
    _tmp: TempDir,
    home: PathBuf,
    repo: PathBuf,
}

fn sandbox(agent_dirs: &[&str]) -> Sandbox {
    let tmp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(tmp.path()).unwrap();
    let home = root.join("home dir \u{fc}");
    let repo = root.join("project");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&repo).unwrap();
    for d in agent_dirs {
        fs::create_dir_all(home.join(d)).unwrap();
    }
    git_repo(&repo);
    Sandbox { _tmp: tmp, home, repo }
}

fn setup(s: &Sandbox, extra: &[&str]) -> Output {
    Command::new(BIN)
        .current_dir(&s.repo)
        .arg("setup")
        .arg("--home")
        .arg(&s.home)
        .args(extra)
        .output()
        .unwrap()
}

/// The binary's own path the way setup writes it: canonical, without the Windows `\\?\` prefix.
fn own_path() -> String {
    let canonical = fs::canonicalize(BIN).unwrap().to_string_lossy().into_owned();
    let backslash = char::from(92u8);
    canonical.strip_prefix(&format!("{backslash}{backslash}?{backslash}")).map(str::to_owned).unwrap_or(canonical)
}

fn json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn setup_places_everything_and_a_real_server_answers() {
    let s = sandbox(&[".cursor", ".claude"]);
    let o = setup(&s, &[]);
    let out = text(&o);
    assert!(o.status.success(), "{out}");
    assert!(out.contains("4 tools discovered"), "{out}");
    assert!(out.contains("answered `no_relevant_evidence`"), "{out}");
    assert!(s.repo.join(".vexen/state.sqlite").is_file());
    let cursor = json(&s.home.join(".cursor/mcp.json"));
    assert_eq!(cursor["mcpServers"]["vexen"]["command"], own_path().as_str());
    assert_eq!(cursor["mcpServers"]["vexen"]["args"][1], "--stdio");
    let skill = fs::read_to_string(s.home.join(".claude/skills/vexen/SKILL.md")).unwrap();
    assert!(skill.starts_with("---\nname: vexen\n"));
    assert!(skill.contains(&own_path()), "the skill names the command for this machine");
    // The agent step is never claimed.
    assert!(out.contains("Vexen cannot check that from here"), "{out}");
}

#[test]
fn setup_twice_creates_nothing_new_and_keeps_every_file() {
    let s = sandbox(&[".cursor", ".claude"]);
    assert!(setup(&s, &[]).status.success());
    let files = [".cursor/mcp.json", ".claude.json", ".claude/skills/vexen/SKILL.md"];
    let before: Vec<String> = files.iter().map(|f| fs::read_to_string(s.home.join(f)).unwrap()).collect();
    let second = setup(&s, &[]);
    assert!(second.status.success(), "{}", text(&second));
    assert!(text(&second).contains("already"), "{}", text(&second));
    let after: Vec<String> = files.iter().map(|f| fs::read_to_string(s.home.join(f)).unwrap()).collect();
    assert_eq!(before, after);
}

#[test]
fn no_agent_found_is_a_failure_not_a_ready_message() {
    let s = sandbox(&[]);
    let o = setup(&s, &[]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(2), "{out}");
    assert!(out.contains("no supported agent"), "{out}");
    assert!(!out.contains("Then, in your agent's chat"), "must not announce readiness: {out}");
    assert!(s.repo.join(".vexen").is_dir(), "the project's memory is still started");
}

#[test]
fn a_skill_you_edited_is_left_alone_while_the_rest_is_done() {
    let s = sandbox(&[".cursor", ".claude"]);
    let skill = s.home.join(".claude/skills/vexen/SKILL.md");
    fs::create_dir_all(skill.parent().unwrap()).unwrap();
    fs::write(&skill, "my own notes about vexen\n").unwrap();
    let o = setup(&s, &[]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert_eq!(fs::read_to_string(&skill).unwrap(), "my own notes about vexen\n");
    assert!(text(&o).contains("left alone"), "{}", text(&o));
    assert!(s.home.join(".cursor/mcp.json").is_file(), "other steps still ran");
    // --force replaces it.
    assert!(setup(&s, &["--force"]).status.success());
    assert!(fs::read_to_string(&skill).unwrap().contains("vexen:managed"));
}

#[test]
fn a_config_it_cannot_edit_safely_is_reported_with_the_snippet_and_memory_survives() {
    let s = sandbox(&[".cursor"]);
    let cfg = s.home.join(".cursor/mcp.json");
    let original = "{\n  // keep my comment\n  \"mcpServers\": {}\n}\n";
    fs::write(&cfg, original).unwrap();
    let o = setup(&s, &[]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(2), "{out}");
    assert!(out.contains("by hand"), "{out}");
    assert!(out.contains("mcpServers"), "the snippet is printed: {out}");
    assert_eq!(fs::read_to_string(&cfg).unwrap(), original);
    assert!(s.repo.join(".vexen/state.sqlite").is_file());
}

#[test]
fn dry_run_writes_nothing_anywhere() {
    let s = sandbox(&[".cursor", ".claude"]);
    let o = setup(&s, &["--dry-run"]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("plan"), "{}", text(&o));
    assert!(!s.repo.join(".vexen").exists());
    assert!(!s.home.join(".cursor/mcp.json").exists());
    assert!(!s.home.join(".claude.json").exists());
    assert!(!s.home.join(".claude/skills").exists());
}

#[test]
fn a_binary_running_from_a_cache_is_copied_to_a_permanent_folder() {
    let s = sandbox(&[".cursor"]);
    let cache = s.home.join("cache");
    let exe_name = Path::new(BIN).file_name().unwrap();
    let cached = cache.join("0.0.0").join("target").join(exe_name);
    fs::create_dir_all(cached.parent().unwrap()).unwrap();
    fs::copy(BIN, &cached).unwrap();
    let install_dir = s.home.join("permanent bin");
    let o = Command::new(&cached)
        .current_dir(&s.repo)
        .env("VEXEN_HOME", &cache)
        .args(["setup", "--agent", "cursor", "--install-dir"])
        .arg(&install_dir)
        .arg("--home")
        .arg(&s.home)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", text(&o));
    let placed = install_dir.join(exe_name);
    assert!(placed.is_file(), "{}", text(&o));
    let cfg = json(&s.home.join(".cursor/mcp.json"));
    let written = cfg["mcpServers"]["vexen"]["command"].as_str().unwrap();
    assert!(Path::new(written).starts_with(fs::canonicalize(&install_dir).unwrap_or(install_dir.clone())) || written.contains("permanent bin"), "{written}");
    assert!(!written.contains("0.0.0"), "must not point into the cache: {written}");
    // The copy actually runs.
    let v = Command::new(&placed).arg("--version").output().unwrap();
    assert!(String::from_utf8_lossy(&v.stdout).starts_with("vexen "));
}

#[test]
fn outside_a_repository_the_agents_are_still_configured_and_init_is_left_to_the_user() {
    let s = sandbox(&[".cursor"]);
    let elsewhere = s.home.join("not a repo");
    fs::create_dir_all(&elsewhere).unwrap();
    let o = Command::new(BIN).current_dir(&elsewhere).arg("setup").arg("--home").arg(&s.home).output().unwrap();
    let out = text(&o);
    assert!(o.status.success(), "{out}");
    assert!(out.contains("vexen init"), "{out}");
    assert!(s.home.join(".cursor/mcp.json").is_file());
    assert!(!elsewhere.join(".vexen").exists());
}

#[test]
fn uninstall_with_skill_removes_what_setup_wrote_and_nothing_else() {
    let s = sandbox(&[".claude"]);
    fs::write(s.home.join(".claude/settings.json"), "{\"theme\":\"dark\"}\n").unwrap();
    assert!(setup(&s, &[]).status.success());
    let o = Command::new(BIN)
        .args(["uninstall", "--scope", "global", "--agent", "claude-code", "--skill", "--home"])
        .arg(&s.home)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", text(&o));
    assert!(!s.home.join(".claude/skills/vexen/SKILL.md").exists());
    assert!(json(&s.home.join(".claude.json"))["mcpServers"].get("vexen").is_none());
    assert_eq!(fs::read_to_string(s.home.join(".claude/settings.json")).unwrap(), "{\"theme\":\"dark\"}\n");
    assert!(s.repo.join(".vexen").is_dir(), "the project's memory is never removed");
}

#[test]
fn demo_is_synthetic_cleans_up_and_leaves_your_project_alone() {
    let s = sandbox(&[]);
    let o = Command::new(BIN).current_dir(&s.repo).arg("demo").output().unwrap();
    let out = text(&o);
    assert!(o.status.success(), "{out}");
    assert!(out.contains("synthetic"), "{out}");
    assert!(out.contains("observed_failure"), "{out}");
    assert!(out.contains("no_relevant_evidence"), "{out}");
    assert!(out.contains("Your own repository is not touched"), "{out}");
    assert!(!s.repo.join(".vexen").exists());
    let line = out.lines().find(|l| l.starts_with("Working in a throwaway repository: ")).unwrap();
    let dir = line.trim_start_matches("Working in a throwaway repository: ");
    assert!(!Path::new(dir).exists(), "the throwaway repository is deleted: {dir}");
}

#[test]
fn demo_refuses_a_folder_that_is_not_empty() {
    let s = sandbox(&[]);
    let o = Command::new(BIN).current_dir(&s.repo).args(["demo", "--dir"]).arg(&s.repo).output().unwrap();
    assert!(!o.status.success());
    assert!(text(&o).contains("not empty"), "{}", text(&o));
}

#[test]
fn doctor_mcp_starts_a_real_server_and_says_what_it_proves() {
    let s = sandbox(&[]);
    assert!(Command::new(BIN).current_dir(&s.repo).arg("init").status().unwrap().success());
    let o = Command::new(BIN).current_dir(&s.repo).args(["doctor", "--mcp"]).output().unwrap();
    let out = text(&o);
    assert!(o.status.success(), "{out}");
    assert!(out.contains("mcp: ok (4 tools"), "{out}");
    assert!(out.contains("not that your agent is connected"), "{out}");
}
