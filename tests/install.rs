use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;

fn vexen(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vexen"))
        .args(args)
        .arg("--home")
        .arg(home)
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn home() -> (TempDir, PathBuf) {
    let d = tempfile::tempdir().unwrap();
    let p = fs::canonicalize(d.path()).unwrap();
    (d, p)
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).unwrap()
}

const CURSOR: [&str; 4] = ["--agent", "cursor", "--scope", "global"];

fn with(cmd: &str, base: &[&str], extra: &[&str]) -> Vec<String> {
    std::iter::once(cmd)
        .chain(base.iter().copied())
        .chain(extra.iter().copied())
        .map(str::to_owned)
        .collect()
}

fn run(h: &Path, cmd: &str, base: &[&str], extra: &[&str]) -> Output {
    let args = with(cmd, base, extra);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    vexen(h, &refs)
}

#[test]
fn creates_a_config_and_is_idempotent() {
    let (_d, h) = home();
    let first = run(&h, "install", &CURSOR, &[]);
    assert!(first.status.success(), "{}", out(&first));
    let file = h.join(".cursor/mcp.json");
    let v: serde_json::Value = serde_json::from_str(&read(&file)).unwrap();
    assert_eq!(v["mcpServers"]["vexen"]["command"], "vexen");
    assert_eq!(v["mcpServers"]["vexen"]["args"][1], "--stdio");
    let before = read(&file);
    let again = run(&h, "install", &CURSOR, &[]);
    assert!(out(&again).contains("unchanged"), "{}", out(&again));
    assert_eq!(read(&file), before);
}

#[test]
fn preserves_other_servers_and_unrelated_keys_and_backs_up_once() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let original = r#"{"theme":"dark","mcpServers":{"other":{"command":"x","args":["1"]}}}"#;
    fs::write(&file, original).unwrap();
    assert!(run(&h, "install", &CURSOR, &[]).status.success());
    let v: serde_json::Value = serde_json::from_str(&read(&file)).unwrap();
    assert_eq!(v["theme"], "dark");
    assert_eq!(v["mcpServers"]["other"]["command"], "x");
    assert!(v["mcpServers"]["vexen"].is_object());
    let backup = h.join(".cursor/mcp.json.vexen-backup");
    assert_eq!(read(&backup), original);
    // A later modifying run must not overwrite the first backup.
    assert!(run(&h, "uninstall", &CURSOR, &[]).status.success());
    assert!(run(&h, "install", &CURSOR, &["--command", "other-vexen"]).status.success());
    assert_eq!(read(&backup), original);
}

#[test]
fn dry_run_writes_nothing() {
    let (_d, h) = home();
    let o = run(&h, "install", &CURSOR, &["--dry-run"]);
    assert!(o.status.success());
    assert!(out(&o).contains("would created"), "{}", out(&o));
    assert!(!h.join(".cursor").exists());
}

#[test]
fn jsonc_comments_are_refused_with_a_snippet_and_the_file_is_untouched() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let text = "{\n  // my servers\n  \"mcpServers\": {}\n}\n";
    fs::write(&file, text).unwrap();
    let o = run(&h, "install", &CURSOR, &[]);
    assert!(!o.status.success());
    assert!(out(&o).contains("add this by hand"), "{}", out(&o));
    assert_eq!(read(&file), text);
    assert!(!h.join(".cursor/mcp.json.vexen-backup").exists());
}

#[test]
fn a_different_existing_vexen_entry_is_not_overwritten_without_force() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let text = r#"{"mcpServers":{"vexen":{"command":"custom"}}}"#;
    fs::write(&file, text).unwrap();
    let o = run(&h, "install", &CURSOR, &[]);
    assert!(!o.status.success());
    assert_eq!(read(&file), text);
    let forced = run(&h, "install", &CURSOR, &["--force"]);
    assert!(forced.status.success());
    assert!(read(&file).contains("--stdio"));
}

#[test]
fn uninstall_removes_only_our_entry_and_keeps_a_modified_one() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, r#"{"mcpServers":{"other":{"command":"x"}}}"#).unwrap();
    run(&h, "install", &CURSOR, &[]);
    let o = run(&h, "uninstall", &CURSOR, &[]);
    assert!(o.status.success(), "{}", out(&o));
    let v: serde_json::Value = serde_json::from_str(&read(&file)).unwrap();
    assert!(v["mcpServers"].get("vexen").is_none());
    assert_eq!(v["mcpServers"]["other"]["command"], "x");

    fs::write(&file, r#"{"mcpServers":{"vexen":{"command":"mine"}}}"#).unwrap();
    let kept = run(&h, "uninstall", &CURSOR, &[]);
    assert!(!kept.status.success());
    assert!(read(&file).contains("mine"));
}

#[test]
fn codex_toml_keeps_comments_and_other_tables() {
    let (_d, h) = home();
    let file = h.join(".codex/config.toml");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let original = "# my codex settings\nmodel = \"x\"\n\n[mcp_servers.other]\ncommand = \"o\"\n";
    fs::write(&file, original).unwrap();
    let codex = ["--agent", "codex", "--scope", "global"];
    let o = run(&h, "install", &codex, &[]);
    assert!(o.status.success(), "{}", out(&o));
    let text = read(&file);
    assert!(text.contains("# my codex settings"));
    assert!(text.contains("[mcp_servers.other]"));
    assert!(text.contains("[mcp_servers.vexen]"));
    assert!(text.contains("\"--stdio\""));
    assert!(out(&run(&h, "install", &codex, &[])).contains("unchanged"));
    assert!(run(&h, "uninstall", &codex, &[]).status.success());
    let after = read(&file);
    assert!(after.contains("# my codex settings") && after.contains("[mcp_servers.other]"));
    assert!(!after.contains("vexen"));
}

#[test]
fn auto_installs_only_clients_that_are_present() {
    let (_d, h) = home();
    fs::create_dir_all(h.join(".cursor")).unwrap();
    let o = vexen(&h, &["install", "--scope", "global"]);
    assert!(o.status.success(), "{}", out(&o));
    assert!(h.join(".cursor/mcp.json").exists());
    assert!(!h.join(".codex").exists());
    assert!(!h.join(".gemini").exists());
}

#[test]
fn project_scope_writes_inside_the_repository() {
    let (_d, h) = home();
    let (_r, repo) = home();
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(["init", "-q"])
            .status()
            .unwrap()
            .success()
    );
    let o = Command::new(env!("CARGO_BIN_EXE_vexen"))
        .args(["install", "--agent", "claude-code", "--scope", "project", "--repo"])
        .arg(&repo)
        .arg("--home")
        .arg(&h)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", out(&o));
    assert!(read(&repo.join(".mcp.json")).contains("mcpServers"));
    assert!(!h.join(".claude.json").exists());
}

#[test]
fn print_config_and_unsupported_clients_never_write() {
    let (_d, h) = home();
    let zed = ["--agent", "zed", "--scope", "global"];
    let p = run(&h, "install", &zed, &["--print-config"]);
    assert!(p.status.success());
    assert!(out(&p).contains("mcp"));
    let z = run(&h, "install", &zed, &[]);
    assert!(!z.status.success());
    assert!(out(&z).contains("manual edit"));
    let bad = run(&h, "install", &["--agent", "nope", "--scope", "global"], &[]);
    assert!(!bad.status.success());
    assert_eq!(fs::read_dir(&h).unwrap().count(), 0);
}

#[test]
fn crlf_line_endings_are_preserved() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "{\r\n  \"mcpServers\": {}\r\n}\r\n").unwrap();
    assert!(run(&h, "install", &CURSOR, &[]).status.success());
    let text = read(&file);
    assert!(text.contains("\r\n") && !text.replace("\r\n", "").contains('\n'));
}
