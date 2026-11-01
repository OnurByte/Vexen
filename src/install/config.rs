use super::registry::{Adapter, Scope, Shape};
use serde_json::{Map, Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml_edit::{Array, DocumentMut, Item, Table, value};

const NAME: &str = "vexen";

pub struct Request<'a> {
    pub adapter: &'a Adapter,
    pub scope: Scope,
    pub home: &'a Path,
    pub repo: &'a Path,
    pub command: &'a str,
    pub dry_run: bool,
    pub force: bool,
    pub remove: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Outcome {
    Created(PathBuf),
    Updated(PathBuf),
    Removed(PathBuf),
    Unchanged(PathBuf),
    /// Nothing was written; the reason and a snippet to add by hand.
    Refused {
        path: Option<PathBuf>,
        reason: String,
        snippet: String,
    },
}

pub fn snippet(adapter: &Adapter, command: &str) -> String {
    match adapter.shape {
        Shape::Toml => format!(
            "[mcp_servers.{NAME}]\ncommand = {}\nargs = [\"mcp\", \"--stdio\"]\n",
            serde_json::to_string(command).unwrap_or_default()
        ),
        Shape::JsonOpenCode => pretty(&json!({"mcp": {NAME: entry(adapter, command)}})),
        Shape::JsonServers { key, .. } => pretty(&json!({key: {NAME: entry(adapter, command)}})),
        Shape::PrintOnly => pretty(&json!({NAME: {"command": command, "args": ["mcp", "--stdio"]}})),
    }
}

fn entry(adapter: &Adapter, command: &str) -> Value {
    match adapter.shape {
        Shape::JsonServers { typed: true, .. } => {
            json!({"type": "stdio", "command": command, "args": ["mcp", "--stdio"]})
        }
        Shape::JsonOpenCode => {
            json!({"type": "local", "command": [command, "mcp", "--stdio"], "enabled": true})
        }
        _ => json!({"command": command, "args": ["mcp", "--stdio"]}),
    }
}

fn pretty(v: &Value) -> String {
    let mut s = serde_json::to_string_pretty(v).unwrap_or_default();
    s.push('\n');
    s
}

fn refuse(path: Option<PathBuf>, reason: impl Into<String>, adapter: &Adapter, command: &str) -> Outcome {
    Outcome::Refused {
        path,
        reason: reason.into(),
        snippet: snippet(adapter, command),
    }
}

pub fn run(req: &Request) -> Result<Outcome, String> {
    let adapter = req.adapter;
    if adapter.shape == Shape::PrintOnly {
        return Ok(refuse(
            None,
            format!("{} needs a manual edit ({})", adapter.label, adapter.support),
            adapter,
            req.command,
        ));
    }
    let Some(path) = adapter.config_path(req.scope, req.home, req.repo) else {
        return Ok(refuse(
            None,
            format!("{} has no {:?} config location", adapter.label, req.scope),
            adapter,
            req.command,
        ));
    };
    let existing = match fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    let edited = match adapter.shape {
        Shape::Toml => edit_toml(existing.as_deref(), req),
        _ => edit_json(existing.as_deref(), req),
    };
    let (new_text, changed) = match edited {
        Ok(v) => v,
        Err(reason) => return Ok(refuse(Some(path), reason, adapter, req.command)),
    };
    if !changed {
        return Ok(Outcome::Unchanged(path));
    }
    if !req.dry_run {
        write_atomic(&path, existing.as_deref(), &new_text)?;
    }
    Ok(match (req.remove, existing.is_some()) {
        (true, _) => Outcome::Removed(path),
        (false, true) => Outcome::Updated(path),
        (false, false) => Outcome::Created(path),
    })
}

/// Returns the new file text and whether it differs from the old one.
fn edit_json(existing: Option<&str>, req: &Request) -> Result<(String, bool), String> {
    let crlf = existing.is_some_and(|t| t.contains("\r\n"));
    let mut root: Value = match existing {
        None => Value::Object(Map::new()),
        Some(t) if t.trim().is_empty() => Value::Object(Map::new()),
        Some(t) => serde_json::from_str(t).map_err(|e| {
            format!("not strict JSON (comments or a syntax error?), left untouched: {e}")
        })?,
    };
    let Value::Object(top) = &mut root else {
        return Err("top level is not a JSON object, left untouched".into());
    };
    let key = match req.adapter.shape {
        Shape::JsonOpenCode => "mcp",
        Shape::JsonServers { key, .. } => key,
        _ => unreachable!("caller filters other shapes"),
    };
    let wanted = entry(req.adapter, req.command);
    let servers = if req.remove {
        match top.get_mut(key) {
            None => return Ok((existing.unwrap_or_default().to_owned(), false)),
            Some(Value::Object(m)) => m,
            Some(_) => return Err(format!("`{key}` is not an object, left untouched")),
        }
    } else {
        match top.entry(key).or_insert_with(|| Value::Object(Map::new())) {
            Value::Object(m) => m,
            _ => return Err(format!("`{key}` is not an object, left untouched")),
        }
    };
    let changed = apply(servers.get(NAME), &wanted, req)?;
    if !changed {
        return Ok((existing.unwrap_or_default().to_owned(), false));
    }
    if req.remove {
        servers.remove(NAME);
    } else {
        servers.insert(NAME.to_owned(), wanted);
    }
    let mut text = pretty(&root);
    if crlf {
        text = text.replace('\n', "\r\n");
    }
    Ok((text, true))
}

/// Shared decision: is a write needed, and is it safe?
fn apply<T: PartialEq<T>>(current: Option<&T>, wanted: &T, req: &Request) -> Result<bool, String> {
    match (current, req.remove) {
        (None, true) => Ok(false),
        (None, false) => Ok(true),
        (Some(c), true) => {
            if c == wanted || req.force {
                Ok(true)
            } else {
                Err("the `vexen` entry differs from what install writes, so it is not ours to remove (use --force)".into())
            }
        }
        (Some(c), false) => {
            if c == wanted {
                Ok(false)
            } else if req.force {
                Ok(true)
            } else {
                Err("a different `vexen` entry already exists; left untouched (use --force to replace it)".into())
            }
        }
    }
}

fn edit_toml(existing: Option<&str>, req: &Request) -> Result<(String, bool), String> {
    let original = existing.unwrap_or_default();
    let mut doc: DocumentMut = original
        .parse()
        .map_err(|e| format!("not valid TOML, left untouched: {e}"))?;
    let mut wanted = Table::new();
    wanted.insert("command", value(req.command));
    let mut args = Array::new();
    args.push("mcp");
    args.push("--stdio");
    wanted.insert("args", value(args));

    let servers = if req.remove {
        match doc.get_mut("mcp_servers") {
            None => return Ok((original.to_owned(), false)),
            Some(item) => item
                .as_table_mut()
                .ok_or("`mcp_servers` is not a table, left untouched")?,
        }
    } else {
        let item = doc.entry("mcp_servers").or_insert_with(|| {
            let mut t = Table::new();
            t.set_implicit(true);
            Item::Table(t)
        });
        item.as_table_mut()
            .ok_or("`mcp_servers` is not a table, left untouched")?
    };
    let current = servers.get(NAME).map(|i| match i.as_table_like() {
        Some(t) => t.iter().count() == 2 && {
            let c = t.get("command").and_then(Item::as_str) == Some(req.command);
            let a = t.get("args").and_then(Item::as_array).is_some_and(|a| {
                a.iter().map(|v| v.as_str()).eq([Some("mcp"), Some("--stdio")])
            });
            c && a
        },
        None => false,
    });
    // Shared policy over booleans: does the existing entry match what we write?
    let changed = apply(current.as_ref(), &true, req)?;
    if !changed {
        return Ok((original.to_owned(), false));
    }
    if req.remove {
        servers.remove(NAME);
    } else {
        servers.insert(NAME, Item::Table(wanted));
    }
    Ok((doc.to_string(), true))
}

/// Back up once, then write via a temp file in the same directory.
fn write_atomic(path: &Path, existing: Option<&str>, text: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("config path has no parent")?;
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    if existing.is_some() {
        let mut backup = path.as_os_str().to_owned();
        backup.push(".vexen-backup");
        let backup = PathBuf::from(backup);
        if !backup.exists() {
            fs::copy(path, &backup).map_err(|e| format!("cannot back up {}: {e}", path.display()))?;
        }
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".vexen-tmp-{}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, text).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    if let Ok(meta) = fs::metadata(path) {
        let _ = fs::set_permissions(&tmp, meta.permissions());
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("cannot replace {}: {e}", path.display())
    })
}
