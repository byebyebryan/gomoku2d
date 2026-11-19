# Gomoku2D v0.6.5 release record

Published November 19, 2026. This release closes the 0.6 line. My games shows
the final board for replayable games, and eligible players can start a
publishing draft from the game row. The draft preserves the chosen opening
when returning from inspection. Returning to My games brings the selected
game back into view and restores focus. [Play](https://gomoku2d.byebyebryan.com/) ·
[GitHub release](https://github.com/byebyebryan/gomoku2d/releases/tag/v0.6.5)

The Dev site shows the before and after of My games, the draft, and the
published listing in nine frozen captures. The capture record distinguishes
the shipped 0.6.4 baseline from the emulator-configured 0.6.5 candidate.
[My games and publishing review](https://gomoku2d.byebyebryan.com/dev/reviews/my-games-publishing-065/) ·
[Capture record](../working/v0_6_5_ui_evidence/README.md)

## Source and verification

- Annotated tag `v0.6.5` is object `9c82e3f27844008957b2a32433274ac3e5328cf4`
  and peels to `25d67c1360566d7ada265962d20cf52d93d9ab32`.
- CI passed on the tagged commit with 348 Rust tests, 586 web unit tests,
  44 tool checks, 51 Firestore rules checks, 306 browser passes, 72 Auth and
  Firestore emulator journeys, and 45 repeated conflict, account and reset
  checks. Six Chromium-only WebGL cases were skipped on Firefox and WebKit.
  [CI run](https://github.com/byebyebryan/gomoku2d/actions/runs/44613408212)
- Production npm and Cargo audits found zero advisories.
- Chrome for Testing `156.0.8078.4` ran two mobile and touch scenarios at DPR 3
  using disposable emulator accounts. The checks covered My games, publishing,
  focus and the return path at `390 × 844` and `844 × 390` CSS pixels at 100% and
  200% text, and `320 × 900` at 200%. Controls stayed reachable and clear of
  horizontal overflow and overlap. The initial fixtures assumed a sign-in
  popup and stacked actions. The adapted tests used the mobile redirect and
  measured reachability and bounds directly. Inspect and Publish can stay
  side by side at 200% where both fit. Profile tab labels wrap at enlarged text.

## Deployment and production checks

The GitHub Release, Pages and Firestore workflows passed at `25d67c1`.
Pages deployment `7926111838` serves artifact `14575651374`. All 607
files (23,344,361 payload bytes) matched over the public domain. The manifest
identifies 13 direct-entry routes, including the new UI design review.
[Release workflow](https://github.com/byebyebryan/gomoku2d/actions/runs/44617861197) ·
[Pages workflow](https://github.com/byebyebryan/gomoku2d/actions/runs/44617861117) ·
[Served manifest](v0_6_5_release_manifest.json)

Twenty-one public-origin browser checks passed across Chromium, Firefox
and WebKit. They covered local bot play, guest history, direct shared replay
entry, private details and portable export, Play From Here attribution, and
the Dev site design reviews. All three engines showed `v0.6.5` on Home and
decoded all nine captures in the new review. Verification used fresh guest
sessions and local fixtures.

## Firestore and recovery

The tagged workflow stored and activated the rules, verified their bytes,
and read back the live pointer. A separate readback confirmed the same rules
and all nine single-field index overrides. The rules bytes and index settings
match 0.6.4. This release required no separate migration or index deployment.
[Rules workflow](https://github.com/byebyebryan/gomoku2d/actions/runs/44617861100)

| Action | Ruleset ID in `gomoku2d` |
|---|---|
| Before tagging | `f3668f76-a550-4e53-9fe2-5ac1bffcd2f0` |
| Verified after publication | `15061f70-939c-41bd-8ef7-54c58aa64bd1` |

Rules SHA-256 is `430c9c56dd1bc63e26d206b840b513933cf9331864cafd1ebe16928d6abde95c`.
The prior web recovery point is `v0.6.4` at `48822e2`, which supports replay
library schema 2. Keep the compatible rules in place for web recovery and
preserve release tags. The release runbook records the steps for deploying
the prior web build and restoring a rules pointer if needed.
[0.6.4 release record](v0_6_4_release.md) · [Release runbook](../reference/ops/release.md)

## Remaining acceptance

The real Google sign-in and live profile, statistics, publish, guest viewing,
and cleanup round trip remains pending. Physical touch, the OS keyboard,
lower-powered devices, human screen-reader and native zoom checks, two-device
private continuity, and novice feedback retain their existing deferrals.
Curated learning remains unnumbered and multiplayer stays deferred.
[Roadmap](../reference/product/roadmap.md)

The Luna Max assignment covered the isolated Chrome mobile lane. The primary
handled integration, CI, publication and live verification.
[Evidence receipt](v0_6_5_release_evidence.json)
