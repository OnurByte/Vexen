use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope {
    Global,
    Project,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Shape {
    /// `{ "<key>": { "vexen": { "command", "args" } } }`
    JsonServers { key: &'static str, typed: bool },
    /// OpenCode: `{ "mcp": { "vexen": { "type": "local", "command": [..] } } }`
    JsonOpenCode,
    /// Codex: `[mcp_servers.vexen]`
    Toml,
    /// No safe automatic edit; print a snippet only.
    PrintOnly,
}

/// One client. Paths are relative to the home directory (global) or the
/// repository root (project).
#[derive(Clone, Copy, Debug)]
pub struct Adapter {
    pub id: &'static str,
    pub label: &'static str,
    pub shape: Shape,
    pub project: Option<&'static str>,
    pub global: Option<&'static str>,
    /// Home-relative path whose existence suggests the client is installed.
    pub detect: Option<&'static str>,
    /// What has actually been checked. Never "live-tested" without a live test.
    pub support: &'static str,
}

const SERVERS: Shape = Shape::JsonServers {
    key: "mcpServers",
    typed: false,
};

const ALL: &[Adapter] = &[
    Adapter {
        id: "claude-code",
        label: "Claude Code",
        shape: SERVERS,
        project: Some(".mcp.json"),
        global: Some(".claude.json"),
        detect: Some(".claude"),
        support: "config-tested",
    },
    Adapter {
        id: "codex",
        label: "Codex CLI",
        shape: Shape::Toml,
        project: None,
        global: Some(".codex/config.toml"),
        detect: Some(".codex"),
        support: "config-tested",
    },
    Adapter {
        id: "cursor",
        label: "Cursor",
        shape: SERVERS,
        project: Some(".cursor/mcp.json"),
        global: Some(".cursor/mcp.json"),
        detect: Some(".cursor"),
        support: "config-tested",
    },
    Adapter {
        id: "gemini",
        label: "Gemini CLI",
        shape: SERVERS,
        project: Some(".gemini/settings.json"),
        global: Some(".gemini/settings.json"),
        detect: Some(".gemini"),
        support: "config-tested",
    },
    Adapter {
        id: "windsurf",
        label: "Windsurf",
        shape: SERVERS,
        project: None,
        global: Some(".codeium/windsurf/mcp_config.json"),
        detect: Some(".codeium/windsurf"),
        support: "config-tested",
    },
    Adapter {
        id: "kiro",
        label: "Kiro",
        shape: SERVERS,
        project: Some(".kiro/settings/mcp.json"),
        global: Some(".kiro/settings/mcp.json"),
        detect: Some(".kiro"),
        support: "config-tested",
    },
    Adapter {
        id: "vscode",
        label: "VS Code",
        shape: Shape::JsonServers {
            key: "servers",
            typed: true,
        },
        project: Some(".vscode/mcp.json"),
        global: None,
        detect: None,
        support: "config-tested",
    },
    Adapter {
        id: "opencode",
        label: "OpenCode",
        shape: Shape::JsonOpenCode,
        project: Some("opencode.json"),
        global: Some(".config/opencode/opencode.json"),
        detect: Some(".config/opencode"),
        support: "config-tested",
    },
    Adapter {
        id: "zed",
        label: "Zed",
        shape: Shape::PrintOnly,
        project: None,
        global: None,
        detect: None,
        support: "requires-adapter (settings.json allows comments)",
    },
    Adapter {
        id: "cline",
        label: "Cline",
        shape: Shape::PrintOnly,
        project: None,
        global: None,
        detect: None,
        support: "requires-adapter",
    },
];

pub fn adapters() -> &'static [Adapter] {
    ALL
}

pub fn find(id: &str) -> Option<&'static Adapter> {
    ALL.iter().find(|a| a.id == id)
}

impl Adapter {
    pub fn config_path(&self, scope: Scope, home: &Path, repo: &Path) -> Option<PathBuf> {
        match scope {
            Scope::Global => self.global.map(|p| home.join(p)),
            Scope::Project => self.project.map(|p| repo.join(p)),
        }
    }

    pub fn detected(&self, home: &Path) -> bool {
        self.detect.is_some_and(|p| home.join(p).exists())
    }
}
