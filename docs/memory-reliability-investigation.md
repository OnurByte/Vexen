# Vexen memory reliability: investigation and fixes

Updated: 2026-11-02. Status: blocked on Claude Code authentication for the fresh real-agent evaluation.

## Goal and acceptance

The requested goal is to make Vexen complete the task set successfully. The existing five targeted Python tasks and their hidden assertions must remain unchanged. A successful product check must deliver the applicable lesson before editing; the real agent must then pass the hidden test. Filtering stale or unrelated records must continue to work. Do not relabel a deterministic fixture check as a real-agent result or replace historical pilot data with new results.

Completion requires a fresh, fully reported real-agent evaluation across all targeted tasks and repetitions, plus the stale/unrelated controls and relevant product regression checks. No such result has been obtained yet. Passing this suite would still not establish perfect performance on arbitrary tasks.

## Evidence from the historical pilot

Source: `benchmarks/memory/results/pilot-2.jsonl`, 76 total rows.

| Target-task condition / last response | Runs | Passed | Failed |
|---|---:|---:|---:|
| Memory off | 20 | 2 | 18 |
| Memory on, last status `observed_failure` | 12 | 11 | 1 |
| Memory on, last status `no_relevant_evidence` | 8 | 0 | 8 |

The eight enabled runs with an empty last response were: BOM rep 0; rounding reps 1 and 2; slug rep 0; dates reps 1 and 3; CSV reps 1 and 2. The failed enabled run with a failure record was slug rep 3.

These rows do not prove why a response was empty or why a task failed. `run.py` keeps only the last detected status and deletes the temporary repository, without retaining the agent's full tool sequence. The CSV rep 1 run made two check calls; its last empty response does not establish that it never received a note.

## Confirmed product issue: text-only MCP clients lose the lesson

In `src/mcp/server.rs`, `response` produces both `structuredContent` and a text block. The structured result includes each advisory's `text`. `compact_text` retains IDs and statuses but omits `text` entirely. A client exposing only text therefore receives a record indicator without the failure lesson it needs to apply.

This is independently visible in the source, but it is not yet established as the cause of any specific historical failure. The old evaluation's exact tested binary and full transcripts are unavailable in its row data.

Correction implemented: the compact text now includes the already filtered, qualified advisory text. It preserves status and record identity, and shortens the longest lesson on Unicode character boundaries when necessary to stay within the existing 1,700-byte serialized text budget. Shortened lessons are marked `text_truncated`; the structured result remains unchanged.

Two regression tests first failed on the old behavior, then passed after the fix. An integration test also checks the full store → applicability → MCP response path and verifies that changing the anchor still suppresses the old lesson.

## Confirmed evaluation observability gaps

- The parser searches substrings in every tool result rather than matching `tool_use_id` to a Vexen check invocation. A note containing a status word can confuse classification.
- Only the last detected check status is saved. Earlier successful retrievals and their timing relative to edits disappear.
- Agent stdout/stderr and the task workspace are discarded, preventing reconstruction of paths, query filters and edit order.
- The runner uses Windows-only `py -3` for hidden scoring and in task prompts. Linux needs a platform-correct interpreter without changing task requirements.
- No pre-agent check proves that seeded memory is actually retrievable in an enabled cell, or absent in stale/control cells.
- Agent authentication errors become task failures instead of being clearly separated as an invalid agent run.

Corrections implemented in `benchmarks/memory/run.py`:

- Match tool results to Vexen check IDs and parse envelope status, not substrings in note text or other tools.
- Save a `check_trace` with each call's inputs, status and whether an observed editor-tool call preceded it. This observes Edit/Write/MultiEdit/NotebookEdit order; it cannot prove that a Bash/Python command did not change a file.
- Verify seeded memory through the CLI before launching an enabled or stale cell.
- Use the running Python interpreter for hidden scoring and the agent's test command instead of requiring Windows `py` on Linux.
- Preserve each agent stream, stderr, hidden-test output and final target file under `traces/`. `--keep-work` additionally retains complete task workspaces.
- Record binary, runner and task hashes, source commit/dirty state, Python version, client version, seed and repetitions in a manifest.
- Mark agent errors as invalid (`passed: null`), keep them separate from scored failures and exit with status 2 if any runs are invalid.
- Refuse to overwrite a directory containing prior results.

New fields do not pretend that old rows contain information they did not record. The historical result files remain unchanged.

## Unproven hypotheses to investigate

1. The agent may have edited the anchor before checking it, making a valid note stale by design.
2. The query may have named a different file or added a search filter that excluded the seeded note.
3. Client rendering may have discarded structured advice, exposing only the incomplete text response.
4. The note may have been delivered correctly but applied incorrectly, particularly on the failed slug run.

Do not relax stale-file checks, expose unrelated notes, inject hidden-test solutions or retry until a passing sample can replace a failed one. Resolve each hypothesis using recorded inputs and outputs.

## Work log and verification

- Baseline `cargo test --locked`: passed on 2026-11-02 before product changes.
- Fresh real-agent rounding reproduction attempted using the existing debug binary and an isolated temporary repository. Claude Code 2.1.312 returned `Not logged in · Please run /login`; zero Vexen calls, zero model cost. This is an authentication blocker, not a valid performance result.
- Local reproduction artifacts: `/tmp/vexen-diagnose-1g3g8g06/`. These temporary paths are diagnostic only, not portable published evidence.
- Product tests: two new compact-response regression tests pass; the MCP integration check passes, including stale-anchor suppression.
- Parser tests: three pass, covering misleading status words/other-tool output, multiple checks around an edit, and authentication error classification.
- `python benchmarks/memory/verify_delivery.py`: 12/12 deterministic task/condition cells pass against the rebuilt Vexen binary. All five target lessons reach both CLI and MCP text output; five stale cases and two unrelated controls correctly return no evidence. **This measures delivery and filtering, not an agent solving tasks.**
- The instrumented runner's one-cell authentication check confirms an invalid agent run is excluded rather than scored as a failed task; results are retained at `/tmp/vexen-instrumented-auth-check-20261102/`.
- `cargo fmt --check` was unavailable under the default toolchain; the installed Rust 1.99.1 `rustfmt` executable was used directly on changed Rust files.
- Final verification: `cargo test --locked` passed all 88 Rust tests; all three Python parser regression tests passed; all 12 deterministic delivery cells passed; changed Rust files passed direct `rustfmt --check`; `git diff --check` passed.
- Fresh real-agent evaluation: pending Claude Code authentication.

## Related research and marketing

`docs/research.md` separates task outcomes, relevance filtering, connection checks and the unsuccessful file-hint experiment. README cards must stay tied to verified results. Until a fresh experiment passes, keep the historical 11/20 versus 2/20 result and its limitations; do not publish 100% or general improvement claims.


## Reproduce the remaining evaluation

After Claude Code authentication is available:

```sh
cargo build --release --locked
python -m unittest discover -s benchmarks/memory -p 'test_*.py'
python benchmarks/memory/verify_delivery.py --vexen target/release/vexen
python benchmarks/memory/run.py --vexen target/release/vexen --claude claude --reps 4 --out /tmp/vexen-memory-reliability-next --keep-work
```

Use a new output directory for every attempt. Report the complete planned matrix, including unsuccessful and invalid cells; do not keep rerunning only failures and merge successful attempts into a fabricated perfect cohort. If a valid run fails, inspect its preserved trace and hidden-test error before making another product change. The memory-off and stale tasks intentionally lack the hidden lesson; demanding 100% from those conditions would not be a test of memory delivery.

## Broader research backlog

1. Establish the failure cause for each previously empty-response pattern using fresh correlated traces; historical summary rows cannot reconstruct it.
2. Measure autonomous check-before-edit use separately from the explicitly prompted condition.
3. Add matched irrelevant/empty-memory and instruction-only conditions to separate information availability from tool/prompt effects.
4. Expand to real repository tasks and a second agent/model without reclassifying those runs as a replication of the Haiku pilot.
5. Measure repeated failed approaches, false reminders, task success, elapsed time and cost. Keep cost comparisons at the whole-run level, not selected successful subgroups.
6. Analyze uncertainty at the task level; shared task repetitions are not independent new tasks.
7. Verify the corrected MCP text path through clients that differ in structured-content handling; the thirteen prior read-call checks do not prove this fix was present in those binaries.


## Follow-up: valid lessons hidden by a global candidate limit

Confirmed on 2026-11-02 in the current product source. `mcp::check` fetched the newest 100 records and only then applied repository/file/evidence eligibility. This produced two independently reproduced errors:

1. A valid decision for `main.rs`, followed by 105 unrelated records for `other.rs`, returned `no_relevant_evidence`.
2. A passing attempt for a file state, followed by 105 unrelated records and a recent failed attempt for the same state, returned a definitive failure item instead of `conflicted`.

Both new end-to-end tests failed on the old implementation and passed after the correction. **This does not explain the historical tiny-fixture pilot, which did not have hundreds of notes.** It is a separate defect affecting accumulated project memory.

Correction: the store now exposes an internal streaming search. The applicability layer selects up to three eligible lessons without a global pre-filter cutoff. It retains at most three selected records, rather than collecting the entire store in memory. If any selected item is a failed attempt, a second stream reconciles its outcome group against all stored evidence, including passing attempts that do not match the text search. A keyword filter must not hide a contradiction. A passing attempt is never chosen as the representative text of a failed approach.

The public limited `Store::search` API and the three-item output bound are preserved. Applicability, stale hashes, ancestry, conditions, supersession and secret handling remain enforced. Large stores may require scanning more records; this removes a recall cutoff, not the cost of reading candidate rows. Performance at large real-world scale is not yet measured.

Claude Code authentication was rechecked using `claude auth status`: `loggedIn: false`, `authMethod: none`. Fresh real-agent task outcomes remain unmeasured; local product regressions cannot substitute for them.


### Independent review and follow-up verification

A separate code review found no concrete correctness regression in the new selection/reconciliation logic, but identified a performance issue in keyword search: ordering by `bm25(...)` and a row-ID tie-breaker forced SQLite to materialize a temporary sort of matched record payloads. Actual `EXPLAIN QUERY PLAN` output contained `USE TEMP B-TREE FOR ORDER BY`.

The internal streaming query now orders by FTS5's hidden `rank` column. Both the local stored-fixture plan and a regression test using the product's bundled SQLite and real migrations show no temporary B-tree sort. The regression test failed before the query change. This removes that extra payload sort; it is not a claim of constant memory or measured latency for every size of FTS index.

The review also exposed a preexisting response-budget gap: clipping an error to 500 Unicode characters could exceed the UTF-8 byte budget (the reproducer returned 1,949 bytes). Errors now also respect the 1,700-byte JSON-encoded string budget, truncating only on Unicode character boundaries. Its regression test failed before the correction.


Reviewed follow-up verification: all **92 Rust tests**, all **3 Python parser tests** and all **12 deterministic delivery cells** passed. The rebuilt debug binary was used for the delivery checks. Changed Rust files passed direct formatting checks; `git diff --check` passed. The independent reviewer reran and approved the two review-follow-up tests. Historical task definitions and both original pilot JSONL files have no diff. These checks still do not establish real-agent task completion; authentication remains the external prerequisite for the fresh planned cohort.


## Current external blocker

Claude Code authentication remained unavailable on the third consecutive goal turn: a fresh `claude auth status` returned `loggedIn: false` and `authMethod: none`. Local fixes, regression checks and independent review are recorded above. Further task-success conclusions require real agent executions; additional unrelated edits or deterministic checks cannot establish the requested outcome. Run Claude Code `/login` in a terminal, then resume the evaluation using the commands above. The goal is blocked, not complete.


## GitHub research: architecture-compatible strengthening roadmap

Research snapshot: 2026-11-02. The [detailed GitHub assessment](github-strengthening-opportunities.md) contains primary-source links, observed project maturity, integration costs and exclusions. These are proposed follow-up investments, not implemented features or approved dependencies. They do not replace the pending real-agent evaluation.

| Priority | Opportunity and source | Integration boundary | Acceptance evidence | Current decision |
|---|---|---|---|---|
| P1 | Independent delivery checks — [MCP Inspector](https://github.com/modelcontextprotocol/inspector) | Version-pinned development/CI client; no end-user runtime dependency | Assert lesson text and structured result for empty, relevant, stale and conflicting fixtures; fail on missing lesson content | Recommended next validation step |
| P1 | Generated lifecycle scenarios — [proptest](https://github.com/proptest-rs/proptest) | Rust development dependency only | Generated record/outcome/edit/branch/supersede/import sequences preserve scope, staleness, contradictions and output bounds; minimized failures become permanent regression cases | Recommended robustness work |
| P2 | Structured test evidence — [nextest](https://github.com/nextest-rs/nextest) / JUnit reports | Explicit report adapter; local allowlisted metadata only | Distinguish failed, passed, skipped and retried tests; retain report hash and code-state linkage; repeated imports are idempotent | Design an adapter after current outcome evaluation |
| P2 | Versioned client configuration fixtures — [add-mcp](https://github.com/neon-solutions/add-mcp) | Reference fixtures and format knowledge; keep native Rust installer | Repeated installation is idempotent; unrelated settings/comments survive where supported; uninstall removes only the owned entry; no permission changes | Reference source, not a core dependency |
| P3 | Code-fragment memory — [Tree-sitter](https://github.com/tree-sitter/tree-sitter) / [ast-grep](https://github.com/ast-grep/ast-grep) | Single-language, read-only research path | Measure useful recovery after unrelated edits alongside false reminders after semantic changes; stale lessons remain candidates until revalidated | Experimental; not default behavior |
| P3 | Distribution maintenance — [cargo-dist](https://github.com/axodotdev/cargo-dist) | Release tooling only; preserve existing binary/bootstrap contracts | Existing platform archives, checksums, installation and upgrade paths still work; demonstrate reduced maintenance | Conditional on measured maintenance need |

### Adoption gates

Before selecting an exact dependency version, verify its release, license, supported platforms and maintenance status again. Pin tooling used in reproducible checks. A repository's age, official ownership or popularity is supporting context, not proof that its behavior fits Vexen.

For every candidate, record a short decision covering the existing problem, smallest useful integration, alternatives, runtime/development dependency impact, data handled, acceptance experiment and rollback path. A failed acceptance experiment leaves the capability outside the default product.

Keep three authority boundaries explicit:

1. A report supplied by an agent is an imported assertion until its execution and code-state provenance are established. It must not silently become an observed command result.
2. A matching symbol or syntax tree is a retrieval signal, not proof that an old lesson still applies. Preserve current stale-file protection until a separately evaluated validity model exists.
3. Protocol compatibility and generated invariants validate particular system properties. They do not establish that an agent solves every task.

### Delivery order and stop conditions

First finish the pending real-agent cohort using current fixes and retained traces. Independent delivery checks and lifecycle tests can then strengthen the validation baseline. Add structured test evidence and client fixtures as separate changes, each with its own acceptance results. Keep code-fragment memory in a separate experiment. Consider release-tool migration only when its benefit exceeds the cost of replacing the existing working pipeline.

Stop or defer adoption when it adds a mandatory cloud/model service, an always-on daemon, passive transcript collection, silent permissions changes, unbounded result output, or weaker source applicability checks. An integration that improves a benchmark by exposing hidden task answers or selecting only successful retries fails the evaluation standard.

### Watchlist rather than adoption

[MCP conformance](https://github.com/modelcontextprotocol/conformance) needs a transport/scenario fit assessment for Vexen's stdio interface. [adrkit](https://github.com/mbeacom/adrkit) is an interoperability research candidate rather than a governance subsystem to embed. [Engram](https://github.com/Gentleman-Programming/engram) is a useful onboarding/product reference rather than a replacement memory engine. Gitleaks' disclosed maintenance transition is recorded in the detailed assessment; no new mandatory scanner dependency is proposed.


## README marketing research with the Oxocarbon constraint

The [academic marketing and comprehension assessment](readme-marketing-academic-research.md) links six research sources to specific README recommendations. The Oxocarbon palette, existing logo and Geist typography are fixed. The assessment distinguishes published findings from Vexen-specific hypotheses, includes first-screen/card copy drafts, flags mobile text scaling, and proposes a formative user study. It does not change README assets or claim a measured conversion/comprehension improvement. Research claims remain separate from Vexen's internal coding pilot.


## Native installation direction

The [Rust-native installation assessment](rust-native-installation.md) applies Ponytail's integrated onboarding approach while retaining Rust-owned product/setup logic. It records the closest Rust setup example (`jujutsu-mcp`), mature native distribution candidates (`cargo-dist`, `cargo-binstall`), and exclusions for archived or unsuitable projects. Native delivery, agent integration, project initialization and actual client verification are distinct acceptance steps. No plugin/native bootstrap mechanism or new `setup` command is claimed to exist yet; Node/npx is an optional compatibility path rather than the proposed primary runtime.


Native-installation research was expanded with pinned source snapshots for Rust update/replacement/locking mechanisms, integrity versus authenticity distinctions, and an explicit clean-machine/partial-failure test matrix. See [the canonical Rust installation assessment](rust-native-installation.md). This is documentation-only research; no updater, setup command or plugin bootstrap was implemented or validated as a shipped feature.
