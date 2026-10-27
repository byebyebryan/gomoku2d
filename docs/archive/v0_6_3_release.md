# Gomoku2D v0.6.3 release record

Published October 27, 2026 as the successor to v0.6.2. Exact-source CI, all
three tag workflows, complete served-artifact parity and automated production
acceptance pass. The [Dev site](https://gomoku2d.byebyebryan.com/dev/) is now
public. Real signed-in sharing and inherited human/device checks remain open.

The [evidence receipt](v0_6_3_release_evidence.json) retains source, workflow,
backend, audit, provenance and browser results with raw-evidence hashes. The
[served manifest](v0_6_3_release_manifest.json) retains every checked path and
both artifact and served SHA-256 values. The
[release plan](../working/v0_6_3_release_plan.md) is the executed preparation snapshot.

## Source and CI

- Annotated tag [v0.6.3](https://github.com/byebyebryan/gomoku2d/releases/tag/v0.6.3)
  is object `f0897e92a76f67293959fca5d99b13da21cc52f7`, peeled to
  `5981bda1a34abd07790425b6c5ec48705c8005e7`.
- [CI 40573575015](https://github.com/byebyebryan/gomoku2d/actions/runs/40573575015)
  passes on that exact source: 348 Rust workspace tests, 558 web units, 44 Node
  tool checks, 49 rules checks, 297 ordinary browser passes, 60 cloud journeys
  and 45 repeated conflict/account/reset races. Types, lint, fresh Node-24
  installation, Wasm and production build also pass.
- Six ordinary browser skips are intentional Firefox/WebKit WebGL
  qualifications. Forced Canvas passes in all engines; WebGL is qualified in
  Chromium. No failed or flaky result is treated as a pass.
- Production npm and Cargo audits are clean. The full npm audit retains only
  three moderate affected-package entries in the documented Firebase CLI /
  OpenTelemetry development-tool exception. Narrow body-parser/fast-uri
  patches are included; release packaging changes only version metadata.

## What shipped

The static developer area brings together Visual design, UI design and Bot lab:
41 canonical icon sources with native/enlarged pixel inspection, sprite and
style references, eight design cases and two reference/comparison records, and
UI history covering 24 meaningful milestones and 15 screen families with 274
selected captures. Its default UI design entry is Current screens; historical
media retains its actual release/capture identity.

The release includes the narrow Analysis reflow and profile Auth-readiness
fixes, safe standalone export destinations, complete lazy-image audit handling
and maintained capture/export helper coverage in CI. Native code, backend
schemas/rules, portable replay format and recorded Lab data are unchanged.
The developer area adds no cloud service or production data migration.

Luna's independent provenance audit validates local catalog/media/manifests and
175 published GitHub objects, with 25/25 historical release tags matching their
recorded commits. Two supporting pilot links use mutable `main`; they resolve
at the checked source, while underlying release identities remain pinned.
Retrospective reconstructions are labelled with their modern toolchain,
synthetic state and capture date, rather than presented as original binaries
or original-period acceptance.

## Publication and production checks

All tag workflows succeed at `5981bda`:

- [GitHub Release](https://github.com/byebyebryan/gomoku2d/actions/runs/40577585699)
- [Pages](https://github.com/byebyebryan/gomoku2d/actions/runs/40577585755)
- [Firestore rules](https://github.com/byebyebryan/gomoku2d/actions/runs/40577585712)

Pages deployment `7369495285` serves artifact `12766719431`: **585 files /
22,840,116 bytes**, plus 41 direct-entry route checks. All **626 served URLs**
match the exact artifact on the first verification pass. This covers scripts,
Wasm, styles, fonts, canonical assets, review/history media, reports and static
route entries.

Production browser verification passes **168 cases**, with six intentional
WebGL skips and zero failures/flaky results, across Chromium, Firefox and
WebKit. Ten spec files cover developer navigation, matched-scale comparisons,
icon pixels, sprites, history/return, narrow Analysis, guest Profile, example
replays, sharing/saving/continuations and local game flows. Fresh contexts and
local fixtures make no account or public-catalog writes.

A separate production-origin check visits all **31 developer/history entries
in each engine: 93 visits**. It confirms zero private storage reads, IndexedDB
opens, player/engine/private chunks, external requests or page errors. Each
engine separately confirms visible `v0.6.3` on Home. Persisted signed-in
continuity and pending-write navigation are covered by the exact-source
emulator lane; this does not claim a real-account round trip.

The original browser-wrapper observation handle disappeared at a goal
continuation. The same container finished without a restart. Its complete
Playwright JSON/log and Docker's retained exit-0 event agree; the raw receipt
records that recovery separately from the privacy check.

## Backend and recovery

The existing tag workflow reapplies identical rules bytes:
`3c48346cebfefdd30068755bdcd08050b73146389bd53a3e772472980699d288`.
The previous ruleset was
`projects/gomoku2d/rulesets/8cea8731-6760-41ac-909f-8cddd989738b`; the deployed
and independently read-back ruleset is
`projects/gomoku2d/rulesets/4391d371-1145-44b1-8d42-58d555ad2ee3`.
All nine configured index fields match their before-release readbacks. No
manual rules/index migration or production account write was performed.

For a web regression, dispatch Pages at accepted `v0.6.2` / `37a122d`, then
verify its deployment and served bytes. Do not move release tags. For a rules
incident, restore and read back the recorded previous release pointer through
the authenticated operational path. No rollback was performed.

## Remaining acceptance and handoff

- The real signed-in public-profile/statistics/game-publication round trip
  remains pending. Emulator and guest results do not close it.
- Physical phone/touch/lower-powered performance, native browser zoom, human
  screen-reader use, two-device private settings/history/reset and novice
  feedback retain their previous pending/deferred limits. Generic portrait
  references are not measured OnePlus acceptance.
- Inconclusive contrast around layered content/offscreen examples still needs
  manual judgment. Automated reflow/pixel checks do not close human acceptance.

Existing preview/report listeners and the foreign worktree are preserved.
Owned browser containers are removed; no temporary servers, emulators or
worktrees were created, and no user-browser tabs were opened. The manual
preview was not rebuilt or relabelled as production.

Luna handled bounded release preparation and provenance validation. The primary
owned scope/copy review, integration, audits and live baselines, publication,
artifact/backend/browser verification and acceptance. The next planned work is
0.6.4 replay collection/metadata design; curated learning and multiplayer remain
outside this release.
