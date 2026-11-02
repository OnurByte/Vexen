# Vexen: product, reliability, documentation, and legacy-cleanup execution brief

**Repository:** `OnurByte/Vexen`  
**Target branch:** `implementation`  
**Mode:** inspect, implement, test, document, and commit locally. **Do not push, publish, tag, or release without explicit authorization.**

## Objective

Make Vexen easier to understand, install, activate, and trust while improving its actual memory-delivery reliability. Consolidate the public documentation and remove obsolete or unreferenced legacy structures. Keep the product small, offline, evidence-led, and maintainable. The goal is a working improvement, not a longer plan or another collection of speculative Markdown files.

## 0. Read and establish truth first

1. Confirm the current `implementation` HEAD and inspect `git status --short`. Preserve existing user changes; do not reset or overwrite them.
2. Read `AGENTS.md` if present, `brand.md`, `README.md`, and these documents in full:
   - `docs/memory-reliability-investigation.md`
   - `docs/github-strengthening-opportunities.md`
   - `docs/readme-marketing-academic-research.md`
   - `docs/research.md`
3. Inspect the relevant implementation, tests, benchmark runners, assets, installer, npm launcher, release pipeline, and the remainder of `docs/` before editing. Follow repository-specific agent instructions.
4. Prepare a brief inventory that distinguishes:
   - **Verified and working:** backed by current source and suitable tests.
   - **Implemented but not sufficiently validated:** code exists, evidence is incomplete.
   - **Proposed/experimental:** not a default product capability.
   - **Blocked:** depends on authentication, permissions, an external runtime, or another unavailable prerequisite.
   - **Legacy/stale/dead:** historical material, contradicted claims, duplicate files, or unused code/assets.
5. Give a short execution plan with the highest-impact items and their verification. Then proceed autonomously on routine, reversible decisions. Do not redo already verified fixes just because a document proposes them.

## Hard constraints

- Preserve the existing Vexen logo, **Oxocarbon** visual identity, `brand.md` palette, **Geist / Geist Mono**, and the coherent SVG design language. Do not redesign the brand or introduce decorative UI noise.
- Keep Vexen **free, open source, local-first and functional offline** after installation. No mandatory accounts, cloud, API keys, model services, hosted memory, telemetry, vector database, or always-on daemon.
- Preserve explicit per-checkout repository scope, safe path handling, file-content and branch staleness checks, supersession, conflicting-evidence abstention, secret-handling safeguards, and bounded output. Never silently promote stored text to instructions or execute untrusted memory.
- Do not turn a fixture, parser unit test, deterministic MCP call, or integration smoke test into a claim of real-agent task improvement.
- Preserve all historical benchmark datasets, raw results, task definitions, hidden tests, original splits, and unsuccessful runs. Write fresh evaluations to **new paths**, with exact code/model versions and full valid/invalid-run accounting.
- No success-only reruns or replacement of failed observations. Report invalid authentication runs separately. Do not claim 100% success, universal safety, or general coding uplift without appropriate evidence.
- Prefer changes that reduce complexity. Do not add dependencies or frameworks for hypothetical future capabilities. Experimental code-fragment memory and similar ideas stay **off by default** until a comparative evaluation supports them.
- No destructive broad cleanup, `git reset --hard`, or silent removal of user-owned work. Use targeted, inspectable changes; keep Git history and important research evidence accessible.

## Priority 1: README understanding and first-use value

The target reader uses Claude Code, Codex, Cursor, or a comparable coding agent but may not know MCP, memory architecture, `PATH`, or Rust.

- Make the first screen answer **what Vexen does, why it matters, and how to try it**. Use a concrete failed-fix example, not generic "intelligent memory" promises.
- Keep an honest before/after story: a failed attempt must actually be recorded and the next agent must check for it. Never imply automatic conversation capture or guaranteed error prevention.
- Show the shortest verified installation path early. Put source builds, manual binaries, scopes, and protocol internals in clearly labeled advanced guidance.
- Demonstrate first value. An empty new project correctly returns `no_relevant_evidence`, but that must not be the only onboarding experience. Implement a **safe, separate example/demo** if feasible, with clearly synthetic records and real Vexen calls; never seed fake lessons into a user's repository without explicit intent.
- Retain one clear, contextual research proof: the historical **11/20 vs 2/20** pilot is on synthetic tasks with tailored notes, explicit agent prompting, and one model. Keep a visible short caveat and links to the full method and raw data. Do not describe all 76 runs as the sample size of that particular comparison.
- Move secondary research cards, implementation logs, and negative-result details behind clearly labeled links or a research page. Keep the negative results accessible, not hidden.
- Verify SVG legibility in actual GitHub Markdown at narrow widths (at least 360px and 390px), tablet, desktop, and light/dark themes. Small text embedded in a 1200-unit SVG must not shrink into unreadable labels. Adjust layouts, not the prescribed palette/logo.
- Prefer concise, direct English copy for global README readers. No invented testimonials, adoption numbers, endorsements, "never fails," or unsupported benchmarks.

**Acceptance:** An unfamiliar user can identify the problem, find the relevant setup, complete a verified connection, and see an actual saved/retrieved lesson or an explicitly labeled demo. Record usability/visual observations; do not fabricate conversion gains.

## Priority 2: Installation and activation

- Trace the real `npm`/`npx` launcher, binary cache, installer output, MCP configuration, optional skill and agent behavior on supported platforms.
- Resolve the current mismatch: `npx @onurbyte/vexen ...` runs a cached launcher while `skills/vexen/SKILL.md` expects a `vexen` executable on `PATH`. Choose the smallest safe end-to-end design, then make README, skill, CLI, and installer consistent. Avoid requiring a separate Rust toolchain.
- Where feasible, offer one guided setup path that detects or explicitly selects agents, initializes the checkout, configures MCP, installs/activates appropriate guidance, verifies it, and reports any manual steps. Do **not** document a `setup` command until it exists and is tested.
- Distinguish **installed**, **MCP discoverable**, **successful tool call**, and **agent actually uses it**. A config write alone is not activation. Make errors and corrective commands comprehensible.
- Respect per-client global/project scopes and user permission controls. Idempotent install/uninstall must preserve unrelated config and avoid auto-approving tools.
- Add focused tests for npx-only users, binary-on-PATH users, repeated installation, uninstall, Windows invocation, incompatible/ambiguous configs, and no client detected.

**Acceptance:** Every advertised quick-start path matches the implementation; a fresh user can verify a `vexen_check` call and understand how recording begins. An optional skill is genuinely usable with the advertised install path.

## Priority 3: Memory correctness and delivery

Start from the fixes already documented in `docs/memory-reliability-investigation.md`, including text-only MCP lesson delivery, retrieval beyond the old global candidate limit, contradiction reconciliation, Unicode byte budgets, and instrumentation. Inspect the current source and regression tests before doing more work.

- Verify the entire path: record -> observed test outcome or accepted decision -> eligibility -> MCP structured and text response -> agent-visible lesson.
- Preserve current file/hash/branch/scope/condition checks, suppression of stale or irrelevant notes, and conflict handling even when a query filter is present.
- Include both short and long/adversarial text, Unicode, many unrelated records, conflicting outcomes, imports, and text-only client behavior.
- Measure retrieval and output correctness separately from task success and performance. If a newly discovered issue has a reproducible fixture, write a failing regression test first, fix minimally, and rerun relevant checks.
- Do not relax eligibility merely to improve the benchmark. A historical note is not permission to change current code blindly.

**Acceptance:** Applicable lesson text is delivered to both MCP representations; stale/unrelated evidence stays silent; contradictions are not buried; bounded responses and privacy invariants hold.

## Priority 4: Independent protocol and lifecycle validation

- Evaluate **MCP Inspector** as a version-pinned development/CI smoke client, not an end-user runtime dependency. Test real stdio startup, tool discovery, tool calls, and relevant/empty/stale/conflicted responses, including readable lesson text.
- Consider **proptest** for generated record/outcome/edit/branch/supersede/import lifecycles. Minimize failing cases and keep them as deterministic regressions.
- Consider **nextest/JUnit**, `add-mcp`, and related candidates only where they address an observed failure or missing verification. Keep adapters explicit and optional; never accept raw test reports as independently observed execution without provenance.
- Document the need, license, maintained version, architectural impact, measurable acceptance test, and rollback for any new dependency. Reject dependencies whose maintenance/operational cost outweighs the verified benefit.
- Do not ship Tree-sitter/ast-grep fragment recall or Git co-change hints by default until they beat useful baselines without increasing incorrect reminders.

**Acceptance:** Scoped independent tests run reproducibly and add confidence beyond the existing internal fixtures, without adding mandatory production services or bloating the runtime.

## Priority 5: Real-agent evaluation

- First verify whether the environment is authenticated for an actual coding agent. If not, record the exact blocker and continue independent work; do not seek or repurpose credentials outside normal configured access.
- Preserve the original pilot task difficulty, hidden assertions, baselines, and historical result files. Use the instrumented runner and a fresh result directory with task-level traces, model/agent version, commit, seeds, calls, outputs, hidden-test results, elapsed time, and cost.
- Run the full predeclared comparison, not only failing or promising cells. Separate valid successes/failures from harness/authentication invalid runs. Do not keep retrying failures until an attractive aggregate appears.
- Diagnose failure at the right layer: no tool invocation, wrong query/scope, stale-after-edit, missing text delivery, note misapplication, or model task failure. Fix product defects with regressions; do not change hidden tests to fit Vexen.
- Once feasible, broaden the study with real repository tasks, at least another agent/model, an instruction-only/empty-MCP control, and task-aware uncertainty estimates. These are future evaluation gates, not reasons to invent claims now.

**Acceptance:** Publish complete fresh outcomes and failures with enough traces to audit them. Keep historical pilot figures unchanged unless a distinct new study is clearly identified.

## Priority 6: Documentation, legacy code and dead-structure cleanup

Treat this as a **real cleanup pass**, not a request to generate more documentation. Audit `docs/`, root Markdown files, generated SVGs, scripts, old modules, fixture directories, stale configuration examples, and references from README, tests, CI, release scripts, and source.

### Classification and action

| Classification | Required action |
| --- | --- |
| Current user documentation | Correct, simplify, link from a compact docs index |
| Active technical reference | Keep accurate and separate from onboarding |
| Research evidence and negative experiments | Preserve with datasets, code, original results and explicit limitations |
| Historical implementation/task report | Move to a clearly labeled `docs/archive/` or equivalent; retain provenance |
| Superseded duplicate or demonstrably unreferenced artifact | Remove **only after** validating references, generator ownership and Git history; record what was removed |
| Unimplemented proposal | Mark as proposed/blocked or move to a roadmap; do not describe it as shipped |

### Known starting points to verify, not blindly apply

- `docs/brand-report.md` says CLI/MCP/release are missing; this is old snapshot material. Archive or update its status.
- `docs/specification.md` begins as an unimplemented design and contains a long task backlog and outdated first-run Git co-change promises. Preserve it as historical specification, and make any README *Roadmap* link point to a concise **current** status instead.
- `docs/progress.md`, `docs/task-1.md`, and `docs/task-*-report.md` are milestone/work logs rather than newcomer documentation; classify and archive where appropriate.
- `docs/security.md` has an old compatibility statement conflicting with later client smoke tests. Reconcile current supported/verified distinctions.
- `docs/pilot-results.md` contains wording implying that every Vexen-enabled run received a usable note and that negative transfer was ruled out. Reconcile with raw rows and `docs/research.md` (20 calls, 12 last observed-failure statuses, eight last empty statuses; stale 0/20 vs off 2/20). Preserve the actual historical data.
- `docs/getting-started.md` opens with manual binary download before the simpler npx route and mixes basic/advanced steps; reorganize for new users.
- `docs/client-smoke-tests.md` includes machine-local absolute test paths; retain auditable fixture metadata, but remove unnecessary personal machine paths from public explanatory prose.
- `docs/research.md` already separates coding outcomes, MCP interoperability, relevance filtering and negative co-change results. Reuse it rather than creating a contradictory second research summary.
- `docs/readme-marketing-academic-research.md`, `docs/github-strengthening-opportunities.md`, and `docs/memory-reliability-investigation.md` contain useful investigation history but should not masquerade as product setup guidance. Keep their current/blocked/proposed states explicit; move or link according to a simple hierarchy.
- Inspect legacy `docs/assets/*.svg` alongside their generating scripts and all usages. Retain canonical brand assets and source-of-truth generators. Remove or archive obsolete illustrations only after checking README/docs/release references and visual impact.

### Documentation target

Prefer one discoverable `docs/README.md` index or equivalent with four paths:

1. **Start:** install, configure a supported agent, verify, first successful lesson, troubleshooting.
2. **Reference:** CLI/MCP contract, data format, security/privacy, compatibility limits.
3. **Research:** study summaries, methodology, negative results, raw datasets, reproduction.
4. **Archive/Development:** historical plans, milestone reports and prior design decisions.

Do not move files mechanically without updating relative links, anchors, cross-references, READMEs, generator inputs, and CI/docs checks. Check for broken references after cleanup. Avoid scattering new `PLAN.md`, `REPORT.md`, or redundant status documents across the root. Prefer updating one canonical page per topic.

**Acceptance:** A new visitor can find installation and first use without reading internal project history. Current claims no longer contradict shipped code; research and legacy evidence remain discoverable. The repository has measurably less dead/duplicated material, with an inventory of removed or archived targets.

## Verification and delivery workflow

1. Work in small related batches. Before each batch, identify affected files and a reversible change boundary. Avoid unrelated refactors.
2. Use failing tests to reproduce behavioral defects when possible. Run appropriate Rust, Python, CLI, installer, protocol and integration checks after each relevant batch.
3. For README and SVG changes, render/check GitHub Markdown and perform visual QA at mobile/tablet/desktop widths in light/dark themes. Do not equate an SVG generator exiting successfully with readable output.
4. For cleanup, run link/reference checks and inspect the final Git diff. Ensure no test datasets, hidden assertions, research evidence, or user modifications were lost.
5. Keep `README.md`, `docs/research.md`, compatibility, security, and current roadmap/status aligned. Mark **completed**, **experimental**, and **blocked** explicitly.
6. Make coherent local commits if useful. **Do not push, create a release, publish npm, or change external settings.** Stop and ask when new authority is required.

## Final handoff format

Report concisely:

- **Implemented:** functional changes and user-visible improvements.
- **Cleaned:** archived/removed legacy docs, dead code/assets and broken references, with exact paths.
- **Verified:** commands and test/visual results; clearly separate unit/regression, MCP client tests and real-agent outcomes.
- **Blocked or not implemented:** authentication, unavailable platforms, proposed features and next acceptance gates.
- **Commits:** local commit hashes, if any. Explicitly confirm **not pushed / not released**.

Do not close the task merely because the document looks cleaner. The priority is a simpler **working** product with truthful evidence and a maintainable repository.
