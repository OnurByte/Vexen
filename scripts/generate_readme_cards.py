"""Regenerate README SVG cards: uv run --with fonttools python scripts/generate_readme_cards.py."""
from pathlib import Path
from html import escape
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
root=Path(__file__).resolve().parents[1]
fonts={k:TTFont(root/'docs/assets/fonts'/v) for k,v in [('regular','Geist-Regular.otf'),('bold','Geist-SemiBold.otf')]}
def text(s,x,y,size=24,color='#f2f4f8',weight='regular'):
 f=fonts[weight]; gs=f.getGlyphSet(); cmap=f.getBestCmap(); scale=size/f['head'].unitsPerEm; paths=[]; offset=0
 for c in s:
  name=cmap[ord(c)]; pen=SVGPathPen(gs); gs[name].draw(pen)
  paths.append(f'<path transform="translate({offset},0)" d="{pen.getCommands()}"/>'); offset+=f['hmtx'][name][0]
 return f'<g fill="{color}" transform="translate({x},{y}) scale({scale},-{scale})">'+''.join(paths)+'</g>'
def rect(x,y,w,h,fill='#262626',stroke='none'):
 return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="20" fill="{fill}" stroke="{stroke}"/>'
def save(name,h,title,desc,parts):
 svg=f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 {h}" role="img" aria-labelledby="title desc"><title id="title">{escape(title)}</title><desc id="desc">{escape(desc)}</desc>'+rect(0,0,1200,h,'#161616')+''.join(parts)+'</svg>'
 (root/'docs/assets'/name).write_text(svg)
save('readme-before-after.svg',540,'Same project. A better starting point.','Illustrative scenario: without saved memory a new agent may repeat a failed fix. With Vexen it can check the earlier failure and its test result before deciding what to try.',[
 text('A new chat. The same old mistake?',48,66,38,weight='bold'),text('Keep the lesson when you switch sessions or agents.',48,108,24,'#a6c8ff'),
 rect(32,144,556,350),rect(612,144,556,350,stroke='#78a9ff'),
 text('WITHOUT VEXEN',60,189,18,'#a6c8ff','bold'),text('Start from scratch.',60,235,32,weight='bold'),
 text('MONDAY · CLAUDE',60,284,17,'#a6c8ff'),text('Tries a fix. The test fails.',60,319,25),
 text('FRIDAY · CODEX',60,374,17,'#a6c8ff'),text('A fresh chat may try it again.',60,409,25),text('You explain the same lesson again.',60,458,21,'#a6c8ff'),
 text('WITH VEXEN',640,189,18,'#78a9ff','bold'),text('Pick up the lesson.',640,235,32,weight='bold'),
 text('MONDAY · CLAUDE',640,284,17,'#a6c8ff'),text('Saves the failure and test result.',640,319,25),
 text('FRIDAY · CODEX',640,374,17,'#a6c8ff'),text('Checks what failed before editing.',640,409,25),text('Then checks if it still fits the code.',640,458,21,'#a6c8ff'),
 text('ILLUSTRATIVE SCENARIO · Notes must be saved; Vexen does not record your chats.',48,522,16,'#a6c8ff')])
save('readme-benefits.svg',300,'Keep what you learned. Keep control.','Save failed attempts and decisions. Check the file and test result behind a reminder. Keep memory locally with no account or telemetry.',[
 rect(24,24,368,252),rect(416,24,368,252),rect(808,24,368,252),
 *[p for x,num,heading,lines in [(48,'01','Keep the lesson.',['Failed fixes. Key decisions.','Ready for the next session.']),(440,'02','See the proof.',['Find the file and test result','behind a saved reminder.']),(832,'03','Keep it local.',['On your machine. No account.','No cloud. No telemetry.'])] for p in [text(num,x,70,20,'#78a9ff','bold'),text(heading,x,124,30,weight='bold'),text(lines[0],x,182,22,'#a6c8ff'),text(lines[1],x,218,22,'#a6c8ff')]]] )
save('readme-evidence.svg',340,'Does remembering help? A small internal coding pilot.','On five synthetic tasks with hidden requirements, relevant pre-recorded Vexen memory yielded 11 successes in 20 runs, compared with 2 in 20 without memory. One model, explicit tool-use instructions. Not a general coding performance claim.',[
 text('Does remembering past mistakes help?',48,64,36,weight='bold'),text('A small internal coding experiment',48,104,23,'#a6c8ff'),
 rect(32,132,556,136),rect(612,132,556,136,stroke='#78a9ff'),text('2 / 20',60,206,54,weight='bold'),text('without memory',265,202,25,'#a6c8ff'),text('11 / 20',640,206,54,'#78a9ff','bold'),text('with relevant memory',870,202,24,'#a6c8ff'),
 text('Five synthetic tasks · One model · Pre-recorded evidence · Explicit instructions to check memory',48,303,19,'#a6c8ff'),text('These results do not establish general coding improvement.',48,329,18,'#a6c8ff')])


def research_pair(name, cards):
 parts=[]
 for x, c in zip((24,612), cards):
  label,headline,subtitle,lines,limits=c
  parts += [rect(x,24,564,358),text(label,x+28,66,17,'#78a9ff','bold'),text(headline,x+28,122,38,weight='bold'),text(subtitle,x+28,162,23,'#a6c8ff')]
  parts += [text(line,x+28,218+i*32,23) for i,line in enumerate(lines)]
  parts += [text(line,x+28,322+i*26,17,'#a6c8ff') for i,line in enumerate(limits)]
 desc=' '.join(' '.join([a,b,c,*d,*e]) for a,b,c,d,e in cards)
 save(name,406,'Evidence beyond the headline',desc,parts)

research_pair('readme-relevance.svg',[
 ('RELEVANCE CHECK · MEMORY PILOT','20 / 20 stale runs','returned no relevant evidence',
  ['After the file changed, the old note','stayed out of the answer.'],
  ['Five synthetic tasks, four runs each.','Measures filtering, not task success.']),
 ('CONTROL TASKS · MEMORY PILOT','8 / 8 solved, both ways','with and without Vexen',
  ['Unrelated notes stayed quiet.','Both groups solved all control runs.'],
  ['Two synthetic tasks, four runs each.','Small sample; no general safety claim.'])])
research_pair('readme-open-tests.svg',[
 ('CONNECTION CHECK · CLIENT STUDY','13 additional clients','completed a real Vexen read call',
  ['Installed clients requested a note','and received the expected record.'],
  ['Linux, Vexen 0.2.5; mostly scripted drivers.','Checks connections, not coding improvement.']),
 ('PUBLISHED FAILURE · FILE HINT STUDY','10 repos. A failed gate.','1,874 held-out file queries',
  ['Our file-hint method lost to simple','history counts in every repository.'],
  ['File hints remain research-only.','This study did not measure coding success.'])])
