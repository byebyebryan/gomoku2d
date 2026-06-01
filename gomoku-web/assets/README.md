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
- Icons: DOM shell icon inventory from the icon manifest

## Folders

- [sprites](./sprites/): runtime board spritesheets and animation inventory
- [icons](./icons/): DOM shell icon source sheet, manifest, and exported SVGs
- [fonts](./fonts/): PixelOperator source fonts and license
