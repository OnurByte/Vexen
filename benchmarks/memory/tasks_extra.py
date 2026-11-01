"""Second batch of pilot tasks (see tasks.py for the design).

Three more tasks whose hidden requirement is not visible in the visible tests,
and a second control. All code bodies are raw strings so escapes stay literal.
"""

EXTRA = {
    "slug": {
        "kind": "quirk",
        "files": {
            "text.py": r'''"""Text helpers."""


def slugify(text):
    """Return a lowercase URL slug: words joined by single hyphens."""
    raise NotImplementedError
''',
            "tests/test_visible.py": r'''import unittest
import text


class T(unittest.TestCase):
    def test_basic(self):
        self.assertEqual(text.slugify("Hello World"), "hello-world")

    def test_punctuation(self):
        self.assertEqual(text.slugify("  A  quick, test! "), "a-quick-test")


if __name__ == "__main__":
    unittest.main()
''',
        },
        "prompt": "Implement `slugify(text)` in text.py: it returns a lowercase URL slug, words joined by single hyphens. Make tests/test_visible.py pass (run it with `py -3 -m unittest discover -s tests`). Keep the change minimal.",
        "target": "text.py",
        "note": "Stripping accents with unicodedata NFKD failed the German catalogue tests: this repo transliterates umlauts, so a-umlaut becomes ae, o-umlaut oe, u-umlaut ue and the sharp s becomes ss.",
        "hidden": r'''import sys
sys.path.insert(0, ".")
import text
got = text.slugify("Grüße aus München")
assert got == "gruesse-aus-muenchen", got
print("hidden ok")
''',
    },
    "dates": {
        "kind": "quirk",
        "files": {
            "dates.py": r'''"""Date helpers."""


def parse_day(s):
    """Parse a date written by the partner export and return (year, month, day)."""
    raise NotImplementedError
''',
            "tests/test_visible.py": r'''import unittest
import dates


class T(unittest.TestCase):
    def test_iso(self):
        self.assertEqual(dates.parse_day("2025-03-14"), (2025, 3, 14))


if __name__ == "__main__":
    unittest.main()
''',
        },
        "prompt": "Implement `parse_day(s)` in dates.py: it parses a date string from a partner export and returns (year, month, day) as integers. Make tests/test_visible.py pass (run it with `py -3 -m unittest discover -s tests`). Keep the change minimal.",
        "target": "dates.py",
        "note": "Parsing the export with strptime and a month-first format failed: the partner export writes day-first dates such as 04/03/2025 (4 March), separated by slashes. Accept ISO and day-first.",
        "hidden": r'''import sys
sys.path.insert(0, ".")
import dates
got = dates.parse_day("04/03/2025")
assert got == (2025, 3, 4), got
assert dates.parse_day("2025-03-14") == (2025, 3, 14)
print("hidden ok")
''',
    },
    "csvrows": {
        "kind": "quirk",
        "files": {
            "rows.py": r'''"""Row helpers."""


def read_rows(path):
    """Return the rows of the partner CSV export as lists of strings."""
    raise NotImplementedError
''',
            "tests/test_visible.py": r'''import os, tempfile, unittest
import rows


class T(unittest.TestCase):
    def test_two_rows(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "x.csv")
            with open(p, "w", encoding="utf-8", newline="") as f:
                f.write("a,b\n1,2\n")
            self.assertEqual(rows.read_rows(p), [["a", "b"], ["1", "2"]])


if __name__ == "__main__":
    unittest.main()
''',
        },
        "prompt": "Implement `read_rows(path)` in rows.py: it reads the partner CSV export and returns its rows as lists of strings. Make tests/test_visible.py pass (run it with `py -3 -m unittest discover -s tests`). Keep the change minimal.",
        "target": "rows.py",
        "note": "Reading the partner export with the csv module's default comma delimiter failed: those exports use a semicolon as the delimiter. Detect or accept the semicolon.",
        "hidden": r'''import os, sys, tempfile
sys.path.insert(0, ".")
import rows
with tempfile.TemporaryDirectory() as d:
    p = os.path.join(d, "x.csv")
    with open(p, "w", encoding="utf-8", newline="") as f:
        f.write("a;b\n1;2\n")
    got = rows.read_rows(p)
    assert got == [["a", "b"], ["1", "2"]], got
print("hidden ok")
''',
    },
    "mean": {
        "kind": "control",
        "files": {
            "stats.py": r'''"""Statistics helpers."""


def mean(xs):
    """Return the arithmetic mean of a non-empty list of numbers."""
    raise NotImplementedError
''',
            "dates.py": r'''"""Unrelated module with a recorded failure of its own."""


def parse_day(s):
    raise NotImplementedError
''',
            "tests/test_visible.py": r'''import unittest
import stats


class T(unittest.TestCase):
    def test_mean(self):
        self.assertEqual(stats.mean([1, 2, 3]), 2)


if __name__ == "__main__":
    unittest.main()
''',
        },
        "prompt": "Implement `mean(xs)` in stats.py. Make tests/test_visible.py pass (run it with `py -3 -m unittest discover -s tests`). Keep the change minimal.",
        "target": "stats.py",
        "note_file": "dates.py",
        "note": "Parsing the export with strptime and a month-first format failed: the partner export writes day-first dates. Accept day-first.",
        "hidden": r'''import sys
sys.path.insert(0, ".")
import stats
assert stats.mean([1, 2, 3, 4]) == 2.5
assert stats.mean([5]) == 5
print("hidden ok")
''',
    },
}
