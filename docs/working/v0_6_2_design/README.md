# 0.6.2 public-profile design study

Owner: Bryan. October 23, 2026. Source baseline: roadmap commit `069eb66`.
Status: research/prototypes ready for visual review; no layout adopted and no
production implementation begun.

**[Open the interactive comparison](http://starship:8787/reports/29c155940e692a2afdbd03537113c7ddd1a035b7ea19d74d3a09544c075cf2c7/).**
The [report catalog](http://starship:8787/reports/#project=gomoku2d) retains
earlier project reports. Earlier visual studies are historical, not an adopted
backlog for this pass.

## Finding

The current visitor screen shares the app's tokens but not its composition:
an 820px centered card, five separate statistics boxes, a prominent disclosure
before games, and Home at the bottom. Profile, Replay and the public teaching
pages use stronger headers, visible navigation, section rhythm and divided
records. The design issue can be addressed with existing data.

The source review included PublicPlayerRoute, PublicProfileManager,
ProfileRoute styles, Settings/Rules styles, global tokens and the actual icon
assets. Recorded Settings/Replay/public-profile/owner captures are identified
by provenance in the report; they are not newly captured production evidence.
External research used first-party
[Chess.com collection documentation](https://support.chess.com/en/articles/13557248-how-do-i-use-game-collections)
for reader/owner roles and
[W3C navigation](https://www.w3.org/WAI/WCAG22/Understanding/consistent-navigation)
and [reflow guidance](https://www.w3.org/WAI/WCAG22/Understanding/reflow.html).
A Lichess public profile returned HTTP 403 and was excluded as visual evidence.

## Alternatives

| Option | Composition | Tradeoff |
|---|---|---|
| A: Wide record | Player-name app header; shared-record strip; full-width divided game list | Recommended starting point. Keeps games prominent and collapses naturally when statistics are hidden. |
| B: Profile columns | App header; identity/record rail; game list beside it | Closest to private Profile. The rail is less useful when little is public and reduces room for game titles. |
| C: Compact record | Visible app navigation above one panel containing identity, record and games | Contained and restrained, but retains more of the standalone-card character. |

The current structure is reconstructed with the same fixtures for comparison;
it is not a screenshot of a real account or a pixel-exact runtime replica.
All alternatives use the existing fonts, palette, controls and SVG icons.
The owner companion is one shared exploration of General Settings-style rows
inside the Profile shell, not three independently redesigned managers.

The comparison supports 32/one/zero games, shared/hidden statistics,
normal/long Unicode labels, desktop/390px/320px viewports, enlarged text and
loading/error/unavailable states. Pagination and replay-return focus work.
Owner actions simulate name/stats/public-visibility changes in memory only;
navigation buttons describe their destination without leaving the study.
The owner publication list is a short composition sample.

All game metadata is fictional. The shared 38-game record is independent of
the selected public-game count. Replay and optional thumbnail boards are
illustrations, not validated full-game fixtures or a new renderer. The
thumbnail toggle is explicitly a `0.6.3` composition sketch and is off by
default. No avatars, bios, extra statistics, ratings or played dates are added.

## Recommendation and decisions still open

Start from **A**, retaining B's continuity with the private record screen.
Settle composition, record density, disclosure placement and the visitor Copy
link action with Bryan before turning the study into a bounded implementation
plan. A compact player-shared/unverified label stays visible; fuller context
is behind About these records rather than a large warning block above games.

This recommendation is not human visual acceptance. The study does not close
the real-account sharing, physical touch or human accessibility handoffs.
The [0.6.2–0.6.4 plan](../v0_6_followup_plan.md) retains the release boundaries.

## Build and delivery

Authoring files live here. Prepare only the intended static export:

```sh
rtk proxy python3 docs/working/v0_6_2_design/prepare_export.py
```

The default output is ignored
`gomoku-web/test-results/v062-design/export/`. Preparation copies the current
global stylesheet, three runtime fonts and their license, the needed original
SVG icons, and four named reference captures into the export. It does not
change those source assets, the app entrypoints or the existing manual preview.
Serve the export on an owned free port or publish it with the HTML report tool.
Do not use unrelated port 8001 or the manual preview/emulator ports.

Publication is an immutable 16-file, 245,173-byte report. All 16 served files
matched the prepared export byte-for-byte from Starship. The shared report
server was reused; this is not an app deployment. Carbon/device reachability
and human review are not inferred from the Starship delivery check.

## Prototype validation

The [evidence receipt](evidence.json) records the source/asset hashes, published
identity and checks. Raw screenshots and scripts remain in ignored
`gomoku-web/test-results/v062-design/`.

- Chromium inspection covered desktop/mobile A/B/C/current, sparse profiles,
  the owner companion, and 200% text with long labels. It exposed a nested
  grid's minimum-width overflow; containment and statistics reflow were fixed.
- The three-engine pass completed **117 layout/interaction checks** across
  Chromium, Firefox and WebKit. It covered list states, long labels, enlarged
  text, owner controls, stable pagination, replay return to the same page and
  trigger focus, report controls and the HTTP clipboard fallback. The pass
  reported one WebKit ResizeObserver warning in the report wrapper.
- The wrapper was corrected to react only to width changes and defer resize
  writes. **12 targeted rechecks passed with no page errors**, covering report
  switching/resizing in all three engines, the refined current-layout
  reconstruction and narrow enlarged-text thumbnail sketches.
- The published report received a separate Chromium interaction and HTTP-copy
  fallback check. Copied prototype links retain the standalone study notice.
  No external runtime requests or cloud operations were needed by the study.

These are checks of an isolated static prototype, not application CI, a
Firestore/account lifecycle test, a WCAG conformance assessment or visual
approval. Product runtime, persistence schemas, rules, manual emulator data
and production user data were not modified. No user-browser tabs were opened.
