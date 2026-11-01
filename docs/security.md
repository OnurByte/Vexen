# Security and privacy model

This describes what the code does today. It is not an audit.

## Data Vexen stores

Records you or your agent save: a short summary, a repo-relative file path, that file's SHA-256, an optional Git commit id and conditions. Outcomes: a test identifier, an exit code and an optional output hash. Nothing else.

It does not store prompts, chat transcripts, environment variables, tokens, raw command output or credentials. `vexen observe` shows a command's output to you and stores only its SHA-256.

## Network

Vexen makes no network calls and has no telemetry. The only network use in this repository is the optional npm launcher, which downloads a release archive on first run and verifies its SHA-256 before executing anything.

## Input handling

- Secret-like text in a summary, a condition or the anchored file is refused (`privacy::contains_secret`); import applies the same check. Detection is heuristic and can miss things, so do not rely on it as the only safeguard.
- Paths must be repo-relative and are canonicalized; `..`, absolute paths, symlink escapes and hidden or secret-named files are rejected.
- `vexen_check` with `diff_ref` passes Git arguments without a shell.
- Recorded text, imported files, ADRs and commit messages are untrusted data. Vexen returns them as quoted evidence and never treats them as instructions. A note that says "disable checks" is just a note.
- Queries are bound to the explicitly named repository; there is no cross-repository recall.

## Concurrency and failure

Multiple processes can use one store. Writes are immediate transactions with a busy timeout, and an interrupted migration or write leaves the previous consistent state.

## Client configuration

`vexen install` previews with `--dry-run`, writes only the `vexen` entry, never changes auto-approval or permission settings, backs a file up once as `<file>.vexen-backup`, writes atomically, and refuses (printing a snippet) when it cannot edit a file losslessly, such as JSON with comments. `vexen uninstall` removes only an entry identical to what install writes. Config formats are checked against fixtures, not against live clients.

## Known limits

- Client support: Claude Code 2.1.309 on Windows is `client-smoke-tested` (one real session started the server from an installer-written config and a `vexen_check` call returned the recorded failure). Every other client is `config-tested` only: formats are covered by tests, nothing live. A Codex check could not run because the Codex build used for it could not call any tools.
- Re-serializing a JSON config normalizes its whitespace and may alter numbers beyond 64-bit precision.
- Release archives are checksummed (SHA-256), and carry GitHub build-provenance attestations when the attestation step succeeds: `gh attestation verify <archive> --repo OnurByte/Vexen`. They are not signed with a project-held key.
- Windows and macOS each skip one test fixture the OS cannot create (see the README).

## Reporting a problem

Open a GitHub issue for ordinary bugs. For a vulnerability, avoid posting details publicly; contact the repository owner through their GitHub profile.
