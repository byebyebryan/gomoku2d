#!/usr/bin/env python3
"""
Generate a Phaser-compatible AngelCode bitmap font (.fnt + .png)
from a TTF file using FreeType monochrome rendering (zero AA).

Output files are named after the input font and written to this directory,
then copied to ../../public/assets/sprites/ for the web build to pick up.

Usage (run from this directory):
    uv run --with freetype-py --with pillow python gen_bitmap_font.py

To switch fonts, change FONT_FILE below and re-run. Output files are named
automatically after the font (e.g. PixelOperator8-Bold.png / .fnt).

After generation, update src/scenes/boot.ts to reference the new filenames:
    this.load.bitmapFont(
        "minecraft",
        "assets/sprites/PixelOperator8-Bold.png",
        "assets/sprites/PixelOperator8-Bold.fnt",
    );

Tuning knobs:
    FONT_SIZE      — must match the font's native pixel grid (16 for PixelOperator)
    LETTER_SPACING — extra px added to every character's advance width
    MONOSPACE      — force all advances to the widest glyph

Notes on lineHeight:
    FreeType reports height = ascender + descender + line_gap. We drop the
    line_gap so lineHeight == FONT_SIZE, which means Phaser's fontSize=16
    renders at exactly 1:1 with no sub-pixel scaling.
"""

import os
import shutil
import freetype
from PIL import Image
import math
import xml.etree.ElementTree as ET

# ── configuration ─────────────────────────────────────────────────────────────
FONT_FILE       = "PixelOperator8-Bold.ttf"
FONT_SIZE       = 16           # px — native design size for PixelOperator family
CHARS           = list(range(32, 127))  # printable ASCII
PADDING         = 1            # px gap between atlas cells
LETTER_SPACING  = 1            # extra px added to every xadvance
MONOSPACE       = False        # set True to fix all advances to widest glyph

# ── derived paths ─────────────────────────────────────────────────────────────
SCRIPT_DIR  = os.path.dirname(os.path.abspath(__file__))
SPRITES_DIR = os.path.normpath(os.path.join(SCRIPT_DIR, "../../public/assets/sprites"))

font_name = os.path.splitext(FONT_FILE)[0]
OUT_PNG   = os.path.join(SCRIPT_DIR, font_name + ".png")
OUT_FNT   = os.path.join(SCRIPT_DIR, font_name + ".fnt")

# ── render glyphs in 1-bit (monochrome) mode ──────────────────────────────────
face = freetype.Face(os.path.join(SCRIPT_DIR, FONT_FILE))
face.set_pixel_sizes(0, FONT_SIZE)

metrics     = face.size
ascender    = metrics.ascender   >> 6
descender   = -(metrics.descender >> 6)
line_height = ascender + descender   # drop line gap so lineHeight == FONT_SIZE
base        = ascender

glyphs = {}
for c in CHARS:
    face.load_char(c, freetype.FT_LOAD_RENDER | freetype.FT_LOAD_TARGET_MONO)
    g  = face.glyph
    bm = g.bitmap
    rows, cols, pitch = bm.rows, bm.width, bm.pitch

    pixels = []
    for r in range(rows):
        for col in range(cols):
            byte = bm.buffer[r * pitch + col // 8]
            bit  = (byte >> (7 - col % 8)) & 1
            pixels.append(255 if bit else 0)

    glyphs[c] = {
        "pixels":   pixels,
        "w":        cols,
        "h":        rows,
        "bearingX": g.bitmap_left,
        "bearingY": g.bitmap_top,
        "advance":  g.advance.x >> 6,
    }

# ── pack into a square-ish atlas ──────────────────────────────────────────────
n      = len(glyphs)
cols_n = math.ceil(math.sqrt(n))
rows_n = math.ceil(n / cols_n)

cell_w = max(d["w"] for d in glyphs.values()) + PADDING
cell_h = line_height + PADDING

atlas_w = cols_n * cell_w + PADDING
atlas_h = rows_n * cell_h + PADDING
atlas   = Image.new("RGBA", (atlas_w, atlas_h), (0, 0, 0, 0))

mono_advance = max(d["advance"] for d in glyphs.values()) if MONOSPACE else None

char_data = []
for i, c in enumerate(sorted(glyphs.keys())):
    d     = glyphs[c]
    col_i = i % cols_n
    row_i = i // cols_n

    cell_x = PADDING + col_i * cell_w
    cell_y = PADDING + row_i * cell_h

    # Center glyph horizontally in cell (cosmetic — rendering uses xoffset/yoffset)
    gx = cell_x + (cell_w - PADDING - d["w"]) // 2
    gy = cell_y

    if d["w"] > 0 and d["h"] > 0:
        glyph_img = Image.new("RGBA", (d["w"], d["h"]), (0, 0, 0, 0))
        glyph_img.putdata([(255, 255, 255, p) for p in d["pixels"]])
        atlas.paste(glyph_img, (gx, gy))

    xadv = (mono_advance if MONOSPACE else d["advance"]) + LETTER_SPACING
    xoff = d["bearingX"]
    yoff = base - d["bearingY"]  # px below line top where this glyph starts

    char_data.append((c, gx, gy, d["w"], d["h"], xoff, yoff, xadv))

atlas.save(OUT_PNG)
print(f"Saved {OUT_PNG}  ({atlas_w}×{atlas_h})  lineHeight={line_height}  base={base}")

# ── write AngelCode XML .fnt ───────────────────────────────────────────────────
root = ET.Element("font")

info = ET.SubElement(root, "info")
info.set("face", font_name)
info.set("size", str(FONT_SIZE))
info.set("bold", "0"); info.set("italic", "0")
info.set("charset", ""); info.set("unicode", "1")
info.set("stretchH", "100"); info.set("smooth", "0")
info.set("aa", "0"); info.set("padding", "0,0,0,0")
info.set("spacing", f"{PADDING},{PADDING}")

common = ET.SubElement(root, "common")
common.set("lineHeight", str(line_height))
common.set("base",       str(base))
common.set("scaleW",     str(atlas_w))
common.set("scaleH",     str(atlas_h))
common.set("pages",      "1")
common.set("packed",     "0")

pages = ET.SubElement(root, "pages")
page  = ET.SubElement(pages, "page")
page.set("id", "0")
page.set("file", os.path.basename(OUT_PNG))

chars_el = ET.SubElement(root, "chars")
chars_el.set("count", str(len(char_data)))

for (c, x, y, w, h, xoff, yoff, xadv) in char_data:
    ch = ET.SubElement(chars_el, "char")
    ch.set("id", str(c)); ch.set("x", str(x)); ch.set("y", str(y))
    ch.set("width", str(w)); ch.set("height", str(h))
    ch.set("xoffset", str(xoff)); ch.set("yoffset", str(yoff))
    ch.set("xadvance", str(xadv)); ch.set("page", "0"); ch.set("chnl", "15")

tree = ET.ElementTree(root)
ET.indent(tree, space="  ")
tree.write(OUT_FNT, encoding="unicode", xml_declaration=False)
print(f"Saved {OUT_FNT}")

# ── copy to sprites/ for the web build ────────────────────────────────────────
for src in (OUT_PNG, OUT_FNT):
    dst = os.path.join(SPRITES_DIR, os.path.basename(src))
    shutil.copy(src, dst)
    print(f"Copied → {dst}")
