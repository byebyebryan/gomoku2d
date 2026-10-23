# Publish replay globe draft

Review-only proposal; the app button still uses the profile icon. No runtime
icon registration, canonical manifest change, application rebuild or cloud-data
mutation is included.

- Source: `publish.svg`, a transparent 24×24 currentColor SVG made of integer
  rectangle runs. It has two-pixel margins on every side and both horizontal
  and vertical symmetry.
- The stepped outer contour follows the existing Circle silhouette. Meridians
  and equator use two-pixel bands, matching the straight frame weight of the
  Profile, Save and Share assets.
- `index.html` compares the actual source SVGs for Publish, Share, Save, Profile,
  Circle and Home at 1×, 2× and a 10× pixel grid, with centre axes and light/dark
  surfaces. `comparison.png` is the inspected dark comparison.
- `geometry.json` records source hashes, margins, filled pixels and symmetry.
  `render.json` records exact browser raster readback at 1×/2×/10× for all six
  assets, working grid/theme controls, no preview load errors and no horizontal
  overflow at 320px.
- `preview.json` verifies that Carbon serves identical HTML, SVG and PNG bytes.

Regenerate the HTML and geometry receipt from the repository root with:

```sh
rtk proxy python3 docs/working/publish_icon_review/build_review.py
```

The review is available through the existing Starship/Carbon preview at
<http://127.0.0.1:8013/icon-review/publish/>. Its HTML, SVG and PNG are copied
under the preserved `dist-cloud/icon-review/publish/` directory; the existing
comparison index has a Publish draft link. Existing app files and approved
source icons are untouched.
