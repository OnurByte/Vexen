"""Regenerate README SVG cards: uv run --with fonttools python scripts/generate_readme_cards.py.

Cards are 720 units wide with no text below 24 units, so they stay readable when a phone
scales them to about 360 px (12 px text). Text is outlined from the bundled Geist fonts.
"""
from pathlib import Path
from html import escape
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen

root = Path(__file__).resolve().parents[1]
fonts = {k: TTFont(root / 'docs/assets/fonts' / v) for k, v in [('regular', 'Geist-Regular.otf'), ('bold', 'Geist-SemiBold.otf')]}
W = 720
PAD = 32
MIN = 24
BLUE, SOFT, INK, CARD, BG = '#78a9ff', '#a6c8ff', '#f2f4f8', '#262626', '#161616'


def width(s, size, weight='regular'):
    f = fonts[weight]
    cmap = f.getBestCmap()
    return sum(f['hmtx'][cmap[ord(c)]][0] for c in s) * size / f['head'].unitsPerEm


def text(s, x, y, size=MIN, color=INK, weight='regular'):
    assert size >= MIN, f'text under {MIN} units: {s!r}'
    f = fonts[weight]
    gs = f.getGlyphSet()
    cmap = f.getBestCmap()
    scale = size / f['head'].unitsPerEm
    paths, offset = [], 0
    for c in s:
        name = cmap[ord(c)]
        pen = SVGPathPen(gs)
        gs[name].draw(pen)
        paths.append(f'<path transform="translate({offset},0)" d="{pen.getCommands()}"/>')
        offset += f['hmtx'][name][0]
    return f'<g fill="{color}" transform="translate({x},{y}) scale({scale},-{scale})">' + ''.join(paths) + '</g>'


def wrap(s, size, max_w, weight='regular'):
    lines, cur = [], ''
    for word in s.split():
        trial = f'{cur} {word}'.strip()
        if width(trial, size, weight) <= max_w:
            cur = trial
        else:
            lines.append(cur)
            cur = word
    return lines + [cur]


class Card:
    """A vertical flow: add rows, then render. Every row reports its own height."""

    def __init__(self):
        self.parts, self.y = [], 0

    def gap(self, n):
        self.y += n

    def para(self, s, size=MIN, color=INK, weight='regular', x=PAD, max_w=W - 2 * PAD, lead=1.3):
        for line in wrap(s, size, max_w, weight):
            self.y += size * lead if self.y else 0
            self.parts.append(text(line, x, round(self.y), size, color, weight))
        return self

    def box(self, build, stroke='none', pad=24):
        top = self.y
        sub = Card()
        sub.y = 0
        build(sub, PAD + pad, W - 2 * (PAD + pad))
        h = round(sub.y + pad * 1.2)
        self.parts.append(f'<rect x="{PAD}" y="{round(top)}" width="{W - 2 * PAD}" height="{h}" rx="20" fill="{CARD}" stroke="{stroke}" stroke-width="2"/>')
        shift = round(top)
        for p in sub.parts:
            self.parts.append(f'<g transform="translate(0,{shift})">{p}</g>')
        self.y = top + h
        return self


def save(name, title, desc, build):
    c = Card()
    c.y = 0
    build(c)
    h = round(c.y + PAD)
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {h}" role="img" aria-labelledby="title desc">'
           f'<title id="title">{escape(title)}</title><desc id="desc">{escape(desc)}</desc>'
           f'<rect width="{W}" height="{h}" rx="20" fill="{BG}"/>' + ''.join(c.parts) + '</svg>')
    (root / 'docs/assets' / name).write_text(svg, encoding='utf-8')


def head(c, title, sub):
    c.y = PAD + 18 - round(36 * 1.25)
    c.para(title, 36, INK, 'bold', lead=1.25)
    c.y += 14
    c.para(sub, MIN, SOFT)
    c.gap(28)


def block(label, heading, lines, label_color=SOFT, stroke='none'):
    def build(s, x, w):
        s.y = 20
        s.para(label, MIN, label_color, 'bold', x=x, max_w=w, lead=1.0)
        s.gap(18)
        s.para(heading, 32, INK, 'bold', x=x, max_w=w)
        s.gap(10)
        for line in lines:
            s.para(line, MIN, SOFT, x=x, max_w=w)
    return build, stroke


def add(c, label, heading, lines, label_color=SOFT, stroke='none'):
    build, st = block(label, heading, lines, label_color, stroke)
    c.box(build, st)
    c.gap(20)


def before_after(c):
    head(c, 'A new chat. The same old mistake?', 'Keep the lesson when you switch sessions or agents.')
    add(c, 'WITHOUT VEXEN', 'Start from scratch.',
        ['Monday, Claude: tries a fix and the test fails.', 'Friday, Codex: a fresh chat may try it again.', 'You explain the same lesson again.'])
    add(c, 'WITH VEXEN', 'Pick up the lesson.',
        ['Monday, Claude: saves the failure and the test result.', 'Friday, Codex: checks what failed before editing.', 'Then checks whether it still fits the code.'], BLUE, BLUE)
    c.para('Illustrative scenario. Notes must be saved; Vexen does not record your chats.', MIN, SOFT)


def benefits(c):
    c.y = PAD - 20
    for num, h, ls in [('01', 'Keep the lesson.', ['Failed fixes and key decisions,', 'ready for the next session.']),
                       ('02', 'See the proof.', ['Find the file and test result', 'behind a saved reminder.']),
                       ('03', 'Keep it local.', ['On your machine. No account,', 'no cloud, no telemetry.'])]:
        add(c, num, h, ls, BLUE)
    c.y -= 20


def tiles(c, items):
    """Two stat tiles side by side: a big figure, what it is, and the counts behind it."""
    tile_w = (W - 2 * PAD - 16) // 2
    top = round(c.y)
    for i, (num, label, counts, color, stroke) in enumerate(items):
        x = PAD + i * (tile_w + 16)
        c.parts.append(f'<rect x="{x}" y="{top}" width="{tile_w}" height="176" rx="20" fill="{CARD}" stroke="{stroke}" stroke-width="2"/>')
        c.parts.append(text(num, x + 24, top + 78, 60, color, 'bold'))
        c.parts.append(text(label, x + 24, top + 122, MIN, INK))
        c.parts.append(text(counts, x + 24, top + 156, MIN, SOFT))
    c.y = top + 176 + 28


def evidence(c):
    head(c, 'Does remembering past mistakes help?', 'A small internal coding experiment.')
    tiles(c, [('10%', 'solved without memory', '2 of 20 runs', INK, 'none'), ('55%', 'solved with memory', '11 of 20 runs', BLUE, BLUE)])
    c.para('That is 45 percentage points more tasks solved.', MIN, INK, 'bold')
    c.gap(10)
    c.para('Five synthetic tasks, one model, pre-recorded evidence, and explicit instructions to check memory. These results do not establish general coding improvement.', MIN, SOFT)


def research(items):
    def build(c):
        c.y = PAD - 20
        for label, headline, sub, lines, limits in items:
            def b(s, x, w, label=label, headline=headline, sub=sub, lines=lines, limits=limits):
                s.y = 20
                s.para(label, MIN, BLUE, 'bold', x=x, max_w=w, lead=1.0)
                s.gap(18)
                s.para(headline, 34, INK, 'bold', x=x, max_w=w)
                s.para(sub, MIN, SOFT, x=x, max_w=w)
                s.gap(16)
                s.para(' '.join(lines), MIN, INK, x=x, max_w=w)
                s.gap(12)
                s.para(' '.join(limits), MIN, SOFT, x=x, max_w=w)
            c.box(b)
            c.gap(20)
        c.y -= 20
    return build


def flat(items):
    return ' '.join(' '.join([a, b, c, *d, *e]) for a, b, c, d, e in items)


save('readme-before-after.svg', 'Same project. A better starting point.',
     'Illustrative scenario: without saved memory a new agent may repeat a failed fix. With Vexen it can check the earlier failure and its test result before deciding what to try.', before_after)
save('readme-benefits.svg', 'Keep what you learned. Keep control.',
     'Save failed attempts and decisions. Check the file and test result behind a reminder. Keep memory locally with no account or telemetry.', benefits)
save('readme-evidence.svg', 'Does remembering help? A small internal coding pilot.',
     'On five synthetic tasks with hidden requirements, relevant pre-recorded Vexen memory solved 55% of runs (11 of 20), compared with 10% (2 of 20) without memory, 45 percentage points more. One model, explicit tool-use instructions. Not a general coding performance claim.', evidence)

RELEVANCE = [
    ('RELEVANCE CHECK · MEMORY PILOT', 'All 20 stale notes stayed out', 'every run returned no relevant evidence',
     ['After the file changed, the old note stayed out of the answer.'],
     ['Five synthetic tasks, four runs each. Measures filtering, not task success.']),
    ('CONTROL TASKS · MEMORY PILOT', 'All 8 control runs solved', 'with Vexen and without it',
     ['Unrelated notes stayed quiet. Both groups solved all control runs.'],
     ['Two synthetic tasks, four runs each. Small sample; no general safety claim.'])]
OPEN_TESTS = [
    ('CONNECTION CHECK · CLIENT STUDY', '13 more clients connected', 'each completed a real Vexen read call',
     ['Installed clients requested a note and received the expected record.'],
     ['Linux, Vexen 0.2.5; mostly scripted drivers. Checks connections, not coding improvement.']),
    ('PUBLISHED FAILURE · FILE HINT STUDY', 'Lost in 10 of 10 repositories', 'our file-hint method, 1,874 held-out queries',
     ['Our file-hint method lost to simple history counts in every repository.'],
     ['File hints remain research-only. This study did not measure coding success.'])]
save('readme-relevance.svg', 'Evidence beyond the headline', flat(RELEVANCE), research(RELEVANCE))
save('readme-open-tests.svg', 'Evidence beyond the headline', flat(OPEN_TESTS), research(OPEN_TESTS))


def clients(c):
    import json
    import re
    labels = json.loads((root / 'docs/assets/icons/sources.json').read_text(encoding='utf-8'))['icons'][:18]
    head(c, 'Your agents. One project memory.', 'Pick up where your last session left off.')
    cols, gap, tile_h = 2, 12, 64
    tile_w = (W - 2 * PAD - gap) // cols
    top = round(c.y)
    for i, item in enumerate(labels):
        x = PAD + (i % cols) * (tile_w + gap)
        y = top + (i // cols) * (tile_h + gap)
        raw = (root / 'docs/assets/icons' / f"{item['id']}.svg").read_text(encoding='utf-8')
        inner = re.sub(r'^<svg[^>]*>', '', raw.strip())[:-len('</svg>')]
        inner = re.sub(r'<title>.*?</title>', '', inner)
        c.parts.append(f'<rect x="{x}" y="{y}" width="{tile_w}" height="{tile_h}" rx="16" fill="{CARD}"/>')
        c.parts.append(f'<svg x="{x + 8}" y="{y + 8}" width="48" height="48" viewBox="0 0 48 48">{inner}</svg>')
        c.parts.append(text(item['label'], x + 72, y + 40, MIN, INK, 'bold'))
    rows = (len(labels) + cols - 1) // cols
    c.y = top + rows * (tile_h + gap) + 16
    c.para('Setup and test coverage vary by client. See the compatibility guide.', MIN, SOFT)


save('clients.svg', 'Your agents. One project memory.',
     'Coding tools with Vexen setup or runtime test coverage, including Claude Code, Codex, Cursor, Gemini CLI and VS Code. Setup steps and test coverage differ by tool.', clients)


def cost(c):
    head(c, '78% cheaper per solved task.', 'What one solved task cost in our pilot.')
    tiles(c, [('$0.048', 'without memory', 'per solved task', INK, 'none'), ('$0.011', 'with memory', 'per solved task', BLUE, BLUE)])
    c.para('A run itself cost about 11% less ($0.0073 to $0.0065). The big drop comes from more runs succeeding: 15% solved without memory, 60% with it.', MIN, INK)
    c.gap(10)
    c.para('One check adds at most 1,800 bytes, roughly 450 tokens.', MIN, INK)
    c.gap(10)
    c.para('20 runs per group, one model, synthetic tasks. Cost per solved task is a ratio of small counts. Not a general cost-saving claim.', MIN, SOFT)


save('readme-cost.svg', '78% cheaper per solved task.',
     'In a small internal pilot with one model, a solved task cost about 4.8 cents without memory and 1.1 cents with relevant Vexen memory, 78% less. A run cost about 11% less; the rest of the drop is because more runs succeeded (15% against 60%). A check adds at most 1,800 bytes, roughly 450 tokens. Twenty runs per group, synthetic tasks, not a general cost-saving claim.', cost)
