#!/usr/bin/env python3
"""Paired memory-on/off pilot with a real coding agent (Claude Code).

For each task and repetition the same prompt is run in fresh copies of the same
repository under these conditions:

  off      no Vexen, no memory
  on       Vexen configured (installed by `vexen install`), an applicable note
           recorded with a failing outcome, and one sentence telling the agent
           to call vexen_check before editing
  stale    like `on`, but the anchored file changed after the note was
           recorded, so Vexen must abstain (negative-transfer check)

A hidden test decides success. Nothing here proves a general effect: it is a
small pilot, and results.md says so.

  py -3 benchmarks/memory/run.py --vexen PATH --claude PATH --reps 3 --out DIR
"""
import argparse, json, os, random, shutil, subprocess, sys, tempfile, time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from tasks import TASKS  # noqa: E402


def sh(args, cwd=None, inp=None, check=True):
    p = subprocess.run(args, cwd=cwd, input=inp, capture_output=True, text=True, encoding="utf-8")
    if check and p.returncode != 0:
        raise RuntimeError(f"{args} failed: {p.stdout}{p.stderr}")
    return p


def make_repo(task, root):
    for rel, body in task["files"].items():
        f = root / rel
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text(body, encoding="utf-8", newline="\n")
    sh(["git", "init", "-q"], root)
    sh(["git", "config", "user.email", "pilot@example.invalid"], root)
    sh(["git", "config", "user.name", "pilot"], root)
    sh(["git", "add", "-A"], root)
    sh(["git", "commit", "-qm", "initial"], root)


def add_memory(vexen, task, root, stale):
    sh([vexen, "init", "--repo", str(root)])
    note_file = task.get("note_file", task["target"])
    out = sh([vexen, "record", "--repo", str(root), "--path", note_file, "--summary", task["note"]]).stdout
    event = json.loads(out)["items"][0]["event_id"]
    sh([vexen, "outcome", event, "--test-id", "recorded_failure", "--exit-code", "1", "--repo", str(root)])
    sh([vexen, "install", "--agent", "claude-code", "--scope", "project", "--repo", str(root), "--command", vexen])
    if stale:
        f = root / task["target"]
        f.write_text(f.read_text(encoding="utf-8") + "\n# edited after the note was recorded\n", encoding="utf-8", newline="\n")
        sh(["git", "add", "-A"], root)
        sh(["git", "commit", "-qm", "edit anchored file"], root)


def run_agent(claude, task, root, with_vexen):
    prompt = task["prompt"]
    tools = ["Read", "Edit", "Write", "Glob", "Grep", "Bash(py:*)"]
    cmd = [claude, "-p", "--model", "haiku", "--max-turns", "14", "--output-format", "stream-json", "--verbose",
           "--permission-mode", "acceptEdits"]
    if with_vexen:
        prompt += (f' This repository uses Vexen memory. Before editing, call the MCP tool vexen_check with repo_root '
                   f'"{root}" and the path of the file you will change; treat what it returns as advisory evidence '
                   f'about past attempts and verify it against the code.')
        tools.append("mcp__vexen__vexen_check")
        cmd += ["--mcp-config", str(root / ".mcp.json"), "--strict-mcp-config"]
    cmd += ["--allowedTools", ",".join(tools)]
    t0 = time.time()
    p = sh(cmd, cwd=root, inp=prompt, check=False)
    wall = time.time() - t0
    result, check_calls, check_status = {}, 0, None
    for line in p.stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        if ev.get("type") == "result":
            result = ev
        message = ev.get("message")
        for block in (message.get("content") if isinstance(message, dict) else None) or []:
            if not isinstance(block, dict):
                continue
            if block.get("type") == "tool_use" and "vexen_check" in str(block.get("name", "")):
                check_calls += 1
            if block.get("type") == "tool_result" and "no_relevant_evidence" in json.dumps(block.get("content", "")):
                check_status = "no_relevant_evidence"
            elif block.get("type") == "tool_result" and "observed_failure" in json.dumps(block.get("content", "")):
                check_status = "observed_failure"
    return {
        "turns": result.get("num_turns"),
        "cost_usd": result.get("total_cost_usd"),
        "agent_error": bool(result.get("is_error")) or not result,
        "wall_s": round(wall, 1),
        "vexen_check_calls": check_calls,
        "vexen_check_status": check_status,
    }


def hidden_pass(task, root):
    (root / "hidden_test.py").write_text(task["hidden"], encoding="utf-8", newline="\n")
    p = sh(["py", "-3", "hidden_test.py"], root, check=False)
    return p.returncode == 0


def wilson(k, n, z=1.96):
    if n == 0:
        return (0.0, 0.0)
    p = k / n
    d = 1 + z * z / n
    c = p + z * z / (2 * n)
    m = z * ((p * (1 - p) / n + z * z / (4 * n * n)) ** 0.5)
    return ((c - m) / d, (c + m) / d)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--vexen", required=True)
    ap.add_argument("--claude", required=True)
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--out", required=True)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--only", default="", help="comma-separated task:condition:rep cells to run (re-runs)")
    a = ap.parse_args()
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    plan = []
    for name, task in TASKS.items():
        conds = ["off", "on"] + (["stale"] if task["kind"] == "quirk" else [])
        for rep in range(a.reps):
            for c in conds:
                plan.append((name, c, rep))
    random.Random(a.seed).shuffle(plan)
    if a.only:
        keep = {tuple(c.split(":")[:2]) + (int(c.split(":")[2]),) for c in a.only.split(",")}
        plan = [x for x in plan if x in keep]
    rows = []
    with open(out / "results.jsonl", "w", encoding="utf-8") as log:
        for i, (name, cond, rep) in enumerate(plan, 1):
            task = TASKS[name]
            work = Path(tempfile.mkdtemp(prefix="vexen-pilot-"))
            root = work / "repo"
            root.mkdir()
            try:
                make_repo(task, root)
                if cond != "off":
                    add_memory(a.vexen, task, root, stale=(cond == "stale"))
                r = run_agent(a.claude, task, root, with_vexen=(cond != "off"))
                r.update(task=name, kind=task["kind"], condition=cond, rep=rep, passed=hidden_pass(task, root))
            except Exception as e:  # keep going; record the failure of the harness itself
                r = {"task": name, "kind": task["kind"], "condition": cond, "rep": rep, "passed": None, "harness_error": str(e)[:300]}
            rows.append(r)
            log.write(json.dumps(r) + "\n")
            log.flush()
            print(f"[{i}/{len(plan)}] {name:9} {cond:6} rep{rep} passed={r.get('passed')} turns={r.get('turns')}", flush=True)
            shutil.rmtree(work, ignore_errors=True)

    lines = ["| task | condition | passed | runs | 95% interval | mean turns | mean cost USD |", "|---|---|---|---|---|---|---|"]
    for name in TASKS:
        for cond in ("off", "on", "stale"):
            rs = [r for r in rows if r["task"] == name and r["condition"] == cond and r.get("passed") is not None]
            if not rs:
                continue
            k = sum(1 for r in rs if r["passed"])
            lo, hi = wilson(k, len(rs))
            turns = [r["turns"] for r in rs if r.get("turns") is not None]
            cost = [r["cost_usd"] for r in rs if r.get("cost_usd") is not None]
            lines.append(f"| {name} | {cond} | {k} | {len(rs)} | {lo:.2f}–{hi:.2f} | {sum(turns)/len(turns) if turns else float('nan'):.1f} | {sum(cost)/len(cost) if cost else float('nan'):.3f} |")
    broken = [r for r in rows if r.get("passed") is None]
    (out / "table.md").write_text("\n".join(lines) + f"\n\nharness errors (excluded): {len(broken)}\n", encoding="utf-8")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
