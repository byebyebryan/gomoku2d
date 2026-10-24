#!/usr/bin/env python3
"""Prepare an isolated static export using the current app's actual assets."""
from pathlib import Path
import argparse
import json
import re
import shutil

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
parser = argparse.ArgumentParser()
parser.add_argument("--output", type=Path, default=ROOT / "gomoku-web/test-results/v062-design/export")
args = parser.parse_args()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
for name in ("index.html", "study.css", "study.js", "prototype.html", "prototype.css", "prototype.js"):
    shutil.copy2(HERE / name, output / name)
assets = output / "assets"
(assets / "fonts").mkdir(parents=True, exist_ok=True)
fonts = ROOT / "gomoku-web/assets/fonts"
for name in ("PixelOperator.ttf", "PixelOperator-Bold.ttf", "PixelOperator8-Bold.ttf", "LICENSE.txt"):
    shutil.copy2(fonts / name, assets / "fonts" / name)
global_css = (ROOT / "gomoku-web/src/app/global.css").read_text()
(assets / "global.css").write_text(global_css.replace("../../assets/fonts/", "fonts/"))
icon_names = ("play", "profile", "home", "copy", "help", "replay", "prev", "next", "back", "reset", "save", "publish", "minus", "bot", "human", "sign_out")
icons = {}
for name in icon_names:
    raw = (ROOT / "gomoku-web/assets/icons" / (name + ".svg")).read_text()
    icons[name] = re.sub(r"<title>.*?</title>\s*", "", raw, flags=re.S)
(assets / "icons.json").write_text(json.dumps(icons))
references = {
    "reference-replay.png": ROOT / "docs/assets/readme-replay-analysis.png",
    "reference-owner.png": ROOT / "docs/working/v0_6_1_evidence/sharing-chromium-profile-manager-desktop.png",
    "reference-public.png": ROOT / "docs/working/v0_6_1_evidence/sharing-chromium-public-profile.png",
    "reference-settings.png": ROOT / "docs/archive/reconciliation_2026_10/assets/reconciliation-settings-2026-10-15.png",
}
for name, source in references.items():
    shutil.copy2(source, assets / name)
print(json.dumps({"export": str(output), "files": len(list(output.rglob("*")))}))
