# v0.7 Match and Replay UI evidence

Captured November 21, 2026 from the Guide example **Review a finished game**.
All four `match-*.png` files show move 27 just after branching. The `before`
images show the archived v0.6.5 app. The `after` images show the local candidate.

## Source and build receipt

- The archived Pages baseline uses source SHA `25d67c1360566d7ada265962d20cf52d93d9ab32`.
  All 607 files in `gomoku-web/test-results/v070-start/baseline` matched the
  [v0.6.5 manifest](../../archive/v0_6_5_release_manifest.json), totaling 23,344,361 bytes.
- The capture candidate uses HEAD `2345b661ae9c8e88bca014387d5c26948c08f276` and
  a 35-path diff with SHA-256 `e8756475b5d8c9c70b348e56d892aa39aeafd2dd06328036f43c789d48b29c2d`.
  The local validation receipts retain that diff in `capture-source.patch`.
  The snapshot also included the following six untracked files.

  | Path | SHA-256 |
  |---|---|
  | `gomoku-web/scripts/catalog_media.mjs` | `482cc9b0e6561386cdd106955ee4c23002b062bb99432487197c4f14f428b342` |
  | `gomoku-web/scripts/tests/catalog_media.test.mjs` | `512da39564e5227b707bbbd1cebda0aa579030cbe0a1c653993b61869835553e` |
  | `gomoku-web/src/replay/replay_library_merge.test.ts` | `1620342eda4466bf3e9ee285bb89258b669abe97ae2c93cf8cc5e61d44c9eb37` |
  | `gomoku-web/src/replay/replay_library_merge.ts` | `da4887653fcaa244f529f0f8c5b2609a452b58c0836ce795514b5fc042b011f5` |
  | `gomoku-web/src/routes/lab-report/ReportBoardShell.tsx` | `e62a99e9f5639906ef7e733d824dc0f1b0ebab99108648f9d9e9609d13a69f47` |
  | `scripts/install-ci-wasm-pack.sh` | `74d7b476cb1f0b5f45d8a926e80cd9ceb37454a59246a3ced641b0efb53e9343` |

- The production-style build used Rust 1.100.0, wasm-pack 0.15.1 and Node 24.22.3.
  The capture rebuilt the bundler release Wasm package, installed the web
  dependencies with `npm ci`, and ran `npm run build` and `npm run preview`.
  The SHA-256 of `gomoku_wasm_bg.wasm` is
  `25a7b89a57ec09aaf3c8e48cd79880793c3fde9d91892863b403f771a39f5216`.
- Playwright 1.64.1 drove headless Chromium 156.0.8078.4 (revision 1248).
  Desktop used 1440 × 900. Compact used 390 × 844 with touch enabled. Both used
  a device scale factor of 1.
- The captures qualify the local 0.7 candidate. Its package metadata reads 0.6.5.

## Observed navigation

Both builds opened the 28-move shared replay from `/guide/#examples`, moved to
27/28, then used **Play From Here**. In v0.6.5, browser Back from the move-27
Match returned to `/replay/shared/` at 28/28. In the candidate, **Source replay**
opened Replay Analysis at 27/28 and focused its heading. Browser Back restored
`/match/local` at move 27. These transitions passed at both viewports.

## Captured outputs

| Output | Dimensions / duration | SHA-256 |
|---|---|---|
| `match-desktop-before.png` | 1440 × 900 | `22ba03645a1ceca9393f56ef1f8768d85eb55e221ce20b4953190596f4c1510c` |
| `match-desktop-after.png` | 1440 × 900 | `dcbee4e485530450eada204da5b25682117a40228660bb34f70580017fbecb5b` |
| `match-compact-before.png` | 390 × 844 | `083c6b764118be64baa238b2c1ce5e9b01f5e701b93c439e2e6763e802b0ef28` |
| `match-compact-after.png` | 390 × 844 | `b00e586be03b116cd55d7592d996b783e92e94cf22cf5a7507563f265e4220c1` |
| `docs/assets/readme-gameplay.gif` | 960 × 600, 6 s, 5 fps | `c037af8d5677522a4cf0f916221b289e7d8f3d5a86356904ff368beb5c175848` |
| `docs/assets/readme-analysis.gif` | 960 × 600, 6 s, 5 fps | `d946db0f0ac76eaf196d0497ad7da70a73eb5bbca954b4aaeee84d395b0bff3f` |

Both GIFs use the same Guide fixture in a local guest preview. Gameplay shows
moves 28 and 27, the branch at 27, and the winning F11 continuation. Analysis
waits for the final **White has won** verdict and an annotated replay timeline,
then steps 28 → 27 → 26 → 25 → 26 → 27.

## Reproduction

Rebuild Wasm and the web app from the source you want to capture. Serve that
build on `127.0.0.1:8016` and the manifest-verified v0.6.5 baseline on
`127.0.0.1:8015`. To reproduce the original candidate capture, use the
source snapshot recorded above. The script uses Node 24, the Playwright 1.64.1
installation under `gomoku-web` with Chromium available, and FFmpeg.

Run the full capture from the root of a separate checkout. It writes both
README GIFs and all four cataloged PNG paths. For a README-only refresh, bring
back the intended GIFs and record their new inputs and hashes in
[media provenance](../../assets/README.md).

```sh
node docs/working/v0_7_ui_evidence/capture.mjs
```

The analysis-only mode uses just the candidate preview. It writes
`docs/assets/readme-analysis.gif` after waiting for a settled verdict and
annotated timeline.

```sh
node docs/working/v0_7_ui_evidence/capture.mjs --analysis-only
```
