# 0.5.5 Release Preparation

Prepared on October 16, 2026. This is the final patch packaging the reopened
`0.5.x` reconciliation. Tagging and production deployment are separate from
this preparation record.

## Source and Packaging

- The Git attribution rewrite was reconciled from the old checkout to
  `8a5cc854b1e6dee289653bd30259ddc57134ea80` after verifying identical file trees.
  Its [full CI](https://github.com/byebyebryan/gomoku2d/actions/runs/38961095241)
  passed. Existing historical measurement SHAs and evidence bytes remain intact.
- Package and lockfile versions are `0.5.5`; dependency resolutions are unchanged.
- `CHANGELOG.md` has a dated `0.5.5` release section, empty `[Unreleased]`, and
  compare links ready for `v0.5.5`.
- README GIFs, Replay Analysis still, and OG assets were captured automatically
  from a production preview whose application/version inputs match source
  `f300a30c7bda13738010637c700e67ec8aaa0670`. The capture viewport is 1280×800
  to include the full replay controls.
  See [media provenance](../assets/README.md).
- The June Lab reports retain their committed data, clean historical provenance,
  and original source identities. No tournament/report regeneration is needed
  for this packaging pass.

## Final Documentation and Repository Review

The final pass before tagging checked the root/module READMEs, documentation navigation,
current reference paths, release/media instructions, and GitHub description,
homepage, topics, license and CI badge. It corrected:

- verification commands tied to one workstation and duplicated command guidance;
- the missing command to push the release tag;
- OG capture provenance and the final build needed after media regeneration;
- stale corpus paths and the PR template's older verification checklist;
- public-doc navigation and the distinction between historical Lab data,
  editorial drafts and current release evidence.

Validation covered 89 tracked Markdown files, 269 local paths/anchors and 11
public navigation, badge and release URLs, with no broken links found. This
pass comprises Markdown and template changes; the existing media source identity
above still describes the captured application/version inputs.

## Validation and Acceptance

The completed [unattended review](reconciliation_2026_10/v0_5_unattended_evidence.md)
records 348 native tests, 378 web unit tests, 27 real-rules tests, 168 ordinary
browser journeys, 21 cloud journeys and 45 fresh-state race repetitions across
three engines. Its broader desktop Wasm matrix covers legal bot requests,
uncached/cached analysis, active cancellation/replacement and resource cleanup.
These remain the substantive engineering acceptance for this patch.

Fresh release audits on October 16 passed: production npm vulnerabilities zero;
Rust vulnerabilities zero, with no informational warnings. The existing
Firebase CLI development-only exceptions remain in
[dependency maintenance](../working/dependency_maintenance.md). Local production
build/typecheck and automated media capture exercise the versioned candidate.
The release gate is successful CI on the exact pushed preparation commit plus
`scripts/release.sh --check 0.5.5` from clean, pushed `main`.

Real Google OAuth, physical-device/touch performance, actual screen-reader/zoom
use and human feedback retain Bryan's explicit deferrals. Emulator/desktop
results do not count as those passes, and no new manual checks are added here.

The working checkout's 87 local UI-study files were preserved. Release checks
use the clean checkout at `/home/bryan/code/gomoku2d-release-0.5.5`; scratch
capture frames and raw audit outputs stay outside the reviewed release files.

## Publication Handoff

Use the [release workflow](../reference/ops/release.md) after the exact preparation
commit is green and the read-only preflight passes. Tag `v0.5.5` only then;
pushing it dispatches GitHub Release, Pages and Firestore rules workflows.
Record the deployed source SHA, Pages result and previous/current ruleset IDs,
then run the automated public-origin journeys and direct-route checks. Carry
account/device/human deferrals forward with the published acceptance record.
