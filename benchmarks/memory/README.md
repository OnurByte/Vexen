# Memory-on/off pilot (Task 9, first step)

A small paired experiment with a real coding agent. It exists to exercise the whole path (record a failed attempt, install the MCP config, let an agent call `vexen_check`, score the outcome with a hidden test) and to look for **negative transfer**. It is not evidence of a general improvement.

## Design

- **Agent:** Claude Code (Haiku), same prompt, tools, turn limit and fresh copy of the same repository for every run. Order is shuffled with a fixed seed.
- **Tasks** (`tasks.py`, `tasks_extra.py`), tiny Python repositories:
  - `bom`, `rounding`, `slug`, `dates` and `csvrows` carry a requirement the visible tests do not show and a hidden test checks (a UTF-8 BOM in input files; half-up rounding). A recorded failed attempt with a failing outcome holds the missing knowledge. These are the *quirk* tasks.
  - `clamp` and `mean` are controls: the only note is about another file, so Vexen should abstain and memory should neither help nor hurt.
- **Conditions:**
  - `off`: no Vexen.
  - `on`: Vexen configured by `vexen install`, an applicable note, and one sentence asking the agent to call `vexen_check` before editing.
  - `stale` (quirk tasks only): the anchored file is edited after the note was recorded, so Vexen must return no evidence. This is the negative-transfer check: does having Vexen present with nothing to say change the result?
- **Outcome:** a hidden test passes or fails. Also recorded: agent turns and cost, and whether (and with what result) the agent called `vexen_check`.
- **Statistics:** pass counts with 95% Wilson intervals. With a handful of runs per cell the intervals are wide; read the table as a mechanics check.

## Run

```sh
py -3 benchmarks/memory/run.py \
  --vexen target/release/vexen --claude /path/to/claude --reps 3 --out /tmp/pilot
```

It writes `results.jsonl` (one row per run) and `table.md`. It uses your agent account and makes real model calls.

## Limits

Toy tasks written by the people who wrote the tool, one agent, one model, one platform, notes written to match the quirk. A positive result here shows the mechanism can help when an applicable note exists; it does not estimate how often that happens in real repositories. The paired study in the specification (many real tasks, memory-on vs memory-off, confidence intervals, negative-transfer cases) is still to be done.

## Results

See [`docs/pilot-results.md`](../../docs/pilot-results.md).
