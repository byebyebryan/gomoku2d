# Gomoku2D v0.6.4 release record

Published November 18, 2026. This release completes the planned 0.6 line.
Saved replays and public listings show board thumbnails at their chosen
opening. Players can give a saved replay a private title, note and opening.
Publishers can edit a listing in place while previously copied links retain
their original opening. [Play](https://gomoku2d.byebyebryan.com/) ·
[GitHub release](https://github.com/byebyebryan/gomoku2d/releases/tag/v0.6.4)

The Dev site records the replay editors, board previews and conflict flow in
seven frozen captures from implementation commit `ffec605`. Current screens
retains its labeled 0.6.2 reference set.
[Replay collections review](https://gomoku2d.byebyebryan.com/dev/reviews/replay-collections-064/) ·
[Capture record](../working/v0_6_4_ui_evidence/README.md)

## Source and verification

- Annotated tag `v0.6.4` is object `d531df4efc7e6b777cefd719ee3c9398063757f9`
  and peels to `48822e22f3ee7e5f32753e511ab83e10528e0d59`.
- CI passed on that commit with 348 Rust tests, 582 web unit tests, 44 tool
  checks, 51 Firestore rules checks, 306 browser passes, 69 Auth/Firestore
  emulator journeys and 45 repeated conflict, account and reset checks.
  Chromium qualified WebGL, and all three engines exercised forced Canvas.
  Firefox and WebKit intentionally skip six WebGL checks in total.
  The first web job was canceled during a slow Ubuntu mirror download. The
  retry passed at the same source with the successful Rust result retained.
  [Exact-source CI](https://github.com/byebyebryan/gomoku2d/actions/runs/44413002128)
- The lockfile updates `re2js` from `2.8.6` to `2.10.0`. Production npm and Cargo
  audits report zero advisories. Review of the full npm audit found 15 moderate
  entries confined to development dependencies.
  [re2js advisory](https://github.com/advisories/GHSA-gg6h-894w-4m9c)
- Chrome for Testing `156.0.8078.4` ran with mobile and touch emulation at
  DPR 3. Three scenarios covered saved replay details, publication edits with a
  disposable emulator account, and direct entry to the Dev site review. At
  `390 × 844`, `320 × 900`, and `844 × 390` CSS pixels, relevant controls stayed
  reachable and the measured document, body, main and editor widths had no
  horizontal overflow. The checks also covered draft discard, save and focus
  restoration. Bryan selected this as the release mobile layout gate.

## Deployment and production checks

The release, Pages and tagged rules workflows all passed at `48822e2`. Pages
deployment `7899082893` serves artifact `14480862217`. All 596 files
(`23,050,409` bytes of payload) and 10 direct-entry URLs matched the artifact
over the public domain. [Release workflow](https://github.com/byebyebryan/gomoku2d/actions/runs/44419634582) ·
[Pages workflow](https://github.com/byebyebryan/gomoku2d/actions/runs/44419634328) ·
[Served manifest](v0_6_4_release_manifest.json)

Eighteen public-origin browser checks passed across Chromium, Firefox and
WebKit. They covered local bot play, a direct shared replay, private details
excluded from exported links, Play From Here attribution and the Dev site
design-review index and cases. Fresh browser contexts used local fixtures.
Each engine showed `v0.6.4` on Home and decoded all seven images on direct
entry to the Replay collections review. The review made zero external requests
and raised zero page errors in those visits. No production account or catalog
writes were performed.

## Firestore and recovery

The release adds schema 2 for private replay libraries at the existing owner
path. Firestore accepts v1-to-v2 and v2-to-v2 writes. A stale merge leaves the
device copy available for the next review. Browser-generated thumbnails add no
image uploads, Firestore reads or hosted service.

The exact-source rules went live before the tag was pushed. Both workflows
compared the stored rules bytes with the release source before activation and
verified the active pointer after activation. Both deployments used rules bytes
with SHA-256 `430c9c56dd1bc63e26d206b840b513933cf9331864cafd1ebe16928d6abde95c`.
No index changes were deployed.

| Action | Live ruleset ID in `gomoku2d` |
|---|---|
| Prior release | `4391d371-1145-44b1-8d42-58d555ad2ee3` |
| [Pre-tag activation](https://github.com/byebyebryan/gomoku2d/actions/runs/44419547738) | `f00b05a4-2cca-4985-9e05-67868dba4b36` |
| [Tag activation](https://github.com/byebyebryan/gomoku2d/actions/runs/44419634352) | `f3668f76-a550-4e53-9fe2-5ac1bffcd2f0` |

Keep the additive rules during a frontend rollback. Once an account has a v2
library, favor a compatible forward fix or a v2-compatible deployment so the
account can still open and clear it. Preserve the release tag. [Release runbook](../reference/ops/release.md)

## Remaining acceptance

The real signed-in profile, statistics and publication round trip remains
pending. Physical touch, the OS keyboard, lower-powered device behavior, human
screen-reader and zoom checks, two-device private continuity, and novice
feedback retain their recorded deferrals. Curated learning remains unnumbered
and multiplayer stays deferred. [Roadmap](../reference/product/roadmap.md)

Luna ran the isolated Chrome mobile lane. The primary handled the dependency
fix, release preparation, publication, live verification and closeout.
[Evidence receipt](v0_6_4_release_evidence.json)
