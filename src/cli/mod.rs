mod check;
mod doctor;
mod export;
mod import;
mod init;
mod install;
mod observe;
mod record;

use clap::{Parser, Subcommand, ValueEnum};
use std::{
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

pub type CliResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(
    name = "vexen",
    version,
    about = "Local, evidence-based repository memory for coding agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create .vexen/ in a Git checkout (idempotent).
    Init(RepoArg),
    /// Serve the MCP protocol on stdio.
    Mcp {
        #[arg(long, required = true)]
        stdio: bool,
    },
    /// Look up applicable evidence before editing.
    Check {
        #[command(flatten)]
        repo: RepoArg,
        /// Git ref whose changed files are the query scope.
        #[arg(long)]
        diff: Option<String>,
        /// Repo-relative path to check (repeatable).
        #[arg(long = "path")]
        paths: Vec<String>,
        /// Free-text search over recorded summaries.
        #[arg(long)]
        query: Option<String>,
        /// Condition the query is made under (repeatable); must equal a note's conditions exactly.
        #[arg(long = "condition")]
        conditions: Vec<String>,
    },
    /// Show the provenance chain of a record.
    Explain {
        id: String,
        #[command(flatten)]
        repo: RepoArg,
    },
    /// Report store health without changing anything.
    Doctor(RepoArg),
    /// Write all events as versioned JSONL or readable Markdown to stdout.
    Export {
        #[arg(long, value_enum, default_value = "jsonl")]
        format: Format,
        #[command(flatten)]
        repo: RepoArg,
    },
    /// Save a failed attempt or an accepted decision, anchored to a file.
    Record {
        /// Repo-relative file the note is about.
        #[arg(long)]
        path: String,
        /// What was tried and what happened, in plain words (no secrets).
        #[arg(long)]
        summary: String,
        #[arg(long, value_enum, default_value = "attempt")]
        kind: record::Kind,
        /// Idempotency key; defaults to a hash of kind, file, content and summary.
        #[arg(long)]
        key: Option<String>,
        /// Condition under which the note applies (repeatable).
        #[arg(long = "condition")]
        conditions: Vec<String>,
        #[command(flatten)]
        repo: RepoArg,
    },
    /// Attach an observed test result (id and exit code only) to a record.
    Outcome {
        /// Record id printed by `vexen record`.
        id: String,
        #[arg(long)]
        test_id: String,
        #[arg(long, allow_hyphen_values = true)]
        exit_code: i32,
        /// Idempotency key; defaults to a hash of test id and exit code.
        #[arg(long)]
        key: Option<String>,
        /// SHA-256 of the test output, if you want to keep a fingerprint.
        #[arg(long)]
        output_sha256: Option<String>,
        #[command(flatten)]
        repo: RepoArg,
    },
    /// Run a test command you type, then record its exit code and an output hash on a record.
    Observe {
        /// Record id printed by `vexen record`.
        id: String,
        #[arg(long)]
        test_id: String,
        /// Idempotency key; defaults to a hash of test id, exit code and output hash.
        #[arg(long)]
        key: Option<String>,
        #[command(flatten)]
        repo: RepoArg,
        /// The command to run, after `--`.
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    /// Add Vexen's MCP entry to agent config files (previews with --dry-run).
    Install(install::InstallArgs),
    /// Remove only the MCP entry that `install` wrote.
    Uninstall(install::InstallArgs),
    /// Replay a JSONL export into this checkout; safe to repeat.
    Import {
        file: PathBuf,
        #[command(flatten)]
        repo: RepoArg,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Format {
    Jsonl,
    Markdown,
}

#[derive(clap::Args)]
pub struct RepoArg {
    /// Checkout root. Defaults to the checkout containing the current directory.
    #[arg(long)]
    repo: Option<PathBuf>,
}

impl RepoArg {
    pub fn root(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let start = match &self.repo {
            Some(path) => path.clone(),
            None => {
                let cwd = std::env::current_dir()?;
                let out = Command::new("git")
                    .arg("-C")
                    .arg(&cwd)
                    .args(["rev-parse", "--show-cdup"])
                    .output()?;
                if !out.status.success() {
                    return Err("not inside a Git checkout".into());
                }
                cwd.join(String::from_utf8_lossy(&out.stdout).trim())
            }
        };
        Ok(std::fs::canonicalize(start)?)
    }
}

/// Strip the Windows verbatim prefix so messages show ordinary paths.
pub fn display_path(path: &Path) -> String {
    let text = path_str(path);
    text.strip_prefix(r"\\?\").map(str::to_owned).unwrap_or(text)
}

pub fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Cmd::Init(repo) => init::run(&repo),
        Cmd::Mcp { .. } => serve(),
        Cmd::Check {
            repo,
            diff,
            paths,
            query,
            conditions,
        } => check::run(&repo, diff, paths, query, conditions),
        Cmd::Explain { id, repo } => check::explain(&repo, id),
        Cmd::Doctor(repo) => return doctor::run(&repo),
        Cmd::Observe { id, test_id, key, repo, command } => observe::run(&repo, id, test_id, key, command),
        Cmd::Install(args) => return install::run_install(args, false),
        Cmd::Uninstall(args) => return install::run_install(args, true),
        Cmd::Export { format, repo } => export::run(&repo, format),
        Cmd::Record {
            path,
            summary,
            kind,
            key,
            conditions,
            repo,
        } => record::record(&repo, path, summary, kind, key, conditions),
        Cmd::Outcome {
            id,
            test_id,
            exit_code,
            key,
            output_sha256,
            repo,
        } => record::outcome(&repo, id, test_id, exit_code, key, output_sha256),
        Cmd::Import { file, repo } => import::run(&repo, &file),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("vexen: {error}");
            ExitCode::FAILURE
        }
    }
}

fn serve() -> CliResult {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime
        .block_on(crate::mcp::serve_stdio())
        .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })
}
