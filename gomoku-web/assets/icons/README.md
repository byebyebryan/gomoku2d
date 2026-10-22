# Gomoku2D Icon Pack

Pixel-style SVG icon pack for the outer DOM shell.

This folder is built around [icon_sheet.png](./icon_sheet.png), which is now the
canonical source sheet for the original 32 icons. Those SVGs are direct `24x24`
cell conversions of that sheet. The `share`, `copy`, and `save` additions are
standalone SVGs on the same grid; they are not cells in the original sheet.

## Files

- `icon_sheet.png`: current source sheet
- `manifest.json`: inferred icon naming, grid positions, categories, notes,
  and source data for the published `/visuals/` Visual Design reference
- `*.svg`: per-icon runtime assets

## Grid Map

The source sheet is a `8 x 4` grid of exact `24 x 24` cells.

| Row | Col 1 | Col 2 | Col 3 | Col 4 | Col 5 | Col 6 | Col 7 | Col 8 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `home` | `profile` | `replay` | `win` | `plus` | `minus` | `reset` | `undo` |
| 2 | `play` | `play_reverse` | `pause` | `last` | `first` | `fast_forward` | `fast_rewind` | `next` |
| 3 | `prev` | `double_next` | `double_prev` | `forward` | `back` | `settings` | `bot` | `human` |
| 4 | `grid` | `filled_circle` | `circle` | `forbidden` | `info` | `close` | `confirm` | `help` |

## Inference Notes

These names are inferred from the current sheet. A few are still intentionally
provisional and can be trimmed or renamed later.

- `pause` is newly available for autoplay or media-style control states.
- `play_reverse` is kept provisional until we decide whether it deserves a
  real product meaning or should be dropped later.
- `last` / `first` are the bar-plus-triangle jump controls.
- `next` / `prev` are the simple single-step controls.
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

The sharing additions use the same integer rectangle runs, two-pixel outer
gutter, transparent background and monochrome fill as the original pack.
`share` is horizontally symmetric, including its arrow and tray. `save` is a
floppy disk with a clipped upper-right corner and separate shutter/label areas.

The SVGs are authored as:

- `24 x 24`
- transparent background
- monochrome shapes using `currentColor`
- pixel-snapped rect runs

For actual UI use, prefer inline SVGs or imported SVG components so the icons
inherit text color cleanly.

If you use plain `<img src="...svg">`, treat that as a static asset path, not
as a themed `currentColor` pipeline.
