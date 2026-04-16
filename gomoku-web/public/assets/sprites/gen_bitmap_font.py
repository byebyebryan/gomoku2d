#!/usr/bin/env python3
"""
Generate a Phaser-compatible AngelCode bitmap font (.fnt + .png)
from a TTF file using FreeType monochrome rendering (zero AA).

Glyphs are placed at their baseline-relative Y position in each atlas cell
so the atlas visually reads as a properly aligned text grid.
lineHeight is set to (ascender - descender) with no extra line gap, so
Phaser's fontSize=FONT_SIZE renders at exactly 1:1 with no sub-pixel scaling.
"""

import freetype
from PIL import Image
import math
import xml.etree.ElementTree as ET

FONT_FILE       = "PixelOperator8-Bold.ttf"
FONT_SIZE       = 16          # px — native design size
CHARS           = list(range(32, 127))  # printable ASCII
PADDING         = 1           # px gap between cells in atlas
LETTER_SPACING  = 1           # extra px added to every xadvance
MONOSPACE       = False       # fix all xadvance to the widest glyph
OUT_PNG         = "minecraft.png"
OUT_FNT         = "minecraft.fnt"

face = freetype.Face(FONT_FILE)
face.set_pixel_sizes(0, FONT_SIZE)

metrics     = face.size
ascender    = metrics.ascender  >> 6   # px above baseline
descender   = -(metrics.descender >> 6)  # px below baseline (positive)
line_height = ascender + descender      # exact content height, no line gap
base        = ascender                  # pixels from line top to baseline

# ── render every glyph in 1-bit mode ──────────────────────────────────────────
glyphs = {}

for c in CHARS:
    face.load_char(c, freetype.FT_LOAD_RENDER | freetype.FT_LOAD_TARGET_MONO)
    g  = face.glyph
    bm = g.bitmap

    rows  = bm.rows
    cols  = bm.width
    pitch = bm.pitch

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

# Cells are tall enough for the full line (ascender + descender)
cell_w = max((d["w"] for d in glyphs.values()), default=0) + PADDING
cell_h = line_height + PADDING

atlas_w = cols_n * cell_w + PADDING
atlas_h = rows_n * cell_h + PADDING

atlas = Image.new("RGBA", (atlas_w, atlas_h), (0, 0, 0, 0))

mono_advance = max(d["advance"] for d in glyphs.values()) if MONOSPACE else None

char_data = []

for i, c in enumerate(sorted(glyphs.keys())):
    d     = glyphs[c]
    col_i = i % cols_n
    row_i = i // cols_n

    # Top-left of this cell in the atlas
    cell_x = PADDING + col_i * cell_w
    cell_y = PADDING + row_i * cell_h

    # Center glyph horizontally in cell (cosmetic only — rendering uses xoffset/yoffset)
    gx = cell_x + (cell_w - PADDING - d["w"]) // 2
    gy = cell_y  # pixels sit at top of cell; yoffset in .fnt handles vertical placement

    if d["w"] > 0 and d["h"] > 0:
        glyph_img = Image.new("RGBA", (d["w"], d["h"]), (0, 0, 0, 0))
        glyph_img.putdata([(255, 255, 255, p) for p in d["pixels"]])
        atlas.paste(glyph_img, (gx, gy))

    xadv = (mono_advance if MONOSPACE else d["advance"]) + LETTER_SPACING

    # xoffset/yoffset tell Phaser where to draw the pixels relative to the pen
    xoff = d["bearingX"]
    yoff = base - d["bearingY"]  # how far below line top this glyph starts

    char_data.append((c, gx, gy, d["w"], d["h"], xoff, yoff, xadv))

atlas.save(OUT_PNG)
print(f"Saved {OUT_PNG}  ({atlas_w}×{atlas_h})  lineHeight={line_height}  base={base}")

# ── write AngelCode XML .fnt ───────────────────────────────────────────────────
root = ET.Element("font")

info = ET.SubElement(root, "info")
info.set("face", "minecraft")
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
page.set("id", "0"); page.set("file", OUT_PNG)

chars_el = ET.SubElement(root, "chars")
chars_el.set("count", str(len(char_data)))

for (c, x, y, w, h, xoff, yoff, xadv) in char_data:
    ch = ET.SubElement(chars_el, "char")
    ch.set("id",       str(c))
    ch.set("x",        str(x))
    ch.set("y",        str(y))
    ch.set("width",    str(w))
    ch.set("height",   str(h))
    ch.set("xoffset",  str(xoff))
    ch.set("yoffset",  str(yoff))
    ch.set("xadvance", str(xadv))
    ch.set("page",     "0")
    ch.set("chnl",     "15")

tree = ET.ElementTree(root)
ET.indent(tree, space="  ")
tree.write(OUT_FNT, encoding="unicode", xml_declaration=False)
print(f"Saved {OUT_FNT}")
