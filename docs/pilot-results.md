# Memory pilot: results

Two runs of the paired memory-on/off pilot in [`benchmarks/memory`](../benchmarks/memory/README.md), both with real Claude Code sessions (Haiku, 14-turn limit, fresh copy of the same repository per run, order shuffled with a fixed seed).

## Run 2 (76 runs, 7 tasks, 4 repetitions per cell)

Five *quirk* tasks hide a requirement that the visible tests do not show and a hidden test checks (UTF-8 BOM, half-up rounding, German umlaut transliteration, day-first dates, semicolon CSV). Two *control* tasks (`clamp`, `mean`) have nothing for memory to add; their only recorded note concerns another file. Raw rows: [`pilot-2.jsonl`](../benchmarks/memory/results/pilot-2.jsonl). Seven runs were first lost to a bug in my own log parser (not the agent) and were re-run, so all 76 cells have a result.

`on`: Vexen installed by `vexen install`, an applicable recorded failure, and one sentence asking the agent to call `vexen_check` first. `stale`: the same, but the anchored file changed after the note was recorded, so Vexen has nothing to say. `off`: no Vexen.

| kind | condition | passed | runs | 95% interval | agent called `vexen_check` |
|---|---|---|---|---|---|
| quirk | off | 2 | 20 | 0.03–0.30 | 0/20 |
| quirk | on | 11 | 20 | 0.34–0.74 | 20/20 |
| quirk | stale | 0 | 20 | 0.00–0.16 | 20/20 |
| control | off | 8 | 8 | 0.68–1.00 | 0/8 |
| control | on | 8 | 8 | 0.68–1.00 | 8/8 |

Per task:

| task | condition | passed | runs | 95% interval | called vexen_check | mean turns | mean cost USD |
|---|---|---|---|---|---|---|---|
| bom | off | 1 | 4 | 0.05–0.70 | 0/4 | 6.2 | 0.008 |
| bom | on | 3 | 4 | 0.30–0.95 | 4/4 | 8.0 | 0.006 |
| bom | stale | 0 | 4 | 0.00–0.49 | 4/4 | 7.5 | 0.006 |
| rounding | off | 1 | 4 | 0.05–0.70 | 0/4 | 5.8 | 0.007 |
| rounding | on | 2 | 4 | 0.15–0.85 | 4/4 | 6.8 | 0.006 |
| rounding | stale | 0 | 4 | 0.00–0.49 | 4/4 | 7.0 | 0.005 |
| slug | off | 0 | 4 | 0.00–0.49 | 0/4 | 4.0 | 0.007 |
| slug | on | 2 | 4 | 0.15–0.85 | 4/4 | 7.2 | 0.006 |
| slug | stale | 0 | 4 | 0.00–0.49 | 4/4 | 7.2 | 0.006 |
| dates | off | 0 | 4 | 0.00–0.49 | 0/4 | 4.0 | 0.006 |
| dates | on | 2 | 4 | 0.15–0.85 | 4/4 | 6.8 | 0.005 |
| dates | stale | 0 | 4 | 0.00–0.49 | 4/4 | 7.0 | 0.005 |
| csvrows | off | 0 | 4 | 0.00–0.49 | 0/4 | 5.2 | 0.007 |
| csvrows | on | 2 | 4 | 0.15–0.85 | 4/4 | 8.0 | 0.006 |
| csvrows | stale | 0 | 4 | 0.00–0.49 | 4/4 | 7.5 | 0.006 |
| clamp | off | 4 | 4 | 0.51–1.00 | 0/4 | 4.2 | 0.006 |
| clamp | on | 4 | 4 | 0.51–1.00 | 4/4 | 8.5 | 0.006 |
| mean | off | 4 | 4 | 0.51–1.00 | 0/4 | 4.2 | 0.006 |
| mean | on | 4 | 4 | 0.51–1.00 | 4/4 | 7.0 | 0.005 |

### What it shows

- **An applicable note changed the outcome.** On tasks with a hidden requirement the agent passed 2 of 20 times without memory and 11 of 20 with an applicable note; the intervals do not overlap.
- **It is the note, not the extra sentence or the tool.** The `stale` runs carry the same sentence and the same server, every one of them called `vexen_check`, and it returned no evidence; they passed 0 of 20.
- **No negative transfer here.** Where Vexen had nothing to say or nothing relevant (`stale`, and both controls), results did not get worse; the controls passed 8 of 8 either way. Vexen runs used about one to three more agent turns and, at Haiku prices, a fraction of a cent.
- **Even with the note, 9 of 20 runs still failed.** The agent received the note and did not always apply it; this tool does not make a model follow evidence.

### What it does not show

- The tasks are toy Python repositories written by the people who wrote the tool, and the notes were written to match the quirks. This shows the mechanism can help when an applicable note exists; it says nothing about how often real repositories have one.
- One model, one agent, one platform. Four repetitions per cell, so per-task intervals are wide, and the pooled intervals treat runs as independent although they share five task definitions.
- The agent was told to call `vexen_check`, as the skill instructs. How often an agent would do so unprompted was not tested.

The paired study the specification calls for (many real tasks, more than one agent, confidence intervals, negative-transfer cases) is still to be done.

## Run 1 (24 runs, 3 tasks, 3 repetitions per cell)

The first, smaller run of the same harness in [`benchmarks/memory`](../benchmarks/memory/README.md). 24 real Claude Code sessions (Haiku, 14-turn limit), 3 repetitions per cell, order shuffled with a fixed seed, no harness errors. Raw rows: [`pilot-1.jsonl`](../benchmarks/memory/results/pilot-1.jsonl).

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
