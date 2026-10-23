# Gomoku2D v0.6.1 release record

Published October 23, 2026 as the direct successor to `v0.5.5`; there was no
intermediate `v0.6.0`. The release is deployed and its automated production
acceptance passed. One real signed-in public-sharing round trip remains
pending. This record summarizes the release; the
[machine-readable evidence](v0_6_1_release_evidence.json) contains the served
file manifest, field readbacks, browser artifacts and raw-evidence hashes.

## Source and CI

- Tag: [`v0.6.1`](https://github.com/byebyebryan/gomoku2d/releases/tag/v0.6.1),
  annotated tag object `a053bc552d7e7560f57d9f02f550d23d68fe7391`, peeled to
  source commit `b1a67d32feee5b57275e3d04c684e27bcd1a571f`.
- Exact-source CI
  [40027552184](https://github.com/byebyebryan/gomoku2d/actions/runs/40027552184)
  passed: 348 Rust workspace tests, 493 web units, 49 rules/SDK checks, 219
  ordinary browser passes and six intentional WebGL skips, 42 cloud checks and
  45 configured one-worker race repetitions. The six skips are the WebGL pixel
  cases in Firefox and WebKit; Chromium's three WebGL pixel cases ran. A
  separate local [integrated review](../working/v0_6_1_evidence/wrap_review.json)
  recorded 45 three-worker race repetitions.
- Initial CI
  [40026924846](https://github.com/byebyebryan/gomoku2d/actions/runs/40026924846)
  at `789d5b1` failed a single unit assertion that expected a Pacific-local
  snapshot date at midnight UTC. The fixture assertion was made timezone-
  portable; it passed 14/14 under UTC and 14/14 under
  `America/Los_Angeles` before the final CI run. This was a test expectation,
  not an application defect.
- Production npm and Rust advisory audits reported zero vulnerabilities; the
  Rust audit reported zero warnings. The full npm audit retained three moderate
  dev-only affected entries through `firebase-tools` → `@google-cloud/pubsub` →
  `@opentelemetry/core` (`GHSA-8988-4f7v-96qf`). No dependency resolution
  changed.

## Publication and served artifact

All tag workflows succeeded at the release source:

| Workflow | Run | Result |
|---|---:|---|
| GitHub Release | [40031565001](https://github.com/byebyebryan/gomoku2d/actions/runs/40031565001) | Success |
| Firestore rules | [40031565082](https://github.com/byebyebryan/gomoku2d/actions/runs/40031565082) | Success |
| GitHub Pages | [40031565071](https://github.com/byebyebryan/gomoku2d/actions/runs/40031565071) | Success |

Pages deployment `7289818516` serves
[`gomoku2d.byebyebryan.com`](https://gomoku2d.byebyebryan.com/). The live Home
shows `v0.6.1` with no page errors. All 89 selected served files matched the
deployment artifact, including all 12 verified direct/static routes. The
selected set includes the index and entry script, reports, public routes,
assets and fonts. Artifact `12534483135` contains 144 files and has archive
SHA-256 `9a313620517a189da4c2e26e8bb9a2aa5e96c1621dadce27fa383826250ad69a`.
The served index SHA-256 is
`5b502c156b399a9db66ae82948f9181d66f7d94472ba2088e389a6565aaab8c4`; entry
`/assets/index-CcWy7q8r.js` SHA-256 is
`d60a151c7d3f48100208da77de13b34f57147e38e78247bc932ff900f1ef708d`.

The production browser smoke ran the reviewed smoke, shared replay, replay
library, profile and board-colour journeys in fresh Chromium, Firefox and
WebKit contexts against the source-qualified deployed host. It passed 96 checks
with six intentional Firefox/WebKit WebGL pixel skips and zero failures (102
scheduled). It covered guest/local play, replay, profile, saving and
continuation, direct routes and actual Canvas colours in all engines; Chromium
also exercised WebGL. It did not perform the signed-in publication lifecycle.

## Firestore rules and indexes

The pre-tag rules workflow
[40029725502](https://github.com/byebyebryan/gomoku2d/actions/runs/40029725502)
succeeded. The pre-tag ruleset was
`projects/gomoku2d/rulesets/dccb94b6-7dc7-41a5-bdf2-d9c807fb07f9`; after the
tag-triggered workflow, the live ruleset was
`projects/gomoku2d/rulesets/ec093a55-d063-48bd-a75b-19c2392bc1c3`. Both match
the reviewed `firestore.rules` SHA-256
`3c48346cebfefdd30068755bdcd08050b73146389bd53a3e772472980699d288`. The
earlier `v0.5.5` ruleset was
`projects/gomoku2d/rulesets/bb6ecb9e-0c7d-4e31-b601-82f95799b3b6`.

Five single-field exemptions were applied and read back in `(default)`:

| Collection group | Field |
|---|---|
| `public_profiles` | `display_name` |
| `public_profiles` | `shared_stats` |
| `public_profiles` | `catalog_page_counts` |
| `pages` | `entries` |
| `replay_libraries` | `entries_json` |

The four existing `profiles.match_history` index settings for the parent,
`replay_matches`, `summary_matches` and `archived_stats` remained unchanged.
The five field requests were issued sequentially; their background operations
were jointly awaited, and all five completed successfully before the release
tag. Production configuration changes were limited to the reviewed rules
deployment and these five scoped exemptions. No user documents were written;
no new services, IAM or billing settings were changed.

## Open acceptance and retained deferrals

The real signed-in sharing round trip remains pending; it has not been recorded
as passed or explicitly deferred, and no designated test account/data is
available. The remaining action is to publish synthetic game and statistics
data from a disposable account, verify the signed-out public view, then disable
sharing and confirm cleanup. Earlier production Google sign-ins and the
reported cleared OAuth warning do not close this gate.

Physical-device/mobile performance and touch comfort, human screen-reader and
browser-zoom acceptance, the two-device private account/settings/history/reset
round trip, and novice feedback remain deferred. These were not converted to
automated passes. SDK operation counts do not establish billing totals or a
free-tier traffic guarantee.

See the [release plan](../working/v0_6_1_release_plan.md) for scope, the
[acceptance evidence ledger](../working/v0_6_1_evidence/README.md) for
historical candidate results, and the [release workflow](../reference/ops/release.md)
for rollback and compatibility guidance.
