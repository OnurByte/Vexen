use super::registry::{Adapter, Scope, Shape};
use serde_json::{Map, Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml_edit::{Array, DocumentMut, Item, Table, value};

const NAME: &str = "vexen";

/// What a client should run: a command plus its arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Launch {
    pub command: String,
    pub args: Vec<String>,
}

impl Launch {
    /// A `vexen` binary (by name on PATH or by absolute path).
    pub fn binary(command: &str) -> Self {
        Self {
            command: command.to_owned(),
            args: vec!["mcp".into(), "--stdio".into()],
        }
    }

    /// The npm launcher, for people who never installed the binary. Windows
    /// clients start `.cmd` shims through `cmd /c`.
    pub fn npx() -> Self {
        let mut args: Vec<String> = Vec::new();
        let command = if cfg!(windows) {
            args.extend(["/c".to_owned(), "npx".to_owned()]);
            "cmd"
        } else {
            "npx"
        };
        args.extend(["-y".to_owned(), "@onurbyte/vexen".to_owned(), "mcp".to_owned(), "--stdio".to_owned()]);
        Self { command: command.to_owned(), args }
    }

    fn argv(&self) -> Vec<&str> {
        std::iter::once(self.command.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect()
    }
}

pub struct Request<'a> {
    pub adapter: &'a Adapter,
    pub scope: Scope,
    pub home: &'a Path,
    pub repo: &'a Path,
    pub launch: &'a Launch,
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

pub fn snippet(adapter: &Adapter, launch: &Launch) -> String {
    match adapter.shape {
        Shape::Toml => format!(
            "[mcp_servers.{NAME}]\ncommand = {}\nargs = {}\n",
            serde_json::to_string(&launch.command).unwrap_or_default(),
            serde_json::to_string(&launch.args).unwrap_or_default()
        ),
        Shape::JsonOpenCode => pretty(&json!({"mcp": {"servers": {NAME: entry(adapter, launch, true)}}})),
        Shape::JsonServers { key, .. } => pretty(&json!({key: {NAME: entry(adapter, launch, false)}})),
        Shape::PrintOnly => pretty(&json!({NAME: {"command": launch.command, "args": launch.args}})),
    }
}

/// `nested` is OpenCode's current `mcp.servers.<name>` layout; the older
/// `mcp.<name>` layout also carries `enabled`.
fn entry(adapter: &Adapter, launch: &Launch, nested: bool) -> Value {
    match adapter.shape {
        Shape::JsonServers { typed: true, .. } => {
            json!({"type": "stdio", "command": launch.command, "args": launch.args})
        }
        Shape::JsonOpenCode if nested => {
            json!({"type": "local", "command": launch.argv()})
        }
        Shape::JsonOpenCode => {
            json!({"type": "local", "command": launch.argv(), "enabled": true})
        }
        _ => json!({"command": launch.command, "args": launch.args}),
    }
}

fn pretty(v: &Value) -> String {
    let mut s = serde_json::to_string_pretty(v).unwrap_or_default();
    s.push('\n');
    s
}

fn refuse(path: Option<PathBuf>, reason: impl Into<String>, adapter: &Adapter, launch: &Launch) -> Outcome {
    Outcome::Refused {
        path,
        reason: reason.into(),
        snippet: snippet(adapter, launch),
    }
}

pub fn run(req: &Request) -> Result<Outcome, String> {
    let adapter = req.adapter;
    if adapter.shape == Shape::PrintOnly {
        return Ok(refuse(
            None,
            format!("{} needs a manual edit ({})", adapter.label, adapter.support),
            adapter,
            req.launch,
        ));
    }
    let Some(path) = adapter.config_path(req.scope, req.home, req.repo) else {
        return Ok(refuse(
            None,
            format!("{} has no {:?} config location", adapter.label, req.scope),
            adapter,
            req.launch,
        ));
    };
    if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Ok(refuse(
            Some(path),
            "the config file is a symbolic link, so it was left untouched (edit its target by hand)",
            adapter,
            req.launch,
        ));
    }
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
        Err(reason) => return Ok(refuse(Some(path), reason, adapter, req.launch)),
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
/// A JSON value that refuses duplicate object keys. `serde_json::Value` keeps
/// the last one, which would silently drop the first on rewrite.
struct Strict(Value);

impl<'de> serde::Deserialize<'de> for Strict {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(StrictVisitor).map(Strict)
    }
}

struct StrictVisitor;

impl<'de> serde::de::Visitor<'de> for StrictVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("any JSON value")
    }
    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }
    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(Strict(v)) = seq.next_element()? {
            items.push(v);
        }
        Ok(Value::Array(items))
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut out = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if out.contains_key(&key) {
                return Err(serde::de::Error::custom(format!("duplicate key `{key}`")));
            }
            let Strict(v) = map.next_value()?;
            out.insert(key, v);
        }
        Ok(Value::Object(out))
    }
}

fn edit_json(existing: Option<&str>, req: &Request) -> Result<(String, bool), String> {
    let crlf = existing.is_some_and(|t| t.contains("\r\n"));
    let mut root: Value = match existing {
        None => Value::Object(Map::new()),
        Some(t) if t.trim().is_empty() => Value::Object(Map::new()),
        Some(t) => serde_json::from_str::<Strict>(t)
            .map(|s| s.0)
            .map_err(|e| format!("not strict JSON (comments, a syntax error or a duplicate key?), left untouched: {e}"))?,
    };
    let Value::Object(top) = &mut root else {
        return Err("top level is not a JSON object, left untouched".into());
    };
    let key = match req.adapter.shape {
        Shape::JsonOpenCode => "mcp",
        Shape::JsonServers { key, .. } => key,
        _ => unreachable!("caller filters other shapes"),
    };
    // OpenCode: use the current `mcp.servers` layout unless the file already
    // holds entries in the older `mcp.<name>` layout, which is then kept.
    let nested = matches!(req.adapter.shape, Shape::JsonOpenCode)
        && match top.get(key) {
            Some(Value::Object(m)) => m.contains_key("servers") || m.keys().all(|k| k == "timeout"),
            _ => true,
        };
    let wanted = entry(req.adapter, req.launch, nested);
    let unchanged = || Ok((existing.unwrap_or_default().to_owned(), false));
    let container = if req.remove {
        match top.get_mut(key) {
            None => return unchanged(),
            Some(Value::Object(m)) => m,
            Some(_) => return Err(format!("`{key}` is not an object, left untouched")),
        }
    } else {
        match top.entry(key).or_insert_with(|| Value::Object(Map::new())) {
            Value::Object(m) => m,
            _ => return Err(format!("`{key}` is not an object, left untouched")),
        }
    };
    let servers = if nested {
        if req.remove {
            match container.get_mut("servers") {
                None => return unchanged(),
                Some(Value::Object(m)) => m,
                Some(_) => return Err("`mcp.servers` is not an object, left untouched".into()),
            }
        } else {
            match container.entry("servers").or_insert_with(|| Value::Object(Map::new())) {
                Value::Object(m) => m,
                _ => return Err("`mcp.servers` is not an object, left untouched".into()),
            }
        }
    } else {
        container
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
    wanted.insert("command", value(req.launch.command.as_str()));
    let mut args = Array::new();
    for a in &req.launch.args {
        args.push(a.as_str());
    }
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
            let c = t.get("command").and_then(Item::as_str) == Some(req.launch.command.as_str());
            let a = t.get("args").and_then(Item::as_array).is_some_and(|a| {
                a.iter().map(|v| v.as_str()).eq(req.launch.args.iter().map(|s| Some(s.as_str())))
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

/// Mode bits of an existing file, so copies are never more readable than it.
#[cfg(unix)]
fn mode_of(path: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).ok().map(|m| m.permissions().mode() & 0o7777)
}

#[cfg(not(unix))]
fn mode_of(_path: &Path) -> Option<u32> {
    None
}

/// Create a file that must not exist yet, with the given mode from the first byte.
fn create_new(path: &Path, mode: Option<u32>) -> std::io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode.unwrap_or(0o666));
    }
    #[cfg(not(unix))]
    let _ = mode;
    options.open(path)
}

/// Back up once, then write via a fresh temp file in the same directory.
/// The backup is created exclusively (never overwritten), the temp file is
/// created exclusively with the original's mode, and the write is abandoned if
/// the config changed since it was read.
fn write_atomic(path: &Path, existing: Option<&str>, text: &str) -> Result<(), String> {
    use std::io::Write;
    let dir = path.parent().ok_or("config path has no parent")?;
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let mode = mode_of(path);
    if let Some(original) = existing {
        let mut backup = path.as_os_str().to_owned();
        backup.push(".vexen-backup");
        let backup = PathBuf::from(backup);
        match create_new(&backup, mode) {
            Ok(mut f) => f
                .write_all(original.as_bytes())
                .map_err(|e| format!("cannot back up {}: {e}", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(format!("cannot back up {}: {e}", path.display())),
        }
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".vexen-tmp-{}-{nanos}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    let mut file = create_new(&tmp, mode).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    let written = file.write_all(text.as_bytes()).and_then(|()| file.sync_all());
    drop(file);
    if let Err(e) = written {
        let _ = fs::remove_file(&tmp);
        return Err(format!("cannot write {}: {e}", tmp.display()));
    }
    // Lost-update guard: someone else saved the config while we were working.
    let now = match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            return Err(format!("cannot re-read {}: {e}", path.display()));
        }
    };
    if now.as_deref() != existing {
        let _ = fs::remove_file(&tmp);
        return Err(format!("{} changed while installing; nothing was written, run again", path.display()));
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("cannot replace {}: {e}", path.display())
    })
}
