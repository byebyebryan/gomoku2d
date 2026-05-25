# Gomoku2D Assets

Source assets for the web game.

These are the authoring/reference assets imported by the app or used to derive
runtime visuals. Deployed static copies, when needed, live under
`public/assets/`.

## Published Preview

- Published asset viewer after deploy:
  <https://gomoku2d.byebyebryan.com/assets/>

GitHub renders this Markdown file, but the visual preview is now part of the
React app. The web build publishes raw files and `asset_manifest.json` to
`dist/assets/`, then serves the app route at `/assets/`.

## Folders

- [sprites](./sprites/): runtime board spritesheets and animation inventory
- [icons](./icons/): DOM shell icon source sheet, manifest, and exported SVGs
- [fonts](./fonts/): PixelOperator source fonts and license
