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


## Batch 3: Frozen Wasm Workloads

The expanded opt-in profiler validates its manifest against the current Wasm
core before collecting timings and checks that Vite serves that identical Wasm
binary. Twelve ongoing positions span 4-80 plies (six per ruleset). Six decisive
replays span 18, 20, 28, 31, 50, and 51 plies. Every prefix is legal/ongoing and
every completed replay has the stated terminal result under its selected rules.
The preserved published report is read only; no tournament/report regeneration
or Renju-semantic change is part of this pass.

The manifest SHA-256 is
`fc7cf9ac62c3ec79701b8dce14ea739d25b3666b6ec845cb7e2b3a9178d29514`;
source SHA-256 is
`eb58c6e54ff02930289214802f38ae9560d8d641074d77dcb39350304a8d4b43`.
The harness separates 108 fresh-worker bot requests, nine reused-worker groups
(first request plus three warm repeats each), 18 uncached analyses and their
cached revisits. Five further independent trials are automatically recorded for
Normal/Hard or short-analysis trigger breaches. Startup uses five fresh contexts
and warm reloads, plus a separately labeled page-only rate-6 diagnostic. Pending
replacement, active-route cancellation, recovery and 30 navigation cycles use
the production application/worker paths. A 28-minute test limit leaves cleanup
and evidence writing inside the 30-minute global cap. Timing results follow.

Additional cloud runner checks passed: occupied-port refusal preserved the
existing listener; interrupting the runner after preview startup released all
six owned ports. Successful/failing test runs also cleaned up before reuse.
