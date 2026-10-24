# v0.6.2 release execution

Owner: Bryan. Authorized October 24, 2026 with the Luna goal loop after the
[developer-site direction](v0_6_3_developer_site_plan.md) was committed as
`028de0c`. This authorization includes reviewed main pushes, the v0.6.2 tag,
existing release workflows and deployed verification. No 0.6.3 implementation.

## Frozen product scope

Release the existing [0.6.2 candidate](v0_6_2_candidate.md): public-player/mobile
refinements, return navigation, Guide help/two legal static examples, explicit
enable confirmation and focused documentation/media. Application/version/media
inputs match the validated capture at `bbc652f`; later changes are documentation.
No Rust, rules/index, schema, dependency graph or replay-format change.

The roadmap now schedules the developer site/process presentation for 0.6.3,
replay collections for 0.6.4 and curated learning later. These plans do not add
runtime work to this release.

## Gates and ownership

| Gate | Work and evidence |
|---|---|
| Independent readiness | Luna checks source/evidence, changed UI contracts and release pitfalls; primary resolves material findings and accepts the result. |
| Local baseline | Verify clean main, version/lock/changelog/report gates and validated application/media identities. Refresh production npm/Rust audits; use existing applicable browser/cloud evidence. |
| Live baseline | Read existing Pages deployment, Firestore release/rules bytes and all nine index exemptions. Record rollback identity; no user-document or index changes. |
| Push and CI | Push reviewed clean main and require all CI jobs to succeed on the exact release SHA. Inspect complete web/rules/cloud/race and Rust results. Fix and rerun only if required. |
| Tag | Confirm deployed backend compatibility with unchanged reviewed source, run the normal release check, create annotated v0.6.2 and push that tag. |
| Publication | Require GitHub Release, Pages and Firestore workflows to succeed on the tag source. Verify deployment identity, artifact/served bytes and rules readback; index configuration stays unchanged. |
| Production acceptance | Run production-safe guest/local browser journeys in Chromium/Firefox/WebKit, including new Guide examples and existing save/continuation/colour cases. Keep real-account and physical/human acceptance separate. |
| Closeout | Commit/push a dated release record and machine-readable evidence, update current-status docs and retain the candidate as historical evidence. |

The current rules/indexes are unchanged from v0.6.1. A successful live source
and index readback establishes the pre-tag compatibility checkpoint; there is
no rules-first migration or index application to repeat. The existing tag
workflow will reapply identical rules bytes and create a new ruleset identity.
Record the before/after pointers and verify both sources match. Do not modify
backend configuration to make a UI/content release appear more complete.

## Acceptance boundaries

Bryan approved the profile direction and gave a broadly positive earlier mobile
pass. This release authorization does not claim an unperformed final visual
pass, physical phone/touch/browser keyboard, screen-reader/zoom, novice or
two-device private-account check. Carry the existing deferrals honestly.

The inherited real signed-in profile/statistics/game-publication round trip
remains pending, with no designated disposable account/data available. Keep it
explicitly pending in the release record; emulator and guest production results
do not close it. Do not operate on an ordinary user's production documents.

## Preservation and recovery

Preserve the manual preview, emulators, Carbon forwarding and demo fields.
Local cloud harnesses always use `GOMOKU_CLOUD_PORT_OFFSET=1000`; do not claim
port 8001 or open user-browser tabs for routine automation. Put raw evidence
under ignored `gomoku-bot-lab/outputs/release-prep/0.6.2/` and browser artifacts
under ignored `gomoku-web/test-results/v062-release/`.

Stop before tagging on a material correctness/privacy regression, failed CI,
unexpected backend drift or conflicting remote history. Resolve ordinary test
or implementation defects within scope. For a deployed regression, follow
[Release Workflow](../reference/ops/release.md#recovery) using the accepted
v0.6.1 Pages source and recorded compatible rules pointer; never move tags or
reset user data. A transient build/CDN delay is incomplete publication evidence,
not permission to bypass the gate.
