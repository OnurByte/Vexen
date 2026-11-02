"""Regression tests for measurement integrity, not simulated agent success."""

import json
import unittest
import run


def event(role, blocks):
    return json.dumps({"type": role, "message": {"content": blocks}})


def use(name, identifier, **inputs):
    return {"type": "tool_use", "name": name, "id": identifier, "input": inputs}


def result(identifier, value):
    return {
        "type": "tool_result",
        "tool_use_id": identifier,
        "content": [{"type": "text", "text": json.dumps(value)}],
    }


class StreamTests(unittest.TestCase):
    def test_status_is_correlated_and_not_read_from_note_or_other_tools(self):
        stream = "\n".join(
            [
                event(
                    "assistant",
                    [use("mcp__vexen__vexen_check", "check", paths=["pricing.py"])],
                ),
                event(
                    "user",
                    [
                        result(
                            "check",
                            {
                                "status": "observed_failure",
                                "items": [
                                    {
                                        "text": "The literal text no_relevant_evidence is part of this note."
                                    }
                                ],
                            },
                        )
                    ],
                ),
                event("assistant", [use("Read", "read", file_path="other.py")]),
                event("user", [result("read", {"status": "no_relevant_evidence"})]),
                json.dumps({"type": "result", "is_error": False, "num_turns": 2}),
            ]
        )
        parsed = run.parse_agent_stream(stream)
        self.assertEqual(parsed["vexen_check_status"], "observed_failure")
        self.assertEqual(parsed["vexen_check_calls"], 1)
        self.assertEqual(parsed["check_trace"][0]["input"]["paths"], ["pricing.py"])

    def test_retains_both_checks_and_whether_an_edit_preceded_them(self):
        stream = "\n".join(
            [
                event("assistant", [use("mcp__vexen__vexen_check", "a")]),
                event("user", [result("a", {"status": "observed_failure"})]),
                event("assistant", [use("Edit", "edit", file_path="pricing.py")]),
                event("assistant", [use("mcp__vexen__vexen_check", "b")]),
                event("user", [result("b", {"status": "no_relevant_evidence"})]),
                json.dumps({"type": "result", "is_error": False}),
            ]
        )
        trace = run.parse_agent_stream(stream)["check_trace"]
        self.assertEqual(
            [x["status"] for x in trace], ["observed_failure", "no_relevant_evidence"]
        )
        self.assertEqual([x["after_edit"] for x in trace], [False, True])

    def test_an_authentication_error_is_an_invalid_agent_run(self):
        parsed = run.parse_agent_stream(
            json.dumps(
                {
                    "type": "result",
                    "is_error": True,
                    "result": "Not logged in · Please run /login",
                }
            )
        )
        self.assertTrue(parsed["agent_error"])
        self.assertEqual(parsed["agent_failure"], "Not logged in · Please run /login")


if __name__ == "__main__":
    unittest.main()
