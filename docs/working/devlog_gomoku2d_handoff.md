# Gomoku2D portfolio refresh — implementation handoff

Prepared October 28, 2026 for the existing **devlog** agent, which owns
`/home/bryan/code/devlog` and `https://dev.byebyebryan.com/`.

Bryan asked the Gomoku thread to work out the presentation update, then hand
implementation to that agent. This is the primary's recommended implementation
direction; final wording and renders have not received a separate human pass.

## Completed implementation — October 28

The Devlog agent published the refresh as `f5a264503e2eb4e247703f7526ea420d15a1b25d`.
[Pages run 40763848744](https://github.com/byebyebryan/byebyebryan.github.io/actions/runs/40763848744)
succeeded on that exact commit. The [live portfolio](https://dev.byebyebryan.com/)
uses the proposed copy, Play / Dev site / Code links, a populated analysis poster,
and a 20-second 1280×720 reel covering guest play, replay/Share, neighboring UI
versions and native/enlarged icon inspection.

The silent H.264/yuv420p MP4 is 503,677 bytes, SHA-256
`0b8cb3bc640f28868cb804a056723c3c18e229a377ee1ead8d4a6c30b598d9ef`;
the PNG poster is SHA-256
`e34a338c8d16a0e0d8c8f8ccaf59ca0830bbea7f723e0225ad73797513624e30`.
Both are pinned to the unmodified released 0.6.3 artifact in the Devlog manifest.
The owner reports successful Jekyll, responsive, keyboard, video/motion/fallback
and byte-range checks, plus Snap parity for 18 served files including all 13
media assets. Only the card, its two assets, manifest and license notices changed.

The Gomoku primary independently confirmed the exact successful Pages SHA,
live card copy/link order, both live media hashes and video format/duration;
inspected the published card and reel contact sheet; and reviewed the owner's
structural, browser and publication receipts. This closes the implementation
handoff with agent review. It adds no human or physical-phone acceptance claim.
The original brief below remains the dated implementation contract.

## Outcome

Refresh the existing Gomoku2D homepage card so visitors can recognize both the
game and the work behind it. Lead with playable features, then offer concrete
development evidence. Keep the established portfolio layout and card position.

The game's own [Dev site](https://gomoku2d.byebyebryan.com/dev/) owns visual
references, UI history/reviews and Bot lab. Link there rather than creating a
second Gomoku microsite or copying that catalog into the portfolio. A longer
development article can follow separately; this handoff does not require one.

## Verified baseline

| Surface | Checked state |
|---|---|
| Portfolio | Clean local/remote `main` at `3f7be3c1e551ebe6e8e5498ec12c18e8040a9488`; GitHub repository `byebyebryan/byebyebryan.github.io` |
| Existing card | `index.md`: Browser game; one sentence about bots/hints/analysis; Play and Code links |
| Existing media | `gomoku2d.mp4`, 960×540, 5 fps, 10.8 seconds; PNG poster. Both public files match local bytes |
| Media provenance gap | Manifest names an older `readme-gameplay.gif` SHA-256 (`ebeaeb88…`) without a source revision; it differs from the current tracked gameplay GIF (`e4590904…`). Do not relabel it as a new 0.6.3 capture |
| Gomoku release | Published `v0.6.3`, source `5981bda1a34abd07790425b6c5ec48705c8005e7`; release documentation at `b6b3bbc1b7eb0e84be4fb58252905b039ce04004` |
| Release status | Exact-source CI, Release/Pages/Firestore workflows and production checks pass. Closing-documentation CI also passed |

The [release record](../archive/v0_6_3_release.md) owns publication and acceptance
limits. [Media provenance](../assets/README.md) distinguishes the older README
captures; the [developer contract](../reference/app/developer_area.md) owns
the public/private boundary and inspection tools.

## Card copy and links

Keep **Gomoku2D** and **Browser game**. Proposed description:

> Gomoku and Renju with tactical hints, replay analysis and shareable games.
> Built with AI, with design reviews and bot experiments open to inspect.

Keep this to two short sentences; the owner can trim to fit the existing card.
The product establishes interest; the second sentence points to inspectable
work. Human product/design judgment remains part of the process. Do not imply
an autonomous build without human decisions, a prompt-count challenge or a
fully solved/verified game-analysis engine.

Use this link order:

| Label | Destination |
|---|---|
| Play | `https://gomoku2d.byebyebryan.com/` |
| Dev site | `https://gomoku2d.byebyebryan.com/dev/` |
| Code | `https://github.com/byebyebryan/gomoku2d` |

The Dev site is the single entry to Visual design, UI design and Bot lab.
Individual feature links would make this card unnecessarily dense.

## One short reel and a useful still

Replace the gameplay-only export with one edited browser-capture reel, roughly
16–20 seconds. Show the real application and tools, rather than a title card
about development methodology. Suggested sequence:

| Beat | Approximate time | What to show |
|---|---|---|
| Play | 0–4 s | A populated board with both stone colours and tactical hints; a move and bot response |
| Replay and share | 4–12 s | Open a Guide example, inspect an analysis moment, then show the full-sidebar Share replay view |
| Inspect the work | 12–20 s | A neighboring-version UI history comparison, then native/enlarged icon inspection |

Capture settled states and cut waiting/navigation time. Adjust timing for
readability; showing these ideas clearly matters more than fitting every click.
Use one coherent reel, with a populated board/analysis frame as the poster.
No split-screen montage or extra autoplay panel is needed. Optional short
editorial labels must be recorded as presentation additions, not product UI.

Useful account-free starting points, checked over HTTPS:

- `/guide/#examples` — use the existing legal bot-game examples; source is
  `gomoku-web/src/replay/replay_examples.ts`.
- `/dev/reviews/history/?version=0.6.2&layout=portrait` — inspect an adjacent
  release change or open its screen history; keep the original version labels.
- `/visuals/?view=icons&compare=share,save,publish` — native previews and grids.

Prefer fresh captures of the published 0.6.3 app, or an isolated exact-tag local
build. Verify the source/version before capture. Use fresh guest browser contexts;
no Google sign-in, real public player, publication write or production account
data is needed. A public-profile demo would require a labelled synthetic fixture
and is outside this minimal reel.

The card is **16:9** and its video uses `object-fit: cover`. Capture a suitable
16:9 viewport, such as 1280×720 at DPR 1, and encode to the same aspect ratio.
Do not put the 960×600 README GIFs into this slot without deliberate containment
or framing: silent cropping could remove replay controls. Review at actual card
width, not just the full-resolution source. Avoid unnecessary upscaling; keep
both stone colours and pixel artwork intact. Use H.264/yuv420p, a fast-start MP4,
no audio, and a bounded payload; approximately 1 MB is a useful target, subject
to visible quality. The owner chooses final encoding after inspecting it.

The [existing media guide](../assets/README.md) provides fallbacks if fresh capture
is blocked. Preserve each reused scene's actual source/version/date and editing
treatment; return a capture blocker rather than inventing newer provenance.

## Ownership and implementation boundary

Devlog owns implementation in its repository:

- `index.md`: this card's description, alt text, three links and asset URLs.
- `assets/projects/gomoku2d.mp4` and `.png` (or an equally bounded replacement
  poster): the reviewed reel/still, with content-hash cache-busting references.
- `assets/projects/manifest.json`: exact source revision/release, capture date,
  browser/viewport/DPR, scene URLs/state, output hashes and editing treatment.

Keep other cards, their order/media, the shared visual style and operational
README intact. Use existing video markup/motion behavior. Change shared layout,
CSS or JavaScript only if a demonstrated requirement needs it; update the
stylesheet cache-buster if CSS changes. Do not change Gomoku application code,
frozen history/review media, releases or the existing preview/report services.

Keep capture scripts/raw frames/build output isolated and ignored. Preserve
concurrent work. Follow the recipient thread's existing commit/publication
authorization; this handoff adds no unrelated push/deploy or product scope.

## Acceptance and return

1. Build Jekyll with locked dependencies into an owned temporary destination;
   preserve existing previews and `_site/`. Do not add dependency updates.
2. Inspect the card at 320, 390, 681 and 1160 CSS-pixel widths. Confirm readable
   framing, no overflow/unintended crop, stable links, keyboard focus and touch
   access. Check actual black/white stones and a still from every reel beat.
3. Verify native playback controls, global Pause motion, reduced-motion, manual
   pause, offscreen behavior and the no-JavaScript/poster fallback. A visitor
   should understand the game and reach its development evidence without motion.
4. Check the three destinations, media hashes/manifest, aspect ratio, duration,
   audio absence and cache-busted URLs. Recheck other cards and media references.
5. Return the final copy, representative card/reel frames, commit and checks.
   Distinguish local candidate, pushed source and deployed/live verification.
   If publication is authorized, require Pages success and served-byte parity.

Do not turn the release's open real-account, physical-phone or human-accessibility
checks into portfolio claims. Retrospective capture is not original-period human
acceptance; emulation is not physical-device evidence.
