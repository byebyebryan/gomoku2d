# 0.6.1 public-profile acceptance

Local functional candidate for the [reviewed contract](../v0_6_1_public_profile.md).
Luna Max implemented the data/rules and UI slices; the primary reviewed the
contracts, extracted shared played-game stats, isolated cloud acceptance from
the manual preview, and owns integration and final acceptance. No release, push
or production deployment is part of this checkpoint.

## Evidence boundaries

- [Sharing operations and bounded shapes](sharing_shape.json): production Web
  SDK method instrumentation against an isolated demo Firestore emulator.
  Counters exclude rule-dependent reads, retries, protocol/index overhead and
  billing; they do not establish a free-tier traffic capacity.
- Browser checks use desktop Chromium, Firefox and WebKit engines in the
  qualified Ubuntu Playwright container. They do not establish physical-device
  or human assistive-technology acceptance.
- The manual Carbon URL is forwarded to Starship's loopback demo preview and
  Auth/Firestore emulators. Test lanes use separate ports and disposable data.
- Real Google OAuth, public-host/release acceptance, and the existing Canvas
  stone-colour defect remain separate. No rendering/core changes are included.

## Overall review — October 23, 2026

Reviewed runtime `70e7f25`, including the accepted profile/publishing UI, action
icons and fixed pagination positions. Bryan accepted the current presentation
in the conversation. This pass traced the data, rules and route contracts and
reran the complete relevant web gates; it changes no application code.
The [review receipt](overall_review.json) records the current results, findings,
raw artifact locations and remaining boundaries.

Two findings remain:

1. **P1 — existing Canvas stone colours block release acceptance.**
   [BoardRenderer](../../../gomoku-web/src/board/board_renderer.ts) draws both
   sides with the same white sprite and relies on `setTint` for Black. Phaser's
   tint component is WebGL-only, while the board selects `Phaser.AUTO`. A fresh
   Chromium context with WebGL disabled reproduced both sides appearing white
   in the real shared replay. The renderer source is unchanged from v0.5.5.
   Flow tests passing does not close this finding; fix the fallback and add
   actual rendered-colour coverage before release.
2. **P3 — timestamp-only cloud fixtures collide under parallel repetition.**
   [C4's account name](../../../gomoku-web/cloudtests/continuity.spec.ts) uses
   only `Date.now()`. An extra three-worker repetition run produced identical
   account emails and duplicate emulator account-picker entries: 42/45 passed,
   with three Chromium failures at the sign-in selector before the application
   race assertions. Preserved traces show the duplicate email. The configured
   one-worker run passed 45/45. Give each fixture a collision-proof identifier
   before relying on parallel runs; keep the strict selector.

No new functional, privacy or data-integrity defect was found in the 0.6.1
changes. The reviewed implementation keeps statistics as explicit snapshots of
the My games totals, waits for the current account's complete history, excludes
study copies, and preserves stale-write/account barriers. Sparse and legacy
catalogs still form eight-row logical pages. Private Reset preserves public
sharing; disable and Delete Cloud clear it with the revision markers retained.
Fresh public routes avoid private history work. Sharing adds no polling,
per-move traffic or service. Logical SDK operation checks remain distinct from
billing or a promised free-tier traffic capacity.

| Fresh gate at `70e7f25` | Result |
|---|---|
| App/scripts/tools type checks and production build | Passed |
| Web unit tests | 488/488, 48 files |
| Firestore rules and production SDK scenarios | 49/49 |
| Ordinary browser journeys, three engines | 207/207 |
| Full cloud journeys, three engines | 42/42 |
| Configured C4/C6/C7 repetitions, five per engine | 45/45 |
| Public-profile/publishing accessibility audits | 18 reports, zero violations and zero incomplete checks |
| Production direct/static entries | 12/12, served HTML and entry script match the build |
| Production dependency audit | Zero reported vulnerabilities |

Raw artifacts are retained under ignored
`gomoku-web/test-results/v061-overall/`. Both the failed extra parallel run and
the successful configured run survive there. The temporary ordinary-browser
preview was stopped; the manual preview/emulators and Carbon forwarder remain.
Starship serves the reviewed emulator build. Carbon is offline on Tailscale at
the final check, so current Carbon reachability is unverified.

The next implementation should close the Canvas finding and harden the test
fixture IDs. Release preparation then needs its own version/exact-head CI,
real OAuth/public-host round trip, and reviewed rules/index deployment gates.
Physical-device and human assistive-technology acceptance are not established
by this pass. No push, version change, tag or production deployment occurred.

## Functional checkpoint

Runtime implementation: `bf7d963` on local `main` (main feature slice `7e9f803`),
following reviewed data/rules
commit `a01d1ec`, the sharing-switch fix `c36ff48`, and unused-index exemptions
`c9d894c`. Package version remains 0.5.5 until separate release preparation.

| Gate | Result |
|---|---|
| Full app, script and test-tool type checks | Passed |
| Web unit tests | 488/488 in 48 files |
| Firestore rules and production SDK scenarios | 49/49; unchanged after the final rules pass |
| Production build and direct entries | Passed; 12 HTTP/static-file entries and entry-script parity in [static_routes.json](static_routes.json) |
| Ordinary browser lane | 204/207 passed initially; its three failures were the same stale continuation assertion, corrected and rerun 3/3 across engines |
| Full cloud lane | 42/42 across Chromium, Firefox and WebKit |
| Required C4/C6/C7 repetitions | 45/45, five fresh-state repetitions in each engine |
| Additional reflow and demo audit | Eleven width/text-size combinations, no document or internal main overflow; zero axe violations at 640px/200%, and Replay opens at 4/9. [Receipt](demo_reflow.json) |
| Carbon preview | HTML and entry-script bytes match Starship; Auth ready, public fixture readable, three managed services active. [Receipt](preview.json) |

The final visitor-only responsive adjustment was subsequently covered by all
three cloud engines (C15), including actual shared stats and maximum fields.
The ordinary browser scenario fix changes its expectation to the already
accepted continuation behavior; no app workaround was made for those failures.
Rust/core code and dependencies are unchanged in this checkpoint.

Review of the retained reports exposed labels on generic containers. The
final semantic correction names the stats group properly and uses hidden text
for row context. Its affected route units passed 42/42, the production build
passed again, and C11/C15 passed 6/6 across the engines. That pass recorded
twelve audits with **zero violations and zero incomplete checks**; raw JSON survives the line reporter under ignored
`gomoku-web/test-results/v061-accessibility`. These assertions do not establish
complete WCAG or human screen-reader acceptance. The visible layout is unchanged
by this final semantic correction.

Primary commands ran from `gomoku-web` (cloud commands include all three
engines by default):

```sh
rtk proxy npm run typecheck
rtk proxy npm test
rtk proxy npm run test:rules
rtk proxy env GOMOKU_BASE_PATH=/ npm run build
rtk proxy env GOMOKU_CLOUD_PORT_OFFSET=1000 GOMOKU_WRITE_EVIDENCE=1 npm run test:cloud -- --container --reporter=line
rtk proxy env GOMOKU_CLOUD_PORT_OFFSET=1000 npm run test:cloud -- --container --grep 'C4|C6|C7' --repeat-each=5 --reporter=line
rtk proxy env GOMOKU_CLOUD_PORT_OFFSET=1000 npm run test:cloud -- --container --grep 'C11:|C15:' --reporter=line
```

Ordinary journeys used the repository's Playwright configuration in the pinned
`mcr.microsoft.com/playwright:v1.59.1-noble` container, with host networking,
the matching read-only browser cache, `PLAYWRIGHT_BASE_URL=http://127.0.0.1:8014`,
`--browser=all --workers=3 --reporter=line`. The corrected keyboard scenario
was rerun alone in all engines; the temporary target has since been stopped.

## Public-profile and publishing UI refinement

The subsequent visual pass (`81922a4`) follows General Settings' grouped label/control
rows, stacking by available card width. Public identity, profile links,
statistics and publication controls have bounded actions and existing icons.
Publish replay uses the dedicated globe icon and swaps the whole replay
sidebar, sharing the Share replay styling, entry motion, paused playback and
Back/Escape return with focus and scroll restoration. A compact preview replaces
paragraphs with the destination, title, opening move and black/white players;
short copy keeps the public-copy and unverified-record disclosures visible.
Both sidebar panels explicitly paint their existing background, so offscreen
text has a defined background during scrolling and accessibility inspection.

Validation after this UI change:

- App/scripts/tools type checks passed. The production build and final isolated
  cloud/manual emulator builds passed.
- Affected PublicSharing, ProfileRoute and ReplayRoute component suites: **52/52**.
- Shared replay browser scenarios: **18/18**, across Chromium, Firefox and WebKit.
- Final C11/C15: **6/6** across the three engines, including public opt-in,
  publishing, unchanged board canvas, playback pause, sidebar replacement,
  Escape/focus return, unpublish/disable and maximum Unicode fields.
- C15 additionally covers owner-settings and publishing reflow at 640px / 200%
  text. [Eighteen current audits](accessibility.json) report zero violations and
  zero incomplete checks. Desktop and mobile screenshots were inspected.
- The backend, rules, indexes and public visitor behavior are unchanged; their
  earlier full-lane results above are the functional baseline, not new runs.

The manual preview was refreshed from `81922a4`; Carbon and Starship served
identical HTML and entry-script bytes. Auth and the existing public demo fixture
were reachable, and all three managed services remained active.

Raw final cloud artifacts are retained under ignored
`gomoku-web/test-results/v061-ui-final`. The current
[preview receipt](preview.json) identifies the served runtime commit.

## Statistics control follow-up

Public statistics use the same Off/On segmented buttons as General Settings.
The selected state follows the saved snapshot. On publishes once; selecting the
active button does no cloud work. Off removes the snapshot even while private
history is unavailable. Refresh stays a separate Update public stats action.

The affected component suites passed 42/42, full app/scripts/tools type checks
passed, and the emulator preview build passed. C11/C14/C15 passed 9/9 across
Chromium, Firefox and WebKit; the eighteen [current audits](accessibility.json)
have zero violations and incomplete checks. Owner desktop/mobile captures were
refreshed; raw artifacts are in ignored `gomoku-web/test-results/v061-stats-buttons`.

## Accepted icon integration

The reviewed Publish, Sign in/out, Trash, Sync and power-style Exit icons are
canonical assets and appear in Visuals. Publish uses the globe; account entry /
exit, deletion and account-library sync use their dedicated icons. Existing
Replay, Copy, Reset and Minus complete related controls. Exit is registered for
future quit controls. Cloud is deferred and removed from the current pack and
review gallery. The manual preview is refreshed and verified locally; Carbon
SSH currently times out, so its current served-byte parity remains unverified
until the existing tunnel reconnects.

App/scripts/tools type checks, the production and emulator builds, 60 affected
route component tests and 9 C11/C14/C15 journeys passed. Eighteen [current
audits](accessibility.json) have zero violations and incomplete checks. Raw
artifacts are under ignored `gomoku-web/test-results/v061-accepted-icons`.

## Contract review

| Requirement | Acceptance evidence |
|---|---|
| Full third tab, deep link and keyboard navigation | Profile route units and the three-engine narrow-screen browser scenario |
| Same totals as My games, including continuations and excluding study copies | Shared pure stats helper scenarios, ordinary continuation browser journey, C3/C5 and C11 |
| Explicit opt-in, snapshot refresh/removal and valid zero record | Owner component tests and three-engine C11/C16 |
| Loading/account barriers and delayed callbacks | Current-account cache/readiness scenario, delayed stats-save component scenario, C6 repetitions and rules conflicts |
| Compact eight-row list, gaps and cache reuse | Paging helper/component scenarios and C16: physical 5/0/8/5 gives logical 8/8/2 |
| Removal of last logical page and legacy roots | C16 reloads after removal to 8/8, then reads the manifest-free root without a migration write |
| Correct result attribution and unavailable old/malformed records | Four black/white win/loss scenarios, neutral malformed/unattributed cases, actual portable demo replay |
| Public-only route makes no private reads or promotion | C10 signed-in route variants in all three engines |
| Reset preserves snapshot; disable/delete clear it and retain barriers | C14, C11, stale refresh/clear rules races, exact raw timestamp preservation tests |
| Bounded explicit backend work | [SDK operation receipt](sharing_shape.json): stats 1 read/1 write, ordinary catalog 2 reads/2 writes; the legacy SDK scenario separately asserts 5 reads/2 writes |
| Responsive and accessible controls | C11/C15 axe assertions, maximum Unicode/200% text, internal-overflow regression, inspected desktop/mobile captures |

No avatar, bio, featured game, board thumbnail, automatic stats sync, per-move
sharing writes, polling or new service was added. Index exemptions are source
configuration only; no production rules/index deployment was performed.

## Preview and inspected captures

From Carbon, open [Public profile settings](http://127.0.0.1:8013/profile?tab=public)
or the [18-game demo profile](http://127.0.0.1:8013/player/?id=v061-demo-player).
The latter is an explicitly seeded local emulator fixture with valid playable
portable records, not evidence of owner publication. Owner publication is
verified separately through the actual SDK/UI in C11/C14. Existing manual demo
records were preserved. Temporary acceptance previews were stopped; the
managed manual preview, emulator and Carbon tunnel remain available.

Inspected examples:

- [Desktop owner controls](sharing-chromium-profile-manager-desktop.png) and [mobile controls](sharing-chromium-profile-manager.png).
- [Desktop publishing sidebar](sharing-chromium-publish-replay-desktop.png) and [mobile sidebar](sharing-chromium-publish-replay.png).
- [Desktop paged visitor list](public-demo-1280.png) and [mobile list](public-demo-390.png).
- [Actual owner-published visitor page](sharing-chromium-public-profile.png).

Three-engine C11 and C15 captures are retained alongside these. Full-cloud
artifacts remain under ignored `gomoku-web/test-results/v061-cloud-full`, race
artifacts under `v061-cloud-races`, and the corrected ordinary keyboard
scenario under `v061-reconciliation`. Source/evidence commits are local only;
no push, tag, production deployment or real cloud-data mutation occurred.

## Human review path

1. Open `/profile?tab=public`. Check the full main-area tab on desktop and a
   narrow window, its keyboard navigation, and the signed-out guidance.
2. Sign in to the local demo. Enable a public name; check that public statistics
   start off. Share the current summary, view the public page, then update or
   remove the snapshot. A zero-game record should show an em dash for win rate.
3. Publish games from My games. Check the sidebar replacement, preview, Back /
   Escape return, and unchanged opening position. Browse the public list, open a replay, and
   unpublish an item from management. Fewer than nine games need no pagination.
4. Check that private Reset explains that public stats and publications stay.
   Disable public sharing if desired; enabling again starts empty, stats off.

Use a disposable demo account for reset/disable/delete checks. Further visual
refinement can follow along the 0.6 line; this checkpoint establishes working
identity, visibility, snapshots and game-list navigation.
