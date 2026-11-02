//! `vexen setup`: one guided path from "I have the binary" to "an agent can use
//! it", built from the existing pieces (`init`, `install`, the embedded skill and
//! the protocol self-check). It reports four things separately and never calls
//! the result ready on its own:
//!
//! 1. installed:  the native binary is somewhere that will still exist
//! 2. configured: the agent's MCP entry (and skill) are written
//! 3. callable:   a real server process lists the tools and answers a check
//! 4. used:       whether the agent itself loads and calls it is NOT verified here

use super::{RepoArg, ScopeArg, display_path, home_dir};
use crate::{
    install::{Adapter, Launch, Outcome, Request, Scope, adapters, find, run},
    skill::{self, SkillOutcome},
    store::Store,
    verify::probe_mcp,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::Duration,
};

#[derive(clap::Args)]
pub struct SetupArgs {
    /// Client id (repeatable) or `auto` for every supported client found under the home directory.
    #[arg(long = "agent", default_value = "auto")]
    agents: Vec<String>,
    /// Where to register Vexen. Default: global when the client supports it, otherwise this repository.
    #[arg(long, value_enum)]
    scope: Option<ScopeArg>,
    /// Show every step and write nothing.
    #[arg(long)]
    dry_run: bool,
    /// Replace a `vexen` entry or a modified skill file that differs from what setup writes.
    #[arg(long)]
    force: bool,
    /// Have clients start Vexen through `npx @onurbyte/vexen` instead of a native binary.
    #[arg(long, conflicts_with = "command")]
    npx: bool,
    /// Binary clients should launch (absolute path or a name on PATH). Default: this binary.
    #[arg(long)]
    command: Option<String>,
    /// Do not copy this binary to a permanent folder when it runs from a cache or temp folder.
    #[arg(long)]
    no_copy: bool,
    /// Permanent folder for the binary. Default: ~/.local/bin, or %LOCALAPPDATA%\Programs\Vexen on Windows.
    #[arg(long)]
    install_dir: Option<PathBuf>,
    /// Do not install the agent skill.
    #[arg(long)]
    no_skill: bool,
    /// Do not initialize the current repository.
    #[arg(long)]
    no_init: bool,
    /// Home directory to use instead of the current user's (mainly for tests).
    #[arg(long)]
    home: Option<PathBuf>,
    #[command(flatten)]
    repo: RepoArg,
}

struct Report {
    problems: Vec<String>,
    todo: Vec<String>,
}

impl Report {
    fn line(&self, tag: &str, text: impl AsRef<str>) {
        println!("  [{tag:<6}] {}", text.as_ref());
    }
}

pub fn run_setup(args: SetupArgs) -> ExitCode {
    match go(args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("vexen: {error}");
            ExitCode::FAILURE
        }
    }
}

fn go(args: SetupArgs) -> Result<ExitCode, String> {
    let home = home_dir(&args.home)?;
    let mut report = Report { problems: vec![], todo: vec![] };
    let dry = args.dry_run;
    println!("{}", if dry { "Vexen setup (dry run: nothing is written)" } else { "Vexen setup" });

    // 1. The binary agents will start.
    println!("\n1. Native binary");
    let launch = if args.npx {
        report.line("ok", "agents will start Vexen through npx (needs Node.js 18+ every time)");
        Launch::npx()
    } else if let Some(command) = &args.command {
        report.line("ok", format!("agents will start `{command}`"));
        Launch::binary(command)
    } else {
        let path = stable_binary(&args, &home, &report)?;
        Launch::binary(&display_path(&path))
    };

    // 2. The project.
    println!("\n2. Project memory");
    let explicit_repo = args.repo.is_explicit();
    let project: Option<PathBuf> = match args.repo.root() {
        Ok(root) => Some(root),
        Err(e) if explicit_repo => return Err(e.to_string()),
        Err(_) => None,
    };
    match (&project, args.no_init) {
        (_, true) => report.line("skip", "not initializing a repository (--no-init)"),
        (None, _) => {
            report.line("skip", "not inside a Git repository; run `vexen setup` there, or `vexen init`, to start its memory");
            report.todo.push("Run `vexen init` inside the project you want Vexen to remember.".into());
        }
        (Some(root), _) => {
            let initialized = Store::open_read_only(root).is_ok();
            if initialized {
                report.line("ok", format!("{} is already initialized", display_path(root)));
            } else if dry {
                report.line("plan", format!("would initialize {}", display_path(root)));
            } else {
                match Store::initialize(root) {
                    Ok(()) => report.line("done", format!("initialized {} (notes stay in .vexen/, database files are git-ignored)", display_path(root))),
                    Err(e) => {
                        report.line("FAILED", format!("cannot initialize {}: {e}", display_path(root)));
                        report.problems.push(format!("project not initialized: {e}"));
                    }
                }
            }
        }
    }

    // 3. Agents: MCP entry and skill.
    println!("\n3. Agents");
    let mut chosen: Vec<&'static Adapter> = Vec::new();
    for id in &args.agents {
        if id == "auto" {
            chosen.extend(adapters().iter().filter(|a| a.detected(&home)));
        } else {
            chosen.push(find(id).ok_or_else(|| {
                let known: Vec<_> = adapters().iter().map(|a| a.id).collect();
                format!("unknown client `{id}`; known: {}", known.join(", "))
            })?);
        }
    }
    chosen.sort_by_key(|a| a.id);
    chosen.dedup_by_key(|a| a.id);
    let mut configured = 0usize;
    if chosen.is_empty() {
        report.line("NONE", format!("no supported agent found under {}", display_path(&home)));
        report.problems.push("no agent was configured".into());
        report.todo.push("Install a supported coding tool, or name one with `--agent <id>` (see `vexen install --help`).".into());
    }
    let repo_for_scope = project.clone().unwrap_or_else(|| home.clone());
    for adapter in &chosen {
        let scope = match args.scope {
            Some(ScopeArg::Global) => Scope::Global,
            Some(ScopeArg::Project) => Scope::Project,
            None if adapter.global.is_some() => Scope::Global,
            None => Scope::Project,
        };
        if scope == Scope::Project && project.is_none() {
            report.line("skip", format!("{}: needs a project; run setup inside a Git repository", adapter.label));
            report.problems.push(format!("{} not configured (project scope, no repository)", adapter.id));
            continue;
        }
        let outcome = run(&Request {
            adapter,
            scope,
            home: &home,
            repo: &repo_for_scope,
            launch: &launch,
            dry_run: dry,
            force: args.force,
            remove: false,
        })?;
        let label = format!("{} ({} scope)", adapter.label, if scope == Scope::Global { "global" } else { "project" });
        match outcome {
            Outcome::Created(p) | Outcome::Updated(p) => {
                configured += 1;
                report.line(if dry { "plan" } else { "done" }, format!("{label}: MCP entry {} in {}", if dry { "would be written" } else { "written" }, display_path(&p)));
                report.todo.push(format!("Restart {} so it reads the new MCP entry.", adapter.label));
            }
            Outcome::Unchanged(p) => {
                configured += 1;
                report.line("ok", format!("{label}: MCP entry already in {}", display_path(&p)));
            }
            Outcome::Removed(_) => {}
            Outcome::Refused { path, reason, snippet } => {
                report.line("FAILED", format!("{label}: not changed{}: {reason}", path.map(|p| format!(" ({})", display_path(&p))).unwrap_or_default()));
                report.problems.push(format!("{} needs a manual step", adapter.id));
                report.todo.push(format!("Add this to {}'s MCP configuration by hand:\n{snippet}", adapter.label));
            }
        }
        if !args.no_skill {
            skill_step(adapter, scope, &home, &repo_for_scope, &launch, &args, &mut report)?;
        }
    }

    // 4. A real server, started the way the agents will start it.
    println!("\n4. Does it work? (a real Vexen server process, not a mock)");
    if dry {
        report.line("plan", "would start the server, list its tools and make one check call");
    } else {
        let probe_root = project.as_deref().filter(|r| Store::open_read_only(r).is_ok());
        match probe_mcp(&launch, probe_root, Duration::from_secs(20)) {
            Ok(probe) => {
                report.line("ok", format!("server started; {} tools discovered ({})", probe.tools.len(), probe.tools.join(", ")));
                match (probe_root, &probe.check_status) {
                    (Some(root), Some(status)) => report.line("ok", format!("a vexen_check call on {} answered `{status}`", display_path(root))),
                    _ => report.line("skip", "no initialized repository to run a check on"),
                }
            }
            Err(e) => {
                report.line("FAILED", format!("the configured command does not give a working server: {e}"));
                report.problems.push("the server did not start correctly".into());
            }
        }
    }

    // Summary: never "ready"; the agent step is always unverified.
    println!("\nSummary");
    report.line(if report.problems.is_empty() { "ok" } else { "ATTN" }, if configured > 0 { format!("{configured} agent(s) configured") } else { "no agent configured".to_string() });
    report.line("not yet", "whether your agent loads the entry and chooses to call it. Vexen cannot check that from here.");
    if !report.todo.is_empty() {
        println!("\nNext steps");
        for (i, step) in report.todo.iter().enumerate() {
            println!("  {}. {step}", i + 1);
        }
    }
    if report.problems.is_empty() {
        println!("\nThen, in your agent's chat, try:\n  Use Vexen to check this project's README.md for past failures or decisions.\nOn a new project \"no relevant evidence\" is the expected answer.");
        println!("To see a lesson being saved and found without touching your project: `vexen demo`.");
        Ok(ExitCode::SUCCESS)
    } else {
        println!("\nNot everything was set up:");
        for p in &report.problems {
            println!("  - {p}");
        }
        Ok(ExitCode::from(2))
    }
}

fn skill_step(
    adapter: &Adapter,
    scope: Scope,
    home: &Path,
    repo: &Path,
    launch: &Launch,
    args: &SetupArgs,
    report: &mut Report,
) -> Result<(), String> {
    let Some(path) = skill::target(adapter.id, scope, home, repo) else {
        report.line("skip", format!("{}: no checked skill location; use `npx skills add OnurByte/Vexen` if your tool supports skills", adapter.label));
        return Ok(());
    };
    match skill::install(&path, &skill::render(launch), args.dry_run, args.force)? {
        SkillOutcome::Created(p) | SkillOutcome::Updated(p) => {
            report.line(if args.dry_run { "plan" } else { "done" }, format!("{}: skill {} at {}", adapter.label, if args.dry_run { "would be written" } else { "written" }, display_path(&p)));
        }
        SkillOutcome::Unchanged(p) => report.line("ok", format!("{}: skill already at {}", adapter.label, display_path(&p))),
        SkillOutcome::Foreign(p) => {
            report.line("FAILED", format!("{}: {} was changed by you, so it was left alone (use --force to replace it)", adapter.label, display_path(&p)));
            report.problems.push(format!("{} skill left unchanged", adapter.id));
        }
        SkillOutcome::Removed(_) | SkillOutcome::Absent(_) => {}
    }
    Ok(())
}

/// Where this binary should live so agent configuration keeps working.
fn stable_binary(args: &SetupArgs, home: &Path, report: &Report) -> Result<PathBuf, String> {
    let current = fs::canonicalize(std::env::current_exe().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let transient = !args.no_copy && is_transient(&current, home);
    if !transient {
        report.line("ok", format!("agents will start {}", display_path(&current)));
        return Ok(current);
    }
    let dir = args.install_dir.clone().unwrap_or_else(|| default_install_dir(home));
    let name = current.file_name().ok_or("the binary has no file name")?;
    let dest = dir.join(name);
    if dest == current {
        return Ok(current);
    }
    if args.dry_run {
        report.line("plan", format!("would copy this binary from {} to {} (it runs from a cache or temp folder)", display_path(&current), display_path(&dest)));
        return Ok(dest);
    }
    let same = fs::read(&dest).ok().zip(fs::read(&current).ok()).is_some_and(|(a, b)| Sha256::digest(a) == Sha256::digest(b));
    if same {
        report.line("ok", format!("{} is already this binary", display_path(&dest)));
        return Ok(dest);
    }
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let mut staged = dest.as_os_str().to_owned();
    staged.push(format!(".vexen-new-{}", std::process::id()));
    let staged = PathBuf::from(staged);
    let result = (|| -> Result<(), String> {
        fs::copy(&current, &staged).map_err(|e| format!("cannot copy to {}: {e}", staged.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&staged, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
        }
        let out = Command::new(&staged).arg("--version").output().map_err(|e| format!("the copy does not run: {e}"))?;
        let said = String::from_utf8_lossy(&out.stdout);
        let expected = format!("vexen {}", env!("CARGO_PKG_VERSION"));
        if !out.status.success() || said.trim() != expected {
            return Err(format!("the copy reports `{}`, expected `{expected}`", said.trim()));
        }
        fs::rename(&staged, &dest).map_err(|e| format!("cannot replace {} (is it running?): {e}", dest.display()))
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&staged);
        report.line("FAILED", format!("could not place the binary: {e}"));
        return Err(format!("could not place the binary in {}: {e}. Nothing else was changed", dir.display()));
    }
    report.line("done", format!("copied this binary to {} so agent configuration keeps working after the cache is cleared", display_path(&dest)));
    if !on_path(&dir) {
        report.line("note", format!("{} is not on your PATH. That is only needed to type `vexen` in a terminal; agents use the full path.", display_path(&dir)));
    }
    Ok(dest)
}

fn default_install_dir(home: &Path) -> PathBuf {
    if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("AppData").join("Local"))
            .join("Programs")
            .join("Vexen")
    } else {
        home.join(".local").join("bin")
    }
}

/// Binary caches and temp folders may be cleaned; agent configuration must not point there.
fn is_transient(exe: &Path, home: &Path) -> bool {
    let mut roots = vec![std::env::temp_dir()];
    if let Some(v) = std::env::var_os("VEXEN_HOME") {
        roots.push(PathBuf::from(v));
    } else if cfg!(windows) {
        if let Some(v) = std::env::var_os("LOCALAPPDATA") {
            roots.push(PathBuf::from(v).join("vexen"));
        }
    } else {
        let base = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".cache"));
        roots.push(base.join("vexen"));
    }
    roots.iter().filter_map(|r| fs::canonicalize(r).ok()).any(|r| exe.starts_with(r))
}

fn on_path(dir: &Path) -> bool {
    let want = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_owned());
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| fs::canonicalize(&d).unwrap_or(d) == want))
        .unwrap_or(false)
}
