use super::RepoArg;
use crate::install::{Adapter, Outcome, Request, Scope, adapters, find, run};
use std::{path::PathBuf, process::ExitCode};

#[derive(clap::Args)]
pub struct InstallArgs {
    /// Client id (repeatable) or `auto` for every client found under the home directory.
    #[arg(long = "agent", default_value = "auto")]
    agents: Vec<String>,
    /// Where to write: the user's home config or this repository's config.
    #[arg(long, value_enum, required = true)]
    scope: ScopeArg,
    /// Show what would change and write nothing.
    #[arg(long)]
    dry_run: bool,
    /// Print the config snippet for each client and write nothing.
    #[arg(long)]
    print_config: bool,
    /// Replace or remove a `vexen` entry that differs from what install writes.
    #[arg(long)]
    force: bool,
    /// Command clients should launch.
    #[arg(long, default_value = "vexen")]
    command: String,
    /// Home directory to use instead of the current user's (mainly for tests).
    #[arg(long)]
    home: Option<PathBuf>,
    #[command(flatten)]
    repo: RepoArg,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum ScopeArg {
    Global,
    Project,
}

pub fn run_install(args: InstallArgs, remove: bool) -> ExitCode {
    match go(args, remove) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("vexen: {error}");
            ExitCode::FAILURE
        }
    }
}

fn home_dir(arg: &Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(home) = arg {
        return Ok(home.clone());
    }
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "cannot find the home directory; pass --home".to_owned())
}

fn go(args: InstallArgs, remove: bool) -> Result<ExitCode, String> {
    let home = home_dir(&args.home)?;
    let scope = match args.scope {
        ScopeArg::Global => Scope::Global,
        ScopeArg::Project => Scope::Project,
    };
    let repo = match scope {
        Scope::Project => args.repo.root().map_err(|e| e.to_string())?,
        Scope::Global => home.clone(),
    };
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
    if chosen.is_empty() {
        println!("no supported client found under {}; pass --agent <id>", home.display());
        return Ok(ExitCode::SUCCESS);
    }
    let mut failed = false;
    for adapter in chosen {
        if args.print_config {
            println!("# {} ({})\n{}", adapter.label, adapter.support, crate::install::snippet(adapter, &args.command));
            continue;
        }
        let outcome = run(&Request {
            adapter,
            scope,
            home: &home,
            repo: &repo,
            command: &args.command,
            dry_run: args.dry_run,
            force: args.force,
            remove,
        })?;
        let tag = if args.dry_run { "would " } else { "" };
        match outcome {
            Outcome::Created(p) => println!("{}: {tag}created {}", adapter.id, p.display()),
            Outcome::Updated(p) => println!("{}: {tag}updated {}", adapter.id, p.display()),
            Outcome::Removed(p) => println!("{}: {tag}removed entry from {}", adapter.id, p.display()),
            Outcome::Unchanged(p) => println!("{}: unchanged {}", adapter.id, p.display()),
            Outcome::Refused { path, reason, snippet } => {
                failed = true;
                println!("{}: NOT changed{}: {reason}", adapter.id, path.map(|p| format!(" ({})", p.display())).unwrap_or_default());
                if !remove {
                    println!("add this by hand:\n{snippet}");
                }
            }
        }
    }
    Ok(if failed { ExitCode::from(2) } else { ExitCode::SUCCESS })
}
