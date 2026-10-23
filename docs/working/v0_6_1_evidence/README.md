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

## Final gate

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
passed again, and C11/C15 passed 6/6 across the engines. [Twelve final audit
summaries](accessibility.json) record **zero violations and zero incomplete
checks**; raw JSON survives the line reporter under ignored
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

- [Desktop owner tab](owner-desktop-tab.png) and [mobile owner controls](sharing-chromium-profile-manager.png).
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
3. Publish games from My games. Browse the public list, open a replay, and
   unpublish an item from management. Fewer than nine games need no pagination.
4. Check that private Reset explains that public stats and publications stay.
   Disable public sharing if desired; enabling again starts empty, stats off.

Use a disposable demo account for reset/disable/delete checks. Further visual
refinement can follow along the 0.6 line; this checkpoint establishes working
identity, visibility, snapshots and game-list navigation.
