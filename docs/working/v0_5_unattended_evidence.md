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

## Batch 2: Cloud Continuity and a Confirmed Sync Repair

C1-C9 now run through the application, actual SDK, and unchanged emulator rules:
new/existing accounts, a real branched-game finish inspected in a second client,
stale-client rejection, transport failure/reload/recovery without duplication,
delayed account-A reads across an in-place switch to B, and the reset barrier
plus immediate cooldown refusal. A separate embedded-client case exercises the
supported redirect flow. Static GAPI telemetry and generic connectivity probes
are fulfilled locally; identity/database operations remain loopback-only.

**Fixed: a no-op flush advanced the cached write timestamp without a server
write.** C3 exposed the resulting false five-minute client cooldown: a newly
finished game remained queued despite an eligible server timestamp. The focused
unit regression failed before the change and passes after it. A no-op now keeps
the authoritative timestamp and does not claim a write. Actual write cooldowns,
identity checks, schemas, rules, and UI are unchanged.

Validation before the batch commit:

- Full web unit suite: 378 passed (39 files).
- Focused sync/promotion regressions: 23 passed; web/tools/scripts typecheck passed.
- Chromium: the full C1-C9 matrix passed, plus the initial touch-policy redirect.
- Firefox: the full C1-C9 matrix and embedded-policy redirect passed.
- C4/C6/C7: five fresh-state repetitions each in Chromium and Firefox, 30 passed.
- The same local all-engine repeat invocation recorded 15 WebKit launch failures
  because this host lacks libicu74/libxml2/libflite1. These are incomplete local
  coverage, not application passes; supported Ubuntu CI supplies those libraries.
- CI now runs the full three-engine matrix and five repetitions of the conflict,
  account-switch, and reset cases. Exact pushed-head results are recorded below.

Harness findings were distinguished from the product defect: Firefox touch
emulation does not supply the coarse-pointer/mobile-data policy inputs, so the
redirect case uses the supported embedded-client path instead. An optional GAPI
telemetry request needs a local empty response. Neither changes application
authentication behavior or claims physical mobile acceptance.
