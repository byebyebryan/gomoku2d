# Current Media Assets

This folder holds release-facing media for the root README and related
project showcase surfaces. Captures have different source versions:

| Assets | Capture source and date |
|---|---|
| `readme-sharing.gif`, `readme-public-profile.png`, `readme-analysis.gif`, `readme-replay-analysis.png` | Prepared `0.6.2`, October 24, 2026; clean application/version inputs at `bbc652f31fc4a211a85d1e60f61905617136f5a6`. Captured from the local emulator-enabled preview. This is candidate evidence, not production acceptance. |
| `readme-gameplay.gif`, `readme-lab.gif`, `readme-visuals.gif`, public/source OG artwork | Prepared `0.5.5`, October 16, 2026; application/version inputs at `f300a30c7bda13738010637c700e67ec8aaa0670`. These unaffected assets were retained. |

The 1280×800 capture viewport keeps replay controls in frame; GIFs are 960px
wide at 5 fps, and OG images are 1200×630. The new sharing clip is six seconds;
the refreshed analysis clip is 5.4 seconds. The public profile uses the existing
local 18-game Demo Player fixture, not a production player's account. Only
keyboard focus was cleared; content/styles were not substituted for capture.
Hashes and acceptance limits live in the
[0.6.2 candidate](../working/v0_6_2_candidate.md) and its evidence. Publication
is recorded in the [0.6.2 release](../archive/v0_6_2_release.md); the earlier
[0.5.5 preparation record](../archive/v0_5_5_release_preparation.md)
retains the original media history.
The October review keeps focused development screenshots in the
[October reconciliation archive](../archive/reconciliation_2026_10/README.md).

Tracked outputs:

- `readme-gameplay.gif`: a short gameplay loop with tactical hints, a played
  move, and the bot response.
- `readme-analysis.gif`: a short Replay Analysis loop stepping backward from a
  finished frame through a longer setup corridor to the last escape.
- `readme-sharing.gif`: Guide example → finished replay → earlier move → Share.
- `readme-public-profile.png`: public player card, optional record and game list.
- `readme-lab.gif`: a lab report loop showing ranking, search telemetry, and a
  long-corridor analysis proof drill-down.
- `readme-visuals.gif`: a visual guide loop showing style, icon exports, and
  sprite previews.
- `readme-replay-analysis.png`: a still Replay Analysis frame.
- `gomoku-web/public/og.png` and `gomoku-web/assets/og_source.png`: social
  preview images combining a separate board-only replay-branch capture with the
  project title and tagline.

The OG artwork is also uploaded as the GitHub repository's social preview.
GitHub stores its own copy; refresh it in repository Settings → General →
Social preview when changing the artwork. See
[GitHub's upload instructions](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/customizing-your-repositorys-social-media-preview).

For the original gameplay/Lab/Visuals/OG recipe, start an owned production
preview (verify the port is free first):

```sh
cd gomoku-web
npm run build
npm run preview -- --host 127.0.0.1 --port 8014 --strictPort
```

Then, in a second shell:

```sh
cd gomoku-web
GOMOKU_MEDIA_BASE_URL=http://127.0.0.1:8014 npm run media:readme
```

Stop the preview with Ctrl-C after capture. Run `npm run build` again to copy
the updated public OG image into `dist/` before publishing.

For sharing/profile/analysis media, use an owned emulator-enabled preview with
a reviewed Demo Player fixture. The recipe reads that profile; it does not
seed or mutate it. The original capture used port 8013 and UID
`v061-demo-player`:

```sh
cd gomoku-web
GOMOKU_MEDIA_BASE_URL=http://127.0.0.1:8013 \
GOMOKU_MEDIA_PUBLIC_PROFILE_ID=v061-demo-player npm run media:sharing
```

The script requires loopback, an explicit UID, installed Playwright Chromium
and native FFmpeg. When Chromium runs in a container without FFmpeg, pass
`GOMOKU_MEDIA_FRAMES_ONLY=1` to the capture stage, then run
`node scripts/encode_sharing_media.mjs` on the host against the same frame
directory. Both stages honor `GOMOKU_MEDIA_TMP_DIR`. The default is ignored
`gomoku-web/test-results/v062-completion/media/`, including `capture.json`.
The original recipe's scratch files remain under `/tmp/gomoku2d-readme-media`.

Only commit the polished outputs referenced by public docs. Review the images,
GIF duration and capture metadata before replacing their provenance above.
