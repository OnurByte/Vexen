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
    assert!(out(&o).contains("would create"), "{}", out(&o));
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

#[test]
fn opencode_gets_the_current_servers_shape_by_default() {
    let (_d, h) = home();
    let oc = ["--agent", "opencode", "--scope", "global"];
    assert!(run(&h, "install", &oc, &[]).status.success());
    let v: serde_json::Value =
        serde_json::from_str(&read(&h.join(".config/opencode/opencode.json"))).unwrap();
    let entry = &v["mcp"]["servers"]["vexen"];
    assert_eq!(entry["type"], "local");
    assert_eq!(entry["command"][1], "mcp");
    assert!(v["mcp"].get("vexen").is_none());
    assert!(out(&run(&h, "install", &oc, &[])).contains("unchanged"));
    assert!(run(&h, "uninstall", &oc, &[]).status.success());
    let after: serde_json::Value =
        serde_json::from_str(&read(&h.join(".config/opencode/opencode.json"))).unwrap();
    assert!(after["mcp"]["servers"].get("vexen").is_none());
}

#[test]
fn opencode_keeps_a_legacy_layout_when_the_file_already_uses_it() {
    let (_d, h) = home();
    let file = h.join(".config/opencode/opencode.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        r#"{"mcp":{"other":{"type":"local","command":["x"],"enabled":true}}}"#,
    )
    .unwrap();
    let oc = ["--agent", "opencode", "--scope", "global"];
    assert!(run(&h, "install", &oc, &[]).status.success());
    let v: serde_json::Value = serde_json::from_str(&read(&file)).unwrap();
    assert_eq!(v["mcp"]["vexen"]["enabled"], true);
    assert!(v["mcp"].get("servers").is_none());
    assert_eq!(v["mcp"]["other"]["command"][0], "x");
}

#[test]
fn duplicate_json_keys_are_refused_instead_of_silently_dropped() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let text = r#"{"mcpServers":{"other":{"command":"x"}},"mcpServers":{}}"#;
    fs::write(&file, text).unwrap();
    let o = run(&h, "install", &CURSOR, &[]);
    assert!(!o.status.success());
    assert!(out(&o).contains("duplicate"), "{}", out(&o));
    assert_eq!(read(&file), text);
}

#[test]
fn an_existing_backup_is_never_overwritten() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, r#"{"theme":"dark"}"#).unwrap();
    let backup = h.join(".cursor/mcp.json.vexen-backup");
    fs::write(&backup, "older backup").unwrap();
    assert!(run(&h, "install", &CURSOR, &[]).status.success());
    assert_eq!(read(&backup), "older backup");
}

#[cfg(unix)]
#[test]
fn a_symlinked_config_is_left_alone_and_its_target_is_not_modified() {
    let (_d, h) = home();
    let real = h.join("dotfiles-mcp.json");
    fs::write(&real, r#"{"mcpServers":{}}"#).unwrap();
    let dir = h.join(".cursor");
    fs::create_dir_all(&dir).unwrap();
    std::os::unix::fs::symlink(&real, dir.join("mcp.json")).unwrap();
    let o = run(&h, "install", &CURSOR, &[]);
    assert!(!o.status.success());
    assert!(out(&o).contains("symbolic link"), "{}", out(&o));
    assert_eq!(read(&real), r#"{"mcpServers":{}}"#);
    assert!(fs::symlink_metadata(dir.join("mcp.json")).unwrap().file_type().is_symlink());
}

fn npx_argv() -> Vec<&'static str> {
    if cfg!(windows) {
        vec!["cmd", "/c", "npx", "-y", "@onurbyte/vexen", "mcp", "--stdio"]
    } else {
        vec!["npx", "-y", "@onurbyte/vexen", "mcp", "--stdio"]
    }
}

#[test]
fn npx_flag_writes_the_launcher_entry_and_is_idempotent() {
    let (_d, h) = home();
    assert!(run(&h, "install", &CURSOR, &["--npx"]).status.success());
    let file = h.join(".cursor/mcp.json");
    let v: serde_json::Value = serde_json::from_str(&read(&file)).unwrap();
    let entry = &v["mcpServers"]["vexen"];
    let want = npx_argv();
    assert_eq!(entry["command"], want[0]);
    let args: Vec<&str> = entry["args"].as_array().unwrap().iter().map(|a| a.as_str().unwrap()).collect();
    assert_eq!(args, want[1..]);
    assert!(out(&run(&h, "install", &CURSOR, &["--npx"])).contains("unchanged"));
    // A plain install now sees a different entry and refuses to overwrite it.
    assert!(!run(&h, "install", &CURSOR, &[]).status.success());
    assert!(run(&h, "uninstall", &CURSOR, &["--npx"]).status.success());
    let after: serde_json::Value = serde_json::from_str(&read(&file)).unwrap();
    assert!(after["mcpServers"].get("vexen").is_none());
}

#[test]
fn npx_flag_works_for_toml_and_opencode_and_conflicts_with_command() {
    let (_d, h) = home();
    let codex = ["--agent", "codex", "--scope", "global"];
    assert!(run(&h, "install", &codex, &["--npx"]).status.success());
    let toml = read(&h.join(".codex/config.toml"));
    assert!(toml.contains("@onurbyte/vexen") && toml.contains("--stdio"), "{toml}");
    assert!(out(&run(&h, "install", &codex, &["--npx"])).contains("unchanged"));
    let oc = ["--agent", "opencode", "--scope", "global"];
    assert!(run(&h, "install", &oc, &["--npx"]).status.success());
    let v: serde_json::Value = serde_json::from_str(&read(&h.join(".config/opencode/opencode.json"))).unwrap();
    let cmd: Vec<&str> = v["mcp"]["servers"]["vexen"]["command"].as_array().unwrap().iter().map(|a| a.as_str().unwrap()).collect();
    assert_eq!(cmd, npx_argv());
    let both = run(&h, "install", &CURSOR, &["--npx", "--command", "x"]);
    assert!(!both.status.success(), "--npx and --command must conflict");
}

#[test]
fn uninstall_removes_an_entry_written_with_an_absolute_binary_path() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        r#"{"mcpServers":{"other":{"command":"x"},"vexen":{"command":"C:/Program Files/Vexen/vexen.exe","args":["mcp","--stdio"]}}}"#,
    )
    .unwrap();
    let o = run(&h, "uninstall", &CURSOR, &[]);
    assert!(o.status.success(), "{}", out(&o));
    let v: serde_json::Value = serde_json::from_str(&read(&file)).unwrap();
    assert!(v["mcpServers"].get("vexen").is_none());
    assert_eq!(v["mcpServers"]["other"]["command"], "x");
}

#[test]
fn uninstall_does_not_touch_a_look_alike_that_is_not_ours() {
    let (_d, h) = home();
    let file = h.join(".cursor/mcp.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let text = r#"{"mcpServers":{"vexen":{"command":"/opt/other-tool","args":["serve"]}}}"#;
    fs::write(&file, text).unwrap();
    assert!(!run(&h, "uninstall", &CURSOR, &[]).status.success());
    assert_eq!(read(&file), text);
}
