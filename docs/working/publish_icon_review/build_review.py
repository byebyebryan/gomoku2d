"""Build the action-icon comparison directly from the source SVGs."""
from pathlib import Path
import hashlib
import json
import re
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PROPOSALS = {"sign_in", "sign_out", "trash", "sync"}
NAMES = ["sign_in", "sign_out", "trash", "sync", "publish", "share",
         "save", "profile", "reset", "copy", "circle", "home"]
NOTES = {
    "sign_in": "New draft · fixed C frame / inward arrow",
    "sign_out": "New draft · fixed C frame / outward arrow",
    "trash": "New draft · 2px lid, body and bin details",
    "sync": "New draft · opposing arrows / return tails",
    "publish": "Selected · globe · 2px meridians / equator",
    "share": "Current addition · 2px tray / stepped arrow",
    "save": "Current addition · 2px frame / floppy disk",
    "profile": "Original set · 2px frame / filled details",
    "reset": "Original set · single curved reset arrow",
    "copy": "Current addition · overlapping 2px frames",
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
    return raw, pixels, {"sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                         "margins_ltrb": margins, "filled_pixels": len(pixels)}


cards = []
receipts = {}
masks = {}
for name in NAMES:
    path = HERE / f"{name}.svg" if name in PROPOSALS | {"publish"} else ROOT / f"gomoku-web/assets/icons/{name}.svg"
    svg, pixels, receipt = inspect(path)
    masks[name] = pixels
    if name in PROPOSALS | {"publish"}:
        assert receipt["margins_ltrb"] == [2, 2, 2, 2], name
    if name == "publish":
        assert pixels == {(23 - x, y) for x, y in pixels}
        assert pixels == {(x, 23 - y) for x, y in pixels}
        receipt.update(horizontal_symmetry=True, vertical_symmetry=True)
    if name in {"sign_in", "sign_out"}:
        assert pixels == {(x, 23 - y) for x, y in pixels}
        receipt.update(vertical_symmetry=True)
    if name == "trash":
        assert pixels == {(23 - x, y) for x, y in pixels}
        receipt.update(horizontal_symmetry=True)
    if name == "sync":
        assert pixels == {(23 - x, 23 - y) for x, y in pixels}
        receipt.update(rotation_180_symmetry=True)
    receipts[name] = receipt
    svg = re.sub(r"<title>.*?</title>\s*", "", svg, flags=re.S).replace('<svg ', '<svg aria-hidden="true" ')
    label = name.replace("_", " ").title()
    href = f"{name}.svg" if name in PROPOSALS | {"publish"} else f"/assets/icons/{name}.svg"
    margins = " / ".join(map(str, receipt["margins_ltrb"]))
    state = "candidate" if name in PROPOSALS else "accepted" if name == "publish" else ""
    cards.append(f'''<article class="card {state}" id="{name}" aria-labelledby="{name}-title">
      <h2 id="{name}-title">{label}</h2><p class="note">{NOTES[name]}</p>
      <div class="sizes"><div><span>1×</span><div class="native">{svg}</div></div><div><span>2×</span><div class="double">{svg}</div></div></div>
      <div class="grid" role="img" aria-label="{label}, 24 by 24 pixel grid">{svg}</div>
      <p class="metrics">Margins {margins} · {len(pixels)} filled pixels</p>
      <a class="source" href="{href}">Source SVG</a>
    </article>''')

# Both account icons retain exactly the same frame. Sign out is a literal
# quarter-turn of Share; Sign in reverses only its arrow inside the same bounds.
tray = {p for p in masks["share"] if p[0] in {2, 3, 20, 21} or p[1] >= 20}
frame = {(23 - y, x) for x, y in tray}
arrow = {(23 - y, x) for x, y in masks["share"] - tray}
assert masks["sign_out"] == frame | arrow
assert masks["sign_in"] == frame | {(29 - x, y) for x, y in arrow}
receipts["sign_in"]["fixed_frame_shared_with"] = "sign_out"
receipts["sign_out"]["derived_from"] = "share, rotated 90 degrees clockwise"
html = '''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Action icons — pixel comparison</title><style>
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
.accepted { border-color:#5f8b82; }
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
.source { font-size:16px; margin-top:4px; }
footer { margin-top:20px; }
@media(max-width:940px) { .comparison { grid-template-columns:repeat(2,minmax(0,1fr)); } }
@media(max-width:620px) { body { padding:16px; } .comparison { grid-template-columns:minmax(0,1fr); } h1 { font-size:32px; } }
</style></head><body><main>
<nav aria-label="Icon reviews"><a href="/icon-review/">Existing icon comparison</a><a href="#sign_in">Sign in / out</a><a href="#trash">Trash</a><a href="#sync">Sync</a><a href="new-icons.png">New icon image</a><a href="new-icons-light.png">Light image</a><a href="comparison.png">Full comparison image</a></nav>
<h1>Action icons · four new drafts</h1>
<p>24 × 24 source pixels · 10× pixel grid · two-pixel outer margins</p>
<p>Gold lines mark the centre axes. The 1× and 2× samples show the actual SVGs without grid overlays.</p>
<p>Gold cards are the new proposals. Teal marks the selected Publish globe; plain cards are existing assets.</p>
<div class="toolbar"><button id="grid-toggle" type="button" aria-pressed="true">Grid on</button><button id="theme-toggle" type="button" aria-pressed="false">Light surface</button></div>
<section class="comparison" aria-label="New action icons compared with current icons">''' + "".join(cards) + '''</section>
<footer><p>Icon review only. The app controls have not changed.</p><p>Geometry uses integer rectangle runs, transparent backgrounds and currentColor. Existing icons are copied directly from their source SVGs.</p></footer>
</main><script>
document.querySelector('#grid-toggle').addEventListener('click',event=>{ const hidden=document.body.classList.toggle('no-grid'); event.currentTarget.setAttribute('aria-pressed',String(!hidden)); event.currentTarget.textContent=hidden?'Grid off':'Grid on'; });
document.querySelector('#theme-toggle').addEventListener('click',event=>{ const light=document.body.classList.toggle('light'); event.currentTarget.setAttribute('aria-pressed',String(light)); event.currentTarget.textContent=light?'Dark surface':'Light surface'; });
</script></body></html>'''
(HERE / "index.html").write_text(html)
(HERE / "geometry.json").write_text(json.dumps({"scope":"Draft source geometry, not app integration", "icons":receipts}, indent=2)+"\n")
print("Built comparison for " + ", ".join(NAMES))
