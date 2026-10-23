"""Build the draft publish-icon comparison directly from the source SVGs."""
from pathlib import Path
import hashlib
import json
import re
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAMES = ["publish", "share", "save", "profile", "circle", "home"]
NOTES = {
    "publish": "New draft · globe · 2px meridians / equator",
    "share": "Current addition · 2px tray / stepped arrow",
    "save": "Current addition · 2px frame / floppy disk",
    "profile": "Original set · 2px frame / filled details",
    "circle": "Original set · stepped circular contour",
    "home": "Original set · filled silhouette",
}


def inspect(path):
    raw = path.read_text()
    root = ET.fromstring(raw)
    assert root.attrib["viewBox"] == "0 0 24 24"
    pixels = set()
    for rect in root.findall(".//{http://www.w3.org/2000/svg}rect"):
        x, y, w, h = [int(rect.attrib[k]) for k in ["x", "y", "width", "height"]]
        assert w > 0 and h > 0 and 0 <= x < x + w <= 24 and 0 <= y < y + h <= 24
        pixels.update((a, b) for a in range(x, x + w) for b in range(y, y + h))
    assert all(node.tag.rsplit("}", 1)[-1] in {"svg", "g", "title", "rect"} for node in root.iter())
    margins = [min(x for x, y in pixels), min(y for x, y in pixels),
               23 - max(x for x, y in pixels), 23 - max(y for x, y in pixels)]
    assert margins == [2, 2, 2, 2], (path, margins)
    return raw, pixels, {"sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                         "margins_ltrb": margins, "filled_pixels": len(pixels)}


cards = []
receipts = {}
for name in NAMES:
    path = HERE / "publish.svg" if name == "publish" else ROOT / f"gomoku-web/assets/icons/{name}.svg"
    svg, pixels, receipt = inspect(path)
    if name == "publish":
        assert pixels == {(23 - x, y) for x, y in pixels}
        assert pixels == {(x, 23 - y) for x, y in pixels}
        receipt.update(horizontal_symmetry=True, vertical_symmetry=True)
    receipts[name] = receipt
    svg = re.sub(r"<title>.*?</title>\s*", "", svg, flags=re.S).replace('<svg ', '<svg aria-hidden="true" ')
    label = name.title()
    cards.append(f'''<article class="card {'candidate' if name == 'publish' else ''}" aria-labelledby="{name}-title">
      <h2 id="{name}-title">{label}</h2><p class="note">{NOTES[name]}</p>
      <div class="sizes"><div><span>1×</span><div class="native">{svg}</div></div><div><span>2×</span><div class="double">{svg}</div></div></div>
      <div class="grid" role="img" aria-label="{label}, 24 by 24 pixel grid">{svg}</div>
      <p class="metrics">Margins 2 / 2 / 2 / 2 · {len(pixels)} filled pixels</p>
    </article>''')
html = '''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Publish icon — pixel comparison</title><style>
@font-face { font-family: Pixel; src: url('/assets/fonts/PixelOperator.ttf'); }
@font-face { font-family: PixelBold; src: url('/assets/fonts/PixelOperator-Bold.ttf'); }
* { box-sizing: border-box; }
:root { color-scheme: dark; --bg:#1e1e1e; --panel:#232323; --text:#f5f5f5; --muted:#b5b5ae; --border:#575756; --accent:#fccb57; }
body { margin:0; padding:28px; background:var(--bg); color:var(--text); font:20px Pixel,monospace; }
body.light { color-scheme:light; --bg:#f5f5f5; --panel:#fff; --text:#1e1e1e; --muted:#555; --border:#aaa; --accent:#755300; }
main { max-width:1024px; margin:auto; }
nav { display:flex; flex-wrap:wrap; gap:12px 24px; align-items:center; }
a { color:var(--accent); }
h1,h2 { font-family:PixelBold,monospace; font-weight:normal; }
h1 { margin:24px 0 8px; font-size:38px; }
p { margin:8px 0; color:var(--muted); line-height:1.3; }
.toolbar { display:flex; flex-wrap:wrap; gap:12px; margin:20px 0; }
button { font:inherit; padding:8px 14px; background:var(--panel); color:var(--text); border:2px solid var(--border); cursor:pointer; }
button:focus-visible,a:focus-visible { outline:2px solid var(--accent); outline-offset:3px; }
.comparison { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:16px; }
.card { display:grid; justify-items:center; align-content:start; min-width:0; padding:16px; background:var(--panel); border:1px solid var(--border); }
.candidate { border-color:var(--accent); }
h2 { font-size:28px; margin:0; justify-self:start; }
.note { font-size:16px; min-height:38px; justify-self:start; margin:4px 0 8px; }
.sizes { display:flex; align-items:flex-end; gap:36px; margin:0 0 16px; height:75px; }
.sizes > div { display:grid; justify-items:center; gap:5px; }
.sizes span { color:var(--muted); font-size:16px; }
svg { display:block; }
.native svg { width:24px; height:24px; }
.double svg { width:48px; height:48px; }
.grid { position:relative; width:240px; height:240px; background:var(--bg); outline:1px solid var(--border); }
.grid > svg { width:240px; height:240px; }
.grid::after { content:""; position:absolute; inset:0; pointer-events:none; background-image:linear-gradient(to right,#8a8a8a70 1px,transparent 1px),linear-gradient(to bottom,#8a8a8a70 1px,transparent 1px); background-size:10px 10px; }
.grid::before { content:""; position:absolute; inset:0; pointer-events:none; z-index:1; background-image:linear-gradient(to right,transparent 120px,#d6aa4d99 120px,#d6aa4d99 121px,transparent 121px),linear-gradient(to bottom,transparent 120px,#d6aa4d99 120px,#d6aa4d99 121px,transparent 121px); }
.no-grid .grid::after,.no-grid .grid::before { display:none; }
.metrics { font-size:16px; margin-top:12px; text-align:center; }
footer { margin-top:20px; }
@media(max-width:940px) { .comparison { grid-template-columns:repeat(2,minmax(0,1fr)); } }
@media(max-width:620px) { body { padding:16px; } .comparison { grid-template-columns:minmax(0,1fr); } h1 { font-size:32px; } }
</style></head><body><main>
<nav aria-label="Icon reviews"><a href="/icon-review/">Existing icon comparison</a><a href="publish.svg">Draft SVG</a><a href="comparison.png">Comparison image</a></nav>
<h1>Publish replay · globe draft</h1>
<p>24 × 24 source pixels · 10× pixel grid · two-pixel outer margins</p>
<p>Gold lines mark the centre axes. The 1× and 2× samples show the actual SVGs without grid overlays.</p>
<div class="toolbar"><button id="grid-toggle" type="button" aria-pressed="true">Grid on</button><button id="theme-toggle" type="button" aria-pressed="false">Light surface</button></div>
<section class="comparison" aria-label="Publish icon compared with current icons">''' + "".join(cards) + '''</section>
<footer><p>Draft for visual review. The app's Publish replay button has not changed.</p><p>Geometry uses integer rectangle runs, transparent backgrounds and currentColor. Existing icons are copied directly from their source SVGs.</p></footer>
</main><script>
document.querySelector('#grid-toggle').addEventListener('click',event=>{ const hidden=document.body.classList.toggle('no-grid'); event.currentTarget.setAttribute('aria-pressed',String(!hidden)); event.currentTarget.textContent=hidden?'Grid off':'Grid on'; });
document.querySelector('#theme-toggle').addEventListener('click',event=>{ const light=document.body.classList.toggle('light'); event.currentTarget.setAttribute('aria-pressed',String(light)); event.currentTarget.textContent=light?'Dark surface':'Light surface'; });
</script></body></html>'''
(HERE / "index.html").write_text(html)
(HERE / "geometry.json").write_text(json.dumps({"scope":"Draft source geometry, not app integration", "icons":receipts}, indent=2)+"\n")
print("Built comparison for " + ", ".join(NAMES))
