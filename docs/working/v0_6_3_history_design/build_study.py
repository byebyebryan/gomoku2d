"""Prepare an allowlisted, standalone UI-history design study. No app mutations."""
from pathlib import Path
import hashlib
import json
import struct
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / "gomoku-web/test-results/v063-history-design/export"
OUT.mkdir(parents=True, exist_ok=True)
inputs = {}

def read(path):
    data = (ROOT / path).read_bytes()
    inputs[path] = hashlib.sha256(data).hexdigest()
    return data

def copy(path, name):
    data = read(path)
    target = OUT / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    return hashlib.sha256(data).hexdigest()

prefix = HERE.relative_to(ROOT).as_posix()
for name in ["index.html", "study.css", "study.js"]:
    copy(f"{prefix}/{name}", name)
css = read("gomoku-web/src/app/global.css").decode()
assert css.count("../../assets/fonts/") == 3
(OUT / "app-controls.css").write_text(css.replace("../../assets/fonts/", "fonts/"))
copy("gomoku-web/src/developer/developer.css", "developer.css")
for name in ["PixelOperator.ttf", "PixelOperator-Bold.ttf", "PixelOperator8-Bold.ttf", "LICENSE.txt"]:
    copy(f"gomoku-web/assets/fonts/{name}", f"fonts/{name}")

assets = {}
pilot_prefix = "docs/working/v0_6_3_retrospective_capture/pilot-2026-10-25"
pilot = json.loads(read(f"{pilot_prefix}/capture.json"))
assert pilot["status"] == "complete"
for capture in pilot["captures"]:
    relative = f'{pilot_prefix}/{capture["file"]}'
    destination = f'captures/{capture["file"]}'
    digest = copy(relative, destination)
    assert digest == capture["sha256"]
    version = capture["source"].lstrip("v")
    screen = capture["screen"].capitalize()
    assets[f'{version}|{screen}|{capture["profile"]}'] = {
        "file": destination, "width": capture["width"], "height": capture["height"],
        "label": f"v{version}", "kind": "retrospective capture", "sha256": digest,
        "sourcePath": relative, "revision": next(s["commit"] for s in pilot["sources"] if s["tag"] == capture["source"]),
        "alt": f'Retrospective v{version} {screen}, {capture["profile"]}; original captured pixels',
    }

extras = [
    ("0.6.1", "Public player", "portrait", "docs/working/v0_6_1_evidence/sharing-chromium-public-profile.png", "0.6.1-era checkpoint", "development capture", None),
    ("0.6.2", "Public player", "portrait", "docs/working/v0_6_3_core_reference/public-player-portrait.png", "v0.6.2", "release reference", "37a122d2bd8e899bcfc581055e685c0327f43cb4"),
    ("0.6.2", "Public player", "desktop", "docs/working/v0_6_3_core_reference/public-player-desktop.png", "v0.6.2", "release reference", "37a122d2bd8e899bcfc581055e685c0327f43cb4"),
    ("0.6.2", "Settings", "portrait", "docs/working/v0_6_3_core_reference/settings-portrait.png", "v0.6.2", "release reference", "37a122d2bd8e899bcfc581055e685c0327f43cb4"),
    ("0.6.2", "Settings", "desktop", "docs/working/v0_6_3_core_reference/settings-desktop.png", "v0.6.2", "release reference", "37a122d2bd8e899bcfc581055e685c0327f43cb4"),
]
for version, screen, viewport, source, label, kind, revision in extras:
    destination = f'captures/{version}-{screen.lower().replace(" ","-")}-{viewport}.png'
    digest = copy(source, destination)
    width, height = struct.unpack(">II", (ROOT / source).read_bytes()[16:24])
    assets[f'{version}|{screen}|{viewport}'] = {
        "file": destination, "width": width, "height": height, "label": label,
        "kind": kind, "revision": revision, "sha256": digest, "sourcePath": source,
        "alt": f'{label} {screen}, {viewport}; synthetic public/local fixture; {kind}',
    }

milestones = json.loads(read(f"{prefix}/milestones.json"))
changelog = read("CHANGELOG.md")
read(f"{prefix}/build_study.py")
public_source = subprocess.check_output(["rtk", "proxy", "git", "rev-parse", "origin/main"], cwd=ROOT, text=True).strip()
assert subprocess.check_output(["rtk", "proxy", "git", "show", f"{public_source}:CHANGELOG.md"], cwd=ROOT) == changelog
changelog_url = f"https://github.com/byebyebryan/gomoku2d/blob/{public_source}/CHANGELOG.md"
head = subprocess.check_output(["rtk", "proxy", "git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
icons = {"play": read("gomoku-web/assets/icons/play.svg").decode()}
data = {"milestones": milestones, "assets": assets, "icons": icons, "changelogUrl": changelog_url, "baseCommit": head}
(OUT / "data.js").write_text("window.HISTORY_STUDY = " + json.dumps(data, ensure_ascii=False, separators=(",", ":")) + ";\n")
outputs = {p.relative_to(OUT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUT.rglob("*")) if p.is_file() and p.name != "source-manifest.json"}
manifest = {"baseCommit": head, "kind": "Standalone design proposal; not app implementation or completed history coverage", "inputs": inputs, "outputFiles": outputs, "captures": assets,
            "notes": ["Original PNG and font bytes retained", "Canonical CSS with only font URLs rebased", "Proposed release roster; source inventory still pending", "0.6.1-era public example is not an exact-tag recapture", "Missing and introduced states use explicit text panels; images never relabelled"]}
(OUT / "source-manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
print(json.dumps({"export": str(OUT), "files": len(outputs) + 1, "captures": len(assets), "milestones": len(milestones)}))
