# Gomoku2D Icon Pack

Pixel-style SVG icon pack for the outer DOM shell.

This folder is built around [icon_sheet.png](./icon_sheet.png), which is now the
canonical source sheet. The exported SVGs are direct `24x24` cell conversions of
that sheet.

## Files

- `icon_sheet.png`: current source sheet
- `manifest.json`: inferred icon naming, grid positions, categories, and notes
- `preview.html`: human preview of the source sheet and exported SVG pack
- `*.svg`: per-icon runtime assets

## Grid Map

The source sheet is a `5 x 6` grid of exact `24 x 24` cells.

| Row | Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
| --- | --- | --- | --- | --- | --- |
| 1 | `home` | `play` | `profile` | `replay` | `win` |
| 2 | `plus` | `minus` | `reset` | `undo` | `last` |
| 3 | `first` | `fast_forward` | `fast_rewind` | `next` | `prev` |
| 4 | `double_next` | `double_prev` | `forward` | `back` | `settings` |
| 5 | `bot` | `human` | `grid` | `filled_circle` | `circle` |
| 6 | `forbidden` | `info` | `close` | `confirm` | `help` |

## Inference Notes

These names are inferred from the current sheet. A few are still intentionally
provisional and can be trimmed or renamed later.

- `last` / `first` are the bar-plus-triangle jump controls.
- `next` / `prev` are the simple chevron step controls.
- `fast_forward` / `fast_rewind` are the double-triangle transport family.
- `double_next` / `double_prev` are alternate transport icons. Their final app
  meaning is still open.
- `forward` / `back` are arrow-family navigation icons, distinct from replay
  transport.
- `grid`, `filled_circle`, and `circle` intentionally stay generic. That keeps
  the pack flexible if the same shapes get reused outside strict gameplay
  stone semantics.
- `plus` / `minus` are kept generic on purpose instead of overfitting them to a
  specific screen flow too early.

## Runtime Guidance

The SVGs are authored as:

- `24 x 24`
- transparent background
- monochrome shapes using `currentColor`
- pixel-snapped rect runs

For actual UI use, prefer inline SVGs or imported SVG components so the icons
inherit text color cleanly.

If you use plain `<img src=\"...svg\">`, treat that as a static asset path, not
as a themed `currentColor` pipeline.
