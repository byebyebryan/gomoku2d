# Gomoku2D 0.6.4 implementation checkpoint

Local candidate based on `f94dde6`. The published app is still 0.6.3.
[Approved UI review](v0_6_4_replay_collection_review.md) ·
[Design and data contract](v0_6_4_replay_collection_contract.md)

## What players can do

- Recognize positions in Saved replays and public profiles by their board
  thumbnail. A thumbnail shows the same opening as Replay, including move 0.
- Open a saved replay, then choose **Edit details** in Replay Analysis. Set a
  private title, note and opening. Move the timeline to inspect positions.
  Select **Use move N** to stage an opening, then **Save details** to keep it.
  The board stays at its current position and the new opening applies when
  the replay is reopened. Returning to the library focuses the originating row.
- Open **Edit** from Profile → Public settings → Published games. The same
  Replay Analysis screen provides a public title and opening editor. A saved
  edit keeps the row in place and existing copied links keep their original ply.
- Compare conflicting private details before merging or copying guest saves
  into an account. Choose one complete set of details per conflicting replay.
  A reviewed merge or import checks the local revision and size before sending
  its choices to Firestore. The device applies them after Firestore accepts
  the reviewed cloud revision. If that revision changed, reload to compare
  the original device copy with the latest cloud copy. If the device changes
  during a successful cloud write, the next review includes its newer work.
  After a cloud clear, review retained device copies before restoring them.

## Compatibility and operating cost

Portable replay links and fingerprints still use v1. Private details travel only
with the private library. The local cache uses a separate v2 key and serializes
durable edits with an origin-scoped Web Lock. When persistence is unavailable,
the UI labels session copies and reports cloud confirmation separately. Cloud
storage uses the existing owner-scoped document with a monotonic revision and
an empty marker after cleanup. The 64-private and 32-public limits still apply.

The browser draws thumbnails from data already loaded for the list. It uploads
zero images and makes zero additional Firestore reads for thumbnail rendering.
An explicit public edit updates the existing root and catalog page in one
transaction. The implementation adds no hosted service or dependency.

## Local verification

| Gate | Result |
|---|---|
| Web typecheck and production build | Passed, including the static route and developer-site publishers |
| Web unit tests and tool checks | 582 unit tests passed after the merge fix. The original candidate passed 44 tool tests |
| Firestore rules and API tests | 51 passed against an isolated emulator |
| Full browser suite | 306 passed across Chromium, Firefox and WebKit. Six engine-specific WebGL pixel checks were skipped by design. All three engines ran forced Canvas checks |
| Replay and profile browser rerun | 51 passed across all three engines before the merge fix. All 18 replay-library checks passed on the fixed candidate |
| Emulator journeys after the merge fix | 69 passed across Chromium, Firefox and WebKit against isolated Auth and Firestore emulators, including the C22 stale-review regression in each engine |
| Repeated emulator races | 60 passed across all three engines, five repetitions each for stale writes, account switch, reset and competing private details |

The browser scenarios cover a 64-replay v1 migration, a full 32-publication
catalog, a stale same-origin tab, denied persistence, private metadata excluded
from copied links, exact Black and White thumbnail pixels, a public move 0
edit and the original copied link, and explicit review of competing private
details. The stale-merge regression checks that a rejected review preserves the
original device title, note and opening and offers them in the next review.
Automated accessibility checks ran on the affected replay and profile
surfaces. Browser captures and traces are under the local ignored directory
`gomoku-web/test-results/v064-implementation/`.

## Release handoff

The tag starts Pages and Firestore workflows independently. For this release,
verify the additive rules from the exact release commit before pushing its tag.

1. Prepare the release commit on `main`, push it and wait for CI on that SHA.
2. Dispatch `gh workflow run deploy-firestore.yml --ref main`. Find the run
   with `gh run list --workflow deploy-firestore.yml --event workflow_dispatch --branch main --commit <release-sha>`
   and wait for it with `gh run watch <run-id> --exit-status`.
3. Record the source SHA, previous ruleset and deployed ruleset from the run
   summary. Verify the SHA matches the release commit and the live
   `cloud.firestore` release points to the deployed ruleset. Stop before
   tagging if the run or readback disagrees.
4. Run `scripts/release.sh --check 0.6.4`, then create and push the release tag
   from the verified commit. Wait for the Pages and Firestore tag workflows.
   Verify the served build and read the live ruleset again. The tag workflow
   may create a new ruleset ID from the same source bytes.

Keep the additive rules during a frontend rollback. Once an account has a v2
library, its reader and cleanup path require a v2-compatible build.

Physical Android and iOS browser checks, real Google sign-in and a human
screen-reader pass remain open. For manual review, pay particular attention to
the sidebar with the phone keyboard open, the unsaved-draft prompt, the focus
and scroll when returning to a library row, and the cloud conflict choices.
