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
Ruling: Begin Task 1 only; chronological real-repository evaluation must justify proceeding. Synthetic fixtures verify mechanics only. External validation remains an honest gate.
Task 1: in progress.
