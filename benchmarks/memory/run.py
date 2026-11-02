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

import argparse, hashlib, json, os, random, shlex, shutil, subprocess, sys, tempfile, time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from tasks import TASKS  # noqa: E402


def sh(args, cwd=None, inp=None, check=True):
    p = subprocess.run(
        args, cwd=cwd, input=inp, capture_output=True, text=True, encoding="utf-8"
    )
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
    out = sh(
        [
            vexen,
            "record",
            "--repo",
            str(root),
            "--path",
            note_file,
            "--summary",
            task["note"],
        ]
    ).stdout
    event = json.loads(out)["items"][0]["event_id"]
    sh(
        [
            vexen,
            "outcome",
            event,
            "--test-id",
            "recorded_failure",
            "--exit-code",
            "1",
            "--repo",
            str(root),
        ]
    )
    sh(
        [
            vexen,
            "install",
            "--agent",
            "claude-code",
            "--scope",
            "project",
            "--repo",
            str(root),
            "--command",
            vexen,
        ]
    )
    if stale:
        f = root / task["target"]
        f.write_text(
            f.read_text(encoding="utf-8") + "\n# edited after the note was recorded\n",
            encoding="utf-8",
            newline="\n",
        )
        sh(["git", "add", "-A"], root)
        sh(["git", "commit", "-qm", "edit anchored file"], root)


def run_agent(claude, task, root, with_vexen, artifacts=None):
    prompt = task["prompt"].replace("py -3", shlex.quote(sys.executable))
    tools = ["Read", "Edit", "Write", "Glob", "Grep", f"Bash({sys.executable}:*)"]
    cmd = [
        claude,
        "-p",
        "--model",
        "haiku",
        "--max-turns",
        "14",
        "--output-format",
        "stream-json",
        "--verbose",
        "--permission-mode",
        "acceptEdits",
    ]
    if with_vexen:
        prompt += (
            f" This repository uses Vexen memory. Before editing, call the MCP tool vexen_check with repo_root "
            f'"{root}" and the path of the file you will change; treat what it returns as advisory evidence '
            f"about past attempts and verify it against the code."
        )
        tools.append("mcp__vexen__vexen_check")
        cmd += ["--mcp-config", str(root / ".mcp.json"), "--strict-mcp-config"]
    cmd += ["--allowedTools", ",".join(tools)]
    t0 = time.time()
    p = sh(cmd, cwd=root, inp=prompt, check=False)
    wall = time.time() - t0
    parsed = parse_agent_stream(p.stdout)
    parsed["wall_s"] = round(wall, 1)
    parsed["agent_returncode"] = p.returncode
    if p.returncode != 0:
        parsed["agent_error"] = True
        parsed["agent_failure"] = (
            parsed["agent_failure"] or f"agent exited with code {p.returncode}"
        )
    if artifacts is not None:
        artifacts.mkdir(parents=True, exist_ok=True)
        (artifacts / "agent.jsonl").write_text(p.stdout, encoding="utf-8")
        (artifacts / "agent.stderr").write_text(p.stderr, encoding="utf-8")
    return parsed


def parse_agent_stream(stream):
    """Correlate check responses by tool ID; preserve every check and edit order."""
    result, trace, pending, after_edit = {}, [], {}, False

    def status_from(content):
        if isinstance(content, str):
            try:
                return status_from(json.loads(content))
            except (json.JSONDecodeError, TypeError):
                return None
        if isinstance(content, list):
            for part in content:
                status = status_from(part)
                if status is not None:
                    return status
        if isinstance(content, dict):
            if content.get("status") in {
                "observed_failure",
                "accepted_decision",
                "no_relevant_evidence",
            }:
                return content["status"]
            for field in ("structuredContent", "text", "content"):
                status = status_from(content.get(field))
                if status is not None:
                    return status
        return None

    for line in stream.splitlines():
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(ev, dict):
            continue
        if ev.get("type") == "result":
            result = ev
        message = ev.get("message")
        blocks = message.get("content", []) if isinstance(message, dict) else []
        if not isinstance(blocks, list):
            continue
        for block in blocks:
            if not isinstance(block, dict):
                continue
            if block.get("type") == "tool_use":
                name = str(block.get("name", ""))
                if name in {"Edit", "Write", "MultiEdit", "NotebookEdit"}:
                    after_edit = True
                if name == "vexen_check" or name.endswith("__vexen_check"):
                    call = {
                        "tool_use_id": block.get("id"),
                        "input": block.get("input"),
                        "after_edit": after_edit,
                        "status": None,
                    }
                    trace.append(call)
                    pending[block.get("id")] = call
            elif block.get("type") == "tool_result":
                call = pending.get(block.get("tool_use_id"))
                if call is not None:
                    call["status"] = status_from(block.get("content"))
                    call["is_error"] = bool(block.get("is_error"))
    error = bool(result.get("is_error")) or not result
    return {
        "turns": result.get("num_turns"),
        "cost_usd": result.get("total_cost_usd"),
        "agent_error": error,
        "agent_failure": str(result.get("result", "missing agent result"))[:500]
        if error
        else None,
        "vexen_check_calls": len(trace),
        "vexen_check_status": trace[-1]["status"] if trace else None,
        "check_trace": trace,
    }


def hidden_pass(task, root, artifacts=None):
    (root / "hidden_test.py").write_text(task["hidden"], encoding="utf-8", newline="\n")
    p = sh([sys.executable, "hidden_test.py"], root, check=False)
    if artifacts is not None:
        (artifacts / "hidden.stdout").write_text(p.stdout, encoding="utf-8")
        (artifacts / "hidden.stderr").write_text(p.stderr, encoding="utf-8")
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
    ap.add_argument(
        "--keep-work", action="store_true", help="retain diagnostic task repositories"
    )
    ap.add_argument(
        "--only",
        default="",
        help="comma-separated task:condition:rep cells to run (re-runs)",
    )
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
        keep = {
            tuple(c.split(":")[:2]) + (int(c.split(":")[2]),) for c in a.only.split(",")
        }
        plan = [x for x in plan if x in keep]
    if (out / "results.jsonl").exists():
        ap.error(
            "output already has results.jsonl; use a new directory to preserve evidence"
        )
    a.vexen = str(Path(a.vexen).resolve())
    a.claude = shutil.which(a.claude) or str(Path(a.claude).resolve())
    manifest = {
        "vexen_sha256": hashlib.sha256(Path(a.vexen).read_bytes()).hexdigest(),
        "runner_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "source_commit": sh(
            ["git", "rev-parse", "HEAD"], Path(__file__).resolve().parents[2]
        ).stdout.strip(),
        "source_dirty": bool(
            sh(
                ["git", "status", "--porcelain"], Path(__file__).resolve().parents[2]
            ).stdout.strip()
        ),
        "python": sys.version,
        "python_executable": sys.executable,
        "planned_cells": [{"task": n, "condition": c, "rep": r} for n, c, r in plan],
        "seed": a.seed,
        "repetitions": a.reps,
        "claude_version": sh([a.claude, "--version"]).stdout.strip(),
        "task_sha256": {
            name: hashlib.sha256(json.dumps(task, sort_keys=True).encode()).hexdigest()
            for name, task in TASKS.items()
        },
    }
    (out / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
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
                preflight = None
                if cond != "off":
                    preflight = json.loads(
                        sh(
                            [
                                a.vexen,
                                "check",
                                "--repo",
                                str(root),
                                "--path",
                                task["target"],
                            ]
                        ).stdout
                    )
                    expected = (
                        "observed_failure"
                        if cond == "on" and task["kind"] == "quirk"
                        else "no_relevant_evidence"
                    )
                    if preflight["status"] != expected:
                        raise RuntimeError(
                            f"seed preflight: expected {expected}, got {preflight['status']}"
                        )
                artifacts = out / "traces" / f"{name}-{cond}-{rep}"
                r = run_agent(
                    a.claude,
                    task,
                    root,
                    with_vexen=(cond != "off"),
                    artifacts=artifacts,
                )
                r.update(
                    task=name,
                    kind=task["kind"],
                    condition=cond,
                    rep=rep,
                    preflight_status=preflight["status"] if preflight else None,
                    passed=None
                    if r["agent_error"]
                    else hidden_pass(task, root, artifacts=artifacts),
                )
                (artifacts / "target-after.py").write_bytes(
                    (root / task["target"]).read_bytes()
                )
                if a.keep_work:
                    r["workspace"] = str(root)
            except (
                Exception
            ) as e:  # keep going; record the failure of the harness itself
                r = {
                    "task": name,
                    "kind": task["kind"],
                    "condition": cond,
                    "rep": rep,
                    "passed": None,
                    "harness_error": str(e)[:300],
                }
            rows.append(r)
            log.write(json.dumps(r) + "\n")
            log.flush()
            print(
                f"[{i}/{len(plan)}] {name:9} {cond:6} rep{rep} passed={r.get('passed')} turns={r.get('turns')}",
                flush=True,
            )
            if not a.keep_work:
                shutil.rmtree(work, ignore_errors=True)

    lines = [
        "| task | condition | passed | runs | 95% interval | mean turns | mean cost USD |",
        "|---|---|---|---|---|---|---|",
    ]
    for name in TASKS:
        for cond in ("off", "on", "stale"):
            rs = [
                r
                for r in rows
                if r["task"] == name
                and r["condition"] == cond
                and r.get("passed") is not None
            ]
            if not rs:
                continue
            k = sum(1 for r in rs if r["passed"])
            lo, hi = wilson(k, len(rs))
            turns = [r["turns"] for r in rs if r.get("turns") is not None]
            cost = [r["cost_usd"] for r in rs if r.get("cost_usd") is not None]
            lines.append(
                f"| {name} | {cond} | {k} | {len(rs)} | {lo:.2f}–{hi:.2f} | {sum(turns) / len(turns) if turns else float('nan'):.1f} | {sum(cost) / len(cost) if cost else float('nan'):.3f} |"
            )
    broken = [r for r in rows if r.get("passed") is None]
    (out / "table.md").write_text(
        "\n".join(lines)
        + f"\n\ninvalid runs (harness or agent error, excluded): {len(broken)}\n",
        encoding="utf-8",
    )
    print("\n".join(lines))
    if broken:
        raise SystemExit(2)


if __name__ == "__main__":
    main()
