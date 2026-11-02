#!/usr/bin/env python3
"""Check relative links in the repository's Markdown files.

Looks at Markdown links and HTML src/href attributes that point at files in this
repository, and at #anchors that point at a heading. It does not touch the
network. Exit status 1 if anything is broken.

    python scripts/check-docs-links.py
"""
import re
import subprocess
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MD_LINK = re.compile(r"(?<!!)\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)|!\[[^\]]*\]\(([^)\s]+)\)")
HTML_ATTR = re.compile(r"""(?:src|srcset|href)=["']([^"']+)["']""")
HEADING = re.compile(r"^\s{0,3}(#{1,6})\s+(.*?)\s*#*\s*$")
FENCE = re.compile(r"^\s*(```|~~~)")


def tracked_markdown():
    out = subprocess.run(
        ["git", "ls-files", "*.md"], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout.split("\n")
    return [ROOT / p for p in out if p]


def slug(text, seen):
    """GitHub's heading anchor: lowercase, drop punctuation, spaces to hyphens."""
    text = re.sub(r"<[^>]+>", "", text)
    text = re.sub(r"[`*~]", "", text)
    text = unicodedata.normalize("NFKC", text).strip().lower()
    text = "".join(c for c in text if c.isalnum() or c in " -_")
    base = text.replace(" ", "-")
    n = seen.get(base, 0)
    seen[base] = n + 1
    return base if n == 0 else f"{base}-{n}"


def anchors(path, cache={}):
    if path not in cache:
        found, seen, fenced = set(), {}, False
        for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
            if FENCE.match(line):
                fenced = not fenced
                continue
            m = None if fenced else HEADING.match(line)
            if m:
                found.add(slug(m.group(2), seen))
            for a in re.findall(r"""<a\s+(?:name|id)=["']([^"']+)["']""", line):
                found.add(a)
        cache[path] = found
    return cache[path]


def links(path):
    text = path.read_text(encoding="utf-8", errors="replace")
    fenced = False
    for number, line in enumerate(text.splitlines(), 1):
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        line = re.sub(r"`[^`]*`", "", line)
        for m in MD_LINK.finditer(line):
            yield number, m.group(1) or m.group(2)
        for m in HTML_ATTR.finditer(line):
            yield number, m.group(1).split(",")[0].split()[0]


def main():
    bad = []
    for md in tracked_markdown():
        for number, target in links(md):
            if re.match(r"^[a-z][a-z0-9+.-]*:", target, re.I) or target.startswith("//"):
                continue
            target = target.strip("<>")
            file_part, _, frag = target.partition("#")
            dest = (md.parent / file_part).resolve() if file_part else md
            rel = md.relative_to(ROOT)
            if file_part and not dest.exists():
                bad.append(f"{rel}:{number}: missing file {target}")
                continue
            if frag and dest.suffix == ".md" and dest.is_file() and frag not in anchors(dest):
                bad.append(f"{rel}:{number}: no heading #{frag} in {dest.relative_to(ROOT)}")
    if bad:
        print("\n".join(bad))
        print(f"\n{len(bad)} broken link(s)")
        return 1
    print("docs links ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
