> **Historical document, kept for provenance.** It records a plan, a milestone or a snapshot from earlier in the project and may no longer match the code. For the current state see the [status and roadmap](../roadmap.md).

# Task 3 report: validity, ranking, and privacy

**Status: implemented; focused security tests pass.** Advice is emitted only after local evidence is rechecked against the explicitly opened checkout, its current `HEAD`, the exact working file bytes, exact query path, and exact conditions vector. No record text is interpreted as an instruction.

## API and behavior

- `RepoContext::new(&Store, query_paths, conditions)` captures the Store's canonical root/repository ID, current verified Git `HEAD`, validated repo-relative query paths, and caller conditions. An empty query path list is the supported text-only context: records still require a scoped repo ID, safe in-root anchor, matching conditions, current file hash, and ancestor commit. `RepoContext::unavailable()` is an explicit abstention context.
- `check_applicability(&MemoryRecord, &RepoContext) -> Eligibility` verifies checkout binding, exact query path when paths were supplied, exact condition equality, anchor metadata, safe in-root canonical path resolution, working-file SHA-256 equality, and that the anchored full commit ID is an ancestor of current `HEAD`. Missing or unreadable files return `MissingEvidence`; changed content or branch history returns `Stale`; path/repo/condition mismatches return `WrongScope`; typed supersession returns `Superseded`.
- `rank_applicable(records, ctx, limit)` preserves the Store's input order (so BM25/FTS ranking is retained), filters through applicability first, returns at most three advisories, and bounds combined advisory prose to 1,800 UTF-8 bytes. Attempts require recorded observed outcomes; candidate attempts are omitted. A failure is described as an observed failure, never as verified or guaranteed. Accepted decisions are labeled separately and never promoted into test outcomes.
- Conflicting success and failure outcomes for the same applicable path/hash/conditions produce one `Conflicted` advisory with explicit abstention wording, regardless of success/failure record ordering. Contradictory events are not deleted or rewritten.
- `privacy::contains_secret` scans for common private-key PEM headers, common token prefixes, credential assignments, and sensitive argv flags. Optional capture/export callers must reject the whole operation when this returns `true`, must use an allowlisted test identifier, and must persist only sanitized/allowlisted metadata (such as the existing `NewOutcome` fields), never raw argv or output. `redact_secrets` is for bounded display/export cleanup; it is not permission to store raw sensitive input.
- `privacy::resolve_repo_path` rejects absolute/traversal paths and canonical targets outside the checkout, including external symlink targets. It allows a missing leaf only when its nearest existing parent resolves inside the root.

`Eligibility::Superseded` is present for the contract, but Task 2's event model has no explicit supersession event or relation. This slice does not infer supersession from chronology because doing so could erase conflicting observations; callers receive current applicability only until a typed supersession model is added.

## Test-first and verification

- **RED:** the first `cargo test --test security` failed to compile because the requested analysis/privacy APIs and public modules did not exist. After adding those tests and the initial implementation, the focused suite exposed path-resolution and redaction defects; both were corrected.
- **GREEN:** after the store and validity review fixes, `RUSTUP_TOOLCHAIN=1.99.1 cargo test --test store --test security` passed all 25 focused tests. The 9 security tests cover unrelated project/query scope, exact conditions, branch drift, changed/deleted anchors, symlink escape, contradictory outcomes in both record orders, candidate suppression, text-only contexts with hard scope checks, typed supersession, untrusted instruction-like text, output budget, secret detection, and empty-set abstention.
- Full `RUSTUP_TOOLCHAIN=1.99.1 cargo test` passed all 39 tests (2 unit, 12 Git, 9 security, 16 store) plus doc tests. The store tests include read-only open, full event-envelope tampering, literal FTS, raw non-UTF-8 path binding, migration rollback, and supersession persistence.
- `cargo clippy --lib -- -D warnings` could not run because the installed stable toolchain does not have the `clippy` component. `rustfmt` was run on this slice's Rust files and `src/lib.rs`.

## Caveats

The secret detector is a reusable conservative pattern scanner, not an exhaustive secret classifier. Its caller must fail closed and still apply the metadata allowlist. Context conditions use exact vector equality, including ordering. The Store's search ordering is preserved rather than recomputed, because the public record model does not expose a numeric FTS score.
