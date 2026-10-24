# Gomoku2D v0.6.2 release record

Published October 24, 2026 as the direct successor to v0.6.1. Exact-source CI,
the tag workflows, Pages artifact verification, and automated production browser
acceptance passed. The inherited real signed-in sharing round trip remains
pending. Machine-readable results and raw-evidence hashes are in the
[release evidence](v0_6_2_release_evidence.json).

## Source and CI

- Tag [`v0.6.2`](https://github.com/byebyebryan/gomoku2d/releases/tag/v0.6.2)
  is annotated object `dde76c73d8e901ebfef7e22a52fa8f0d32fa50a0`, peeled to
  source commit `37a122d2bd8e899bcfc581055e685c0327f43cb4`.
- Exact-source [CI run 40130988854](https://github.com/byebyebryan/gomoku2d/actions/runs/40130988854)
  passed: 348 Rust workspace tests, 508 web units, 49 rules/SDK checks, 231
  ordinary browser passes with six intentional Firefox/WebKit WebGL skips,
  54 cloud checks, and 45 one-worker race repetitions. The six browser skips
  are the known WebGL qualification cases; forced Canvas runs in all engines
  and WebGL runs in Chromium.
- Fresh production npm and Rust audits found zero vulnerabilities; the Rust
  audit reported zero warnings. Dependency resolution did not change.
- The release check passed after clean main was pushed and exact-source CI
  succeeded. Candidate-local validation also passed typecheck, 508 web units,
  231 browser checks with six intentional skips, 33 selected emulator cloud
  checks on port offset 1000, production/manual builds, and candidate preview
  byte comparison. Candidate application and capture inputs match
  `bbc652f31fc4a211a85d1e60f61905617136f5a6`.

## Release workflows and Pages artifact

All tag workflows completed successfully on source commit
`37a122d2bd8e899bcfc581055e685c0327f43cb4`:

| Workflow | Run | Result |
|---|---:|---|
| GitHub Release | [40132511611](https://github.com/byebyebryan/gomoku2d/actions/runs/40132511611) | Success |
| GitHub Pages | [40132511700](https://github.com/byebyebryan/gomoku2d/actions/runs/40132511700) | Success |
| Firestore rules | [40132511653](https://github.com/byebyebryan/gomoku2d/actions/runs/40132511653) | Success |

Pages deployment [7304454931](https://github.com/byebyebryan/gomoku2d/actions/runs/40132511700)
serves [`gomoku2d.byebyebryan.com`](https://gomoku2d.byebyebryan.com/). Artifact
`12575268240` contains 147 files and has tar SHA-256
`464d74d289d61852839d69b8d48502c68c128842f81ecaac2a4edaf16dc5a9f1`. All 147
artifact files and 12 pretty static routes matched their served bytes (159
verified URLs total). The served index SHA-256 is
`d80146e76de59c7666ba667641395edfa2a4b080c739e6db830040212b880a74`; entry
`/assets/index-BiQoxxFR.js` SHA-256 is
`38a2bec00b38c09aedb05c2ebe44078597459abdbe606b8ea4327c525afa35ae`.

## Firestore compatibility

The pre-tag ruleset was
`projects/gomoku2d/rulesets/ec093a55-d063-48bd-a75b-19c2392bc1c3`. The tag
workflow advanced the release pointer to
`projects/gomoku2d/rulesets/8cea8731-6760-41ac-909f-8cddd989738b`; both use the
same reviewed `firestore.rules` SHA-256
`3c48346cebfefdd30068755bdcd08050b73146389bd53a3e772472980699d288`. This
release made no rules or index migration. All nine existing single-field
index exclusions were read back unchanged. There were no schema or dependency
changes, and the release work made no agent account/user-document writes. The
tag workflow only reapplied the identical rules bytes and created a new ruleset
identity.

## Production acceptance and preservation

The production browser smoke ran in fresh guest/local contexts against the
qualified deployed source: 108 passes, six intentional WebGL skips, zero
failures or flaky tests (114 scheduled). By engine: Chromium 38 passes;
Firefox 35 passes and three skips; WebKit 35 passes and three skips. The
journeys covered smoke, shared replay, replay library, profile, board colours,
and Guide examples. No account data or public-catalog writes were made.

The manual preview, Firebase Auth/Firestore emulators, and Carbon tunnel
remained active.
All five demo application-field hashes matched the recovery baseline; four
owned temporary ports were closed, port 8001 was left untouched, and no
user-browser tabs were opened.

## Open acceptance and recovery

The real signed-in public-profile/statistics/game-publication round trip remains
**pending**, as in v0.6.1; it is not newly deferred. No designated disposable
account/data was available. Emulator and guest production results do not close
this gate.

The final v0.6.2 help/example/confirmation/media visual review is not claimed as
complete. Physical phone/touch/browser-keyboard, human screen-reader/zoom,
novice-feedback, and two-device private account/settings/history/reset checks
also remain unclaimed and retain their existing deferral status.

The accepted pre-release web source was v0.6.1 deployment `7289818516`; the
compatible previous rules pointer is recorded above. For a web regression,
redeploy the accepted v0.6.1 tag and verify its source and served bytes. For a
rules regression, restore the recorded previous ruleset through the authorized
operational path and read it back. Do not move release tags or reset user data;
coordinate the web/rules pairing before rollback. See the
[release workflow](../reference/ops/release.md) for recovery details.

The roadmap schedules developer-site/process presentation for 0.6.3 and replay
collections/metadata for 0.6.4. Neither expands the 0.6.2 runtime scope. The
[candidate record](../working/v0_6_2_candidate.md),
[execution plan](../working/v0_6_2_release_plan.md), and
[v0.6.1 release record](v0_6_1_release.md) retain the design and acceptance
history.
