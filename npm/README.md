# @onurbyte/vexen

Bootstrap for [Vexen](https://github.com/OnurByte/Vexen), local evidence-based memory for coding agents.

> **Status: not published to npm yet.** Release binaries for v0.1.0 exist on GitHub and the wrapper was verified against them on Windows, but the package itself is not on npm. Until then, download from the Releases page or build from source: `cargo install --git https://github.com/OnurByte/Vexen --locked`.

## What it does

`npx @onurbyte/vexen <args>` runs `vexen <args>`:

1. Picks the release asset for your platform (Windows x64, Linux x64/arm64, macOS x64/arm64).
2. Downloads `vexen-<version>-<target>.(tar.gz|zip)` and its `.sha256` from the matching GitHub release.
3. Refuses to run the archive unless the SHA-256 matches.
4. Unpacks to a per-version cache and runs the binary with your arguments.

Node 18+ is needed for this step only. Nothing is installed globally and no config files are touched.

```sh
npx @onurbyte/vexen init
npx @onurbyte/vexen doctor
npx @onurbyte/vexen check --diff HEAD
```

As an MCP server command (stdout carries only protocol messages; diagnostics go to stderr):

```json
{ "command": "npx", "args": ["-y", "@onurbyte/vexen", "mcp", "--stdio"] }
```

`vexen install` (client configuration) is not implemented yet.

## Environment

| Variable | Effect |
| --- | --- |
| `VEXEN_BINARY` | Run this binary and skip the download (offline installs). |
| `VEXEN_HOME` | Cache directory. Default: `%LOCALAPPDATA%\vexen` on Windows, `$XDG_CACHE_HOME/vexen` or `~/.cache/vexen` elsewhere. |

The only network access is the explicit download above, on first run of a version.

## Release checklist

1. Bump `version` in `Cargo.toml` and `npm/package.json` to the same value.
2. Tag `vX.Y.Z`. The release workflow builds the five targets and uploads archives with `.sha256` files.
3. Publish the package after the release assets exist (the workflow does this when the repository variable `PUBLISH_NPM` is `true` and the `NPM_TOKEN` secret is set).

The package name `@onurbyte/vexen` needs an npm account or organization named `onurbyte`; the unscoped name `vexen` is already taken by another project.
