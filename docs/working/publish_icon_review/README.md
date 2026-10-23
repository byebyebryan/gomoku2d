# Publish and account action icon review

The Publish globe was selected by the user. Sign in, Sign out, Trash, Sync,
Cloud and Exit are additional visual proposals. This is an asset review: no runtime icon
registration, canonical manifest change, application rebuild or cloud-data
mutation is included.

## Geometry and visual treatment

All seven additions are transparent 24×24 currentColor SVGs made of integer
rectangle runs. Cloud uses a naturally wide 20×14 silhouette, centered with
2 / 5 / 2 / 5 margins; the other additions have two-pixel margins on all sides.

- Publish retains the selected globe unchanged: stepped Circle silhouette,
  two-pixel meridians and equator, symmetry on both axes.
- Sign out is the approved Share mask rotated exactly 90 degrees clockwise.
  Sign in retains exactly that C-shaped frame and reverses only the arrow
  within the same bounds. Both are vertically symmetric.
- Trash has a two-pixel handle, lid, body and inner bars, with horizontal
  symmetry and a one-pixel gap separating the lid from the bin.
- Sync has two opposing arrows and short return tails, with two-pixel shafts
  and 180-degree rotational symmetry.
- Cloud has asymmetric rounded lobes: one larger hump on the right and a
  smaller puff on the left, with a two-pixel outline and flat two-pixel base.
  It is hand-tuned in row runs to keep the lobe junction and native silhouette
  clear. Its wider proportions retain the same centered 20×14 bounds.
- Exit tries a power/quit symbol following feedback that the doorway duplicated
  Sign out. It reuses the original Circle mask, opens the top arc and adds a
  centered four-pixel stem. It is horizontally symmetric. This remains a visual
  alternative; no product action or sign-out behavior has changed.

`index.html` compares fourteen actual source SVGs: the six proposals, selected
Publish globe, and existing Share, Save, Profile, Reset, Copy, Circle and Home.
It offers 1× and 2× samples plus a 10× pixel grid, centre axes, light/dark
surfaces and grid visibility controls. Existing Reset retains its original
margins; the page reports each source's measured margins rather than changing
old assets to match the additions.

`comparison.png` is the full comparison; `new-icons.png` and
`new-icons-light.png` show the top six cards in dark and light surfaces.

## Validation and regeneration

`geometry.json` records source hashes, margins, filled pixels and symmetry.
The generator also proves both account masks derive from the same Share frame
and arrow. `render.json` records exact browser raster readback at 1×/2×/10× for
all fourteen assets, working grid/theme controls, no preview load errors and no
horizontal overflow at 320px. Dark and light renders were visually inspected.
`preview.json` verifies Carbon serves identical source/review bytes.

Regenerate the HTML and geometry receipt from the repository root with:

```sh
rtk proxy python3 docs/working/publish_icon_review/build_review.py
```

The browser renderer is `render_review.mjs`. Run it with the repository's pinned
Playwright container, matching read-only browser cache and host networking,
after copying the HTML and source SVGs to the running local preview directory.

The review is available through the existing Starship/Carbon preview at
<http://127.0.0.1:8013/icon-review/actions/>. The earlier Publish review URL
continues to work. The HTML, source SVGs and PNGs are copied under both
`dist-cloud/icon-review/actions/` and `dist-cloud/icon-review/publish/`; the
existing comparison index links to Action icon drafts. Existing app files,
approved source icons and emulator demo data are untouched.
