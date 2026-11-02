> **Historical document, kept for provenance.** It records a plan, a milestone or a snapshot from earlier in the project and may no longer match the code. For the current state see the [status and roadmap](../roadmap.md).

# Task 4 — MCP server

## RED / GREEN record

- **RED, compile checkpoint:** the first `cargo check --lib` after wiring rmcp exposed missing SDK trait imports, missing `JsonSchema` implementations on the public store inputs, and an `EventId` conversion gap. Those were fixed before verification. The first integration test compile also caught an incorrect transport import and a malformed JSON-RPC format string; the corrected tests then ran against rmcp's in-memory SDK transport.
- **GREEN:** `RUSTUP_TOOLCHAIN=1.99.1 cargo test --test mcp` passed both current `server/discover`/inline request lifecycle and legacy `initialize` lifecycle tests. They list all four sorted tools, verify input/output schemas and read-only annotations, call check/record/outcome, confirm read-only check did not append events, exercise explicit canonical `repo_root`, and check bounded text content and truthful invalid-root errors.
- **TDD note:** these integration tests were first executed after the MCP implementation compiled; no pre-implementation behavior-failure run was recorded. The compile failures above are the actual RED evidence.

## MCP surface

The library exposes `VexenServer` and `serve_stdio()` for Task 5 wiring. rmcp is pinned to **3.8.0** with server and stdio transport features; test-only client support exercises both protocol generations. Tool discovery is deterministic and sorted:

1. `vexen_check`
2. `vexen_explain`
3. `vexen_outcome`
4. `vexen_record`

All tool inputs use typed schemas with unknown fields rejected. Responses include `structuredContent` and a compact JSON text summary capped below 1,800 bytes. Read tools open the Store read-only. Record validates the explicit canonical checkout, safe repository path, current anchor digest, and secret-like text. Outcome accepts exactly one observed outcome or typed supersession. `diff_ref` resolves to a full commit before `git diff` is invoked with argument arrays and a `--` path separator; untracked files are not part of Git's diff result.

Request validation and business failures return bounded MCP tool errors (`isError`); malformed tool arguments are rejected by the SDK schema/parameter layer. No tool launches arbitrary commands or executes stored text. The stdio entrypoint emits no diagnostics itself.

## Verification limits

The standalone CLI does not exist until Task 5, so MCP Inspector could not yet launch the process. The equivalent request/discovery/tool-call flows were verified through rmcp 3.8.0's in-memory SDK transport. A separate manual Inspector pass remains for the CLI integration stage.

The MCP test target passed before the parallel installer/CLI work began. A subsequent workspace clippy rerun was temporarily blocked by the in-progress installer module's missing child modules; this does not change the earlier successful MCP test result and should be rechecked after that integration lands.
