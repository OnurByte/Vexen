# Memory pilot: results

First run of the paired memory-on/off pilot in [`benchmarks/memory`](../benchmarks/memory/README.md). 24 real Claude Code sessions (Haiku, 14-turn limit), 3 repetitions per cell, order shuffled with a fixed seed, no harness errors. Raw rows: [`pilot-1.jsonl`](../benchmarks/memory/results/pilot-1.jsonl).

| task | condition | passed | runs | 95% interval | mean turns | mean cost USD |
|---|---|---|---|---|---|---|
| bom | off | 0 | 3 | 0.00–0.56 | 6.0 | 0.008 |
| bom | on | 2 | 3 | 0.21–0.94 | 7.7 | 0.006 |
| bom | stale | 0 | 3 | 0.00–0.56 | 8.3 | 0.006 |
| rounding | off | 0 | 3 | 0.00–0.56 | 5.7 | 0.007 |
| rounding | on | 2 | 3 | 0.21–0.94 | 6.7 | 0.005 |
| rounding | stale | 0 | 3 | 0.00–0.56 | 7.3 | 0.005 |
| clamp (control) | off | 3 | 3 | 0.44–1.00 | 5.3 | 0.007 |
| clamp (control) | on | 3 | 3 | 0.44–1.00 | 7.7 | 0.006 |

`on`: Vexen installed by `vexen install`, an applicable recorded failure, and a sentence asking the agent to call `vexen_check` first. `stale`: same, but the anchored file changed after the note was recorded, so Vexen has nothing to say. `off`: no Vexen.

## What this does and does not show

- **Mechanism works end to end.** On both tasks that carry a requirement the visible tests hide, the agent without memory failed the hidden test 0 of 3 times, and with an applicable note it passed 2 of 3. The two tasks were chosen so that the missing knowledge lives in a recorded note, so this shows the path can help; it does not say how often real repositories have such a note.
- **Not just the extra prompt sentence.** `stale` carries the same sentence and the same MCP server but no applicable evidence, and it behaved like `off` (0 of 3 on both tasks). The lift appears only when Vexen returns an applicable note.
- **No negative transfer observed.** With nothing applicable to say (`stale`, and `clamp` where the only note concerns another file), Vexen did not make results worse: `clamp` passed 3 of 3 with and without it. Cost and turns were similar (a turn or two more when Vexen is present, a fraction of a cent either way).
- **The sample is tiny.** Three runs per cell, wide intervals that overlap (for example 0.00–0.56 versus 0.21–0.94). Treat the direction as suggestive, not as an effect size.
- **Limits of the design.** Toy Python tasks written by the people who built the tool, notes written to match the quirk, one model, one agent, one platform. Whether each `on` run actually called `vexen_check` was not logged per run; the pass pattern is consistent with it, but it is not proven here. The agent's token usage was not compared beyond the mean cost shown.

The paired study the specification calls for (many real tasks, confidence intervals, negative-transfer cases, more than one agent) is still to be done.
