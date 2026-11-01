Status: Design and execution plan, not an implemented product. This single document contains both the product specification and the implementation backlog. Planned commands, package names and performance targets are not claims about an existing release.
For implementers: Complete the tasks in order, test before implementing each behavior, review each finished milestone, and do not add features outside the defined scope without evidence from evaluation.
Goal: Build a fully free, open-source, local-first MCP tool that lets coding agents learn useful change relationships from Git history and reuse evidence-backed failed attempts across harnesses without flooding context or enforcing permissions.
Positioning: Your coding agents should learn from the repo, not repeat its mistakes.
Architecture: A single Rust executable provides the CLI, stdio MCP server, Git history analyzer, evidence checker, and installer. Multiple harness-launched processes access one SQLite WAL database per initialized checkout. Git-derived associations are advisory; explicitly observed attempts and outcomes form an append-only audit trail. Every lookup may abstain.
Stack: Rust, clap, official rmcp, tokio, rusqlite with bundled SQLite FTS5/WAL, serde, serde_json, thiserror, git CLI. GPL-3.0-or-later target, subject to dependency-license audit.
Spec: This document, sections 1–8. Delivery: Tasks 1–9, sections 9–11.
Coding agents lose important local experience when a session ends. A plain Git log preserves successful commits but not abandoned attempts; generic memory may supply irrelevant or stale advice and degrade task completion. Vexen must answer two concrete questions:
1. Change impact: “I am changing these files. Which other files might require checking, and why?”
2. Failed attempt: “Has a similar approach failed under comparable conditions? What evidence supports that observation?”
Must have: first-run value from existing Git history; compact results; truthful provenance; same repo state shared by Codex, Claude Code, Hermes, Cursor and other MCP clients; one-command installer; offline operation; versioned data portability.
Must not have: cloud, accounts, API keys, telemetry, dashboard, agent orchestrator, ticket tracker, vector DB, mandatory embeddings/LLM, always-on daemon, auto-executed tests, mandatory approval/policy gate, passive collection of entire chat transcripts, or a large agent-instruction prompt.
Vexen does not guarantee a bug will be prevented. “Co-changed before” is not “must always change together.” A passing test is evidence about one code state, not a universal causal proof. no_relevant_evidence is an acceptable and often desirable result.
- [PROJECTMEM](https://github.com/riponcm/projectmem): local event log and failed-approach warnings, already closely overlaps with Vexen.
- [GitHub Copilot's agentic memory](https://github.blog/ai-and-ml/github-copilot/building-an-agentic-memory-system-for-github-copilot/): code-cited, verified-at-use cross-agent memory; this is prior art, not a Vexen invention.
- [agentlog](https://github.com/ThunderbirdLabs/agentlog): transcript-derived unsuccessful attempts and deterministic source anchors.
- [adrkit](https://github.com/mbeacom/adrkit): Git-native decisions with affected-file relationships; interoperate, do not rebuild ADR governance.
- [add-mcp](https://github.com/neon-solutions/add-mcp) and [mcp-manager](https://github.com/xjeway/mcp-manager): broad multi-client config handling; study their format fixtures and compatibility edges.
Honest differentiation: a small, native, completely offline cross-harness package combining immediate Git change-impact hints, evidence-bound failure memory, aggressive abstention, reversible setup, and published negative-transfer evaluations. Not an invented memory algorithm.
Proposed, not currently published, CLI:
```sh
# Native package or release binary first; optional npx bootstrap only for install.
vexen install --agent auto --scope global
cd my-repo
vexen init
vexen doctor
vexen check --diff HEAD
vexen explain <record-id>
vexen export --format jsonl
vexen uninstall --agent auto
```
An optional npx @onurbyte/vexen or equivalent bootstrap may download a checksummed platform binary and call vexen install. The package name and registry availability must be checked before claiming that command works. Node is not needed after bootstrapping.
vexen install detects compatible clients and previews proposed config changes. Project/global scope is explicit. It preserves other MCP entries and comments where the format allows, makes a scoped backup, and can roll back only Vexen-owned modifications. No changes to auto-approval settings. vexen uninstall removes Vexen integration, not the project's memory; deleting memory requires a separate explicit command.
vexen init creates .vexen/ and a project-scoped SQLite store, with .vexen/.gitignore excluding state.sqlite and WAL/SHM files. No changes to root .gitignore; optional export can be committed separately. Existing Git history becomes the first searchable data. vexen doctor reports the detected clients, config locations, permissions, protocol handshake/tool-call compatibility, database integrity, and stale records without changing anything unless --fix was requested.
Demonstration: Agent A changes payments/retry.rs, runs a test which fails from duplicate charging, and records the attempted approach and test outcome. Agent B later opens the same repository and asks vexen_check before editing. Vexen returns at most three applicable items with source/commit/test references, or nothing if branch, code and conditions no longer support them. It never tells the agent it is forbidden to proceed.
Index ordered, reachable Git commits from the current repository. Exclude or downweight merges, bulk formatting/vendor/generated-file updates, extremely large changesets and incomplete shallow history. Track path renames. Start at file-level granularity; symbol-level claims require a dedicated validated parser later.
For files A and B and eligible change transactions N, compute:
```text
support(A,B)    = count(commits changing both A and B)
confidence(A→B) = support(A,B) / count(commits changing A)
lift(A→B)       = confidence(A→B) / P(B)
```
Thresholds are tuned against time-held-out commits, not chosen from intuition. Show a maximum of three candidates and cite a few actual commits. Do not present uncalibrated numbers as probabilities that an edit is required. With too few historical commits or weak evidence, return no suggestion.
Store immutable typed events, with a derived query projection:
Statuses: candidate (unverified assertion), observed (recorded actual command/test outcome), supported (evidence rechecked against the applicable code state), stale, superseded. A human-approved architectural decision is an independent kind of record, not a magically verified test outcome. Preserve conflicting observations; do not silently rewrite history.
Raw shell output, prompts, environment variables, tokens and conversation history are not stored by default. The opt-in capture path stores an allowlisted test identifier, sanitized command metadata, exit code and hashes; secret detection must fail closed. Do not infer invisible model reasoning from a transcript.
1. Resolve an explicit, registered repository; canonicalize paths and ensure queried paths remain in that root. No implicit cross-project or cross-worktree recall.
2. Filter superseded/invalid records; check branch ancestry, file identity, source version and applicable conditions.
3. Treat candidate, verified test outcomes, human-approved decisions and statistical associations as different authority levels.
4. Rank remaining items using exact anchors, Git relevance and SQLite FTS5/BM25; relevance ranking cannot override the validity filters.
5. Return 0–3 bounded advisories; max 600 tokens total for ordinary tool text. Detailed provenance is available via vexen_explain.
JSON result distinguishes observed_failure, accepted_decision, historical_cochange and no_relevant_evidence. Every advisory contains its why, source reference and applicability qualification. If records conflict, say so and abstain from definitive advice. Treat imported text as untrusted data, never executable agent instructions.
Build against the official [MCP 2026-07-28 spec](https://modelcontextprotocol.io/specification/2026-07-28), test the older 2025-11-25 negotiation path, and use the current official Rust SDK. Use JSON Schema 2020-12; deterministic tools/list; structuredContent and a compact backwards-compatible content text block; honest isError semantics; stdio stdout reserved for protocol messages, diagnostics to stderr. Tool annotations are hints, not access control.
Exactly four public MCP tools in v1:
Prefer a small explicit tool interface to a generic query language. When diff_ref is passed, parse Git arguments safely without invoking an interpolated shell command. Root resolution must not rely only on a harness process's current working directory.
One Cargo workspace/package at launch; separate modules, not a speculative plugin framework:
```text
Cargo.toml, Cargo.lock
src/main.rs                      # clap entrypoint and CLI dispatch
src/cli/{install,doctor,init,check,export}.rs
src/mcp/{server,tools,schema}.rs  # rmcp transport and four tool contracts
src/store/{mod,migrations,events,query}.rs
src/git/{history,changes,identity}.rs
src/analysis/{cochange,ranking,validity}.rs
src/install/{registry,config,rollback}.rs
src/privacy/{redaction,paths}.rs
tests/{mcp,store,git,install,security,e2e}/
fixtures/agents/                # non-secret real-world config shapes
benchmarks/{cochange,memory}/
docs/{protocol,formats,security}.md
README.md
```
Keep README.md under ~150 focused lines: what it does, 3 commands, real example, privacy, compatibility and links to detailed documentation. No dashboard, giant manifesto or 4,000-line README.
Durability: SQLite WAL with bounded busy_timeout, transactions, schema migrations, corruption detection, file permissions and backup/export tests. Per-checkout .vexen/state.sqlite is the working source of truth. JSONL export includes typed events and hashes; import is idempotent. Markdown export is portable human-readable documentation, not a lossy replacement for audit data. vexen init must not write outside the selected repo except explicit client configuration during install.
Start with 20+ versioned configuration adapters, each carrying detect, scope, config_path, format, render, merge, verify and remove behavior. Initial priority: Codex, Claude Code, Hermes, Cursor, OpenCode, Gemini CLI, VS Code, Copilot CLI, Cline, Windsurf, Zed and Kiro. Expand the matrix only when there are real fixtures. Pi requires an adapter and must not be advertised as native MCP.
Treat TOML, JSON, JSONC and YAML separately. Preserve unrelated keys, comments where possible, file mode and line endings. For an unsupported or ambiguous config format, emit a --print-config snippet and leave the file untouched. Check both global and project locations, client-specific trust requirements and alternate config generations. Use atomic replace plus owned-entry backups; make two consecutive installs idempotent. An uninstall must remove only the entry inserted by Vexen.
Support labels must be honest: config-tested, client-smoke-tested, full-task-tested, and requires-adapter. At least 12 real client smoke tests across supported OSes before calling the installer stable; 20+ config adapters are not synonymous with 20 live integrations.
Use 10–20 independently selected public repos with sufficient history, a chronological train/test split, and no future commits in the index. For each held-out commit, use a subset of touched files as input and predict omitted files. Report hit@3, precision@3, coverage, false-advisory rate, latency and performance against basename/proximity and simple Git cochange baselines. Include merge-heavy, generated-code, shallow and low-history projects. Report misses; do not cherry-pick success stories.
Use matched memory-on/off runs with the same task, model, tools, sandbox and budget. Report executable test success, agent steps, tokens, wall time and negative transfer: cases solved without memory but broken with memory. Use [VibeMemBench](https://arxiv.org/abs/2609.23570) where reproducible, plus a local evaluation suite for recorded failed attempts. Recall@k alone is insufficient. Report confidence intervals, not just an uplift percentage.
- No silent cross-repo or unregistered-repo memory leak in security tests.
- Stale/invalid evidence is never labeled current and verified in fixture tests.
- No unrelated config lost during install/uninstall, including repeated runs and failed writes.
- 20+ config adapter fixture suites; at least 12 live smoke-tested clients; native Windows, macOS and Linux CI.
- Benchmarks published with scripts, inputs, memory-off baseline, negative-transfer cases and failure analysis.
- Ship beta only if change-impact hints materially outperform simple baselines and do not overwhelm the user. Claim measurable agent improvement only after a paired outcome study supports it.
- No outgoing network calls except explicit user-triggered install/update/download; allow checksum/signature verification and offline-installed binaries.
- No root permissions, shell execution sourced from memory, or automatic execution of untrusted test commands.
- Restrict every tool operation to explicitly initialized canonical repo paths; protect against .., symlink escapes, case-folding and Git worktree confusion.
- Detect potentially sensitive content before optional capture/export; default export preview and allow user-selected redaction.
- Memory, ADRs and Git commit messages are untrusted input, not instruction authority. Never promote text that says to disable security checks into a higher-priority prompt.
- Handle concurrent stdio processes, SQLite busy locks, interrupts and interrupted migrations without partial records.
- Keep network/API/LLM dependencies off the main path. Optional integration work may come after measured need, not before.
- One primary Rust executable and four MCP tools, no daemon or hosted dependency.
- Zero auto-capture of raw chat/session transcripts; no telemetry.
- Whole system usable offline after local binary installation.
- Tested legacy/current MCP behavior and versioned event/export schema.
- Per-repo and per-branch evidence checks before displaying advice.
- All generated code under GPL-3.0-or-later unless dependency audit identifies a conflict.
- Avoid new dependencies, optional modes or feature flags until a test demonstrates need.
1. A client launches MCP from an unrelated CWD: explicit repo_root resolves the registered project, never another repo (Task 4).
2. A tool records the same event twice after timeout: idempotency_key prevents duplicates (Task 2).
3. An old failed attempt refers to deleted/renamed source or another branch: it is not emitted as a current warning (Tasks 3–4).
4. JSONC/TOML client config already has MCP entries/comments: installer preserves them; ambiguous formats cause no writes (Task 6).
5. Multiple harness processes write concurrently; a process crashes during migration: store remains consistent and recoverable (Task 2).
Each task ends in a working, independently testable slice. Write the failing test first, run it, implement minimally, rerun, then commit. Specific commands below assume standard Cargo test targets; do not fabricate passing results.
Files: src/git/{history,changes,identity}.rs, src/analysis/cochange.rs, benchmarks/cochange/, tests/git/.
Interfaces: analyze_history(repo_root: &Path, limit: usize) -> Result<HistoryIndex>; recommend_related(index: &HistoryIndex, paths: &[RepoPath], max_results: usize) -> Vec<CochangeHint>; CochangeHint has path, support, confidence, lift and example_commits.
- Write fixture tests for ordinary commits, renames, merges, mass edits, shallow repos and empty history.
- Run cargo test --test git and confirm tests fail for the unimplemented interfaces.
- Implement safe Git history parsing and change-transaction filtering.
- Implement file-pair counts and ranked associations with explicit no-result behavior.
- Build chronological benchmark with baseline comparison; report accuracy and false hints.
- Run cargo test --test git and the benchmark; commit only after evidence is captured.
Gate: If predictive value does not beat simple baselines, remove the default cochange feature and revisit positioning before implementing further scope.
Files: src/store/{mod,migrations,events,query}.rs, tests/store/.
Interfaces: Store::open(repo_root: &Path) -> Result<Store>; Store::append(event: NewEvent) -> Result<EventId>; Store::outcome(id: EventId, data: NewOutcome) -> Result<EventId>; Store::get(id: EventId) -> Result<EventChain>; Store::search(query: StoreQuery) -> Result<Vec<MemoryRecord>>.
- Add tests for migration, append-only history, duplicate idempotency keys, concurrent writes and crash-safe reopen.
- Run cargo test --test store to verify expected failures.
- Implement SQLite schema, transaction boundaries, WAL, FTS5 indexing and derived record projection.
- Run cargo test --test store; document schema version and migration safety.
- Commit passing tested store slice.
Files: src/analysis/{validity,ranking}.rs, src/privacy/{redaction,paths}.rs, tests/security/, tests/store/.
Interfaces: check_applicability(record: &MemoryRecord, ctx: &RepoContext) -> Eligibility; rank_applicable(records: &[MemoryRecord], ctx: &RepoContext, limit: usize) -> Vec<Advisory>; Eligibility is Eligible, Stale, Superseded, WrongScope, or MissingEvidence.
- Write tests for branch drift, hash change, deleted path, unrelated project, conflicting records, untrusted strings and empty eligible set.
- Verify failures with cargo test --test security.
- Implement deterministic anchors, hard eligibility filtering, redaction and bounded result length.
- Confirm zero false “verified” labels in invalid-evidence fixtures and run tests.
- Commit eligible-only retrieval behavior.
Files: src/mcp/{server,tools,schema}.rs, tests/mcp/.
Interfaces: MCP tool names and inputs from §4; map each to Store or recommend_related without a second data model. All outputs share schema_version, repo_id, items, status and optional reason.
- Write tool discovery/call tests for both supported MCP protocol generations, output schemas, read-only behavior and failures.
- Confirm tests fail; implement vexen_check, vexen_record, vexen_outcome, vexen_explain.
- Test incorrect CWD, explicit repo root, stdout cleanliness, JSON compatibility and 600-token budget.
- Run cargo test --test mcp; manually verify with MCP Inspector.
- Commit a working native stdio server usable with a hand-written client config.
Files: src/main.rs, src/cli/{init,check,doctor,export}.rs, tests/e2e/.
Interfaces: vexen init, vexen mcp --stdio, vexen check --diff <git-ref>, vexen doctor, vexen export --format jsonl|markdown, vexen import <file>.
- Test init idempotency, normal worktree, detached HEAD, bare repo rejection, malformed import and export/import round trip.
- Run failing end-to-end CLI tests.
- Wire CLI to existing Git, store and MCP modules; implement lossless versioned JSONL and readable Markdown export.
- Run CLI and e2e tests on a temporary fixture repository; check no unrelated files change.
- Commit documented first-run functionality.
Files: src/install/{registry,config,rollback}.rs, src/cli/install.rs, fixtures/agents/, tests/install/.
Interfaces: detect_clients() -> Vec<ClientInstallation>; preview_install(target: &ClientInstallation, scope: Scope) -> Result<ConfigChange>; apply_install(change: &ConfigChange) -> Result<Receipt>; rollback(receipt: &Receipt) -> Result<()>.
- Add client fixtures by exact app version and config format; start with priority clients in §6.
- Test comment/key preservation, duplicate install, mixed config versions, inaccessible targets, recovery from interrupted writes and owned-entry uninstall.
- Verify failing tests; implement adapter registry and safe atomic writes.
- Add 20+ fixture-tested adapters; label each native/adapter and smoke-test state honestly.
- Run cargo test --test install; smoke test 12 real clients before stable claim.
- Commit installer with --dry-run, --print-config, scoped rollback and standalone uninstall.
Files: src/cli/observe.rs, src/privacy/redaction.rs, tests/security/, tests/e2e/.
Interfaces: vexen observe -- <user-command> is explicitly invoked; only allowlisted test metadata, exit code and a digest are stored. Client-specific hooks, if supported, invoke this path only after explicit user opt-in.
- Add tests for a passing test, failing test, sensitive argv, terminal interruption, subprocess failure and no raw log retention.
- Prove tests fail; implement local-only subprocess wrapper with argument arrays (no shell string interpolation).
- Confirm the wrapped command returns the original exit code; fail closed on unsafe capture.
- Run tests and commit optional capture. Keep automatic transcript extraction out of v1.
Files: .github/workflows/ci.yml, .github/workflows/release.yml, docs/{protocol,formats,security}.md, README.md, tests/e2e/.
- Set up Linux/macOS/Windows matrix for formatting, clippy, unit/contract/e2e tests and packaging.
- Verify fresh offline launch from downloaded binary; no node, Python, account or daemon needed at runtime.
- Add reproducibility information, SHA-256/checksums, license inventory and supply-chain review.
- Document privacy threat model, local data location, complete uninstall, known client limitations and compatibility tiers.
- Keep README short, with two real examples and link to detailed docs.
- Release only after tests pass on all three operating systems.
Files: benchmarks/memory/, benchmarks/cochange/, docs/benchmarks.md.
- Freeze representative coding tasks, agent versions, memory records, budgets and executable test runners before experiments.
- Run memory-off, valid-memory, stale-memory and irrelevant-memory conditions with matching seeds where possible.
- Compute per-task resolved rates, paired deltas, negative transfer, context cost, abstention and confidence intervals.
- Publish scripts, raw aggregated results, benchmark limitations and negative examples.
- Decide ship / narrow / stop from measured outcomes; do not claim a success uplift without statistical support.
The estimates are planning placeholders, not deadlines. Tasks 1–6 yield a usable beta; Tasks 7–9 harden and validate it. Do not introduce a dashboard, semantic embeddings, distributed sync or multi-agent orchestration to rescue a failed benchmark.
- [VibeMemBench, 2026](https://arxiv.org/abs/2609.23570) — memory availability is not the same as coding-task improvement; matched memory-on/off outcomes and negative transfer.
- [ROSE: Mining Version Histories to Guide Software Changes, 2004](https://www.st.cs.uni-saarland.de/papers/icse2004/icse.pdf) — co-change discovery; historical results do not automatically generalize.
- [Is code co-committal an indicator of evolutionary coupling?, 2026](https://pure.qub.ac.uk/en/publications/is-code-co-committal-an-indicator-of-evolutionary-coupling-in-sof/) — cochange validity depends on repository/developer behavior.
- [SelectiveNet, ICML 2019](https://proceedings.mlr.press/v97/geifman19a.html) — risk–coverage and rejection option; inspiration for abstention, not a proof of a Vexen guarantee.
- [MCP 2026-07-28 specification](https://modelcontextprotocol.io/specification/2026-07-28) and [official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk) — protocol interoperability.
- [SQLite FTS5](https://www.sqlite.org/fts5.html) and [WAL](https://www.sqlite.org/wal.html) — built-in search and concurrency semantics.
- [CodeGraph installer documentation](https://github.com/colbymchenry/codegraph/blob/main/site/src/content/docs/getting-started/installation.md) — installer ergonomics, not a requirement to replicate CodeGraph's index.
Final rule: If Vexen merely remembers more things while making agents worse, it is not a product worth shipping. Improve the evidence, applicability and evaluated task outcomes first.