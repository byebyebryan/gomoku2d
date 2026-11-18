# 0.6.4 replay collection UI captures

Captured November 18, 2026 from `ffec605ba22bc4adbd361e399c8a0f9edb880cea` on the local candidate.
The Dev site displays the allowlisted images at
`/dev/reviews/replay-collections-064/`. The images came from production-style
builds and were copied without image transformations. They show guest fixtures
or disposable Auth and Firestore emulator accounts.

Headless Chromium 156.0.8078.4 from Playwright, DPR 1, default text size.
The ordinary browser lane ran `playtests/replay_library.spec.ts` on Chromium,
Firefox and WebKit. The emulator lane ran C20, C21 and C22 on the same engines.
The portrait captures use resized desktop browsers.

| Capture | Scenario | CSS viewport | PNG dimensions | SHA-256 |
|---|---|---|---|---|
| [Saved replays · desktop](saved-library-desktop.png) | Guest save and remove | 1440 × 1100 | 1440 × 1100 | `7dfa5c2b275eeba014da31f38d0aba6694ab576a93f8fb7b19d031f7a406818d` |
| [Saved replays · portrait](saved-library-portrait.png) | Guest save and remove | 390 × 844 | 390 × 844 | `e6d7544344536215997516ed50b294d917b91fd3593981a7e3a01ef6c1d898c4` |
| [Private details · portrait](private-editor-portrait.png) | Private details round trip | 390 × 844, full page | 390 × 1135 | `e952bf0b0f85181f82f5747322f584321be683568882f87d2fd2e1e85a485265` |
| [Edit publication · desktop](public-editor-desktop.png) | C20 publication edit | 1280 × 900 | 1280 × 900 | `091bc197258268c663a7948817f05b9889c72075791286ab97bbea796f932a8c` |
| [Before edit · move 4](public-opening-before.png) | C20 visitor before edit | 390 × 844 | 390 × 844 | `fe877ad49f9e24b961c828bce636dbceb80e51b491278490dc066978954eef60` |
| [After edit · move 0](public-opening-after.png) | C20 visitor after edit | 390 × 844 | 390 × 844 | `0c64687c75e4516e21805f7162943a4a0d6b2e507d8f67077c3927e0648821df` |
| [Device and cloud choices](private-merge-review.png) | C21 two-client conflict | 1280 × 900 | 1280 × 900 | `692718780aa003ce8fb4cc132c714e875844163401f69377441137dbb78dbdf5` |
