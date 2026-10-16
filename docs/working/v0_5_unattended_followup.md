# v0.5 Unattended Cloud and Wasm Follow-up

Status: active; implementation authorized through a goal loop. Owner: Bryan.
Planning baseline: `30430a4d74a0f05e27e50a239663bc3e8e97ab67`.

This is a bounded extension after the completed
[October reconciliation](../archive/reconciliation_2026_10/README.md). It adds
automated evidence for two remaining questions: does the signed-in application
preserve its contracts across clients, and how does production Wasm behave on
a broader workload set? Findings drive small repairs within those contracts.

## Scope and Evidence

- Run unattended with disposable local accounts, Auth/Firestore emulators,
  Playwright, and the existing production profiling lane.
- Use the Chrome extension for agent-operated inspection of reproduced UI or
  recovery states when useful. Repeatable acceptance belongs in the test suite.
- Preserve the selected UI/copy, existing cloud checkpoint model, persisted
  schemas, bot presets, and Renju rules unless a demonstrated defect requires
  a scoped repair. Renju validation remains closed; this is no new solver or
  strength-evaluation project.
- Real Google OAuth, physical devices, actual zoom/screen-reader use, and human
  feedback retain their existing deferrals. Emulator accounts and independent
  browser contexts establish application integration, not real-device acceptance.
- Release preparation, versioning, tags, deployment, and production-account
  writes are separate from this follow-up.

## Execution Order

| Stage | Work | Exit evidence | State |
|---|---|---|---|
| 1 | Build the isolated cloud harness and one signed-in round trip | Application signs in through emulated Auth, writes through the real SDK/rules, and restores state in a second browser context | in progress |
| 2 | Complete the cloud continuity and failure matrix | Named assertions, emulator document readback, and repeatable recovery/isolation results | planned |
| 3 | Expand and run production Wasm workloads | Reproducible cold/warm distributions, slow-case records, cancellation and cleanup evidence | planned |
| 4 | Repair demonstrated failures, integrate checks, and close the extension | Affected regressions pass, appropriate full gates pass, and every finding has a disposition | planned |

Run cloud and profiling stages sequentially. Use isolated output directories,
ports, and reports; preserve the existing untracked visual-review artifacts and
any unrelated listeners. Prefer a separate checkout/worktree for execution.

## A. Cloud Application Integration

### Harness

Add an explicit emulator mode to `gomoku-web/src/cloud/firebase.ts`, a separate
cloud Playwright configuration/test directory, and an owned runner. Proposed
files are `playwright.cloud.config.ts`, `cloudtests/`, a dedicated Firebase
acceptance config, and `scripts/run_cloud_acceptance.mjs`. Include the new tools
in `tsconfig.tools.json`. These files/commands do not exist yet.

The runner will:

1. Use a `demo-` project ID, loopback Auth/Firestore endpoints, fake web config,
   the repository's actual `firestore.rules`, and a distinct production build.
   Require complete emulator configuration; partial or inconsistent test config
   fails before connecting. Ordinary production and unconfigured guest builds
   keep their existing initialization paths.
2. Start, health-check, and stop its own emulators/preview. Refuse occupied ports
   instead of adopting or terminating unrelated processes. Bound all waits and
   clean up after failure. Verify the emulator build cannot contact production
   Firebase/Google identity services.
3. Use fresh browser contexts with independent cookies/storage. Operate the
   app's Sign in/Sign out, Settings, game, Profile, and reset flows. Automate the
   emulator's provider UI; include popup and supported redirect paths.
4. Seed disposable accounts/cloud documents in test setup and read the emulator
   documents back after actions. Assertions exercise real client SDK requests
   and rules; fixture administration must not perform the tested user writes.
5. Start with one cloud worker and unique account IDs per case. Keep fixture
   reset/cleanup isolated from other cases and from existing rules tests.

Firestore uses server time for the five-minute write/reset cooldowns. Seed
eligible starting timestamps where appropriate; separately verify immediate
cooldown refusal. Browser clock changes must not stand in for server-time
acceptance, and the harness must not weaken rules or sleep through cooldowns.
If a retry requires eligible server state, use a documented emulator-only fixture
transition, then perform the actual retry through the application.

### Scenario Matrix

| ID | Scenario | Required evidence |
|---|---|---|
| C1 | New account signs in with guest settings and finished history | Profile initialization and eligible history promotion agree in UI, local state, and emulator document; no duplicate records |
| C2 | Existing account has different cloud setup and history | Account setup restores before upload; eligible guest history merges under existing ownership/trust rules |
| C3 | One client completes a game; another signs in or reloads | Finish follows the real game/save path; the second context can inspect the synced replay after explicit refresh |
| C4 | Two clients hold different snapshots and one writes first | Stale replacement is rejected; the newer cloud state survives and valid pending local games remain available for retry |
| C5 | Firestore transport fails during sync, then recovers | Local play/history remain usable; sync is pending/error as appropriate and a later eligible retry saves once |
| C6 | Account A signs out or switches to B while A's request is delayed | A's completion cannot patch B's profile/cache or upload to B; document readback and visible account state agree |
| C7 | Profile reset occurs while another client has old or pending history | Reset barrier survives stale writes/reconnect; records covered by reset are not resurrected |
| C8 | Reload after a successful sync and after a failed sync | Auth/setup/history restore consistently; valid pending work survives according to the existing cache contract |
| C9 | A write/reset is attempted inside its cooldown | The real rules refuse the operation; the app retains usable local state and truthful pending/error state |

These cases test explicit checkpoint/refresh behavior. They do not require live
collaborative merge or immediate updates in another open client.

For delayed-request cases, use controlled transport barriers that release on
observed events. Start with Playwright routing; use a small owned local transport
proxy only if the Firestore protocol requires it. Do not manufacture the race by
mutating stores or replacing the guards being tested. Existing unit/rules cases
remain the cheaper coverage for permutations that add no integration evidence.

### Acceptance and CI

- First establish C1/C2/C3 in Chromium, then the full matrix. Repeat the focused
  conflict/account/reset cases five times with fresh state and no blanket retries.
- Run the cloud matrix in Chromium, Firefox, and WebKit on the supported CI
  image. Record any engine-specific harness limitation as incomplete coverage;
  it cannot silently count as a pass. Local missing browser libraries can use
  that same supported CI lane.
- Retain traces/screenshots, emulator logs, scenario IDs, and sanitized document
  readback on failure. Keep normal app screenshots unchanged unless repairing
  a reproduced failure requires different behavior.
- Add a proposed `npm run test:cloud` entry point and a separate CI step/build
  that cannot replace the ordinary production artifact or leak emulator flags
  into deployment. Keep the existing unit/rules/three-engine lanes intact.

## B. Production Wasm Workloads

Extend `gomoku-web/perftests/browser_profile.spec.ts` and its helpers rather than
creating a second measurement system. Use the production worker and analyzer
paths with the current presets and model settings.

### Workload and Measurement Matrix

| Workload | Initial bounded sample | Measurements/assertions |
|---|---|---|
| Startup | Five fresh browser contexts and warm reloads | Home readiness, usable board readiness, transferred assets, worker startup |
| Bot turns | Twelve distinct legal positions, six per ruleset, spanning opening/midgame/dense forcing positions; all three presets; three fresh-worker trials per case | Request latency, result/error/timeout, legal returned move, main-thread heartbeat; retain case IDs and sample counts |
| Warm bot repeats | Three representative positions with a reused worker | Record warm/TT reuse separately from independent-position results |
| Replay analysis | Six legal decisive replays across both rulesets, including short and longer forcing endings; three uncached trials and a cached revisit | First useful annotation, completion, outcome status, cache behavior, timeout/error |
| Cancellation/replacement | Cancel active analysis, leave its route, replace pending work, and run subsequent analysis | Old worker ends; stale results do not replace the successor; later work completes |
| Repeated use | At least the existing 30 match/replay/navigation cycles | Workers/canvases return to the expected idle baseline; page heap checkpoints retain their stated measurement scope |

Select fixtures deterministically and validate their legality/completion with
the current core before measurement. Derive legal prefixes from preserved games
or small generated legal sequences; do not treat an arbitrary filled board as
a valid dense workload. Commit the small workload manifest, not regenerated
tournament results. Freeze it before comparing a performance repair.

Keep fresh-worker/new-position, warm-worker, cached-analysis, and cold-start
samples separate. Publish per-case median/max and pooled preset distributions
with sample counts; a pooled p95 over a small fixed corpus is diagnostic evidence,
not a device population or service guarantee.

Use the existing 15-second bot request cap and 60-second analysis cap as watchdogs,
plus a 30-minute overall profiling limit. Record timed-out/interrupted cases and
incomplete coverage explicitly; do not omit them from denominators or call a
partial run a pass. Shrink or stage the initial matrix only with the changed
coverage and reason recorded.

### Investigation and Repair Rules

- Carry forward the earlier diagnostic triggers: 2 seconds to local Home/board,
  1 second for Normal, 3 seconds for Hard, and 2/5 seconds to useful/complete
  short-ending analysis on the documented desktop setup. They trigger diagnosis,
  not default CI timing failures or physical-device promises.
- For a breach, repeat the same case in five independent workers/contexts and
  distinguish a reproducible workload cost from host noise. Retain all samples.
  Classify longer analysis separately before assigning it a usability expectation.
- Measure the real worker requests. Page-only CDP CPU throttling remains labeled
  as page-only; it cannot establish slow-device Wasm performance. Record any
  process-wide CPU constraint only if its effect on worker processes is verified.
- Measure usable cancellation/recovery and main-thread interaction while costly
  work runs. Page heap excludes worker/Wasm/GPU memory; retain that limit.
- Repair demonstrated leaks, stale results, hangs, or avoidable costs in small
  batches. A preset/model tradeoff needs its own correctness and product
  justification; do not turn this pass into open-ended bot-strength tuning.
- Keep timings opt-in. Promote deterministic worker lifecycle, legality, and
  recovery regressions to ordinary tests where they protect a concrete contract.

## Findings, Artifacts, and Final Validation

For each finding record: scenario/workload ID, source/build identity, trigger,
expected and actual behavior, impact, reproduction, disposition, and remaining
uncertainty. Use `fix`, `retain`, or `defer`; a deferred item needs an explicit
effect on the closeout decision.

Keep scratch traces, replay batches, and full JSON under ignored
`gomoku-bot-lab/outputs/`. Retain a compact reviewed evidence summary and selected
JSON/captures beside this plan when execution produces them. Record source SHA,
tracked/untracked dirty state, bundle hash, browser/engine versions, host details,
workload/fixture IDs, sample counts, and exact commands. Never relabel older
profiling results as measurements of a later repair.

Validation follows the touched boundary: Firebase bootstrap tests, web/tool
typecheck, unit tests, real rules emulator, ordinary production build/no-config
guest journeys, new cloud acceptance, and affected three-engine journeys. Rust,
Wasm, or search changes also require their relevant native/core/corpus checks.
Verify CI against the exact pushed implementation commit.

Close this extension when:

1. The cloud matrix has unattended results and all reproduced correctness,
   identity, data-loss, or broken-recovery failures are repaired.
2. The fixed Wasm matrix has reproducible measurements and every slow, failed,
   or incomplete case has an explicit disposition with justified acceptance.
3. Repairs pass their focused regressions and relevant integrated gates; no
   unexplained failure or hidden skipped coverage remains.
4. The evidence distinguishes emulator/browser/desktop results from the existing
   real-account/device/human deferrals and updates the `0.5` readiness decision.

Then return to release packaging. No new general review, Renju research, UI
redesign, or feature work is required by this plan.

## Tool References

- [Firebase Authentication emulator](https://firebase.google.com/docs/emulator-suite/connect_auth): emulated provider UI and unattended account flows.
- [Playwright browser-context isolation](https://playwright.dev/docs/browser-contexts): independent cookies/storage for client A/B.
- [Playwright network control](https://playwright.dev/docs/network): repeatable failure and request timing controls.
