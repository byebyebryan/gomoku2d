# v0.6 local candidate evidence

October 21, 2026. This is local production-build and emulator evidence from the
Luna goal loop, not a deployed release. The package version remains 0.5.5;
v0.6 changes are Unreleased. The original accepted application snapshot was `e98f4f7`; `ecff07f` disables preview
auto-open. [Final build identity](build_identity.json) proves that snapshot's rebuilt 134-file
dist tree is byte-identical to the browser-qualified candidate. Later application fixes have separate
[scoped validation below](#post-review-fixes). Implementation commits start at `e4a8a44`; see the
[execution record](../v0_6_execution.md) and repository history for slice heads.
The later [unattended review report](../v0_6_review_evidence/README.md) qualifies
application source `f1bc7ee` and records its fixes, fresh checks and manual handoff.

## Original full-candidate checks

| Lane | Result |
|---|---|
| Web typecheck | Pass, application/scripts/tools |
| Web units | 44 files / 433 tests pass |
| Firestore emulator rules | 2 files / 43 tests pass |
| Production build | Pass; copied shared/player static entries and reports present |
| Three-engine browser flows | 189/189 pass, Chromium/Firefox/WebKit |
| Complete cloud application lane | 36/36 pass, Chromium/Firefox/WebKit |
| Cloud race repetitions | 45/45 pass, C4/C6/C7 × five fresh runs × three engines |
| Production npm audit | Zero reported vulnerabilities |
| Cargo audit | Pass, 100 locked crate dependencies scanned |
| Plain static HTTP entry | Home/shared/player return 200; [receipt](static_routes.json) |

Browser acceptance uses the lockfile’s Playwright 1.64.1 browsers. Host Chromium
and Firefox run, but host WebKit lacks system libraries. The qualified Ubuntu
container image `mcr.microsoft.com/playwright:v1.59.1-noble` supplies libraries;
the mounted host browser cache supplies the matching 1.64.1 binaries. This
specific combination passed; an image’s bundled older browser cache was not
used. Ordinary and cloud lanes have separate output directories. Preview auto-open is explicitly disabled, so automated runs do not create
external user-browser tabs. Routine tests put captures in test output; `GOMOKU_WRITE_EVIDENCE=1` opts into refreshing the screenshots
checked into this directory.

Commands from `gomoku-web` (all shell commands in this workspace use `rtk`):

```sh
rtk npm run typecheck
rtk npm test
rtk npm run test:rules
rtk npm run build
rtk proxy docker run --rm --init --network host --ipc host --user 1000:1000 -e CI=1 \
  -v /home/bryan/code/gomoku2d:/workspace \
  -v /home/bryan/.cache/ms-playwright:/ms-playwright:ro \
  -w /workspace/gomoku-web mcr.microsoft.com/playwright:v1.59.1-noble \
  node node_modules/@playwright/test/cli.js test --browser=all --workers=3
rtk npm run test:cloud -- --container
rtk npm run test:cloud -- --container --grep 'C[467]' --repeat-each=5
rtk proxy npm audit --omit=dev
```

`cargo audit` ran from `gomoku-bot-lab`. Rust/core/bot/Wasm source and dependency
locks were unchanged; no new bot tournament or research measurement was started.
The existing manual-check deferrals remain in force. Incomplete axe findings
are retained in Playwright attachments; automated scans do not prove human
screen-reader use. New scans cover guest shared viewing/Saved replays at both
sizes plus public management/publishing/visitor views in the cloud journey.

## Post-review fixes

Review after `da0491a` found two UI defects. The private cloud-session guard used
a stricter path expression than React Router, allowing public-route case,
trailing-slash and encoded-path variants to initialize private synchronization.
The guard now uses React Router's route matcher; regressions verify public
variants, private routes and navigation back into the private profile.

Replay saves now disclose when copies are kept only for the current session,
explain reload/close loss and recommend retaining the replay link. The Save
control and guest library no longer promise a device save after persistence
fails. A confirmed cloud save still reports account durability even if the
device cache is unavailable. Tests exercise actual quota failures, unusable
caches, guest fallback and successful/failed account sync.

Scoped revalidation: 56 unit/component checks across five files, complete web
typecheck and a fresh production build pass. Shared/library journeys pass 24/24
across Chromium/Firefox/WebKit, including an injected quota failure, temporary
copy loss on reload, 390px overflow checks and an axe scan of the failure state.
Browser output is in `gomoku-web/test-results/review-fixes`; checked-in captures
and original build receipts above retain their earlier snapshot meaning. The
full browser/cloud matrices above were not rerun for this follow-up.

## Journeys and review

Guest tests exercise selected-ply direct opening, preview/redaction/copy fallback,
no automatic history import, repeated-save deduplication, reload and removal,
keyboard practice completion, source preservation, inherited-prefix re-share,
and played-statistics isolation. Shared visits with persisted signed-in Auth
perform no Firestore work. Public and library server reads only follow explicit
entry/save/manage/load actions.

The original broader runs caught stale assertions expecting branch completions
in played history and non-exact Profile headings. Scenarios were updated to
protect the new separate-practice contract while retaining real finish, offline
reload, recovery and second-client checks. Product fixes covered delayed
replacements, clear epochs, stale preview generations, pending saves/imports,
account changes, retained-copy review and old-client cloud deletion.

The primary visually inspected current shared mobile, library desktop, practice
desktop, and public management/publish/visitor captures; Luna inspected the guest
mobile/desktop set. These are actual candidate screens using disposable fixture
accounts. They retain the chosen existing visual language; they are not the
retired UI-study mocks.

| View | Desktop | Mobile |
|---|---|---|
| Shared replay | [capture](portable_replay_shared_desktop.png) | [capture](portable_replay_shared_mobile.png) |
| Saved library | [capture](portable_replay_library_desktop.png) | [capture](portable_replay_library_mobile.png) |
| Practice history | [capture](portable_replay_practice_desktop.png) | [capture](portable_replay_practice_mobile.png) |

Public mobile captures: [management](sharing-chromium-profile-manager.png),
[publish](sharing-chromium-publish-replay.png), [visitor](sharing-chromium-public-profile.png).
Corresponding Firefox/WebKit captures have their engine in the filename.

## Cost and compatibility boundaries

[Instrumented operation/size data](sharing_shape.json) records actual production
SDK calls against the emulator, with JSON fixture byte counts. A full public
page fixture is 66,500 bytes; a full private fixture is 533,610 bytes. These are
schema JSON sizes, excluding Firestore/index/protocol overhead. Two client reads
open public identity/page; a page write is two transaction reads plus one write;
visibility is five reads/five writes; whole sharing clear is six/six. Rules can
add dependent reads and retries cost more. No measured free-tier traffic capacity
or production billing result is claimed. [Cost contract](../../reference/ops/backend_cost.md)
contains the complete operation table and official pricing references.

Existing profile schema 5 and saved-match schema 2 remain compatible. The
[source receipt](compatibility.json) records byte-identical schema/history/
promotion sources against the pre-implementation baseline; executable rules and
application journeys establish their behavior separately.
Separate new storage families keep provenance out of old promotion/aggregation.
Rules reject old profile deletion if newer sharing data remains, plus stale
revision/generation writes and visibility changes without atomic page clearing.
[Rollout/rollback](../../reference/ops/release.md#v06-sharing-rollout-and-compatibility)
requires rules-first deployment, separately applied index exemptions, retained
empty markers and a compatible cleanup UI.

The original goal ended without a push, tag, public-host deployment, production
data mutation, real Google OAuth, physical mobile-device continuity or human
screen-reader acceptance. Bryan subsequently authorized pushing `0995af8`; its
full GitHub CI passed. Later local review fixes and remaining acceptance gates
are recorded in the newer report linked above.

Final cleanup: preview, cloud preview and Auth/Firestore emulator ports 8001,
8013, 18080, 18081 and 19099 are closed. The unintended external preview tab was
closed; unrelated user tabs were preserved. Routine preview startup no longer
opens the user’s browser. Final application/script/tools typecheck and whitespace/
relative-document-link checks pass after the test-output gating changes.
