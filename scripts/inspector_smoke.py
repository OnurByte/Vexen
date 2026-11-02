#!/usr/bin/env python3
"""Independent MCP check: drive Vexen's stdio server with the official MCP
Inspector (CLI mode) and verify what an agent would receive.

This is a development and CI check, not a runtime dependency: the Inspector needs
Node 22.19 or newer, Vexen itself does not need Node. The Inspector version is
pinned. It covers four situations, each checked in BOTH response forms (the text
block and the structured content), because a text-only client sees only the first:

  empty       nothing recorded                    -> no_relevant_evidence
  relevant    failing result for the asked file   -> the lesson text, observed_failure
  stale       the file changed after recording    -> no_relevant_evidence
  conflicted  a passing and a failing result      -> a conflicted item, not buried

A protocol check shows transport, schema and dispatch work; it does not show that
an agent solves tasks better.

    python scripts/inspector_smoke.py --vexen target/debug/vexen [--inspector 2.13.0]
"""
import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

LESSON = "Retrying after a timeout charged the customer twice"
MAX_TEXT_BYTES = 1800


def run(cmd, cwd=None, check=True):
    p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, encoding="utf-8")
    if check and p.returncode != 0:
        raise SystemExit(f"command failed: {cmd}\n{p.stdout}\n{p.stderr}")
    return p


def inspector(npx, version, vexen, method, *extra):
    cmd = [npx, "-y", f"@modelcontextprotocol/inspector@{version}", "--cli", vexen, "mcp", "--stdio", "--method", method, *extra]
    p = run(cmd, check=False)
    if p.returncode != 0:
        raise SystemExit(f"inspector failed: {method}\n{p.stdout}\n{p.stderr}")
    return json.loads(p.stdout)


def check(npx, version, vexen, root):
    return inspector(npx, version, vexen, "tools/call", "--tool-name", "vexen_check", "--tool-arg", f"repo_root={root}", "--tool-arg", 'paths=["retry.rs"]')


def both_forms(reply):
    """The text block parsed as JSON, and the structured content."""
    assert reply.get("isError") is False, reply
    text = reply["content"][0]["text"]
    assert len(text.encode("utf-8")) <= MAX_TEXT_BYTES, f"text block is {len(text.encode('utf-8'))} bytes"
    return json.loads(text), reply["structuredContent"]


def expect(name, condition, detail=""):
    print(f"  {'ok  ' if condition else 'FAIL'}  {name}{(': ' + str(detail)) if not condition else ''}")
    return bool(condition)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--vexen", required=True)
    ap.add_argument("--inspector", default="2.13.0")
    a = ap.parse_args()
    vexen = str(Path(a.vexen).resolve())
    npx = shutil.which("npx") or shutil.which("npx.cmd")
    if not npx:
        raise SystemExit("npx not found (Node 22.19 or newer is needed for the Inspector)")

    work = Path(tempfile.mkdtemp(prefix="vexen-inspector-"))
    repo = work / "repo"
    repo.mkdir()
    try:
        for c in (["init", "-q"], ["config", "user.email", "t@example.invalid"], ["config", "user.name", "t"]):
            run(["git", "-C", str(repo), *c])
        (repo / "retry.rs").write_text("fn retry() {}\n", encoding="utf-8", newline="\n")
        run(["git", "-C", str(repo), "add", "-A"])
        run(["git", "-C", str(repo), "commit", "-qm", "init"])
        root = str(repo.resolve())
        run([vexen, "init", "--repo", root])
        ok = True

        print(f"MCP Inspector {a.inspector} against {vexen}")
        tools = inspector(npx, a.inspector, vexen, "tools/list")
        names = sorted(t["name"] for t in tools["tools"])
        ok &= expect("tools/list offers the four tools", names == ["vexen_check", "vexen_explain", "vexen_outcome", "vexen_record"], names)

        text, structured = both_forms(check(npx, a.inspector, vexen, root))
        ok &= expect("empty: no relevant evidence in both forms", text["status"] == structured["status"] == "no_relevant_evidence" and not structured["items"], structured)

        rec = json.loads(run([vexen, "record", "--repo", root, "--path", "retry.rs", "--summary", LESSON]).stdout)
        event = rec["items"][0]["event_id"]
        run([vexen, "outcome", event, "--repo", root, "--test-id", "retry_no_double_charge", "--exit-code", "1"])
        text, structured = both_forms(check(npx, a.inspector, vexen, root))
        ok &= expect("relevant: the lesson is in the structured content", structured["status"] == "observed_failure" and LESSON in structured["items"][0]["text"], structured)
        ok &= expect("relevant: the lesson is in the TEXT block too (text-only clients)", LESSON in text["items"][0]["text"], text)

        run([vexen, "outcome", event, "--repo", root, "--test-id", "retry_no_double_charge", "--exit-code", "0", "--key", "passing-run"])
        text, structured = both_forms(check(npx, a.inspector, vexen, root))
        item = structured["items"][0] if structured["items"] else {}
        ok &= expect("conflicted: reported as a conflict, not buried", item.get("status") == "conflicted" and "Conflicting" in item.get("text", ""), structured)
        ok &= expect("conflicted: visible in the text block as well", bool(text["items"]) and text["items"][0]["status"] == "conflicted", text)

        (repo / "retry.rs").write_text("fn retry() { /* fixed */ }\n", encoding="utf-8", newline="\n")
        text, structured = both_forms(check(npx, a.inspector, vexen, root))
        ok &= expect("stale: silent after the file changed, in both forms", text["status"] == structured["status"] == "no_relevant_evidence" and not structured["items"] and not text["items"], structured)

        print("PASS" if ok else "FAIL")
        return 0 if ok else 1
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
