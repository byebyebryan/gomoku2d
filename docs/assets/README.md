# Current Media Assets

This folder holds current release-facing media for the root README and related
project showcase surfaces. These captures document the prepared `0.5.5` build,
recorded on October 16, 2026 from a production preview whose application and
version inputs match source
`f300a30c7bda13738010637c700e67ec8aaa0670`. They show the retained UI and current
replay-analysis presentation. The 1280×800 capture viewport keeps replay
controls in frame; GIFs are 960px wide at 5 fps, and OG images are 1200×630.
Publication/deployment is tracked separately in
the [release preparation record](../archive/v0_5_5_release_preparation.md).
The October review keeps focused development screenshots in the
[October reconciliation archive](../archive/reconciliation_2026_10/README.md).

Tracked outputs:

- `readme-gameplay.gif`: a short gameplay loop with tactical hints, a played
  move, and the bot response.
- `readme-analysis.gif`: a short Replay Analysis loop stepping backward from a
  finished frame through a longer setup corridor to the last escape.
- `readme-lab.gif`: a lab report loop showing ranking, search telemetry, and a
  long-corridor analysis proof drill-down.
- `readme-visuals.gif`: a visual guide loop showing style, icon exports, and
  sprite previews.
- `readme-replay-analysis.png`: a still Replay Analysis frame used as the Open
  Graph source crop.
- `gomoku-web/public/og.png` and `gomoku-web/assets/og_source.png`: social
  preview images generated from the Replay Analysis still.

Regenerate the README media from a production preview:

```sh
cd gomoku-web
npm run build
npm run preview -- --host 0.0.0.0 --port 8001
```

Then, in a second shell:

```sh
cd gomoku-web
GOMOKU_MEDIA_BASE_URL=http://127.0.0.1:8001 npm run media:readme
```

Only commit the polished outputs that are referenced by public docs. Scratch
frames and intermediate files should stay under `/tmp/gomoku2d-readme-media`.
