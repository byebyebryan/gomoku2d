from pathlib import Path
import shutil

source = Path(__file__).resolve().parent
root = source.parents[2]
output = root / "gomoku-web/test-results/v062-mobile/export"
output.mkdir(parents=True, exist_ok=True)
shutil.copy2(source / "gallery.html", output / "index.html")
for path in source.glob("*.png"):
    shutil.copy2(path, output / path.name)
(output / "fonts").mkdir(exist_ok=True)
for name in ["PixelOperator.ttf", "PixelOperator-Bold.ttf", "LICENSE.txt"]:
    shutil.copy2(root / "gomoku-web/assets/fonts" / name, output / "fonts" / name)
print(output)
