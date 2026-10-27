# v0.6.3 release plan — October 27, 2026

This plan prepares the reviewed 0.6.3 source for release. It records the
current production boundary and the checks required before treating the new
version as published. The [release workflow](../reference/ops/release.md) owns
the tag and deployment sequence; a dated archive record must own actual
publication results.

## State at release preparation

- The current production release is **v0.6.2**, published October 24, 2026.
  The 0.6.3 package metadata, changelog and this plan prepare a future release;
  they do not change production.
- Preparation began from clean local `main` at `77cf92d`. At that point local
  `main` was 47 commits ahead of `origin/main` at `4052d94`, and no remote
  `v0.6.3` tag existed. Recheck the branch, remote heads and tags immediately
  before the authorized push and tag steps.
- The final review accepted candidate `b4fdeb4` and implementation source
  `e34129f` for release preparation. It did not claim exact-source hosted CI,
  a 0.6.3 tag, deployment, or production acceptance.

## Frozen scope

- Publish the static developer area: Visual design, UI design and Bot lab;
  source-based inspection of 41 icons; eight design cases and two references;
  and UI history with 24 milestones, 15 screen families and 274 selected
  captures.
- Preserve its public static boundary. Developer pages do not initialize Auth,
  private player/history state, Phaser, Wasm or private cloud services. Shared
  replay and public player routes keep their existing authentication-aware
  boundary and are not part of this account-independent reference surface.
- Include the narrow Analysis layout correction, the Auth-loading guard and
  restored-identity wait for signed-in profile reset, the three-entry
  `body-parser`/`fast-uri` development lock refresh, and CI coverage for all 44
  maintained Node tool checks.
- Preserve existing report data and measurements, native code, game and
  historical media, backend schemas and rules, and release workflows. The
  reviewed retrospective captures and review media retain their tagged source
  and date. Do not recapture or modify approved media during release
  preparation, or add feature work.

## Review evidence and limits

The final review and release-fix records retain the following local evidence:

| Check | Recorded result and qualification |
|---|---|
| Application and tools | 558 unit tests; type checks, production build and publishers pass. |
| Node tool checks | 44/44 pass in Node 24. The final-review run used a fresh shallow source checkout with existing locked dependencies mounted read-only; it was not a new install or hosted CI run. |
| Browser and cloud | 297 ordinary browser passes with six intentional WebGL skips; 49 rules checks and 120 cloud/repetition checks pass. The broad lanes are retained source-qualified evidence, not a rerun in the final review. |
| Dependency audits | Production npm and Rust audits report zero findings. The full npm audit retains three moderate entries from the documented Firebase CLI/OpenTelemetry chain. |
| Build/media | The production-configured local build served 585 files / 22,833,908 bytes with matching hashes. All 488 inventoried developer-site payload files match the inspected portable preview; its source/date receipt differences are qualified in the review. |

These results support local readiness only. Hosted CI must pass on the exact
release-preparation commit, and deployment, served-byte parity and production
browser checks must be recorded after publication.

## Release gates

| Gate | Required evidence |
|---|---|
| Preparation review | Review only the assigned package/lock metadata, changelog and this plan alongside the primary-owned status updates. Confirm the package graph is unchanged and `git diff --check` passes. |
| Push and CI | Push the reviewed clean `main` as authorized. Require successful CI with `headSha` exactly equal to the release-preparation commit, including the fresh Node 24 dependency install and Wasm build. Resolve failures before proceeding. |
| Release check and tag | Run `scripts/release.sh --check 0.6.3` from the clean, pushed source after exact-SHA CI passes. Then create the annotated tag with the release script and push `v0.6.3`. |
| Tag workflows | Require GitHub Release, Pages and Firestore workflows to succeed on the tag source. Record the Pages deployment/run and artifact identity, plus the previous and current Firestore ruleset IDs and source. Rules and indexes are not intended to change; investigate and stop on unexpected drift. |
| Production verification | Compare the deployed artifact and public served bytes, then exercise the game and direct developer routes, reload/history behavior, selected review exports and public static/privacy boundary. Record exact source, routes, browser results and any failures. |
| Closeout | Add a dated archive release record and evidence receipt. Update current-status pointers to separate the published 0.6.3 state from this preparation snapshot and its remaining manual acceptance. |

Do not treat a pushed commit, successful tag creation or workflow dispatch alone
as publication. All required workflows and deployed-artifact checks must pass
before recording 0.6.3 as current production.

## Manual acceptance that remains open

Automation and local review do not close these inherited checks:

- The real signed-in public-profile/statistics/game-publication round trip
  inherited from v0.6.2 remains pending. Two-device private settings/history/
  reset checks also remain unperformed; emulator and guest checks do not close
  either gate. Use disposable test data if these checks are later performed.
- Physical lower-powered/mobile performance and touch comfort, actual browser
  zoom, human screen-reader use and novice/human visual feedback remain
  unverified. Responsive and automated accessibility checks are not physical
  device or human assistive-technology acceptance.
- Manual contrast judgment for layered sprite/board content and offscreen
  examples remains inconclusive, not a pass.

Carry these limits into the release decision and release record. The detailed
manual deferrals are in the [October acceptance record](../archive/reconciliation_2026_10/README.md#deferred-acceptance)
and the [v0.6.2 release record](../archive/v0_6_2_release.md#open-acceptance-and-recovery).
