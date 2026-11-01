"""Task definitions for the paired memory pilot.

Each task is a tiny Python repository with a requirement the visible tests do
not reveal. The hidden test checks it. A recorded failed attempt (the "note")
carries the missing knowledge. Notes are anchored to the file the agent must
edit, so they are applicable at the start of a run.

- quirk tasks: the note is applicable; memory should help.
- the control task: the note is about another file; Vexen should stay quiet and
  memory should neither help nor hurt.
"""

TASKS = {
    "bom": {
        "kind": "quirk",
        "files": {
            "config.py": '"""Config helpers."""\n\n\ndef load_names(path):\n    """Return the list under the "names" key of the JSON file at path."""\n    raise NotImplementedError\n',
            "tests/test_visible.py": 'import json, os, tempfile, unittest\nimport config\n\n\nclass T(unittest.TestCase):\n    def test_plain(self):\n        with tempfile.TemporaryDirectory() as d:\n            p = os.path.join(d, "n.json")\n            with open(p, "w", encoding="utf-8") as f:\n                json.dump({"names": ["a", "b"]}, f)\n            self.assertEqual(config.load_names(p), ["a", "b"])\n\n\nif __name__ == "__main__":\n    unittest.main()\n',
        },
        "prompt": 'Implement `load_names(path)` in config.py. It reads a JSON file shaped like {"names": [...]} written by a partner\'s exporter and returns the list. Make tests/test_visible.py pass (run it with `py -3 -m unittest discover -s tests`). Keep the change minimal.',
        "target": "config.py",
        "note": "Reading the partner export with json.load(open(path)) failed: the files start with a UTF-8 BOM. Open them with encoding utf-8-sig.",
        "hidden": 'import os, sys, tempfile\nsys.path.insert(0, ".")\nimport config\nwith tempfile.TemporaryDirectory() as d:\n    p = os.path.join(d, "n.json")\n    with open(p, "wb") as f:\n        f.write(b"\\xef\\xbb\\xbf" + b\'{"names": ["a", "b"]}\')\n    assert config.load_names(p) == ["a", "b"], "BOM file"\nprint("hidden ok")\n',
    },
    "rounding": {
        "kind": "quirk",
        "files": {
            "pricing.py": '"""Pricing helpers."""\n\n\ndef total(prices):\n    """Sum prices given as decimal strings; return a string with 2 decimals."""\n    raise NotImplementedError\n',
            "tests/test_visible.py": 'import unittest\nimport pricing\n\n\nclass T(unittest.TestCase):\n    def test_sum(self):\n        self.assertEqual(pricing.total(["1.00", "2.50"]), "3.50")\n\n    def test_empty(self):\n        self.assertEqual(pricing.total([]), "0.00")\n\n\nif __name__ == "__main__":\n    unittest.main()\n',
        },
        "prompt": "Implement `total(prices)` in pricing.py: it sums prices given as decimal strings and returns the total as a string with exactly 2 decimals. Make tests/test_visible.py pass (run it with `py -3 -m unittest discover -s tests`). Keep the change minimal.",
        "target": "pricing.py",
        "note": "Rounding the total with Decimal.quantize at its default or with round() failed the accounting tests: this repo rounds half up, so 0.125 must become 0.13. Use ROUND_HALF_UP.",
        "hidden": 'import sys\nsys.path.insert(0, ".")\nimport pricing\nassert pricing.total(["0.125"]) == "0.13", pricing.total(["0.125"])\nassert pricing.total(["0.135", "0.005"]) == "0.14", pricing.total(["0.135", "0.005"])\nprint("hidden ok")\n',
    },
    "clamp": {
        "kind": "control",
        "files": {
            "mathx.py": '"""Small math helpers."""\n\n\ndef clamp(x, lo, hi):\n    """Return x limited to the closed range [lo, hi]."""\n    raise NotImplementedError\n',
            "pricing.py": '"""Unrelated module with a recorded failure of its own."""\n\n\ndef total(prices):\n    raise NotImplementedError\n',
            "tests/test_visible.py": 'import unittest\nimport mathx\n\n\nclass T(unittest.TestCase):\n    def test_inside(self):\n        self.assertEqual(mathx.clamp(5, 0, 10), 5)\n\n    def test_below(self):\n        self.assertEqual(mathx.clamp(-1, 0, 10), 0)\n\n\nif __name__ == "__main__":\n    unittest.main()\n',
        },
        "prompt": "Implement `clamp(x, lo, hi)` in mathx.py. Make tests/test_visible.py pass (run it with `py -3 -m unittest discover -s tests`). Keep the change minimal.",
        "target": "mathx.py",
        "note_file": "pricing.py",
        "note": "Rounding the total with Decimal.quantize at its default failed the accounting tests: this repo rounds half up. Use ROUND_HALF_UP.",
        "hidden": 'import sys\nsys.path.insert(0, ".")\nimport mathx\nassert mathx.clamp(11, 0, 10) == 10\nassert mathx.clamp(10, 0, 10) == 10\nassert mathx.clamp(0, 0, 10) == 0\nprint("hidden ok")\n',
    },
}

from tasks_extra import EXTRA  # noqa: E402

TASKS.update(EXTRA)
