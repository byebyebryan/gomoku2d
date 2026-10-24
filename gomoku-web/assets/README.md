# Gomoku2D Assets

Source assets for the web game.

These are the authoring/reference assets imported by the app or used to derive
runtime visuals. The web build publishes raw visual-guide files to
`dist/assets/`.

## Published Visual Design

- Published visual design reference after deploy:
  <https://gomoku2d.byebyebryan.com/visuals/>

GitHub renders this Markdown file, but the browsable design reference is part
of the React app. The web build publishes raw files and `asset_manifest.json` to
`dist/assets/`, then serves the app route at `/visuals/`.

The reference has three sections:

- Style: palette, button roles, type roles, and visual principles
- Sprites: board-space spritesheets, static poses, animation loops, and z-order
- Icons: all canonical icons, native 1×/2× samples, integer pixel grid, margins,
  bounded comparison, light/dark surfaces and exact source SVGs

## Folders

- [sprites](./sprites/): runtime board spritesheets and animation inventory
- [icons](./icons/): DOM shell icon source sheet, manifest, and exported SVGs
- [fonts](./fonts/): PixelOperator source fonts and license

The shared developer frame links these references to Lab, curated reviews and
Process. Visuals accepts `view=style|icons|sprites`; icon/animation selections are
URL driven. Palette values are read from runtime CSS; the publisher derives the
exported manifest values from those tokens. SVG/font/PNG source bytes remain
unchanged. Sprite previews use the board's pure RGB multiplication operation and
check animation ranges against runtime constants. See the
[developer-area contract](../../docs/reference/app/developer_area.md).
