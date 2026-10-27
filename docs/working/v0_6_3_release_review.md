# 0.6.3 release-readiness review — October 27, 2026

Reviewed `98d60f5` on clean local `main` against published `v0.6.2` (`4052d94`).
This is a review of the whole release, including the public developer boundary,
game/cloud regression, source and capture provenance, docs, publishing inputs,
dependencies and release procedure. Product, test, lockfile and workflow source
are unchanged by this review. The [validation receipt](v0_6_3_release_review/validation.json)
qualifies fresh checks and reused evidence separately.

**Follow-up, October 27:** RR1/RR2 and the RR3 CI improvement are resolved by the
[release-fix checkpoint](v0_6_3_release_fixes.md), tested at `e34129f`. The full
cloud and repetition gates now pass. The findings and results below remain the
original `98d60f5` review snapshot, including its retained failures.

**Recommendation: finish the two bounded items below, then prepare the release.**
The dev-site scope and presentation are coherent, and the previous five review
findings remain resolved. The broad cloud gate is currently red; this should be
corrected before relying on exact-source CI or tagging.

## Findings

| ID | Priority | Finding and next action |
|---|---|---|
| RR1 | P2 · validation/readiness | **C14 races persisted sign-in at both destructive-action entry points.** The full cloud lane passes 59/60; Firefox fails before the private-reset confirmation. A five-per-engine C14 rerun passes 14/15; WebKit fails before Delete Cloud. Both traces show Reset being clicked while cloud identity is still checking, followed by the local-only confirmation. The account-change effect then clears the confirmation as the persisted user restores. `cloudtests/sharing.spec.ts:550` and `:572` navigate to Profile without waiting for restored sign-in before clicking; `ProfileRoute.tsx:300` intentionally clears confirmation on UID change, while `:433` does not disable the trigger during Auth loading. Make signed-in test readiness explicit at both boundaries; separately consider preventing the misleading early-click affordance. Keep the account-change safeguard. Rerun focused repetitions and the full cloud lane before closing this item. |
| RR2 | P2 · dependency maintenance | **The development audit has actionable additions beyond the recorded exception.** The lock resolves `body-parser` 1.20.8 and a nested 2.3.0, plus `fast-uri` 3.1.8; all are development dependencies. Audit reports a high-severity body-parser DoS and moderate/low fast-uri advisories, with compatible patched lines available. Update the affected lock resolutions narrowly and verify the emulator/build tooling. Production npm and Rust audits remain clean. These dependencies and the C14/Profile files are unchanged from v0.6.2; this pass surfaced the issues rather than demonstrating a new gameplay or backend regression. |

RR1 fails before the destructive confirmation in each affected phase. The
Firefox run never confirms Reset; WebKit's focused failure follows a completed
private-reset check but never reaches Delete Cloud. This is not evidence of
private/public data loss. Passing reruns are retained alongside failures; they
do not turn the full failed lane into a pass.

The body-parser patches are 1.20.9 and 2.4.0; the affected fast-uri 3.x line is
patched in 3.1.9. See the maintainer advisories for
[body-parser](https://github.com/advisories/GHSA-27r3-jxmh-p828),
[fast-uri scheme decoding](https://github.com/advisories/GHSA-w74x-v6f9-j3xg),
[relative normalization](https://github.com/advisories/GHSA-xhq7-xmqj-769h),
[host canonicalization](https://github.com/advisories/GHSA-wjr9-j795-jxcr), and
[suffix references](https://github.com/advisories/GHSA-333x-w4r6-6crp).
The full audit counts five affected package entries: one high and four moderate,
including the three pre-existing OpenTelemetry/Firebase CLI parent entries.
These counts are not five independent production vulnerabilities.

## Whole-release assessment

- **Scope:** one shared Dev site on the existing package/domain. Visual design,
  UI design and Bot lab have consistent navigation; Current screens is the UI
  entry. Multiplayer, replay collections and narrative journey articles stay
  outside 0.6.3.
- **Useful process presentation:** 41 source icons and sprite/control inspectors,
  eight design cases plus two reference/comparison records, and 24 milestones
  across 15 screen families. The history uses 137 snapshots and 274 selected
  captures. Introductions/removals, dated recaptures and original evidence remain
  distinct. The screen reference is explicitly the released 0.6.2 baseline.
- **Boundaries:** lazy developer entry, static curated inputs and export viewers
  stay independent of Auth/private initialization and the game engine. The full
  ordinary and cloud runs cover both direct public entry and game/account return;
  D01/D02 pass in all engines. No new backend, rules/index/schema, engine,
  dependency-lock or deployment-workflow change is included.
- **Provenance/publication:** publishers verify the catalog/manifest/image graph.
  All 151 pinned source links resolve to local committed objects and paths. All
  31 developer direct-entry files actually exist, separately from preview-server
  fallback. Recorded Lab JSON and the selected assets/media match source. The
  built tree contains no unexpected private output paths or internal preview-host
  dependencies in its static data. Release publication still requires remote
  link/artifact checks after pushing and deployment.
- **Presentation:** the corrected Analysis layout and contextual capture names
  remain covered by the full browser run. The immediately preceding 60-view and
  66-view audits are still source-identical; their inconclusive contrast results
  remain explicit, alongside generic/unmeasured portrait-capture limitations.

## Validation

| Check | Result and qualification |
|---|---|
| Rust formatting, Clippy and workspace tests | Fresh pass; 348 tests across 16 targets, no ignored tests. Engine source is unchanged. |
| Production dependency audits | Fresh npm production audit: zero findings. Cargo audit: zero vulnerabilities/warnings, database `3279c99911354c400e1d41e47c7d60e39258b147`. |
| Full npm audit | Five development-only affected entries; RR2 above. Installed dependency tree validates with `npm ls --all`. No dependency install/update performed. |
| Production-configured build | Fresh build/publishers using the checked-in Pages workflow's public Firebase configuration, base `/`, owned output directory. Existing manual `dist` preview preserved. |
| Full ordinary browser lane | Fresh **297 passed, six intentional skips**, across Chromium/Firefox/WebKit. Skips are non-Chromium WebGL colour qualification; Canvas coverage runs in all engines. |
| Rules | Fresh **49 passed** on an isolated demo-only emulator, with separate ports and config. |
| Full cloud lane | Fresh **59 passed, one failed** (Firefox C14). D01/D02 pass in all engines. |
| C14 five-per-engine repetition | Fresh **14 passed, one failed** (WebKit at the second Profile entry). Original failing artifacts preserved separately. |
| Types, unit and tool checks | Prior **556 unit / 44 Node** checks and app/scripts/tools types remain valid: implementation inputs are identical to `c1c3af6`; `98d60f5` only closes documentation/evidence. Not presented as reruns. |
| UI/accessibility/export | Reused source-identical fix checkpoint: 60-view Chromium and 66-view three-engine audits have zero violations; seven/nine overlapping contrast checks remain inconclusive. Standalone/full export, actual-prefix controls, history return and all-file delivery from Snap remain recorded separately. |
| Served build/source bindings | Fresh **585 files / 22,833,886 bytes** match SHA-256; 76 review-media bindings, 31 developer entries, unchanged recorded Lab inputs and publisher-validated history pass. |

The normal repeated C4/C6/C7 race gate is left for the corrected final candidate;
it is not recorded as passing here. No Rust tournaments, new capture campaign,
fresh dependency installation, real-account writes or production mutations were
needed for this review.

## Release preparation still required

1. Resolve RR1/RR2, retain the existing compatibility/acceptance limits, and run
   the affected checks plus the final ordinary/cloud/race gate on that candidate.
2. Set package and lock metadata to 0.6.3; write a dated changelog section and
   comparison links. Refresh README/status/release records for the final scope.
   The current package remains 0.6.2 and Unreleased is empty.
3. Push only after authorization, then require successful CI on the exact
   preparation SHA, including its fresh Wasm build and `npm ci`.
   Remote `main` and latest successful CI still identify `4052d94`.
4. Run `scripts/release.sh --check 0.6.3`, then follow the normal separately
   authorized tag/publication sequence and deployed-byte/browser verification.
   The current read-only check correctly stops at unpushed main; it does not
   reach the later metadata gates. No 0.6.3 tag exists remotely.

Human phone/touch, screen-reader, real signed-in sharing and two-device checks
retain their documented open/deferred status. This release does not silently
close those inherited checks or require a new backend rollout. The tag-triggered
Firestore workflow still runs; verify its unchanged rules source and release
pointer as part of normal publication.

## Non-blocking improvement

CI's Vitest command only includes `src/**/*.test.*`; the 44 Node capture/export/
audit-helper checks are currently manual and absent from the workflow. Add the
existing Node command as a small CI step when practical. This is a coverage
improvement, separate from the observed RR1 failure and dependency refresh.

## Repository and resource handoff

The review changes documentation/evidence only, committed locally. No push,
version bump, tag or release occurred. Owned preview 47438, offset-5000 cloud
services and isolated rules services are stopped after evidence collection.
Existing preview 46862 and shared report host 8787 remain available. No user tabs
or worktrees were opened/removed. Raw artifacts remain in the ignored
`gomoku-web/test-results/v063-release-review-20261027/` directory.
