# Gomoku2D Assets

Source assets for the web game.

These are the authoring/reference assets imported by the app or used to derive
runtime visuals. Deployed static copies, when needed, live under
`public/assets/`.

## Preview Pages

- Published preview index after deploy:
  <https://dev.byebyebryan.com/gomoku2d/assets/>
- [sprites/preview.html](./sprites/preview.html): board-space sprites,
  animation loops, static poses, and z-order cases
- [icons/preview.html](./icons/preview.html): icon sheet and exported SVG pack
- [fonts/preview.html](./fonts/preview.html): PixelOperator font variants and
  runtime text samples

GitHub renders this Markdown file, but it does not execute these standalone
HTML previews in the repository browser. The web build publishes a curated copy
to `dist/assets/`, so GitHub Pages can serve the live previews under
`/gomoku2d/assets/`.

## Folders

- [sprites](./sprites/): runtime board spritesheets and animation inventory
- [icons](./icons/): DOM shell icon source sheet, manifest, and exported SVGs
- [fonts](./fonts/): PixelOperator source fonts and license
