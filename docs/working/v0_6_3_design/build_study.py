"""Export the design study from canonical assets and dated report evidence."""
from pathlib import Path
import hashlib
import json
import re
import shutil
import struct
import subprocess
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / "gomoku-web/test-results/v063-design/export"
OUT.mkdir(parents=True, exist_ok=True)
inputs = {}


def read(relative):
    path = ROOT / relative
    raw = path.read_bytes()
    inputs[relative] = hashlib.sha256(raw).hexdigest()
    return raw


def copy(relative, destination):
    raw = read(relative)
    path = OUT / destination
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw)
    return hashlib.sha256(raw).hexdigest()


for name in ["index.html", "study.css", "study.js"]:
    copy(f"docs/working/v0_6_3_design/{name}", name)

css = read("gomoku-web/src/app/global.css").decode()
assert css.count("../../assets/fonts/") == 3
(OUT / "app-controls.css").write_text(css.replace("../../assets/fonts/", "fonts/"))
for name in ["PixelOperator.ttf", "PixelOperator-Bold.ttf", "PixelOperator8-Bold.ttf", "LICENSE.txt"]:
    copy("gomoku-web/assets/fonts/" + name, "fonts/" + name)

manifest = json.loads(read("gomoku-web/assets/asset_manifest.json"))
icon_manifest = json.loads(read("gomoku-web/assets/icons/manifest.json"))
icons = []
for icon in icon_manifest["icons"]:
    name = icon["name"]
    raw = read(f"gomoku-web/assets/icons/{name}.svg")
    tree = ET.fromstring(raw)
    assert tree.attrib["viewBox"] == "0 0 24 24"
    assert all(n.tag.split("}")[-1] in {"svg", "g", "rect", "title"} for n in tree.iter())
    pixels = set()
    for rect in tree.findall(".//{http://www.w3.org/2000/svg}rect"):
        x, y, w, h = [int(rect.attrib[k]) for k in ["x", "y", "width", "height"]]
        assert w > 0 and h > 0 and 0 <= x < x+w <= 24 and 0 <= y < y+h <= 24
        pixels.update((a, b) for a in range(x, x+w) for b in range(y, y+h))
    bounds = [min(x for x,y in pixels), min(y for x,y in pixels), 23-max(x for x,y in pixels), 23-max(y for x,y in pixels)]
    copy(f"gomoku-web/assets/icons/{name}.svg", f"icons/{name}.svg")
    icons.append({**icon, "svg": raw.decode(), "margins": bounds, "pixels": len(pixels),
                  "mask": sorted(y*24+x for x,y in pixels), "sha256": hashlib.sha256(raw).hexdigest()})

constants = read("gomoku-web/src/board/constants.ts").decode()
runtime_anims = {key: (int(start), int(end), int(fps)) for start,end,fps,key in re.findall(
    r'start:\s*(\d+),\s*end:\s*(\d+),\s*frameRate:\s*(\d+),\s*key:\s*"([^"]+)"', constants)}
for anim in manifest["sprites"]["animations"]:
    assert runtime_anims[anim["name"]] == (anim["start"], anim["end"], anim["fps"])
for sheet in manifest["sprites"]["sheets"]:
    raw = read("gomoku-web/assets/" + sheet["file"])
    assert struct.unpack(">II", raw[16:24]) == (sheet["cols"]*16, sheet["rows"]*16)
    copy("gomoku-web/assets/" + sheet["file"], sheet["file"])
assert "STONE_BLACK:  P.DARK" in constants and "DARK:   0x404040" in constants

bot = json.loads(read("reports/lab/bot-report.json"))
analysis = json.loads(read("reports/lab/analysis-report.json"))
assert not bot["provenance"]["git_dirty"] and not analysis["provenance"]["git_dirty"]
entry = next(e for s in analysis["sections"] for e in s["entries"]
             if e["path"] == "match_0095__search-d1__vs__search-d3_pattern-eval")
assert entry["match_report"]["move_count"] == 28
entry = {**entry, "proof_details": {"proof_frames": [f for f in entry["proof_details"]["proof_frames"] if f["ply"] in (15,25,28)]}}

mobile = json.loads(read("docs/working/v0_6_2_mobile/evidence.json"))
captures = {}
for name in ["profile-before.png", "profile-after.png", "public-before.png", "public-after.png"]:
    raw = read("docs/working/v0_6_2_mobile/"+name)
    expected = next(c for c in mobile["captures"] if c["path"] == name)
    assert hashlib.sha256(raw).hexdigest() == expected["sha256"]
    copy("docs/working/v0_6_2_mobile/"+name, "review/"+name)
    captures[name] = {**expected, "dimensions": struct.unpack(">II", raw[16:24])}
release = json.loads(read("docs/archive/v0_6_2_release_evidence.json"))
head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
public_source = subprocess.check_output(["git", "rev-parse", "origin/main"], cwd=ROOT, text=True).strip()
# Source links must resolve before this local-only study is pushed.
for linked in ["reports/lab/bot-report.json", "reports/lab/analysis-report.json",
               "docs/working/v0_6_2_mobile/README.md", "docs/archive/v0_6_2_release.md"]:
    assert subprocess.check_output(["git", "show", f"{public_source}:{linked}"], cwd=ROOT) == read(linked)
tokens = dict(re.findall(r"(--[\w-]+):\s*([^;]+);", re.search(r":root\s*\{([^}]+)\}", css)[1]))
data = {
    "source": head, "publicSource": public_source, "applicationTree": subprocess.check_output(["git","rev-parse","HEAD:gomoku-web/src"], cwd=ROOT,text=True).strip(),
    "kind": "Standalone design study; no application integration or visual acceptance claimed",
    "icons": icons, "sprites": manifest["sprites"], "tokens": tokens,
    "lab": {"run": bot["run"], "provenance": bot["provenance"], "standings": bot["standings"],
            "analysisProvenance": analysis["provenance"], "entry": entry},
    "review": {"baseline": mobile["baseline"], "runtime": mobile["runtime_commit"], "captures": captures,
               "date": "October 23–24, 2026", "followUp": "9c4c3bb", "human": "Initial mobile feedback broadly positive; physical-phone acceptance remains unperformed."},
    "release": {"version": "0.6.2", "ci": release["ci"], "production": release["production_browser"]},
}
(OUT / "data.js").write_text("window.STUDY_DATA = " + json.dumps(data, separators=(",", ":")) + ";\n")
output_files = {p.relative_to(OUT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in sorted(OUT.rglob("*")) if p.is_file() and p.name != "source-manifest.json"}
receipt = {"source": head, "public_source": public_source, "application_tree": data["applicationTree"], "kind": data["kind"],
           "inputs": inputs, "output_files": output_files,
           "notes": ["Canonical CSS with only font URLs rebased", "Source SVG bytes retained exactly",
                     "Lab values and run provenance copied from published JSON, not recomputed tournaments",
                     "Dated mobile captures retained byte-for-byte; not a current release baseline"]}
(OUT / "source-manifest.json").write_text(json.dumps(receipt, indent=2)+"\n")
(HERE / "source_manifest.json").write_text(json.dumps(receipt, indent=2)+"\n")
print(json.dumps({"export": str(OUT), "icons": len(icons), "lab_rows": len(bot["standings"]), "files": len(output_files)+1}))
