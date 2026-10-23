# Accepted action icon additions

The six retained icons are now canonical assets under `gomoku-web/assets/icons/`:
Publish, Sign in, Sign out, Trash, Sync and Exit. Their SVG bytes match the
reviewed versions. Cloud is deferred; its SVG drafts and comparison rows were
removed. Prior experiments remain available in Git history.

## Integration

The runtime icon registry and the Visuals manifest include all six additions.
Publish replaces the ambiguous Profile icon on the replay trigger, panel heading
and confirmation action. Sign in/out use the account pair. Delete Cloud and
private replay removal use Trash; explicit account-library sync uses Sync.
Existing Replay, Copy, Reset and Minus icons complete related controls. Minus
marks Unpublish because that removes a listing rather than the private game.
The selected power-style Exit is available in the pack; no new quit action was
introduced.

The original 32-icon sheet stays unchanged. Every added glyph uses a transparent
24×24 currentColor SVG, integer rectangle runs and two-pixel margins.
Sign out is Share rotated clockwise; Sign in reverses only its arrow within the
fixed frame. Publish and Exit retain the selected circle-derived shapes.

## Comparison and validation

`index.html` compares thirteen canonical source SVGs, with native 1×/2× samples,
a 10× pixel grid, centre axes, and dark/light surface controls. Existing Reset
retains its original margins. `new-icons.png` and `new-icons-light.png` show the
six accepted additions; `comparison.png` includes the reference icons.

`geometry.json` verifies bounds, integer rectangles, symmetry and the account /
power source derivations. `render.json` compares the preview against current
canonical source masks and validates exact raster pixels at 1×/2×/10×, controls
and 320px reflow. `preview.json` records Carbon served-byte verification.

The integration passed app/scripts/tools type checks, the production and manual
emulator builds, 60 affected route component tests, and 9 C11/C14/C15 journeys
across Chromium, Firefox and WebKit. Eighteen current accessibility reports have
zero violations and incomplete checks. Raw cloud artifacts are under ignored
`gomoku-web/test-results/v061-accepted-icons`.

Rebuild the comparison from the repository root with:

```sh
rtk proxy python3 docs/working/publish_icon_review/build_review.py
```

Run `render_review.mjs` in the pinned Playwright container after copying the
comparison HTML to the running local preview. Both
<http://127.0.0.1:8013/icon-review/actions/> and the earlier Publish URL remain
available. Canonical assets are also published through `/visuals/`.
