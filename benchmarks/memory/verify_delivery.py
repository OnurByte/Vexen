"""Verify seeded lesson delivery, NOT real-agent task success.

Build first: cargo build --locked
Run: python benchmarks/memory/verify_delivery.py [--vexen PATH]
"""

import argparse, hashlib, sys, tempfile, json, subprocess, queue, threading
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import run

repo = Path(__file__).resolve().parents[2]
ap = argparse.ArgumentParser(description=__doc__)
ap.add_argument("--vexen", default=str(repo / "target/debug/vexen"))
a = ap.parse_args()
binary = str(Path(a.vexen).resolve())


def receive(process):
    q = queue.Queue()
    threading.Thread(
        target=lambda: q.put(process.stdout.readline()), daemon=True
    ).start()
    try:
        return json.loads(q.get(timeout=15))
    except (queue.Empty, json.JSONDecodeError):
        process.kill()
        process.communicate(timeout=5)
        raise RuntimeError("MCP response missing or invalid")


rows = []
for name, task in run.TASKS.items():
    for condition in ("on", "stale") if task["kind"] == "quirk" else ("on",):
        with tempfile.TemporaryDirectory(prefix="vexen-seed-verify-") as work:
            root = Path(work) / "repo"
            root.mkdir()
            run.make_repo(task, root)
            run.add_memory(binary, task, root, condition == "stale")
            expected = (
                "observed_failure"
                if condition == "on" and task["kind"] == "quirk"
                else "no_relevant_evidence"
            )
            response = json.loads(
                run.sh(
                    [binary, "check", "--repo", str(root), "--path", task["target"]]
                ).stdout
            )
            assert response["status"] == expected, (name, condition, response)
            p = subprocess.Popen(
                [binary, "mcp", "--stdio"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )

            def send(msg):
                p.stdin.write(json.dumps(msg) + "\n")
                p.stdin.flush()

            send(
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2025-11-25",
                        "capabilities": {},
                        "clientInfo": {"name": "seed-diagnostic", "version": "1"},
                    },
                }
            )
            init = receive(p)
            assert init["id"] == 1
            send({"jsonrpc": "2.0", "method": "notifications/initialized"})
            send(
                {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "vexen_check",
                        "arguments": {
                            "repo_root": str(root),
                            "paths": [task["target"]],
                        },
                    },
                }
            )
            call = receive(p)["result"]
            text = json.loads(call["content"][0]["text"])
            assert text["status"] == expected, (name, condition, text)
            if expected == "observed_failure":
                assert task["note"] in text["items"][0]["text"], (name, text)
            else:
                assert text["items"] == []
            rows.append(
                {
                    "task": name,
                    "condition": condition,
                    "cli_status": response["status"],
                    "mcp_text_status": text["status"],
                    "lesson_visible": expected == "observed_failure",
                }
            )
            p.terminate()
            p.communicate(timeout=5)
print(
    json.dumps(
        {
            "kind": "deterministic_seed_and_mcp_delivery_check",
            "not_agent_task_outcomes": True,
            "vexen_sha256": hashlib.sha256(Path(binary).read_bytes()).hexdigest(),
            "checks": rows,
        },
        indent=2,
    )
)
