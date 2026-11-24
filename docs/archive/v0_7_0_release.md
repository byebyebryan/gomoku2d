# Gomoku2D v0.7.0 release record

Published November 24, 2026. Replay continuations survive browser navigation,
completed rounds save once, and opening another replay panel cancels a pending
Play From Here launch. The release also improves compact play, private history
and reset recovery, and the Dev site, and refreshes the development toolchain.
[Play](https://gomoku2d.byebyebryan.com/) ·
[GitHub Release](https://github.com/byebyebryan/gomoku2d/releases/tag/v0.7.0) ·
[Changelog](../../CHANGELOG.md#070---2026-11-24)

## Source and CI

- Annotated tag `v0.7.0` is object `464712ac6089eaf5f14ea8f4fd6cb30a03f6588d`
  and resolves to `ecb44b2af9a90da3784b0e72692a37552e9b7a37`.
- CI passed on the tagged commit with 354 Rust tests, 610 web unit tests,
  50 tool checks, five release validator checks, 51 rules tests, 318 browser
  passes, 75 Auth and Firestore emulator journeys, and 45 repeated cloud race
  checks. Six Chromium-only WebGL cases were skipped on Firefox and WebKit.
  [CI run](https://github.com/byebyebryan/gomoku2d/actions/runs/45458127244)
- The production npm and Cargo audits found zero advisories. The regression
  test failed without the replay cancellation call and passed with it.
  [Replay regression](../../gomoku-web/playtests/replay.spec.ts)

## Deployment and public verification

The tagged GitHub Release and Firestore workflows succeeded. The tagged Pages
run reached artifact upload, then failed to find the just-uploaded artifact.
Its rerun found two artifacts with the same name, and a fresh tag dispatch
reproduced the first failure. Workflow commit
`aca32e1dd2168064ce898417a649a2b2598ac50f` gives each attempt a distinct artifact
name and waits for the exact upload to appear. The recovery dispatch ran that
workflow from `main` and verified the checkout of `v0.7.0` at `ecb44b2`
before building. The tag stayed on the original CI-verified commit.
[Release workflow](https://github.com/byebyebryan/gomoku2d/actions/runs/45463045191) ·
[Initial Pages run](https://github.com/byebyebryan/gomoku2d/actions/runs/45463045501) ·
[Successful Pages recovery](https://github.com/byebyebryan/gomoku2d/actions/runs/45464347342)

Pages deployment `8037659169` serves artifact `14953352541`. All 614 files
(23,415,949 payload bytes) and 44 direct route URLs matched the public domain.
Thirty browser checks passed in fresh guest contexts across Chromium, Firefox,
and WebKit. They covered the visible release version and enabled sign-in entry,
the replay cancellation regression, guest play and replay, compact finishes,
and the Dev, Lab, and Visuals surfaces. The Dev isolation checks used the
production origin in place of the test fixture's loopback allowlist.
[Served manifest](v0_7_0_release_manifest.json)

## Firestore and recovery

| Action | Ruleset ID in `gomoku2d` |
|---|---|
| Before tagging | `15061f70-939c-41bd-8ef7-54c58aa64bd1` |
| Verified after publication | `6ab99d4e-c7a0-429d-8d0a-e12983b52a68` |

Independent readback matched the tagged rules SHA-256
`430c9c56dd1bc63e26d206b840b513933cf9331864cafd1ebe16928d6abde95c`.
All nine configured index exemptions matched, and the before/after index
configurations matched. Google sign-in remains enabled for the public origin.
The compatible web recovery point is `v0.6.5` at `25d67c1`.
[Rules workflow](https://github.com/byebyebryan/gomoku2d/actions/runs/45463045141) ·
[Recovery runbook](../reference/ops/release.md#recovery)

## Remaining acceptance

An earlier live Chrome run on 0.6.5 covered Google return to an existing
account, publication and editing, shared statistics, signed-out readback, and
cleanup. The 0.7.0 public smoke used guest contexts. A production account
round trip on 0.7.0, first-time Google consent, new public-profile setup,
physical touch and software keyboard, lower-powered hardware, human
screen-reader use, two-device continuity, and novice feedback remain open or
deferred. [Refinement and acceptance record](../working/v0_7_refinement_plan.md)
