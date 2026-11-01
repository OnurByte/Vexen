# SDD ledger — plan: docs/specification.md

Preflight:
| Tasks | Shared contract | Finding |
| 1 | Git interfaces / benchmark | Predictive gate must precede later scope |
| 2 | Store append / projection | coherent |
| 3 | Store records / eligibility | coherent |
| 4 | MCP / store and Git | §4 omits precise input schemas; define minimal explicit inputs |
| 5 | CLI / existing modules | coherent |
| 6 | Installer / fixture evidence | 12 live clients unavailable locally; never claim stable |
| 7 | Observe / privacy | opt-in user commands only |
| 8 | Release / CI | remote OS results cannot be claimed locally |
| 9 | Evaluation / ship | actual paired runs required, no invented uplift |
| 1,3,4,5,9 | History and recommendations | gate applies to later default feature |
| 2,3,4,5,7,9 | Events, outcomes and validity | shared model, no duplicate authority |
| 3,5,7 | Paths/redaction/export | reject scope escape and sensitive capture |
| 4,5,8 | Protocol/CLI/packaging | official SDK/version availability needs verification |
| 5,6,8 | Install/init/docs | only explicit installation writes outside repo |
| 6,8 | Compatibility labels | fixture tests do not imply live tests |
Original plan: evaluate Task 1 before proceeding to later scope; synthetic fixtures verify mechanics only. The user later directed continuation of the remaining product scope despite the failed co-change gate.
Task 1 implementation complete in commits d3fd909 and 53f7b5a. The corrected frozen corpus run is recorded in `docs/benchmarks.md` and `benchmarks/cochange/results/`.

Evaluation ruling: thresholded co-change Hit@3 was below plain historical co-change counts on all ten repositories. Co-change remains research-only and no default hints or release efficacy claim are approved. Per the user's direction, continue the remaining product scope independently of this failed co-change gate.

Ruling: User explicitly directed continuing to finish after the failed gate. Keep historical cochange out of default advice; continue memory/MCP/CLI/installer scope. Do not claim predictive superiority, stable live compatibility, or agent outcome uplift. Cost if wrong: memory-first positioning may require revision, but no unvalidated default hints are exposed.
Task 1: fix round 1/5 all five findings addressed, commits d3fd909..53f7b5a; corpus completed all10, no Hit@3 superiority to plain counts.

## 2026-11-01 — additional client runtime checks

Vexen 0.2.5 completed a real `vexen_check` call in 13 previously untested clients on Linux: GitHub Copilot CLI, Gemini CLI, Qwen Code, Continue CLI, Goose, Hermes, VS Code, Cline CLI, fast-agent, Crush, Kilo CLI, Mistral Vibe and OpenHands CLI. The earlier Claude Code/Codex/OpenCode checks are not counted again. One run used a real model, eleven used a deterministic local model endpoint, and one used VS Code's native extension test host. The actual clients and actual Vexen process performed the MCP operations.

Four-tool discovery also succeeded in Cursor, Kiro, Zed and the Cline editor extension, without a verified tool invocation in those surfaces. Windsurf's bundled Devin print flow remained login-blocked. These partial checks are excluded from the thirteen. See `docs/client-smoke-tests.md`, raw fixture evidence in `benchmarks/clients/results/2026-11-01/`, and `python benchmarks/clients/verify_results.py` for the evidence audit.

This advances experimental runtime compatibility, not the full stable-installer gate: broad OS/client coverage, the planned adapter matrix, and paired evaluation on real repositories remain open.
