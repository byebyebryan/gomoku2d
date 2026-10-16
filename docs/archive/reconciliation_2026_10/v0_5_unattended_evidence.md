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


## Measured Wasm Results

The retained [profile JSON](assets/browser-profile-2026-10-16.json) contains
individual samples, per-case median/max, pooled preset distributions, startup
transfers, cache/lifecycle results, manifest hashes and measurement limits.
`npm run profile:browser` passed in 2.4 minutes on clean source
`2c2b2d308dc19e51b75a1012aa6e6d3ce6e633e2` (no tracked or untracked changes).
Bundle SHA-256:
`4bf2bab72eb771535b728d1941c814ea16332856d296e3ac470308afba79eb33`.
Wasm SHA-256:
`82e6be1525d83cca9bcb82c99a5e5a3f7c7cf51ed0c292acd54c5930b0ab417a`.
Host: Ryzen 9 7900X, 24 logical CPUs, Linux 7.2.9-arch1-1, Node 26.11.1,
Chromium 156.0.8078.4. The unchanged build tools were checked after the run:
Rust 1.99.0, wasm-pack 0.14.0, Playwright 1.64.1, and Vite 8.3.4.
This is local production-preview evidence.

| Preset | Fresh-worker requests | Median | Diagnostic p95 | Max |
|---|---:|---:|---:|---:|
| Easy | 36 | 10 ms | 53 ms | 54 ms |
| Normal | 36 | 139 ms | 678 ms | 686 ms |
| Hard | 36 | 392 ms | 2,333 ms | 2,396 ms |

Across five fresh contexts, local Home readiness was at most 108 ms and usable
board readiness at most 840 ms; warm reload/board readiness was at most 879 ms.
The rate-6 page-only diagnostic is separately labeled in the JSON.

All 108 fresh-worker and 36 reused-worker requests returned legal moves, with
zero errors/timeouts. The 36 reused-worker requests include nine first requests
and 27 warm repeats; they are excluded from the fresh-worker distribution.
The largest sampled bot-period main-thread heartbeat gap was about 16 ms.
Normal/Hard and startup stayed below their diagnostic triggers, so no extra
five-trial breach investigation was needed. The corpus is finite and all bot
positions are Black to move; these distributions are not a population guarantee,
strength test, or new Renju validation claim.

All six decisive replays completed in all three fresh-context trials (18/18),
and all 18 cached revisits completed without an analysis worker. First useful
annotations appeared within 813 ms. Maximum completion was 3.61 seconds among
short endings (18-31 plies), 6.62 seconds for the 50-ply Renju replay, and 7.63
seconds for the 51-ply Freestyle replay. The longer endings are retained as
separate workload costs; useful progressive output and cancellation remain
available. Cached revisit timings include route/board readiness, not just lookup.

Both replacement and route cancellation waited for visible progress while the
long analysis was still running. The old worker closed in 609 ms, its successor
completed in 896 ms without stale replacement, and leaving active analysis
returned to zero workers/canvases in 883 ms. All 30 subsequent navigation cycles
returned to that idle baseline. Forced-GC page heap was 9.03 MB initially and
9.72/10.96/11.28/11.48 MB at cycles 5/10/20/30. These bounded samples do not prove
absence of long-run growth and exclude worker/Wasm/GPU allocations.

The initial clean `a33c67d` run also passed and remains separately under ignored
outputs. The retained run tightened cancellation to wait for actual progress;
no engine, preset, analyzer, or UI change occurred between the two measurements.

## Integrated Verification

The first full cloud CI run on `37fd703` passed native checks, 378 web unit tests,
27 rules tests, 168 ordinary browser journeys, and C1-C9 in all three engines.
Its sole failure was the WebKit redirect harness selecting Firebase's hidden
iframe as well as the test frame. The test now uses an isolated static embedding
host and its titled frame. Redirect passed locally in Chromium and Firefox after
that repair. Failure artifacts now also retain the latest disposable emulator
documents read by each test, alongside engine version, traces and screenshots.

[Exact-head CI](https://github.com/byebyebryan/gomoku2d/actions/runs/38957667087)
passed on implementation commit `078b67b245a38e87a04f477c3b183d9098156676`:
348 native tests plus formatting/lint, Wasm build, web/tool/script typecheck,
378 web unit tests, 27 real-rules tests, production build, 168 ordinary browser
journeys, 21 cloud journeys, and all 45 fresh-state conflict/account/reset
repetitions across Chromium, Firefox, and WebKit. No retries or skipped cloud
coverage were used. The supported CI image closes the local WebKit dependency
limitation; real-account and physical-device deferrals remain separate.


## Finding Dispositions

| Finding | Disposition | Evidence and closeout effect |
|---|---|---|
| No-op sync falsely closed upload eligibility | Fixed in `37fd703` | Unit regression failed before/passed after; C3 real finish subsequently syncs to another client |
| Embedded redirect selected the SDK helper iframe | Harness fixed in `078b67b` | Isolated host and titled frame; Chromium/Firefox local pass and full three-engine CI pass |
| Longer (50/51-ply) replay analysis costs 6-8 seconds | Retain | Progressive annotations within 813 ms, successful completion, active cancellation and successor recovery; no preset/analyzer tuning is justified by this bounded sample |
| Page JS heap is higher after repeated navigation | Retain measured limit | 9.03 to 11.48 MB over 30 cycles, idle workers/canvases every cycle; insufficient evidence to classify long-run or worker/Wasm/GPU memory behavior |
| Real OAuth, physical-device and human acceptance | Defer as already directed | Emulator integration and desktop automation narrow the engineering uncertainty; they do not claim those deferred passes or add manual chores to this extension |


## Closeout Decision

The unattended extension and the `0.5.x` engineering review are complete. The
cross-client application matrix, fixed Wasm workloads, active interruption and
recovery, and relevant integrated gates all have unattended passing evidence.
The one demonstrated product defect is repaired; remaining workload costs and
measurement limits have explicit dispositions. No additional general review,
Renju research, UI/copy redesign, or manual acceptance work is required to close
this pass.

Return to release packaging and the next product/design checkpoint. No version,
tag, deployment, production rules/account write, preset change, or analyzer
semantic change was made here. Carry the previously directed real OAuth,
physical-device/accessibility, and human-feedback deferrals into any release
decision. They remain unverified and do not keep this engineering review open.

The plan and this ledger are archived with the October review. Current commands
and ownership are synchronized in the testing guide and app/web references.
The original 87 untracked local UI-study artifacts remain outside the commit.
