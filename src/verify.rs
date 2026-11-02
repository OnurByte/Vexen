//! Protocol self-check: start the MCP server exactly as an agent would, then
//! discover its tools and make one real call.
//!
//! This proves that the command written into a client's configuration starts a
//! working server. It does not prove that any particular agent loaded that
//! configuration or chooses to call the tools; callers must say so.

use crate::install::Launch;
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// The four tools every Vexen server must expose.
pub const TOOLS: [&str; 4] = ["vexen_check", "vexen_explain", "vexen_outcome", "vexen_record"];

const MAX_LINE: usize = 1_000_000;

#[derive(Debug)]
pub struct Probe {
    /// Tool names the server listed, sorted.
    pub tools: Vec<String>,
    /// `status` of a real `vexen_check` call, when a repository was given.
    pub check_status: Option<String>,
}

/// Start `launch`, run `initialize`, `tools/list` and, if `repo_root` is given,
/// one `vexen_check` call. Always terminates the child. Never waits longer than
/// `timeout` overall.
pub fn probe_mcp(launch: &Launch, repo_root: Option<&Path>, timeout: Duration) -> Result<Probe, String> {
    let mut child = Command::new(&launch.command)
        .args(&launch.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot start `{}`: {e}", launch.command))?;
    let result = converse(&mut child, repo_root, timeout);
    let _ = child.kill();
    let _ = child.wait();
    result
}

fn converse(child: &mut Child, repo_root: Option<&Path>, timeout: Duration) -> Result<Probe, String> {
    let mut stdin = child.stdin.take().ok_or("no stdin to the server")?;
    let stdout = child.stdout.take().ok_or("no stdout from the server")?;
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines().take(256) {
            match line {
                Ok(l) if l.len() <= MAX_LINE => {
                    if tx.send(l).is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    });
    let deadline = Instant::now() + timeout;
    let mut send = |value: Value| -> Result<(), String> {
        writeln!(stdin, "{value}")
            .and_then(|()| stdin.flush())
            .map_err(|e| format!("the server closed its input: {e}"))
    };
    let receive = |id: u64| -> Result<Value, String> {
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let line = rx
                .recv_timeout(left)
                .map_err(|_| format!("no answer from the server within {} s", timeout.as_secs()))?;
            if let Ok(value) = serde_json::from_str::<Value>(&line)
                && value.get("id") == Some(&json!(id))
            {
                return Ok(value);
            }
        }
    };

    send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2025-11-25","capabilities":{},
        "clientInfo":{"name":"vexen-self-check","version":env!("CARGO_PKG_VERSION")}}}))?;
    let init = receive(1)?;
    if init.get("result").is_none() {
        return Err(format!("the server refused to initialize: {}", init.get("error").unwrap_or(&Value::Null)));
    }
    send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))?;
    let list = receive(2)?;
    let mut tools: Vec<String> = list["result"]["tools"]
        .as_array()
        .ok_or("the server returned no tool list")?
        .iter()
        .filter_map(|t| t["name"].as_str().map(str::to_owned))
        .collect();
    tools.sort();
    let missing: Vec<&str> = TOOLS.iter().copied().filter(|t| !tools.iter().any(|n| n == t)).collect();
    if !missing.is_empty() {
        return Err(format!("the server does not offer: {}", missing.join(", ")));
    }

    let mut check_status = None;
    if let Some(root) = repo_root {
        let root = std::fs::canonicalize(root).map_err(|e| format!("cannot resolve the repository: {e}"))?;
        send(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
            "name":"vexen_check","arguments":{"repo_root":root.to_string_lossy()}}}))?;
        let reply = receive(3)?;
        let result = reply.get("result").ok_or_else(|| format!("vexen_check failed: {}", reply["error"]))?;
        if result["isError"].as_bool() == Some(true) {
            return Err(format!("vexen_check returned an error: {}", result["content"][0]["text"].as_str().unwrap_or("?")));
        }
        check_status = result["structuredContent"]["status"].as_str().map(str::to_owned);
        if check_status.is_none() {
            return Err("vexen_check returned no status".into());
        }
    }
    Ok(Probe { tools, check_status })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn a_silent_server_times_out_instead_of_hanging() {
        let launch = Launch { command: "sleep".into(), args: vec!["30".into()] };
        let started = Instant::now();
        let err = probe_mcp(&launch, None, Duration::from_millis(600)).unwrap_err();
        assert!(err.contains("no answer"), "{err}");
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn a_missing_command_is_reported() {
        let launch = Launch { command: "definitely-not-a-real-command-vexen".into(), args: vec![] };
        assert!(probe_mcp(&launch, None, Duration::from_secs(2)).unwrap_err().contains("cannot start"));
    }
}
