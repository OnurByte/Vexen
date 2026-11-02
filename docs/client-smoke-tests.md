# Client runtime checks — 2026-11-01

**13 additional clients completed a real `vexen_check` call against Vexen 0.2.5 on Linux x86_64.** Claude Code, Codex CLI and OpenCode, which already had documented Windows checks, are excluded from this count.

This is an experimental compatibility check. It is not a claim that 13 models followed Vexen's advice, that every client installer works, or that the installer is stable on three operating systems.

## Verified tool calls

| Client | Installed version | Driver | Result evidence |
|---|---|---|---|
| GitHub Copilot CLI | 1.0.105 | Real model, existing authenticated account | Client log shows the tool invocation and returned note |
| Gemini CLI | 0.66.0 | Local deterministic model endpoint | MCP request and structured response |
| Qwen Code | 0.26.5 | Local deterministic model endpoint | MCP request and structured response |
| Continue CLI | 1.5.47 | Local deterministic model endpoint | MCP request and structured response |
| Goose | 1.57.0 | Local deterministic model endpoint | MCP request and structured response |
| Hermes | 0.22.5 | Local deterministic model endpoint | MCP request and structured response |
| Visual Studio Code | 1.144.0 | Native extension test host; no model | Editor discovers tools and calls `vscode.lm.invokeTool` |
| Cline CLI | 3.0.74 | Local deterministic model endpoint | MCP request and structured response |
| fast-agent | 0.10.49 | Local deterministic model endpoint | MCP request and structured response |
| Crush | 0.99.0 | Local deterministic model endpoint | MCP request and structured response |
| Kilo CLI | 7.8.18 | Local deterministic model endpoint | Completed client tool event with matching record ID |
| Mistral Vibe | 3.0.4 | Local deterministic model endpoint | MCP request and structured response |
| OpenHands CLI | 1.16.0; SDK 1.21.0 | Local deterministic model endpoint | MCP request and structured response |

Totals: one real-model run, eleven deterministic-model runs, and one native editor API run. Every row uses the actual installed client application and the actual Vexen executable. No handwritten MCP client or SDK-only test substitutes for these clients. Kilo is an OpenCode-derived client; this is product coverage, not thirteen independent protocol implementations.

For the eleven deterministic runs, only the model endpoint is replaced: a local HTTP server emits a tool request using the client's exposed schema, then stops after the result. The real client launches Vexen, negotiates MCP, discovers its tools, checks permissions, dispatches the call and consumes the response. Hermes, Qwen and Vibe additionally exercise their deferred-tool discovery paths. OpenHands requires its own `security_risk` input before dispatch; the test driver supplies `LOW`. No Vexen product fix was needed.

## Fixture and evidence

- The exact Vexen source commit and binary SHA-256 are in the [manifest](../benchmarks/clients/results/2026-11-01/manifest.json).
- A separate Git checkout contains `sample.py` and one human-accepted decision: `SMOKE-20261101: empty input must return zero; a previous implementation returned None.` The file is unchanged when queried.
- The check uses the explicit checkout root and `paths: ["sample.py"]`. Expected result: `accepted_decision`, the fixture's repository and record IDs, and the original note.
- A transparent stdio proxy records the messages it forwards without modifying them. Eleven clients have wire evidence. Copilot has a client log; Kilo has a completed client tool event. Kilo's event retains the compact result and matching record ID, not the full structured note.
- Installed package versions above come from package metadata or version commands. Some clients advertise placeholder versions in `clientInfo`; those are retained verbatim in the wire logs.
- Published evidence contains fixture protocol messages and selected client events, not the local model-request logs or account credentials.

The [recorded results](../benchmarks/clients/results/2026-11-01/) include raw messages, client events, a manifest, and SHA-256 checksums. Audit the saved evidence with:

```sh
python benchmarks/clients/verify_results.py
```

This audits recorded results; it does not launch clients again. It requires thirteen distinct new client IDs, matches requests to responses, checks four-tool discovery for the wire-backed runs, and validates the returned fixture record. A model's final sentence alone never counts as a passing wire-backed run.

The diagnostic [model endpoint](../benchmarks/clients/mock_provider.py) and [stdio proxy](../benchmarks/clients/mcp_proxy.py) are snapshots of the scripts actually used. They retain the test machine's fixture and binary paths, so they are an audit record and not a portable CI runner. The local client installations and profiles stayed on the test machine. The fixture checkout stays at the path recorded in the [manifest](../benchmarks/clients/results/2026-11-01/manifest.json), because its store identity is tied to its canonical checkout path.

## Additional checks, excluded from the thirteen

| Client | Observation | What remains unverified |
|---|---|---|
| Cursor | CLI approved the server and listed all four tools | Tool invocation; its editor does not expose VS Code's `workbench.mcp.startServer` command |
| Kiro | Native editor MCP runtime negotiated discovery and returned all four tools | Tool invocation; separate Kiro CLI setup required login |
| Zed | Native editor negotiated MCP and discovered all four tools | Tool invocation; cloud model listing returned HTTP 402, and a local-provider UI attempt did not produce a call |
| Cline VS Code extension 4.1.26 | Native extension discovered all four tools | Extension-driven tool invocation; the successful Cline CLI row is the single counted Cline result |
| Windsurf 3.11.238 / bundled Devin | Editor activated; native CLI print attempt stopped with `Login canceled` | MCP handshake and tool invocation; a generic VS Code test command is unavailable in this editor |

These observations must not be relabeled as successful tool-call tests. In particular, a missing VS Code command in an editor fork does not establish an incompatibility with Vexen's MCP server.

## Limits and remaining release gates

These are single-session Linux checks of discovery and one read tool on a toy fixture. They do not cover every tool, repeated install/uninstall, client upgrades, all operating systems, stale/conflicting evidence through every client UI, or useful task outcomes. Local test-driver runs establish transport, schema and dispatch compatibility, not unprompted model use or task improvement.

The specification's broader adapter matrix, cross-platform client coverage and real-repository paired evaluation remain open. These results alone do not authorize a stable-installer or general agent-uplift claim.
