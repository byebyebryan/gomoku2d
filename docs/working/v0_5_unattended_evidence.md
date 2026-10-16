# v0.5 Unattended Follow-up Evidence

Owner: Bryan. This ledger records execution of the
[cloud and Wasm plan](v0_5_unattended_followup.md). The prior October review,
retained UI/Renju decisions, and manual acceptance deferrals remain in force.

## Batch 1: Isolated Cloud Harness

The demo-only runner owns Auth/Firestore emulators and a production-style
`dist-cloud` bundle/preview, leaving normal `dist` and deployed configuration
separate. It uses the actual repository rules. Invalid, partial, non-demo, or
remote emulator configuration fails rather than falling back to production.
The browser blocks live identity/database endpoints. Firebase's supported
popup/redirect transport still loads Google's static GAPI library; cosmetic
emulator-widget CDN dependencies are fulfilled locally. This is an unattended
emulated sign-in test, not real Google OAuth acceptance.

The first browser scenario passed in both Chromium and Firefox: a new account
received guest settings and history; an independent client restored the cloud
setup over its different local choice and merged a second game; reload retained
both replays and the restored setup. Server document readback confirms the two
distinct game IDs. An emulator-only timestamp fixture made the second write
eligible; user writes still crossed the real SDK transaction and unchanged rules.

Validation:

- Rebuilt Wasm in the isolated worktree.
- Web/tool/script typecheck passed.
- Firebase bootstrap/configuration suite: 12 passed.
- `npm run test:cloud -- --browser=chromium`: 1 passed (4.9 s).
- `npm run test:cloud -- --browser=firefox`: 1 passed (4.8 s).
- Added the cloud command after ordinary three-engine journeys in CI; it uses
  a separate bundle and cannot replace the production build artifact.

The initial runner exposed configuration-path and health-endpoint mistakes;
both were corrected before these passing runs. Network isolation also exposed
the SDK's GAPI static dependency, which is now explicitly distinguished from
identity/database traffic. No application defect was inferred from those
harness failures. Remaining cloud cases and Wasm profiling are still pending.
